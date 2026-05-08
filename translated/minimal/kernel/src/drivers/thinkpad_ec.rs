











use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use crate::arch::Port;





const ATE_: u16 = 0x62;
const UC_: u16 = 0x66;  


const BVZ_: u8 = 0x01;  
const BVY_: u8 = 0x02;  


const BVU_: u8 = 0x80;
const BVV_: u8 = 0x81;






const ATF_: u8 = 0x2F;


const BVW_: u8 = 0x84;
const BVX_: u8 = 0x85;


const ATG_: u8 = 0x78;
const ADJ_: usize = 8;  


const DBN_: [&str; 8] = [
    "CPU",          
    "miniPCI",      
    "HDD",          
    "GPU",          
    "Battery",      
    "Sensor 5",     
    "Sensor 6",     
    "Sensor 7",     
];





const BDD_: u32 = 0x198;
const BDC_: u32 = 0x199;
const CKM_: u32 = 0x1A0;
const CKN_: u32 = 0x19C;





static ATD_: AtomicBool = AtomicBool::new(false);






fn doi() -> bool {
    let mut bjk: Port<u8> = Port::new(UC_);
    let mut frf: Port<u8> = Port::new(0x80);
    for _ in 0..100_000u32 {
        let status = unsafe { bjk.read() };
        if status & BVY_ == 0 {
            return true;
        }
        unsafe { frf.read(); }
    }
    false
}


fn lnj() -> bool {
    let mut bjk: Port<u8> = Port::new(UC_);
    let mut frf: Port<u8> = Port::new(0x80);
    for _ in 0..100_000u32 {
        let status = unsafe { bjk.read() };
        if status & BVZ_ != 0 {
            return true;
        }
        unsafe { frf.read(); }
    }
    false
}


pub fn ciq(reg: u8) -> Option<u8> {
    let mut chg: Port<u8> = Port::new(UC_);
    let mut zu: Port<u8> = Port::new(ATE_);
    if !doi() { return None; }
    unsafe { chg.write(BVU_); }
    if !doi() { return None; }
    unsafe { zu.write(reg); }
    if !lnj() { return None; }
    Some(unsafe { zu.read() })
}


pub fn lnk(reg: u8, val: u8) -> bool {
    let mut chg: Port<u8> = Port::new(UC_);
    let mut zu: Port<u8> = Port::new(ATE_);
    if !doi() { return false; }
    unsafe { chg.write(BVV_); }
    if !doi() { return false; }
    unsafe { zu.write(reg); }
    if !doi() { return false; }
    unsafe { zu.write(val); }
    true
}






pub fn probe() -> bool {
    if let Some(ts) = ciq(ATG_) {
        
        if ts >= 10 && ts <= 120 {
            ATD_.store(true, Ordering::Relaxed);
            crate::serial_println!("[EC] ThinkPad EC detected — CPU temp: {}°C", ts);
            return true;
        }
    }
    crate::serial_println!("[EC] ThinkPad EC not detected or not responding");
    false
}

pub fn sw() -> bool {
    ATD_.load(Ordering::Relaxed)
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FanLevel {
    Gp(u8),   
    Auto,        
    FullSpeed,   
}


pub fn luh() -> Option<u8> {
    ciq(ATF_)
}


pub fn eme(level: FanLevel) -> bool {
    let val = match level {
        FanLevel::Gp(l) => {
            if l > 7 { return false; }
            l
        }
        FanLevel::Auto => 0x80,      
        FanLevel::FullSpeed => 0x40, 
    };
    lnk(ATF_, val)
}


pub fn fwf() -> Option<u16> {
    let hi = ciq(BVW_)?;
    let lo = ciq(BVX_)?;
    Some(((hi as u16) << 8) | lo as u16)
}






pub fn pdu(sensor: usize) -> Option<u8> {
    if sensor >= ADJ_ { return None; }
    let val = ciq(ATG_ + sensor as u8)?;
    
    if val == 0 || val >= 128 { return None; }
    Some(val)
}


pub fn pdr(sensor: usize) -> &'static str {
    if sensor < ADJ_ {
        DBN_[sensor]
    } else {
        "Unknown"
    }
}






static ASC_: AtomicU32 = AtomicU32::new(0);





#[cfg(target_arch = "x86_64")]
pub fn dmy() -> u32 {
    let bfd = ASC_.load(Ordering::Relaxed);
    if bfd != 0 {
        return bfd;
    }
    let bsx = if let Some(val) = crate::debug::rf(0xCD) {
        
        match val & 0x07 {
            0b101 => 100, 
            0b001 => 133, 
            0b011 => 167, 
            0b010 => 200, 
            0b000 => 267, 
            0b100 => 333, 
            _     => 200, 
        }
    } else {
        
        let kza = unsafe {
            let result: u32;
            core::arch::asm!(
                "push rbx",
                "cpuid",
                "pop rbx",
                in("eax") 0u32,
                lateout("eax") result,
                out("ecx") _, out("edx") _,
            );
            result
        };
        if kza >= 0x16 {
            let ehh = unsafe {
                let result: u32;
                core::arch::asm!(
                    "push rbx",
                    "cpuid",
                    "pop rbx",
                    in("eax") 0x16u32,
                    lateout("ecx") result,
                    out("edx") _,
                );
                result & 0xFFFF
            };
            if ehh > 0 { ehh } else { 100 }
        } else {
            100 
        }
    };
    ASC_.store(bsx, Ordering::Relaxed);
    bsx
}

#[cfg(not(target_arch = "x86_64"))]
pub fn dmy() -> u32 { 100 }



fn fwn(fid: u32) -> u32 {
    fid * dmy()
}


#[cfg(target_arch = "x86_64")]
pub fn fou() -> Option<(u32, u32)> {
    let val = crate::debug::rf(BDD_)?;
    
    
    let fid = ((val >> 8) & 0xFF) as u32;
    let bpx = (val & 0xFF) as u32;
    
    let dqg = fwn(fid);
    
    
    
    let ptb = if bpx > 0 {
        712 + bpx * 12  
    } else {
        0
    };
    
    Some((dqg, ptb))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn fou() -> Option<(u32, u32)> {
    None
}


#[cfg(target_arch = "x86_64")]
pub fn hoi() -> Option<u16> {
    let val = crate::debug::rf(BDC_)?;
    Some((val & 0xFFFF) as u16)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn hoi() -> Option<u16> {
    None
}




#[cfg(target_arch = "x86_64")]
pub fn eja(fid: u8, bpx: u8) -> bool {
    let val = ((fid as u64) << 8) | (bpx as u64);
    crate::debug::cfm(BDC_, val);
    true
}

#[cfg(not(target_arch = "x86_64"))]
pub fn eja(_fid: u8, _vid: u8) -> bool {
    false
}


#[cfg(target_arch = "x86_64")]
pub fn hvg() -> Option<bool> {
    let val = crate::debug::rf(CKM_)?;
    Some((val & (1 << 16)) != 0)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn hvg() -> Option<bool> {
    None
}


#[cfg(target_arch = "x86_64")]
pub fn foy() -> Option<(bool, u8)> {
    let val = crate::debug::rf(CKN_)?;
    let valid = (val & (1 << 31)) != 0;  
    let gqm = ((val >> 16) & 0x7F) as u8;  
    Some((valid, gqm))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn foy() -> Option<(bool, u8)> {
    None
}






#[cfg(target_arch = "x86_64")]
pub fn frx() -> Option<u8> {
    
    let val = crate::debug::rf(BDD_)?;
    let ims = ((val >> 40) & 0x1F) as u8;
    if ims > 0 {
        return Some(ims);
    }
    
    if let Some(plat) = crate::debug::rf(0xCE) {
        let aug = ((plat >> 8) & 0xFF) as u8;
        if aug > 0 {
            return Some(aug);
        }
    }
    
    let hpr = ((val >> 8) & 0xFF) as u8;
    if hpr > 0 { Some(hpr) } else { None }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn frx() -> Option<u8> { None }


#[cfg(target_arch = "x86_64")]
pub fn fry() -> Option<u8> {
    
    if let Some(plat) = crate::debug::rf(0xCE) {
        let inu = ((plat >> 40) & 0xFF) as u8;
        if inu > 0 {
            return Some(inu);
        }
    }
    
    
    Some(6)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn fry() -> Option<u8> { None }






pub fn knt(args: &[&str]) {
    use crate::framebuffer::*;

    if !sw() {
        if !probe() {
            crate::n!(A_, "EC not available — not a ThinkPad or EC unresponsive");
            return;
        }
    }

    match args.first().copied() {
        None | Some("status") => {
            
            crate::n!(C_, "=== ThinkPad Fan Status ===");
            
            if let Some(level) = luh() {
                let desc = match level {
                    0x80 => "auto (EC controlled)",
                    0x40 => "DISENGAGED (full speed)",
                    l if l <= 7 => match l {
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
                crate::println!("  Level: 0x{:02X} — {}", level, desc);
            } else {
                crate::n!(A_, "  Level: read failed");
            }

            if let Some(bvn) = fwf() {
                if bvn == 0 || bvn == 0xFFFF {
                    crate::println!("  RPM:   stopped or N/A");
                } else {
                    crate::println!("  RPM:   {}", bvn);
                }
            } else {
                crate::println!("  RPM:   read failed");
            }
        }

        Some("auto") => {
            if eme(FanLevel::Auto) {
                crate::n!(B_, "Fan set to AUTO (EC controlled)");
            } else {
                crate::n!(A_, "Failed to set fan to auto");
            }
        }

        Some("max") | Some("full") => {
            if eme(FanLevel::FullSpeed) {
                crate::n!(D_, "Fan set to FULL SPEED (disengaged)");
            } else {
                crate::n!(A_, "Failed to set fan to full speed");
            }
        }

        Some("off") | Some("0") => {
            crate::n!(D_, "WARNING: Turning fan off! Monitor temperatures carefully.");
            if eme(FanLevel::Gp(0)) {
                crate::n!(A_, "Fan OFF");
            } else {
                crate::n!(A_, "Failed to turn fan off");
            }
        }

        Some(ae) => {
            if let Ok(level) = ae.parse::<u8>() {
                if level <= 7 {
                    if eme(FanLevel::Gp(level)) {
                        crate::n!(B_, "Fan set to level {}", level);
                    } else {
                        crate::n!(A_, "Failed to set fan level");
                    }
                } else {
                    crate::n!(A_, "Fan level must be 0-7, 'auto', 'max', or 'off'");
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


pub fn ksn(_args: &[&str]) {
    use crate::framebuffer::*;

    if !sw() {
        if !probe() {
            crate::n!(A_, "EC not available — not a ThinkPad or EC unresponsive");
            return;
        }
    }

    crate::n!(C_, "=== ThinkPad Temperature Sensors ===");
    
    let mut hfj = false;
    for i in 0..ADJ_ {
        if let Some(ts) = pdu(i) {
            hfj = true;
            let color = if ts >= 90 {
                A_
            } else if ts >= 70 {
                D_
            } else {
                B_
            };
            crate::print!("  {:10} ", pdr(i));
            crate::n!(color, "{}°C", ts);
        }
    }

    if !hfj {
        crate::println!("  No temperature sensors responded");
    }

    
    #[cfg(target_arch = "x86_64")]
    if let Some((valid, dts)) = foy() {
        if valid {
            
            let tj_max: u8 = 100;
            let cpu_temp = tj_max.saturating_sub(dts);
            let color = if cpu_temp >= 90 {
                A_
            } else if cpu_temp >= 70 {
                D_
            } else {
                B_
            };
            crate::print!("  {:10} ", "CPU (DTS)");
            crate::n!(color, "{}°C (TjMax={}, margin={}°C)", cpu_temp, tj_max, dts);
        }
    }

    
    crate::println!();
    if let Some(bvn) = fwf() {
        if bvn > 0 && bvn != 0xFFFF {
            crate::println!("  Fan:       {} RPM", bvn);
        } else {
            crate::println!("  Fan:       stopped");
        }
    }
}


pub fn kmn(args: &[&str]) {
    use crate::framebuffer::*;

    match args.first().copied() {
        None | Some("status") => {
            crate::n!(C_, "=== CPU Frequency / Voltage ===");

            
            match hvg() {
                Some(true) => crate::n!(B_, "  SpeedStep (EIST): enabled"),
                Some(false) => crate::n!(D_, "  SpeedStep (EIST): disabled"),
                None => crate::println!("  SpeedStep (EIST): unknown"),
            }

            
            if let Some((freq, voltage)) = fou() {
                crate::println!("  Current freq:     {} MHz", freq);
                if voltage > 0 {
                    crate::println!("  Current voltage:  {}.{:03} V", voltage / 1000, voltage % 1000);
                }
            } else {
                crate::println!("  Current P-state:  read failed");
            }

            
            if let Some(target) = hoi() {
                let jmg = (target >> 8) & 0xFF;
                let pok = target & 0xFF;
                crate::println!("  Target:           FID={} VID={} ({}MHz)", jmg, pok, fwn(jmg as u32));
            }

            
            #[cfg(target_arch = "x86_64")]
            if let Some((valid, dts)) = foy() {
                if valid {
                    crate::println!("  CPU temp (DTS):   {}°C (margin: {}°C to TjMax)", 100u8.saturating_sub(dts), dts);
                }
            }

            
            crate::println!();
            let bsx = dmy();
            crate::n!(C_, "  Detected FSB: {}MHz", bsx);
            if let (Some(max_fid), Some(min_fid)) = (frx(), fry()) {
                crate::n!(C_, "  P-state range (FID x FSB):");
                let mut fid = max_fid;
                while fid >= min_fid && fid > 0 {
                    let freq = (fid as u32) * bsx;
                    let label = if fid == max_fid { " (max)" } else if fid == min_fid { " (min)" } else { "" };
                    crate::println!("    FID={:2}  → {} MHz{}", fid, freq, label);
                    if fid <= min_fid { break; }
                    fid = fid.saturating_sub(2); 
                    if fid < min_fid { fid = min_fid; }
                }
            } else {
                crate::println!("  (Could not detect CPU P-state range)");
            }
        }

        Some("set") => {
            if args.len() < 3 {
                crate::println!("Usage: cpufreq set <fid> <vid>");
                crate::println!("  Use 'cpufreq status' to see known P-states");
                return;
            }
            let fid = match args[1].parse::<u8>() {
                Ok(f) => f,
                Err(_) => {
                    crate::n!(A_, "Invalid FID: {}", args[1]);
                    return;
                }
            };
            let bpx = match args[2].parse::<u8>() {
                Ok(v) => v,
                Err(_) => {
                    crate::n!(A_, "Invalid VID: {}", args[2]);
                    return;
                }
            };
            crate::n!(D_, "Setting P-state: FID={} VID={} ({}MHz)", fid, bpx, fwn(fid as u32));
            if eja(fid, bpx) {
                crate::n!(B_, "P-state change requested");
                
                if let Some((freq, voltage)) = fou() {
                    crate::println!("  Now running at: {} MHz, {}.{:03} V", freq, voltage / 1000, voltage % 1000);
                }
            } else {
                crate::n!(A_, "Failed to set P-state");
            }
        }

        Some("max") => {
            if let Some(max_fid) = frx() {
                let bsx = dmy();
                let freq = max_fid as u32 * bsx;
                crate::n!(D_, "Setting CPU to max: FID={} ({}MHz)", max_fid, freq);
                
                eja(max_fid, 0);
                crate::n!(B_, "Done");
            } else {
                crate::n!(A_, "Could not detect max P-state");
            }
        }

        Some("min") | Some("powersave") => {
            if let Some(min_fid) = fry() {
                let bsx = dmy();
                let freq = min_fid as u32 * bsx;
                crate::n!(D_, "Setting CPU to min: FID={} ({}MHz)", min_fid, freq);
                eja(min_fid, 0);
                crate::n!(B_, "Done");
            } else {
                crate::n!(A_, "Could not detect min P-state");
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
