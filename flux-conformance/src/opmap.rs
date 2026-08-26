//! Byte → meaning tables for every implementation in the repo.
//!
//! Rows marked `MachineChecked` are cross-verified against the
//! implementation's own decoder (`from_u8` / `from_byte`) for all 256 bytes
//! by the harness self-tests. Rows marked `StaticOnly` are transcribed from
//! source (C macros, the SAT8 fragment, the Python demo) and carry their
//! source reference.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImplId {
    CoreC,
    MonitorC,
    VmRs,
    Isa,
    Mini,
    Std,
    Edge,
    Thor,
    Sat8Fragment,
    PyMaritime,
}

impl ImplId {
    pub const ALL: [ImplId; 10] = [
        ImplId::CoreC,
        ImplId::MonitorC,
        ImplId::VmRs,
        ImplId::Isa,
        ImplId::Mini,
        ImplId::Std,
        ImplId::Edge,
        ImplId::Thor,
        ImplId::Sat8Fragment,
        ImplId::PyMaritime,
    ];
    pub fn name(&self) -> &'static str {
        match self {
            ImplId::CoreC => "C core (flux_runtime_arm.c)",
            ImplId::MonitorC => "C monitor (flux_monitor_arm.c)",
            ImplId::VmRs => "vm/flux_vm.rs",
            ImplId::Isa => "flux-isa",
            ImplId::Mini => "flux-isa-mini",
            ImplId::Std => "flux-isa-std",
            ImplId::Edge => "flux-isa-edge",
            ImplId::Thor => "flux-isa-thor",
            ImplId::Sat8Fragment => "SAT8 fragment (.inc, orphan)",
            ImplId::PyMaritime => "maritime_constraints.py",
        }
    }
    pub fn runnable(&self) -> bool {
        !matches!(self, ImplId::Sat8Fragment | ImplId::PyMaritime)
    }
}

/// Semantic class used to rank collisions (push vs pop vs arithmetic vs ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SemClass {
    PushImm,
    Pop,
    Dup,
    Swap,
    Arithmetic,
    Logic,
    Compare,
    RangeCheck,
    Constraint,
    Halt,
    ControlFlow,
    Memory,
    Io,
    Crypto,
    Temporal,
    Security,
    Sat8,
    Nop,
    Misc,
}

impl SemClass {
    pub fn name(&self) -> &'static str {
        match self {
            SemClass::PushImm => "push-imm",
            SemClass::Pop => "pop",
            SemClass::Dup => "dup",
            SemClass::Swap => "swap",
            SemClass::Arithmetic => "arith",
            SemClass::Logic => "logic",
            SemClass::Compare => "cmp",
            SemClass::RangeCheck => "range-check",
            SemClass::Constraint => "assert/constraint",
            SemClass::Halt => "halt",
            SemClass::ControlFlow => "control-flow",
            SemClass::Memory => "memory",
            SemClass::Io => "io",
            SemClass::Crypto => "crypto",
            SemClass::Temporal => "temporal",
            SemClass::Security => "security",
            SemClass::Sat8 => "sat8",
            SemClass::Nop => "nop",
            SemClass::Misc => "misc",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ByteMeaning {
    pub meaning: &'static str,
    pub class: SemClass,
    pub source: &'static str,
}

pub fn table(impl_id: ImplId) -> BTreeMap<u8, ByteMeaning> {
    let mut m = BTreeMap::new();
    let add = |m: &mut BTreeMap<u8, ByteMeaning>,
               b: u8,
               meaning: &'static str,
               class: SemClass,
               src: &'static str| {
        m.insert(
            b,
            ByteMeaning {
                meaning,
                class,
                source: src,
            },
        );
    };
    match impl_id {
        ImplId::CoreC => {
            let src = "src/flux_runtime_arm.h:60-89";
            add(&mut m, 0x00, "NOP", SemClass::Nop, src);
            add(&mut m, 0x01, "PUSH_I8", SemClass::PushImm, src);
            add(&mut m, 0x02, "PUSH_I16", SemClass::PushImm, src);
            add(&mut m, 0x03, "PUSH_I32", SemClass::PushImm, src);
            add(&mut m, 0x10, "LOAD_INPUT", SemClass::PushImm, src);
            add(&mut m, 0x1A, "HALT", SemClass::Halt, src);
            add(&mut m, 0x1B, "ASSERT", SemClass::Constraint, src);
            add(&mut m, 0x1C, "CHECK_DOMAIN", SemClass::Constraint, src);
            add(
                &mut m,
                0x1D,
                "RANGE(i16 lo, i16 hi)",
                SemClass::RangeCheck,
                src,
            );
            add(&mut m, 0x20, "ADD", SemClass::Arithmetic, src);
            add(&mut m, 0x21, "SUB", SemClass::Arithmetic, src);
            add(&mut m, 0x22, "MUL", SemClass::Arithmetic, src);
            add(&mut m, 0x23, "EQ", SemClass::Compare, src);
            add(&mut m, 0x24, "LT", SemClass::Compare, src);
            add(&mut m, 0x25, "GT", SemClass::Compare, src);
            add(&mut m, 0x26, "BOOL_AND", SemClass::Logic, src);
            add(&mut m, 0x27, "BOOL_OR", SemClass::Logic, src);
            add(&mut m, 0x28, "DUP", SemClass::Dup, src);
            add(&mut m, 0x29, "SWAP", SemClass::Swap, src);
            add(&mut m, 0x2A, "NOT", SemClass::Logic, src);
        }
        ImplId::MonitorC => {
            let src = "src/flux_monitor_arm.c:26-46";
            add(&mut m, 0x00, "NOP", SemClass::Nop, src);
            add(&mut m, 0x01, "PUSH_I32", SemClass::PushImm, src);
            add(&mut m, 0x02, "PUSH_F32", SemClass::PushImm, src);
            add(&mut m, 0x03, "POP", SemClass::Pop, src);
            add(&mut m, 0x04, "DUP", SemClass::Dup, src);
            add(&mut m, 0x05, "SWAP", SemClass::Swap, src);
            add(&mut m, 0x10, "CHECK_RANGE_I32", SemClass::RangeCheck, src);
            add(&mut m, 0x11, "CHECK_RANGE_F32", SemClass::RangeCheck, src);
            add(
                &mut m,
                0x12,
                "CHECK_DOMAIN (defined, NOT dispatched)",
                SemClass::Constraint,
                src,
            );
            add(&mut m, 0x20, "AND", SemClass::Logic, src);
            add(&mut m, 0x21, "OR", SemClass::Logic, src);
            add(&mut m, 0x22, "NOT", SemClass::Logic, src);
            add(
                &mut m,
                0x30,
                "ADD_I32 (defined, NOT dispatched)",
                SemClass::Arithmetic,
                src,
            );
            add(
                &mut m,
                0x31,
                "SUB_I32 (defined, NOT dispatched)",
                SemClass::Arithmetic,
                src,
            );
            add(
                &mut m,
                0x32,
                "MUL_I32 (defined, NOT dispatched)",
                SemClass::Arithmetic,
                src,
            );
            add(
                &mut m,
                0x40,
                "CMP_EQ (defined, NOT dispatched)",
                SemClass::Compare,
                src,
            );
            add(
                &mut m,
                0x41,
                "CMP_LT (defined, NOT dispatched)",
                SemClass::Compare,
                src,
            );
            add(
                &mut m,
                0x42,
                "CMP_GE (defined, NOT dispatched)",
                SemClass::Compare,
                src,
            );
            add(&mut m, 0x50, "CHECKPOINT", SemClass::Temporal, src);
            add(&mut m, 0x51, "REVERT", SemClass::Temporal, src);
            add(&mut m, 0xFF, "HALT", SemClass::Halt, src);
        }
        ImplId::VmRs => {
            let src = "vm/flux_vm.rs:145-455 (match on raw byte)";
            add(&mut m, 0x00, "PUSH u8 imm", SemClass::PushImm, src);
            add(&mut m, 0x01, "POP", SemClass::Pop, src);
            add(&mut m, 0x02, "DUP", SemClass::Dup, src);
            add(&mut m, 0x03, "SWAP", SemClass::Swap, src);
            add(&mut m, 0x04, "LOAD u8 addr", SemClass::Memory, src);
            add(&mut m, 0x05, "STORE u8 addr", SemClass::Memory, src);
            add(&mut m, 0x06, "ADD (u8 wrap)", SemClass::Arithmetic, src);
            add(&mut m, 0x07, "SUB (u8 wrap)", SemClass::Arithmetic, src);
            add(&mut m, 0x08, "MUL (u8 wrap)", SemClass::Arithmetic, src);
            add(&mut m, 0x09, "AND (bitwise)", SemClass::Logic, src);
            add(&mut m, 0x0A, "OR (bitwise)", SemClass::Logic, src);
            add(&mut m, 0x0B, "XOR (bitwise)", SemClass::Logic, src);
            add(&mut m, 0x0C, "NOT (bitwise !)", SemClass::Logic, src);
            add(&mut m, 0x0D, "SHL 1", SemClass::Arithmetic, src);
            add(&mut m, 0x0E, "SHR 1", SemClass::Arithmetic, src);
            add(&mut m, 0x0F, "EQ", SemClass::Compare, src);
            add(&mut m, 0x10, "NEQ", SemClass::Compare, src);
            add(&mut m, 0x11, "LT", SemClass::Compare, src);
            add(&mut m, 0x12, "GT", SemClass::Compare, src);
            add(&mut m, 0x13, "LTE", SemClass::Compare, src);
            add(&mut m, 0x14, "GTE", SemClass::Compare, src);
            add(&mut m, 0x15, "JUMP", SemClass::ControlFlow, src);
            add(&mut m, 0x16, "JZ", SemClass::ControlFlow, src);
            add(&mut m, 0x17, "JNZ", SemClass::ControlFlow, src);
            add(&mut m, 0x18, "CALL", SemClass::ControlFlow, src);
            add(&mut m, 0x19, "RET", SemClass::ControlFlow, src);
            add(&mut m, 0x1A, "HALT", SemClass::Halt, src);
            add(&mut m, 0x1B, "ASSERT", SemClass::Constraint, src);
            add(&mut m, 0x1C, "CHECK_DOMAIN mask", SemClass::Constraint, src);
            add(
                &mut m,
                0x1D,
                "BITMASK_RANGE lo hi",
                SemClass::RangeCheck,
                src,
            );
            add(&mut m, 0x1E, "LOAD_GUARD", SemClass::Security, src);
            add(&mut m, 0x1F, "MERKLE_VERIFY", SemClass::Crypto, src);
            add(&mut m, 0x20, "GUARD_TRAP", SemClass::Security, src);
            add(&mut m, 0x21, "CRC32", SemClass::Crypto, src);
            add(&mut m, 0x22, "PUSH_HASH", SemClass::Crypto, src);
            add(&mut m, 0x23, "XNOR_POPCOUNT", SemClass::Crypto, src);
            add(&mut m, 0x24, "CMP_GE", SemClass::Compare, src);
            add(&mut m, 0x25, "CARRY_LT", SemClass::Compare, src);
            add(&mut m, 0x26, "JFAIL", SemClass::ControlFlow, src);
            add(&mut m, 0x27, "NOP", SemClass::Nop, src);
            add(&mut m, 0x28, "FLUSH", SemClass::Misc, src);
            add(&mut m, 0x29, "YIELD", SemClass::Misc, src);
            add(&mut m, 0x2A, "TICK", SemClass::Temporal, src);
            add(&mut m, 0x2B, "DEADLINE", SemClass::Temporal, src);
            add(&mut m, 0x2C, "CHECKPOINT", SemClass::Temporal, src);
            add(&mut m, 0x2D, "REVERT", SemClass::Temporal, src);
            add(&mut m, 0x2E, "ELAPSED", SemClass::Temporal, src);
            add(&mut m, 0x2F, "DRIFT", SemClass::Temporal, src);
            add(&mut m, 0x30, "NOP_TEMP", SemClass::Nop, src);
            add(&mut m, 0x31, "DEADLINE_CHECK", SemClass::Temporal, src);
            add(&mut m, 0x32, "SANDBOX_ENTER", SemClass::Security, src);
            add(&mut m, 0x33, "SANDBOX_EXIT", SemClass::Security, src);
            add(&mut m, 0x34, "CAP_GRANT", SemClass::Security, src);
            add(&mut m, 0x35, "CAP_REVOKE", SemClass::Security, src);
            add(&mut m, 0x36, "MEM_GUARD", SemClass::Security, src);
            add(&mut m, 0x37, "PROVE", SemClass::Security, src);
            add(&mut m, 0x38, "AUDIT_PUSH", SemClass::Security, src);
            add(&mut m, 0x39, "SEAL", SemClass::Security, src);
        }
        ImplId::Isa => {
            let src = "flux-isa/src/opcode.rs (machine-checked)";
            add(&mut m, 0x01, "Add", SemClass::Arithmetic, src);
            add(&mut m, 0x02, "Sub", SemClass::Arithmetic, src);
            add(&mut m, 0x03, "Mul", SemClass::Arithmetic, src);
            add(&mut m, 0x04, "Div", SemClass::Arithmetic, src);
            add(&mut m, 0x05, "Mod", SemClass::Arithmetic, src);
            add(&mut m, 0x10, "Assert", SemClass::Constraint, src);
            add(&mut m, 0x11, "Check", SemClass::Constraint, src);
            add(&mut m, 0x12, "Validate", SemClass::RangeCheck, src);
            add(&mut m, 0x13, "Reject", SemClass::Constraint, src);
            add(&mut m, 0x20, "Jump", SemClass::ControlFlow, src);
            add(&mut m, 0x21, "Branch", SemClass::ControlFlow, src);
            add(&mut m, 0x22, "Call", SemClass::ControlFlow, src);
            add(&mut m, 0x23, "Return", SemClass::ControlFlow, src);
            add(&mut m, 0x24, "Halt", SemClass::Halt, src);
            add(&mut m, 0x30, "Load", SemClass::PushImm, src);
            add(&mut m, 0x31, "Store", SemClass::Memory, src);
            add(&mut m, 0x32, "Push", SemClass::PushImm, src);
            add(&mut m, 0x33, "Pop", SemClass::Pop, src);
            add(&mut m, 0x34, "Swap", SemClass::Swap, src);
            add(&mut m, 0x40, "Snap", SemClass::Misc, src);
            add(&mut m, 0x41, "Quantize", SemClass::Misc, src);
            add(&mut m, 0x42, "Cast", SemClass::Misc, src);
            add(&mut m, 0x43, "Promote", SemClass::Misc, src);
            add(&mut m, 0x50, "And", SemClass::Logic, src);
            add(&mut m, 0x51, "Or", SemClass::Logic, src);
            add(&mut m, 0x52, "Not", SemClass::Logic, src);
            add(&mut m, 0x53, "Xor", SemClass::Logic, src);
            add(&mut m, 0x60, "Eq", SemClass::Compare, src);
            add(&mut m, 0x61, "Neq", SemClass::Compare, src);
            add(&mut m, 0x62, "Lt", SemClass::Compare, src);
            add(&mut m, 0x63, "Gt", SemClass::Compare, src);
            add(&mut m, 0x64, "Lte", SemClass::Compare, src);
            add(&mut m, 0x65, "Gte", SemClass::Compare, src);
            add(&mut m, 0x70, "Nop", SemClass::Nop, src);
            add(&mut m, 0x71, "Debug", SemClass::Io, src);
            add(&mut m, 0x72, "Trace", SemClass::Io, src);
            add(&mut m, 0x73, "Dump", SemClass::Io, src);
        }
        ImplId::Mini => {
            let src = "flux-isa-mini/src/opcode.rs (machine-checked)";
            add(&mut m, 0x01, "Add", SemClass::Arithmetic, src);
            add(&mut m, 0x02, "Sub", SemClass::Arithmetic, src);
            add(&mut m, 0x03, "Mul", SemClass::Arithmetic, src);
            add(&mut m, 0x04, "Div", SemClass::Arithmetic, src);
            add(&mut m, 0x05, "Mod", SemClass::Arithmetic, src);
            add(&mut m, 0x10, "Eq", SemClass::Compare, src);
            add(&mut m, 0x11, "Lt", SemClass::Compare, src);
            add(&mut m, 0x12, "Gt", SemClass::Compare, src);
            add(&mut m, 0x13, "Lte", SemClass::Compare, src);
            add(&mut m, 0x14, "Gte", SemClass::Compare, src);
            add(&mut m, 0x20, "Assert", SemClass::Constraint, src);
            add(&mut m, 0x21, "Check", SemClass::Constraint, src);
            add(&mut m, 0x22, "Validate", SemClass::RangeCheck, src);
            add(&mut m, 0x23, "Reject", SemClass::Constraint, src);
            add(&mut m, 0x30, "Load", SemClass::PushImm, src);
            add(&mut m, 0x31, "Push", SemClass::PushImm, src);
            add(&mut m, 0x32, "Pop", SemClass::Pop, src);
            add(&mut m, 0x40, "Snap", SemClass::Misc, src);
            add(&mut m, 0x41, "Quantize", SemClass::Misc, src);
            add(&mut m, 0xF0, "Halt", SemClass::Halt, src);
            add(&mut m, 0xFF, "Nop", SemClass::Nop, src);
        }
        ImplId::Std => {
            let src = "flux-isa-std/src/opcode.rs (machine-checked)";
            add(&mut m, 0x01, "Push", SemClass::PushImm, src);
            add(&mut m, 0x02, "Pop", SemClass::Pop, src);
            add(&mut m, 0x03, "Dup", SemClass::Dup, src);
            add(&mut m, 0x04, "Swap", SemClass::Swap, src);
            add(&mut m, 0x05, "Over", SemClass::Dup, src);
            add(&mut m, 0x06, "Rot", SemClass::Misc, src);
            add(&mut m, 0x07, "Depth", SemClass::Misc, src);
            add(&mut m, 0x10, "Add", SemClass::Arithmetic, src);
            add(&mut m, 0x11, "Sub", SemClass::Arithmetic, src);
            add(&mut m, 0x12, "Mul", SemClass::Arithmetic, src);
            add(&mut m, 0x13, "Div", SemClass::Arithmetic, src);
            add(&mut m, 0x14, "Mod", SemClass::Arithmetic, src);
            add(&mut m, 0x15, "Negate", SemClass::Arithmetic, src);
            add(&mut m, 0x16, "Abs", SemClass::Arithmetic, src);
            add(&mut m, 0x20, "And", SemClass::Logic, src);
            add(&mut m, 0x21, "Or", SemClass::Logic, src);
            add(&mut m, 0x22, "Not", SemClass::Logic, src);
            add(&mut m, 0x23, "Xor", SemClass::Logic, src);
            add(&mut m, 0x24, "Shl", SemClass::Arithmetic, src);
            add(&mut m, 0x30, "Eq", SemClass::Compare, src);
            add(&mut m, 0x31, "Ne", SemClass::Compare, src);
            add(&mut m, 0x32, "Lt", SemClass::Compare, src);
            add(&mut m, 0x33, "Gt", SemClass::Compare, src);
            add(&mut m, 0x34, "Le", SemClass::Compare, src);
            add(&mut m, 0x35, "Ge", SemClass::Compare, src);
            add(&mut m, 0x40, "Jmp", SemClass::ControlFlow, src);
            add(&mut m, 0x41, "Call", SemClass::ControlFlow, src);
            add(&mut m, 0x42, "Ret", SemClass::ControlFlow, src);
            add(&mut m, 0x43, "Halt", SemClass::Halt, src);
            add(&mut m, 0x44, "Nop", SemClass::Nop, src);
            add(&mut m, 0x50, "Load", SemClass::Memory, src);
            add(&mut m, 0x51, "Store", SemClass::Memory, src);
            add(&mut m, 0x52, "LoadConst", SemClass::PushImm, src);
            add(&mut m, 0x60, "Assert", SemClass::Constraint, src);
            add(&mut m, 0x61, "Check", SemClass::Constraint, src);
            add(&mut m, 0x70, "Print", SemClass::Io, src);
            add(&mut m, 0x71, "Emit", SemClass::Io, src);
        }
        ImplId::Edge => {
            let src = "flux-isa-edge/src/opcode.rs (machine-checked)";
            add(&mut m, 0x00, "Nop", SemClass::Nop, src);
            add(&mut m, 0x01, "Push", SemClass::PushImm, src);
            add(&mut m, 0x02, "Pop", SemClass::Pop, src);
            add(&mut m, 0x03, "Dup", SemClass::Dup, src);
            add(&mut m, 0x04, "Swap", SemClass::Swap, src);
            add(&mut m, 0x05, "Load", SemClass::Memory, src);
            add(&mut m, 0x06, "Store", SemClass::Memory, src);
            add(&mut m, 0x10, "Add", SemClass::Arithmetic, src);
            add(&mut m, 0x11, "Sub", SemClass::Arithmetic, src);
            add(&mut m, 0x12, "Mul", SemClass::Arithmetic, src);
            add(&mut m, 0x13, "Div", SemClass::Arithmetic, src);
            add(&mut m, 0x14, "Mod", SemClass::Arithmetic, src);
            add(&mut m, 0x15, "Neg", SemClass::Arithmetic, src);
            add(&mut m, 0x20, "Eq", SemClass::Compare, src);
            add(&mut m, 0x21, "Ne", SemClass::Compare, src);
            add(&mut m, 0x22, "Lt", SemClass::Compare, src);
            add(&mut m, 0x23, "Le", SemClass::Compare, src);
            add(&mut m, 0x24, "Gt", SemClass::Compare, src);
            add(&mut m, 0x25, "Ge", SemClass::Compare, src);
            add(&mut m, 0x30, "And", SemClass::Logic, src);
            add(&mut m, 0x31, "Or", SemClass::Logic, src);
            add(&mut m, 0x32, "Not", SemClass::Logic, src);
            add(&mut m, 0x40, "Validate", SemClass::RangeCheck, src);
            add(&mut m, 0x41, "Assert", SemClass::Constraint, src);
            add(&mut m, 0x42, "Tolerance", SemClass::RangeCheck, src);
            add(&mut m, 0x43, "Clamp", SemClass::RangeCheck, src);
            add(&mut m, 0x50, "Jump", SemClass::ControlFlow, src);
            add(&mut m, 0x51, "JumpIf", SemClass::ControlFlow, src);
            add(&mut m, 0x52, "Call", SemClass::ControlFlow, src);
            add(&mut m, 0x53, "Ret", SemClass::ControlFlow, src);
            add(&mut m, 0x54, "Halt", SemClass::Halt, src);
            add(&mut m, 0x60, "Input", SemClass::Io, src);
            add(&mut m, 0x61, "Output", SemClass::Io, src);
            add(&mut m, 0x70, "Sync", SemClass::Io, src);
        }
        ImplId::Thor => {
            let src = "flux-isa-thor/src/opcode.rs (machine-checked; Push imm is f64 BE, Load/Store u32 BE)";
            add(&mut m, 0x00, "Nop", SemClass::Nop, src);
            add(&mut m, 0x01, "Push", SemClass::PushImm, src);
            add(&mut m, 0x02, "Pop", SemClass::Pop, src);
            add(&mut m, 0x03, "Dup", SemClass::Dup, src);
            add(&mut m, 0x04, "Swap", SemClass::Swap, src);
            add(&mut m, 0x05, "Load", SemClass::Memory, src);
            add(&mut m, 0x06, "Store", SemClass::Memory, src);
            add(&mut m, 0x10, "Add", SemClass::Arithmetic, src);
            add(&mut m, 0x11, "Sub", SemClass::Arithmetic, src);
            add(&mut m, 0x12, "Mul", SemClass::Arithmetic, src);
            add(&mut m, 0x13, "Div", SemClass::Arithmetic, src);
            add(&mut m, 0x14, "Mod", SemClass::Arithmetic, src);
            add(&mut m, 0x15, "Neg", SemClass::Arithmetic, src);
            add(&mut m, 0x20, "And", SemClass::Logic, src);
            add(&mut m, 0x21, "Or", SemClass::Logic, src);
            add(&mut m, 0x22, "Not", SemClass::Logic, src);
            add(&mut m, 0x30, "Eq", SemClass::Compare, src);
            add(&mut m, 0x31, "Ne", SemClass::Compare, src);
            add(&mut m, 0x32, "Lt", SemClass::Compare, src);
            add(&mut m, 0x33, "Le", SemClass::Compare, src);
            add(&mut m, 0x34, "Gt", SemClass::Compare, src);
            add(&mut m, 0x35, "Ge", SemClass::Compare, src);
            add(&mut m, 0x40, "Jmp", SemClass::ControlFlow, src);
            add(&mut m, 0x41, "Jz", SemClass::ControlFlow, src);
            add(&mut m, 0x42, "Jnz", SemClass::ControlFlow, src);
            add(&mut m, 0x43, "Call", SemClass::ControlFlow, src);
            add(&mut m, 0x44, "Ret", SemClass::ControlFlow, src);
            add(&mut m, 0x45, "Halt", SemClass::Halt, src);
            add(&mut m, 0x50, "Assert", SemClass::Constraint, src);
            add(&mut m, 0x51, "Constrain", SemClass::Constraint, src);
            add(&mut m, 0x52, "Propagate", SemClass::Constraint, src);
            add(&mut m, 0x53, "Solve", SemClass::Constraint, src);
            add(&mut m, 0x54, "Verify", SemClass::Constraint, src);
            add(&mut m, 0x60, "Print", SemClass::Io, src);
            add(&mut m, 0x61, "Debug", SemClass::Io, src);
            add(&mut m, 0x80, "ParallelBranch", SemClass::Misc, src);
            add(&mut m, 0x81, "Reduce", SemClass::Misc, src);
            add(&mut m, 0x82, "GpuCompile", SemClass::Misc, src);
            add(&mut m, 0x83, "BatchSolve", SemClass::Misc, src);
            add(&mut m, 0x84, "SonarBatch", SemClass::Misc, src);
            add(&mut m, 0x85, "TileCommit", SemClass::Misc, src);
            add(&mut m, 0x86, "Pathfind", SemClass::Misc, src);
            add(&mut m, 0x87, "ExtendedEnd", SemClass::Misc, src);
        }
        ImplId::Sat8Fragment => {
            let src = "src/flux_sat8_ops.inc:44-80 (orphan fragment)";
            add(&mut m, 0x30, "SAT8", SemClass::Sat8, src);
            add(&mut m, 0x31, "SAT8_ADD", SemClass::Sat8, src);
            add(&mut m, 0x32, "SAT8_SUB", SemClass::Sat8, src);
            add(&mut m, 0x33, "SAT8_MUL", SemClass::Sat8, src);
            add(&mut m, 0x34, "SAT8_NEG", SemClass::Sat8, src);
            add(&mut m, 0x35, "SAT8_CHECK", SemClass::Sat8, src);
            add(&mut m, 0x36, "SAT8_CHECK_M", SemClass::Sat8, src);
            add(&mut m, 0x37, "SAT8_MASK_READ", SemClass::Sat8, src);
        }
        ImplId::PyMaritime => {
            let src = "src/maritime_constraints.py:26-34 (CPU fallback interp)";
            add(&mut m, 0x1A, "HALT", SemClass::Halt, src);
            add(
                &mut m,
                0x1B,
                "ASSERT (peeks then pops)",
                SemClass::Constraint,
                src,
            );
            add(
                &mut m,
                0x1D,
                "RANGE(u8 lo, u8 hi) — 2 operand bytes",
                SemClass::RangeCheck,
                src,
            );
            add(&mut m, 0x26, "BOOL_AND", SemClass::Logic, src);
            add(&mut m, 0x27, "BOOL_OR", SemClass::Logic, src);
            add(&mut m, 0x28, "DUP", SemClass::Dup, src);
            add(&mut m, 0x29, "SWAP", SemClass::Swap, src);
        }
    }
    m
}

#[derive(Debug, Clone)]
pub struct Collision {
    pub byte: u8,
    /// (impl, meaning, class, source) for each impl that decodes this byte.
    pub entries: Vec<(ImplId, &'static str, SemClass, &'static str)>,
    /// Distinct semantic classes assigned to this byte.
    pub distinct_classes: usize,
    /// Distinct meanings (finer-grained than classes).
    pub distinct_meanings: usize,
}

/// All bytes that at least two implementations assign *different meanings*.
pub fn collisions() -> Vec<Collision> {
    let mut out = Vec::new();
    for byte in 0u8..=255 {
        let mut entries = Vec::new();
        for id in ImplId::ALL {
            if let Some(bm) = table(id).get(&byte) {
                entries.push((id, bm.meaning, bm.class, bm.source));
            }
        }
        if entries.len() < 2 {
            continue;
        }
        let distinct_meanings = entries
            .iter()
            .map(|(_, m, _, _)| *m)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let distinct_classes = entries
            .iter()
            .map(|(_, _, c, _)| *c)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        if distinct_meanings > 1 {
            out.push(Collision {
                byte,
                entries,
                distinct_classes,
                distinct_meanings,
            });
        }
    }
    out
}
