


















use crate::android_boot::{self, FdtHeader, DtbInfo, SocFamily};
use crate::hwscan::dtb_parser;






static mut ASV_: u64 = 0;


static mut AMI_: bool = false;


static mut BVE_: Option<DtbInfo> = None;






const COJ_: u64 = 0x8400_0000;
const EEY_: u64 = 0x8400_0008;
const EEZ_: u64 = 0x8400_0009;
const COI_: u64 = 0xC400_0003;
const EEX_: u64 = 0x8400_000A;

extern "C" {
    
    fn android_smc_call(fid: u64, x1: u64, x2: u64, x3: u64) -> u64;
    
    fn android_psci_off() -> !;
    
    fn android_psci_reset() -> !;
}


pub fn smc(fid: u64, x1: u64, x2: u64, x3: u64) -> u64 {
    unsafe { android_smc_call(fid, x1, x2, x3) }
}


pub fn nzf() -> (u32, u32) {
    let v = smc(COJ_, 0, 0, 0);
    let axz = (v >> 16) as u32;
    let ayh = (v & 0xFFFF) as u32;
    (axz, ayh)
}


pub fn pcc() -> ! {
    unsafe { android_psci_off() }
}


pub fn pcd() -> ! {
    unsafe { android_psci_reset() }
}


pub fn qbl(target_cpu: u64, entry_point: u64, context_id: u64) -> i64 {
    smc(COI_, target_cpu, entry_point, context_id) as i64
}







unsafe fn cwy(c: u8) {
    extern "C" {
        static __uart_base: u64;
    }
    let base = __uart_base;
    if base != 0 {
        let ptr = base as *mut u32;
        
        for _ in 0..10000 {
            let ko = ptr.add(6).read_volatile(); 
            if ko & (1 << 5) == 0 {
                break;
            }
        }
        ptr.write_volatile(c as u32);
    }
}

unsafe fn qy(j: &[u8]) {
    for &c in j {
        cwy(c);
    }
}


unsafe fn bbf(val: u64) {
    let ga = b"0123456789ABCDEF";
    for i in (0..16).rev() {
        let nkk = ((val >> (i * 4)) & 0xF) as usize;
        cwy(ga[nkk]);
    }
}


unsafe fn ppy(new_base: u64) {
    extern "C" {
        static mut __uart_base: u64;
    }
    if new_base != 0 {
        __uart_base = new_base;
    }
}



















#[no_mangle]
pub unsafe extern "C" fn android_main(dtb_ptr: u64) -> ! {
    
    AMI_ = true;
    ASV_ = dtb_ptr;

    qy(b"[ANDROID] TrustOS android_main entered\r\n");

    
    if dtb_ptr != 0 {
        let valid = FdtHeader::bpu(dtb_ptr as *const u8);
        if valid {
            qy(b"[ANDROID] DTB valid at 0x");
            bbf(dtb_ptr);
            qy(b"\r\n");
            BVE_ = DtbInfo::lzd(dtb_ptr as *const u8);
        } else {
            qy(b"[ANDROID] WARNING: DTB magic invalid!\r\n");
        }
    } else {
        qy(b"[ANDROID] No DTB provided\r\n");
    }

    
    let (psci_major, psci_minor) = nzf();
    qy(b"[ANDROID] PSCI version: ");
    cwy(b'0' + psci_major as u8);
    cwy(b'.');
    cwy(b'0' + psci_minor as u8);
    qy(b"\r\n");

    
    
    extern "C" {
        static __kernel_end: u8;
    }
    let bhk = &__kernel_end as *const u8 as u64;
    let heap_start = (bhk + 0xFFF) & !0xFFF; 
    let atz: usize = 32 * 1024 * 1024; 

    qy(b"[ANDROID] Heap: 32 MB at 0x");
    bbf(heap_start);
    qy(b"\r\n");

    crate::memory::mov(heap_start, atz);
    qy(b"[ANDROID] Heap initialized\r\n");

    
    let mut ram_base: u64 = 0x4000_0000; 
    let mut ram_size: u64 = 128 * 1024 * 1024; 
    let mut edd: u64 = 0;
    let mut gaa = false;

    if dtb_ptr != 0 {
        if let Some(parsed) = dtb_parser::ewg(dtb_ptr as *const u8) {
            qy(b"[DTB] Parsed ");
            bbf(parsed.node_count as u64);
            qy(b" nodes, ");
            bbf(parsed.devices.len() as u64);
            qy(b" devices\r\n");

            
            if let Some(&(base, size)) = parsed.memory.first() {
                ram_base = base;
                ram_size = size;
                qy(b"[DTB] RAM: 0x");
                bbf(base);
                qy(b" size 0x");
                bbf(size);
                qy(b"\r\n");
            }

            
            if parsed.uart_base != 0 {
                edd = parsed.uart_base;
                qy(b"[DTB] UART: 0x");
                bbf(edd);
                qy(b"\r\n");
            } else {
                
                for s in &parsed.devices {
                    if s.compatible.contains("pl011")
                        || s.compatible.contains("uart")
                        || s.compatible.contains("serial")
                        || s.compatible.contains("ns16550")
                        || s.compatible.contains("geni")
                    {
                        if s.status == "okay" || s.status == "ok" {
                            edd = s.reg_base;
                            qy(b"[DTB] UART found: 0x");
                            bbf(s.reg_base);
                            qy(b"\r\n");
                            break;
                        }
                    }
                }
            }

            
            if edd != 0 {
                ppy(edd);
                qy(b"[ANDROID] UART switched to DTB address\r\n");
            }

            
            let fbe = ldz(&parsed.compatible);
            android_boot::YM_ = fbe;
            qy(b"[DTB] SoC detected: ");
            match fbe {
                SocFamily::QemuVirt => qy(b"QEMU virt\r\n"),
                SocFamily::Qualcomm => qy(b"Qualcomm Snapdragon\r\n"),
                SocFamily::Tensor => qy(b"Google Tensor\r\n"),
                SocFamily::Broadcom => qy(b"Broadcom (RPi)\r\n"),
                SocFamily::Exynos => qy(b"Samsung Exynos\r\n"),
                SocFamily::MediaTek => qy(b"MediaTek\r\n"),
                SocFamily::Unknown => qy(b"Unknown\r\n"),
            }

            
            if let Some(ref sfb) = parsed.simplefb {
                qy(b"[DTB] SimpleFB: 0x");
                bbf(sfb.base);
                qy(b" ");
                bbf(sfb.width as u64);
                qy(b"x");
                bbf(sfb.height as u64);
                qy(b"\r\n");

                
                let bpp = match sfb.format.as_str() {
                    "a8r8g8b8" | "x8r8g8b8" | "a8b8g8r8" => 32u16,
                    "r5g6b5" => 16u16,
                    _ => 32u16, 
                };

                let pitch = if sfb.stride > 0 {
                    sfb.stride as u64
                } else {
                    sfb.width as u64 * (bpp as u64 / 8)
                };

                
                crate::framebuffer::init(
                    sfb.base as *mut u8,
                    sfb.width as u64,
                    sfb.height as u64,
                    pitch,
                    bpp,
                );
                crate::framebuffer::gcq();
                gaa = true;
                qy(b"[ANDROID] Framebuffer initialized from DTB SimpleFB\r\n");
            } else {
                
                for s in &parsed.devices {
                    if s.compatible.contains("simple-framebuffer") {
                        qy(b"[DTB] Found simple-framebuffer device\r\n");
                        
                        if s.reg_base != 0 {
                            
                            let w = 1080u64;
                            let h = 1920u64;
                            let bpp = 32u16;
                            let pitch = w * 4;
                            crate::framebuffer::init(
                                s.reg_base as *mut u8, w, h, pitch, bpp,
                            );
                            crate::framebuffer::gcq();
                            gaa = true;
                        }
                        break;
                    }
                }
            }

            
            if !parsed.reserved.is_empty() {
                qy(b"[DTB] Reserved memory: ");
                bbf(parsed.reserved.len() as u64);
                qy(b" regions (firmware/TZ)\r\n");
            }
        } else {
            qy(b"[DTB] Parse failed - using hardcoded defaults\r\n");
        }
    }

    
    if android_boot::YM_ == SocFamily::Unknown {
        android_boot::YM_ = SocFamily::QemuVirt;
    }

    
    crate::serial::init();
    crate::serial_println!("[ANDROID] Serial module initialized");
    crate::serial_println!("[ANDROID] TrustOS v0.6.0-Android boot");

    
    crate::interrupts::init();
    crate::serial_println!("[ANDROID] Interrupts initialized");

    
    if gaa {
        crate::serial_println!("[ANDROID] SimpleFB active — entering graphical shell");
        crate::println!();
        crate::n!(crate::framebuffer::G_, 
            "  TrustOS v0.6.0 — Android Boot Mode");
        crate::n!(crate::framebuffer::B_, 
            "  Booted via fastboot | DTB-driven hardware discovery");
        crate::println!();
        crate::n!(crate::framebuffer::R_,
            "  Type 'hwscan dtb' to see discovered hardware");
        crate::n!(crate::framebuffer::R_,
            "  Type 'hwscan auto' for full security reconnaissance");
        crate::println!();
        crate::shell::run();
    } else {
        
        crate::serial_println!("[ANDROID] No framebuffer — entering serial console");
        crate::serial_println!("TrustOS v0.6.0 — Serial Console (Android Boot)");
        crate::serial_println!("Type 'hwscan dtb' to see discovered hardware");
        crate::serial_println!("Type 'hwscan auto' for full security reconnaissance");
        
        
        ooi();
    }
}


fn ldz(compatible: &[alloc::string::String]) -> SocFamily {
    for c in compatible {
        let gj = c.to_ascii_lowercase();
        if gj.contains("qemu") || gj.contains("virt") {
            return SocFamily::QemuVirt;
        }
        if gj.contains("qualcomm") || gj.contains("qcom") || gj.contains("sdm") 
            || gj.contains("sm8") || gj.contains("msm") {
            return SocFamily::Qualcomm;
        }
        if gj.contains("google,gs") || gj.contains("tensor") {
            return SocFamily::Tensor;
        }
        if gj.contains("samsung") || gj.contains("exynos") {
            return SocFamily::Exynos;
        }
        if gj.contains("mediatek") || gj.contains("mt6") || gj.contains("mt8") {
            return SocFamily::MediaTek;
        }
        if gj.contains("brcm") || gj.contains("broadcom") || gj.contains("bcm2") 
            || gj.contains("raspberrypi") {
            return SocFamily::Broadcom;
        }
    }
    SocFamily::Unknown
}


unsafe fn ooi() -> ! {
    use alloc::string::String;
    use alloc::vec::Vec;

    crate::serial_println!("\nTrustOS> ");
    let mut dku = Vec::<u8>::new();

    loop {
        if let Some(byte) = crate::serial::poa() {
            match byte {
                b'\r' | b'\n' => {
                    crate::serial_println!("");
                    let cmd = String::from_utf8_lossy(&dku).into_owned();
                    let cmd = cmd.trim();
                    if !cmd.is_empty() {
                        mij(cmd);
                    }
                    dku.clear();
                    crate::serial_print!("TrustOS> ");
                }
                0x7F | 0x08 => {
                    
                    if !dku.is_empty() {
                        dku.pop();
                        crate::serial_print!("\x08 \x08");
                    }
                }
                _ => {
                    dku.push(byte);
                    unsafe { cwy(byte); }
                }
            }
        }
        
        core::arch::asm!("yield", options(nomem, nostack));
    }
}


fn mij(cmd: &str) {
    let au: alloc::vec::Vec<&str> = cmd.split_whitespace().collect();
    let command = au.first().copied().unwrap_or("");
    let args = &au[1..];

    match command {
        "hwscan" | "trustprobe" | "probe" => {
            let result = crate::hwscan::idk(args);
            
            let blc: alloc::string::String = result.chars()
                .filter(|&c| c != '\x01')
                .collect();
            crate::serial_println!("{}", blc);
        }
        "reboot" => {
            crate::serial_println!("Rebooting via PSCI...");
            pcd();
        }
        "poweroff" | "halt" => {
            crate::serial_println!("Shutting down via PSCI...");
            pcc();
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






pub fn qlz() -> bool {
    unsafe { AMI_ }
}


pub fn ftl() -> u64 {
    unsafe { ASV_ }
}
