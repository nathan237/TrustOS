




use alloc::vec::Vec;
use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;


#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Ipv6Address(pub [u8; 16]);

impl Ipv6Address {
    pub const Afs: Self = Self([0; 16]);
    pub const Ayo: Self = Self([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,1]);
    
    pub const AMG_: Self = Self([0xff,0x02,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,1]);
    
    pub const BMM_: Self = Self([0xff,0x02,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,2]);

    
    pub const fn new(bytes: [u8; 16]) -> Self { Self(bytes) }

    
    pub fn lzk(mac: [u8; 6]) -> Self {
        let mut addr = [0u8; 16];
        addr[0] = 0xfe; addr[1] = 0x80;
        
        addr[8] = mac[0] ^ 0x02;  
        addr[9] = mac[1];
        addr[10] = mac[2];
        addr[11] = 0xff;
        addr[12] = 0xfe;
        addr[13] = mac[3];
        addr[14] = mac[4];
        addr[15] = mac[5];
        Self(addr)
    }

    
    pub fn is_link_local(&self) -> bool {
        self.0[0] == 0xfe && (self.0[1] & 0xc0) == 0x80
    }

    
    pub fn is_multicast(&self) -> bool {
        self.0[0] == 0xff
    }

    
    pub fn solicited_node_multicast(&self) -> Self {
        let mut addr = [0u8; 16];
        addr[0] = 0xff; addr[1] = 0x02;
        addr[11] = 0x01; addr[12] = 0xff;
        addr[13] = self.0[13];
        addr[14] = self.0[14];
        addr[15] = self.0[15];
        Self(addr)
    }

    
    pub fn multicast_mac(&self) -> [u8; 6] {
        [0x33, 0x33, self.0[12], self.0[13], self.0[14], self.0[15]]
    }
}

impl fmt::Display for Ipv6Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        let a = &self.0;
        write!(f, "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
            a[0],a[1], a[2],a[3], a[4],a[5], a[6],a[7],
            a[8],a[9], a[10],a[11], a[12],a[13], a[14],a[15])
    }
}

impl fmt::Debug for Ipv6Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ipv6({})", self)
    }
}


pub mod next_header {
    pub const DSY_: u8 = 0;
    pub const Aqh: u8 = 6;
    pub const Aqy: u8 = 17;
    pub const Ka: u8 = 58;
    pub const DYW_: u8 = 59;
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Ipv6Header {
    pub version_tc_fl: u32,   
    pub payload_length: u16,  
    pub next_header: u8,
    pub epj: u8,
    pub src: [u8; 16],
    pub dst: [u8; 16],
}

impl Ipv6Header {
    pub const Z: usize = 40;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::Z { return None; }
        Some(unsafe { core::ptr::read_unaligned(data.as_ptr() as *const Self) })
    }

    pub fn version(&self) -> u8 {
        ((u32::from_be(self.version_tc_fl) >> 28) & 0xF) as u8
    }

    pub fn payload_len(&self) -> u16 {
        u16::from_be(self.payload_length)
    }

    pub fn src_addr(&self) -> Ipv6Address { Ipv6Address(self.src) }
    pub fn dst_addr(&self) -> Ipv6Address { Ipv6Address(self.dst) }
}





static Cq: AtomicBool = AtomicBool::new(false);

struct Aag {
    link_local: Ipv6Address,
}

static Dz: Mutex<Aag> = Mutex::new(Aag {
    link_local: Ipv6Address::Afs,
});






pub fn init() {
    let mac = crate::drivers::net::aqt()
        .or_else(crate::network::aqu)
        .unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);

    let link_local = Ipv6Address::lzk(mac);
    Dz.lock().link_local = link_local;
    Cq.store(true, Ordering::SeqCst);

    crate::log!("[IPv6] Link-local: {}", link_local);

    
    let _ = super::icmpv6::ooa(link_local);
}


pub fn lq() -> bool {
    Cq.load(Ordering::Relaxed)
}


pub fn esz() -> Ipv6Address {
    Dz.lock().link_local
}


pub fn alq(data: &[u8]) {
    if !Cq.load(Ordering::Relaxed) { return; }

    let header = match Ipv6Header::parse(data) {
        Some(h) => h,
        None => return,
    };

    if header.version() != 6 { return; }

    let payload_len = header.payload_len() as usize;
    let payload = &data[Ipv6Header::Z..];
    if payload.len() < payload_len { return; }
    let payload = &payload[..payload_len];

    let dst = header.dst_addr();
    let amc = Dz.lock().link_local;

    
    let lxi = dst == amc
        || dst == Ipv6Address::AMG_
        || dst == amc.solicited_node_multicast()
        || dst.is_multicast();

    if !lxi { return; }

    match header.next_header {
        next_header::Ka => {
            super::icmpv6::alq(header.src_addr(), header.dst_addr(), payload);
        }
        next_header::Aqh => {
            crate::serial_println!("[IPv6] TCP packet from {} (not implemented)", header.src_addr());
        }
        next_header::Aqy => {
            crate::serial_println!("[IPv6] UDP packet from {} (not implemented)", header.src_addr());
        }
        _ => {}
    }
}


pub fn aha(dst: Ipv6Address, next_header: u8, payload: &[u8]) -> Result<(), &'static str> {
    let src = Dz.lock().link_local;
    if src == Ipv6Address::Afs {
        return Err("IPv6 not initialized");
    }

    fab(src, dst, next_header, 64, payload)
}


pub fn fab(
    src: Ipv6Address,
    dst: Ipv6Address,
    next_header: u8,
    epj: u8,
    payload: &[u8],
) -> Result<(), &'static str> {
    let mut be = Vec::with_capacity(Ipv6Header::Z + payload.len());

    
    let version_tc_fl: u32 = 6 << 28;
    be.extend_from_slice(&version_tc_fl.to_be_bytes());
    be.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    be.push(next_header);
    be.push(epj);
    be.extend_from_slice(&src.0);
    be.extend_from_slice(&dst.0);
    be.extend_from_slice(payload);

    
    let dst_mac = if dst.is_multicast() {
        dst.multicast_mac()
    } else {
        
        super::icmpv6::nar(dst).unwrap_or(Ipv6Address::AMG_.multicast_mac())
    };

    super::cdq(dst_mac, super::ethertype::Zz, &be)
}


pub fn epz(src: &Ipv6Address, dst: &Ipv6Address, payload: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    
    for df in src.0.chunks(2) {
        sum += ((df[0] as u32) << 8) | (df[1] as u32);
    }
    for df in dst.0.chunks(2) {
        sum += ((df[0] as u32) << 8) | (df[1] as u32);
    }
    
    sum += payload.len() as u32;
    
    sum += next_header::Ka as u32;

    
    let mut i = 0;
    while i + 1 < payload.len() {
        sum += ((payload[i] as u32) << 8) | (payload[i + 1] as u32);
        i += 2;
    }
    if i < payload.len() {
        sum += (payload[i] as u32) << 8;
    }

    
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !(sum as u16)
}
