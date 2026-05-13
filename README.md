# FLUX VM

## What It Is

FLUX VM is a gas-bounded stack machine VM for constraint verification. Not general-purpose.

### Verified Properties

| Property | Status |
|----------|--------|
| Turing-incomplete | ✅ Confirmed — gas-bounded dispatch, no loops/recursion |
| Bounded WCET | ✅ Each opcode has a fixed gas cost; dispatch loop terminates when gas exhausts or pc reaches bytecode end |

### Opcode Architecture

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

**Total across all implementations: ~46 opcode definitions**
(These are from 3 separate VMs, not one coherent ISA. They do not share opcode numbering.)

### Execution Model

Gas-based execution. Each opcode consumes gas. Mandatory `max_gas` parameter bounds all computation. No loops, no jumps, no recursion — straight-line bytecode only.

```c
while ((st.fault == 0U) && (st.pc < bc_len)) {
    consume_gas(&st, gas_cost);
    // dispatch to opcode handler
}
```

### Implementation Details

- **Language:** C (MISRA-C 2012 compliant, comments say)
- **Targets:** ARM architecture
- **Computed goto** dispatch for opcode routing
- **INT8 saturation** arithmetic in sat8 extension
- **Transactional checkpoints** in monitor VM

### What This Is NOT

- Not a general-purpose VM
- No Coq formal specifications (zero .v files in repo)
- No DAL A certification artifacts
- No TrustZone bridge to FLUX-X (those don't exist)
- Not 50 opcodes — that's a marketing number from the old README

### What Needs to Be Built

- Coherent opcode ISA with defined numbering across all ops
- Formal specification (Coq or similar)
- Certification documentation (DO-178C DAL A path is aspirational in comments)
- TrustZone bridge (planned? no implementation exists)
- Active test suite (current test binary is x86-64, not runnable on ARM)

## License

Apache 2.0 — Cocapn fleet infrastructure.