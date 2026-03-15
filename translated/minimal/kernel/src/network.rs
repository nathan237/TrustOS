




use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};


pub const CFM_: usize = 1518;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkState {
    Fm,
    Ek,
    Q,
}


#[derive(Debug, Clone, Copy)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const fn new(bf: [u8; 6]) -> Self {
        Self(bf)
    }
    
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }
}

impl core::fmt::Display for MacAddress {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(bb, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5])
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub const fn new(q: u8, o: u8, r: u8, bc: u8) -> Self {
        Self([q, o, r, bc])
    }
    
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
}

impl core::fmt::Display for Ipv4Address {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(bb, "{}.{}.{}.{}",
            self.0[0], self.0[1], self.0[2], self.0[3])
    }
}


pub struct Bnf {
    pub ed: MacAddress,
    pub ip: Option<Ipv4Address>,
    pub up: Option<Ipv4Address>,
    pub auj: Option<Ipv4Address>,
    pub g: NetworkState,
}


#[derive(Debug, Clone, Copy, Default)]
pub struct Avy {
    pub egc: u64,
    pub dub: u64,
    pub feb: u64,
    pub cdm: u64,
    pub bqn: u64,
}


#[repr(C, packed)]
pub struct Cwe {
    pub amc: [u8; 6],
    pub atn: [u8; 6],
    pub ethertype: u16,
}


#[repr(C, packed)]
pub struct ArpPacket {
    pub ock: u16,      
    pub frq: u16,      
    pub tpg: u8,        
    pub hvh: u8,        
    pub ayh: u16,  
    pub eik: [u8; 6],
    pub eij: [u8; 4],
    pub jsl: [u8; 6],
    pub blk: [u8; 4],
}


static Jo: Mutex<Option<Bnf>> = Mutex::new(None);


static AHK_: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());


static CZC_: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());


static Uh: Mutex<Avy> = Mutex::new(Avy {
    egc: 0,
    dub: 0,
    feb: 0,
    cdm: 0,
    bqn: 0,
});


static Be: AtomicBool = AtomicBool::new(false);


static BBN_: Mutex<Option<Awb>> = Mutex::new(None);


static GG_: Mutex<[u8; 4]> = Mutex::new([8, 8, 8, 8]);


static Boj: Mutex<Platform> = Mutex::new(Platform::F);


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    F,
    Yi,         
    Axg,      
    Afo,   
    Baj,       
    Biw,       
    Bcn,    
}

impl core::fmt::Display for Platform {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Platform::F => write!(bb, "Unknown"),
            Platform::Yi => write!(bb, "QEMU (TCG)"),
            Platform::Axg => write!(bb, "QEMU/KVM"),
            Platform::Afo => write!(bb, "VirtualBox"),
            Platform::Baj => write!(bb, "VMware"),
            Platform::Biw => write!(bb, "Hyper-V"),
            Platform::Bcn => write!(bb, "Bare Metal"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Awb {
    pub ml: u16,
    pub mx: u16,
    pub cip: String,
    pub rj: String,
    pub aew: u64,
    pub irq: u8,
}


pub fn tef() -> Option<Awb> {
    BBN_.lock().clone()
}


fn trk(ml: u16, mx: u16) -> &'static str {
    match (ml, mx) {
        
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


pub fn rwu() -> Platform {
    
    
    #[cfg(target_arch = "x86_64")]
    let (ocn, iap) = unsafe {
        let q: u32;
        let o: u32;
        let r: u32;
        let bc: u32;
        
        
        core::arch::asm!(
            "push rbx",
            "mov eax, 0x40000000",
            "cpuid",
            "mov r8d, ebx",
            "pop rbx",
            bd("eax") q,
            bd("r8d") o,
            bd("ecx") r,
            bd("edx") bc,
        );
        let mut sig = [0u8; 12];
        sig[0..4].dg(&o.ho());
        sig[4..8].dg(&r.ho());
        sig[8..12].dg(&bc.ho());
        (q, sig)
    };
    #[cfg(not(target_arch = "x86_64"))]
    let (ocn, iap): (u32, [u8; 12]) = (0, [0u8; 12]);
    if ocn >= 0x40000000 {
        
        if let Ok(e) = core::str::jg(&iap) {
            crate::serial_println!("[NET] Hypervisor CPUID: '{}'", e);
            if e.contains("KVMKVMKVM") {
                return Platform::Axg;
            } else if e.contains("VBoxVBoxVBox") {
                return Platform::Afo;
            } else if e.contains("VMwareVMware") {
                return Platform::Baj;
            } else if e.contains("Microsoft Hv") {
                return Platform::Biw;
            } else if e.contains("TCGTCGTCGTCG") {
                return Platform::Yi;
            }
        }
    }
    
    
    if let Some(co) = crate::acpi::ani() {
        let lpu = co.clo.em();
        if lpu.dha("VBOX") {
            return Platform::Afo;
        } else if lpu.dha("BOCHS") || lpu.dha("BXPC") {
            return Platform::Yi;
        }
    }
    
    
    let jix = crate::pci::fjm();
    for ba in &jix {
        match ba.ml {
            0x80EE => return Platform::Afo, 
            0x15AD => return Platform::Baj,     
            0x1234 => return Platform::Yi,       
            0x1AF4 => {
                
                
                return Platform::Yi;
            }
            _ => {}
        }
    }
    
    
    Platform::Bcn
}


pub fn tej() -> Platform {
    *Boj.lock()
}


pub fn tdl() -> [u8; 4] {
    *GG_.lock()
}


pub fn wis(dns: [u8; 4]) {
    *GG_.lock() = dns;
    crate::log!("[NET] DNS server: {}.{}.{}.{}", dns[0], dns[1], dns[2], dns[3]);
}


pub fn init() {
    
    let platform = rwu();
    *Boj.lock() = platform;
    crate::log!("[NET] Platform detected: {}", platform);
    
    
    let hfr = match platform {
        Platform::Yi | Platform::Axg => [10, 0, 2, 3],     
        Platform::Afo => [10, 0, 2, 3],                    
        _ => [8, 8, 8, 8],                                       
    };
    *GG_.lock() = hfr;
    crate::serial_println!("[NET] Default DNS: {}.{}.{}.{}", hfr[0], hfr[1], hfr[2], hfr[3]);
    
    
    let opg = crate::pci::ebq(crate::pci::class::Qa);
    
    if opg.is_empty() {
        crate::log_warn!("[NET] No network controller found");
        return;
    }
    
    
    let ba = &opg[0];
    let rj = trk(ba.ml, ba.mx);
    
    
    let lot = Awb {
        ml: ba.ml,
        mx: ba.mx,
        cip: String::from(ba.cip()),
        rj: String::from(rj),
        aew: ba.cje(0).unwrap_or(0),
        irq: ba.esw,
    };
    
    crate::log!("[NET] NIC: {:04X}:{:04X} {} [{}] BAR0={:#X} IRQ={}",
        ba.ml, ba.mx, 
        rj, ba.cip(),
        lot.aew, lot.irq);
    
    *BBN_.lock() = Some(lot);
    
    
    crate::pci::fhp(ba);
    crate::pci::fhq(ba);
    
    
    let ed = if crate::virtio_net::ky() {
        if let Some(hxd) = crate::virtio_net::cez() {
            MacAddress::new(hxd)
        } else {
            nxl()
        }
    } else {
        nxl()
    };
    
    
    
    let akf = Bnf {
        ed,
        ip: None,
        up: None,
        auj: None,
        g: NetworkState::Ek,
    };
    
    *Jo.lock() = Some(akf);
    Be.store(true, Ordering::SeqCst);
    
    crate::log!("[NET] Interface up: MAC={} (awaiting DHCP)", ed);
}


fn nxl() -> MacAddress {
    let qb = crate::logger::lh();
    MacAddress::new([
        0x52, 0x54, 0x00,  
        ((qb >> 8) & 0xFF) as u8,
        ((qb >> 16) & 0xFF) as u8,
        (qb & 0xFF) as u8,
    ])
}


pub fn anl() -> bool {
    Be.load(Ordering::Relaxed)
}


pub fn gif() -> Option<(MacAddress, Option<Ipv4Address>, NetworkState)> {
    let ben = Jo.lock();
    ben.as_ref().map(|a| (a.ed, a.ip, a.g))
}


pub fn aou() -> Option<(Ipv4Address, Ipv4Address, Option<Ipv4Address>)> {
    let ben = Jo.lock();
    ben.as_ref().and_then(|a| {
        let ip = a.ip?;
        let up = a.up.unwrap_or(Ipv4Address::new(255, 255, 255, 0));
        Some((ip, up, a.auj))
    })
}


pub fn ckt() -> Option<[u8; 6]> {
    let ben = Jo.lock();
    ben.as_ref().map(|a| *a.ed.as_bytes())
}


pub fn asx() -> Avy {
    *Uh.lock()
}


pub fn blc(f: &[u8]) -> Result<(), &'static str> {
    if !Be.load(Ordering::Relaxed) {
        return Err("Network not initialized");
    }
    
    if f.len() > CFM_ {
        return Err("Packet too large");
    }
    
    
    if crate::drivers::net::bzy() {
        return crate::drivers::net::baq(f);
    }
    
    
    if crate::virtio_net::ky() {
        return crate::virtio_net::blc(f);
    }
    
    
    let mut gx = CZC_.lock();
    gx.agt(f.ip());
    
    
    let mut cm = Uh.lock();
    cm.egc += 1;
    cm.feb += f.len() as u64;
    
    Ok(())
}


pub fn pao() -> Option<Vec<u8>> {
    if !Be.load(Ordering::Relaxed) {
        return None;
    }
    
    
    if crate::drivers::net::bzy() {
        if let Some(ex) = crate::drivers::net::chb() {
            let mut cm = Uh.lock();
            cm.dub += 1;
            cm.cdm += ex.len() as u64;
            return Some(ex);
        }
    }
    
    
    if crate::virtio_net::ky() {
        if let Some(ex) = crate::virtio_net::pao() {
            let mut cm = Uh.lock();
            cm.dub += 1;
            cm.cdm += ex.len() as u64;
            return Some(ex);
        }
    }
    
    
    let mut kb = AHK_.lock();
    if let Some(ex) = kb.awp() {
        let mut cm = Uh.lock();
        cm.dub += 1;
        cm.cdm += ex.len() as u64;
        Some(ex)
    } else {
        None
    }
}


pub fn yya(f: Vec<u8>) {
    let mut kb = AHK_.lock();
    kb.agt(f);
}


pub fn zmg(blk: Ipv4Address) -> Result<(), &'static str> {
    let ben = Jo.lock();
    let akf = ben.as_ref().ok_or("No interface")?;
    
    let mut ex = Vec::fc(42); 
    
    
    ex.bk(&[0xFF; 6]); 
    ex.bk(akf.ed.as_bytes());
    ex.push(0x08); ex.push(0x06); 
    
    
    ex.push(0x00); ex.push(0x01); 
    ex.push(0x08); ex.push(0x00); 
    ex.push(0x06);                     
    ex.push(0x04);                     
    ex.push(0x00); ex.push(0x01); 
    ex.bk(akf.ed.as_bytes()); 
    if let Some(ip) = akf.ip {
        ex.bk(ip.as_bytes()); 
    } else {
        ex.bk(&[0; 4]);
    }
    ex.bk(&[0; 6]);    
    ex.bk(blk.as_bytes()); 
    
    drop(ben);
    blc(&ex)
}


pub fn mdt(blk: Ipv4Address) -> Result<Awy, &'static str> {
    
    static Clh: AtomicU64 = AtomicU64::new(1);
    let ls = Clh.fetch_add(1, Ordering::Relaxed) as u16;
    
    
    crate::netstack::icmp::hcx();
    
    
    let wsw = crate::cpu::tsc::loz();
    
    
    let jaw = blk.as_bytes();
    crate::netstack::icmp::mdr([jaw[0], jaw[1], jaw[2], jaw[3]], 0x1234, ls)?;
    
    
    let mk = crate::netstack::icmp::mqe(ls, 1000);
    
    
    let fhk = crate::cpu::tsc::loz().ao(wsw);
    
    
    let ejx = if fhk < 1000 {
        
        1
    } else {
        (fhk / 1000) as u32
    };
    
    if let Some(lj) = mk {
        
        let mut cm = Uh.lock();
        cm.dub += 1;
        cm.cdm += 64; 
        
        Ok(Awy {
            ls: lj.ls,
            akv: lj.akv,
            ejx,
            dwu: fhk,
            vx: true,
        })
    } else {
        
        Ok(Awy {
            ls,
            akv: 0,
            ejx,
            dwu: fhk,
            vx: false,
        })
    }
}


#[derive(Debug, Clone)]
pub struct Awy {
    pub ls: u16,
    pub akv: u8,
    pub ejx: u32,
    pub dwu: u64,  
    pub vx: bool,
}


fn zon(cd: &Ipv4Address) -> u32 {
    let bf = cd.as_bytes();
    
    
    if bf[0] == 127 {
        return 1;
    }
    
    
    if bf[0] == 10 && bf[1] == 0 && bf[2] == 2 && bf[3] == 2 {
        return 2;
    }
    
    
    if bf[0] == 10 && bf[1] == 0 && bf[2] == 2 {
        return 5;
    }
    
    
    if bf[0] == 192 && bf[1] == 168 {
        return 10;
    }
    if bf[0] == 10 {
        return 15;
    }
    
    
    
    if bf[0] == 8 && bf[1] == 8 && bf[2] == 8 && bf[3] == 8 {
        return 25;
    }
    
    
    if bf[0] == 1 && bf[1] == 1 && bf[2] == 1 && bf[3] == 1 {
        return 20;
    }
    
    
    50
}


pub fn znf(ip: Ipv4Address, up: Ipv4Address, auj: Ipv4Address) {
    let mut ben = Jo.lock();
    if let Some(ref mut akf) = *ben {
        akf.ip = Some(ip);
        akf.up = Some(up);
        akf.auj = Some(auj);
        crate::log!("[NET] IP configured: {}", ip);
    }
}


pub fn hzx(ip: Ipv4Address, up: Ipv4Address, auj: Option<Ipv4Address>) {
    let mut ben = Jo.lock();
    if let Some(ref mut akf) = *ben {
        akf.ip = Some(ip);
        akf.up = Some(up);
        akf.auj = auj;
        akf.g = NetworkState::Ek;
    }
}




pub fn slt() {
    let qhi = {
        let ben = Jo.lock();
        ben.as_ref().and_then(|a| a.ip).is_some()
    };
    if qhi {
        return;
    }
    let ed = ckt().unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
    let kh = if ed[5] == 0 { 1 } else if ed[5] == 255 { 254 } else { ed[5] };
    let ip = Ipv4Address::new(10, 0, 100, kh);
    let up = Ipv4Address::new(255, 255, 255, 0);
    hzx(ip, up, None);
    crate::serial_println!("[NET] Fallback IP from MAC: {}", ip);
}


pub fn bln() {
    let mut ben = Jo.lock();
    if let Some(ref mut akf) = *ben {
        akf.g = NetworkState::Ek;
        crate::log!("[NET] Interface up");
    }
}


pub fn hgr() {
    let mut ben = Jo.lock();
    if let Some(ref mut akf) = *ben {
        akf.g = NetworkState::Fm;
        crate::log!("[NET] Interface down");
    }
}


pub fn pxf() {
    
    if let Some(hxd) = crate::drivers::net::cez() {
        let mut ben = Jo.lock();
        if let Some(ref mut akf) = *ben {
            akf.ed = MacAddress::new(hxd);
            crate::log!("[NET] MAC updated: {}", akf.ed);
        }
        return;
    }
    
    
    if let Some(hxd) = crate::virtio_net::cez() {
        let mut ben = Jo.lock();
        if let Some(ref mut akf) = *ben {
            akf.ed = MacAddress::new(hxd);
            crate::log!("[NET] MAC updated: {}", akf.ed);
        }
    }
}


pub fn poll() {
    if crate::drivers::net::bzy() {
        crate::netstack::poll(); 
    } else if crate::virtio_net::ky() {
        crate::virtio_net::poll();
    }
}


pub fn tnb() -> bool {
    crate::drivers::net::bzy() || crate::virtio_net::ky()
}


pub fn tdm() -> (u64, u64, u64, u64) {
    if crate::drivers::net::bzy() {
        let cm = crate::drivers::net::cm();
        (cm.cuz, cm.dbo, cm.bpc, cm.bsc)
    } else if crate::virtio_net::ky() {
        crate::virtio_net::asx()
    } else {
        let cm = Uh.lock();
        (cm.egc, cm.dub, cm.feb, cm.cdm)
    }
}
