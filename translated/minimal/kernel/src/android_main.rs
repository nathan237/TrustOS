


















use crate::android_boot::{self, FdtHeader, DtbInfo, SocFamily};
use crate::hwscan::dtb_parser;






static mut AQS_: u64 = 0;


static mut AKO_: bool = false;


static mut BSI_: Option<DtbInfo> = None;






const CLA_: u64 = 0x8400_0000;
const EBI_: u64 = 0x8400_0008;
const EBJ_: u64 = 0x8400_0009;
const CKZ_: u64 = 0xC400_0003;
const EBH_: u64 = 0x8400_000A;

extern "C" {
    
    fn qih(aos: u64, dn: u64, hy: u64, ajr: u64) -> u64;
    
    fn qif() -> !;
    
    fn qig() -> !;
}


pub fn smc(aos: u64, dn: u64, hy: u64, ajr: u64) -> u64 {
    unsafe { qih(aos, dn, hy, ajr) }
}


pub fn vnu() -> (u32, u32) {
    let p = smc(CLA_, 0, 0, 0);
    let efb = (p >> 16) as u32;
    let efm = (p & 0xFFFF) as u32;
    (efb, efm)
}


pub fn wzp() -> ! {
    unsafe { qif() }
}


pub fn wzq() -> ! {
    unsafe { qig() }
}


pub fn yke(cih: u64, mi: u64, rol: u64) -> i64 {
    smc(CKZ_, cih, mi, rol) as i64
}







unsafe fn gfu(r: u8) {
    extern "C" {
        static jxt: u64;
    }
    let ar = jxt;
    if ar != 0 {
        let ptr = ar as *mut u32;
        
        for _ in 0..10000 {
            let xb = ptr.add(6).read_volatile(); 
            if xb & (1 << 5) == 0 {
                break;
            }
        }
        ptr.write_volatile(r as u32);
    }
}

unsafe fn ahk(e: &[u8]) {
    for &r in e {
        gfu(r);
    }
}


unsafe fn cxf(ap: u64) {
    let nu = b"0123456789ABCDEF";
    for a in (0..16).vv() {
        let uur = ((ap >> (a * 4)) & 0xF) as usize;
        gfu(nu[uur]);
    }
}


unsafe fn xpa(opj: u64) {
    extern "C" {
        static mut jxt: u64;
    }
    if opj != 0 {
        jxt = opj;
    }
}



















#[no_mangle]
pub unsafe extern "C" fn android_main(ceg: u64) -> ! {
    
    AKO_ = true;
    AQS_ = ceg;

    ahk(b"[ANDROID] TrustOS android_main entered\r\n");

    
    if ceg != 0 {
        let blq = FdtHeader::dxi(ceg as *const u8);
        if blq {
            ahk(b"[ANDROID] DTB valid at 0x");
            cxf(ceg);
            ahk(b"\r\n");
            BSI_ = DtbInfo::sxt(ceg as *const u8);
        } else {
            ahk(b"[ANDROID] WARNING: DTB magic invalid!\r\n");
        }
    } else {
        ahk(b"[ANDROID] No DTB provided\r\n");
    }

    
    let (vnr, vns) = vnu();
    ahk(b"[ANDROID] PSCI version: ");
    gfu(b'0' + vnr as u8);
    gfu(b'.');
    gfu(b'0' + vns as u8);
    ahk(b"\r\n");

    
    
    extern "C" {
        static qbg: u8;
    }
    let dip = &qbg as *const u8 as u64;
    let caa = (dip + 0xFFF) & !0xFFF; 
    let cre: usize = 32 * 1024 * 1024; 

    ahk(b"[ANDROID] Heap: 32 MB at 0x");
    cxf(caa);
    ahk(b"\r\n");

    crate::memory::tta(caa, cre);
    ahk(b"[ANDROID] Heap initialized\r\n");

    
    let mut brw: u64 = 0x4000_0000; 
    let mut cbf: u64 = 128 * 1024 * 1024; 
    let mut ifq: u64 = 0;
    let mut lbh = false;

    if ceg != 0 {
        if let Some(bez) = dtb_parser::jis(ceg as *const u8) {
            ahk(b"[DTB] Parsed ");
            cxf(bez.fpb as u64);
            ahk(b" nodes, ");
            cxf(bez.ik.len() as u64);
            ahk(b" devices\r\n");

            
            if let Some(&(ar, aw)) = bez.memory.fv() {
                brw = ar;
                cbf = aw;
                ahk(b"[DTB] RAM: 0x");
                cxf(ar);
                ahk(b" size 0x");
                cxf(aw);
                ahk(b"\r\n");
            }

            
            if bez.cnl != 0 {
                ifq = bez.cnl;
                ahk(b"[DTB] UART: 0x");
                cxf(ifq);
                ahk(b"\r\n");
            } else {
                
                for ba in &bez.ik {
                    if ba.bjp.contains("pl011")
                        || ba.bjp.contains("uart")
                        || ba.bjp.contains("serial")
                        || ba.bjp.contains("ns16550")
                        || ba.bjp.contains("geni")
                    {
                        if ba.status == "okay" || ba.status == "ok" {
                            ifq = ba.cbi;
                            ahk(b"[DTB] UART found: 0x");
                            cxf(ba.cbi);
                            ahk(b"\r\n");
                            break;
                        }
                    }
                }
            }

            
            if ifq != 0 {
                xpa(ifq);
                ahk(b"[ANDROID] UART switched to DTB address\r\n");
            }

            
            let jqr = rwv(&bez.bjp);
            android_boot::XF_ = jqr;
            ahk(b"[DTB] SoC detected: ");
            match jqr {
                SocFamily::Aeb => ahk(b"QEMU virt\r\n"),
                SocFamily::Ali => ahk(b"Qualcomm Snapdragon\r\n"),
                SocFamily::Anm => ahk(b"Google Tensor\r\n"),
                SocFamily::Agu => ahk(b"Broadcom (RPi)\r\n"),
                SocFamily::Ahz => ahk(b"Samsung Exynos\r\n"),
                SocFamily::Akf => ahk(b"MediaTek\r\n"),
                SocFamily::F => ahk(b"Unknown\r\n"),
            }

            
            if let Some(ref bxc) = bez.jql {
                ahk(b"[DTB] SimpleFB: 0x");
                cxf(bxc.ar);
                ahk(b" ");
                cxf(bxc.z as u64);
                ahk(b"x");
                cxf(bxc.ac as u64);
                ahk(b"\r\n");

                
                let cwa = match bxc.format.as_str() {
                    "a8r8g8b8" | "x8r8g8b8" | "a8b8g8r8" => 32u16,
                    "r5g6b5" => 16u16,
                    _ => 32u16, 
                };

                let jb = if bxc.oq > 0 {
                    bxc.oq as u64
                } else {
                    bxc.z as u64 * (cwa as u64 / 8)
                };

                
                crate::framebuffer::init(
                    bxc.ar as *mut u8,
                    bxc.z as u64,
                    bxc.ac as u64,
                    jb,
                    cwa,
                );
                crate::framebuffer::leh();
                lbh = true;
                ahk(b"[ANDROID] Framebuffer initialized from DTB SimpleFB\r\n");
            } else {
                
                for ba in &bez.ik {
                    if ba.bjp.contains("simple-framebuffer") {
                        ahk(b"[DTB] Found simple-framebuffer device\r\n");
                        
                        if ba.cbi != 0 {
                            
                            let d = 1080u64;
                            let i = 1920u64;
                            let cwa = 32u16;
                            let jb = d * 4;
                            crate::framebuffer::init(
                                ba.cbi as *mut u8, d, i, jb, cwa,
                            );
                            crate::framebuffer::leh();
                            lbh = true;
                        }
                        break;
                    }
                }
            }

            
            if !bez.awt.is_empty() {
                ahk(b"[DTB] Reserved memory: ");
                cxf(bez.awt.len() as u64);
                ahk(b" regions (firmware/TZ)\r\n");
            }
        } else {
            ahk(b"[DTB] Parse failed - using hardcoded defaults\r\n");
        }
    }

    
    if android_boot::XF_ == SocFamily::F {
        android_boot::XF_ = SocFamily::Aeb;
    }

    
    crate::serial::init();
    crate::serial_println!("[ANDROID] Serial module initialized");
    crate::serial_println!("[ANDROID] TrustOS v0.6.0-Android boot");

    
    crate::interrupts::init();
    crate::serial_println!("[ANDROID] Interrupts initialized");

    
    if lbh {
        crate::serial_println!("[ANDROID] SimpleFB active — entering graphical shell");
        crate::println!();
        crate::h!(crate::framebuffer::G_, 
            "  TrustOS v0.6.0 — Android Boot Mode");
        crate::h!(crate::framebuffer::B_, 
            "  Booted via fastboot | DTB-driven hardware discovery");
        crate::println!();
        crate::h!(crate::framebuffer::Q_,
            "  Type 'hwscan dtb' to see discovered hardware");
        crate::h!(crate::framebuffer::Q_,
            "  Type 'hwscan auto' for full security reconnaissance");
        crate::println!();
        crate::shell::vw();
    } else {
        
        crate::serial_println!("[ANDROID] No framebuffer — entering serial console");
        crate::serial_println!("TrustOS v0.6.0 — Serial Console (Android Boot)");
        crate::serial_println!("Type 'hwscan dtb' to see discovered hardware");
        crate::serial_println!("Type 'hwscan auto' for full security reconnaissance");
        
        
        whx();
    }
}


fn rwv(bjp: &[alloc::string::String]) -> SocFamily {
    for r in bjp {
        let pb = r.avd();
        if pb.contains("qemu") || pb.contains("virt") {
            return SocFamily::Aeb;
        }
        if pb.contains("qualcomm") || pb.contains("qcom") || pb.contains("sdm") 
            || pb.contains("sm8") || pb.contains("msm") {
            return SocFamily::Ali;
        }
        if pb.contains("google,gs") || pb.contains("tensor") {
            return SocFamily::Anm;
        }
        if pb.contains("samsung") || pb.contains("exynos") {
            return SocFamily::Ahz;
        }
        if pb.contains("mediatek") || pb.contains("mt6") || pb.contains("mt8") {
            return SocFamily::Akf;
        }
        if pb.contains("brcm") || pb.contains("broadcom") || pb.contains("bcm2") 
            || pb.contains("raspberrypi") {
            return SocFamily::Agu;
        }
    }
    SocFamily::F
}


unsafe fn whx() -> ! {
    use alloc::string::String;
    use alloc::vec::Vec;

    crate::serial_println!("\nTrustOS> ");
    let mut hdc = Vec::<u8>::new();

    loop {
        if let Some(hf) = crate::serial::xmu() {
            match hf {
                b'\r' | b'\n' => {
                    crate::serial_println!("");
                    let cmd = String::azw(&hdc).bkc();
                    let cmd = cmd.em();
                    if !cmd.is_empty() {
                        tkw(cmd);
                    }
                    hdc.clear();
                    crate::serial_print!("TrustOS> ");
                }
                0x7F | 0x08 => {
                    
                    if !hdc.is_empty() {
                        hdc.pop();
                        crate::serial_print!("\x08 \x08");
                    }
                }
                _ => {
                    hdc.push(hf);
                    unsafe { gfu(hf); }
                }
            }
        }
        
        core::arch::asm!("yield", options(nomem, nostack));
    }
}


fn tkw(cmd: &str) {
    let ek: alloc::vec::Vec<&str> = cmd.ayt().collect();
    let ro = ek.fv().hu().unwrap_or("");
    let n = &ek[1..];

    match ro {
        "hwscan" | "trustprobe" | "probe" => {
            let result = crate::hwscan::oaf(n);
            
            let dox: alloc::string::String = result.bw()
                .hi(|&r| r != '\x01')
                .collect();
            crate::serial_println!("{}", dox);
        }
        "reboot" => {
            crate::serial_println!("Rebooting via PSCI...");
            wzq();
        }
        "poweroff" | "halt" => {
            crate::serial_println!("Shutting down via PSCI...");
            wzp();
        }
        "help" => {
            crate::serial_println!("TrustOS Serial Console Commands:");
            crate::serial_println!("  hwscan <cmd>  - Hardware security scanner");
            crate::serial_println!("  reboot        - Reboot via PSCI");
            crate::serial_println!("  poweroff      - Shutdown via PSCI");
            crate::serial_println!("  help          - This message");
        }
        _ => {
            crate::serial_println!("Unknown command: {}", cmd);
            crate::serial_println!("Type 'help' for available commands");
        }
    }
}






pub fn yyw() -> bool {
    unsafe { AKO_ }
}


pub fn kry() -> u64 {
    unsafe { AQS_ }
}
