




use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};


pub const CIV_: usize = 1518;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkState {
    Down,
    Up,
    Error,
}


#[derive(Debug, Clone, Copy)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }
}

impl core::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5])
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }
    
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
}

impl core::fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}",
            self.0[0], self.0[1], self.0[2], self.0[3])
    }
}


pub struct Abr {
    pub mac: MacAddress,
    pub ip: Option<Ipv4Address>,
    pub subnet: Option<Ipv4Address>,
    pub gateway: Option<Ipv4Address>,
    pub state: NetworkState,
}


#[derive(Debug, Clone, Copy, Default)]
pub struct Tw {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
}


#[repr(C, packed)]
pub struct Avi {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,
}


#[repr(C, packed)]
pub struct ArpPacket {
    pub htype: u16,      
    pub ptype: u16,      
    pub hlen: u8,        
    pub plen: u8,        
    pub operation: u16,  
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}


static Dv: Mutex<Option<Abr>> = Mutex::new(None);


static AJG_: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());


static DCU_: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());


static Iw: Mutex<Tw> = Mutex::new(Tw {
    packets_sent: 0,
    packets_received: 0,
    bytes_sent: 0,
    bytes_received: 0,
    errors: 0,
});


static Ah: AtomicBool = AtomicBool::new(false);


static BDQ_: Mutex<Option<Tx>> = Mutex::new(None);


static GX_: Mutex<[u8; 4]> = Mutex::new([8, 8, 8, 8]);


static Acc: Mutex<Platform> = Mutex::new(Platform::Unknown);


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Unknown,
    Qemu,         
    QemuKvm,      
    VirtualBox,   
    VMware,       
    HyperV,       
    BareMetal,    
}

impl core::fmt::Display for Platform {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Platform::Unknown => write!(f, "Unknown"),
            Platform::Qemu => write!(f, "QEMU (TCG)"),
            Platform::QemuKvm => write!(f, "QEMU/KVM"),
            Platform::VirtualBox => write!(f, "VirtualBox"),
            Platform::VMware => write!(f, "VMware"),
            Platform::HyperV => write!(f, "Hyper-V"),
            Platform::BareMetal => write!(f, "Bare Metal"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Tx {
    pub vendor_id: u16,
    pub device_id: u16,
    pub vendor_name: String,
    pub driver: String,
    pub bar0: u64,
    pub irq: u8,
}


pub fn mdn() -> Option<Tx> {
    BDQ_.lock().clone()
}


fn mnm(vendor_id: u16, device_id: u16) -> &'static str {
    match (vendor_id, device_id) {
        
        (0x8086, 0x100E) => "e1000",      
        (0x8086, 0x100F) => "e1000",      
        (0x8086, 0x10D3) => "e1000e",     
        (0x8086, 0x153A) => "e1000e",     
        (0x8086, 0x15B8) => "e1000e",     
        
        
        (0x1AF4, 0x1000) => "virtio-net",
        (0x1AF4, 0x1041) => "virtio-net", 
        
        
        (0x10EC, 0x8139) => "rtl8139",
        (0x10EC, 0x8168) => "r8168",
        (0x10EC, 0x8169) => "r8169",
        
        
        (0x14E4, _) => "bnx2/tg3",
        
        
        (0x15AD, 0x0720) => "vmxnet",
        (0x15AD, 0x07B0) => "vmxnet3",
        
        _ => "generic",
    }
}


pub fn ldy() -> Platform {
    
    
    #[cfg(target_arch = "x86_64")]
    let (hv_max, dzl) = unsafe {
        let a: u32;
        let b: u32;
        let c: u32;
        let d: u32;
        
        
        core::arch::asm!(
            "push rbx",
            "mov eax, 0x40000000",
            "cpuid",
            "mov r8d, ebx",
            "pop rbx",
            out("eax") a,
            out("r8d") b,
            out("ecx") c,
            out("edx") d,
        );
        let mut sig = [0u8; 12];
        sig[0..4].copy_from_slice(&b.to_le_bytes());
        sig[4..8].copy_from_slice(&c.to_le_bytes());
        sig[8..12].copy_from_slice(&d.to_le_bytes());
        (a, sig)
    };
    #[cfg(not(target_arch = "x86_64"))]
    let (hv_max, dzl): (u32, [u8; 12]) = (0, [0u8; 12]);
    if hv_max >= 0x40000000 {
        
        if let Ok(j) = core::str::from_utf8(&dzl) {
            crate::serial_println!("[NET] Hypervisor CPUID: '{}'", j);
            if j.contains("KVMKVMKVM") {
                return Platform::QemuKvm;
            } else if j.contains("VBoxVBoxVBox") {
                return Platform::VirtualBox;
            } else if j.contains("VMwareVMware") {
                return Platform::VMware;
            } else if j.contains("Microsoft Hv") {
                return Platform::HyperV;
            } else if j.contains("TCGTCGTCGTCG") {
                return Platform::Qemu;
            }
        }
    }
    
    
    if let Some(info) = crate::acpi::rk() {
        let gkj = info.oem_id.trim();
        if gkj.eq_ignore_ascii_case("VBOX") {
            return Platform::VirtualBox;
        } else if gkj.eq_ignore_ascii_case("BOCHS") || gkj.eq_ignore_ascii_case("BXPC") {
            return Platform::Qemu;
        }
    }
    
    
    let ccn = crate::pci::aqs();
    for s in &ccn {
        match s.vendor_id {
            0x80EE => return Platform::VirtualBox, 
            0x15AD => return Platform::VMware,     
            0x1234 => return Platform::Qemu,       
            0x1AF4 => {
                
                
                return Platform::Qemu;
            }
            _ => {}
        }
    }
    
    
    Platform::BareMetal
}


pub fn fyv() -> Platform {
    *Acc.lock()
}


pub fn mcy() -> [u8; 4] {
    *GX_.lock()
}


pub fn oou(dns: [u8; 4]) {
    *GX_.lock() = dns;
    crate::log!("[NET] DNS server: {}.{}.{}.{}", dns[0], dns[1], dns[2], dns[3]);
}


pub fn init() {
    
    let platform = ldy();
    *Acc.lock() = platform;
    crate::log!("[NET] Platform detected: {}", platform);
    
    
    let dmp = match platform {
        Platform::Qemu | Platform::QemuKvm => [10, 0, 2, 3],     
        Platform::VirtualBox => [10, 0, 2, 3],                    
        _ => [8, 8, 8, 8],                                       
    };
    *GX_.lock() = dmp;
    crate::serial_println!("[NET] Default DNS: {}.{}.{}.{}", dmp[0], dmp[1], dmp[2], dmp[3]);
    
    
    let ipm = crate::pci::bsp(crate::pci::class::Gr);
    
    if ipm.is_empty() {
        crate::log_warn!("[NET] No network controller found");
        return;
    }
    
    
    let s = &ipm[0];
    let driver = mnm(s.vendor_id, s.device_id);
    
    
    let gjo = Tx {
        vendor_id: s.vendor_id,
        device_id: s.device_id,
        vendor_name: String::from(s.vendor_name()),
        driver: String::from(driver),
        bar0: s.bar_address(0).unwrap_or(0),
        irq: s.interrupt_line,
    };
    
    crate::log!("[NET] NIC: {:04X}:{:04X} {} [{}] BAR0={:#X} IRQ={}",
        s.vendor_id, s.device_id, 
        driver, s.vendor_name(),
        gjo.bar0, gjo.irq);
    
    *BDQ_.lock() = Some(gjo);
    
    
    crate::pci::bzi(s);
    crate::pci::bzj(s);
    
    
    let mac = if crate::virtio_net::is_initialized() {
        if let Some(real_mac) = crate::virtio_net::aqt() {
            MacAddress::new(real_mac)
        } else {
            iba()
        }
    } else {
        iba()
    };
    
    
    
    let interface = Abr {
        mac,
        ip: None,
        subnet: None,
        gateway: None,
        state: NetworkState::Up,
    };
    
    *Dv.lock() = Some(interface);
    Ah.store(true, Ordering::SeqCst);
    
    crate::log!("[NET] Interface up: MAC={} (awaiting DHCP)", mac);
}


fn iba() -> MacAddress {
    let gx = crate::logger::eg();
    MacAddress::new([
        0x52, 0x54, 0x00,  
        ((gx >> 8) & 0xFF) as u8,
        ((gx >> 16) & 0xFF) as u8,
        (gx & 0xFF) as u8,
    ])
}


pub fn sw() -> bool {
    Ah.load(Ordering::Relaxed)
}


pub fn cyp() -> Option<(MacAddress, Option<Ipv4Address>, NetworkState)> {
    let adv = Dv.lock();
    adv.as_ref().map(|i| (i.mac, i.ip, i.state))
}


pub fn rd() -> Option<(Ipv4Address, Ipv4Address, Option<Ipv4Address>)> {
    let adv = Dv.lock();
    adv.as_ref().and_then(|i| {
        let ip = i.ip?;
        let subnet = i.subnet.unwrap_or(Ipv4Address::new(255, 255, 255, 0));
        Some((ip, subnet, i.gateway))
    })
}


pub fn aqu() -> Option<[u8; 6]> {
    let adv = Dv.lock();
    adv.as_ref().map(|i| *i.mac.as_bytes())
}


pub fn get_stats() -> Tw {
    *Iw.lock()
}


pub fn aha(data: &[u8]) -> Result<(), &'static str> {
    if !Ah.load(Ordering::Relaxed) {
        return Err("Network not initialized");
    }
    
    if data.len() > CIV_ {
        return Err("Packet too large");
    }
    
    
    if crate::drivers::net::aoh() {
        return crate::drivers::net::send(data);
    }
    
    
    if crate::virtio_net::is_initialized() {
        return crate::virtio_net::aha(data);
    }
    
    
    let mut bu = DCU_.lock();
    bu.push_back(data.to_vec());
    
    
    let mut stats = Iw.lock();
    stats.packets_sent += 1;
    stats.bytes_sent += data.len() as u64;
    
    Ok(())
}


pub fn iyr() -> Option<Vec<u8>> {
    if !Ah.load(Ordering::Relaxed) {
        return None;
    }
    
    
    if crate::drivers::net::aoh() {
        if let Some(be) = crate::drivers::net::receive() {
            let mut stats = Iw.lock();
            stats.packets_received += 1;
            stats.bytes_received += be.len() as u64;
            return Some(be);
        }
    }
    
    
    if crate::virtio_net::is_initialized() {
        if let Some(be) = crate::virtio_net::iyr() {
            let mut stats = Iw.lock();
            stats.packets_received += 1;
            stats.bytes_received += be.len() as u64;
            return Some(be);
        }
    }
    
    
    let mut da = AJG_.lock();
    if let Some(be) = da.pop_front() {
        let mut stats = Iw.lock();
        stats.packets_received += 1;
        stats.bytes_received += be.len() as u64;
        Some(be)
    } else {
        None
    }
}


pub fn qlj(data: Vec<u8>) {
    let mut da = AJG_.lock();
    da.push_back(data);
}


pub fn qvf(target_ip: Ipv4Address) -> Result<(), &'static str> {
    let adv = Dv.lock();
    let interface = adv.as_ref().ok_or("No interface")?;
    
    let mut be = Vec::with_capacity(42); 
    
    
    be.extend_from_slice(&[0xFF; 6]); 
    be.extend_from_slice(interface.mac.as_bytes());
    be.push(0x08); be.push(0x06); 
    
    
    be.push(0x00); be.push(0x01); 
    be.push(0x08); be.push(0x00); 
    be.push(0x06);                     
    be.push(0x04);                     
    be.push(0x00); be.push(0x01); 
    be.extend_from_slice(interface.mac.as_bytes()); 
    if let Some(ip) = interface.ip {
        be.extend_from_slice(ip.as_bytes()); 
    } else {
        be.extend_from_slice(&[0; 4]);
    }
    be.extend_from_slice(&[0; 6]);    
    be.extend_from_slice(target_ip.as_bytes()); 
    
    drop(adv);
    aha(&be)
}


pub fn gty(target_ip: Ipv4Address) -> Result<Ul, &'static str> {
    
    static Aow: AtomicU64 = AtomicU64::new(1);
    let seq = Aow.fetch_add(1, Ordering::Relaxed) as u16;
    
    
    crate::netstack::icmp::dkt();
    
    
    let owh = crate::cpu::tsc::gjt();
    
    
    let erd = target_ip.as_bytes();
    crate::netstack::icmp::gtw([erd[0], erd[1], erd[2], erd[3]], 0x1234, seq)?;
    
    
    let fa = crate::netstack::icmp::hcb(seq, 1000);
    
    
    let elapsed_micros = crate::cpu::tsc::gjt().saturating_sub(owh);
    
    
    let time_ms = if elapsed_micros < 1000 {
        
        1
    } else {
        (elapsed_micros / 1000) as u32
    };
    
    if let Some(eo) = fa {
        
        let mut stats = Iw.lock();
        stats.packets_received += 1;
        stats.bytes_received += 64; 
        
        Ok(Ul {
            seq: eo.seq,
            ttl: eo.ttl,
            time_ms,
            time_us: elapsed_micros,
            success: true,
        })
    } else {
        
        Ok(Ul {
            seq,
            ttl: 0,
            time_ms,
            time_us: elapsed_micros,
            success: false,
        })
    }
}


#[derive(Debug, Clone)]
pub struct Ul {
    pub seq: u16,
    pub ttl: u8,
    pub time_ms: u32,
    pub time_us: u64,  
    pub success: bool,
}


fn qxa(target: &Ipv4Address) -> u32 {
    let bytes = target.as_bytes();
    
    
    if bytes[0] == 127 {
        return 1;
    }
    
    
    if bytes[0] == 10 && bytes[1] == 0 && bytes[2] == 2 && bytes[3] == 2 {
        return 2;
    }
    
    
    if bytes[0] == 10 && bytes[1] == 0 && bytes[2] == 2 {
        return 5;
    }
    
    
    if bytes[0] == 192 && bytes[1] == 168 {
        return 10;
    }
    if bytes[0] == 10 {
        return 15;
    }
    
    
    
    if bytes[0] == 8 && bytes[1] == 8 && bytes[2] == 8 && bytes[3] == 8 {
        return 25;
    }
    
    
    if bytes[0] == 1 && bytes[1] == 1 && bytes[2] == 1 && bytes[3] == 1 {
        return 20;
    }
    
    
    50
}


pub fn opd(ip: Ipv4Address, subnet: Ipv4Address, gateway: Ipv4Address) {
    let mut adv = Dv.lock();
    if let Some(ref mut interface) = *adv {
        interface.ip = Some(ip);
        interface.subnet = Some(subnet);
        interface.gateway = Some(gateway);
        crate::log!("[NET] IP configured: {}", ip);
    }
}


pub fn deh(ip: Ipv4Address, subnet: Ipv4Address, gateway: Option<Ipv4Address>) {
    let mut adv = Dv.lock();
    if let Some(ref mut interface) = *adv {
        interface.ip = Some(ip);
        interface.subnet = Some(subnet);
        interface.gateway = gateway;
        interface.state = NetworkState::Up;
    }
}




pub fn lqi() {
    let jvf = {
        let adv = Dv.lock();
        adv.as_ref().and_then(|i| i.ip).is_some()
    };
    if jvf {
        return;
    }
    let mac = aqu().unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    let host = if mac[5] == 0 { 1 } else if mac[5] == 255 { 254 } else { mac[5] };
    let ip = Ipv4Address::new(10, 0, 100, host);
    let subnet = Ipv4Address::new(255, 255, 255, 0);
    deh(ip, subnet, None);
    crate::serial_println!("[NET] Fallback IP from MAC: {}", ip);
}


pub fn up() {
    let mut adv = Dv.lock();
    if let Some(ref mut interface) = *adv {
        interface.state = NetworkState::Up;
        crate::log!("[NET] Interface up");
    }
}


pub fn dno() {
    let mut adv = Dv.lock();
    if let Some(ref mut interface) = *adv {
        interface.state = NetworkState::Down;
        crate::log!("[NET] Interface down");
    }
}


pub fn jpe() {
    
    if let Some(real_mac) = crate::drivers::net::aqt() {
        let mut adv = Dv.lock();
        if let Some(ref mut interface) = *adv {
            interface.mac = MacAddress::new(real_mac);
            crate::log!("[NET] MAC updated: {}", interface.mac);
        }
        return;
    }
    
    
    if let Some(real_mac) = crate::virtio_net::aqt() {
        let mut adv = Dv.lock();
        if let Some(ref mut interface) = *adv {
            interface.mac = MacAddress::new(real_mac);
            crate::log!("[NET] MAC updated: {}", interface.mac);
        }
    }
}


pub fn poll() {
    if crate::drivers::net::aoh() {
        crate::netstack::poll(); 
    } else if crate::virtio_net::is_initialized() {
        crate::virtio_net::poll();
    }
}


pub fn mjz() -> bool {
    crate::drivers::net::aoh() || crate::virtio_net::is_initialized()
}


pub fn mcz() -> (u64, u64, u64, u64) {
    if crate::drivers::net::aoh() {
        let stats = crate::drivers::net::stats();
        (stats.tx_packets, stats.rx_packets, stats.tx_bytes, stats.rx_bytes)
    } else if crate::virtio_net::is_initialized() {
        crate::virtio_net::get_stats()
    } else {
        let stats = Iw.lock();
        (stats.packets_sent, stats.packets_received, stats.bytes_sent, stats.bytes_received)
    }
}
