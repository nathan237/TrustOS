// TrustOS Universal Architecture Translation Layer
//
// ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
// │  x86_64 ELF  │     │ AArch64 ELF  │     │  MIPS64 ELF  │
// └──────┬───────┘     └──────┬───────┘     └──────┬───────┘
//        │                    │                    │
//        ▼                    ▼                    ▼
// ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
// │ x86_decoder  │     │ arm_decoder  │     │ (future)     │
// └──────┬───────┘     └──────┬───────┘     └──────┬───────┘
//        │                    │                    │
//        └────────────┬───────┘────────────────────┘
//                     ▼
//        ┌────────────────────────┐
//        │    RISC-V IR (ir.rs)   │  ← Universal intermediate form
//        │  Clean, minimal ISA    │
//        └────────────┬───────────┘
//                     ▼
//        ┌────────────────────────┐
//        │  interpreter.rs        │  ← Executes on ANY host
//        │  (RvInterpreter)       │
//        └────────────┬───────────┘
//                     ▼
//        ┌────────────────────────┐
//        │  syscall.rs            │  ← Unified syscall handling
//        │  (UnifiedSyscall)      │
//        └────────────────────────┘
//
// Why RISC-V as IR?
//   - Simplest real-world ISA: load/store, fixed-width, register-register
//   - Open standard: no legal concerns
//   - Clean semantics: no complex flags register, no variable-length encoding
//   - Natural target: can emit real RISC-V machine code for JIT on RISC-V hosts
//   - 32 registers: enough to map x86 (16) and ARM (31) without spilling

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

/// Detect architecture from ELF header
pub fn detect_elf_arch(data: &[u8]) -> Option<SourceArch> {
    if data.len() < 20 || &data[0..4] != b"\x7FELF" {
        return None;
    }

    // e_machine field at offset 18 (2 bytes LE)
    let e_machine = u16::from_le_bytes([data[18], data[19]]);

    match e_machine {
        0x3E => Some(SourceArch::X86_64),    // EM_X86_64
        0xB7 => Some(SourceArch::Aarch64),   // EM_AARCH64
        0xF3 => Some(SourceArch::Riscv64),   // EM_RISCV
        0x08 => Some(SourceArch::Mips64),    // EM_MIPS
        _    => None,
    }
}

/// Extract code and entry point from ELF
fn extract_elf_code(data: &[u8]) -> Option<(u64, Vec<u8>)> {
    if data.len() < 64 || &data[0..4] != b"\x7FELF" {
        return None;
    }

    let entry_point = u64::from_le_bytes(data[24..32].try_into().ok()?);
    let ph_offset = u64::from_le_bytes(data[32..40].try_into().ok()?) as usize;
    let ph_size = u16::from_le_bytes(data[54..56].try_into().ok()?) as usize;
    let ph_count = u16::from_le_bytes(data[56..58].try_into().ok()?) as usize;

    // Find executable PT_LOAD segment
    for i in 0..ph_count {
        let ph = ph_offset + i * ph_size;
        if ph + ph_size > data.len() { break; }

        let p_type = u32::from_le_bytes(data[ph..ph+4].try_into().ok()?);
        let p_flags = u32::from_le_bytes(data[ph+4..ph+8].try_into().ok()?);

        if p_type == 1 && (p_flags & 1) != 0 {
            // PT_LOAD + PF_X
            let p_offset = u64::from_le_bytes(data[ph+8..ph+16].try_into().ok()?) as usize;
            let p_vaddr = u64::from_le_bytes(data[ph+16..ph+24].try_into().ok()?);
            let p_filesz = u64::from_le_bytes(data[ph+32..ph+40].try_into().ok()?) as usize;

            if p_offset + p_filesz <= data.len() {
                let code = data[p_offset..p_offset + p_filesz].to_vec();
                return Some((p_vaddr, code));
            }
        }
    }

    None
}

/// Run ANY architecture's ELF binary through the RISC-V translation layer
///
/// This is the main entry point. Give it an ELF binary from x86_64, ARM64,
/// or RISC-V and it will:
///   1. Detect the source architecture
///   2. Translate to RISC-V IR
///   3. Execute via the interpreter
///   4. Handle syscalls universally
pub fn translate_and_run(elf_data: &[u8]) -> Result<i64, String> {
    // 1. Detect architecture
    let arch = detect_elf_arch(elf_data)
        .ok_or_else(|| String::from("Not a valid ELF binary or unsupported architecture"))?;

    crate::serial_println!("[RV-XLAT] Detected architecture: {}", arch.name());

    // 2. Extract code
    let (base_addr, code) = extract_elf_code(elf_data)
        .ok_or_else(|| String::from("Failed to extract executable code from ELF"))?;

    crate::serial_println!("[RV-XLAT] Code: {} bytes at 0x{:X}", code.len(), base_addr);

    // 3. Translate to RISC-V IR
    let blocks = match arch {
        SourceArch::X86_64 => {
            let mut decoder = x86_decoder::X86Decoder::new(&code, base_addr);
            let blocks = decoder.translate_all();
            crate::serial_println!("[RV-XLAT] x86_64: {} blocks, {} instructions → {} RV instructions ({}x expansion)",
                decoder.stats.blocks_translated,
                decoder.stats.instructions_translated,
                decoder.stats.rv_instructions_emitted,
                if decoder.stats.instructions_translated > 0 {
                    format!("{:.1}", decoder.stats.expansion_ratio())
                } else {
                    String::from("N/A")
                });
            if decoder.stats.unsupported_instructions > 0 {
                crate::serial_println!("[RV-XLAT] WARNING: {} unsupported x86 instructions",
                    decoder.stats.unsupported_instructions);
            }
            blocks
        }
        SourceArch::Aarch64 => {
            let mut decoder = arm_decoder::ArmDecoder::new(&code, base_addr);
            let blocks = decoder.translate_all();
            crate::serial_println!("[RV-XLAT] aarch64: {} blocks, {} instructions → {} RV instructions ({}x expansion)",
                decoder.stats.blocks_translated,
                decoder.stats.instructions_translated,
                decoder.stats.rv_instructions_emitted,
                if decoder.stats.instructions_translated > 0 {
                    format!("{:.1}", decoder.stats.expansion_ratio())
                } else {
                    String::from("N/A")
                });
            blocks
        }
        SourceArch::Riscv64 => {
            // Native RISC-V — no translation needed!
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

    // 4. Set up interpreter
    let mut interp = RvInterpreter::new();

    // Map code into interpreter memory
    interp.mem.map_with_data(base_addr, &code);

    // Map stack (1 MB)
    let stack_base = 0x7FF0_0000u64;
    let stack_size = 0x10_0000; // 1 MB
    interp.mem.map(stack_base, stack_size);
    interp.cpu.set(Reg::X2, stack_base + stack_size as u64 - 16); // sp

    // Map heap area (4 MB)
    interp.mem.map(0x1000_0000, 0x40_0000);

    // Load translated blocks
    interp.load_blocks(&blocks);

    // Set up entry point via the first block
    let entry_block_addr = blocks[0].src_addr;
    interp.cpu.pc = entry_block_addr;

    crate::serial_println!("[RV-XLAT] Starting execution at 0x{:X}", entry_block_addr);

    // 5. Execute with syscall handling loop
    let mut exit_code: i64 = 0;

    loop {
        // Find and execute the block at current PC
        let pc = interp.cpu.pc;

        if let Some(block_insts) = interp.block_cache.get(&pc).map(|v| v.clone()) {
            let result = interp.exec_block(&block_insts);

            match result {
                ExecResult::Continue => {
                    // PC may have been updated by a branch — continue loop
                    if interp.cpu.pc == pc {
                        // Fell through — no more blocks
                        break;
                    }
                }
                ExecResult::Syscall { number, args } => {
                    let (ret, should_exit) = syscall::handle_syscall(
                        arch, number, &args, &mut interp.mem,
                    );

                    // Set return value in a0 (x10)
                    interp.cpu.set(Reg::X10, ret as u64);

                    if should_exit {
                        exit_code = args[0] as i64;
                        break;
                    }
                    // Continue from where we left off - need to find next block
                    // The syscall was mid-block, so PC should be set to continue
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
                    exit_code = -11; // SIGSEGV
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
            // No translated block at this PC
            crate::serial_println!("[RV-XLAT] No block at PC=0x{:X}, stopping", pc);
            break;
        }
    }

    crate::serial_println!("[RV-XLAT] Execution complete: {} instructions, exit code {}",
        interp.cpu.inst_count, exit_code);

    Ok(exit_code)
}

/// Translate and show RISC-V IR disassembly (without executing)
pub fn translate_and_disasm(elf_data: &[u8]) -> Result<String, String> {
    let arch = detect_elf_arch(elf_data)
        .ok_or_else(|| String::from("Not a valid ELF"))?;

    let (base_addr, code) = extract_elf_code(elf_data)
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
            output.push_str(&format!("  {:3}: {}\n", i, format_rv_inst(inst)));
        }
        output.push('\n');
    }

    Ok(output)
}

/// Format a RISC-V IR instruction for display
fn format_rv_inst(inst: &RvInst) -> String {
    match inst {
        RvInst::Add { rd, rs1, rs2 } => format!("add  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Sub { rd, rs1, rs2 } => format!("sub  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::And { rd, rs1, rs2 } => format!("and  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Or { rd, rs1, rs2 }  => format!("or   {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Xor { rd, rs1, rs2 } => format!("xor  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Sll { rd, rs1, rs2 } => format!("sll  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Srl { rd, rs1, rs2 } => format!("srl  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Sra { rd, rs1, rs2 } => format!("sra  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Slt { rd, rs1, rs2 } => format!("slt  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Sltu { rd, rs1, rs2 }=> format!("sltu {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Mul { rd, rs1, rs2 } => format!("mul  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Mulh { rd, rs1, rs2 }=> format!("mulh {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Div { rd, rs1, rs2 } => format!("div  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Divu { rd, rs1, rs2 }=> format!("divu {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Rem { rd, rs1, rs2 } => format!("rem  {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Remu { rd, rs1, rs2 }=> format!("remu {}, {}, {}", rd.abi_name(), rs1.abi_name(), rs2.abi_name()),
        RvInst::Addi { rd, rs1, imm } => format!("addi {}, {}, {}", rd.abi_name(), rs1.abi_name(), imm),
        RvInst::Andi { rd, rs1, imm } => format!("andi {}, {}, 0x{:X}", rd.abi_name(), rs1.abi_name(), *imm as u64),
        RvInst::Ori { rd, rs1, imm }  => format!("ori  {}, {}, 0x{:X}", rd.abi_name(), rs1.abi_name(), *imm as u64),
        RvInst::Xori { rd, rs1, imm } => format!("xori {}, {}, 0x{:X}", rd.abi_name(), rs1.abi_name(), *imm as u64),
        RvInst::Slli { rd, rs1, shamt }=> format!("slli {}, {}, {}", rd.abi_name(), rs1.abi_name(), shamt),
        RvInst::Srli { rd, rs1, shamt }=> format!("srli {}, {}, {}", rd.abi_name(), rs1.abi_name(), shamt),
        RvInst::Srai { rd, rs1, shamt }=> format!("srai {}, {}, {}", rd.abi_name(), rs1.abi_name(), shamt),
        RvInst::Slti { rd, rs1, imm } => format!("slti {}, {}, {}", rd.abi_name(), rs1.abi_name(), imm),
        RvInst::Sltiu { rd, rs1, imm }=> format!("sltiu {}, {}, {}", rd.abi_name(), rs1.abi_name(), imm),
        RvInst::Lui { rd, imm } => format!("lui  {}, 0x{:X}", rd.abi_name(), imm),
        RvInst::Auipc { rd, imm } => format!("auipc {}, 0x{:X}", rd.abi_name(), imm),
        RvInst::Lb { rd, rs1, offset } => format!("lb   {}, {}({})", rd.abi_name(), offset, rs1.abi_name()),
        RvInst::Lbu { rd, rs1, offset }=> format!("lbu  {}, {}({})", rd.abi_name(), offset, rs1.abi_name()),
        RvInst::Lh { rd, rs1, offset } => format!("lh   {}, {}({})", rd.abi_name(), offset, rs1.abi_name()),
        RvInst::Lhu { rd, rs1, offset }=> format!("lhu  {}, {}({})", rd.abi_name(), offset, rs1.abi_name()),
        RvInst::Lw { rd, rs1, offset } => format!("lw   {}, {}({})", rd.abi_name(), offset, rs1.abi_name()),
        RvInst::Lwu { rd, rs1, offset }=> format!("lwu  {}, {}({})", rd.abi_name(), offset, rs1.abi_name()),
        RvInst::Ld { rd, rs1, offset } => format!("ld   {}, {}({})", rd.abi_name(), offset, rs1.abi_name()),
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
        RvInst::Jal { rd, offset } => {
            if *rd as u8 == 0 {
                format!("j    0x{:X}", *offset as u64)
            } else {
                format!("jal  {}, 0x{:X}", rd.abi_name(), *offset as u64)
            }
        }
        RvInst::Jalr { rd, rs1, offset } => {
            if *rd as u8 == 0 && *offset == 0 {
                format!("jr   {}", rs1.abi_name())
            } else {
                format!("jalr {}, {}({})", rd.abi_name(), offset, rs1.abi_name())
            }
        }
        RvInst::Ecall  => String::from("ecall"),
        RvInst::Ebreak => String::from("ebreak"),
        RvInst::Fence  => String::from("fence"),
        RvInst::AmoswapD { rd, rs2, rs1 } => format!("amoswap.d {}, {}, ({})", rd.abi_name(), rs2.abi_name(), rs1.abi_name()),
        RvInst::AmoaddD { rd, rs2, rs1 }  => format!("amoadd.d  {}, {}, ({})", rd.abi_name(), rs2.abi_name(), rs1.abi_name()),
        RvInst::Li { rd, imm } => format!("li   {}, 0x{:X}", rd.abi_name(), *imm as u64),
        RvInst::Mv { rd, rs } => format!("mv   {}, {}", rd.abi_name(), rs.abi_name()),
        RvInst::Nop => String::from("nop"),
        RvInst::Ret => String::from("ret"),
        RvInst::Call { offset } => format!("call 0x{:X}", *offset as u64),
        RvInst::CmpFlags { rs1, rs2 } => format!("cmp  {}, {}", rs1.abi_name(), rs2.abi_name()),
        RvInst::BranchCond { cond, offset } => format!("b{:?} 0x{:X}", cond, *offset as u64),
        RvInst::SrcAnnotation { arch, addr, text } => format!("; [{} @ 0x{:X}] {}", arch.name(), addr, text),
    }
}
