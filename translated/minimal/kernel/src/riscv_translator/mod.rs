


































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


pub fn kpn(f: &[u8]) -> Option<SourceArch> {
    if f.len() < 20 || &f[0..4] != b"\x7FELF" {
        return None;
    }

    
    let cqb = u16::dj([f[18], f[19]]);

    match cqb {
        0x3E => Some(SourceArch::BT_),    
        0xB7 => Some(SourceArch::Fg),   
        0xF3 => Some(SourceArch::Jy),   
        0x08 => Some(SourceArch::Acz),    
        _    => None,
    }
}


fn nsk(f: &[u8]) -> Option<(u64, Vec<u8>)> {
    if f.len() < 64 || &f[0..4] != b"\x7FELF" {
        return None;
    }

    let mi = u64::dj(f[24..32].try_into().bq()?);
    let bnu = u64::dj(f[32..40].try_into().bq()?) as usize;
    let egq = u16::dj(f[54..56].try_into().bq()?) as usize;
    let egp = u16::dj(f[56..58].try_into().bq()?) as usize;

    
    for a in 0..egp {
        let afv = bnu + a * egq;
        if afv + egq > f.len() { break; }

        let bku = u32::dj(f[afv..afv+4].try_into().bq()?);
        let bvv = u32::dj(f[afv+4..afv+8].try_into().bq()?);

        if bku == 1 && (bvv & 1) != 0 {
            
            let caz = u64::dj(f[afv+8..afv+16].try_into().bq()?) as usize;
            let ctg = u64::dj(f[afv+16..afv+24].try_into().bq()?);
            let cgh = u64::dj(f[afv+32..afv+40].try_into().bq()?) as usize;

            if caz + cgh <= f.len() {
                let aj = f[caz..caz + cgh].ip();
                return Some((ctg, aj));
            }
        }
    }

    None
}









pub fn xlv(pu: &[u8]) -> Result<i64, String> {
    
    let arch = kpn(pu)
        .ok_or_else(|| String::from("Not a valid ELF binary or unsupported architecture"))?;

    crate::serial_println!("[RV-XLAT] Detected architecture: {}", arch.j());

    
    let (sm, aj) = nsk(pu)
        .ok_or_else(|| String::from("Failed to extract executable code from ELF"))?;

    crate::serial_println!("[RV-XLAT] Code: {} bytes at 0x{:X}", aj.len(), sm);

    
    let xk = match arch {
        SourceArch::BT_ => {
            let mut azm = x86_decoder::X86Decoder::new(&aj, sm);
            let xk = azm.iev();
            crate::serial_println!("[RV-XLAT] x86_64: {} blocks, {} instructions → {} RV instructions ({}x expansion)",
                azm.cm.ilv,
                azm.cm.esv,
                azm.cm.hyi,
                if azm.cm.esv > 0 {
                    format!("{:.1}", azm.cm.nrx())
                } else {
                    String::from("N/A")
                });
            if azm.cm.ddf > 0 {
                crate::serial_println!("[RV-XLAT] WARNING: {} unsupported x86 instructions",
                    azm.cm.ddf);
            }
            xk
        }
        SourceArch::Fg => {
            let mut azm = arm_decoder::ArmDecoder::new(&aj, sm);
            let xk = azm.iev();
            crate::serial_println!("[RV-XLAT] aarch64: {} blocks, {} instructions → {} RV instructions ({}x expansion)",
                azm.cm.ilv,
                azm.cm.esv,
                azm.cm.hyi,
                if azm.cm.esv > 0 {
                    format!("{:.1}", azm.cm.nrx())
                } else {
                    String::from("N/A")
                });
            xk
        }
        SourceArch::Jy => {
            
            crate::serial_println!("[RV-XLAT] RISC-V native binary — passthrough");
            return Err(String::from("RISC-V binaries can run natively, no translation needed"));
        }
        _ => {
            return Err(format!("Architecture {} translation not yet implemented", arch.j()));
        }
    };

    if xk.is_empty() {
        return Err(String::from("No code blocks translated"));
    }

    
    let mut ahp = RvInterpreter::new();

    
    ahp.mem.ujt(sm, &aj);

    
    let dce = 0x7FF0_0000u64;
    let ibn = 0x10_0000; 
    ahp.mem.map(dce, ibn);
    ahp.cpu.oj(Reg::Ds, dce + ibn as u64 - 16); 

    
    ahp.mem.map(0x1000_0000, 0x40_0000);

    
    ahp.ugl(&xk);

    
    let nqn = xk[0].cbz;
    ahp.cpu.fz = nqn;

    crate::serial_println!("[RV-XLAT] Starting execution at 0x{:X}", nqn);

    
    let mut nz: i64 = 0;

    loop {
        
        let fz = ahp.cpu.fz;

        if let Some(kea) = ahp.block_cache.get(&fz).map(|p| p.clone()) {
            let result = ahp.nrj(&kea);

            match result {
                ExecResult::Cg => {
                    
                    if ahp.cpu.fz == fz {
                        
                        break;
                    }
                }
                ExecResult::Hg { aqb, n } => {
                    let (aux, wna) = syscall::ixo(
                        arch, aqb, &n, &mut ahp.mem,
                    );

                    
                    ahp.cpu.oj(Reg::Je, aux as u64);

                    if wna {
                        nz = n[0] as i64;
                        break;
                    }
                    
                    
                }
                ExecResult::Amb(ap) => {
                    nz = ap as i64;
                    break;
                }
                ExecResult::Bcu => {
                    crate::serial_println!("[RV-XLAT] Breakpoint at 0x{:X}", ahp.cpu.fz);
                    crate::serial_println!("{}", ahp.noi());
                    break;
                }
                ExecResult::Hw(ag) => {
                    crate::serial_println!("[RV-XLAT] SEGFAULT: memory access at 0x{:X}", ag);
                    crate::serial_println!("{}", ahp.noi());
                    nz = -11; 
                    break;
                }
                ExecResult::Auj => {
                    crate::serial_println!("[RV-XLAT] Instruction limit reached ({} instructions)",
                        ahp.cpu.flq);
                    break;
                }
                ExecResult::Ceu => break,
            }
        } else {
            
            crate::serial_println!("[RV-XLAT] No block at PC=0x{:X}, stopping", fz);
            break;
        }
    }

    crate::serial_println!("[RV-XLAT] Execution complete: {} instructions, exit code {}",
        ahp.cpu.flq, nz);

    Ok(nz)
}


pub fn pvz(pu: &[u8]) -> Result<String, String> {
    let arch = kpn(pu)
        .ok_or_else(|| String::from("Not a valid ELF"))?;

    let (sm, aj) = nsk(pu)
        .ok_or_else(|| String::from("No executable code"))?;

    let xk = match arch {
        SourceArch::BT_ => {
            let mut bc = x86_decoder::X86Decoder::new(&aj, sm);
            bc.iev()
        }
        SourceArch::Fg => {
            let mut bc = arm_decoder::ArmDecoder::new(&aj, sm);
            bc.iev()
        }
        _ => return Err(format!("{} not supported for disasm", arch.j())),
    };

    let mut an = format!("=== RISC-V IR Translation ({} → rv64) ===\n", arch.j());
    an.t(&format!("Entry: 0x{:X} | {} blocks\n\n", sm, xk.len()));

    for block in &xk {
        an.t(&format!("Block @ 0x{:X} ({} src instructions → {} RV IR):\n",
            block.cbz, block.jrg, block.instructions.len()));

        for (a, fi) in block.instructions.iter().cf() {
            an.t(&format!("  {:3}: {}\n", a, swa(fi)));
        }
        an.push('\n');
    }

    Ok(an)
}


fn swa(fi: &RvInst) -> String {
    match fi {
        RvInst::Add { ck, cp, et } => format!("add  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Sub { ck, cp, et } => format!("sub  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Ex { ck, cp, et } => format!("and  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Fx { ck, cp, et }  => format!("or   {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Aga { ck, cp, et } => format!("xor  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Amt { ck, cp, et } => format!("sll  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Amx { ck, cp, et } => format!("srl  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Azc { ck, cp, et } => format!("sra  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Btb { ck, cp, et } => format!("slt  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Bte { ck, cp, et }=> format!("sltu {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Mul { ck, cp, et } => format!("mul  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Bms { ck, cp, et }=> format!("mulh {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Div { ck, cp, et } => format!("div  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Arb { ck, cp, et }=> format!("divu {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Rem { ck, cp, et } => format!("rem  {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Bqx { ck, cp, et }=> format!("remu {}, {}, {}", ck.kj(), cp.kj(), et.kj()),
        RvInst::Gf { ck, cp, gf } => format!("addi {}, {}, {}", ck.kj(), cp.kj(), gf),
        RvInst::Ou { ck, cp, gf } => format!("andi {}, {}, 0x{:X}", ck.kj(), cp.kj(), *gf as u64),
        RvInst::Akw { ck, cp, gf }  => format!("ori  {}, {}, 0x{:X}", ck.kj(), cp.kj(), *gf as u64),
        RvInst::Aoq { ck, cp, gf } => format!("xori {}, {}, 0x{:X}", ck.kj(), cp.kj(), *gf as u64),
        RvInst::Ayv { ck, cp, bcp }=> format!("slli {}, {}, {}", ck.kj(), cp.kj(), bcp),
        RvInst::Aze { ck, cp, bcp }=> format!("srli {}, {}, {}", ck.kj(), cp.kj(), bcp),
        RvInst::Azd { ck, cp, bcp }=> format!("srai {}, {}, {}", ck.kj(), cp.kj(), bcp),
        RvInst::Btc { ck, cp, gf } => format!("slti {}, {}, {}", ck.kj(), cp.kj(), gf),
        RvInst::Btd { ck, cp, gf }=> format!("sltiu {}, {}, {}", ck.kj(), cp.kj(), gf),
        RvInst::Blq { ck, gf } => format!("lui  {}, 0x{:X}", ck.kj(), gf),
        RvInst::Bce { ck, gf } => format!("auipc {}, 0x{:X}", ck.kj(), gf),
        RvInst::Bky { ck, cp, l } => format!("lb   {}, {}({})", ck.kj(), l, cp.kj()),
        RvInst::Ajr { ck, cp, l }=> format!("lbu  {}, {}({})", ck.kj(), l, cp.kj()),
        RvInst::Bla { ck, cp, l } => format!("lh   {}, {}({})", ck.kj(), l, cp.kj()),
        RvInst::Ajs { ck, cp, l }=> format!("lhu  {}, {}({})", ck.kj(), l, cp.kj()),
        RvInst::Blr { ck, cp, l } => format!("lw   {}, {}({})", ck.kj(), l, cp.kj()),
        RvInst::Aka { ck, cp, l }=> format!("lwu  {}, {}({})", ck.kj(), l, cp.kj()),
        RvInst::Pt { ck, cp, l } => format!("ld   {}, {}({})", ck.kj(), l, cp.kj()),
        RvInst::Amf { et, cp, l }=> format!("sb   {}, {}({})", et.kj(), l, cp.kj()),
        RvInst::Amo { et, cp, l }=> format!("sh   {}, {}({})", et.kj(), l, cp.kj()),
        RvInst::Ang { et, cp, l }=> format!("sw   {}, {}({})", et.kj(), l, cp.kj()),
        RvInst::Mi { et, cp, l }=> format!("sd   {}, {}({})", et.kj(), l, cp.kj()),
        RvInst::Agp { cp, et, l }  => format!("beq  {}, {}, 0x{:X}", cp.kj(), et.kj(), *l as u64),
        RvInst::Ags { cp, et, l }  => format!("bne  {}, {}, 0x{:X}", cp.kj(), et.kj(), *l as u64),
        RvInst::Bcr { cp, et, l }  => format!("blt  {}, {}, 0x{:X}", cp.kj(), et.kj(), *l as u64),
        RvInst::Bcp { cp, et, l }  => format!("bge  {}, {}, 0x{:X}", cp.kj(), et.kj(), *l as u64),
        RvInst::Bcs { cp, et, l } => format!("bltu {}, {}, 0x{:X}", cp.kj(), et.kj(), *l as u64),
        RvInst::Bcq { cp, et, l } => format!("bgeu {}, {}, 0x{:X}", cp.kj(), et.kj(), *l as u64),
        RvInst::Xh { ck, l } => {
            if *ck as u8 == 0 {
                format!("j    0x{:X}", *l as u64)
            } else {
                format!("jal  {}, 0x{:X}", ck.kj(), *l as u64)
            }
        }
        RvInst::Xi { ck, cp, l } => {
            if *ck as u8 == 0 && *l == 0 {
                format!("jr   {}", cp.kj())
            } else {
                format!("jalr {}, {}({})", ck.kj(), l, cp.kj())
            }
        }
        RvInst::Wk  => String::from("ecall"),
        RvInst::Bfr => String::from("ebreak"),
        RvInst::Bgw  => String::from("fence"),
        RvInst::Bbr { ck, et, cp } => format!("amoswap.d {}, {}, ({})", ck.kj(), et.kj(), cp.kj()),
        RvInst::Bbq { ck, et, cp }  => format!("amoadd.d  {}, {}, ({})", ck.kj(), et.kj(), cp.kj()),
        RvInst::Hu { ck, gf } => format!("li   {}, 0x{:X}", ck.kj(), *gf as u64),
        RvInst::Gl { ck, acl } => format!("mv   {}, {}", ck.kj(), acl.kj()),
        RvInst::Fq => String::from("nop"),
        RvInst::Ama => String::from("ret"),
        RvInst::En { l } => format!("call 0x{:X}", *l as u64),
        RvInst::Ed { cp, et } => format!("cmp  {}, {}", cp.kj(), et.kj()),
        RvInst::Aad { mo, l } => format!("b{:?} 0x{:X}", mo, *l as u64),
        RvInst::Od { arch, ag, text } => format!("; [{} @ 0x{:X}] {}", arch.j(), ag, text),
    }
}
