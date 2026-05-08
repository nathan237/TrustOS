



















use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;






#[derive(Clone)]
pub struct N {
    
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
    
    pub has_sse: bool,
    pub has_sse2: bool,
    pub has_sse3: bool,
    pub has_ssse3: bool,
    pub has_sse4_1: bool,
    pub has_sse4_2: bool,
    pub has_avx: bool,
    pub has_avx2: bool,
    pub has_avx512: bool,
    
    pub has_aesni: bool,
    pub has_pclmulqdq: bool,
    pub has_sha_ext: bool,
    pub has_rdrand: bool,
    pub has_rdseed: bool,
    
    pub has_smep: bool,
    pub has_smap: bool,
    pub has_umip: bool,
    pub has_nx: bool,
    
    pub has_tsc: bool,
    pub has_tsc_invariant: bool,
    pub has_tsc_deadline: bool,
    pub has_rdtscp: bool,
    
    pub has_vmx: bool,
    pub has_svm: bool,

    
    pub total_ram_bytes: u64,
    pub heap_size_bytes: usize,
    pub heap_used_bytes: usize,
    pub heap_free_bytes: usize,
    pub frames_used: usize,
    pub frames_free: usize,
    pub hhdm_offset: u64,

    
    pub acpi_revision: u8,
    pub acpi_oem_id: String,
    pub fadt_sci_int: u16,
    pub fadt_hw_reduced: bool,
    pub fadt_reset_supported: bool,
    pub fadt_low_power_s0: bool,
    pub fadt_pm_tmr_blk: u32,

    
    pub local_apic_addr: u64,
    pub apic_entries: Vec<Wh>,
    pub ioapic_count: usize,
    pub ioapic_entries: Vec<Aae>,
    pub irq_overrides: Vec<Aai>,
    pub apic_nmi_count: usize,

    
    pub pcie_segments: Vec<Acl>,
    pub pcie_available: bool,

    
    pub pci_devices: Vec<Ack>,
    pub pci_device_count: usize,
    
    pub pci_storage_controllers: usize,
    pub pci_network_controllers: usize,
    pub pci_usb_controllers: usize,
    pub pci_audio_controllers: usize,
    pub pci_display_controllers: usize,
    pub pci_bridge_count: usize,
    pub pci_crypto_controllers: usize,

    
    pub storage_devices: Vec<Jb>,
    pub total_storage_bytes: u64,
    pub partitions: Vec<Acj>,
    pub encryption_detected: Vec<Ot>,

    
    pub has_network: bool,
    pub mac_address: Option<[u8; 6]>,
    pub link_up: bool,

    
    pub has_gpu: bool,
    pub gpu_name: String,
    pub gpu_vram_mb: u32,
    pub gpu_compute_units: u32,

    
    pub hpet_available: bool,
    pub tsc_available: bool,
    pub hpet_freq_hz: u64,
    pub hpet_num_timers: u8,
    pub hpet_64bit: bool,
    pub hpet_vendor_id: u16,

    
    pub usb_initialized: bool,
    pub usb_controller_count: usize,
    pub usb_devices: Vec<Afz>,

    
    pub hda_initialized: bool,

    
    pub arch: &'static str,
    pub privilege_level: &'static str,

    
    pub compute_score: f32,
    pub memory_score: f32,
    pub storage_score: f32,
    pub network_score: f32,
    pub security_score: f32,
    pub overall_score: f32,
}

#[derive(Clone)]
pub struct Ack {
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

#[derive(Clone)]
pub struct Jb {
    pub name: String,
    pub kind: StorageKind,
    pub capacity_bytes: u64,
    pub model: String,
    pub serial: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum StorageKind {
    Sata,
    Nvme,
    Ide,
    Unknown,
}

impl StorageKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageKind::Sata => "SATA",
            StorageKind::Nvme => "NVMe",
            StorageKind::Ide => "IDE",
            StorageKind::Unknown => "???",
        }
    }
}



#[derive(Clone)]
pub struct Wh {
    pub apic_id: u32,
    pub processor_id: u32,
    pub enabled: bool,
    pub online_capable: bool,
}

#[derive(Clone)]
pub struct Aae {
    pub id: u8,
    pub address: u64,
    pub gsi_base: u32,
}

#[derive(Clone)]
pub struct Aai {
    pub source_irq: u8,
    pub global_irq: u32,
    pub polarity: u8,
    pub trigger: u8,
}

#[derive(Clone)]
pub struct Acl {
    pub base_address: u64,
    pub segment: u16,
    pub start_bus: u8,
    pub end_bus: u8,
}

#[derive(Clone)]
pub struct Acj {
    pub disk_name: String,
    pub number: u8,
    pub start_lba: u64,
    pub size_bytes: u64,
    pub type_name: String,
    pub bootable: bool,
    pub name: String,
}

#[derive(Clone)]
pub struct Ot {
    pub disk_name: String,
    pub partition: Option<u8>,
    pub encryption_type: EncryptionType,
    pub detail: String,
}

#[derive(Clone, Copy, PartialEq)]
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

impl EncryptionType {
    pub fn as_str(&self) -> &'static str {
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

#[derive(Clone)]
pub struct Afz {
    pub address: u8,
    pub class_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product: String,
}


static BFR_: AtomicBool = AtomicBool::new(false);
static Ach: Mutex<Option<N>> = Mutex::new(None);


pub fn olc() -> N {
    crate::serial_println!("[JARVIS-HW] Starting exhaustive hardware scan...");

    let mut ai = N {
        
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
        
        total_ram_bytes: 0,
        heap_size_bytes: 0,
        heap_used_bytes: 0,
        heap_free_bytes: 0,
        frames_used: 0,
        frames_free: 0,
        hhdm_offset: 0,
        
        acpi_revision: 0,
        acpi_oem_id: String::new(),
        fadt_sci_int: 0,
        fadt_hw_reduced: false,
        fadt_reset_supported: false,
        fadt_low_power_s0: false,
        fadt_pm_tmr_blk: 0,
        
        local_apic_addr: 0,
        apic_entries: Vec::new(),
        ioapic_count: 0,
        ioapic_entries: Vec::new(),
        irq_overrides: Vec::new(),
        apic_nmi_count: 0,
        
        pcie_segments: Vec::new(),
        pcie_available: false,
        
        pci_devices: Vec::new(),
        pci_device_count: 0,
        pci_storage_controllers: 0,
        pci_network_controllers: 0,
        pci_usb_controllers: 0,
        pci_audio_controllers: 0,
        pci_display_controllers: 0,
        pci_bridge_count: 0,
        pci_crypto_controllers: 0,
        
        storage_devices: Vec::new(),
        total_storage_bytes: 0,
        partitions: Vec::new(),
        encryption_detected: Vec::new(),
        
        has_network: false,
        mac_address: None,
        link_up: false,
        
        has_gpu: false,
        gpu_name: String::new(),
        gpu_vram_mb: 0,
        gpu_compute_units: 0,
        
        hpet_available: false,
        tsc_available: false,
        hpet_freq_hz: 0,
        hpet_num_timers: 0,
        hpet_64bit: false,
        hpet_vendor_id: 0,
        
        usb_initialized: false,
        usb_controller_count: 0,
        usb_devices: Vec::new(),
        
        hda_initialized: false,
        
        arch: if cfg!(target_arch = "x86_64") { "x86_64" }
              else if cfg!(target_arch = "aarch64") { "aarch64" }
              else if cfg!(target_arch = "riscv64") { "riscv64" }
              else { "unknown" },
        privilege_level: "ring0",
        
        compute_score: 0.0,
        memory_score: 0.0,
        storage_score: 0.0,
        network_score: 0.0,
        security_score: 0.0,
        overall_score: 0.0,
    };

    
    nxm(&mut ai);
    nxs(&mut ai);

    
    nxk(&mut ai);

    
    ccv(&mut ai);

    
    gom(&mut ai);
    nxv(&mut ai);
    gok(&mut ai);
    goj(&mut ai);
    nxy(&mut ai);
    gon(&mut ai);
    goh(&mut ai);

    
    kwo(&mut ai);

    crate::serial_println!("[JARVIS-HW] Exhaustive scan complete:");
    crate::serial_println!("  score={:.0}%, {} PCI devs, {} storage, {} partitions, {} encrypted",
        ai.overall_score * 100.0, ai.pci_device_count,
        ai.storage_devices.len(), ai.partitions.len(),
        ai.encryption_detected.len());
    crate::serial_println!("  {}MB RAM, {} APIC CPUs, {} IOAPICs, {} USB devs",
        ai.total_ram_bytes / (1024 * 1024), ai.apic_entries.len(),
        ai.ioapic_count, ai.usb_devices.len());

    
    *Ach.lock() = Some(ai.clone());
    BFR_.store(true, Ordering::Release);

    ai
}


pub fn cur() -> Option<N> {
    Ach.lock().clone()
}


pub fn mtq() -> bool {
    BFR_.load(Ordering::Acquire)
}





fn nxm(ai: &mut N) {
    #[cfg(target_arch = "x86_64")]
    {
        let caps = crate::cpu::CpuCapabilities::bfx();

        ai.cpu_vendor = match caps.vendor {
            crate::cpu::CpuVendor::Intel => String::from("Intel"),
            crate::cpu::CpuVendor::Amd => String::from("AMD"),
            crate::cpu::CpuVendor::Unknown => String::from("Unknown"),
        };

        
        let kdt = caps.brand_string.iter()
            .position(|&b| b == 0)
            .unwrap_or(48);
        if let Ok(j) = core::str::from_utf8(&caps.brand_string[..kdt]) {
            ai.cpu_brand = String::from(j.trim());
        }

        ai.cpu_family = caps.family;
        ai.cpu_model = caps.model;
        ai.cpu_stepping = caps.stepping;
        ai.cpu_cores = crate::cpu::smp::cpu_count();
        ai.tsc_freq_hz = caps.tsc_frequency_hz;
        ai.max_logical_cpus = caps.max_logical_cpus;
        ai.max_physical_cpus = caps.max_physical_cpus;
        ai.apic_id = caps.apic_id;

        
        ai.has_sse = caps.sse;
        ai.has_sse2 = caps.sse2;
        ai.has_sse3 = caps.sse3;
        ai.has_ssse3 = caps.ssse3;
        ai.has_sse4_1 = caps.sse4_1;
        ai.has_sse4_2 = caps.sse4_2;
        ai.has_avx = caps.avx;
        ai.has_avx2 = caps.avx2;
        ai.has_avx512 = caps.avx512f;

        
        ai.has_aesni = caps.aesni;
        ai.has_pclmulqdq = caps.pclmulqdq;
        ai.has_sha_ext = caps.sha_ext;
        ai.has_rdrand = caps.rdrand;
        ai.has_rdseed = caps.rdseed;

        
        ai.has_smep = caps.smep;
        ai.has_smap = caps.smap;
        ai.has_umip = caps.umip;
        ai.has_nx = caps.nx;

        
        ai.has_tsc = caps.tsc;
        ai.has_tsc_invariant = caps.tsc_invariant;
        ai.has_tsc_deadline = caps.tsc_deadline;
        ai.has_rdtscp = caps.rdtscp;

        
        ai.has_vmx = caps.vmx;
        ai.has_svm = caps.svm;

        crate::serial_println!("[JARVIS-HW] CPU: {} F{}M{}S{} {}C SSE2={} AVX2={} AES={} SMEP={} NX={}",
            ai.cpu_brand, caps.family, caps.model, caps.stepping,
            ai.cpu_cores, caps.sse2, caps.avx2, caps.aesni, caps.smep, caps.nx);
    }

    #[cfg(target_arch = "aarch64")]
    {
        ai.cpu_vendor = String::from("ARM");
        ai.cpu_brand = String::from("AArch64 Processor");
        ai.cpu_cores = 1;
        ai.privilege_level = "EL1";
    }
}

fn nxs(ai: &mut N) {
    ai.total_ram_bytes = crate::memory::ceo();
    ai.heap_size_bytes = crate::memory::atz();
    ai.heap_used_bytes = crate::memory::heap::used();
    ai.heap_free_bytes = crate::memory::heap::free();
    ai.hhdm_offset = crate::memory::hhdm_offset();

    let stats = crate::memory::stats();
    ai.frames_used = stats.frames_used;
    ai.frames_free = stats.frames_free;

    crate::serial_println!("[JARVIS-HW] RAM: {} MB total, heap {} KB / {} KB, frames {}/{}",
        ai.total_ram_bytes / (1024 * 1024),
        ai.heap_used_bytes / 1024, ai.heap_free_bytes / 1024,
        ai.frames_used, ai.frames_free);
}

fn nxk(ai: &mut N) {
    if let Some(acpi) = crate::acpi::rk() {
        ai.acpi_revision = acpi.revision;
        ai.acpi_oem_id = acpi.oem_id.clone();
        ai.local_apic_addr = acpi.local_apic_addr;

        
        if let Some(ref fadt) = acpi.fadt {
            ai.fadt_sci_int = fadt.sci_int;
            ai.fadt_hw_reduced = fadt.is_hw_reduced();
            ai.fadt_reset_supported = fadt.supports_reset();
            ai.fadt_low_power_s0 = (fadt.flags & crate::acpi::fadt::FadtInfo::BYM_) != 0;
            ai.fadt_pm_tmr_blk = fadt.pm_tmr_blk;
        }

        
        for lapic in &acpi.local_apics {
            ai.apic_entries.push(Wh {
                apic_id: lapic.apic_id,
                processor_id: lapic.processor_id,
                enabled: lapic.enabled,
                online_capable: lapic.online_capable,
            });
        }

        
        ai.ioapic_count = acpi.io_apics.len();
        for ioapic in &acpi.io_apics {
            ai.ioapic_entries.push(Aae {
                id: ioapic.id,
                address: ioapic.address,
                gsi_base: ioapic.gsi_base,
            });
        }

        
        for ovr in &acpi.int_overrides {
            ai.irq_overrides.push(Aai {
                source_irq: ovr.source,
                global_irq: ovr.gsi,
                polarity: ovr.polarity,
                trigger: ovr.trigger,
            });
        }

        ai.apic_nmi_count = acpi.local_apic_nmis.len();

        
        if let Some(ref hpet) = acpi.hpet {
            ai.hpet_available = true;
            ai.hpet_freq_hz = hpet.frequency();
            ai.hpet_num_timers = hpet.num_comparators;
            ai.hpet_64bit = hpet.counter_64bit;
            ai.hpet_vendor_id = hpet.vendor_id;
        }

        
        ai.pcie_available = !acpi.mcfg_regions.is_empty();
        for gq in &acpi.mcfg_regions {
            ai.pcie_segments.push(Acl {
                base_address: gq.base_address,
                segment: gq.segment,
                start_bus: gq.start_bus,
                end_bus: gq.end_bus,
            });
        }

        crate::serial_println!("[JARVIS-HW] ACPI: rev={} OEM='{}' {} CPUs {} IOAPICs {} overrides PCIe={}",
            acpi.revision, acpi.oem_id, acpi.local_apics.len(),
            ai.ioapic_count, ai.irq_overrides.len(), ai.pcie_available);
    } else {
        crate::serial_println!("[JARVIS-HW] ACPI: not available");
    }
}

fn ccv(ai: &mut N) {
    let devices = crate::pci::aqs();
    ai.pci_device_count = devices.len();

    for s in &devices {
        ai.pci_devices.push(Ack {
            bus: s.bus,
            device: s.device,
            function: s.function,
            vendor_id: s.vendor_id,
            device_id: s.device_id,
            class_code: s.class_code,
            subclass: s.subclass,
            class_name: String::from(s.class_name()),
            subclass_name: String::from(s.subclass_name()),
        });

        
        match s.class_code {
            0x01 => ai.pci_storage_controllers += 1,
            0x02 => ai.pci_network_controllers += 1,
            0x03 => ai.pci_display_controllers += 1,
            0x04 => ai.pci_audio_controllers += 1,
            0x06 => ai.pci_bridge_count += 1,
            0x0C => {
                if s.subclass == 0x03 { 
                    ai.pci_usb_controllers += 1;
                }
            }
            0x10 => ai.pci_crypto_controllers += 1, 
            _ => {}
        }
    }

    crate::serial_println!("[JARVIS-HW] PCI: {} devices ({}stor {}net {}usb {}audio {}disp {}bridge)",
        ai.pci_device_count, ai.pci_storage_controllers,
        ai.pci_network_controllers, ai.pci_usb_controllers,
        ai.pci_audio_controllers, ai.pci_display_controllers,
        ai.pci_bridge_count);
}

fn gom(ai: &mut N) {
    
    if crate::drivers::ahci::is_initialized() {
        for port in crate::drivers::ahci::adz() {
            let cap = port.sector_count * 512;
            ai.total_storage_bytes += cap;
            ai.storage_devices.push(Jb {
                name: format!("SATA port {}", port.port_num),
                kind: StorageKind::Sata,
                capacity_bytes: cap,
                model: port.model.clone(),
                serial: port.serial.clone(),
            });
        }
    }

    
    if crate::nvme::is_initialized() {
        for ayq in crate::nvme::mze() {
            let cap = ayq.size_lbas * ayq.lba_size as u64;
            ai.total_storage_bytes += cap;
            ai.storage_devices.push(Jb {
                name: format!("NVMe ns{}", ayq.nsid),
                kind: StorageKind::Nvme,
                capacity_bytes: cap,
                model: String::from("NVMe"),
                serial: String::new(),
            });
        }
    }

    crate::serial_println!("[JARVIS-HW] Storage: {} devices, {} GB total",
        ai.storage_devices.len(),
        ai.total_storage_bytes / (1024 * 1024 * 1024));
}


fn nxv(ai: &mut N) {
    
    if crate::drivers::ahci::is_initialized() {
        let nwe = crate::drivers::ahci::ibt();
        for port_num in 0..nwe {
            if crate::drivers::ahci::fyw(port_num).is_some() {
                
                if let Ok(jd) = crate::drivers::partition::gqd(port_num) {
                    let disk_name = format!("SATA:{}", port_num);
                    for jn in &jd.partitions {
                        ai.partitions.push(Acj {
                            disk_name: disk_name.clone(),
                            number: jn.number,
                            start_lba: jn.start_lba,
                            size_bytes: jn.size_bytes(),
                            type_name: format!("{:?}", jn.partition_type),
                            bootable: jn.bootable,
                            name: jn.name.clone(),
                        });
                    }
                }

                
                let mut buf = [0u8; 512];
                if crate::drivers::ahci::read_sectors(port_num, 0, 1, &mut buf).is_ok() {
                    hrw(&buf, &format!("SATA:{}", port_num), None, ai);
                }
                
                if crate::drivers::ahci::read_sectors(port_num, 6, 1, &mut buf).is_ok() {
                    hrw(&buf, &format!("SATA:{}", port_num), None, ai);
                }
            }
        }
    }

    crate::serial_println!("[JARVIS-HW] Partitions: {} found, {} encrypted volumes detected",
        ai.partitions.len(), ai.encryption_detected.len());
}


fn hrw(buf: &[u8], disk_name: &str, part_num: Option<u8>, ai: &mut N) {
    if buf.len() < 512 { return; }

    
    if buf.len() >= 6 && buf[0] == b'L' && buf[1] == b'U' && buf[2] == b'K'
        && buf[3] == b'S' && buf[4] == 0xBA && buf[5] == 0xBE
    {
        let version = if buf.len() >= 8 {
            ((buf[6] as u16) << 8) | buf[7] as u16
        } else { 0 };

        let elj = if version == 2 { EncryptionType::Luks2 } else { EncryptionType::Luks1 };
        let detail = format!("LUKS v{} detected at sector 0", version);

        
        if !ai.encryption_detected.iter().any(|e| e.disk_name == disk_name && e.encryption_type == elj) {
            ai.encryption_detected.push(Ot {
                disk_name: String::from(disk_name),
                partition: part_num,
                encryption_type: elj,
                detail,
            });
        }
    }

    
    if buf.len() >= 11
        && buf[3] == b'-' && buf[4] == b'F' && buf[5] == b'V' && buf[6] == b'E'
        && buf[7] == b'-' && buf[8] == b'F' && buf[9] == b'S' && buf[10] == b'-'
    {
        if !ai.encryption_detected.iter().any(|e| e.disk_name == disk_name && e.encryption_type == EncryptionType::BitLocker) {
            ai.encryption_detected.push(Ot {
                disk_name: String::from(disk_name),
                partition: part_num,
                encryption_type: EncryptionType::BitLocker,
                detail: String::from("BitLocker BDE signature (-FVE-FS-) detected"),
            });
        }
    }

    
    
    if buf.len() >= 4 && buf[0] == b'V' && buf[1] == b'E' && buf[2] == b'R' && buf[3] == b'A' {
        if !ai.encryption_detected.iter().any(|e| e.disk_name == disk_name && e.encryption_type == EncryptionType::VeraCrypt) {
            ai.encryption_detected.push(Ot {
                disk_name: String::from(disk_name),
                partition: part_num,
                encryption_type: EncryptionType::VeraCrypt,
                detail: String::from("VeraCrypt volume header signature detected"),
            });
        }
    }

    
    
}

fn gok(ai: &mut N) {
    ai.has_network = crate::drivers::net::aoh();
    if ai.has_network {
        ai.mac_address = crate::drivers::net::aqt();
        ai.link_up = crate::drivers::net::link_up();
    }

    crate::serial_println!("[JARVIS-HW] Network: detected={} link_up={}",
        ai.has_network, ai.link_up);
}

fn goj(ai: &mut N) {
    ai.has_gpu = crate::drivers::amdgpu::aud();
    if ai.has_gpu {
        if let Some(info) = crate::drivers::amdgpu::rk() {
            ai.gpu_name = String::from(info.gpu_name());
            ai.gpu_vram_mb = (info.vram_aperture_size / (1024 * 1024)) as u32;
            ai.gpu_compute_units = info.compute_units;
        }
    }

    crate::serial_println!("[JARVIS-HW] GPU: detected={} name='{}'",
        ai.has_gpu, ai.gpu_name);
}

fn nxy(ai: &mut N) {
    #[cfg(target_arch = "x86_64")]
    {
        ai.tsc_available = ai.has_tsc;
        
        if ai.hpet_freq_hz > 0 {
            ai.hpet_available = true;
        }
    }
}

fn gon(ai: &mut N) {
    ai.usb_initialized = crate::drivers::usb::is_initialized();
    if ai.usb_initialized {
        ai.usb_controller_count = crate::drivers::usb::kxn();
        let devices = crate::drivers::usb::lqv();
        for s in &devices {
            ai.usb_devices.push(Afz {
                address: s.address,
                class_name: format!("{:?}", s.class),
                vendor_id: s.vendor_id,
                product_id: s.product_id,
                product: s.product.clone(),
            });
        }
    }

    crate::serial_println!("[JARVIS-HW] USB: init={} controllers={} devices={}",
        ai.usb_initialized, ai.usb_controller_count, ai.usb_devices.len());
}

fn goh(ai: &mut N) {
    ai.hda_initialized = crate::drivers::hda::is_initialized();
    crate::serial_println!("[JARVIS-HW] Audio HDA: init={}", ai.hda_initialized);
}





fn kwo(ai: &mut N) {
    
    let osx = if ai.has_avx512 { 4.0 }
        else if ai.has_avx2 { 2.0 }
        else if ai.has_avx { 1.5 }
        else if ai.has_sse2 { 1.0 }
        else { 0.5 };

    let kxy = (ai.cpu_cores as f32).min(32.0) / 32.0;
    let lyt = (ai.tsc_freq_hz as f32 / 5_000_000_000.0).min(1.0);
    let mfq = if ai.has_gpu { 0.3 } else { 0.0 };

    ai.compute_score = ((kxy * 0.4 + lyt * 0.3 + osx / 4.0 * 0.3) + mfq).min(1.0);

    
    let obb = ai.total_ram_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    ai.memory_score = (obb / 64.0).min(1.0);

    
    let oxq = ai.total_storage_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let fzx = ai.storage_devices.iter().any(|j| j.kind == StorageKind::Nvme);
    let ouv = if fzx { 1.0 } else { 0.5 };
    ai.storage_score = ((oxq / 2048.0) * ouv).min(1.0);

    
    ai.network_score = if ai.has_network && ai.link_up { 1.0 }
        else if ai.has_network { 0.5 }
        else { 0.0 };

    
    let mut lx = 0.0f32;
    if ai.has_aesni { lx += 0.12; }
    if ai.has_rdrand { lx += 0.08; }
    if ai.has_rdseed { lx += 0.05; }
    if ai.has_sha_ext { lx += 0.05; }
    if ai.has_pclmulqdq { lx += 0.05; }
    if ai.has_smep { lx += 0.10; }
    if ai.has_smap { lx += 0.10; }
    if ai.has_umip { lx += 0.05; }
    if ai.has_nx { lx += 0.10; }
    
    lx += 0.15;
    
    if ai.ioapic_count > 0 { lx += 0.05; }
    
    if ai.pcie_available { lx += 0.05; }
    
    if ai.pci_crypto_controllers > 0 { lx += 0.05; }
    ai.security_score = lx.min(1.0);

    
    ai.overall_score = ai.compute_score * 0.30
        + ai.memory_score * 0.20
        + ai.storage_score * 0.15
        + ai.network_score * 0.10
        + ai.security_score * 0.25;
}





impl N {
    
    pub fn format_report(&self) -> String {
        let mut j = String::new();

        j.push_str("\x01C╔══════════════════════════════════════════════════════════╗\n");
        j.push_str("║       JARVIS Exhaustive Hardware Intelligence Report      ║\n");
        j.push_str("╚══════════════════════════════════════════════════════════╝\x01W\n\n");

        
        j.push_str(&format!("\x01Y[CPU]\x01W {}\n", self.cpu_brand));
        j.push_str(&format!("  Vendor: {}  Family: {}  Model: {}  Stepping: {}\n",
            self.cpu_vendor, self.cpu_family, self.cpu_model, self.cpu_stepping));
        j.push_str(&format!("  Cores: {} (logical={} physical={})  TSC: {} MHz\n",
            self.cpu_cores, self.max_logical_cpus, self.max_physical_cpus,
            self.tsc_freq_hz / 1_000_000));
        j.push_str(&format!("  APIC ID: {}  TSC: inv={} deadline={} rdtscp={}\n",
            self.apic_id, self.has_tsc_invariant, self.has_tsc_deadline, self.has_rdtscp));
        j.push_str(&format!("  SIMD: SSE={} SSE2={} SSE3={} SSSE3={} SSE4.1={} SSE4.2={}\n",
            self.has_sse, self.has_sse2, self.has_sse3, self.has_ssse3,
            self.has_sse4_1, self.has_sse4_2));
        j.push_str(&format!("        AVX={} AVX2={} AVX-512={}\n",
            self.has_avx, self.has_avx2, self.has_avx512));
        j.push_str(&format!("  Crypto: AES-NI={} PCLMULQDQ={} SHA={} RDRAND={} RDSEED={}\n",
            self.has_aesni, self.has_pclmulqdq, self.has_sha_ext,
            self.has_rdrand, self.has_rdseed));
        j.push_str(&format!("  Security: SMEP={} SMAP={} UMIP={} NX={}\n",
            self.has_smep, self.has_smap, self.has_umip, self.has_nx));
        j.push_str(&format!("  Virt: VMX={} SVM={}\n\n", self.has_vmx, self.has_svm));

        
        j.push_str(&format!("\x01Y[Memory]\x01W {} MB physical\n", self.total_ram_bytes / (1024 * 1024)));
        j.push_str(&format!("  Heap: {} KB used / {} KB free (of {} KB)\n",
            self.heap_used_bytes / 1024, self.heap_free_bytes / 1024, self.heap_size_bytes / 1024));
        j.push_str(&format!("  Frames: {} used / {} free  HHDM: 0x{:X}\n\n",
            self.frames_used, self.frames_free, self.hhdm_offset));

        
        j.push_str(&format!("\x01Y[ACPI/Firmware]\x01W Rev={} OEM='{}'\n", self.acpi_revision, self.acpi_oem_id));
        j.push_str(&format!("  FADT: SCI={} HW_Reduced={} Reset={} LowPowerS0={} PM_TMR=0x{:X}\n",
            self.fadt_sci_int, self.fadt_hw_reduced, self.fadt_reset_supported,
            self.fadt_low_power_s0, self.fadt_pm_tmr_blk));
        j.push_str(&format!("  Local APIC: 0x{:X}  {} CPU APIC entries\n",
            self.local_apic_addr, self.apic_entries.len()));
        j.push_str(&format!("  IOAPICs: {}  IRQ Overrides: {}  NMIs: {}\n",
            self.ioapic_count, self.irq_overrides.len(), self.apic_nmi_count));
        if self.pcie_available {
            j.push_str(&format!("  PCIe: {} segment(s)\n", self.pcie_segments.len()));
        }
        j.push('\n');

        
        if self.hpet_available {
            j.push_str(&format!("\x01Y[HPET]\x01W {} MHz, {} timers, 64-bit={}, vendor=0x{:04X}\n\n",
                self.hpet_freq_hz / 1_000_000, self.hpet_num_timers,
                self.hpet_64bit, self.hpet_vendor_id));
        }

        
        j.push_str(&format!("\x01Y[Storage]\x01W {} device(s), {} GB total\n",
            self.storage_devices.len(), self.total_storage_bytes / (1024 * 1024 * 1024)));
        for s in &self.storage_devices {
            j.push_str(&format!("  {} [{}] {} — {} GB\n",
                s.name, s.kind.as_str(), s.model,
                s.capacity_bytes / (1024 * 1024 * 1024)));
            if !s.serial.is_empty() {
                j.push_str(&format!("    Serial: {}\n", s.serial));
            }
        }

        
        if !self.partitions.is_empty() {
            j.push_str(&format!("  {} partition(s):\n", self.partitions.len()));
            for aa in &self.partitions {
                let nhp = if !aa.name.is_empty() { format!(" '{}'", aa.name) } else { String::new() };
                j.push_str(&format!("    #{} [{}] {} {} GB{}{}\n",
                    aa.number, aa.disk_name, aa.type_name,
                    aa.size_bytes / (1024 * 1024 * 1024),
                    if aa.bootable { " *BOOT*" } else { "" },
                    nhp));
            }
        }

        
        if !self.encryption_detected.is_empty() {
            j.push_str("\x01R  ⚠ Encrypted volumes detected:\x01W\n");
            for enc in &self.encryption_detected {
                j.push_str(&format!("    \x01R[{}]\x01W {} — {}\n",
                    enc.encryption_type.as_str(), enc.disk_name, enc.detail));
            }
        }
        j.push('\n');

        
        j.push_str(&format!("\x01Y[Network]\x01W detected={} link={}\n", self.has_network, self.link_up));
        if let Some(mac) = self.mac_address {
            j.push_str(&format!("  MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
        }
        j.push('\n');

        
        if self.has_gpu {
            j.push_str(&format!("\x01Y[GPU]\x01W {}\n", self.gpu_name));
            j.push_str(&format!("  VRAM: {} MB  CUs: {}\n\n", self.gpu_vram_mb, self.gpu_compute_units));
        } else {
            j.push_str("\x01Y[GPU]\x01W None detected\n\n");
        }

        
        j.push_str(&format!("\x01Y[USB]\x01W init={} controllers={} devices={}\n",
            self.usb_initialized, self.usb_controller_count, self.usb_devices.len()));
        for usb in &self.usb_devices {
            j.push_str(&format!("  [{:04X}:{:04X}] {} ({})\n",
                usb.vendor_id, usb.product_id, usb.product, usb.class_name));
        }
        j.push('\n');

        
        j.push_str(&format!("\x01Y[Audio]\x01W HDA init={}\n\n", self.hda_initialized));

        
        j.push_str(&format!("\x01Y[PCI Bus]\x01W {} devices ({}stor {}net {}usb {}audio {}disp {}bridge {}crypto)\n",
            self.pci_device_count, self.pci_storage_controllers,
            self.pci_network_controllers, self.pci_usb_controllers,
            self.pci_audio_controllers, self.pci_display_controllers,
            self.pci_bridge_count, self.pci_crypto_controllers));
        for s in self.pci_devices.iter().take(20) {
            j.push_str(&format!("  {:02X}:{:02X}.{} [{:04X}:{:04X}] {} — {}\n",
                s.bus, s.device, s.function,
                s.vendor_id, s.device_id,
                s.class_name, s.subclass_name));
        }
        if self.pci_device_count > 20 {
            j.push_str(&format!("  ... and {} more\n", self.pci_device_count - 20));
        }
        j.push('\n');

        
        j.push_str("\x01C═══ Capability Scores ═══\x01W\n");
        j.push_str(&format!("  Compute:  {} {}\n", ddv(self.compute_score), cyb(self.compute_score)));
        j.push_str(&format!("  Memory:   {} {}\n", ddv(self.memory_score), cyb(self.memory_score)));
        j.push_str(&format!("  Storage:  {} {}\n", ddv(self.storage_score), cyb(self.storage_score)));
        j.push_str(&format!("  Network:  {} {}\n", ddv(self.network_score), cyb(self.network_score)));
        j.push_str(&format!("  Security: {} {}\n", ddv(self.security_score), cyb(self.security_score)));
        j.push_str(&format!("  \x01COverall:  {} {}\x01W\n", ddv(self.overall_score), cyb(self.overall_score)));

        j
    }

    
    pub fn qpx(&self) -> String {
        format!("{} {}C {}MB {}xStorage {}GPU score={:.0}%",
            self.arch, self.cpu_cores,
            self.total_ram_bytes / (1024 * 1024),
            self.storage_devices.len(),
            if self.has_gpu { "+" } else { "-" },
            self.overall_score * 100.0)
    }

    
    pub fn to_ai_context(&self) -> String {
        let mut j = String::new();
        j.push_str("HARDWARE CONTEXT [exhaustive]:\n");

        
        j.push_str(&format!("CPU: vendor={} brand='{}' arch={} family={} model={} stepping={}\n",
            self.cpu_vendor, self.cpu_brand, self.arch, self.cpu_family,
            self.cpu_model, self.cpu_stepping));
        j.push_str(&format!("  cores={} logical={} physical={} tsc_mhz={} apic_id={}\n",
            self.cpu_cores, self.max_logical_cpus, self.max_physical_cpus,
            self.tsc_freq_hz / 1_000_000, self.apic_id));

        
        let simd = if self.has_avx512 { "avx512" }
            else if self.has_avx2 { "avx2" }
            else if self.has_avx { "avx" }
            else if self.has_sse4_2 { "sse4.2" }
            else if self.has_sse2 { "sse2" }
            else { "none" };
        j.push_str(&format!("  simd_level={} tsc_invariant={} rdtscp={}\n",
            simd, self.has_tsc_invariant, self.has_rdtscp));

        
        j.push_str(&format!("  crypto: aesni={} pclmulqdq={} sha={} rdrand={} rdseed={}\n",
            self.has_aesni, self.has_pclmulqdq, self.has_sha_ext,
            self.has_rdrand, self.has_rdseed));
        
        j.push_str(&format!("  security: smep={} smap={} umip={} nx={} vmx={} svm={}\n",
            self.has_smep, self.has_smap, self.has_umip, self.has_nx,
            self.has_vmx, self.has_svm));

        
        j.push_str(&format!("MEMORY: total={}MB heap={}KB(used={}KB free={}KB) frames_used={} frames_free={}\n",
            self.total_ram_bytes / (1024 * 1024),
            self.heap_size_bytes / 1024, self.heap_used_bytes / 1024,
            self.heap_free_bytes / 1024, self.frames_used, self.frames_free));

        
        j.push_str(&format!("ACPI: rev={} oem='{}' hw_reduced={} reset={}\n",
            self.acpi_revision, self.acpi_oem_id, self.fadt_hw_reduced, self.fadt_reset_supported));
        j.push_str(&format!("  apic_cpus={} ioapics={} irq_overrides={} pcie_segments={}\n",
            self.apic_entries.len(), self.ioapic_count,
            self.irq_overrides.len(), self.pcie_segments.len()));

        
        j.push_str(&format!("STORAGE: devices={} total_gb={}\n",
            self.storage_devices.len(), self.total_storage_bytes / (1024 * 1024 * 1024)));
        for s in &self.storage_devices {
            j.push_str(&format!("  {} [{}] {}GB model='{}'\n",
                s.name, s.kind.as_str(), s.capacity_bytes / (1024 * 1024 * 1024), s.model));
        }

        
        if !self.partitions.is_empty() {
            j.push_str(&format!("PARTITIONS: {}\n", self.partitions.len()));
            for aa in &self.partitions {
                j.push_str(&format!("  disk={} #{} type={} {}GB boot={}\n",
                    aa.disk_name, aa.number, aa.type_name,
                    aa.size_bytes / (1024 * 1024 * 1024), aa.bootable));
            }
        }

        
        if !self.encryption_detected.is_empty() {
            j.push_str("ENCRYPTION_DETECTED:\n");
            for enc in &self.encryption_detected {
                j.push_str(&format!("  disk={} type={} detail='{}'\n",
                    enc.disk_name, enc.encryption_type.as_str(), enc.detail));
            }
        } else {
            j.push_str("ENCRYPTION_DETECTED: none\n");
        }

        
        j.push_str(&format!("NETWORK: has_driver={} link_up={}", self.has_network, self.link_up));
        if let Some(mac) = self.mac_address {
            j.push_str(&format!(" mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
        }
        j.push('\n');

        
        j.push_str(&format!("GPU: has={} name='{}' vram_mb={} compute_units={}\n",
            self.has_gpu, self.gpu_name, self.gpu_vram_mb, self.gpu_compute_units));

        
        j.push_str(&format!("TIMERS: tsc={} hpet={} hpet_mhz={} hpet_timers={}\n",
            self.tsc_available, self.hpet_available,
            self.hpet_freq_hz / 1_000_000, self.hpet_num_timers));

        
        j.push_str(&format!("USB: controllers={} devices={}\n",
            self.usb_controller_count, self.usb_devices.len()));
        for usb in &self.usb_devices {
            j.push_str(&format!("  [{:04X}:{:04X}] {} ({})\n",
                usb.vendor_id, usb.product_id, usb.product, usb.class_name));
        }

        
        j.push_str(&format!("AUDIO: hda_init={}\n", self.hda_initialized));

        
        j.push_str(&format!("PCI: total={} storage={} net={} usb={} audio={} display={} crypto={}\n",
            self.pci_device_count, self.pci_storage_controllers,
            self.pci_network_controllers, self.pci_usb_controllers,
            self.pci_audio_controllers, self.pci_display_controllers,
            self.pci_crypto_controllers));

        
        j.push_str(&format!("SCORES: compute={:.0}% memory={:.0}% storage={:.0}% network={:.0}% security={:.0}% overall={:.0}%\n",
            self.compute_score * 100.0, self.memory_score * 100.0,
            self.storage_score * 100.0, self.network_score * 100.0,
            self.security_score * 100.0, self.overall_score * 100.0));

        j
    }

    
    pub fn qkm(&self, cap: &str) -> bool {
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

fn ddv(score: f32) -> String {
    let oz = (score * 20.0) as usize;
    let empty = 20 - oz;
    format!("[{}{}]",
        "#".repeat(oz),
        "-".repeat(empty))
}

fn cyb(score: f32) -> String {
    format!("{:.0}%", score * 100.0)
}
