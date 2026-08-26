//! Per-implementation runners: compile the semantic IR into each
//! implementation's own encoding, execute, and produce a
//! (result, gas_used, fault) triple.

pub mod ccore;
pub mod cmonitor;
pub mod edge;
pub mod isa;
pub mod mini;
pub mod stdvm;
pub mod thor;
pub mod vmrs;

use crate::corpus::Case;
use crate::ir::Triple;
use crate::opmap::ImplId;

pub fn run_all(case: &Case) -> Vec<(ImplId, Triple)> {
    vec![
        (ImplId::CoreC, ccore::run(case)),
        (ImplId::MonitorC, cmonitor::run(case)),
        (ImplId::VmRs, vmrs::run(case)),
        (ImplId::Isa, isa::run(case)),
        (ImplId::Mini, mini::run(case)),
        (ImplId::Std, stdvm::run(case)),
        (ImplId::Edge, edge::run(case)),
        (ImplId::Thor, thor::run(case)),
    ]
}

/// Hex for minimal repros.
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Skip outcome with a reason.
pub fn skip(reason: &str) -> Triple {
    Triple {
        outcome: crate::ir::Outcome::Skipped(reason.into()),
        gas_used: None,
        encoded: String::new(),
        note: String::new(),
    }
}
