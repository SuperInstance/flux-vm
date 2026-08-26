//! flux-isa-thor runner — async CSP-flavored VM over raw bytes.
//!
//! Contract / quirks (all load-bearing for the conformance report):
//! - PUSH (0x01) immediate is f64 **big-endian** (the only BE encoder in the
//!   repo);
//! - comparisons push Value::Bool; arithmetic pushes Value::F64;
//! - ASSERT pops via `Value::as_bool()`, which is TYPE-STRICT: an F64(1.0)
//!   is not a bool, so asserting a numeric value fails;
//! - binop/binop_bool silently DROP values (shrink the stack) on type
//!   mismatch instead of faulting;
//! - Div by zero yields NaN, not a fault;
//! - POP on an empty stack is a silent no-op (no underflow fault).

use crate::corpus::Case;
use crate::ir::{FaultKind, Op, Outcome, Triple};
use crate::runners::{hex, skip};
use flux_isa_thor::cuda::GpuDispatcher;
use flux_isa_thor::fleet::{FleetHandle, FleetNode, NodeRole, NodeStatus};
use flux_isa_thor::opcode::Opcode;
use flux_isa_thor::plato::cache::TileCache;
use flux_isa_thor::plato::client::PlatoClient;
use flux_isa_thor::plato::PlatoHandle;
use flux_isa_thor::vm::{ThorVm, VmConfig, VmStatus};
use std::sync::Arc;
use std::time::Duration;

fn push_f64_be(v: i64, bc: &mut Vec<u8>) {
    bc.push(0x01);
    bc.extend_from_slice(&(v as f64).to_be_bytes());
}

fn compile(ops: &[Op], input: i64) -> Result<Vec<u8>, String> {
    let mut bc: Vec<u8> = Vec::new();
    for &op in ops {
        match op {
            Op::Input => push_f64_be(input, &mut bc),
            Op::PushI(v) => push_f64_be(v, &mut bc),
            Op::Add => bc.push(Opcode::Add as u8),
            Op::Sub => bc.push(Opcode::Sub as u8),
            Op::Mul => bc.push(Opcode::Mul as u8),
            Op::DivI => bc.push(Opcode::Div as u8),
            Op::Eq => bc.push(Opcode::Eq as u8),
            Op::Ne => bc.push(Opcode::Ne as u8),
            Op::Lt => bc.push(Opcode::Lt as u8),
            Op::Gt => bc.push(Opcode::Gt as u8),
            Op::Le => bc.push(Opcode::Le as u8),
            Op::Ge => bc.push(Opcode::Ge as u8),
            Op::And => bc.push(Opcode::And as u8),
            Op::Or => bc.push(Opcode::Or as u8),
            Op::Not => bc.push(Opcode::Not as u8),
            Op::RangeI { lo, hi } => {
                // DUP; Push lo; Ge; Swap; Push hi; Le; And
                // (Swap keeps the original value available for the second
                // comparison — comparisons consume their operands.)
                bc.push(Opcode::Dup as u8);
                push_f64_be(lo, &mut bc);
                bc.push(Opcode::Ge as u8);
                bc.push(Opcode::Swap as u8);
                push_f64_be(hi, &mut bc);
                bc.push(Opcode::Le as u8);
                bc.push(Opcode::And as u8);
            }
            Op::Assert => bc.push(Opcode::Assert as u8),
            Op::Nop => bc.push(Opcode::Nop as u8),
            Op::Halt => bc.push(Opcode::Halt as u8),
            Op::Pop => bc.push(Opcode::Pop as u8),
        }
    }
    Ok(bc)
}

fn make_vm() -> Arc<ThorVm> {
    let gpu = Arc::new(GpuDispatcher::new(false, 0, 4));
    let plato_client = Arc::new(PlatoClient::new(
        "http://localhost:1",
        4,
        Duration::from_secs(1),
    ));
    let cache = Arc::new(tokio::sync::RwLock::new(TileCache::new(100)));
    let plato = Arc::new(PlatoHandle::new(plato_client, cache));
    let fleet = Arc::new(FleetHandle::new(FleetNode {
        id: "conformance".into(),
        hostname: "conformance".into(),
        role: NodeRole::Thor,
        gpu_available: false,
        gpu_memory_mb: 0,
        status: NodeStatus::Online,
        last_heartbeat: 0,
    }));
    let config = VmConfig {
        trace_enabled: false,
        ..VmConfig::default()
    };
    Arc::new(ThorVm::new(config, gpu, plato, fleet))
}

pub fn run(case: &Case) -> Triple {
    if case.gas_override.is_some() {
        return skip("no gas metering — gas-budget probe not applicable");
    }
    let bc = match compile(&case.ops, case.input) {
        Ok(x) => x,
        Err(e) => return skip(&e),
    };
    let encoded = hex(&bc);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let vm = make_vm();
    let result = rt.block_on(async { vm.execute(&bc).await });

    let outcome = match result.status {
        VmStatus::Ok | VmStatus::Halted => Outcome::Pass,
        // With our encodings the only Error sources are a failed ASSERT (a
        // constraint result) — jump/truncation errors cannot occur.
        VmStatus::Error => Outcome::Fail,
        VmStatus::Timeout => Outcome::Fault(FaultKind::GasExhausted),
    };
    Triple {
        outcome,
        gas_used: None,
        encoded,
        note: format!(
            "instructions={} final_stack={:?}",
            result.metrics.instructions_executed, result.stack
        ),
    }
}
