//! FADT (Fixed ACPI Description Table) Parser
//!
//! The FADT provides fixed hardware information for power management.

use super::tables::{SdtHeader, GenericAddress};

/// FADT structure (partial - main fields we need)
#[repr(C, packed)]
struct Fadt {
    header: SdtHeader,
    
    /// Physical address of FACS
    firmware_ctrl: u32,
    /// Physical address of DSDT
    dsdt: u32,
    
    /// Reserved (ACPI 1.0 INT_MODEL)
    _reserved0: u8,
    /// Preferred Power Management Profile
    preferred_pm_profile: u8,
    /// System Control Interrupt
    sci_interrupt: u16,
    /// SMI Command Port
    smi_cmd: u32,
    /// Value to write to SMI_CMD to disable ownership of ACPI
    acpi_enable: u8,
    /// Value to write to SMI_CMD to re-enable SMI ownership
    acpi_disable: u8,
    /// Value to write to SMI_CMD to enter S4BIOS state
    s4bios_req: u8,
    /// Processor performance state control
    pstate_cnt: u8,
    
    /// PM1a Event Block address (I/O)
    pm1a_evt_blk: u32,
    /// PM1b Event Block address (I/O)
    pm1b_evt_blk: u32,
    /// PM1a Control Block address (I/O)
    pm1a_cnt_blk: u32,
    /// PM1b Control Block address (I/O)
    pm1b_cnt_blk: u32,
    /// PM2 Control Block address (I/O)
    pm2_cnt_blk: u32,
    /// PM Timer Block address (I/O)
    pm_tmr_blk: u32,
    /// GPE0 Block address (I/O)
    gpe0_blk: u32,
    /// GPE1 Block address (I/O)
    gpe1_blk: u32,
    
    /// PM1 Event Block length
    pm1_evt_len: u8,
    /// PM1 Control Block length
    pm1_cnt_len: u8,
    /// PM2 Control Block length
    pm2_cnt_len: u8,
    /// PM Timer length
    pm_tmr_len: u8,
    /// GPE0 Block length
    gpe0_blk_len: u8,
    /// GPE1 Block length
    gpe1_blk_len: u8,
    /// GPE1 Base offset
    gpe1_base: u8,
    /// CST_CNT support
    cst_cnt: u8,
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
    reset_reg: GenericAddress,
    /// Value to write to reset_reg
    reset_value: u8,
    /// ARM boot architecture flags
    arm_boot_arch: u16,
    /// FADT minor version
    fadt_minor_version: u8,
    
    // Extended fields (ACPI 2.0+, 64-bit addresses)
    /// 64-bit FACS address
    x_firmware_ctrl: u64,
    /// 64-bit DSDT address
    x_dsdt: u64,
    /// Extended PM1a Event Block
    x_pm1a_evt_blk: GenericAddress,
    /// Extended PM1b Event Block
    x_pm1b_evt_blk: GenericAddress,
    /// Extended PM1a Control Block
    x_pm1a_cnt_blk: GenericAddress,
    /// Extended PM1b Control Block
    x_pm1b_cnt_blk: GenericAddress,
    /// Extended PM2 Control Block
    x_pm2_cnt_blk: GenericAddress,
    /// Extended PM Timer Block
    x_pm_tmr_blk: GenericAddress,
    /// Extended GPE0 Block
    x_gpe0_blk: GenericAddress,
    /// Extended GPE1 Block
    x_gpe1_blk: GenericAddress,
    /// Sleep Control register
    sleep_control_reg: GenericAddress,
    /// Sleep Status register
    sleep_status_reg: GenericAddress,
}

/// Parsed FADT information
#[derive(Debug, Clone)]
pub struct FadtInfo {
    /// SCI interrupt number
    pub sci_int: u16,
    /// SMI command port
    pub smi_cmd: u32,
    /// ACPI enable value
    pub acpi_enable: u8,
    /// ACPI disable value
    pub acpi_disable: u8,
    
    /// PM1a Event Block I/O address
    pub pm1a_evt_blk: u32,
    /// PM1b Event Block I/O address
    pub pm1b_evt_blk: u32,
    /// PM1a Control Block I/O address
    pub pm1a_cnt_blk: u32,
    /// PM1b Control Block I/O address
    pub pm1b_cnt_blk: u32,
    /// PM Timer I/O address
    pub pm_tmr_blk: u32,
    
    /// RTC century register index
    pub century_reg: u8,
    
    /// Reset register
    pub reset_reg: GenericAddress,
    /// Reset value
    pub reset_value: u8,
    
    /// Sleep Control register
    pub sleep_ctrl_reg: Option<GenericAddress>,
    /// Sleep Status register
    pub sleep_status_reg: Option<GenericAddress>,
    
    /// FADT flags
    pub flags: u32,
}

impl FadtInfo {
    /// Flag: HW_REDUCED_ACPI
    pub const FLAG_HW_REDUCED: u32 = 1 << 20;
    /// Flag: LOW_POWER_S0_IDLE_CAPABLE
    pub const FLAG_LOW_POWER_S0: u32 = 1 << 21;
    /// Flag: WBINVD supported
    pub const FLAG_WBINVD: u32 = 1 << 0;
    /// Flag: Reset register supported
    pub const FLAG_RESET_REG_SUP: u32 = 1 << 10;
    
    pub fn is_hw_reduced(&self) -> bool {
        (self.flags & Self::FLAG_HW_REDUCED) != 0
    }
    
    pub fn supports_reset(&self) -> bool {
        (self.flags & Self::FLAG_RESET_REG_SUP) != 0
    }
}

/// Parse FADT table
pub fn parse(fadt_virt: u64, _hhdm: u64) -> Option<FadtInfo> {
    let header = unsafe { &*(fadt_virt as *const SdtHeader) };
    
    // Verify signature
    if &header.signature != b"FACP" {
        return None;
    }
    
    let fadt = unsafe { &*(fadt_virt as *const Fadt) };
    
    // Read fields carefully (packed struct)
    let sci_int = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.sci_interrupt)) };
    let smi_cmd = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.smi_cmd)) };
    let pm1a_evt = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm1a_evt_blk)) };
    let pm1b_evt = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm1b_evt_blk)) };
    let pm1a_cnt = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm1a_cnt_blk)) };
    let pm1b_cnt = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm1b_cnt_blk)) };
    let pm_tmr = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm_tmr_blk)) };
    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.flags)) };
    let reset_reg = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.reset_reg)) };
    
    // Check if we have extended fields (table length > 244)
    let has_extended = header.length >= 244;
    
    let (sleep_ctrl, sleep_status) = if has_extended && header.length >= 276 {
        let ctrl = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.sleep_control_reg)) };
        let status = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.sleep_status_reg)) };
        (
            if ctrl.is_valid() { Some(ctrl) } else { None },
            if status.is_valid() { Some(status) } else { None }
        )
    } else {
        (None, None)
    };
    
    Some(FadtInfo {
        sci_int,
        smi_cmd,
        acpi_enable: fadt.acpi_enable,
        acpi_disable: fadt.acpi_disable,
        pm1a_evt_blk: pm1a_evt,
        pm1b_evt_blk: pm1b_evt,
        pm1a_cnt_blk: pm1a_cnt,
        pm1b_cnt_blk: pm1b_cnt,
        pm_tmr_blk: pm_tmr,
        century_reg: fadt.century,
        reset_reg,
        reset_value: fadt.reset_value,
        sleep_ctrl_reg: sleep_ctrl,
        sleep_status_reg: sleep_status,
        flags,
    })
}

/// Shutdown the system using ACPI
pub fn shutdown(fadt: &FadtInfo) {
    crate::serial_println!("[ACPI] Attempting ACPI shutdown...");
    
    // For S5 (soft-off), we need to write SLP_TYPx | SLP_EN to PM1a_CNT
    // The SLP_TYPx value should come from \_S5 in DSDT, but we'll try common values
    
    // Common S5 sleep type values
    const SLP_TYP_S5_TYPICAL: u16 = 0x00 << 10;  // Some systems
    const SLP_TYP_S5_COMMON: u16 = 0x05 << 10;   // Common value
    const SLP_TYP_S5_ALT: u16 = 0x07 << 10;      // Alternative
    const SLP_EN: u16 = 1 << 13;
    
    // Try HW-reduced ACPI first if available
    if fadt.is_hw_reduced() {
        if let Some(ref sleep_ctrl) = fadt.sleep_ctrl_reg {
            unsafe {
                // Write S5 type + enable
                sleep_ctrl.write((SLP_TYP_S5_COMMON >> 10) as u64 | 0x20);
            }
            // If we're here, it didn't work
        }
    }
    
    // Traditional ACPI shutdown via PM1a_CNT
    if fadt.pm1a_cnt_blk != 0 {
        unsafe {
            // Try different S5 type values
            let port = fadt.pm1a_cnt_blk as u16;
            
            // Read current value and preserve bits
            let current = x86_64::instructions::port::Port::<u16>::new(port).read();
            let preserved = current & 0x0203; // Preserve SCI_EN and some bits
            
            // Try common S5 value
            x86_64::instructions::port::Port::<u16>::new(port)
                .write(preserved | SLP_TYP_S5_COMMON | SLP_EN);
            
            // Small delay
            for _ in 0..1000000 { core::hint::spin_loop(); }
            
            // Try typical S5 value
            x86_64::instructions::port::Port::<u16>::new(port)
                .write(preserved | SLP_TYP_S5_TYPICAL | SLP_EN);
            
            // Small delay
            for _ in 0..1000000 { core::hint::spin_loop(); }
            
            // Try alternative S5 value
            x86_64::instructions::port::Port::<u16>::new(port)
                .write(preserved | SLP_TYP_S5_ALT | SLP_EN);
        }
        
        // Also try PM1b if present
        if fadt.pm1b_cnt_blk != 0 {
            unsafe {
                let port = fadt.pm1b_cnt_blk as u16;
                x86_64::instructions::port::Port::<u16>::new(port)
                    .write(SLP_TYP_S5_COMMON | SLP_EN);
            }
        }
    }
    
    crate::serial_println!("[ACPI] Shutdown via PM1 failed, trying QEMU/Bochs...");
    
    // Fallback: QEMU shutdown via debug port
    unsafe {
        // QEMU debug exit
        x86_64::instructions::port::Port::<u8>::new(0xf4).write(0x00);
        
        // Bochs/older QEMU shutdown
        x86_64::instructions::port::Port::<u16>::new(0xB004).write(0x2000);
        
        // Another common QEMU shutdown port
        x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
    }
}

/// Reset the system using ACPI
pub fn reset(fadt: &FadtInfo) {
    crate::serial_println!("[ACPI] Attempting ACPI reset...");
    
    if fadt.supports_reset() && fadt.reset_reg.is_valid() {
        unsafe {
            fadt.reset_reg.write(fadt.reset_value as u64);
        }
        
        // Give it a moment
        for _ in 0..1000000 { core::hint::spin_loop(); }
    }
    
    crate::serial_println!("[ACPI] Reset register failed");
}
