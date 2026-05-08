//! Hardware Probing & Fingerprinting — Exhaustive Device Profile
//!
//! Aggregates **every piece of hardware context** from all TrustOS subsystems
//! into one `HardwareProfile` that Jarvis reasons about. More context = safer
//! decisions = fewer crashes.
//!
//! Data sources:
//!   - CPUID full (vendor, family, model, stepping, brand, all flags)
//!   - ACPI (FADT, MADT, HPET, MCFG — firmware info, interrupt topology)
//!   - PCI bus (all devices, BARs, class/subclass/prog_if)
//!   - Memory (physical RAM, heap, frame stats, HHDM offset)
//!   - Storage (AHCI, NVMe, ATA — with partition + encryption detection)
//!   - Network (driver, MAC, link state)
//!   - GPU (amdgpu detection, VRAM, compute units)
//!   - Timers (TSC freq, HPET period/count, calibrated)
//!   - USB (xHCI/EHCI controllers, device enumeration)
//!   - Audio (HDA codec detection)
//!   - Interrupt topology (APIC, IOAPIC count, IRQ overrides)
//!   - Boot info (architecture, privilege, boot mode)

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// Hardware Profile — Exhaustive device fingerprint
// ═══════════════════════════════════════════════════════════════════════════════

/// Complete hardware profile. Every field Jarvis may ever need.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct HardwareProfile {
    // ── CPU ──────────────────────────────────────────────────────────────────
    pub cpu_vendor: String,
    pub cpu_brand: String,
    pub cpu_cores: u32,
    pub cpu_family: u8,
    pub cpu_model: u8,
    pub cpu_stepping: u8,
    pub tsc_freq_hz: u64,
    pub max_logical_cpus: u8,
    pub max_physical_cpus: u8,
    pub apic_id: u8,
    // SIMD
    pub has_sse: bool,
    pub has_sse2: bool,
    pub has_sse3: bool,
    pub has_ssse3: bool,
    pub has_sse4_1: bool,
    pub has_sse4_2: bool,
    pub has_avx: bool,
    pub has_avx2: bool,
    pub has_avx512: bool,
    // Crypto
    pub has_aesni: bool,
    pub has_pclmulqdq: bool,
    pub has_sha_ext: bool,
    pub has_rdrand: bool,
    pub has_rdseed: bool,
    // Security
    pub has_smep: bool,
    pub has_smap: bool,
    pub has_umip: bool,
    pub has_nx: bool,
    // TSC
    pub has_tsc: bool,
    pub has_tsc_invariant: bool,
    pub has_tsc_deadline: bool,
    pub has_rdtscp: bool,
    // Virtualization
    pub has_vmx: bool,
    pub has_svm: bool,

    // ── Memory ───────────────────────────────────────────────────────────────
    pub total_ram_bytes: u64,
    pub heap_size_bytes: usize,
    pub heap_used_bytes: usize,
    pub heap_free_bytes: usize,
    pub frames_used: usize,
    pub frames_free: usize,
    pub hhdm_offset: u64,

    // ── ACPI / Firmware ──────────────────────────────────────────────────────
    pub acpi_revision: u8,
    pub acpi_oem_id: String,
    pub fadt_sci_int: u16,
    pub fadt_hw_reduced: bool,
    pub fadt_reset_supported: bool,
    pub fadt_low_power_s0: bool,
    pub fadt_pm_tmr_blk: u32,

    // ── Interrupt Topology ───────────────────────────────────────────────────
    pub local_apic_addr: u64,
    pub apic_entries: Vec<ApicSummary>,
    pub ioapic_count: usize,
    pub ioapic_entries: Vec<IoApicSummary>,
    pub irq_overrides: Vec<IrqOverrideSummary>,
    pub apic_nmi_count: usize,

    // ── PCIe / MCFG ─────────────────────────────────────────────────────────
    pub pcie_segments: Vec<PcieSegmentSummary>,
    pub pcie_available: bool,

    // ── PCI Bus ──────────────────────────────────────────────────────────────
    pub pci_devices: Vec<PciDeviceSummary>,
    pub pci_device_count: usize,
    // Categorized counts from PCI scan
    pub pci_storage_controllers: usize,
    pub pci_network_controllers: usize,
    pub pci_usb_controllers: usize,
    pub pci_audio_controllers: usize,
    pub pci_display_controllers: usize,
    pub pci_bridge_count: usize,
    pub pci_crypto_controllers: usize,

    // ── Storage ──────────────────────────────────────────────────────────────
    pub storage_devices: Vec<StorageInformation>,
    pub total_storage_bytes: u64,
    pub partitions: Vec<PartitionSummary>,
    pub encryption_detected: Vec<EncryptionInformation>,

    // ── Network ──────────────────────────────────────────────────────────────
    pub has_network: bool,
    pub mac_address: Option<[u8; 6]>,
    pub link_up: bool,

    // ── GPU ──────────────────────────────────────────────────────────────────
    pub has_gpu: bool,
    pub gpu_name: String,
    pub gpu_vram_mb: u32,
    pub gpu_compute_units: u32,

    // ── Timers ───────────────────────────────────────────────────────────────
    pub hpet_available: bool,
    pub tsc_available: bool,
    pub hpet_freq_hz: u64,
    pub hpet_num_timers: u8,
    pub hpet_64bit: bool,
    pub hpet_vendor_id: u16,

    // ── USB ──────────────────────────────────────────────────────────────────
    pub usb_initialized: bool,
    pub usb_controller_count: usize,
    pub usb_devices: Vec<UsbDeviceSummary>,

    // ── Audio ────────────────────────────────────────────────────────────────
    pub hda_initialized: bool,

    // ── Architecture / Boot ──────────────────────────────────────────────────
    pub arch: &'static str,
    pub privilege_level: &'static str,

    // ── Computed scores (0.0 - 1.0) ──────────────────────────────────────────
    pub compute_score: f32,
    pub memory_score: f32,
    pub storage_score: f32,
    pub network_score: f32,
    pub security_score: f32,
    pub overall_score: f32,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct PciDeviceSummary {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub class_name: String,
    pub subclass_name: String,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct StorageInformation {
    pub name: String,
    pub kind: StorageKind,
    pub capacity_bytes: u64,
    pub model: String,
    pub serial: String,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum StorageKind {
    Sata,
    Nvme,
    Ide,
    Unknown,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl StorageKind {
        // Fonction publique — appelable depuis d'autres modules.
pub fn as_str(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            StorageKind::Sata => "SATA",
            StorageKind::Nvme => "NVMe",
            StorageKind::Ide => "IDE",
            StorageKind::Unknown => "???",
        }
    }
}

// ── New summary types for the expanded profile ────────────────────────────────

#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct ApicSummary {
    pub apic_id: u32,
    pub processor_id: u32,
    pub enabled: bool,
    pub online_capable: bool,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct IoApicSummary {
    pub id: u8,
    pub address: u64,
    pub gsi_base: u32,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct IrqOverrideSummary {
    pub source_irq: u8,
    pub global_irq: u32,
    pub polarity: u8,
    pub trigger: u8,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct PcieSegmentSummary {
    pub base_address: u64,
    pub segment: u16,
    pub start_bus: u8,
    pub end_bus: u8,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct PartitionSummary {
    pub disk_name: String,
    pub number: u8,
    pub start_lba: u64,
    pub size_bytes: u64,
    pub type_name: String,
    pub bootable: bool,
    pub name: String,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct EncryptionInformation {
    pub disk_name: String,
    pub partition: Option<u8>,
    pub encryption_type: EncryptionType,
    pub detail: String,
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum EncryptionType {
    Luks1,
    Luks2,
    BitLocker,
    VeraCrypt,
    FileVault2,
    DmCrypt,
    OpalSed,
    Unknown,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl EncryptionType {
        // Fonction publique — appelable depuis d'autres modules.
pub fn as_str(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            EncryptionType::Luks1 => "LUKS1",
            EncryptionType::Luks2 => "LUKS2",
            EncryptionType::BitLocker => "BitLocker",
            EncryptionType::VeraCrypt => "VeraCrypt",
            EncryptionType::FileVault2 => "FileVault2",
            EncryptionType::DmCrypt => "dm-crypt",
            EncryptionType::OpalSed => "OPAL SED",
            EncryptionType::Unknown => "Unknown encryption",
        }
    }
}

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct UsbDeviceSummary {
    pub address: u8,
    pub class_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product: String,
}

// Global cached profile
static PROFILE_READY: AtomicBool = AtomicBool::new(false);
// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static PROFILE: Mutex<Option<HardwareProfile>> = Mutex::new(None);

/// Perform a full hardware scan and build the exhaustive profile
pub fn scan_hardware() -> HardwareProfile {
    crate::serial_println!("[JARVIS-HW] Starting exhaustive hardware scan...");

    let mut profile = HardwareProfile {
        // CPU
        cpu_vendor: String::new(),
        cpu_brand: String::new(),
        cpu_cores: 1,
        cpu_family: 0,
        cpu_model: 0,
        cpu_stepping: 0,
        tsc_freq_hz: 0,
        max_logical_cpus: 0,
        max_physical_cpus: 0,
        apic_id: 0,
        has_sse: false,
        has_sse2: false,
        has_sse3: false,
        has_ssse3: false,
        has_sse4_1: false,
        has_sse4_2: false,
        has_avx: false,
        has_avx2: false,
        has_avx512: false,
        has_aesni: false,
        has_pclmulqdq: false,
        has_sha_ext: false,
        has_rdrand: false,
        has_rdseed: false,
        has_smep: false,
        has_smap: false,
        has_umip: false,
        has_nx: false,
        has_tsc: false,
        has_tsc_invariant: false,
        has_tsc_deadline: false,
        has_rdtscp: false,
        has_vmx: false,
        has_svm: false,
        // Memory
        total_ram_bytes: 0,
        heap_size_bytes: 0,
        heap_used_bytes: 0,
        heap_free_bytes: 0,
        frames_used: 0,
        frames_free: 0,
        hhdm_offset: 0,
        // ACPI / Firmware
        acpi_revision: 0,
        acpi_oem_id: String::new(),
        fadt_sci_int: 0,
        fadt_hw_reduced: false,
        fadt_reset_supported: false,
        fadt_low_power_s0: false,
        fadt_pm_tmr_blk: 0,
        // Interrupt topology
        local_apic_addr: 0,
        apic_entries: Vec::new(),
        ioapic_count: 0,
        ioapic_entries: Vec::new(),
        irq_overrides: Vec::new(),
        apic_nmi_count: 0,
        // PCIe
        pcie_segments: Vec::new(),
        pcie_available: false,
        // PCI
        pci_devices: Vec::new(),
        pci_device_count: 0,
        pci_storage_controllers: 0,
        pci_network_controllers: 0,
        pci_usb_controllers: 0,
        pci_audio_controllers: 0,
        pci_display_controllers: 0,
        pci_bridge_count: 0,
        pci_crypto_controllers: 0,
        // Storage
        storage_devices: Vec::new(),
        total_storage_bytes: 0,
        partitions: Vec::new(),
        encryption_detected: Vec::new(),
        // Network
        has_network: false,
        mac_address: None,
        link_up: false,
        // GPU
        has_gpu: false,
        gpu_name: String::new(),
        gpu_vram_mb: 0,
        gpu_compute_units: 0,
        // Timers
        hpet_available: false,
        tsc_available: false,
        hpet_freq_hz: 0,
        hpet_num_timers: 0,
        hpet_64bit: false,
        hpet_vendor_id: 0,
        // USB
        usb_initialized: false,
        usb_controller_count: 0,
        usb_devices: Vec::new(),
        // Audio
        hda_initialized: false,
        // Architecture
        arch: if cfg!(target_arch = "x86_64") { "x86_64" }
              else if cfg!(target_arch = "aarch64") { "aarch64" }
              else if cfg!(target_arch = "riscv64") { "riscv64" }
              else { "unknown" },
        privilege_level: "ring0",
        // Scores
        compute_score: 0.0,
        memory_score: 0.0,
        storage_score: 0.0,
        network_score: 0.0,
        security_score: 0.0,
        overall_score: 0.0,
    };

    // ── Phase 1: Core hardware ──
    probe_cpu(&mut profile);
    probe_memory(&mut profile);

    // ── Phase 2: ACPI / Firmware / Interrupt topology ──
    probe_acpi_firmware(&mut profile);

    // ── Phase 3: Bus enumeration ──
    probe_pci(&mut profile);

    // ── Phase 4: Subsystems ──
    probe_storage(&mut profile);
    probe_partitions_and_encryption(&mut profile);
    probe_network(&mut profile);
    probe_gpu(&mut profile);
    probe_timers(&mut profile);
    probe_usb(&mut profile);
    probe_audio(&mut profile);

    // ── Phase 5: Scoring ──
    compute_scores(&mut profile);

    crate::serial_println!("[JARVIS-HW] Exhaustive scan complete:");
    crate::serial_println!("  score={:.0}%, {} PCI devs, {} storage, {} partitions, {} encrypted",
        profile.overall_score * 100.0, profile.pci_device_count,
        profile.storage_devices.len(), profile.partitions.len(),
        profile.encryption_detected.len());
    crate::serial_println!("  {}MB RAM, {} APIC CPUs, {} IOAPICs, {} USB devs",
        profile.total_ram_bytes / (1024 * 1024), profile.apic_entries.len(),
        profile.ioapic_count, profile.usb_devices.len());

    // Cache it
    *PROFILE.lock() = Some(profile.clone());
    PROFILE_READY.store(true, Ordering::Release);

    profile
}

/// Get cached profile (None if scan hasn't run)
pub fn cached_profile() -> Option<HardwareProfile> {
    PROFILE.lock().clone()
}

/// Has the scan been performed?
pub fn is_scanned() -> bool {
    PROFILE_READY.load(Ordering::Acquire)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Individual probe functions — exhaustive context gathering
// ═══════════════════════════════════════════════════════════════════════════════

fn probe_cpu(profile: &mut HardwareProfile) {
    #[cfg(target_arch = "x86_64")]
    {
        let caps = crate::cpu::CpuCapabilities::detect();

        profile.cpu_vendor = // Correspondance de motifs — branchement exhaustif de Rust.
match caps.vendor {
            crate::cpu::CpuVendor::Intel => String::from("Intel"),
            crate::cpu::CpuVendor::Amd => String::from("AMD"),
            crate::cpu::CpuVendor::Unknown => String::from("Unknown"),
        };

        // Brand string
        let brand_length = caps.brand_string.iter()
            .position(|&b| b == 0)
            .unwrap_or(48);
        if let Ok(s) = core::str::from_utf8(&caps.brand_string[..brand_length]) {
            profile.cpu_brand = String::from(s.trim());
        }

        profile.cpu_family = caps.family;
        profile.cpu_model = caps.model;
        profile.cpu_stepping = caps.stepping;
        profile.cpu_cores = crate::cpu::smp::cpu_count();
        profile.tsc_freq_hz = caps.tsc_frequency_hz;
        profile.max_logical_cpus = caps.max_logical_cpus;
        profile.max_physical_cpus = caps.max_physical_cpus;
        profile.apic_id = caps.apic_id;

        // SIMD
        profile.has_sse = caps.sse;
        profile.has_sse2 = caps.sse2;
        profile.has_sse3 = caps.sse3;
        profile.has_ssse3 = caps.ssse3;
        profile.has_sse4_1 = caps.sse4_1;
        profile.has_sse4_2 = caps.sse4_2;
        profile.has_avx = caps.avx;
        profile.has_avx2 = caps.avx2;
        profile.has_avx512 = caps.avx512f;

        // Crypto
        profile.has_aesni = caps.aesni;
        profile.has_pclmulqdq = caps.pclmulqdq;
        profile.has_sha_ext = caps.sha_ext;
        profile.has_rdrand = caps.rdrand;
        profile.has_rdseed = caps.rdseed;

        // Security
        profile.has_smep = caps.smep;
        profile.has_smap = caps.smap;
        profile.has_umip = caps.umip;
        profile.has_nx = caps.nx;

        // TSC
        profile.has_tsc = caps.tsc;
        profile.has_tsc_invariant = caps.tsc_invariant;
        profile.has_tsc_deadline = caps.tsc_deadline;
        profile.has_rdtscp = caps.rdtscp;

        // Virtualization
        profile.has_vmx = caps.vmx;
        profile.has_svm = caps.svm;

        crate::serial_println!("[JARVIS-HW] CPU: {} F{}M{}S{} {}C SSE2={} AVX2={} AES={} SMEP={} NX={}",
            profile.cpu_brand, caps.family, caps.model, caps.stepping,
            profile.cpu_cores, caps.sse2, caps.avx2, caps.aesni, caps.smep, caps.nx);
    }

    #[cfg(target_arch = "aarch64")]
    {
        profile.cpu_vendor = String::from("ARM");
        profile.cpu_brand = String::from("AArch64 Processor");
        profile.cpu_cores = 1;
        profile.privilege_level = "EL1";
    }
}

fn probe_memory(profile: &mut HardwareProfile) {
    profile.total_ram_bytes = crate::memory::total_physical_memory();
    profile.heap_size_bytes = crate::memory::heap_size();
    profile.heap_used_bytes = crate::memory::heap::used();
    profile.heap_free_bytes = crate::memory::heap::free();
    profile.hhdm_offset = crate::memory::hhdm_offset();

    let stats = crate::memory::stats();
    profile.frames_used = stats.frames_used;
    profile.frames_free = stats.frames_free;

    crate::serial_println!("[JARVIS-HW] RAM: {} MB total, heap {} KB / {} KB, frames {}/{}",
        profile.total_ram_bytes / (1024 * 1024),
        profile.heap_used_bytes / 1024, profile.heap_free_bytes / 1024,
        profile.frames_used, profile.frames_free);
}

fn probe_acpi_firmware(profile: &mut HardwareProfile) {
    if let Some(acpi) = crate::acpi::get_information() {
        profile.acpi_revision = acpi.revision;
        profile.acpi_oem_id = acpi.oem_id.clone();
        profile.local_apic_addr = acpi.local_apic_addr;

        // FADT
        if let Some(ref fadt) = acpi.fadt {
            profile.fadt_sci_int = fadt.sci_int;
            profile.fadt_hw_reduced = fadt.is_hw_reduced();
            profile.fadt_reset_supported = fadt.supports_reset();
            profile.fadt_low_power_s0 = (fadt.flags & crate::acpi::fadt::FadtInfo::FLAG_LOW_POWER_S0) != 0;
            profile.fadt_pm_tmr_blk = fadt.pm_tmr_blk;
        }

        // MADT: Local APICs (per-CPU)
        for lapic in &acpi.local_apics {
            profile.apic_entries.push(ApicSummary {
                apic_id: lapic.apic_id,
                processor_id: lapic.processor_id,
                enabled: lapic.enabled,
                online_capable: lapic.online_capable,
            });
        }

        // IOAPICs
        profile.ioapic_count = acpi.io_apics.len();
        for ioapic in &acpi.io_apics {
            profile.ioapic_entries.push(IoApicSummary {
                id: ioapic.id,
                address: ioapic.address,
                gsi_base: ioapic.gsi_base,
            });
        }

        // IRQ overrides
        for ovr in &acpi.int_overrides {
            profile.irq_overrides.push(IrqOverrideSummary {
                source_irq: ovr.source,
                global_irq: ovr.gsi,
                polarity: ovr.polarity,
                trigger: ovr.trigger,
            });
        }

        profile.apic_nmi_count = acpi.local_apic_nmis.len();

        // HPET
        if let Some(ref hpet) = acpi.hpet {
            profile.hpet_available = true;
            profile.hpet_freq_hz = hpet.frequency();
            profile.hpet_num_timers = hpet.num_comparators;
            profile.hpet_64bit = hpet.counter_64bit;
            profile.hpet_vendor_id = hpet.vendor_id;
        }

        // PCIe segments (MCFG)
        profile.pcie_available = !acpi.mcfg_regions.is_empty();
        for seg in &acpi.mcfg_regions {
            profile.pcie_segments.push(PcieSegmentSummary {
                base_address: seg.base_address,
                segment: seg.segment,
                start_bus: seg.start_bus,
                end_bus: seg.end_bus,
            });
        }

        crate::serial_println!("[JARVIS-HW] ACPI: rev={} OEM='{}' {} CPUs {} IOAPICs {} overrides PCIe={}",
            acpi.revision, acpi.oem_id, acpi.local_apics.len(),
            profile.ioapic_count, profile.irq_overrides.len(), profile.pcie_available);
    } else {
        crate::serial_println!("[JARVIS-HW] ACPI: not available");
    }
}

fn probe_pci(profile: &mut HardwareProfile) {
    let devices = crate::pci::get_devices();
    profile.pci_device_count = devices.len();

    for dev in &devices {
        profile.pci_devices.push(PciDeviceSummary {
            bus: dev.bus,
            device: dev.device,
            function: dev.function,
            vendor_id: dev.vendor_id,
            device_id: dev.device_id,
            class_code: dev.class_code,
            subclass: dev.subclass,
            class_name: String::from(dev.class_name()),
            subclass_name: String::from(dev.subclass_name()),
        });

        // Categorize by class
        match dev.class_code {
            0x01 => profile.pci_storage_controllers += 1,
            0x02 => profile.pci_network_controllers += 1,
            0x03 => profile.pci_display_controllers += 1,
            0x04 => profile.pci_audio_controllers += 1,
            0x06 => profile.pci_bridge_count += 1,
            0x0C => {
                if dev.subclass == 0x03 { // USB
                    profile.pci_usb_controllers += 1;
                }
            }
            0x10 => profile.pci_crypto_controllers += 1, // Encryption/Decryption
            _ => {}
        }
    }

    crate::serial_println!("[JARVIS-HW] PCI: {} devices ({}stor {}net {}usb {}audio {}disp {}bridge)",
        profile.pci_device_count, profile.pci_storage_controllers,
        profile.pci_network_controllers, profile.pci_usb_controllers,
        profile.pci_audio_controllers, profile.pci_display_controllers,
        profile.pci_bridge_count);
}

fn probe_storage(profile: &mut HardwareProfile) {
    // AHCI/SATA
    if crate::drivers::ahci::is_initialized() {
        for port in crate::drivers::ahci::list_devices() {
            let cap = port.sector_count * 512;
            profile.total_storage_bytes += cap;
            profile.storage_devices.push(StorageInformation {
                name: format!("SATA port {}", port.port_num),
                kind: StorageKind::Sata,
                capacity_bytes: cap,
                model: port.model.clone(),
                serial: port.serial.clone(),
            });
        }
    }

    // NVMe
    if crate::nvme::is_initialized() {
        for ns in crate::nvme::list_namespaces() {
            let cap = ns.size_lbas * ns.lba_size as u64;
            profile.total_storage_bytes += cap;
            profile.storage_devices.push(StorageInformation {
                name: format!("NVMe ns{}", ns.nsid),
                kind: StorageKind::Nvme,
                capacity_bytes: cap,
                model: String::from("NVMe"),
                serial: String::new(),
            });
        }
    }

    crate::serial_println!("[JARVIS-HW] Storage: {} devices, {} GB total",
        profile.storage_devices.len(),
        profile.total_storage_bytes / (1024 * 1024 * 1024));
}

/// Probe partitions and detect disk encryption (LUKS, BitLocker, VeraCrypt)
fn probe_partitions_and_encryption(profile: &mut HardwareProfile) {
    // Try to read partition tables from AHCI disks
    if crate::drivers::ahci::is_initialized() {
        let port_count = crate::drivers::ahci::get_port_count();
        for port_num in 0..port_count {
            if crate::drivers::ahci::get_port_information(port_num).is_some() {
                // Read partition table
                if let Ok(pt) = crate::drivers::partition::read_from_ahci(port_num) {
                    let disk_name = format!("SATA:{}", port_num);
                    for part in &pt.partitions {
                        profile.partitions.push(PartitionSummary {
                            disk_name: disk_name.clone(),
                            number: part.number,
                            start_lba: part.start_lba,
                            size_bytes: part.size_bytes(),
                            type_name: format!("{:?}", part.partition_type),
                            bootable: part.bootable,
                            name: part.name.clone(),
                        });
                    }
                }

                // Probe first sector for whole-disk encryption headers
                let mut buf = [0u8; 512];
                if crate::drivers::ahci::read_sectors(port_num, 0, 1, &mut buf).is_ok() {
                    detect_encryption_header(&buf, &format!("SATA:{}", port_num), None, profile);
                }
                // Also check sector 6 (BitLocker backup) and sector 1 (LUKS on partition)
                if crate::drivers::ahci::read_sectors(port_num, 6, 1, &mut buf).is_ok() {
                    detect_encryption_header(&buf, &format!("SATA:{}", port_num), None, profile);
                }
            }
        }
    }

    crate::serial_println!("[JARVIS-HW] Partitions: {} found, {} encrypted volumes detected",
        profile.partitions.len(), profile.encryption_detected.len());
}

/// Check a 512-byte sector buffer for known encryption magic signatures
fn detect_encryption_header(buf: &[u8], disk_name: &str, part_num: Option<u8>, profile: &mut HardwareProfile) {
    if buf.len() < 512 { return; }

    // LUKS magic: "LUKS\xBA\xBE" at offset 0
    if buf.len() >= 6 && buf[0] == b'L' && buf[1] == b'U' && buf[2] == b'K'
        && buf[3] == b'S' && buf[4] == 0xBA && buf[5] == 0xBE
    {
        let version = if buf.len() >= 8 {
            ((buf[6] as u16) << 8) | buf[7] as u16
        } else { 0 };

        let encrypt_type = if version == 2 { EncryptionType::Luks2 } else { EncryptionType::Luks1 };
        let detail = format!("LUKS v{} detected at sector 0", version);

        // Avoid duplicates
        if !profile.encryption_detected.iter().any(|e| e.disk_name == disk_name && e.encryption_type == encrypt_type) {
            profile.encryption_detected.push(EncryptionInformation {
                disk_name: String::from(disk_name),
                partition: part_num,
                encryption_type: encrypt_type,
                detail,
            });
        }
    }

    // BitLocker: "-FVE-FS-" at offset 3 (in the OEM ID field of the BPB)
    if buf.len() >= 11
        && buf[3] == b'-' && buf[4] == b'F' && buf[5] == b'V' && buf[6] == b'E'
        && buf[7] == b'-' && buf[8] == b'F' && buf[9] == b'S' && buf[10] == b'-'
    {
        if !profile.encryption_detected.iter().any(|e| e.disk_name == disk_name && e.encryption_type == EncryptionType::BitLocker) {
            profile.encryption_detected.push(EncryptionInformation {
                disk_name: String::from(disk_name),
                partition: part_num,
                encryption_type: EncryptionType::BitLocker,
                detail: String::from("BitLocker BDE signature (-FVE-FS-) detected"),
            });
        }
    }

    // VeraCrypt: Check for VeraCrypt signature at sector 0
    // VeraCrypt uses "VERA" at specific offset in header, or the TC signature "TRUE"
    if buf.len() >= 4 && buf[0] == b'V' && buf[1] == b'E' && buf[2] == b'R' && buf[3] == b'A' {
        if !profile.encryption_detected.iter().any(|e| e.disk_name == disk_name && e.encryption_type == EncryptionType::VeraCrypt) {
            profile.encryption_detected.push(EncryptionInformation {
                disk_name: String::from(disk_name),
                partition: part_num,
                encryption_type: EncryptionType::VeraCrypt,
                detail: String::from("VeraCrypt volume header signature detected"),
            });
        }
    }

    // dm-crypt plain mode has no header, but we can detect unusual entropy
    // (not easily detectable without statistical analysis, skip for now)
}

fn probe_network(profile: &mut HardwareProfile) {
    profile.has_network = crate::drivers::net::has_driver();
    if profile.has_network {
        profile.mac_address = crate::drivers::net::get_mac();
        profile.link_up = crate::drivers::net::link_up();
    }

    crate::serial_println!("[JARVIS-HW] Network: detected={} link_up={}",
        profile.has_network, profile.link_up);
}

fn probe_gpu(profile: &mut HardwareProfile) {
    profile.has_gpu = crate::drivers::amdgpu::is_detected();
    if profile.has_gpu {
        if let Some(info) = crate::drivers::amdgpu::get_information() {
            profile.gpu_name = String::from(info.gpu_name());
            profile.gpu_vram_mb = (info.vram_aperture_size / (1024 * 1024)) as u32;
            profile.gpu_compute_units = info.compute_units;
        }
    }

    crate::serial_println!("[JARVIS-HW] GPU: detected={} name='{}'",
        profile.has_gpu, profile.gpu_name);
}

fn probe_timers(profile: &mut HardwareProfile) {
    #[cfg(target_arch = "x86_64")]
    {
        profile.tsc_available = profile.has_tsc;
        // HPET details are filled from ACPI probe; mark available if freq > 0
        if profile.hpet_freq_hz > 0 {
            profile.hpet_available = true;
        }
    }
}

fn probe_usb(profile: &mut HardwareProfile) {
    profile.usb_initialized = crate::drivers::usb::is_initialized();
    if profile.usb_initialized {
        profile.usb_controller_count = crate::drivers::usb::controller_count();
        let devices = crate::drivers::usb::enumerate_devices();
        for dev in &devices {
            profile.usb_devices.push(UsbDeviceSummary {
                address: dev.address,
                class_name: format!("{:?}", dev.class),
                vendor_id: dev.vendor_id,
                product_id: dev.product_id,
                product: dev.product.clone(),
            });
        }
    }

    crate::serial_println!("[JARVIS-HW] USB: init={} controllers={} devices={}",
        profile.usb_initialized, profile.usb_controller_count, profile.usb_devices.len());
}

fn probe_audio(profile: &mut HardwareProfile) {
    profile.hda_initialized = crate::drivers::hda::is_initialized();
    crate::serial_println!("[JARVIS-HW] Audio HDA: init={}", profile.hda_initialized);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Capability Scoring — normalize everything to 0.0..1.0
// ═══════════════════════════════════════════════════════════════════════════════

fn compute_scores(profile: &mut HardwareProfile) {
    // Compute score: cores × SIMD width × clock speed + GPU
    let simd_mult = if profile.has_avx512 { 4.0 }
        else if profile.has_avx2 { 2.0 }
        else if profile.has_avx { 1.5 }
        else if profile.has_sse2 { 1.0 }
        else { 0.5 };

    let core_factor = (profile.cpu_cores as f32).min(32.0) / 32.0;
    let frequency_factor = (profile.tsc_freq_hz as f32 / 5_000_000_000.0).min(1.0);
    let gpu_bonus = if profile.has_gpu { 0.3 } else { 0.0 };

    profile.compute_score = ((core_factor * 0.4 + frequency_factor * 0.3 + simd_mult / 4.0 * 0.3) + gpu_bonus).min(1.0);

    // Memory score: how much RAM relative to 64GB reference
    let ram_gb = profile.total_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    profile.memory_score = (ram_gb / 64.0).min(1.0);

    // Storage score: capacity + speed tier (NVMe > SATA >> IDE)
    let storage_gb = profile.total_storage_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let has_nvme = profile.storage_devices.iter().any(|s| s.kind == StorageKind::Nvme);
    let speed_mult = if has_nvme { 1.0 } else { 0.5 };
    profile.storage_score = ((storage_gb / 2048.0) * speed_mult).min(1.0);

    // Network score
    profile.network_score = if profile.has_network && profile.link_up { 1.0 }
        else if profile.has_network { 0.5 }
        else { 0.0 };

    // Security score: comprehensive
    let mut sec = 0.0f32;
    if profile.has_aesni { sec += 0.12; }
    if profile.has_rdrand { sec += 0.08; }
    if profile.has_rdseed { sec += 0.05; }
    if profile.has_sha_ext { sec += 0.05; }
    if profile.has_pclmulqdq { sec += 0.05; }
    if profile.has_smep { sec += 0.10; }
    if profile.has_smap { sec += 0.10; }
    if profile.has_umip { sec += 0.05; }
    if profile.has_nx { sec += 0.10; }
    // ring0 = full control
    sec += 0.15;
    // IOMMU awareness (having IOAPICs is good)
    if profile.ioapic_count > 0 { sec += 0.05; }
    // PCIe config space access
    if profile.pcie_available { sec += 0.05; }
    // TPM/crypto controller on PCI
    if profile.pci_crypto_controllers > 0 { sec += 0.05; }
    profile.security_score = sec.min(1.0);

    // Overall = weighted average
    profile.overall_score = profile.compute_score * 0.30
        + profile.memory_score * 0.20
        + profile.storage_score * 0.15
        + profile.network_score * 0.10
        + profile.security_score * 0.25;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Display — Exhaustive report
// ═══════════════════════════════════════════════════════════════════════════════

impl HardwareProfile {
    /// Format as a multi-line report for the shell
    pub fn format_report(&self) -> String {
        let mut s = String::new();

        s.push_str("\x01C╔══════════════════════════════════════════════════════════╗\n");
        s.push_str("║       JARVIS Exhaustive Hardware Intelligence Report      ║\n");
        s.push_str("╚══════════════════════════════════════════════════════════╝\x01W\n\n");

        // CPU
        s.push_str(&format!("\x01Y[CPU]\x01W {}\n", self.cpu_brand));
        s.push_str(&format!("  Vendor: {}  Family: {}  Model: {}  Stepping: {}\n",
            self.cpu_vendor, self.cpu_family, self.cpu_model, self.cpu_stepping));
        s.push_str(&format!("  Cores: {} (logical={} physical={})  TSC: {} MHz\n",
            self.cpu_cores, self.max_logical_cpus, self.max_physical_cpus,
            self.tsc_freq_hz / 1_000_000));
        s.push_str(&format!("  APIC ID: {}  TSC: inv={} deadline={} rdtscp={}\n",
            self.apic_id, self.has_tsc_invariant, self.has_tsc_deadline, self.has_rdtscp));
        s.push_str(&format!("  SIMD: SSE={} SSE2={} SSE3={} SSSE3={} SSE4.1={} SSE4.2={}\n",
            self.has_sse, self.has_sse2, self.has_sse3, self.has_ssse3,
            self.has_sse4_1, self.has_sse4_2));
        s.push_str(&format!("        AVX={} AVX2={} AVX-512={}\n",
            self.has_avx, self.has_avx2, self.has_avx512));
        s.push_str(&format!("  Crypto: AES-NI={} PCLMULQDQ={} SHA={} RDRAND={} RDSEED={}\n",
            self.has_aesni, self.has_pclmulqdq, self.has_sha_ext,
            self.has_rdrand, self.has_rdseed));
        s.push_str(&format!("  Security: SMEP={} SMAP={} UMIP={} NX={}\n",
            self.has_smep, self.has_smap, self.has_umip, self.has_nx));
        s.push_str(&format!("  Virt: VMX={} SVM={}\n\n", self.has_vmx, self.has_svm));

        // Memory
        s.push_str(&format!("\x01Y[Memory]\x01W {} MB physical\n", self.total_ram_bytes / (1024 * 1024)));
        s.push_str(&format!("  Heap: {} KB used / {} KB free (of {} KB)\n",
            self.heap_used_bytes / 1024, self.heap_free_bytes / 1024, self.heap_size_bytes / 1024));
        s.push_str(&format!("  Frames: {} used / {} free  HHDM: 0x{:X}\n\n",
            self.frames_used, self.frames_free, self.hhdm_offset));

        // ACPI / Firmware
        s.push_str(&format!("\x01Y[ACPI/Firmware]\x01W Rev={} OEM='{}'\n", self.acpi_revision, self.acpi_oem_id));
        s.push_str(&format!("  FADT: SCI={} HW_Reduced={} Reset={} LowPowerS0={} PM_TMR=0x{:X}\n",
            self.fadt_sci_int, self.fadt_hw_reduced, self.fadt_reset_supported,
            self.fadt_low_power_s0, self.fadt_pm_tmr_blk));
        s.push_str(&format!("  Local APIC: 0x{:X}  {} CPU APIC entries\n",
            self.local_apic_addr, self.apic_entries.len()));
        s.push_str(&format!("  IOAPICs: {}  IRQ Overrides: {}  NMIs: {}\n",
            self.ioapic_count, self.irq_overrides.len(), self.apic_nmi_count));
        if self.pcie_available {
            s.push_str(&format!("  PCIe: {} segment(s)\n", self.pcie_segments.len()));
        }
        s.push('\n');

        // HPET
        if self.hpet_available {
            s.push_str(&format!("\x01Y[HPET]\x01W {} MHz, {} timers, 64-bit={}, vendor=0x{:04X}\n\n",
                self.hpet_freq_hz / 1_000_000, self.hpet_num_timers,
                self.hpet_64bit, self.hpet_vendor_id));
        }

        // Storage
        s.push_str(&format!("\x01Y[Storage]\x01W {} device(s), {} GB total\n",
            self.storage_devices.len(), self.total_storage_bytes / (1024 * 1024 * 1024)));
        for dev in &self.storage_devices {
            s.push_str(&format!("  {} [{}] {} — {} GB\n",
                dev.name, dev.kind.as_str(), dev.model,
                dev.capacity_bytes / (1024 * 1024 * 1024)));
            if !dev.serial.is_empty() {
                s.push_str(&format!("    Serial: {}\n", dev.serial));
            }
        }

        // Partitions
        if !self.partitions.is_empty() {
            s.push_str(&format!("  {} partition(s):\n", self.partitions.len()));
            for p in &self.partitions {
                let name_suffix = if !p.name.is_empty() { format!(" '{}'", p.name) } else { String::new() };
                s.push_str(&format!("    #{} [{}] {} {} GB{}{}\n",
                    p.number, p.disk_name, p.type_name,
                    p.size_bytes / (1024 * 1024 * 1024),
                    if p.bootable { " *BOOT*" } else { "" },
                    name_suffix));
            }
        }

        // Encryption
        if !self.encryption_detected.is_empty() {
            s.push_str("\x01R  ⚠ Encrypted volumes detected:\x01W\n");
            for enc in &self.encryption_detected {
                s.push_str(&format!("    \x01R[{}]\x01W {} — {}\n",
                    enc.encryption_type.as_str(), enc.disk_name, enc.detail));
            }
        }
        s.push('\n');

        // Network
        s.push_str(&format!("\x01Y[Network]\x01W detected={} link={}\n", self.has_network, self.link_up));
        if let Some(mac) = self.mac_address {
            s.push_str(&format!("  MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
        }
        s.push('\n');

        // GPU
        if self.has_gpu {
            s.push_str(&format!("\x01Y[GPU]\x01W {}\n", self.gpu_name));
            s.push_str(&format!("  VRAM: {} MB  CUs: {}\n\n", self.gpu_vram_mb, self.gpu_compute_units));
        } else {
            s.push_str("\x01Y[GPU]\x01W None detected\n\n");
        }

        // USB
        s.push_str(&format!("\x01Y[USB]\x01W init={} controllers={} devices={}\n",
            self.usb_initialized, self.usb_controller_count, self.usb_devices.len()));
        for usb in &self.usb_devices {
            s.push_str(&format!("  [{:04X}:{:04X}] {} ({})\n",
                usb.vendor_id, usb.product_id, usb.product, usb.class_name));
        }
        s.push('\n');

        // Audio
        s.push_str(&format!("\x01Y[Audio]\x01W HDA init={}\n\n", self.hda_initialized));

        // PCI overview
        s.push_str(&format!("\x01Y[PCI Bus]\x01W {} devices ({}stor {}net {}usb {}audio {}disp {}bridge {}crypto)\n",
            self.pci_device_count, self.pci_storage_controllers,
            self.pci_network_controllers, self.pci_usb_controllers,
            self.pci_audio_controllers, self.pci_display_controllers,
            self.pci_bridge_count, self.pci_crypto_controllers));
        for dev in self.pci_devices.iter().take(20) {
            s.push_str(&format!("  {:02X}:{:02X}.{} [{:04X}:{:04X}] {} — {}\n",
                dev.bus, dev.device, dev.function,
                dev.vendor_id, dev.device_id,
                dev.class_name, dev.subclass_name));
        }
        if self.pci_device_count > 20 {
            s.push_str(&format!("  ... and {} more\n", self.pci_device_count - 20));
        }
        s.push('\n');

        // Scores
        s.push_str("\x01C═══ Capability Scores ═══\x01W\n");
        s.push_str(&format!("  Compute:  {} {}\n", score_bar(self.compute_score), format_pct(self.compute_score)));
        s.push_str(&format!("  Memory:   {} {}\n", score_bar(self.memory_score), format_pct(self.memory_score)));
        s.push_str(&format!("  Storage:  {} {}\n", score_bar(self.storage_score), format_pct(self.storage_score)));
        s.push_str(&format!("  Network:  {} {}\n", score_bar(self.network_score), format_pct(self.network_score)));
        s.push_str(&format!("  Security: {} {}\n", score_bar(self.security_score), format_pct(self.security_score)));
        s.push_str(&format!("  \x01COverall:  {} {}\x01W\n", score_bar(self.overall_score), format_pct(self.overall_score)));

        s
    }

    /// Compact one-line summary for Jarvis prompts
    pub fn one_liner(&self) -> String {
        format!("{} {}C {}MB {}xStorage {}GPU score={:.0}%",
            self.arch, self.cpu_cores,
            self.total_ram_bytes / (1024 * 1024),
            self.storage_devices.len(),
            if self.has_gpu { "+" } else { "-" },
            self.overall_score * 100.0)
    }

    /// Generate exhaustive text description for Jarvis AI reasoning
    pub fn to_ai_context(&self) -> String {
        let mut s = String::new();
        s.push_str("HARDWARE CONTEXT [exhaustive]:\n");

        // CPU
        s.push_str(&format!("CPU: vendor={} brand='{}' arch={} family={} model={} stepping={}\n",
            self.cpu_vendor, self.cpu_brand, self.arch, self.cpu_family,
            self.cpu_model, self.cpu_stepping));
        s.push_str(&format!("  cores={} logical={} physical={} tsc_mhz={} apic_id={}\n",
            self.cpu_cores, self.max_logical_cpus, self.max_physical_cpus,
            self.tsc_freq_hz / 1_000_000, self.apic_id));

        // SIMD level
        let simd = if self.has_avx512 { "avx512" }
            else if self.has_avx2 { "avx2" }
            else if self.has_avx { "avx" }
            else if self.has_sse4_2 { "sse4.2" }
            else if self.has_sse2 { "sse2" }
            else { "none" };
        s.push_str(&format!("  simd_level={} tsc_invariant={} rdtscp={}\n",
            simd, self.has_tsc_invariant, self.has_rdtscp));

        // Crypto
        s.push_str(&format!("  crypto: aesni={} pclmulqdq={} sha={} rdrand={} rdseed={}\n",
            self.has_aesni, self.has_pclmulqdq, self.has_sha_ext,
            self.has_rdrand, self.has_rdseed));
        // Security
        s.push_str(&format!("  security: smep={} smap={} umip={} nx={} vmx={} svm={}\n",
            self.has_smep, self.has_smap, self.has_umip, self.has_nx,
            self.has_vmx, self.has_svm));

        // Memory
        s.push_str(&format!("MEMORY: total={}MB heap={}KB(used={}KB free={}KB) frames_used={} frames_free={}\n",
            self.total_ram_bytes / (1024 * 1024),
            self.heap_size_bytes / 1024, self.heap_used_bytes / 1024,
            self.heap_free_bytes / 1024, self.frames_used, self.frames_free));

        // ACPI
        s.push_str(&format!("ACPI: rev={} oem='{}' hw_reduced={} reset={}\n",
            self.acpi_revision, self.acpi_oem_id, self.fadt_hw_reduced, self.fadt_reset_supported));
        s.push_str(&format!("  apic_cpus={} ioapics={} irq_overrides={} pcie_segments={}\n",
            self.apic_entries.len(), self.ioapic_count,
            self.irq_overrides.len(), self.pcie_segments.len()));

        // Storage
        s.push_str(&format!("STORAGE: devices={} total_gb={}\n",
            self.storage_devices.len(), self.total_storage_bytes / (1024 * 1024 * 1024)));
        for dev in &self.storage_devices {
            s.push_str(&format!("  {} [{}] {}GB model='{}'\n",
                dev.name, dev.kind.as_str(), dev.capacity_bytes / (1024 * 1024 * 1024), dev.model));
        }

        // Partitions
        if !self.partitions.is_empty() {
            s.push_str(&format!("PARTITIONS: {}\n", self.partitions.len()));
            for p in &self.partitions {
                s.push_str(&format!("  disk={} #{} type={} {}GB boot={}\n",
                    p.disk_name, p.number, p.type_name,
                    p.size_bytes / (1024 * 1024 * 1024), p.bootable));
            }
        }

        // Encryption — critical for disk access queries
        if !self.encryption_detected.is_empty() {
            s.push_str("ENCRYPTION_DETECTED:\n");
            for enc in &self.encryption_detected {
                s.push_str(&format!("  disk={} type={} detail='{}'\n",
                    enc.disk_name, enc.encryption_type.as_str(), enc.detail));
            }
        } else {
            s.push_str("ENCRYPTION_DETECTED: none\n");
        }

        // Network
        s.push_str(&format!("NETWORK: has_driver={} link_up={}", self.has_network, self.link_up));
        if let Some(mac) = self.mac_address {
            s.push_str(&format!(" mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
        }
        s.push('\n');

        // GPU
        s.push_str(&format!("GPU: has={} name='{}' vram_mb={} compute_units={}\n",
            self.has_gpu, self.gpu_name, self.gpu_vram_mb, self.gpu_compute_units));

        // Timers
        s.push_str(&format!("TIMERS: tsc={} hpet={} hpet_mhz={} hpet_timers={}\n",
            self.tsc_available, self.hpet_available,
            self.hpet_freq_hz / 1_000_000, self.hpet_num_timers));

        // USB
        s.push_str(&format!("USB: controllers={} devices={}\n",
            self.usb_controller_count, self.usb_devices.len()));
        for usb in &self.usb_devices {
            s.push_str(&format!("  [{:04X}:{:04X}] {} ({})\n",
                usb.vendor_id, usb.product_id, usb.product, usb.class_name));
        }

        // Audio
        s.push_str(&format!("AUDIO: hda_init={}\n", self.hda_initialized));

        // PCI summary
        s.push_str(&format!("PCI: total={} storage={} net={} usb={} audio={} display={} crypto={}\n",
            self.pci_device_count, self.pci_storage_controllers,
            self.pci_network_controllers, self.pci_usb_controllers,
            self.pci_audio_controllers, self.pci_display_controllers,
            self.pci_crypto_controllers));

        // Scores
        s.push_str(&format!("SCORES: compute={:.0}% memory={:.0}% storage={:.0}% network={:.0}% security={:.0}% overall={:.0}%\n",
            self.compute_score * 100.0, self.memory_score * 100.0,
            self.storage_score * 100.0, self.network_score * 100.0,
            self.security_score * 100.0, self.overall_score * 100.0));

        s
    }

    /// Check if a specific capability is present (for query engine)
    pub fn has_capability(&self, cap: &str) -> bool {
                // Correspondance de motifs — branchement exhaustif de Rust.
match cap {
            "aesni" | "aes" => self.has_aesni,
            "rdrand" | "random" => self.has_rdrand,
            "rdseed" => self.has_rdseed,
            "sha" => self.has_sha_ext,
            "avx2" => self.has_avx2,
            "avx512" => self.has_avx512,
            "gpu" => self.has_gpu,
            "network" | "net" => self.has_network,
            "storage" | "disk" => !self.storage_devices.is_empty(),
            "usb" => self.usb_initialized,
            "audio" | "sound" => self.hda_initialized,
            "smep" => self.has_smep,
            "smap" => self.has_smap,
            "nx" => self.has_nx,
            "vmx" | "vt-x" => self.has_vmx,
            "svm" | "amd-v" => self.has_svm,
            "pcie" => self.pcie_available,
            "hpet" => self.hpet_available,
            "encryption" | "encrypted" => !self.encryption_detected.is_empty(),
            _ => false,
        }
    }
}

fn score_bar(score: f32) -> String {
    let filled = (score * 20.0) as usize;
    let empty = 20 - filled;
    format!("[{}{}]",
        "#".repeat(filled),
        "-".repeat(empty))
}

fn format_pct(score: f32) -> String {
    format!("{:.0}%", score * 100.0)
}
