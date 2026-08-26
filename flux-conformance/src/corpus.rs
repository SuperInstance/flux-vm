//! The golden corpus.
//!
//! Every case is a semantic-IR program (see `ir`) plus an input value. The
//! expected outcome is computed by the reference interpreter — the corpus
//! itself never hardcodes expectations, so a case cannot silently agree with
//! a buggy implementation.
//!
//! Provenance:
//! - `TheirTests` — lifted from the repo's own test suites / examples.
//! - `Maritime` — the maritime safety corpus (`src/maritime_constraints.py`,
//!   `flux-isa-mini/src/sonar_check.rs`, `flux-isa-std` sonar bounds).
//! - `Plainsong` — numeric-range predicates shaped like the plainsong [Perf]
//!   velocity envelopes (vel ∈ [1,127]; velocity_std ∈ [0.113,0.257] scaled
//!   to milli-units [113,257] so u8/i32/f64 domains all stay exact). These
//!   are the cell-cascade guard use-case.
//! - `Harness` — probes added by this harness to pin operand order and
//!   opcode semantics across implementations.

use crate::ir::Op;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    TheirTests,
    Maritime,
    Plainsong,
    Harness,
}

impl Origin {
    pub fn name(&self) -> &'static str {
        match self {
            Origin::TheirTests => "their-tests",
            Origin::Maritime => "maritime",
            Origin::Plainsong => "plainsong-perf",
            Origin::Harness => "harness-probe",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub name: String,
    pub origin: Origin,
    pub ops: Vec<Op>,
    pub input: i64,
    /// Gas/steps budget override for implementations that meter gas.
    pub gas_override: Option<u32>,
    pub note: String,
}

fn case(name: &str, origin: Origin, input: i64, ops: &[Op], note: &str) -> Case {
    Case {
        name: name.into(),
        origin,
        ops: ops.to_vec(),
        input,
        gas_override: None,
        note: note.into(),
    }
}

/// The velocity-domain range used by the plainsong [Perf] envelopes.
const VEL_LO: i64 = 1;
const VEL_HI: i64 = 127;
/// velocity_std ∈ [0.113, 0.257] scaled ×1000 (exact in u8-adjacent, i32 and
/// f64 domains; 257 itself exceeds u8, which vm/flux_vm.rs cannot express —
/// itself a documented finding).
const VSTD_LO: i64 = 113;
const VSTD_HI: i64 = 257;

#[allow(clippy::vec_init_then_push)] // built incrementally with provenance
pub fn corpus() -> Vec<Case> {
    let mut c = Vec::new();

    // ---- Their tests -------------------------------------------------
    c.push(case(
        "add-3-4-assert",
        Origin::TheirTests,
        0,
        &[Op::PushI(3), Op::PushI(4), Op::Add, Op::Assert, Op::Halt],
        "flux-isa vm.rs test_basic / vm-rs test_push_add",
    ));
    c.push(case(
        "assert-zero-fails",
        Origin::TheirTests,
        0,
        &[Op::PushI(0), Op::Assert, Op::Halt],
        "flux-isa test_constraint_violation",
    ));
    c.push(case(
        "assert-literal-truthy",
        Origin::TheirTests,
        0,
        &[Op::PushI(1), Op::Assert, Op::Halt],
        "asserting a non-bool literal — thor Value::as_bool is type-strict",
    ));
    c.push(case(
        "div-by-zero",
        Origin::TheirTests,
        0,
        &[Op::PushI(1), Op::PushI(0), Op::DivI, Op::Assert, Op::Halt],
        "flux-isa Div error / mini DivisionByZero",
    ));
    c.push(case(
        "pop-empty-underflow",
        Origin::TheirTests,
        0,
        &[Op::Pop, Op::Assert, Op::Halt],
        "underflow probe; thor POP silently no-ops on empty stack",
    ));
    c.push(case(
        "validate-bounds-in",
        Origin::TheirTests,
        0,
        &[
            Op::PushI(5),
            Op::RangeI { lo: 0, hi: 10 },
            Op::Assert,
            Op::Halt,
        ],
        "flux-isa test_validate_bounds (5 in [0,10])",
    ));
    c.push(case(
        "validate-bounds-out",
        Origin::TheirTests,
        0,
        &[
            Op::PushI(5),
            Op::RangeI { lo: 6, hi: 10 },
            Op::Assert,
            Op::Halt,
        ],
        "flux-isa Validate with 5 in [6,10] → false",
    ));
    {
        let mut ops = vec![Op::Nop; 10];
        ops.extend_from_slice(&[
            Op::Input,
            Op::RangeI { lo: 1, hi: 127 },
            Op::Assert,
            Op::Halt,
        ]);
        c.push(Case {
            name: "gas-exhaustion".into(),
            origin: Origin::TheirTests,
            ops,
            input: 64,
            gas_override: Some(5),
            note: "C-core FLUX_GAS_EXHAUSTED / vm-rs GasExhausted probe; \
                   reference passes with adequate gas"
                .into(),
        });
    }
    c.push(case(
        "not-true-is-false",
        Origin::TheirTests,
        0,
        &[Op::PushI(1), Op::Not, Op::Assert, Op::Halt],
        "logical NOT(1)=0 → assert fails; vm-rs NOT is bitwise (!1=0xFE)",
    ));
    c.push(case(
        "not-false-is-true",
        Origin::TheirTests,
        0,
        &[Op::PushI(0), Op::Not, Op::Assert, Op::Halt],
        "logical NOT(0)=1 → passes; vm-rs bitwise !0=0xFF also passes",
    ));

    // ---- Maritime corpus ----------------------------------------------
    let maritime: &[(&str, i64, i64, i64, &str)] = &[
        ("draft-ok", 3, 0, 6, "check_draft(3.5→3, max 6)"),
        ("draft-over", 7, 0, 6, "check_draft(7.2→7, max 6)"),
        ("wind-ok", 15, 0, 40, "check_weather wind ∈ [0,40]"),
        ("wind-storm", 45, 0, 40, "check_weather storm 45 kn"),
        ("waves-ok", 4, 0, 15, "check_weather waves ∈ [0,15]"),
        ("waves-over", 16, 0, 15, "wave height 16 m"),
        ("vis-ok", 10, 1, 255, "visibility ∈ [1,255]"),
        ("vis-zero", 0, 1, 255, "visibility 0 → fail (min is 1)"),
        ("crew-ok", 10, 0, 16, "check_crew_hours 10/16"),
        ("crew-over", 20, 0, 16, "crew hours 20 > 16"),
        (
            "catch-cap",
            200,
            0,
            255,
            "check_catch_weight capped to u8 domain",
        ),
        ("catch-over", 300, 0, 255, "300 kg over capped hold"),
        (
            "sound-speed-ok",
            1500,
            1430,
            1560,
            "Mackenzie bounds, mini sonar_check",
        ),
        (
            "sound-speed-low",
            1400,
            1430,
            1560,
            "mini test_sound_speed_out_of_range",
        ),
        (
            "sound-speed-max-bound",
            1560,
            1430,
            1560,
            "inclusive upper bound",
        ),
        (
            "sound-speed-just-over",
            1561,
            1430,
            1560,
            "one past upper bound",
        ),
        ("freq-ok", 50, 1, 500, "sonar frequency kHz ∈ [1,500]"),
        ("depth-ok", 50, 0, 200, "check_depth_pressure(50, 200)"),
        ("depth-negative", -1, 0, 200, "mini test_depth_negative"),
    ];
    for (name, input, lo, hi, note) in maritime {
        c.push(case(
            name,
            Origin::Maritime,
            *input,
            &[
                Op::Input,
                Op::RangeI { lo: *lo, hi: *hi },
                Op::Assert,
                Op::Halt,
            ],
            note,
        ));
    }

    // ---- Plainsong [Perf] velocity envelopes (cell-cascade guards) ------
    for vel in [0i64, 1, 2, 63, 64, 65, 126, 127, 128, 255] {
        let expect = (VEL_LO..=VEL_HI).contains(&vel);
        c.push(case(
            &format!("vel-{}-{}", vel, if expect { "in" } else { "out" }),
            Origin::Plainsong,
            vel,
            &[
                Op::Input,
                Op::RangeI {
                    lo: VEL_LO,
                    hi: VEL_HI,
                },
                Op::Assert,
                Op::Halt,
            ],
            "plainsong [Perf] velocity bound: vel ∈ [1,127]",
        ));
    }
    for vstd in [112i64, 113, 156, 257, 258] {
        c.push(case(
            &format!("velstd-milli-{}", vstd),
            Origin::Plainsong,
            vstd,
            &[
                Op::Input,
                Op::RangeI {
                    lo: VSTD_LO,
                    hi: VSTD_HI,
                },
                Op::Assert,
                Op::Halt,
            ],
            "plainsong velocity_std ×1000 ∈ [113,257] (0.113–0.257)",
        ));
    }
    // Combined guard: cue/alarm cell fires iff vel AND vel_std both in range.
    c.push(case(
        "guard-combined-pass",
        Origin::Plainsong,
        156,
        &[
            Op::PushI(64),
            Op::RangeI {
                lo: VEL_LO,
                hi: VEL_HI,
            },
            Op::Input,
            Op::RangeI {
                lo: VSTD_LO,
                hi: VSTD_HI,
            },
            Op::And,
            Op::Assert,
            Op::Halt,
        ],
        "cell-cascade guard: vel=64 ∧ velstd=156 → fires",
    ));
    c.push(case(
        "guard-combined-fail",
        Origin::Plainsong,
        300,
        &[
            Op::PushI(64),
            Op::RangeI {
                lo: VEL_LO,
                hi: VEL_HI,
            },
            Op::Input,
            Op::RangeI {
                lo: VSTD_LO,
                hi: VSTD_HI,
            },
            Op::And,
            Op::Assert,
            Op::Halt,
        ],
        "cell-cascade guard: vel ok ∧ velstd=300 out → no fire",
    ));
    c.push(case(
        "guard-boundary-both-edges",
        Origin::Plainsong,
        113,
        &[
            Op::PushI(127),
            Op::RangeI {
                lo: VEL_LO,
                hi: VEL_HI,
            },
            Op::Input,
            Op::RangeI {
                lo: VSTD_LO,
                hi: VSTD_HI,
            },
            Op::And,
            Op::Assert,
            Op::Halt,
        ],
        "both predicates at their inclusive bounds (127, 113)",
    ));

    // ---- Harness probes (pin operand order everywhere) ------------------
    c.push(case(
        "sub-order-10-3",
        Origin::Harness,
        0,
        &[Op::PushI(10), Op::PushI(3), Op::Sub, Op::Assert, Op::Halt],
        "10−3=7 (non-zero) pins pop order: deeper−top",
    ));
    c.push(case(
        "lt-order-3-10-true",
        Origin::Harness,
        0,
        &[Op::PushI(3), Op::PushI(10), Op::Lt, Op::Assert, Op::Halt],
        "3<10 → 1",
    ));
    c.push(case(
        "lt-order-10-3-false",
        Origin::Harness,
        0,
        &[Op::PushI(10), Op::PushI(3), Op::Lt, Op::Assert, Op::Halt],
        "10<3 → 0 → assert fails",
    ));
    c.push(case(
        "mul-15-15-in-u8",
        Origin::Harness,
        0,
        &[
            Op::PushI(15),
            Op::PushI(15),
            Op::Mul,
            Op::RangeI { lo: 0, hi: 255 },
            Op::Assert,
            Op::Halt,
        ],
        "225 fits every value domain; range on computed value",
    ));

    c
}
