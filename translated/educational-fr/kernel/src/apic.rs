//! APIC — Advanced Programmable Interrupt Controller
//!
//! Replaces the legacy 8259 PIC with:
//! - Local APIC (per-CPU): timer, IPI, EOI
//! - I/O APIC: routes external IRQs (keyboard, mouse, etc.)
//!
//! This enables:
//! 1. Per-CPU timer interrupts → preemptive scheduling
//! 2. IPI for cross-core communication
//! 3. Proper IRQ routing for SMP
//!
//! LAPIC registers are memory-mapped at the address from ACPI MADT.
//! I/O APIC registers use indirect register access (index + data).

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// ═══════════════════════════════════════════════════════════════════════
// Local APIC Register Offsets
// ═══════════════════════════════════════════════════════════════════════

const LAPIC_ID: u32         = 0x020;  // Local APIC ID
const LAPIC_VERSION: u32    = 0x030;  // Version
const LAPIC_TPR: u32        = 0x080;  // Task Priority Register
const LAPIC_EOI: u32        = 0x0B0;  // End Of Interrupt
const LAPIC_SVR: u32        = 0x0F0;  // Spurious Interrupt Vector Register
const LAPIC_ICR_LO: u32     = 0x300;  // Interrupt Command Register (low)
const LAPIC_ICR_HI: u32     = 0x310;  // Interrupt Command Register (high)
const LAPIC_TIMER_LVT: u32  = 0x320;  // Timer LVT entry
const LAPIC_LINT0_LVT: u32  = 0x350;  // LINT0 local vector table
const LAPIC_LINT1_LVT: u32  = 0x360;  // LINT1 local vector table
const LAPIC_ERROR_LVT: u32  = 0x370;  // Error LVT entry
const LAPIC_TIMER_ICR: u32  = 0x380;  // Timer Initial Count Register
const LAPIC_TIMER_CCR: u32  = 0x390;  // Timer Current Count Register
const LAPIC_TIMER_DCR: u32  = 0x3E0;  // Timer Divide Configuration Register

// SVR bits
const SVR_APIC_ENABLED: u32 = 1 << 8;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPURIOUS_VECTOR: u32  = 0xFF;

// Timer LVT bits
const TIMER_PERIODIC: u32   = 1 << 17;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMER_MASKED: u32     = 1 << 16;

// Timer divider values for DCR register
const TIMER_DIV_16: u32     = 0x03; // divide by 16

// ═══════════════════════════════════════════════════════════════════════
// I/O APIC Register Offsets
// ═══════════════════════════════════════════════════════════════════════

const IOAPIC_REGISTER_ID: u32       = 0x00;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOAPIC_REGISTER_VER: u32      = 0x01;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOAPIC_REGISTER_REDTBL: u32   = 0x10; // Base for redirection entries (2 regs each)

// Redirection entry flags
const IOAPIC_MASKED: u64            = 1 << 16;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOAPIC_LEVEL_TRIGGERED: u64   = 1 << 15;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IOAPIC_ACTIVE_LOW: u64        = 1 << 13;
// Destination mode: 0 = physical, 1 = logical
// Delivery mode: 000 = Fixed

// ═══════════════════════════════════════════════════════════════════════
// Timer interrupt vector (must not conflict with exceptions 0-31, PIC 32-47)
// ═══════════════════════════════════════════════════════════════════════

/// APIC timer interrupt vector
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TIMER_VECTOR: u8      = 48;
/// Spurious interrupt vector  
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SPURIOUS_VEC: u8      = 0xFF;
/// IPI vector (keep compatibility with existing 0xFE)
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IPI_VECTOR: u8        = 0xFE;

/// IRQ base for I/O APIC routed interrupts
/// Keyboard = IRQ_BASE + 1, Mouse = IRQ_BASE + 12
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INTERRUPT_REQUEST_BASE: u8          = 49;

/// APIC-routed keyboard vector
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const KEYBOARD_VECTOR: u8   = INTERRUPT_REQUEST_BASE + 1;  // 50
/// APIC-routed mouse vector
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MOUSE_VECTOR: u8      = INTERRUPT_REQUEST_BASE + 12; // 61
/// VirtIO interrupt vector (shared by all VirtIO devices on the same PCI IRQ)
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VIRTIO_VECTOR: u8     = 62;

// ═══════════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════════

static APIC_ENABLED: AtomicBool = AtomicBool::new(false);
// Variable atomique — accès thread-safe sans verrou.
static LAPIC_BASE_VIRT: AtomicU64 = AtomicU64::new(0);
// Variable atomique — accès thread-safe sans verrou.
static IOAPIC_BASE_VIRT: AtomicU64 = AtomicU64::new(0);

/// Ticks per ms calibrated during init
static TICKS_PER_MOUSE: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════
// LAPIC — Local APIC (per-CPU)
// ═══════════════════════════════════════════════════════════════════════

/// Read a LAPIC register
#[inline]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn lapic_read(offset: u32) -> u32 {
    let base = LAPIC_BASE_VIRT.load(Ordering::Relaxed);
    core::ptr::read_volatile((base + offset as u64) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32)
}

/// Write a LAPIC register
#[inline]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn lapic_write(offset: u32, value: u32) {
    let base = LAPIC_BASE_VIRT.load(Ordering::Relaxed);
    core::ptr::write_volatile((base + offset as u64) as *mut u32, value);
}

/// Send End-Of-Interrupt to Local APIC
pub fn lapic_eoi() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        lapic_write(LAPIC_EOI, 0);
    }
}

/// Get current CPU's LAPIC ID
pub fn lapic_id() -> u32 {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { lapic_read(LAPIC_ID) >> 24 }
}

/// Enable the Local APIC on this CPU
fn enable_lapic() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // Set Spurious Interrupt Vector Register: enable APIC + set spurious vector
        let svr = lapic_read(LAPIC_SVR);
        lapic_write(LAPIC_SVR, svr | SVR_APIC_ENABLED | SPURIOUS_VECTOR);
        
        // Set Task Priority to 0 (accept all interrupts)
        lapic_write(LAPIC_TPR, 0);
    }
}

/// Conservative default: ~1000 ticks/ms works for most CPUs when PIT calibration fails.
/// At divider 16, this corresponds to a ~16 MHz bus clock — safe minimum for any x86.
const FALLBACK_TICKS_PER_MOUSE: u64 = 1000;

/// Calibrate LAPIC timer using PIT (one-time on BSP)
/// Returns ticks per millisecond (guaranteed non-zero: uses fallback if PIT fails)
fn calibrate_timer() -> u64 {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // Configure timer: divide by 16, one-shot, masked
        lapic_write(LAPIC_TIMER_DCR, TIMER_DIV_16);
        lapic_write(LAPIC_TIMER_LVT, TIMER_MASKED);
        
        // Set a large initial count
        lapic_write(LAPIC_TIMER_ICR, 0xFFFF_FFFF);
        
        // Wait 10ms using PIT
        crate::cpu::tsc::pit_delay_mouse(10);
        
        // Read how many ticks elapsed
        let remaining = lapic_read(LAPIC_TIMER_CCR);
        let elapsed = 0xFFFF_FFFFu64 - remaining as u64;
        
        // Stop timer
        lapic_write(LAPIC_TIMER_LVT, TIMER_MASKED);
        
        // ticks per ms = elapsed / 10
        let tpm = elapsed / 10;
        
        if tpm == 0 {
            crate::serial_println!("[APIC] WARNING: PIT calibration failed (elapsed={}), using fallback {} ticks/ms", elapsed, FALLBACK_TICKS_PER_MOUSE);
            return FALLBACK_TICKS_PER_MOUSE;
        }
        
        crate::serial_println!("[APIC] Timer calibrated: {} ticks/ms ({} ticks in 10ms)", tpm, elapsed);
        tpm
    }
}

/// Start LAPIC timer in periodic mode
/// `interval_ms` = time between interrupts
pub fn start_timer(interval_mouse: u64) {
    let mut tpm = TICKS_PER_MOUSE.load(Ordering::Relaxed);
    if tpm == 0 {
        crate::serial_println!("[APIC] WARNING: Timer not calibrated, using fallback {} ticks/ms", FALLBACK_TICKS_PER_MOUSE);
        tpm = FALLBACK_TICKS_PER_MOUSE;
        TICKS_PER_MOUSE.store(tpm, Ordering::SeqCst);
    }
    
    let count = tpm * interval_mouse;
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // Divide by 16
        lapic_write(LAPIC_TIMER_DCR, TIMER_DIV_16);
        
        // Periodic mode, unmask, vector = TIMER_VECTOR
        lapic_write(LAPIC_TIMER_LVT, TIMER_PERIODIC | TIMER_VECTOR as u32);
        
        // Set initial count (starts counting)
        lapic_write(LAPIC_TIMER_ICR, count as u32);
    }
    
    crate::serial_println!("[APIC] Timer started: {}ms interval, count={}", interval_mouse, count);
}

/// Stop the LAPIC timer
pub fn stop_timer() {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        lapic_write(LAPIC_TIMER_LVT, TIMER_MASKED);
        lapic_write(LAPIC_TIMER_ICR, 0);
    }
}

/// Send IPI to a specific CPU (by APIC ID)
pub fn send_ipi(target_apic_id: u32, vector: u8) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // Set destination APIC ID
        lapic_write(LAPIC_ICR_HI, target_apic_id << 24);
        // Send: fixed delivery, vector
        lapic_write(LAPIC_ICR_LO, vector as u32);
    }
}

/// Send IPI to all other CPUs
pub fn send_ipi_all_others(vector: u8) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        lapic_write(LAPIC_ICR_HI, 0);
        // Shorthand = 11 (all excluding self), fixed delivery
        lapic_write(LAPIC_ICR_LO, (0b11 << 18) | vector as u32);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// I/O APIC — Routes external hardware IRQs
// ═══════════════════════════════════════════════════════════════════════

/// Write to I/O APIC register (indirect: write index, then data)
unsafe fn ioapic_write(reg: u32, value: u32) {
    let base = IOAPIC_BASE_VIRT.load(Ordering::Relaxed);
    // IOREGSEL = base + 0x00
    core::ptr::write_volatile(base as *mut u32, reg);
    // IOWIN = base + 0x10
    core::ptr::write_volatile((base + 0x10) as *mut u32, value);
}

/// Read from I/O APIC register
unsafe fn ioapic_read(reg: u32) -> u32 {
    let base = IOAPIC_BASE_VIRT.load(Ordering::Relaxed);
    core::ptr::write_volatile(base as *mut u32, reg);
    core::ptr::read_volatile((base + 0x10) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32)
}

/// Write a 64-bit redirection entry for an IRQ
/// `irq` = IRQ number (0=timer, 1=keyboard, 12=mouse)
/// `vector` = IDT vector to fire
/// `dest_apic` = destination APIC ID (usually BSP = 0)
/// `flags` = IOAPIC entry flags (level/edge, polarity)
unsafe fn ioapic_route_interrupt_request(irq: u8, vector: u8, dest_apic: u8, flags: u64) {
    let register_lo = IOAPIC_REGISTER_REDTBL + (irq as u32) * 2;
    let register_hi = register_lo + 1;
    
    // Low 32 bits: vector + flags
    let entry_lo = (vector as u64) | flags;
    // High 32 bits: destination APIC ID in bits [56:63] of full entry = bits [24:31] of high dword
    let entry_hi = (dest_apic as u64) << 24;
    
    ioapic_write(register_lo, entry_lo as u32);
    ioapic_write(register_hi, entry_hi as u32);
}


/// Get max redirection entries from I/O APIC
unsafe fn ioapic_maximum_entries() -> u8 {
    let ver = ioapic_read(IOAPIC_REGISTER_VER);
    ((ver >> 16) & 0xFF) as u8
}

// ═══════════════════════════════════════════════════════════════════════
// I/O APIC IRQ routing setup
// ═══════════════════════════════════════════════════════════════════════

/// Set up I/O APIC redirection entries for all hardware IRQs
/// Uses interrupt source overrides from ACPI MADT
fn setup_ioapic_routing() {
    let acpi_information = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::acpi::get_information() {
        Some(information) => information,
        None => {
            crate::serial_println!("[APIC] WARNING: No ACPI info, cannot set up I/O APIC");
            return;
        }
    };
    
    if acpi_information.io_apics.is_empty() {
        crate::serial_println!("[APIC] WARNING: No I/O APIC found in MADT");
        return;
    }
    
    let ioapic = &acpi_information.io_apics[0];
    // Map I/O APIC MMIO region into kernel page tables
    let ioapic_virt = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::memory::map_mmio(ioapic.address, 4096) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[APIC] Failed to map I/O APIC MMIO at {:#x}: {}", ioapic.address, e);
            return;
        }
    };
    IOAPIC_BASE_VIRT.store(ioapic_virt, Ordering::SeqCst);
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let maximum_entries = ioapic_maximum_entries();
        crate::serial_println!("[APIC] I/O APIC id={}, addr={:#x}, GSI base={}, max_entries={}",
            ioapic.id, ioapic.address, ioapic.gsi_base, maximum_entries);
        
        // First: mask all entries
        for i in 0..=maximum_entries {
            let register_lo = IOAPIC_REGISTER_REDTBL + (i as u32) * 2;
            let lo = ioapic_read(register_lo);
            ioapic_write(register_lo, lo | IOAPIC_MASKED as u32);
        }
        
        // Route keyboard (IRQ1)
        let mut kbd_interrupt_request: u8 = 1;
        let mut kbd_flags: u64 = 0; // edge-triggered, active high (default ISA)
        
        // Check for interrupt source overrides
        for ovr in &acpi_information.int_overrides {
            if ovr.source == 1 {
                kbd_interrupt_request = ovr.gsi as u8;
                kbd_flags = override_to_flags(ovr);
                crate::serial_println!("[APIC] Keyboard IRQ override: ISA 1 → GSI {}", ovr.gsi);
            }
        }
        ioapic_route_interrupt_request(kbd_interrupt_request, KEYBOARD_VECTOR, 0, kbd_flags);
        crate::serial_println!("[APIC] Routed keyboard: IRQ {} → vector {}", kbd_interrupt_request, KEYBOARD_VECTOR);
        
        // Route mouse (IRQ12)
        let mut mouse_interrupt_request: u8 = 12;
        let mut mouse_flags: u64 = 0;
        
        for ovr in &acpi_information.int_overrides {
            if ovr.source == 12 {
                mouse_interrupt_request = ovr.gsi as u8;
                mouse_flags = override_to_flags(ovr);
                crate::serial_println!("[APIC] Mouse IRQ override: ISA 12 → GSI {}", ovr.gsi);
            }
        }
        ioapic_route_interrupt_request(mouse_interrupt_request, MOUSE_VECTOR, 0, mouse_flags);
        crate::serial_println!("[APIC] Routed mouse: IRQ {} → vector {}", mouse_interrupt_request, MOUSE_VECTOR);
    }
}

/// Convert MADT interrupt source override flags to IOAPIC redirection entry flags
fn override_to_flags(ovr: &crate::acpi::madt::IntSourceOverride) -> u64 {
    let mut flags: u64 = 0;
    
    // Polarity
    match ovr.polarity {
        0 => {} // bus default (ISA = active high)
        1 => {} // active high (no flag)
        3 => flags |= IOAPIC_ACTIVE_LOW,
        _ => {}
    }
    
    // Trigger mode
    match ovr.trigger {
        0 => {} // bus default (ISA = edge)
        1 => {} // edge triggered (no flag)
        3 => flags |= IOAPIC_LEVEL_TRIGGERED,
        _ => {}
    }
    
    flags
}

// ═══════════════════════════════════════════════════════════════════════
// Disable legacy PIC (mask all IRQs)
// ═══════════════════════════════════════════════════════════════════════

unsafe fn disable_pic() {
    use x86_64::instructions::port::Port;
    
    // Mask all IRQs on both PICs
    let mut pic1_data = Port::<u8>::new(0x21);
    let mut pic2_data = Port::<u8>::new(0xA1);
    
    pic1_data.write(0xFF);
    pic2_data.write(0xFF);
    
    crate::serial_println!("[APIC] Legacy PIC disabled (all IRQs masked)");
}

/// Configure LAPIC NMI based on MADT type-4 entries.
/// Programs LINT0/LINT1 with NMI delivery mode + correct polarity/trigger.
fn configure_lapic_nmi() {
    let information = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::acpi::get_information() {
        Some(i) => i,
        None => return,
    };
    
    if information.local_apic_nmis.is_empty() {
        // Default: assume LINT1 = NMI (common for most PC hardware)
        unsafe {
            // NMI delivery mode (0x400) on LINT1
            lapic_write(LAPIC_LINT1_LVT, 0x0400);
        }
        crate::serial_println!("[APIC] NMI: default LINT1=NMI (no MADT entries)");
        return;
    }
    
    for nmi in &information.local_apic_nmis {
        // processor_uid 0xFF means all processors
        // We configure on current CPU; APs will get the same in init_ap()
        let lint_register = if nmi.lint == 0 { LAPIC_LINT0_LVT } else { LAPIC_LINT1_LVT };
        
        // Build LVT entry: delivery mode = NMI (0b100 << 8 = 0x400)
        let mut lvt: u32 = 0x0400; // NMI delivery mode
        
        // Polarity: 0/1 = active-high (default), 3 = active-low → set bit 13
        if nmi.polarity == 3 {
            lvt |= 1 << 13; // active low
        }
        
        // Trigger: 0/1 = edge (default), 3 = level → set bit 15
        if nmi.trigger == 3 {
            lvt |= 1 << 15; // level triggered
        }
        
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { lapic_write(lint_register, lvt); }
        crate::serial_println!("[APIC] NMI: LINT{} = NMI (pol={}, trig={}, lvt={:#x})",
            nmi.lint, nmi.polarity, nmi.trigger, lvt);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════

/// Initialize the APIC subsystem (called once from BSP)
/// 1. Disables legacy PIC
/// 2. Enables Local APIC
/// 3. Calibrates LAPIC timer
/// 4. Sets up I/O APIC routing
/// 5. Starts periodic timer
pub fn init() -> bool {
    let lapic_physical = crate::acpi::local_apic_address();
    if lapic_physical == 0 {
        crate::serial_println!("[APIC] No LAPIC address from ACPI, staying on PIC");
        return false;
    }
    
    // Map LAPIC MMIO region into kernel page tables
    // (HHDM from Limine only covers RAM, not device MMIO like the LAPIC)
    let lapic_virt = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::memory::map_mmio(lapic_physical, 4096) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[APIC] Failed to map LAPIC MMIO at {:#x}: {}", lapic_physical, e);
            return false;
        }
    };
    LAPIC_BASE_VIRT.store(lapic_virt, Ordering::SeqCst);
    
    crate::serial_println!("[APIC] LAPIC at phys={:#x}, virt={:#x}", lapic_physical, lapic_virt);
    
    // 1. Disable legacy PIC
    unsafe { disable_pic(); }
    
    // 2. Enable Local APIC
    enable_lapic();
    
    let id = lapic_id();
    let version = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { lapic_read(LAPIC_VERSION) } & 0xFF;
    crate::serial_println!("[APIC] LAPIC enabled: id={}, version={:#x}", id, version);
    
    // 3. Calibrate timer
    let tpm = calibrate_timer();
    TICKS_PER_MOUSE.store(tpm, Ordering::SeqCst);
    
    // 4. Set up I/O APIC
    setup_ioapic_routing();
    
    // 4.5. Program LAPIC NMI (from MADT type 4 entries)
    configure_lapic_nmi();
    
    // 5. Start periodic timer (10ms = 100 Hz scheduling frequency)
    start_timer(10);
    
    APIC_ENABLED.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[APIC] ✓ APIC fully initialized — preemptive scheduling enabled");
    true
}

/// Initialize LAPIC on an Application Processor (after GDT/IDT loaded)
pub fn initialize_ap() {
    let lapic_virt = LAPIC_BASE_VIRT.load(Ordering::Relaxed);
    if lapic_virt == 0 {
        return;
    }
    
    enable_lapic();
    
    // Configure NMI on this AP too
    configure_lapic_nmi();
    
    // Use same calibrated timer rate
    let tpm = TICKS_PER_MOUSE.load(Ordering::Relaxed);
    if tpm > 0 {
        // Start periodic timer on this AP (same 10ms interval)
        start_timer(10);
    }
    
    let id = lapic_id();
    crate::serial_println!("[APIC] AP LAPIC enabled: id={}", id);
}

/// Check if APIC is enabled
pub fn is_enabled() -> bool {
    APIC_ENABLED.load(Ordering::Relaxed)
}

/// Get calibrated ticks per millisecond
pub fn ticks_per_mouse() -> u64 {
    TICKS_PER_MOUSE.load(Ordering::Relaxed)
}

/// Route a PCI interrupt line through I/O APIC to a specific vector
/// `irq` = PCI interrupt line (e.g. 10, 11)
/// `vector` = IDT vector to fire
/// Level-triggered, active-low (standard for PCI interrupts)
pub fn route_pci_interrupt_request(irq: u8, vector: u8) {
    if !is_enabled() {
        return;
    }
    let ioapic_base = IOAPIC_BASE_VIRT.load(Ordering::Relaxed);
    if ioapic_base == 0 {
        crate::serial_println!("[APIC] Cannot route IRQ {}: IOAPIC not initialized", irq);
        return;
    }
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // PCI interrupts are level-triggered, active-low
        let flags = IOAPIC_LEVEL_TRIGGERED | IOAPIC_ACTIVE_LOW;
        ioapic_route_interrupt_request(irq, vector, 0, flags);
    }
    crate::serial_println!("[APIC] Routed PCI IRQ {} → vector {} (level/low)", irq, vector);
}
