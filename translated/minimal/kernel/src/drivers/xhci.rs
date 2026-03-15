














use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::string::String;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;






#[repr(C)]
pub struct Aoo {
    pub gcf: u8,          
    pub asi: u8,          
    pub tnq: u16,        
    pub lbq: u32,        
    pub iyd: u32,        
    pub yws: u32,        
    pub obd: u32,        
    pub rtv: u32,             
    pub wbd: u32,            
    pub ywr: u32,        
}

impl Aoo {
    pub fn efi(&self) -> u8 {
        (self.lbq & 0xFF) as u8
    }
    
    pub fn zck(&self) -> u16 {
        ((self.lbq >> 8) & 0x7FF) as u16
    }
    
    pub fn fnx(&self) -> u8 {
        ((self.lbq >> 24) & 0xFF) as u8
    }
    
    pub fn enz(&self) -> usize {
        if (self.obd & (1 << 2)) != 0 { 64 } else { 32 }
    }
}


#[repr(C)]
pub struct Bxb {
    pub igb: u32,            
    pub gvs: u32,            
    pub zer: u32,          
    pub fzp: [u32; 2],   
    pub ymo: u32,            
    pub rqh: u64,              
    pub jyk: [u32; 4],   
    pub rtw: u64,            
    pub config: u32,            
    
}


const BHY_: u32 = 1 << 0;
const AJI_: u32 = 1 << 1;
const CZR_: u32 = 1 << 2;


const XZ_: u32 = 1 << 0;  
const BHZ_: u32 = 1 << 11; 


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Aop {
    pub bht: u32,    
    pub zga: u32,  
    pub zfz: u32,    
    pub zfy: u32, 
}


pub(crate) const AGO_: u32 = 1 << 0;     
pub(crate) const OZ_: u32 = 1 << 1;     
pub(crate) const PA_: u32 = 1 << 4;      
pub(crate) const EAS_: u32 = 0xF << 5; 
pub(crate) const EAT_: u32 = 1 << 9;      
pub(crate) const BDK_: u32 = 0xF << 10; 
pub(crate) const BDJ_: u32 = 1 << 17;    
pub(crate) const KY_: u32 = 1 << 21;    


const BGC_: u32 = 1;   
const BGD_: u32 = 2;    
const AIA_: u32 = 3;   
const AIB_: u32 = 4;  


#[repr(C, align(16))]
#[derive(Clone, Copy, Default)]
pub struct Trb {
    pub bhr: u64,
    pub status: u32,
    pub control: u32,
}

impl Trb {
    pub fn new() -> Self {
        Self { bhr: 0, status: 0, control: 0 }
    }
    
    pub fn arl(uul: u64) -> Self {
        Self {
            bhr: uul,
            status: 0,
            control: (XU_ << 10) | DA_,
        }
    }
    
    pub fn fah(&self) -> u8 {
        ((self.control >> 10) & 0x3F) as u8
    }
    
    pub fn yld(&self) -> bool {
        (self.control & DA_) != 0
    }
}


pub(crate) const AJE_: u32 = 1;
pub(crate) const XV_: u32 = 2;
pub(crate) const AJD_: u32 = 3;
pub(crate) const XW_: u32 = 4;
pub(crate) const XU_: u32 = 6;
pub(crate) const EIT_: u32 = 7;
pub(crate) const EIU_: u32 = 8;
pub(crate) const CYN_: u32 = 9;
pub(crate) const EIR_: u32 = 10;
pub(crate) const CYM_: u32 = 11;
pub(crate) const BHQ_: u32 = 12;
pub(crate) const EIS_: u32 = 13;
pub(crate) const EIX_: u32 = 14;
pub(crate) const EIV_: u32 = 23;


pub(crate) const CYP_: u32 = 32;
pub(crate) const BHP_: u32 = 33;
pub(crate) const EIW_: u32 = 34;


pub(crate) const DA_: u32 = 1 << 0;
pub(crate) const EV_: u32 = 1 << 5;   


#[repr(C, align(64))]
pub struct Bdq {
    pub cuw: [Trb; 256],
}


#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct Bgb {
    pub vyy: u64,
    pub hxs: u16,
    pub asi: [u16; 3],
}


#[repr(C)]
pub struct Zp {
    pub tsa: u32,      
    pub tsg: u32,      
    pub snj: u32,    
    pub asi: u32,
    pub sni: u64,    
    pub fhy: u64,      
}


#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct SlotContext {
    pub f: [u32; 8],
}


#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct EndpointContext {
    pub f: [u32; 8],
}


#[repr(C, align(64))]
pub struct Ahk {
    pub gk: SlotContext,
    pub nqe: [EndpointContext; 31],
}


#[repr(C, align(64))]
pub struct Czu {
    pub yyd: Cfy,
    pub gk: SlotContext,
    pub nqe: [EndpointContext; 31],
}

#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct Cfy {
    pub ynr: u32,
    pub yek: u32,
    pub asi: [u32; 6],
}






#[derive(Clone, Debug)]
pub struct Ve {
    pub fw: u8,
    pub port: u8,
    pub ig: u8,
    pub ml: u16,
    pub cgt: u16,
    pub class: u8,
    pub adl: u8,
    pub protocol: u8,
    pub lpi: u8,
    pub gmh: u16,
    pub lkg: String,
    pub baj: String,
}


pub struct Bm {
    pub blz: u64,
    pub cvt: u64,
    pub feh: *mut Aoo,
    pub lqr: *mut Bxb,
    pub bub: u64,
    pub ftj: u64,
    
    
    pub hfh: Box<[u64; 256]>,
    pub kof: u64,
    
    
    pub ffb: Box<Bdq>,
    pub hdj: u64,
    pub hdg: usize,
    pub hdf: bool,
    
    
    pub fib: Box<[Trb; 256]>,
    pub epz: u64,
    pub ktz: Box<[Bgb; 1]>,
    pub kua: u64,
    pub bgy: usize,
    pub cqg: bool,
    
    
    pub kpp: [Option<Box<Ahk>>; 256],
    
    
    pub ik: Vec<Ve>,
    
    pub efi: u8,
    pub fnx: u8,
    pub enz: usize,
    pub jr: bool,
}



unsafe impl Send for Bm {}

pub(crate) static Bn: Mutex<Option<Bm>> = Mutex::new(None);
pub(crate) static Be: AtomicBool = AtomicBool::new(false);





fn abw(ju: u64) -> u64 {
    let hp = crate::memory::lr();
    ju.nj(hp)
}

pub fn auv(ht: u64) -> u64 {
    let hp = crate::memory::lr();
    ht.cn(hp)
}






pub fn init(aew: u64) -> bool {
    if aew == 0 || aew == 0xFFFFFFFF {
        crate::serial_println!("[xHCI] Invalid BAR0");
        return false;
    }
    
    crate::serial_println!("[xHCI] Initializing controller at phys {:#x}", aew);
    
    
    let cvt = match crate::memory::bki(aew, 0x4000) {
        Ok(p) => p,
        Err(aa) => {
            crate::serial_println!("[xHCI] Failed to map MMIO: {}", aa);
            return false;
        }
    };
    
    crate::serial_println!("[xHCI] Mapped to virt {:#x}", cvt);
    
    
    let feh = cvt as *mut Aoo;
    let mh = unsafe { &*feh };
    
    let gcf = mh.gcf as u64;
    let dk = mh.tnq;
    let efi = mh.efi();
    let fnx = mh.fnx();
    let enz = mh.enz();
    
    crate::serial_println!("[xHCI] Version: {}.{}", dk >> 8, dk & 0xFF);
    crate::serial_println!("[xHCI] Max slots: {}, Max ports: {}, Context size: {}", 
        efi, fnx, enz);
    
    
    let uyo = cvt + gcf;
    let lqr = uyo as *mut Bxb;
    
    
    let bub = cvt + (mh.rtv as u64);
    let ftj = cvt + (mh.wbd as u64);
    
    
    
    
    let qaq = ((mh.obd >> 16) & 0xFFFF) as u64;
    if qaq != 0 {
        let mut epp = cvt + (qaq << 2);
        for _ in 0..32 {
            let isi = unsafe { core::ptr::read_volatile(epp as *const u32) };
            let sil = isi & 0xFF;
            let jgy = (isi >> 8) & 0xFF;

            if sil == 1 {
                
                crate::serial_println!("[xHCI] Found USBLEGSUP at offset {:#x}", epp - cvt);

                let qpp = (isi >> 16) & 1;
                if qpp != 0 {
                    crate::serial_println!("[xHCI] BIOS owns controller, requesting handoff...");

                    
                    unsafe { core::ptr::write_volatile(epp as *mut u32, isi | (1 << 24)); }

                    
                    let mut bq = false;
                    for a in 0..1000u32 {
                        let p = unsafe { core::ptr::read_volatile(epp as *const u32) };
                        if (p >> 16) & 1 == 0 {
                            bq = true;
                            crate::serial_println!("[xHCI] BIOS handoff complete ({}ms)", a);
                            break;
                        }
                        for _ in 0..10000 { core::hint::hc(); }
                    }
                    if !bq {
                        crate::serial_println!("[xHCI] WARNING: BIOS handoff timed out, forcing");
                        let p = unsafe { core::ptr::read_volatile(epp as *const u32) };
                        unsafe { core::ptr::write_volatile(epp as *mut u32, (p & !(1u32 << 16)) | (1 << 24)); }
                    }

                    
                    
                    let rre = (epp + 4) as *mut u32;
                    unsafe { core::ptr::write_volatile(rre, 0); }
                    crate::serial_println!("[xHCI] USB SMI disabled");
                } else {
                    crate::serial_println!("[xHCI] No BIOS ownership, handoff not needed");
                }
                break;
            }

            if jgy == 0 { break; }
            epp += (jgy as u64) << 2;
        }
    }

    
    let op = unsafe { &mut *lqr };
    if (op.gvs & XZ_) == 0 {
        crate::serial_println!("[xHCI] Halting controller...");
        op.igb &= !BHY_;
        
        
        for _ in 0..1000 {
            if (op.gvs & XZ_) != 0 {
                break;
            }
            for _ in 0..10000 { core::hint::hc(); }
        }
    }
    
    
    crate::serial_println!("[xHCI] Resetting controller...");
    op.igb |= AJI_;
    
    
    for _ in 0..1000 {
        if (op.igb & AJI_) == 0 && (op.gvs & BHZ_) == 0 {
            break;
        }
        for _ in 0..10000 { core::hint::hc(); }
    }
    
    if (op.igb & AJI_) != 0 || (op.gvs & BHZ_) != 0 {
        crate::serial_println!("[xHCI] Reset failed!");
        return false;
    }
    
    crate::serial_println!("[xHCI] Reset complete");
    
    
    let mut hfh = Box::new([0u64; 256]);
    let kof = abw(hfh.fq() as u64);
    
    
    let mut ffb = Box::new(Bdq { cuw: [Trb::new(); 256] });
    let hdj = abw(ffb.cuw.fq() as u64);
    
    
    ffb.cuw[255] = Trb::arl(hdj);
    
    
    let fib = Box::new([Trb::new(); 256]);
    let epz = abw(fib.fq() as u64);
    
    
    let mut ktz = Box::new([Bgb {
        vyy: epz,
        hxs: 256,
        asi: [0; 3],
    }]);
    let kua = abw(ktz.fq() as u64);
    
    
    op.config = efi as u32;
    
    
    op.rtw = kof;
    
    
    op.rqh = hdj | 1;
    
    
    let flv = (ftj + 0x20) as *mut Zp;
    let edo = unsafe { &mut *flv };
    
    edo.snj = 1;  
    edo.sni = kua;
    edo.fhy = epz;
    edo.tsa = 0;    
    edo.tsg = 0;
    
    
    op.igb = BHY_ | CZR_;
    
    
    for _ in 0..1000 {
        if (op.gvs & XZ_) == 0 {
            break;
        }
        for _ in 0..10000 { core::hint::hc(); }
    }
    
    if (op.gvs & XZ_) != 0 {
        crate::serial_println!("[xHCI] Failed to start controller");
        return false;
    }
    
    crate::serial_println!("[xHCI] Controller running");
    
    
    const CHY_: Option<Box<Ahk>> = None;
    let kpp: [Option<Box<Ahk>>; 256] = [CHY_; 256];
    
    
    let df = Bm {
        blz: aew,
        cvt,
        feh,
        lqr,
        bub,
        ftj,
        hfh,
        kof,
        ffb,
        hdj,
        hdg: 0,
        hdf: true,
        fib,
        epz,
        ktz,
        kua,
        bgy: 0,
        cqg: true,
        kpp,
        ik: Vec::new(),
        efi,
        fnx,
        enz,
        jr: true,
    };
    
    *Bn.lock() = Some(df);
    Be.store(true, Ordering::SeqCst);
    
    
    {
        let mut db = Bn.lock();
        if let Some(r) = db.as_mut() {
            qgv(r);
        }
    }
    
    
    ttv(efi);
    
    
    sml();
    
    
    wkn();
    
    true
}


fn sml() {
    let mut db = Bn.lock();
    let df = match db.as_mut() {
        Some(r) => r,
        None => return,
    };
    
    let hvr = df.cvt + 
        (unsafe { &*df.feh }.gcf as u64) + 0x400;
    
    crate::serial_println!("[xHCI] Enumerating {} ports...", df.fnx);
    
    for kg in 0..df.fnx {
        let hvs = (hvr + (kg as u64 * 16)) as *mut Aop;
        let port = unsafe { &mut *hvs };
        
        let bht = port.bht;
        
        
        if (bht & AGO_) != 0 {
            let ig = (bht & BDK_) >> 10;
            let mgr = match ig {
                BGD_ => "Low (1.5 Mbps)",
                BGC_ => "Full (12 Mbps)",
                AIA_ => "High (480 Mbps)",
                AIB_ => "Super (5 Gbps)",
                _ => "Unknown",
            };
            
            crate::serial_println!("[xHCI] Port {}: Device connected, speed: {}", 
                kg + 1, mgr);
            
            
            port.bht = bht | BDJ_ | KY_;
            
            
            if (bht & OZ_) == 0 {
                crate::serial_println!("[xHCI] Port {}: Resetting...", kg + 1);
                
                
                port.bht = (bht & !OZ_) | PA_;
                
                
                for _ in 0..100 {
                    for _ in 0..100000 { core::hint::hc(); }
                    let fox = port.bht;
                    if (fox & PA_) == 0 && (fox & KY_) != 0 {
                        
                        port.bht = fox | KY_;
                        break;
                    }
                }
                
                let sst = port.bht;
                if (sst & OZ_) != 0 {
                    crate::serial_println!("[xHCI] Port {}: Enabled after reset", kg + 1);
                    
                    
                    df.ik.push(Ve {
                        fw: 0,
                        port: kg + 1,
                        ig: ig as u8,
                        ml: 0,
                        cgt: 0,
                        class: 0,
                        adl: 0,
                        protocol: 0,
                        lpi: 0,
                        gmh: 0,
                        lkg: String::new(),
                        baj: String::new(),
                    });
                }
            } else {
                crate::serial_println!("[xHCI] Port {}: Already enabled", kg + 1);
                
                df.ik.push(Ve {
                    fw: 0,
                    port: kg + 1,
                    ig: ig as u8,
                    ml: 0,
                    cgt: 0,
                    class: 0,
                    adl: 0,
                    protocol: 0,
                    lpi: 0,
                    gmh: 0,
                    lkg: String::new(),
                    baj: String::new(),
                });
            }
        }
    }
    
    crate::serial_println!("[xHCI] Found {} connected devices", df.ik.len());
}






pub(crate) fn icd(df: &mut Bm, abj: Trb) {
    let w = df.hdg;
    
    
    let mut cmd = abj;
    if df.hdf {
        cmd.control |= DA_;
    } else {
        cmd.control &= !DA_;
    }
    
    df.ffb.cuw[w] = cmd;
    
    
    df.hdg += 1;
    if df.hdg >= 255 {
        
        let lit = (XU_ << 10) | if df.hdf { DA_ } else { 0 } | (1 << 1); 
        df.ffb.cuw[255].control = lit;
        df.ffb.cuw[255].bhr = df.hdj;
        df.hdf = !df.hdf;
        df.hdg = 0;
    }
    
    
    unsafe {
        let ng = df.bub as *mut u32;
        ptr::write_volatile(ng, 0);
    }
}


fn jwg(df: &mut Bm) -> Option<(u8, u8, u64)> {
    for _ in 0..2_000_000u32 {
        let w = df.bgy;
        let abj = df.fib[w];
        
        
        let ib = (abj.control & DA_) != 0;
        if ib == df.cqg {
            
            df.bgy += 1;
            if df.bgy >= 256 {
                df.bgy = 0;
                df.cqg = !df.cqg;
            }
            
            
            let hig = df.epz + (df.bgy as u64 * 16);
            let flv = (df.ftj + 0x20) as *mut Zp;
            unsafe {
                (*flv).fhy = hig | (1 << 3); 
            }
            
            let fah = (abj.control >> 10) & 0x3F;
            
            if fah == BHP_ {
                let enu = ((abj.status >> 24) & 0xFF) as u8;
                let fw = ((abj.control >> 24) & 0xFF) as u8;
                return Some((enu, fw, abj.bhr));
            }
            
            continue;
        }
        core::hint::hc();
    }
    None
}


fn jwh(df: &mut Bm) -> Option<(u8, u32, u8)> {
    for _ in 0..5_000_000u32 {
        let w = df.bgy;
        let abj = df.fib[w];
        
        let ib = (abj.control & DA_) != 0;
        if ib == df.cqg {
            df.bgy += 1;
            if df.bgy >= 256 {
                df.bgy = 0;
                df.cqg = !df.cqg;
            }
            
            let hig = df.epz + (df.bgy as u64 * 16);
            let flv = (df.ftj + 0x20) as *mut Zp;
            unsafe {
                (*flv).fhy = hig | (1 << 3);
            }
            
            let fah = (abj.control >> 10) & 0x3F;
            
            if fah == CYP_ {
                let enu = ((abj.status >> 24) & 0xFF) as u8;
                let mmt = abj.status & 0xFFFFFF;
                let ktp = ((abj.control >> 16) & 0x1F) as u8;
                return Some((enu, mmt, ktp));
            }
            if fah == BHP_ {
                let enu = ((abj.status >> 24) & 0xFF) as u8;
                let fw = ((abj.control >> 24) & 0xFF) as u8;
                return Some((enu, 0, fw));
            }
            continue;
        }
        core::hint::hc();
    }
    None
}






pub(crate) struct TransferRing {
    cuw: Box<[Trb; 256]>,
    pub(crate) ht: u64,
    pub(crate) ggg: usize,
    pub(crate) bgq: bool,
}

impl TransferRing {
    pub(crate) fn new() -> Option<Self> {
        let cuw = Box::new([Trb::new(); 256]);
        let ht = abw(cuw.fq() as u64);
        Some(Self { cuw, ht, ggg: 0, bgq: true })
    }
    
    pub(crate) fn azt(&mut self, mut abj: Trb) {
        if self.bgq {
            abj.control |= DA_;
        } else {
            abj.control &= !DA_;
        }
        self.cuw[self.ggg] = abj;
        self.ggg += 1;
        if self.ggg >= 255 {
            
            let lit = (XU_ << 10) | if self.bgq { DA_ } else { 0 } | (1 << 1);
            self.cuw[255].control = lit;
            self.cuw[255].bhr = self.ht;
            self.bgq = !self.bgq;
            self.ggg = 0;
        }
    }
}



pub(crate) struct Bta {
    pub(crate) beb: TransferRing,         
    pub(crate) lff: Option<TransferRing>, 
    pub(crate) lfe: u8,         
    pub(crate) fea: Option<TransferRing>,    
    pub(crate) dzi: u8,
    pub(crate) gbu: Option<TransferRing>,   
    pub(crate) enc: u8,
}





fn qgv(df: &mut Bm) {
    let mh = unsafe { &*df.feh };
    let iyd = mh.iyd;
    
    let gd = ((iyd >> 21) & 0x1F) as u32;
    let hh = ((iyd >> 27) & 0x1F) as u32;
    let lpg = (gd << 5) | hh;
    
    if lpg == 0 {
        return;
    }
    
    crate::serial_println!("[xHCI] Allocating {} scratchpad buffers", lpg);
    
    
    let mwj = match crate::memory::frame::azg() {
        Some(ai) => ai,
        None => { crate::serial_println!("[xHCI] OOM for scratchpad array"); return; }
    };
    let qkq = auv(mwj) as *mut u64;
    
    
    for a in 0..lpg.v(512) as usize {
        if let Some(dai) = crate::memory::frame::azg() {
            unsafe { ptr::write_volatile(qkq.add(a), dai); }
        }
    }
    
    
    df.hfh[0] = mwj;
}






pub(crate) static CC_: Mutex<Vec<Option<Bta>>> = Mutex::new(Vec::new());

fn ttv(efi: u8) {
    let mut um = CC_.lock();
    um.clear();
    for _ in 0..=efi {
        um.push(None);
    }
}


fn sld(df: &mut Bm) -> Option<u8> {
    let abj = Trb {
        bhr: 0,
        status: 0,
        control: (CYN_ << 10),
    };
    
    icd(df, abj);
    
    if let Some((nn, fw, qdg)) = jwg(df) {
        if nn == 1 { 
            crate::serial_println!("[xHCI] Enable Slot → slot_id={}", fw);
            return Some(fw);
        }
        crate::serial_println!("[xHCI] Enable Slot failed: cc={}", nn);
    }
    None
}


fn qft(df: &mut Bm, fw: u8, kg: u8, ig: u8) -> bool {
    
    let nlb = Box::new(Ahk {
        gk: SlotContext::default(),
        nqe: [EndpointContext::default(); 31],
    });
    let rwy = abw(&*nlb as *const _ as u64);
    df.hfh[fw as usize] = rwy;
    df.kpp[fw as usize] = Some(nlb);
    
    
    let nqs = match TransferRing::new() {
        Some(m) => m,
        None => { crate::serial_println!("[xHCI] OOM for EP0 ring"); return false; }
    };
    let smp = nqs.ht;
    
    
    let jag = match crate::memory::frame::azg() {
        Some(ai) => ai,
        None => { crate::serial_println!("[xHCI] OOM for input context"); return false; }
    };
    let les = auv(jag);
    let eac = df.enz;
    
    
    unsafe {
        let fkw = les as *mut u32;
        ptr::write_volatile(fkw.add(1), 0x3); 
    }
    
    
    let wpq = les + eac as u64;
    let czp = match ig as u32 {
        BGD_ => 8u16,
        BGC_ => 8,
        AIA_ => 64,
        AIB_ => 512,
        _ => 64,
    };
    
    unsafe {
        let gk = wpq as *mut u32;
        
        let ibc = (ig as u32) << 20;
        let hdy = 1u32 << 27; 
        ptr::write_volatile(gk, ibc | hdy);
        
        ptr::write_volatile(gk.add(1), (kg as u32) << 16);
    }
    
    
    let smo = les + (2 * eac) as u64;
    unsafe {
        let ktv = smo as *mut u32;
        
        let smy = 4u32 << 3;
        let gci = 3u32 << 1;
        let lmv = (czp as u32) << 16;
        ptr::write_volatile(ktv.add(1), gci | smy | lmv);
        
        let xkx = smp | 1; 
        ptr::write_volatile(ktv.add(2) as *mut u64, xkx);
        
        ptr::write_volatile(ktv.add(4), 8);
    }
    
    
    {
        let mut um = CC_.lock();
        if (fw as usize) < um.len() {
            um[fw as usize] = Some(Bta {
                beb: nqs,
                lff: None,
                lfe: 0,
                fea: None,
                dzi: 0,
                gbu: None,
                enc: 0,
            });
        }
    }
    
    
    let abj = Trb {
        bhr: jag,
        status: 0,
        control: (CYM_ << 10) | ((fw as u32) << 24),
    };
    
    icd(df, abj);
    
    if let Some((nn, ycu, qdg)) = jwg(df) {
        if nn == 1 {
            crate::serial_println!("[xHCI] Address Device slot {} → success", fw);
            crate::memory::frame::apt(jag);
            return true;
        }
        crate::serial_println!("[xHCI] Address Device failed: cc={}", nn);
    }
    crate::memory::frame::apt(jag);
    false
}






const LK_: u8 = 0x80;
const YA_: u8 = 0x00;
const LM_: u8 = 0x00;
const LL_: u8 = 0x00;
const QH_: u8 = 0x01;

const QI_: u8 = 0x06;
const CZX_: u8 = 0x09;
const CZZ_: u8 = 0x0B;
const CZY_: u8 = 0x0A;

const CZT_: u8 = 0x01;
const BIA_: u8 = 0x02;
const EJQ_: u8 = 0x03;
const CZV_: u8 = 0x04;
const CZU_: u8 = 0x05;
const EJO_: u8 = 0x21;
const EJP_: u8 = 0x22;


fn kks(
    df: &mut Bm,
    fw: u8,
    cdh: u8,
    cda: u8,
    cis: u16,
    cir: u16,
    bsw: u16,
    rg: u64,
) -> Option<u32> {
    let mut um = CC_.lock();
    let bcr = um.ds(fw as usize)?.as_mut()?;
    
    
    let dlq = (cdh as u64)
        | ((cda as u64) << 8)
        | ((cis as u64) << 16)
        | ((cir as u64) << 32)
        | ((bsw as u64) << 48);
    
    let mfd = Trb {
        bhr: dlq,
        status: 8, 
        control: (XV_ << 10) | (1 << 6) | (3 << 16), 
    };
    bcr.beb.azt(mfd);
    
    
    if bsw > 0 {
        let iqq = Trb {
            bhr: rg,
            status: bsw as u32,
            control: (AJD_ << 10) | (1 << 16), 
        };
        bcr.beb.azt(iqq);
    }
    
    
    let mhp = Trb {
        bhr: 0,
        status: 0,
        control: (XW_ << 10) | EV_, 
    };
    bcr.beb.azt(mhp);
    
    
    drop(um);
    unsafe {
        let ng = (df.bub + (fw as u64) * 4) as *mut u32;
        ptr::write_volatile(ng, 1); 
    }
    
    
    if let Some((nn, dmq, xyy)) = jwh(df) {
        if nn == 1 || nn == 13 { 
            return Some(dmq);
        }
        crate::serial_println!("[xHCI] Control IN failed: cc={}", nn);
    }
    None
}


fn kkt(
    df: &mut Bm,
    fw: u8,
    cdh: u8,
    cda: u8,
    cis: u16,
    cir: u16,
) -> bool {
    let mut um = CC_.lock();
    let bcr = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
        Some(m) => m,
        None => return false,
    };
    
    let dlq = (cdh as u64)
        | ((cda as u64) << 8)
        | ((cis as u64) << 16)
        | ((cir as u64) << 32);
    
    let mfd = Trb {
        bhr: dlq,
        status: 8,
        control: (XV_ << 10) | (1 << 6), 
    };
    bcr.beb.azt(mfd);
    
    let mhp = Trb {
        bhr: 0,
        status: 0,
        control: (XW_ << 10) | EV_ | (1 << 16), 
    };
    bcr.beb.azt(mhp);
    
    drop(um);
    unsafe {
        let ng = (df.bub + (fw as u64) * 4) as *mut u32;
        ptr::write_volatile(ng, 1);
    }
    
    if let Some((nn, _, _)) = jwh(df) {
        return nn == 1;
    }
    false
}






fn tdh(df: &mut Bm, fw: u8, ba: &mut Ve) -> bool {
    let rg = match crate::memory::frame::azg() {
        Some(ai) => ai,
        None => return false,
    };
    let aak = auv(rg) as *const u8;
    
    
    let result = kks(
        df, fw,
        LK_ | LM_ | LL_,
        QI_,
        (CZT_ as u16) << 8,
        0, 18, rg,
    );
    
    if result.is_some() {
        unsafe {
            let xya = ptr::md(aak.add(2) as *const u16);
            ba.class = *aak.add(4);
            ba.adl = *aak.add(5);
            ba.protocol = *aak.add(6);
            ba.gmh = *aak.add(7) as u16;
            ba.ml = ptr::md(aak.add(8) as *const u16);
            ba.cgt = ptr::md(aak.add(10) as *const u16);
            ba.lpi = *aak.add(17);
        }
        crate::serial_println!("[xHCI] Device: VID={:04X} PID={:04X} class={:02X}:{:02X}:{:02X}",
            ba.ml, ba.cgt, ba.class, ba.adl, ba.protocol);
    }
    
    crate::memory::frame::apt(rg);
    result.is_some()
}



fn tdc(df: &mut Bm, fw: u8) 
    -> Option<Vec<(u8, u8, u8, u8, u8, u16, u8)>> 
{
    let rg = match crate::memory::frame::azg() {
        Some(ai) => ai,
        None => return None,
    };
    let aak = auv(rg) as *const u8;
    
    
    kks(
        df, fw,
        LK_ | LM_ | LL_,
        QI_,
        (BIA_ as u16) << 8,
        0, 9, rg,
    )?;
    
    let dmo = unsafe { ptr::md(aak.add(2) as *const u16) };
    let cgy = dmo.v(4096);
    
    
    kks(
        df, fw,
        LK_ | LM_ | LL_,
        QI_,
        (BIA_ as u16) << 8,
        0, cgy, rg,
    )?;
    
    let rnr = unsafe { *aak.add(5) };
    let mut hoi = Vec::new();
    
    
    let mut l = 0usize;
    let mut eof = (0u8, 0u8, 0u8, 0u8); 
    
    while l + 1 < cgy as usize {
        let len = unsafe { *aak.add(l) } as usize;
        let rwd = unsafe { *aak.add(l + 1) };
        
        if len == 0 { break; }
        
        match rwd {
            CZV_ if len >= 9 => {
                let gjk = unsafe { *aak.add(l + 2) };
                let drx = unsafe { *aak.add(l + 5) };
                let ldg = unsafe { *aak.add(l + 6) };
                let izl = unsafe { *aak.add(l + 7) };
                
                eof = (drx, gjk, ldg, izl);
                
                if drx == 0x03 {
                    crate::serial_println!("[xHCI]   HID interface {}: subclass={} protocol={} ({})",
                        gjk, ldg, izl,
                        match izl { 1 => "keyboard", 2 => "mouse", _ => "other" });
                } else if drx == 0x08 {
                    crate::serial_println!("[xHCI]   Mass Storage interface {}: subclass={:#x} protocol={:#x}",
                        gjk, ldg, izl);
                }
            }
            CZU_ if len >= 7 => {
                let dgz = unsafe { *aak.add(l + 2) };
                let smq = unsafe { *aak.add(l + 3) };
                let nqu = unsafe { ptr::md(aak.add(l + 4) as *const u16) };
                let nqt = unsafe { *aak.add(l + 6) };
                let nqv = smq & 0x03;
                
                let drx = eof.0;
                if drx == 0x03 && nqv == 3 && (dgz & 0x80 != 0) {
                    
                    hoi.push((
                        drx, eof.1, eof.2, eof.3,
                        dgz, nqu & 0x7FF, nqt,
                    ));
                } else if drx == 0x08 && nqv == 2 {
                    
                    hoi.push((
                        drx, eof.1, eof.2, eof.3,
                        dgz, nqu & 0x7FF, nqt,
                    ));
                }
            }
            _ => {}
        }
        
        l += len;
    }
    
    
    if !hoi.is_empty() {
        kkt(
            df, fw,
            YA_ | LM_ | LL_,
            CZX_,
            rnr as u16,
            0,
        );
    }
    
    crate::memory::frame::apt(rg);
    
    if hoi.is_empty() { None } else { Some(hoi) }
}


fn rnt(
    df: &mut Bm,
    fw: u8,
    kg: u8,
    ig: u8,
    dgz: u8,
    czp: u16,
    crp: u8,
) -> bool {
    let smt = dgz & 0x0F;
    let bms = (smt * 2 + 1) as u8; 
    
    
    let jal = match TransferRing::new() {
        Some(m) => m,
        None => return false,
    };
    let tvm = jal.ht;
    
    
    let flo = match crate::memory::frame::azg() {
        Some(ai) => ai,
        None => return false,
    };
    let esr = auv(flo);
    let eac = df.enz;
    
    unsafe {
        let fkw = esr as *mut u32;
        
        ptr::write_volatile(fkw.add(1), 1 | (1u32 << bms));
        
        
        let gk = (esr + eac as u64) as *mut u32;
        let ibc = (ig as u32) << 20;
        let hdy = (bms as u32) << 27;
        ptr::write_volatile(gk, ibc | hdy);
        ptr::write_volatile(gk.add(1), (kg as u32) << 16);
        
        
        let isy = (esr + ((1 + bms as usize) * eac) as u64) as *mut u32;
        
        
        let xwm = match ig as u32 {
            AIA_ | AIB_ => crp.am(1) as u32,
            _ => {
                
                let vj = (crp as u32).am(1) * 8;
                let mut okb = 0u32;
                let mut p = vj;
                while p > 1 { p >>= 1; okb += 1; }
                okb + 1
            }
        };
        ptr::write_volatile(isy, (xwm << 16));
        
        
        let smz = 7u32 << 3;
        let gci = 3u32 << 1;
        let lmv = (czp as u32) << 16;
        ptr::write_volatile(isy.add(1), gci | smz | lmv);
        
        
        ptr::write_volatile(isy.add(2) as *mut u64, tvm | 1);
        
        
        ptr::write_volatile(isy.add(4), (czp as u32) | ((czp as u32) << 16));
    }
    
    
    {
        let mut um = CC_.lock();
        if let Some(Some(bcr)) = um.ds(fw as usize) {
            bcr.lff = Some(jal);
            bcr.lfe = bms;
        }
    }
    
    
    let abj = Trb {
        bhr: flo,
        status: 0,
        control: (BHQ_ << 10) | ((fw as u32) << 24),
    };
    
    icd(df, abj);
    
    let vx = if let Some((nn, _, _)) = jwg(df) {
        if nn == 1 {
            crate::serial_println!("[xHCI] Configure Endpoint slot {} DCI {} → success", fw, bms);
            true
        } else {
            crate::serial_println!("[xHCI] Configure Endpoint failed: cc={}", nn);
            false
        }
    } else {
        false
    };
    
    crate::memory::frame::apt(flo);
    vx
}


fn rns(
    df: &mut Bm,
    fw: u8,
    kg: u8,
    ig: u8,
    smr: u8,
    smu: u8,
    hrd: u16,
    hre: u16,
) -> bool {
    let sms = smr & 0x0F;
    let hfi = (sms * 2 + 1) as u8;  
    let smv = smu & 0x0F;
    let hfj = (smv * 2) as u8;     
    let ula = hfi.am(hfj);
    
    
    let nar = match TransferRing::new() {
        Some(m) => m,
        None => return false,
    };
    let nas = match TransferRing::new() {
        Some(m) => m,
        None => return false,
    };
    let tsn = nar.ht;
    let uzx = nas.ht;
    
    
    let flo = match crate::memory::frame::azg() {
        Some(ai) => ai,
        None => return false,
    };
    let esr = auv(flo);
    let eac = df.enz;
    
    unsafe {
        let fkw = esr as *mut u32;
        
        ptr::write_volatile(fkw.add(1), 1 | (1u32 << hfi) | (1u32 << hfj));
        
        
        let gk = (esr + eac as u64) as *mut u32;
        let ibc = (ig as u32) << 20;
        let hdy = (ula as u32) << 27;
        ptr::write_volatile(gk, ibc | hdy);
        ptr::write_volatile(gk.add(1), (kg as u32) << 16);
        
        
        let ktw = (esr + ((1 + hfi as usize) * eac) as u64) as *mut u32;
        let gci = 3u32 << 1;
        let smw = 6u32 << 3;
        let uqa = (hrd as u32) << 16;
        ptr::write_volatile(ktw.add(1), gci | smw | uqa);
        ptr::write_volatile(ktw.add(2) as *mut u64, tsn | 1);
        ptr::write_volatile(ktw.add(4), hrd as u32);
        
        
        let ktx = (esr + ((1 + hfj as usize) * eac) as u64) as *mut u32;
        let smx = 2u32 << 3;
        let uqb = (hre as u32) << 16;
        ptr::write_volatile(ktx.add(1), gci | smx | uqb);
        ptr::write_volatile(ktx.add(2) as *mut u64, uzx | 1);
        ptr::write_volatile(ktx.add(4), hre as u32);
    }
    
    
    {
        let mut um = CC_.lock();
        if let Some(Some(bcr)) = um.ds(fw as usize) {
            bcr.fea = Some(nar);
            bcr.dzi = hfi;
            bcr.gbu = Some(nas);
            bcr.enc = hfj;
        }
    }
    
    
    let abj = Trb {
        bhr: flo,
        status: 0,
        control: (BHQ_ << 10) | ((fw as u32) << 24),
    };
    
    icd(df, abj);
    
    let vx = if let Some((nn, _, _)) = jwg(df) {
        if nn == 1 {
            crate::serial_println!("[xHCI] Bulk endpoints configured: slot {} IN_DCI={} OUT_DCI={}",
                fw, hfi, hfj);
            true
        } else {
            crate::serial_println!("[xHCI] Configure bulk EPs failed: cc={}", nn);
            false
        }
    } else {
        false
    };
    
    crate::memory::frame::apt(flo);
    vx
}






fn wij(df: &mut Bm, fw: u8, akf: u8) -> bool {
    kkt(
        df, fw,
        YA_ | (1 << 5) | QH_, 
        CZZ_,
        0, 
        akf as u16,
    )
}


fn wiy(df: &mut Bm, fw: u8, akf: u8) -> bool {
    kkt(
        df, fw,
        YA_ | (1 << 5) | QH_,
        CZY_,
        0, 
        akf as u16,
    )
}


fn vpk(df: &Bm, fw: u8, rg: u64, czp: u16) -> bool {
    let mut um = CC_.lock();
    let bcr = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
        Some(m) => m,
        None => return false,
    };
    
    let jal = match bcr.lff.as_mut() {
        Some(m) => m,
        None => return false,
    };
    let bms = bcr.lfe;
    
    let abj = Trb {
        bhr: rg,
        status: czp as u32,
        control: (AJE_ << 10) | EV_,
    };
    jal.azt(abj);
    
    
    drop(um);
    unsafe {
        let ng = (df.bub + (fw as u64) * 4) as *mut u32;
        ptr::write_volatile(ng, bms as u32);
    }
    
    true
}









fn vmp(report: &[u8]) {
    if report.len() < 8 { return; }
    
    let ybi = report[0];
    
    
    for &fmi in &report[2..8] {
        if fmi == 0 { continue; }
        
        
        let ascii = toq(fmi, report[0]);
        if ascii != 0 {
            crate::keyboard::voj(ascii);
        }
    }
}






fn vmq(report: &[u8]) {
    if report.len() < 3 { return; }
    
    let cjk = report[0];
    let dx = report[1] as i8 as i32;
    let bg = report[2] as i8 as i32;
    let jc = if report.len() >= 4 { report[3] as i8 } else { 0 };
    
    crate::mouse::tup(
        dx, bg,
        cjk & 1 != 0,
        cjk & 2 != 0,
        cjk & 4 != 0,
        jc,
    );
}


fn toq(fmi: u8, modifiers: u8) -> u8 {
    let acn = (modifiers & 0x22) != 0; 
    let msg = (modifiers & 0x11) != 0;
    
    match fmi {
        
        0x04..=0x1D => {
            let ar = b'a' + (fmi - 0x04);
            if acn { ar - 32 } else { ar }
        }
        
        0x1E..=0x26 => {
            if acn {
                match fmi {
                    0x1E => b'!', 0x1F => b'@', 0x20 => b'#', 0x21 => b'$',
                    0x22 => b'%', 0x23 => b'^', 0x24 => b'&', 0x25 => b'*',
                    0x26 => b'(',
                    _ => 0,
                }
            } else {
                b'1' + (fmi - 0x1E)
            }
        }
        0x27 => if acn { b')' } else { b'0' },
        0x28 => b'\r',   
        0x29 => 0x1B,    
        0x2A => 0x08,    
        0x2B => b'\t',   
        0x2C => b' ',    
        0x2D => if acn { b'_' } else { b'-' },
        0x2E => if acn { b'+' } else { b'=' },
        0x2F => if acn { b'{' } else { b'[' },
        0x30 => if acn { b'}' } else { b']' },
        0x31 => if acn { b'|' } else { b'\\' },
        0x33 => if acn { b':' } else { b';' },
        0x34 => if acn { b'"' } else { b'\'' },
        0x35 => if acn { b'~' } else { b'`' },
        0x36 => if acn { b'<' } else { b',' },
        0x37 => if acn { b'>' } else { b'.' },
        0x38 => if acn { b'?' } else { b'/' },
        _ => 0,
    }
}






fn wkn() {
    let mut oof: Vec<(u8, u8, u8, u16, u16)> = Vec::new();
    
    {
        let mut db = Bn.lock();
        let df = match db.as_mut() {
            Some(r) => r,
            None => return,
        };
        
        let lpj = df.ik.len();
        if lpj == 0 {
            return;
        }
        
        crate::serial_println!("[xHCI] Setting up {} devices...", lpj);
        
        
        for a in 0..lpj {
            let port = df.ik[a].port;
            let ig = df.ik[a].ig;
            
            
            let fw = match sld(df) {
                Some(ad) => ad,
                None => {
                    crate::serial_println!("[xHCI] Failed to enable slot for port {}", port);
                    continue;
                }
            };
            
            df.ik[a].fw = fw;
            
            
            if !qft(df, fw, port, ig) {
                crate::serial_println!("[xHCI] Failed to address device on port {}", port);
                continue;
            }
            
            
            let mut ba = df.ik[a].clone();
            if !tdh(df, fw, &mut ba) {
                crate::serial_println!("[xHCI] Failed to get device descriptor for slot {}", fw);
                continue;
            }
            df.ik[a] = ba;
            
            
            if let Some(qgh) = tdc(df, fw) {
                let mut ooh: Option<(u8, u16)> = None;
                let mut ooi: Option<(u8, u16)> = None;
                
                for &(drx, gjk, adl, protocol, dgz, czp, crp) in &qgh {
                    match drx {
                        0x03 => {
                            
                            let _ = wij(df, fw, gjk);
                            let _ = wiy(df, fw, gjk);
                            
                            rnt(
                                df, fw, port, ig,
                                dgz, czp, crp,
                            );
                            
                            if df.ik[a].class == 0 {
                                df.ik[a].class = 0x03;
                                df.ik[a].adl = adl;
                                df.ik[a].protocol = protocol;
                            }
                            
                            crate::serial_println!("[xHCI] HID endpoint configured: slot {} EP {:#x} max_pkt {} interval {}",
                                fw, dgz, czp, crp);
                        }
                        0x08 => {
                            
                            if dgz & 0x80 != 0 {
                                ooh = Some((dgz, czp));
                            } else {
                                ooi = Some((dgz, czp));
                            }
                        }
                        _ => {}
                    }
                }
                
                
                if let (Some((izp, izs)), Some((jhz, jib))) = (ooh, ooi) {
                    if rns(df, fw, port, ig, izp, jhz, izs, jib) {
                        oof.push((fw, izp, jhz, izs, jib));
                        
                        if df.ik[a].class == 0 {
                            df.ik[a].class = 0x08;
                            df.ik[a].adl = 0x06;
                            df.ik[a].protocol = 0x50;
                        }
                    }
                }
            }
        }
        
        crate::serial_println!("[xHCI] Device setup complete");
    }
    
    
    
    for (fw, izp, jhz, izs, jib) in oof {
        super::usb_storage::ttj(fw, izp, jhz, izs, jib);
    }
}






pub fn quo(fw: u8, bms: u8, rg: u64, go: u32) -> bool {
    
    {
        let mut um = CC_.lock();
        let bcr = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
            Some(m) => m,
            None => return false,
        };
        
        let mz = match bcr.gbu.as_mut() {
            Some(m) => m,
            None => return false,
        };
        
        let abj = Trb {
            bhr: rg,
            status: go,
            control: (AJE_ << 10) | EV_,
        };
        mz.azt(abj);
    }
    
    
    let mut db = Bn.lock();
    let df = match db.as_mut() {
        Some(r) => r,
        None => return false,
    };
    
    unsafe {
        let ng = (df.bub + (fw as u64) * 4) as *mut u32;
        ptr::write_volatile(ng, bms as u32);
    }
    
    if let Some((nn, _, _)) = jwh(df) {
        return nn == 1 || nn == 13; 
    }
    false
}


pub fn qun(fw: u8, bms: u8, rg: u64, go: u32) -> Option<u32> {
    
    {
        let mut um = CC_.lock();
        let bcr = match um.ds(fw as usize).and_then(|m| m.as_mut()) {
            Some(m) => m,
            None => return None,
        };
        
        let mz = match bcr.fea.as_mut() {
            Some(m) => m,
            None => return None,
        };
        
        let abj = Trb {
            bhr: rg,
            status: go,
            control: (AJE_ << 10) | EV_,
        };
        mz.azt(abj);
    }
    
    
    let mut db = Bn.lock();
    let df = match db.as_mut() {
        Some(r) => r,
        None => return None,
    };
    
    unsafe {
        let ng = (df.bub + (fw as u64) * 4) as *mut u32;
        ptr::write_volatile(ng, bms as u32);
    }
    
    if let Some((nn, vxu, _)) = jwh(df) {
        if nn == 1 || nn == 13 {
            
            return Some(go.ao(vxu));
        }
    }
    None
}


pub fn zfv() {
    
    let top: Vec<(u8, u16, u8)> = {
        let db = Bn.lock();
        match db.as_ref() {
            Some(r) => r.ik.iter()
                .hi(|bc| bc.fw != 0 && bc.class == 0x03)
                .map(|bc| (bc.fw, bc.gmh, bc.protocol))
                .collect(),
            None => return,
        }
    };
    
    for &(fw, gmh, protocol) in &top {
        let rg = match crate::memory::frame::azg() {
            Some(ai) => ai,
            None => continue,
        };
        let aak = auv(rg);
        let omc = gmh.am(8);
        
        
        {
            let db = Bn.lock();
            if let Some(df) = db.as_ref() {
                if !vpk(df, fw, rg, omc) {
                    crate::memory::frame::apt(rg);
                    continue;
                }
            } else {
                crate::memory::frame::apt(rg);
                continue;
            }
        }
        
        
        {
            let mut rrf = Bn.lock();
            if let Some(df) = rrf.as_mut() {
                for _ in 0..50_000u32 {
                    let w = df.bgy;
                    let abj = df.fib[w];
                    let ib = (abj.control & DA_) != 0;
                    if ib == df.cqg {
                        df.bgy += 1;
                        if df.bgy >= 256 {
                            df.bgy = 0;
                            df.cqg = !df.cqg;
                        }
                        let fhy = df.epz + (df.bgy as u64 * 16);
                        let edo = (df.ftj + 0x20) as *mut Zp;
                        unsafe { (*edo).fhy = fhy | (1 << 3); }
                        
                        let nn = ((abj.status >> 24) & 0xFF) as u8;
                        if nn == 1 || nn == 13 { 
                            let report = unsafe {
                                core::slice::anh(aak as *const u8, omc as usize)
                            };
                            
                            match protocol {
                                1 => vmp(report),
                                2 => vmq(report),
                                _ => {}
                            }
                        }
                        break;
                    }
                    core::hint::hc();
                }
            }
        }
        
        crate::memory::frame::apt(rg);
        return; 
    }
}






pub fn ky() -> bool {
    Be.load(Ordering::SeqCst)
}


pub fn cjx() -> usize {
    Bn.lock().as_ref().map(|r| r.ik.len()).unwrap_or(0)
}


pub fn bhh() -> Vec<Ve> {
    Bn.lock().as_ref()
        .map(|r| r.ik.clone())
        .age()
}


pub fn ytb(fw: u8) -> Option<Ve> {
    Bn.lock().as_ref()
        .and_then(|r| r.ik.iter().du(|bc| bc.fw == fw).abn())
}
