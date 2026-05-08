
















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::jarvis_hw::probe::N;



pub fn generate(ai: &N) -> Vec<String> {
    let mut amt = Vec::with_capacity(120);

    kys(ai, &mut amt);
    neh(ai, &mut amt);
    nsu(ai, &mut amt);
    oxt(ai, &mut amt);
    lpz(ai, &mut amt);
    nin(ai, &mut amt);
    mfx(ai, &mut amt);
    pqh(ai, &mut amt);
    pjt(ai, &mut amt);
    omz(ai, &mut amt);
    jtl(ai, &mut amt);
    olp(ai, &mut amt);
    odm(ai, &mut amt);

    amt
}





fn kys(aa: &N, j: &mut Vec<String>) {
    
    j.push(format!("Q: What CPU is installed? A: {}", aa.cpu_brand));
    j.push(format!("Q: Quel processeur? A: {}", aa.cpu_brand));
    j.push(format!("Q: CPU vendor? A: {}", aa.cpu_vendor));
    j.push(format!("Q: How many CPU cores? A: {} logical, {} physical",
        aa.max_logical_cpus, aa.max_physical_cpus));
    j.push(format!("Q: Combien de coeurs? A: {} logiques, {} physiques",
        aa.max_logical_cpus, aa.max_physical_cpus));
    j.push(format!("Q: CPU family/model? A: family {} model {} stepping {}",
        aa.cpu_family, aa.cpu_model, aa.cpu_stepping));

    
    if aa.tsc_freq_hz > 0 {
        let inq = aa.tsc_freq_hz / 1_000_000;
        j.push(format!("Q: CPU frequency? A: ~{} MHz (TSC)", inq));
        j.push(format!("Q: Quelle frequence CPU? A: ~{} MHz", inq));
    }

    
    let simd = osy(aa);
    j.push(format!("Q: What SIMD is available? A: {}", simd));
    j.push(format!("Q: SIMD disponible? A: {}", simd));

    
    if aa.has_avx2 {
        j.push(format!("My SIMD kernels use AVX2+FMA: 8-wide float ops, 2-3x faster than SSE2."));
    } else {
        j.push(format!("My SIMD kernels use SSE2: 4-wide float ops. No AVX2 available."));
    }

    
    let crypto = kzr(aa);
    if !crypto.is_empty() {
        j.push(format!("Q: Hardware crypto? A: {}", crypto));
    }

    
    if aa.has_vmx {
        j.push(format!("Q: Virtualization? A: Intel VT-x supported"));
    }
    if aa.has_svm {
        j.push(format!("Q: Virtualization? A: AMD-V supported"));
    }
}

fn osy(aa: &N) -> String {
    let mut au = Vec::new();
    if aa.has_sse { au.push("SSE"); }
    if aa.has_sse2 { au.push("SSE2"); }
    if aa.has_sse3 { au.push("SSE3"); }
    if aa.has_sse4_2 { au.push("SSE4.2"); }
    if aa.has_avx { au.push("AVX"); }
    if aa.has_avx2 { au.push("AVX2"); }
    if aa.has_avx512 { au.push("AVX-512"); }
    if au.is_empty() { return String::from("none"); }
    let mut j = String::new();
    for (i, jn) in au.iter().enumerate() {
        if i > 0 { j.push_str(", "); }
        j.push_str(jn);
    }
    j
}

fn kzr(aa: &N) -> String {
    let mut au = Vec::new();
    if aa.has_aesni { au.push("AES-NI"); }
    if aa.has_sha_ext { au.push("SHA-NI"); }
    if aa.has_rdrand { au.push("RDRAND"); }
    if aa.has_rdseed { au.push("RDSEED"); }
    let mut j = String::new();
    for (i, jn) in au.iter().enumerate() {
        if i > 0 { j.push_str(", "); }
        j.push_str(jn);
    }
    j
}





fn neh(aa: &N, j: &mut Vec<String>) {
    let ram_mb = aa.total_ram_bytes / (1024 * 1024);
    j.push(format!("Q: How much RAM? A: {} MB", ram_mb));
    j.push(format!("Q: Combien de RAM? A: {} MB", ram_mb));

    let mkt = aa.heap_size_bytes / 1024;
    let fee = aa.heap_used_bytes / 1024;
    let fxr = aa.heap_free_bytes / 1024;
    j.push(format!("Q: Heap status? A: {} KB total, {} KB used, {} KB free",
        mkt, fee, fxr));

    j.push(format!("Q: Physical frames? A: {} used, {} free",
        aa.frames_used, aa.frames_free));

    
    let total_frames = aa.frames_used + aa.frames_free;
    if total_frames > 0 {
        let edk = (aa.frames_used * 100) / total_frames;
        if edk > 80 {
            j.push(format!("Memory pressure is high: {}% of frames in use. Be conservative with allocations.", edk));
        } else if edk < 20 {
            j.push(format!("Memory is abundant: only {}% in use. Safe to allocate large buffers.", edk));
        } else {
            j.push(format!("Memory usage is moderate: {}% of frames in use.", edk));
        }
    }

    if ram_mb <= 64 {
        j.push(String::from("This is a low-memory system. I should minimize allocations."));
    } else if ram_mb >= 1024 {
        j.push(String::from("Plenty of RAM available. Can use larger model buffers."));
    }
}





fn nsu(aa: &N, j: &mut Vec<String>) {
    j.push(format!("Q: How many PCI devices? A: {}", aa.pci_device_count));

    if aa.pci_storage_controllers > 0 {
        j.push(format!("Q: Storage controllers? A: {} PCI storage controller(s)",
            aa.pci_storage_controllers));
    }
    if aa.pci_network_controllers > 0 {
        j.push(format!("Q: Network adapters? A: {} PCI network controller(s)",
            aa.pci_network_controllers));
    }
    if aa.pci_display_controllers > 0 {
        j.push(format!("Q: Display adapters? A: {} PCI display controller(s)",
            aa.pci_display_controllers));
    }
    if aa.pci_usb_controllers > 0 {
        j.push(format!("Q: USB controllers? A: {} PCI USB controller(s)",
            aa.pci_usb_controllers));
    }
    if aa.pci_audio_controllers > 0 {
        j.push(format!("Q: Audio devices? A: {} PCI audio controller(s)",
            aa.pci_audio_controllers));
    }

    
    for (i, s) in aa.pci_devices.iter().take(5).enumerate() {
        j.push(format!("PCI device {}: {} {} at {:02x}:{:02x}.{} (vendor {:04x}, device {:04x})",
            i, s.class_name, s.subclass_name,
            s.bus, s.device, s.function,
            s.vendor_id, s.device_id));
    }
}





fn oxt(aa: &N, j: &mut Vec<String>) {
    if aa.storage_devices.is_empty() {
        j.push(String::from("Q: Any storage? A: No block storage devices detected."));
        return;
    }

    let plx = aa.total_storage_bytes / (1024 * 1024 * 1024);
    j.push(format!("Q: Total storage? A: {} GB across {} device(s)",
        plx, aa.storage_devices.len()));

    for s in &aa.storage_devices {
        let khj = s.capacity_bytes / (1024 * 1024);
        let kind = match s.kind {
            crate::jarvis_hw::probe::StorageKind::Nvme => "NVMe",
            crate::jarvis_hw::probe::StorageKind::Sata => "SATA",
            crate::jarvis_hw::probe::StorageKind::Ide => "IDE",
            crate::jarvis_hw::probe::StorageKind::Unknown => "unknown",
        };
        j.push(format!("Storage: {} ({}) — {} MB", s.name, kind, khj));
    }

    
    for jn in &aa.partitions {
        let size_mb = jn.size_bytes / (1024 * 1024);
        j.push(format!("Partition: {} #{} — {} MB, type: {}{}",
            jn.disk_name, jn.number, size_mb, jn.type_name,
            if jn.bootable { " (bootable)" } else { "" }));
    }
}





fn lpz(aa: &N, j: &mut Vec<String>) {
    if aa.encryption_detected.is_empty() {
        j.push(String::from("Q: Encrypted disks? A: No disk encryption detected."));
        return;
    }

    j.push(format!("Q: Encrypted disks? A: {} encrypted volume(s) detected.",
        aa.encryption_detected.len()));

    for enc in &aa.encryption_detected {
        let elj = match enc.encryption_type {
            crate::jarvis_hw::probe::EncryptionType::Luks1 => "LUKS1",
            crate::jarvis_hw::probe::EncryptionType::Luks2 => "LUKS2",
            crate::jarvis_hw::probe::EncryptionType::BitLocker => "BitLocker",
            crate::jarvis_hw::probe::EncryptionType::VeraCrypt => "VeraCrypt",
            crate::jarvis_hw::probe::EncryptionType::FileVault2 => "FileVault2",
            crate::jarvis_hw::probe::EncryptionType::DmCrypt => "dm-crypt",
            crate::jarvis_hw::probe::EncryptionType::OpalSed => "Opal SED",
            crate::jarvis_hw::probe::EncryptionType::Unknown => "unknown",
        };
        j.push(format!("Encryption: {} on {} — {}", elj, enc.disk_name, enc.detail));
    }

    
    j.push(String::from("To access encrypted data I need the user to provide a passphrase or key."));
}





fn nin(aa: &N, j: &mut Vec<String>) {
    if !aa.has_network {
        j.push(String::from("Q: Network available? A: No network interface detected."));
        return;
    }

    let ikk = if aa.link_up { "up" } else { "down" };
    j.push(format!("Q: Network status? A: Interface detected, link {}", ikk));
    j.push(format!("Q: Reseau disponible? A: Interface detectee, lien {}", ikk));

    if let Some(mac) = aa.mac_address {
        j.push(format!("Q: MAC address? A: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
    }
}





fn mfx(aa: &N, j: &mut Vec<String>) {
    if !aa.has_gpu {
        j.push(String::from("Q: GPU available? A: No dedicated GPU. Using CPU SIMD for compute."));
        j.push(String::from("Without a GPU, my neural compute runs on CPU SIMD (SSE2 or AVX2)."));
        return;
    }

    j.push(format!("Q: What GPU? A: {} ({} MB VRAM, {} CUs)",
        aa.gpu_name, aa.gpu_vram_mb, aa.gpu_compute_units));
    j.push(format!("Q: Quel GPU? A: {} ({} MB VRAM)", aa.gpu_name, aa.gpu_vram_mb));

    if aa.gpu_vram_mb >= 256 {
        j.push(format!("GPU has enough VRAM for neural weights. Can offload matmul."));
    }
}





fn pqh(aa: &N, j: &mut Vec<String>) {
    if !aa.usb_initialized || aa.usb_devices.is_empty() {
        j.push(String::from("Q: USB devices? A: No USB devices connected."));
        return;
    }

    j.push(format!("Q: USB devices? A: {} device(s) on {} controller(s)",
        aa.usb_devices.len(), aa.usb_controller_count));

    for s in aa.usb_devices.iter().take(4) {
        j.push(format!("USB: {} ({}) at address {}",
            s.product, s.class_name, s.address));
    }
}





fn pjt(aa: &N, j: &mut Vec<String>) {
    let mut timers = Vec::new();
    if aa.tsc_available { timers.push("TSC"); }
    if aa.hpet_available { timers.push("HPET"); }

    if !timers.is_empty() {
        let mut gyx = String::new();
        for (i, t) in timers.iter().enumerate() {
            if i > 0 { gyx.push_str(", "); }
            gyx.push_str(t);
        }
        j.push(format!("Q: Available timers? A: {}", gyx));
    }

    if aa.hpet_available && aa.hpet_num_timers > 0 {
        j.push(format!("HPET: {} timers, {}bit, vendor 0x{:04x}",
            aa.hpet_num_timers,
            if aa.hpet_64bit { "64" } else { "32" },
            aa.hpet_vendor_id));
    }
}





fn omz(aa: &N, j: &mut Vec<String>) {
    let mut features = Vec::new();
    if aa.has_smep { features.push("SMEP"); }
    if aa.has_smap { features.push("SMAP"); }
    if aa.has_nx { features.push("NX/XD"); }
    if aa.has_umip { features.push("UMIP"); }

    if !features.is_empty() {
        let mut emk = String::new();
        for (i, f) in features.iter().enumerate() {
            if i > 0 { emk.push_str(", "); }
            emk.push_str(f);
        }
        j.push(format!("Q: Security features? A: {}", emk));
        j.push(format!("Q: Protection hardware? A: {}", emk));
    }

    if aa.has_smep && aa.has_smap && aa.has_nx {
        j.push(String::from("Full hardware exploit mitigations active: SMEP+SMAP+NX."));
    }
}





fn jtl(aa: &N, j: &mut Vec<String>) {
    j.push(format!("Q: ACPI revision? A: {} (OEM: {})",
        aa.acpi_revision, aa.acpi_oem_id));

    if aa.fadt_hw_reduced {
        j.push(String::from("ACPI hardware-reduced mode is active."));
    }

    if aa.ioapic_count > 0 {
        j.push(format!("Q: Interrupt controllers? A: {} I/O APIC(s), {} CPU LAPIC(s)",
            aa.ioapic_count, aa.apic_entries.len()));
    }

    if aa.pcie_available && !aa.pcie_segments.is_empty() {
        j.push(format!("Q: PCIe segments? A: {}", aa.pcie_segments.len()));
    }
}





fn olp(aa: &N, j: &mut Vec<String>) {
    j.push(format!("Q: System overall score? A: {:.0}%", aa.overall_score * 100.0));
    j.push(format!("Compute score: {:.0}%, Memory: {:.0}%, Storage: {:.0}%, Network: {:.0}%, Security: {:.0}%",
        aa.compute_score * 100.0, aa.memory_score * 100.0,
        aa.storage_score * 100.0, aa.network_score * 100.0,
        aa.security_score * 100.0));

    
    if aa.compute_score >= 0.7 {
        j.push(String::from("This system has strong compute capability. Complex inference is feasible."));
    } else if aa.compute_score < 0.3 {
        j.push(String::from("Compute resources are limited. Keep inference sequences short."));
    }

    if aa.memory_score >= 0.7 {
        j.push(String::from("Memory is plentiful. Can cache activations and KV values."));
    } else if aa.memory_score < 0.3 {
        j.push(String::from("Memory is tight. Must be careful with sequence lengths and caching."));
    }
}





fn odm(aa: &N, j: &mut Vec<String>) {
    let ram_mb = aa.total_ram_bytes / (1024 * 1024);

    
    if ram_mb >= 512 {
        j.push(format!(
            "I have {} MB RAM. My weights are ~18 MB. I have room for larger batch accumulation.",
            ram_mb));
    } else if ram_mb >= 64 {
        j.push(format!(
            "I have {} MB RAM. My weights are ~18 MB plus optimizer state ~36 MB. Memory is sufficient but not abundant.",
            ram_mb));
    } else {
        j.push(format!(
            "I have only {} MB RAM. Must minimize memory usage. Consider smaller model.",
            ram_mb));
    }

    
    if aa.has_avx2 {
        j.push(String::from(
            "AVX2 is available. My matmul processes 32 floats per iteration with FMA. Training is 2-3x faster than SSE2."));
    } else if aa.has_sse2 {
        j.push(String::from(
            "Only SSE2 is available. My matmul processes 16 floats per iteration. Training works but is slower."));
    }

    
    if aa.has_gpu && aa.gpu_vram_mb >= 64 {
        j.push(format!(
            "GPU detected with {} MB VRAM. Could offload weight matrices for 10-50x matmul speedup.",
            aa.gpu_vram_mb));
    } else {
        j.push(String::from(
            "No GPU available. All neural compute runs on CPU SIMD. Focus on efficient kernels."));
    }

    
    if !aa.encryption_detected.is_empty() {
        j.push(String::from(
            "Encrypted volumes detected. I cannot access that data without the user providing credentials. This protects privacy."));
    }

    
    if aa.max_logical_cpus > 1 {
        j.push(format!(
            "System has {} logical CPUs. Could parallelize multi-head attention across cores in the future.",
            aa.max_logical_cpus));
    } else {
        j.push(String::from(
            "Single CPU core. All computation is sequential. Kernel optimization matters most."));
    }

    
    j.push(format!("I am running on {} in {} mode.", aa.arch, aa.privilege_level));

    
    j.push(format!(
        "System summary: {} cores, {} MB RAM, {} PCI devices, {} storage, {}GPU, {}net.",
        aa.max_logical_cpus, ram_mb, aa.pci_device_count,
        aa.storage_devices.len(),
        if aa.has_gpu { "" } else { "no " },
        if aa.has_network { "" } else { "no " }));
}
