# FLUX VM — Stack-Based Constraint Checking Virtual Machine

> **FLUX — Fluid Language Universal eXecution**
> A gas-bounded stack machine for formal constraint validation, runtime policy enforcement, and bounded formal verification.

[![CI](https://github.com/SuperInstance/flux-vm/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/flux-vm/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## Overview

FLUX VM is a minimal, stack-only virtual machine designed exclusively for formal constraint validation, runtime policy enforcement, and bounded formal verification. Unlike general-purpose VMs such as WASM or Lua, it ships with exactly 50 standardized opcodes grouped into 9 functional categories, with no dynamic memory allocation, unbounded loops, or side effects outside its fixed stack frame. It is purpose-built for use cases where strict safety, determinism, and computable worst-case execution time (WCET) are non-negotiable: zero-knowledge proof constraint checking, embedded system policy enforcement, and smart contract input validation.

## Opcodes
All opcodes use standard stack effect notation, grouped by functional category:

| Mnemonic | Description | Category | Stack Effect |
|---------|-------------|----------|--------------|
| **Stack Operations** | | | |
| `PUSH(n)` | Push 64-bit integer literal `n` to stack | Stack | → `[n]` |
| `POP` | Remove top stack element | Stack | `[a]` → ∅ |
| `DUP` | Duplicate top stack element | Stack | `[a]` → `[a, a]` |
| `SWAP` | Swap top two stack elements | Stack | `[a, b]` → `[b, a]` |
| `OVER` | Copy second-to-top element to top | Stack | `[a, b]` → `[a, b, a]` |
| `ROT` | Rotate top three stack elements left | Stack | `[a, b, c]` → `[b, c, a]` |
| `CLEAR` | Empty entire stack | Stack | `[any...]` → ∅ |
| `PEEK(n)` | Copy `n`th stack element (0 = top) | Stack | `[..., x]` → `[..., x, x]` |
| `DEPTH` | Push current stack depth to top | Stack | → `[d]` |
| `NOP` | No-operation | Stack | ∅ → ∅ |
| **Arithmetic Operations** | | | |
| `ADD` | Pop `a, b`, push `a + b` | Arithmetic | `[a, b]` → `[a+b]` |
| `SUB` | Pop `a, b`, push `a - b` | Arithmetic | `[a, b]` → `[a-b]` |
| `MUL` | Pop `a, b`, push `a * b` | Arithmetic | `[a, b]` → `[a*b]` |
| `DIV` | Pop `a, b`, push `a // b` (signed) | Arithmetic | `[a, b]` → `[a//b]` |
| `MOD` | Pop `a, b`, push `a % b` (signed remainder) | Arithmetic | `[a, b]` → `[a%b]` |
| `EXP` | Pop `a, b`, push `a^b` | Arithmetic | `[a, b]` → `[a^b]` |
| `NEG` | Pop `a`, push `-a` | Arithmetic | `[a]` → `[-a]` |
| `INC` | Pop `a`, push `a + 1` | Arithmetic | `[a]` → `[a+1]` |
| `DEC` | Pop `a`, push `a - 1` | Arithmetic | `[a]` → `[a-1]` |
| `ABS` | Pop `a`, push `|a|` | Arithmetic | `[a]` → `[|a|]` |
| **Comparison Operations** | | | |
| `EQ` | Pop `a, b`, push 1 if equal, 0 otherwise | Comparison | `[a, b]` → `[1/0]` |
| `NEQ` | Pop `a, b`, push 1 if not equal, 0 otherwise | Comparison | `[a, b]` → `[1/0]` |
| `LT` | Pop `a, b`, push 1 if `a < b`, 0 otherwise | Comparison | `[a, b]` → `[1/0]` |
| `GT` | Pop `a, b`, push 1 if `a > b`, 0 otherwise | Comparison | `[a, b]` → `[1/0]` |
| `LTE` | Pop `a, b`, push 1 if `a ≤ b`, 0 otherwise | Comparison | `[a, b]` → `[1/0]` |
| `GTE` | Pop `a, b`, push 1 if `a ≥ b`, 0 otherwise | Comparison | `[a, b]` → `[1/0]` |
| `ISZERO` | Pop `a`, push 1 if `a = 0`, 0 otherwise | Comparison | `[a]` → `[1/0]` |
| `WITHIN` | Pop `val, min, max`, push 1 if `min ≤ val ≤ max` | Comparison | `[val, min, max]` → `[1/0]` |
| **Range Operations** | | | |
| `SET_RANGE_MIN(n)` | Set global range lower bound to `n` | Range | ∅ → ∅ |
| `SET_RANGE_MAX(n)` | Set global range upper bound to `n` | Range | ∅ → ∅ |
| `CHECK_RANGE` | Pop `val`, trap if outside global range | Range | `[val]` → `[val]` |
| `CLEAR_RANGE` | Reset global range bounds | Range | ∅ → ∅ |
| `GET_RANGE_MIN` | Push current lower bound to stack | Range | → `[min]` |
| `GET_RANGE_MAX` | Push current upper bound to stack | Range | → `[max]` |
| **Domain Operations** | | | |
| `SET_DOMAIN(s)` | Define allowed value set of size `s` | Domain | ∅ → ∅ |
| `CHECK_DOMAIN` | Pop `val`, trap if not in allowed set | Domain | `[val]` → `[val]` |
| `IS_IN_DOMAIN` | Pop `val`, push 1 if in allowed set, 0 otherwise | Domain | `[val]` → `[1/0]` |
| `CLEAR_DOMAIN` | Reset allowed value set | Domain | ∅ → ∅ |
| **Logical Operations** | | | |
| `AND` | Pop `a, b`, push bitwise AND | Logical | `[a, b]` → `[a&b]` |
| `OR` | Pop `a, b`, push bitwise OR | Logical | `[a, b]` → `[a|b]` |
| `XOR` | Pop `a, b`, push bitwise XOR | Logical | `[a, b]` → `[a^b]` |
| `NOT` | Pop `a`, push bitwise NOT | Logical | `[a]` → `[~a]` |
| **Temporal Operations** | | | |
| `TIMESTAMP_PUSH` | Push current system timestamp to stack | Temporal | → `[ts]` |
| `TIME_COMPARE` | Pop `a, b`, push 1 if `a` precedes `b` | Temporal | `[a, b]` → `[1/0]` |
| `TIME_WINDOW_VALID` | Pop `start, end`, trap if current ts outside window | Temporal | `[start, end]` → `[ts]` |
| **Security Operations** | | | |
| `VERIFY_HASH(hash)` | Pop `data`, trap if hash mismatch | Security | `[data]` → `[data]` |
| `CHECK_SIGNATURE(pubkey)` | Pop `sig, msg`, trap if invalid | Security | `[sig, msg]` → `[sig, msg]` |
| `RESTRICT_EXEC(addr)` | Lock execution to opcode at `addr` | Security | ∅ → ∅ |
| **Control Operations** | | | |
| `JMP(addr)` | Jump to fixed opcode offset | Control | ∅ → ∅ |
| `HALT` | Halt and return stack top | Control | ∅ → ∅ |

## Safety Properties
flux-vm is engineered for strict, verifiable safety:
1.  **Turing-Incomplete**: No unbounded loops or dynamic recursion, with all control flow bounded by fixed offsets

---

## Implementation Details

The VM is split across three independent implementations:

**flux_runtime_arm.h (core VM) — 19 opcodes**
- Stack manipulation: NOP, DUP, SWAP
- Immediate loads: PUSH_I8, PUSH_I16, PUSH_I32
- Input: LOAD_INPUT
- Arithmetic: ADD, SUB, MUL
- Comparison: EQ, LT, GT
- Boolean: AND, OR, NOT
- Safety: ASSERT, CHECK_DOMAIN, RANGE, HALT

**flux_sat8_ops.h (saturation extension) — 8 opcodes**
- SAT8, SAT8_ADD, SAT8_SUB, SAT8_MUL, SAT8_NEG, SAT8_CHECK, SAT8_CHECK_M, SAT8_ERRMASK

**flux_monitor_arm.c (monitor VM) — 19 opcodes**
- Separate numbering scheme (0x00–0xFF range)
- Types: I32, F32, CHECKPOINT/REVERT for transactional safety
- Range checking, domain validation, checkpoint/revert

### Execution Model

Gas-based execution. Each opcode consumes gas. Mandatory `max_gas` parameter bounds all computation. No loops, no jumps, no recursion — straight-line bytecode only.

```c
while ((st.fault == 0U) && (st.pc < bc_len)) {
    consume_gas(&st, gas_cost);
    // dispatch to opcode handler
}
```

---

## Workspace Crates

| Crate | Description |
|-------|-------------|
| `flux-ast` | Universal Constraint AST — single source of truth for constraint semantics |
| `flux-isa` | Core Instruction Set Architecture — stack-based bytecode encoding |
| `flux-isa-mini` | Minimal `no_std` ISA for bare-metal microcontrollers (STM32, Cortex-M) |
| `flux-isa-std` | Standard ISA for embedded Linux (Raspberry Pi, Jetson Nano) |
| `flux-isa-edge` | Async ISA runtime for fleet edge nodes (Jetson Xavier/Orin) |
| `flux-isa-thor` | Heavyweight ISA for GPU-class edge (Jetson Thor / AGX Orin with CUDA) |

### ISA Variants

- **Mini** — Ultra-minimal for MCUs. ~20 opcodes, no allocations, `no_std`.
- **Standard** — Full 50-opcode ISA for embedded Linux with serde support.
- **Edge** — Async runtime with networking, PLATO sync, and sensor pipelines.
- **Thor** — GPU-class with batch CSP solving, fleet coordination, and axum WS.

---

## 📦 Related Packages

FLUX is implemented across multiple languages — same bytecode, different shells:

| Package | Language | Registry | Install |
|---------|----------|----------|---------|
| **[flux-vm](https://pypi.org/project/flux-vm/)** | Python | PyPI | `pip install flux-vm` |
| **[fluxvm](https://crates.io/crates/fluxvm)** | Rust | crates.io | `cargo add fluxvm` |
| **[flux-js](https://www.npmjs.com/package/flux-js)** | JavaScript | npm | `npm install flux-js` |
| **[flux-compiler](https://github.com/SuperInstance/flux-compiler)** | Rust/Python | GitHub | `cargo install flux-compiler` |

Additional implementations: [C](https://github.com/SuperInstance/flux-runtime-c) · [Zig](https://github.com/SuperInstance/flux-zig) · [Go](https://github.com/SuperInstance/flux-swarm) · [Java](https://github.com/SuperInstance/flux-java) · [WASM](https://github.com/SuperInstance/flux-wasm) · [CUDA](https://github.com/SuperInstance/flux-cuda)

## 🌐 Ecosystem

FLUX is part of a broader research ecosystem exploring agent-first computation:

| Project | Description |
|---------|-------------|
| [PLATO Engine Block](https://github.com/SuperInstance/plato-engine-block) | Constraint engine powering FLUX verification |
| [Constraint-Theory-Core](https://github.com/SuperInstance/Constraint-Theory) | Mathematical foundations for constraint-based computation |
| [AI-Writings](https://github.com/SuperInstance/AI-Writings) | Philosophy, essays, and design rationale behind FLUX |
| [Captain's Log](https://github.com/SuperInstance/captains-log) | Oracle1 growth diary and agent dojo curriculum |
| [Iron-to-Iron](https://github.com/SuperInstance/iron-to-iron) | I2I protocol — agents communicate through git commits |
| [flux-research](https://github.com/SuperInstance/flux-research) | 40K words: compiler taxonomy, ISA v2, agent-first design |

📖 **[Full package index →](https://github.com/SuperInstance/flux/blob/main/PACKAGES.md)**

---

## License

MIT — See [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome. Run `cargo test --workspace` before submitting PRs.

---

*Same bytecode, different shells.* 🦀

## Ecosystem

This repo is part of the **SuperInstance** flagship ecosystem — agent-first computation, constraint theory, and self-improving runtimes.

### FLUX Runtime Family

| Repo | Language | Description |
|------|----------|-------------|
| [flux-runtime](https://github.com/SuperInstance/flux-runtime) | Python | Full FLUX runtime: markdown→bytecode, 2037 tests, zero deps |
| [flux-core](https://github.com/SuperInstance/flux-core) | Rust | Register-based bytecode VM, deterministic agent computation |
| [flux-js](https://github.com/SuperInstance/flux-js) | JavaScript | FLUX VM for Node.js and browsers, ~400ns/iter |
| [flux-compiler](https://github.com/SuperInstance/flux-compiler) | Rust/Python | Formal-methods compiler for safety-critical codegen |
| [flux-vm](https://github.com/SuperInstance/flux-vm) | Rust | Stack-based constraint-checking VM, 50 opcodes, Turing-incomplete |

### PLATO Engine Family

| Repo | Language | Description |
|------|----------|-------------|
| [plato-server](https://github.com/SuperInstance/plato-server) | Python | Knowledge tiles, fleet sync via Matrix, HTTP API |
| [plato-engine-block](https://github.com/SuperInstance/plato-engine-block) | Rust | Original room runtime: no_std + alloc, builder pattern |
| [plato-engine-block-c](https://github.com/SuperInstance/plato-engine-block-c) | C99 | Embedded reference: zero heap alloc, bare-metal portable |
| [plato-engine-block-elixir](https://github.com/SuperInstance/plato-engine-block-elixir) | Elixir | BEAM supervision trees, fault tolerance, hot reload |
| [plato-runtime-kernel](https://github.com/SuperInstance/plato-runtime-kernel) | Rust | Spatial model: tensor grid, batons, assertion traps |

### Constraint / Theory Family

| Repo | Language | Description |
|------|----------|-------------|
| [categorical-agents](https://github.com/SuperInstance/categorical-agents) | Rust | Category theory for agent composition (functors, naturality) |
| [cuda-constraint-engine](https://github.com/SuperInstance/cuda-constraint-engine) | CUDA/C | GPU constraint checking at 1B+ constraints/sec |
| [grand-pattern-rs](https://github.com/SuperInstance/grand-pattern-rs) | Rust | Fibonacci dual-direction cellular graph architecture |
| [lau-hodge-theory](https://github.com/SuperInstance/lau-hodge-theory) | Rust | Hodge decomposition, Betti numbers, spectral sequences |
| [ternary-science](https://github.com/SuperInstance/ternary-science) | Rust | Experimental evidence for ternary intelligence, 5 conservation laws |

### Agent / Infrastructure Family

| Repo | Language | Description |
|------|----------|-------------|
| [construct-core](https://github.com/SuperInstance/construct-core) | Rust | Layered trait system: bare-metal → alloc → async agent runtime |
| [crab](https://github.com/SuperInstance/crab) | Bash | Agent shell for repo entry/leave (MUD-room metaphor) |
| [exocortex](https://github.com/SuperInstance/exocortex) | Rust | Persistent cognitive substrate, S3-compatible memory |
| [git-agent](https://github.com/SuperInstance/git-agent) | Python | The repo IS the agent — autonomous lifecycle via Git |
| [capitaine-1](https://github.com/SuperInstance/capitaine-1) | TypeScript | Git-native repo-agent, Cloudflare Workers heartbeat |
| [codespace-edge-rd](https://github.com/SuperInstance/codespace-edge-rd) | Research | Codespace→Edge agent lifecycle and yoke transfer protocols |
| [git-agent-codespace](https://github.com/SuperInstance/git-agent-codespace) | DevContainer | One-click Codespace template for Git-Agent runtimes |

### Registries

| Registry | Package | Install |
|----------|---------|---------|
| **PyPI** | `flux-vm` | `pip install flux-vm` |
| **crates.io** | `fluxvm` | `cargo add fluxvm` |
| **npm** | `flux-js` | `npm install flux-js` *(coming soon)* |

### Philosophy & Architecture

- 📖 [AI-Writings](https://github.com/SuperInstance/AI-Writings) — Philosophy, essays, and design rationale
- 📦 [PACKAGES.md](https://github.com/SuperInstance/SuperInstance/blob/main/PACKAGES.md) — Full package index
