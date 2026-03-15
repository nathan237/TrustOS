


















































use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use core::ptr;

use super::Sw;
use super::stage2::Stage2Tables;






pub const BYS_: u64 = 0x4000_0000;


pub const ADR_: u64 = 0x0020_0000;  


pub const ABF_: u64 = 0x0400_0000;     


pub const ADM_: u64 = 0x0500_0000;  


pub const BRG_: u64 = 512 * 1024 * 1024;


pub const AZS_: usize = 64 * 1024 * 1024;


pub const AZM_: usize = 1 * 1024 * 1024;


pub const AZR_: usize = 400 * 1024 * 1024;


pub const AKT_: u32 = 0x644d5241; 


pub const UW_: usize = 0x38;


pub const DQW_: usize = 0x10;


pub const EHN_: usize = 0x08;






#[derive(Debug, Clone)]
pub struct Arm64ImageHeader {
    
    pub nem: u32,
    
    pub fwo: u64,
    
    pub gjn: u64,
    
    pub flags: u64,
    
    pub sj: u32,
}

impl Arm64ImageHeader {
    
    pub fn parse(f: &[u8]) -> Result<Self, &'static str> {
        if f.len() < 64 {
            return Err("Image too small (need at least 64 bytes for header)");
        }

        let sj = u32::dj([
            f[UW_],
            f[UW_ + 1],
            f[UW_ + 2],
            f[UW_ + 3],
        ]);

        if sj != AKT_ {
            return Err("Invalid ARM64 Image magic (expected 0x644d5241 'ARM\\x64')");
        }

        let nem = u32::dj([f[0], f[1], f[2], f[3]]);

        let fwo = u64::dj([
            f[0x08], f[0x09], f[0x0A], f[0x0B],
            f[0x0C], f[0x0D], f[0x0E], f[0x0F],
        ]);

        let gjn = u64::dj([
            f[0x10], f[0x11], f[0x12], f[0x13],
            f[0x14], f[0x15], f[0x16], f[0x17],
        ]);

        let flags = u64::dj([
            f[0x18], f[0x19], f[0x1A], f[0x1B],
            f[0x1C], f[0x1D], f[0x1E], f[0x1F],
        ]);

        Ok(Arm64ImageHeader {
            nem,
            fwo,
            gjn,
            flags,
            sj,
        })
    }

    
    pub fn tyb(&self) -> bool {
        self.flags & 1 == 0
    }

    
    pub fn npj(&self, yy: usize) -> usize {
        if self.gjn > 0 && (self.gjn as usize) <= yy {
            self.gjn as usize
        } else {
            yy
        }
    }
}






const JZ_: u32 = 0xD00DFEED;


const ARR_: u32 = 0x00000001;
const ARS_: u32   = 0x00000002;
const ARU_: u32        = 0x00000003;
const ART_: u32         = 0x00000004;
const ACA_: u32         = 0x00000009;


pub fn pxx(f: &[u8]) -> Result<(), &'static str> {
    if f.len() < 40 {
        return Err("DTB too small");
    }
    let sj = u32::oa([f[0], f[1], f[2], f[3]]);
    if sj != JZ_ {
        return Err("Invalid DTB magic (expected 0xD00DFEED)");
    }
    let aay = u32::oa([f[4], f[5], f[6], f[7]]) as usize;
    if aay > f.len() {
        return Err("DTB total_size exceeds buffer");
    }
    Ok(())
}








pub fn veu(
    azq: &mut [u8],
    nog: usize,
    yxy: Option<u64>,
    gjz: Option<u64>,
    haz: Option<&str>,
) -> Result<usize, &'static str> {
    
    
    
    
    
    
    
    
    
    
    pxx(&azq[..nog])?;
    
    
    
    Ok(nog)
}






#[derive(Debug, Clone)]
pub struct GuestLoadConfig {
    
    pub brw: u64,
    
    pub cbf: u64,
    
    pub wx: String,
    
    pub guw: Vec<(u64, u64)>,
    
    pub fxj: bool,
    
    pub fxk: bool,
}

impl Default for GuestLoadConfig {
    fn default() -> Self {
        Self {
            brw: BYS_,
            cbf: BRG_,
            wx: String::from("console=ttyAMA0 earlycon=pl011,0x09000000 earlyprintk"),
            guw: vec![
                
                (0x0800_0000, 0x0001_0000),  
                (0x0801_0000, 0x0001_0000),  
                (0x0900_0000, 0x0000_1000),  
                (0x0901_0000, 0x0000_1000),  
                (0x0A00_0000, 0x0000_0200),  
                (0x0A00_0200, 0x0000_0200),  
            ],
            fxj: true,
            fxk: false,
        }
    }
}


#[derive(Debug)]
pub struct Aiu {
    
    pub eed: u64,
    
    pub bvc: usize,
    
    pub bqh: u64,
    
    pub dgv: usize,
    
    pub edg: Option<u64>,
    
    pub hny: Option<usize>,
    
    pub dh: Arm64ImageHeader,
    
    pub dru: Sw,
}










pub fn ugj(
    abr: &[u8],
    hhe: &[u8],
    cyw: Option<&[u8]>,
    config: &GuestLoadConfig,
) -> Result<Aiu, String> {
    
    if abr.len() < 64 {
        return Err(String::from("Kernel image too small"));
    }
    if abr.len() > AZS_ {
        return Err(format!("Kernel too large: {}MB (max {}MB)",
            abr.len() / (1024*1024), AZS_ / (1024*1024)));
    }

    let dh = Arm64ImageHeader::parse(abr)
        .jd(|aa| String::from(aa))?;

    
    pxx(hhe).jd(|aa| String::from(aa))?;
    if hhe.len() > AZM_ {
        return Err(format!("DTB too large: {}KB (max {}KB)",
            hhe.len() / 1024, AZM_ / 1024));
    }

    
    if let Some(apw) = cyw {
        if apw.len() > AZR_ {
            return Err(format!("initrd too large: {}MB (max {}MB)",
                apw.len() / (1024*1024), AZR_ / (1024*1024)));
        }
    }

    
    let eed = config.brw + ADR_;
    let bqh = config.brw + ABF_;
    let edg = config.brw + ADM_;

    
    let dip = eed + dh.npj(abr.len()) as u64;
    if dip > bqh {
        return Err(format!("Kernel too large ({}MB), overlaps DTB region",
            abr.len() / (1024*1024)));
    }
    if let Some(apw) = cyw {
        let gjz = edg + apw.len() as u64;
        let ozg = config.brw + config.cbf;
        if gjz > ozg {
            return Err(format!("initrd doesn't fit in guest RAM (need {}MB, have {}MB free)",
                apw.len() / (1024*1024),
                (ozg - edg) / (1024*1024)));
        }
    }

    
    let bvc = dh.npj(abr.len());
    unsafe {
        ptr::copy_nonoverlapping(
            abr.fq(),
            eed as *mut u8,
            bvc,
        );
    }

    
    
    let mut isa = [0u8; 1048576]; 
    let krz = hhe.len().v(isa.len());
    isa[..krz].dg(&hhe[..krz]);

    
    let tui = cyw.map(|bc| edg + bc.len() as u64);
    let nof = veu(
        &mut isa,
        krz,
        cyw.map(|_| edg),
        tui,
        if config.wx.is_empty() { None } else { Some(&config.wx) },
    ).jd(|aa| String::from(aa))?;

    unsafe {
        ptr::copy_nonoverlapping(
            isa.fq(),
            bqh as *mut u8,
            nof,
        );
    }

    
    let (tuk, tul) = if let Some(apw) = cyw {
        unsafe {
            ptr::copy_nonoverlapping(
                apw.fq(),
                edg as *mut u8,
                apw.len(),
            );
        }
        (Some(edg), Some(apw.len()))
    } else {
        (None, None)
    };

    
    #[cfg(target_arch = "aarch64")]
    unsafe {
        
        
        core::arch::asm!(
            "dsb ish",
            "ic ialluis",    
            "dsb ish",
            "isb",
            options(nomem, nostack)
        );
    }

    
    let dru = Sw {
        hma: eed,
        ixj: bqh,
        hmd: config.brw,
        hme: config.cbf,
        iew: config.guw.clone(),
        fxj: config.fxj,
        fxk: config.fxk,
    };

    Ok(Aiu {
        eed,
        bvc,
        bqh,
        dgv: nof,
        edg: tuk,
        hny: tul,
        dh,
        dru,
    })
}


pub fn svv(result: &Aiu) -> String {
    let mut e = String::new();
    e.t("=== ARM64 Guest Loaded ===\n");
    e.t(&format!("  Kernel:  0x{:08X} ({} KB)\n",
        result.eed, result.bvc / 1024));
    e.t(&format!("  DTB:     0x{:08X} ({} KB)\n",
        result.bqh, result.dgv / 1024));
    if let (Some(ag), Some(aw)) = (result.edg, result.hny) {
        e.t(&format!("  initrd:  0x{:08X} ({} KB)\n", ag, aw / 1024));
    }
    e.t(&format!("  Entry:   0x{:08X}\n", result.dru.hma));
    e.t(&format!("  RAM:     0x{:08X} - 0x{:08X} ({} MB)\n",
        result.dru.hmd,
        result.dru.hmd + result.dru.hme,
        result.dru.hme / (1024*1024)));
    e.t(&format!("  Endian:  {}\n",
        if result.dh.tyb() { "Little (LE)" } else { "Big (BE)" }));
    e.t(&format!("  MMIO traps: {} regions\n",
        result.dru.iew.len()));
    for (ar, aw) in &result.dru.iew {
        e.t(&format!("    0x{:08X} - 0x{:08X} ({})\n",
            ar, ar + aw, super::mmio_spy::eda(*ar)));
    }
    e.t(&format!("  SMC trap: {}\n",
        if result.dru.fxj { "ON" } else { "OFF" }));
    e
}







pub fn wgy(brw: u64, cbf: u64) -> Result<Aiu, String> {
    
    
    
    
    
    let mut guo = [0u8; 72];
    
    
    
    guo[0..4].dg(&0x14000010u32.ho());
    
    
    
    guo[0x10..0x18].dg(&72u64.ho());
    
    
    guo[0x38..0x3C].dg(&AKT_.ho());
    
    
    
    guo[64..68].dg(&0xD503207Fu32.ho()); 
    guo[68..72].dg(&0x17FFFFFFu32.ho()); 
    
    
    let mut efl = [0u8; 48];
    efl[0..4].dg(&JZ_.ft());     
    efl[4..8].dg(&48u32.ft());          
    efl[8..12].dg(&40u32.ft());         
    efl[12..16].dg(&44u32.ft());        
    efl[16..20].dg(&28u32.ft());        
    efl[20..24].dg(&17u32.ft());        
    efl[24..28].dg(&16u32.ft());        
    
    
    efl[40..44].dg(&ACA_.ft());
    
    
    let mut config = GuestLoadConfig::default();
    config.brw = brw;
    config.cbf = cbf;
    config.fxk = true; 
    config.wx = String::new();
    
    ugj(&guo, &efl, None, &config)
}
