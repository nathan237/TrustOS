






#![no_std]
#![no_main]
#![cfg_attr(target_arch = "x86_64", feature(ydz))]
#![feature(alloc_error_handler)]



#![allow(bgr)]
#![allow(moc)]
#![allow(zut)]
#![allow(zuq)]
#![allow(zur)]
#![allow(zum)]
#![allow(zuo)]
#![allow(zus)]
#![allow(zun)]
#![allow(zug)]
#![allow(zuh)]
#![allow(ynt)]
#![allow(deprecated)]
#![allow(zdt)]
#![allow(zds)]
#![allow(zdv)]
#![allow(zpq)]
#![allow(yfd)]
#![allow(zgn)]
#![allow(ztx)]
#![allow(zcv)]
#![allow(zup)]
#![allow(yry)]
#![allow(zuf)]
extern crate alloc;


pub mod arch;


mod serial;
mod logger;
mod framebuffer;
mod keyboard;
mod shell;
mod ramfs;
mod rtc;
mod mouse;
mod touch;
mod gesture;
mod task;
mod desktop;
mod mobile;
mod visualizer;
mod drone_swarm;
mod disk;
mod network;
mod pci;
mod virtio;
mod virtio_net;
mod virtio_blk;
mod nvme;
mod drivers;
mod netstack;
mod time;
mod rng;
mod file_assoc;
mod accessibility;
mod ui;
mod apps;
mod graphics;
mod icons;
mod browser;
mod game3d; 
mod chess;   
mod chess3d; 
#[cfg(feature = "emulators")]
mod nes;     
#[cfg(feature = "emulators")]
mod gameboy; 
#[cfg(feature = "emulators")]
mod game_lab; 
#[cfg(feature = "emulators")]
mod embedded_roms; 
mod cosmic; 
mod compositor; 
mod holovolume; 
mod matrix_fast; 
mod formula3d;   
mod gpu_emu;      

mod tls13;

#[cfg(target_arch = "x86_64")]
mod cpu;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/cpu.rs"]
mod cpu;


#[cfg(target_arch = "x86_64")]
mod acpi;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/acpi.rs"]
mod acpi;


#[cfg(target_arch = "x86_64")]
mod apic;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/apic.rs"]
mod apic;


mod vfs;
mod process;
mod elf;
mod exec;
mod init;
mod pipe;
#[cfg(target_arch = "x86_64")]
mod gdt;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/gdt.rs"]
mod gdt;

#[cfg(target_arch = "x86_64")]
mod userland;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/userland.rs"]
mod userland;
mod thread;
mod auth;


mod distro;


mod linux;


mod linux_compat;


mod compression;


mod persistence;


mod wayland;


mod transpiler;


mod riscv_translator;


mod binary_analysis;


mod netscan;


mod hwscan;


mod httpd;


mod trustpkg;


mod lab_mode;



#[cfg_attr(not(feature = "hires-logo"), path = "logo_bitmap_stub.rs")]
mod logo_bitmap;


mod trustlang;


mod video;


#[cfg(target_arch = "aarch64")]
mod android_boot;
#[cfg(target_arch = "aarch64")]
mod android_main;


mod memory;
mod interrupts;
mod scheduler;
mod ipc;
mod security;
mod trace;
mod syscall;
mod gui;
mod theme;
mod image;
mod perf;
#[cfg(target_arch = "x86_64")]
mod hypervisor;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/hypervisor.rs"]
mod hypervisor;
mod rasterizer;
mod model_editor;


pub mod math;


pub mod draw_utils;


mod devtools;


mod debug;


mod audio;


mod trustdaw;


mod sandbox;


mod signature;

pub mod ed25519;


mod sync;

mod signals;

mod tty;

mod pty;

mod ptrace;

mod usercopy;


mod jarvis;


mod jarvis_hw;

use core::panic::Ciy;
use core::alloc::Layout;
use limine::request::{
    FramebufferRequest, MemoryMapRequest, HhdmRequest,
    RequestsStartMarker, RequestsEndMarker, ModuleRequest,
    RsdpRequest, SmpRequest, KernelAddressRequest, KernelFileRequest,
    StackSizeRequest,
};
use limine::BaseRevision;






#[mr]
#[unsafe(link_section = ".requests_start_marker")]
static ELM_: RequestsStartMarker = RequestsStartMarker::new();


#[mr]
#[unsafe(link_section = ".requests")]
static BKZ_: BaseRevision = BaseRevision::new();


#[mr]
#[unsafe(link_section = ".requests")]
static BVJ_: FramebufferRequest = FramebufferRequest::new();


#[mr]
#[unsafe(link_section = ".requests")]
static CGG_: MemoryMapRequest = MemoryMapRequest::new();


#[mr]
#[unsafe(link_section = ".requests")]
static BZL_: HhdmRequest = HhdmRequest::new();


#[mr]
#[unsafe(link_section = ".requests")]
static CGZ_: ModuleRequest = ModuleRequest::new();


static mut AXM_: Option<&'static [u8]> = None;


#[mr]
#[unsafe(link_section = ".requests")]
static CPF_: RsdpRequest = RsdpRequest::new();


#[mr]
#[unsafe(link_section = ".requests")]
static CTO_: SmpRequest = SmpRequest::new();


#[mr]
#[unsafe(link_section = ".requests")]
static CCU_: KernelAddressRequest = KernelAddressRequest::new();


#[mr]
#[unsafe(link_section = ".requests")]
static CCV_: KernelFileRequest = KernelFileRequest::new();




#[mr]
#[unsafe(link_section = ".requests")]
static EGS_: StackSizeRequest = StackSizeRequest::new().zwh(512 * 1024);


#[mr]
#[unsafe(link_section = ".requests_end_marker")]
static ELG_: RequestsEndMarker = RequestsEndMarker::new();






#[mr]
#[link_section = ".text"]
static DSG_: unsafe extern "C" fn() -> ! = ubn;









#[no_mangle]
#[link_section = ".text.kmain"]
pub unsafe extern "C" fn ubn() -> ! {
    
    if !BKZ_.gkj() {
        hmj();
    }

    
    
    
    #[cfg(target_arch = "aarch64")]
    {
        if let Some(ohn) = CCU_.fjo() {
            crate::arch::platform::boot::wle(
                ohn.zvk(),
                ohn.zfb(),
            );
        }
        
    }

    
    if let Some(ubj) = CCV_.fjo() {
        let file = ubj.file();
        let ptr = file.ag();
        let aw = file.aw() as usize;
        if aw > 0 {
            unsafe { jarvis::pxe_replicator::vue(ptr, aw); }
        }
    }

    
    serial::init();
    debug::cpc(debug::CKP_, "Serial port initialized");
    serial_println!("T-RustOs Kernel v0.2.0");
    serial_println!("Limine protocol supported");

    
    #[cfg(target_arch = "aarch64")]
    {
        let kms: u64;
        core::arch::asm!("mrs {}, CurrentEL", bd(reg) kms);
        let ij = (kms >> 2) & 3;
        serial_println!("[AARCH64] CurrentEL = EL{}", ij);
        if ij == 2 {
            serial_println!("[EL2] *** Hypervisor mode detected! ***");
            serial_println!("[EL2] ARM EL2 MMIO Spy available — use 'hv el2' in shell");
        } else if ij == 1 {
            serial_println!("[EL1] Standard kernel mode (no hypervisor)");
        }
    }

    
    if let Some(srf) = BVJ_.fjo() {
        if let Some(pq) = srf.yrn().next() {
            framebuffer::init(
                pq.ag(),
                pq.z(),
                pq.ac(),
                pq.jb(),
                pq.cwa(),
            );
            serial_println!("Framebuffer: {}x{} @ {:p}", pq.z(), pq.ac(), pq.ag());
        }
    }
    
    
    framebuffer::led();
    
    use framebuffer::BootStatus;

    
    serial_println!("Initializing memory management...");
    
    let mut lbz = false;
    
    if let Some(fol) = CGG_.fjo() {
        let lr = BZL_.fjo()
            .map(|m| m.l())
            .unwrap_or(0);
        
        serial_println!("HHDM offset: {:#x}", lr);
        serial_println!("Memory map entries: {}", fol.ch().len());
        
        
        let mut juy: Option<u64> = None;
        let mut dip: u64 = 0;
        let mut iel: u64 = 0;
        
        
        for bt in fol.ch() {
            let kk = match bt.avt {
                limine::memory_map::EntryType::Qz => "USABLE",
                limine::memory_map::EntryType::Bqb => "RESERVED",
                limine::memory_map::EntryType::AKE_ => "ACPI_RECLAIM",
                limine::memory_map::EntryType::BJI_ => "ACPI_NVS",
                limine::memory_map::EntryType::BKX_ => "BAD",
                limine::memory_map::EntryType::ZM_ => "BOOTLOADER",
                limine::memory_map::EntryType::ARL_ => "KERNEL",
                limine::memory_map::EntryType::Cdf => "FRAMEBUFFER",
                _ => "UNKNOWN",
            };
            serial_println!("  {:#012x} - {:#012x} ({:12} bytes) {}", 
                bt.ar, 
                bt.ar + bt.go,
                bt.go,
                kk
            );
            
            
            let ifl = match bt.avt {
                limine::memory_map::EntryType::Qz => 0u8,
                limine::memory_map::EntryType::Bqb => 1,
                limine::memory_map::EntryType::AKE_ => 2,
                limine::memory_map::EntryType::BJI_ => 3,
                limine::memory_map::EntryType::BKX_ => 4,
                limine::memory_map::EntryType::ZM_ => 5,
                limine::memory_map::EntryType::ARL_ => 6,
                limine::memory_map::EntryType::Cdf => 7,
                _ => 0xFF,
            };
            memory::wuv(bt.ar, bt.go, ifl);
            
            
            if bt.avt == limine::memory_map::EntryType::ARL_
                || bt.avt == limine::memory_map::EntryType::ZM_
            {
                let ci = bt.ar + bt.go;
                if ci > dip {
                    dip = ci;
                }
            }
            
            if bt.avt == limine::memory_map::EntryType::Qz
                || bt.avt == limine::memory_map::EntryType::ZM_
                || bt.avt == limine::memory_map::EntryType::AKE_
            {
                iel += bt.go;
            }
        }

        
        memory::wjt(iel);
        serial_println!("[MEM] Total physical memory: {} MB", iel / 1024 / 1024);

        
        let epn = memory::rnh(iel);
        serial_println!("[HEAP] Dynamic size: {} MB (25% of {} MB RAM)", 
            epn / 1024 / 1024, iel / 1024 / 1024);

        
        let ijg = |ag: u64, align: u64| -> u64 {
            if ag % align == 0 { ag } else { ag + (align - (ag % align)) }
        };

        
        let oni = ijg(core::cmp::am(0x100000, dip), 0x1000);
        for bt in fol.ch() {
            if bt.avt != limine::memory_map::EntryType::Qz {
                continue;
            }
            let lyo = bt.ar;
            let exn = bt.ar.akq(bt.go);
            if exn <= oni {
                continue;
            }
            let caa = core::cmp::am(lyo, oni);
            if exn >= caa + epn as u64 {
                juy = Some(caa);
                break;
            }
        }

        
        if juy.is_none() {
            let mut fdg: u64 = 0;
            let mut cjg: u64 = 0;
            for bt in fol.ch() {
                if bt.avt != limine::memory_map::EntryType::Qz {
                    continue;
                }
                if bt.go > cjg {
                    cjg = bt.go;
                    fdg = bt.ar;
                }
            }

            if cjg >= epn as u64 {
                let mut caa = ijg(fdg, 0x1000);
                if caa < 0x100000 {
                    caa = ijg(0x100000, 0x1000);
                }
                if fdg.akq(cjg) >= caa + epn as u64 {
                    juy = Some(caa);
                }
            }
        }
        
        
        if let Some(dhz) = juy {
            serial_println!("[HEAP] Using mmap region at phys {:#x}, size {} MB", dhz, epn / 1024 / 1024);
            
            memory::lej(lr, dhz, epn);
            lbz = true;
            serial_println!("[HEAP] Initialized: free={} KB", memory::heap::aez() / 1024);
            
            
            
            let fau: alloc::vec::Vec<memory::frame::Adt> = fol.ch()
                .iter()
                .hi(|aa| aa.avt == limine::memory_map::EntryType::Qz)
                .map(|aa| memory::frame::Adt { ar: aa.ar, go: aa.go })
                .collect();
            memory::frame::init(&fau, dhz, epn as u64);
        } else {
            
            
            let mut fdg: u64 = 0;
            let mut cjg: u64 = 0;
            for bt in fol.ch() {
                if bt.avt != limine::memory_map::EntryType::Qz && bt.go > cjg {
                    
                }
                if bt.avt == limine::memory_map::EntryType::Qz && bt.go > cjg {
                    cjg = bt.go;
                    fdg = bt.ar;
                }
            }
            
            if cjg >= memory::TW_ as u64 {
                let iyf = ijg(core::cmp::am(fdg, 0x100000), 0x1000);
                let xpe = fdg.akq(cjg);
                let hmu = core::cmp::v(
                    (xpe - iyf) as usize,
                    epn,
                );
                let hmu = (hmu / 4096) * 4096; 
                
                serial_println!("[HEAP] Fallback: using {:#x} size {} MB", iyf, hmu / 1024 / 1024);
                memory::lej(lr, iyf, hmu);
                lbz = true;
                serial_println!("[HEAP] Initialized: free={} KB", memory::heap::aez() / 1024);
                
                
                let fau: alloc::vec::Vec<memory::frame::Adt> = fol.ch()
                    .iter()
                    .hi(|aa| aa.avt == limine::memory_map::EntryType::Qz)
                    .map(|aa| memory::frame::Adt { ar: aa.ar, go: aa.go })
                    .collect();
                memory::frame::init(&fau, iyf, hmu as u64);
            } else {
                serial_println!("[HEAP] ERROR: No usable region found for heap!");
            }
        }
    }
    
    if !lbz {
        
        serial_println!("[HEAP] Using fallback init");
        memory::init();
    }
    
    
    serial_println!("[FB] Initializing scrollback buffer...");
    framebuffer::leh();
    
    
    signature::ttp();
    
    
    signature::ttl(&[0x54, 0x72, 0x75, 0x73, 0x74, 0x4f, 0x53]); 
    
    
    framebuffer::bir(0, "Memory management initialized");
    framebuffer::sd("Memory management initialized", BootStatus::Ok);

    
    #[cfg(target_arch = "x86_64")]
    {
        debug::cpc(debug::CKK_, "GDT init");
        serial_println!("Initializing GDT with Ring 0/3 support...");
        gdt::init();
        framebuffer::bir(1, "GDT initialized (Ring 0/3)");
        framebuffer::sd("GDT initialized (Ring 0/3)", BootStatus::Ok);
    }
    
    
    debug::cpc(debug::CKL_, "IDT/interrupts init");
    serial_println!("Initializing early interrupts...");
    interrupts::init();
    framebuffer::bir(2, "Interrupts initialized");
    framebuffer::sd("Interrupts (early)", BootStatus::Ok);
    
    
    #[cfg(target_arch = "x86_64")]
    {
        
        debug::cpc(debug::CKI_, "CPU detection");
        serial_println!("Detecting CPU capabilities...");
        cpu::init();
        framebuffer::bir(3, "CPU capabilities detected");
        framebuffer::sd("CPU capabilities detected", BootStatus::Ok);
        
        
        debug::cpc(debug::CKG_, "ACPI tables parsing");
        serial_println!("Parsing ACPI tables...");
        if let Some(waw) = CPF_.fjo() {
            let gre = waw.re();
            serial_println!("[DEBUG] RSDP pointer from Limine: {:#x}", gre as usize);
            if acpi::oeh(gre as u64) {
                if let Some(co) = acpi::ani() {
                    framebuffer::sd(&alloc::format!(
                        "ACPI: {} CPUs, {} I/O APICs", 
                        co.aao, co.cyx.len()
                    ), BootStatus::Ok);
                }
            } else {
                framebuffer::sd("ACPI init failed", BootStatus::Ej);
            }
        } else {
            framebuffer::sd("No RSDP from bootloader", BootStatus::Ej);
        }
        
        
        debug::cpc(debug::CKH_, "APIC init");
        serial_println!("Initializing APIC...");
        if apic::init() {
            framebuffer::sd("APIC initialized (LAPIC + IOAPIC)", BootStatus::Ok);
        } else {
            serial_println!("[APIC] Not available, staying on legacy PIC");
            framebuffer::sd("APIC not available (legacy PIC)", BootStatus::Ej);
        }

        
        if acpi::hpet::init() {
            framebuffer::sd("HPET initialized", BootStatus::Ok);
        } else {
            framebuffer::sd("HPET not available", BootStatus::Ej);
        }

        
        debug::cpc(debug::CKR_, "SMP multi-core init");
        serial_println!("Initializing SMP...");
        cpu::smp::init();
        
        if let Some(plq) = CTO_.fjo() {
            let aao = plq.cdv().len();
            serial_println!("[SMP] Found {} CPUs via Limine", aao);
            
            for cpu in plq.cdv().iter() {
                if cpu.ad != 0 {
                    serial_println!("[SMP] Starting AP {} (LAPIC ID: {})", cpu.ad, cpu.ett);
                    cpu.dhq.write(cpu::smp::mvx);
                }
            }
            
            let mut exk = 1u32;
            for _ in 0..1000 {
                exk = cpu::smp::boc();
                if exk >= aao as u32 { break; }
                for _ in 0..10000 { core::hint::hc(); }
            }
            
            serial_println!("[SMP] {} of {} CPUs online", exk, aao);
            cpu::smp::piv(aao as u32);
            framebuffer::bir(4, "SMP multi-core active");
            framebuffer::sd(&alloc::format!("SMP: {} cores active", exk), BootStatus::Ok);
        } else {
            serial_println!("[SMP] No SMP response from bootloader");
            framebuffer::sd("SMP: single core", BootStatus::Ok);
        }
    }
    
    
    serial_println!("Initializing paging subsystem...");
    memory::paging::init();  
    framebuffer::bir(5, "Paging & memory protection");
    framebuffer::sd("Paging initialized (NX enabled)", BootStatus::Ok);
    
    
    #[cfg(target_arch = "x86_64")]
    let lgp = acpi::ani()
        .map(|co| co.clo.em().dha("VBOX"))
        .unwrap_or(false);
    #[cfg(not(target_arch = "x86_64"))]
    let lgp = false;
    if lgp {
        serial_println!("[PAT] Skipping Write-Combining on VirtualBox (VMSVGA compat)");
    } else {
        memory::paging::wlh();
    }
    
    
    #[cfg(target_arch = "x86_64")]
    {
        serial_println!("Initializing userland support...");
        userland::oen();
        userland::init();
        framebuffer::sd("Userland support ready", BootStatus::Ok);
    }
    
    
    serial_println!("Initializing thread subsystem...");
    thread::init();
    framebuffer::bir(6, "Thread subsystem ready");
    framebuffer::sd("Thread subsystem ready", BootStatus::Ok);
    
    
    serial_println!("Initializing security subsystem...");
    
    framebuffer::sd("Security (basic)", BootStatus::Ok);

    
    #[cfg(target_arch = "x86_64")]
    {
    
    keyboard::tto();
    serial_println!("Keyboard driver ready");
    framebuffer::bir(7, "Keyboard & input devices");
    framebuffer::sd("Keyboard ready", BootStatus::Ok);

    
    
    
    
    const BTR_: bool = true;
    if BTR_ {
        serial_println!("[RTC] init start");
        if rtc::xmo() {
            framebuffer::sd("RTC initialized", BootStatus::Ok);
        } else {
            framebuffer::sd("RTC skipped", BootStatus::Ej);
        }
        serial_println!("[RTC] init done");
    } else {
        framebuffer::sd("RTC disabled", BootStatus::Ej);
    }

    
    rng::init();
    framebuffer::sd("RNG (CSPRNG)", BootStatus::Ok);
    
    
    mouse::init();
    let (lu, qh) = framebuffer::yn();
    mouse::dbw(lu, qh);
    framebuffer::bir(8, "Mouse & touch input");
    framebuffer::sd("Mouse initialized", BootStatus::Ok);
    
    
    touch::init();
    touch::dbw(lu, qh);
    framebuffer::sd("Touch input ready", BootStatus::Ok);
    
    
    const BTQ_: bool = true;
    const BTS_: bool = true;
    const BTN_: bool = true;   
    const BTO_: bool = true;
    const BTP_: bool = true;

    
    debug::cpc(debug::CKN_, "PCI bus enumeration");
    serial_println!("[PHASE] PCI init start");
    framebuffer::sd("PCI bus scanning...", BootStatus::V);
    if BTQ_ {
        pci::init();
        framebuffer::bir(9, "PCI bus enumeration");
        framebuffer::sd("PCI bus scanned", BootStatus::Ok);
    } else {
        framebuffer::sd("PCI disabled", BootStatus::Ej);
    }
    serial_println!("[PHASE] PCI init done");
    
    
    serial_println!("[PHASE] Task scheduler init start");
    if BTS_ {
        task::init();
        scheduler::init();
        framebuffer::bir(10, "Task scheduler");
        framebuffer::sd("Task scheduler ready", BootStatus::Ok);
    } else {
        framebuffer::sd("Task scheduler disabled", BootStatus::Ej);
    }
    serial_println!("[PHASE] Task scheduler init done");
    
    
    debug::cpc(debug::CKJ_, "Disk I/O init");
    serial_println!("[PHASE] Disk init start");
    framebuffer::sd("Disk subsystem...", BootStatus::V);
    if BTN_ {
        
        let osa: alloc::vec::Vec<_> = pci::fjm().iter()
            .hi(|bc| bc.ajz == 0x01 && bc.adl == 0x08)
            .abn()
            .collect();
        
        if !osa.is_empty() {
            match nvme::init(&osa[0]) {
                Ok(()) => {
                    if let Some((model, msv, aw, cak)) = nvme::ani() {
                        let csm = (aw * cak as u64) / (1024 * 1024);
                        framebuffer::sd(
                            &alloc::format!("NVMe: {} ({} MB)", model, csm), BootStatus::Ok);
                    }
                }
                Err(aa) => {
                    crate::log_warn!("[DISK] NVMe init failed: {}", aa);
                }
            }
        }
        
        
        if !nvme::ky() {
            let mzo: alloc::vec::Vec<_> = pci::fjm().iter()
                .hi(|bc| bc.ml == 0x1AF4 && bc.mx == 0x1001)
                .abn()
                .collect();
            
            if !mzo.is_empty() {
                if let Err(aa) = virtio_blk::init(&mzo[0]) {
                    crate::log_warn!("[DISK] virtio-blk init failed: {}", aa);
                    disk::init();
                } else {
                    framebuffer::sd(&alloc::format!("virtio-blk: {} MB storage", 
                        (virtio_blk::aty() * 512) / (1024 * 1024)), BootStatus::Ok);
                }
            } else {
                
                disk::init();
            }
        }
        
        if disk::anl() || virtio_blk::ky() || nvme::ky() {
            framebuffer::sd("Disk driver ready", BootStatus::Ok);
        } else {
            framebuffer::sd("No disk detected", BootStatus::Ej);
        }
    } else {
        framebuffer::sd("Disk disabled", BootStatus::Ej);
    }
    serial_println!("[PHASE] Disk init done");
    
    
    serial_println!("[PHASE] Driver framework init start");
    if BTO_ {
        drivers::init();
        
        drivers::lvr();
        framebuffer::bir(12, "Driver framework");
        framebuffer::sd("Driver framework initialized", BootStatus::Ok);
        if drivers::oba() {
            framebuffer::sd("Persistent storage detected", BootStatus::Ok);
        }
    } else {
        framebuffer::sd("Driver framework disabled", BootStatus::Ej);
    }
    serial_println!("[PHASE] Driver framework init done");
    
    
    serial_println!("[PHASE] VirtIO GPU init start");
    framebuffer::sd("VirtIO GPU...", BootStatus::V);
    drivers::virtio_gpu::oel().bq();
    if drivers::virtio_gpu::anl() {
        framebuffer::sd(&alloc::format!("VirtIO GPU: {}", drivers::virtio_gpu::lea()), BootStatus::Ok);
    } else {
        framebuffer::sd("VirtIO GPU: not found (fallback framebuffer)", BootStatus::Ej);
    }
    serial_println!("[PHASE] VirtIO GPU init done");
    
    
    serial_println!("[PHASE] AMD GPU init start");
    framebuffer::sd("AMD GPU...", BootStatus::V);
    drivers::amdgpu::init();
    if drivers::amdgpu::clb() {
        framebuffer::sd(&alloc::format!("AMD GPU: {}", drivers::amdgpu::awz()), BootStatus::Ok);
    } else {
        framebuffer::sd("AMD GPU: not found (VM or non-AMD)", BootStatus::Ej);
    }
    serial_println!("[PHASE] AMD GPU init done");
    
    
    serial_println!("[PHASE] NVIDIA GPU init start");
    framebuffer::sd("NVIDIA GPU...", BootStatus::V);
    drivers::nvidia::init();
    if drivers::nvidia::clb() {
        framebuffer::sd(&alloc::format!("NVIDIA GPU: {}", drivers::nvidia::awz()), BootStatus::Ok);
    } else {
        framebuffer::sd("NVIDIA GPU: not found", BootStatus::Ej);
    }
    serial_println!("[PHASE] NVIDIA GPU init done");
    
    
    
    if !lgp {
        let dqt = framebuffer::BJ_.load(core::sync::atomic::Ordering::SeqCst);
        let gz = framebuffer::AB_.load(core::sync::atomic::Ordering::SeqCst) as usize;
        let kc = framebuffer::Z_.load(core::sync::atomic::Ordering::SeqCst) as usize;
        if !dqt.abq() && gz > 0 && kc > 0 {
            let srg = gz * kc * 4;
            let _ = memory::paging::vus(dqt as u64, srg);
        }
    } else {
        serial_println!("[PAT] Skipping framebuffer WC remap on VirtualBox");
    }
    
    
    debug::cpc(debug::CKM_, "Network init");
    serial_println!("[PHASE] Network init start");
    framebuffer::sd("Network subsystem...", BootStatus::V);
    if BTP_ {
        network::init();
        if network::anl() {
            
            let platform = network::tej();
            framebuffer::sd(&alloc::format!("Platform: {}", platform), BootStatus::V);
            
            
            let kpq = pci::ebq(pci::class::Qa);
            if !kpq.is_empty() {
                for ba in &kpq {
                    if drivers::net::lvo(ba) {
                        network::pxf();
                        let kro = if ba.ml == 0x1AF4 { "virtio-net" } 
                            else if ba.ml == 0x8086 { "e1000" }
                            else if ba.ml == 0x10EC { "rtl8139" }
                            else { "unknown" };
                        framebuffer::sd(&alloc::format!("Network driver: {}", kro), BootStatus::Ok);
                        break;
                    }
                }
            }

            
            if !drivers::net::bzy() && !crate::virtio_net::ky() {
                if let Some(ba) = kpq.iter().du(|bc| bc.ml == 0x1AF4) {
                    if let Err(aa) = crate::virtio_net::init(ba) {
                        crate::log_warn!("[NET] Legacy virtio-net init failed: {}", aa);
                    } else {
                        network::pxf();
                        framebuffer::sd("Network driver: virtio-net (legacy)", BootStatus::Ok);
                    }
                }
            }
            
            framebuffer::bir(14, "Network stack ready");
            framebuffer::sd("Network ready", BootStatus::Ok);
            
            
            netstack::dhcp::ay();
            netstack::ipv6::init();
            netstack::tcp::tts();
        }
    } else {
        framebuffer::sd("Network disabled", BootStatus::Ej);
    }
    serial_println!("[PHASE] Network init done");
    } 

    
    #[cfg(target_arch = "aarch64")]
    {
        serial_println!("[AARCH64] Skipping x86 peripherals (keyboard, RTC, mouse, PCI)");
        framebuffer::sd("Keyboard: N/A (serial)", BootStatus::Ej);
        framebuffer::sd("RTC: N/A", BootStatus::Ej);
        framebuffer::sd("PCI: N/A (ECAM not yet)", BootStatus::Ej);
        framebuffer::sd("Network: N/A", BootStatus::Ej);
    }

    
    
    
    debug::cpc(debug::CKS_, "VFS init");
    serial_println!("[PHASE] VFS init start");
    vfs::init();
    serial_println!("[PHASE] VFS init done");
    framebuffer::bir(15, "Virtual filesystem (VFS)");
    framebuffer::sd("Virtual filesystem ready", BootStatus::Ok);
    
    
    
    
    #[cfg(target_arch = "x86_64")]
    {
        serial_println!("[PHASE] Linux Subsystem init start");
        if let Some(upm) = CGZ_.fjo() {
            let lmo = upm.lmo();
            serial_println!("[TSL] Found {} boot modules", lmo.len());
            
            let mut abr: Option<&'static [u8]> = None;
            let mut izz: Option<&'static [u8]> = None;
            
            for apz in lmo {
                let dzn = apz.wx();
                let wx = core::str::jg(dzn).unwrap_or("unknown");
                let path = apz.path().zsy().unwrap_or("unknown");
                let ag = apz.ag();
                let aw = apz.aw() as usize;
                
                serial_println!("[TSL] Module: {} ({}), {} bytes at {:p}", 
                    wx, path, aw, ag);
                
                let f = unsafe { core::slice::anh(ag, aw) };
                
                if wx.contains("linux-kernel") || path.contains("bzImage") {
                    abr = Some(f);
                    serial_println!("[TSL] Linux kernel loaded: {} bytes", aw);
                } else if wx.contains("linux-initramfs") || path.contains("initramfs") {
                    izz = Some(f);
                    serial_println!("[TSL] Initramfs loaded: {} bytes", aw);
                } else if wx.contains("jarvis-brain") || path.contains("jarvis_pretrained") {
                    
                    serial_println!("[JARVIS] Boot module: {} bytes brain weights (deferred to RamFS)", aw);
                    unsafe { AXM_ = Some(f); }
                }
            }
            
            if let (Some(acf), Some(buz)) = (abr, izz) {
                hypervisor::linux_subsystem::bcu().piw(acf, buz);
                framebuffer::sd("Linux Subsystem (TSL) ready", BootStatus::Ok);
            } else {
                framebuffer::sd("Linux Subsystem (partial)", BootStatus::Ej);
            }
        } else {
            framebuffer::sd("Linux Subsystem (no modules)", BootStatus::Ej);
        }
        serial_println!("[PHASE] Linux Subsystem init done");
    }
    
    
    
    
    file_assoc::init();
    framebuffer::sd("File associations ready", BootStatus::Ok);
    
    
    
    
    debug::cpc(debug::CKO_, "Process manager init");
    serial_println!("[PHASE] Process manager init start");
    process::init();
    framebuffer::bir(17, "Process manager");
    framebuffer::sd("Process manager ready", BootStatus::Ok);
    serial_println!("[PHASE] Process manager init done");
    
    
    
    
    serial_println!("[PHASE] Auth system init");
    auth::init();
    auth::nhc();
    framebuffer::bir(18, "Authentication system");
    framebuffer::sd("Authentication ready", BootStatus::Ok);
    
    
    serial_println!("[PHASE] TTY/PTY init");
    tty::init();
    pty::init();
    framebuffer::sd("TTY/PTY subsystem ready", BootStatus::Ok);
    
    
    
    
    serial_println!("[PHASE] Init process start");
    init::ay();
    framebuffer::sd("Init process started (PID 1)", BootStatus::Ok);
    serial_println!("[PHASE] Init process done");
    
    
    
    
    serial_println!("[PHASE] RAM filesystem init");
    ramfs::init();
    framebuffer::bir(19, "RAM filesystem");
    
    ramfs::fh(|fs| {
        let _ = fs.ut("/tmp");
        let _ = fs.ut("/var");
        let _ = fs.ut("/home");
        let _ = fs.ut("/bin");
        let _ = fs.ut("/usr");
        let _ = fs.ut("/etc");
    });

    
    if let Some(kes) = unsafe { AXM_.take() } {
        serial_println!("[JARVIS] Copying {} KB brain weights to RamFS...", kes.len() / 1024);
        ramfs::fh(|fs| {
            let _ = fs.ut("/jarvis");
        });
        ramfs::fh(|fs| {
            let _ = fs.touch("/jarvis/weights.bin");
            match fs.ns("/jarvis/weights.bin", kes) {
                Ok(_) => serial_println!("[JARVIS] Brain weights cached to /jarvis/weights.bin ({} KB)", kes.len() / 1024),
                Err(_) => serial_println!("[JARVIS] WARNING: Failed to cache brain to RamFS"),
            }
        });
    }

    framebuffer::sd("RAM filesystem ready", BootStatus::Ok);

    
    auth::ugw();

    
    
    
    serial_println!("[PHASE] Persistence init");
    persistence::init();
    framebuffer::bir(20, "Persistence layer");
    framebuffer::sd("Persistence system ready", BootStatus::Ok);

    
    
    
    serial_println!("[PHASE] Web Sandbox init");
    sandbox::init();
    framebuffer::sd("Web Sandbox ready", BootStatus::Ok);

    
    serial_println!("[PHASE] Container daemon boot");
    sandbox::container::qqw();
    framebuffer::bir(21, "System ready!");
    framebuffer::sd("Container daemon ready", BootStatus::Ok);

    
    
    for _ in 0..5_000_000u64 { core::hint::hc(); }
    framebuffer::kuv();

    
    println!();
    framebuffer::krj(framebuffer::gia().1 as u32 * 16, framebuffer::B_);
    println!();
    h!(framebuffer::G_, "  System ready - TRust-OS v0.2.0");
    h!(framebuffer::B_, "  Type 'desktop' to launch the desktop, or use shell commands.");
    println!();

    
    if persistence::anl() {
        persistence::vng();
    }

    
    auth::mww();

    
    serial_println!("[BOOT] Running crypto self-tests...");
    tls13::crypto::peq();
    serial_println!("[BOOT] Crypto self-tests complete");

    
    debug::cpc(debug::CKQ_, "Shell ready — boot complete");
    serial_println!("Starting shell...");
    shell::vw();
}







fn hmj() -> ! {
    arch::hmj()
}

#[alloc_error_handler]
fn yeq(layout: Layout) -> ! {
    serial_println!("\n!!! ALLOC ERROR !!!");
    serial_println!("layout: size={}, align={}", layout.aw(), layout.align());
    if framebuffer::ky() {
        framebuffer::dbv(framebuffer::A_);
        println!("\n!!! ALLOC ERROR !!!");
        println!("layout: size={}, align={}", layout.aw(), layout.align());
    }
    hmj();
}


#[panic_handler]
fn panic(co: &Ciy) -> ! {
    
    debug::jjr(debug::BDL_);
    
    
    debug::vbm();
    
    
    serial_println!("\n!!! KERNEL PANIC !!!");
    serial_println!("{}", co);
    
    
    if framebuffer::ky() {
        framebuffer::dbv(framebuffer::A_);
        println!("\n!!! KERNEL PANIC !!!");
        println!("{}", co);
        framebuffer::dbv(0xFFAAAAAA);
        println!("Full crash dump sent to serial port (115200 8N1).");
        println!("Connect serial cable and reboot to capture output.");
        
        let qsd = debug::ivi(8);
        for line in &qsd {
            println!("{}", line);
        }
    }
    
    hmj();
}
