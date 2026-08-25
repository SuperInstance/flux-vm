# FLUX VM

A collection of stack-based virtual machines for bounded constraint checking,
built independently in C and Rust under the same "FLUX" name. They are not
one coherent product — see "What's actually here" below.

[![CI](https://github.com/SuperInstance/flux-vm/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/flux-vm/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**This repository audits itself.** [`VERIFICATION.md`](VERIFICATION.md) is an
internal review of an earlier version of this README against the actual code,
and it found real problems: an opcode count that didn't exist anywhere in
source, a certification claim with no supporting artifacts, a formal-methods
claim with no formal methods. This README is the rewrite that followed —
every number below was counted directly from the source in this repo, not
carried over from the old copy. The method behind that review is written up,
domain-neutrally, in [`docs/verification.md`](docs/verification.md); it's
worth reading regardless of what you think of this project, because the
failure mode it describes ("a claim outran the thing that was supposed to
check it, and nothing noticed") is not specific to FLUX.

## What's actually here

There is no single FLUX VM. This repository contains **several independent
implementations that share a name and a rough idea** (a small stack machine
for checking numeric constraints against bounds) but do not share bytecode,
opcode numbering, or a build system:

| Implementation | Language | Where | Part of `cargo build --workspace`? |
|---|---|---|---|
| Core runtime | C | `src/flux_runtime_arm.c` / `flux_runtime_arm.h` | No — not a Cargo crate |
| SAT8 saturation extension | C | `src/flux_sat8_ops.h` | No |
| Runtime monitor | C | `src/flux_monitor_arm.c` | No |
| `flux-isa` | Rust | `flux-isa/` | Yes |
| `flux-isa-mini` | Rust | `flux-isa-mini/` | Yes |
| `flux-isa-std` | Rust | `flux-isa-std/` | Yes |
| `flux-isa-edge` | Rust | `flux-isa-edge/` | Yes |
| `flux-isa-thor` | Rust | `flux-isa-thor/` | Yes |
| `flux-ast` | Rust | `flux-ast/` | Yes |
| FLUX-C → FLUX-X bridge | Python | `bridge/flux_c_to_x.py` | No (Python, runs standalone) |
| A `.rs` file that is actually a design doc | — | `docs/flux-x-bridge-design.md` | No — see below |

The Rust crates above are one Cargo workspace and do build and test together
(details under "Build and test"). The C files and the top-level `vm/` and
`tests/*.rs` files are **not** referenced by any `Cargo.toml` and are not
built by CI. `.github/workflows/ci.yml` runs `cargo check`, `cargo test`,
`cargo clippy`, and `cargo fmt` — all scoped to `--workspace`, i.e. the six
Rust crates only. Nothing in CI compiles the C sources or runs the Python
scripts.

## Opcodes — actual counts, per implementation

The old README claimed "exactly 50 standardized opcodes." That number
appears nowhere in the source. Every implementation in this repo defines its
own opcode set, and most of them disagree with each other about what a given
byte value means. Counts below were obtained by counting `#define`/`enum`
opcode entries directly (see the file for each):

| Implementation | File | Opcode count | Notes |
|---|---|---|---|
| Core runtime | `src/flux_runtime_arm.h` | 20 | `#define FLUX_OP_*` |
| SAT8 extension | `src/flux_sat8_ops.h` | 8 | Extends the core numbering (opcodes `0x30`–`0x37`, no collisions) — core + SAT8 = **28** opcodes in one coherent scheme |
| Runtime monitor | `src/flux_monitor_arm.c` | 21 | Its own numbering scheme, incompatible with the core runtime: e.g. opcode `0x01` is `PUSH_I8` in the core but `PUSH_I32` in the monitor |
| `flux-isa` | `flux-isa/src/opcode.rs` | 37 | |
| `flux-isa-mini` | `flux-isa-mini/src/opcode.rs` | 21 | Header comment says "21 essential operations stripped from the full 35" — matches its own count, not the "full 35" it refers to |
| `flux-isa-std` | `flux-isa-std/src/opcode.rs` | 37 | Header comment says "All 35 FLUX ISA opcodes" — the comment is wrong; the enum has 37 variants |
| `flux-isa-edge` | `flux-isa-edge/src/opcode.rs` | 34 | Header comment says "All 35" — actual count is 34 |
| `flux-isa-thor` | `flux-isa-thor/src/opcode.rs` | 35 base + 8 Thor-specific = 43 | The only count here that's asserted by a test (`opcode::tests::base_count`, `opcode::tests::thor_count`) rather than just a comment |
| FLUX-C → FLUX-X bridge map | `bridge/flux_c_to_x.py` | 29 | `OPCODE_MAP` entries. The module docstring separately claims FLUX-X has "247 opcodes" — that number is not backed by any opcode list in this repo |

None of these is 50. No two of the C implementations use the same numbering
for the same byte, so "the FLUX-C opcode set" is not a single well-defined
thing even within this repo — it's whichever of three files you happen to be
reading. The 50-row opcode table that used to be in this README (`PUSH`,
`POP`, `ROT`, `JMP`, `WITHIN`, `VERIFY_HASH`, `TIME_WINDOW_VALID`, `SET_DOMAIN`,
and about a dozen others) does not correspond to any implementation in this
repo — most of those mnemonics do not appear anywhere in the source. It has
been removed rather than fixed, because fixing it would mean inventing a
50-opcode ISA that doesn't exist. If one gets built, it belongs here with a
count a test actually asserts, the way `flux-isa-thor` already does it.

## Safety properties

**"Turing-incomplete" is true of the C core runtime, and only there.**
`src/flux_runtime_arm.c`'s dispatch loop is bounded by gas and by program
counter, and the ISA it interprets has no jump, call, or loop opcode at all —
a program of N instructions can execute at most N steps, structurally, not
just by convention:

```c
while ((st.fault == 0U) && (st.pc < bc_len)) {
    consume_gas(&st, gas_cost);
    /* dispatch to opcode handler */
}
```

That is not true of every implementation in this repo. `flux-isa-std`
(`flux-isa-std/src/vm.rs`) implements `Jmp`, `Call`, and `Ret` opcodes, and
`Jmp` can set the instruction pointer backward — it is a real loop construct.
Execution there is bounded the ordinary way sandboxed VMs are bounded (a
configurable `max_instructions` counter, default 1,000,000), not by the
absence of a loop opcode. Don't read "Turing-incomplete" as a blanket
property of "FLUX VM" — it's a property of one specific C implementation.

## Build and test

Requires a Rust toolchain for the workspace crates; a C compiler and Python 3
for the pieces outside it.

```bash
# The Cargo workspace (flux-ast, flux-isa, flux-isa-mini, flux-isa-std,
# flux-isa-edge, flux-isa-thor) — this is what CI runs.
cargo build --workspace
cargo test --workspace
```

Verified in this repo: `cargo build --workspace` completes with warnings
only (unused imports/fields, no errors), and `cargo test --workspace` passes
83 tests across the six crates, 0 failed.

```bash
# The Python pieces, outside the Cargo workspace and outside CI.
python3 -m pytest tests/                # 12 passed
python3 src/maritime_constraints.py     # standalone demo, runs directly
python3 bridge/flux_c_to_x.py           # standalone demo, runs directly
```

The C core runtime (`src/flux_runtime_arm.c`, `src/flux_runtime_arm.h`) now
compiles cleanly as a plain translation unit:

```bash
gcc -c -std=c11 -Wall -Wextra -Werror -Isrc src/flux_runtime_arm.c
```

It did not, until recently, and it is worth recording why. The header declared
`uint16_t`/`int32_t`/`uint8_t` without including `<stdint.h>`, and the `.c` file
included the header before `<stdint.h>` rather than after — so the first
inclusion failed with "unknown type name" and the file never built at all.
Making the header self-contained then exposed a second error underneath it: a
duplicated pair of declarations in `flux_check`, which nothing had ever reached
because the build died earlier.

Neither was going to be found by reading, and CI could not find them either —
it ran `cargo check`, `cargo test`, `clippy` and `fmt`, all Rust, while the C
sat unbuilt. There is now a `c` job that compiles every `.c` with
`-Wall -Wextra -Werror` and compiles every header on its own, so a header that
depends on being included second fails the build rather than waiting for the
next person.

It is still written and commented for `arm-none-eabi-gcc -mcpu=cortex-r5
-mthumb`, and building for *that* target from this repo remains unverified.
`src/flux_monitor_arm.c` compiles standalone and always did.

`test_sat8` is a prebuilt binary, x86-64 only (confirmed with `file`), not
buildable from source in this repo. It runs and passes 5 tests on an x86-64
host; it will not execute on an ARM host despite the C sources being written
for ARM targets — the binary and the source it's presumably built from are
for different architectures.

## Workspace crates

| Crate | What it is |
|---|---|
| `flux-ast` | Constraint AST types shared across the other crates |
| `flux-isa` | A stack-based bytecode ISA and encoder |
| `flux-isa-mini` | A `no_std` subset ISA (21 opcodes) aimed at bare-metal microcontroller targets |
| `flux-isa-std` | An ISA with `serde` support and a VM that includes jump/call/return (see "Safety properties" above) |
| `flux-isa-edge` | An async ISA runtime with a PLATO sync client and a sensor pipeline module |
| `flux-isa-thor` | An ISA with a batch constraint solver and an `axum`-based WebSocket server; its "GPU" solve path is a CPU fallback with simulated GPU timing — the code comment for it reads `// Production: FFI to libflux_cuda.so` / `// For now: CPU fallback with GPU timing simulation`. No CUDA FFI is implemented in this repo. |

All six build and their own test suites pass under `cargo test --workspace`
(see counts above). That's what "part of the workspace" verifies — it does
not mean every described capability (PLATO sync against a real server, GPU
dispatch, fleet coordination) has been exercised against the real thing
rather than against its own test doubles; check each crate's tests before
relying on a specific capability.

## The FLUX-C / FLUX-X bridge

`bridge/flux_c_to_x.py` is a real, standalone Python script. It converts
FLUX-C style variable-length bytecode into a fixed 4-byte instruction format
it calls FLUX-X, using an opcode table (`OPCODE_MAP`) of 29 entries. Run it
directly — `python3 bridge/flux_c_to_x.py` — and it prints a worked example.

There used to be a second file, `bridge/flux_bridge.rs`, that read as a Rust
security bridge between "FLUX-X" and "FLUX-C" with comments describing it as
following "ARM TrustZone SMC principles." It is not Rust: it's prose with
Markdown headers and code fences (` ### `, ` #### Cargo.toml `, ` ```rust `)
pasted directly into a `.rs` file, it was never a member of the Cargo
workspace, and it does not compile. It has been renamed and moved to
[`docs/flux-x-bridge-design.md`](docs/flux-x-bridge-design.md) so it stops
presenting as buildable source. Treat it as a design sketch, not as code.

There is no ARM TrustZone code anywhere in this repository — no secure-world
switch, no SMC instructions, nothing that runs in or talks to a TrustZone
secure world. The phrase describes an intended security model in the design
doc, not something implemented here.

## Not built yet

These appear in comments or in the design doc, but nothing in this repo
implements them. Listed here so they're visible as intentions rather than
silently dropped or, worse, silently implied to exist:

- **DO-178C DAL A certification.** One comment in `src/flux_sat8_ops.h`
  describes the code as "Safe for DO-178C DAL A certification path." There
  are no certification artifacts in this repo — no compliance matrix, no
  software development plan, no requirements traceability, nothing filed
  with a certification authority. If that work starts, it belongs in its
  own directory with its own paper trail.
- **A Coq formalisation.** No `.v` files exist anywhere in this repository
  (`find . -name "*.v"` returns nothing). Comments in `src/flux_sat8_ops.h`
  reference a file, `flux_saturation_coq.v`, that is not present.
- **A TrustZone bridge to a 247-opcode "FLUX-X" ISA**, in the sense the old
  README implied — a real secure-world bridge with a real 247-entry opcode
  table. See "The FLUX-C / FLUX-X bridge" above for what actually exists: a
  working but much smaller (29-entry) Python conversion table, and an
  unbuilt Rust design sketch.
- **GPU execution for `flux-isa-thor`.** The solver falls back to CPU with
  simulated timing; the CUDA FFI path is a comment, not code.
- **A single 50-opcode ISA that all of the above agree on.** See "Opcodes"
  above.

## License

Apache License 2.0 — see [LICENSE](LICENSE) for the full text. `Cargo.toml`
also declares `license = "Apache-2.0"` for the workspace crates. (An earlier
version of this README and its badge said MIT; that was wrong — the
`LICENSE` file has always been Apache-2.0, and the license file is what
governs.)

## Contributing

`cargo test --workspace` is what CI runs and what a PR touching the Rust
crates should pass locally first. If you're touching the C sources or the
Python scripts, run the commands under "Build and test" above directly —
they aren't wired into CI, so nothing else will catch a break.

## Related repositories

Several other repositories under the same GitHub org (`SuperInstance`) use
the "FLUX" or "PLATO" name for related but separate projects — different
codebases, different languages, not audited as part of this review and not
verified from within this repository. If you're evaluating one of them,
apply the same standard: read the code, don't take the README's word for it.
