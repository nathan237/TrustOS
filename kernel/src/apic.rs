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
const SPURIOUS_VECTOR: u32  = 0xFF;

// Timer LVT bits
const TIMER_PERIODIC: u32   = 1 << 17;
const TIMER_MASKED: u32     = 1 << 16;

// Timer divider values for DCR register
const TIMER_DIV_16: u32     = 0x03; // divide by 16

// ═══════════════════════════════════════════════════════════════════════
// I/O APIC Register Offsets
// ═══════════════════════════════════════════════════════════════════════

const IOAPIC_REG_ID: u32       = 0x00;
const IOAPIC_REG_VER: u32      = 0x01;
const IOAPIC_REG_REDTBL: u32   = 0x10; // Base for redirection entries (2 regs each)

// Redirection entry flags
const IOAPIC_MASKED: u64            = 1 << 16;
const IOAPIC_LEVEL_TRIGGERED: u64   = 1 << 15;
const IOAPIC_ACTIVE_LOW: u64        = 1 << 13;
// Destination mode: 0 = physical, 1 = logical
// Delivery mode: 000 = Fixed

// ═══════════════════════════════════════════════════════════════════════
// Timer interrupt vector (must not conflict with exceptions 0-31, PIC 32-47)
// ═══════════════════════════════════════════════════════════════════════

/// APIC timer interrupt vector
pub const TIMER_VECTOR: u8      = 48;
/// Spurious interrupt vector  
pub const SPURIOUS_VEC: u8      = 0xFF;
/// IPI vector (keep compatibility with existing 0xFE)
pub const IPI_VECTOR: u8        = 0xFE;

/// IRQ base for I/O APIC routed interrupts
/// Keyboard = IRQ_BASE + 1, Mouse = IRQ_BASE + 12
pub const IRQ_BASE: u8          = 49;

/// APIC-routed keyboard vector
pub const KEYBOARD_VECTOR: u8   = IRQ_BASE + 1;  // 50
/// APIC-routed mouse vector
pub const MOUSE_VECTOR: u8      = IRQ_BASE + 12; // 61
/// VirtIO interrupt vector (shared by all VirtIO devices on the same PCI IRQ)
pub const VIRTIO_VECTOR: u8     = 62;

// ═══════════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════════

static APIC_ENABLED: AtomicBool = AtomicBool::new(false);
static LAPIC_BASE_VIRT: AtomicU64 = AtomicU64::new(0);
static IOAPIC_BASE_VIRT: AtomicU64 = AtomicU64::new(0);

/// Ticks per ms calibrated during init
static TICKS_PER_MS: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════
// LAPIC — Local APIC (per-CPU)
// ═══════════════════════════════════════════════════════════════════════

/// Read a LAPIC register
#[inline]
unsafe fn lapic_read(offset: u32) -> u32 {
    let base = LAPIC_BASE_VIRT.load(Ordering::Relaxed);
    core::ptr::read_volatile((base + offset as u64) as *const u32)
}

/// Write a LAPIC register
#[inline]
unsafe fn lapic_write(offset: u32, value: u32) {
    let base = LAPIC_BASE_VIRT.load(Ordering::Relaxed);
    core::ptr::write_volatile((base + offset as u64) as *mut u32, value);
}

/// Send End-Of-Interrupt to Local APIC
pub fn lapic_eoi() {
    unsafe {
        lapic_write(LAPIC_EOI, 0);
    }
}

/// Get current CPU's LAPIC ID
pub fn lapic_id() -> u32 {
    unsafe { lapic_read(LAPIC_ID) >> 24 }
}

/// Enable the Local APIC on this CPU
fn enable_lapic() {
    unsafe {
        // Set Spurious Interrupt Vector Register: enable APIC + set spurious vector
        let svr = lapic_read(LAPIC_SVR);
        lapic_write(LAPIC_SVR, svr | SVR_APIC_ENABLED | SPURIOUS_VECTOR);
        
        // Set Task Priority to 0 (accept all interrupts)
        lapic_write(LAPIC_TPR, 0);
    }
}

/// Calibrate LAPIC timer using PIT (one-time on BSP)
/// Returns ticks per millisecond
fn calibrate_timer() -> u64 {
    unsafe {
        // Configure timer: divide by 16, one-shot, masked
        lapic_write(LAPIC_TIMER_DCR, TIMER_DIV_16);
        lapic_write(LAPIC_TIMER_LVT, TIMER_MASKED);
        
        // Set a large initial count
        lapic_write(LAPIC_TIMER_ICR, 0xFFFF_FFFF);
        
        // Wait 10ms using PIT
        crate::cpu::tsc::pit_delay_ms(10);
        
        // Read how many ticks elapsed
        let remaining = lapic_read(LAPIC_TIMER_CCR);
        let elapsed = 0xFFFF_FFFFu64 - remaining as u64;
        
        // Stop timer
        lapic_write(LAPIC_TIMER_LVT, TIMER_MASKED);
        
        // ticks per ms = elapsed / 10
        let tpm = elapsed / 10;
        crate::serial_println!("[APIC] Timer calibrated: {} ticks/ms ({} ticks in 10ms)", tpm, elapsed);
        tpm
    }
}

/// Start LAPIC timer in periodic mode
/// `interval_ms` = time between interrupts
pub fn start_timer(interval_ms: u64) {
    let tpm = TICKS_PER_MS.load(Ordering::Relaxed);
    if tpm == 0 {
        crate::serial_println!("[APIC] WARNING: Timer not calibrated, cannot start");
        return;
    }
    
    let count = tpm * interval_ms;
    
    unsafe {
        // Divide by 16
        lapic_write(LAPIC_TIMER_DCR, TIMER_DIV_16);
        
        // Periodic mode, unmask, vector = TIMER_VECTOR
        lapic_write(LAPIC_TIMER_LVT, TIMER_PERIODIC | TIMER_VECTOR as u32);
        
        // Set initial count (starts counting)
        lapic_write(LAPIC_TIMER_ICR, count as u32);
    }
    
    crate::serial_println!("[APIC] Timer started: {}ms interval, count={}", interval_ms, count);
}

/// Stop the LAPIC timer
pub fn stop_timer() {
    unsafe {
        lapic_write(LAPIC_TIMER_LVT, TIMER_MASKED);
        lapic_write(LAPIC_TIMER_ICR, 0);
    }
}

/// Send IPI to a specific CPU (by APIC ID)
pub fn send_ipi(target_apic_id: u32, vector: u8) {
    unsafe {
        // Set destination APIC ID
        lapic_write(LAPIC_ICR_HI, target_apic_id << 24);
        // Send: fixed delivery, vector
        lapic_write(LAPIC_ICR_LO, vector as u32);
    }
}

/// Send IPI to all other CPUs
pub fn send_ipi_all_others(vector: u8) {
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
    core::ptr::read_volatile((base + 0x10) as *const u32)
}

/// Write a 64-bit redirection entry for an IRQ
/// `irq` = IRQ number (0=timer, 1=keyboard, 12=mouse)
/// `vector` = IDT vector to fire
/// `dest_apic` = destination APIC ID (usually BSP = 0)
/// `flags` = IOAPIC entry flags (level/edge, polarity)
unsafe fn ioapic_route_irq(irq: u8, vector: u8, dest_apic: u8, flags: u64) {
    let reg_lo = IOAPIC_REG_REDTBL + (irq as u32) * 2;
    let reg_hi = reg_lo + 1;
    
    // Low 32 bits: vector + flags
    let entry_lo = (vector as u64) | flags;
    // High 32 bits: destination APIC ID in bits [56:63] of full entry = bits [24:31] of high dword
    let entry_hi = (dest_apic as u64) << 24;
    
    ioapic_write(reg_lo, entry_lo as u32);
    ioapic_write(reg_hi, entry_hi as u32);
}


/// Get max redirection entries from I/O APIC
unsafe fn ioapic_max_entries() -> u8 {
    let ver = ioapic_read(IOAPIC_REG_VER);
    ((ver >> 16) & 0xFF) as u8
}

// ═══════════════════════════════════════════════════════════════════════
// I/O APIC IRQ routing setup
// ═══════════════════════════════════════════════════════════════════════

/// Set up I/O APIC redirection entries for all hardware IRQs
/// Uses interrupt source overrides from ACPI MADT
fn setup_ioapic_routing() {
    let acpi_info = match crate::acpi::get_info() {
        Some(info) => info,
        None => {
            crate::serial_println!("[APIC] WARNING: No ACPI info, cannot set up I/O APIC");
            return;
        }
    };
    
    if acpi_info.io_apics.is_empty() {
        crate::serial_println!("[APIC] WARNING: No I/O APIC found in MADT");
        return;
    }
    
    let ioapic = &acpi_info.io_apics[0];
    // Map I/O APIC MMIO region into kernel page tables
    let ioapic_virt = match crate::memory::map_mmio(ioapic.address, 4096) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[APIC] Failed to map I/O APIC MMIO at {:#x}: {}", ioapic.address, e);
            return;
        }
    };
    IOAPIC_BASE_VIRT.store(ioapic_virt, Ordering::SeqCst);
    
    unsafe {
        let max_entries = ioapic_max_entries();
        crate::serial_println!("[APIC] I/O APIC id={}, addr={:#x}, GSI base={}, max_entries={}",
            ioapic.id, ioapic.address, ioapic.gsi_base, max_entries);
        
        // First: mask all entries
        for i in 0..=max_entries {
            let reg_lo = IOAPIC_REG_REDTBL + (i as u32) * 2;
            let lo = ioapic_read(reg_lo);
            ioapic_write(reg_lo, lo | IOAPIC_MASKED as u32);
        }
        
        // Route keyboard (IRQ1)
        let mut kbd_irq: u8 = 1;
        let mut kbd_flags: u64 = 0; // edge-triggered, active high (default ISA)
        
        // Check for interrupt source overrides
        for ovr in &acpi_info.int_overrides {
            if ovr.source == 1 {
                kbd_irq = ovr.gsi as u8;
                kbd_flags = override_to_flags(ovr);
                crate::serial_println!("[APIC] Keyboard IRQ override: ISA 1 → GSI {}", ovr.gsi);
            }
        }
        ioapic_route_irq(kbd_irq, KEYBOARD_VECTOR, 0, kbd_flags);
        crate::serial_println!("[APIC] Routed keyboard: IRQ {} → vector {}", kbd_irq, KEYBOARD_VECTOR);
        
        // Route mouse (IRQ12)
        let mut mouse_irq: u8 = 12;
        let mut mouse_flags: u64 = 0;
        
        for ovr in &acpi_info.int_overrides {
            if ovr.source == 12 {
                mouse_irq = ovr.gsi as u8;
                mouse_flags = override_to_flags(ovr);
                crate::serial_println!("[APIC] Mouse IRQ override: ISA 12 → GSI {}", ovr.gsi);
            }
        }
        ioapic_route_irq(mouse_irq, MOUSE_VECTOR, 0, mouse_flags);
        crate::serial_println!("[APIC] Routed mouse: IRQ {} → vector {}", mouse_irq, MOUSE_VECTOR);
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
    let info = match crate::acpi::get_info() {
        Some(i) => i,
        None => return,
    };
    
    if info.local_apic_nmis.is_empty() {
        // Default: assume LINT1 = NMI (common for most PC hardware)
        unsafe {
            // NMI delivery mode (0x400) on LINT1
            lapic_write(LAPIC_LINT1_LVT, 0x0400);
        }
        crate::serial_println!("[APIC] NMI: default LINT1=NMI (no MADT entries)");
        return;
    }
    
    for nmi in &info.local_apic_nmis {
        // processor_uid 0xFF means all processors
        // We configure on current CPU; APs will get the same in init_ap()
        let lint_reg = if nmi.lint == 0 { LAPIC_LINT0_LVT } else { LAPIC_LINT1_LVT };
        
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
        
        unsafe { lapic_write(lint_reg, lvt); }
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
    let lapic_phys = crate::acpi::local_apic_address();
    if lapic_phys == 0 {
        crate::serial_println!("[APIC] No LAPIC address from ACPI, staying on PIC");
        return false;
    }
    
    // Map LAPIC MMIO region into kernel page tables
    // (HHDM from Limine only covers RAM, not device MMIO like the LAPIC)
    let lapic_virt = match crate::memory::map_mmio(lapic_phys, 4096) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[APIC] Failed to map LAPIC MMIO at {:#x}: {}", lapic_phys, e);
            return false;
        }
    };
    LAPIC_BASE_VIRT.store(lapic_virt, Ordering::SeqCst);
    
    crate::serial_println!("[APIC] LAPIC at phys={:#x}, virt={:#x}", lapic_phys, lapic_virt);
    
    // 1. Disable legacy PIC
    unsafe { disable_pic(); }
    
    // 2. Enable Local APIC
    enable_lapic();
    
    let id = lapic_id();
    let version = unsafe { lapic_read(LAPIC_VERSION) } & 0xFF;
    crate::serial_println!("[APIC] LAPIC enabled: id={}, version={:#x}", id, version);
    
    // 3. Calibrate timer
    let tpm = calibrate_timer();
    TICKS_PER_MS.store(tpm, Ordering::SeqCst);
    
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
pub fn init_ap() {
    let lapic_virt = LAPIC_BASE_VIRT.load(Ordering::Relaxed);
    if lapic_virt == 0 {
        return;
    }
    
    enable_lapic();
    
    // Configure NMI on this AP too
    configure_lapic_nmi();
    
    // Use same calibrated timer rate
    let tpm = TICKS_PER_MS.load(Ordering::Relaxed);
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
pub fn ticks_per_ms() -> u64 {
    TICKS_PER_MS.load(Ordering::Relaxed)
}

/// Route a PCI interrupt line through I/O APIC to a specific vector
/// `irq` = PCI interrupt line (e.g. 10, 11)
/// `vector` = IDT vector to fire
/// Level-triggered, active-low (standard for PCI interrupts)
pub fn route_pci_irq(irq: u8, vector: u8) {
    if !is_enabled() {
        return;
    }
    let ioapic_base = IOAPIC_BASE_VIRT.load(Ordering::Relaxed);
    if ioapic_base == 0 {
        crate::serial_println!("[APIC] Cannot route IRQ {}: IOAPIC not initialized", irq);
        return;
    }
    unsafe {
        // PCI interrupts are level-triggered, active-low
        let flags = IOAPIC_LEVEL_TRIGGERED | IOAPIC_ACTIVE_LOW;
        ioapic_route_irq(irq, vector, 0, flags);
    }
    crate::serial_println!("[APIC] Routed PCI IRQ {} → vector {} (level/low)", irq, vector);
}
