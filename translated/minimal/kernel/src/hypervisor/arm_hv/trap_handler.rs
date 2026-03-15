






















use super::mmio_spy;


pub mod esr {
    
    #[inline(always)]
    pub fn ec(esr: u64) -> u32 {
        ((esr >> 26) & 0x3F) as u32
    }

    
    #[inline(always)]
    pub fn odh(esr: u64) -> bool {
        (esr >> 25) & 1 != 0
    }

    
    #[inline(always)]
    pub fn ayb(esr: u64) -> u32 {
        (esr & 0x01FF_FFFF) as u32
    }
}


pub mod dabt {
    
    #[inline(always)]
    pub fn tzw(ayb: u32) -> bool {
        (ayb >> 24) & 1 != 0
    }

    
    #[inline(always)]
    pub fn wcs(ayb: u32) -> u32 {
        (ayb >> 22) & 0x3
    }

    
    #[inline(always)]
    pub fn cct(ayb: u32) -> u32 {
        1 << wcs(ayb)
    }

    
    #[inline(always)]
    pub fn wrw(ayb: u32) -> u32 {
        (ayb >> 16) & 0x1F
    }

    
    #[inline(always)]
    pub fn eim(ayb: u32) -> bool {
        (ayb >> 15) & 1 != 0
    }

    
    #[inline(always)]
    pub fn rm(ayb: u32) -> bool {
        (ayb >> 6) & 1 != 0
    }
}


pub mod smc {
    
    #[derive(Debug, Clone, Copy)]
    pub enum SmcType {
        
        Axd,
        
        Ayn,
        
        Bnt,
        
        Biv,
        
        F,
    }

    
    pub fn ndc(aos: u64) -> SmcType {
        let vah = (aos >> 24) & 0x3F;
        match vah {
            0x04 => SmcType::Axd,          
            0x00..=0x01 => SmcType::Ayn,
            0x02..=0x03 => SmcType::Ayn,
            0x30..=0x31 => SmcType::Bnt,
            0x05 => SmcType::Biv,
            _ => SmcType::F,
        }
    }

    
    pub fn vnt(aos: u64) -> &'static str {
        match aos & 0xFFFF_FFFF {
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
    pub const APT_: u32  = 0b10_0100;  
    pub const AXA_: u32  = 0b10_0000;  
    pub const Bin: u32             = 0b01_0110;   
    pub const Brz: u32             = 0b01_0111;   
    pub const BBB_: u32           = 0b01_1000;   
    pub const Bwg: u32              = 0b00_0001;   
    pub const Dic: u32             = 0b01_0101;   
    pub const DMD_: u32            = 0b10_1100;   
}


#[derive(Debug)]
pub enum TrapAction {
    
    Gw,
    
    Bhj,
    
    Auf,
    
    Ath,
}











pub fn tld(
    esr: u64,
    adt: u64,
    esb: u64,
    ej: &mut [u64; 31],
) -> TrapAction {
    let soa = esr::ec(esr);
    let ayb = esr::ayb(esr);

    match soa {
        ec::APT_ => {
            tjh(ayb, adt, esb, ej)
        }
        ec::AXA_ => {
            
            let akh = (esb & 0x0000_000F_FFFF_FFF0) << 8;
            mmio_spy::jdw(mmio_spy::Lv {
                akh,
                asf: adt,
                bn: 0,
                cct: 4,
                rm: false,
                gwm: true,
                dgg: mmio_spy::eda(akh),
            });
            TrapAction::Auf
        }
        ec::Brz => {
            tky(ej)
        }
        ec::Bin => {
            
            
            let lcw = ej[0];
            lau(lcw, ej)
        }
        ec::BBB_ => {
            
            tle(ayb, ej)
        }
        ec::Bwg => {
            
            TrapAction::Gw
        }
        _ => {
            
            TrapAction::Auf
        }
    }
}


fn tjh(
    ayb: u32,
    adt: u64,
    esb: u64,
    ej: &mut [u64; 31],
) -> TrapAction {
    
    
    let twm = (esb & 0x0000_000F_FFFF_FFF0) << 8;
    let akh = twm | (adt & 0xFFF);

    if !dabt::tzw(ayb) {
        
        
        
        mmio_spy::jdw(mmio_spy::Lv {
            akh,
            asf: adt,
            bn: 0,
            cct: 0,
            rm: false,
            gwm: false,
            dgg: mmio_spy::eda(akh),
        });
        return TrapAction::Gw;
    }

    let rm = dabt::rm(ayb);
    let cct = dabt::cct(ayb);
    let alq = dabt::wrw(ayb) as usize;

    if rm {
        
        let bn = if alq < 31 { ej[alq] } else { 0 };

        
        mmio_spy::jdw(mmio_spy::Lv {
            akh,
            asf: adt,
            bn,
            cct,
            rm: true,
            gwm: false,
            dgg: mmio_spy::eda(akh),
        });

        
        rzx(akh, bn, cct);
    } else {
        
        let bn = rzw(akh, cct);

        
        mmio_spy::jdw(mmio_spy::Lv {
            akh,
            asf: adt,
            bn,
            cct,
            rm: false,
            gwm: false,
            dgg: mmio_spy::eda(akh),
        });

        
        if alq < 31 {
            ej[alq] = bn;
        }
    }

    TrapAction::Gw
}


fn tky(ej: &mut [u64; 31]) -> TrapAction {
    let aos = ej[0];
    let dn = ej[1];
    let hy = ej[2];
    let ajr = ej[3];

    let pln = smc::ndc(aos);

    
    mmio_spy::uhx(mmio_spy::Um {
        aos,
        dn,
        hy,
        ajr,
        jqo: match pln {
            smc::SmcType::Axd => smc::vnt(aos),
            smc::SmcType::Ayn => "SECURE_SVC",
            smc::SmcType::Bnt => "OEM_SVC",
            smc::SmcType::Biv => "HYP_SVC",
            smc::SmcType::F => "UNKNOWN",
        },
    });

    
    if let smc::SmcType::Axd = pln {
        match aos & 0xFFFF_FFFF {
            0x8400_0008 => return TrapAction::Ath,  
            0x8400_0009 => return TrapAction::Ath,  
            _ => {}
        }
    }

    
    TrapAction::Bhj
}


fn lau(lcw: u64, ej: &mut [u64; 31]) -> TrapAction {
    match lcw {
        
        0x5452_5553 => {
            
            ej[0] = mmio_spy::mmj() as u64;
            ej[1] = mmio_spy::jty() as u64;
            TrapAction::Gw
        }
        _ => TrapAction::Gw,
    }
}


fn tle(yag: u32, xzo: &mut [u64; 31]) -> TrapAction {
    
    
    TrapAction::Gw
}


fn rzw(awk: u64, aw: u32) -> u64 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let ap: u64;
        match aw {
            1 => {
                let p: u8;
                core::arch::asm!(
                    "ldrb {val:w}, [{addr}]",
                    ag = in(reg) awk,
                    ap = bd(reg) p,
                    options(nostack, awr)
                );
                return p as u64;
            }
            2 => {
                let p: u16;
                core::arch::asm!(
                    "ldrh {val:w}, [{addr}]",
                    ag = in(reg) awk,
                    ap = bd(reg) p,
                    options(nostack, awr)
                );
                return p as u64;
            }
            4 => {
                let p: u32;
                core::arch::asm!(
                    "ldr {val:w}, [{addr}]",
                    ag = in(reg) awk,
                    ap = bd(reg) p,
                    options(nostack, awr)
                );
                return p as u64;
            }
            8 => {
                core::arch::asm!(
                    "ldr {val}, [{addr}]",
                    ag = in(reg) awk,
                    ap = bd(reg) ap,
                    options(nostack, awr)
                );
                return ap;
            }
            _ => return 0,
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (awk, aw);
        0
    }
}


fn rzx(awk: u64, bn: u64, aw: u32) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        match aw {
            1 => {
                core::arch::asm!(
                    "strb {val:w}, [{addr}]",
                    ag = in(reg) awk,
                    ap = in(reg) bn as u32,
                    options(nostack)
                );
            }
            2 => {
                core::arch::asm!(
                    "strh {val:w}, [{addr}]",
                    ag = in(reg) awk,
                    ap = in(reg) bn as u32,
                    options(nostack)
                );
            }
            4 => {
                core::arch::asm!(
                    "str {val:w}, [{addr}]",
                    ag = in(reg) awk,
                    ap = in(reg) bn as u32,
                    options(nostack)
                );
            }
            8 => {
                core::arch::asm!(
                    "str {val}, [{addr}]",
                    ag = in(reg) awk,
                    ap = in(reg) bn,
                    options(nostack)
                );
            }
            _ => {}
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (awk, bn, aw);
    }
}
