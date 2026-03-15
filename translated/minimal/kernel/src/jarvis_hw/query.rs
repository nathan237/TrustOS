











use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::probe::{T, EncryptionType};


pub struct Bl {
    pub yt: String,
    pub azi: f32,      
    pub bbd: Option<bool>, 
    pub yw: Vec<String>, 
}


pub fn qiv(query: &str, cc: &T) -> Bl {
    let fm = query.avd();

    
    if fm.contains("can you") || fm.contains("peux tu") || fm.contains("peut tu")
        || fm.contains("est-ce que") || fm.contains("is it possible")
    {
        
        if fm.contains("encrypt") || fm.contains("chiffr") || fm.contains("crypt")
            || fm.contains("donnees") || fm.contains("data")
        {
            return vpb(cc);
        }

        
        if fm.contains("gpu") || fm.contains("compute") || fm.contains("cuda")
            || fm.contains("opencl") || fm.contains("calcul")
        {
            return oyt(cc);
        }

        
        if fm.contains("network") || fm.contains("internet") || fm.contains("reseau")
            || fm.contains("connect") || fm.contains("web")
        {
            return oyu(cc);
        }

        
        if fm.contains("usb") || fm.contains("peripheri") || fm.contains("device") {
            return vph(cc);
        }

        
        if fm.contains("audio") || fm.contains("sound") || fm.contains("son")
            || fm.contains("music") || fm.contains("musique")
        {
            return voz(cc);
        }

        
        if fm.contains("disk") || fm.contains("disque") || fm.contains("storage")
            || fm.contains("ssd") || fm.contains("nvme") || fm.contains("sata")
        {
            return vpf(cc);
        }
    }

    
    if fm.contains("what") || fm.contains("quel") || fm.contains("combien")
        || fm.contains("how much") || fm.contains("how many")
    {
        
        if fm.contains("ram") || fm.contains("memory") || fm.contains("memoire") {
            return vpc(cc);
        }

        
        if fm.contains("cpu") || fm.contains("processor") || fm.contains("processeur") {
            return vpa(cc);
        }

        
        if fm.contains("disk") || fm.contains("disque") || fm.contains("storage")
            || fm.contains("stockage")
        {
            return vpg(cc);
        }

        
        if fm.contains("encrypt") || fm.contains("chiffr") || fm.contains("crypt") {
            return oys(cc);
        }

        
        if fm.contains("pci") || fm.contains("device") || fm.contains("bus") {
            return vpd(cc);
        }
    }

    
    if fm.cj("is ") || fm.cj("are ") || fm.cj("do ")
        || fm.contains("est-ce") || fm.cj("y a")
    {
        if fm.contains("encrypt") || fm.contains("chiffr") || fm.contains("crypt") {
            return oys(cc);
        }
        if fm.contains("network") || fm.contains("reseau") || fm.contains("connect") {
            return oyu(cc);
        }
        if fm.contains("gpu") {
            return oyt(cc);
        }
    }

    
    if fm.contains("score") || fm.contains("capability") || fm.contains("capacite")
        || fm.contains("performance")
    {
        return vpe(cc);
    }

    
    Bl {
        yt: format!("I scanned: {} arch, {} cores, {}MB RAM, {} storage devices, {} partitions, {} encrypted volumes, score {:.0}%.",
            cc.arch, cc.azj,
            cc.ccf / (1024 * 1024),
            cc.aqm.len(),
            cc.aqd.len(),
            cc.avs.len(),
            cc.dkj * 100.0),
        azi: 0.5,
        bbd: None,
        yw: Vec::new(),
    }
}





fn vpb(cc: &T) -> Bl {
    let mut yw = Vec::new();

    
    if cc.aqm.is_empty() {
        return Bl {
            yt: String::from("No storage devices detected. I cannot access any disk data."),
            azi: 1.0,
            bbd: Some(false),
            yw: Vec::new(),
        };
    }

    yw.push(format!("{} storage device(s) detected", cc.aqm.len()));

    
    if cc.avs.is_empty() {
        return Bl {
            yt: format!(
                "No disk encryption detected on {} storage device(s). The data appears to be in cleartext — I can read it directly via the {} interface.",
                cc.aqm.len(),
                if cc.aqm.iter().any(|e| e.kk == super::probe::StorageKind::Xv) { "NVMe" }
                else if cc.aqm.iter().any(|e| e.kk == super::probe::StorageKind::Qr) { "AHCI/SATA" }
                else { "storage" }
            ),
            azi: 0.9,
            bbd: Some(true),
            yw,
        };
    }

    
    yw.push(format!("{} encrypted volume(s) detected", cc.avs.len()));

    let mut yt = String::new();
    let vae = false;

    for bdy in &cc.avs {
        yw.push(format!("[{}] {} — {}", bdy.ckf.as_str(), bdy.app, bdy.eu));

        match bdy.ckf {
            EncryptionType::Ajy | EncryptionType::Ajz => {
                yt.t(&format!(
                    "Disk '{}' has {} encryption. I can detect the volume but CANNOT decrypt without the passphrase/key. ",
                    bdy.app, bdy.ckf.as_str()));

                if cc.cfe {
                    yt.t("AES-NI hardware is available — decryption would be fast IF a key is provided. ");
                    yw.push(String::from("AES-NI hardware accelerator available for LUKS"));
                }
            }
            EncryptionType::Aaa => {
                yt.t(&format!(
                    "Disk '{}' has BitLocker encryption. I can detect the BDE header but CANNOT decrypt without the recovery key or TPM. ",
                    bdy.app));

                if cc.fqn > 0 {
                    yt.t("A crypto controller (possibly TPM) is on the PCI bus. ");
                    yw.push(String::from("PCI crypto/TPM controller detected"));
                }
            }
            EncryptionType::Afn => {
                yt.t(&format!(
                    "Disk '{}' appears to have a VeraCrypt volume. Cannot decrypt without the password. ",
                    bdy.app));
            }
            _ => {
                yt.t(&format!(
                    "Disk '{}' has {} encryption. Cannot access encrypted contents without credentials. ",
                    bdy.app, bdy.ckf.as_str()));
            }
        }
    }

    
    let mut gdu = Vec::new();
    if cc.cfe { gdu.push("AES-NI"); }
    if cc.ecm { gdu.push("SHA-EXT"); }
    if cc.crd { gdu.push("RDRAND"); }
    if cc.git { gdu.push("PCLMULQDQ"); }

    if !gdu.is_empty() {
        yt.t(&format!("\nHardware crypto available: {}. If a key is provided, I can attempt decryption at hardware speed.",
            gdu.rr(", ")));
    } else {
        yt.t("\nNo hardware crypto acceleration — software decryption would be slow.");
    }

    Bl {
        yt,
        azi: 0.95,
        bbd: Some(vae),
        yw,
    }
}

fn oyt(cc: &T) -> Bl {
    if cc.bqz {
        Bl {
            yt: format!("Yes! GPU detected: {} with {} MB VRAM and {} compute units. I can use it for parallel computation.",
                cc.beh, cc.dhr, cc.erk),
            azi: 1.0,
            bbd: Some(true),
            yw: Vec::new(),
        }
    } else {
        Bl {
            yt: String::from("No GPU detected. Only CPU compute available."),
            azi: 1.0,
            bbd: Some(false),
            yw: Vec::new(),
        }
    }
}

fn oyu(cc: &T) -> Bl {
    if cc.bzz && cc.aik {
        let mut yt = String::from("Yes, network is up and operational.");
        if let Some(ed) = cc.csg {
            yt.t(&format!(" MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]));
        }
        Bl {
            yt,
            azi: 1.0,
            bbd: Some(true),
            yw: Vec::new(),
        }
    } else if cc.bzz {
        Bl {
            yt: String::from("Network driver loaded but link is DOWN. Cable/WiFi issue."),
            azi: 0.9,
            bbd: Some(false),
            yw: Vec::new(),
        }
    } else {
        Bl {
            yt: format!("No network driver active. {} network PCI controllers found on bus.",
                cc.egg),
            azi: 1.0,
            bbd: Some(false),
            yw: Vec::new(),
        }
    }
}

fn vph(cc: &T) -> Bl {
    if cc.fav {
        Bl {
            yt: format!("USB is active: {} controller(s), {} device(s) detected.",
                cc.fxv, cc.cvc.len()),
            azi: 1.0,
            bbd: Some(true),
            yw: cc.cvc.iter()
                .map(|bc| format!("[{:04X}:{:04X}] {} ({})", bc.ml, bc.cgt, bc.baj, bc.bpz))
                .collect(),
        }
    } else {
        Bl {
            yt: format!("USB not initialized. {} USB controllers found on PCI bus.",
                cc.egh),
            azi: 0.9,
            bbd: Some(false),
            yw: Vec::new(),
        }
    }
}

fn voz(cc: &T) -> Bl {
    if cc.fki {
        Bl {
            yt: String::from("Audio (Intel HDA) is initialized and ready."),
            azi: 1.0,
            bbd: Some(true),
            yw: Vec::new(),
        }
    } else {
        Bl {
            yt: format!("Audio not initialized. {} audio PCI controllers found on bus.",
                cc.egf),
            azi: 0.9,
            bbd: Some(false),
            yw: Vec::new(),
        }
    }
}

fn vpf(cc: &T) -> Bl {
    if cc.aqm.is_empty() {
        return Bl {
            yt: String::from("No storage devices detected."),
            azi: 1.0,
            bbd: Some(false),
            yw: Vec::new(),
        };
    }

    let yw: Vec<String> = cc.aqm.iter()
        .map(|bc| format!("{} [{}] {} — {} GB", bc.j, bc.kk.as_str(), bc.model,
            bc.fei / (1024 * 1024 * 1024)))
        .collect();

    Bl {
        yt: format!("Yes, {} storage device(s) available with {} GB total. I have direct ring0 sector read access.",
            cc.aqm.len(),
            cc.dmp / (1024 * 1024 * 1024)),
        azi: 1.0,
        bbd: Some(true),
        yw,
    }
}

fn vpc(cc: &T) -> Bl {
    Bl {
        yt: format!("{} MB total RAM. Heap: {} KB used / {} KB free. {} page frames used, {} free.",
            cc.ccf / (1024 * 1024),
            cc.ecw / 1024, cc.erx / 1024,
            cc.ceu, cc.dhj),
        azi: 1.0,
        bbd: None,
        yw: Vec::new(),
    }
}

fn vpa(cc: &T) -> Bl {
    let simd = if cc.drm { "AVX-512" }
        else if cc.bzx { "AVX2" }
        else if cc.fke { "AVX" }
        else if cc.dro { "SSE2" }
        else { "none" };

    Bl {
        yt: format!("{} ({}) — {} cores, {} MHz, SIMD: {}, Crypto: AES-NI={} SHA={} RDRAND={}",
            cc.dpf, cc.avo,
            cc.azj, cc.fam / 1_000_000,
            simd, cc.cfe, cc.ecm, cc.crd),
        azi: 1.0,
        bbd: None,
        yw: Vec::new(),
    }
}

fn vpg(cc: &T) -> Bl {
    let mut yw = Vec::new();
    for ba in &cc.aqm {
        yw.push(format!("{} [{}] '{}' — {} GB", ba.j, ba.kk.as_str(),
            ba.model, ba.fei / (1024 * 1024 * 1024)));
    }
    for ai in &cc.aqd {
        yw.push(format!("  Partition #{} [{}] {} — {} GB{}",
            ai.aqb, ai.app, ai.ddc,
            ai.afz / (1024 * 1024 * 1024),
            if ai.cji { " (boot)" } else { "" }));
    }

    Bl {
        yt: format!("{} storage device(s), {} GB total, {} partition(s), {} encrypted.",
            cc.aqm.len(),
            cc.dmp / (1024 * 1024 * 1024),
            cc.aqd.len(),
            cc.avs.len()),
        azi: 1.0,
        bbd: None,
        yw,
    }
}

fn oys(cc: &T) -> Bl {
    if cc.avs.is_empty() {
        return Bl {
            yt: String::from("No disk encryption detected on any storage device."),
            azi: 0.85,
            bbd: None,
            yw: Vec::new(),
        };
    }

    let yw: Vec<String> = cc.avs.iter()
        .map(|aa| format!("[{}] {} — {}", aa.ckf.as_str(), aa.app, aa.eu))
        .collect();

    Bl {
        yt: format!("{} encrypted volume(s) detected.", cc.avs.len()),
        azi: 1.0,
        bbd: None,
        yw,
    }
}

fn vpd(cc: &T) -> Bl {
    Bl {
        yt: format!("{} PCI device(s): {} storage, {} network, {} USB, {} audio, {} display, {} bridge, {} crypto.",
            cc.dal, cc.ewl,
            cc.egg, cc.egh,
            cc.egf, cc.ewk,
            cc.hur, cc.fqn),
        azi: 1.0,
        bbd: None,
        yw: cc.hus.iter().take(10)
            .map(|bc| format!("{:02X}:{:02X}.{} [{:04X}:{:04X}] {} — {}",
                bc.aq, bc.de, bc.gw, bc.ml, bc.mx,
                bc.bpz, bc.bor))
            .collect(),
    }
}

fn vpe(cc: &T) -> Bl {
    Bl {
        yt: format!("Overall: {:.0}% — Compute: {:.0}%, Memory: {:.0}%, Storage: {:.0}%, Network: {:.0}%, Security: {:.0}%",
            cc.dkj * 100.0,
            cc.cwl * 100.0, cc.dte * 100.0,
            cc.ezb * 100.0, cc.evg * 100.0,
            cc.eyh * 100.0),
        azi: 1.0,
        bbd: None,
        yw: Vec::new(),
    }
}





pub fn svy(result: &Bl) -> String {
    let mut e = String::new();

    
    let kkj = if result.azi >= 0.9 { "\x01G[CERTAIN]\x01W" }
        else if result.azi >= 0.7 { "\x01Y[LIKELY]\x01W" }
        else { "\x01R[UNCERTAIN]\x01W" };

    
    if let Some(qvv) = result.bbd {
        if qvv {
            e.t(&format!("\x01G[YES]\x01W {} {}\n", kkj, result.yt));
        } else {
            e.t(&format!("\x01R[NO]\x01W {} {}\n", kkj, result.yt));
        }
    } else {
        e.t(&format!("{} {}\n", kkj, result.yt));
    }

    
    if !result.yw.is_empty() {
        e.t("\x01C  Evidence:\x01W\n");
        for eu in &result.yw {
            e.t(&format!("    • {}\n", eu));
        }
    }

    e
}
