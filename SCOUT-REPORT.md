# SCOUT REPORT — flux-vm

**Scouted:** 2026-08-25 21:00–21:20 AKDT (lane-fluxscout)
**Repo:** SuperInstance/flux-vm — cloned to `/home/eileen/projects/flux-vm` (was not local)
**Head:** `7027fe5` (merge PR #3), latest pushes today 16:26–16:44Z + follow-up branch tip 21:31Z
**Method:** direct source read + claude/sonnet cross-assessment in tmux `fleet:lane-fluxscout`; both factual anchors verified against source by the scout lane.

---

## What flux-vm is (one paragraph)

flux-vm is **not one VM** — it's a family of small stack-based virtual machines for **bounded numeric-constraint checking** ("is this sensor value within domain D?"), built independently in C and Rust under one name: a hardened C core written for ARM Cortex-R5 (gas-bounded, **loop-free ISA** — no JMP/CALL, N instructions can execute at most N steps, structurally), a SAT8 saturating-int8 extension (opcodes 0x30–0x37), and six Cargo-workspace Rust crates (`flux-ast`, `flux-isa`, `-mini`, `-std`, `-edge`, `-thor`) that reimplement the idea with **mutually incompatible opcode numbering**. The domain is real and specific: **commercial fishing vessel sonar safety** — Mackenzie-1981 sound speed, Francois-Garrison absorption, maritime range constraints. The repo is also a methodological exhibit: `VERIFICATION.md` is a self-audit that refuted 3 of 5 of its own README's headline claims (50 opcodes, DAL A certification, Coq proofs, TrustZone bridge — none exist), and today's commits were a hardening pass (C core now compiles, clippy/fmt clean for the first time, CI now builds every `.c` with `-Werror`).

## Maturity

| Check | Status |
|---|---|
| Rust workspace | ✅ builds; 83 tests pass, 0 fail (CI: check/test/clippy/fmt) |
| C core | ✅ compiles `-std=c11 -Wall -Wextra -Werror` (fixed today); Cortex-R5 cross-build **unverified** |
| Python | ✅ 12 pytest pass; bridge + maritime demo run standalone |
| clippy/fmt | ✅ clean **as of today** (first time, toolchain pinned) |
| C in CI | ✅ as of today (new `c` job) |
| ⚠️ Gaps | `flux_sat8_ops.inc` is an orphan fragment (calls undefined `flux_pop/push`); `test_sat8` is a prebuilt x86-64 binary, not buildable from source; thor "GPU" is a comment + CPU fallback; unbuilt claims list in README (Coq, TrustZone, DAL A, 50-op ISA, CUDA) |

## The flux-* family (name-sharing, separate codebases)

- **flux-runtime** — "Fluid Language Universal eXecution," Python, markdown→bytecode, 2037 tests, **published on PyPI as `flux-vm`** (⚠️ package-name collision with this repo's name)
- **flux-cross-assembler** — dual-target assembler for the ISA-v3 spec (cloud 4-byte / edge variable-width) — the *assembler* layer flux-vm lacks
- **flux-dsh-plugin** — mounts flux-runtime into DeepSeek Harness as one tool (embassy pattern, SEAM-REPORT.md)
- **flux-genome-rs** — genetic algorithm for musical traditions; unrelated domain, shares only the name

## Relationship to quilt-vm — the overlap verdict

**Sibling niche, not redundant — and complementary by design.**

- **quilt-vm** (5 opcodes: BIND/LINK/EFFECT/VIEW/TICK) = **graph topology**: things and relations, no arithmetic, no bound. It's the substrate that *organizes*.
- **flux-vm** (20–43 opcodes: PUSH/ADD/LT/RANGE/ASSERT/SAT8) = **numeric predicates**: bounded scalar evaluation with a hard step-count guarantee. It's a **leaf-node verifier** the graph could *call*.

**Conformance/seam issue: yes, and it's worse than the polyformal case.** Byte `0x01` is `PUSH_I8` in the C core, `PUSH_I32` in the C monitor, and `Add` in `flux-isa-mini` — three meanings, all passing their own tests, inside one repo. The polyformal suite tests the *same* wire format across Rust/C/Zig/Python/CUDA; flux-vm has **different wire formats within a single `cargo build --workspace`**, and only `flux-isa-thor` asserts its opcode counts by test (the model to copy — the README says so itself). This is the polyformal failure mode internalized: uncaught by CI because every implementation is only ever tested against itself.

## Synergy with today's work — 3 opportunities, ranked

1. **Cell-cascade tissue ISA** — flux-vm as a compilation target for `.fbc` constraint programs serving as **computed guards for sclerotic cells** (alarm clocks, cue firers) — the same seam qm-bridge opened (.qm → real quilt-vm-rust). Today's sclerotic rule tables do canonical-JSON first-match; a bounded flux-vm predicate would give them a *typed, verifiable* guard language with a structural no-runaway guarantee (fitting the stem-cell doctrine: cheap specific tissue). **Highest payoff; gated on a stable opcode scheme.**
2. **Conformance-harness pilot using plainsong [Perf] envelopes** — perf-v1's velocity_std bounds (0.113→0.257) are exactly the RANGE-opcode shape: a real, already-measured numeric envelope. Use it (plus the maritime corpus already in-repo) as the **golden corpus** for a per-implementation conformance runner — the minimal fix for the 0x01 divergence, and the prerequisite for opportunity 1. **Best effort-to-value; do this first.**
3. **Boat lane (F/V EILEEN)** — flux-vm's actual domain. The C core targets Cortex-R5, edge crate targets A53/A72, sonar physics is real; pairs with the Liquid-LFM2.5 local boat brain for a no-cloud offshore safety layer (60mi out). **Not a today-task; it's the reason this repo deserves the conformance investment.**

(Yard-band drift envelopes were assessed and **deprioritized**: the 1ms-drift soak is already deterministic schedule arithmetic; routing it through a VM adds dispatch overhead for no correctness gain.)

## Claude's blunt close (endorsed by scout)

> flux-vm isn't ready to be infrastructure for anything yet. Fix the opcode-scheme fragmentation first, pilot on the [Perf]/maritime golden corpus, and only then consider wiring it into cell-cascade's guard path.

Full claude transcript: `/tmp/fluxscout-claude.txt`.
