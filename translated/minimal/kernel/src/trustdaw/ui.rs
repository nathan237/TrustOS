









use alloc::format;
use alloc::string::String;
use core::sync::atomic::Ordering;

use super::piano_roll::PianoRoll;
use super::{Qi, Mf, FR_, Hi, AE_, FM_};






mod colors {
    pub const Apk: u32 = 0x0D0D1A;
    pub const GO_: u32 = 0x1A1A2E;
    pub const XT_: u32 = 0x151525;
    pub const BHO_: u32 = 0x121222;
    pub const CYK_: u32 = 0x1A1A3A;
    pub const AJC_: u32 = 0x2A2A4A;
    pub const AC_: u32 = 0xDDDDEE;
    pub const N_: u32 = 0x8888AA;
    pub const AV_: u32 = 0x555577;
    pub const OY_: u32 = 0x44DD44;
    pub const CUM_: u32 = 0xDD4444;
    pub const WN_: u32 = 0xFF2222;
    pub const MA_: u32 = 0x2A2A4A;
    pub const DEC_: u32 = 0x3A3A6A;
    pub const CHH_: u32 = 0xFF8800;
    pub const CTQ_: u32 = 0xFFDD00;
    pub const AFJ_: u32 = 0x44CC44;
    pub const AFL_: u32 = 0xCCCC44;
    pub const AFK_: u32 = 0xCC4444;
    pub const VD_: u32 = 0x1A1A2A;
}


const QG_: u32 = 40;
const QF_: u32 = 180;
const JF_: u32 = 28;
const EIO_: u32 = 200;






pub struct DawUI {
    
    pub eih: usize,
    
    pub piano_roll: PianoRoll,
    
    pub jqc: bool,
    
    pub no: bool,
}

impl DawUI {
    
    pub fn new() -> Self {
        let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
        let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;

        let ovl = QG_ + QF_;
        let vhp = kc.ao(ovl);

        Self {
            eih: 0,
            piano_roll: PianoRoll::new(0, ovl, gz, vhp),
            jqc: true,
            no: true,
        }
    }

    
    pub fn po(&mut self) {
        let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
        let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;
        if gz == 0 || kc == 0 { return; }

        
        crate::framebuffer::ah(0, 0, gz, kc, colors::Apk);

        
        let nv = super::Fc.lock();
        let mixer = super::Lu.lock();

        if let (Some(aci), Some(bno)) = (nv.as_ref(), mixer.as_ref()) {
            
            self.krn(gz, aci);

            
            self.sgj(gz, aci, bno);

            
            if self.jqc {
                if let Some(track) = aci.af.get(self.eih) {
                    let vjd = FR_.load(Ordering::Relaxed);
                    self.piano_roll.po(track, vjd);
                } else {
                    
                    let x = QG_ + QF_ + 20;
                    crate::framebuffer::cb("No tracks. Use 'daw track add <name>' to create one.",
                        20, x, colors::N_);
                }
            }
        } else {
            crate::framebuffer::cb("TrustDAW not initialized. Run 'daw init'",
                20, 20, colors::AC_);
        }

        self.no = false;
    }

    
    fn krn(&self, gz: u32, aci: &super::track::Project) {
        crate::framebuffer::ah(0, 0, gz, QG_, colors::XT_);

        
        crate::framebuffer::cb("TrustDAW", 8, 4, colors::AC_);

        
        let j = aci.amj();
        crate::framebuffer::cb(j, 100, 4, colors::N_);

        
        let uu = Qi.load(Ordering::Relaxed);
        let ehe = Mf.load(Ordering::Relaxed);

        
        let gtf = 250;
        if ehe {
            crate::framebuffer::abc(gtf + 8, 12, 6, colors::WN_);
            crate::framebuffer::cb("REC", gtf + 20, 4, colors::WN_);
        } else if uu {
            
            crate::framebuffer::ah(gtf, 6, 3, 12, colors::OY_);
            crate::framebuffer::cb("PLAY", gtf + 20, 4, colors::OY_);
        } else {
            crate::framebuffer::ah(gtf, 6, 12, 12, colors::CUM_);
            crate::framebuffer::cb("STOP", gtf + 20, 4, colors::AV_);
        }

        
        let kz = Hi.load(Ordering::Relaxed);
        let gbk = format!("BPM: {}", kz);
        crate::framebuffer::cb(&gbk, 350, 4, colors::AC_);

        
        let u = FR_.load(Ordering::Relaxed);
        let bar = u / (AE_ * 4);
        let rf = (u % (AE_ * 4)) / AE_;
        let or = u % AE_;
        let dar = format!("{}:{:02}:{:03}", bar + 1, rf + 1, or);
        crate::framebuffer::cb(&dar, 450, 4, colors::AC_);

        
        let xln = format!("Tracks: {}/{}", aci.af.len(), FM_);
        crate::framebuffer::cb(&xln, 560, 4, colors::N_);

        
        crate::framebuffer::zs(0, QG_ - 1, gz, colors::AJC_);

        
        crate::framebuffer::cb(
            "[Space] Play/Stop  [R] Record  [+/-] BPM  [Tab] Track  [F5] Piano Roll  [F8] Export WAV",
            8, 22, colors::AV_
        );
    }

    
    fn sgj(&self, gz: u32, aci: &super::track::Project, mixer: &super::mixer::Mixer) {
        let ou = QG_;
        crate::framebuffer::ah(0, ou, gz, QF_, colors::BHO_);

        
        crate::framebuffer::ah(0, ou, gz, JF_, colors::GO_);
        crate::framebuffer::cb(" # ", 4, ou + 6, colors::AV_);
        crate::framebuffer::cb("Track", 30, ou + 6, colors::AV_);
        crate::framebuffer::cb("Wave", 140, ou + 6, colors::AV_);
        crate::framebuffer::cb("Notes", 200, ou + 6, colors::AV_);
        crate::framebuffer::cb("Vol", 260, ou + 6, colors::AV_);
        crate::framebuffer::cb("Pan", 310, ou + 6, colors::AV_);
        crate::framebuffer::cb("M S", 360, ou + 6, colors::AV_);
        crate::framebuffer::cb("Meter", 400, ou + 6, colors::AV_);

        
        for (a, track) in aci.af.iter().cf() {
            let afy = ou + JF_ + (a as u32 * JF_);
            if afy + JF_ > ou + QF_ {
                break; 
            }

            
            let ei = if a == self.eih {
                colors::CYK_
            } else {
                colors::BHO_
            };
            crate::framebuffer::ah(0, afy, gz, JF_, ei);

            
            crate::framebuffer::ah(0, afy, 4, JF_, track.s);

            
            let ajh = format!("{}", a);
            crate::framebuffer::cb(&ajh, 8, afy + 6, colors::AC_);

            
            crate::framebuffer::cb(track.amj(), 30, afy + 6, colors::AC_);

            
            crate::framebuffer::cb(track.ve.dbz(), 140, afy + 6, colors::N_);

            
            let uvr = format!("{}", track.ts.len());
            crate::framebuffer::cb(&uvr, 200, afy + 6, colors::N_);

            
            if let Some(bm) = mixer.lq.get(a) {
                let igu = format!("{}", bm.hq);
                crate::framebuffer::cb(&igu, 260, afy + 6, colors::AC_);

                
                let lry = if bm.arp == 0 { String::from("C") }
                    else if bm.arp > 0 { format!("R{}", bm.arp) }
                    else { format!("L{}", -bm.arp) };
                crate::framebuffer::cb(&lry, 310, afy + 6, colors::N_);

                
                if bm.so {
                    crate::framebuffer::cb("M", 360, afy + 6, colors::CHH_);
                }

                
                if bm.cic {
                    crate::framebuffer::cb("S", 376, afy + 6, colors::CTQ_);
                }

                
                let llq: u32 = 400;
                let onc: u32 = (gz - llq).ao(20).v(200);
                let adu = (onc as u32 * bm.hq as u32) / 255;
                crate::framebuffer::ah(llq, afy + 8, onc, 12, colors::VD_);
                if adu > 0 {
                    let unw = if bm.hq > 230 { colors::AFK_ }
                        else if bm.hq > 180 { colors::AFL_ }
                        else { colors::AFJ_ };
                    crate::framebuffer::ah(llq, afy + 8, adu, 12, unw);
                }
            }

            
            crate::framebuffer::zs(0, afy + JF_ - 1, gz, colors::AJC_);
        }

        
        crate::framebuffer::zs(0, ou + QF_ - 1, gz, colors::AJC_);
    }

    

    
    pub fn loq(&mut self) {
        let nv = super::Fc.lock();
        if let Some(aci) = nv.as_ref() {
            if !aci.af.is_empty() {
                self.eih = (self.eih + 1) % aci.af.len();
                self.no = true;
            }
        }
    }

    
    pub fn oxm(&mut self) {
        let nv = super::Fc.lock();
        if let Some(aci) = nv.as_ref() {
            if !aci.af.is_empty() {
                if self.eih == 0 {
                    self.eih = aci.af.len() - 1;
                } else {
                    self.eih -= 1;
                }
                self.no = true;
            }
        }
    }

    
    pub fn xix(&mut self) {
        self.jqc = !self.jqc;
        self.no = true;
    }
}


pub fn ucx() -> Result<(), &'static str> {
    super::aqz()?;

    let mut ui = DawUI::new();
    ui.po();

    crate::println!("TrustDAW GUI launched. Press [Esc] to return to shell.");

    
    loop {
        if let Some(scancode) = crate::keyboard::xw() {
            let bep = scancode & 0x80 != 0;
            if bep { continue; } 

            match scancode {
                0x01 => break, 
                0x39 => { 
                    if Qi.load(Ordering::Relaxed) {
                        super::qg();
                    } else {
                        let _ = super::daq();
                    }
                    ui.no = true;
                }
                0x13 => { 
                    if Mf.load(Ordering::Relaxed) {
                        Mf.store(false, Ordering::Relaxed);
                    } else {
                        let _ = super::recorder::pas(ui.eih);
                    }
                    ui.no = true;
                }
                0x0F => { 
                    ui.loq();
                }
                0x3F => { 
                    ui.xix();
                }
                0x42 => { 
                    let _ = super::hio("/home/output.wav");
                    crate::println!("Exported to /home/output.wav");
                }
                0x0C => { 
                    let kz = Hi.load(Ordering::Relaxed);
                    super::mef(kz.ao(5));
                    ui.no = true;
                }
                0x0D => { 
                    let kz = Hi.load(Ordering::Relaxed);
                    super::mef(kz + 5);
                    ui.no = true;
                }
                
                0x48 => { ui.piano_roll.rsk(); ui.no = true; }    
                0x50 => { ui.piano_roll.rsd(); ui.no = true; }  
                0x4B => { ui.piano_roll.rse(); ui.no = true; }  
                0x4D => { ui.piano_roll.rsg(); ui.no = true; } 
                _ => {}
            }

            if ui.no {
                ui.po();
            }
        }

        
        for _ in 0..5000 {
            core::hint::hc();
        }
    }

    Ok(())
}
