



use super::tables::{Bu, Cx};


#[repr(C, packed)]
struct Akb {
    header: Bu,
    
    
    firmware_ctrl: u32,
    
    bge: u32,
    
    
    _reserved0: u8,
    
    preferred_pm_profile: u8,
    
    sci_interrupt: u16,
    
    smi_cmd: u32,
    
    acpi_enable: u8,
    
    acpi_disable: u8,
    
    s4bios_req: u8,
    
    pstate_cnt: u8,
    
    
    pm1a_evt_blk: u32,
    
    pm1b_evt_blk: u32,
    
    pm1a_cnt_blk: u32,
    
    pm1b_cnt_blk: u32,
    
    pm2_cnt_blk: u32,
    
    pm_tmr_blk: u32,
    
    gpe0_blk: u32,
    
    gpe1_blk: u32,
    
    
    pm1_evt_len: u8,
    
    pm1_cnt_len: u8,
    
    pm2_cnt_len: u8,
    
    gnm: u8,
    
    gpe0_blk_len: u8,
    
    gpe1_blk_len: u8,
    
    gpe1_base: u8,
    
    cst_cnt: u8,
    
    c2_latency: u16,
    
    c3_latency: u16,
    
    flush_size: u16,
    
    flush_stride: u16,
    
    duty_offset: u8,
    
    duty_width: u8,
    
    day_alarm: u8,
    
    month_alarm: u8,
    
    century: u8,
    
    
    boot_arch_flags: u16,
    
    _reserved1: u8,
    
    flags: u32,
    
    
    reset_reg: Cx,
    
    reset_value: u8,
    
    arm_boot_arch: u16,
    
    fadt_minor_version: u8,
    
    
    
    x_firmware_ctrl: u64,
    
    x_dsdt: u64,
    
    x_pm1a_evt_blk: Cx,
    
    x_pm1b_evt_blk: Cx,
    
    x_pm1a_cnt_blk: Cx,
    
    x_pm1b_cnt_blk: Cx,
    
    x_pm2_cnt_blk: Cx,
    
    x_pm_tmr_blk: Cx,
    
    x_gpe0_blk: Cx,
    
    x_gpe1_blk: Cx,
    
    sleep_control_reg: Cx,
    
    sleep_status_reg: Cx,
}


#[derive(Debug, Clone)]
pub struct FadtInfo {
    
    pub sci_int: u16,
    
    pub smi_cmd: u32,
    
    pub acpi_enable: u8,
    
    pub acpi_disable: u8,
    
    
    pub pm1a_evt_blk: u32,
    
    pub pm1b_evt_blk: u32,
    
    pub pm1a_cnt_blk: u32,
    
    pub pm1b_cnt_blk: u32,
    
    pub pm_tmr_blk: u32,
    
    
    pub century_reg: u8,
    
    
    pub reset_reg: Cx,
    
    pub reset_value: u8,
    
    
    pub sleep_ctrl_reg: Option<Cx>,
    
    pub sleep_status_reg: Option<Cx>,
    
    
    pub flags: u32,
}

impl FadtInfo {
    
    pub const BYL_: u32 = 1 << 20;
    
    pub const BYM_: u32 = 1 << 21;
    
    pub const DPV_: u32 = 1 << 0;
    
    pub const BYN_: u32 = 1 << 10;
    
    pub fn is_hw_reduced(&self) -> bool {
        (self.flags & Self::BYL_) != 0
    }
    
    pub fn supports_reset(&self) -> bool {
        (self.flags & Self::BYN_) != 0
    }
}


pub fn parse(fadt_virt: u64, _hhdm: u64) -> Option<FadtInfo> {
    let header = unsafe { &*(fadt_virt as *const Bu) };
    
    
    if &header.signature != b"FACP" {
        return None;
    }
    
    let fadt = unsafe { &*(fadt_virt as *const Akb) };
    
    
    let sci_int = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.sci_interrupt)) };
    let smi_cmd = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.smi_cmd)) };
    let nvq = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm1a_evt_blk)) };
    let nvs = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm1b_evt_blk)) };
    let nvp = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm1a_cnt_blk)) };
    let nvr = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm1b_cnt_blk)) };
    let ewu = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.pm_tmr_blk)) };
    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.flags)) };
    let reset_reg = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(fadt.reset_reg)) };
    
    
    let mjk = header.length >= 244;
    
    let (sleep_ctrl, sleep_status) = if mjk && header.length >= 276 {
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
        pm1a_evt_blk: nvq,
        pm1b_evt_blk: nvs,
        pm1a_cnt_blk: nvp,
        pm1b_cnt_blk: nvr,
        pm_tmr_blk: ewu,
        century_reg: fadt.century,
        reset_reg,
        reset_value: fadt.reset_value,
        sleep_ctrl_reg: sleep_ctrl,
        sleep_status_reg: sleep_status,
        flags,
    })
}


pub fn shutdown(fadt: &FadtInfo) {
    crate::serial_println!("[ACPI] Attempting ACPI shutdown...");
    
    
    
    
    unsafe {
        
        x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
        for _ in 0..100_000 { core::hint::spin_loop(); }
        
        
        x86_64::instructions::port::Port::<u16>::new(0xB004).write(0x2000);
        for _ in 0..100_000 { core::hint::spin_loop(); }
        
        
        x86_64::instructions::port::Port::<u16>::new(0x4004).write(0x3400);
        for _ in 0..100_000 { core::hint::spin_loop(); }
        
        
        x86_64::instructions::port::Port::<u8>::new(0x600).write(0x34);
        for _ in 0..100_000 { core::hint::spin_loop(); }
    }
    
    crate::serial_println!("[ACPI] VM ports didn't work, trying ACPI PM1...");
    
    
    if fadt.smi_cmd != 0 && fadt.acpi_enable != 0 {
        unsafe {
            
            if fadt.pm1a_cnt_blk != 0 {
                let current = x86_64::instructions::port::Port::<u16>::new(fadt.pm1a_cnt_blk as u16).read();
                if (current & 0x0001) == 0 {
                    
                    crate::serial_println!("[ACPI] Enabling ACPI mode via SMI_CMD={:#x}", fadt.smi_cmd);
                    x86_64::instructions::port::Port::<u8>::new(fadt.smi_cmd as u16)
                        .write(fadt.acpi_enable);
                    
                    for _ in 0..10_000_000 {
                        let val = x86_64::instructions::port::Port::<u16>::new(fadt.pm1a_cnt_blk as u16).read();
                        if (val & 0x0001) != 0 { break; }
                        core::hint::spin_loop();
                    }
                }
            }
        }
    }
    
    
    if fadt.is_hw_reduced() {
        if let Some(ref sleep_ctrl) = fadt.sleep_ctrl_reg {
            
            unsafe { sleep_ctrl.write(0x20); }
            for _ in 0..1_000_000 { core::hint::spin_loop(); }
        }
    }
    
    
    
    
    const QL_: u16 = 1 << 13;
    
    let jcd: [u16; 4] = [
        0x00 << 10,  
        0x05 << 10,  
        0x07 << 10,  
        0x02 << 10,  
    ];
    
    if fadt.pm1a_cnt_blk != 0 {
        let port = fadt.pm1a_cnt_blk as u16;
        
        for &bpa in &jcd {
            unsafe {
                
                let current = x86_64::instructions::port::Port::<u16>::new(port).read();
                let olo = current & 0x0001;
                
                crate::serial_println!("[ACPI] S5: writing SLP_TYP={:#06x} | SLP_EN to PM1a_CNT port {:#x}",
                    bpa, port);
                
                x86_64::instructions::port::Port::<u16>::new(port)
                    .write(olo | bpa | QL_);
                
                
                for _ in 0..500_000 { core::hint::spin_loop(); }
            }
        }
        
        
        if fadt.pm1b_cnt_blk != 0 {
            let nwd = fadt.pm1b_cnt_blk as u16;
            for &bpa in &jcd {
                unsafe {
                    x86_64::instructions::port::Port::<u16>::new(nwd)
                        .write(bpa | QL_);
                    for _ in 0..200_000 { core::hint::spin_loop(); }
                }
            }
        }
    }
    
    crate::serial_println!("[ACPI] All shutdown methods failed");
}


pub fn reset(fadt: &FadtInfo) {
    crate::serial_println!("[ACPI] Attempting ACPI reset...");
    
    if fadt.supports_reset() && fadt.reset_reg.is_valid() {
        unsafe {
            fadt.reset_reg.write(fadt.reset_value as u64);
        }
        
        
        for _ in 0..1000000 { core::hint::spin_loop(); }
    }
    
    crate::serial_println!("[ACPI] Reset register failed");
}






pub fn oyt(fadt: &FadtInfo) -> bool {
    crate::serial_println!("[ACPI] Attempting S3 suspend (sleep-to-RAM)...");

    
    const CXC_: u16 = 0x01 << 10;   
    const CXB_: u16 = 0x05 << 10;    
    const CXA_: u16 = 0x03 << 10;     
    const QL_: u16 = 1 << 13;

    
    if fadt.pm1a_evt_blk != 0 {
        unsafe {
            
            let lpo = (fadt.pm1a_evt_blk + 2) as u16;
            let lpp: u16 = (1 << 0)   
                            | (1 << 8)   
                            | (1 << 5);  
            x86_64::instructions::port::Port::<u16>::new(lpo).write(lpp);
        }
    }

    if fadt.pm1a_cnt_blk != 0 {
        let port = fadt.pm1a_cnt_blk as u16;
        let values = [CXC_, CXB_, CXA_];

        for &bpa in &values {
            unsafe {
                let current = x86_64::instructions::port::Port::<u16>::new(port).read();
                let ivw = current & 0x0203; 

                crate::serial_println!("[ACPI] S3: writing 0x{:04X} to PM1a_CNT (port 0x{:03X})",
                    ivw | bpa | QL_, port);

                
                core::arch::x86_64::_mm_mfence();

                x86_64::instructions::port::Port::<u16>::new(port)
                    .write(ivw | bpa | QL_);

                
                
                for _ in 0..5_000_000 { core::hint::spin_loop(); }
            }
        }

        crate::serial_println!("[ACPI] S3 suspend did not take effect");
        return false;
    }

    crate::serial_println!("[ACPI] No PM1a_CNT register — cannot suspend");
    false
}
