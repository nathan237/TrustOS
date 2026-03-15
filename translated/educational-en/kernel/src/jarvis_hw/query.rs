//! JARVIS Hardware Query Engine — Natural language hardware reasoning
//!
//! Answers questions like:
//!   "can you access the encrypted data on this disk?"
//!   "do we have AES hardware?"
//!   "how much RAM is available?"
//!   "is the network up?"
//!   "what encryption is on the disk?"
//!
//! This module does pattern-matching + hardware context to give Jarvis
//! the ability to reason about physical capabilities in real-time.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::probe::{HardwareProfile, EncryptionType};

/// Result of a hardware query
pub struct QueryResult {
    pub answer: String,
    pub confidence: f32,      // 0.0 = guess, 1.0 = certain
    pub can_do: Option<bool>, // Some(true/false) for "can you" questions, None for info queries
    pub details: Vec<String>, // supporting evidence
}

/// Process a natural language hardware query
pub fn answer_query(query: &str, profile: &HardwareProfile) -> QueryResult {
    let q = query.to_ascii_lowercase();

    // ── "can you access / read / decrypt" patterns ──────────────────────────
    if q.contains("can you") || q.contains("peux tu") || q.contains("peut tu")
        || q.contains("est-ce que") || q.contains("is it possible")
    {
        // Encrypted disk access
        if q.contains("encrypt") || q.contains("chiffr") || q.contains("crypt")
            || q.contains("donnees") || q.contains("data")
        {
            return query_encrypted_access(profile);
        }

        // GPU compute
        if q.contains("gpu") || q.contains("compute") || q.contains("cuda")
            || q.contains("opencl") || q.contains("calcul")
        {
            return query_gpu_access(profile);
        }

        // Network
        if q.contains("network") || q.contains("internet") || q.contains("reseau")
            || q.contains("connect") || q.contains("web")
        {
            return query_network_access(profile);
        }

        // USB
        if q.contains("usb") || q.contains("peripheri") || q.contains("device") {
            return query_usb_access(profile);
        }

        // Audio
        if q.contains("audio") || q.contains("sound") || q.contains("son")
            || q.contains("music") || q.contains("musique")
        {
            return query_audio_access(profile);
        }

        // Generic disk/storage
        if q.contains("disk") || q.contains("disque") || q.contains("storage")
            || q.contains("ssd") || q.contains("nvme") || q.contains("sata")
        {
            return query_storage_access(profile);
        }
    }

    // ── "what / quel / combien" info queries ────────────────────────────────
    if q.contains("what") || q.contains("quel") || q.contains("combien")
        || q.contains("how much") || q.contains("how many")
    {
        // RAM info
        if q.contains("ram") || q.contains("memory") || q.contains("memoire") {
            return query_memory_information(profile);
        }

        // CPU info
        if q.contains("cpu") || q.contains("processor") || q.contains("processeur") {
            return query_cpu_information(profile);
        }

        // Disk/storage info
        if q.contains("disk") || q.contains("disque") || q.contains("storage")
            || q.contains("stockage")
        {
            return query_storage_information(profile);
        }

        // Encryption info
        if q.contains("encrypt") || q.contains("chiffr") || q.contains("crypt") {
            return query_encryption_information(profile);
        }

        // PCI info
        if q.contains("pci") || q.contains("device") || q.contains("bus") {
            return query_pci_information(profile);
        }
    }

    // ── "is / est-ce que" boolean queries ───────────────────────────────────
    if q.starts_with("is ") || q.starts_with("are ") || q.starts_with("do ")
        || q.contains("est-ce") || q.starts_with("y a")
    {
        if q.contains("encrypt") || q.contains("chiffr") || q.contains("crypt") {
            return query_encryption_information(profile);
        }
        if q.contains("network") || q.contains("reseau") || q.contains("connect") {
            return query_network_access(profile);
        }
        if q.contains("gpu") {
            return query_gpu_access(profile);
        }
    }

    // ── Score/capability queries ────────────────────────────────────────────
    if q.contains("score") || q.contains("capability") || q.contains("capacite")
        || q.contains("performance")
    {
        return query_scores(profile);
    }

    // ── Fallback: try to give a useful summary ──────────────────────────────
    QueryResult {
        answer: format!("I scanned: {} arch, {} cores, {}MB RAM, {} storage devices, {} partitions, {} encrypted volumes, score {:.0}%.",
            profile.arch, profile.cpu_cores,
            profile.total_ram_bytes / (1024 * 1024),
            profile.storage_devices.len(),
            profile.partitions.len(),
            profile.encryption_detected.len(),
            profile.overall_score * 100.0),
        confidence: 0.5,
        can_do: None,
        details: Vec::new(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Query handlers
// ═══════════════════════════════════════════════════════════════════════════════

fn query_encrypted_access(profile: &HardwareProfile) -> QueryResult {
    let mut details = Vec::new();

    // Step 1: Do we even have storage?
    if profile.storage_devices.is_empty() {
        return QueryResult {
            answer: String::from("No storage devices detected. I cannot access any disk data."),
            confidence: 1.0,
            can_do: Some(false),
            details: Vec::new(),
        };
    }

    details.push(format!("{} storage device(s) detected", profile.storage_devices.len()));

    // Step 2: Is there encryption?
    if profile.encryption_detected.is_empty() {
        return QueryResult {
            answer: format!(
                "No disk encryption detected on {} storage device(s). The data appears to be in cleartext — I can read it directly via the {} interface.",
                profile.storage_devices.len(),
                if profile.storage_devices.iter().any(|s| s.kind == super::probe::StorageKind::Nvme) { "NVMe" }
                else if profile.storage_devices.iter().any(|s| s.kind == super::probe::StorageKind::Sata) { "AHCI/SATA" }
                else { "storage" }
            ),
            confidence: 0.9,
            can_do: Some(true),
            details,
        };
    }

    // Step 3: Encryption found — analyze feasibility
    details.push(format!("{} encrypted volume(s) detected", profile.encryption_detected.len()));

    let mut answer = String::new();
    let overall_can = false;

    for encrypt in &profile.encryption_detected {
        details.push(format!("[{}] {} — {}", encrypt.encryption_type.as_str(), encrypt.disk_name, encrypt.detail));

                // Pattern matching — Rust's exhaustive branching construct.
match encrypt.encryption_type {
            EncryptionType::Luks1 | EncryptionType::Luks2 => {
                answer.push_str(&format!(
                    "Disk '{}' has {} encryption. I can detect the volume but CANNOT decrypt without the passphrase/key. ",
                    encrypt.disk_name, encrypt.encryption_type.as_str()));

                if profile.has_aesni {
                    answer.push_str("AES-NI hardware is available — decryption would be fast IF a key is provided. ");
                    details.push(String::from("AES-NI hardware accelerator available for LUKS"));
                }
            }
            EncryptionType::BitLocker => {
                answer.push_str(&format!(
                    "Disk '{}' has BitLocker encryption. I can detect the BDE header but CANNOT decrypt without the recovery key or TPM. ",
                    encrypt.disk_name));

                if profile.pci_crypto_controllers > 0 {
                    answer.push_str("A crypto controller (possibly TPM) is on the PCI bus. ");
                    details.push(String::from("PCI crypto/TPM controller detected"));
                }
            }
            EncryptionType::VeraCrypt => {
                answer.push_str(&format!(
                    "Disk '{}' appears to have a VeraCrypt volume. Cannot decrypt without the password. ",
                    encrypt.disk_name));
            }
            _ => {
                answer.push_str(&format!(
                    "Disk '{}' has {} encryption. Cannot access encrypted contents without credentials. ",
                    encrypt.disk_name, encrypt.encryption_type.as_str()));
            }
        }
    }

    // Add crypto capability assessment
    let mut crypto_caps = Vec::new();
    if profile.has_aesni { crypto_caps.push("AES-NI"); }
    if profile.has_sha_ext { crypto_caps.push("SHA-EXT"); }
    if profile.has_rdrand { crypto_caps.push("RDRAND"); }
    if profile.has_pclmulqdq { crypto_caps.push("PCLMULQDQ"); }

    if !crypto_caps.is_empty() {
        answer.push_str(&format!("\nHardware crypto available: {}. If a key is provided, I can attempt decryption at hardware speed.",
            crypto_caps.join(", ")));
    } else {
        answer.push_str("\nNo hardware crypto acceleration — software decryption would be slow.");
    }

    QueryResult {
        answer,
        confidence: 0.95,
        can_do: Some(overall_can),
        details,
    }
}

fn query_gpu_access(profile: &HardwareProfile) -> QueryResult {
    if profile.has_gpu {
        QueryResult {
            answer: format!("Yes! GPU detected: {} with {} MB VRAM and {} compute units. I can use it for parallel computation.",
                profile.gpu_name, profile.gpu_vram_mb, profile.gpu_compute_units),
            confidence: 1.0,
            can_do: Some(true),
            details: Vec::new(),
        }
    } else {
        QueryResult {
            answer: String::from("No GPU detected. Only CPU compute available."),
            confidence: 1.0,
            can_do: Some(false),
            details: Vec::new(),
        }
    }
}

fn query_network_access(profile: &HardwareProfile) -> QueryResult {
    if profile.has_network && profile.link_up {
        let mut answer = String::from("Yes, network is up and operational.");
        if let Some(mac) = profile.mac_address {
            answer.push_str(&format!(" MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
        }
        QueryResult {
            answer,
            confidence: 1.0,
            can_do: Some(true),
            details: Vec::new(),
        }
    } else if profile.has_network {
        QueryResult {
            answer: String::from("Network driver loaded but link is DOWN. Cable/WiFi issue."),
            confidence: 0.9,
            can_do: Some(false),
            details: Vec::new(),
        }
    } else {
        QueryResult {
            answer: format!("No network driver active. {} network PCI controllers found on bus.",
                profile.pci_network_controllers),
            confidence: 1.0,
            can_do: Some(false),
            details: Vec::new(),
        }
    }
}

fn query_usb_access(profile: &HardwareProfile) -> QueryResult {
    if profile.usb_initialized {
        QueryResult {
            answer: format!("USB is active: {} controller(s), {} device(s) detected.",
                profile.usb_controller_count, profile.usb_devices.len()),
            confidence: 1.0,
            can_do: Some(true),
            details: profile.usb_devices.iter()
                .map(|d| format!("[{:04X}:{:04X}] {} ({})", d.vendor_id, d.product_id, d.product, d.class_name))
                .collect(),
        }
    } else {
        QueryResult {
            answer: format!("USB not initialized. {} USB controllers found on PCI bus.",
                profile.pci_usb_controllers),
            confidence: 0.9,
            can_do: Some(false),
            details: Vec::new(),
        }
    }
}

fn query_audio_access(profile: &HardwareProfile) -> QueryResult {
    if profile.hda_initialized {
        QueryResult {
            answer: String::from("Audio (Intel HDA) is initialized and ready."),
            confidence: 1.0,
            can_do: Some(true),
            details: Vec::new(),
        }
    } else {
        QueryResult {
            answer: format!("Audio not initialized. {} audio PCI controllers found on bus.",
                profile.pci_audio_controllers),
            confidence: 0.9,
            can_do: Some(false),
            details: Vec::new(),
        }
    }
}

fn query_storage_access(profile: &HardwareProfile) -> QueryResult {
    if profile.storage_devices.is_empty() {
        return QueryResult {
            answer: String::from("No storage devices detected."),
            confidence: 1.0,
            can_do: Some(false),
            details: Vec::new(),
        };
    }

    let details: Vec<String> = profile.storage_devices.iter()
        .map(|d| format!("{} [{}] {} — {} GB", d.name, d.kind.as_str(), d.model,
            d.capacity_bytes / (1024 * 1024 * 1024)))
        .collect();

    QueryResult {
        answer: format!("Yes, {} storage device(s) available with {} GB total. I have direct ring0 sector read access.",
            profile.storage_devices.len(),
            profile.total_storage_bytes / (1024 * 1024 * 1024)),
        confidence: 1.0,
        can_do: Some(true),
        details,
    }
}

fn query_memory_information(profile: &HardwareProfile) -> QueryResult {
    QueryResult {
        answer: format!("{} MB total RAM. Heap: {} KB used / {} KB free. {} page frames used, {} free.",
            profile.total_ram_bytes / (1024 * 1024),
            profile.heap_used_bytes / 1024, profile.heap_free_bytes / 1024,
            profile.frames_used, profile.frames_free),
        confidence: 1.0,
        can_do: None,
        details: Vec::new(),
    }
}

fn query_cpu_information(profile: &HardwareProfile) -> QueryResult {
    let simd = if profile.has_avx512 { "AVX-512" }
        else if profile.has_avx2 { "AVX2" }
        else if profile.has_avx { "AVX" }
        else if profile.has_sse2 { "SSE2" }
        else { "none" };

    QueryResult {
        answer: format!("{} ({}) — {} cores, {} MHz, SIMD: {}, Crypto: AES-NI={} SHA={} RDRAND={}",
            profile.cpu_brand, profile.cpu_vendor,
            profile.cpu_cores, profile.tsc_frequency_hz / 1_000_000,
            simd, profile.has_aesni, profile.has_sha_ext, profile.has_rdrand),
        confidence: 1.0,
        can_do: None,
        details: Vec::new(),
    }
}

fn query_storage_information(profile: &HardwareProfile) -> QueryResult {
    let mut details = Vec::new();
    for device in &profile.storage_devices {
        details.push(format!("{} [{}] '{}' — {} GB", device.name, device.kind.as_str(),
            device.model, device.capacity_bytes / (1024 * 1024 * 1024)));
    }
    for p in &profile.partitions {
        details.push(format!("  Partition #{} [{}] {} — {} GB{}",
            p.number, p.disk_name, p.type_name,
            p.size_bytes / (1024 * 1024 * 1024),
            if p.bootable { " (boot)" } else { "" }));
    }

    QueryResult {
        answer: format!("{} storage device(s), {} GB total, {} partition(s), {} encrypted.",
            profile.storage_devices.len(),
            profile.total_storage_bytes / (1024 * 1024 * 1024),
            profile.partitions.len(),
            profile.encryption_detected.len()),
        confidence: 1.0,
        can_do: None,
        details,
    }
}

fn query_encryption_information(profile: &HardwareProfile) -> QueryResult {
    if profile.encryption_detected.is_empty() {
        return QueryResult {
            answer: String::from("No disk encryption detected on any storage device."),
            confidence: 0.85,
            can_do: None,
            details: Vec::new(),
        };
    }

    let details: Vec<String> = profile.encryption_detected.iter()
        .map(|e| format!("[{}] {} — {}", e.encryption_type.as_str(), e.disk_name, e.detail))
        .collect();

    QueryResult {
        answer: format!("{} encrypted volume(s) detected.", profile.encryption_detected.len()),
        confidence: 1.0,
        can_do: None,
        details,
    }
}

fn query_pci_information(profile: &HardwareProfile) -> QueryResult {
    QueryResult {
        answer: format!("{} PCI device(s): {} storage, {} network, {} USB, {} audio, {} display, {} bridge, {} crypto.",
            profile.pci_device_count, profile.pci_storage_controllers,
            profile.pci_network_controllers, profile.pci_usb_controllers,
            profile.pci_audio_controllers, profile.pci_display_controllers,
            profile.pci_bridge_count, profile.pci_crypto_controllers),
        confidence: 1.0,
        can_do: None,
        details: profile.pci_devices.iter().take(10)
            .map(|d| format!("{:02X}:{:02X}.{} [{:04X}:{:04X}] {} — {}",
                d.bus, d.device, d.function, d.vendor_id, d.device_id,
                d.class_name, d.subclass_name))
            .collect(),
    }
}

fn query_scores(profile: &HardwareProfile) -> QueryResult {
    QueryResult {
        answer: format!("Overall: {:.0}% — Compute: {:.0}%, Memory: {:.0}%, Storage: {:.0}%, Network: {:.0}%, Security: {:.0}%",
            profile.overall_score * 100.0,
            profile.compute_score * 100.0, profile.memory_score * 100.0,
            profile.storage_score * 100.0, profile.network_score * 100.0,
            profile.security_score * 100.0),
        confidence: 1.0,
        can_do: None,
        details: Vec::new(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Format a QueryResult for terminal display
// ═══════════════════════════════════════════════════════════════════════════════

pub fn format_query_result(result: &QueryResult) -> String {
    let mut s = String::new();

    // Confidence indicator
    let configuration_icon = if result.confidence >= 0.9 { "\x01G[CERTAIN]\x01W" }
        else if result.confidence >= 0.7 { "\x01Y[LIKELY]\x01W" }
        else { "\x01R[UNCERTAIN]\x01W" };

    // Can-do indicator
    if let Some(can) = result.can_do {
        if can {
            s.push_str(&format!("\x01G[YES]\x01W {} {}\n", configuration_icon, result.answer));
        } else {
            s.push_str(&format!("\x01R[NO]\x01W {} {}\n", configuration_icon, result.answer));
        }
    } else {
        s.push_str(&format!("{} {}\n", configuration_icon, result.answer));
    }

    // Details
    if !result.details.is_empty() {
        s.push_str("\x01C  Evidence:\x01W\n");
        for detail in &result.details {
            s.push_str(&format!("    • {}\n", detail));
        }
    }

    s
}
