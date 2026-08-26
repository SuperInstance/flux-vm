//! flux-isa-mini runner — no_std, 32-slot f64 stack, fixed 24-byte
//! instructions (opcode + 2×f64), "FL"-magic wire format.
//!
//! Contract:
//! - Load/Push push operand[0];
//! - Validate pops upper, lower, value (push order: val, lower, upper),
//!   pushes 1/0 and clears constraints_ok on false;
//! - Assert errors on zero; Halt stops.

use crate::corpus::Case;
use crate::ir::{FaultKind, Op, Outcome, Triple};
use crate::runners::{hex, skip};
use flux_isa_mini::instruction::FluxInstruction;
use flux_isa_mini::opcode::FluxOpcode;
use flux_isa_mini::vm::{FluxError, FluxVm};

fn compile(ops: &[Op], input: i64) -> Result<Vec<FluxInstruction>, String> {
    let mut prog = Vec::new();
    let lit = |v: i64| FluxInstruction::new(FluxOpcode::Load, v as f64, 0.0);
    for &op in ops {
        match op {
            Op::Input => prog.push(lit(input)),
            Op::PushI(v) => prog.push(lit(v)),
            Op::Add => prog.push(FluxInstruction::new(FluxOpcode::Add, 0.0, 0.0)),
            Op::Sub => prog.push(FluxInstruction::new(FluxOpcode::Sub, 0.0, 0.0)),
            Op::Mul => prog.push(FluxInstruction::new(FluxOpcode::Mul, 0.0, 0.0)),
            Op::DivI => prog.push(FluxInstruction::new(FluxOpcode::Div, 0.0, 0.0)),
            Op::Eq => prog.push(FluxInstruction::new(FluxOpcode::Eq, 0.0, 0.0)),
            Op::Lt => prog.push(FluxInstruction::new(FluxOpcode::Lt, 0.0, 0.0)),
            Op::Gt => prog.push(FluxInstruction::new(FluxOpcode::Gt, 0.0, 0.0)),
            Op::Le => prog.push(FluxInstruction::new(FluxOpcode::Lte, 0.0, 0.0)),
            Op::Ge => prog.push(FluxInstruction::new(FluxOpcode::Gte, 0.0, 0.0)),
            Op::RangeI { lo, hi } => {
                prog.push(lit(lo));
                prog.push(lit(hi));
                prog.push(FluxInstruction::new(FluxOpcode::Validate, 0.0, 0.0));
            }
            Op::Assert => prog.push(FluxInstruction::new(FluxOpcode::Assert, 0.0, 0.0)),
            Op::Nop => prog.push(FluxInstruction::new(FluxOpcode::Nop, 0.0, 0.0)),
            Op::Halt => prog.push(FluxInstruction::new(FluxOpcode::Halt, 0.0, 0.0)),
            Op::Pop => prog.push(FluxInstruction::new(FluxOpcode::Pop, 0.0, 0.0)),
            Op::Ne | Op::And | Op::Or | Op::Not => {
                return Err("flux-isa-mini has no NE/AND/OR/NOT (no DUP to compose)".into());
            }
        }
    }
    Ok(prog)
}

pub fn run(case: &Case) -> Triple {
    if case.gas_override.is_some() {
        return skip("no gas metering — gas-budget probe not applicable");
    }
    let prog = match compile(&case.ops, case.input) {
        Ok(x) => x,
        Err(e) => return skip(&e),
    };
    // Native wire format for the repro.
    let mut buf = vec![0u8; 4 + prog.len() * 24];
    let n = flux_isa_mini::encode::encode(&prog, &mut buf);
    let encoded = hex(&buf[..n]);

    let mut vm = FluxVm::new();
    match vm.execute(&prog) {
        Ok(result) => {
            if result.constraints_satisfied {
                Triple {
                    outcome: Outcome::Pass,
                    gas_used: None,
                    encoded,
                    note: format!("steps={}", result.steps_executed),
                }
            } else {
                Triple {
                    outcome: Outcome::Fail,
                    gas_used: None,
                    encoded,
                    note: "constraints_satisfied=false".into(),
                }
            }
        }
        Err(FluxError::ConstraintViolation) => Triple {
            outcome: Outcome::Fail,
            gas_used: None,
            encoded,
            note: String::new(),
        },
        Err(FluxError::DivisionByZero) => Triple {
            outcome: Outcome::Fault(FaultKind::DivByZero),
            gas_used: None,
            encoded,
            note: String::new(),
        },
        Err(FluxError::StackUnderflow) => Triple {
            outcome: Outcome::Fault(FaultKind::StackUnderflow),
            gas_used: None,
            encoded,
            note: String::new(),
        },
        Err(FluxError::StackOverflow) => Triple {
            outcome: Outcome::Fault(FaultKind::StackOverflow),
            gas_used: None,
            encoded,
            note: String::new(),
        },
        Err(FluxError::InvalidInstruction(b)) => Triple {
            outcome: Outcome::Fault(FaultKind::InvalidOpcode),
            gas_used: None,
            encoded,
            note: format!("invalid instruction byte {b:#X}"),
        },
    }
}
