



















use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;






#[derive(Clone)]
pub struct T {
    
    pub avo: String,
    pub dpf: String,
    pub azj: u32,
    pub heb: u8,
    pub hec: u8,
    pub hee: u8,
    pub fam: u64,
    pub cau: u8,
    pub djk: u8,
    pub aed: u8,
    
    pub ixu: bool,
    pub dro: bool,
    pub ixv: bool,
    pub lbk: bool,
    pub lbj: bool,
    pub hmq: bool,
    pub fke: bool,
    pub bzx: bool,
    pub drm: bool,
    
    pub cfe: bool,
    pub git: bool,
    pub ecm: bool,
    pub crd: bool,
    pub fkh: bool,
    
    pub erv: bool,
    pub eru: bool,
    pub giv: bool,
    pub ert: bool,
    
    pub lbl: bool,
    pub ixx: bool,
    pub lbm: bool,
    pub ixt: bool,
    
    pub giw: bool,
    pub giu: bool,

    
    pub ccf: u64,
    pub drr: usize,
    pub ecw: usize,
    pub erx: usize,
    pub ceu: usize,
    pub dhj: usize,
    pub lr: u64,

    
    pub gxt: u8,
    pub gxs: String,
    pub kvd: u16,
    pub hiu: bool,
    pub itu: bool,
    pub kvb: bool,
    pub kvc: u32,

    
    pub cap: u64,
    pub gae: Vec<Bbt>,
    pub edq: usize,
    pub ofi: Vec<Bjy>,
    pub hoq: Vec<Bke>,
    pub kar: usize,

    
    pub gpa: Vec<Bot>,
    pub fqo: bool,

    
    pub hus: Vec<Bos>,
    pub dal: usize,
    
    pub ewl: usize,
    pub egg: usize,
    pub egh: usize,
    pub egf: usize,
    pub ewk: usize,
    pub hur: usize,
    pub fqn: usize,

    
    pub aqm: Vec<Azg>,
    pub dmp: u64,
    pub aqd: Vec<Boq>,
    pub avs: Vec<Ahs>,

    
    pub bzz: bool,
    pub csg: Option<[u8; 6]>,
    pub aik: bool,

    
    pub bqz: bool,
    pub beh: String,
    pub dhr: u32,
    pub erk: u32,

    
    pub esa: bool,
    pub juh: bool,
    pub hnd: u64,
    pub gjf: u8,
    pub iys: bool,
    pub iyt: u16,

    
    pub fav: bool,
    pub fxv: usize,
    pub cvc: Vec<Bvm>,

    
    pub fki: bool,

    
    pub arch: &'static str,
    pub jjz: &'static str,

    
    pub cwl: f32,
    pub dte: f32,
    pub ezb: f32,
    pub evg: f32,
    pub eyh: f32,
    pub dkj: f32,
}

#[derive(Clone)]
pub struct Bos {
    pub aq: u8,
    pub de: u8,
    pub gw: u8,
    pub ml: u16,
    pub mx: u16,
    pub ajz: u8,
    pub adl: u8,
    pub bpz: String,
    pub bor: String,
}

#[derive(Clone)]
pub struct Azg {
    pub j: String,
    pub kk: StorageKind,
    pub fei: u64,
    pub model: String,
    pub serial: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StorageKind {
    Qr,
    Xv,
    Bjk,
    F,
}

impl StorageKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageKind::Qr => "SATA",
            StorageKind::Xv => "NVMe",
            StorageKind::Bjk => "IDE",
            StorageKind::F => "???",
        }
    }
}



#[derive(Clone)]
pub struct Bbt {
    pub aed: u32,
    pub bny: u32,
    pub iq: bool,
    pub htp: bool,
}

#[derive(Clone)]
pub struct Bjy {
    pub ad: u8,
    pub re: u64,
    pub ech: u32,
}

#[derive(Clone)]
pub struct Bke {
    pub wqk: u8,
    pub tfz: u32,
    pub dkr: u8,
    pub dmt: u8,
}

#[derive(Clone)]
pub struct Bot {
    pub bps: u64,
    pub ie: u16,
    pub cca: u8,
    pub cej: u8,
}

#[derive(Clone)]
pub struct Boq {
    pub app: String,
    pub aqb: u8,
    pub aag: u64,
    pub afz: u64,
    pub ddc: String,
    pub cji: bool,
    pub j: String,
}

#[derive(Clone)]
pub struct Ahs {
    pub app: String,
    pub partition: Option<u8>,
    pub ckf: EncryptionType,
    pub eu: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EncryptionType {
    Ajy,
    Ajz,
    Aaa,
    Afn,
    Bgz,
    Bes,
    Bnu,
    F,
}

impl EncryptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EncryptionType::Ajy => "LUKS1",
            EncryptionType::Ajz => "LUKS2",
            EncryptionType::Aaa => "BitLocker",
            EncryptionType::Afn => "VeraCrypt",
            EncryptionType::Bgz => "FileVault2",
            EncryptionType::Bes => "dm-crypt",
            EncryptionType::Bnu => "OPAL SED",
            EncryptionType::F => "Unknown encryption",
        }
    }
}

#[derive(Clone)]
pub struct Bvm {
    pub re: u8,
    pub bpz: String,
    pub ml: u16,
    pub cgt: u16,
    pub baj: String,
}


static BDO_: AtomicBool = AtomicBool::new(false);
static Boo: Mutex<Option<T>> = Mutex::new(None);


pub fn wdu() -> T {
    crate::serial_println!("[JARVIS-HW] Starting exhaustive hardware scan...");

    let mut cc = T {
        
        avo: String::new(),
        dpf: String::new(),
        azj: 1,
        heb: 0,
        hec: 0,
        hee: 0,
        fam: 0,
        cau: 0,
        djk: 0,
        aed: 0,
        ixu: false,
        dro: false,
        ixv: false,
        lbk: false,
        lbj: false,
        hmq: false,
        fke: false,
        bzx: false,
        drm: false,
        cfe: false,
        git: false,
        ecm: false,
        crd: false,
        fkh: false,
        erv: false,
        eru: false,
        giv: false,
        ert: false,
        lbl: false,
        ixx: false,
        lbm: false,
        ixt: false,
        giw: false,
        giu: false,
        
        ccf: 0,
        drr: 0,
        ecw: 0,
        erx: 0,
        ceu: 0,
        dhj: 0,
        lr: 0,
        
        gxt: 0,
        gxs: String::new(),
        kvd: 0,
        hiu: false,
        itu: false,
        kvb: false,
        kvc: 0,
        
        cap: 0,
        gae: Vec::new(),
        edq: 0,
        ofi: Vec::new(),
        hoq: Vec::new(),
        kar: 0,
        
        gpa: Vec::new(),
        fqo: false,
        
        hus: Vec::new(),
        dal: 0,
        ewl: 0,
        egg: 0,
        egh: 0,
        egf: 0,
        ewk: 0,
        hur: 0,
        fqn: 0,
        
        aqm: Vec::new(),
        dmp: 0,
        aqd: Vec::new(),
        avs: Vec::new(),
        
        bzz: false,
        csg: None,
        aik: false,
        
        bqz: false,
        beh: String::new(),
        dhr: 0,
        erk: 0,
        
        esa: false,
        juh: false,
        hnd: 0,
        gjf: 0,
        iys: false,
        iyt: 0,
        
        fav: false,
        fxv: 0,
        cvc: Vec::new(),
        
        fki: false,
        
        arch: if cfg!(target_arch = "x86_64") { "x86_64" }
              else if cfg!(target_arch = "aarch64") { "aarch64" }
              else if cfg!(target_arch = "riscv64") { "riscv64" }
              else { "unknown" },
        jjz: "ring0",
        
        cwl: 0.0,
        dte: 0.0,
        ezb: 0.0,
        evg: 0.0,
        eyh: 0.0,
        dkj: 0.0,
    };

    
    vls(&mut cc);
    vlx(&mut cc);

    
    vlp(&mut cc);

    
    gpv(&mut cc);

    
    lvr(&mut cc);
    vmb(&mut cc);
    lvq(&mut cc);
    lvp(&mut cc);
    vme(&mut cc);
    lvs(&mut cc);
    lvn(&mut cc);

    
    rnk(&mut cc);

    crate::serial_println!("[JARVIS-HW] Exhaustive scan complete:");
    crate::serial_println!("  score={:.0}%, {} PCI devs, {} storage, {} partitions, {} encrypted",
        cc.dkj * 100.0, cc.dal,
        cc.aqm.len(), cc.aqd.len(),
        cc.avs.len());
    crate::serial_println!("  {}MB RAM, {} APIC CPUs, {} IOAPICs, {} USB devs",
        cc.ccf / (1024 * 1024), cc.gae.len(),
        cc.edq, cc.cvc.len());

    
    *Boo.lock() = Some(cc.clone());
    BDO_.store(true, Ordering::Release);

    cc
}


pub fn gby() -> Option<T> {
    Boo.lock().clone()
}


pub fn tyv() -> bool {
    BDO_.load(Ordering::Acquire)
}





fn vls(cc: &mut T) {
    #[cfg(target_arch = "x86_64")]
    {
        let dr = crate::cpu::CpuCapabilities::dgf();

        cc.avo = match dr.acs {
            crate::cpu::CpuVendor::Ef => String::from("Intel"),
            crate::cpu::CpuVendor::Ct => String::from("AMD"),
            crate::cpu::CpuVendor::F => String::from("Unknown"),
        };

        
        let qrt = dr.dem.iter()
            .qf(|&o| o == 0)
            .unwrap_or(48);
        if let Ok(e) = core::str::jg(&dr.dem[..qrt]) {
            cc.dpf = String::from(e.em());
        }

        cc.heb = dr.family;
        cc.hec = dr.model;
        cc.hee = dr.bxi;
        cc.azj = crate::cpu::smp::aao();
        cc.fam = dr.ekf;
        cc.cau = dr.cau;
        cc.djk = dr.djk;
        cc.aed = dr.aed;

        
        cc.ixu = dr.eiw;
        cc.dro = dr.eix;
        cc.ixv = dr.fvj;
        cc.lbk = dr.fvl;
        cc.lbj = dr.fvk;
        cc.hmq = dr.eyy;
        cc.fke = dr.dof;
        cc.bzx = dr.dog;
        cc.drm = dr.eml;

        
        cc.cfe = dr.doa;
        cc.git = dr.ewm;
        cc.ecm = dr.eyl;
        cc.crd = dr.cbg;
        cc.fkh = dr.cmc;

        
        cc.erv = dr.cia;
        cc.eru = dr.cul;
        cc.giv = dr.ddd;
        cc.ert = dr.vt;

        
        cc.lbl = dr.tsc;
        cc.ixx = dr.fan;
        cc.lbm = dr.ifc;
        cc.ixt = dr.fsd;

        
        cc.giw = dr.vmx;
        cc.giu = dr.svm;

        crate::serial_println!("[JARVIS-HW] CPU: {} F{}M{}S{} {}C SSE2={} AVX2={} AES={} SMEP={} NX={}",
            cc.dpf, dr.family, dr.model, dr.bxi,
            cc.azj, dr.eix, dr.dog, dr.doa, dr.cia, dr.vt);
    }

    #[cfg(target_arch = "aarch64")]
    {
        cc.avo = String::from("ARM");
        cc.dpf = String::from("AArch64 Processor");
        cc.azj = 1;
        cc.jjz = "EL1";
    }
}

fn vlx(cc: &mut T) {
    cc.ccf = crate::memory::fxc();
    cc.drr = crate::memory::cre();
    cc.ecw = crate::memory::heap::mr();
    cc.erx = crate::memory::heap::aez();
    cc.lr = crate::memory::lr();

    let cm = crate::memory::cm();
    cc.ceu = cm.ceu;
    cc.dhj = cm.dhj;

    crate::serial_println!("[JARVIS-HW] RAM: {} MB total, heap {} KB / {} KB, frames {}/{}",
        cc.ccf / (1024 * 1024),
        cc.ecw / 1024, cc.erx / 1024,
        cc.ceu, cc.dhj);
}

fn vlp(cc: &mut T) {
    if let Some(acpi) = crate::acpi::ani() {
        cc.gxt = acpi.afe;
        cc.gxs = acpi.clo.clone();
        cc.cap = acpi.cap;

        
        if let Some(ref fadt) = acpi.fadt {
            cc.kvd = fadt.grm;
            cc.hiu = fadt.ogb();
            cc.itu = fadt.ppx();
            cc.kvb = (fadt.flags & crate::acpi::fadt::FadtInfo::BVF_) != 0;
            cc.kvc = fadt.jjn;
        }

        
        for ku in &acpi.dja {
            cc.gae.push(Bbt {
                aed: ku.aed,
                bny: ku.bny,
                iq: ku.iq,
                htp: ku.htp,
            });
        }

        
        cc.edq = acpi.cyx.len();
        for ioapic in &acpi.cyx {
            cc.ofi.push(Bjy {
                ad: ioapic.ad,
                re: ioapic.re,
                ech: ioapic.ech,
            });
        }

        
        for bvu in &acpi.gka {
            cc.hoq.push(Bke {
                wqk: bvu.iy,
                tfz: bvu.bup,
                dkr: bvu.dkr,
                dmt: bvu.dmt,
            });
        }

        cc.kar = acpi.fne.len();

        
        if let Some(ref hpet) = acpi.hpet {
            cc.esa = true;
            cc.hnd = hpet.fjc();
            cc.gjf = hpet.lph;
            cc.iys = hpet.eoc;
            cc.iyt = hpet.ml;
        }

        
        cc.fqo = !acpi.eut.is_empty();
        for pk in &acpi.eut {
            cc.gpa.push(Bot {
                bps: pk.bps,
                ie: pk.ie,
                cca: pk.cca,
                cej: pk.cej,
            });
        }

        crate::serial_println!("[JARVIS-HW] ACPI: rev={} OEM='{}' {} CPUs {} IOAPICs {} overrides PCIe={}",
            acpi.afe, acpi.clo, acpi.dja.len(),
            cc.edq, cc.hoq.len(), cc.fqo);
    } else {
        crate::serial_println!("[JARVIS-HW] ACPI: not available");
    }
}

fn gpv(cc: &mut T) {
    let ik = crate::pci::fjm();
    cc.dal = ik.len();

    for ba in &ik {
        cc.hus.push(Bos {
            aq: ba.aq,
            de: ba.de,
            gw: ba.gw,
            ml: ba.ml,
            mx: ba.mx,
            ajz: ba.ajz,
            adl: ba.adl,
            bpz: String::from(ba.bpz()),
            bor: String::from(ba.bor()),
        });

        
        match ba.ajz {
            0x01 => cc.ewl += 1,
            0x02 => cc.egg += 1,
            0x03 => cc.ewk += 1,
            0x04 => cc.egf += 1,
            0x06 => cc.hur += 1,
            0x0C => {
                if ba.adl == 0x03 { 
                    cc.egh += 1;
                }
            }
            0x10 => cc.fqn += 1, 
            _ => {}
        }
    }

    crate::serial_println!("[JARVIS-HW] PCI: {} devices ({}stor {}net {}usb {}audio {}disp {}bridge)",
        cc.dal, cc.ewl,
        cc.egg, cc.egh,
        cc.egf, cc.ewk,
        cc.hur);
}

fn lvr(cc: &mut T) {
    
    if crate::drivers::ahci::ky() {
        for port in crate::drivers::ahci::bhh() {
            let mh = port.agw * 512;
            cc.dmp += mh;
            cc.aqm.push(Azg {
                j: format!("SATA port {}", port.kg),
                kk: StorageKind::Qr,
                fei: mh,
                model: port.model.clone(),
                serial: port.serial.clone(),
            });
        }
    }

    
    if crate::nvme::ky() {
        for csw in crate::nvme::ufs() {
            let mh = csw.mfy * csw.bni as u64;
            cc.dmp += mh;
            cc.aqm.push(Azg {
                j: format!("NVMe ns{}", csw.bvp),
                kk: StorageKind::Xv,
                fei: mh,
                model: String::from("NVMe"),
                serial: String::new(),
            });
        }
    }

    crate::serial_println!("[JARVIS-HW] Storage: {} devices, {} GB total",
        cc.aqm.len(),
        cc.dmp / (1024 * 1024 * 1024));
}


fn vmb(cc: &mut T) {
    
    if crate::drivers::ahci::ky() {
        let vjx = crate::drivers::ahci::nyj();
        for kg in 0..vjx {
            if crate::drivers::ahci::kyv(kg).is_some() {
                
                if let Ok(se) = crate::drivers::partition::lxn(kg) {
                    let app = format!("SATA:{}", kg);
                    for vu in &se.aqd {
                        cc.aqd.push(Boq {
                            app: app.clone(),
                            aqb: vu.aqb,
                            aag: vu.aag,
                            afz: vu.afz(),
                            ddc: format!("{:?}", vu.duf),
                            cji: vu.cji,
                            j: vu.j.clone(),
                        });
                    }
                }

                
                let mut k = [0u8; 512];
                if crate::drivers::ahci::ain(kg, 0, 1, &mut k).is_ok() {
                    nkz(&k, &format!("SATA:{}", kg), None, cc);
                }
                
                if crate::drivers::ahci::ain(kg, 6, 1, &mut k).is_ok() {
                    nkz(&k, &format!("SATA:{}", kg), None, cc);
                }
            }
        }
    }

    crate::serial_println!("[JARVIS-HW] Partitions: {} found, {} encrypted volumes detected",
        cc.aqd.len(), cc.avs.len());
}


fn nkz(k: &[u8], app: &str, lsy: Option<u8>, cc: &mut T) {
    if k.len() < 512 { return; }

    
    if k.len() >= 6 && k[0] == b'L' && k[1] == b'U' && k[2] == b'K'
        && k[3] == b'S' && k[4] == 0xBA && k[5] == 0xBE
    {
        let dk = if k.len() >= 8 {
            ((k[6] as u16) << 8) | k[7] as u16
        } else { 0 };

        let iss = if dk == 2 { EncryptionType::Ajz } else { EncryptionType::Ajy };
        let eu = format!("LUKS v{} detected at sector 0", dk);

        
        if !cc.avs.iter().any(|aa| aa.app == app && aa.ckf == iss) {
            cc.avs.push(Ahs {
                app: String::from(app),
                partition: lsy,
                ckf: iss,
                eu,
            });
        }
    }

    
    if k.len() >= 11
        && k[3] == b'-' && k[4] == b'F' && k[5] == b'V' && k[6] == b'E'
        && k[7] == b'-' && k[8] == b'F' && k[9] == b'S' && k[10] == b'-'
    {
        if !cc.avs.iter().any(|aa| aa.app == app && aa.ckf == EncryptionType::Aaa) {
            cc.avs.push(Ahs {
                app: String::from(app),
                partition: lsy,
                ckf: EncryptionType::Aaa,
                eu: String::from("BitLocker BDE signature (-FVE-FS-) detected"),
            });
        }
    }

    
    
    if k.len() >= 4 && k[0] == b'V' && k[1] == b'E' && k[2] == b'R' && k[3] == b'A' {
        if !cc.avs.iter().any(|aa| aa.app == app && aa.ckf == EncryptionType::Afn) {
            cc.avs.push(Ahs {
                app: String::from(app),
                partition: lsy,
                ckf: EncryptionType::Afn,
                eu: String::from("VeraCrypt volume header signature detected"),
            });
        }
    }

    
    
}

fn lvq(cc: &mut T) {
    cc.bzz = crate::drivers::net::bzy();
    if cc.bzz {
        cc.csg = crate::drivers::net::cez();
        cc.aik = crate::drivers::net::aik();
    }

    crate::serial_println!("[JARVIS-HW] Network: detected={} link_up={}",
        cc.bzz, cc.aik);
}

fn lvp(cc: &mut T) {
    cc.bqz = crate::drivers::amdgpu::clb();
    if cc.bqz {
        if let Some(co) = crate::drivers::amdgpu::ani() {
            cc.beh = String::from(co.beh());
            cc.dhr = (co.igx / (1024 * 1024)) as u32;
            cc.erk = co.cwm;
        }
    }

    crate::serial_println!("[JARVIS-HW] GPU: detected={} name='{}'",
        cc.bqz, cc.beh);
}

fn vme(cc: &mut T) {
    #[cfg(target_arch = "x86_64")]
    {
        cc.juh = cc.lbl;
        
        if cc.hnd > 0 {
            cc.esa = true;
        }
    }
}

fn lvs(cc: &mut T) {
    cc.fav = crate::drivers::usb::ky();
    if cc.fav {
        cc.fxv = crate::drivers::usb::roo();
        let ik = crate::drivers::usb::smj();
        for ba in &ik {
            cc.cvc.push(Bvm {
                re: ba.re,
                bpz: format!("{:?}", ba.class),
                ml: ba.ml,
                cgt: ba.cgt,
                baj: ba.baj.clone(),
            });
        }
    }

    crate::serial_println!("[JARVIS-HW] USB: init={} controllers={} devices={}",
        cc.fav, cc.fxv, cc.cvc.len());
}

fn lvn(cc: &mut T) {
    cc.fki = crate::drivers::hda::ky();
    crate::serial_println!("[JARVIS-HW] Audio HDA: init={}", cc.fki);
}





fn rnk(cc: &mut T) {
    
    let wol = if cc.drm { 4.0 }
        else if cc.bzx { 2.0 }
        else if cc.fke { 1.5 }
        else if cc.dro { 1.0 }
        else { 0.5 };

    let rpb = (cc.azj as f32).v(32.0) / 32.0;
    let sxi = (cc.fam as f32 / 5_000_000_000.0).v(1.0);
    let tgy = if cc.bqz { 0.3 } else { 0.0 };

    cc.cwl = ((rpb * 0.4 + sxi * 0.3 + wol / 4.0 * 0.3) + tgy).v(1.0);

    
    let vpw = cc.ccf as f32 / (1024.0 * 1024.0 * 1024.0);
    cc.dte = (vpw / 64.0).v(1.0);

    
    let wur = cc.dmp as f32 / (1024.0 * 1024.0 * 1024.0);
    let lbe = cc.aqm.iter().any(|e| e.kk == StorageKind::Xv);
    let wqz = if lbe { 1.0 } else { 0.5 };
    cc.ezb = ((wur / 2048.0) * wqz).v(1.0);

    
    cc.evg = if cc.bzz && cc.aik { 1.0 }
        else if cc.bzz { 0.5 }
        else { 0.0 };

    
    let mut zw = 0.0f32;
    if cc.cfe { zw += 0.12; }
    if cc.crd { zw += 0.08; }
    if cc.fkh { zw += 0.05; }
    if cc.ecm { zw += 0.05; }
    if cc.git { zw += 0.05; }
    if cc.erv { zw += 0.10; }
    if cc.eru { zw += 0.10; }
    if cc.giv { zw += 0.05; }
    if cc.ert { zw += 0.10; }
    
    zw += 0.15;
    
    if cc.edq > 0 { zw += 0.05; }
    
    if cc.fqo { zw += 0.05; }
    
    if cc.fqn > 0 { zw += 0.05; }
    cc.eyh = zw.v(1.0);

    
    cc.dkj = cc.cwl * 0.30
        + cc.dte * 0.20
        + cc.ezb * 0.15
        + cc.evg * 0.10
        + cc.eyh * 0.25;
}





impl T {
    
    pub fn fix(&self) -> String {
        let mut e = String::new();

        e.t("\x01C╔══════════════════════════════════════════════════════════╗\n");
        e.t("║       JARVIS Exhaustive Hardware Intelligence Report      ║\n");
        e.t("╚══════════════════════════════════════════════════════════╝\x01W\n\n");

        
        e.t(&format!("\x01Y[CPU]\x01W {}\n", self.dpf));
        e.t(&format!("  Vendor: {}  Family: {}  Model: {}  Stepping: {}\n",
            self.avo, self.heb, self.hec, self.hee));
        e.t(&format!("  Cores: {} (logical={} physical={})  TSC: {} MHz\n",
            self.azj, self.cau, self.djk,
            self.fam / 1_000_000));
        e.t(&format!("  APIC ID: {}  TSC: inv={} deadline={} rdtscp={}\n",
            self.aed, self.ixx, self.lbm, self.ixt));
        e.t(&format!("  SIMD: SSE={} SSE2={} SSE3={} SSSE3={} SSE4.1={} SSE4.2={}\n",
            self.ixu, self.dro, self.ixv, self.lbk,
            self.lbj, self.hmq));
        e.t(&format!("        AVX={} AVX2={} AVX-512={}\n",
            self.fke, self.bzx, self.drm));
        e.t(&format!("  Crypto: AES-NI={} PCLMULQDQ={} SHA={} RDRAND={} RDSEED={}\n",
            self.cfe, self.git, self.ecm,
            self.crd, self.fkh));
        e.t(&format!("  Security: SMEP={} SMAP={} UMIP={} NX={}\n",
            self.erv, self.eru, self.giv, self.ert));
        e.t(&format!("  Virt: VMX={} SVM={}\n\n", self.giw, self.giu));

        
        e.t(&format!("\x01Y[Memory]\x01W {} MB physical\n", self.ccf / (1024 * 1024)));
        e.t(&format!("  Heap: {} KB used / {} KB free (of {} KB)\n",
            self.ecw / 1024, self.erx / 1024, self.drr / 1024));
        e.t(&format!("  Frames: {} used / {} free  HHDM: 0x{:X}\n\n",
            self.ceu, self.dhj, self.lr));

        
        e.t(&format!("\x01Y[ACPI/Firmware]\x01W Rev={} OEM='{}'\n", self.gxt, self.gxs));
        e.t(&format!("  FADT: SCI={} HW_Reduced={} Reset={} LowPowerS0={} PM_TMR=0x{:X}\n",
            self.kvd, self.hiu, self.itu,
            self.kvb, self.kvc));
        e.t(&format!("  Local APIC: 0x{:X}  {} CPU APIC entries\n",
            self.cap, self.gae.len()));
        e.t(&format!("  IOAPICs: {}  IRQ Overrides: {}  NMIs: {}\n",
            self.edq, self.hoq.len(), self.kar));
        if self.fqo {
            e.t(&format!("  PCIe: {} segment(s)\n", self.gpa.len()));
        }
        e.push('\n');

        
        if self.esa {
            e.t(&format!("\x01Y[HPET]\x01W {} MHz, {} timers, 64-bit={}, vendor=0x{:04X}\n\n",
                self.hnd / 1_000_000, self.gjf,
                self.iys, self.iyt));
        }

        
        e.t(&format!("\x01Y[Storage]\x01W {} device(s), {} GB total\n",
            self.aqm.len(), self.dmp / (1024 * 1024 * 1024)));
        for ba in &self.aqm {
            e.t(&format!("  {} [{}] {} — {} GB\n",
                ba.j, ba.kk.as_str(), ba.model,
                ba.fei / (1024 * 1024 * 1024)));
            if !ba.serial.is_empty() {
                e.t(&format!("    Serial: {}\n", ba.serial));
            }
        }

        
        if !self.aqd.is_empty() {
            e.t(&format!("  {} partition(s):\n", self.aqd.len()));
            for ai in &self.aqd {
                let uri = if !ai.j.is_empty() { format!(" '{}'", ai.j) } else { String::new() };
                e.t(&format!("    #{} [{}] {} {} GB{}{}\n",
                    ai.aqb, ai.app, ai.ddc,
                    ai.afz / (1024 * 1024 * 1024),
                    if ai.cji { " *BOOT*" } else { "" },
                    uri));
            }
        }

        
        if !self.avs.is_empty() {
            e.t("\x01R  ⚠ Encrypted volumes detected:\x01W\n");
            for bdy in &self.avs {
                e.t(&format!("    \x01R[{}]\x01W {} — {}\n",
                    bdy.ckf.as_str(), bdy.app, bdy.eu));
            }
        }
        e.push('\n');

        
        e.t(&format!("\x01Y[Network]\x01W detected={} link={}\n", self.bzz, self.aik));
        if let Some(ed) = self.csg {
            e.t(&format!("  MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
                ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]));
        }
        e.push('\n');

        
        if self.bqz {
            e.t(&format!("\x01Y[GPU]\x01W {}\n", self.beh));
            e.t(&format!("  VRAM: {} MB  CUs: {}\n\n", self.dhr, self.erk));
        } else {
            e.t("\x01Y[GPU]\x01W None detected\n\n");
        }

        
        e.t(&format!("\x01Y[USB]\x01W init={} controllers={} devices={}\n",
            self.fav, self.fxv, self.cvc.len()));
        for usb in &self.cvc {
            e.t(&format!("  [{:04X}:{:04X}] {} ({})\n",
                usb.ml, usb.cgt, usb.baj, usb.bpz));
        }
        e.push('\n');

        
        e.t(&format!("\x01Y[Audio]\x01W HDA init={}\n\n", self.fki));

        
        e.t(&format!("\x01Y[PCI Bus]\x01W {} devices ({}stor {}net {}usb {}audio {}disp {}bridge {}crypto)\n",
            self.dal, self.ewl,
            self.egg, self.egh,
            self.egf, self.ewk,
            self.hur, self.fqn));
        for ba in self.hus.iter().take(20) {
            e.t(&format!("  {:02X}:{:02X}.{} [{:04X}:{:04X}] {} — {}\n",
                ba.aq, ba.de, ba.gw,
                ba.ml, ba.mx,
                ba.bpz, ba.bor));
        }
        if self.dal > 20 {
            e.t(&format!("  ... and {} more\n", self.dal - 20));
        }
        e.push('\n');

        
        e.t("\x01C═══ Capability Scores ═══\x01W\n");
        e.t(&format!("  Compute:  {} {}\n", grn(self.cwl), ghk(self.cwl)));
        e.t(&format!("  Memory:   {} {}\n", grn(self.dte), ghk(self.dte)));
        e.t(&format!("  Storage:  {} {}\n", grn(self.ezb), ghk(self.ezb)));
        e.t(&format!("  Network:  {} {}\n", grn(self.evg), ghk(self.evg)));
        e.t(&format!("  Security: {} {}\n", grn(self.eyh), ghk(self.eyh)));
        e.t(&format!("  \x01COverall:  {} {}\x01W\n", grn(self.dkj), ghk(self.dkj)));

        e
    }

    
    pub fn zeh(&self) -> String {
        format!("{} {}C {}MB {}xStorage {}GPU score={:.0}%",
            self.arch, self.azj,
            self.ccf / (1024 * 1024),
            self.aqm.len(),
            if self.bqz { "+" } else { "-" },
            self.dkj * 100.0)
    }

    
    pub fn xig(&self) -> String {
        let mut e = String::new();
        e.t("HARDWARE CONTEXT [exhaustive]:\n");

        
        e.t(&format!("CPU: vendor={} brand='{}' arch={} family={} model={} stepping={}\n",
            self.avo, self.dpf, self.arch, self.heb,
            self.hec, self.hee));
        e.t(&format!("  cores={} logical={} physical={} tsc_mhz={} apic_id={}\n",
            self.azj, self.cau, self.djk,
            self.fam / 1_000_000, self.aed));

        
        let simd = if self.drm { "avx512" }
            else if self.bzx { "avx2" }
            else if self.fke { "avx" }
            else if self.hmq { "sse4.2" }
            else if self.dro { "sse2" }
            else { "none" };
        e.t(&format!("  simd_level={} tsc_invariant={} rdtscp={}\n",
            simd, self.ixx, self.ixt));

        
        e.t(&format!("  crypto: aesni={} pclmulqdq={} sha={} rdrand={} rdseed={}\n",
            self.cfe, self.git, self.ecm,
            self.crd, self.fkh));
        
        e.t(&format!("  security: smep={} smap={} umip={} nx={} vmx={} svm={}\n",
            self.erv, self.eru, self.giv, self.ert,
            self.giw, self.giu));

        
        e.t(&format!("MEMORY: total={}MB heap={}KB(used={}KB free={}KB) frames_used={} frames_free={}\n",
            self.ccf / (1024 * 1024),
            self.drr / 1024, self.ecw / 1024,
            self.erx / 1024, self.ceu, self.dhj));

        
        e.t(&format!("ACPI: rev={} oem='{}' hw_reduced={} reset={}\n",
            self.gxt, self.gxs, self.hiu, self.itu));
        e.t(&format!("  apic_cpus={} ioapics={} irq_overrides={} pcie_segments={}\n",
            self.gae.len(), self.edq,
            self.hoq.len(), self.gpa.len()));

        
        e.t(&format!("STORAGE: devices={} total_gb={}\n",
            self.aqm.len(), self.dmp / (1024 * 1024 * 1024)));
        for ba in &self.aqm {
            e.t(&format!("  {} [{}] {}GB model='{}'\n",
                ba.j, ba.kk.as_str(), ba.fei / (1024 * 1024 * 1024), ba.model));
        }

        
        if !self.aqd.is_empty() {
            e.t(&format!("PARTITIONS: {}\n", self.aqd.len()));
            for ai in &self.aqd {
                e.t(&format!("  disk={} #{} type={} {}GB boot={}\n",
                    ai.app, ai.aqb, ai.ddc,
                    ai.afz / (1024 * 1024 * 1024), ai.cji));
            }
        }

        
        if !self.avs.is_empty() {
            e.t("ENCRYPTION_DETECTED:\n");
            for bdy in &self.avs {
                e.t(&format!("  disk={} type={} detail='{}'\n",
                    bdy.app, bdy.ckf.as_str(), bdy.eu));
            }
        } else {
            e.t("ENCRYPTION_DETECTED: none\n");
        }

        
        e.t(&format!("NETWORK: has_driver={} link_up={}", self.bzz, self.aik));
        if let Some(ed) = self.csg {
            e.t(&format!(" mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]));
        }
        e.push('\n');

        
        e.t(&format!("GPU: has={} name='{}' vram_mb={} compute_units={}\n",
            self.bqz, self.beh, self.dhr, self.erk));

        
        e.t(&format!("TIMERS: tsc={} hpet={} hpet_mhz={} hpet_timers={}\n",
            self.juh, self.esa,
            self.hnd / 1_000_000, self.gjf));

        
        e.t(&format!("USB: controllers={} devices={}\n",
            self.fxv, self.cvc.len()));
        for usb in &self.cvc {
            e.t(&format!("  [{:04X}:{:04X}] {} ({})\n",
                usb.ml, usb.cgt, usb.baj, usb.bpz));
        }

        
        e.t(&format!("AUDIO: hda_init={}\n", self.fki));

        
        e.t(&format!("PCI: total={} storage={} net={} usb={} audio={} display={} crypto={}\n",
            self.dal, self.ewl,
            self.egg, self.egh,
            self.egf, self.ewk,
            self.fqn));

        
        e.t(&format!("SCORES: compute={:.0}% memory={:.0}% storage={:.0}% network={:.0}% security={:.0}% overall={:.0}%\n",
            self.cwl * 100.0, self.dte * 100.0,
            self.ezb * 100.0, self.evg * 100.0,
            self.eyh * 100.0, self.dkj * 100.0));

        e
    }

    
    pub fn ywh(&self, mh: &str) -> bool {
        match mh {
            "aesni" | "aes" => self.cfe,
            "rdrand" | "random" => self.crd,
            "rdseed" => self.fkh,
            "sha" => self.ecm,
            "avx2" => self.bzx,
            "avx512" => self.drm,
            "gpu" => self.bqz,
            "network" | "net" => self.bzz,
            "storage" | "disk" => !self.aqm.is_empty(),
            "usb" => self.fav,
            "audio" | "sound" => self.fki,
            "smep" => self.erv,
            "smap" => self.eru,
            "nx" => self.ert,
            "vmx" | "vt-x" => self.giw,
            "svm" | "amd-v" => self.giu,
            "pcie" => self.fqo,
            "hpet" => self.esa,
            "encryption" | "encrypted" => !self.avs.is_empty(),
            _ => false,
        }
    }
}

fn grn(ol: f32) -> String {
    let adu = (ol * 20.0) as usize;
    let azs = 20 - adu;
    format!("[{}{}]",
        "#".afd(adu),
        "-".afd(azs))
}

fn ghk(ol: f32) -> String {
    format!("{:.0}%", ol * 100.0)
}
