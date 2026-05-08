







use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};


const JC_: usize = 512;
const LK_: usize = 128;


#[derive(Debug, Clone, Copy)]
pub struct Ey {
    
    pub ipa: u64,
    
    pub va: u64,
    
    pub value: u64,
    
    pub access_size: u32,
    
    pub is_write: bool,
    
    pub was_inst_fetch: bool,
    
    pub device_name: &'static str,
}


#[derive(Debug, Clone, Copy)]
pub struct Iz {
    
    pub fid: u64,
    
    pub x1: u64,
    
    pub x2: u64,
    
    pub x3: u64,
    
    pub smc_type_name: &'static str,
}


#[derive(Clone, Copy)]
struct Tu {
    seq: u64,
    event: Ey,
}

#[derive(Clone, Copy)]
struct Vd {
    seq: u64,
    event: Iz,
}


const BWG_: Ey = Ey {
    ipa: 0,
    va: 0,
    value: 0,
    access_size: 0,
    is_write: false,
    was_inst_fetch: false,
    device_name: "",
};


const BWH_: Iz = Iz {
    fid: 0,
    x1: 0,
    x2: 0,
    x3: 0,
    smc_type_name: "",
};


static mut AHL_: [Tu; JC_] = {
    let slot = Tu { seq: 0, event: BWG_ };
    [slot; JC_]
};

static mut BIB_: [Vd; LK_] = {
    let slot = Vd { seq: 0, event: BWH_ };
    [slot; LK_]
};


static BCV_: AtomicUsize = AtomicUsize::new(0);

static WQ_: AtomicU64 = AtomicU64::new(0);


static BIC_: AtomicUsize = AtomicUsize::new(0);

static AJV_: AtomicU64 = AtomicU64::new(0);


pub fn etg(event: Ey) {
    let idx = BCV_.fetch_add(1, Ordering::Relaxed) % JC_;
    let seq = WQ_.fetch_add(1, Ordering::Relaxed) + 1;
    unsafe {
        AHL_[idx] = Tu { seq, event };
    }
}


pub fn nal(event: Iz) {
    let idx = BIC_.fetch_add(1, Ordering::Relaxed) % LK_;
    let seq = AJV_.fetch_add(1, Ordering::Relaxed) + 1;
    unsafe {
        BIB_[idx] = Vd { seq, event };
    }
}


pub fn gzs() -> u64 {
    WQ_.load(Ordering::Relaxed)
}


pub fn fdl() -> u64 {
    AJV_.load(Ordering::Relaxed)
}


pub fn iyt(count: usize) -> alloc::vec::Vec<Ey> {
    let av = WQ_.load(Ordering::Acquire) as usize;
    if av == 0 {
        return alloc::vec::Vec::new();
    }

    let ae = count.min(av).min(JC_);
    let write_pos = BCV_.load(Ordering::Acquire);
    let mut events = alloc::vec::Vec::with_capacity(ae);

    for i in 0..ae {
        let idx = (write_pos + JC_ - 1 - i) % JC_;
        let slot = unsafe { &AHL_[idx] };
        if slot.seq > 0 {
            events.push(slot.event);
        }
    }

    events
}


pub fn gqq(count: usize) -> alloc::vec::Vec<Iz> {
    let av = AJV_.load(Ordering::Acquire) as usize;
    if av == 0 {
        return alloc::vec::Vec::new();
    }

    let ae = count.min(av).min(LK_);
    let write_pos = BIC_.load(Ordering::Acquire);
    let mut events = alloc::vec::Vec::with_capacity(ae);

    for i in 0..ae {
        let idx = (write_pos + LK_ - 1 - i) % LK_;
        let slot = unsafe { &BIB_[idx] };
        if slot.seq > 0 {
            events.push(slot.event);
        }
    }

    events
}


pub fn hrz() -> alloc::vec::Vec<(&'static str, u64, u64)> {
    
    let mut stats: alloc::vec::Vec<(&str, u64, u64)> = alloc::vec::Vec::new();

    let av = WQ_.load(Ordering::Acquire) as usize;
    let ae = av.min(JC_);

    for i in 0..ae {
        let slot = unsafe { &AHL_[i] };
        if slot.seq == 0 {
            continue;
        }

        let name = slot.event.device_name;
        if let Some(entry) = stats.iter_mut().find(|j| j.0 == name) {
            if slot.event.is_write {
                entry.2 += 1;
            } else {
                entry.1 += 1;
            }
        } else {
            if slot.event.is_write {
                stats.push((name, 0, 1));
            } else {
                stats.push((name, 1, 0));
            }
        }
    }

    stats
}





pub fn btg(ipa: u64) -> &'static str {
    match ipa {
        
        0x0800_0000..=0x0800_FFFF => "GIC-Dist",
        0x0801_0000..=0x0801_FFFF => "GIC-Redist",
        0x0802_0000..=0x0803_FFFF => "GIC-ITS",
        0x0900_0000..=0x0900_0FFF => "PL011-UART",
        0x0901_0000..=0x0901_0FFF => "RTC (PL031)",
        0x0903_0000..=0x0903_0FFF => "GPIO",
        0x0A00_0000..=0x0A00_01FF => "VirtIO-0",
        0x0A00_0200..=0x0A00_03FF => "VirtIO-1",
        0x0A00_0400..=0x0A00_05FF => "VirtIO-2",
        0x0A00_0600..=0x0A00_07FF => "VirtIO-3",
        0x0C00_0000..=0x0C1F_FFFF => "PCIe-ECAM",
        0x1000_0000..=0x3EFF_FFFF => "PCIe-MMIO",
        0x4010_0000..=0x4010_0FFF => "Platform-Bus",

        
        0x0B00_0000..=0x0B0F_FFFF => "QC-APCS-GIC",
        0x0B11_0000..=0x0B11_0FFF => "QC-Timer",
        0x0780_0000..=0x07FF_FFFF => "QC-BLSP-UART",
        0x0100_0000..=0x0100_FFFF => "QC-CLK-CTL",
        0x0050_0000..=0x005F_FFFF => "QC-CRYPTO",
        0x0080_0000..=0x008F_FFFF => "QC-IMEM",

        
        0x1000_0000..=0x100F_FFFF => "Exynos-CMU",
        0x1200_0000..=0x120F_FFFF => "Exynos-DMC",
        0x1385_0000..=0x1385_FFFF => "Exynos-UART",

        
        0x0000_0000..=0x07FF_FFFF => "LowPeripheral",
        0x0800_0000..=0x0FFF_FFFF => "MidPeripheral",
        _ => "Unknown-MMIO",
    }
}


pub fn lxo(event: &Ey) -> alloc::string::String {
    use alloc::format;
    let direction = if event.is_write { "WR" } else { "RD" };
    let td = match event.access_size {
        1 => "B",
        2 => "H",
        4 => "W",
        8 => "D",
        _ => "?",
    };
    format!(
        "[{}] {} @0x{:08X} = 0x{:X} ({}{})",
        event.device_name,
        direction,
        event.ipa,
        event.value,
        td,
        if event.was_inst_fetch { " IFETCH!" } else { "" }
    )
}


pub fn hzq(event: &Iz) -> alloc::string::String {
    use alloc::format;
    format!(
        "SMC {} FID=0x{:08X} x1=0x{:X} x2=0x{:X} x3=0x{:X}",
        event.smc_type_name,
        event.fid,
        event.x1,
        event.x2,
        event.x3,
    )
}
