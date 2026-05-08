//! T-RustOs Kernel
//! 
//! Microkernel architecture with capability-based security.
//! Boots via Limine bootloader on UEFI systems.
//!
//! Supported architectures: x86_64, aarch64 (ARM64), riscv64 (RISC-V)

#![no_std]
#![no_main]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![feature(alloc_error_handler)]

// Suppress harmless warnings common in kernel/OS development.
// Many modules define infrastructure for future use or hardware register names.
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

// Architecture abstraction layer — provides unified interface across all CPUs
pub mod arch;

// Core modules
mod serial;
#[macro_use]
pub mod debug_trace;
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
mod pcie_recovery;
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
#[cfg_attr(not(feature = "extras"), path = "game3d_stub.rs")]
mod game3d; // 3D raycasting FPS game engine
#[cfg_attr(not(feature = "extras"), path = "chess_stub.rs")]
mod chess;   // Chess game engine with AI
#[cfg_attr(not(feature = "extras"), path = "chess3d_stub.rs")]
mod chess3d; // 3D Matrix-style chess renderer
#[cfg(feature = "emulators")]
mod nes;     // NES emulator (MOS 6502 + 2C02 PPU, iNES ROMs)
#[cfg(feature = "emulators")]
mod gameboy; // Game Boy emulator (Sharp LR35902, MBC1/3/5)
#[cfg(feature = "emulators")]
mod game_lab; // GameLab — real-time Game Boy emulator analysis dashboard
#[cfg(feature = "emulators")]
mod embedded_roms; // ROM data embedded at compile time from kernel/roms/
mod cosmic; // COSMIC-style UI framework (libcosmic-inspired)
mod compositor; // Multi-layer compositor for flicker-free rendering
#[cfg_attr(not(feature = "extras"), path = "holovolume_stub.rs")]
mod holovolume; // Volumetric ASCII raymarcher - 3D holographic desktop
#[cfg_attr(not(feature = "extras"), path = "matrix_fast_stub.rs")]
mod matrix_fast; // Ultra-optimized Matrix rain with Braille sub-pixels
#[cfg_attr(not(feature = "extras"), path = "formula3d_stub.rs")]
mod formula3d;   // Tsoding-inspired wireframe 3D renderer (perspective projection)
#[cfg_attr(not(feature = "extras"), path = "gpu_emu_stub.rs")]
mod gpu_emu;      // Virtual GPU - CPU cores emulating GPU parallelism
#[cfg(feature = "woa")]
mod woa;         // WOA — World of Ants (roguelike platformer engine)
// TLS 1.3 pure Rust implementation (no C dependencies)
mod tls13;
// Crypto primitives shared by TLS / WiFi / etc. (SHA-1, HMAC-SHA1, PBKDF2, AES-CCMP)
mod crypto;
// CPU hardware exploitation (TSC, AES-NI, SIMD, SMP)
#[cfg(target_arch = "x86_64")]
mod cpu;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/cpu.rs"]
mod cpu;

// ACPI tables parsing (MADT, FADT, MCFG, HPET)
#[cfg(target_arch = "x86_64")]
mod acpi;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/acpi.rs"]
mod acpi;

// APIC driver (Local APIC + I/O APIC) — replaces legacy PIC for preemptive scheduling
#[cfg(target_arch = "x86_64")]
mod apic;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/apic.rs"]
mod apic;

// New OS infrastructure
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
mod userland_audit;
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

// Universal RISC-V Translation Layer (run ANY architecture's binaries via RISC-V IR)
mod riscv_translator;

// TrustView — binary analysis engine (ELF parser, disassembler, xrefs)
mod binary_analysis;

// TrustScan — network security scanning toolkit (port scanner, sniffer, vuln scanner)
mod netscan;

// TrustProbe — bare-metal hardware security research toolkit (MMIO, TrustZone, DMA, GPIO)
mod hwscan;

// HWDiag — comprehensive hardware debugging toolkit for PXE boot diagnostics
mod hwdiag;

// Marionet — bare-metal hardware dashboard (full-screen TUI)
mod marionet;

// ELECTRO — comprehensive electrical power & thermal dashboard (~90 metrics)
mod electro;

// Thermal management daemon — fan curves + watchdog
mod thermal;

// CoreMark EEMBC benchmark (bare-metal, minimal boot path)
#[cfg(feature = "coremark")]
mod coremark;

// HTTP Server — embedded web server
mod httpd;

// TrustPkg — package manager
mod trustpkg;

// TrustLab — real-time educational OS introspection laboratory
mod lab_mode;

// TrustWave — real-time WiFi wave analyzer & visualizer
mod wifi_analyzer;

// TrustOS Logo — embedded ARGB bitmap (auto-generated by convert_logo.py)
// In slim builds (no "hires-logo" feature), a stub with no-op draws is used instead.
#[cfg_attr(not(feature = "hires-logo"), path = "logo_bitmap_stub.rs")]
mod logo_bitmap;

// TrustLang — integrated programming language (Rust-like, bytecode VM)
mod trustlang;

// TrustVideo — custom video codec & player (delta+RLE, no external APIs)
#[cfg_attr(not(feature = "extras"), path = "video_stub.rs")]
mod video;

// Android boot support (boot.img, DTB, PSCI) — aarch64 only
#[cfg(target_arch = "aarch64")]
mod android_boot;
#[cfg(target_arch = "aarch64")]
mod android_main;

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
#[cfg(target_arch = "x86_64")]
mod hypervisor;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/hypervisor.rs"]
mod hypervisor;
mod rasterizer;
#[cfg_attr(not(feature = "extras"), path = "model_editor_stub.rs")]
mod model_editor;

// Shared math utilities (sin, cos, sqrt, atan2) — centralized
pub mod math;

// Shared buffer drawing helpers (Bresenham, fill_rect, xorshift)
pub mod draw_utils;

// Developer tools (profiler, dmesg, memdbg, devpanel, peek/poke)
mod devtools;

// Hardware debug toolkit (POST codes, backtrace, crash dump, watchdog)
mod debug;

// TrustSynth — polyphonic audio synthesizer engine
mod audio;

// TrustDAW — bare-metal Digital Audio Workstation
#[cfg(feature = "daw")]
mod trustdaw;

// Web Sandbox — capability-gated isolated web execution environment
mod sandbox;

// Kernel signature & proof of authorship
mod signature;
// Ed25519 digital signatures (asymmetric crypto)
pub mod ed25519;

// Synchronization primitives (Redox-inspired)
mod sync;
// POSIX signals (x86_64 only — uses CpuContext register fields)
#[cfg(target_arch = "x86_64")]
mod signals;
#[cfg(not(target_arch = "x86_64"))]
#[path = "stubs/signals.rs"]
mod signals;
// TTY subsystem (line discipline, sessions)
mod tty;
// Pseudo-terminal (PTY) pairs
mod pty;
// Process tracing (debugging) — x86_64 only
#[cfg(target_arch = "x86_64")]
mod ptrace;
// Safe user/kernel memory copy
mod usercopy;

// Jarvis Neural Brain — self-hosted tiny transformer for on-device AI
#[cfg(feature = "jarvis")]
mod jarvis;

// Jarvis Hardware Intelligence — AI-driven hardware awareness and self-optimization
#[cfg(feature = "jarvis")]
mod jarvis_hw;

// TrustOS Installer — self-install to SATA/NVMe from live boot
mod installer;

use core::panic::PanicInfo;
use core::alloc::Layout;
use limine::request::{
    FramebufferRequest, MemoryMapRequest, HhdmRequest,
    RequestsStartMarker, RequestsEndMarker, ModuleRequest,
    RsdpRequest, SmpRequest, KernelAddressRequest, KernelFileRequest,
    StackSizeRequest,
};
use limine::BaseRevision;

// ============================================================================
// Boot Mode (parsed from Limine cmdline)
// ============================================================================

/// Boot mode determined by Limine cmdline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMode {
    /// Normal live boot with all features
    Live,
    /// Live boot without JARVIS AI
    LiveNoJarvis,
    /// Installation mode — install to disk
    Install,
}

static BOOT_MODE: spin::Mutex<BootMode> = spin::Mutex::new(BootMode::Live);

/// Get the current boot mode
pub fn boot_mode() -> BootMode {
    *BOOT_MODE.lock()
}

/// Parse boot mode from Limine kernel cmdline
fn parse_boot_cmdline(cmdline: &str) {
    let mut mode = BootMode::Live;
    let mut jarvis = true;
    for param in cmdline.split_whitespace() {
        if let Some(val) = param.strip_prefix("mode=") {
            match val {
                "install" => mode = BootMode::Install,
                _ => mode = BootMode::Live,
            }
        }
        if let Some(val) = param.strip_prefix("jarvis=") {
            if val == "no" || val == "false" || val == "0" {
                jarvis = false;
            }
        }
    }
    if mode == BootMode::Live && !jarvis {
        mode = BootMode::LiveNoJarvis;
    }
    *BOOT_MODE.lock() = mode;
}

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

/// Deferred JARVIS brain module data (saved during module scan, written to RamFS after Phase 16)
static mut JARVIS_BRAIN_MODULE: Option<&'static [u8]> = None;

/// Request RSDP (ACPI tables) from Limine
#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

/// Request SMP (multi-core) support from Limine
#[used]
#[unsafe(link_section = ".requests")]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

/// Request kernel address info (needed for virt→phys translation on aarch64)
#[used]
#[unsafe(link_section = ".requests")]
static KERNEL_ADDRESS_REQUEST: KernelAddressRequest = KernelAddressRequest::new();

/// Request kernel file data (for PXE self-replication — serve our own binary)
#[used]
#[unsafe(link_section = ".requests")]
static KERNEL_FILE_REQUEST: KernelFileRequest = KernelFileRequest::new();

/// Request larger stack (512 KB) from Limine to support deep desktop rendering + interrupts
/// 256 KB was insufficient for the desktop's nested draw_background() call chain
/// (4-layer matrix rain × 256 columns × visualizer + drone_swarm per frame)
#[used]
#[unsafe(link_section = ".requests")]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(512 * 1024);

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
    // Boot timing: read TSC at entry for profiling
    #[cfg(target_arch = "x86_64")]
    let tsc_boot_start: u64 = core::arch::x86_64::_rdtsc();
    #[cfg(not(target_arch = "x86_64"))]
    let tsc_boot_start: u64 = 0;

    // Ensure Limine protocol version is supported
    if !BASE_REVISION.is_supported() {
        halt_loop();
    }

    // Phase 0.5: Set up MMIO identity mapping (aarch64 needs Device-nGnRnE attributes)
    // Limine clears TTBR0 and HHDM maps MMIO as Normal Cacheable.
    // We need identity-mapped Device memory for PL011 UART access.
    #[cfg(target_arch = "aarch64")]
    {
        if let Some(ka_resp) = KERNEL_ADDRESS_REQUEST.get_response() {
            crate::arch::platform::boot::setup_mmio_identity_map(
                ka_resp.virtual_base(),
                ka_resp.physical_base(),
            );
        }
        // UART_BASE stays at 0x09000000 (identity-mapped with Device attributes)
    }

    // Phase 0.9: Register kernel file for PXE self-replication + parse boot cmdline
    if let Some(kf_resp) = KERNEL_FILE_REQUEST.get_response() {
        let file = kf_resp.file();
        let ptr = file.addr();
        let size = file.size() as usize;
        if size > 0 {
            #[cfg(feature = "jarvis")]
            unsafe { jarvis::pxe_replicator::register_kernel_file(ptr, size); }
            // Also register for installer (self-copy to disk)
            unsafe { installer::register_kernel_binary(ptr, size); }
        }
        // Parse boot cmdline (mode=live|install, jarvis=yes|no)
        let cmdline_bytes = file.cmdline();
        if let Ok(cmdline_str) = core::str::from_utf8(cmdline_bytes) {
            parse_boot_cmdline(cmdline_str);
        }
    }

    // Phase 1: Early init - serial port for debug output
    serial::init();
    crate::boot_phase!(1, "serial init");
    debug::checkpoint(debug::POST_SERIAL_INIT, "Serial port initialized");
    serial_println!("T-RustOs Kernel v0.2.0");
    serial_println!("Limine protocol supported");
    serial_println!("[BOOT] Mode: {:?}", boot_mode());

    // Boot timing helper: print elapsed ms since boot
    #[cfg(target_arch = "x86_64")]
    let tsc_freq_mhz: u64 = {
        // Estimate TSC freq: ~2000 MHz default, refined later if needed
        // Celeron G1610 = 2.6 GHz, i5 = ~3.2 GHz — use 2000 as safe lower bound
        2000
    };
    #[cfg(not(target_arch = "x86_64"))]
    let tsc_freq_mhz: u64 = 1;
    
    /// Print boot phase timing
    macro_rules! boot_timing {
        ($label:expr) => {
            #[cfg(target_arch = "x86_64")]
            {
                let now = core::arch::x86_64::_rdtsc();
                let elapsed_us = (now - tsc_boot_start) / tsc_freq_mhz;
                serial_println!("[BOOT +{}ms] {}", elapsed_us / 1000, $label);
            }
        };
    }
    
    boot_timing!("Serial init");

    // Debug: re-read cmdline after serial is up
    if let Some(kf_resp) = KERNEL_FILE_REQUEST.get_response() {
        let file = kf_resp.file();
        let cmdline_bytes = file.cmdline();
        if let Ok(s) = core::str::from_utf8(cmdline_bytes) {
            serial_println!("[BOOT] Kernel cmdline raw: '{}'", s);
        } else {
            serial_println!("[BOOT] Kernel cmdline: {} bytes (not UTF-8)", cmdline_bytes.len());
        }
        if let Ok(p) = file.path().to_str() {
            serial_println!("[BOOT] Kernel path: '{}'", p);
        }
    }

    // Phase 1.1: Detect exception level on aarch64
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
    
    // Visual POST: framebuffer alive (0x11)
    framebuffer::visual_post_code(debug::POST_FRAMEBUFFER);
    boot_timing!("Framebuffer init");
    
    // === BOOT SPLASH: Draw logo + empty progress bar immediately ===
    framebuffer::init_boot_splash();
    boot_timing!("Boot splash drawn");
    
    use framebuffer::BootStatus;

    // Phase 3: Memory management (MUST be before any println! that allocates)
    framebuffer::visual_post_code(debug::POST_MEMORY);
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
            
            // Store for debug diagnostics
            let type_code = match entry.entry_type {
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
            memory::store_memory_region(entry.base, entry.length, type_code);
            
            // Track end of kernel/modules and bootloader reclaimable regions
            if entry.entry_type == limine::memory_map::EntryType::EXECUTABLE_AND_MODULES
                || entry.entry_type == limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE
            {
                let end = entry.base + entry.length;
                if end > kernel_end {
                    kernel_end = end;
                }
            }
            // Sum only usable memory (not reserved / MMIO / framebuffer)
            if entry.entry_type == limine::memory_map::EntryType::USABLE
                || entry.entry_type == limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE
                || entry.entry_type == limine::memory_map::EntryType::ACPI_RECLAIMABLE
            {
                total_phys_memory += entry.length;
            }
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
            
            // Initialize frame allocator now that the heap is ready
            // Collect USABLE regions from the memory map
            let usable_regions: alloc::vec::Vec<memory::frame::PhysRegion> = mmap_response.entries()
                .iter()
                .filter(|e| e.entry_type == limine::memory_map::EntryType::USABLE)
                .map(|e| memory::frame::PhysRegion { base: e.base, length: e.length })
                .collect();
            memory::frame::init(&usable_regions, heap_phys, dynamic_heap_size as u64);
        } else {
            // No region large enough for the computed heap size.
            // Try again with the largest usable region we can find.
            let mut best_base: u64 = 0;
            let mut best_len: u64 = 0;
            for entry in mmap_response.entries() {
                if entry.entry_type != limine::memory_map::EntryType::USABLE && entry.length > best_len {
                    // skip non-usable
                }
                if entry.entry_type == limine::memory_map::EntryType::USABLE && entry.length > best_len {
                    best_len = entry.length;
                    best_base = entry.base;
                }
            }
            
            if best_len >= memory::HEAP_SIZE_MIN as u64 {
                let heap_start2 = align_up(core::cmp::max(best_base, 0x100000), 0x1000);
                let usable_end = best_base.saturating_add(best_len);
                let heap_size2 = core::cmp::min(
                    (usable_end - heap_start2) as usize,
                    dynamic_heap_size,
                );
                let heap_size2 = (heap_size2 / 4096) * 4096; // page-align down
                
                serial_println!("[HEAP] Fallback: using {:#x} size {} MB", heap_start2, heap_size2 / 1024 / 1024);
                memory::init_with_hhdm_dynamic(hhdm_offset, heap_start2, heap_size2);
                heap_initialized = true;
                serial_println!("[HEAP] Initialized: free={} KB", memory::heap::free() / 1024);
                
                // Initialize frame allocator
                let usable_regions: alloc::vec::Vec<memory::frame::PhysRegion> = mmap_response.entries()
                    .iter()
                    .filter(|e| e.entry_type == limine::memory_map::EntryType::USABLE)
                    .map(|e| memory::frame::PhysRegion { base: e.base, length: e.length })
                    .collect();
                memory::frame::init(&usable_regions, heap_start2, heap_size2 as u64);
            } else {
                serial_println!("[HEAP] ERROR: No usable region found for heap!");
            }
        }
    }
    
    if !heap_initialized {
        // Fallback
        serial_println!("[HEAP] Using fallback init");
        memory::init();
        crate::boot_phase!(2, "memory init");
    }
    
    // Now that heap is initialized, initialize scrollback buffer (allocates ~3MB)
    serial_println!("[FB] Initializing scrollback buffer...");
    framebuffer::init_scrollback();
    
    // Compute SHA-256 of kernel .text section for runtime integrity verification
    signature::init_integrity();
    
    // Initialize Ed25519 asymmetric signature (derives keypair from kernel digest)
    signature::init_ed25519(&[0x54, 0x72, 0x75, 0x73, 0x74, 0x4f, 0x53]); // "TrustOS" seed
    
    // Now that heap is initialized, show boot banner
    framebuffer::update_boot_splash(0, "Memory management initialized");
    framebuffer::print_boot_status("Memory management initialized", BootStatus::Ok);

    // Phase 3.5: GDT with Ring 0/3 support (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    {
        framebuffer::visual_post_code(debug::POST_GDT);
        debug::checkpoint(debug::POST_GDT, "GDT init");
        serial_println!("Initializing GDT with Ring 0/3 support...");
        gdt::init();
        framebuffer::update_boot_splash(1, "GDT initialized (Ring 0/3)");
        framebuffer::print_boot_status("GDT initialized (Ring 0/3)", BootStatus::Ok);
    }
    
    // Phase 3.51: Early interrupts (needed for page fault debugging)
    framebuffer::visual_post_code(debug::POST_IDT);
    debug::checkpoint(debug::POST_IDT, "IDT/interrupts init");
    serial_println!("Initializing early interrupts...");
    interrupts::init();
    crate::boot_phase!(3, "interrupts init");
    framebuffer::update_boot_splash(2, "Interrupts initialized");
    framebuffer::print_boot_status("Interrupts (early)", BootStatus::Ok);
    
    // Phase 3.55–3.56: x86_64-specific hardware init (CPU, ACPI, APIC, SMP)
    #[cfg(target_arch = "x86_64")]
    {
        // CPU hardware exploitation (TSC, AES-NI, SIMD, SMP)
        framebuffer::visual_post_code(debug::POST_CPU_DETECT);
        debug::checkpoint(debug::POST_CPU_DETECT, "CPU detection");
        serial_println!("Detecting CPU capabilities...");
        cpu::init();
        // Init per-CPU GS base (BSP) — required for fast current_cpu_id() via gs:[8]
        sync::percpu::init_bsp();
        // Try enabling PCID (Process-Context Identifiers): keeps kernel TLB hot
        // across CR3 reloads. Safe no-op if CPU lacks support.
        let _pcid_on = arch::memory::enable_pcid();
        framebuffer::update_boot_splash(3, "CPU capabilities detected");
        framebuffer::print_boot_status("CPU capabilities detected", BootStatus::Ok);
        
        // ACPI tables parsing
        framebuffer::visual_post_code(debug::POST_ACPI);
        debug::checkpoint(debug::POST_ACPI, "ACPI tables parsing");
        serial_println!("Parsing ACPI tables...");
        if let Some(rsdp_response) = RSDP_REQUEST.get_response() {
            let rsdp_ptr = rsdp_response.address();
            serial_println!("[DEBUG] RSDP pointer from Limine: {:#x}", rsdp_ptr as usize);
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
        
        // APIC initialization — replaces legacy PIC
        framebuffer::visual_post_code(debug::POST_APIC);
        debug::checkpoint(debug::POST_APIC, "APIC init");
        serial_println!("Initializing APIC...");
        if apic::init() {
            framebuffer::print_boot_status("APIC initialized (LAPIC + IOAPIC)", BootStatus::Ok);
        } else {
            serial_println!("[APIC] Not available, staying on legacy PIC");
            framebuffer::print_boot_status("APIC not available (legacy PIC)", BootStatus::Skip);
        }

        // HPET initialization — high-precision timer
        if acpi::hpet::init() {
            framebuffer::print_boot_status("HPET initialized", BootStatus::Ok);
        } else {
            framebuffer::print_boot_status("HPET not available", BootStatus::Skip);
        }

        // SMP initialization - Start all CPU cores!
        framebuffer::visual_post_code(debug::POST_SMP);
        debug::checkpoint(debug::POST_SMP, "SMP multi-core init");
        serial_println!("Initializing SMP...");
        cpu::smp::init();
        // [BOOT-LOOP-ISO] Per-CPU magazine cache disabled — calls current_cpu_id()
        // on every alloc, suspect for SMP-step crash.
        // memory::heap::enable_percpu_cache();
        // JARVIS trace pipeline TEMPORARILY DISABLED to isolate boot loop.
        // jarvis::trace::init();
        // let _ = jarvis::trace::pmu::init();
        serial_println!("[BOOT-CKPT] percpu_cache + jarvis::trace + AP-AVX disabled (boot-loop isolation)");
        
        if let Some(smp_response) = SMP_REQUEST.get_response() {
            let cpu_count = smp_response.cpus().len();
            serial_println!("[SMP] Found {} CPUs via Limine", cpu_count);
            
            for cpu in smp_response.cpus().iter() {
                if cpu.id != 0 {
                    serial_println!("[SMP] Starting AP {} (LAPIC ID: {})", cpu.id, cpu.lapic_id);
                    cpu.goto_address.write(cpu::smp::ap_entry);
                }
            }
            
            let mut ready_count = 1u32;
            for _ in 0..200 {
                ready_count = cpu::smp::ready_cpu_count();
                if ready_count >= cpu_count as u32 { break; }
                for _ in 0..1000 { core::hint::spin_loop(); }
            }
            
            serial_println!("[SMP] {} of {} CPUs online", ready_count, cpu_count);
            cpu::smp::set_cpu_count(cpu_count as u32);
            framebuffer::update_boot_splash(4, "SMP multi-core active");
            framebuffer::print_boot_status(&alloc::format!("SMP: {} cores active", ready_count), BootStatus::Ok);
            boot_timing!("SMP done");
        } else {
            serial_println!("[SMP] No SMP response from bootloader");
            framebuffer::print_boot_status("SMP: single core", BootStatus::Ok);
        }
    }
    
    // Phase 3.6: Paging subsystem
    framebuffer::visual_post_code(debug::POST_PAGING);
    serial_println!("Initializing paging subsystem...");
    memory::paging::init();  // Saves kernel CR3, enables NX
    framebuffer::update_boot_splash(5, "Paging & memory protection");
    framebuffer::print_boot_status("Paging initialized (NX enabled)", BootStatus::Ok);
    
    // PAT: Enable Write-Combining (WC) — standard GPU driver optimization
    #[cfg(target_arch = "x86_64")]
    let is_vbox = acpi::get_info()
        .map(|info| info.oem_id.trim().eq_ignore_ascii_case("VBOX"))
        .unwrap_or(false);
    #[cfg(not(target_arch = "x86_64"))]
    let is_vbox = false;
    if is_vbox {
        serial_println!("[PAT] Skipping Write-Combining on VirtualBox (VMSVGA compat)");
    } else {
        memory::paging::setup_pat_write_combining();
        // WC remap framebuffer IMMEDIATELY after PAT setup — before any more FB writes
        let fb_addr = framebuffer::FB_ADDR.load(core::sync::atomic::Ordering::SeqCst);
        let fb_w = framebuffer::FB_WIDTH.load(core::sync::atomic::Ordering::SeqCst) as usize;
        let fb_h = framebuffer::FB_HEIGHT.load(core::sync::atomic::Ordering::SeqCst) as usize;
        if !fb_addr.is_null() && fb_w > 0 && fb_h > 0 {
            let fb_size = fb_w * fb_h * 4;
            if memory::paging::remap_region_write_combining(fb_addr as u64, fb_size).is_ok() {
                serial_println!("[PAT] Framebuffer WC remap done (early)");
                boot_timing!("WC remap done");
            }
        }
    }
    
    // Phase 3.7: Userland support (SYSCALL/SYSRET) — x86_64 only
    #[cfg(target_arch = "x86_64")]
    {
        serial_println!("Initializing userland support...");
        userland::init_syscall_stack();
        userland::init();
        framebuffer::print_boot_status("Userland support ready", BootStatus::Ok);
    }
    
    // Phase 3.8: Thread subsystem
    serial_println!("Initializing thread subsystem...");
    thread::init();
    framebuffer::update_boot_splash(6, "Thread subsystem ready");
    framebuffer::print_boot_status("Thread subsystem ready", BootStatus::Ok);
    
    // Phase 3.9: Security subsystem (SMEP, SMAP, capabilities)
    // DISABLED: security::init() breaks RTL8169 network on B75 hardware
    // Likely SMEP CR4 write affects MMIO-mapped driver state
    // serial_println!("Initializing security subsystem...");
    // security::init();
    framebuffer::print_boot_status("Security (basic)", BootStatus::Ok);


    // ── x86_64-specific peripheral init (PS/2, CMOS, PIO-based PCI) ──
    #[cfg(target_arch = "x86_64")]
    {
    // Phase 4: Keyboard driver (interrupts already initialized early)
    framebuffer::visual_post_code(0x43); // POST 43 = keyboard init
    keyboard::init_i8042();
    serial_println!("Keyboard driver ready");
    boot_timing!("Keyboard init done");
    framebuffer::update_boot_splash(7, "Keyboard & input devices");
    framebuffer::print_boot_status("Keyboard ready", BootStatus::Ok);

    // Full boot: initialize all subsystems
    // (Set to true only for debugging boot freezes)
    
    // Phase 6: RTC (Real-Time Clock)
    const ENABLE_RTC: bool = true;
    framebuffer::visual_post_code(0x44); // POST 44 = RTC init
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

    // Phase 6.1: RNG — detect RDRAND/RDSEED hardware support
    framebuffer::visual_post_code(0x45); // POST 45 = RNG init
    rng::init();
    framebuffer::print_boot_status("RNG (CSPRNG)", BootStatus::Ok);
    
    // Phase 7: Mouse driver
    framebuffer::visual_post_code(0x46); // POST 46 = mouse init
    mouse::init();
    let (fb_width, fb_height) = framebuffer::get_dimensions();
    mouse::set_screen_size(fb_width, fb_height);
    framebuffer::update_boot_splash(8, "Mouse & touch input");
    framebuffer::print_boot_status("Mouse initialized", BootStatus::Ok);
    
    // Phase 7.1: Touch input driver
    framebuffer::visual_post_code(0x47); // POST 47 = touch init
    touch::init();
    touch::set_screen_size(fb_width, fb_height);
    framebuffer::print_boot_status("Touch input ready", BootStatus::Ok);
    
    // Phase 8-12: Subsystem toggles (enable for full functionality)
    const ENABLE_PCI: bool = true;
    const ENABLE_TASKS: bool = true;
    const ENABLE_DISK: bool = true;   // Enable for virtio-blk persistence
    const ENABLE_DRIVERS: bool = true;
    const ENABLE_NETWORK: bool = true;

    // Phase 8: PCI Bus Enumeration (BEFORE device drivers)
    framebuffer::visual_post_code(debug::POST_PCI);
    debug::checkpoint(debug::POST_PCI, "PCI bus enumeration");
    serial_println!("[PHASE] PCI init start");
    framebuffer::print_boot_status("PCI bus scanning...", BootStatus::Info);
    if ENABLE_PCI {
        pci::init();
        crate::boot_phase!(5, "pci enumerated");
        framebuffer::update_boot_splash(9, "PCI bus enumeration");
        framebuffer::print_boot_status("PCI bus scanned", BootStatus::Ok);
        boot_timing!("PCI scan done");
    } else {
        framebuffer::print_boot_status("PCI disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] PCI init done");
    
    // Phase 9: Task scheduler
    serial_println!("[PHASE] Task scheduler init start");
    if ENABLE_TASKS {
        task::init();
        scheduler::init();
        framebuffer::update_boot_splash(10, "Task scheduler");
        framebuffer::print_boot_status("Task scheduler ready", BootStatus::Ok);
    } else {
        framebuffer::print_boot_status("Task scheduler disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Task scheduler init done");
    
    // Phase 10: Disk I/O
    framebuffer::visual_post_code(debug::POST_DISK);
    debug::checkpoint(debug::POST_DISK, "Disk I/O init");
    serial_println!("[PHASE] Disk init start");
    framebuffer::print_boot_status("Disk subsystem...", BootStatus::Info);
    if ENABLE_DISK {
        // Try NVMe first (fastest, real hardware SSD)
        let nvme_devs: alloc::vec::Vec<_> = pci::get_devices().iter()
            .filter(|d| d.class_code == 0x01 && d.subclass == 0x08)
            .cloned()
            .collect();
        
        if !nvme_devs.is_empty() {
            match nvme::init(&nvme_devs[0]) {
                Ok(()) => {
                    if let Some((model, _serial, size, lba_sz)) = nvme::get_info() {
                        let mb = (size * lba_sz as u64) / (1024 * 1024);
                        framebuffer::print_boot_status(
                            &alloc::format!("NVMe: {} ({} MB)", model, mb), BootStatus::Ok);
                    }
                }
                Err(e) => {
                    crate::log_warn!("[DISK] NVMe init failed: {}", e);
                }
            }
        }
        
        // Try virtio-blk (for QEMU guests without NVMe)
        if !nvme::is_initialized() {
            let blk_devs: alloc::vec::Vec<_> = pci::get_devices().iter()
                .filter(|d| d.vendor_id == 0x1AF4 && d.device_id == 0x1001)
                .cloned()
                .collect();
            
            if !blk_devs.is_empty() {
                if let Err(e) = virtio_blk::init(&blk_devs[0]) {
                    crate::log_warn!("[DISK] virtio-blk init failed: {}", e);
                    disk::init();
                } else {
                    framebuffer::print_boot_status(&alloc::format!("virtio-blk: {} MB storage", 
                        (virtio_blk::capacity() * 512) / (1024 * 1024)), BootStatus::Ok);
                }
            } else {
                // No NVMe, no virtio-blk → RAM disk
                disk::init();
            }
        }
        
        if disk::is_available() || virtio_blk::is_initialized() || nvme::is_initialized() {
            framebuffer::print_boot_status("Disk driver ready", BootStatus::Ok);
        } else {
            framebuffer::print_boot_status("No disk detected", BootStatus::Skip);
        }
    } else {
        framebuffer::print_boot_status("Disk disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Disk init done");
    
    // Phase 11: Driver Framework
    framebuffer::visual_post_code(0x48); // POST 48 = driver framework
    serial_println!("[PHASE] Driver framework init start");
    if ENABLE_DRIVERS {
        drivers::init();
        framebuffer::visual_post_code(0x49); // POST 49 = storage probe
        serial_println!("[PHASE] Storage probe start");
        // Probe storage controllers (AHCI, IDE)
        drivers::probe_storage();
        serial_println!("[PHASE] Storage probe done");
        framebuffer::visual_post_code(0x4A); // POST 4A = driver framework done
        framebuffer::update_boot_splash(12, "Driver framework");
        framebuffer::print_boot_status("Driver framework initialized", BootStatus::Ok);
        if drivers::has_storage() {
            framebuffer::print_boot_status("Persistent storage detected", BootStatus::Ok);
        }
    } else {
        framebuffer::print_boot_status("Driver framework disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Driver framework init done");
    
    // Phase 11b: VirtIO GPU
    framebuffer::visual_post_code(0x4B); // POST 4B = VirtIO GPU
    serial_println!("[PHASE] VirtIO GPU init start");
    framebuffer::print_boot_status("VirtIO GPU...", BootStatus::Info);
    drivers::virtio_gpu::init_from_pci().ok();
    if drivers::virtio_gpu::is_available() {
        framebuffer::print_boot_status(&alloc::format!("VirtIO GPU: {}", drivers::virtio_gpu::info_string()), BootStatus::Ok);
    } else {
        framebuffer::print_boot_status("VirtIO GPU: not found (fallback framebuffer)", BootStatus::Skip);
    }
    serial_println!("[PHASE] VirtIO GPU init done");
    
    // Phase 11c: AMD GPU — DEFERRED to after network init (GPU MMIO mapping breaks RTL8168 on B75)
    // See Phase 12+ for AMD GPU init
    
    // Phase 11d: NVIDIA GPU (NV50/Tesla — ThinkPad T61)
    framebuffer::visual_post_code(0x4D); // POST 4D = NVIDIA GPU
    serial_println!("[PHASE] NVIDIA GPU init start");
    framebuffer::print_boot_status("NVIDIA GPU...", BootStatus::Info);
    drivers::nvidia::init();
    if drivers::nvidia::is_detected() {
        framebuffer::print_boot_status(&alloc::format!("NVIDIA GPU: {}", drivers::nvidia::summary()), BootStatus::Ok);
    } else {
        framebuffer::print_boot_status("NVIDIA GPU: not found", BootStatus::Skip);
    }
    serial_println!("[PHASE] NVIDIA GPU init done");
    
    // WC remap already done early (after PAT setup) — skip duplicate
    
    // Phase 12: Network (with universal driver system)
    framebuffer::visual_post_code(debug::POST_NETWORK);
    debug::checkpoint(debug::POST_NETWORK, "Network init");
    serial_println!("[PHASE] Network init start");
    framebuffer::print_boot_status("Network subsystem...", BootStatus::Info);
    if ENABLE_NETWORK {
        network::init();
        if network::is_available() {
            // Display detected platform
            let platform = network::get_platform();
            framebuffer::print_boot_status(&alloc::format!("Platform: {}", platform), BootStatus::Info);
            
            // Auto-probe network drivers in two passes:
            // Pass 1: WiFi — DEFERRED to first `wifi` command (map_bar0 can triple-fault on bare metal)
            // Pass 2: Ethernet (probe + start)
            framebuffer::print_boot_status("PCI scan...", BootStatus::Info);
            let devs = pci::find_by_class(pci::class::NETWORK);
            framebuffer::print_boot_status(&alloc::format!("Found {} network devices", devs.len()), BootStatus::Info);
            
            // WiFi: just remember PCI info, don't touch hardware yet
            // The actual probe (map_bar0 + MMIO) happens on first `wifi` shell command
            for dev in &devs {
                if dev.vendor_id == 0x8086 
                    && (dev.subclass == 0x80 || dev.class_code == 0x0D 
                        || drivers::net::iwl4965::IWL4965_DEVICE_IDS.contains(&dev.device_id))
                {
                    framebuffer::print_boot_status(
                        &alloc::format!("WiFi detected: {:04X}:{:04X} (deferred)", dev.vendor_id, dev.device_id),
                        BootStatus::Info);
                    // Store PCI location for lazy probe later
                    drivers::net::wifi::set_deferred_pci(dev.bus, dev.device, dev.function);
                }
            }
            
            // Pass 2: Probe Ethernet driver (first match only)
            println!("  Ethernet probe starting...");
            framebuffer::print_boot_status("Ethernet probe...", BootStatus::Info);
            // BIG PCI banner at top of screen so we can see NICs without keyboard
            {
                let mut y = 4u32;
                framebuffer::draw_text_centered("=== PCI NIC SCAN ===", y, 0xFFFF00);
                y += 18;
                for dev in &devs {
                    let line = alloc::format!(
                        "{:04X}:{:04X} cls={:02X}.{:02X} bar0={:#010X}",
                        dev.vendor_id, dev.device_id, dev.class_code, dev.subclass, dev.bar[0]
                    );
                    framebuffer::draw_text_centered(&line, y, 0x00FFFF);
                    y += 16;
                }
                if devs.is_empty() {
                    framebuffer::draw_text_centered("(no NIC found)", y, 0xFF0000);
                }
            }
            for dev in &devs {
                framebuffer::print_boot_status(
                    &alloc::format!("NIC PCI {:04X}:{:04X} class {:02X}.{:02X}",
                        dev.vendor_id, dev.device_id, dev.class_code, dev.subclass),
                    BootStatus::Info);
                serial_println!("[NET] PCI NIC: {:04X}:{:04X} bus={} dev={} fn={} BAR0={:#x}",
                    dev.vendor_id, dev.device_id, dev.bus, dev.device, dev.function, dev.bar[0]);
                if drivers::net::probe_device(dev) {
                    network::update_mac_from_driver();
                    let driver_name = if dev.vendor_id == 0x1AF4 { "virtio-net" } 
                        else if dev.vendor_id == 0x8086 { "e1000" }
                        else if dev.vendor_id == 0x10EC && (dev.device_id == 0x8139 || dev.device_id == 0x8138) { "rtl8139" }
                        else if dev.vendor_id == 0x10EC { "rtl8169" }
                        else { "unknown" };
                    framebuffer::print_boot_status(&alloc::format!("Network driver: {}", driver_name), BootStatus::Ok);
                    break;
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

            if !drivers::net::has_driver() && !crate::virtio_net::is_initialized() {
                framebuffer::print_boot_status("NO NETWORK DRIVER LOADED", BootStatus::Fail);
                serial_println!("[NET] WARNING: No network driver matched any PCI device!");
                framebuffer::draw_text_centered("!! NO NIC DRIVER LOADED !!",
                    4 + 18 + (devs.len() as u32 + 1) * 16, 0xFF0000);
            } else {
                let mac = drivers::net::get_mac().unwrap_or([0;6]);
                let line = alloc::format!(
                    "DRIVER OK  MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    mac[0],mac[1],mac[2],mac[3],mac[4],mac[5]);
                framebuffer::draw_text_centered(&line,
                    4 + 18 + (devs.len() as u32 + 1) * 16, 0x00FF00);
            }
            
            framebuffer::update_boot_splash(14, "Network stack ready");
            framebuffer::print_boot_status("Network ready", BootStatus::Ok);
            
            // Start network stack
            netstack::dhcp::start();
            netstack::ipv6::init();
            netstack::tcp::init_isn_secret();
            netstack::mark_initialized();

            // Auto-start remote shell: wait for DHCP, then enable remoteshell + netconsole
            // This allows headless operation (no screen/keyboard needed)
            if drivers::net::has_driver() {
                serial_println!("[NET] Waiting for DHCP (up to 800ms)...");
                framebuffer::print_boot_status("DHCP...", BootStatus::Info);
                let dhcp_start = logger::get_ticks();
                let mut spin_count = 0u64;
                loop {
                    for _ in 0..30 { netstack::poll(); }
                    if network::get_ipv4_config().is_some() { break; }
                    let elapsed = logger::get_ticks().saturating_sub(dhcp_start);
                    if elapsed > 800 { break; }
                    // Fallback: if ticks aren't advancing (no timer IRQ), use spin count
                    spin_count += 1;
                    if spin_count > 800_000 { break; }
                    for _ in 0..1000 { core::hint::spin_loop(); }
                }
                // Fallback static IP if DHCP failed
                if network::get_ipv4_config().is_none() {
                    serial_println!("[NET] DHCP timeout — applying static 10.0.0.100/24");
                    network::set_ipv4_config(
                        network::Ipv4Address::new(10, 0, 0, 100),
                        network::Ipv4Address::new(255, 255, 255, 0),
                        Some(network::Ipv4Address::new(10, 0, 0, 1)),
                    );
                }
                if let Some((ip, _, _)) = network::get_ipv4_config() {
                    let b = ip.as_bytes();
                    serial_println!("[NET] IP: {}.{}.{}.{}", b[0], b[1], b[2], b[3]);
                    framebuffer::print_boot_status(
                        &alloc::format!("IP: {}.{}.{}.{}", b[0], b[1], b[2], b[3]),
                        BootStatus::Ok,
                    );
                }
                // Start remote shell + netconsole (headless access)
                debug::remoteshell::start();
                framebuffer::print_boot_status(
                    &alloc::format!("Remote shell: UDP port {}", debug::remoteshell::LISTEN_PORT),
                    BootStatus::Ok,
                );
                // Auto-start netconsole on broadcast
                if let Some((src_ip, mask, _)) = network::get_ipv4_config() {
                    let s = src_ip.as_bytes();
                    let m = mask.as_bytes();
                    let bcast = [s[0] | !m[0], s[1] | !m[1], s[2] | !m[2], s[3] | !m[3]];
                    debug::netconsole::start(bcast, debug::netconsole::DEFAULT_PORT);
                    framebuffer::print_boot_status("Netconsole: broadcast", BootStatus::Ok);
                }
            }
        }
    } else {
        framebuffer::print_boot_status("Network disabled", BootStatus::Skip);
    }
    serial_println!("[PHASE] Network init done");
    boot_timing!("Network init done");

    // Phase 11c (deferred): AMD GPU — init AFTER network to avoid MMIO mapping breaking RTL8168
    #[cfg(feature = "amdgpu")]
    {
    framebuffer::visual_post_code(0x4C); // POST 4C = AMD GPU
    serial_println!("[PHASE] AMD GPU init start");
    framebuffer::print_boot_status("AMD GPU...", BootStatus::Info);
    debug::tco_watchdog_pet(); // pet before potentially long GPU init
    drivers::amdgpu::init();
    crate::boot_phase!(10, "amdgpu init");
    if drivers::amdgpu::is_detected() {
        framebuffer::print_boot_status("AMD GPU: detected", BootStatus::Ok);
    } else {
        framebuffer::print_boot_status("AMD GPU: not found (VM or non-AMD)", BootStatus::Skip);
    }
    serial_println!("[PHASE] AMD GPU init done");
    }

    } // end #[cfg(target_arch = "x86_64")] block

    // ── aarch64: skip x86 peripherals, use minimal boot ──
    #[cfg(target_arch = "aarch64")]
    {
        serial_println!("[AARCH64] Skipping x86 peripherals (keyboard, RTC, mouse, PCI)");
        framebuffer::print_boot_status("Keyboard: N/A (serial)", BootStatus::Skip);
        framebuffer::print_boot_status("RTC: N/A", BootStatus::Skip);
        framebuffer::print_boot_status("PCI: N/A (ECAM not yet)", BootStatus::Skip);
        framebuffer::print_boot_status("Network: N/A", BootStatus::Skip);
    }

    // ========================================================================
    // Phase 12: VFS Initialization
    // ========================================================================
    debug::tco_watchdog_pet(); // pet after GPU init
    framebuffer::visual_post_code(debug::POST_VFS);
    debug::checkpoint(debug::POST_VFS, "VFS init");
    serial_println!("[PHASE] VFS init start");
    vfs::init();
    serial_println!("[PHASE] VFS init done");
    framebuffer::update_boot_splash(15, "Virtual filesystem (VFS)");
    framebuffer::print_boot_status("Virtual filesystem ready", BootStatus::Ok);
    
    // ========================================================================
    // Phase 12a: Linux Subsystem (TSL) - x86_64 only (uses hypervisor)
    // ========================================================================
    #[cfg(target_arch = "x86_64")]
    {
        serial_println!("[PHASE] Linux Subsystem init start");
        if let Some(module_response) = MODULE_REQUEST.get_response() {
            let modules = module_response.modules();
            serial_println!("[TSL] Found {} boot modules", modules.len());
            
            let mut kernel_data: Option<&'static [u8]> = None;
            let mut initramfs_data: Option<&'static [u8]> = None;
            
            for module in modules {
                let cmdline_bytes = module.cmdline();
                let cmdline = core::str::from_utf8(cmdline_bytes).unwrap_or("unknown");
                let path = module.path().to_str().unwrap_or("unknown");
                let addr = module.addr();
                let size = module.size() as usize;
                
                serial_println!("[TSL] Module: {} ({}), {} bytes at {:p}", 
                    cmdline, path, size, addr);
                
                let data = unsafe { core::slice::from_raw_parts(addr, size) };
                
                if cmdline.contains("linux-kernel") || path.contains("bzImage") {
                    kernel_data = Some(data);
                    serial_println!("[TSL] Linux kernel loaded: {} bytes", size);
                } else if cmdline.contains("linux-initramfs") || path.contains("initramfs") {
                    initramfs_data = Some(data);
                    serial_println!("[TSL] Initramfs loaded: {} bytes", size);
                } else if cmdline.contains("jarvis-brain") || path.contains("jarvis_pretrained") {
                    // Save pointer — RamFS not yet initialized, will copy after Phase 16
                    serial_println!("[JARVIS] Boot module: {} bytes brain weights (deferred to RamFS)", size);
                    unsafe { JARVIS_BRAIN_MODULE = Some(data); }
                } else if cmdline.contains("iwlwifi") || path.contains("iwlwifi") || path.contains(".ucode") {
                    // WiFi firmware module — store for driver initialization
                    serial_println!("[WIFI] Firmware module: {} bytes at {:p}", size, addr);
                    let fw_vec = alloc::vec::Vec::from(data);
                    drivers::net::iwl4965::set_firmware_data(&fw_vec);
                }
            }
            
            if let (Some(kernel), Some(initramfs)) = (kernel_data, initramfs_data) {
                hypervisor::linux_subsystem::subsystem().set_embedded_images(kernel, initramfs);
                framebuffer::print_boot_status("Linux Subsystem (TSL) ready", BootStatus::Ok);
            } else {
                framebuffer::print_boot_status("Linux Subsystem (partial)", BootStatus::Skip);
            }
        } else {
            framebuffer::print_boot_status("Linux Subsystem (no modules)", BootStatus::Skip);
        }
        serial_println!("[PHASE] Linux Subsystem init done");
    }
    
    // ========================================================================
    // Phase 12b: File Associations
    // ========================================================================
    file_assoc::init();
    framebuffer::print_boot_status("File associations ready", BootStatus::Ok);
    
    // ========================================================================
    // Phase 13: Process Manager
    // ========================================================================
    debug::tco_watchdog_pet(); // pet after TSL/VFS init
    debug::checkpoint(debug::POST_PROCESS, "Process manager init");
    serial_println!("[PHASE] Process manager init start");
    process::init();
    framebuffer::update_boot_splash(17, "Process manager");
    framebuffer::print_boot_status("Process manager ready", BootStatus::Ok);
    serial_println!("[PHASE] Process manager init done");
    
    // ========================================================================
    // Phase 14: Authentication System
    // ========================================================================
    serial_println!("[PHASE] Auth system init");
    auth::init();
    auth::create_etc_files();
    framebuffer::update_boot_splash(18, "Authentication system");
    framebuffer::print_boot_status("Authentication ready", BootStatus::Ok);
    
    // Phase 14b: TTY / PTY subsystem
    serial_println!("[PHASE] TTY/PTY init");
    tty::init();
    pty::init();
    framebuffer::print_boot_status("TTY/PTY subsystem ready", BootStatus::Ok);
    
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
    framebuffer::update_boot_splash(19, "RAM filesystem");
    // Create standard directories
    ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/tmp");
        let _ = fs.mkdir("/var");
        let _ = fs.mkdir("/home");
        let _ = fs.mkdir("/bin");
        let _ = fs.mkdir("/usr");
        let _ = fs.mkdir("/etc");
    });

    debug::tco_watchdog_pet(); // pet before deferred GPU firmware + JARVIS load
    // Deferred AMD GPU firmware: populate ramfs now that it's available
    #[cfg(all(target_arch = "x86_64", feature = "amdgpu"))]
    if drivers::amdgpu::is_detected() {
        if let Some(info) = drivers::amdgpu::get_info() {
            if info.gpu_gen != drivers::amdgpu::GpuGen::Polaris {
                serial_println!("[AMDGPU] Deferred firmware load (ramfs now ready)");
                drivers::amdgpu::firmware::reload(info.mmio_base_virt);
            }
        }
    }

    // Copy deferred JARVIS brain module into RamFS (saved during Phase 12a)
    if let Some(brain_data) = unsafe { JARVIS_BRAIN_MODULE.take() } {
        serial_println!("[JARVIS] Copying {} KB brain weights to RamFS...", brain_data.len() / 1024);
        ramfs::with_fs(|fs| {
            let _ = fs.mkdir("/jarvis");
        });
        ramfs::with_fs(|fs| {
            let _ = fs.touch("/jarvis/weights.bin");
            match fs.write_file("/jarvis/weights.bin", brain_data) {
                Ok(_) => serial_println!("[JARVIS] Brain weights cached to /jarvis/weights.bin ({} KB)", brain_data.len() / 1024),
                Err(_) => serial_println!("[JARVIS] WARNING: Failed to cache brain to RamFS"),
            }
        });
    }

    framebuffer::print_boot_status("RAM filesystem ready", BootStatus::Ok);

    // Load user/group data from /etc files (if persisted from previous boot)
    auth::load_from_filesystem();

    // ========================================================================
    // Phase 17: Persistence System
    // ========================================================================
    serial_println!("[PHASE] Persistence init");
    persistence::init();
    framebuffer::update_boot_splash(20, "Persistence layer");
    framebuffer::print_boot_status("Persistence system ready", BootStatus::Ok);

    // ========================================================================
    // Phase 18: Web Sandbox
    // ========================================================================
    serial_println!("[PHASE] Web Sandbox init");
    sandbox::init();
    framebuffer::print_boot_status("Web Sandbox ready", BootStatus::Ok);

    // Phase 18b: Container Daemon (auto-starts default web container)
    serial_println!("[PHASE] Container daemon boot");
    sandbox::container::boot_daemon();
    framebuffer::update_boot_splash(21, "System ready!");
    framebuffer::print_boot_status("Container daemon ready", BootStatus::Ok);

    // === BOOT SPLASH: Fast fade out ===
    boot_timing!("Pre-fade");
    framebuffer::fade_out_splash();
    boot_timing!("Boot complete");

    // Final boot summary
    println!();
    framebuffer::draw_separator(framebuffer::get_cursor().1 as u32 * 16, framebuffer::COLOR_GREEN);
    println!();
    println_color!(framebuffer::COLOR_BRIGHT_GREEN, "  System ready - TRust-OS v0.2.0");
    println_color!(framebuffer::COLOR_GREEN, "  Type 'desktop' to launch the desktop, or use shell commands.");
    println!();

    // Check for saved data and ask user
    if persistence::is_available() {
        persistence::prompt_restore();
    }

    // Auto-login as root for now (development mode)
    auth::auto_login_root();

    // Marionet auto-dump + crypto self-tests: deferred to shell commands
    // Use `marionet dump` and `crypto-test` to run manually

    // Start shell (runs forever)
    debug::tco_watchdog_pet(); // pet just before shell — from here poll() takes over
    framebuffer::visual_post_code(debug::POST_SHELL_READY);
    debug::checkpoint(debug::POST_SHELL_READY, "Shell ready — boot complete");
    serial_println!("Starting shell...");

    // Auto-mount SSD if available (AHCI port 5, LBA 2048 — LBA 0 overwritten by BIOS)
    ssd_autoexec();

    // ── JARVIS Birth System ─────────────────────────────────────────
    // Try to resume JARVIS from SSD (waking up from previous session).
    // If no saved state exists, this is the FIRST BIRTH — JARVIS starts as Fetus.
    // Either way, background training starts automatically.
    #[cfg(feature = "jarvis")]
    jarvis_birth();

    // If booted in install mode, run the installer wizard
    if boot_mode() == BootMode::Install {
        // Enable keyboard interrupts before wizard (normally done in shell::run)
        crate::interrupts::set_bootstrap_ready(true);
        serial_println!("[BOOT] Install mode — launching installer wizard");
        installer::run_wizard();
    }

    // ── CoreMark: full boot done (network + netconsole active), run benchmark ──
    #[cfg(feature = "coremark")]
    coremark::run();

    // ── Headless VM self-test (CI / `qemu-selftest.ps1`) ─────────────
    // Runs WiFi crypto + frame fixtures + WPA2 4-way handshake against
    // the in-tree MockAccessPoint, prints a single PASS/FAIL marker on
    // serial COM1 the host script can grep for, then continues to the
    // shell as normal.
    #[cfg(feature = "vm-selftest")]
    vm_selftest();

    shell::run();
}

// ============================================================================
// JARVIS Birth System
// ============================================================================

/// JARVIS Birth — Resume or first-boot the neural brain.
/// Called once after SSD is mounted, before the shell starts.
#[cfg(feature = "jarvis")]
fn jarvis_birth() {
    serial_println!("[JARVIS-BIRTH] ═══════════════════════════════════════════");
    serial_println!("[JARVIS-BIRTH]        JARVIS LIFECYCLE SYSTEM             ");
    serial_println!("[JARVIS-BIRTH] ═══════════════════════════════════════════");

    // Try to resume from SSD (previous session's saved state)
    let resumed = jarvis::resume_from_ssd();

    if resumed {
        // JARVIS woke up — continue growing
        serial_println!("[JARVIS-BIRTH] ★ Welcome back, JARVIS ★");
        framebuffer::print_boot_status("JARVIS resumed from SSD", framebuffer::BootStatus::Ok);
    } else {
        // First birth — JARVIS starts as Fetus with fresh random weights
        serial_println!("[JARVIS-BIRTH] ★ FIRST BIRTH — Welcome to the world, JARVIS ★");
        framebuffer::print_boot_status("JARVIS first birth!", framebuffer::BootStatus::Ok);

        // Ensure JARVIS is initialized with random weights if not loaded via boot module
        if !jarvis::has_full_brain() {
            jarvis::init_random();
            serial_println!("[JARVIS-BIRTH] Random brain initialized for first training");
        }
    }

    // Don't auto-start training at boot — forward+backward takes ~2 min per step
    // on the G4400 (no AVX) and starves the network. Start manually with: jarvis train
    // jarvis::auto_start_background_training();

    serial_println!("[JARVIS-BIRTH] ═══════════════════════════════════════════");
    serial_println!("[JARVIS-BIRTH]   JARVIS is alive. Stage: {}",
        jarvis::developmental::current_stage().name());
    serial_println!("[JARVIS-BIRTH] ═══════════════════════════════════════════");
}

// ============================================================================
// Core Functions
// ============================================================================

/// Halt the CPU in an infinite loop
/// Used when kernel cannot continue or has nothing to do
fn halt_loop() -> ! {
    arch::halt_loop()
}

/// Headless VM self-test — runs the WiFi stack offline harness and prints
/// a single line on COM1 (`[VM-SELFTEST] PASS` or `[VM-SELFTEST] FAIL: …`)
/// that the host-side `qemu-selftest.ps1` script greps for.
///
/// Does not panic on failure: the shell still starts so a developer who
/// boots a `vm-selftest` build interactively can keep using it.
#[cfg(feature = "vm-selftest")]
fn vm_selftest() {
    serial_println!("[VM-SELFTEST] start");

    // Stage 1 — 802.11 frame parsers.
    if let Err(e) = crate::netstack::wifi::fixtures::run_parser_fixtures() {
        serial_println!("[VM-SELFTEST] FAIL: parsers: {}", e);
        return;
    }
    serial_println!("[VM-SELFTEST] parsers ok");

    // Stage 2 — WPA2-PSK 4-way handshake STA <-> mock AP, with CCMP roundtrip
    // on the negotiated Temporal Key.
    let sta_mac = [0x02, 0x11, 0x22, 0x33, 0x44, 0x55];
    let bssid   = [0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    if let Err(e) = run_wpa2_then_ccmp(sta_mac, bssid) {
        serial_println!("[VM-SELFTEST] FAIL: handshake/ccmp: {}", e);
        return;
    }
    serial_println!("[VM-SELFTEST] handshake ok");
    serial_println!("[VM-SELFTEST] ccmp ok");

    // Stage 3 — Standalone CCM round-trip + tamper detection.
    if let Err(e) = ccmp_smoke_tests() {
        serial_println!("[VM-SELFTEST] FAIL: ccmp-smoke: {}", e);
        return;
    }
    serial_println!("[VM-SELFTEST] ccmp-smoke ok");

    serial_println!("[VM-SELFTEST] PASS");
}

#[cfg(feature = "vm-selftest")]
fn run_wpa2_then_ccmp(sta_mac: [u8; 6], bssid: [u8; 6]) -> Result<(), &'static str> {
    use crate::netstack::wifi::mock_ap::MockAccessPoint;
    use crate::netstack::wifi::supplicant::{Supplicant, AuthMethod};

    let ssid = "TrustOS-AP";
    let pass = "trustos-test-pass";

    let mut ap = MockAccessPoint::new(ssid, bssid, pass);
    let mut sta = Supplicant::new(
        AuthMethod::Wpa2Psk,
        alloc::string::String::from(ssid),
        alloc::string::String::from(pass),
        bssid,
        sta_mac,
    );
    sta.start().map_err(|_| "supplicant.start")?;
    let anonce = ap.build_msg1();
    let msg2 = sta.handle_msg1(anonce).map_err(|_| "msg2 build")?;
    let msg3 = ap.handle_msg2(sta_mac, &msg2).map_err(|_| "ap msg2")?;
    let msg4 = sta.handle_msg3(&msg3).map_err(|_| "sta msg3")?;
    ap.handle_msg4(&msg4).map_err(|_| "ap msg4")?;
    if !sta.is_connected() { return Err("sta not connected"); }

    // Both sides must have derived the same TK.
    let sta_tk = sta.ptk().ok_or("sta no ptk")?.tk;
    let ap_tk = *ap.tk().ok_or("ap no tk")?;
    if sta_tk != ap_tk { return Err("TK mismatch"); }

    // Encrypt a frame on STA side, decrypt on AP side using the negotiated TK.
    let pn = [0u8, 0, 0, 0, 0, 1]; // PN = 1, big-endian
    let nonce = crate::crypto::ccmp::build_ccmp_nonce(0, &sta_mac, &pn);
    let aad = b"\x08\x41\x00\x00"; // dummy MAC header AAD
    let payload = b"trustos -> ap (ccmp ok)";
    let ct = crate::crypto::ccmp::ccm_encrypt(&sta_tk, &nonce, aad, payload)
        .map_err(|_| "ccm encrypt")?;
    let pt = crate::crypto::ccmp::ccm_decrypt(&ap_tk, &nonce, aad, &ct)
        .map_err(|_| "ccm decrypt")?;
    if pt.as_slice() != payload { return Err("decrypted payload mismatch"); }
    Ok(())
}

#[cfg(feature = "vm-selftest")]
fn ccmp_smoke_tests() -> Result<(), &'static str> {
    use crate::crypto::ccmp::{ccm_encrypt, ccm_decrypt, CcmpError};

    // Round-trip with arbitrary data.
    let key = [0x42u8; 16];
    let nonce = [0x10u8; 13];
    let aad = b"hdr";
    let pt = b"hello ccmp world";
    let ct = ccm_encrypt(&key, &nonce, aad, pt).map_err(|_| "encrypt")?;
    let dec = ccm_decrypt(&key, &nonce, aad, &ct).map_err(|_| "decrypt")?;
    if dec.as_slice() != pt { return Err("plaintext mismatch"); }

    // MIC tamper must be rejected.
    let mut ct_bad = ct.clone();
    let last = ct_bad.len() - 1;
    ct_bad[last] ^= 0x01;
    if !matches!(ccm_decrypt(&key, &nonce, aad, &ct_bad), Err(CcmpError::MicMismatch)) {
        return Err("tamper not detected");
    }
    Ok(())
}

/// Auto-mount SSD and execute /mnt/sda1/autoexec.sh + load training.cfg
fn ssd_autoexec() {
    use alloc::sync::Arc;

    const SSD_PATH: &str = "/mnt/sda1";
    const AUTOEXEC: &str = "/mnt/sda1/autoexec.sh";
    const TRAINING_CFG: &str = "/mnt/sda1/training.cfg";

    // Check if already mounted (use mount list, not stat — stat can be fooled by empty dirs)
    let already = vfs::list_mounts().iter().any(|(p, _)| p == SSD_PATH);
    if already {
        serial_println!("[SSD] Already mounted at {}", SSD_PATH);
    } else {
        serial_println!("[SSD] Auto-mounting AHCI port 5 (LBA 2048) at {}", SSD_PATH);
        let _ = vfs::mkdir("/mnt");
        let _ = vfs::mkdir(SSD_PATH);

        // Retry up to 3 times — SSD may need time to spin up after power-on
        let mut mounted = false;
        for attempt in 0..3u8 {
            let reader = Arc::new(vfs::fat32::AhciBlockReader::new(5, 2048));
            match vfs::fat32::Fat32Fs::mount(reader) {
                Ok(fs) => {
                    match vfs::mount(SSD_PATH, Arc::new(fs)) {
                        Ok(()) => {
                            serial_println!("[SSD] Mounted FAT32 at {} (attempt {})", SSD_PATH, attempt + 1);
                            framebuffer::print_boot_status("SSD mounted (FAT32)", framebuffer::BootStatus::Ok);
                            mounted = true;
                            break;
                        }
                        Err(e) => {
                            serial_println!("[SSD] VFS mount error: {:?}", e);
                            break; // VFS error won't fix on retry
                        }
                    }
                }
                Err(e) => {
                    serial_println!("[SSD] FAT32 mount attempt {} failed: {:?}", attempt + 1, e);
                    if attempt < 2 {
                        // Brief spin-wait, but keep watchdog/network alive. Under TCG,
                        // a raw 600M-cycle spin can exceed the TCO watchdog window.
                        for i in 0..20_000u32 {
                            if i % 1000 == 0 {
                                debug::tco_watchdog_pet();
                                #[cfg(feature = "netstack")]
                                crate::netstack::poll();
                            }
                            for _ in 0..1000 {
                                core::hint::spin_loop();
                            }
                        }
                    }
                }
            }
        }
        if !mounted {
            serial_println!("[SSD] All mount attempts failed — skipping autoexec");
            framebuffer::print_boot_status("SSD mount failed", framebuffer::BootStatus::Fail);
            return;
        }
    }

    // Load training.cfg → apply to training loop atomics
    if let Ok(data) = vfs::read_file(TRAINING_CFG) {
        if let Ok(text) = core::str::from_utf8(&data) {
            serial_println!("[SSD] Loading {}", TRAINING_CFG);
            apply_training_config(text);
        }
    }

    // Execute autoexec.sh
    if let Ok(data) = vfs::read_file(AUTOEXEC) {
        if let Ok(script) = core::str::from_utf8(&data) {
            serial_println!("[SSD] Running {}", AUTOEXEC);
            framebuffer::print_boot_status("Running autoexec.sh", framebuffer::BootStatus::Ok);
            for line in script.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                serial_println!("[autoexec] {}", trimmed);
                shell::execute_command(trimmed);
            }
        }
    }
}

/// Parse key=value config and apply to training loop parameters
pub fn apply_training_config(text: &str) {
    use core::sync::atomic::Ordering;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim();
            let val = val.trim();
            match key {
                #[cfg(feature = "jarvis")]
                "train_max_seq" => {
                    if let Ok(v) = val.parse::<usize>() {
                        // Store in a global atomic for training_tick to read
                        crate::jarvis::TRAIN_MAX_SEQ_CFG.store(v, Ordering::Release);
                        serial_println!("[cfg] train_max_seq = {}", v);
                    }
                }
                #[cfg(feature = "jarvis")]
                "checkpoint_every" => {
                    if let Ok(v) = val.parse::<u32>() {
                        crate::jarvis::training_loop::CHECKPOINT_EVERY.store(v, Ordering::Release);
                        serial_println!("[cfg] checkpoint_every = {}", v);
                    }
                }
                #[cfg(feature = "jarvis")]
                "lr_max" | "lr" => {
                    if let Ok(v) = val.parse::<f32>() {
                        crate::jarvis::LR_MAX_CFG.store(v.to_bits(), Ordering::Release);
                        serial_println!("[cfg] lr_max = {}", v);
                    }
                }
                #[cfg(feature = "jarvis")]
                "lr_min" => {
                    if let Ok(v) = val.parse::<f32>() {
                        crate::jarvis::LR_MIN_CFG.store(v.to_bits(), Ordering::Release);
                        serial_println!("[cfg] lr_min = {}", v);
                    }
                }
                #[cfg(feature = "jarvis")]
                "epochs" => {
                    if let Ok(v) = val.parse::<u32>() {
                        crate::jarvis::EPOCHS_CFG.store(v, Ordering::Release);
                        serial_println!("[cfg] epochs = {}", v);
                    }
                }
                #[cfg(feature = "jarvis")]
                "early_stop" => {
                    let v = matches!(val, "true" | "1" | "yes");
                    crate::jarvis::EARLY_STOP_CFG.store(v, Ordering::Release);
                    serial_println!("[cfg] early_stop = {}", v);
                }
                _ => {
                    // Set as shell variable
                    shell::scripting::set_var(key, val);
                    serial_println!("[cfg] ${}={}", key, val);
                }
            }
        }
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
    debug::set_panicked();
    crate::apic::watchdog_arm(10_000);
    serial_println!("[ALLOC_ERROR] Auto-reboot armed: APIC=10s, TCO=~30s");
    halt_loop();
}

/// Panic handler - called on unrecoverable errors
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // POST code 0xFF = panic
    debug::post_code(debug::POST_PANIC);
    
    // Full crash dump to serial (registers, backtrace, stack, checkpoints)
    debug::panic_dump();
    
    // Also print the panic message itself
    serial_println!("\n!!! KERNEL PANIC !!!");
    serial_println!("{}", info);
    
    // Best-effort: send panic info via UDP broadcast (netconsole port 6666)
    // Guard: only if netstack is initialized to avoid crash loop on early panic
    #[cfg(not(test))]
    if crate::netstack::is_initialized() {
        use core::fmt::Write;
        let mut buf = alloc::string::String::with_capacity(1024);
        let _ = write!(buf, "!!! KERNEL PANIC !!!\n{}", info);
        if buf.len() > 1400 { buf.truncate(1400); }
        let _ = crate::netstack::udp::send_to([255, 255, 255, 255], 6666, 6666, buf.as_bytes());
    }

    // Try framebuffer if available
    if framebuffer::is_initialized() {
        framebuffer::set_fg_color(framebuffer::COLOR_RED);
        println!("\n!!! KERNEL PANIC !!!");
        println!("{}", info);
        framebuffer::set_fg_color(0xFFAAAAAA);
        println!("Rebooting in ~10s...");
        // Show backtrace on screen too
        let bt = debug::format_backtrace(8);
        for line in &bt {
            println!("{}", line);
        }
    }

    // Stop petting TCO → hardware reboot in ~30s (backup)
    debug::set_panicked();
    // Arm APIC software watchdog → reboot in 10s via ISA port 0x64 (primary)
    crate::apic::watchdog_arm(10_000);
    serial_println!("[PANIC] Auto-reboot armed: APIC=10s, TCO=~30s");

    halt_loop();
}
