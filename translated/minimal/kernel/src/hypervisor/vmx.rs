








use core::arch::asm;
use alloc::boxed::Box;
use super::{HypervisorError, Result, Nv};


#[repr(C, align(4096))]
pub struct VmxonRegion {
    pub azj: u32,
    pub data: [u8; 4092],
}

impl VmxonRegion {
    pub fn new(azj: u32) -> Self {
        VmxonRegion {
            azj,
            data: [0; 4092],
        }
    }
}


static mut BKU_: Option<Box<VmxonRegion>> = None;





pub const VH_: u32 = 0x480;
pub const CEL_: u32 = 0x481;
pub const AYR_: u32 = 0x482;
pub const CEK_: u32 = 0x483;
pub const CEJ_: u32 = 0x484;
pub const AYS_: u32 = 0x48B;
pub const CEO_: u32 = 0x48D;
pub const CEP_: u32 = 0x48E;
pub const CEN_: u32 = 0x48F;
pub const CEM_: u32 = 0x490;
pub const CED_: u32 = 0x3A;




pub fn dhg(msr: u32, desired: u32) -> u32 {
    let iow = ach(msr);
    let nhd = iow as u32;        
    let ndq = (iow >> 32) as u32; 
    
    (desired | nhd) & ndq
}


pub fn eov() -> bool {
    let csl = ach(VH_);
    (csl & (1u64 << 55)) != 0
}


pub fn nus() -> u32 {
    if eov() { CEO_ } else { CEL_ }
}


pub fn nyf() -> u32 {
    if eov() { CEP_ } else { AYR_ }
}


pub fn lsj() -> u32 {
    if eov() { CEN_ } else { CEK_ }
}


pub fn lqo() -> u32 {
    if eov() { CEM_ } else { CEJ_ }
}


pub fn bjv(virt: u64) -> u64 {
    crate::memory::lc(virt).unwrap_or(virt)
}


pub fn ehv() -> Result<Nv> {
    let mut caps = Nv {
        supported: false,
        ept_supported: false,
        unrestricted_guest: false,
        vpid_supported: false,
        vmcs_revision_id: 0,
    };
    
    
    let cpuid_result: u32;
    unsafe {
        asm!(
            "push rbx",       
            "mov eax, 1",
            "cpuid",
            "mov {0:e}, ecx",
            "pop rbx",        
            out(reg) cpuid_result,
            out("eax") _,
            out("ecx") _,
            out("edx") _,
        );
    }
    
    caps.supported = (cpuid_result & (1 << 5)) != 0;
    
    if !caps.supported {
        return Ok(caps);
    }
    
    
    let csl = ach(VH_);
    caps.vmcs_revision_id = (csl & 0x7FFF_FFFF) as u32;
    
    
    
    let nyd = ach(AYR_);
    let omp = (nyd >> 32) & (1 << 31) != 0;
    
    if omp {
        let nye = ach(AYS_);
        let fgv = (nye >> 32) as u32;
        
        caps.ept_supported = (fgv & (1 << 1)) != 0;           
        caps.unrestricted_guest = (fgv & (1 << 7)) != 0;      
        caps.vpid_supported = (fgv & (1 << 5)) != 0;          
    }
    
    Ok(caps)
}


pub fn lpv() -> Result<()> {
    
    let fwj = ach(CED_);
    
    
    
    if (fwj & 1) != 0 {
        
        if (fwj & (1 << 2)) == 0 {
            crate::serial_println!("[VMX] ERROR: VMX disabled by BIOS (locked)");
            return Err(HypervisorError::VmxNotSupported);
        }
    } else {
        
        let gjj = fwj | (1 << 2) | 1; 
        cfm(0x3A, gjj);
        crate::serial_println!("[VMX] Enabled VMX in IA32_FEATURE_CONTROL");
    }
    
    
    unsafe {
        asm!(
            "mov rax, cr4",
            "or rax, 0x2000",  
            "mov cr4, rax",
            out("rax") _,
        );
    }
    
    crate::serial_println!("[VMX] CR4.VMXE enabled");
    
    Ok(())
}


pub fn psy() -> Result<()> {
    
    let csl = ach(VH_);
    let azj = (csl & 0x7FFF_FFFF) as u32;
    
    
    let qd = Box::new(VmxonRegion::new(azj));
    let izj = qd.as_ref() as *const VmxonRegion as u64;
    let region_phys = bjv(izj);
    
    crate::serial_println!("[VMX] VMXON region virt=0x{:016X} phys=0x{:016X}, revision 0x{:08X}", 
                          izj, region_phys, azj);
    
    
    unsafe {
        BKU_ = Some(qd);
    }
    
    
    let cf: u8;
    let zf: u8;
    unsafe {
        asm!(
            "vmxon [{addr}]",
            "setc {cf}",
            "setz {zf}",
            addr = in(reg) &region_phys,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        crate::serial_println!("[VMX] VMXON failed! CF={} ZF={}", cf, zf);
        return Err(HypervisorError::VmxonFailed);
    }
    
    crate::serial_println!("[VMX] VMXON successful - Now in VMX root operation");
    
    Ok(())
}


pub fn psx() -> Result<()> {
    unsafe {
        asm!("vmxoff");
    }
    
    
    unsafe {
        asm!(
            "mov rax, cr4",
            "and rax, ~0x2000",  
            "mov cr4, rax",
            out("rax") _,
        );
    }
    
    
    unsafe {
        BKU_ = None;
    }
    
    crate::serial_println!("[VMX] VMXOFF complete - Left VMX operation");
    
    Ok(())
}



pub fn hbu(vmcs_phys: u64) -> Result<()> {
    let cf: u8;
    let zf: u8;
    unsafe {
        asm!(
            "vmclear [{addr}]",
            "setc {cf}",
            "setz {zf}",
            addr = in(reg) &vmcs_phys,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        crate::serial_println!("[VMX] VMCLEAR failed for phys=0x{:X} CF={} ZF={}", vmcs_phys, cf, zf);
        return Err(HypervisorError::VmclearFailed);
    }
    
    Ok(())
}



pub fn jqk(vmcs_phys: u64) -> Result<()> {
    let cf: u8;
    let zf: u8;
    unsafe {
        asm!(
            "vmptrld [{addr}]",
            "setc {cf}",
            "setz {zf}",
            addr = in(reg) &vmcs_phys,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        crate::serial_println!("[VMX] VMPTRLD failed for phys=0x{:X} CF={} ZF={}", vmcs_phys, cf, zf);
        return Err(HypervisorError::VmptrldFailed);
    }
    
    Ok(())
}


pub fn edv(field: u64) -> Result<u64> {
    let value: u64;
    let cf: u8;
    let zf: u8;
    
    unsafe {
        asm!(
            "vmread {val}, {field}",
            "setc {cf}",
            "setz {zf}",
            val = out(reg) value,
            field = in(reg) field,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        return Err(HypervisorError::VmreadFailed);
    }
    
    Ok(value)
}


pub fn jql(field: u64, value: u64) -> Result<()> {
    let cf: u8;
    let zf: u8;
    
    unsafe {
        asm!(
            "vmwrite {field}, {val}",
            "setc {cf}",
            "setz {zf}",
            field = in(reg) field,
            val = in(reg) value,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        return Err(HypervisorError::VmwriteFailed);
    }
    
    Ok(())
}




pub fn rcc() -> Result<()> {
    let cf: u8;
    let zf: u8;
    
    unsafe {
        asm!(
            "vmlaunch",
            "setc {cf}",
            "setz {zf}",
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    
    let error_code = edv(0x4400).unwrap_or(0xFFFF); 
    crate::serial_println!("[VMX] VMLAUNCH failed! CF={} ZF={} error={}", cf, zf, error_code);
    Err(HypervisorError::VmlaunchFailed)
}


pub fn rce() -> Result<()> {
    let cf: u8;
    let zf: u8;
    
    unsafe {
        asm!(
            "vmresume",
            "setc {cf}",
            "setz {zf}",
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    let error_code = edv(0x4400).unwrap_or(0xFFFF);
    crate::serial_println!("[VMX] VMRESUME failed! CF={} ZF={} error={}", cf, zf, error_code);
    Err(HypervisorError::VmresumeFailed)
}


pub fn ach(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
    }
    
    ((high as u64) << 32) | (low as u64)
}


pub fn cfm(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
        );
    }
}
