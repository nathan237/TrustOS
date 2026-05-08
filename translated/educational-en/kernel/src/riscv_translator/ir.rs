// TrustOS Universal Architecture Translation Layer
// RISC-V Intermediate Representation (IR)
//
// Uses RISC-V as a clean, minimal IR to translate ANY architecture.
// RISC-V's simplicity (load/store, fixed-width, register-register) makes
// it the perfect universal intermediate form for binary translation.

use alloc::string::String;
use alloc::vec::Vec;

/// RISC-V general-purpose registers (x0-x31)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
// Enumeration — a type that can be one of several variants.
pub enum Reg {
    X0 = 0,   // zero — hardwired zero
    X1 = 1,   // ra — return address
    X2 = 2,   // sp — stack pointer
    X3 = 3,   // gp — global pointer
    X4 = 4,   // tp — thread pointer
    X5 = 5,   // t0 — temp
    X6 = 6,   // t1
    X7 = 7,   // t2
    X8 = 8,   // s0/fp — saved/frame pointer
    X9 = 9,   // s1
    X10 = 10, // a0 — arg0 / return value
    X11 = 11, // a1 — arg1
    X12 = 12, // a2
    X13 = 13, // a3
    X14 = 14, // a4
    X15 = 15, // a5
    X16 = 16, // a6
    X17 = 17, // a7 — syscall number (ecall)
    X18 = 18, // s2
    X19 = 19, // s3
    X20 = 20, // s4
    X21 = 21, // s5
    X22 = 22, // s6
    X23 = 23, // s7
    X24 = 24, // s8
    X25 = 25, // s9
    X26 = 26, // s10
    X27 = 27, // s11
    X28 = 28, // t3
    X29 = 29, // t4
    X30 = 30, // t5
    X31 = 31, // t6

    // Virtual registers for translation (mapped to memory spill slots)
    // Used when source arch has more state than 32 GPRs
    VFlags = 32,  // Condition flags (from x86 EFLAGS, ARM NZCV)
    VPc = 33,     // Source program counter (tracking)
}

// Implementation block — defines methods for the type above.
impl Reg {
        // Public function — callable from other modules.
pub fn from_index(i: u8) -> Self {
                // Pattern matching — Rust's exhaustive branching construct.
match i {
            0 => Reg::X0, 1 => Reg::X1, 2 => Reg::X2, 3 => Reg::X3,
            4 => Reg::X4, 5 => Reg::X5, 6 => Reg::X6, 7 => Reg::X7,
            8 => Reg::X8, 9 => Reg::X9, 10 => Reg::X10, 11 => Reg::X11,
            12 => Reg::X12, 13 => Reg::X13, 14 => Reg::X14, 15 => Reg::X15,
            16 => Reg::X16, 17 => Reg::X17, 18 => Reg::X18, 19 => Reg::X19,
            20 => Reg::X20, 21 => Reg::X21, 22 => Reg::X22, 23 => Reg::X23,
            24 => Reg::X24, 25 => Reg::X25, 26 => Reg::X26, 27 => Reg::X27,
            28 => Reg::X28, 29 => Reg::X29, 30 => Reg::X30, 31 => Reg::X31,
            32 => Reg::VFlags, 33 => Reg::VPc,
            _ => Reg::X0,
        }
    }

    /// ABI name
    pub fn abi_name(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            Reg::X0 => "zero", Reg::X1 => "ra", Reg::X2 => "sp", Reg::X3 => "gp",
            Reg::X4 => "tp", Reg::X5 => "t0", Reg::X6 => "t1", Reg::X7 => "t2",
            Reg::X8 => "fp", Reg::X9 => "s1", Reg::X10 => "a0", Reg::X11 => "a1",
            Reg::X12 => "a2", Reg::X13 => "a3", Reg::X14 => "a4", Reg::X15 => "a5",
            Reg::X16 => "a6", Reg::X17 => "a7", Reg::X18 => "s2", Reg::X19 => "s3",
            Reg::X20 => "s4", Reg::X21 => "s5", Reg::X22 => "s6", Reg::X23 => "s7",
            Reg::X24 => "s8", Reg::X25 => "s9", Reg::X26 => "s10", Reg::X27 => "s11",
            Reg::X28 => "t3", Reg::X29 => "t4", Reg::X30 => "t5", Reg::X31 => "t6",
            Reg::VFlags => "vflags", Reg::VPc => "vpc",
        }
    }
}

/// RISC-V IR Instructions — the universal intermediate form
///
/// Every instruction from x86_64, ARM64, MIPS, etc. gets lowered to
/// one or more of these simple RISC-V operations.
#[derive(Debug, Clone)]
// Enumeration — a type that can be one of several variants.
pub enum RvInst {
    // === Integer Register-Register (R-type) ===
    Add { rd: Reg, rs1: Reg, rs2: Reg },
    Sub { rd: Reg, rs1: Reg, rs2: Reg },
    And { rd: Reg, rs1: Reg, rs2: Reg },
    Or  { rd: Reg, rs1: Reg, rs2: Reg },
    Xor { rd: Reg, rs1: Reg, rs2: Reg },
    Sll { rd: Reg, rs1: Reg, rs2: Reg },  // Shift left logical
    Srl { rd: Reg, rs1: Reg, rs2: Reg },  // Shift right logical
    Sra { rd: Reg, rs1: Reg, rs2: Reg },  // Shift right arithmetic
    Slt { rd: Reg, rs1: Reg, rs2: Reg },  // Set less than (signed)
    Sltu { rd: Reg, rs1: Reg, rs2: Reg }, // Set less than (unsigned)

    // === M extension (multiply/divide) ===
    Mul    { rd: Reg, rs1: Reg, rs2: Reg },
    Mulh   { rd: Reg, rs1: Reg, rs2: Reg }, // High bits of signed*signed
    Div    { rd: Reg, rs1: Reg, rs2: Reg },
    Divu   { rd: Reg, rs1: Reg, rs2: Reg },
    Rem    { rd: Reg, rs1: Reg, rs2: Reg },
    Remu   { rd: Reg, rs1: Reg, rs2: Reg },

    // === Integer Register-Immediate (I-type) ===
    Addi  { rd: Reg, rs1: Reg, imm: i64 },
    Andi  { rd: Reg, rs1: Reg, imm: i64 },
    Ori   { rd: Reg, rs1: Reg, imm: i64 },
    Xori  { rd: Reg, rs1: Reg, imm: i64 },
    Slli  { rd: Reg, rs1: Reg, shamt: u8 },
    Srli  { rd: Reg, rs1: Reg, shamt: u8 },
    Srai  { rd: Reg, rs1: Reg, shamt: u8 },
    Slti  { rd: Reg, rs1: Reg, imm: i64 },
    Sltiu { rd: Reg, rs1: Reg, imm: i64 },

    // === Upper Immediate ===
    Lui   { rd: Reg, imm: i64 },           // Load upper immediate
    Auipc { rd: Reg, imm: i64 },           // Add upper immediate to PC

    // === Load/Store (memory access) ===
    Lb  { rd: Reg, rs1: Reg, offset: i64 },  // Load byte (sign-extend)
    Lbu { rd: Reg, rs1: Reg, offset: i64 },  // Load byte unsigned
    Lh  { rd: Reg, rs1: Reg, offset: i64 },  // Load halfword
    Lhu { rd: Reg, rs1: Reg, offset: i64 },  // Load halfword unsigned
    Lw  { rd: Reg, rs1: Reg, offset: i64 },  // Load word
    Lwu { rd: Reg, rs1: Reg, offset: i64 },  // Load word unsigned
    Ld  { rd: Reg, rs1: Reg, offset: i64 },  // Load doubleword (RV64)
    Sb  { rs2: Reg, rs1: Reg, offset: i64 }, // Store byte
    Sh  { rs2: Reg, rs1: Reg, offset: i64 }, // Store halfword
    Sw  { rs2: Reg, rs1: Reg, offset: i64 }, // Store word
    Sd  { rs2: Reg, rs1: Reg, offset: i64 }, // Store doubleword (RV64)

    // === Branches ===
    Beq  { rs1: Reg, rs2: Reg, offset: i64 },  // Branch if equal
    Bne  { rs1: Reg, rs2: Reg, offset: i64 },  // Branch if not equal
    Blt  { rs1: Reg, rs2: Reg, offset: i64 },  // Branch if less than (signed)
    Bge  { rs1: Reg, rs2: Reg, offset: i64 },  // Branch if >= (signed)
    Bltu { rs1: Reg, rs2: Reg, offset: i64 },  // Branch if less than (unsigned)
    Bgeu { rs1: Reg, rs2: Reg, offset: i64 },  // Branch if >= (unsigned)

    // === Jump ===
    Jal  { rd: Reg, offset: i64 },              // Jump and link
    Jalr { rd: Reg, rs1: Reg, offset: i64 },    // Jump and link register

    // === System ===
    Ecall,                                        // Environment call (syscall)
    Ebreak,                                       // Breakpoint
    Fence,                                        // Memory fence

    // === Atomics (A extension) ===
    AmoswapD { rd: Reg, rs2: Reg, rs1: Reg },   // Atomic swap
    AmoaddD  { rd: Reg, rs2: Reg, rs1: Reg },   // Atomic add

    // === Pseudo-instructions (translation helpers) ===
    /// Load immediate (expands to lui+addi or more)
    Li { rd: Reg, imm: i64 },
    /// Move register (expands to addi rd, rs, 0)
    Mv { rd: Reg, rs: Reg },
    /// No operation
    Nop,
    /// Return (jalr x0, x1, 0)
    Ret,
    /// Call label (jal x1, offset)
    Call { offset: i64 },

    // === Translation-specific extensions ===
    /// Set flags from comparison (stores result in VFlags virtual register)
    /// Used to translate x86 EFLAGS or ARM NZCV flags
    CmpFlags { rs1: Reg, rs2: Reg },
    /// Branch based on translated flag condition
    BranchCond { condition: FlagCond, offset: i64 },
    /// Source architecture annotation (for debugging)
    SrcAnnotation { arch: SourceArch, addr: u64, text: String },
}

/// Flag-based branch conditions (for x86/ARM translation)
#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum FlagCond {
    Eq,    // ZF=1         (x86: JE/JZ,     ARM: BEQ)
    Ne,    // ZF=0         (x86: JNE/JNZ,   ARM: BNE)
    Lt,    // SF!=OF       (x86: JL/JNGE,   ARM: BLT)
    Ge,    // SF==OF       (x86: JGE/JNL,   ARM: BGE)
    Ltu,   // CF=1         (x86: JB/JNAE,   ARM: BLO)
    Geu,   // CF=0         (x86: JAE/JNB,   ARM: BHS)
    Le,    // ZF=1|SF!=OF  (x86: JLE/JNG,   ARM: BLE)
    Gt,    // ZF=0&SF==OF  (x86: JG/JNLE,   ARM: BGT)
    Neg,   // SF=1         (x86: JS,        ARM: BMI)
    Pos,   // SF=0         (x86: JNS,       ARM: BPL)
    Ovf,   // OF=1         (x86: JO,        ARM: BVS)
    NoOvf, // OF=0         (x86: JNO,       ARM: BVC)
}

/// Source architecture being translated
#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum SourceArch {
    X86_64,
    Aarch64,
    Riscv64,  // Passthrough (no translation needed)
    Mips64,
    Wasm,     // WebAssembly (future)
}

// Implementation block — defines methods for the type above.
impl SourceArch {
        // Public function — callable from other modules.
pub fn name(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            SourceArch::X86_64 => "x86_64",
            SourceArch::Aarch64 => "aarch64",
            SourceArch::Riscv64 => "riscv64",
            SourceArch::Mips64 => "mips64",
            SourceArch::Wasm => "wasm",
        }
    }
}

/// A translated basic block — sequence of RISC-V IR instructions
/// with no internal branches (entry at top, exit at bottom)
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct TranslatedBlock {
    /// Source address this block was translated from
    pub src_addr: u64,
    /// Source architecture
    pub src_arch: SourceArch,
    /// RISC-V IR instructions
    pub instructions: Vec<RvInst>,
    /// Number of source instructions that produced this block
    pub src_inst_count: usize,
    /// Possible successor addresses
    pub successors: Vec<u64>,
}

// Implementation block — defines methods for the type above.
impl TranslatedBlock {
        // Public function — callable from other modules.
pub fn new(src_addr: u64, src_arch: SourceArch) -> Self {
        Self {
            src_addr,
            src_arch,
            instructions: Vec::new(),
            src_inst_count: 0,
            successors: Vec::new(),
        }
    }

        // Public function — callable from other modules.
pub fn emit(&mut self, inst: RvInst) {
        self.instructions.push(inst);
    }

        // Public function — callable from other modules.
pub fn emit_many(&mut self, insts: &[RvInst]) {
        self.instructions.extend_from_slice(insts);
    }
}

/// Translation statistics
#[derive(Debug, Default, Clone)]
// Public structure — visible outside this module.
pub struct TranslationStats {
    pub blocks_translated: u64,
    pub instructions_translated: u64,
    pub rv_instructions_emitted: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub unsupported_instructions: u64,
}

// Implementation block — defines methods for the type above.
impl TranslationStats {
        // Public function — callable from other modules.
pub fn expansion_ratio(&self) -> f64 {
        if self.instructions_translated == 0 {
            return 0.0;
        }
        self.rv_instructions_emitted as f64 / self.instructions_translated as f64
    }
}
