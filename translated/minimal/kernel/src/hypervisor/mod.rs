

















pub mod vmx;
pub mod vmcs;
pub mod ept;
pub mod vm;


pub mod svm;
pub mod svm_vm;


pub mod console;
pub mod virtfs;
pub mod guests;


pub mod api;
pub mod vpid;
pub mod isolation;
pub mod branding;


pub mod vmi;


pub mod linux_loader;
pub mod acpi;


pub mod mmio;
pub mod ioapic;


pub mod backend;
pub mod hpet;


pub mod pci;


pub mod virtio_blk;


pub mod tests;


pub mod debug_monitor;


pub mod virtio_console;
pub mod linux_subsystem;
pub mod linux_vm;

use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};


static TY_: AtomicBool = AtomicBool::new(false);
static AJR_: AtomicU64 = AtomicU64::new(0);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CpuVendor {
    F = 0,
    Ef = 1,
    Ct = 2,
}


static AAT_: AtomicU8 = AtomicU8::new(0);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtBackend {
    None,
    Ajf,
    Agh,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HypervisorError {
    
    Bwd,
    
    Btq,
    
    Tr,
    
    Dlu,
    
    Cpv,
    
    Cpq,
    
    Cpr,
    
    Bvz,
    
    Bwb,
    
    Cps,
    
    Cpt,
    
    Ns,
    
    Mo,
    
    Xd,
    
    Lj,
    
    Cid,
    
    Cno,
    
    Bd,
    
    Bxw,
    
    Acg,
    
    Cgc,
    
    Bjt,
}

pub type Result<T> = core::result::Result<T, HypervisorError>;


pub fn dpw() -> CpuVendor {
    let fyb: u32;
    let fyc: u32;
    let fyd: u32;
    
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {0:e}, ebx",
            "pop rbx",
            bd(reg) fyb,
            inout("eax") 0u32 => _,
            lateout("ecx") fyc,
            lateout("edx") fyd,
            options(nostack, preserves_flags)
        );
    }
    
    
    let txt = fyb == 0x756E_6547 
        && fyd == 0x4965_6E69 
        && fyc == 0x6C65_746E;
    
    
    let jba = fyb == 0x6874_7541 
        && fyd == 0x6974_6E65 
        && fyc == 0x444D_4163;
    
    if txt {
        AAT_.store(CpuVendor::Ef as u8, Ordering::SeqCst);
        CpuVendor::Ef
    } else if jba {
        AAT_.store(CpuVendor::Ct as u8, Ordering::SeqCst);
        CpuVendor::Ct
    } else {
        CpuVendor::F
    }
}


pub fn avo() -> CpuVendor {
    match AAT_.load(Ordering::SeqCst) {
        1 => CpuVendor::Ef,
        2 => CpuVendor::Ct,
        _ => CpuVendor::F,
    }
}


pub fn xrs() -> VirtBackend {
    if !zu() {
        return VirtBackend::None;
    }
    match avo() {
        CpuVendor::Ef => VirtBackend::Ajf,
        CpuVendor::Ct => VirtBackend::Agh,
        CpuVendor::F => VirtBackend::None,
    }
}


#[derive(Debug, Clone)]
pub struct Afp {
    pub dme: bool,
    pub fhw: bool,
    pub gvo: bool,
    pub gwj: bool,
    pub igr: u32,
}


pub fn init() -> Result<()> {
    crate::serial_println!("{}", branding::wth());
    crate::serial_println!("[HV] Initializing TrustVM hypervisor...");
    
    
    let acs = dpw();
    crate::serial_println!("[HV] CPU Vendor: {:?}", acs);
    
    match acs {
        CpuVendor::Ef => ttq(),
        CpuVendor::Ct => tsz(),
        CpuVendor::F => {
            crate::serial_println!("[HV] Unknown CPU vendor - virtualization not supported");
            Err(HypervisorError::Tr)
        }
    }
}


fn ttq() -> Result<()> {
    crate::serial_println!("[HV] Initializing Intel VT-x (VMX)...");
    
    
    let dr = vmx::inj()?;
    
    crate::serial_println!("[HV] VMX supported: {}", dr.dme);
    crate::serial_println!("[HV] EPT supported: {}", dr.fhw);
    crate::serial_println!("[HV] VPID supported: {}", dr.gwj);
    crate::serial_println!("[HV] Unrestricted guest: {}", dr.gvo);
    crate::serial_println!("[HV] VMCS revision: 0x{:08X}", dr.igr);
    
    if !dr.dme {
        return Err(HypervisorError::Bwd);
    }
    
    
    vmx::slg()?;
    
    
    vmx::xsr()?;
    
    
    let fyk = vpid::init();
    
    TY_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[HV] TrustVM hypervisor initialized successfully (Intel VT-x)!");
    crate::serial_println!("{}", branding::pzg(fyk, dr.fhw));
    
    
    api::eps(
        api::VmEventType::Cu,
        0, 
        api::VmEventData::Cj(alloc::string::String::from("TrustVM initialized (Intel VT-x)")),
    );
    
    Ok(())
}


fn tsz() -> Result<()> {
    crate::serial_println!("[HV] Initializing AMD-V (SVM)...");
    
    
    if !svm::gkj() {
        crate::serial_println!("[HV] SVM not supported or disabled in BIOS");
        return Err(HypervisorError::Btq);
    }
    
    
    let features = svm::fjn();
    
    crate::serial_println!("[HV] SVM Revision: {}", features.afe);
    crate::serial_println!("[HV] NPT supported: {}", features.npt);
    crate::serial_println!("[HV] NRIP Save: {}", features.evl);
    crate::serial_println!("[HV] Flush by ASID: {}", features.hjy);
    crate::serial_println!("[HV] Decode Assists: {}", features.iqs);
    crate::serial_println!("[HV] Available ASIDs: {}", features.fph);
    
    
    svm::init().jd(|_| HypervisorError::Cno)?;
    
    TY_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[HV] TrustVM hypervisor initialized successfully (AMD SVM)!");
    crate::serial_println!("{}", branding::pzg(true, features.npt));
    
    
    api::eps(
        api::VmEventType::Cu,
        0, 
        api::VmEventData::Cj(alloc::string::String::from("TrustVM initialized (AMD SVM)")),
    );
    
    Ok(())
}


pub fn zu() -> bool {
    TY_.load(Ordering::SeqCst)
}


pub fn dpg(j: &str, afc: usize) -> Result<u64> {
    if !zu() {
        return Err(HypervisorError::Tr);
    }
    
    crate::serial_println!("[HV] Creating VM '{}' (Memory: {}MB)", j, afc);
    
    
    match avo() {
        CpuVendor::Ct => {
            svm_vm::dpg(j, afc)
        }
        CpuVendor::Ef => {
            let fk = AJR_.fetch_add(1, Ordering::SeqCst);
            vm::dpg(fk, j, afc)?;
            Ok(fk)
        }
        CpuVendor::F => {
            Err(HypervisorError::Tr)
        }
    }
}


pub fn poi(fk: u64) -> Result<()> {
    match avo() {
        CpuVendor::Ct => {
            svm_vm::coa(fk, |vm| vm.ay()).unwrap_or(Err(HypervisorError::Mo))?;
            Ok(())
        }
        CpuVendor::Ef => vm::poi(fk),
        CpuVendor::F => Err(HypervisorError::Tr),
    }
}


pub fn gte(fk: u64, bzw: &str) -> Result<()> {
    match avo() {
        CpuVendor::Ct => {
            
            let txz = bzw == "linux-test" 
                || bzw.pp(".bzimage") 
                || bzw.pp(".bzImage");
            
            if txz {
                
                let cwc = if bzw == "linux-test" {
                    linux_loader::klw()
                } else {
                    guests::iwr(bzw)
                        .ok_or(HypervisorError::Mo)?
                        .ip()
                };
                
                svm_vm::coa(fk, |vm| {
                    vm.fvn(
                        &cwc,
                        "console=ttyS0 earlyprintk=serial nokaslr",
                        None,
                    )
                }).unwrap_or(Err(HypervisorError::Mo))?;
                Ok(())
            } else {
                
                let thx = guests::iwr(bzw)
                    .ok_or(HypervisorError::Mo)?;
                
                let typ = bzw == "pm-test" || bzw == "protected";
                
                svm_vm::coa(fk, |vm| {
                    
                    if vm.vmcb.is_none() {
                        vm.cfp()?;
                    }
                    
                    vm.diy(&thx, 0x1000)?;
                    if typ {
                        
                        vm.iab(0x1000, 0x8000)?;
                    } else {
                        
                        vm.jpk(0x1000)?;
                    }
                    vm.ay()
                }).unwrap_or(Err(HypervisorError::Mo))?;
                Ok(())
            }
        }
        CpuVendor::Ef => vm::gte(fk, bzw),
        CpuVendor::F => Err(HypervisorError::Tr),
    }
}


pub fn jru(fk: u64) -> Result<()> {
    match avo() {
        CpuVendor::Ct => {
            svm_vm::coa(fk, |vm| vm.rb()).unwrap_or(Err(HypervisorError::Mo))?;
            Ok(())
        }
        CpuVendor::Ef => vm::jru(fk),
        CpuVendor::F => Err(HypervisorError::Tr),
    }
}


pub fn elx(fk: u64, cac: &str, bqx: &str, awr: bool) {
    virtfs::elx(fk, cac, bqx, awr);
}


pub fn leo(fk: u64, f: &[u8]) {
    console::hoa(fk, f);
}


pub fn iwo(fk: u64) -> alloc::string::String {
    console::teh(fk)
}


pub fn hpy() -> &'static [&'static str] {
    guests::hpy()
}


pub fn dna() -> u64 {
    AJR_.load(Ordering::SeqCst)
}


pub fn cbu() -> Result<()> {
    if !zu() {
        return Ok(());
    }
    
    crate::serial_println!("[HV] Shutting down TrustVM hypervisor...");
    
    
    
    
    
    vmx::xsq()?;
    
    TY_.store(false, Ordering::SeqCst);
    
    crate::serial_println!("[HV] Hypervisor shutdown complete");
    
    Ok(())
}






pub fn iwn() -> api::Capabilities {
    api::iwn()
}


pub fn dk() -> &'static str {
    api::DAL_
}


pub fn nya(az: usize) -> alloc::vec::Vec<api::Uw> {
    api::ten(az)
}


pub fn ipp(fk: u64, j: &str) -> u64 {
    api::ipp(fk, j, 4096)
}


pub fn nci(cjo: u64, f: &[u8]) -> core::result::Result<usize, &'static str> {
    api::nci(cjo, f)
}


pub fn znu(fk: u64, lla: usize, llh: usize) {
    let lws = api::ResourceQuota {
        lla,
        llh,
        ..api::ResourceQuota::default()
    };
    api::wjm(fk, lws);
}






pub fn fyk() -> bool {
    vpid::zu()
}


pub fn pyu() -> usize {
    vpid::qgz()
}


pub fn fhx() -> u64 {
    isolation::pyh()
}


pub fn pap(az: usize) -> alloc::vec::Vec<isolation::Lj> {
    isolation::vte(az)
}


pub fn zqb() -> bool {
    isolation::ppw()
}






pub fn logo() -> &'static str {
    branding::AZC_
}


pub fn jma() -> alloc::string::String {
    let dr = iwn();
    branding::jma(dr.cvr())
}


pub fn jmb() -> alloc::string::String {
    branding::jmb(
        vpid::zu(),
        true, 
        isolation::ppw(),
        isolation::pyh(),
    )
}






pub fn kbt() -> alloc::string::String {
    match xrs() {
        VirtBackend::Ajf => {
            alloc::format!("Intel VT-x (VMX) - VMCS, EPT, VPID")
        }
        VirtBackend::Agh => {
            let features = svm::fjn();
            alloc::format!(
                "AMD-V (SVM) Rev{} - VMCB, NPT:{}, ASIDs:{}",
                features.afe,
                if features.npt { "Yes" } else { "No" },
                features.fph
            )
        }
        VirtBackend::None => {
            alloc::format!("No virtualization backend active")
        }
    }
}


pub fn yyv() -> bool {
    avo() == CpuVendor::Ct && zu()
}


pub fn yzl() -> bool {
    avo() == CpuVendor::Ef && zu()
}


pub fn zqf() -> Option<svm::SvmFeatures> {
    if avo() == CpuVendor::Ct {
        Some(svm::fjn())
    } else {
        None
    }
}

