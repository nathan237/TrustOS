//! MARIONET Probe — Hardware data collection
//!
//! Collects structured data from all hardware subsystems.
//! Each collector returns a typed struct, not serialized text.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

// ─── Data Structures ───────────────────────────────────────────────────────

/// All hardware data collected in one shot
pub struct SystemData {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub pci_devices: Vec<PciDeviceInfo>,
    pub irq: IrqInfo,
    pub storage: StorageInfo,
    pub network: NetworkInfo,
    pub thermal: ThermalInfo,
    pub smbios: Option<crate::hwdiag::smbios::SmbiosInfo>,
    pub smart_disks: Vec<crate::hwdiag::smart::SmartData>,
    pub efi: crate::hwdiag::efi_vars::EfiInfo,
    pub battery: Option<crate::hwdiag::acpi_battery::BatteryInfo>,
    pub thermal_zones: Vec<crate::hwdiag::acpi_battery::ThermalZone>,
}

pub struct CpuInfo {
    pub brand: String,
    pub vendor: String,
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub cores: u8,
    pub features: Vec<String>,
    pub tsc_freq_mhz: u64,
}

pub struct MemoryInfo {
    pub total_bytes: u64,
    pub total_mb: u64,
    pub heap_used: usize,
    pub heap_free: usize,
    pub heap_total: usize,
    pub regions: Vec<String>,
}

#[derive(Clone)]
pub struct PciDeviceInfo {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub vendor_name: String,
    pub class_name: String,
    pub subclass_name: String,
    pub irq_line: u8,
    pub irq_pin: u8,
}

pub struct IrqInfo {
    pub local_apic_addr: u64,
    pub io_apic_count: usize,
    pub override_count: usize,
    pub cpu_count: usize,
    pub details: Vec<String>,
}

pub struct StorageInfo {
    pub devices: Vec<String>,
}

pub struct NetworkInfo {
    pub interfaces: Vec<String>,
}

pub struct ThermalInfo {
    pub cpu_temp: Option<i32>,
    pub tj_max: i32,
    pub details: Vec<String>,
}

// ─── Collectors ────────────────────────────────────────────────────────────

pub fn collect_all() -> SystemData {
    let power_info = crate::hwdiag::acpi_battery::collect_all();
    SystemData {
        cpu: collect_cpu(),
        memory: collect_memory(),
        pci_devices: collect_pci(),
        irq: collect_irq(),
        storage: collect_storage(),
        network: collect_network(),
        thermal: collect_thermal(),
        smbios: crate::hwdiag::smbios::get_info().cloned(),
        smart_disks: crate::hwdiag::smart::collect_all(),
        efi: crate::hwdiag::efi_vars::collect_efi_info(),
        battery: power_info.battery,
        thermal_zones: power_info.thermal_zones,
    }
}

pub fn collect_cpu() -> CpuInfo {
    let mut info = CpuInfo {
        brand: String::from("Unknown"),
        vendor: String::from("Unknown"),
        family: 0,
        model: 0,
        stepping: 0,
        cores: 1,
        features: Vec::new(),
        tsc_freq_mhz: 0,
    };

    if let Some(caps) = crate::cpu::capabilities() {
        info.brand = String::from(caps.brand());
        info.vendor = match caps.vendor {
            crate::cpu::CpuVendor::Intel => String::from("Intel"),
            crate::cpu::CpuVendor::Amd => String::from("AMD"),
            crate::cpu::CpuVendor::Unknown => String::from("Unknown"),
        };
        info.family = caps.family;
        info.model = caps.model;
        info.stepping = caps.stepping;
        info.cores = caps.max_logical_cpus;
        info.tsc_freq_mhz = caps.tsc_frequency_hz / 1_000_000;

        // Build feature list
        if caps.sse    { info.features.push(String::from("SSE")); }
        if caps.sse2   { info.features.push(String::from("SSE2")); }
        if caps.sse3   { info.features.push(String::from("SSE3")); }
        if caps.ssse3  { info.features.push(String::from("SSSE3")); }
        if caps.sse4_1 { info.features.push(String::from("SSE4.1")); }
        if caps.sse4_2 { info.features.push(String::from("SSE4.2")); }
        if caps.avx    { info.features.push(String::from("AVX")); }
        if caps.avx2   { info.features.push(String::from("AVX2")); }
        if caps.avx512f { info.features.push(String::from("AVX-512")); }
        if caps.fma    { info.features.push(String::from("FMA")); }
        if caps.aesni  { info.features.push(String::from("AES-NI")); }
        if caps.rdrand { info.features.push(String::from("RDRAND")); }
        if caps.rdseed { info.features.push(String::from("RDSEED")); }
        if caps.smep   { info.features.push(String::from("SMEP")); }
        if caps.smap   { info.features.push(String::from("SMAP")); }
        if caps.vmx    { info.features.push(String::from("VT-x")); }
        if caps.svm    { info.features.push(String::from("AMD-V")); }
        if caps.nx     { info.features.push(String::from("NX")); }
        if caps.tsc_invariant { info.features.push(String::from("InvariantTSC")); }
    }

    info
}

pub fn collect_memory() -> MemoryInfo {
    let total = crate::memory::total_physical_memory();
    let stats = crate::memory::stats();
    let heap_total = crate::memory::heap_size();

    MemoryInfo {
        total_bytes: total,
        total_mb: total / 1024 / 1024,
        heap_used: stats.heap_used,
        heap_free: stats.heap_free,
        heap_total,
        regions: Vec::new(), // Could parse memmap if available
    }
}

pub fn collect_pci() -> Vec<PciDeviceInfo> {
    let devices = crate::pci::scan();
    devices.iter().map(|d| {
        let subclass = d.subclass_name();
        let class = d.class_name();
        let display_class = if subclass.is_empty() {
            String::from(class)
        } else {
            format!("{} ({})", class, subclass)
        };
        PciDeviceInfo {
            bus: d.bus,
            device: d.device,
            function: d.function,
            vendor_id: d.vendor_id,
            device_id: d.device_id,
            vendor_name: String::from(d.vendor_name()),
            class_name: display_class,
            subclass_name: String::from(subclass),
            irq_line: d.interrupt_line,
            irq_pin: d.interrupt_pin,
        }
    }).collect()
}

pub fn collect_irq() -> IrqInfo {
    let mut info = IrqInfo {
        local_apic_addr: 0xFEE0_0000,
        io_apic_count: 0,
        override_count: 0,
        cpu_count: 1,
        details: Vec::new(),
    };

    if let Some(acpi) = crate::acpi::get_info() {
        info.local_apic_addr = acpi.local_apic_addr;
        info.io_apic_count = acpi.io_apics.len();
        info.override_count = acpi.int_overrides.len();
        info.cpu_count = acpi.cpu_count;

        for (i, ioapic) in acpi.io_apics.iter().enumerate() {
            info.details.push(format!("I/O APIC #{}: id={} addr=0x{:08X} gsi_base={}",
                i, ioapic.id, ioapic.address, ioapic.gsi_base));
        }

        for ovr in &acpi.int_overrides {
            info.details.push(format!("IRQ Override: source={} -> GSI={} polarity={:?} trigger={:?}",
                ovr.source, ovr.gsi, ovr.polarity, ovr.trigger));
        }

        for lapic in &acpi.local_apics {
            let state = if lapic.enabled { "enabled" } else { "disabled" };
            info.details.push(format!("CPU: APIC_ID={} processor_id={} [{}]",
                lapic.apic_id, lapic.processor_id, state));
        }
    }

    info
}

pub fn collect_thermal() -> ThermalInfo {
    let mut info = ThermalInfo {
        cpu_temp: None,
        tj_max: 100,
        details: Vec::new(),
    };

    #[cfg(target_arch = "x86_64")]
    {
        // Read TjMax
        if let Some(temp_target) = crate::debug::read_msr_safe(0x1A2) {
            info.tj_max = ((temp_target >> 16) & 0xFF) as i32;
        }

        // Read IA32_THERM_STATUS
        if let Some(therm) = crate::debug::read_msr_safe(0x19C) {
            let status = therm as u32;
            let reading_valid = status & (1 << 31) != 0;
            if reading_valid {
                let digital_readout = (status >> 16) & 0x7F;
                let temp = info.tj_max - digital_readout as i32;
                info.cpu_temp = Some(temp);
                info.details.push(format!("CPU Package: ~{}°C (delta={})", temp, digital_readout));

                if status & (1 << 2) != 0 {
                    info.details.push(String::from("PROCHOT# ACTIVE — CPU is throttling!"));
                }
            }
        }

        // Package thermal
        if let Some(pkg) = crate::debug::read_msr_safe(0x1B1) {
            let readout = ((pkg as u32) >> 16) & 0x7F;
            let pkg_temp = info.tj_max - readout as i32;
            info.details.push(format!("Package: ~{}°C", pkg_temp));
        }

        // IA32_PERF_STATUS — current P-state
        if let Some(perf) = crate::debug::read_msr_safe(0x198) {
            let ratio = (perf >> 8) & 0xFF;
            let bus_freq = 100; // Assume 100 MHz bus
            info.details.push(format!("P-State ratio: {}x ({}MHz)", ratio, ratio * bus_freq));
        }
    }

    info
}

pub fn collect_storage() -> StorageInfo {
    let mut devices = Vec::new();

    // Check PCI for storage controllers
    let pci_devs = crate::pci::scan();
    for d in &pci_devs {
        if d.class_code == 0x01 { // Mass Storage
            let sub = d.subclass_name();
            let name = if sub.is_empty() { d.class_name() } else { sub };
            devices.push(format!("{:02x}:{:02x}.{} [{:04x}:{:04x}] {} ({})",
                d.bus, d.device, d.function, d.vendor_id, d.device_id,
                d.vendor_name(), name));
        }
    }

    if devices.is_empty() {
        devices.push(String::from("No storage controllers detected"));
    }

    StorageInfo { devices }
}

pub fn collect_network() -> NetworkInfo {
    let mut interfaces = Vec::new();

    let pci_devs = crate::pci::scan();
    for d in &pci_devs {
        if d.class_code == 0x02 { // Network Controller
            let sub = d.subclass_name();
            let name = if sub.is_empty() { d.class_name() } else { sub };
            interfaces.push(format!("{:02x}:{:02x}.{} [{:04x}:{:04x}] {} ({})",
                d.bus, d.device, d.function, d.vendor_id, d.device_id,
                d.vendor_name(), name));
        }
    }

    if interfaces.is_empty() {
        interfaces.push(String::from("No network controllers detected"));
    }

    NetworkInfo { interfaces }
}
