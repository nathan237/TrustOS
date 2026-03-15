








use core::sync::atomic::{AtomicBool, Ordering};
use crate::arch::Port;





const ARB_: u16 = 0x62;
const SW_: u16 = 0x66;  


const BTD_: u8 = 0x01;  
const BTC_: u8 = 0x02;  


const BSY_: u8 = 0x80;
const BSZ_: u8 = 0x81;






const ARC_: u8 = 0x2F;


const BTA_: u8 = 0x84;
const BTB_: u8 = 0x85;


const ARD_: u8 = 0x78;
const ABT_: usize = 8;  


const CXV_: [&str; 8] = [
    "CPU",          
    "miniPCI",      
    "HDD",          
    "GPU",          
    "Battery",      
    "Sensor 5",     
    "Sensor 6",     
    "Sensor 7",     
];





const CHD_: u32 = 0x198;
const BBA_: u32 = 0x199;
const CHC_: u32 = 0x1A0;
const CHE_: u32 = 0x19C;





static ARA_: AtomicBool = AtomicBool::new(false);






fn hhr() -> bool {
    let mut dma: Port<u8> = Port::new(SW_);
    let mut kos: Port<u8> = Port::new(0x80);
    for _ in 0..100_000u32 {
        let status = unsafe { dma.read() };
        if status & BTC_ == 0 {
            return true;
        }
        unsafe { kos.read(); }
    }
    false
}


fn sih() -> bool {
    let mut dma: Port<u8> = Port::new(SW_);
    let mut kos: Port<u8> = Port::new(0x80);
    for _ in 0..100_000u32 {
        let status = unsafe { dma.read() };
        if status & BTD_ != 0 {
            return true;
        }
        unsafe { kos.read(); }
    }
    false
}


pub fn hhq(reg: u8) -> Option<u8> {
    let mut ffa: Port<u8> = Port::new(SW_);
    let mut axr: Port<u8> = Port::new(ARB_);
    if !hhr() { return None; }
    unsafe { ffa.write(BSY_); }
    if !hhr() { return None; }
    unsafe { axr.write(reg); }
    if !sih() { return None; }
    Some(unsafe { axr.read() })
}


pub fn sii(reg: u8, ap: u8) -> bool {
    let mut ffa: Port<u8> = Port::new(SW_);
    let mut axr: Port<u8> = Port::new(ARB_);
    if !hhr() { return false; }
    unsafe { ffa.write(BSZ_); }
    if !hhr() { return false; }
    unsafe { axr.write(reg); }
    if !hhr() { return false; }
    unsafe { axr.write(ap); }
    true
}






pub fn probe() -> bool {
    if let Some(bcz) = hhq(ARD_) {
        
        if bcz >= 10 && bcz <= 120 {
            ARA_.store(true, Ordering::Relaxed);
            crate::serial_println!("[EC] ThinkPad EC detected — CPU temp: {}°C", bcz);
            return true;
        }
    }
    crate::serial_println!("[EC] ThinkPad EC not detected or not responding");
    false
}

pub fn anl() -> bool {
    ARA_.load(Ordering::Relaxed)
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FanLevel {
    Pv(u8),   
    Api,        
    Bhl,   
}


pub fn sqz() -> Option<u8> {
    hhq(ARC_)
}


pub fn ity(jy: FanLevel) -> bool {
    let ap = match jy {
        FanLevel::Pv(dm) => {
            if dm > 7 { return false; }
            dm
        }
        FanLevel::Api => 0x80,      
        FanLevel::Bhl => 0x40, 
    };
    sii(ARC_, ap)
}


pub fn nsu() -> Option<u16> {
    let gd = hhq(BTA_)?;
    let hh = hhq(BTB_)?;
    Some(((gd as u16) << 8) | hh as u16)
}






pub fn xbo(hzq: usize) -> Option<u8> {
    if hzq >= ABT_ { return None; }
    let ap = hhq(ARD_ + hzq as u8)?;
    
    if ap == 0 || ap >= 128 { return None; }
    Some(ap)
}


pub fn xbn(hzq: usize) -> &'static str {
    if hzq < ABT_ {
        CXV_[hzq]
    } else {
        "Unknown"
    }
}






#[cfg(target_arch = "x86_64")]
pub fn kll() -> Option<(u32, u32)> {
    let ap = crate::debug::fsg(CHD_)?;
    
    
    let aos = ((ap >> 8) & 0xFF) as u32;
    let cck = (ap & 0xFF) as u32;
    
    
    
    let kxe = aos * 200;
    
    
    
    let xsu = if cck > 0 {
        712 + cck * 12  
    } else {
        0
    };
    
    Some((kxe, xsu))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn kll() -> Option<(u32, u32)> {
    None
}


#[cfg(target_arch = "x86_64")]
pub fn ngq() -> Option<u16> {
    let ap = crate::debug::fsg(BBA_)?;
    Some((ap & 0xFFFF) as u16)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn ngq() -> Option<u16> {
    None
}




#[cfg(target_arch = "x86_64")]
pub fn ipk(aos: u8, cck: u8) -> bool {
    let ap = ((aos as u64) << 8) | (cck as u64);
    crate::debug::fbs(BBA_, ap);
    true
}

#[cfg(not(target_arch = "x86_64"))]
pub fn ipk(xzg: u8, yds: u8) -> bool {
    false
}


#[cfg(target_arch = "x86_64")]
pub fn npk() -> Option<bool> {
    let ap = crate::debug::fsg(CHC_)?;
    Some((ap & (1 << 16)) != 0)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn npk() -> Option<bool> {
    None
}


#[cfg(target_arch = "x86_64")]
pub fn klq() -> Option<(bool, u8)> {
    let ap = crate::debug::fsg(CHE_)?;
    let blq = (ap & (1 << 31)) != 0;  
    let vsv = ((ap >> 16) & 0x7F) as u8;  
    Some((blq, vsv))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn klq() -> Option<(bool, u8)> {
    None
}







pub const AIP_: &[(&str, u8, u8)] = &[
    ("2.0 GHz (max)",  10, 38),  
    ("1.6 GHz",         8, 30),
    ("1.2 GHz",         6, 22),
    ("800 MHz (min)",   4, 16),
];






pub fn reg(n: &[&str]) {
    use crate::framebuffer::*;

    if !anl() {
        if !probe() {
            crate::h!(A_, "EC not available — not a ThinkPad or EC unresponsive");
            return;
        }
    }

    match n.fv().hu() {
        None | Some("status") => {
            
            crate::h!(C_, "=== ThinkPad Fan Status ===");
            
            if let Some(jy) = sqz() {
                let desc = match jy {
                    0x80 => "auto (EC controlled)",
                    0x40 => "DISENGAGED (full speed)",
                    dm if dm <= 7 => match dm {
                        0 => "0 (off)",
                        1 => "1 (lowest)",
                        2 => "2",
                        3 => "3",
                        4 => "4",
                        5 => "5",
                        6 => "6",
                        7 => "7 (highest manual)",
                        _ => "?",
                    },
                    _ => "unknown",
                };
                crate::println!("  Level: 0x{:02X} — {}", jy, desc);
            } else {
                crate::h!(A_, "  Level: read failed");
            }

            if let Some(ftf) = nsu() {
                if ftf == 0 || ftf == 0xFFFF {
                    crate::println!("  RPM:   stopped or N/A");
                } else {
                    crate::println!("  RPM:   {}", ftf);
                }
            } else {
                crate::println!("  RPM:   read failed");
            }
        }

        Some("auto") => {
            if ity(FanLevel::Api) {
                crate::h!(B_, "Fan set to AUTO (EC controlled)");
            } else {
                crate::h!(A_, "Failed to set fan to auto");
            }
        }

        Some("max") | Some("full") => {
            if ity(FanLevel::Bhl) {
                crate::h!(D_, "Fan set to FULL SPEED (disengaged)");
            } else {
                crate::h!(A_, "Failed to set fan to full speed");
            }
        }

        Some("off") | Some("0") => {
            crate::h!(D_, "WARNING: Turning fan off! Monitor temperatures carefully.");
            if ity(FanLevel::Pv(0)) {
                crate::h!(A_, "Fan OFF");
            } else {
                crate::h!(A_, "Failed to turn fan off");
            }
        }

        Some(bo) => {
            if let Ok(jy) = bo.parse::<u8>() {
                if jy <= 7 {
                    if ity(FanLevel::Pv(jy)) {
                        crate::h!(B_, "Fan set to level {}", jy);
                    } else {
                        crate::h!(A_, "Failed to set fan level");
                    }
                } else {
                    crate::h!(A_, "Fan level must be 0-7, 'auto', 'max', or 'off'");
                }
            } else {
                crate::println!("Usage: fan [status|auto|max|off|0-7]");
                crate::println!("  fan          Show current fan status");
                crate::println!("  fan auto     Let EC control the fan");
                crate::println!("  fan max      Full speed (disengaged)");
                crate::println!("  fan off      Turn fan off (DANGEROUS)");
                crate::println!("  fan 0-7      Set manual speed level");
            }
        }
    }
}


pub fn rjc(elm: &[&str]) {
    use crate::framebuffer::*;

    if !anl() {
        if !probe() {
            crate::h!(A_, "EC not available — not a ThinkPad or EC unresponsive");
            return;
        }
    }

    crate::h!(C_, "=== ThinkPad Temperature Sensors ===");
    
    let mut mvw = false;
    for a in 0..ABT_ {
        if let Some(bcz) = xbo(a) {
            mvw = true;
            let s = if bcz >= 90 {
                A_
            } else if bcz >= 70 {
                D_
            } else {
                B_
            };
            crate::print!("  {:10} ", xbn(a));
            crate::h!(s, "{}°C", bcz);
        }
    }

    if !mvw {
        crate::println!("  No temperature sensors responded");
    }

    
    #[cfg(target_arch = "x86_64")]
    if let Some((blq, hhf)) = klq() {
        if blq {
            
            let pto: u8 = 100;
            let klp = pto.ao(hhf);
            let s = if klp >= 90 {
                A_
            } else if klp >= 70 {
                D_
            } else {
                B_
            };
            crate::print!("  {:10} ", "CPU (DTS)");
            crate::h!(s, "{}°C (TjMax={}, margin={}°C)", klp, pto, hhf);
        }
    }

    
    crate::println!();
    if let Some(ftf) = nsu() {
        if ftf > 0 && ftf != 0xFFFF {
            crate::println!("  Fan:       {} RPM", ftf);
        } else {
            crate::println!("  Fan:       stopped");
        }
    }
}


pub fn rdd(n: &[&str]) {
    use crate::framebuffer::*;

    match n.fv().hu() {
        None | Some("status") => {
            crate::h!(C_, "=== CPU Frequency / Voltage ===");

            
            match npk() {
                Some(true) => crate::h!(B_, "  SpeedStep (EIST): enabled"),
                Some(false) => crate::h!(D_, "  SpeedStep (EIST): disabled"),
                None => crate::println!("  SpeedStep (EIST): unknown"),
            }

            
            if let Some((kx, gwi)) = kll() {
                crate::println!("  Current freq:     {} MHz", kx);
                if gwi > 0 {
                    crate::println!("  Current voltage:  {}.{:03} V", gwi / 1000, gwi % 1000);
                }
            } else {
                crate::println!("  Current P-state:  read failed");
            }

            
            if let Some(cd) = ngq() {
                let psv = (cd >> 8) & 0xFF;
                let xni = cd & 0xFF;
                crate::println!("  Target:           FID={} VID={} ({}MHz)", psv, xni, psv as u32 * 200);
            }

            
            #[cfg(target_arch = "x86_64")]
            if let Some((blq, hhf)) = klq() {
                if blq {
                    crate::println!("  CPU temp (DTS):   {}°C (margin: {}°C to TjMax)", 100u8.ao(hhf), hhf);
                }
            }

            
            crate::println!();
            crate::h!(C_, "  Known T61 P-states (Core 2 Duo, FSB 800MHz):");
            for (cu, aos, cck) in AIP_ {
                crate::println!("    FID={:2} VID={:2}  → {}", aos, cck, cu);
            }
        }

        Some("set") => {
            if n.len() < 3 {
                crate::println!("Usage: cpufreq set <fid> <vid>");
                crate::println!("  Use 'cpufreq status' to see known P-states");
                return;
            }
            let aos = match n[1].parse::<u8>() {
                Ok(bb) => bb,
                Err(_) => {
                    crate::h!(A_, "Invalid FID: {}", n[1]);
                    return;
                }
            };
            let cck = match n[2].parse::<u8>() {
                Ok(p) => p,
                Err(_) => {
                    crate::h!(A_, "Invalid VID: {}", n[2]);
                    return;
                }
            };
            crate::h!(D_, "Setting P-state: FID={} VID={} ({}MHz)", aos, cck, aos as u32 * 200);
            if ipk(aos, cck) {
                crate::h!(B_, "P-state change requested");
                
                if let Some((kx, gwi)) = kll() {
                    crate::println!("  Now running at: {} MHz, {}.{:03} V", kx, gwi / 1000, gwi % 1000);
                }
            } else {
                crate::h!(A_, "Failed to set P-state");
            }
        }

        Some("max") => {
            if let Some(&(cu, aos, cck)) = AIP_.fv() {
                crate::h!(D_, "Setting CPU to {}", cu);
                ipk(aos, cck);
                crate::h!(B_, "Done");
            }
        }

        Some("min") | Some("powersave") => {
            if let Some(&(cu, aos, cck)) = AIP_.qv() {
                crate::h!(D_, "Setting CPU to {}", cu);
                ipk(aos, cck);
                crate::h!(B_, "Done");
            }
        }

        _ => {
            crate::println!("Usage: cpufreq [status|set|max|min]");
            crate::println!("  cpufreq            Show current frequency/voltage");
            crate::println!("  cpufreq set <f> <v> Set P-state (FID, VID)");
            crate::println!("  cpufreq max        Set maximum performance");
            crate::println!("  cpufreq min        Set minimum (powersave)");
        }
    }
}
