//! flux-isa-edge runner — async edge VM with step/time limits.
//!
//! Contract:
//! - Instruction { opcode, operand: Option<f64> }, wire format
//!   [opcode, flag, (f64 LE)];
//! - Validate pops max, min, value (push order: val, min, max), pushes 1/0;
//! - Assert pops; zero halts with ConstraintViolation;
//! - ExecutionResult.success == false on any halt-reason error; violations
//!   counts failed Validate/Assert/Tolerance checks.

use crate::corpus::Case;
use crate::ir::{FaultKind, Op, Outcome, Triple};
use crate::runners::{hex, skip};
use flux_isa_edge::bytecode::Bytecode;
use flux_isa_edge::instruction::Instruction;
use flux_isa_edge::opcode::OpCode;
use flux_isa_edge::vm::{ExecutionLimits, Vm};

fn compile(ops: &[Op], input: i64) -> Result<Vec<Instruction>, String> {
    let mut prog = Vec::new();
    let lit = |v: i64| Instruction::with_operand(OpCode::Push, v as f64);
    for &op in ops {
        match op {
            Op::Input => prog.push(lit(input)),
            Op::PushI(v) => prog.push(lit(v)),
            Op::Add => prog.push(Instruction::new(OpCode::Add)),
            Op::Sub => prog.push(Instruction::new(OpCode::Sub)),
            Op::Mul => prog.push(Instruction::new(OpCode::Mul)),
            Op::DivI => prog.push(Instruction::new(OpCode::Div)),
            Op::Eq => prog.push(Instruction::new(OpCode::Eq)),
            Op::Ne => prog.push(Instruction::new(OpCode::Ne)),
            Op::Lt => prog.push(Instruction::new(OpCode::Lt)),
            Op::Gt => prog.push(Instruction::new(OpCode::Gt)),
            Op::Le => prog.push(Instruction::new(OpCode::Le)),
            Op::Ge => prog.push(Instruction::new(OpCode::Ge)),
            Op::And => prog.push(Instruction::new(OpCode::And)),
            Op::Or => prog.push(Instruction::new(OpCode::Or)),
            Op::Not => prog.push(Instruction::new(OpCode::Not)),
            Op::RangeI { lo, hi } => {
                prog.push(lit(lo));
                prog.push(lit(hi));
                prog.push(Instruction::new(OpCode::Validate));
            }
            Op::Assert => prog.push(Instruction::new(OpCode::Assert)),
            Op::Nop => prog.push(Instruction::new(OpCode::Nop)),
            Op::Halt => prog.push(Instruction::new(OpCode::Halt)),
            Op::Pop => prog.push(Instruction::new(OpCode::Pop)),
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
    let bc = Bytecode::new(prog);
    let encoded = hex(&bc.encode());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let result = rt.block_on(async {
        let mut vm = Vm::new(ExecutionLimits::default());
        vm.execute(&bc).await
    });

    let constraint_failed = result
        .error
        .as_deref()
        .is_some_and(|e| e.contains("ConstraintViolation"))
        || (result.success && result.violations > 0);
    let outcome = if result.success && result.violations == 0 {
        Outcome::Pass
    } else if constraint_failed {
        Outcome::Fail
    } else {
        let err = result.error.clone().unwrap_or_default();
        let kind = match err.as_str() {
            "DivisionByZero" => FaultKind::DivByZero,
            "StackUnderflow" => FaultKind::StackUnderflow,
            "StackOverflow" => FaultKind::StackOverflow,
            "MaxStepsExceeded" | "Timeout" => FaultKind::GasExhausted,
            _ => FaultKind::Other,
        };
        Outcome::Fault(kind)
    };
    Triple {
        outcome,
        gas_used: None,
        encoded,
        note: format!(
            "steps={} violations={}",
            result.steps_executed, result.violations
        ),
    }
}
