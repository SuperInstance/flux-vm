//! flux-isa runner — instruction-list VM (f64 stack), `ConstraintVM`.
//!
//! Contract:
//! - programs are Vec<FluxInstruction> (opcode + f64 operands + metadata);
//! - the binary encode() is a 4-byte header (opcode, argc, flags, reserved)
//!   plus 8-byte LE f64 operands per operand;
//! - Load pushes operand[0]; Validate takes [min, max] operands, pops the
//!   value, pushes 1/0; Assert pops and errors on zero.

use crate::corpus::Case;
use crate::ir::{FaultKind, Op, Outcome, Triple};
use crate::runners::skip;
use flux_isa::bytecode::FluxBytecode;
use flux_isa::instruction::FluxInstruction;
use flux_isa::opcode::FluxOpcode;
use flux_isa::vm::ConstraintVM;

fn compile(ops: &[Op], input: i64) -> Result<Vec<FluxInstruction>, String> {
    let mut prog = Vec::new();
    let f = |v: i64| v as f64;
    for &op in ops {
        let instr = match op {
            Op::Input => FluxInstruction::with_operands(FluxOpcode::Load, vec![f(input)]),
            Op::PushI(v) => FluxInstruction::with_operands(FluxOpcode::Load, vec![f(v)]),
            Op::Add => FluxInstruction::new(FluxOpcode::Add),
            Op::Sub => FluxInstruction::new(FluxOpcode::Sub),
            Op::Mul => FluxInstruction::new(FluxOpcode::Mul),
            Op::DivI => FluxInstruction::new(FluxOpcode::Div),
            Op::Eq => FluxInstruction::new(FluxOpcode::Eq),
            Op::Ne => FluxInstruction::new(FluxOpcode::Neq),
            Op::Lt => FluxInstruction::new(FluxOpcode::Lt),
            Op::Gt => FluxInstruction::new(FluxOpcode::Gt),
            Op::Le => FluxInstruction::new(FluxOpcode::Lte),
            Op::Ge => FluxInstruction::new(FluxOpcode::Gte),
            Op::And => FluxInstruction::new(FluxOpcode::And),
            Op::Or => FluxInstruction::new(FluxOpcode::Or),
            Op::Not => FluxInstruction::new(FluxOpcode::Not),
            Op::RangeI { lo, hi } => {
                FluxInstruction::with_operands(FluxOpcode::Validate, vec![f(lo), f(hi)])
            }
            Op::Assert => FluxInstruction::new(FluxOpcode::Assert),
            Op::Nop => FluxInstruction::new(FluxOpcode::Nop),
            Op::Halt => FluxInstruction::new(FluxOpcode::Halt),
            Op::Pop => FluxInstruction::new(FluxOpcode::Pop),
        };
        prog.push(instr);
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
    let bc = FluxBytecode::from_instructions(prog);
    let encoded = crate::runners::hex(&bc.encode());
    let mut vm = ConstraintVM::new();
    match vm.execute(&bc) {
        Ok(result) => {
            if result.constraints_satisfied {
                Triple {
                    outcome: Outcome::Pass,
                    gas_used: None,
                    encoded,
                    note: String::new(),
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
        Err(e) => {
            let dbg = format!("{e:?}");
            // flux-isa reports a failed ASSERT as ConstraintViolation — that
            // is a constraint RESULT, not an execution fault.
            if dbg.contains("ConstraintViolation") {
                Triple {
                    outcome: Outcome::Fail,
                    gas_used: None,
                    encoded,
                    note: format!("FluxError: {e}"),
                }
            } else {
                let kind = if dbg.contains("Arithmetic") {
                    FaultKind::DivByZero
                } else if dbg.contains("Underflow") {
                    FaultKind::StackUnderflow
                } else if dbg.contains("Overflow") {
                    FaultKind::StackOverflow
                } else {
                    FaultKind::Other
                };
                Triple {
                    outcome: Outcome::Fault(kind),
                    gas_used: None,
                    encoded,
                    note: format!("FluxError: {e}"),
                }
            }
        }
    }
}
