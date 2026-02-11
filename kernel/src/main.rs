//! T-RustOs Kernel
//! 
//! Microkernel architecture with capability-based security.
//! Boots via Limine bootloader on UEFI systems.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
extern crate alloc;

// Core modules
mod serial;
mod logger;
mod framebuffer;
mod keyboard;
mod shell;
mod ramfs;
mod rtc;
mod mouse;
mod task;
mod desktop;
mod disk;
mod network;
mod pci;
mod virtio;
mod virtio_net;
mod virtio_blk;
mod drivers;
mod netstack;
mod time;
mod rng;
mod file_assoc;
mod ui;
mod apps;
mod graphics;
mod icons;
mod browser;
mod game3d; // 3D raycasting FPS game engine
mod cosmic; // COSMIC-style UI framework (libcosmic-inspired)
mod compositor; // Multi-layer compositor for flicker-free rendering
mod holovolume; // Volumetric ASCII raymarcher - 3D holographic desktop
mod matrix_fast; // Ultra-optimized Matrix rain with Braille sub-pixels
mod formula3d;   // Tsoding-inspired wireframe 3D renderer (perspective projection)
mod gpu_emu;      // Virtual GPU - CPU cores emulating GPU parallelism
// TLS 1.3 pure Rust implementation (no C dependencies)
mod tls13;
// CPU hardware exploitation (TSC, AES-NI, SIMD, SMP)
mod cpu;
// ACPI tables parsing (MADT, FADT, MCFG, HPET)
mod acpi;

// New OS infrastructure
mod vfs;
mod process;
mod elf;
mod exec;
mod init;
mod gdt;
mod userland;
mod thread;
mod auth;

// Linux distribution manager
mod distro;

// Linux Subsystem (Alpine Linux environment)
mod linux;

// Linux Binary Compatibility Layer (execute real Linux binaries)
mod linux_compat;

// Compression utilities for tar.gz
mod compression;

// Persistence (save downloads to disk)
mod persistence;

// Wayland Compositor (native display server)
mod wayland;

// Binary-to-Rust transpiler (analyze and convert Linux binaries)
mod transpiler;

// TrustLang — integrated programming language (Rust-like, bytecode VM)
mod trustlang;

// TrustVideo — custom video codec & player (delta+RLE, no external APIs)
mod video;

// Subsystems
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
mod hypervisor;
mod rasterizer;
mod model_editor;

// Developer tools (profiler, dmesg, memdbg, devpanel, peek/poke)
mod devtools;

// Kernel signature & proof of authorship
mod signature;

// Synchronization primitives (Redox-inspired)
mod sync;
// POSIX signals
mod signals;
// Process tracing (debugging)
mod ptrace;
// Safe user/kernel memory copy
mod usercopy;

use core::panic::PanicInfo;
use core::alloc::Layout;
use limine::request::{
    FramebufferRequest, MemoryMapRequest, HhdmRequest,
    RequestsStartMarker, RequestsEndMarker, ModuleRequest,
    RsdpRequest, SmpRequest
};
use limine::BaseRevision;

// ============================================================================
// Limine Protocol Requests
// ============================================================================

/// Limine requests start marker
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

/// Limine base revision - ensures protocol compatibility
#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

/// Request framebuffer from Limine for graphics output
#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// Request memory map from Limine for memory management
#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

/// Request higher half direct map offset for physical memory access
#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Request modules from Limine (Linux kernel and initramfs)
#[used]
#[unsafe(link_section = ".requests")]
static MODULE_REQUEST: ModuleRequest = ModuleRequest::new();

/// Request RSDP (ACPI tables) from Limine
#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

/// Request SMP (multi-core) support from Limine
#[used]
#[unsafe(link_section = ".requests")]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

/// Limine requests end marker
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

// ============================================================================
// Kernel Entry Point
// ============================================================================

// Force the linker to include kmain by creating a static reference to it
#[used]
#[link_section = ".text"]
static KMAIN_REFERENCE: unsafe extern "C" fn() -> ! = kmain;

/// Kernel entry point - called by Limine bootloader
/// 
/// Initializes all kernel subsystems in the correct order:
/// 1. Serial port (for early debug output)
/// 2. Framebuffer console (for screen output)
/// 3. Memory management (heap allocator)
/// 4. Interrupt handling (IDT, PIC)
/// 5. Other subsystems (scheduler, IPC, etc.)
#[no_mangle]
#[link_section = ".text.kmain"]
pub unsafe extern "C" fn kmain() -> ! {
    // Ensure Limine protocol version is supported
    if !BASE_REVISION.is_supported() {
        halt_loop();
    }

    // Phase 1: Early init - serial port for debug output
    serial::init();
    serial_println!("T-RustOs Kernel v0.1.0");
    serial_println!("Limine protocol supported");

    // Phase 2: Initialize framebuffer console
    if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
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
    
    use framebuffer::BootStatus;

    // Phase 3: Memory management (MUST be before any println! that allocates)
    serial_println!("Initializing memory management...");
    
    let mut heap_initialized = false;
    
    if let Some(mmap_response) = MEMORY_MAP_REQUEST.get_response() {
        let hhdm_offset = HHDM_REQUEST.get_response()
            .map(|r| r.offset())
            .unwrap_or(0);
        
        serial_println!("HHDM offset: {:#x}", hhdm_offset);
        serial_println!("Memory map entries: {}", mmap_response.entries().len());
        
        // Find a usable region for heap (skip first 1MB and kernel/modules)
        let mut usable_for_heap: Option<u64> = None;
        let mut kernel_end: u64 = 0;
        let mut total_phys_memory: u64 = 0;
        
        // Log memory regions and find heap location
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
            
            // Track end of kernel/modules and bootloader reclaimable regions
            if entry.entry_type == limine::memory_map::EntryType::EXECUTABLE_AND_MODULES
                || entry.entry_type == limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE
            {
                let end = entry.base + entry.length;
                if end > kernel_end {
                    kernel_end = end;
                }
            }
            // Sum all memory (usable + reclaimable)
            total_phys_memory += entry.length;
        }

        // Store total physical memory for later use
        memory::set_total_physical_memory(total_phys_memory);
        serial_println!("[MEM] Total physical memory: {} MB", total_phys_memory / 1024 / 1024);

        // Compute dynamic heap size: 25% of detected RAM, clamped [64 MB, 512 MB]
        let dynamic_heap_size = memory::compute_heap_size(total_phys_memory);
        serial_println!("[HEAP] Dynamic size: {} MB (25% of {} MB RAM)", 
            dynamic_heap_size / 1024 / 1024, total_phys_memory / 1024 / 1024);

        // Align up to 4 KiB
        let align_up = |addr: u64, align: u64| -> u64 {
            if addr % align == 0 { addr } else { addr + (align - (addr % align)) }
        };

        // Find suitable heap region after kernel/modules
        let min_heap_base = align_up(core::cmp::max(0x100000, kernel_end), 0x1000);
        for entry in mmap_response.entries() {
            if entry.entry_type != limine::memory_map::EntryType::USABLE {
                continue;
            }
            let region_start = entry.base;
            let region_end = entry.base.saturating_add(entry.length);
            if region_end <= min_heap_base {
                continue;
            }
            let heap_start = core::cmp::max(region_start, min_heap_base);
            if region_end >= heap_start + dynamic_heap_size as u64 {
                usable_for_heap = Some(heap_start);
                break;
            }
        }

        // If not found, fall back to the largest usable region
        if usable_for_heap.is_none() {
            let mut best_base: u64 = 0;
            let mut best_len: u64 = 0;
            for entry in mmap_response.entries() {
                if entry.entry_type != limine::memory_map::EntryType::USABLE {
                    continue;
                }
                if entry.length > best_len {
                    best_len = entry.length;
                    best_base = entry.base;
                }
            }

            if best_len >= dynamic_heap_size as u64 {
                let mut heap_start = align_up(best_base, 0x1000);
                if heap_start < 0x100000 {
                    heap_start = align_up(0x100000, 0x1000);
                }
                if best_base.saturating_add(best_len) >= heap_start + dynamic_heap_size as u64 {
                    usable_for_heap = Some(heap_start);
                }
            }
        }
        
        // Initialize heap with HHDM (dynamic size)
        if let Some(heap_phys) = usable_for_heap {
            serial_println!("[HEAP] Using mmap region at phys {:#x}, size {} MB", heap_phys, dynamic_heap_size / 1024 / 1024);
            // NOTE: Do NOT use println! here - heap not yet initialized!
            memory::init_with_hhdm_dynamic(hhdm_offset, heap_phys, dynamic_heap_size);
            heap_initialized = true;
            serial_println!("[HEAP] Initialized: free={} KB", memory::heap::free() / 1024);
        } else {
            serial_println!("[HEAP] ERROR: No usable region found for {} MB heap!", dynamic_heap_size / 1024 / 1024);
        }
    }
    
    if !heap_initialized {
        // Fallback
        serial_println!("[HEAP] Using fallback init");
        memory::init();
    }
    
    // Now that heap is initialized, initialize scrollback buffer (allocates ~3MB)
    serial_println!("[FB] Initializing scrollback buffer...");
    framebuffer::init_scrollback();
    
    // Compute SHA-256 of kernel .text section for runtime integrity verification
    signature::init_integrity();
    
    // Now that heap is initialized, show boot banner
    framebuffer::show_simple_boot_header();
    framebuffer::print_boot_status("Memory management initialized", BootStatus::Ok);

    // Phase 3.5: GDT with Ring 0/3 support
    serial_println!("Initializing GDT with Ring 0/3 support...");
    gdt::init();
    framebuffer::print_boot_status("GDT initialized (Ring 0/3)", BootStatus::Ok);
    
    // Phase 3.51: Early interrupts (needed for page fault debugging)
    serial_println!("Initializing early interrupts...");
    interrupts::init();
    framebuffer::print_boot_status("Interrupts (early)", BootStatus::Ok);
    
    // Phase 3.55: CPU hardware exploitation (TSC, AES-NI, SIMD, SMP)
    serial_println!("Detecting CPU capabilities...");
    cpu::init();
    framebuffer::print_boot_status("CPU capabilities detected", BootStatus::Ok);
    
    // Phase 3.555: ACPI tables parsing
    serial_println!("Parsing ACPI tables...");
    if let Some(rsdp_response) = RSDP_REQUEST.get_response() {
        // Limine gives us a pointer that's directly usable (already mapped)
        let rsdp_ptr = rsdp_response.address();
        serial_println!("[DEBUG] RSDP pointer from Limine: {:#x}", rsdp_ptr as usize);
        // Use the pointer directly - Limine has already mapped it for us
        if acpi::init_direct(rsdp_ptr as u64) {
            if let Some(info) = acpi::get_info() {
                framebuffer::print_boot_status(&alloc::format!(
                    "ACPI: {} CPUs, {} I/O APICs", 
                    info.cpu_count, info.io_apics.len()
                ), BootStatus::Ok);
            }
        } else {
            framebuffer::print_boot_status("ACPI init failed", BootStatus::Skip);
        }
    } else {
        framebuffer::print_boot_status("No RSDP from bootloader", BootStatus::Skip);
    }
    
    // Phase 3.56: SMP initialization - Start all CPU cores!
    serial_println!("Initializing SMP...");
    cpu::smp::init();
    
    // Boot Application Processors (APs)
    if let Some(smp_response) = SMP_REQUEST.get_response() {
        let cpu_count = smp_response.cpus().len();
        serial_println!("[SMP] Found {} CPUs via Limine", cpu_count);
        
        // Start each AP
        for cpu in smp_response.cpus().iter() {
            if cpu.id != 0 {  // Skip BSP (Bootstrap Processor)
                serial_println!("[SMP] Starting AP {} (LAPIC ID: {})", cpu.id, cpu.lapic_id);
                // Set the entry point for this AP
                cpu.goto_address.write(cpu::smp::ap_entry);
            }
        }
        
        // Wait for APs to initialize (with timeout)
        let mut ready_count = 1u32; // BSP is ready
        for _ in 0..1000 {
            ready_count = cpu::smp::ready_cpu_count();
            if ready_count >= cpu_count as u32 {
                break;
            }
            // Small delay
            for _ in 0..10000 { core::hint::spin_loop(); }
        }
        
        serial_println!("[SMP] {} of {} CPUs online", ready_count, cpu_count);
        cpu::smp::set_cpu_count(cpu_count as u32);
        framebuffer::print_boot_status(&alloc::format!("SMP: {} cores active", ready_count), BootStatus::Ok);
    } else {
        serial_println!("[SMP] No SMP response from bootloader");
        framebuffer::print_boot_status("SMP: single core", BootStatus::Ok);
    }
    
    // Phase 3.6: Paging subsystem
    serial_println!("Initializing paging subsystem...");
    memory::paging::init();  // Saves kernel CR3, enables NX
    framebuffer::print_boot_status("Paging initialized (NX enabled)", BootStatus::Ok);
    
    // PAT: Enable Write-Combining (WC) — standard GPU driver optimization
    // All GPU drivers (Mesa, NVIDIA, AMD) use WC for VRAM writes:
    // batches individual stores into 64-byte burst transfers (10-20x faster)
    // Skip on VirtualBox: VMSVGA dirty-tracking breaks with WC memory type,
    // causing the display to freeze (writes are buffered and VBox stops detecting them)
    let is_vbox = acpi::get_info()
        .map(|info| info.oem_id.trim().eq_ignore_ascii_case("VBOX"))
        .unwrap_or(false);
    if is_vbox {
        serial_println!("[PAT] Skipping Write-Combining on VirtualBox (VMSVGA compat)");
    } else {
        memory::paging::setup_pat_write_combining();
    }
    
    // Phase 3.7: Userland support (SYSCALL/SYSRET)
    serial_println!("Initializing userland support...");
    userland::init_syscall_stack();
    userland::init();
    framebuffer::print_boot_status("Userland support ready", BootStatus::Ok);
    
    // Phase 3.8: Thread subsystem
    serial_println!("Initializing thread subsystem...");
    thread::init();
    framebuffer::print_boot_status("Thread subsystem ready", BootStatus::Ok);
    
    // Phase 3.9: Security subsystem (SMEP, SMAP, capabilities)
    serial_println!("Initializing security subsystem...");
    // security::init();  // Disabled for now - CPUID calls may crash on some QEMU
    framebuffer::print_boot_status("Security (basic)", BootStatus::Ok);

    // Phase 4: Keyboard driver (interrupts already initialized early)
    serial_println!("Keyboard driver ready");
    framebuffer::print_boot_status("Keyboard ready", BootStatus::Ok);

    // Full boot: initialize all subsystems
    // (Set to true only for debugging boot freezes)
    
    // Phase 6: RTC (Real-Time Clock)
    const ENABLE_RTC: bool = true;
    if ENABLE_RTC {
        serial_println!("[RTC] init start");
        if rtc::try_init() {
            framebuffer::print_boot_status("RTC initialized", BootStatus::Ok);
        } else {
            framebuffer::print_boot_status("RTC skipped", BootStatus::Skip);
        }
        serial_println!("[RTC] init done");
    } else {
        framebuffer::print_boot_status("RTC disabled", BootStatus::Skip);
    }
    
    // Phase 7: Mouse driver
    mouse::init();
    let (fb_width, fb_height) = framebuffer::get_dimensions();
    mouse::set_screen_size(fb_width, fb_height);
    framebuffer::print_boot_status("Mouse initialized", BootStatus::Ok);
    
    // Phase 8-12: Subsystem toggles (enable for full functionality)
    const ENABLE_PCI: bool = true;
    const ENABLE_TASKS: bool = true;
    const ENABLE_DISK: bool = true;   // Enable for virtio-blk persistence
    const ENABLE_DRIVERS: bool = true;
    const ENABLE_NETWORK: bool = true;

    // Phase 8: PCI Bus Enumeration (BEFORE device drivers)
    serial_println!("[PHASE] PCI init start");
    framebuffer::print_boot_status("PCI bus scanning...", BootStatus::Info);
    if ENABLE_PCI {
        pci::init();
        framebuffer::print_boot_status("PCI bus scanned", BootStatus::Ok);
    } else {
        framebuffer::print_boot_status("PCI disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] PCI init done");
    
    // Phase 9: Task scheduler
    serial_println!("[PHASE] Task scheduler init start");
    if ENABLE_TASKS {
        task::init();
        scheduler::init();
        framebuffer::print_boot_status("Task scheduler ready", BootStatus::Ok);
    } else {
        framebuffer::print_boot_status("Task scheduler disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Task scheduler init done");
    
    // Phase 10: Disk I/O
    serial_println!("[PHASE] Disk init start");
    framebuffer::print_boot_status("Disk subsystem...", BootStatus::Info);
    if ENABLE_DISK {
        // Try virtio-blk first (for persistent storage in QEMU)
        let blk_devs: alloc::vec::Vec<_> = pci::get_devices().iter()
            .filter(|d| d.vendor_id == 0x1AF4 && d.device_id == 0x1001)
            .cloned()
            .collect();
        
        if !blk_devs.is_empty() {
            if let Err(e) = virtio_blk::init(&blk_devs[0]) {
                crate::log_warn!("[DISK] virtio-blk init failed: {}", e);
                // Fallback to RAM disk
                disk::init();
            } else {
                framebuffer::print_boot_status(&alloc::format!("virtio-blk: {} MB storage", 
                    (virtio_blk::capacity() * 512) / (1024 * 1024)), BootStatus::Ok);
            }
        } else {
            // No virtio-blk, use RAM disk
            disk::init();
        }
        
        if disk::is_available() || virtio_blk::is_initialized() {
            framebuffer::print_boot_status("Disk driver ready", BootStatus::Ok);
        } else {
            framebuffer::print_boot_status("No disk detected", BootStatus::Skip);
        }
    } else {
        framebuffer::print_boot_status("Disk disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Disk init done");
    
    // Phase 11: Driver Framework
    serial_println!("[PHASE] Driver framework init start");
    if ENABLE_DRIVERS {
        drivers::init();
        // Probe storage controllers (AHCI, IDE)
        drivers::probe_storage();
        framebuffer::print_boot_status("Driver framework initialized", BootStatus::Ok);
        if drivers::has_storage() {
            framebuffer::print_boot_status("Persistent storage detected", BootStatus::Ok);
        }
    } else {
        framebuffer::print_boot_status("Driver framework disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Driver framework init done");
    
    // Phase 11b: VirtIO GPU
    serial_println!("[PHASE] VirtIO GPU init start");
    framebuffer::print_boot_status("VirtIO GPU...", BootStatus::Info);
    drivers::virtio_gpu::init_from_pci().ok();
    if drivers::virtio_gpu::is_available() {
        framebuffer::print_boot_status(&alloc::format!("VirtIO GPU: {}", drivers::virtio_gpu::info_string()), BootStatus::Ok);
    } else {
        framebuffer::print_boot_status("VirtIO GPU: not found (fallback framebuffer)", BootStatus::Skip);
    }
    serial_println!("[PHASE] VirtIO GPU init done");
    
    // Remap framebuffer as Write-Combining for faster MMIO writes
    // Skip on VirtualBox: VMSVGA dirty-tracking breaks with WC pages
    if !is_vbox {
        let fb_addr = framebuffer::FB_ADDR.load(core::sync::atomic::Ordering::SeqCst);
        let fb_w = framebuffer::FB_WIDTH.load(core::sync::atomic::Ordering::SeqCst) as usize;
        let fb_h = framebuffer::FB_HEIGHT.load(core::sync::atomic::Ordering::SeqCst) as usize;
        if !fb_addr.is_null() && fb_w > 0 && fb_h > 0 {
            let fb_size = fb_w * fb_h * 4;
            let _ = memory::paging::remap_region_write_combining(fb_addr as u64, fb_size);
        }
    } else {
        serial_println!("[PAT] Skipping framebuffer WC remap on VirtualBox");
    }
    
    // Phase 12: Network (with universal driver system)
    serial_println!("[PHASE] Network init start");
    framebuffer::print_boot_status("Network subsystem...", BootStatus::Info);
    if ENABLE_NETWORK {
        network::init();
        if network::is_available() {
            // Display detected platform
            let platform = network::get_platform();
            framebuffer::print_boot_status(&alloc::format!("Platform: {}", platform), BootStatus::Info);
            
            // Auto-probe and load network drivers
            let devs = pci::find_by_class(pci::class::NETWORK);
            if !devs.is_empty() {
                for dev in &devs {
                    if drivers::net::probe_device(dev) {
                        network::update_mac_from_driver();
                        let driver_name = if dev.vendor_id == 0x1AF4 { "virtio-net" } 
                            else if dev.vendor_id == 0x8086 { "e1000" }
                            else if dev.vendor_id == 0x10EC { "rtl8139" }
                            else { "unknown" };
                        framebuffer::print_boot_status(&alloc::format!("Network driver: {}", driver_name), BootStatus::Ok);
                        break;
                    }
                }
            }

            // Fallback: init legacy virtio-net if no active universal driver
            if !drivers::net::has_driver() && !crate::virtio_net::is_initialized() {
                if let Some(dev) = devs.iter().find(|d| d.vendor_id == 0x1AF4) {
                    if let Err(e) = crate::virtio_net::init(dev) {
                        crate::log_warn!("[NET] Legacy virtio-net init failed: {}", e);
                    } else {
                        network::update_mac_from_driver();
                        framebuffer::print_boot_status("Network driver: virtio-net (legacy)", BootStatus::Ok);
                    }
                }
            }
            
            framebuffer::print_boot_status("Network ready", BootStatus::Ok);
            
            // Start network stack
            netstack::dhcp::start();
        }
    } else {
        framebuffer::print_boot_status("Network disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Network init done");
    
    // ========================================================================
    // Phase 12: VFS Initialization
    // ========================================================================
    serial_println!("[PHASE] VFS init start");
    vfs::init();
    framebuffer::print_boot_status("Virtual filesystem ready", BootStatus::Ok);
    serial_println!("[PHASE] VFS init done");
    
    // ========================================================================
    // Phase 12a: Linux Subsystem (TSL) - Load kernel and initramfs from modules
    // ========================================================================
    serial_println!("[PHASE] Linux Subsystem init start");
    if let Some(module_response) = MODULE_REQUEST.get_response() {
        let modules = module_response.modules();
        serial_println!("[TSL] Found {} boot modules", modules.len());
        
        let mut kernel_data: Option<&'static [u8]> = None;
        let mut initramfs_data: Option<&'static [u8]> = None;
        
        for module in modules {
            // Get cmdline and path - cmdline is &[u8], path is &CStr
            let cmdline_bytes = module.cmdline();
            let cmdline = core::str::from_utf8(cmdline_bytes).unwrap_or("unknown");
            let path = module.path().to_str().unwrap_or("unknown");
            let addr = module.addr();
            let size = module.size() as usize;
            
            serial_println!("[TSL] Module: {} ({}), {} bytes at {:p}", 
                cmdline, path, size, addr);
            
            // Safety: Limine ensures modules are valid and accessible
            let data = unsafe { core::slice::from_raw_parts(addr, size) };
            
            if cmdline.contains("linux-kernel") || path.contains("bzImage") {
                kernel_data = Some(data);
                serial_println!("[TSL] Linux kernel loaded: {} bytes", size);
            } else if cmdline.contains("linux-initramfs") || path.contains("initramfs") {
                initramfs_data = Some(data);
                serial_println!("[TSL] Initramfs loaded: {} bytes", size);
            }
        }
        
        // Set embedded images in the Linux subsystem
        if let (Some(kernel), Some(initramfs)) = (kernel_data, initramfs_data) {
            hypervisor::linux_subsystem::subsystem().set_embedded_images(kernel, initramfs);
            framebuffer::print_boot_status("Linux Subsystem (TSL) ready", BootStatus::Ok);
        } else {
            if kernel_data.is_none() {
                serial_println!("[TSL] Warning: Linux kernel not found in boot modules");
            }
            if initramfs_data.is_none() {
                serial_println!("[TSL] Warning: Initramfs not found in boot modules");
            }
            framebuffer::print_boot_status("Linux Subsystem (partial)", BootStatus::Skip);
        }
    } else {
        serial_println!("[TSL] No boot modules available");
        framebuffer::print_boot_status("Linux Subsystem (no modules)", BootStatus::Skip);
    }
    serial_println!("[PHASE] Linux Subsystem init done");
    
    // ========================================================================
    // Phase 12b: File Associations
    // ========================================================================
    file_assoc::init();
    framebuffer::print_boot_status("File associations ready", BootStatus::Ok);
    
    // ========================================================================
    // Phase 13: Process Manager
    // ========================================================================
    serial_println!("[PHASE] Process manager init start");
    process::init();
    framebuffer::print_boot_status("Process manager ready", BootStatus::Ok);
    serial_println!("[PHASE] Process manager init done");
    
    // ========================================================================
    // Phase 14: Authentication System
    // ========================================================================
    serial_println!("[PHASE] Auth system init");
    auth::init();
    auth::create_etc_files();
    framebuffer::print_boot_status("Authentication ready", BootStatus::Ok);
    
    // ========================================================================
    // Phase 15: Init Process (PID 1)
    // ========================================================================
    serial_println!("[PHASE] Init process start");
    init::start();
    framebuffer::print_boot_status("Init process started (PID 1)", BootStatus::Ok);
    serial_println!("[PHASE] Init process done");
    
    // ========================================================================
    // Phase 16: RAM Filesystem
    // ========================================================================
    serial_println!("[PHASE] RAM filesystem init");
    ramfs::init();
    // Create standard directories
    ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/tmp");
        let _ = fs.mkdir("/var");
        let _ = fs.mkdir("/home");
        let _ = fs.mkdir("/bin");
        let _ = fs.mkdir("/usr");
        let _ = fs.mkdir("/etc");
    });
    framebuffer::print_boot_status("RAM filesystem ready", BootStatus::Ok);

    // ========================================================================
    // Phase 17: Persistence System
    // ========================================================================
    serial_println!("[PHASE] Persistence init");
    persistence::init();
    framebuffer::print_boot_status("Persistence system ready", BootStatus::Ok);

    // Final boot summary
    println!();
    framebuffer::draw_separator(framebuffer::get_cursor().1 as u32 * 16, framebuffer::COLOR_GREEN);
    println!();
    println_color!(framebuffer::COLOR_BRIGHT_GREEN, "  System ready - TRust-OS v0.1.0");
    println_color!(framebuffer::COLOR_GREEN, "  Type 'desktop' to launch the desktop, or use shell commands.");
    println!();

    // Check for saved data and ask user
    if persistence::is_available() {
        persistence::prompt_restore();
    }

    // Auto-login as root for now (development mode)
    auth::auto_login_root();

    // Run crypto self-tests at startup
    serial_println!("[BOOT] Running crypto self-tests...");
    tls13::crypto::run_self_tests();
    serial_println!("[BOOT] Crypto self-tests complete");

    // Start shell (runs forever)
    serial_println!("Starting shell...");
    shell::run();
}

// ============================================================================
// Core Functions
// ============================================================================

/// Halt the CPU in an infinite loop
/// Used when kernel cannot continue or has nothing to do
fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    serial_println!("\n!!! ALLOC ERROR !!!");
    serial_println!("layout: size={}, align={}", layout.size(), layout.align());
    if framebuffer::is_initialized() {
        framebuffer::set_fg_color(framebuffer::COLOR_RED);
        println!("\n!!! ALLOC ERROR !!!");
        println!("layout: size={}, align={}", layout.size(), layout.align());
    }
    halt_loop();
}

/// Panic handler - called on unrecoverable errors
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Try to output panic info via serial
    serial_println!("\n!!! KERNEL PANIC !!!");
    serial_println!("{}", info);
    
    // Try framebuffer if available
    if framebuffer::is_initialized() {
        framebuffer::set_fg_color(framebuffer::COLOR_RED);
        println!("\n!!! KERNEL PANIC !!!");
        println!("{}", info);
    }
    
    halt_loop();
}
