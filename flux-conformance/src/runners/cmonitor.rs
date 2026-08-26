//! C monitor runner — `src/flux_monitor_arm.c` via the build-time shim.
//!
//! Contract and gotchas (all documented in CONFORMANCE.md):
//! - tagged-union stack (i32/f32/bool), no gas metering;
//! - PUSH_I32 (0x01) reads a 4-byte LE immediate;
//! - CHECK_RANGE_I32 (0x10) pops max, min, value (push order: val, min, max);
//! - flux_monitor_run() returns the boolean at the BOTTOM of the stack
//!   (stack[0]) — for single-result programs that is the check result;
//! - there is no ASSERT opcode: an assert over a boolean value is deferred
//!   to the run() result; an assert over a raw value is UNREPRESENTABLE
//!   (the runner skips rather than record a vacuous pass);
//! - ADD_I32/SUB_I32/MUL_I32/CMP_EQ/CMP_LT/CMP_GE/CHECK_DOMAIN are #defined
//!   but have no dispatch cases — they fault with INVALID_OPCODE at
//!   runtime, so the runner skips them (documented finding, not hidden).

use crate::corpus::Case;
use crate::ir::{FaultKind, Op, Outcome, Triple};
use crate::runners::{hex, skip};
use std::ffi::c_void;

extern "C" {
    fn conf_monitor_new() -> *mut c_void;
    fn conf_monitor_load(m: *mut c_void, bc: *const u8, len: u16) -> bool;
    fn conf_monitor_run(m: *mut c_void) -> bool;
    fn conf_monitor_error(m: *mut c_void) -> u8;
    fn conf_monitor_free(m: *mut c_void);
}

fn push_i32(v: i64) -> Result<Vec<u8>, String> {
    if !(-2147483648..=2147483647).contains(&v) {
        return Err(format!("PUSH {v} outside i32 immediate domain"));
    }
    let mut b = vec![0x01];
    b.extend_from_slice(&(v as i32).to_le_bytes());
    Ok(b)
}

fn compile(ops: &[Op], input: i64) -> Result<Vec<u8>, String> {
    let mut bc: Vec<u8> = Vec::new();
    // Type-state: is the top of the stack a monitor bool? Only RangeI /
    // And / Or / Not produce bools; PUSH_I32 produces i32.
    let mut top_is_bool = false;
    for &op in ops {
        match op {
            Op::Input => {
                bc.extend_from_slice(&push_i32(input)?);
                top_is_bool = false;
            }
            Op::PushI(v) => {
                bc.extend_from_slice(&push_i32(v)?);
                top_is_bool = false;
            }
            Op::Add | Op::Sub | Op::Mul | Op::Eq | Op::Lt | Op::Ge | Op::Le | Op::Gt | Op::Ne => {
                return Err(
                    "monitor defines ADD/SUB/MUL/CMP_* macros but never dispatches them \
                     (runtime INVALID_OPCODE)"
                        .into(),
                );
            }
            Op::And => {
                bc.push(0x20);
                top_is_bool = true;
            }
            Op::Or => {
                bc.push(0x21);
                top_is_bool = true;
            }
            Op::Not => {
                bc.push(0x22);
                top_is_bool = true;
            }
            Op::RangeI { lo, hi } => {
                bc.extend_from_slice(&push_i32(lo)?);
                bc.extend_from_slice(&push_i32(hi)?);
                bc.push(0x10); // CHECK_RANGE_I32 pops max, min, val
                top_is_bool = true;
            }
            Op::Assert => {
                if !top_is_bool {
                    return Err(
                        "monitor has no ASSERT opcode; asserting a non-bool value is \
                         unrepresentable (run() would vacuously pass)"
                            .into(),
                    );
                }
                // Deferred: the run() boolean is the outcome.
            }
            Op::Nop => bc.push(0x00),
            Op::Halt => bc.push(0xFF),
            Op::Pop => {
                bc.push(0x03);
                top_is_bool = false;
            }
            Op::DivI => return Err("C monitor has no DIV".into()),
        }
    }
    Ok(bc)
}

pub fn run(case: &Case) -> Triple {
    if case.gas_override.is_some() {
        return skip("no gas metering in monitor — gas-budget probe not applicable");
    }
    let bc = match compile(&case.ops, case.input) {
        Ok(x) => x,
        Err(e) => return skip(&e),
    };
    if bc.len() > 256 {
        return skip("program larger than FLUX_MAX_PROGRAM (256)");
    }
    let m = unsafe { conf_monitor_new() };
    if m.is_null() {
        return skip("monitor alloc failed");
    }
    let loaded = unsafe { conf_monitor_load(m, bc.as_ptr(), bc.len() as u16) };
    let passed = loaded && unsafe { conf_monitor_run(m) };
    let err = unsafe { conf_monitor_error(m) };
    unsafe { conf_monitor_free(m) };

    if !loaded {
        return Triple {
            outcome: Outcome::Fault(FaultKind::BadBytecode),
            gas_used: None,
            encoded: hex(&bc),
            note: "flux_monitor_load rejected program".into(),
        };
    }
    let outcome = if err != 0 {
        match err {
            1 => Outcome::Fault(FaultKind::StackOverflow),
            2 => Outcome::Fault(FaultKind::StackUnderflow),
            3 => Outcome::Fault(FaultKind::Other), // type mismatch
            4 => Outcome::Fault(FaultKind::InvalidOpcode),
            5 => Outcome::Fault(FaultKind::BadBytecode),
            _ => Outcome::Fault(FaultKind::Other),
        }
    } else if passed {
        Outcome::Pass
    } else {
        Outcome::Fail
    };
    Triple {
        outcome,
        gas_used: None,
        encoded: hex(&bc),
        note: "no gas metering in monitor".into(),
    }
}
