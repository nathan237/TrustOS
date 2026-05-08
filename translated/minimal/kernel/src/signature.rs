




















use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;














pub const ARJ_: &str = 
    "TrustOS Kernel — Created by Nated0ge (nathan237) — Sole author and originator — All rights reserved 2025-2026";


pub const BSJ_: &str = "Nated0ge";
pub const BSI_: &str = "nathan237";









pub const ARI_: [u8; 32] = [
    0x0c, 0x1a, 0x99, 0xfb, 0x1e, 0x87, 0x77, 0xce,
    0x12, 0x0c, 0xca, 0x83, 0x4e, 0x75, 0x60, 0x8e,
    0x95, 0xa4, 0xb6, 0xc5, 0xd3, 0x04, 0x7a, 0x92,
    0xa1, 0xfe, 0x10, 0xb3, 0x10, 0xb8, 0x7c, 0xbd,
];


pub const BOF_: &str = env!("TRUSTOS_BUILD_TIME", "unknown");


pub const OS_: &str = "0.1.2";






pub struct Aga {
    pub name: String,
    pub fingerprint: [u8; 32],
    pub timestamp: u64, 
}


static ZI_: Mutex<Option<Aga>> = Mutex::new(None);






pub fn bgu(hash: &[u8; 32]) -> String {
    let mut j = String::with_capacity(64);
    for &b in hash.iter() {
        let hi = b >> 4;
        let lo = b & 0x0F;
        j.push(if hi < 10 { (b'0' + hi) as char } else { (b'a' + hi - 10) as char });
        j.push(if lo < 10 { (b'0' + lo) as char } else { (b'a' + lo - 10) as char });
    }
    j
}


pub fn hou() -> String {
    bgu(&ARI_)
}




pub fn prq(seed: &[u8]) -> bool {
    let fnx = crate::tls13::crypto::bmu(seed, ARJ_.as_bytes());
    
    let mut jr = 0u8;
    for i in 0..32 {
        jr |= fnx[i] ^ ARI_[i];
    }
    jr == 0
}




pub fn oso(name: &str, amd: &[u8]) {
    
    let mut payload = Vec::new();
    payload.extend_from_slice(b"TrustOS User Signature: ");
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(b" -- co-signed kernel v");
    payload.extend_from_slice(OS_.as_bytes());

    let fingerprint = crate::tls13::crypto::bmu(amd, &payload);

    let jy = crate::rtc::iby() as u64;

    let sig = Aga {
        name: String::from(name),
        fingerprint,
        timestamp: jy,
    };

    let mut slot = ZI_.lock();
    *slot = Some(sig);
}


pub fn eoe() -> Option<(String, String, u64)> {
    let slot = ZI_.lock();
    slot.as_ref().map(|j| (j.name.clone(), bgu(&j.fingerprint), j.timestamp))
}


pub fn prv(name: &str, amd: &[u8]) -> bool {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"TrustOS User Signature: ");
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(b" -- co-signed kernel v");
    payload.extend_from_slice(OS_.as_bytes());

    let fnx = crate::tls13::crypto::bmu(amd, &payload);

    let slot = ZI_.lock();
    if let Some(ref sig) = *slot {
        if sig.name != name {
            return false;
        }
        let mut jr = 0u8;
        for i in 0..32 {
            jr |= fnx[i] ^ sig.fingerprint[i];
        }
        jr == 0
    } else {
        false
    }
}


pub fn kkv() {
    let mut slot = ZI_.lock();
    *slot = None;
}











































static NV_: Mutex<Option<[u8; 32]>> = Mutex::new(None);


static UD_: Mutex<Option<[u8; 64]>> = Mutex::new(None);



pub fn mpe(seed: &[u8]) {
    
    let dyx = crate::tls13::crypto::asg(seed);
    let pubkey = crate::ed25519::ftz(&dyx);
    
    
    let cik = MW_.lock();
    if let Some(bry) = *cik {
        drop(cik);
        let sig = crate::ed25519::huz(&bry, &dyx, &pubkey);
        *NV_.lock() = Some(pubkey);
        *UD_.lock() = Some(sig);
        
        let ixb = crate::ed25519::fkk(&pubkey);
        crate::serial_println!("[ED25519] Public key: {}...{}", &ixb[..16], &ixb[48..]);
        crate::serial_println!("[ED25519] Kernel digest signed");
    } else {
        drop(cik);
        
        *NV_.lock() = Some(pubkey);
        crate::serial_println!("[ED25519] Public key initialized (no digest to sign yet)");
    }
}


pub fn lnv() -> Option<String> {
    NV_.lock().map(|k| crate::ed25519::fkk(&k))
}


pub fn lnu() -> Option<String> {
    UD_.lock().map(|j| crate::ed25519::fkk(&j))
}




pub fn prs() -> Result<bool, &'static str> {
    let pubkey = NV_.lock().ok_or("Ed25519 not initialized")?;
    let sig = UD_.lock().ok_or("No Ed25519 signature")?;
    let kdf = MW_.lock().ok_or("Integrity not initialized")?;
    
    Ok(crate::ed25519::hva(&kdf, &sig, &pubkey))
}



pub fn lnw(seed: &[u8]) -> bool {
    let dyx = crate::tls13::crypto::asg(seed);
    let pubkey = crate::ed25519::ftz(&dyx);
    
    let cik = MW_.lock();
    if let Some(bry) = *cik {
        drop(cik);
        let sig = crate::ed25519::huz(&bry, &dyx, &pubkey);
        
        
        if crate::ed25519::hva(&bry, &sig, &pubkey) {
            *NV_.lock() = Some(pubkey);
            *UD_.lock() = Some(sig);
            true
        } else {
            false
        }
    } else {
        drop(cik);
        false
    }
}


pub fn huy() -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(String::from("  Ed25519 Digital Signature"));
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    
    if let Some(ga) = lnv() {
        lines.push(alloc::format!("  Public key  : {}", ga));
    } else {
        lines.push(String::from("  Public key  : NOT INITIALIZED"));
        lines.push(String::from("  Run: signature ed25519 sign <seed>"));
        return lines;
    }
    
    if let Some(ga) = lnu() {
        lines.push(alloc::format!("  Signature   : {}...", &ga[..64]));
        lines.push(alloc::format!("                ...{}", &ga[64..]));
    } else {
        lines.push(String::from("  Signature   : NONE"));
    }
    
    match prs() {
        Ok(true) => {
            lines.push(String::from("  Verification: ✅ VALID — kernel signed by key holder"));
        }
        Ok(false) => {
            lines.push(String::from("  Verification: ❌ INVALID — signature does not match!"));
        }
        Err(e) => {
            lines.push(alloc::format!("  Verification: ⚠  {}", e));
        }
    }
    
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    lines.push(String::from("  Algorithm: Ed25519 (RFC 8032)"));
    lines.push(String::from("  Curve: twisted Edwards / Curve25519"));
    lines.push(String::from("  Asymmetric: public key verifies, private key signs"));
    
    lines
}

extern "C" {
    static __text_start: u8;
    static __text_end: u8;
    static __rodata_start: u8;
    static __rodata_end: u8;
}


static SQ_: Mutex<Option<[u8; 32]>> = Mutex::new(None);


static SP_: Mutex<Option<[u8; 32]>> = Mutex::new(None);


static MW_: Mutex<Option<[u8; 32]>> = Mutex::new(None);


fn eox() -> [u8; 32] {
    let start = unsafe { &__text_start as *const u8 as usize };
    let end = unsafe { &__text_end as *const u8 as usize };
    let size = end.saturating_sub(start);
    
    let pig = unsafe {
        core::slice::from_raw_parts(start as *const u8, size)
    };
    
    crate::tls13::crypto::asg(pig)
}


fn eow() -> [u8; 32] {
    let start = unsafe { &__rodata_start as *const u8 as usize };
    let end = unsafe { &__rodata_end as *const u8 as usize };
    let size = end.saturating_sub(start);
    
    let ohv = unsafe {
        core::slice::from_raw_parts(start as *const u8, size)
    };
    
    crate::tls13::crypto::asg(ohv)
}



fn hng(ebm: &[u8; 32], dxx: &[u8; 32]) -> [u8; 32] {
    let mut bav = [0u8; 64];
    bav[..32].copy_from_slice(ebm);
    bav[32..].copy_from_slice(dxx);
    crate::tls13::crypto::asg(&bav)
}



pub fn mph() {
    let ebm = eox();
    let dxx = eow();
    let bry = hng(&ebm, &dxx);
    
    let jmd = bgu(&ebm);
    let jbf = bgu(&dxx);
    let hsf = bgu(&bry);
    
    crate::serial_println!("[INTEGRITY] .text   : {} bytes, SHA-256: {}...{}", 
        jme(), &jmd[..16], &jmd[56..]);
    crate::serial_println!("[INTEGRITY] .rodata : {} bytes, SHA-256: {}...{}", 
        jbg(), &jbf[..16], &jbf[56..]);
    crate::serial_println!("[INTEGRITY] kernel digest: {}...{}", 
        &hsf[..16], &hsf[56..]);
    
    *SQ_.lock() = Some(ebm);
    *SP_.lock() = Some(dxx);
    *MW_.lock() = Some(bry);
}


pub fn jme() -> usize {
    let start = unsafe { &__text_start as *const u8 as usize };
    let end = unsafe { &__text_end as *const u8 as usize };
    end.saturating_sub(start)
}


pub fn jbg() -> usize {
    let start = unsafe { &__rodata_start as *const u8 as usize };
    let end = unsafe { &__rodata_end as *const u8 as usize };
    end.saturating_sub(start)
}


pub fn kdm() -> Option<String> {
    SQ_.lock().map(|h| bgu(&h))
}


pub fn kdj() -> Option<String> {
    SP_.lock().map(|h| bgu(&h))
}


pub fn kdg() -> Option<String> {
    MW_.lock().map(|h| bgu(&h))
}



pub fn jqa() -> Result<bool, &'static str> {
    let kdl = SQ_.lock().ok_or("Integrity not initialized")?;
    let kdi = SP_.lock().ok_or("Integrity not initialized")?;
    
    let ejo = eox();
    let ejn = eow();
    
    
    let mut jr = 0u8;
    for i in 0..32 {
        jr |= kdl[i] ^ ejo[i];
        jr |= kdi[i] ^ ejn[i];
    }
    
    Ok(jr == 0)
}


pub fn pru() -> Result<bool, &'static str> {
    let fjp = SQ_.lock().ok_or("Integrity not initialized")?;
    let current = eox();
    let mut jr = 0u8;
    for i in 0..32 { jr |= fjp[i] ^ current[i]; }
    Ok(jr == 0)
}


pub fn prt() -> Result<bool, &'static str> {
    let fjp = SP_.lock().ok_or("Integrity not initialized")?;
    let current = eow();
    let mut jr = 0u8;
    for i in 0..32 { jr |= fjp[i] ^ current[i]; }
    Ok(jr == 0)
}


pub fn mqw() -> Vec<String> {
    let mut lines = Vec::new();
    let crn = jme();
    let jbh = jbg();
    
    lines.push(String::from("  Kernel Integrity Verification"));
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    lines.push(alloc::format!("  .text section   : {} bytes ({} KB)", crn, crn / 1024));
    lines.push(alloc::format!("  .rodata section : {} bytes ({} KB)", jbh, jbh / 1024));
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    
    
    if let Some(ga) = kdm() {
        lines.push(alloc::format!("  .text boot hash   : {}", ga));
    } else {
        lines.push(String::from("  .text boot hash   : NOT INITIALIZED"));
        return lines;
    }
    let ejo = eox();
    lines.push(alloc::format!("  .text current     : {}", bgu(&ejo)));
    
    match pru() {
        Ok(true) => lines.push(String::from("  .text status      : ✅ INTACT")),
        Ok(false) => lines.push(String::from("  .text status      : ❌ MODIFIED")),
        Err(e) => lines.push(alloc::format!("  .text status      : ⚠️  {}", e)),
    }
    
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    
    
    if let Some(ga) = kdj() {
        lines.push(alloc::format!("  .rodata boot hash : {}", ga));
    } else {
        lines.push(String::from("  .rodata boot hash : NOT INITIALIZED"));
    }
    let ejn = eow();
    lines.push(alloc::format!("  .rodata current   : {}", bgu(&ejn)));
    
    match prt() {
        Ok(true) => lines.push(String::from("  .rodata status    : ✅ INTACT")),
        Ok(false) => lines.push(String::from("  .rodata status    : ❌ MODIFIED")),
        Err(e) => lines.push(alloc::format!("  .rodata status    : ⚠️  {}", e)),
    }
    
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    
    
    if let Some(ga) = kdg() {
        lines.push(alloc::format!("  Kernel digest     : {}", ga));
    }
    let bry = hng(&ejo, &ejn);
    lines.push(alloc::format!("  Current digest    : {}", bgu(&bry)));
    
    match jqa() {
        Ok(true) => {
            lines.push(String::from("  Overall status    : ✅ INTEGRITY OK — kernel unmodified"));
        }
        Ok(false) => {
            lines.push(String::from("  Overall status    : ❌ INTEGRITY VIOLATION — kernel was tampered!"));
            lines.push(String::from("  WARNING: Code or read-only data modified since boot."));
        }
        Err(e) => {
            lines.push(alloc::format!("  Overall status    : ⚠️  {}", e));
        }
    }
    
    lines.push(String::from("  ─────────────────────────────────────────────────"));
    lines.push(String::from("  Algorithm: SHA-256 per-section + combined digest"));
    lines.push(String::from("  Threat model: detects post-boot code/data tampering"));
    
    lines
}
