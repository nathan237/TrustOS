








pub mod features;
pub mod tsc;
pub mod simd;
pub mod smp;

use core::sync::atomic::{AtomicBool, Ordering};


static APA_: AtomicBool = AtomicBool::new(false);
static mut AOY_: Option<CpuCapabilities> = None;


#[derive(Debug, Clone)]
pub struct CpuCapabilities {
    
    pub acs: CpuVendor,
    pub family: u8,
    pub model: u8,
    pub bxi: u8,
    pub dem: [u8; 48],
    
    
    pub tsc: bool,           
    pub fan: bool, 
    pub ifc: bool,  
    pub fsd: bool,        
    
    
    pub eiw: bool,
    pub eix: bool,
    pub fvj: bool,
    pub fvl: bool,
    pub fvk: bool,
    pub eyy: bool,
    pub dof: bool,
    pub dog: bool,
    pub eml: bool,
    pub hka: bool,
    
    
    pub doa: bool,         
    pub ewm: bool,     
    pub eyl: bool,       
    pub cbg: bool,        
    pub cmc: bool,        
    
    
    pub cia: bool,
    pub cul: bool,
    pub ddd: bool,
    pub vt: bool,
    
    
    pub vmx: bool,           
    pub svm: bool,           
    
    
    pub cau: u8,
    pub djk: u8,
    pub aed: u8,
    
    
    pub ekf: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CpuVendor {
    Ef,
    Ct,
    F,
}

impl CpuCapabilities {
    
    pub fn dgf() -> Self {
        let mut dr = Self {
            acs: CpuVendor::F,
            family: 0,
            model: 0,
            bxi: 0,
            dem: [0; 48],
            tsc: false,
            fan: false,
            ifc: false,
            fsd: false,
            eiw: false,
            eix: false,
            fvj: false,
            fvl: false,
            fvk: false,
            eyy: false,
            dof: false,
            dog: false,
            eml: false,
            hka: false,
            doa: false,
            ewm: false,
            eyl: false,
            cbg: false,
            cmc: false,
            cia: false,
            cul: false,
            ddd: false,
            vt: false,
            vmx: false,
            svm: false,
            cau: 1,
            djk: 1,
            aed: 0,
            ekf: 0,
        };
        
        
        let gdp = unsafe { core::arch::x86_64::ddo(0) };
        let olm = gdp.eax;
        
        
        let moz = [
            gdp.ebx.ho(),
            gdp.edx.ho(),
            gdp.ecx.ho(),
        ];
        
        let mut gvz = [0u8; 12];
        gvz[0..4].dg(&moz[0]);
        gvz[4..8].dg(&moz[1]);
        gvz[8..12].dg(&moz[2]);
        
        dr.acs = match &gvz {
            b"GenuineIntel" => CpuVendor::Ef,
            b"AuthenticAMD" => CpuVendor::Ct,
            _ => CpuVendor::F,
        };
        
        
        if olm >= 1 {
            let bgn = unsafe { core::arch::x86_64::ddo(1) };
            
            
            dr.bxi = (bgn.eax & 0xF) as u8;
            dr.model = ((bgn.eax >> 4) & 0xF) as u8;
            dr.family = ((bgn.eax >> 8) & 0xF) as u8;
            
            
            if dr.family == 0xF {
                dr.family += ((bgn.eax >> 20) & 0xFF) as u8;
            }
            if dr.family >= 6 {
                dr.model += (((bgn.eax >> 16) & 0xF) << 4) as u8;
            }
            
            
            dr.aed = ((bgn.ebx >> 24) & 0xFF) as u8;
            dr.cau = ((bgn.ebx >> 16) & 0xFF) as u8;
            
            
            dr.fvj = (bgn.ecx & (1 << 0)) != 0;
            dr.ewm = (bgn.ecx & (1 << 1)) != 0;
            dr.fvl = (bgn.ecx & (1 << 9)) != 0;
            dr.fvk = (bgn.ecx & (1 << 19)) != 0;
            dr.eyy = (bgn.ecx & (1 << 20)) != 0;
            dr.doa = (bgn.ecx & (1 << 25)) != 0;
            dr.dof = (bgn.ecx & (1 << 28)) != 0;
            dr.hka = (bgn.ecx & (1 << 12)) != 0;
            dr.cbg = (bgn.ecx & (1 << 30)) != 0;
            dr.vmx = (bgn.ecx & (1 << 5)) != 0;
            dr.ifc = (bgn.ecx & (1 << 24)) != 0;
            
            
            dr.tsc = (bgn.edx & (1 << 4)) != 0;
            dr.eiw = (bgn.edx & (1 << 25)) != 0;
            dr.eix = (bgn.edx & (1 << 26)) != 0;
        }
        
        
        if olm >= 7 {
            let dfh = unsafe { core::arch::x86_64::qbf(7, 0) };
            
            dr.cia = (dfh.ebx & (1 << 7)) != 0;
            dr.dog = (dfh.ebx & (1 << 5)) != 0;
            dr.eml = (dfh.ebx & (1 << 16)) != 0;
            dr.eyl = (dfh.ebx & (1 << 29)) != 0;
            dr.cmc = (dfh.ebx & (1 << 18)) != 0;
            dr.cul = (dfh.ebx & (1 << 20)) != 0;
            dr.ddd = (dfh.ecx & (1 << 2)) != 0;
        }
        
        
        let rqc = unsafe { core::arch::x86_64::ddo(0x80000000) };
        let lkw = rqc.eax;
        
        
        if lkw >= 0x80000001 {
            let klr = unsafe { core::arch::x86_64::ddo(0x80000001) };
            
            dr.vt = (klr.edx & (1 << 20)) != 0;
            dr.fsd = (klr.edx & (1 << 27)) != 0;
            dr.svm = (klr.ecx & (1 << 2)) != 0;
        }
        
        
        if lkw >= 0x80000007 {
            let rqb = unsafe { core::arch::x86_64::ddo(0x80000007) };
            dr.fan = (rqb.edx & (1 << 8)) != 0;
        }
        
        
        if lkw >= 0x80000004 {
            for a in 0..3 {
                let ipm = unsafe { core::arch::x86_64::ddo(0x80000002 + a) };
                let l = (a as usize) * 16;
                dr.dem[l..l+4].dg(&ipm.eax.ho());
                dr.dem[l+4..l+8].dg(&ipm.ebx.ho());
                dr.dem[l+8..l+12].dg(&ipm.ecx.ho());
                dr.dem[l+12..l+16].dg(&ipm.edx.ho());
            }
        }
        
        dr
    }
    
    
    pub fn keu(&self) -> &str {
        let ci = self.dem.iter()
            .qf(|&o| o == 0)
            .unwrap_or(48);
        core::str::jg(&self.dem[..ci])
            .unwrap_or("Unknown CPU")
            .em()
    }
}


pub fn init() {
    if APA_.swap(true, Ordering::SeqCst) {
        return; 
    }
    
    
    let mut dr = CpuCapabilities::dgf();
    
    
    dr.ekf = tsc::nbj();
    
    
    unsafe {
        AOY_ = Some(dr.clone());
    }
    
    
    crate::serial_println!("[CPU] {}", dr.keu());
    crate::serial_println!("[CPU] Vendor: {:?}, Family: {}, Model: {}", 
        dr.acs, dr.family, dr.model);
    crate::serial_println!("[CPU] TSC: {} (invariant: {}, freq: {} MHz)", 
        dr.tsc, dr.fan, dr.ekf / 1_000_000);
    crate::serial_println!("[CPU] SIMD: SSE={} SSE2={} SSE4.2={} AVX={} AVX2={} FMA={}", 
        dr.eiw, dr.eix, dr.eyy, dr.dof, dr.dog, dr.hka);
    crate::serial_println!("[CPU] Crypto: AES-NI={} PCLMULQDQ={} SHA={} RDRAND={}", 
        dr.doa, dr.ewm, dr.eyl, dr.cbg);
    crate::serial_println!("[CPU] Security: SMEP={} SMAP={} NX={}", 
        dr.cia, dr.cul, dr.vt);
    crate::serial_println!("[CPU] Virt: VMX={} SVM={}", dr.vmx, dr.svm);
    
    
    if dr.eiw {
        simd::ktg();
    }
    
    
    if dr.dof {
        simd::ktd();
    }
    
    
    tsc::init(dr.ekf);
}


pub fn bme() -> Option<&'static CpuCapabilities> {
    if !APA_.load(Ordering::Relaxed) {
        return None;
    }
    unsafe { AOY_.as_ref() }
}


pub fn mnh() -> u64 {
    bme().map(|r| r.ekf).unwrap_or(3_000_000_000)
}


pub fn cfe() -> bool {
    bme().map(|r| r.doa).unwrap_or(false)
}


pub fn crd() -> bool {
    bme().map(|r| r.cbg).unwrap_or(false)
}


pub fn gdj() -> u8 {
    bme().map(|r| r.cau).unwrap_or(1)
}


pub fn cbg() -> Option<u64> {
    if !crd() {
        return None;
    }
    
    let mut bn: u64;
    let vx: u8;
    
    unsafe {
        core::arch::asm!(
            "rdrand {0}",
            "setc {1}",
            bd(reg) bn,
            bd(reg_byte) vx,
            options(nostack)
        );
    }
    
    if vx != 0 {
        Some(bn)
    } else {
        None
    }
}


pub fn cmc() -> Option<u64> {
    let dr = bme()?;
    if !dr.cmc {
        return None;
    }
    
    let mut bn: u64;
    let vx: u8;
    
    unsafe {
        core::arch::asm!(
            "rdseed {0}",
            "setc {1}",
            bd(reg) bn,
            bd(reg_byte) vx,
            options(nostack)
        );
    }
    
    if vx != 0 {
        Some(bn)
    } else {
        None
    }
}
