






















use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use spin::Mutex;






#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IoStatus {
    
    Active,
    
    Detected,
    
    Absent,
    
    Unknown,
}

impl IoStatus {
    fn symbol(self) -> &'static str {
        match self {
            IoStatus::Active => "[+]",
            IoStatus::Detected => "[~]",
            IoStatus::Absent => "[-]",
            IoStatus::Unknown => "[?]",
        }
    }

    fn is_active(self) -> bool {
        self == IoStatus::Active
    }
}


pub struct Fs {
    pub keyboard: IoStatus,
    pub mouse: IoStatus,
    pub serial: IoStatus,
    pub network: IoStatus,
    pub disk: IoStatus,
    pub display: IoStatus,
    pub usb: IoStatus,
    pub audio: IoStatus,
    pub touch: IoStatus,
    pub cpu_smp: IoStatus,
    pub gpu: IoStatus,
    pub storage_ahci: IoStatus,
    pub storage_ata: IoStatus,
    pub storage_nvme: IoStatus,
    pub pci: IoStatus,
}


static BAR_: AtomicU64 = AtomicU64::new(0);


static CGW_: Mutex<Option<Fs>> = Mutex::new(None);


static AHM_: AtomicBool = AtomicBool::new(false);


static AZI_: AtomicU64 = AtomicU64::new(0);







pub fn dqi() -> Fs {
    let audit = Fs {
        keyboard: nxr(),
        mouse: nxt(),
        serial: nxw(),
        network: gok(),
        disk: nxo(),
        display: nxp(),
        usb: gon(),
        audio: goh(),
        touch: nxz(),
        cpu_smp: nxn(),
        gpu: goj(),
        storage_ahci: gog(),
        storage_ata: nxl(),
        storage_nvme: gol(),
        pci: ccv(),
    };

    
    BAR_.store(crate::time::uptime_ms(), Ordering::SeqCst);
    *CGW_.lock() = Some(Fs {
        keyboard: audit.keyboard,
        mouse: audit.mouse,
        serial: audit.serial,
        network: audit.network,
        disk: audit.disk,
        display: audit.display,
        usb: audit.usb,
        audio: audit.audio,
        touch: audit.touch,
        cpu_smp: audit.cpu_smp,
        gpu: audit.gpu,
        storage_ahci: audit.storage_ahci,
        storage_ata: audit.storage_ata,
        storage_nvme: audit.storage_nvme,
        pci: audit.pci,
    });

    AZI_.fetch_add(1, Ordering::Relaxed);
    audit
}





fn nxr() -> IoStatus {
    if crate::drivers::input::idr() {
        IoStatus::Active
    } else if crate::keyboard::has_input() {
        IoStatus::Active
    } else {
        
        IoStatus::Detected
    }
}

fn nxt() -> IoStatus {
    if crate::mouse::is_initialized() {
        if crate::drivers::input::mjr() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn nxw() -> IoStatus {
    
    
    IoStatus::Active
}

fn gok() -> IoStatus {
    if crate::network::sw() {
        if crate::network::aqu().is_some() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn nxo() -> IoStatus {
    if crate::disk::sw() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn nxp() -> IoStatus {
    if crate::framebuffer::is_initialized() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn gon() -> IoStatus {
    if crate::drivers::usb::is_initialized() {
        if crate::drivers::usb::mjm() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn goh() -> IoStatus {
    if crate::drivers::hda::is_initialized() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn nxz() -> IoStatus {
    if crate::touch::sw() {
        if crate::touch::is_initialized() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn nxn() -> IoStatus {
    let count = crate::cpu::smp::cpu_count();
    let ready = crate::cpu::smp::ail();
    if ready > 1 {
        IoStatus::Active
    } else if count > 1 {
        IoStatus::Detected
    } else {
        
        IoStatus::Active
    }
}

fn goj() -> IoStatus {
    if crate::drivers::amdgpu::aud() {
        if crate::drivers::amdgpu::compute::is_ready() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn gog() -> IoStatus {
    if crate::drivers::ahci::is_initialized() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn nxl() -> IoStatus {
    if crate::drivers::ata::is_initialized() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn gol() -> IoStatus {
    
    if crate::drivers::ied() {
        IoStatus::Detected
    } else {
        IoStatus::Absent
    }
}

fn ccv() -> IoStatus {
    
    IoStatus::Active
}









pub fn cvo(audit: &Fs) -> u8 {
    let mut score: u32 = 0;
    let mut cbr: u32 = 0;

    
    let aqb = [
        (audit.keyboard, 15u32),
        (audit.serial, 15),
        (audit.network, 15),
        (audit.disk, 15),
        (audit.display, 15),
    ];

    
    let nns = [
        (audit.mouse, 5u32),
        (audit.usb, 5),
        (audit.audio, 5),
        (audit.touch, 5),
        (audit.gpu, 5),
    ];

    for (status, tv) in &aqb {
        cbr += tv;
        match status {
            IoStatus::Active => score += tv,
            IoStatus::Detected => score += tv / 2,
            _ => {}
        }
    }

    for (status, tv) in &nns {
        cbr += tv;
        match status {
            IoStatus::Active => score += tv,
            IoStatus::Detected => score += tv / 2,
            _ => {}
        }
    }

    ((score * 100) / cbr).min(100) as u8
}



pub fn duz(audit: &Fs) -> bool {
    audit.network.is_active()
        && (audit.keyboard.is_active() || audit.serial.is_active())
        && audit.disk.is_active()
}


pub fn iai(audit: &Fs) -> bool {
    cvo(audit) >= 75
}






pub fn format_report(audit: &Fs) -> Vec<String> {
    let score = cvo(audit);
    let ready = duz(audit);

    let mut lines = Vec::with_capacity(24);
    lines.push(String::from("╔═══════════════════════════════════════════════════╗"));
    lines.push(String::from("║       JARVIS I/O CONTROL AUDIT                   ║"));
    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));

    
    lines.push(String::from("║ CRITICAL CHANNELS:                                ║"));
    lines.push(format!("║  {} Keyboard    {}", audit.keyboard.symbol(), ahd(audit.keyboard)));
    lines.push(format!("║  {} Serial      {}", audit.serial.symbol(), ahd(audit.serial)));
    lines.push(format!("║  {} Network     {}", audit.network.symbol(), ahd(audit.network)));
    lines.push(format!("║  {} Disk        {}", audit.disk.symbol(), ahd(audit.disk)));
    lines.push(format!("║  {} Display     {}", audit.display.symbol(), ahd(audit.display)));

    lines.push(String::from("║ EXTENDED CHANNELS:                                ║"));
    lines.push(format!("║  {} Mouse       {}", audit.mouse.symbol(), ahd(audit.mouse)));
    lines.push(format!("║  {} USB         {}", audit.usb.symbol(), ahd(audit.usb)));
    lines.push(format!("║  {} Audio       {}", audit.audio.symbol(), ahd(audit.audio)));
    lines.push(format!("║  {} Touch       {}", audit.touch.symbol(), ahd(audit.touch)));
    lines.push(format!("║  {} GPU Compute {}", audit.gpu.symbol(), ahd(audit.gpu)));

    lines.push(String::from("║ STORAGE:                                          ║"));
    lines.push(format!("║  {} AHCI/SATA  {}", audit.storage_ahci.symbol(), ahd(audit.storage_ahci)));
    lines.push(format!("║  {} ATA/IDE    {}", audit.storage_ata.symbol(), ahd(audit.storage_ata)));

    lines.push(String::from("║ COMPUTE:                                          ║"));
    lines.push(format!("║  {} CPU/SMP    {}", audit.cpu_smp.symbol(), ahd(audit.cpu_smp)));
    lines.push(format!("║  {} PCI Bus    {}", audit.pci.symbol(), ahd(audit.pci)));

    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));
    lines.push(format!("║  Control Score: {}%                                 ║",
        if score < 10 { format!(" {}", score) } else { format!("{}", score) }));
    lines.push(format!("║  Network Ready: {}                                ║",
        if ready { "YES ✓" } else { "NO  ✗" }));
    lines.push(format!("║  Full Control:  {}                                ║",
        if iai(audit) { "YES ✓" } else { "NO  ✗" }));
    lines.push(String::from("╚═══════════════════════════════════════════════════╝"));

    lines
}


fn ahd(status: IoStatus) -> &'static str {
    match status {
        IoStatus::Active =>   "Active                          ║",
        IoStatus::Detected => "Detected                        ║",
        IoStatus::Absent =>   "Absent                          ║",
        IoStatus::Unknown =>  "Unknown                         ║",
    }
}















pub fn hjv(audit: &Fs) -> u16 {
    let mut mask: u16 = 0;
    if audit.keyboard.is_active() { mask |= 1 << 0; }
    if audit.mouse.is_active()    { mask |= 1 << 1; }
    if audit.serial.is_active()   { mask |= 1 << 2; }
    if audit.network.is_active()  { mask |= 1 << 3; }
    if audit.disk.is_active()     { mask |= 1 << 4; }
    if audit.display.is_active()  { mask |= 1 << 5; }
    if audit.usb.is_active()      { mask |= 1 << 6; }
    if audit.audio.is_active()    { mask |= 1 << 7; }
    if audit.touch.is_active()    { mask |= 1 << 8; }
    if audit.gpu.is_active()      { mask |= 1 << 9; }
    if audit.cpu_smp.is_active()  { mask |= 1 << 10; }
    if audit.storage_ahci.is_active() { mask |= 1 << 11; }
    if audit.storage_ata.is_active()  { mask |= 1 << 12; }
    if audit.storage_nvme.is_active() { mask |= 1 << 13; }
    if audit.pci.is_active()      { mask |= 1 << 14; }
    mask
}


pub fn qcr(mask: u16) -> String {
    let mut caps = Vec::new();
    if mask & (1 << 0) != 0 { caps.push("kbd"); }
    if mask & (1 << 1) != 0 { caps.push("mouse"); }
    if mask & (1 << 2) != 0 { caps.push("serial"); }
    if mask & (1 << 3) != 0 { caps.push("net"); }
    if mask & (1 << 4) != 0 { caps.push("disk"); }
    if mask & (1 << 5) != 0 { caps.push("display"); }
    if mask & (1 << 6) != 0 { caps.push("usb"); }
    if mask & (1 << 7) != 0 { caps.push("audio"); }
    if mask & (1 << 8) != 0 { caps.push("touch"); }
    if mask & (1 << 9) != 0 { caps.push("gpu"); }
    if mask & (1 << 10) != 0 { caps.push("smp"); }
    if mask & (1 << 11) != 0 { caps.push("ahci"); }
    if mask & (1 << 12) != 0 { caps.push("ata"); }
    if mask & (1 << 13) != 0 { caps.push("nvme"); }
    if mask & (1 << 14) != 0 { caps.push("pci"); }

    if caps.is_empty() {
        String::from("none")
    } else {
        let mut j = String::new();
        for (i, c) in caps.iter().enumerate() {
            if i > 0 { j.push_str(", "); }
            j.push_str(c);
        }
        j
    }
}






pub fn qex() {
    AHM_.store(true, Ordering::SeqCst);
    crate::serial_println!("[IO-CTRL] Continuous I/O monitoring enabled");
}


pub fn qcz() {
    AHM_.store(false, Ordering::SeqCst);
}



pub fn poll() {
    if !AHM_.load(Ordering::SeqCst) {
        return;
    }

    let cy = crate::time::uptime_ms();
    let last = BAR_.load(Ordering::SeqCst);
    if cy.wrapping_sub(last) < 10_000 {
        return;
    }

    let audit = dqi();
    let score = cvo(&audit);

    
    if score < 60 {
        crate::serial_println!("[IO-CTRL] WARNING: I/O control score dropped to {}%", score);
    }
}


pub fn qfh() -> u64 {
    AZI_.load(Ordering::Relaxed)
}


pub fn qrq() -> String {
    let audit = dqi();
    let score = cvo(&audit);
    let mask = hjv(&audit);
    let ready = duz(&audit);
    format!("io_score={}% caps=0x{:04X} mesh_ready={}", score, mask, ready)
}
