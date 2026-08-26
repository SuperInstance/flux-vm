//! vm/flux_vm.rs runner — the standalone Rust VM under `vm/` (wired into the
//! workspace for the first time by this crate; see CONFORMANCE.md).
//!
//! Contract:
//! - u8 stack, PUSH (0x00) with 1-byte unsigned immediate;
//! - BITMASK_RANGE (0x1D) pops v, pushes 1 if lo <= v <= hi (u8 bounds);
//! - ASSERT (0x1B) pops; zero → Fault::AssertFailed;
//! - gas is charged 1/step but is NOT observable through the public API;
//! - execute(bytes, max_steps) returns Ok(()) when HALT is reached *and also
//!   when the step budget is silently exhausted* (see CONFORMANCE.md).

use crate::corpus::Case;
use crate::ir::{FaultKind, Op, Outcome, Triple};
use crate::runners::{hex, skip};
use crate::vmrs_src::{Fault, FluxVM};

fn u8imm(v: i64, what: &str) -> Result<u8, String> {
    if (0..=255).contains(&v) {
        Ok(v as u8)
    } else {
        Err(format!("{what} {v} outside u8 domain"))
    }
}

fn compile(ops: &[Op], input: i64) -> Result<Vec<u8>, String> {
    let mut bc: Vec<u8> = Vec::new();
    let mut asserts = 0usize;
    for &op in ops {
        match op {
            Op::Input => {
                bc.push(0x00);
                bc.push(u8imm(input, "input")?);
            }
            Op::PushI(v) => {
                bc.push(0x00);
                bc.push(u8imm(v, "PUSH")?);
            }
            Op::Add => bc.push(0x06),
            Op::Sub => bc.push(0x07),
            Op::Mul => bc.push(0x08),
            Op::Eq => bc.push(0x0F),
            Op::Ne => bc.push(0x10),
            Op::Lt => bc.push(0x11),
            Op::Gt => bc.push(0x12),
            Op::Le => bc.push(0x13),
            Op::Ge => bc.push(0x14),
            Op::And => bc.push(0x09), // bitwise on u8 — equals logical on 0/1
            Op::Or => bc.push(0x0A),
            Op::Not => bc.push(0x0C), // BITWISE ! — see CONFORMANCE.md
            Op::RangeI { lo, hi } => {
                let lo = u8imm(lo, "RANGE lo")?;
                let hi = u8imm(hi, "RANGE hi")?;
                bc.extend_from_slice(&[0x1D, lo, hi]);
            }
            Op::Assert => {
                bc.push(0x1B);
                asserts += 1;
            }
            Op::Nop => bc.push(0x27),
            Op::Halt => bc.push(0x1A),
            Op::Pop => bc.push(0x01),
            Op::DivI => return Err("vm/flux_vm.rs has no DIV".into()),
        }
    }
    let _ = asserts;
    Ok(bc)
}

pub fn run(case: &Case) -> Triple {
    let bc = match compile(&case.ops, case.input) {
        Ok(x) => x,
        Err(e) => return skip(&e),
    };
    let gas = case.gas_override.unwrap_or(4096);
    let mut vm = FluxVM::new(gas);
    let res = vm.execute(&bc, 100_000);
    let outcome = match res {
        Ok(()) => Outcome::Pass, // corpus programs end in ASSERT; HALT
        Err(faults) => match faults.first() {
            Some(Fault::AssertFailed) => Outcome::Fail,
            Some(Fault::GasExhausted) => Outcome::Fault(FaultKind::GasExhausted),
            Some(Fault::StackUnderflow) => Outcome::Fault(FaultKind::StackUnderflow),
            Some(Fault::StackOverflow) => Outcome::Fault(FaultKind::StackOverflow),
            Some(_) => Outcome::Fault(FaultKind::Other),
            None => Outcome::Fault(FaultKind::Other),
        },
    };
    Triple {
        outcome,
        gas_used: None, // metered internally; no public accessor
        encoded: hex(&bc),
        note: "gas metered but not observable via public API".into(),
    }
}
