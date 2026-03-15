




use alloc::vec::Vec;
use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;


#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Ipv6Address(pub [u8; 16]);

impl Ipv6Address {
    pub const Bvc: Self = Self([0; 16]);
    pub const Dbd: Self = Self([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,1]);
    
    pub const AKM_: Self = Self([0xff,0x02,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,1]);
    
    pub const BKC_: Self = Self([0xff,0x02,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,2]);

    
    pub const fn new(bf: [u8; 16]) -> Self { Self(bf) }

    
    pub fn sya(ed: [u8; 6]) -> Self {
        let mut ag = [0u8; 16];
        ag[0] = 0xfe; ag[1] = 0x80;
        
        ag[8] = ed[0] ^ 0x02;  
        ag[9] = ed[1];
        ag[10] = ed[2];
        ag[11] = 0xff;
        ag[12] = 0xfe;
        ag[13] = ed[3];
        ag[14] = ed[4];
        ag[15] = ed[5];
        Self(ag)
    }

    
    pub fn txx(&self) -> bool {
        self.0[0] == 0xfe && (self.0[1] & 0xc0) == 0x80
    }

    
    pub fn ogi(&self) -> bool {
        self.0[0] == 0xff
    }

    
    pub fn pma(&self) -> Self {
        let mut ag = [0u8; 16];
        ag[0] = 0xff; ag[1] = 0x02;
        ag[11] = 0x01; ag[12] = 0xff;
        ag[13] = self.0[13];
        ag[14] = self.0[14];
        ag[15] = self.0[15];
        Self(ag)
    }

    
    pub fn ooq(&self) -> [u8; 6] {
        [0x33, 0x33, self.0[12], self.0[13], self.0[14], self.0[15]]
    }
}

impl fmt::Display for Ipv6Address {
    fn fmt(&self, bb: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        let q = &self.0;
        write!(bb, "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
            q[0],q[1], q[2],q[3], q[4],q[5], q[6],q[7],
            q[8],q[9], q[10],q[11], q[12],q[13], q[14],q[15])
    }
}

impl fmt::Debug for Ipv6Address {
    fn fmt(&self, bb: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(bb, "Ipv6({})", self)
    }
}


pub mod next_header {
    pub const DPE_: u8 = 0;
    pub const Cnr: u8 = 6;
    pub const Com: u8 = 17;
    pub const Xb: u8 = 58;
    pub const DVF_: u8 = 59;
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Ipv6Header {
    pub mpc: u32,   
    pub oux: u16,  
    pub next_header: u8,
    pub iyn: u8,
    pub cy: [u8; 16],
    pub cs: [u8; 16],
}

impl Ipv6Header {
    pub const Am: usize = 40;

    pub fn parse(f: &[u8]) -> Option<Self> {
        if f.len() < Self::Am { return None; }
        Some(unsafe { core::ptr::md(f.fq() as *const Self) })
    }

    pub fn dk(&self) -> u8 {
        ((u32::eqv(self.mpc) >> 28) & 0xF) as u8
    }

    pub fn bvx(&self) -> u16 {
        u16::eqv(self.oux)
    }

    pub fn cbz(&self) -> Ipv6Address { Ipv6Address(self.cy) }
    pub fn dgu(&self) -> Ipv6Address { Ipv6Address(self.cs) }
}





static Li: AtomicBool = AtomicBool::new(false);

struct Bkc {
    eep: Ipv6Address,
}

static Jz: Mutex<Bkc> = Mutex::new(Bkc {
    eep: Ipv6Address::Bvc,
});






pub fn init() {
    let ed = crate::drivers::net::cez()
        .or_else(crate::network::ckt)
        .unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);

    let eep = Ipv6Address::sya(ed);
    Jz.lock().eep = eep;
    Li.store(true, Ordering::SeqCst);

    crate::log!("[IPv6] Link-local: {}", eep);

    
    let _ = super::icmpv6::whn(eep);
}


pub fn zu() -> bool {
    Li.load(Ordering::Relaxed)
}


pub fn jdo() -> Ipv6Address {
    Jz.lock().eep
}


pub fn bur(f: &[u8]) {
    if !Li.load(Ordering::Relaxed) { return; }

    let dh = match Ipv6Header::parse(f) {
        Some(i) => i,
        None => return,
    };

    if dh.dk() != 6 { return; }

    let bvx = dh.bvx() as usize;
    let ew = &f[Ipv6Header::Am..];
    if ew.len() < bvx { return; }
    let ew = &ew[..bvx];

    let cs = dh.dgu();
    let bvt = Jz.lock().eep;

    
    let svp = cs == bvt
        || cs == Ipv6Address::AKM_
        || cs == bvt.pma()
        || cs.ogi();

    if !svp { return; }

    match dh.next_header {
        next_header::Xb => {
            super::icmpv6::bur(dh.cbz(), dh.dgu(), ew);
        }
        next_header::Cnr => {
            crate::serial_println!("[IPv6] TCP packet from {} (not implemented)", dh.cbz());
        }
        next_header::Com => {
            crate::serial_println!("[IPv6] UDP packet from {} (not implemented)", dh.cbz());
        }
        _ => {}
    }
}


pub fn blc(cs: Ipv6Address, next_header: u8, ew: &[u8]) -> Result<(), &'static str> {
    let cy = Jz.lock().eep;
    if cy == Ipv6Address::Bvc {
        return Err("IPv6 not initialized");
    }

    joj(cy, cs, next_header, 64, ew)
}


pub fn joj(
    cy: Ipv6Address,
    cs: Ipv6Address,
    next_header: u8,
    iyn: u8,
    ew: &[u8],
) -> Result<(), &'static str> {
    let mut ex = Vec::fc(Ipv6Header::Am + ew.len());

    
    let mpc: u32 = 6 << 28;
    ex.bk(&mpc.ft());
    ex.bk(&(ew.len() as u16).ft());
    ex.push(next_header);
    ex.push(iyn);
    ex.bk(&cy.0);
    ex.bk(&cs.0);
    ex.bk(ew);

    
    let amc = if cs.ogi() {
        cs.ooq()
    } else {
        
        super::icmpv6::uih(cs).unwrap_or(Ipv6Address::AKM_.ooq())
    };

    super::fug(amc, super::ethertype::Bjg, &ex)
}


pub fn izf(cy: &Ipv6Address, cs: &Ipv6Address, ew: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    
    for jj in cy.0.btq(2) {
        sum += ((jj[0] as u32) << 8) | (jj[1] as u32);
    }
    for jj in cs.0.btq(2) {
        sum += ((jj[0] as u32) << 8) | (jj[1] as u32);
    }
    
    sum += ew.len() as u32;
    
    sum += next_header::Xb as u32;

    
    let mut a = 0;
    while a + 1 < ew.len() {
        sum += ((ew[a] as u32) << 8) | (ew[a + 1] as u32);
        a += 2;
    }
    if a < ew.len() {
        sum += (ew[a] as u32) << 8;
    }

    
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !(sum as u16)
}
