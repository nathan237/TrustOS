










use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};






mod reg {
    pub const Bhn: u32     = 0x00;  
    pub const Cpe: u32     = 0x02;  
    pub const Cpc: u32     = 0x03;  
    pub const Ddk: u32   = 0x04;  
    pub const Cys: u32    = 0x06;  
    pub const Km: u32     = 0x08;  
    pub const Dlx: u32   = 0x0C;  
    pub const Ayi: u32 = 0x0E;  
    pub const Cxz: u32     = 0x10;  
    pub const Atu: u32   = 0x20;  
    pub const Cyw: u32   = 0x24;  
    pub const Cqa: u32   = 0x30;  
    pub const Bsa: u32    = 0x38;  

    
    pub const Bzc: u32 = 0x40;  
    pub const Bzd: u32 = 0x44;  
    pub const Apq: u32    = 0x48;  
    pub const Bdc: u32    = 0x4A;  
    pub const Bdb: u32   = 0x4C;  
    pub const Csn: u32   = 0x4D;  
    pub const Bdd: u32  = 0x4E;  

    
    pub const Cjw: u32 = 0x50;  
    pub const Cjy: u32 = 0x54;  
    pub const Bqe: u32    = 0x58;  
    pub const Cjv: u32   = 0x5A;  
    pub const Bqc: u32   = 0x5C;  
    pub const Cjx: u32   = 0x5D;  
    pub const Bqd: u32  = 0x5E;  

    
    pub const Cyo: u32  = 0x60;  
    pub const Czg: u32  = 0x64;  
    pub const Cyq: u32 = 0x68;  

    
    pub const Can: u32 = 0x70;  
    pub const Cao: u32 = 0x74;  

    
    pub const CRY_: u32 = 0x80;
    pub const CRZ_: u32 = 0x20;
}


mod sd {
    pub const Cx: u32    = 0x00;  
    pub const Ui: u32    = 0x03;  
    pub const Bkm: u32   = 0x04;  
    pub const Agw: u32    = 0x08;  
    pub const Ajp: u32    = 0x0C;  
    pub const Ccz: u32  = 0x10;  
    pub const Aia: u32    = 0x12;  
    pub const Agm: u32  = 0x18;  
    pub const Agn: u32  = 0x1C;  
}


mod gctl {
    pub const Rs: u32   = 1 << 0;   
    pub const Cwq: u32 = 1 << 1;   
    pub const Bvb: u32  = 1 << 8;   
}


mod sctl {
    pub const Mh: u32 = 1 << 0;     
    pub const Ub: u32  = 1 << 1;     
    pub const Cfo: u32 = 1 << 2;     
    
    pub const AIJ_: u32 = 20;
}


mod ssts {
    pub const Bye: u8 = 1 << 2;   
    pub const Ccx: u8 = 1 << 3;  
    pub const Cae: u8 = 1 << 4;   
    pub const Ccy: u8 = 1 << 5; 
}





mod verb {
    
    pub const EI_: u32        = 0xF00;
    pub const BWU_: u32        = 0xF02;
    pub const BWV_: u32      = 0xF01;
    pub const ASV_: u32      = 0xF07;
    pub const BWT_: u32   = 0xF1C;
    pub const ASU_: u32             = 0xF0C;
    pub const TI_: u32      = 0xF05;
    pub const AST_: u32   = 0xF06;

    
    pub const DI_: u32         = 0xB00;  
    pub const BXB_: u32    = 0xA00;  

    
    pub const CSI_: u32      = 0x701;
    pub const PL_: u32      = 0x705;
    pub const CSH_: u32   = 0x706;
    pub const AHT_: u32      = 0x707;
    pub const AHS_: u32             = 0x70C;

    
    pub const EFM_: u32    = 0x300;
    pub const CSN_: u32    = 0x200;
    pub const EFQ_: u32       = 0x500;  
    pub const CSL_: u32        = 0x400;  
    pub const DMS_: u32       = 0xD00;  
    pub const DMX_: u32        = 0xC00;  

    
    pub const BFM_: u32        = 0x715;
    pub const CSK_: u32        = 0x716;
    pub const CSJ_: u32         = 0x717;
    pub const BWW_: u32        = 0xF15;
    pub const BWY_: u32        = 0xF16;
    pub const BWX_: u32         = 0xF17;
    pub const CIQ_: u32    = 0x11;  

    
    pub const AGI_: u32     = 0x00;
    pub const DYU_: u32      = 0x02;
    pub const BCP_: u32    = 0x04;
    pub const CIP_: u32 = 0x05;
    pub const OU_: u32    = 0x09;  
    pub const DYS_: u32     = 0x0A;  
    pub const DYV_: u32   = 0x0B;  
    pub const CIR_: u32      = 0x0C;  
    pub const WB_: u32   = 0x0D;  
    pub const CIO_: u32 = 0x0E;  
    pub const DYT_: u32  = 0x0F;  
    pub const OT_: u32  = 0x12;  
    pub const DYW_: u32 = 0x13;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WidgetType {
    Zw  = 0,
    Bcc   = 1,
    Apg   = 2,
    Aph = 3,
    Tx   = 4,
    Hb        = 5,
    Bwe   = 6,
    Bco      = 7,
    Bvs    = 0xF,
    F      = 0xFF,
}

impl WidgetType {
    fn sxs(dr: u32) -> Self {
        match (dr >> 20) & 0xF {
            0 => Self::Zw,
            1 => Self::Bcc,
            2 => Self::Apg,
            3 => Self::Aph,
            4 => Self::Tx,
            5 => Self::Hb,
            6 => Self::Bwe,
            7 => Self::Bco,
            0xF => Self::Bvs,
            _ => Self::F,
        }
    }

    fn j(&self) -> &'static str {
        match self {
            Self::Zw => "Audio Output (DAC)",
            Self::Bcc => "Audio Input (ADC)",
            Self::Apg => "Audio Mixer",
            Self::Aph => "Audio Selector",
            Self::Tx => "Pin Complex",
            Self::Hb => "Power Widget",
            Self::Bwe => "Volume Knob",
            Self::Bco => "Beep Generator",
            Self::Bvs => "Vendor Defined",
            Self::F => "Unknown",
        }
    }
}


fn lty(config: u32) -> &'static str {
    match (config >> 20) & 0xF {
        0x0 => "Line Out",
        0x1 => "Speaker",
        0x2 => "HP Out",
        0x3 => "CD",
        0x4 => "SPDIF Out",
        0x5 => "Digital Other Out",
        0x6 => "Modem Line Side",
        0x7 => "Modem Handset",
        0x8 => "Line In",
        0x9 => "AUX",
        0xA => "Mic In",
        0xB => "Telephony",
        0xC => "SPDIF In",
        0xD => "Digital Other In",
        0xE => "Reserved",
        0xF => "Other",
        _ => "?",
    }
}






#[derive(Debug, Clone)]
pub struct Cf {
    pub lb: u16,
    pub ekw: WidgetType,
    pub dr: u32,
    pub dui: u32,
    pub dpc: Vec<u16>,
    pub gad: u32,
    pub eme: u32,
}


#[derive(Debug, Clone)]
pub struct Bcd {
    pub bnw: u16,
    pub eak: u16,
    pub path: Vec<u16>,  
    pub ceb: &'static str,
}


#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
struct Byl {
    re: u64,    
    go: u32,     
    twi: u32,        
}


pub struct HdaController {
    
    hv: u64,
    
    goe: u8,
    
    htf: u8,
    
    jhn: u8,
    
    jzk: bool,

    
    ffv: u64,
    ffu: u64,
    fft: u16,

    
    ftb: u64,
    fta: u64,
    fsz: u16,
    gra: u16,  

    
    bml: Vec<u8>,
    
    widgets: Vec<Cf>,
    
    bhq: Vec<Bcd>,

    
    ejf: u8,
    
    dym: u64,
    kbk: u64,
    btf: u32,
    
    gav: u64,
    ded: u64,

    
    uu: bool,
    
    fzy: u32,
    fzx: u32,
}


static Fa: Mutex<Option<HdaController>> = Mutex::new(None);
static AVS_: AtomicBool = AtomicBool::new(false);





impl HdaController {
    #[inline]
    unsafe fn akm(&self, l: u32) -> u8 {
        core::ptr::read_volatile((self.hv + l as u64) as *const u8)
    }

    #[inline]
    unsafe fn aym(&self, l: u32) -> u16 {
        core::ptr::read_volatile((self.hv + l as u64) as *const u16)
    }

    #[inline]
    unsafe fn amp(&self, l: u32) -> u32 {
        core::ptr::read_volatile((self.hv + l as u64) as *const u32)
    }

    #[inline]
    unsafe fn akw(&self, l: u32, ap: u8) {
        core::ptr::write_volatile((self.hv + l as u64) as *mut u8, ap);
    }

    #[inline]
    unsafe fn asg(&self, l: u32, ap: u16) {
        core::ptr::write_volatile((self.hv + l as u64) as *mut u16, ap);
    }

    #[inline]
    unsafe fn aiu(&self, l: u32, ap: u32) {
        core::ptr::write_volatile((self.hv + l as u64) as *mut u32, ap);
    }

    
    fn evt(&self, bo: u8) -> u32 {
        reg::CRY_ + ((self.goe + bo) as u32) * reg::CRZ_
    }

    
    
    

    
    pub fn init(ba: &crate::pci::S) -> Result<Self, &'static str> {
        crate::serial_println!("[HDA] Initializing Intel HDA controller...");
        crate::serial_println!("[HDA]   PCI {:02X}:{:02X}.{} {:04X}:{:04X}",
            ba.aq, ba.de, ba.gw, ba.ml, ba.mx);

        
        crate::pci::fhp(ba);
        crate::pci::fhq(ba);

        
        let fcz = ba.cje(0).ok_or("HDA: no BAR0")?;
        crate::serial_println!("[HDA]   BAR0 phys = {:#010X}", fcz);

        
        let hp = crate::memory::lr();
        let hv = fcz + hp;

        
        for awl in 0..4 {
            let ht = (fcz & !0xFFF) + awl * 0x1000;
            let ju = ht + hp;
            crate::memory::paging::oky(ju, ht)?;
        }

        crate::serial_println!("[HDA]   MMIO mapped at virt {:#018X}", hv);

        let mut db = HdaController {
            hv,
            goe: 0, htf: 0, jhn: 0,
            jzk: false,
            ffv: 0, ffu: 0, fft: 0,
            ftb: 0, fta: 0, fsz: 0,
            gra: 0,
            bml: Vec::new(),
            widgets: Vec::new(),
            bhq: Vec::new(),
            ejf: 1,
            dym: 0, kbk: 0, btf: 0,
            gav: 0, ded: 0,
            uu: false,
            fzy: 0,
            fzx: 0,
        };

        
        unsafe {
            let cew = db.aym(reg::Bhn);
            let xsm = db.akm(reg::Cpe);
            let xsh = db.akm(reg::Cpc);

            db.htf = ((cew >> 12) & 0xF) as u8;
            db.goe = ((cew >> 8) & 0xF) as u8;
            db.jhn = ((cew >> 3) & 0x1F) as u8;
            db.jzk = (cew & 1) != 0;

            crate::serial_println!("[HDA]   Version {}.{}", xsh, xsm);
            crate::serial_println!("[HDA]   Streams: {} output, {} input, {} bidir",
                db.htf, db.goe, db.jhn);
            crate::serial_println!("[HDA]   64-bit: {}", db.jzk);

            if db.htf == 0 {
                return Err("HDA: no output streams available");
            }
        }

        
        db.apa()?;

        
        db.wkl()?;

        
        db.ryd()?;

        
        db.stn();

        
        db.wlf()?;

        crate::serial_println!("[HDA] Initialization complete!");
        Ok(db)
    }

    
    fn apa(&mut self) -> Result<(), &'static str> {
        crate::serial_println!("[HDA] Resetting controller...");
        unsafe {
            
            self.asg(reg::Ayi, 0xFFFF);

            
            let gctl = self.amp(reg::Km);
            self.aiu(reg::Km, gctl & !gctl::Rs);

            
            for _ in 0..1000 {
                if self.amp(reg::Km) & gctl::Rs == 0 {
                    break;
                }
                Self::azo(10);
            }
            if self.amp(reg::Km) & gctl::Rs != 0 {
                return Err("HDA: reset enter timeout");
            }

            
            let gctl = self.amp(reg::Km);
            self.aiu(reg::Km, gctl | gctl::Rs);

            
            for _ in 0..1000 {
                if self.amp(reg::Km) & gctl::Rs != 0 {
                    break;
                }
                Self::azo(10);
            }
            if self.amp(reg::Km) & gctl::Rs == 0 {
                return Err("HDA: reset exit timeout");
            }

            
            
            
            let mut fvp = 0u16;
            for kbj in 0..10 {
                Self::azo(if kbj == 0 { 1000 } else { 5000 });
                fvp = self.aym(reg::Ayi);
                if fvp != 0 { break; }
            }

            
            let gctl = self.amp(reg::Km);
            self.aiu(reg::Km, gctl | gctl::Bvb);

            
            
            self.aiu(reg::Bsa, 0x00000000);

            
            
            self.aiu(reg::Cao, 0);
            self.aiu(reg::Can, 0x01); 

            crate::serial_println!("[HDA]   STATESTS = {:#06X} (codec presence)", fvp);

            if fvp == 0 {
                return Err("HDA: no codecs detected after reset");
            }

            
            for a in 0..15u8 {
                if fvp & (1 << a) != 0 {
                    self.bml.push(a);
                    crate::serial_println!("[HDA]   Codec {} present", a);
                }
            }
        }
        Ok(())
    }

    
    
    

    fn wkl(&mut self) -> Result<(), &'static str> {
        crate::serial_println!("[HDA] Setting up CORB/RIRB...");
        let hp = crate::memory::lr();

        unsafe {
            
            self.akw(reg::Bdb, 0);
            self.akw(reg::Bqc, 0);
            Self::azo(100);

            
            let ngc = self.akm(reg::Bdd);
            let (roy, fft) = if ngc & 0x40 != 0 {
                (2u8, 256u16)
            } else if ngc & 0x20 != 0 {
                (1, 16)
            } else {
                (0, 2)
            };
            self.akw(reg::Bdd, roy);
            self.fft = fft;
            crate::serial_println!("[HDA]   CORB: {} entries", fft);

            
            let ngb = (fft as usize) * 4;
            let nga: Vec<u8> = vec![0u8; ngb + 4096]; 
            let roz = nga.fq() as u64;
            let ffv = (roz + 0xFFF) & !0xFFF; 
            core::mem::forget(nga);

            let ffu = ffv.enj(hp)
                .ok_or("HDA: CORB virt->phys failed")?;
            self.ffv = ffv;
            self.ffu = ffu;

            
            core::ptr::ahx(ffv as *mut u8, 0, ngb);

            
            self.aiu(reg::Bzc, ffu as u32);
            self.aiu(reg::Bzd, (ffu >> 32) as u32);

            
            self.asg(reg::Bdc, 1 << 15); 
            Self::azo(100);
            
            self.asg(reg::Bdc, 0);
            Self::azo(100);

            
            self.asg(reg::Apq, 0);

            
            let pdo = self.akm(reg::Bqd);
            let (vzh, fsz) = if pdo & 0x40 != 0 {
                (2u8, 256u16)
            } else if pdo & 0x20 != 0 {
                (1, 16)
            } else {
                (0, 2)
            };
            self.akw(reg::Bqd, vzh);
            self.fsz = fsz;
            crate::serial_println!("[HDA]   RIRB: {} entries", fsz);

            
            let pdn = (fsz as usize) * 8;
            let pdm: Vec<u8> = vec![0u8; pdn + 4096];
            let vzi = pdm.fq() as u64;
            let ftb = (vzi + 0xFFF) & !0xFFF;
            core::mem::forget(pdm);

            let fta = ftb.enj(hp)
                .ok_or("HDA: RIRB virt->phys failed")?;
            self.ftb = ftb;
            self.fta = fta;

            core::ptr::ahx(ftb as *mut u8, 0, pdn);

            
            self.aiu(reg::Cjw, fta as u32);
            self.aiu(reg::Cjy, (fta >> 32) as u32);

            
            self.asg(reg::Bqe, 1 << 15);
            Self::azo(100);

            
            self.asg(reg::Cjv, 1);

            self.gra = 0;

            
            self.akw(reg::Bdb, 0x02); 
            self.akw(reg::Bqc, 0x02); 
            Self::azo(100);

            crate::serial_println!("[HDA]   CORB phys={:#010X}, RIRB phys={:#010X}",
                ffu, fta);
        }

        Ok(())
    }

    
    fn phy(&mut self, codec: u8, lb: u16, verb: u32, ew: u32) -> Result<u32, &'static str> {
        
        let cmd = ((codec as u32) << 28)
            | ((lb as u32 & 0xFF) << 20)
            | (verb & 0xFFFFF);
        
        
        
        let _ = ew; 

        unsafe {
            
            let zd = self.aym(reg::Apq) & 0xFF;
            let oqi = ((zd + 1) % self.fft) as u16;

            let rox = self.ffv as *mut u32;
            core::ptr::write_volatile(rox.add(oqi as usize), cmd);

            
            self.asg(reg::Apq, oqi);

            
            for _ in 0..10000 {
                let vzj = self.aym(reg::Bqe) & 0xFF;
                if vzj != self.gra {
                    
                    self.gra = (self.gra + 1) % self.fsz;
                    let vzg = self.ftb as *const u64;
                    let mk = core::ptr::read_volatile(vzg.add(self.gra as usize));
                    let f = mk as u32;
                    
                    self.akw(reg::Cjx, 0x05);
                    return Ok(f);
                }
                Self::azo(10);
            }
        }
        Err("HDA: RIRB timeout")
    }

    
    fn adp(&mut self, codec: u8, lb: u16, verb: u32, f: u8) -> Result<u32, &'static str> {
        let szc = (verb << 8) | (f as u32);
        self.phy(codec, lb, szc, 0)
    }

    
    fn cku(&mut self, codec: u8, lb: u16, evz: u32) -> Result<u32, &'static str> {
        self.adp(codec, lb, verb::EI_, evz as u8)
    }

    
    fn atk(&mut self, codec: u8, lb: u16, xre: u32, ew: u16) -> Result<u32, &'static str> {
        
        
        let vqi = ((xre & 0xF00) << 8) | (ew as u32);
        self.phy(codec, lb, vqi, 0)
    }

    
    
    

    fn ryd(&mut self) -> Result<(), &'static str> {
        let bml = self.bml.clone();
        for &cjn in &bml {
            crate::serial_println!("[HDA] Walking codec {}...", cjn);

            
            let acs = self.cku(cjn, 0, verb::AGI_)?;
            crate::serial_println!("[HDA]   Vendor={:04X}, Device={:04X}",
                acs >> 16, acs & 0xFFFF);

            
            let fpb = self.cku(cjn, 0, verb::BCP_)?;
            let jrm = ((fpb >> 16) & 0xFF) as u16;
            let ort = (fpb & 0xFF) as u16;
            crate::serial_println!("[HDA]   Root: subnodes {}..{}", jrm, jrm + ort - 1);

            
            for ggx in jrm..(jrm + ort) {
                let sry = self.cku(cjn, ggx, verb::CIP_)?;
                let kvo = sry & 0xFF;
                crate::serial_println!("[HDA]   FG NID {}: type={} ({})", ggx, kvo,
                    if kvo == 1 { "Audio" } else { "Other" });

                if kvo != 1 { continue; } 

                
                let _ = self.adp(cjn, ggx, verb::PL_, 0x00); 

                
                self.fzy = self.cku(cjn, ggx, verb::OT_).unwrap_or(0);
                self.fzx = self.cku(cjn, ggx, verb::WB_).unwrap_or(0);
                crate::serial_println!("[HDA]   AFG amp caps: out={:#010X} in={:#010X}",
                    self.fzy, self.fzx);

                
                let ppj = self.cku(cjn, ggx, verb::BCP_)?;
                let jwe = ((ppj >> 16) & 0xFF) as u16;
                let pyz = (ppj & 0xFF) as u16;
                crate::serial_println!("[HDA]   AFG widgets: {}..{}", jwe, jwe + pyz - 1);

                
                for lb in jwe..(jwe + pyz) {
                    let dr = self.cku(cjn, lb, verb::OU_)?;
                    let ash = WidgetType::sxs(dr);

                    let mut bsy = Cf {
                        lb,
                        ekw: ash,
                        dr,
                        dui: 0,
                        dpc: Vec::new(),
                        gad: 0,
                        eme: 0,
                    };

                    
                    let nfm = self.cku(cjn, lb, verb::CIO_)?;
                    let kkm = (nfm & 0x7F) as u16;
                    let uif = (nfm & 0x80) != 0;

                    if kkm > 0 && !uif {
                        
                        let mut l = 0u8;
                        while (l as u16) < kkm {
                            let lj = self.adp(cjn, lb, verb::BWU_, l)?;
                            for a in 0..4u32 {
                                if (l as u16) + (a as u16) >= kkm { break; }
                                let kkn = ((lj >> (a * 8)) & 0xFF) as u16;
                                bsy.dpc.push(kkn);
                            }
                            l += 4;
                        }
                    }

                    
                    if ash == WidgetType::Tx {
                        bsy.dui = self.adp(cjn, lb, verb::BWT_, 0)?;
                    }

                    
                    
                    
                    let mvp = dr & (1 << 3) != 0;
                    if dr & (1 << 2) != 0 { 
                        if mvp {
                            bsy.eme = self.cku(cjn, lb, verb::OT_)?;
                            if bsy.eme == 0 {
                                bsy.eme = self.fzy;
                            }
                        } else {
                            
                            bsy.eme = self.fzy;
                        }
                    }
                    if dr & (1 << 1) != 0 { 
                        if mvp {
                            bsy.gad = self.cku(cjn, lb, verb::WB_)?;
                            if bsy.gad == 0 {
                                bsy.gad = self.fzx;
                            }
                        } else {
                            bsy.gad = self.fzx;
                        }
                    }

                    crate::serial_println!("[HDA]     NID {:3}: {} conns={:?}{}",
                        lb, ash.j(),
                        bsy.dpc,
                        if ash == WidgetType::Tx {
                            alloc::format!(" [{}]", lty(bsy.dui))
                        } else {
                            String::new()
                        }
                    );

                    self.widgets.push(bsy);
                }
            }
        }
        Ok(())
    }

    
    fn stn(&mut self) {
        crate::serial_println!("[HDA] Searching output paths...");

        
        let vic: Vec<(u16, u32, Vec<u16>)> = self.widgets.iter()
            .hi(|d| d.ekw == WidgetType::Tx)
            .hi(|d| {
                
                let hdw = (d.dui >> 30) & 0x3;
                let rva = (d.dui >> 20) & 0xF;
                
                
                
                let tyn = oh!(rva,
                    0x0 | 0x1 | 0x2 | 0x4 | 0x5 | 0x6 | 0xF);
                
                
                let rnv = hdw == 0 || hdw == 2; 
                (hdw != 1 && tyn) || rnv
            })
            .map(|d| (d.lb, d.dui, d.dpc.clone()))
            .collect();

        for (bnw, dui, zfe) in &vic {
            
            if let Some(path) = self.pvp(*bnw, &mut Vec::new()) {
                let de = lty(*dui);
                crate::serial_println!("[HDA]   Path found: {} -> {:?}", de,
                    path.iter().map(|bo| alloc::format!("{}", bo)).collect::<Vec<_>>());
                self.bhq.push(Bcd {
                    bnw: *bnw,
                    eak: *path.qv().unwrap_or(&0),
                    path: path,
                    ceb: de,
                });
            }
        }

        if self.bhq.is_empty() {
            crate::serial_println!("[HDA]   WARNING: No output paths found!");
        } else {
            crate::serial_println!("[HDA]   {} output path(s) found", self.bhq.len());
        }
    }

    
    fn pvp(&self, lb: u16, bxs: &mut Vec<u16>) -> Option<Vec<u16>> {
        if bxs.contains(&lb) { return None; } 
        bxs.push(lb);

        let bsy = self.widgets.iter().du(|d| d.lb == lb)?;

        if bsy.ekw == WidgetType::Zw {
            return Some(vec![lb]); 
        }

        
        for &kkn in &bsy.dpc {
            if let Some(mut path) = self.pvp(kkn, bxs) {
                path.insert(0, lb);
                return Some(path);
            }
        }

        None
    }

    
    
    

    fn wlf(&mut self) -> Result<(), &'static str> {
        if self.bhq.is_empty() {
            return Err("HDA: no output paths to configure");
        }

        let hp = crate::memory::lr();
        let codec = self.bml[0];
        let path = self.bhq[0].clone();

        crate::serial_println!("[HDA] Setting up output stream for path: {:?}", path.path);

        
        
        for &lb in &path.path {
            let _ = self.adp(codec, lb, verb::PL_, 0x00); 
        }

        
        
        let ml = self.cku(codec, 0, verb::AGI_).unwrap_or(0);
        let mpa = (ml >> 16) & 0xFFFF;
        let rlm = ml & 0xFFFF;
        crate::serial_println!("[HDA]   Codec vendor={:#06X} device={:#06X}", mpa, rlm);

        
        
        let lfu = mpa == 0x11D4; 
        let txa = mpa == 0x14F1; 
        let urs = lfu || txa;

        if urs {
            crate::serial_println!("[HDA]   Applying {} codec quirks",
                if lfu { "Analog Devices AD198x" } else { "Conexant CX205xx" });

            
            let qgl: Vec<u16> = self.widgets.iter().map(|d| d.lb).collect();
            let vaa: Vec<u16> = self.widgets.iter()
                .hi(|d| d.ekw == WidgetType::Tx
                    && oh!((d.dui >> 20) & 0xF, 0x0 | 0x1 | 0x2))
                .map(|d| d.lb)
                .collect();

            
            let _ = self.adp(codec, 1, verb::PL_, 0x00);
            HdaController::azo(10_000); 
            for &lb in &qgl {
                let _ = self.adp(codec, lb, verb::PL_, 0x00);
            }
            
            HdaController::azo(100_000); 

            
            
            for &lb in &vaa {
                let _ = self.adp(codec, lb, verb::AHT_, 0xC0);
                
                
                let _ = self.adp(codec, lb, verb::AHS_, 0x02);
                crate::serial_println!("[HDA]   Pin NID {} -> EAPD=0x02, PIN_CTL=0xC0", lb);
            }

            
            
            
            if lfu {
                let _ = self.atk(codec, 1, verb::CSL_, 0x0008);
                crate::serial_println!("[HDA]   AD1984 DMIC COEF: val=0x08 (default index)");
            }

            
            
            let _ = self.atk(codec, 1, 0x300,
                (1u16 << 15) | (1 << 13) | (1 << 12) | 0x27);
            let _ = self.atk(codec, 1, 0x300,
                (1u16 << 14) | (1 << 13) | (1 << 12) | 0x27);

            
            
            
            
            let _ = self.adp(codec, 1, verb::CSK_, 0x02); 
            let _ = self.adp(codec, 1, verb::CSJ_,  0x02); 
            let _ = self.adp(codec, 1, verb::BFM_, 0x02); 
            crate::serial_println!("[HDA]   GPIO1 HIGH (speaker amp power on)");
        }

        
        let _ = self.adp(codec, path.bnw, verb::AHT_, 0xC0);
        
        
        let _ = self.adp(codec, path.bnw, verb::AHS_, 0x02);
        crate::serial_println!("[HDA]   Output pin {} -> EAPD=0x02, PIN_CTL=0xC0", path.bnw);

        
        
        let ejf = self.ejf;
        let channel = 0u8;
        let fmt: u16 = 0x0011; 

        let qgg: Vec<u16> = self.widgets.iter()
            .hi(|d| d.ekw == WidgetType::Zw)
            .map(|d| d.lb)
            .collect();

        for &eak in &qgg {
            let _ = self.adp(codec, eak, verb::CSH_,
                (ejf << 4) | channel);
            let _ = self.atk(codec, eak, verb::CSN_, fmt);
            crate::serial_println!("[HDA]   DAC NID {} -> stream_tag={}, fmt=0x{:04X}",
                eak, ejf, fmt);
        }

        
        HdaController::azo(5000);

        
        
        
        let muv: Vec<(u16, u32, usize, u32, u32)> = self.widgets.iter()
            .map(|d| (d.lb, d.dr, d.dpc.len(), d.eme, d.gad))
            .collect();

        
        let ije = ((self.fzy >> 8) & 0x7F) as u16;
        let muh = ((self.fzx >> 8) & 0x7F) as u16;

        for &(lb, dr, orp, lra, ldr) in &muv {
            
            
            let goq = ((lra >> 8) & 0x7F) as u16;
            let gju = ((ldr >> 8) & 0x7F) as u16;
            
            
            
            let jia = if goq > 0 { goq } else { ije };
            let hnt = if gju > 0 { gju } else { muh };

            
            
            
            let kam: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (jia & 0x7F);
            let _ = self.atk(codec, lb, 0x300, kam);
            
            let kak: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (hnt & 0x7F);
            let _ = self.atk(codec, lb, 0x300, kak);

            
            if orp > 1 {
                for w in 1..orp.v(16) {
                    let qho: u16 = (1 << 14) | (1 << 13) | (1 << 12) | ((w as u16 & 0xF) << 8) | (hnt & 0x7F);
                    let _ = self.atk(codec, lb, 0x300, qho);
                }
            }
        }
        crate::serial_println!("[HDA]   Unmuted all {} widget amps (separate OUT/IN, afg_out={} afg_in={})",
            muv.len(), ije, muh);

        
        
        
        let jzp = if ije > 0 { ije } else { 3u16 };
        let qgn: Vec<(u16, Vec<u16>)> = self.bhq.iter()
            .map(|ai| (ai.bnw, ai.path.clone()))
            .collect();

        for (bnw, vex) in &qgn {
            
            for &lb in vex {
                let _ = self.adp(codec, lb, verb::PL_, 0x00);
            }
            
            let vhv: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (jzp & 0x7F);
            let _ = self.atk(codec, *bnw, 0x300, vhv);
            
            let vhw: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (jzp & 0x7F);
            let _ = self.atk(codec, *bnw, 0x300, vhw);
            
            let _ = self.adp(codec, *bnw, verb::AHT_, 0xC0);
            let _ = self.adp(codec, *bnw, verb::AHS_, 0x02);
            crate::serial_println!("[HDA]   Path pin NID {} -> amp forced gain={}, EAPD+OUT",
                bnw, jzp);
        }

        
        
        
        let qgo: Vec<Vec<u16>> = self.bhq.iter()
            .map(|ai| ai.path.clone())
            .collect();

        for evu in &qgo {
            let vey: Vec<(u16, WidgetType, Vec<u16>)> = evu.iter()
                .kwb(|&lb| self.widgets.iter().du(|d| d.lb == lb)
                    .map(|d| (lb, d.ekw, d.dpc.clone())))
                .collect();

            for (lb, ydy, dpc) in &vey {
                
                
                
                let uuh = evu.iter()
                    .qf(|&bo| bo == *lb)
                    .and_then(|u| evu.get(u + 1))
                    .hu();
                if let Some(oql) = uuh {
                    if let Some(w) = dpc.iter().qf(|&r| r == oql) {
                        let _ = self.adp(codec, *lb, verb::CSI_, w as u8);
                        crate::serial_println!("[HDA]   NID {} conn_sel={} (-> NID {})",
                            lb, w, oql);
                    }
                }
            }
        }

        
        let uh = self.evt(0); 

        unsafe {
            
            let bqb = self.amp(uh + sd::Cx) & 0xFF;
            self.akw(uh + sd::Cx, (bqb as u8) | sctl::Mh as u8);
            Self::azo(100);
            
            for _ in 0..1000 {
                if self.akm(uh + sd::Cx) & (sctl::Mh as u8) != 0 { break; }
                Self::azo(10);
            }
            
            self.akw(uh + sd::Cx, 0);
            for _ in 0..1000 {
                if self.akm(uh + sd::Cx) & (sctl::Mh as u8) == 0 { break; }
                Self::azo(10);
            }

            
            self.akw(uh + sd::Ui, 0x1C);

            
            let kwz: u32 = 524288; 
            let evn: u32 = 2;
            let aay = kwz * evn;

            let mwt: Vec<u8> = vec![0u8; aay as usize + 4096];
            let qss = mwt.fq() as u64;
            let aak = (qss + 0xFFF) & !0xFFF;
            core::mem::forget(mwt);

            let rg = aak.enj(hp)
                .ok_or("HDA: audio buf virt->phys failed")?;

            self.dym = aak;
            self.kbk = rg;
            self.btf = aay;

            
            core::ptr::ahx(aak as *mut u8, 0, aay as usize);

            
            let myh: Vec<u8> = vec![0u8; 256 + 4096]; 
            let qoh = myh.fq() as u64;
            let gav = (qoh + 127) & !127; 
            core::mem::forget(myh);

            let ded = gav.enj(hp)
                .ok_or("HDA: BDL virt->phys failed")?;

            self.gav = gav;
            self.ded = ded;

            
            let qog = gav as *mut Byl;
            for a in 0..evn {
                let bt = &mut *qog.add(a as usize);
                bt.re = rg + (a as u64) * (kwz as u64);
                bt.go = kwz;
                bt.twi = 1; 
            }

            
            self.aiu(uh + sd::Agw, aay);  
            self.asg(uh + sd::Ajp, (evn - 1) as u16); 
            self.asg(uh + sd::Aia, fmt);  
            self.aiu(uh + sd::Agm, ded as u32);
            self.aiu(uh + sd::Agn, (ded >> 32) as u32);

            
            let hem = (ejf as u32) << (sctl::AIJ_ - 16);
            self.akw(uh + sd::Cx + 2, hem as u8);

            crate::serial_println!("[HDA]   Stream configured: 48kHz 16-bit stereo");
            crate::serial_println!("[HDA]   Audio buf phys={:#010X} size={}",
                rg, aay);
            crate::serial_println!("[HDA]   BDL phys={:#010X} entries={}", ded, evn);
        }

        Ok(())
    }

    
    
    

    
    
    
    
    fn lzp(&mut self) {
        if self.btf == 0 { return; }
        let uh = self.evt(0);
        unsafe {
            
            let bqb = self.akm(uh + sd::Cx);
            self.akw(uh + sd::Cx, (bqb & !(sctl::Ub as u8)) | sctl::Mh as u8);
            for _ in 0..1000 {
                if self.akm(uh + sd::Cx) & (sctl::Mh as u8) != 0 { break; }
                HdaController::azo(10);
            }
            
            self.akw(uh + sd::Cx, 0);
            for _ in 0..1000 {
                if self.akm(uh + sd::Cx) & (sctl::Mh as u8) == 0 { break; }
                HdaController::azo(10);
            }
            
            self.akw(uh + sd::Ui, 0x1C);

            
            let evn: u16 = 2;
            let fmt: u16 = 0x0011; 
            self.aiu(uh + sd::Agw, self.btf);
            self.asg(uh + sd::Ajp, evn - 1);
            self.asg(uh + sd::Aia, fmt);
            self.aiu(uh + sd::Agm, self.ded as u32);
            self.aiu(uh + sd::Agn, (self.ded >> 32) as u32);

            
            let hem = (self.ejf as u32) << (sctl::AIJ_ - 16);
            self.akw(uh + sd::Cx + 2, hem as u8);
        }
        self.uu = false;
        crate::serial_println!("[HDA] Stream reset (LPIB→0, reconfig done)");
    }

    pub fn ssn(&mut self, auf: u32, uk: u32) {
        let auy = 48000u32;
        let lq = 2u32;
        let nbb = 2u32; 
        let ayz = (auy * uk / 1000) as usize;
        let qsr = (self.btf / (lq * nbb)) as usize;
        let pfi = ayz.v(qsr);

        let k = self.dym as *mut i16;

        
        let awn = auy / auf;
        if awn == 0 { return; }
        let exd = (awn / 4).am(1);
        let dyg: i32 = 16000; 

        unsafe {
            for a in 0..pfi {
                let u = (a as u32) % awn;
                
                
                let wcm: i32 = if u < exd {
                    dyg * u as i32 / exd as i32
                } else if u < 3 * exd {
                    dyg * (2 * exd as i32 - u as i32) / exd as i32
                } else {
                    dyg * (u as i32 - awn as i32) / exd as i32
                };
                
                let yr = wcm.qp(-32000, 32000) as i16;

                
                let w = a * lq as usize;
                *k.add(w) = yr;
                *k.add(w + 1) = yr;
            }

            
            let kvy = pfi * lq as usize * nbb as usize;
            if kvy < self.btf as usize {
                core::ptr::ahx(
                    (self.dym as *mut u8).add(kvy),
                    0,
                    self.btf as usize - kvy
                );
            }
        }
    }

    
    pub fn daq(&mut self, ay: bool) {
        let uh = self.evt(0);
        unsafe {
            if ay {
                
                let lez = self.amp(reg::Atu);
                let wva = 1u32 << (self.goe as u32); 
                self.aiu(reg::Atu, lez | (1 << 31) | (1 << 30) | wva);

                
                self.akw(uh + sd::Ui, 0x1C);

                
                
                let bqb = self.akm(uh + sd::Cx);
                self.akw(uh + sd::Cx, (bqb | sctl::Ub as u8) & !(sctl::Cfo as u8));

                self.uu = true;
                crate::serial_println!("[HDA] Playback started");
            } else {
                
                let bqb = self.akm(uh + sd::Cx);
                self.akw(uh + sd::Cx, bqb & !(sctl::Ub as u8));

                self.uu = false;
                crate::serial_println!("[HDA] Playback stopped");
            }
        }
    }

    
    pub fn lgj(&self) -> bool {
        self.uu
    }

    
    pub fn eje(&self) -> u32 {
        let uh = self.evt(0);
        unsafe { self.amp(uh + sd::Bkm) }
    }

    
    
    

    fn azo(ifz: u64) {
        
        for _ in 0..ifz {
            unsafe {
                let mut port: crate::arch::Port<u8> = crate::arch::Port::new(0x80);
                port.write(0);
            }
        }
    }

    
    pub fn wtt(&self) -> String {
        let mut e = String::new();
        e.t(&format!("Intel HDA Controller\n"));
        e.t(&format!("  Streams: {} out, {} in, {} bidir\n",
            self.htf, self.goe, self.jhn));
        e.t(&format!("  Codecs: {:?}\n", self.bml));
        e.t(&format!("  Widgets: {}\n", self.widgets.len()));
        e.t(&format!("  Output paths: {}\n", self.bhq.len()));
        for (a, ai) in self.bhq.iter().cf() {
            e.t(&format!("    [{}] {} -> path {:?}\n", a, ai.ceb, ai.path));
        }
        e.t(&format!("  Playing: {}\n", self.uu));
        if self.uu {
            e.t(&format!("  Position: {}\n", self.eje()));
        }
        e
    }
}






pub fn init() -> Result<(), &'static str> {
    
    let ik = crate::pci::ebq(crate::pci::class::Blx);
    let tns = ik.iter()
        .du(|bc| bc.adl == 0x03) 
        .or_else(|| ik.iter().du(|bc| bc.adl == 0x01)) 
        .ok_or("HDA: no Intel HDA device found on PCI bus")?
        .clone();

    let db = HdaController::init(&tns)?;
    *Fa.lock() = Some(db);
    AVS_.store(true, Ordering::SeqCst);

    Ok(())
}


pub fn ky() -> bool {
    AVS_.load(Ordering::SeqCst)
}


pub fn owd(auf: u32, uk: u32) -> Result<(), &'static str> {
    let mut hda = Fa.lock();
    let db = hda.as_mut().ok_or("HDA: not initialized")?;

    
    db.lzp();
    db.ssn(auf, uk);

    
    let vkb = db.eje();
    db.daq(true);

    
    HdaController::azo(5000); 
    let owr = db.eje();

    
    let auy = 48000u32;
    let xv = (auy * uk / 1000) * 4; 
    let cd = xv.v(db.btf);

    for _ in 0..(uk * 10) {
        HdaController::azo(100);
        let u = db.eje();
        if u >= cd {
            break;
        }
    }

    let owq = db.eje();
    db.daq(false);

    
    crate::serial_println!("[HDA] play_tone: LPIB before={} early={} after={} target={}",
        vkb, owr, owq, cd);
    if owr == 0 && owq == 0 {
        crate::serial_println!("[HDA] WARNING: LPIB never advanced! DMA may not be running.");
    }

    Ok(())
}


pub fn wix(ap: u8) -> Result<(), &'static str> {
    let mut hda = Fa.lock();
    let db = hda.as_mut().ok_or("HDA: not initialized")?;
    if db.bml.is_empty() { return Err("No codecs"); }
    let codec = db.bml[0];
    let _ = db.adp(codec, 1, verb::BFM_, ap);
    crate::serial_println!("[HDA] GPIO DATA set to {:#04X}", ap);
    Ok(())
}


pub fn qg() -> Result<(), &'static str> {
    let mut hda = Fa.lock();
    let db = hda.as_mut().ok_or("HDA: not initialized")?;
    db.daq(false);
    Ok(())
}


pub fn tdy() -> u32 {
    let hda = Fa.lock();
    match hda.as_ref() {
        Some(db) => db.eje(),
        None => 0,
    }
}





pub fn jmf() {
    let hda = Fa.lock();
    if let Some(db) = hda.as_ref() {
        if db.btf == 0 { return; }
        let uh = db.evt(0);
        unsafe {
            
            let bqb = db.akm(uh + sd::Cx);
            db.akw(uh + sd::Cx, (bqb & !(sctl::Ub as u8)) | sctl::Mh as u8);
            for _ in 0..1000 {
                if db.akm(uh + sd::Cx) & (sctl::Mh as u8) != 0 { break; }
                HdaController::azo(10);
            }
            
            db.akw(uh + sd::Cx, 0);
            for _ in 0..1000 {
                if db.akm(uh + sd::Cx) & (sctl::Mh as u8) == 0 { break; }
                HdaController::azo(10);
            }
            
            db.akw(uh + sd::Ui, 0x1C);

            
            let evn: u16 = 2;
            let fmt: u16 = 0x0011; 
            db.aiu(uh + sd::Agw, db.btf);
            db.asg(uh + sd::Ajp, evn - 1);
            db.asg(uh + sd::Aia, fmt);
            db.aiu(uh + sd::Agm, db.ded as u32);
            db.aiu(uh + sd::Agn, (db.ded >> 32) as u32);

            
            let hem = (db.ejf as u32) << (sctl::AIJ_ - 16);
            db.akw(uh + sd::Cx + 2, hem as u8);
        }
        crate::serial_println!("[HDA] Stream reset (LPIB→0, reconfig done)");
    }
}






pub fn dcg(un: &[i16]) -> Result<(), &'static str> {
    let mut hda = Fa.lock();
    let db = hda.as_mut().ok_or("HDA: not initialized")?;

    
    if db.uu {
        db.daq(false);
    }

    
    db.lzp();

    
    if db.dym == 0 || db.btf == 0 {
        return Err("HDA: DMA buffer not initialized");
    }

    
    let k = db.dym as *mut i16;
    let fdv = (db.btf / 2) as usize;
    let acq = un.len().v(fdv);

    unsafe {
        core::ptr::copy_nonoverlapping(un.fq(), k, acq);
        
        if acq < fdv {
            core::ptr::ahx(k.add(acq), 0, fdv - acq);
        }
    }

    let njk = (acq * 2) as u32;
    crate::serial_println!("[HDA] Looped playback: {} bytes ({} ms), buf={}",
        njk, njk / (48000 * 4 / 1000), db.btf);

    
    
    
    db.daq(true);
    Ok(())
}




pub fn gic() -> Option<(*mut i16, usize)> {
    let hda = Fa.lock();
    let db = hda.as_ref()?;
    if db.dym == 0 { return None; }
    Some((db.dym as *mut i16, (db.btf / 2) as usize))
}


pub fn lgj() -> bool {
    let hda = Fa.lock();
    match hda.as_ref() {
        Some(db) => db.lgj(),
        None => false,
    }
}


pub fn hlj() -> u32 {
    let hda = Fa.lock();
    match hda.as_ref() {
        Some(db) => db.eje(),
        None => 0,
    }
}



pub fn wsr() -> Result<(), &'static str> {
    let mut hda = Fa.lock();
    let db = hda.as_mut().ok_or("HDA: not initialized")?;
    if !db.uu {
        db.daq(true);
    }
    Ok(())
}




pub fn ndi() {
    let hda = Fa.lock();
    if let Some(db) = hda.as_ref() {
        let uh = db.evt(0);
        unsafe {
            
            db.akw(uh + sd::Ui, 0x1C);
        }
    }
}




pub fn nqi() -> bool {
    let hda = Fa.lock();
    if let Some(db) = hda.as_ref() {
        if !db.uu { return false; }
        let uh = db.evt(0);
        unsafe {
            let bqb = db.akm(uh + sd::Cx);
            if bqb & (sctl::Ub as u8) == 0 {
                
                db.akw(uh + sd::Ui, 0x1C);
                db.akw(uh + sd::Cx, bqb | sctl::Ub as u8);
                crate::serial_println!("[HDA] Stream stalled — restarted (LPIB={})",
                    db.eje());
                return true;
            }
        }
    }
    false
}


pub fn status() -> String {
    let hda = Fa.lock();
    match hda.as_ref() {
        Some(db) => db.wtt(),
        None => String::from("HDA: not initialized"),
    }
}


pub fn geq() -> String {
    let mut e = String::new();
    let mut hda = Fa.lock();
    let db = match hda.as_mut() {
        Some(r) => r,
        None => {
            e.t("HDA: not initialized\n");
            return e;
        }
    };

    unsafe {
        
        let cew = db.aym(reg::Bhn);
        let gctl = db.amp(reg::Km);
        let lez = db.amp(reg::Atu);
        let tvr = db.amp(0x24); 
        let wsa = db.amp(reg::Bsa);
        let xtn = db.amp(reg::Cqa);
        let fvp = db.aym(reg::Ayi);

        e.t(&format!("=== HDA Hardware Diagnostic ===\n"));
        e.t(&format!("GCAP={:#06X} GCTL={:#010X}\n", cew, gctl));
        e.t(&format!("INTCTL={:#010X} INTSTS={:#010X}\n", lez, tvr));
        e.t(&format!("SSYNC={:#010X} WALCLK={}\n", wsa, xtn));
        e.t(&format!("STATESTS={:#06X}\n", fvp));
        e.t(&format!("CRST={} UNSOL={}\n",
            if gctl & gctl::Rs != 0 { "OK" } else { "IN RESET!" },
            if gctl & gctl::Bvb != 0 { "on" } else { "off" }));

        
        let uh = db.evt(0);
        let kmd = db.akm(uh + sd::Cx);
        let nht = db.akm(uh + sd::Cx + 2);
        let ica = db.akm(uh + sd::Ui);
        let bvg = db.amp(uh + sd::Bkm);
        let qwz = db.amp(uh + sd::Agw);
        let uiw = db.aym(uh + sd::Ajp);
        let ssc = db.aym(uh + sd::Ccz);
        let fmt = db.aym(uh + sd::Aia);
        let qoi = db.amp(uh + sd::Agm);
        let qoj = db.amp(uh + sd::Agn);

        e.t(&format!("\n--- Output Stream 0 (base={:#X}) ---\n", uh));
        e.t(&format!("CTL[0]={:#04X} CTL[2]={:#04X} (RUN={} SRST={} TAG={})\n",
            kmd, nht,
            if kmd & sctl::Ub as u8 != 0 { "YES" } else { "no" },
            if kmd & sctl::Mh as u8 != 0 { "YES!" } else { "no" },
            nht >> 4));
        e.t(&format!("STS={:#04X} (BCIS={} FIFOE={} DESE={} FIFORDY={})\n",
            ica,
            if ica & ssts::Bye != 0 { "Y" } else { "n" },
            if ica & ssts::Ccx != 0 { "ERR" } else { "ok" },
            if ica & ssts::Cae != 0 { "ERR" } else { "ok" },
            if ica & ssts::Ccy != 0 { "Y" } else { "n" }));
        e.t(&format!("LPIB={} CBL={} LVI={} FIFOS={}\n", bvg, qwz, uiw, ssc));
        e.t(&format!("FMT={:#06X} (48kHz/16bit/stereo=0x0011)\n", fmt));
        e.t(&format!("BDL={:#010X}:{:#010X}\n", qoj, qoi));
        e.t(&format!("Audio buf phys={:#010X} size={}\n", db.kbk, db.btf));

        
        if !db.bml.is_empty() && !db.bhq.is_empty() {
            let codec = db.bml[0];
            let path = db.bhq[0].clone();
            e.t(&format!("\n--- Codec {} Path ---\n", codec));
            e.t(&format!("Path: {:?} Type={}\n", path.path, path.ceb));

            
            for &lb in &path.path {
                if let Ok(jkk) = db.adp(codec, lb, verb::TI_, 0) {
                    let elw = jkk & 0xF;
                    let cd = (jkk >> 4) & 0xF;
                    e.t(&format!("  NID {}: power D{}/D{}{}\n",
                        lb, elw, cd,
                        if elw != 0 { " NOT D0!" } else { "" }));
                }
            }

            
            if let Ok(fz) = db.adp(codec, path.bnw, verb::ASV_, 0) {
                e.t(&format!("  Pin {} PIN_CTL={:#04X} (out={})\n",
                    path.bnw, fz, if fz & 0x40 != 0 { "YES" } else { "NO!" }));
            }
            if let Ok(now) = db.adp(codec, path.bnw, verb::ASU_, 0) {
                e.t(&format!("  Pin {} EAPD={:#04X} (on={})\n",
                    path.bnw, now, if now & 0x02 != 0 { "YES" } else { "NO!" }));
            }

            
            if let Ok(jt) = db.adp(codec, path.eak, verb::AST_, 0) {
                let ll = (jt >> 4) & 0xF;
                e.t(&format!("  DAC {} STREAM_TAG={} (expect {})\n",
                    path.eak, ll, db.ejf));
            }
        }
    }
    e
}



pub fn rln() -> String {
    let mut e = String::new();
    let mut hda = Fa.lock();
    let db = match hda.as_mut() {
        Some(r) => r,
        None => {
            e.t("HDA: not initialized\n");
            return e;
        }
    };

    if db.bml.is_empty() {
        e.t("No codecs found\n");
        return e;
    }

    let codec = db.bml[0];

    
    if let Ok(cck) = db.adp(codec, 0, verb::EI_, verb::AGI_ as u8) {
        e.t(&format!("Codec {}: vendor={:#06X} device={:#06X}\n",
            codec, (cck >> 16) & 0xFFFF, cck & 0xFFFF));
    }

    e.t(&format!("Widgets: {} discovered\n", db.widgets.len()));
    e.t(&format!("Output paths: {}\n", db.bhq.len()));
    for (a, ai) in db.bhq.iter().cf() {
        e.t(&format!("  Path[{}]: pin={} dac={} type={} route={:?}\n",
            a, ai.bnw, ai.eak, ai.ceb, ai.path));
    }

    
    let tgv = db.adp(codec, 1, verb::BWW_, 0).unwrap_or(0);
    let tgx = db.adp(codec, 1, verb::BWY_, 0).unwrap_or(0);
    let tgw  = db.adp(codec, 1, verb::BWX_,  0).unwrap_or(0);
    let tgu = db.cku(codec, 1, verb::CIQ_).unwrap_or(0);
    e.t(&format!("GPIO: count={} mask={:#04X} dir={:#04X} data={:#04X}\n",
        tgu & 0xFF, tgx, tgw, tgv));

    
    e.t(&format!("\n--- Pin Widgets ---\n"));
    let vhy: Vec<(u16, u32)> = db.widgets.iter()
        .hi(|d| d.ekw == WidgetType::Tx)
        .map(|d| (d.lb, d.dui))
        .collect();

    for (lb, cfg) in &vhy {
        let ba = lty(*cfg);
        let hdw = match (*cfg >> 30) & 0x3 {
            0 => "Jack",
            1 => "None",
            2 => "Fixed",
            3 => "Both",
            _ => "?",
        };
        let cse = (*cfg >> 24) & 0x3F;

        
        let ltx = db.adp(codec, *lb, verb::ASV_, 0).unwrap_or(0);
        let nox = db.adp(codec, *lb, verb::ASU_, 0).unwrap_or(0);
        let hvt = db.adp(codec, *lb, verb::TI_, 0).unwrap_or(0);
        let cit = db.adp(codec, *lb, verb::EI_, verb::OU_ as u8).unwrap_or(0);
        
        let gym = db.atk(codec, *lb, verb::DI_, 0xA000).unwrap_or(0); 
        let gyn = db.atk(codec, *lb, verb::DI_, 0x8000).unwrap_or(0); 
        let ltw = db.adp(codec, *lb, verb::EI_, verb::CIR_ as u8).unwrap_or(0);
        
        let xuk = db.widgets.iter().du(|d| d.lb == *lb).map(|d| d.eme).unwrap_or(0);

        let uzr = ltx & 0x40 != 0;
        let tqg = ltx & 0x80 != 0;
        let sic = nox & 0x02 != 0;
        let tmj = ltw & (1 << 16) != 0;
        let ywk = ltw & (1 << 4) != 0;
        let drn = cit & (1 << 2) != 0;
        let tma = cit & (1 << 3) != 0;

        e.t(&format!("  NID {:2}: {} ({}) loc={:#04X} cfg={:#010X}\n",
            lb, ba, hdw, cse, cfg));
        e.t(&format!("         wcaps={:#010X}(out_amp={} amp_ovrd={}) pin_caps={:#010X}\n",
            cit, drn, tma, ltw));
        e.t(&format!("         pin_ctl={:#04X}(out={} hp={}) eapd={:#04X}(on={} has={})\n",
            ltx, uzr, tqg, nox, sic, tmj));
        e.t(&format!("         power=D{} amp_out L={:#04X} R={:#04X} amp_caps={:#010X}\n",
            hvt & 0xF, gym, gyn, xuk));
    }

    
    e.t(&format!("\n--- DAC Widgets ---\n"));
    let rtg: Vec<u16> = db.widgets.iter()
        .hi(|d| d.ekw == WidgetType::Zw)
        .map(|d| d.lb)
        .collect();

    for lb in &rtg {
        let hvt = db.adp(codec, *lb, verb::TI_, 0).unwrap_or(0);
        let poz = db.adp(codec, *lb, verb::AST_, 0).unwrap_or(0);
        
        let fmt = db.atk(codec, *lb, verb::BXB_, 0).unwrap_or(0);
        let cit = db.adp(codec, *lb, verb::EI_, verb::OU_ as u8).unwrap_or(0);
        let drn = cit & (1 << 2) != 0;
        let fkg = cit & (1 << 1) != 0;
        
        let gym = db.atk(codec, *lb, verb::DI_, 0xA000).unwrap_or(0); 
        let gyn = db.atk(codec, *lb, verb::DI_, 0x8000).unwrap_or(0); 
        
        let kal = if drn { db.adp(codec, *lb, verb::EI_, verb::OT_ as u8).unwrap_or(0) } else { 0 };

        e.t(&format!("  NID {:2}: power=D{} stream_tag={} chan={} fmt={:#06X}\n",
            lb, hvt & 0xF, (poz >> 4) & 0xF, poz & 0xF, fmt));
        e.t(&format!("         wcaps={:#010X}(out_amp={} in_amp={})\n",
            cit, drn, fkg));
        e.t(&format!("         amp_out L={:#04X} R={:#04X} caps={:#010X}\n",
            gym, gyn, kal));
    }

    
    e.t(&format!("\n--- Path Mixer/Selector ---\n"));
    let mut phn: Vec<u16> = Vec::new();
    let vfa: Vec<Vec<u16>> = db.bhq.iter().map(|ai| ai.path.clone()).collect();
    let xul: Vec<(u16, WidgetType, Vec<u16>)> = db.widgets.iter()
        .map(|d| (d.lb, d.ekw, d.dpc.clone()))
        .collect();
    for path in &vfa {
        for &lb in path {
            let xut = xul.iter().du(|d| d.0 == lb);
            if let Some((_, ash, aan)) = xut {
                if *ash != WidgetType::Apg && *ash != WidgetType::Aph { continue; }
                if phn.contains(&lb) { continue; }
                phn.push(lb);
                let cit = db.adp(codec, lb, verb::EI_, verb::OU_ as u8).unwrap_or(0);
                let drn = cit & (1 << 2) != 0;
                let fkg = cit & (1 << 1) != 0;
                
                let gym = db.atk(codec, lb, verb::DI_, 0xA000).unwrap_or(0);
                let gyn = db.atk(codec, lb, verb::DI_, 0x8000).unwrap_or(0);
                let qhp = db.atk(codec, lb, verb::DI_, 0x2000).unwrap_or(0);
                let qhq = db.atk(codec, lb, verb::DI_, 0x0000).unwrap_or(0);
                let kal = if drn { db.adp(codec, lb, verb::EI_, verb::OT_ as u8).unwrap_or(0) } else { 0 };
                let qhn = if fkg { db.adp(codec, lb, verb::EI_, verb::WB_ as u8).unwrap_or(0) } else { 0 };
                let rnw = db.adp(codec, lb, verb::BWV_, 0).unwrap_or(0);
                let hvt = db.adp(codec, lb, verb::TI_, 0).unwrap_or(0);

                e.t(&format!("  NID {:2}: {} conns={:?}\n", lb, ash.j(), aan));
                e.t(&format!("         wcaps={:#010X}(out_amp={} in_amp={})\n",
                    cit, drn, fkg));
                e.t(&format!("         out L={:#04X} R={:#04X} ocaps={:#010X}\n",
                    gym, gyn, kal));
                e.t(&format!("         in[0] L={:#04X} R={:#04X} icaps={:#010X}\n",
                    qhp, qhq, qhn));
                e.t(&format!("         conn_sel={} power=D{}\n", rnw, hvt & 0xF));
            }
        }
    }

    e
}




pub fn qhr() -> String {
    let mut e = String::new();
    let mut hda = Fa.lock();
    let db = match hda.as_mut() {
        Some(r) => r,
        None => {
            e.t("HDA: not initialized\n");
            return e;
        }
    };

    if db.bml.is_empty() || db.bhq.is_empty() {
        e.t("No codecs or paths\n");
        return e;
    }

    let codec = db.bml[0];
    e.t("=== Amp Probe (SET then GET) ===\n");

    
    let mut oxz: Vec<u16> = Vec::new();
    let vez: Vec<Vec<u16>> = db.bhq.iter().map(|ai| ai.path.clone()).collect();
    for path in &vez {
        for &lb in path {
            if oxz.contains(&lb) { continue; }
            oxz.push(lb);

            let cit = db.adp(codec, lb, verb::EI_, verb::OU_ as u8).unwrap_or(0);
            let ash = (cit >> 20) & 0xF;
            let drn = cit & (1 << 2) != 0;
            let fkg = cit & (1 << 1) != 0;

            
            let myn = db.atk(codec, lb, verb::DI_, 0xA000).unwrap_or(0xDEAD); 
            let mym = db.atk(codec, lb, verb::DI_, 0x2000).unwrap_or(0xDEAD);  

            
            let lra = if drn { db.cku(codec, lb, verb::OT_).unwrap_or(0) } else { 0 };
            let ldr = if fkg { db.cku(codec, lb, verb::WB_).unwrap_or(0) } else { 0 };
            let goq = ((lra >> 8) & 0x7F) as u16;
            let gju = ((ldr >> 8) & 0x7F) as u16;
            let jia = if goq > 0 { goq } else { 0x1F };
            let hnt = if gju > 0 { gju } else { 0x1F };

            
            let wji: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (jia & 0x7F);
            let uzw = db.atk(codec, lb, 0x300, wji);
            
            let wiz: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (hnt & 0x7F);
            let tsm = db.atk(codec, lb, 0x300, wiz);

            
            let muj = db.atk(codec, lb, verb::DI_, 0xA000).unwrap_or(0xDEAD);
            let mui = db.atk(codec, lb, verb::DI_, 0x2000).unwrap_or(0xDEAD);

            let qyd = myn != muj;
            let qyc = mym != mui;

            e.t(&format!("NID {:2} type={} oamp={}({}) iamp={}({})\n",
                lb, ash, drn, goq, fkg, gju));
            e.t(&format!("  OUT: {:#04X}->{:#04X} {} set={} gain={}\n",
                myn, muj,
                if qyd { "CHANGED" } else { "same" },
                if uzw.is_ok() { "ok" } else { "ERR" },
                jia));
            e.t(&format!("  IN:  {:#04X}->{:#04X} {} set={} gain={}\n",
                mym, mui,
                if qyc { "CHANGED" } else { "same" },
                if tsm.is_ok() { "ok" } else { "ERR" },
                hnt));
        }
    }

    e
}



pub fn ele(un: &[i16], uk: u32) -> Result<(), &'static str> {
    let mut hda = Fa.lock();
    let db = hda.as_mut().ok_or("HDA: not initialized")?;

    
    if db.uu {
        db.daq(false);
    }

    
    db.lzp();

    let k = db.dym as *mut i16;
    let fdv = (db.btf / 2) as usize; 

    let acq = un.len().v(fdv);

    unsafe {
        
        core::ptr::copy_nonoverlapping(un.fq(), k, acq);

        
        if acq < fdv {
            core::ptr::ahx(k.add(acq), 0, fdv - acq);
        }
    }

    
    db.daq(true);

    
    let xv = (acq * 2) as u32; 
    let cd = xv.v(db.btf);

    for _ in 0..(uk * 10 + 500) {
        HdaController::azo(100);
        let u = db.eje();
        if u >= cd {
            break;
        }
    }

    db.daq(false);
    Ok(())
}






static Afm: Mutex<u8> = Mutex::new(80);


pub fn chv(jy: u8) -> Result<(), &'static str> {
    let jy = jy.v(100);
    *Afm.lock() = jy;
    
    let mut hda = Fa.lock();
    let db = hda.as_mut().ok_or("HDA: not initialized")?;
    
    if db.bml.is_empty() || db.bhq.is_empty() {
        return Ok(());
    }
    
    let codec = db.bml[0];
    let path = db.bhq[0].clone();
    
    
    
    let vew = path.eak;
    let jfh = db.widgets.iter()
        .du(|d| d.lb == vew)
        .map(|d| ((d.eme >> 8) & 0x7F) as u16)
        .unwrap_or(39);
    let jfh = if jfh == 0 { 39 } else { jfh };
    let dqz = ((jy as u32) * (jfh as u32) / 100) as u16;
    
    
    for &lb in &path.path {
        
        let kam: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (dqz & 0x7F);
        let _ = db.atk(codec, lb, 0x300, kam);
        
        let kak: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (dqz & 0x7F);
        let _ = db.atk(codec, lb, 0x300, kak);
    }
    
    crate::serial_println!("[HDA] Volume set to {}% (gain={})", jy, dqz);
    Ok(())
}


pub fn nyu() -> u8 {
    *Afm.lock()
}


pub fn zde() -> Result<(), &'static str> {
    let mut hda = Fa.lock();
    let db = hda.as_mut().ok_or("HDA: not initialized")?;
    
    if db.bml.is_empty() || db.bhq.is_empty() {
        return Ok(());
    }
    
    let codec = db.bml[0];
    let path = db.bhq[0].clone();
    
    for &lb in &path.path {
        
        let uqu: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (1 << 7);
        let _ = db.atk(codec, lb, 0x300, uqu);
        
        let uqt: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (1 << 7);
        let _ = db.atk(codec, lb, 0x300, uqt);
    }
    
    Ok(())
}


pub fn zue() -> Result<(), &'static str> {
    let jy = *Afm.lock();
    chv(jy)
}







pub fn ghw(auf: u32, uk: u32, dyg: i16) -> Vec<i16> {
    let auy = 48000u32;
    let evo = (auy as u64 * uk as u64 / 1000) as usize;
    let mut un = Vec::fc(evo * 2);
    
    let api = *Afm.lock() as i32;
    let wdn = (dyg as i32 * api / 100) as i16;
    
    for a in 0..evo {
        
        let vhi = ((auf as u64 * a as u64 * 256) / auy as u64) as u32;
        let vhg = (vhi & 0xFF) as u8;
        
        let yr = wox(vhg, wdn);
        un.push(yr); 
        un.push(yr); 
    }
    
    
    let itt = (auy as usize * 5 / 1000).v(evo / 2);
    for a in 0..itt {
        let pv = a as i32 * 256 / itt as i32;
        un[a * 2] = (un[a * 2] as i32 * pv / 256) as i16;
        un[a * 2 + 1] = (un[a * 2 + 1] as i32 * pv / 256) as i16;
    }
    for a in 0..itt {
        let w = evo - 1 - a;
        let pv = a as i32 * 256 / itt as i32;
        if w * 2 + 1 < un.len() {
            un[w * 2] = (un[w * 2] as i32 * pv / 256) as i16;
            un[w * 2 + 1] = (un[w * 2 + 1] as i32 * pv / 256) as i16;
        }
    }
    
    un
}



fn wox(ib: u8, dyg: i16) -> i16 {
    let b = ib as i32;
    
    let tiv = if b < 128 {
        let ab = b - 64; 
        let js = -(ab * ab) + 64 * 64;
        js * 127 / (64 * 64)
    } else {
        let ab = (b - 128) - 64;
        let js = (ab * ab) - 64 * 64;
        js * 127 / (64 * 64)
    };
    
    (tiv as i32 * dyg as i32 / 127) as i16
}


pub fn vjc(auf: u32, uk: u32) -> Result<(), &'static str> {
    let un = ghw(auf, uk, 24000);
    ele(&un, uk)
}






#[derive(Debug, Clone)]
pub struct Bws {
    pub lq: u16,
    pub auy: u32,
    pub emv: u16,
    pub bbj: usize,
    pub cpv: usize,
}


pub fn jiu(f: &[u8]) -> Result<Bws, &'static str> {
    if f.len() < 44 { return Err("WAV: too short"); }
    if &f[0..4] != b"RIFF" { return Err("WAV: missing RIFF"); }
    if &f[8..12] != b"WAVE" { return Err("WAV: missing WAVE"); }
    
    let mut l = 12;
    let mut lq = 0u16;
    let mut auy = 0u32;
    let mut emv = 0u16;
    let mut bbj = 0usize;
    let mut cpv = 0usize;
    
    while l + 8 <= f.len() {
        let ncx = &f[l..l+4];
        let aiw = u32::dj([
            f[l+4], f[l+5], f[l+6], f[l+7]
        ]) as usize;
        
        if ncx == b"fmt " && aiw >= 16 {
            let qlb = u16::dj([f[l+8], f[l+9]]);
            if qlb != 1 { return Err("WAV: not PCM format"); }
            lq = u16::dj([f[l+10], f[l+11]]);
            auy = u32::dj([
                f[l+12], f[l+13], f[l+14], f[l+15]
            ]);
            emv = u16::dj([f[l+22], f[l+23]]);
        } else if ncx == b"data" {
            bbj = l + 8;
            cpv = aiw.v(f.len() - bbj);
            break;
        }
        
        l += 8 + aiw;
        if l % 2 != 0 { l += 1; } 
    }
    
    if bbj == 0 || lq == 0 {
        return Err("WAV: missing fmt or data chunk");
    }
    
    Ok(Bws { lq, auy, emv, bbj, cpv })
}


pub fn zfk(f: &[u8]) -> Result<(), &'static str> {
    let co = jiu(f)?;
    
    if co.emv != 16 {
        return Err("WAV: only 16-bit PCM supported");
    }
    
    let gpc = &f[co.bbj..co.bbj + co.cpv];
    let hth = co.cpv / (2 * co.lq as usize);
    
    let gue = 48000u32;
    let god = (hth as u64 * gue as u64
        / co.auy as u64) as usize;
    let mut an = Vec::fc(god * 2);
    
    let api = *Afm.lock() as i32;
    
    for krt in 0..god {
        let ibk = (krt as u64 * co.auy as u64
            / gue as u64) as usize;
        
        if ibk >= hth { break; }
        
        let w = ibk * co.lq as usize;
        let avk = w * 2;
        
        let fd = if avk + 1 < gpc.len() {
            i16::dj([gpc[avk], gpc[avk + 1]])
        } else { 0 };
        
        let hw = if co.lq >= 2 {
            let kfr = (w + 1) * 2;
            if kfr + 1 < gpc.len() {
                i16::dj([gpc[kfr], gpc[kfr + 1]])
            } else { fd }
        } else { fd };
        
        an.push((fd as i32 * api / 100) as i16);
        an.push((hw as i32 * api / 100) as i16);
    }
    
    let uk = (god as u64 * 1000 / gue as u64) as u32;
    ele(&an, uk + 100)
}






#[derive(Clone, Copy, Debug)]
pub enum SoundEffect {
    
    Byo,
    
    Vy,
    
    Q,
    
    Cic,
    
    Oo,
    
    Hf,
    
    Cgj,
}


pub fn viv(bzk: SoundEffect) -> Result<(), &'static str> {
    let xjk: Vec<(u32, u32, i16)> = match bzk {
        SoundEffect::Byo => vec![
            (523, 150, 20000),  
            (659, 150, 20000),  
            (784, 250, 22000),  
        ],
        SoundEffect::Vy => vec![(1000, 15, 16000)],
        SoundEffect::Q => vec![
            (400, 120, 22000),
            (0, 60, 0),    
            (400, 120, 22000),
        ],
        SoundEffect::Cic => vec![
            (880, 100, 18000),   
            (1109, 100, 18000),  
            (1319, 200, 20000),  
        ],
        SoundEffect::Oo => vec![
            (880, 200, 20000),
            (660, 300, 18000),
        ],
        SoundEffect::Hf => vec![
            (523, 100, 18000),  
            (659, 200, 20000),  
        ],
        SoundEffect::Cgj => vec![(2000, 8, 8000)],
    };
    
    let mut fcp: Vec<i16> = Vec::new();
    let mut alu = 0u32;
    
    for (kx, aie, byf) in &xjk {
        if *kx == 0 {
            let woh = (48000u32 * *aie / 1000) as usize;
            fcp.lg(core::iter::afd(0i16).take(woh * 2));
        } else {
            let mlt = ghw(*kx, *aie, *byf);
            fcp.bk(&mlt);
        }
        alu += aie;
    }
    
    ele(&fcp, alu + 50)
}






#[derive(Clone, Copy, Debug)]
pub struct Note {
    
    pub ti: u8,
    
    pub ksc: u8,
    
    pub qm: u8,
}

impl Note {
    pub fn new(ayg: u8, dqi: u8, bxr: u8) -> Self {
        Self { ti: ayg, ksc: dqi, qm: bxr }
    }
    
    pub fn kr(dqi: u8) -> Self {
        Self { ti: 0, ksc: dqi, qm: 0 }
    }
    
    
    
    pub fn auf(&self) -> u32 {
        if self.ti == 0 { return 0; }
        let joi = self.ti as i32 - 69;
        let lpt = joi.ymh(12);
        let grz = joi.zje(12) as usize;
        
        
        const CSC_: [u32; 12] = [
            1000, 1059, 1122, 1189, 1260, 1335, 1414, 1498, 1587, 1682, 1782, 1888
        ];
        
        let mya = CSC_[grz] * 440 / 1000;
        
        if lpt >= 0 {
            mya << lpt as u32
        } else {
            mya >> (-lpt) as u32
        }
    }
}


pub fn vjb(ts: &[Note], kz: u32) -> Result<(), &'static str> {
    if ts.is_empty() { return Ok(()); }
    
    let woz = 60_000 / (kz * 4);
    
    let mut fcp: Vec<i16> = Vec::new();
    let mut alu = 0u32;
    
    for jp in ts {
        let aie = woz * jp.ksc as u32;
        let kx = jp.auf();
        
        if kx == 0 || jp.qm == 0 {
            let jqj = (48000u32 * aie / 1000) as usize;
            fcp.lg(core::iter::afd(0i16).take(jqj * 2));
        } else {
            let byf = (jp.qm as i32 * 24000 / 127) as i16;
            let mlt = ghw(kx, aie, byf);
            fcp.bk(&mlt);
        }
        alu += aie;
    }
    
    ele(&fcp, alu + 50)
}










pub fn viy(czq: &str, kz: u32) -> Result<(), &'static str> {
    let mut ts = Vec::new();
    
    for bat in czq.ayt() {
        if bat.is_empty() { continue; }
        
        let bf = bat.as_bytes();
        if bf[0] == b'R' || bf[0] == b'r' {
            ts.push(Note::kr(ouf(&bf[1..])));
            continue;
        }
        
        let (oqx, kr) = vcx(bf);
        if oqx == 255 { continue; }
        
        let (cgg, vyc) = if !kr.is_empty() && kr[0] >= b'0' && kr[0] <= b'9' {
            (kr[0] - b'0', &kr[1..])
        } else {
            (4, kr)
        };
        
        let dqi = ouf(vyc);
        let ayg = 12 * (cgg + 1) + oqx;
        ts.push(Note::new(ayg, dqi, 100));
    }
    
    vjb(&ts, kz)
}


fn vcx(bf: &[u8]) -> (u8, &[u8]) {
    if bf.is_empty() { return (255, bf); }
    
    let ar = match bf[0] {
        b'C' | b'c' => 0,
        b'D' | b'd' => 2,
        b'E' | b'e' => 4,
        b'F' | b'f' => 5,
        b'G' | b'g' => 7,
        b'A' | b'a' => 9,
        b'B' | b'b' => 11,
        _ => return (255, bf),
    };
    
    if bf.len() > 1 && bf[1] == b'#' {
        return ((ar + 1) % 12, &bf[2..]);
    }
    
    (ar, &bf[1..])
}


fn ouf(bf: &[u8]) -> u8 {
    if bf.is_empty() { return 4; }
    match bf[0] {
        b'w' => 16,
        b'h' => 8,
        b'q' => 4,
        b'e' => 2,
        b's' => 1,
        _ => 4,
    }
}


pub fn viu() -> Result<(), &'static str> {
    viy("E4q E4q F4q G4q G4q F4q E4q D4q C4q C4q D4q E4q E4q D4h", 120)
}
