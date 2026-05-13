# Flux VM

**The engine block. Cast once, run forever.**

---

## Fifty Opcodes. No Surprises.

Flux VM is a stack-based virtual machine with exactly 50 opcodes. Not 51. Not 49. Fifty.

No dynamic memory allocation. No unbounded loops. No side effects outside a fixed stack frame. Every instruction has a known worst-case execution time that fits on a safety certification form.

This is not a general-purpose VM. WASM runs anything. Lua runs anything. Flux VM does one thing: constraint verification. It receives a stack of constraints, processes them, and returns a verdict. That's the entire design.

## Why Not WASM?

WASM is a general-purpose target. 183 instructions. Dynamic memory. An unbounded stack. Several places with undefined behavior.

Flux VM has 50 instructions. Formally specified. Its entire state is a stack and a fixed number of registers. Every execution path terminates. Every opcode either preserves safety properties or traps.

You can reason about Flux VM in ways you cannot reason about WASM. That reasoning is what certification requires.

## Safety Properties

- **Turing-incomplete** — no unbounded loops, no recursion, all control flow bounded
- **WCET computable** — every opcode has a bounded execution time known before execution
- **Stack-bounded** — maximum stack depth is a static property of the program
- **No implicit state** — no hidden globals, no side channels, no timing attacks
- **Formal instruction set** — 50 opcodes specified in Coq, verified against spec

## Architecture

The TrustZone-style bridge between FLUX-C (50 opcodes) and FLUX-X (247 opcodes) provides a proper isolation hierarchy. The VM is the trusted computing base for safety-critical operations. What it verifies is the untrusted payload.

## License

Apache 2.0 — Cocapn fleet infrastructure.
