//! The differ: runs every implementation on every case and flags divergence
//! — against the reference interpreter and against each other.

use crate::corpus::Case;
use crate::ir::{reference_run, Outcome, Triple};
use crate::opmap::ImplId;
use crate::runners::run_all;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Implementations disagree on pass/fail — a guard would fire on one
    /// boat and stay silent on another.
    Critical,
    /// One implementation faults (underflow/div-zero/gas) where another
    /// returns a boolean result, or vice versa.
    Major,
    /// Both fault, differently.
    Minor,
    /// Coverage gap: an implementation cannot express the case.
    Info,
}

impl Severity {
    pub fn name(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::Major => "MAJOR",
            Severity::Minor => "MINOR",
            Severity::Info => "INFO",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Divergence {
    pub severity: Severity,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct CaseReport {
    pub case: Case,
    pub program_text: String,
    pub ref_outcome: Outcome,
    pub results: Vec<(ImplId, Triple)>,
    pub divergences: Vec<Divergence>,
}

impl CaseReport {
    pub fn has_divergence(&self) -> bool {
        self.divergences
            .iter()
            .any(|d| d.severity != Severity::Info)
    }
}

pub fn evaluate_case(case: Case) -> CaseReport {
    let ref_outcome = reference_run(&case.ops, case.input);
    let results = run_all(&case);
    let mut divergences = Vec::new();
    let gas_probe = case.gas_override.is_some();

    if gas_probe {
        // Gas-budget probe: the reference has no gas, so ref comparison is
        // not applicable. Only the metering implementations run (the others
        // skip); they must agree with each other.
        divergences.push(Divergence {
            severity: Severity::Info,
            detail: "gas-budget probe: reference comparison not applicable; \
                     only gas-metering implementations compared"
                .into(),
        });
    }

    // 1) Reference mismatches.
    for (id, triple) in &results {
        match (&triple.outcome, &ref_outcome) {
            (Outcome::Skipped(reason), _) => {
                divergences.push(Divergence {
                    severity: Severity::Info,
                    detail: format!("{} cannot run this case: {reason}", id.name()),
                });
            }
            _ if gas_probe => {}
            (got, want) if got == want => {}
            (Outcome::Pass | Outcome::Fail, Outcome::Pass | Outcome::Fail) => {
                divergences.push(Divergence {
                    severity: Severity::Critical,
                    detail: format!(
                        "{} returned {} but reference says {}",
                        id.name(),
                        triple.outcome.short(),
                        ref_outcome.short()
                    ),
                });
            }
            (Outcome::Fault(_), Outcome::Pass | Outcome::Fail)
            | (Outcome::Pass | Outcome::Fail, Outcome::Fault(_)) => {
                divergences.push(Divergence {
                    severity: Severity::Major,
                    detail: format!(
                        "{} faulted where the reference says {}: {}",
                        id.name(),
                        ref_outcome.short(),
                        triple.outcome.short()
                    ),
                });
            }
            (got, want) => {
                divergences.push(Divergence {
                    severity: Severity::Minor,
                    detail: format!(
                        "{} fault kind {} vs reference {}",
                        id.name(),
                        got.short(),
                        want.short()
                    ),
                });
            }
        }
    }

    // 2) Inter-implementation splits (independent of the reference).
    let comparables: Vec<_> = results
        .iter()
        .filter_map(|(id, t)| t.outcome.comparable().map(|c| (*id, c, &t.outcome)))
        .collect();
    let classes: std::collections::BTreeSet<&str> =
        comparables.iter().map(|(_, c, _)| *c).collect();
    if classes.len() > 1 {
        let who: Vec<String> = comparables
            .iter()
            .map(|(id, _, o)| format!("{}={}", id.name(), o.short()))
            .collect();
        divergences.push(Divergence {
            severity: Severity::Critical,
            detail: format!("implementations disagree: {}", who.join(", ")),
        });
    } else if classes.len() == 1 && *classes.iter().next().unwrap() == "fault" {
        // All faulted — check fault kinds.
        let kinds: std::collections::BTreeSet<String> = comparables
            .iter()
            .map(|(_, _, o)| match o {
                Outcome::Fault(k) => k.to_string(),
                _ => String::new(),
            })
            .collect();
        if kinds.len() > 1 {
            divergences.push(Divergence {
                severity: Severity::Minor,
                detail: format!("fault kinds differ across implementations: {kinds:?}"),
            });
        }
    }

    CaseReport {
        program_text: crate::ir::fmt_program(&case.ops, case.input),
        ref_outcome,
        results,
        divergences,
        case,
    }
}

pub fn evaluate_corpus() -> Vec<CaseReport> {
    crate::corpus::corpus()
        .into_iter()
        .map(evaluate_case)
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub cases: usize,
    pub critical: usize,
    pub major: usize,
    pub minor: usize,
    pub info: usize,
    pub cases_with_divergence: usize,
    pub impl_runs: usize,
    pub impl_skips: usize,
}

impl Summary {
    pub fn from_reports(reports: &[CaseReport]) -> Self {
        let mut s = Summary {
            cases: reports.len(),
            ..Default::default()
        };
        for r in reports {
            let mut has = false;
            for d in &r.divergences {
                match d.severity {
                    Severity::Critical => {
                        s.critical += 1;
                        has = true;
                    }
                    Severity::Major => {
                        s.major += 1;
                        has = true;
                    }
                    Severity::Minor => {
                        s.minor += 1;
                        has = true;
                    }
                    Severity::Info => s.info += 1,
                }
            }
            if has {
                s.cases_with_divergence += 1;
            }
            for (_, t) in &r.results {
                s.impl_runs += 1;
                if matches!(t.outcome, Outcome::Skipped(_)) {
                    s.impl_skips += 1;
                }
            }
        }
        s
    }
}
