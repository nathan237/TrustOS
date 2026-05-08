






#![no_std]
#![no_main]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![feature(alloc_error_handler)]



#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_doc_comments)]
#![allow(unused_unsafe)]
#![allow(unused_comparisons)]
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
#![allow(dropping_references)]
#![allow(deprecated)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(nonstandard_style)]
#![allow(static_mut_refs)]
#![allow(asm_sub_register)]
#![allow(private_interfaces)]
#![allow(unexpected_cfgs)]
#![allow(mismatched_lifetime_syntaxes)]
#![allow(unused_must_use)]
#![allow(function_casts_as_integer)]
#![allow(unnecessary_transmutes)]
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


mod hwdiag;


mod marionet;


mod httpd;


mod trustpkg;


mod lab_mode;


mod wifi_analyzer;



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

use core::panic::PanicInfo;
use core::alloc::Layout;
use limine::request::{
    FramebufferRequest, MemoryMapRequest, HhdmRequest,
    RequestsStartMarker, RequestsEndMarker, ModuleRequest,
    RsdpRequest, SmpRequest, KernelAddressRequest, KernelFileRequest,
    StackSizeRequest,
};
use limine::BaseRevision;






#[used]
#[unsafe(link_section = ".requests_start_marker")]
static EOX_: RequestsStartMarker = RequestsStartMarker::new();


#[used]
#[unsafe(link_section = ".requests")]
static BNQ_: BaseRevision = BaseRevision::new();


#[used]
#[unsafe(link_section = ".requests")]
static BYP_: FramebufferRequest = FramebufferRequest::new();


#[used]
#[unsafe(link_section = ".requests")]
static CJQ_: MemoryMapRequest = MemoryMapRequest::new();


#[used]
#[unsafe(link_section = ".requests")]
static CCW_: HhdmRequest = HhdmRequest::new();


#[used]
#[unsafe(link_section = ".requests")]
static CKJ_: ModuleRequest = ModuleRequest::new();


static mut AZP_: Option<&'static [u8]> = None;


#[used]
#[unsafe(link_section = ".requests")]
static CSU_: RsdpRequest = RsdpRequest::new();


#[used]
#[unsafe(link_section = ".requests")]
static CXF_: SmpRequest = SmpRequest::new();


#[used]
#[unsafe(link_section = ".requests")]
static CGD_: KernelAddressRequest = KernelAddressRequest::new();


#[used]
#[unsafe(link_section = ".requests")]
static CGE_: KernelFileRequest = KernelFileRequest::new();




#[used]
#[unsafe(link_section = ".requests")]
static EKK_: StackSizeRequest = StackSizeRequest::new().with_size(512 * 1024);


#[used]
#[unsafe(link_section = ".requests_end_marker")]
static EOR_: RequestsEndMarker = RequestsEndMarker::new();






#[used]
#[link_section = ".text"]
static DVZ_: unsafe extern "C" fn() -> ! = mvw;









#[no_mangle]
#[link_section = ".text.kmain"]
pub unsafe extern "C" fn mvw() -> ! {
    
    if !BNQ_.is_supported() {
        dre();
    }

    
    
    
    #[cfg(target_arch = "aarch64")]
    {
        if let Some(ka_resp) = CGD_.get_response() {
            crate::arch::platform::boot::oql(
                ka_resp.virtual_base(),
                ka_resp.physical_base(),
            );
        }
        
    }

    
    if let Some(kf_resp) = CGE_.get_response() {
        let file = kf_resp.file();
        let ptr = file.addr();
        let size = file.size() as usize;
        if size > 0 {
            unsafe { jarvis::pxe_replicator::oej(ptr, size); }
        }
    }

    
    serial::init();
    debug::awn(debug::CNY_, "Serial port initialized");
    serial_println!("T-RustOs Kernel v0.2.0");
    serial_println!("Limine protocol supported");

    
    #[cfg(target_arch = "aarch64")]
    {
        let current_el: u64;
        core::arch::asm!("mrs {}, CurrentEL", out(reg) current_el);
        let el = (current_el >> 2) & 3;
        serial_println!("[AARCH64] CurrentEL = EL{}", el);
        if el == 2 {
            serial_println!("[EL2] *** Hypervisor mode detected! ***");
            serial_println!("[EL2] ARM EL2 MMIO Spy available — use 'hv el2' in shell");
        } else if el == 1 {
            serial_println!("[EL1] Standard kernel mode (no hypervisor)");
        }
    }

    
    if let Some(fb_response) = BYP_.get_response() {
        if let Some(fb) = fb_response.framebuffers().next() {
            framebuffer::init(
                fb.addr(),
                fb.width(),
                fb.height(),
                fb.pitch(),
                fb.bpp(),
            );
            serial_println!("Framebuffer: {}x{} @ {:p}", fb.width(), fb.height(), fb.addr());
        }
    }
    
    
    framebuffer::gcn();
    
    use framebuffer::BootStatus;

    
    serial_println!("Initializing memory management...");
    
    let mut gao = false;
    
    if let Some(mmap_response) = CJQ_.get_response() {
        let hhdm_offset = CCW_.get_response()
            .map(|r| r.offset())
            .unwrap_or(0);
        
        serial_println!("HHDM offset: {:#x}", hhdm_offset);
        serial_println!("Memory map entries: {}", mmap_response.entries().len());
        
        
        let mut fec: Option<u64> = None;
        let mut bhk: u64 = 0;
        let mut ecg: u64 = 0;
        
        
        for entry in mmap_response.entries() {
            let kind = match entry.entry_type {
                limine::memory_map::EntryType::USABLE => "USABLE",
                limine::memory_map::EntryType::RESERVED => "RESERVED",
                limine::memory_map::EntryType::ACPI_RECLAIMABLE => "ACPI_RECLAIM",
                limine::memory_map::EntryType::ACPI_NVS => "ACPI_NVS",
                limine::memory_map::EntryType::BAD_MEMORY => "BAD",
                limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE => "BOOTLOADER",
                limine::memory_map::EntryType::EXECUTABLE_AND_MODULES => "KERNEL",
                limine::memory_map::EntryType::FRAMEBUFFER => "FRAMEBUFFER",
                _ => "UNKNOWN",
            };
            serial_println!("  {:#012x} - {:#012x} ({:12} bytes) {}", 
                entry.base, 
                entry.base + entry.length,
                entry.length,
                kind
            );
            
            
            let ecy = match entry.entry_type {
                limine::memory_map::EntryType::USABLE => 0u8,
                limine::memory_map::EntryType::RESERVED => 1,
                limine::memory_map::EntryType::ACPI_RECLAIMABLE => 2,
                limine::memory_map::EntryType::ACPI_NVS => 3,
                limine::memory_map::EntryType::BAD_MEMORY => 4,
                limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE => 5,
                limine::memory_map::EntryType::EXECUTABLE_AND_MODULES => 6,
                limine::memory_map::EntryType::FRAMEBUFFER => 7,
                _ => 0xFF,
            };
            memory::oxu(entry.base, entry.length, ecy);
            
            
            if entry.entry_type == limine::memory_map::EntryType::EXECUTABLE_AND_MODULES
                || entry.entry_type == limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE
            {
                let end = entry.base + entry.length;
                if end > bhk {
                    bhk = end;
                }
            }
            
            if entry.entry_type == limine::memory_map::EntryType::USABLE
                || entry.entry_type == limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE
                || entry.entry_type == limine::memory_map::EntryType::ACPI_RECLAIMABLE
            {
                ecg += entry.length;
            }
        }

        
        memory::opo(ecg);
        serial_println!("[MEM] Total physical memory: {} MB", ecg / 1024 / 1024);

        
        let bzc = memory::kwm(ecg);
        serial_println!("[HEAP] Dynamic size: {} MB (25% of {} MB RAM)", 
            bzc / 1024 / 1024, ecg / 1024 / 1024);

        
        let align_up = |addr: u64, align: u64| -> u64 {
            if addr % align == 0 { addr } else { addr + (align - (addr % align)) }
        };

        
        let inr = align_up(core::cmp::max(0x100000, bhk), 0x1000);
        for entry in mmap_response.entries() {
            if entry.entry_type != limine::memory_map::EntryType::USABLE {
                continue;
            }
            let gqx = entry.base;
            let cdf = entry.base.saturating_add(entry.length);
            if cdf <= inr {
                continue;
            }
            let heap_start = core::cmp::max(gqx, inr);
            if cdf >= heap_start + bzc as u64 {
                fec = Some(heap_start);
                break;
            }
        }

        
        if fec.is_none() {
            let mut cgg: u64 = 0;
            let mut atb: u64 = 0;
            for entry in mmap_response.entries() {
                if entry.entry_type != limine::memory_map::EntryType::USABLE {
                    continue;
                }
                if entry.length > atb {
                    atb = entry.length;
                    cgg = entry.base;
                }
            }

            if atb >= bzc as u64 {
                let mut heap_start = align_up(cgg, 0x1000);
                if heap_start < 0x100000 {
                    heap_start = align_up(0x100000, 0x1000);
                }
                if cgg.saturating_add(atb) >= heap_start + bzc as u64 {
                    fec = Some(heap_start);
                }
            }
        }
        
        
        if let Some(bgx) = fec {
            serial_println!("[HEAP] Using mmap region at phys {:#x}, size {} MB", bgx, bzc / 1024 / 1024);
            
            memory::gcr(hhdm_offset, bgx, bzc);
            gao = true;
            serial_println!("[HEAP] Initialized: free={} KB", memory::heap::free() / 1024);
            
            
            
            let cew: alloc::vec::Vec<memory::frame::Mw> = mmap_response.entries()
                .iter()
                .filter(|e| e.entry_type == limine::memory_map::EntryType::USABLE)
                .map(|e| memory::frame::Mw { base: e.base, length: e.length })
                .collect();
            memory::frame::init(&cew, bgx, bzc as u64);
        } else {
            
            
            let mut cgg: u64 = 0;
            let mut atb: u64 = 0;
            for entry in mmap_response.entries() {
                if entry.entry_type != limine::memory_map::EntryType::USABLE && entry.length > atb {
                    
                }
                if entry.entry_type == limine::memory_map::EntryType::USABLE && entry.length > atb {
                    atb = entry.length;
                    cgg = entry.base;
                }
            }
            
            if atb >= memory::VE_ as u64 {
                let epb = align_up(core::cmp::max(cgg, 0x100000), 0x1000);
                let pqc = cgg.saturating_add(atb);
                let drj = core::cmp::min(
                    (pqc - epb) as usize,
                    bzc,
                );
                let drj = (drj / 4096) * 4096; 
                
                serial_println!("[HEAP] Fallback: using {:#x} size {} MB", epb, drj / 1024 / 1024);
                memory::gcr(hhdm_offset, epb, drj);
                gao = true;
                serial_println!("[HEAP] Initialized: free={} KB", memory::heap::free() / 1024);
                
                
                let cew: alloc::vec::Vec<memory::frame::Mw> = mmap_response.entries()
                    .iter()
                    .filter(|e| e.entry_type == limine::memory_map::EntryType::USABLE)
                    .map(|e| memory::frame::Mw { base: e.base, length: e.length })
                    .collect();
                memory::frame::init(&cew, epb, drj as u64);
            } else {
                serial_println!("[HEAP] ERROR: No usable region found for heap!");
            }
        }
    }
    
    if !gao {
        
        serial_println!("[HEAP] Using fallback init");
        memory::init();
    }
    
    
    serial_println!("[FB] Initializing scrollback buffer...");
    framebuffer::gcq();
    
    
    signature::mph();
    
    
    signature::mpe(&[0x54, 0x72, 0x75, 0x73, 0x74, 0x4f, 0x53]); 
    
    
    framebuffer::afw(0, "Memory management initialized");
    framebuffer::hm("Memory management initialized", BootStatus::Ok);

    
    #[cfg(target_arch = "x86_64")]
    {
        debug::awn(debug::CNT_, "GDT init");
        serial_println!("Initializing GDT with Ring 0/3 support...");
        gdt::init();
        framebuffer::afw(1, "GDT initialized (Ring 0/3)");
        framebuffer::hm("GDT initialized (Ring 0/3)", BootStatus::Ok);
    }
    
    
    debug::awn(debug::CNU_, "IDT/interrupts init");
    serial_println!("Initializing early interrupts...");
    interrupts::init();
    framebuffer::afw(2, "Interrupts initialized");
    framebuffer::hm("Interrupts (early)", BootStatus::Ok);
    
    
    #[cfg(target_arch = "x86_64")]
    {
        
        debug::awn(debug::CNR_, "CPU detection");
        serial_println!("Detecting CPU capabilities...");
        cpu::init();
        framebuffer::afw(3, "CPU capabilities detected");
        framebuffer::hm("CPU capabilities detected", BootStatus::Ok);
        
        
        debug::awn(debug::CNP_, "ACPI tables parsing");
        serial_println!("Parsing ACPI tables...");
        if let Some(rsdp_response) = CSU_.get_response() {
            let ddr = rsdp_response.address();
            serial_println!("[DEBUG] RSDP pointer from Limine: {:#x}", ddr as usize);
            if acpi::igo(ddr as u64) {
                if let Some(info) = acpi::rk() {
                    framebuffer::hm(&alloc::format!(
                        "ACPI: {} CPUs, {} I/O APICs", 
                        info.cpu_count, info.io_apics.len()
                    ), BootStatus::Ok);
                }
            } else {
                framebuffer::hm("ACPI init failed", BootStatus::Skip);
            }
        } else {
            framebuffer::hm("No RSDP from bootloader", BootStatus::Skip);
        }
        
        
        debug::awn(debug::CNQ_, "APIC init");
        serial_println!("Initializing APIC...");
        if apic::init() {
            framebuffer::hm("APIC initialized (LAPIC + IOAPIC)", BootStatus::Ok);
        } else {
            serial_println!("[APIC] Not available, staying on legacy PIC");
            framebuffer::hm("APIC not available (legacy PIC)", BootStatus::Skip);
        }

        
        if acpi::hpet::init() {
            framebuffer::hm("HPET initialized", BootStatus::Ok);
        } else {
            framebuffer::hm("HPET not available", BootStatus::Skip);
        }

        
        debug::awn(debug::COA_, "SMP multi-core init");
        serial_println!("Initializing SMP...");
        cpu::smp::init();
        
        if let Some(smp_response) = CXF_.get_response() {
            let cpu_count = smp_response.cpus().len();
            serial_println!("[SMP] Found {} CPUs via Limine", cpu_count);
            
            for cpu in smp_response.cpus().iter() {
                if cpu.id != 0 {
                    serial_println!("[SMP] Starting AP {} (LAPIC ID: {})", cpu.id, cpu.lapic_id);
                    cpu.goto_address.write(cpu::smp::hfk);
                }
            }
            
            let mut ready_count = 1u32;
            for _ in 0..1000 {
                ready_count = cpu::smp::ail();
                if ready_count >= cpu_count as u32 { break; }
                for _ in 0..10000 { core::hint::spin_loop(); }
            }
            
            serial_println!("[SMP] {} of {} CPUs online", ready_count, cpu_count);
            cpu::smp::jfb(cpu_count as u32);
            framebuffer::afw(4, "SMP multi-core active");
            framebuffer::hm(&alloc::format!("SMP: {} cores active", ready_count), BootStatus::Ok);
        } else {
            serial_println!("[SMP] No SMP response from bootloader");
            framebuffer::hm("SMP: single core", BootStatus::Ok);
        }
    }
    
    
    serial_println!("Initializing paging subsystem...");
    memory::paging::init();  
    framebuffer::afw(5, "Paging & memory protection");
    framebuffer::hm("Paging initialized (NX enabled)", BootStatus::Ok);
    
    
    #[cfg(target_arch = "x86_64")]
    let gef = acpi::rk()
        .map(|info| info.oem_id.trim().eq_ignore_ascii_case("VBOX"))
        .unwrap_or(false);
    #[cfg(not(target_arch = "x86_64"))]
    let gef = false;
    if gef {
        serial_println!("[PAT] Skipping Write-Combining on VirtualBox (VMSVGA compat)");
    } else {
        memory::paging::oqn();
    }
    
    
    #[cfg(target_arch = "x86_64")]
    {
        serial_println!("Initializing userland support...");
        userland::igu();
        userland::init();
        framebuffer::hm("Userland support ready", BootStatus::Ok);
    }
    
    
    serial_println!("Initializing thread subsystem...");
    thread::init();
    framebuffer::afw(6, "Thread subsystem ready");
    framebuffer::hm("Thread subsystem ready", BootStatus::Ok);
    
    
    serial_println!("Initializing security subsystem...");
    
    framebuffer::hm("Security (basic)", BootStatus::Ok);

    
    #[cfg(target_arch = "x86_64")]
    {
    
    keyboard::mpg();
    serial_println!("Keyboard driver ready");
    framebuffer::afw(7, "Keyboard & input devices");
    framebuffer::hm("Keyboard ready", BootStatus::Ok);

    
    
    
    
    const BWN_: bool = true;
    if BWN_ {
        serial_println!("[RTC] init start");
        if rtc::pnw() {
            framebuffer::hm("RTC initialized", BootStatus::Ok);
        } else {
            framebuffer::hm("RTC skipped", BootStatus::Skip);
        }
        serial_println!("[RTC] init done");
    } else {
        framebuffer::hm("RTC disabled", BootStatus::Skip);
    }

    
    rng::init();
    framebuffer::hm("RNG (CSPRNG)", BootStatus::Ok);
    
    
    mouse::init();
    let (fb_width, fb_height) = framebuffer::kv();
    mouse::set_screen_size(fb_width, fb_height);
    framebuffer::afw(8, "Mouse & touch input");
    framebuffer::hm("Mouse initialized", BootStatus::Ok);
    
    
    touch::init();
    touch::set_screen_size(fb_width, fb_height);
    framebuffer::hm("Touch input ready", BootStatus::Ok);
    
    
    const BWM_: bool = true;
    const BWO_: bool = true;
    const BWJ_: bool = true;   
    const BWK_: bool = true;
    const BWL_: bool = true;

    
    debug::awn(debug::CNW_, "PCI bus enumeration");
    serial_println!("[PHASE] PCI init start");
    framebuffer::hm("PCI bus scanning...", BootStatus::Info);
    if BWM_ {
        pci::init();
        framebuffer::afw(9, "PCI bus enumeration");
        framebuffer::hm("PCI bus scanned", BootStatus::Ok);
    } else {
        framebuffer::hm("PCI disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] PCI init done");
    
    
    serial_println!("[PHASE] Task scheduler init start");
    if BWO_ {
        task::init();
        scheduler::init();
        framebuffer::afw(10, "Task scheduler");
        framebuffer::hm("Task scheduler ready", BootStatus::Ok);
    } else {
        framebuffer::hm("Task scheduler disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Task scheduler init done");
    
    
    debug::awn(debug::CNS_, "Disk I/O init");
    serial_println!("[PHASE] Disk init start");
    framebuffer::hm("Disk subsystem...", BootStatus::Info);
    if BWJ_ {
        
        let irx: alloc::vec::Vec<_> = pci::aqs().iter()
            .filter(|d| d.class_code == 0x01 && d.subclass == 0x08)
            .cloned()
            .collect();
        
        if !irx.is_empty() {
            match nvme::init(&irx[0]) {
                Ok(()) => {
                    if let Some((model, _serial, size, aol)) = nvme::rk() {
                        let aop = (size * aol as u64) / (1024 * 1024);
                        framebuffer::hm(
                            &alloc::format!("NVMe: {} ({} MB)", model, aop), BootStatus::Ok);
                    }
                }
                Err(e) => {
                    crate::log_warn!("[DISK] NVMe init failed: {}", e);
                }
            }
        }
        
        
        if !nvme::is_initialized() {
            let hie: alloc::vec::Vec<_> = pci::aqs().iter()
                .filter(|d| d.vendor_id == 0x1AF4 && d.device_id == 0x1001)
                .cloned()
                .collect();
            
            if !hie.is_empty() {
                if let Err(e) = virtio_blk::init(&hie[0]) {
                    crate::log_warn!("[DISK] virtio-blk init failed: {}", e);
                    disk::init();
                } else {
                    framebuffer::hm(&alloc::format!("virtio-blk: {} MB storage", 
                        (virtio_blk::capacity() * 512) / (1024 * 1024)), BootStatus::Ok);
                }
            } else {
                
                disk::init();
            }
        }
        
        if disk::sw() || virtio_blk::is_initialized() || nvme::is_initialized() {
            framebuffer::hm("Disk driver ready", BootStatus::Ok);
        } else {
            framebuffer::hm("No disk detected", BootStatus::Skip);
        }
    } else {
        framebuffer::hm("Disk disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Disk init done");
    
    
    serial_println!("[PHASE] Driver framework init start");
    if BWK_ {
        drivers::init();
        
        drivers::gom();
        framebuffer::afw(12, "Driver framework");
        framebuffer::hm("Driver framework initialized", BootStatus::Ok);
        if drivers::ied() {
            framebuffer::hm("Persistent storage detected", BootStatus::Ok);
        }
    } else {
        framebuffer::hm("Driver framework disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Driver framework init done");
    
    
    serial_println!("[PHASE] VirtIO GPU init start");
    framebuffer::hm("VirtIO GPU...", BootStatus::Info);
    drivers::virtio_gpu::igs().ok();
    if drivers::virtio_gpu::sw() {
        framebuffer::hm(&alloc::format!("VirtIO GPU: {}", drivers::virtio_gpu::gcl()), BootStatus::Ok);
    } else {
        framebuffer::hm("VirtIO GPU: not found (fallback framebuffer)", BootStatus::Skip);
    }
    serial_println!("[PHASE] VirtIO GPU init done");
    
    
    serial_println!("[PHASE] AMD GPU init start");
    framebuffer::hm("AMD GPU...", BootStatus::Info);
    drivers::amdgpu::init();
    if drivers::amdgpu::aud() {
        framebuffer::hm(&alloc::format!("AMD GPU: {}", drivers::amdgpu::summary()), BootStatus::Ok);
    } else {
        framebuffer::hm("AMD GPU: not found (VM or non-AMD)", BootStatus::Skip);
    }
    serial_println!("[PHASE] AMD GPU init done");
    
    
    serial_println!("[PHASE] NVIDIA GPU init start");
    framebuffer::hm("NVIDIA GPU...", BootStatus::Info);
    drivers::nvidia::init();
    if drivers::nvidia::aud() {
        framebuffer::hm(&alloc::format!("NVIDIA GPU: {}", drivers::nvidia::summary()), BootStatus::Ok);
    } else {
        framebuffer::hm("NVIDIA GPU: not found", BootStatus::Skip);
    }
    serial_println!("[PHASE] NVIDIA GPU init done");
    
    
    
    if !gef {
        let bme = framebuffer::BL_.load(core::sync::atomic::Ordering::SeqCst);
        let fb_w = framebuffer::X_.load(core::sync::atomic::Ordering::SeqCst) as usize;
        let fb_h = framebuffer::W_.load(core::sync::atomic::Ordering::SeqCst) as usize;
        if !bme.is_null() && fb_w > 0 && fb_h > 0 {
            let lul = fb_w * fb_h * 4;
            let _ = memory::paging::oeu(bme as u64, lul);
        }
    } else {
        serial_println!("[PAT] Skipping framebuffer WC remap on VirtualBox");
    }
    
    
    debug::awn(debug::CNV_, "Network init");
    serial_println!("[PHASE] Network init start");
    framebuffer::hm("Network subsystem...", BootStatus::Info);
    if BWL_ {
        network::init();
        if network::sw() {
            
            let platform = network::fyv();
            framebuffer::hm(&alloc::format!("Platform: {}", platform), BootStatus::Info);
            
            
            
            
            framebuffer::hm("PCI scan...", BootStatus::Info);
            let cih = pci::bsp(pci::class::Gr);
            framebuffer::hm(&alloc::format!("Found {} network devices", cih.len()), BootStatus::Info);
            
            
            
            for s in &cih {
                if s.vendor_id == 0x8086 
                    && (s.subclass == 0x80 || s.class_code == 0x0D 
                        || drivers::net::iwl4965::AFI_.contains(&s.device_id))
                {
                    framebuffer::hm(
                        &alloc::format!("WiFi detected: {:04X}:{:04X} (deferred)", s.vendor_id, s.device_id),
                        BootStatus::Info);
                    
                    drivers::net::wifi::oot(s.bus, s.device, s.function);
                }
            }
            
            
            println!("  Ethernet probe starting...");
            framebuffer::hm("Ethernet probe...", BootStatus::Info);
            for s in &cih {
                if drivers::net::goi(s) {
                    network::jpe();
                    let fte = if s.vendor_id == 0x1AF4 { "virtio-net" } 
                        else if s.vendor_id == 0x8086 { "e1000" }
                        else if s.vendor_id == 0x10EC { "rtl8139" }
                        else { "unknown" };
                    framebuffer::hm(&alloc::format!("Network driver: {}", fte), BootStatus::Ok);
                    break;
                }
            }

            
            if !drivers::net::aoh() && !crate::virtio_net::is_initialized() {
                if let Some(s) = cih.iter().find(|d| d.vendor_id == 0x1AF4) {
                    if let Err(e) = crate::virtio_net::init(s) {
                        crate::log_warn!("[NET] Legacy virtio-net init failed: {}", e);
                    } else {
                        network::jpe();
                        framebuffer::hm("Network driver: virtio-net (legacy)", BootStatus::Ok);
                    }
                }
            }
            
            framebuffer::afw(14, "Network stack ready");
            framebuffer::hm("Network ready", BootStatus::Ok);
            
            
            netstack::dhcp::start();
            netstack::ipv6::init();
            netstack::tcp::mpj();
        }
    } else {
        framebuffer::hm("Network disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Network init done");
    } 

    
    #[cfg(target_arch = "aarch64")]
    {
        serial_println!("[AARCH64] Skipping x86 peripherals (keyboard, RTC, mouse, PCI)");
        framebuffer::hm("Keyboard: N/A (serial)", BootStatus::Skip);
        framebuffer::hm("RTC: N/A", BootStatus::Skip);
        framebuffer::hm("PCI: N/A (ECAM not yet)", BootStatus::Skip);
        framebuffer::hm("Network: N/A", BootStatus::Skip);
    }

    
    
    
    debug::awn(debug::COB_, "VFS init");
    serial_println!("[PHASE] VFS init start");
    vfs::init();
    serial_println!("[PHASE] VFS init done");
    framebuffer::afw(15, "Virtual filesystem (VFS)");
    framebuffer::hm("Virtual filesystem ready", BootStatus::Ok);
    
    
    
    
    #[cfg(target_arch = "x86_64")]
    {
        serial_println!("[PHASE] Linux Subsystem init start");
        if let Some(module_response) = CKJ_.get_response() {
            let modules = module_response.modules();
            serial_println!("[TSL] Found {} boot modules", modules.len());
            
            let mut kernel_data: Option<&'static [u8]> = None;
            let mut eqn: Option<&'static [u8]> = None;
            
            for vn in modules {
                let bqz = vn.cmdline();
                let cmdline = core::str::from_utf8(bqz).unwrap_or("unknown");
                let path = vn.path().to_str().unwrap_or("unknown");
                let addr = vn.addr();
                let size = vn.size() as usize;
                
                serial_println!("[TSL] Module: {} ({}), {} bytes at {:p}", 
                    cmdline, path, size, addr);
                
                let data = unsafe { core::slice::from_raw_parts(addr, size) };
                
                if cmdline.contains("linux-kernel") || path.contains("bzImage") {
                    kernel_data = Some(data);
                    serial_println!("[TSL] Linux kernel loaded: {} bytes", size);
                } else if cmdline.contains("linux-initramfs") || path.contains("initramfs") {
                    eqn = Some(data);
                    serial_println!("[TSL] Initramfs loaded: {} bytes", size);
                } else if cmdline.contains("jarvis-brain") || path.contains("jarvis_pretrained") {
                    
                    serial_println!("[JARVIS] Boot module: {} bytes brain weights (deferred to RamFS)", size);
                    unsafe { AZP_ = Some(data); }
                } else if cmdline.contains("iwlwifi") || path.contains("iwlwifi") || path.contains(".ucode") {
                    
                    serial_println!("[WIFI] Firmware module: {} bytes at {:p}", size, addr);
                    let mau = alloc::vec::Vec::from(data);
                    drivers::net::iwl4965::oox(&mau);
                }
            }
            
            if let (Some(ny), Some(initramfs)) = (kernel_data, eqn) {
                hypervisor::linux_subsystem::acs().set_embedded_images(ny, initramfs);
                framebuffer::hm("Linux Subsystem (TSL) ready", BootStatus::Ok);
            } else {
                framebuffer::hm("Linux Subsystem (partial)", BootStatus::Skip);
            }
        } else {
            framebuffer::hm("Linux Subsystem (no modules)", BootStatus::Skip);
        }
        serial_println!("[PHASE] Linux Subsystem init done");
    }
    
    
    
    
    file_assoc::init();
    framebuffer::hm("File associations ready", BootStatus::Ok);
    
    
    
    
    debug::awn(debug::CNX_, "Process manager init");
    serial_println!("[PHASE] Process manager init start");
    process::init();
    framebuffer::afw(17, "Process manager");
    framebuffer::hm("Process manager ready", BootStatus::Ok);
    serial_println!("[PHASE] Process manager init done");
    
    
    
    
    serial_println!("[PHASE] Auth system init");
    auth::init();
    auth::hoo();
    framebuffer::afw(18, "Authentication system");
    framebuffer::hm("Authentication ready", BootStatus::Ok);
    
    
    serial_println!("[PHASE] TTY/PTY init");
    tty::init();
    pty::init();
    framebuffer::hm("TTY/PTY subsystem ready", BootStatus::Ok);
    
    
    
    
    serial_println!("[PHASE] Init process start");
    init::start();
    framebuffer::hm("Init process started (PID 1)", BootStatus::Ok);
    serial_println!("[PHASE] Init process done");
    
    
    
    
    serial_println!("[PHASE] RAM filesystem init");
    ramfs::init();
    framebuffer::afw(19, "RAM filesystem");
    
    ramfs::bh(|fs| {
        let _ = fs.mkdir("/tmp");
        let _ = fs.mkdir("/var");
        let _ = fs.mkdir("/home");
        let _ = fs.mkdir("/bin");
        let _ = fs.mkdir("/usr");
        let _ = fs.mkdir("/etc");
    });

    
    if let Some(brain_data) = unsafe { AZP_.take() } {
        serial_println!("[JARVIS] Copying {} KB brain weights to RamFS...", brain_data.len() / 1024);
        ramfs::bh(|fs| {
            let _ = fs.mkdir("/jarvis");
        });
        ramfs::bh(|fs| {
            let _ = fs.touch("/jarvis/weights.bin");
            match fs.write_file("/jarvis/weights.bin", brain_data) {
                Ok(_) => serial_println!("[JARVIS] Brain weights cached to /jarvis/weights.bin ({} KB)", brain_data.len() / 1024),
                Err(_) => serial_println!("[JARVIS] WARNING: Failed to cache brain to RamFS"),
            }
        });
    }

    framebuffer::hm("RAM filesystem ready", BootStatus::Ok);

    
    auth::nab();

    
    
    
    serial_println!("[PHASE] Persistence init");
    persistence::init();
    framebuffer::afw(20, "Persistence layer");
    framebuffer::hm("Persistence system ready", BootStatus::Ok);

    
    
    
    serial_println!("[PHASE] Web Sandbox init");
    sandbox::init();
    framebuffer::hm("Web Sandbox ready", BootStatus::Ok);

    
    serial_println!("[PHASE] Container daemon boot");
    sandbox::container::kde();
    framebuffer::afw(21, "System ready!");
    framebuffer::hm("Container daemon ready", BootStatus::Ok);

    
    
    for _ in 0..5_000_000u64 { core::hint::spin_loop(); }
    framebuffer::fvz();

    
    println!();
    framebuffer::ftb(framebuffer::cyk().1 as u32 * 16, framebuffer::B_);
    println!();
    n!(framebuffer::G_, "  System ready - TRust-OS v0.2.0");
    n!(framebuffer::B_, "  Type 'desktop' to launch the desktop, or use shell commands.");
    println!();

    
    if persistence::sw() {
        persistence::nyv();
    }

    
    auth::hgb();

    
    marionet::autodump::kdd();

    
    serial_println!("[BOOT] Running crypto self-tests...");
    tls13::crypto::jbv();
    serial_println!("[BOOT] Crypto self-tests complete");

    
    debug::awn(debug::CNZ_, "Shell ready — boot complete");
    serial_println!("Starting shell...");
    shell::run();
}







fn dre() -> ! {
    arch::dre()
}

#[alloc_error_handler]
fn pxy(layout: Layout) -> ! {
    serial_println!("\n!!! ALLOC ERROR !!!");
    serial_println!("layout: size={}, align={}", layout.size(), layout.align());
    if framebuffer::is_initialized() {
        framebuffer::bdr(framebuffer::A_);
        println!("\n!!! ALLOC ERROR !!!");
        println!("layout: size={}, align={}", layout.size(), layout.align());
    }
    dre();
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    
    debug::ewx(debug::BFO_);
    
    
    debug::npv();
    
    
    serial_println!("\n!!! KERNEL PANIC !!!");
    serial_println!("{}", info);
    
    
    if framebuffer::is_initialized() {
        framebuffer::bdr(framebuffer::A_);
        println!("\n!!! KERNEL PANIC !!!");
        println!("{}", info);
        framebuffer::bdr(0xFFAAAAAA);
        println!("Full crash dump sent to serial port (115200 8N1).");
        println!("Connect serial cable and reboot to capture output.");
        
        let ked = debug::enf(8);
        for line in &ked {
            println!("{}", line);
        }
    }
    
    dre();
}
