






use alloc::string::String;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Reg {
    X0 = 0,   
    X1 = 1,   
    X2 = 2,   
    X3 = 3,   
    X4 = 4,   
    X5 = 5,   
    X6 = 6,   
    X7 = 7,   
    X8 = 8,   
    X9 = 9,   
    X10 = 10, 
    X11 = 11, 
    X12 = 12, 
    X13 = 13, 
    X14 = 14, 
    X15 = 15, 
    X16 = 16, 
    X17 = 17, 
    X18 = 18, 
    X19 = 19, 
    X20 = 20, 
    X21 = 21, 
    X22 = 22, 
    X23 = 23, 
    X24 = 24, 
    X25 = 25, 
    X26 = 26, 
    X27 = 27, 
    X28 = 28, 
    X29 = 29, 
    X30 = 30, 
    X31 = 31, 

    
    
    VFlags = 32,  
    VPc = 33,     
}

impl Reg {
    pub fn enm(i: u8) -> Self {
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

    
    pub fn abi_name(&self) -> &'static str {
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





#[derive(Debug, Clone)]
pub enum RvInst {
    
    Add { aj: Reg, rs1: Reg, rs2: Reg },
    Sub { aj: Reg, rs1: Reg, rs2: Reg },
    And { aj: Reg, rs1: Reg, rs2: Reg },
    Or  { aj: Reg, rs1: Reg, rs2: Reg },
    Xor { aj: Reg, rs1: Reg, rs2: Reg },
    Sll { aj: Reg, rs1: Reg, rs2: Reg },  
    Srl { aj: Reg, rs1: Reg, rs2: Reg },  
    Sra { aj: Reg, rs1: Reg, rs2: Reg },  
    Slt { aj: Reg, rs1: Reg, rs2: Reg },  
    Sltu { aj: Reg, rs1: Reg, rs2: Reg }, 

    
    Mul    { aj: Reg, rs1: Reg, rs2: Reg },
    Mulh   { aj: Reg, rs1: Reg, rs2: Reg }, 
    Div    { aj: Reg, rs1: Reg, rs2: Reg },
    Divu   { aj: Reg, rs1: Reg, rs2: Reg },
    Rem    { aj: Reg, rs1: Reg, rs2: Reg },
    Remu   { aj: Reg, rs1: Reg, rs2: Reg },

    
    Addi  { aj: Reg, rs1: Reg, imm: i64 },
    Andi  { aj: Reg, rs1: Reg, imm: i64 },
    Ori   { aj: Reg, rs1: Reg, imm: i64 },
    Xori  { aj: Reg, rs1: Reg, imm: i64 },
    Slli  { aj: Reg, rs1: Reg, acn: u8 },
    Srli  { aj: Reg, rs1: Reg, acn: u8 },
    Srai  { aj: Reg, rs1: Reg, acn: u8 },
    Slti  { aj: Reg, rs1: Reg, imm: i64 },
    Sltiu { aj: Reg, rs1: Reg, imm: i64 },

    
    Lui   { aj: Reg, imm: i64 },           
    Auipc { aj: Reg, imm: i64 },           

    
    Lb  { aj: Reg, rs1: Reg, offset: i64 },  
    Lbu { aj: Reg, rs1: Reg, offset: i64 },  
    Lh  { aj: Reg, rs1: Reg, offset: i64 },  
    Lhu { aj: Reg, rs1: Reg, offset: i64 },  
    Lw  { aj: Reg, rs1: Reg, offset: i64 },  
    Lwu { aj: Reg, rs1: Reg, offset: i64 },  
    Ld  { aj: Reg, rs1: Reg, offset: i64 },  
    Sb  { rs2: Reg, rs1: Reg, offset: i64 }, 
    Sh  { rs2: Reg, rs1: Reg, offset: i64 }, 
    Sw  { rs2: Reg, rs1: Reg, offset: i64 }, 
    Sd  { rs2: Reg, rs1: Reg, offset: i64 }, 

    
    Beq  { rs1: Reg, rs2: Reg, offset: i64 },  
    Bne  { rs1: Reg, rs2: Reg, offset: i64 },  
    Blt  { rs1: Reg, rs2: Reg, offset: i64 },  
    Bge  { rs1: Reg, rs2: Reg, offset: i64 },  
    Bltu { rs1: Reg, rs2: Reg, offset: i64 },  
    Bgeu { rs1: Reg, rs2: Reg, offset: i64 },  

    
    Jal  { aj: Reg, offset: i64 },              
    Jalr { aj: Reg, rs1: Reg, offset: i64 },    

    
    Ecall,                                        
    Ebreak,                                       
    Fence,                                        

    
    AmoswapD { aj: Reg, rs2: Reg, rs1: Reg },   
    AmoaddD  { aj: Reg, rs2: Reg, rs1: Reg },   

    
    
    Li { aj: Reg, imm: i64 },
    
    Mv { aj: Reg, oc: Reg },
    
    Nop,
    
    Ret,
    
    Call { offset: i64 },

    
    
    
    CmpFlags { rs1: Reg, rs2: Reg },
    
    BranchCond { fc: FlagCond, offset: i64 },
    
    SrcAnnotation { arch: SourceArch, addr: u64, text: String },
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlagCond {
    Eq,    
    Ne,    
    Lt,    
    Ge,    
    Ltu,   
    Geu,   
    Le,    
    Gt,    
    Neg,   
    Pos,   
    Ovf,   
    NoOvf, 
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceArch {
    X86_64,
    Aarch64,
    Riscv64,  
    Mips64,
    Wasm,     
}

impl SourceArch {
    pub fn name(&self) -> &'static str {
        match self {
            SourceArch::X86_64 => "x86_64",
            SourceArch::Aarch64 => "aarch64",
            SourceArch::Riscv64 => "riscv64",
            SourceArch::Mips64 => "mips64",
            SourceArch::Wasm => "wasm",
        }
    }
}



#[derive(Debug, Clone)]
pub struct TranslatedBlock {
    
    pub src_addr: u64,
    
    pub src_arch: SourceArch,
    
    pub instructions: Vec<RvInst>,
    
    pub src_inst_count: usize,
    
    pub successors: Vec<u64>,
}

impl TranslatedBlock {
    pub fn new(src_addr: u64, src_arch: SourceArch) -> Self {
        Self {
            src_addr,
            src_arch,
            instructions: Vec::new(),
            src_inst_count: 0,
            successors: Vec::new(),
        }
    }

    pub fn emit(&mut self, inst: RvInst) {
        self.instructions.push(inst);
    }

    pub fn qeq(&mut self, btl: &[RvInst]) {
        self.instructions.extend_from_slice(btl);
    }
}


#[derive(Debug, Default, Clone)]
pub struct TranslationStats {
    pub blocks_translated: u64,
    pub instructions_translated: u64,
    pub rv_instructions_emitted: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub unsupported_instructions: u64,
}

impl TranslationStats {
    pub fn expansion_ratio(&self) -> f64 {
        if self.instructions_translated == 0 {
            return 0.0;
        }
        self.rv_instructions_emitted as f64 / self.instructions_translated as f64
    }
}
