







use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};


const IJ_: usize = 512;
const KR_: usize = 128;


#[derive(Debug, Clone, Copy)]
pub struct Lv {
    
    pub akh: u64,
    
    pub asf: u64,
    
    pub bn: u64,
    
    pub cct: u32,
    
    pub rm: bool,
    
    pub gwm: bool,
    
    pub dgg: &'static str,
}


#[derive(Debug, Clone, Copy)]
pub struct Um {
    
    pub aos: u64,
    
    pub dn: u64,
    
    pub hy: u64,
    
    pub ajr: u64,
    
    pub jqo: &'static str,
}


#[derive(Clone, Copy)]
struct Avt {
    ls: u64,
    id: Lv,
}

#[derive(Clone, Copy)]
struct Ayy {
    ls: u64,
    id: Um,
}


const BTK_: Lv = Lv {
    akh: 0,
    asf: 0,
    bn: 0,
    cct: 0,
    rm: false,
    gwm: false,
    dgg: "",
};


const BTL_: Um = Um {
    aos: 0,
    dn: 0,
    hy: 0,
    ajr: 0,
    jqo: "",
};


static mut AFR_: [Avt; IJ_] = {
    let gk = Avt { ls: 0, id: BTK_ };
    [gk; IJ_]
};

static mut BFX_: [Ayy; KR_] = {
    let gk = Ayy { ls: 0, id: BTL_ };
    [gk; KR_]
};


static BAT_: AtomicUsize = AtomicUsize::new(0);

static VH_: AtomicU64 = AtomicU64::new(0);


static BFY_: AtomicUsize = AtomicUsize::new(0);

static AHZ_: AtomicU64 = AtomicU64::new(0);


pub fn jdw(id: Lv) {
    let w = BAT_.fetch_add(1, Ordering::Relaxed) % IJ_;
    let ls = VH_.fetch_add(1, Ordering::Relaxed) + 1;
    unsafe {
        AFR_[w] = Avt { ls, id };
    }
}


pub fn uhx(id: Um) {
    let w = BFY_.fetch_add(1, Ordering::Relaxed) % KR_;
    let ls = AHZ_.fetch_add(1, Ordering::Relaxed) + 1;
    unsafe {
        BFX_[w] = Ayy { ls, id };
    }
}


pub fn mmj() -> u64 {
    VH_.load(Ordering::Relaxed)
}


pub fn jty() -> u64 {
    AHZ_.load(Ordering::Relaxed)
}


pub fn paq(az: usize) -> alloc::vec::Vec<Lv> {
    let es = VH_.load(Ordering::Acquire) as usize;
    if es == 0 {
        return alloc::vec::Vec::new();
    }

    let bo = az.v(es).v(IJ_);
    let bau = BAT_.load(Ordering::Acquire);
    let mut events = alloc::vec::Vec::fc(bo);

    for a in 0..bo {
        let w = (bau + IJ_ - 1 - a) % IJ_;
        let gk = unsafe { &AFR_[w] };
        if gk.ls > 0 {
            events.push(gk.id);
        }
    }

    events
}


pub fn lyf(az: usize) -> alloc::vec::Vec<Um> {
    let es = AHZ_.load(Ordering::Acquire) as usize;
    if es == 0 {
        return alloc::vec::Vec::new();
    }

    let bo = az.v(es).v(KR_);
    let bau = BFY_.load(Ordering::Acquire);
    let mut events = alloc::vec::Vec::fc(bo);

    for a in 0..bo {
        let w = (bau + KR_ - 1 - a) % KR_;
        let gk = unsafe { &BFX_[w] };
        if gk.ls > 0 {
            events.push(gk.id);
        }
    }

    events
}


pub fn nld() -> alloc::vec::Vec<(&'static str, u64, u64)> {
    
    let mut cm: alloc::vec::Vec<(&str, u64, u64)> = alloc::vec::Vec::new();

    let es = VH_.load(Ordering::Acquire) as usize;
    let bo = es.v(IJ_);

    for a in 0..bo {
        let gk = unsafe { &AFR_[a] };
        if gk.ls == 0 {
            continue;
        }

        let j = gk.id.dgg;
        if let Some(bt) = cm.el().du(|e| e.0 == j) {
            if gk.id.rm {
                bt.2 += 1;
            } else {
                bt.1 += 1;
            }
        } else {
            if gk.id.rm {
                cm.push((j, 0, 1));
            } else {
                cm.push((j, 1, 0));
            }
        }
    }

    cm
}





pub fn eda(akh: u64) -> &'static str {
    match akh {
        
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


pub fn svw(id: &Lv) -> alloc::string::String {
    use alloc::format;
    let sz = if id.rm { "WR" } else { "RD" };
    let als = match id.cct {
        1 => "B",
        2 => "H",
        4 => "W",
        8 => "D",
        _ => "?",
    };
    format!(
        "[{}] {} @0x{:08X} = 0x{:X} ({}{})",
        id.dgg,
        sz,
        id.akh,
        id.bn,
        als,
        if id.gwm { " IFETCH!" } else { "" }
    )
}


pub fn nvs(id: &Um) -> alloc::string::String {
    use alloc::format;
    format!(
        "SMC {} FID=0x{:08X} x1=0x{:X} x2=0x{:X} x3=0x{:X}",
        id.jqo,
        id.aos,
        id.dn,
        id.hy,
        id.ajr,
    )
}
