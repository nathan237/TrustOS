//! FADT (Fixed ACPI Description Table) Parser
//!
//! The FADT provides fixed hardware information for power management.

use super::tables::{SdtHeader, GenericAddress};

/// FADT structure (partial - main fields we need)
#[repr(C, packed)]
struct Fadt {
    header: SdtHeader,
    
    /// Physical address of FACS
    firmware_controller: u32,
    /// Physical address of DSDT
    dsdt: u32,
    
    /// Reserved (ACPI 1.0 INT_MODEL)
    _reserved0: u8,
    /// Preferred Power Management Profile
    preferred_pm_profile: u8,
    /// System Control Interrupt
    sci_interrupt: u16,
    /// SMI Command Port
    smi_command: u32,
    /// Value to write to SMI_CMD to disable ownership of ACPI
    acpi_enable: u8,
    /// Value to write to SMI_CMD to re-enable SMI ownership
    acpi_disable: u8,
    /// Value to write to SMI_CMD to enter S4BIOS state
    s4bios_request: u8,
    /// Processor performance state control
    pstate_count: u8,
    
    /// PM1a Event Block address (I/O)
    pm1a_event_block: u32,
    /// PM1b Event Block address (I/O)
    pm1b_event_block: u32,
    /// PM1a Control Block address (I/O)
    pm1a_count_block: u32,
    /// PM1b Control Block address (I/O)
    pm1b_count_block: u32,
    /// PM2 Control Block address (I/O)
    pm2_count_block: u32,
    /// PM Timer Block address (I/O)
    pm_tmr_block: u32,
    /// GPE0 Block address (I/O)
    gpe0_block: u32,
    /// GPE1 Block address (I/O)
    gpe1_block: u32,
    
    /// PM1 Event Block length
    pm1_event_length: u8,
    /// PM1 Control Block length
    pm1_count_length: u8,
    /// PM2 Control Block length
    pm2_count_length: u8,
    /// PM Timer length
    pm_tmr_length: u8,
    /// GPE0 Block length
    gpe0_block_length: u8,
    /// GPE1 Block length
    gpe1_block_length: u8,
    /// GPE1 Base offset
    gpe1_base: u8,
    /// CST_CNT support
    cst_count: u8,
    /// C2 latency
    c2_latency: u16,
    /// C3 latency
    c3_latency: u16,
    /// Flush size
    flush_size: u16,
    /// Flush stride
    flush_stride: u16,
    /// Duty cycle offset
    duty_offset: u8,
    /// Duty cycle width
    duty_width: u8,
    /// RTC Day Alarm index
    day_alarm: u8,
    /// RTC Month Alarm index
    month_alarm: u8,
    /// RTC Century index
    century: u8,
    
    /// Boot architecture flags (ACPI 2.0+)
    boot_arch_flags: u16,
    /// Reserved
    _reserved1: u8,
    /// Feature flags
    flags: u32,
    
    /// Reset register (GAS)
    reset_register: GenericAddress,
    /// Value to write to reset_reg
    reset_value: u8,
    /// ARM boot architecture flags
    arm_boot_arch: u16,
    /// FADT minor version
    fadt_minor_version: u8,
    
    // Extended fields (ACPI 2.0+, 64-bit addresses)
    /// 64-bit FACS address
    x_firmware_controller: u64,
    /// 64-bit DSDT address
    x_dsdt: u64,
    /// Extended PM1a Event Block
    x_pm1a_event_block: GenericAddress,
    /// Extended PM1b Event Block
    x_pm1b_event_block: GenericAddress,
    /// Extended PM1a Control Block
    x_pm1a_count_block: GenericAddress,
    /// Extended PM1b Control Block
    x_pm1b_count_block: GenericAddress,
    /// Extended PM2 Control Block
    x_pm2_count_block: GenericAddress,
    /// Extended PM Timer Block
    x_pm_tmr_block: GenericAddress,
    /// Extended GPE0 Block
    x_gpe0_block: GenericAddress,
    /// Extended GPE1 Block
    x_gpe1_block: GenericAddress,
    /// Sleep Control register
    sleep_control_register: GenericAddress,
    /// Sleep Status register
    sleep_status_register: GenericAddress,
}

/// Parsed FADT information
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct FadtInfo {
    /// SCI interrupt number
    pub sci_int: u16,
    /// SMI command port
    pub smi_command: u32,
    /// ACPI enable value
    pub acpi_enable: u8,
    /// ACPI disable value
    pub acpi_disable: u8,
    
    /// PM1a Event Block I/O address
    pub pm1a_event_block: u32,
    /// PM1b Event Block I/O address
    pub pm1b_event_block: u32,
    /// PM1a Control Block I/O address
    pub pm1a_count_block: u32,
    /// PM1b Control Block I/O address
    pub pm1b_count_block: u32,
    /// PM Timer I/O address
    pub pm_tmr_block: u32,
    
    /// RTC century register index
    pub century_register: u8,
    
    /// Reset register
    pub reset_register: GenericAddress,
    /// Reset value
    pub reset_value: u8,
    
    /// Sleep Control register
    pub sleep_controller_register: Option<GenericAddress>,
    /// Sleep Status register
    pub sleep_status_register: Option<GenericAddress>,
    
    /// FADT flags
    pub flags: u32,
}

// Implementation block — defines methods for the type above.
impl FadtInfo {
    /// Flag: HW_REDUCED_ACPI
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FLAG_HARDWARE_REDUCED: u32 = 1 << 20;
    /// Flag: LOW_POWER_S0_IDLE_CAPABLE
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FLAG_LOW_POWER_S0: u32 = 1 << 21;
    /// Flag: WBINVD supported
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FLAG_WBINVD: u32 = 1 << 0;
    /// Flag: Reset register supported
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FLAG_RESET_REGISTER_SUP: u32 = 1 << 10;
    
        // Public function — callable from other modules.
pub fn is_hardware_reduced(&self) -> bool {
        (self.flags & Self::FLAG_HARDWARE_REDUCED) != 0
    }
    
        // Public function — callable from other modules.
pub fn supports_reset(&self) -> bool {
        (self.flags & Self::FLAG_RESET_REGISTER_SUP) != 0
    }
}

/// Parse FADT table
pub fn parse(fadt_virt: u64, _hhdm: u64) -> Option<FadtInfo> {
    let header = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(fadt_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const SdtHeader) };
    
    // Verify signature
    if &header.signature != b"FACP" {
        return None;
    }
    
    let fadt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(fadt_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const Fadt) };
    
    // Read fields carefully (packed struct)
    let sci_int = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.sci_interrupt)) };
    let smi_command = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.smi_command)) };
    let pm1a_event = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.pm1a_event_block)) };
    let pm1b_event = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.pm1b_event_block)) };
    let pm1a_count = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.pm1a_count_block)) };
    let pm1b_count = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.pm1b_count_block)) };
    let pm_tmr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.pm_tmr_block)) };
    let flags = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.flags)) };
    let reset_register = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.reset_register)) };
    
    // Check if we have extended fields (table length > 244)
    let has_extended = header.length >= 244;
    
    let (sleep_controller, sleep_status) = if has_extended && header.length >= 276 {
        let controller = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.sleep_control_register)) };
        let status = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_unaligned(core::ptr::address_of!(fadt.sleep_status_register)) };
        (
            if controller.is_valid() { Some(controller) } else { None },
            if status.is_valid() { Some(status) } else { None }
        )
    } else {
        (None, None)
    };
    
    Some(FadtInfo {
        sci_int,
        smi_command,
        acpi_enable: fadt.acpi_enable,
        acpi_disable: fadt.acpi_disable,
        pm1a_event_block: pm1a_event,
        pm1b_event_block: pm1b_event,
        pm1a_count_block: pm1a_count,
        pm1b_count_block: pm1b_count,
        pm_tmr_block: pm_tmr,
        century_register: fadt.century,
        reset_register,
        reset_value: fadt.reset_value,
        sleep_controller_register: sleep_controller,
        sleep_status_register: sleep_status,
        flags,
    })
}

/// Shutdown the system using ACPI
pub fn shutdown(fadt: &FadtInfo) {
    crate::serial_println!("[ACPI] Attempting ACPI shutdown...");
    
    // ── Step 0: Try VM-specific shutdown ports FIRST ───────────────
    // These are no-ops on real hardware (reads 0xFF / ignored writes)
    // but provide instant shutdown on QEMU, Bochs, VirtualBox, Cloud Hypervisor.
    unsafe {
        // QEMU PIIX4-PM / ICH9 (i440fx & q35 machine types)
        x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
        for _ in 0..100_000 { core::hint::spin_loop(); }
        
        // Bochs / older QEMU
        x86_64::instructions::port::Port::<u16>::new(0xB004).write(0x2000);
        for _ in 0..100_000 { core::hint::spin_loop(); }
        
        // VirtualBox (ACPI port)
        x86_64::instructions::port::Port::<u16>::new(0x4004).write(0x3400);
        for _ in 0..100_000 { core::hint::spin_loop(); }
        
        // Cloud Hypervisor
        x86_64::instructions::port::Port::<u8>::new(0x600).write(0x34);
        for _ in 0..100_000 { core::hint::spin_loop(); }
    }
    
    crate::serial_println!("[ACPI] VM ports didn't work, trying ACPI PM1...");
    
    // ── Step 1: Ensure ACPI mode is enabled ────────────────────────
    if fadt.smi_command != 0 && fadt.acpi_enable != 0 {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            // Check if SCI_EN is already set
            if fadt.pm1a_count_block != 0 {
                let current = x86_64::instructions::port::Port::<u16>::new(fadt.pm1a_count_block as u16).read();
                if (current & 0x0001) == 0 {
                    // ACPI not enabled — send ACPI_ENABLE to SMI_CMD
                    crate::serial_println!("[ACPI] Enabling ACPI mode via SMI_CMD={:#x}", fadt.smi_command);
                    x86_64::instructions::port::Port::<u8>::new(fadt.smi_command as u16)
                        .write(fadt.acpi_enable);
                    // Wait for SCI_EN to become set (up to ~100ms)
                    for _ in 0..10_000_000 {
                        let value = x86_64::instructions::port::Port::<u16>::new(fadt.pm1a_count_block as u16).read();
                        if (value & 0x0001) != 0 { break; }
                        core::hint::spin_loop();
                    }
                }
            }
        }
    }
    
    // ── Step 2: HW-reduced ACPI (modern systems) ───────────────────
    if fadt.is_hardware_reduced() {
        if let Some(ref sleep_controller) = fadt.sleep_controller_register {
            // SLP_TYP=0 + SLP_EN(bit 5) for HW-reduced
            unsafe { sleep_controller.write(0x20); }
            for _ in 0..1_000_000 { core::hint::spin_loop(); }
        }
    }
    
    // ── Step 3: Traditional ACPI S5 via PM1a_CNT ───────────────────
    // SLP_TYP for S5 should come from DSDT \_S5_ object.
    // We try the most common values in order of likelihood.
    const SLP_EN: u16 = 1 << 13;
    // SLP_TYP values (bits 12:10): different firmware use different values
    let s5_types: [u16; 4] = [
        0x00 << 10,  // QEMU (i440fx, q35), many modern BIOS
        0x05 << 10,  // PIIX4, some older BIOS
        0x07 << 10,  // VirtualBox, some Lenovo
        0x02 << 10,  // Some Dell / HP
    ];
    
    if fadt.pm1a_count_block != 0 {
        let port = fadt.pm1a_count_block as u16;
        
        for &slp_typ in &s5_types {
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                // Read current, preserve SCI_EN (bit 0) only
                let current = x86_64::instructions::port::Port::<u16>::new(port).read();
                let sci_en = current & 0x0001;
                
                crate::serial_println!("[ACPI] S5: writing SLP_TYP={:#06x} | SLP_EN to PM1a_CNT port {:#x}",
                    slp_typ, port);
                
                x86_64::instructions::port::Port::<u16>::new(port)
                    .write(sci_en | slp_typ | SLP_EN);
                
                // Wait to see if it took effect
                for _ in 0..500_000 { core::hint::spin_loop(); }
            }
        }
        
        // Also try PM1b if present
        if fadt.pm1b_count_block != 0 {
            let port_b = fadt.pm1b_count_block as u16;
            for &slp_typ in &s5_types {
                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    x86_64::instructions::port::Port::<u16>::new(port_b)
                        .write(slp_typ | SLP_EN);
                    for _ in 0..200_000 { core::hint::spin_loop(); }
                }
            }
        }
    }
    
    crate::serial_println!("[ACPI] All shutdown methods failed");
}

/// Reset the system using ACPI
pub fn reset(fadt: &FadtInfo) {
    crate::serial_println!("[ACPI] Attempting ACPI reset...");
    
    if fadt.supports_reset() && fadt.reset_register.is_valid() {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            fadt.reset_register.write(fadt.reset_value as u64);
        }
        
        // Give it a moment
        for _ in 0..1000000 { core::hint::spin_loop(); }
    }
    
    crate::serial_println!("[ACPI] Reset register failed");
}

/// Suspend the system to S3 (sleep-to-RAM)
///
/// The SLP_TYP for S3 should come from \_S3 in the DSDT, but we try common
/// values that work on QEMU, VirtualBox, and many real machines.
/// Returns true if wakeup occurred (resumed from S3), false if S3 failed.
pub fn suspend_s3(fadt: &FadtInfo) -> bool {
    crate::serial_println!("[ACPI] Attempting S3 suspend (sleep-to-RAM)...");

    // Common S3 SLP_TYP values (bits 12:10 of PM1x_CNT)
    const SLP_TYP_S3_QEMU: u16 = 0x01 << 10;   // QEMU i440fx/q35
    const SLP_TYP_S3_PIIX: u16 = 0x05 << 10;    // PIIX4 PM
    const SLP_TYP_S3_ALT: u16 = 0x03 << 10;     // Some real hardware
    const SLP_EN: u16 = 1 << 13;

    // Enable wakeup events: timer (TMR_EN) and power-button (PWRBTN_EN)
    if fadt.pm1a_event_block != 0 {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            // PM1_EN register is at pm1a_evt_blk + pm1_evt_len/2 (typically +2)
            let en_port = (fadt.pm1a_event_block + 2) as u16;
            let en_value: u16 = (1 << 0)   // TMR_EN   — timer overflow wakeup
                            | (1 << 8)   // PWRBTN_EN — power button wakeup
                            | (1 << 5);  // GBL_EN   — global event
            x86_64::instructions::port::Port::<u16>::new(en_port).write(en_value);
        }
    }

    if fadt.pm1a_count_block != 0 {
        let port = fadt.pm1a_count_block as u16;
        let values = [SLP_TYP_S3_QEMU, SLP_TYP_S3_PIIX, SLP_TYP_S3_ALT];

        for &slp_typ in &values {
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                let current = x86_64::instructions::port::Port::<u16>::new(port).read();
                let preserved = current & 0x0203; // Preserve SCI_EN

                crate::serial_println!("[ACPI] S3: writing 0x{:04X} to PM1a_CNT (port 0x{:03X})",
                    preserved | slp_typ | SLP_EN, port);

                // Flush caches before sleeping
                core::arch::x86_64::_mm_mfence();

                x86_64::instructions::port::Port::<u16>::new(port)
                    .write(preserved | slp_typ | SLP_EN);

                // If we reach here, the CPU woke up or S3 didn't take effect
                // Wait a bit and see if we are alive
                for _ in 0..5_000_000 { core::hint::spin_loop(); }
            }
        }

        crate::serial_println!("[ACPI] S3 suspend did not take effect");
        return false;
    }

    crate::serial_println!("[ACPI] No PM1a_CNT register — cannot suspend");
    false
}
