






















use super::mmio_spy;


pub mod esr {
    
    #[inline(always)]
    pub fn ec(esr: u64) -> u32 {
        ((esr >> 26) & 0x3F) as u32
    }

    
    #[inline(always)]
    pub fn il(esr: u64) -> bool {
        (esr >> 25) & 1 != 0
    }

    
    #[inline(always)]
    pub fn xt(esr: u64) -> u32 {
        (esr & 0x01FF_FFFF) as u32
    }
}


pub mod dabt {
    
    #[inline(always)]
    pub fn muo(xt: u32) -> bool {
        (xt >> 24) & 1 != 0
    }

    
    #[inline(always)]
    pub fn okd(xt: u32) -> u32 {
        (xt >> 22) & 0x3
    }

    
    #[inline(always)]
    pub fn access_size(xt: u32) -> u32 {
        1 << okd(xt)
    }

    
    #[inline(always)]
    pub fn ovl(xt: u32) -> u32 {
        (xt >> 16) & 0x1F
    }

    
    #[inline(always)]
    pub fn bvs(xt: u32) -> bool {
        (xt >> 15) & 1 != 0
    }

    
    #[inline(always)]
    pub fn is_write(xt: u32) -> bool {
        (xt >> 6) & 1 != 0
    }
}


pub mod smc {
    
    #[derive(Debug, Clone, Copy)]
    pub enum SmcType {
        
        Psci,
        
        SecureService,
        
        OemService,
        
        HypService,
        
        Unknown,
    }

    
    pub fn hld(fid: u64) -> SmcType {
        let nov = (fid >> 24) & 0x3F;
        match nov {
            0x04 => SmcType::Psci,          
            0x00..=0x01 => SmcType::SecureService,
            0x02..=0x03 => SmcType::SecureService,
            0x30..=0x31 => SmcType::OemService,
            0x05 => SmcType::HypService,
            _ => SmcType::Unknown,
        }
    }

    
    pub fn nze(fid: u64) -> &'static str {
        match fid & 0xFFFF_FFFF {
            0x8400_0000 => "PSCI_VERSION",
            0x8400_0001 => "CPU_SUSPEND (32)",
            0xC400_0001 => "CPU_SUSPEND (64)",
            0x8400_0002 => "CPU_OFF",
            0x8400_0003 => "CPU_ON (32)",
            0xC400_0003 => "CPU_ON (64)",
            0x8400_0004 => "AFFINITY_INFO (32)",
            0xC400_0004 => "AFFINITY_INFO (64)",
            0x8400_0005 => "MIGRATE (32)",
            0x8400_0008 => "SYSTEM_OFF",
            0x8400_0009 => "SYSTEM_RESET",
            0x8400_000A => "FEATURES",
            0x8400_000C => "SYSTEM_RESET2",
            _ => "UNKNOWN_PSCI",
        }
    }
}


mod ec {
    pub const ARV_: u32  = 0b10_0100;  
    pub const AZC_: u32  = 0b10_0000;  
    pub const Zp: u32             = 0b01_0110;   
    pub const Aei: u32             = 0b01_0111;   
    pub const BDE_: u32           = 0b01_1000;   
    pub const Agi: u32              = 0b00_0001;   
    pub const Bde: u32             = 0b01_0101;   
    pub const DPZ_: u32            = 0b10_1100;   
}


#[derive(Debug)]
pub enum TrapAction {
    
    Handled,
    
    ForwardSmc,
    
    InjectFault,
    
    GuestHalt,
}











pub fn mio(
    esr: u64,
    far: u64,
    hpfar: u64,
    guest_regs: &mut [u64; 31],
) -> TrapAction {
    let lrs = esr::ec(esr);
    let xt = esr::xt(esr);

    match lrs {
        ec::ARV_ => {
            mho(xt, far, hpfar, guest_regs)
        }
        ec::AZC_ => {
            
            let ipa = (hpfar & 0x0000_000F_FFFF_FFF0) << 8;
            mmio_spy::etg(mmio_spy::Ey {
                ipa,
                va: far,
                value: 0,
                access_size: 4,
                is_write: false,
                was_inst_fetch: true,
                device_name: mmio_spy::btg(ipa),
            });
            TrapAction::InjectFault
        }
        ec::Aei => {
            mik(guest_regs)
        }
        ec::Zp => {
            
            
            let gbh = guest_regs[0];
            fzs(gbh, guest_regs)
        }
        ec::BDE_ => {
            
            mip(xt, guest_regs)
        }
        ec::Agi => {
            
            TrapAction::Handled
        }
        _ => {
            
            TrapAction::InjectFault
        }
    }
}


fn mho(
    xt: u32,
    far: u64,
    hpfar: u64,
    guest_regs: &mut [u64; 31],
) -> TrapAction {
    
    
    let mru = (hpfar & 0x0000_000F_FFFF_FFF0) << 8;
    let ipa = mru | (far & 0xFFF);

    if !dabt::muo(xt) {
        
        
        
        mmio_spy::etg(mmio_spy::Ey {
            ipa,
            va: far,
            value: 0,
            access_size: 0,
            is_write: false,
            was_inst_fetch: false,
            device_name: mmio_spy::btg(ipa),
        });
        return TrapAction::Handled;
    }

    let is_write = dabt::is_write(xt);
    let access_size = dabt::access_size(xt);
    let tb = dabt::ovl(xt) as usize;

    if is_write {
        
        let value = if tb < 31 { guest_regs[tb] } else { 0 };

        
        mmio_spy::etg(mmio_spy::Ey {
            ipa,
            va: far,
            value,
            access_size,
            is_write: true,
            was_inst_fetch: false,
            device_name: mmio_spy::btg(ipa),
        });

        
        lgp(ipa, value, access_size);
    } else {
        
        let value = lgo(ipa, access_size);

        
        mmio_spy::etg(mmio_spy::Ey {
            ipa,
            va: far,
            value,
            access_size,
            is_write: false,
            was_inst_fetch: false,
            device_name: mmio_spy::btg(ipa),
        });

        
        if tb < 31 {
            guest_regs[tb] = value;
        }
    }

    TrapAction::Handled
}


fn mik(guest_regs: &mut [u64; 31]) -> TrapAction {
    let fid = guest_regs[0];
    let x1 = guest_regs[1];
    let x2 = guest_regs[2];
    let x3 = guest_regs[3];

    let jgs = smc::hld(fid);

    
    mmio_spy::nal(mmio_spy::Iz {
        fid,
        x1,
        x2,
        x3,
        smc_type_name: match jgs {
            smc::SmcType::Psci => smc::nze(fid),
            smc::SmcType::SecureService => "SECURE_SVC",
            smc::SmcType::OemService => "OEM_SVC",
            smc::SmcType::HypService => "HYP_SVC",
            smc::SmcType::Unknown => "UNKNOWN",
        },
    });

    
    if let smc::SmcType::Psci = jgs {
        match fid & 0xFFFF_FFFF {
            0x8400_0008 => return TrapAction::GuestHalt,  
            0x8400_0009 => return TrapAction::GuestHalt,  
            _ => {}
        }
    }

    
    TrapAction::ForwardSmc
}


fn fzs(gbh: u64, guest_regs: &mut [u64; 31]) -> TrapAction {
    match gbh {
        
        0x5452_5553 => {
            
            guest_regs[0] = mmio_spy::gzs() as u64;
            guest_regs[1] = mmio_spy::fdl() as u64;
            TrapAction::Handled
        }
        _ => TrapAction::Handled,
    }
}


fn mip(_iss: u32, _guest_regs: &mut [u64; 31]) -> TrapAction {
    
    
    TrapAction::Handled
}


fn lgo(pa: u64, size: u32) -> u64 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let val: u64;
        match size {
            1 => {
                let v: u8;
                core::arch::asm!(
                    "ldrb {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = out(reg) v,
                    options(nostack, readonly)
                );
                return v as u64;
            }
            2 => {
                let v: u16;
                core::arch::asm!(
                    "ldrh {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = out(reg) v,
                    options(nostack, readonly)
                );
                return v as u64;
            }
            4 => {
                let v: u32;
                core::arch::asm!(
                    "ldr {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = out(reg) v,
                    options(nostack, readonly)
                );
                return v as u64;
            }
            8 => {
                core::arch::asm!(
                    "ldr {val}, [{addr}]",
                    addr = in(reg) pa,
                    val = out(reg) val,
                    options(nostack, readonly)
                );
                return val;
            }
            _ => return 0,
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (pa, size);
        0
    }
}


fn lgp(pa: u64, value: u64, size: u32) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        match size {
            1 => {
                core::arch::asm!(
                    "strb {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = in(reg) value as u32,
                    options(nostack)
                );
            }
            2 => {
                core::arch::asm!(
                    "strh {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = in(reg) value as u32,
                    options(nostack)
                );
            }
            4 => {
                core::arch::asm!(
                    "str {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = in(reg) value as u32,
                    options(nostack)
                );
            }
            8 => {
                core::arch::asm!(
                    "str {val}, [{addr}]",
                    addr = in(reg) pa,
                    val = in(reg) value,
                    options(nostack)
                );
            }
            _ => {}
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (pa, value, size);
    }
}
