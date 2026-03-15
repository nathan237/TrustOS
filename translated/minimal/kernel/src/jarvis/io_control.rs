






















use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use spin::Mutex;






#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IoStatus {
    
    Di,
    
    Jk,
    
    If,
    
    F,
}

impl IoStatus {
    fn cna(self) -> &'static str {
        match self {
            IoStatus::Di => "[+]",
            IoStatus::Jk => "[~]",
            IoStatus::If => "[-]",
            IoStatus::F => "[?]",
        }
    }

    fn rl(self) -> bool {
        self == IoStatus::Di
    }
}


pub struct Ng {
    pub keyboard: IoStatus,
    pub mouse: IoStatus,
    pub serial: IoStatus,
    pub network: IoStatus,
    pub disk: IoStatus,
    pub display: IoStatus,
    pub usb: IoStatus,
    pub audio: IoStatus,
    pub touch: IoStatus,
    pub gdo: IoStatus,
    pub gpu: IoStatus,
    pub gtj: IoStatus,
    pub gtk: IoStatus,
    pub jrv: IoStatus,
    pub pci: IoStatus,
}


static AYQ_: AtomicU64 = AtomicU64::new(0);


static CDN_: Mutex<Option<Ng>> = Mutex::new(None);


static AFS_: AtomicBool = AtomicBool::new(false);


static AXH_: AtomicU64 = AtomicU64::new(0);







pub fn hkp() -> Ng {
    let ma = Ng {
        keyboard: vlw(),
        mouse: vly(),
        serial: vmc(),
        network: lvq(),
        disk: vlu(),
        display: vlv(),
        usb: lvs(),
        audio: lvn(),
        touch: vmf(),
        gdo: vlt(),
        gpu: lvp(),
        gtj: vlq(),
        gtk: vlr(),
        jrv: vlz(),
        pci: gpv(),
    };

    
    AYQ_.store(crate::time::lc(), Ordering::SeqCst);
    *CDN_.lock() = Some(Ng {
        keyboard: ma.keyboard,
        mouse: ma.mouse,
        serial: ma.serial,
        network: ma.network,
        disk: ma.disk,
        display: ma.display,
        usb: ma.usb,
        audio: ma.audio,
        touch: ma.touch,
        gdo: ma.gdo,
        gpu: ma.gpu,
        gtj: ma.gtj,
        gtk: ma.gtk,
        jrv: ma.jrv,
        pci: ma.pci,
    });

    AXH_.fetch_add(1, Ordering::Relaxed);
    ma
}





fn vlw() -> IoStatus {
    if crate::drivers::input::oar() {
        IoStatus::Di
    } else if crate::keyboard::hmo() {
        IoStatus::Di
    } else {
        
        IoStatus::Jk
    }
}

fn vly() -> IoStatus {
    if crate::mouse::ky() {
        if crate::drivers::input::tms() {
            IoStatus::Di
        } else {
            IoStatus::Jk
        }
    } else {
        IoStatus::If
    }
}

fn vmc() -> IoStatus {
    
    
    IoStatus::Di
}

fn lvq() -> IoStatus {
    if crate::network::anl() {
        if crate::network::ckt().is_some() {
            IoStatus::Di
        } else {
            IoStatus::Jk
        }
    } else {
        IoStatus::If
    }
}

fn vlu() -> IoStatus {
    if crate::disk::anl() {
        IoStatus::Di
    } else {
        IoStatus::If
    }
}

fn vlv() -> IoStatus {
    if crate::framebuffer::ky() {
        IoStatus::Di
    } else {
        IoStatus::If
    }
}

fn lvs() -> IoStatus {
    if crate::drivers::usb::ky() {
        if crate::drivers::usb::tmo() {
            IoStatus::Di
        } else {
            IoStatus::Jk
        }
    } else {
        IoStatus::If
    }
}

fn lvn() -> IoStatus {
    if crate::drivers::hda::ky() {
        IoStatus::Di
    } else {
        IoStatus::If
    }
}

fn vmf() -> IoStatus {
    if crate::touch::anl() {
        if crate::touch::ky() {
            IoStatus::Di
        } else {
            IoStatus::Jk
        }
    } else {
        IoStatus::If
    }
}

fn vlt() -> IoStatus {
    let az = crate::cpu::smp::aao();
    let ack = crate::cpu::smp::boc();
    if ack > 1 {
        IoStatus::Di
    } else if az > 1 {
        IoStatus::Jk
    } else {
        
        IoStatus::Di
    }
}

fn lvp() -> IoStatus {
    if crate::drivers::amdgpu::clb() {
        if crate::drivers::amdgpu::compute::uc() {
            IoStatus::Di
        } else {
            IoStatus::Jk
        }
    } else {
        IoStatus::If
    }
}

fn vlq() -> IoStatus {
    if crate::drivers::ahci::ky() {
        IoStatus::Di
    } else {
        IoStatus::If
    }
}

fn vlr() -> IoStatus {
    if crate::drivers::ata::ky() {
        IoStatus::Di
    } else {
        IoStatus::If
    }
}

fn vlz() -> IoStatus {
    
    if crate::drivers::oba() {
        IoStatus::Jk
    } else {
        IoStatus::If
    }
}

fn gpv() -> IoStatus {
    
    IoStatus::Di
}









pub fn gdg(ma: &Ng) -> u8 {
    let mut ol: u32 = 0;
    let mut eus: u32 = 0;

    
    let cpp = [
        (ma.keyboard, 15u32),
        (ma.serial, 15),
        (ma.network, 15),
        (ma.disk, 15),
        (ma.display, 15),
    ];

    
    let uyz = [
        (ma.mouse, 5u32),
        (ma.usb, 5),
        (ma.audio, 5),
        (ma.touch, 5),
        (ma.gpu, 5),
    ];

    for (status, amz) in &cpp {
        eus += amz;
        match status {
            IoStatus::Di => ol += amz,
            IoStatus::Jk => ol += amz / 2,
            _ => {}
        }
    }

    for (status, amz) in &uyz {
        eus += amz;
        match status {
            IoStatus::Di => ol += amz,
            IoStatus::Jk => ol += amz / 2,
            _ => {}
        }
    }

    ((ol * 100) / eus).v(100) as u8
}



pub fn hsn(ma: &Ng) -> bool {
    ma.network.rl()
        && (ma.keyboard.rl() || ma.serial.rl())
        && ma.disk.rl()
}


pub fn nwq(ma: &Ng) -> bool {
    gdg(ma) >= 75
}






pub fn fix(ma: &Ng) -> Vec<String> {
    let ol = gdg(ma);
    let ack = hsn(ma);

    let mut ak = Vec::fc(24);
    ak.push(String::from("╔═══════════════════════════════════════════════════╗"));
    ak.push(String::from("║       JARVIS I/O CONTROL AUDIT                   ║"));
    ak.push(String::from("╠═══════════════════════════════════════════════════╣"));

    
    ak.push(String::from("║ CRITICAL CHANNELS:                                ║"));
    ak.push(format!("║  {} Keyboard    {}", ma.keyboard.cna(), bli(ma.keyboard)));
    ak.push(format!("║  {} Serial      {}", ma.serial.cna(), bli(ma.serial)));
    ak.push(format!("║  {} Network     {}", ma.network.cna(), bli(ma.network)));
    ak.push(format!("║  {} Disk        {}", ma.disk.cna(), bli(ma.disk)));
    ak.push(format!("║  {} Display     {}", ma.display.cna(), bli(ma.display)));

    ak.push(String::from("║ EXTENDED CHANNELS:                                ║"));
    ak.push(format!("║  {} Mouse       {}", ma.mouse.cna(), bli(ma.mouse)));
    ak.push(format!("║  {} USB         {}", ma.usb.cna(), bli(ma.usb)));
    ak.push(format!("║  {} Audio       {}", ma.audio.cna(), bli(ma.audio)));
    ak.push(format!("║  {} Touch       {}", ma.touch.cna(), bli(ma.touch)));
    ak.push(format!("║  {} GPU Compute {}", ma.gpu.cna(), bli(ma.gpu)));

    ak.push(String::from("║ STORAGE:                                          ║"));
    ak.push(format!("║  {} AHCI/SATA  {}", ma.gtj.cna(), bli(ma.gtj)));
    ak.push(format!("║  {} ATA/IDE    {}", ma.gtk.cna(), bli(ma.gtk)));

    ak.push(String::from("║ COMPUTE:                                          ║"));
    ak.push(format!("║  {} CPU/SMP    {}", ma.gdo.cna(), bli(ma.gdo)));
    ak.push(format!("║  {} PCI Bus    {}", ma.pci.cna(), bli(ma.pci)));

    ak.push(String::from("╠═══════════════════════════════════════════════════╣"));
    ak.push(format!("║  Control Score: {}%                                 ║",
        if ol < 10 { format!(" {}", ol) } else { format!("{}", ol) }));
    ak.push(format!("║  Network Ready: {}                                ║",
        if ack { "YES ✓" } else { "NO  ✗" }));
    ak.push(format!("║  Full Control:  {}                                ║",
        if nwq(ma) { "YES ✓" } else { "NO  ✗" }));
    ak.push(String::from("╚═══════════════════════════════════════════════════╝"));

    ak
}


fn bli(status: IoStatus) -> &'static str {
    match status {
        IoStatus::Di =>   "Active                          ║",
        IoStatus::Jk => "Detected                        ║",
        IoStatus::If =>   "Absent                          ║",
        IoStatus::F =>  "Unknown                         ║",
    }
}















pub fn nbs(ma: &Ng) -> u16 {
    let mut hs: u16 = 0;
    if ma.keyboard.rl() { hs |= 1 << 0; }
    if ma.mouse.rl()    { hs |= 1 << 1; }
    if ma.serial.rl()   { hs |= 1 << 2; }
    if ma.network.rl()  { hs |= 1 << 3; }
    if ma.disk.rl()     { hs |= 1 << 4; }
    if ma.display.rl()  { hs |= 1 << 5; }
    if ma.usb.rl()      { hs |= 1 << 6; }
    if ma.audio.rl()    { hs |= 1 << 7; }
    if ma.touch.rl()    { hs |= 1 << 8; }
    if ma.gpu.rl()      { hs |= 1 << 9; }
    if ma.gdo.rl()  { hs |= 1 << 10; }
    if ma.gtj.rl() { hs |= 1 << 11; }
    if ma.gtk.rl()  { hs |= 1 << 12; }
    if ma.jrv.rl() { hs |= 1 << 13; }
    if ma.pci.rl()      { hs |= 1 << 14; }
    hs
}


pub fn yls(hs: u16) -> String {
    let mut dr = Vec::new();
    if hs & (1 << 0) != 0 { dr.push("kbd"); }
    if hs & (1 << 1) != 0 { dr.push("mouse"); }
    if hs & (1 << 2) != 0 { dr.push("serial"); }
    if hs & (1 << 3) != 0 { dr.push("net"); }
    if hs & (1 << 4) != 0 { dr.push("disk"); }
    if hs & (1 << 5) != 0 { dr.push("display"); }
    if hs & (1 << 6) != 0 { dr.push("usb"); }
    if hs & (1 << 7) != 0 { dr.push("audio"); }
    if hs & (1 << 8) != 0 { dr.push("touch"); }
    if hs & (1 << 9) != 0 { dr.push("gpu"); }
    if hs & (1 << 10) != 0 { dr.push("smp"); }
    if hs & (1 << 11) != 0 { dr.push("ahci"); }
    if hs & (1 << 12) != 0 { dr.push("ata"); }
    if hs & (1 << 13) != 0 { dr.push("nvme"); }
    if hs & (1 << 14) != 0 { dr.push("pci"); }

    if dr.is_empty() {
        String::from("none")
    } else {
        let mut e = String::new();
        for (a, r) in dr.iter().cf() {
            if a > 0 { e.t(", "); }
            e.t(r);
        }
        e
    }
}






pub fn ypf() {
    AFS_.store(true, Ordering::SeqCst);
    crate::serial_println!("[IO-CTRL] Continuous I/O monitoring enabled");
}


pub fn ymd() {
    AFS_.store(false, Ordering::SeqCst);
}



pub fn poll() {
    if !AFS_.load(Ordering::SeqCst) {
        return;
    }

    let iu = crate::time::lc();
    let qv = AYQ_.load(Ordering::SeqCst);
    if iu.nj(qv) < 10_000 {
        return;
    }

    let ma = hkp();
    let ol = gdg(&ma);

    
    if ol < 60 {
        crate::serial_println!("[IO-CTRL] WARNING: I/O control score dropped to {}%", ol);
    }
}


pub fn ypr() -> u64 {
    AXH_.load(Ordering::Relaxed)
}


pub fn zhe() -> String {
    let ma = hkp();
    let ol = gdg(&ma);
    let hs = nbs(&ma);
    let ack = hsn(&ma);
    format!("io_score={}% caps=0x{:04X} mesh_ready={}", ol, hs, ack)
}
