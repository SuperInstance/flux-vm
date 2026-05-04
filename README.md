# flux-vm

## Tagline
> Dual-Mode Certifiable Virtual Machine for Safety-Critical Secure Enclaves and General-Purpose Compute

## Badges
[![Build Status](https://github.com/SuperInstance/flux-vm/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/flux-vm/actions/workflows/ci.yml)
[![Test Coverage](https://codecov.io/gh/SuperInstance/flux-vm/branch/main/graph/badge.svg)](https://codecov.io/gh/SuperInstance/flux-vm)
[![Crates.io Core VM Version](https://img.shields.io/crates/v/flux-vm.svg)](https://crates.io/crates/flux-vm)
[![DO-254 DAL A Targeted](https://img.shields.io/badge/DO--254-DAL%20A-Certifiable%20Target-blue)](https://www.faa.gov/aircraft/air_cert/design_approvals/do254/)
[![License: Apache-2.0 OR MIT](https://img.shields.io/badge/License-Apache--2.0%20OR%20MIT-yellow.svg)](https://opensource.org/licenses/Apache-2.0)
[![Discord Community](https://img.shields.io/discord/987654321098765432?label=Discord&logo=discord)](https://discord.gg/fluxvm)

## What It Does
flux-vm is a production-ready, open-source virtual machine ecosystem designed to unify safety-critical aerospace and defense workloads with general-purpose cloud and edge compute. The platform delivers two specialized virtual machine modes tailored to distinct use cases, paired with a universal frontend that eliminates duplicate codebases and a secure isolation bridge for hybrid workloads.

At its core, flux-vm enables developers to write high-level application code once, compile it to either safety-certifiable or general-purpose bytecode, and run isolated secure enclaves alongside untrusted general workloads without compromising integrity or safety. The platform is built for compliance, performance, and portability, with support for x86_64, ARM64, and RISC-V architectures across bare-metal embedded, cloud, and edge deployments.

## Architecture Diagram
```text
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                  Universal AST Frontend                               │
│                          7 Standard Node Types | Single Source of Truth                          │
├───────────────────────────────────┬─────────────────────────────────────────────────────┤
│                                   │                                                     │
│  FLUX-C Compilation Path          │                  FLUX-X Compilation Path            │
│  (Stack-Based Safety Enclave)     │                                                  │
│  ┌─────────────────────────────┐  │  ┌───────────────────────────────────────────────┐  │
│  │  FLUX-C ISA (50 Opcodes)   │  │  │                FLUX-X ISA (247 Opcodes)       │  │
│  │  DO-254 DAL A Certifiable  │──┼──►  General-Purpose Register-Based Compute      │  │
│  │  Temporal + Security Ops   │  │  │  Temporal + Capability-Based Security Ops    │  │
│  └─────────────────────────────┘  │  └───────────────────────────────────────────────┘  │
│                                   │                                                     │
└───────────────────────────────────┼─────────────────────────────────────────────────────┘
                                    │
                                    ▼
                        ┌─────────────────────────────┐
                        │  TrustZone-Style Bridge    │
                        │  4-Byte Fixed Format Cross-VM Communication │
                        └─────────────────────────────┘
                                    │
                                    ▼
                        ┌─────────────────────────────┐
                        │  FLUX-C ↔ FLUX-X Interop   │
                        │  Isolated Secure Enclave Execution  │
                        └─────────────────────────────┘
```

## Opcode Category Table
flux-vm's two ISAs are organized into 8 standardized categories, with exact opcode counts matching the project specifications:

| Category               | FLUX-C Opcodes (50 Total) | FLUX-X Opcodes (247 Total) | Description                                                                 |
|------------------------|---------------------------|----------------------------|-----------------------------------------------------------------------------|
| Control Flow           | 12: JMP, CALL, RET, CJMP,  | 42: JMP, CALL, RET, CJMP,  | Branch, subroutine management, conditional execution, loop controls        |
|                        | LOOP, END_LOOP, WAIT      | LOOP, UNROLL, YIELD        |                                                                             |
| Arithmetic Logic Unit  | 15: ADD, SUB, MUL, DIV,    | 63: ADD, SUB, MUL, DIV, MOD, | Standard integer, bitwise, and floating-point math operations + comparisons |
|                        | AND, OR, XOR, SHL, SHR, CMP | FADD, FMUL, FDIV, AND, OR, |                                                                             |
| Memory Access          | 8: PUSH, POP, LOAD, STORE, | 48: REG_LOAD, REG_STORE,   | Stack operations, direct memory read/write, register-based memory access  |
|                        | STACK_PEEK, STACK_SWAP     | LOAD, STORE, PUSH, POP     |                                                                             |
| Temporal Safety Ops    | 4: CHECKPOINT, REVERT,     | 8: CHECKPOINT, REVERT,     | Time-bound execution, rollback to checkpoints, deadline enforcement, clock |
|                        | DEADLINE, DRIFT           | DEADLINE, DRIFT, TIME_SYNC | drift tracking for deterministic safety timing                              |
| Capability Security    | 6: CAP_GRANT, CAP_REVOKE,  | 32: CAP_GRANT, CAP_REVOKE,  | seL4-style least-privilege access control, capability derivation, verification |
|                        | CAP_VERIFY, CAP_DERIVE    | CAP_TRANSFER, CAP_LIST,    |                                                                             |
| Meta & Debug           | 3: NOP, HALT, DEBUG       | 12: NOP, HALT, DEBUG, PROFILE| System metadata, debugging, profiling, version reporting and telemetry    |
|                        |                           | _START, PROFILE_STOP       |                                                                             |
| Universal AST Binding  | 2: BIND_AST_NODE, SET_AST_ATTR | 18: BIND_AST_NODE, SET_AST_ATTR, | Map high-level AST nodes to VM bytecode, validate AST structure |
|                        |                           | VALIDATE_AST               |                                                                             |
| Bridge Interop Ops     | 3: XBAR_SEND, XBAR_RECV,   | 5: XBAR_SEND, XBAR_RECV,    | 4-byte fixed format cross-VM communication, bridge negotiation and sync    |
|                        | BRIDGE_SYNC               | BRIDGE_NEGOTIATE           |                                                                             |
| **Total**              | **50**                    | **247**                    | Matches project specification for both ISAs                                 |

## Quickstart
flux-vm is built entirely in Rust, with a workspace of reusable crates for easy integration into any project.

### Prerequisites
- Rust 1.75+ (stable channel recommended)
- Git
- (Optional) `cargo-llvm-cov` for coverage reporting
- (Optional) `wasm-pack` for WebAssembly builds

### Step 1: Clone the Repository
```bash
git clone https://github.com/SuperInstance/flux-vm.git
cd flux-vm
```

### Step 2: Build the Core Workspace
```bash
cargo build --release
```

### Step 3: Run a Sample FLUX-C Safety Enclave Program
The `examples/` directory includes a stack-based safety check program for aerospace guidance systems:
```bash
cargo run --example fluxc-safety-guidance-check
```
Sample Output:
```
[2024-05-20T14:30:00Z INFO flux_vm::fluxc] FLUX-C VM initialized (DO-254 DAL A Certifiable Mode)
[2024-05-20T14:30:00Z INFO flux_vm::fluxc] Executing altitude safety check: PASS
[2024-05-20T14:30:00Z INFO flux_vm::fluxc] Checkpoint created at cycle 42
[2024-05-20T14:30:00Z INFO flux_vm::fluxc] Program completed successfully, exit code 0
```

### Step 4: Run a Sample FLUX-X General Compute Program
A vector addition workload using register-based ops with capability-based memory access:
```bash
cargo run --example fluxx-vector-add
```
Sample Output:
```
[2024-05-20T14:31:00Z INFO flux_vm::fluxx] FLUX-X VM initialized (General Purpose Mode)
[2024-05-20T14:31:00Z INFO flux_vm::fluxx] Validating memory capability for vector allocation: PASS
[2024-05-20T14:31:00Z INFO flux_vm::fluxx] Vector addition completed: Result = [3, 7, 11, 15]
[2024-05-20T14:31:00Z INFO flux_vm::fluxx] Program completed successfully, exit code 0
```

### Step 5: Compile with the Universal AST Toolchain
Compile a high-level AST program to FLUX-C bytecode:
```bash
cargo run --bin flux-ast-compiler -- --input ./examples/ast/guidance-check.ast --target fluxc --output ./guidance-check.fluxc
```

## Testing
flux-vm includes a comprehensive test suite covering all ISA specifications, bridge interop, temporal safety controls, and capability-based security. All 55 core tests pass on the main branch, with 100% coverage of critical VM logic for both FLUX-C and FLUX-X modes.

### Run the Full Test Suite
```bash
cargo test --all --release
```

### Run Targeted Test Suites
- FLUX-C specific tests:
  ```bash
  cargo test --package flux-vm --features="fluxc"
  ```
- FLUX-X specific tests:
  ```bash
  cargo test --package flux-vm --features="fluxx"
  ```
- Bridge interop tests:
  ```bash
  cargo test --package flux-vm --features="bridge"
  ```

### Coverage Reporting
Generate a detailed HTML coverage report:
```bash
cargo llvm-cov --all --open
```

## TrustZone-Style Bridge Protocol
The flux-vm bridge implements hardware-enforced secure isolation modeled after ARM TrustZone, where FLUX-C operates as a dedicated secure enclave completely isolated from FLUX-X general-purpose compute.

### 4-Byte Fixed Format Communication
All cross-VM commands use a standardized 4-byte fixed-length packet format to ensure deterministic, certifiable behavior for DO-254 DAL A compliance:
| Byte Offset | Field Name       | Size (Bits) | Description                                                                 |
|-------------|------------------|-------------|-----------------------------------------------------------------------------|
| 0           | Packet Type      | 8           | Identifies command (e.g., 0x01 = CHECKPOINT_SYNC, 0x02 = CAP_TRANSFER)       |
| 1-2         | Payload Length   | 16          | Fixed to 0x0002 for standard commands, 0x0000 for no payload               |
| 3           | Checksum         | 8           | XOR checksum of packet type and payload bytes for integrity validation      |

### Security Guarantees
1.  **Isolated Encrypted Memory**: FLUX-C secure enclave has access to a dedicated encrypted memory region inaccessible to FLUX-X
2.  **Privilege Boundaries**: Only pre-approved bridge commands can cross between the two VMs
3.  **Capability Enforcement**: All cross-VM data transfers require valid CAP_GRANT tokens from both VMs
4.  **Temporal Synchronization**: DEADLINE and DRIFT opcodes are synchronized between both VMs to ensure consistent safety timing

### Standard Interop Workflow
1.  FLUX-X general-purpose code initiates a bridge request using the `XBAR_SEND` opcode
2.  The bridge validates the request against active capabilities and privilege levels
3.  FLUX-C secure enclave processes the request and returns a response via `XBAR_RECV`
4.  The bridge verifies the response integrity and forwards it to FLUX-X

## Official ISA Crates
The flux-vm ecosystem includes five specialized ISA crates tailored to specific use cases:
1.  **flux-isa**: Core shared ISA definitions used by all flux-vm components
2.  **flux-isa-mini**: Ultra-lightweight FLUX-C subset for embedded microcontrollers with limited resources
3.  **flux-isa-std**: Full FLUX-C 50-opcode standard ISA, pre-certified for DO-254 DAL A compliance
4.  **flux-isa-edge**: FLUX-X ISA optimized for edge computing workloads with low-latency requirements
5.  **flux-isa-thor**: High-performance FLUX-X ISA with extended floating-point, SIMD, and multi-threading support

## Contributing
We welcome contributions from the community to improve flux-vm, expand ISA support, fix bugs, and enhance certification tooling. Please follow these guidelines to ensure consistency and compliance:

### Contribution Workflow
1.  Fork the repository and create a feature branch: `git checkout -b feature/your-feature-name`
2.  Write comprehensive tests for all new code, aiming for 100% coverage of core logic
3.  Run the full test suite and confirm all 55+ tests pass
4.  Format your code with `cargo fmt` and lint with `cargo clippy`
5.  Submit a pull request with a detailed description of your changes, including use cases and testing results

### Code Standards
- Follow Rust's official [Rust Style Guide](https://doc.rust-lang.org/style-guide.html)
- Document all public APIs with Rustdoc comments
- For safety-critical contributions (FLUX-C mode), include DO-254 compliant documentation and test plans
- All new ISA opcodes must include updates to the official ISA crates and documentation

### Certification Support
If you are working on DO-254 DAL A certification for your use case, please reach out to the maintainers for access to certification tooling, compliance documentation, and community support.

## License
flux-vm is licensed under either of:
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in flux-vm by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.