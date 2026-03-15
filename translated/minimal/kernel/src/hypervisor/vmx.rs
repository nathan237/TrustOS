








use core::arch::asm;
use alloc::boxed::Box;
use super::{HypervisorError, Result, Afp};


#[repr(C, align(4096))]
pub struct VmxonRegion {
    pub cty: u32,
    pub f: [u8; 4092],
}

impl VmxonRegion {
    pub fn new(cty: u32) -> Self {
        VmxonRegion {
            cty,
            f: [0; 4092],
        }
    }
}


static mut BIN_: Option<Box<VmxonRegion>> = None;





pub const TZ_: u32 = 0x480;
pub const CBA_: u32 = 0x481;
pub const AWP_: u32 = 0x482;
pub const CAZ_: u32 = 0x483;
pub const CAY_: u32 = 0x484;
pub const AWQ_: u32 = 0x48B;
pub const CBD_: u32 = 0x48D;
pub const CBE_: u32 = 0x48E;
pub const CBC_: u32 = 0x48F;
pub const CBB_: u32 = 0x490;
pub const CAS_: u32 = 0x3A;




pub fn gyb(msr: u32, rwe: u32) -> u32 {
    let oom = bcg(msr);
    let uqr = oom as u32;        
    let ume = (oom >> 32) as u32; 
    
    (rwe | uqr) & ume
}


pub fn ixw() -> bool {
    let fyj = bcg(TZ_);
    (fyj & (1u64 << 55)) != 0
}


pub fn vhz() -> u32 {
    if ixw() { CBD_ } else { CBA_ }
}


pub fn vmk() -> u32 {
    if ixw() { CBE_ } else { AWP_ }
}


pub fn soy() -> u32 {
    if ixw() { CBC_ } else { CAZ_ }
}


pub fn sma() -> u32 {
    if ixw() { CBB_ } else { CAY_ }
}


pub fn dmy(ju: u64) -> u64 {
    crate::memory::abw(ju).unwrap_or(ju)
}


pub fn inj() -> Result<Afp> {
    let mut dr = Afp {
        dme: false,
        fhw: false,
        gvo: false,
        gwj: false,
        igr: 0,
    };
    
    
    let gdr: u32;
    unsafe {
        asm!(
            "push rbx",       
            "mov eax, 1",
            "cpuid",
            "mov {0:e}, ecx",
            "pop rbx",        
            bd(reg) gdr,
            bd("eax") _,
            bd("ecx") _,
            bd("edx") _,
        );
    }
    
    dr.dme = (gdr & (1 << 5)) != 0;
    
    if !dr.dme {
        return Ok(dr);
    }
    
    
    let fyj = bcg(TZ_);
    dr.igr = (fyj & 0x7FFF_FFFF) as u32;
    
    
    
    let vmi = bcg(AWP_);
    let wfv = (vmi >> 32) & (1 << 31) != 0;
    
    if wfv {
        let vmj = bcg(AWQ_);
        let kag = (vmj >> 32) as u32;
        
        dr.fhw = (kag & (1 << 1)) != 0;           
        dr.gvo = (kag & (1 << 7)) != 0;      
        dr.gwj = (kag & (1 << 5)) != 0;          
    }
    
    Ok(dr)
}


pub fn slg() -> Result<()> {
    
    let kvk = bcg(CAS_);
    
    
    
    if (kvk & 1) != 0 {
        
        if (kvk & (1 << 2)) == 0 {
            crate::serial_println!("[VMX] ERROR: VMX disabled by BIOS (locked)");
            return Err(HypervisorError::Bwd);
        }
    } else {
        
        let loe = kvk | (1 << 2) | 1; 
        fbs(0x3A, loe);
        crate::serial_println!("[VMX] Enabled VMX in IA32_FEATURE_CONTROL");
    }
    
    
    unsafe {
        asm!(
            "mov rax, cr4",
            "or rax, 0x2000",  
            "mov cr4, rax",
            bd("rax") _,
        );
    }
    
    crate::serial_println!("[VMX] CR4.VMXE enabled");
    
    Ok(())
}


pub fn xsr() -> Result<()> {
    
    let fyj = bcg(TZ_);
    let cty = (fyj & 0x7FFF_FFFF) as u32;
    
    
    let aoz = Box::new(VmxonRegion::new(cty));
    let pbl = aoz.as_ref() as *const VmxonRegion as u64;
    let pbk = dmy(pbl);
    
    crate::serial_println!("[VMX] VMXON region virt=0x{:016X} phys=0x{:016X}, revision 0x{:08X}", 
                          pbl, pbk, cty);
    
    
    unsafe {
        BIN_ = Some(aoz);
    }
    
    
    let vq: u8;
    let aca: u8;
    unsafe {
        asm!(
            "vmxon [{addr}]",
            "setc {cf}",
            "setz {zf}",
            ag = in(reg) &pbk,
            vq = bd(reg_byte) vq,
            aca = bd(reg_byte) aca,
        );
    }
    
    if vq != 0 || aca != 0 {
        crate::serial_println!("[VMX] VMXON failed! CF={} ZF={}", vq, aca);
        return Err(HypervisorError::Cpv);
    }
    
    crate::serial_println!("[VMX] VMXON successful - Now in VMX root operation");
    
    Ok(())
}


pub fn xsq() -> Result<()> {
    unsafe {
        asm!("vmxoff");
    }
    
    
    unsafe {
        asm!(
            "mov rax, cr4",
            "and rax, ~0x2000",  
            "mov cr4, rax",
            bd("rax") _,
        );
    }
    
    
    unsafe {
        BIN_ = None;
    }
    
    crate::serial_println!("[VMX] VMXOFF complete - Left VMX operation");
    
    Ok(())
}



pub fn mps(igq: u64) -> Result<()> {
    let vq: u8;
    let aca: u8;
    unsafe {
        asm!(
            "vmclear [{addr}]",
            "setc {cf}",
            "setz {zf}",
            ag = in(reg) &igq,
            vq = bd(reg_byte) vq,
            aca = bd(reg_byte) aca,
        );
    }
    
    if vq != 0 || aca != 0 {
        crate::serial_println!("[VMX] VMCLEAR failed for phys=0x{:X} CF={} ZF={}", igq, vq, aca);
        return Err(HypervisorError::Cpq);
    }
    
    Ok(())
}



pub fn pyp(igq: u64) -> Result<()> {
    let vq: u8;
    let aca: u8;
    unsafe {
        asm!(
            "vmptrld [{addr}]",
            "setc {cf}",
            "setz {zf}",
            ag = in(reg) &igq,
            vq = bd(reg_byte) vq,
            aca = bd(reg_byte) aca,
        );
    }
    
    if vq != 0 || aca != 0 {
        crate::serial_println!("[VMX] VMPTRLD failed for phys=0x{:X} CF={} ZF={}", igq, vq, aca);
        return Err(HypervisorError::Cpr);
    }
    
    Ok(())
}


pub fn igs(buj: u64) -> Result<u64> {
    let bn: u64;
    let vq: u8;
    let aca: u8;
    
    unsafe {
        asm!(
            "vmread {val}, {field}",
            "setc {cf}",
            "setz {zf}",
            ap = bd(reg) bn,
            buj = in(reg) buj,
            vq = bd(reg_byte) vq,
            aca = bd(reg_byte) aca,
        );
    }
    
    if vq != 0 || aca != 0 {
        return Err(HypervisorError::Cps);
    }
    
    Ok(bn)
}


pub fn pyr(buj: u64, bn: u64) -> Result<()> {
    let vq: u8;
    let aca: u8;
    
    unsafe {
        asm!(
            "vmwrite {field}, {val}",
            "setc {cf}",
            "setz {zf}",
            buj = in(reg) buj,
            ap = in(reg) bn,
            vq = bd(reg_byte) vq,
            aca = bd(reg_byte) aca,
        );
    }
    
    if vq != 0 || aca != 0 {
        return Err(HypervisorError::Cpt);
    }
    
    Ok(())
}




pub fn zvn() -> Result<()> {
    let vq: u8;
    let aca: u8;
    
    unsafe {
        asm!(
            "vmlaunch",
            "setc {cf}",
            "setz {zf}",
            vq = bd(reg_byte) vq,
            aca = bd(reg_byte) aca,
        );
    }
    
    
    let error_code = igs(0x4400).unwrap_or(0xFFFF); 
    crate::serial_println!("[VMX] VMLAUNCH failed! CF={} ZF={} error={}", vq, aca, error_code);
    Err(HypervisorError::Bvz)
}


pub fn zvp() -> Result<()> {
    let vq: u8;
    let aca: u8;
    
    unsafe {
        asm!(
            "vmresume",
            "setc {cf}",
            "setz {zf}",
            vq = bd(reg_byte) vq,
            aca = bd(reg_byte) aca,
        );
    }
    
    let error_code = igs(0x4400).unwrap_or(0xFFFF);
    crate::serial_println!("[VMX] VMRESUME failed! CF={} ZF={} error={}", vq, aca, error_code);
    Err(HypervisorError::Bwb)
}


pub fn bcg(msr: u32) -> u64 {
    let ail: u32;
    let afq: u32;
    
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            bd("eax") ail,
            bd("edx") afq,
        );
    }
    
    ((afq as u64) << 32) | (ail as u64)
}


pub fn fbs(msr: u32, bn: u64) {
    let ail = bn as u32;
    let afq = (bn >> 32) as u32;
    
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") ail,
            in("edx") afq,
        );
    }
}
