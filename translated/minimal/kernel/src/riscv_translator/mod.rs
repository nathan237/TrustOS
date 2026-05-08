


































pub mod ir;
pub mod x86_decoder;
pub mod arm_decoder;
pub mod interpreter;
pub mod syscall;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

use ir::*;
use interpreter::{RvInterpreter, ExecResult};


pub fn frv(data: &[u8]) -> Option<SourceArch> {
    if data.len() < 20 || &data[0..4] != b"\x7FELF" {
        return None;
    }

    
    let e_machine = u16::from_le_bytes([data[18], data[19]]);

    match e_machine {
        0x3E => Some(SourceArch::X86_64),    
        0xB7 => Some(SourceArch::Aarch64),   
        0xF3 => Some(SourceArch::Riscv64),   
        0x08 => Some(SourceArch::Mips64),    
        _    => None,
    }
}


fn hxm(data: &[u8]) -> Option<(u64, Vec<u8>)> {
    if data.len() < 64 || &data[0..4] != b"\x7FELF" {
        return None;
    }

    let entry_point = u64::from_le_bytes(data[24..32].try_into().ok()?);
    let aii = u64::from_le_bytes(data[32..40].try_into().ok()?) as usize;
    let but = u16::from_le_bytes(data[54..56].try_into().ok()?) as usize;
    let bur = u16::from_le_bytes(data[56..58].try_into().ok()?) as usize;

    
    for i in 0..bur {
        let qc = aii + i * but;
        if qc + but > data.len() { break; }

        let p_type = u32::from_le_bytes(data[qc..qc+4].try_into().ok()?);
        let p_flags = u32::from_le_bytes(data[qc+4..qc+8].try_into().ok()?);

        if p_type == 1 && (p_flags & 1) != 0 {
            
            let p_offset = u64::from_le_bytes(data[qc+8..qc+16].try_into().ok()?) as usize;
            let p_vaddr = u64::from_le_bytes(data[qc+16..qc+24].try_into().ok()?);
            let p_filesz = u64::from_le_bytes(data[qc+32..qc+40].try_into().ok()?) as usize;

            if p_offset + p_filesz <= data.len() {
                let code = data[p_offset..p_offset + p_filesz].to_vec();
                return Some((p_vaddr, code));
            }
        }
    }

    None
}









pub fn pne(gz: &[u8]) -> Result<i64, String> {
    
    let arch = frv(gz)
        .ok_or_else(|| String::from("Not a valid ELF binary or unsupported architecture"))?;

    crate::serial_println!("[RV-XLAT] Detected architecture: {}", arch.name());

    
    let (base_addr, code) = hxm(gz)
        .ok_or_else(|| String::from("Failed to extract executable code from ELF"))?;

    crate::serial_println!("[RV-XLAT] Code: {} bytes at 0x{:X}", code.len(), base_addr);

    
    let blocks = match arch {
        SourceArch::X86_64 => {
            let mut aaq = x86_decoder::X86Decoder::new(&code, base_addr);
            let blocks = aaq.translate_all();
            crate::serial_println!("[RV-XLAT] x86_64: {} blocks, {} instructions → {} RV instructions ({}x expansion)",
                aaq.stats.blocks_translated,
                aaq.stats.instructions_translated,
                aaq.stats.rv_instructions_emitted,
                if aaq.stats.instructions_translated > 0 {
                    format!("{:.1}", aaq.stats.expansion_ratio())
                } else {
                    String::from("N/A")
                });
            if aaq.stats.unsupported_instructions > 0 {
                crate::serial_println!("[RV-XLAT] WARNING: {} unsupported x86 instructions",
                    aaq.stats.unsupported_instructions);
            }
            blocks
        }
        SourceArch::Aarch64 => {
            let mut aaq = arm_decoder::ArmDecoder::new(&code, base_addr);
            let blocks = aaq.translate_all();
            crate::serial_println!("[RV-XLAT] aarch64: {} blocks, {} instructions → {} RV instructions ({}x expansion)",
                aaq.stats.blocks_translated,
                aaq.stats.instructions_translated,
                aaq.stats.rv_instructions_emitted,
                if aaq.stats.instructions_translated > 0 {
                    format!("{:.1}", aaq.stats.expansion_ratio())
                } else {
                    String::from("N/A")
                });
            blocks
        }
        SourceArch::Riscv64 => {
            
            crate::serial_println!("[RV-XLAT] RISC-V native binary — passthrough");
            return Err(String::from("RISC-V binaries can run natively, no translation needed"));
        }
        _ => {
            return Err(format!("Architecture {} translation not yet implemented", arch.name()));
        }
    };

    if blocks.is_empty() {
        return Err(String::from("No code blocks translated"));
    }

    
    let mut interp = RvInterpreter::new();

    
    interp.mem.map_with_data(base_addr, &code);

    
    let bdt = 0x7FF0_0000u64;
    let eah = 0x10_0000; 
    interp.mem.map(bdt, eah);
    interp.cpu.set(Reg::X2, bdt + eah as u64 - 16); 

    
    interp.mem.map(0x1000_0000, 0x40_0000);

    
    interp.load_blocks(&blocks);

    
    let hwf = blocks[0].src_addr;
    interp.cpu.pc = hwf;

    crate::serial_println!("[RV-XLAT] Starting execution at 0x{:X}", hwf);

    
    let mut exit_code: i64 = 0;

    loop {
        
        let pc = interp.cpu.pc;

        if let Some(block_insts) = interp.block_cache.get(&pc).map(|v| v.clone()) {
            let result = interp.exec_block(&block_insts);

            match result {
                ExecResult::Continue => {
                    
                    if interp.cpu.pc == pc {
                        
                        break;
                    }
                }
                ExecResult::Syscall { number, args } => {
                    let (ret, should_exit) = syscall::handle_syscall(
                        arch, number, &args, &mut interp.mem,
                    );

                    
                    interp.cpu.set(Reg::X10, ret as u64);

                    if should_exit {
                        exit_code = args[0] as i64;
                        break;
                    }
                    
                    
                }
                ExecResult::Returned(val) => {
                    exit_code = val as i64;
                    break;
                }
                ExecResult::Breakpoint => {
                    crate::serial_println!("[RV-XLAT] Breakpoint at 0x{:X}", interp.cpu.pc);
                    crate::serial_println!("{}", interp.dump_state());
                    break;
                }
                ExecResult::MemoryFault(addr) => {
                    crate::serial_println!("[RV-XLAT] SEGFAULT: memory access at 0x{:X}", addr);
                    crate::serial_println!("{}", interp.dump_state());
                    exit_code = -11; 
                    break;
                }
                ExecResult::InstructionLimit => {
                    crate::serial_println!("[RV-XLAT] Instruction limit reached ({} instructions)",
                        interp.cpu.inst_count);
                    break;
                }
                ExecResult::Halted => break,
            }
        } else {
            
            crate::serial_println!("[RV-XLAT] No block at PC=0x{:X}, stopping", pc);
            break;
        }
    }

    crate::serial_println!("[RV-XLAT] Execution complete: {} instructions, exit code {}",
        interp.cpu.inst_count, exit_code);

    Ok(exit_code)
}


pub fn jon(gz: &[u8]) -> Result<String, String> {
    let arch = frv(gz)
        .ok_or_else(|| String::from("Not a valid ELF"))?;

    let (base_addr, code) = hxm(gz)
        .ok_or_else(|| String::from("No executable code"))?;

    let blocks = match arch {
        SourceArch::X86_64 => {
            let mut d = x86_decoder::X86Decoder::new(&code, base_addr);
            d.translate_all()
        }
        SourceArch::Aarch64 => {
            let mut d = arm_decoder::ArmDecoder::new(&code, base_addr);
            d.translate_all()
        }
        _ => return Err(format!("{} not supported for disasm", arch.name())),
    };

    let mut output = format!("=== RISC-V IR Translation ({} → rv64) ===\n", arch.name());
    output.push_str(&format!("Entry: 0x{:X} | {} blocks\n\n", base_addr, blocks.len()));

    for block in &blocks {
        output.push_str(&format!("Block @ 0x{:X} ({} src instructions → {} RV IR):\n",
            block.src_addr, block.src_inst_count, block.instructions.len()));

        for (i, inst) in block.instructions.iter().enumerate() {
            output.push_str(&format!("  {:3}: {}\n", i, lxt(inst)));
        }
        output.push('\n');
    }

    Ok(output)
}


fn lxt(inst: &RvInst) -> String {
    match inst {
        RvInst::Add { aj, rs1, rs2 } => format!("add  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Sub { aj, rs1, rs2 } => format!("sub  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::And { aj, rs1, rs2 } => format!("and  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Or { aj, rs1, rs2 }  => format!("or   {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Xor { aj, rs1, rs2 } => format!("xor  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Sll { aj, rs1, rs2 } => format!("sll  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Srl { aj, rs1, rs2 } => format!("srl  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Sra { aj, rs1, rs2 } => format!("sra  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Slt { aj, rs1, rs2 } => format!("slt  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Sltu { aj, rs1, rs2 }=> format!("sltu {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Mul { aj, rs1, rs2 } => format!("mul  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Mulh { aj, rs1, rs2 }=> format!("mulh {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Div { aj, rs1, rs2 } => format!("div  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Divu { aj, rs1, rs2 }=> format!("divu {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Rem { aj, rs1, rs2 } => format!("rem  {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Remu { aj, rs1, rs2 }=> format!("remu {}, {}, {}", aj.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Addi { aj, rs1, imm } => format!("addi {}, {}, {}", aj.abi_name(), rs1.abi_name(), imm),
        RvInst::Andi { aj, rs1, imm } => format!("andi {}, {}, 0x{:X}", aj.abi_name(), rs1.abi_name(), *imm as u64),
        RvInst::Ori { aj, rs1, imm }  => format!("ori  {}, {}, 0x{:X}", aj.abi_name(), rs1.abi_name(), *imm as u64),
        RvInst::Xori { aj, rs1, imm } => format!("xori {}, {}, 0x{:X}", aj.abi_name(), rs1.abi_name(), *imm as u64),
        RvInst::Slli { aj, rs1, acn }=> format!("slli {}, {}, {}", aj.abi_name(), rs1.abi_name(), acn),
        RvInst::Srli { aj, rs1, acn }=> format!("srli {}, {}, {}", aj.abi_name(), rs1.abi_name(), acn),
        RvInst::Srai { aj, rs1, acn }=> format!("srai {}, {}, {}", aj.abi_name(), rs1.abi_name(), acn),
        RvInst::Slti { aj, rs1, imm } => format!("slti {}, {}, {}", aj.abi_name(), rs1.abi_name(), imm),
        RvInst::Sltiu { aj, rs1, imm }=> format!("sltiu {}, {}, {}", aj.abi_name(), rs1.abi_name(), imm),
        RvInst::Lui { aj, imm } => format!("lui  {}, 0x{:X}", aj.abi_name(), imm),
        RvInst::Auipc { aj, imm } => format!("auipc {}, 0x{:X}", aj.abi_name(), imm),
        RvInst::Lb { aj, rs1, offset } => format!("lb   {}, {}({})", aj.abi_name(), offset, rs1.abi_name()),
        RvInst::Lbu { aj, rs1, offset }=> format!("lbu  {}, {}({})", aj.abi_name(), offset, rs1.abi_name()),
        RvInst::Lh { aj, rs1, offset } => format!("lh   {}, {}({})", aj.abi_name(), offset, rs1.abi_name()),
        RvInst::Lhu { aj, rs1, offset }=> format!("lhu  {}, {}({})", aj.abi_name(), offset, rs1.abi_name()),
        RvInst::Lw { aj, rs1, offset } => format!("lw   {}, {}({})", aj.abi_name(), offset, rs1.abi_name()),
        RvInst::Lwu { aj, rs1, offset }=> format!("lwu  {}, {}({})", aj.abi_name(), offset, rs1.abi_name()),
        RvInst::Ld { aj, rs1, offset } => format!("ld   {}, {}({})", aj.abi_name(), offset, rs1.abi_name()),
        RvInst::Sb { rs2, rs1, offset }=> format!("sb   {}, {}({})", rs2.abi_name(), offset, rs1.abi_name()),
        RvInst::Sh { rs2, rs1, offset }=> format!("sh   {}, {}({})", rs2.abi_name(), offset, rs1.abi_name()),
        RvInst::Sw { rs2, rs1, offset }=> format!("sw   {}, {}({})", rs2.abi_name(), offset, rs1.abi_name()),
        RvInst::Sd { rs2, rs1, offset }=> format!("sd   {}, {}({})", rs2.abi_name(), offset, rs1.abi_name()),
        RvInst::Beq { rs1, rs2, offset }  => format!("beq  {}, {}, 0x{:X}", rs1.abi_name(), rs2.abi_name(), *offset as u64),
        RvInst::Bne { rs1, rs2, offset }  => format!("bne  {}, {}, 0x{:X}", rs1.abi_name(), rs2.abi_name(), *offset as u64),
        RvInst::Blt { rs1, rs2, offset }  => format!("blt  {}, {}, 0x{:X}", rs1.abi_name(), rs2.abi_name(), *offset as u64),
        RvInst::Bge { rs1, rs2, offset }  => format!("bge  {}, {}, 0x{:X}", rs1.abi_name(), rs2.abi_name(), *offset as u64),
        RvInst::Bltu { rs1, rs2, offset } => format!("bltu {}, {}, 0x{:X}", rs1.abi_name(), rs2.abi_name(), *offset as u64),
        RvInst::Bgeu { rs1, rs2, offset } => format!("bgeu {}, {}, 0x{:X}", rs1.abi_name(), rs2.abi_name(), *offset as u64),
        RvInst::Jal { aj, offset } => {
            if *aj as u8 == 0 {
                format!("j    0x{:X}", *offset as u64)
            } else {
                format!("jal  {}, 0x{:X}", aj.abi_name(), *offset as u64)
            }
        }
        RvInst::Jalr { aj, rs1, offset } => {
            if *aj as u8 == 0 && *offset == 0 {
                format!("jr   {}", rs1.abi_name())
            } else {
                format!("jalr {}, {}({})", aj.abi_name(), offset, rs1.abi_name())
            }
        }
        RvInst::Ecall  => String::from("ecall"),
        RvInst::Ebreak => String::from("ebreak"),
        RvInst::Fence  => String::from("fence"),
        RvInst::AmoswapD { aj, rs2, rs1 } => format!("amoswap.d {}, {}, ({})", aj.abi_name(), rs2.abi_name(), rs1.abi_name()),
        RvInst::AmoaddD { aj, rs2, rs1 }  => format!("amoadd.d  {}, {}, ({})", aj.abi_name(), rs2.abi_name(), rs1.abi_name()),
        RvInst::Li { aj, imm } => format!("li   {}, 0x{:X}", aj.abi_name(), *imm as u64),
        RvInst::Mv { aj, oc } => format!("mv   {}, {}", aj.abi_name(), oc.abi_name()),
        RvInst::Nop => String::from("nop"),
        RvInst::Ret => String::from("ret"),
        RvInst::Call { offset } => format!("call 0x{:X}", *offset as u64),
        RvInst::CmpFlags { rs1, rs2 } => format!("cmp  {}, {}", rs1.abi_name(), rs2.abi_name()),
        RvInst::BranchCond { fc, offset } => format!("b{:?} 0x{:X}", fc, *offset as u64),
        RvInst::SrcAnnotation { arch, addr, text } => format!("; [{} @ 0x{:X}] {}", arch.name(), addr, text),
    }
}
