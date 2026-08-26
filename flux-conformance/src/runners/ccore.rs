//! C core runner — `src/flux_runtime_arm.c` via FFI.
//!
//! Contract (from flux_runtime_arm.h/.c):
//! - `flux_check(bytecode, len, input, max_gas, &gas_used)`;
//! - the input is pushed onto the stack BEFORE execution starts;
//! - HALT pops the top of stack: non-zero → FLUX_PASS, zero → FLUX_FAULT;
//! - ASSERT pops, faults on zero, pushes 1 back on success;
//! - RANGE reads two little-endian i16 immediates (lo, hi);
//! - gas: 1 per op, RANGE=2, CHECK_DOMAIN=3.
//!
//! Quirk: read_i16/read_i32 bounds-check `offset + width >= bc_len`, which
//! requires one byte of lookahead past the final operand byte. The encoder
//! therefore appends a trailing NOP byte so RANGE-at-end programs decode.

use crate::corpus::Case;
use crate::ir::{FaultKind, Op, Outcome, Triple};
use crate::runners::{hex, skip};

extern "C" {
    fn flux_check(
        bytecode: *const u8,
        bc_len: u16,
        input: i32,
        max_gas: u16,
        gas_used: *mut u16,
    ) -> i32;
}

fn compile(ops: &[Op], input: i64) -> Result<(Vec<u8>, String), String> {
    let mut bc: Vec<u8> = Vec::new();
    // The C core pre-pushes the input BEFORE the first instruction, so the
    // initial stack is [input]. The first IR op can virtually consume it
    // ONLY if it is the first op of the program; any later `Input` must
    // emit LOAD_INPUT (the stray bottom value is never touched by our
    // programs — no SWAP/POP in the corpus reaches it).
    let first_op_is_input = ops.first() == Some(&Op::Input);
    for (idx, &op) in ops.iter().enumerate() {
        match op {
            Op::Input => {
                if idx == 0 && first_op_is_input {
                    // initial stack already holds it
                } else {
                    bc.push(0x10); // LOAD_INPUT
                }
            }
            Op::PushI(v) => {
                if (-128..=127).contains(&v) {
                    bc.extend_from_slice(&[0x01, v as i8 as u8]);
                } else if (-32768..=32767).contains(&v) {
                    bc.push(0x02);
                    bc.extend_from_slice(&(v as i16).to_le_bytes());
                } else if (-2147483648..=2147483647).contains(&v) {
                    bc.push(0x03);
                    bc.extend_from_slice(&(v as i32).to_le_bytes());
                } else {
                    return Err(format!("PUSH {v} outside i32 immediate domain"));
                }
            }
            Op::Add => bc.push(0x20),
            Op::Sub => bc.push(0x21),
            Op::Mul => bc.push(0x22),
            Op::Eq => bc.push(0x23),
            Op::Lt => bc.push(0x24),
            Op::Gt => bc.push(0x25),
            Op::Le => bc.extend_from_slice(&[0x25, 0x2A]), // GT; NOT
            Op::Ge => bc.extend_from_slice(&[0x24, 0x2A]), // LT; NOT
            Op::Ne => bc.extend_from_slice(&[0x23, 0x2A]), // EQ; NOT
            Op::And => bc.push(0x26),
            Op::Or => bc.push(0x27),
            Op::Not => bc.push(0x2A),
            Op::RangeI { lo, hi } => {
                if !(-32768..=32767).contains(&lo) || !(-32768..=32767).contains(&hi) {
                    return Err(format!("RANGE [{lo},{hi}] outside i16 immediate domain"));
                }
                bc.push(0x1D);
                bc.extend_from_slice(&(lo as i16).to_le_bytes());
                bc.extend_from_slice(&(hi as i16).to_le_bytes());
            }
            Op::Assert => bc.push(0x1B),
            Op::Nop => bc.push(0x00),
            Op::Halt => bc.push(0x1A),
            Op::Pop => return Err("C core has no POP".into()),
            Op::DivI => return Err("C core has no DIV".into()),
        }
    }
    // Lookahead quirk padding (see module docs) — never executed after HALT.
    bc.push(0x00);
    if !(-2147483648..=2147483647).contains(&input) {
        return Err(format!("input {input} outside i32 domain"));
    }
    Ok((bc, String::new()))
}

pub fn run(case: &Case) -> Triple {
    let (bc, _) = match compile(&case.ops, case.input) {
        Ok(x) => x,
        Err(e) => return skip(&e),
    };
    let max_gas = case.gas_override.unwrap_or(4096).min(65535) as u16;
    let mut gas_used: u16 = 0;
    let rc = unsafe {
        flux_check(
            bc.as_ptr(),
            bc.len() as u16,
            case.input as i32,
            max_gas,
            &mut gas_used,
        )
    };
    let outcome = match rc {
        0 => Outcome::Pass,
        1 => Outcome::Fail, // FLUX_FAULT: assert failed (or invalid opcode)
        2 => Outcome::Fault(FaultKind::GasExhausted),
        3 => Outcome::Fault(FaultKind::StackOverflow),
        4 => Outcome::Fault(FaultKind::StackUnderflow),
        5 => Outcome::Fault(FaultKind::BadBytecode),
        _ => Outcome::Fault(FaultKind::Other),
    };
    if let Outcome::Fault(FaultKind::Other) = outcome {
        return Triple {
            outcome,
            gas_used: Some(gas_used as u64),
            encoded: hex(&bc),
            note: format!("unexpected rc={rc}"),
        };
    }
    Triple {
        outcome,
        gas_used: Some(gas_used as u64),
        encoded: hex(&bc),
        note: String::new(),
    }
}
