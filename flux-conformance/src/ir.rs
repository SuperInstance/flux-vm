//! Semantic IR + reference interpreter.
//!
//! The nine implementations in this repo do not share a wire format, so the
//! golden corpus is defined at the *semantic* level: a small stack-program IR
//! that every runner compiles into its own implementation's encoding. The
//! reference interpreter below evaluates the IR in exact i64 arithmetic and
//! is the ground truth the differ compares against (and impls compare
//! against each other).

use std::fmt;

/// One semantic operation. Stack order convention: for binary ops the IR
/// sequence `[PushI(a), PushI(b), Add]` computes `a + b` (b is on top).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    /// Push the case's input value.
    Input,
    /// Push an integer constant.
    PushI(i64),
    /// Pop b, a; push a + b (wrapping, i64).
    Add,
    /// Pop b, a; push a - b.
    Sub,
    /// Pop b, a; push a * b.
    Mul,
    /// Pop b, a; push a / b truncated; Fault(DivByZero) if b == 0.
    DivI,
    /// Pop b, a; push 1 if a < b else 0.
    Lt,
    /// Pop b, a; push 1 if a <= b else 0.
    Le,
    /// Pop b, a; push 1 if a > b else 0.
    Gt,
    /// Pop b, a; push 1 if a >= b else 0.
    Ge,
    /// Pop b, a; push 1 if a == b else 0.
    Eq,
    /// Pop b, a; push 1 if a != b else 0.
    Ne,
    /// Pop b, a; push 1 if both non-zero (logical).
    And,
    /// Pop b, a; push 1 if either non-zero (logical).
    Or,
    /// Pop a; push 1 if a == 0 else 0 (LOGICAL not).
    Not,
    /// Pop v; push 1 if lo <= v <= hi else 0.
    RangeI {
        lo: i64,
        hi: i64,
    },
    /// Pop v; if v == 0 the program FAILS (constraint violated).
    Assert,
    /// Pop and discard.
    Pop,
    Nop,
    Halt,
}

/// How a program ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Ran to HALT with every ASSERT satisfied.
    Pass,
    /// Ran to an ASSERT that evaluated false (a *result*, not an error).
    Fail,
    /// Execution error (underflow, div-zero, gas, bad bytecode...).
    Fault(FaultKind),
    /// This implementation cannot express/run this case.
    Skipped(String),
}

impl Outcome {
    pub fn short(&self) -> String {
        match self {
            Outcome::Pass => "PASS".into(),
            Outcome::Fail => "FAIL".into(),
            Outcome::Fault(k) => format!("FAULT({k})"),
            Outcome::Skipped(r) => format!("SKIP({r})"),
        }
    }
    pub fn comparable(&self) -> Option<&'static str> {
        match self {
            Outcome::Pass => Some("pass"),
            Outcome::Fail => Some("fail"),
            Outcome::Fault(_) => Some("fault"),
            Outcome::Skipped(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultKind {
    DivByZero,
    StackUnderflow,
    StackOverflow,
    GasExhausted,
    InvalidOpcode,
    BadBytecode,
    ConstraintViolation,
    Other,
}

impl fmt::Display for FaultKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FaultKind::DivByZero => "div-by-zero",
            FaultKind::StackUnderflow => "stack-underflow",
            FaultKind::StackOverflow => "stack-overflow",
            FaultKind::GasExhausted => "gas-exhausted",
            FaultKind::InvalidOpcode => "invalid-opcode",
            FaultKind::BadBytecode => "bad-bytecode",
            FaultKind::ConstraintViolation => "constraint-violation",
            FaultKind::Other => "other",
        };
        f.write_str(s)
    }
}

/// The (result, gas_used, fault) triple every runner produces.
#[derive(Debug, Clone)]
pub struct Triple {
    pub outcome: Outcome,
    /// Gas actually consumed, when the implementation exposes it.
    pub gas_used: Option<u64>,
    /// The encoded bytes this runner produced (minimal-repro material).
    pub encoded: String,
    /// Free-form note (quirks hit, steps executed, ...).
    pub note: String,
}

/// Reference interpreter: exact i64 stack machine, no gas.
pub fn reference_run(ops: &[Op], input: i64) -> Outcome {
    let mut stack: Vec<i64> = Vec::new();

    let pop2 = |stack: &mut Vec<i64>| -> Result<(i64, i64), FaultKind> {
        if stack.len() < 2 {
            return Err(FaultKind::StackUnderflow);
        }
        let b = stack.pop().unwrap();
        let a = stack.pop().unwrap();
        Ok((a, b))
    };

    for &op in ops {
        match op {
            Op::Input => stack.push(input),
            Op::PushI(v) => stack.push(v),
            Op::Add => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(a.wrapping_add(b));
            }
            Op::Sub => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(a.wrapping_sub(b));
            }
            Op::Mul => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(a.wrapping_mul(b));
            }
            Op::DivI => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                if b == 0 {
                    return Outcome::Fault(FaultKind::DivByZero);
                }
                stack.push(a.wrapping_div(b));
            }
            Op::Lt => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(i64::from(a < b));
            }
            Op::Le => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(i64::from(a <= b));
            }
            Op::Gt => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(i64::from(a > b));
            }
            Op::Ge => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(i64::from(a >= b));
            }
            Op::Eq => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(i64::from(a == b));
            }
            Op::Ne => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(i64::from(a != b));
            }
            Op::And => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(i64::from(a != 0 && b != 0));
            }
            Op::Or => {
                let (a, b) = match pop2(&mut stack) {
                    Ok(x) => x,
                    Err(e) => return Outcome::Fault(e),
                };
                stack.push(i64::from(a != 0 || b != 0));
            }
            Op::Not => {
                let Some(a) = stack.pop() else {
                    return Outcome::Fault(FaultKind::StackUnderflow);
                };
                stack.push(i64::from(a == 0));
            }
            Op::RangeI { lo, hi } => {
                let Some(v) = stack.pop() else {
                    return Outcome::Fault(FaultKind::StackUnderflow);
                };
                stack.push(i64::from(lo <= v && v <= hi));
            }
            Op::Assert => {
                let Some(v) = stack.pop() else {
                    return Outcome::Fault(FaultKind::StackUnderflow);
                };
                if v == 0 {
                    return Outcome::Fail;
                }
            }
            Op::Pop => {
                if stack.pop().is_none() {
                    return Outcome::Fault(FaultKind::StackUnderflow);
                }
            }
            Op::Nop => {}
            Op::Halt => break,
        }
    }

    // Every corpus program ends in Assert; Halt, so reaching here means all
    // asserts were satisfied.
    Outcome::Pass
}

/// Human-readable one-line form of a program (minimal-repro text).
pub fn fmt_program(ops: &[Op], input: i64) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for op in ops {
        let tok = match *op {
            Op::Input => "INPUT".into(),
            Op::PushI(v) => format!("PUSH {v}"),
            Op::Add => "ADD".into(),
            Op::Sub => "SUB".into(),
            Op::Mul => "MUL".into(),
            Op::DivI => "DIV".into(),
            Op::Lt => "LT".into(),
            Op::Le => "LE".into(),
            Op::Gt => "GT".into(),
            Op::Ge => "GE".into(),
            Op::Eq => "EQ".into(),
            Op::Ne => "NE".into(),
            Op::And => "AND".into(),
            Op::Or => "OR".into(),
            Op::Not => "NOT".into(),
            Op::RangeI { lo, hi } => format!("RANGE[{lo},{hi}]"),
            Op::Assert => "ASSERT".into(),
            Op::Pop => "POP".into(),
            Op::Nop => "NOP".into(),
            Op::Halt => "HALT".into(),
        };
        let _ = write!(s, "{tok} ");
    }
    format!("input={} | {}", input, s.trim_end())
}
