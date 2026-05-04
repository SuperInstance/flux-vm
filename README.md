# FLUX-VM — FLUX-C Constraint Virtual Machine

Stack-based virtual machine for the FLUX-C constraint language.

## Overview

| Component | Description |
|-----------|-------------|
| **FLUX-C VM** | 50 opcodes, stack-based, DAL A certifiable |
| **FLUX-X Bridge** | TrustZone-style bridge between FLUX-C (50 opcode, DAL A) and FLUX-X (247 opcode) |
| **Universal AST** | 7 node types, shared across all FLUX compilers |
| **ISA Crates** | Modular ISA definitions: mini, std, edge, thor (benchmark) |

## Architecture

```
FLUX-C Source → Universal AST → FLUX-C VM (50 opcodes, DAL A)
                                ↕ TrustZone Bridge
                              FLUX-X VM (247 opcodes)
```

## Components

- `vm/` — Core FLUX-C virtual machine (50 opcodes, 55 tests)
- `bridge/` — FLUX-C↔FLUX-X TrustZone bridge (`flux_bridge.rs` + `flux_c_to_x.py`)
- `flux-ast/` — Universal AST crate (7 node types)
- `flux-isa/` — ISA crate definitions
- `flux-isa-mini/` — Minimal ISA
- `flux-isa-std/` — Standard ISA
- `flux-isa-edge/` — Edge ISA
- `flux-isa-thor/` — Benchmark ISA crate
- `tests/` — End-to-end pipeline tests, fleet integration tests, multi-compiler tests

## License

Apache-2.0
