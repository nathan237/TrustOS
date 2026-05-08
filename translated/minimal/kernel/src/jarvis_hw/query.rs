











use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::probe::{N, EncryptionType};


pub struct Am {
    pub answer: String,
    pub confidence: f32,      
    pub can_do: Option<bool>, 
    pub details: Vec<String>, 
}


pub fn jwi(query: &str, ai: &N) -> Am {
    let q = query.to_ascii_lowercase();

    
    if q.contains("can you") || q.contains("peux tu") || q.contains("peut tu")
        || q.contains("est-ce que") || q.contains("is it possible")
    {
        
        if q.contains("encrypt") || q.contains("chiffr") || q.contains("crypt")
            || q.contains("donnees") || q.contains("data")
        {
            return oah(ai);
        }

        
        if q.contains("gpu") || q.contains("compute") || q.contains("cuda")
            || q.contains("opencl") || q.contains("calcul")
        {
            return ixh(ai);
        }

        
        if q.contains("network") || q.contains("internet") || q.contains("reseau")
            || q.contains("connect") || q.contains("web")
        {
            return ixi(ai);
        }

        
        if q.contains("usb") || q.contains("peripheri") || q.contains("device") {
            return oao(ai);
        }

        
        if q.contains("audio") || q.contains("sound") || q.contains("son")
            || q.contains("music") || q.contains("musique")
        {
            return oaf(ai);
        }

        
        if q.contains("disk") || q.contains("disque") || q.contains("storage")
            || q.contains("ssd") || q.contains("nvme") || q.contains("sata")
        {
            return oal(ai);
        }
    }

    
    if q.contains("what") || q.contains("quel") || q.contains("combien")
        || q.contains("how much") || q.contains("how many")
    {
        
        if q.contains("ram") || q.contains("memory") || q.contains("memoire") {
            return oai(ai);
        }

        
        if q.contains("cpu") || q.contains("processor") || q.contains("processeur") {
            return oag(ai);
        }

        
        if q.contains("disk") || q.contains("disque") || q.contains("storage")
            || q.contains("stockage")
        {
            return oan(ai);
        }

        
        if q.contains("encrypt") || q.contains("chiffr") || q.contains("crypt") {
            return ixg(ai);
        }

        
        if q.contains("pci") || q.contains("device") || q.contains("bus") {
            return oaj(ai);
        }
    }

    
    if q.starts_with("is ") || q.starts_with("are ") || q.starts_with("do ")
        || q.contains("est-ce") || q.starts_with("y a")
    {
        if q.contains("encrypt") || q.contains("chiffr") || q.contains("crypt") {
            return ixg(ai);
        }
        if q.contains("network") || q.contains("reseau") || q.contains("connect") {
            return ixi(ai);
        }
        if q.contains("gpu") {
            return ixh(ai);
        }
    }

    
    if q.contains("score") || q.contains("capability") || q.contains("capacite")
        || q.contains("performance")
    {
        return oak(ai);
    }

    
    Am {
        answer: format!("I scanned: {} arch, {} cores, {}MB RAM, {} storage devices, {} partitions, {} encrypted volumes, score {:.0}%.",
            ai.arch, ai.cpu_cores,
            ai.total_ram_bytes / (1024 * 1024),
            ai.storage_devices.len(),
            ai.partitions.len(),
            ai.encryption_detected.len(),
            ai.overall_score * 100.0),
        confidence: 0.5,
        can_do: None,
        details: Vec::new(),
    }
}





fn oah(ai: &N) -> Am {
    let mut details = Vec::new();

    
    if ai.storage_devices.is_empty() {
        return Am {
            answer: String::from("No storage devices detected. I cannot access any disk data."),
            confidence: 1.0,
            can_do: Some(false),
            details: Vec::new(),
        };
    }

    details.push(format!("{} storage device(s) detected", ai.storage_devices.len()));

    
    if ai.encryption_detected.is_empty() {
        return Am {
            answer: format!(
                "No disk encryption detected on {} storage device(s). The data appears to be in cleartext — I can read it directly via the {} interface.",
                ai.storage_devices.len(),
                if ai.storage_devices.iter().any(|j| j.kind == super::probe::StorageKind::Nvme) { "NVMe" }
                else if ai.storage_devices.iter().any(|j| j.kind == super::probe::StorageKind::Sata) { "AHCI/SATA" }
                else { "storage" }
            ),
            confidence: 0.9,
            can_do: Some(true),
            details,
        };
    }

    
    details.push(format!("{} encrypted volume(s) detected", ai.encryption_detected.len()));

    let mut answer = String::new();
    let nou = false;

    for enc in &ai.encryption_detected {
        details.push(format!("[{}] {} — {}", enc.encryption_type.as_str(), enc.disk_name, enc.detail));

        match enc.encryption_type {
            EncryptionType::Luks1 | EncryptionType::Luks2 => {
                answer.push_str(&format!(
                    "Disk '{}' has {} encryption. I can detect the volume but CANNOT decrypt without the passphrase/key. ",
                    enc.disk_name, enc.encryption_type.as_str()));

                if ai.has_aesni {
                    answer.push_str("AES-NI hardware is available — decryption would be fast IF a key is provided. ");
                    details.push(String::from("AES-NI hardware accelerator available for LUKS"));
                }
            }
            EncryptionType::BitLocker => {
                answer.push_str(&format!(
                    "Disk '{}' has BitLocker encryption. I can detect the BDE header but CANNOT decrypt without the recovery key or TPM. ",
                    enc.disk_name));

                if ai.pci_crypto_controllers > 0 {
                    answer.push_str("A crypto controller (possibly TPM) is on the PCI bus. ");
                    details.push(String::from("PCI crypto/TPM controller detected"));
                }
            }
            EncryptionType::VeraCrypt => {
                answer.push_str(&format!(
                    "Disk '{}' appears to have a VeraCrypt volume. Cannot decrypt without the password. ",
                    enc.disk_name));
            }
            _ => {
                answer.push_str(&format!(
                    "Disk '{}' has {} encryption. Cannot access encrypted contents without credentials. ",
                    enc.disk_name, enc.encryption_type.as_str()));
            }
        }
    }

    
    let mut cvy = Vec::new();
    if ai.has_aesni { cvy.push("AES-NI"); }
    if ai.has_sha_ext { cvy.push("SHA-EXT"); }
    if ai.has_rdrand { cvy.push("RDRAND"); }
    if ai.has_pclmulqdq { cvy.push("PCLMULQDQ"); }

    if !cvy.is_empty() {
        answer.push_str(&format!("\nHardware crypto available: {}. If a key is provided, I can attempt decryption at hardware speed.",
            cvy.join(", ")));
    } else {
        answer.push_str("\nNo hardware crypto acceleration — software decryption would be slow.");
    }

    Am {
        answer,
        confidence: 0.95,
        can_do: Some(nou),
        details,
    }
}

fn ixh(ai: &N) -> Am {
    if ai.has_gpu {
        Am {
            answer: format!("Yes! GPU detected: {} with {} MB VRAM and {} compute units. I can use it for parallel computation.",
                ai.gpu_name, ai.gpu_vram_mb, ai.gpu_compute_units),
            confidence: 1.0,
            can_do: Some(true),
            details: Vec::new(),
        }
    } else {
        Am {
            answer: String::from("No GPU detected. Only CPU compute available."),
            confidence: 1.0,
            can_do: Some(false),
            details: Vec::new(),
        }
    }
}

fn ixi(ai: &N) -> Am {
    if ai.has_network && ai.link_up {
        let mut answer = String::from("Yes, network is up and operational.");
        if let Some(mac) = ai.mac_address {
            answer.push_str(&format!(" MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
        }
        Am {
            answer,
            confidence: 1.0,
            can_do: Some(true),
            details: Vec::new(),
        }
    } else if ai.has_network {
        Am {
            answer: String::from("Network driver loaded but link is DOWN. Cable/WiFi issue."),
            confidence: 0.9,
            can_do: Some(false),
            details: Vec::new(),
        }
    } else {
        Am {
            answer: format!("No network driver active. {} network PCI controllers found on bus.",
                ai.pci_network_controllers),
            confidence: 1.0,
            can_do: Some(false),
            details: Vec::new(),
        }
    }
}

fn oao(ai: &N) -> Am {
    if ai.usb_initialized {
        Am {
            answer: format!("USB is active: {} controller(s), {} device(s) detected.",
                ai.usb_controller_count, ai.usb_devices.len()),
            confidence: 1.0,
            can_do: Some(true),
            details: ai.usb_devices.iter()
                .map(|d| format!("[{:04X}:{:04X}] {} ({})", d.vendor_id, d.product_id, d.product, d.class_name))
                .collect(),
        }
    } else {
        Am {
            answer: format!("USB not initialized. {} USB controllers found on PCI bus.",
                ai.pci_usb_controllers),
            confidence: 0.9,
            can_do: Some(false),
            details: Vec::new(),
        }
    }
}

fn oaf(ai: &N) -> Am {
    if ai.hda_initialized {
        Am {
            answer: String::from("Audio (Intel HDA) is initialized and ready."),
            confidence: 1.0,
            can_do: Some(true),
            details: Vec::new(),
        }
    } else {
        Am {
            answer: format!("Audio not initialized. {} audio PCI controllers found on bus.",
                ai.pci_audio_controllers),
            confidence: 0.9,
            can_do: Some(false),
            details: Vec::new(),
        }
    }
}

fn oal(ai: &N) -> Am {
    if ai.storage_devices.is_empty() {
        return Am {
            answer: String::from("No storage devices detected."),
            confidence: 1.0,
            can_do: Some(false),
            details: Vec::new(),
        };
    }

    let details: Vec<String> = ai.storage_devices.iter()
        .map(|d| format!("{} [{}] {} — {} GB", d.name, d.kind.as_str(), d.model,
            d.capacity_bytes / (1024 * 1024 * 1024)))
        .collect();

    Am {
        answer: format!("Yes, {} storage device(s) available with {} GB total. I have direct ring0 sector read access.",
            ai.storage_devices.len(),
            ai.total_storage_bytes / (1024 * 1024 * 1024)),
        confidence: 1.0,
        can_do: Some(true),
        details,
    }
}

fn oai(ai: &N) -> Am {
    Am {
        answer: format!("{} MB total RAM. Heap: {} KB used / {} KB free. {} page frames used, {} free.",
            ai.total_ram_bytes / (1024 * 1024),
            ai.heap_used_bytes / 1024, ai.heap_free_bytes / 1024,
            ai.frames_used, ai.frames_free),
        confidence: 1.0,
        can_do: None,
        details: Vec::new(),
    }
}

fn oag(ai: &N) -> Am {
    let simd = if ai.has_avx512 { "AVX-512" }
        else if ai.has_avx2 { "AVX2" }
        else if ai.has_avx { "AVX" }
        else if ai.has_sse2 { "SSE2" }
        else { "none" };

    Am {
        answer: format!("{} ({}) — {} cores, {} MHz, SIMD: {}, Crypto: AES-NI={} SHA={} RDRAND={}",
            ai.cpu_brand, ai.cpu_vendor,
            ai.cpu_cores, ai.tsc_freq_hz / 1_000_000,
            simd, ai.has_aesni, ai.has_sha_ext, ai.has_rdrand),
        confidence: 1.0,
        can_do: None,
        details: Vec::new(),
    }
}

fn oan(ai: &N) -> Am {
    let mut details = Vec::new();
    for s in &ai.storage_devices {
        details.push(format!("{} [{}] '{}' — {} GB", s.name, s.kind.as_str(),
            s.model, s.capacity_bytes / (1024 * 1024 * 1024)));
    }
    for aa in &ai.partitions {
        details.push(format!("  Partition #{} [{}] {} — {} GB{}",
            aa.number, aa.disk_name, aa.type_name,
            aa.size_bytes / (1024 * 1024 * 1024),
            if aa.bootable { " (boot)" } else { "" }));
    }

    Am {
        answer: format!("{} storage device(s), {} GB total, {} partition(s), {} encrypted.",
            ai.storage_devices.len(),
            ai.total_storage_bytes / (1024 * 1024 * 1024),
            ai.partitions.len(),
            ai.encryption_detected.len()),
        confidence: 1.0,
        can_do: None,
        details,
    }
}

fn ixg(ai: &N) -> Am {
    if ai.encryption_detected.is_empty() {
        return Am {
            answer: String::from("No disk encryption detected on any storage device."),
            confidence: 0.85,
            can_do: None,
            details: Vec::new(),
        };
    }

    let details: Vec<String> = ai.encryption_detected.iter()
        .map(|e| format!("[{}] {} — {}", e.encryption_type.as_str(), e.disk_name, e.detail))
        .collect();

    Am {
        answer: format!("{} encrypted volume(s) detected.", ai.encryption_detected.len()),
        confidence: 1.0,
        can_do: None,
        details,
    }
}

fn oaj(ai: &N) -> Am {
    Am {
        answer: format!("{} PCI device(s): {} storage, {} network, {} USB, {} audio, {} display, {} bridge, {} crypto.",
            ai.pci_device_count, ai.pci_storage_controllers,
            ai.pci_network_controllers, ai.pci_usb_controllers,
            ai.pci_audio_controllers, ai.pci_display_controllers,
            ai.pci_bridge_count, ai.pci_crypto_controllers),
        confidence: 1.0,
        can_do: None,
        details: ai.pci_devices.iter().take(10)
            .map(|d| format!("{:02X}:{:02X}.{} [{:04X}:{:04X}] {} — {}",
                d.bus, d.device, d.function, d.vendor_id, d.device_id,
                d.class_name, d.subclass_name))
            .collect(),
    }
}

fn oak(ai: &N) -> Am {
    Am {
        answer: format!("Overall: {:.0}% — Compute: {:.0}%, Memory: {:.0}%, Storage: {:.0}%, Network: {:.0}%, Security: {:.0}%",
            ai.overall_score * 100.0,
            ai.compute_score * 100.0, ai.memory_score * 100.0,
            ai.storage_score * 100.0, ai.network_score * 100.0,
            ai.security_score * 100.0),
        confidence: 1.0,
        can_do: None,
        details: Vec::new(),
    }
}





pub fn lxq(result: &Am) -> String {
    let mut j = String::new();

    
    let foa = if result.confidence >= 0.9 { "\x01G[CERTAIN]\x01W" }
        else if result.confidence >= 0.7 { "\x01Y[LIKELY]\x01W" }
        else { "\x01R[UNCERTAIN]\x01W" };

    
    if let Some(can) = result.can_do {
        if can {
            j.push_str(&format!("\x01G[YES]\x01W {} {}\n", foa, result.answer));
        } else {
            j.push_str(&format!("\x01R[NO]\x01W {} {}\n", foa, result.answer));
        }
    } else {
        j.push_str(&format!("{} {}\n", foa, result.answer));
    }

    
    if !result.details.is_empty() {
        j.push_str("\x01C  Evidence:\x01W\n");
        for detail in &result.details {
            j.push_str(&format!("    • {}\n", detail));
        }
    }

    j
}
