




















use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;














pub const APJ_: &str = 
    "TrustOS Kernel — Created by Nated0ge (nathan237) — Sole author and originator — All rights reserved 2025-2026";


pub const BPS_: &str = "Nated0ge";
pub const BPR_: &str = "nathan237";









pub const API_: [u8; 32] = [
    0x0c, 0x1a, 0x99, 0xfb, 0x1e, 0x87, 0x77, 0xce,
    0x12, 0x0c, 0xca, 0x83, 0x4e, 0x75, 0x60, 0x8e,
    0x95, 0xa4, 0xb6, 0xc5, 0xd3, 0x04, 0x7a, 0x92,
    0xa1, 0xfe, 0x10, 0xb3, 0x10, 0xb8, 0x7c, 0xbd,
];


pub const BLM_: &str = env!("TRUSTOS_BUILD_TIME", "unknown");


pub const NU_: &str = "0.1.2";






pub struct Bvn {
    pub j: String,
    pub ius: [u8; 32],
    pub aea: u64, 
}


static YE_: Mutex<Option<Bvn>> = Mutex::new(None);






pub fn dhw(hash: &[u8; 32]) -> String {
    let mut e = String::fc(64);
    for &o in hash.iter() {
        let gd = o >> 4;
        let hh = o & 0x0F;
        e.push(if gd < 10 { (b'0' + gd) as char } else { (b'a' + gd - 10) as char });
        e.push(if hh < 10 { (b'0' + hh) as char } else { (b'a' + hh - 10) as char });
    }
    e
}


pub fn nhk() -> String {
    dhw(&API_)
}




pub fn xrg(dv: &[u8]) -> bool {
    let kkg = crate::tls13::crypto::drt(dv, APJ_.as_bytes());
    
    let mut wz = 0u8;
    for a in 0..32 {
        wz |= kkg[a] ^ API_[a];
    }
    wz == 0
}




pub fn wnz(j: &str, bvw: &[u8]) {
    
    let mut ew = Vec::new();
    ew.bk(b"TrustOS User Signature: ");
    ew.bk(j.as_bytes());
    ew.bk(b" -- co-signed kernel v");
    ew.bk(NU_.as_bytes());

    let ius = crate::tls13::crypto::drt(bvw, &ew);

    let wi = crate::rtc::nyr() as u64;

    let sig = Bvn {
        j: String::from(j),
        ius,
        aea: wi,
    };

    let mut gk = YE_.lock();
    *gk = Some(sig);
}


pub fn iww() -> Option<(String, String, u64)> {
    let gk = YE_.lock();
    gk.as_ref().map(|e| (e.j.clone(), dhw(&e.ius), e.aea))
}


pub fn xrm(j: &str, bvw: &[u8]) -> bool {
    let mut ew = Vec::new();
    ew.bk(b"TrustOS User Signature: ");
    ew.bk(j.as_bytes());
    ew.bk(b" -- co-signed kernel v");
    ew.bk(NU_.as_bytes());

    let kkg = crate::tls13::crypto::drt(bvw, &ew);

    let gk = YE_.lock();
    if let Some(ref sig) = *gk {
        if sig.j != j {
            return false;
        }
        let mut wz = 0u8;
        for a in 0..32 {
            wz |= kkg[a] ^ sig.ius[a];
        }
        wz == 0
    } else {
        false
    }
}


pub fn rbh() {
    let mut gk = YE_.lock();
    *gk = None;
}











































static MW_: Mutex<Option<[u8; 32]>> = Mutex::new(None);


static SX_: Mutex<Option<[u8; 64]>> = Mutex::new(None);



pub fn ttl(dv: &[u8]) {
    
    let hzk = crate::tls13::crypto::chw(dv);
    let cbd = crate::ed25519::ksp(&hzk);
    
    
    let fgt = LZ_.lock();
    if let Some(eau) = *fgt {
        drop(fgt);
        let sig = crate::ed25519::npd(&eau, &hzk, &cbd);
        *MW_.lock() = Some(cbd);
        *SX_.lock() = Some(sig);
        
        let oyj = crate::ed25519::kfv(&cbd);
        crate::serial_println!("[ED25519] Public key: {}...{}", &oyj[..16], &oyj[48..]);
        crate::serial_println!("[ED25519] Kernel digest signed");
    } else {
        drop(fgt);
        
        *MW_.lock() = Some(cbd);
        crate::serial_println!("[ED25519] Public key initialized (no digest to sign yet)");
    }
}


pub fn sio() -> Option<String> {
    MW_.lock().map(|eh| crate::ed25519::kfv(&eh))
}


pub fn sin() -> Option<String> {
    SX_.lock().map(|e| crate::ed25519::kfv(&e))
}




pub fn xri() -> Result<bool, &'static str> {
    let cbd = MW_.lock().ok_or("Ed25519 not initialized")?;
    let sig = SX_.lock().ok_or("No Ed25519 signature")?;
    let qqx = LZ_.lock().ok_or("Integrity not initialized")?;
    
    Ok(crate::ed25519::npe(&qqx, &sig, &cbd))
}



pub fn sip(dv: &[u8]) -> bool {
    let hzk = crate::tls13::crypto::chw(dv);
    let cbd = crate::ed25519::ksp(&hzk);
    
    let fgt = LZ_.lock();
    if let Some(eau) = *fgt {
        drop(fgt);
        let sig = crate::ed25519::npd(&eau, &hzk, &cbd);
        
        
        if crate::ed25519::npe(&eau, &sig, &cbd) {
            *MW_.lock() = Some(cbd);
            *SX_.lock() = Some(sig);
            true
        } else {
            false
        }
    } else {
        drop(fgt);
        false
    }
}


pub fn npc() -> Vec<String> {
    let mut ak = Vec::new();
    ak.push(String::from("  Ed25519 Digital Signature"));
    ak.push(String::from("  ─────────────────────────────────────────────────"));
    
    if let Some(nu) = sio() {
        ak.push(alloc::format!("  Public key  : {}", nu));
    } else {
        ak.push(String::from("  Public key  : NOT INITIALIZED"));
        ak.push(String::from("  Run: signature ed25519 sign <seed>"));
        return ak;
    }
    
    if let Some(nu) = sin() {
        ak.push(alloc::format!("  Signature   : {}...", &nu[..64]));
        ak.push(alloc::format!("                ...{}", &nu[64..]));
    } else {
        ak.push(String::from("  Signature   : NONE"));
    }
    
    match xri() {
        Ok(true) => {
            ak.push(String::from("  Verification: ✅ VALID — kernel signed by key holder"));
        }
        Ok(false) => {
            ak.push(String::from("  Verification: ❌ INVALID — signature does not match!"));
        }
        Err(aa) => {
            ak.push(alloc::format!("  Verification: ⚠  {}", aa));
        }
    }
    
    ak.push(String::from("  ─────────────────────────────────────────────────"));
    ak.push(String::from("  Algorithm: Ed25519 (RFC 8032)"));
    ak.push(String::from("  Curve: twisted Edwards / Curve25519"));
    ak.push(String::from("  Asymmetric: public key verifies, private key signs"));
    
    ak
}

extern "C" {
    static msd: u8;
    static msc: u8;
    static msb: u8;
    static msa: u8;
}


static RO_: Mutex<Option<[u8; 32]>> = Mutex::new(None);


static RN_: Mutex<Option<[u8; 32]>> = Mutex::new(None);


static LZ_: Mutex<Option<[u8; 32]>> = Mutex::new(None);


fn iya() -> [u8; 32] {
    let ay = unsafe { &msd as *const u8 as usize };
    let ci = unsafe { &msc as *const u8 as usize };
    let aw = ci.ao(ay);
    
    let xfs = unsafe {
        core::slice::anh(ay as *const u8, aw)
    };
    
    crate::tls13::crypto::chw(xfs)
}


fn ixz() -> [u8; 32] {
    let ay = unsafe { &msb as *const u8 as usize };
    let ci = unsafe { &msa as *const u8 as usize };
    let aw = ci.ao(ay);
    
    let vzv = unsafe {
        core::slice::anh(ay as *const u8, aw)
    };
    
    crate::tls13::crypto::chw(vzv)
}



fn nfi(idg: &[u8; 32], hxv: &[u8; 32]) -> [u8; 32] {
    let mut cwk = [0u8; 64];
    cwk[..32].dg(idg);
    cwk[32..].dg(hxv);
    crate::tls13::crypto::chw(&cwk)
}



pub fn ttp() {
    let idg = iya();
    let hxv = ixz();
    let eau = nfi(&idg, &hxv);
    
    let pss = dhw(&idg);
    let pdv = dhw(&hxv);
    let nll = dhw(&eau);
    
    crate::serial_println!("[INTEGRITY] .text   : {} bytes, SHA-256: {}...{}", 
        pst(), &pss[..16], &pss[56..]);
    crate::serial_println!("[INTEGRITY] .rodata : {} bytes, SHA-256: {}...{}", 
        pdw(), &pdv[..16], &pdv[56..]);
    crate::serial_println!("[INTEGRITY] kernel digest: {}...{}", 
        &nll[..16], &nll[56..]);
    
    *RO_.lock() = Some(idg);
    *RN_.lock() = Some(hxv);
    *LZ_.lock() = Some(eau);
}


pub fn pst() -> usize {
    let ay = unsafe { &msd as *const u8 as usize };
    let ci = unsafe { &msc as *const u8 as usize };
    ci.ao(ay)
}


pub fn pdw() -> usize {
    let ay = unsafe { &msb as *const u8 as usize };
    let ci = unsafe { &msa as *const u8 as usize };
    ci.ao(ay)
}


pub fn qrf() -> Option<String> {
    RO_.lock().map(|i| dhw(&i))
}


pub fn qrc() -> Option<String> {
    RN_.lock().map(|i| dhw(&i))
}


pub fn qqy() -> Option<String> {
    LZ_.lock().map(|i| dhw(&i))
}



pub fn pye() -> Result<bool, &'static str> {
    let qre = RO_.lock().ok_or("Integrity not initialized")?;
    let qrb = RN_.lock().ok_or("Integrity not initialized")?;
    
    let iqf = iya();
    let iqd = ixz();
    
    
    let mut wz = 0u8;
    for a in 0..32 {
        wz |= qre[a] ^ iqf[a];
        wz |= qrb[a] ^ iqd[a];
    }
    
    Ok(wz == 0)
}


pub fn xrl() -> Result<bool, &'static str> {
    let kee = RO_.lock().ok_or("Integrity not initialized")?;
    let cv = iya();
    let mut wz = 0u8;
    for a in 0..32 { wz |= kee[a] ^ cv[a]; }
    Ok(wz == 0)
}


pub fn xrk() -> Result<bool, &'static str> {
    let kee = RN_.lock().ok_or("Integrity not initialized")?;
    let cv = ixz();
    let mut wz = 0u8;
    for a in 0..32 { wz |= kee[a] ^ cv[a]; }
    Ok(wz == 0)
}


pub fn tvn() -> Vec<String> {
    let mut ak = Vec::new();
    let fwp = pst();
    let pdx = pdw();
    
    ak.push(String::from("  Kernel Integrity Verification"));
    ak.push(String::from("  ─────────────────────────────────────────────────"));
    ak.push(alloc::format!("  .text section   : {} bytes ({} KB)", fwp, fwp / 1024));
    ak.push(alloc::format!("  .rodata section : {} bytes ({} KB)", pdx, pdx / 1024));
    ak.push(String::from("  ─────────────────────────────────────────────────"));
    
    
    if let Some(nu) = qrf() {
        ak.push(alloc::format!("  .text boot hash   : {}", nu));
    } else {
        ak.push(String::from("  .text boot hash   : NOT INITIALIZED"));
        return ak;
    }
    let iqf = iya();
    ak.push(alloc::format!("  .text current     : {}", dhw(&iqf)));
    
    match xrl() {
        Ok(true) => ak.push(String::from("  .text status      : ✅ INTACT")),
        Ok(false) => ak.push(String::from("  .text status      : ❌ MODIFIED")),
        Err(aa) => ak.push(alloc::format!("  .text status      : ⚠️  {}", aa)),
    }
    
    ak.push(String::from("  ─────────────────────────────────────────────────"));
    
    
    if let Some(nu) = qrc() {
        ak.push(alloc::format!("  .rodata boot hash : {}", nu));
    } else {
        ak.push(String::from("  .rodata boot hash : NOT INITIALIZED"));
    }
    let iqd = ixz();
    ak.push(alloc::format!("  .rodata current   : {}", dhw(&iqd)));
    
    match xrk() {
        Ok(true) => ak.push(String::from("  .rodata status    : ✅ INTACT")),
        Ok(false) => ak.push(String::from("  .rodata status    : ❌ MODIFIED")),
        Err(aa) => ak.push(alloc::format!("  .rodata status    : ⚠️  {}", aa)),
    }
    
    ak.push(String::from("  ─────────────────────────────────────────────────"));
    
    
    if let Some(nu) = qqy() {
        ak.push(alloc::format!("  Kernel digest     : {}", nu));
    }
    let eau = nfi(&iqf, &iqd);
    ak.push(alloc::format!("  Current digest    : {}", dhw(&eau)));
    
    match pye() {
        Ok(true) => {
            ak.push(String::from("  Overall status    : ✅ INTEGRITY OK — kernel unmodified"));
        }
        Ok(false) => {
            ak.push(String::from("  Overall status    : ❌ INTEGRITY VIOLATION — kernel was tampered!"));
            ak.push(String::from("  WARNING: Code or read-only data modified since boot."));
        }
        Err(aa) => {
            ak.push(alloc::format!("  Overall status    : ⚠️  {}", aa));
        }
    }
    
    ak.push(String::from("  ─────────────────────────────────────────────────"));
    ak.push(String::from("  Algorithm: SHA-256 per-section + combined digest"));
    ak.push(String::from("  Threat model: detects post-boot code/data tampering"));
    
    ak
}
