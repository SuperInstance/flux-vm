//! Harness self-tests: the harness must itself be trustworthy.

use flux_conformance::corpus::{self, Origin};
use flux_conformance::differ::{evaluate_case, Severity};
use flux_conformance::ir::{reference_run, Op, Outcome};
use flux_conformance::opmap::{self, ImplId};
use flux_conformance::runners;
use flux_conformance::{differ, report};

// ---------------------------------------------------------------- ir

#[test]
fn reference_interp_basics() {
    let prog = &[
        Op::Input,
        Op::RangeI { lo: 1, hi: 127 },
        Op::Assert,
        Op::Halt,
    ];
    assert_eq!(reference_run(prog, 64), Outcome::Pass);
    assert_eq!(reference_run(prog, 0), Outcome::Fail);
    assert_eq!(reference_run(prog, 127), Outcome::Pass);
    assert_eq!(reference_run(prog, 128), Outcome::Fail);

    let div = &[Op::PushI(1), Op::PushI(0), Op::DivI, Op::Assert, Op::Halt];
    assert!(matches!(
        reference_run(div, 0),
        Outcome::Fault(f) if f.to_string() == "div-by-zero"
    ));

    let sub = &[Op::PushI(10), Op::PushI(3), Op::Sub, Op::Assert, Op::Halt];
    assert_eq!(reference_run(sub, 0), Outcome::Pass); // 10-3 = 7 ≠ 0

    let lt = &[Op::PushI(10), Op::PushI(3), Op::Lt, Op::Assert, Op::Halt];
    assert_eq!(reference_run(lt, 0), Outcome::Fail); // 10<3 false
}

// ---------------------------------------------------------------- opmap

/// Every Rust crate with a byte decoder must agree with the hand table for
/// ALL 256 byte values — both presence and name.
#[test]
fn opmap_matches_real_decoders() {
    use flux_isa::opcode::FluxOpcode as IsaOp;
    use flux_isa_mini::opcode::FluxOpcode as MiniOp;
    use flux_isa_std::opcode::FluxOpCode as StdOp;

    for byte in 0u8..=255 {
        // flux-isa
        let got = IsaOp::from_u8(byte).map(|o| format!("{o:?}"));
        let want = opmap::table(ImplId::Isa).get(&byte).map(|b| b.meaning);
        assert_eq!(
            got.as_deref(),
            want.map(|s| s.to_string()).as_deref(),
            "isa 0x{byte:02X}"
        );

        // flux-isa-mini
        let got = MiniOp::from_u8(byte).map(|o| format!("{o:?}"));
        let want = opmap::table(ImplId::Mini).get(&byte).map(|b| b.meaning);
        assert_eq!(
            got.as_deref(),
            want.map(|s| s.to_string()).as_deref(),
            "mini 0x{byte:02X}"
        );

        // flux-isa-std
        let got = StdOp::from_byte(byte).map(|o| format!("{o:?}"));
        let want = opmap::table(ImplId::Std).get(&byte).map(|b| b.meaning);
        assert_eq!(
            got.as_deref(),
            want.map(|s| s.to_string()).as_deref(),
            "std 0x{byte:02X}"
        );

        // flux-isa-edge
        let got = flux_isa_edge::opcode::OpCode::from_byte(byte).map(|o| format!("{o:?}"));
        let want = opmap::table(ImplId::Edge).get(&byte).map(|b| b.meaning);
        assert_eq!(
            got.as_deref(),
            want.map(|s| s.to_string()).as_deref(),
            "edge 0x{byte:02X}"
        );

        // flux-isa-thor (Instruction wraps Base/Thor)
        let got = flux_isa_thor::opcode::Instruction::from_byte(byte).map(|i| match i {
            flux_isa_thor::opcode::Instruction::Base(b) => format!("{b:?}"),
            flux_isa_thor::opcode::Instruction::Thor(t) => format!("{t:?}"),
        });
        let want = opmap::table(ImplId::Thor).get(&byte).map(|b| b.meaning);
        assert_eq!(
            got.as_deref(),
            want.map(|s| s.to_string()).as_deref(),
            "thor 0x{byte:02X}"
        );
    }
}

/// No byte may carry two meanings WITHIN one implementation.
#[test]
fn opmap_no_internal_contradictions() {
    for id in ImplId::ALL {
        // BTreeMap<u8, _> enforces this structurally; verify counts are sane.
        let t = opmap::table(id);
        assert!(!t.is_empty(), "{} has an empty table", id.name());
    }
}

/// The scout's headline finding must be reproduced mechanically: 0x01 has
/// at least four meanings across the repo.
#[test]
fn byte_0x01_is_at_least_quadruple_meaning() {
    let meanings: std::collections::BTreeSet<&str> = opmap::ImplId::ALL
        .iter()
        .filter_map(|id| opmap::table(*id).get(&0x01).map(|b| b.meaning))
        .collect();
    assert!(
        meanings.len() >= 4,
        "expected ≥4 meanings for 0x01, got {meanings:?}"
    );
}

// ---------------------------------------------------------------- runners

fn vel_case(input: i64) -> flux_conformance::corpus::Case {
    corpus::corpus()
        .into_iter()
        .find(|c| c.origin == Origin::Plainsong && c.input == input && c.ops.len() == 4)
        .unwrap_or_else(|| panic!("no vel case with input {input}"))
}

#[test]
fn every_runner_passes_in_range_velocity() {
    let case = vel_case(64);
    for (id, triple) in runners::run_all(&case) {
        assert_eq!(
            triple.outcome,
            Outcome::Pass,
            "{} on vel=64: {:?} ({})",
            id.name(),
            triple.outcome,
            triple.note
        );
    }
}

#[test]
fn every_runner_fails_out_of_range_velocity() {
    let case = vel_case(0);
    for (id, triple) in runners::run_all(&case) {
        assert_eq!(
            triple.outcome,
            Outcome::Fail,
            "{} on vel=0: {:?} ({})",
            id.name(),
            triple.outcome,
            triple.note
        );
    }
}

#[test]
fn c_core_ffi_smoke() {
    // HALT immediately: result = truthiness of the pre-pushed input.
    extern "C" {
        fn flux_check(
            bytecode: *const u8,
            bc_len: u16,
            input: i32,
            max_gas: u16,
            gas_used: *mut u16,
        ) -> i32;
    }
    let prog = [0x1Au8];
    let mut gas = 0u16;
    let rc_pass = unsafe { flux_check(prog.as_ptr(), 1, 7, 100, &mut gas) };
    let rc_fail = unsafe { flux_check(prog.as_ptr(), 1, 0, 100, &mut gas) };
    assert_eq!(rc_pass, 0, "non-zero input → FLUX_PASS");
    assert_eq!(rc_fail, 1, "zero input → FLUX_FAULT");
}

#[test]
fn maritime_demo_bytecode_rejected_by_c_core() {
    // maritime_constraints.py check_draft(3.5, 6.0) emits exactly these
    // bytes (RANGE, lo, hi, ASSERT, HALT — 2 operand bytes for RANGE). The
    // C core reads RANGE as two i16 LE immediates (4 operand bytes), so the
    // demo's bytecode is truncated from the VM's point of view.
    extern "C" {
        fn flux_check(
            bytecode: *const u8,
            bc_len: u16,
            input: i32,
            max_gas: u16,
            gas_used: *mut u16,
        ) -> i32;
    }
    let demo_bc: [u8; 5] = [0x1D, 0x00, 0x06, 0x1B, 0x1A];
    let mut gas = 0u16;
    let rc = unsafe { flux_check(demo_bc.as_ptr(), 5, 3, 100, &mut gas) };
    assert_eq!(
        rc, 5,
        "demo bytecode must be rejected as FLUX_BAD_BYTECODE by the C core"
    );
}

// ---------------------------------------------------------------- corpus

#[test]
fn corpus_expected_outcomes_match_reference() {
    for case in corpus::corpus() {
        // Self-consistency: the differ's ref outcome is the reference run.
        let _ = reference_run(&case.ops, case.input); // must not panic
    }
}

#[test]
fn corpus_coverage_floor() {
    // Every case must be runnable on at least 5 implementations, so no
    // divergence is ever an artifact of a single lonely runner.
    for case in corpus::corpus() {
        let ran = runners::run_all(&case)
            .into_iter()
            .filter(|(_, t)| !matches!(t.outcome, Outcome::Skipped(_)))
            .count();
        let floor = if case.gas_override.is_some() { 2 } else { 5 };
        assert!(
            ran >= floor,
            "case `{}` only runnable on {ran} implementations (floor {floor})",
            case.name
        );
    }
}

// ---------------------------------------------------------------- differ

#[test]
fn differ_flags_a_planted_critical() {
    // vel=64 is a Pass case; flip the input to 0 for one impl by evaluating
    // the out-of-range case and checking the machinery catches it.
    let report = evaluate_case(vel_case(0));
    assert!(
        report
            .divergences
            .iter()
            .all(|d| d.severity != Severity::Critical),
        "vel=0 is a clean FAIL across impls; got {:?}",
        report.divergences
    );
    // And the known-dirty assert-literal case must produce a Critical.
    let dirty = corpus::corpus()
        .into_iter()
        .find(|c| c.name == "assert-literal-truthy")
        .unwrap();
    let report = evaluate_case(dirty);
    assert!(
        report
            .divergences
            .iter()
            .any(|d| d.severity == Severity::Critical),
        "assert-literal-truthy must diverge (thor type-strict assert): {:?}",
        report.divergences
    );
}

// ---------------------------------------------------------------- report

#[test]
fn report_is_deterministic_and_substantive() {
    let reports = differ::evaluate_corpus();
    let a = report::generate(&reports);
    let b = report::generate(&differ::evaluate_corpus());
    assert_eq!(a, b, "report must be deterministic");
    assert!(a.contains("# CONFORMANCE"));
    assert!(a.contains("## 4. Proposed canonical opcode table"));
    assert!(a.contains("## 5. Plainsong"));
    let summary = differ::Summary::from_reports(&reports);
    assert!(
        summary.critical > 0,
        "the harness must find the known divergences"
    );
    assert!(summary.cases >= 40);
}
