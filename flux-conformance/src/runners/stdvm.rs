//! flux-isa-std runner — f64 stack VM with 35-opcode ISA.
//!
//! No RANGE/VALIDATE opcode, so `RangeI{lo,hi}` compiles to the composition
//! `DUP; DUP; Push lo; Ge; Push hi; Le; And` (stack: [v] → [v, v≥lo, v≤hi]
//! → [v, in-range]) followed by the corpus Assert.

use crate::corpus::Case;
use crate::ir::{FaultKind, Op, Outcome, Triple};
use crate::runners::{hex, skip};
use flux_isa_std::bytecode::FluxBytecode;
use flux_isa_std::instruction::FluxInstruction;
use flux_isa_std::opcode::FluxOpCode;
use flux_isa_std::vm::{FluxVM, VMError};

fn ins(op: FluxOpCode) -> FluxInstruction {
    FluxInstruction::new(op)
}
fn insf(op: FluxOpCode, v: i64) -> FluxInstruction {
    FluxInstruction::new(op).with_operand(v as f64)
}

fn compile(ops: &[Op], input: i64) -> Result<Vec<FluxInstruction>, String> {
    let mut prog = Vec::new();
    for &op in ops {
        match op {
            Op::Input => prog.push(insf(FluxOpCode::Push, input)),
            Op::PushI(v) => prog.push(insf(FluxOpCode::Push, v)),
            Op::Add => prog.push(ins(FluxOpCode::Add)),
            Op::Sub => prog.push(ins(FluxOpCode::Sub)),
            Op::Mul => prog.push(ins(FluxOpCode::Mul)),
            Op::DivI => prog.push(ins(FluxOpCode::Div)),
            Op::Eq => prog.push(ins(FluxOpCode::Eq)),
            Op::Ne => prog.push(ins(FluxOpCode::Ne)),
            Op::Lt => prog.push(ins(FluxOpCode::Lt)),
            Op::Gt => prog.push(ins(FluxOpCode::Gt)),
            Op::Le => prog.push(ins(FluxOpCode::Le)),
            Op::Ge => prog.push(ins(FluxOpCode::Ge)),
            Op::And => prog.push(ins(FluxOpCode::And)),
            Op::Or => prog.push(ins(FluxOpCode::Or)),
            Op::Not => prog.push(ins(FluxOpCode::Not)),
            Op::RangeI { lo, hi } => {
                prog.push(ins(FluxOpCode::Dup));
                prog.push(insf(FluxOpCode::Push, lo));
                prog.push(ins(FluxOpCode::Ge));
                prog.push(ins(FluxOpCode::Swap));
                prog.push(insf(FluxOpCode::Push, hi));
                prog.push(ins(FluxOpCode::Le));
                prog.push(ins(FluxOpCode::And));
            }
            Op::Assert => prog.push(ins(FluxOpCode::Assert)),
            Op::Nop => prog.push(ins(FluxOpCode::Nop)),
            Op::Halt => prog.push(ins(FluxOpCode::Halt)),
            Op::Pop => prog.push(ins(FluxOpCode::Pop)),
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
    let bc = FluxBytecode::new(prog);
    let encoded = hex(&bc.encode());
    let mut vm = FluxVM::with_default_config();
    match vm.execute_bytecode(&bc) {
        Ok(()) => Triple {
            outcome: Outcome::Pass,
            gas_used: None,
            encoded,
            note: String::new(),
        },
        Err(VMError::AssertionFailed { .. }) | Err(VMError::ConstraintFailed { .. }) => Triple {
            outcome: Outcome::Fail,
            gas_used: None,
            encoded,
            note: String::new(),
        },
        Err(VMError::DivisionByZero) => Triple {
            outcome: Outcome::Fault(FaultKind::DivByZero),
            gas_used: None,
            encoded,
            note: String::new(),
        },
        Err(VMError::StackUnderflow { .. }) => Triple {
            outcome: Outcome::Fault(FaultKind::StackUnderflow),
            gas_used: None,
            encoded,
            note: String::new(),
        },
        Err(e) => Triple {
            outcome: Outcome::Fault(FaultKind::Other),
            gas_used: None,
            encoded,
            note: format!("VMError: {e}"),
        },
    }
}
