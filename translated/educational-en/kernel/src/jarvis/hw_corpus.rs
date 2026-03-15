//! Hardware-Generated Training Corpus
//!
//! Converts live hardware probe data into training sequences for Jarvis.
//! Each boot produces unique Q&A pairs based on the actual detected hardware,
//! teaching Jarvis to understand and reason about any hardware configuration.
//!
//! Generates ~80-120 sequences per boot covering:
//! - CPU identification & capabilities
//! - Memory layout & capacity
//! - PCI device inventory
//! - Storage & encryption status
//! - Network, GPU, USB, audio
//! - ACPI/firmware details
//! - Security features
//! - Performance scoring
//! - Contextual reasoning chains

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::jarvis_hw::probe::HardwareProfile;

/// Generate all hardware-based training sequences from a probe snapshot.
/// Returns a Vec of training strings (Q&A pairs, facts, reasoning chains).
pub fn generate(profile: &HardwareProfile) -> Vec<String> {
    let mut seqs = Vec::with_capacity(120);

    cpu_sequences(profile, &mut seqs);
    memory_sequences(profile, &mut seqs);
    pci_sequences(profile, &mut seqs);
    storage_sequences(profile, &mut seqs);
    encryption_sequences(profile, &mut seqs);
    network_sequences(profile, &mut seqs);
    gpu_sequences(profile, &mut seqs);
    usb_sequences(profile, &mut seqs);
    timer_sequences(profile, &mut seqs);
    security_sequences(profile, &mut seqs);
    acpi_sequences(profile, &mut seqs);
    score_sequences(profile, &mut seqs);
    reasoning_chains(profile, &mut seqs);

    seqs
}

// ═══════════════════════════════════════════════════════════════════════════════
// CPU
// ═══════════════════════════════════════════════════════════════════════════════

fn cpu_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    // Identity
    s.push(format!("Q: What CPU is installed? A: {}", p.cpu_brand));
    s.push(format!("Q: Quel processeur? A: {}", p.cpu_brand));
    s.push(format!("Q: CPU vendor? A: {}", p.cpu_vendor));
    s.push(format!("Q: How many CPU cores? A: {} logical, {} physical",
        p.maximum_logical_cpus, p.maximum_physical_cpus));
    s.push(format!("Q: Combien de coeurs? A: {} logiques, {} physiques",
        p.maximum_logical_cpus, p.maximum_physical_cpus));
    s.push(format!("Q: CPU family/model? A: family {} model {} stepping {}",
        p.cpu_family, p.cpu_model, p.cpu_stepping));

    // TSC
    if p.tsc_frequency_hz > 0 {
        let mhz = p.tsc_frequency_hz / 1_000_000;
        s.push(format!("Q: CPU frequency? A: ~{} MHz (TSC)", mhz));
        s.push(format!("Q: Quelle frequence CPU? A: ~{} MHz", mhz));
    }

    // SIMD capabilities
    let simd = simd_summary(p);
    s.push(format!("Q: What SIMD is available? A: {}", simd));
    s.push(format!("Q: SIMD disponible? A: {}", simd));

    // AVX2 specific (important for Jarvis self-awareness)
    if p.has_avx2 {
        s.push(format!("My SIMD kernels use AVX2+FMA: 8-wide float ops, 2-3x faster than SSE2."));
    } else {
        s.push(format!("My SIMD kernels use SSE2: 4-wide float ops. No AVX2 available."));
    }

    // Crypto
    let crypto = crypto_summary(p);
    if !crypto.is_empty() {
        s.push(format!("Q: Hardware crypto? A: {}", crypto));
    }

    // Virtualization
    if p.has_vmx {
        s.push(format!("Q: Virtualization? A: Intel VT-x supported"));
    }
    if p.has_svm {
        s.push(format!("Q: Virtualization? A: AMD-V supported"));
    }
}

fn simd_summary(p: &HardwareProfile) -> String {
    let mut parts = Vec::new();
    if p.has_sse { parts.push("SSE"); }
    if p.has_sse2 { parts.push("SSE2"); }
    if p.has_sse3 { parts.push("SSE3"); }
    if p.has_sse4_2 { parts.push("SSE4.2"); }
    if p.has_avx { parts.push("AVX"); }
    if p.has_avx2 { parts.push("AVX2"); }
    if p.has_avx512 { parts.push("AVX-512"); }
    if parts.is_empty() { return String::from("none"); }
    let mut s = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 { s.push_str(", "); }
        s.push_str(part);
    }
    s
}

fn crypto_summary(p: &HardwareProfile) -> String {
    let mut parts = Vec::new();
    if p.has_aesni { parts.push("AES-NI"); }
    if p.has_sha_ext { parts.push("SHA-NI"); }
    if p.has_rdrand { parts.push("RDRAND"); }
    if p.has_rdseed { parts.push("RDSEED"); }
    let mut s = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 { s.push_str(", "); }
        s.push_str(part);
    }
    s
}

// ═══════════════════════════════════════════════════════════════════════════════
// Memory
// ═══════════════════════════════════════════════════════════════════════════════

fn memory_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    let ram_mb = p.total_ram_bytes / (1024 * 1024);
    s.push(format!("Q: How much RAM? A: {} MB", ram_mb));
    s.push(format!("Q: Combien de RAM? A: {} MB", ram_mb));

    let heap_keyboard = p.heap_size_bytes / 1024;
    let used_keyboard = p.heap_used_bytes / 1024;
    let free_keyboard = p.heap_free_bytes / 1024;
    s.push(format!("Q: Heap status? A: {} KB total, {} KB used, {} KB free",
        heap_keyboard, used_keyboard, free_keyboard));

    s.push(format!("Q: Physical frames? A: {} used, {} free",
        p.frames_used, p.frames_free));

    // Memory pressure awareness
    let total_frames = p.frames_used + p.frames_free;
    if total_frames > 0 {
        let usage_pct = (p.frames_used * 100) / total_frames;
        if usage_pct > 80 {
            s.push(format!("Memory pressure is high: {}% of frames in use. Be conservative with allocations.", usage_pct));
        } else if usage_pct < 20 {
            s.push(format!("Memory is abundant: only {}% in use. Safe to allocate large buffers.", usage_pct));
        } else {
            s.push(format!("Memory usage is moderate: {}% of frames in use.", usage_pct));
        }
    }

    if ram_mb <= 64 {
        s.push(String::from("This is a low-memory system. I should minimize allocations."));
    } else if ram_mb >= 1024 {
        s.push(String::from("Plenty of RAM available. Can use larger model buffers."));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PCI devices
// ═══════════════════════════════════════════════════════════════════════════════

fn pci_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    s.push(format!("Q: How many PCI devices? A: {}", p.pci_device_count));

    if p.pci_storage_controllers > 0 {
        s.push(format!("Q: Storage controllers? A: {} PCI storage controller(s)",
            p.pci_storage_controllers));
    }
    if p.pci_network_controllers > 0 {
        s.push(format!("Q: Network adapters? A: {} PCI network controller(s)",
            p.pci_network_controllers));
    }
    if p.pci_display_controllers > 0 {
        s.push(format!("Q: Display adapters? A: {} PCI display controller(s)",
            p.pci_display_controllers));
    }
    if p.pci_usb_controllers > 0 {
        s.push(format!("Q: USB controllers? A: {} PCI USB controller(s)",
            p.pci_usb_controllers));
    }
    if p.pci_audio_controllers > 0 {
        s.push(format!("Q: Audio devices? A: {} PCI audio controller(s)",
            p.pci_audio_controllers));
    }

    // Top 5 PCI devices as factual knowledge
    for (i, device) in p.pci_devices.iter().take(5).enumerate() {
        s.push(format!("PCI device {}: {} {} at {:02x}:{:02x}.{} (vendor {:04x}, device {:04x})",
            i, device.class_name, device.subclass_name,
            device.bus, device.device, device.function,
            device.vendor_id, device.device_id));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Storage
// ═══════════════════════════════════════════════════════════════════════════════

fn storage_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    if p.storage_devices.is_empty() {
        s.push(String::from("Q: Any storage? A: No block storage devices detected."));
        return;
    }

    let total_gb = p.total_storage_bytes / (1024 * 1024 * 1024);
    s.push(format!("Q: Total storage? A: {} GB across {} device(s)",
        total_gb, p.storage_devices.len()));

    for device in &p.storage_devices {
        let capability_mb = device.capacity_bytes / (1024 * 1024);
        let kind = // Pattern matching — Rust's exhaustive branching construct.
match device.kind {
            crate::jarvis_hw::probe::StorageKind::Nvme => "NVMe",
            crate::jarvis_hw::probe::StorageKind::Sata => "SATA",
            crate::jarvis_hw::probe::StorageKind::Ide => "IDE",
            crate::jarvis_hw::probe::StorageKind::Unknown => "unknown",
        };
        s.push(format!("Storage: {} ({}) — {} MB", device.name, kind, capability_mb));
    }

    // Partitions
    for part in &p.partitions {
        let size_mb = part.size_bytes / (1024 * 1024);
        s.push(format!("Partition: {} #{} — {} MB, type: {}{}",
            part.disk_name, part.number, size_mb, part.type_name,
            if part.bootable { " (bootable)" } else { "" }));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Encryption
// ═══════════════════════════════════════════════════════════════════════════════

fn encryption_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    if p.encryption_detected.is_empty() {
        s.push(String::from("Q: Encrypted disks? A: No disk encryption detected."));
        return;
    }

    s.push(format!("Q: Encrypted disks? A: {} encrypted volume(s) detected.",
        p.encryption_detected.len()));

    for encrypt in &p.encryption_detected {
        let encrypt_type = // Pattern matching — Rust's exhaustive branching construct.
match encrypt.encryption_type {
            crate::jarvis_hw::probe::EncryptionType::Luks1 => "LUKS1",
            crate::jarvis_hw::probe::EncryptionType::Luks2 => "LUKS2",
            crate::jarvis_hw::probe::EncryptionType::BitLocker => "BitLocker",
            crate::jarvis_hw::probe::EncryptionType::VeraCrypt => "VeraCrypt",
            crate::jarvis_hw::probe::EncryptionType::FileVault2 => "FileVault2",
            crate::jarvis_hw::probe::EncryptionType::DmCrypt => "dm-crypt",
            crate::jarvis_hw::probe::EncryptionType::OpalSed => "Opal SED",
            crate::jarvis_hw::probe::EncryptionType::Unknown => "unknown",
        };
        s.push(format!("Encryption: {} on {} — {}", encrypt_type, encrypt.disk_name, encrypt.detail));
    }

    // Reasoning about encryption
    s.push(String::from("To access encrypted data I need the user to provide a passphrase or key."));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Network
// ═══════════════════════════════════════════════════════════════════════════════

fn network_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    if !p.has_network {
        s.push(String::from("Q: Network available? A: No network interface detected."));
        return;
    }

    let link_str = if p.link_up { "up" } else { "down" };
    s.push(format!("Q: Network status? A: Interface detected, link {}", link_str));
    s.push(format!("Q: Reseau disponible? A: Interface detectee, lien {}", link_str));

    if let Some(mac) = p.mac_address {
        s.push(format!("Q: MAC address? A: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GPU
// ═══════════════════════════════════════════════════════════════════════════════

fn gpu_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    if !p.has_gpu {
        s.push(String::from("Q: GPU available? A: No dedicated GPU. Using CPU SIMD for compute."));
        s.push(String::from("Without a GPU, my neural compute runs on CPU SIMD (SSE2 or AVX2)."));
        return;
    }

    s.push(format!("Q: What GPU? A: {} ({} MB VRAM, {} CUs)",
        p.gpu_name, p.gpu_vram_mb, p.gpu_compute_units));
    s.push(format!("Q: Quel GPU? A: {} ({} MB VRAM)", p.gpu_name, p.gpu_vram_mb));

    if p.gpu_vram_mb >= 256 {
        s.push(format!("GPU has enough VRAM for neural weights. Can offload matmul."));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// USB
// ═══════════════════════════════════════════════════════════════════════════════

fn usb_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    if !p.usb_initialized || p.usb_devices.is_empty() {
        s.push(String::from("Q: USB devices? A: No USB devices connected."));
        return;
    }

    s.push(format!("Q: USB devices? A: {} device(s) on {} controller(s)",
        p.usb_devices.len(), p.usb_controller_count));

    for device in p.usb_devices.iter().take(4) {
        s.push(format!("USB: {} ({}) at address {}",
            device.product, device.class_name, device.address));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Timers
// ═══════════════════════════════════════════════════════════════════════════════

fn timer_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    let mut timers = Vec::new();
    if p.tsc_available { timers.push("TSC"); }
    if p.hpet_available { timers.push("HPET"); }

    if !timers.is_empty() {
        let mut timer_str = String::new();
        for (i, t) in timers.iter().enumerate() {
            if i > 0 { timer_str.push_str(", "); }
            timer_str.push_str(t);
        }
        s.push(format!("Q: Available timers? A: {}", timer_str));
    }

    if p.hpet_available && p.hpet_number_timers > 0 {
        s.push(format!("HPET: {} timers, {}bit, vendor 0x{:04x}",
            p.hpet_number_timers,
            if p.hpet_64bit { "64" } else { "32" },
            p.hpet_vendor_id));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Security
// ═══════════════════════════════════════════════════════════════════════════════

fn security_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    let mut features = Vec::new();
    if p.has_smep { features.push("SMEP"); }
    if p.has_smap { features.push("SMAP"); }
    if p.has_nx { features.push("NX/XD"); }
    if p.has_umip { features.push("UMIP"); }

    if !features.is_empty() {
        let mut feat_str = String::new();
        for (i, f) in features.iter().enumerate() {
            if i > 0 { feat_str.push_str(", "); }
            feat_str.push_str(f);
        }
        s.push(format!("Q: Security features? A: {}", feat_str));
        s.push(format!("Q: Protection hardware? A: {}", feat_str));
    }

    if p.has_smep && p.has_smap && p.has_nx {
        s.push(String::from("Full hardware exploit mitigations active: SMEP+SMAP+NX."));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACPI / Firmware
// ═══════════════════════════════════════════════════════════════════════════════

fn acpi_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    s.push(format!("Q: ACPI revision? A: {} (OEM: {})",
        p.acpi_revision, p.acpi_oem_id));

    if p.fadt_hardware_reduced {
        s.push(String::from("ACPI hardware-reduced mode is active."));
    }

    if p.ioapic_count > 0 {
        s.push(format!("Q: Interrupt controllers? A: {} I/O APIC(s), {} CPU LAPIC(s)",
            p.ioapic_count, p.apic_entries.len()));
    }

    if p.pcie_available && !p.pcie_segments.is_empty() {
        s.push(format!("Q: PCIe segments? A: {}", p.pcie_segments.len()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Scores (awareness of own situational strengths/weaknesses)
// ═══════════════════════════════════════════════════════════════════════════════

fn score_sequences(p: &HardwareProfile, s: &mut Vec<String>) {
    s.push(format!("Q: System overall score? A: {:.0}%", p.overall_score * 100.0));
    s.push(format!("Compute score: {:.0}%, Memory: {:.0}%, Storage: {:.0}%, Network: {:.0}%, Security: {:.0}%",
        p.compute_score * 100.0, p.memory_score * 100.0,
        p.storage_score * 100.0, p.network_score * 100.0,
        p.security_score * 100.0));

    // Self-aware capability assessment
    if p.compute_score >= 0.7 {
        s.push(String::from("This system has strong compute capability. Complex inference is feasible."));
    } else if p.compute_score < 0.3 {
        s.push(String::from("Compute resources are limited. Keep inference sequences short."));
    }

    if p.memory_score >= 0.7 {
        s.push(String::from("Memory is plentiful. Can cache activations and KV values."));
    } else if p.memory_score < 0.3 {
        s.push(String::from("Memory is tight. Must be careful with sequence lengths and caching."));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Reasoning chains (teach causal thinking about hardware)
// ═══════════════════════════════════════════════════════════════════════════════

fn reasoning_chains(p: &HardwareProfile, s: &mut Vec<String>) {
    let ram_mb = p.total_ram_bytes / (1024 * 1024);

    // Memory-aware reasoning
    if ram_mb >= 512 {
        s.push(format!(
            "I have {} MB RAM. My weights are ~18 MB. I have room for larger batch accumulation.",
            ram_mb));
    } else if ram_mb >= 64 {
        s.push(format!(
            "I have {} MB RAM. My weights are ~18 MB plus optimizer state ~36 MB. Memory is sufficient but not abundant.",
            ram_mb));
    } else {
        s.push(format!(
            "I have only {} MB RAM. Must minimize memory usage. Consider smaller model.",
            ram_mb));
    }

    // SIMD-aware reasoning
    if p.has_avx2 {
        s.push(String::from(
            "AVX2 is available. My matmul processes 32 floats per iteration with FMA. Training is 2-3x faster than SSE2."));
    } else if p.has_sse2 {
        s.push(String::from(
            "Only SSE2 is available. My matmul processes 16 floats per iteration. Training works but is slower."));
    }

    // GPU reasoning
    if p.has_gpu && p.gpu_vram_mb >= 64 {
        s.push(format!(
            "GPU detected with {} MB VRAM. Could offload weight matrices for 10-50x matmul speedup.",
            p.gpu_vram_mb));
    } else {
        s.push(String::from(
            "No GPU available. All neural compute runs on CPU SIMD. Focus on efficient kernels."));
    }

    // Security reasoning
    if !p.encryption_detected.is_empty() {
        s.push(String::from(
            "Encrypted volumes detected. I cannot access that data without the user providing credentials. This protects privacy."));
    }

    // Multi-core reasoning
    if p.maximum_logical_cpus > 1 {
        s.push(format!(
            "System has {} logical CPUs. Could parallelize multi-head attention across cores in the future.",
            p.maximum_logical_cpus));
    } else {
        s.push(String::from(
            "Single CPU core. All computation is sequential. Kernel optimization matters most."));
    }

    // Boot environment
    s.push(format!("I am running on {} in {} mode.", p.arch, p.privilege_level));

    // Hardware fingerprint summary
    s.push(format!(
        "System summary: {} cores, {} MB RAM, {} PCI devices, {} storage, {}GPU, {}net.",
        p.maximum_logical_cpus, ram_mb, p.pci_device_count,
        p.storage_devices.len(),
        if p.has_gpu { "" } else { "no " },
        if p.has_network { "" } else { "no " }));
}
