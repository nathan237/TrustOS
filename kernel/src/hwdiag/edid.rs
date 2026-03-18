//! EDID Display Parser — Extended Display Identification Data
//!
//! Reads EDID from display via DDC/I2C on VGA I/O ports,
//! or from VESA VBE if available. Identifies connected monitors:
//! manufacturer, model, resolution, refresh rate, color depth.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Parsed EDID information
#[derive(Debug, Clone)]
pub struct EdidInfo {
    pub manufacturer: String,
    pub product_code: u16,
    pub serial_number: u32,
    pub manufacture_week: u8,
    pub manufacture_year: u16,
    pub edid_version: u8,
    pub edid_revision: u8,
    pub max_h_size_cm: u8,
    pub max_v_size_cm: u8,
    pub preferred_width: u32,
    pub preferred_height: u32,
    pub preferred_refresh: u32,
    pub monitor_name: String,
    pub monitor_serial: String,
    pub bit_depth: u8,
    pub digital_input: bool,
    pub supported_modes: Vec<DisplayMode>,
}

#[derive(Debug, Clone)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh: u32,
}

impl Default for EdidInfo {
    fn default() -> Self {
        Self {
            manufacturer: String::new(),
            product_code: 0,
            serial_number: 0,
            manufacture_week: 0,
            manufacture_year: 0,
            edid_version: 0,
            edid_revision: 0,
            max_h_size_cm: 0,
            max_v_size_cm: 0,
            preferred_width: 0,
            preferred_height: 0,
            preferred_refresh: 0,
            monitor_name: String::new(),
            monitor_serial: String::new(),
            bit_depth: 0,
            digital_input: false,
            supported_modes: Vec::new(),
        }
    }
}

/// Run EDID display diagnostics
pub fn run(args: &[&str]) {
    let _verbose = args.contains(&"-v") || args.contains(&"--verbose");

    dbg_out!("[EDID] === Display / Monitor Identification ===");

    // Report current framebuffer info from bootloader
    dump_framebuffer_info();

    // Try DDC/I2C EDID read
    #[cfg(target_arch = "x86_64")]
    match read_edid_ddc() {
        Some(edid) => {
            dump_edid(&edid);
        }
        None => {
            dbg_out!("[EDID] DDC/I2C EDID read failed or not available");
            dbg_out!("[EDID] (Monitor may not support DDC, or GPU doesn't expose I2C)");
        }
    }

    // Report PCI GPU devices for context
    dump_gpu_devices();
}

/// Collect EDID info for integration
pub fn collect() -> Option<EdidInfo> {
    #[cfg(target_arch = "x86_64")]
    {
        read_edid_ddc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        None
    }
}

fn dump_framebuffer_info() {
    // Get framebuffer dimensions from the active framebuffer
    let (width, height) = crate::framebuffer::get_dimensions();
    dbg_out!("[DISPLAY] Active framebuffer: {}x{}", width, height);
    dbg_out!("");
}

fn dump_gpu_devices() {
    let devices = crate::pci::scan();
    let gpus: Vec<_> = devices.iter().filter(|d| d.class_code == 0x03).collect();

    if gpus.is_empty() {
        dbg_out!("[GPU] No display controllers found in PCI");
    } else {
        dbg_out!("[GPU] Display controllers:");
        for d in &gpus {
            dbg_out!("[GPU]   {:02x}:{:02x}.{} [{:04x}:{:04x}] {} — {}",
                d.bus, d.device, d.function,
                d.vendor_id, d.device_id,
                d.vendor_name(), d.class_name());
        }
    }
}

fn dump_edid(edid: &EdidInfo) {
    dbg_out!("[EDID] Manufacturer:    {}", edid.manufacturer);
    dbg_out!("[EDID] Product Code:    {:#06x}", edid.product_code);
    if !edid.monitor_name.is_empty() {
        dbg_out!("[EDID] Monitor Name:    {}", edid.monitor_name);
    }
    if !edid.monitor_serial.is_empty() {
        dbg_out!("[EDID] Monitor Serial:  {}", edid.monitor_serial);
    }
    dbg_out!("[EDID] EDID Version:    {}.{}", edid.edid_version, edid.edid_revision);
    dbg_out!("[EDID] Input Type:      {}", if edid.digital_input { "Digital" } else { "Analog" });
    dbg_out!("[EDID] Bit Depth:       {}", edid.bit_depth);
    dbg_out!("[EDID] Year/Week:       {} / W{}", edid.manufacture_year, edid.manufacture_week);
    if edid.max_h_size_cm > 0 && edid.max_v_size_cm > 0 {
        // Integer approximation for diagonal (avoid f64::powi in no_std)
        let h = edid.max_h_size_cm as u32;
        let v = edid.max_v_size_cm as u32;
        let diag_sq = h * h + v * v;
        // Integer sqrt approximation
        let mut diag = 1u32;
        while diag * diag < diag_sq { diag += 1; }
        // Convert cm to inches (approx: *100/254)
        let diag_in_x10 = diag * 1000 / 254;
        dbg_out!("[EDID] Physical Size:   {} x {} cm (~{}.{}\")", edid.max_h_size_cm, edid.max_v_size_cm, diag_in_x10 / 10, diag_in_x10 % 10);
    }
    if edid.preferred_width > 0 {
        dbg_out!("[EDID] Preferred Mode:  {}x{} @ {}Hz",
            edid.preferred_width, edid.preferred_height, edid.preferred_refresh);
    }

    if !edid.supported_modes.is_empty() {
        dbg_out!("[EDID] Standard Modes:");
        for mode in &edid.supported_modes {
            dbg_out!("[EDID]   {}x{} @ {}Hz", mode.width, mode.height, mode.refresh);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DDC/I2C EDID Read (x86_64)
// ═══════════════════════════════════════════════════════════════════════════════
//
// DDC uses I2C protocol over VGA connector pins (SDA=data, SCL=clock).
// On most GPUs, I2C is accessible via GPIO registers mapped through PCI BARs.
// This implementation tries the VGA I2C port approach.

#[cfg(target_arch = "x86_64")]
fn read_edid_ddc() -> Option<EdidInfo> {
    // Try reading EDID via VBE/VESA BIOS Extensions if available
    // Since we're post-boot, we try the PCI GPU I2C approach

    // Find GPU PCI device
    let devices = crate::pci::scan();
    let gpu = devices.iter().find(|d| d.class_code == 0x03)?;

    // Try to read EDID through GPU I2C
    // Intel GPUs: GMCH I2C at BAR0 + 0x5100 (GMBUS)
    // Generic: Try common I2C addresses

    if gpu.vendor_id == 0x8086 {
        // Intel GPU — try GMBUS I2C
        if let Some(edid_bytes) = read_edid_intel_gmbus(gpu) {
            return parse_edid(&edid_bytes);
        }
    }

    // Generic fallback: Try reading EDID from standard I2C address 0x50
    // via VGA/DVI DDC on the GPU's I2C controller
    // This is GPU-specific and may not work on all hardware
    None
}

#[cfg(target_arch = "x86_64")]
fn read_edid_intel_gmbus(gpu: &crate::pci::PciDevice) -> Option<[u8; 128]> {
    // Intel GMBUS (Graphics Memory Bus) I2C controller
    // Located at GPU BAR0 + GMBUS offset
    let bar0 = gpu.bar[0];
    if bar0 == 0 || bar0 == 0xFFFFFFFF { return None; }

    let bar0_phys = (bar0 & !0xF) as u64;

    // Map GPU MMIO region (we need access to GMBUS registers)
    let bar0_virt = crate::memory::map_mmio(bar0_phys, 0x80000).ok()?;

    // GMBUS register offsets (relative to BAR0)
    const GMBUS0: u64 = 0x5100; // Clock/Port select
    const GMBUS1: u64 = 0x5104; // Command/Status
    const GMBUS2: u64 = 0x5108; // Status
    const GMBUS3: u64 = 0x510C; // Data buffer
    const GMBUS4: u64 = 0x5110; // Interrupt mask
    const GMBUS5: u64 = 0x5120; // 2-byte index

    let read_reg = |offset: u64| -> u32 {
        unsafe { *((bar0_virt + offset) as *const u32) }
    };
    let write_reg = |offset: u64, val: u32| {
        unsafe { *((bar0_virt + offset) as *mut u32) = val; }
    };

    // Reset GMBUS
    write_reg(GMBUS1, 1 << 31); // SW_CLR_INT
    write_reg(GMBUS1, 0);

    // Select port — try VGADDC (port 2) for VGA, or DPIOPD (port 3) for DP
    // Pin pair 2 = Analog/VGA DDC
    // Pin pair 3 = DP-B (Display Port B)
    let ports_to_try = [2u32, 3, 4, 5, 1];

    for &port in &ports_to_try {
        write_reg(GMBUS0, port); // Select pin pair
        write_reg(GMBUS5, 0);   // No indexed access

        // Send I2C read command to EDID address 0x50, 128 bytes
        // GMBUS1: [31] SW_CLR_INT=0, [30] SW_RDY=0, [29:25] total bytes,
        //         [24:17] slave addr, [16] direction (1=read),
        //         [15:8] byte count MSB, [7:0] reserved
        // Slave address 0x50 (EDID) = 0xA0 in 8-bit form (0x50 << 1)
        let cmd = (128 << 16) | (0x50 << 1) | (1 << 0) | (0b011 << 25); // WAIT cycle, read
        write_reg(GMBUS1, cmd);

        // Read 128 bytes in 4-byte chunks
        let mut edid = [0u8; 128];
        let mut ok = true;

        for chunk in 0..32 {
            // Wait for GMBUS ready or error
            let mut timeout = 0u32;
            loop {
                let status = read_reg(GMBUS2);
                if status & (1 << 11) != 0 { // NAK
                    ok = false;
                    break;
                }
                if status & (1 << 15) != 0 { // Timeout
                    ok = false;
                    break;
                }
                if status & (1 << 0) != 0 { // HW_RDY — data available
                    break;
                }
                timeout += 1;
                if timeout > 100_000 {
                    ok = false;
                    break;
                }
                core::hint::spin_loop();
            }

            if !ok { break; }

            let data = read_reg(GMBUS3);
            let base = chunk * 4;
            if base < 128 { edid[base] = (data & 0xFF) as u8; }
            if base + 1 < 128 { edid[base + 1] = ((data >> 8) & 0xFF) as u8; }
            if base + 2 < 128 { edid[base + 2] = ((data >> 16) & 0xFF) as u8; }
            if base + 3 < 128 { edid[base + 3] = ((data >> 24) & 0xFF) as u8; }
        }

        // Stop
        write_reg(GMBUS1, (1 << 27) | (1 << 30)); // STOP + SW_RDY

        // Reset
        write_reg(GMBUS1, 1 << 31);
        write_reg(GMBUS1, 0);

        if ok {
            // Validate EDID header
            if edid[0] == 0x00 && edid[1] == 0xFF && edid[2] == 0xFF && edid[3] == 0xFF
                && edid[4] == 0xFF && edid[5] == 0xFF && edid[6] == 0xFF && edid[7] == 0x00 {
                crate::serial_println!("[EDID] Valid EDID found on GMBUS port {}", port);
                return Some(edid);
            }
        }
    }

    None
}

/// Parse 128-byte EDID block
fn parse_edid(raw: &[u8; 128]) -> Option<EdidInfo> {
    // Validate header
    if raw[0] != 0x00 || raw[1] != 0xFF || raw[7] != 0x00 {
        return None;
    }

    // Validate checksum
    let sum: u8 = raw.iter().fold(0u8, |a, &b| a.wrapping_add(b));
    if sum != 0 {
        crate::serial_println!("[EDID] Checksum failed");
        return None;
    }

    let mut info = EdidInfo::default();

    // Manufacturer ID (bytes 8-9, big-endian, 3 compressed ASCII letters)
    let mfg_code = ((raw[8] as u16) << 8) | raw[9] as u16;
    let c1 = ((mfg_code >> 10) & 0x1F) as u8 + b'A' - 1;
    let c2 = ((mfg_code >> 5) & 0x1F) as u8 + b'A' - 1;
    let c3 = (mfg_code & 0x1F) as u8 + b'A' - 1;
    info.manufacturer = format!("{}{}{}", c1 as char, c2 as char, c3 as char);

    // Product code (bytes 10-11, little-endian)
    info.product_code = u16::from_le_bytes([raw[10], raw[11]]);

    // Serial number (bytes 12-15, little-endian)
    info.serial_number = u32::from_le_bytes([raw[12], raw[13], raw[14], raw[15]]);

    // Manufacture date
    info.manufacture_week = raw[16];
    info.manufacture_year = 1990 + raw[17] as u16;

    // EDID version
    info.edid_version = raw[18];
    info.edid_revision = raw[19];

    // Video input — byte 20
    info.digital_input = raw[20] & 0x80 != 0;
    if info.digital_input {
        info.bit_depth = match (raw[20] >> 4) & 0x07 {
            1 => 6,
            2 => 8,
            3 => 10,
            4 => 12,
            5 => 14,
            6 => 16,
            _ => 0,
        };
    }

    // Max physical size
    info.max_h_size_cm = raw[21];
    info.max_v_size_cm = raw[22];

    // Standard timings (bytes 38-53, 8 entries of 2 bytes)
    for i in 0..8 {
        let b1 = raw[38 + i * 2];
        let b2 = raw[39 + i * 2];
        if b1 == 0x01 && b2 == 0x01 { continue; } // Unused
        if b1 == 0x00 { continue; }

        let width = (b1 as u32 + 31) * 8;
        let aspect = (b2 >> 6) & 0x03;
        let height = match aspect {
            0 => width * 10 / 16, // 16:10
            1 => width * 3 / 4,   // 4:3
            2 => width * 4 / 5,   // 5:4
            _ => width * 9 / 16,  // 16:9
        };
        let refresh = (b2 & 0x3F) as u32 + 60;

        info.supported_modes.push(DisplayMode { width, height, refresh });
    }

    // Detailed Timing Descriptors (bytes 54-125, four 18-byte blocks)
    for block in 0..4 {
        let offset = 54 + block * 18;
        let pixel_clock = u16::from_le_bytes([raw[offset], raw[offset + 1]]);

        if pixel_clock == 0 {
            // Monitor descriptor (not timing)
            let tag = raw[offset + 3];
            match tag {
                0xFC => {
                    // Monitor name
                    info.monitor_name = extract_descriptor_string(&raw[offset + 5..offset + 18]);
                }
                0xFF => {
                    // Monitor serial
                    info.monitor_serial = extract_descriptor_string(&raw[offset + 5..offset + 18]);
                }
                _ => {}
            }
        } else if block == 0 {
            // First detailed timing = preferred mode
            let h_active = raw[offset + 2] as u32 | (((raw[offset + 4] >> 4) as u32) << 8);
            let h_blank = raw[offset + 3] as u32 | (((raw[offset + 4] & 0x0F) as u32) << 8);
            let v_active = raw[offset + 5] as u32 | (((raw[offset + 7] >> 4) as u32) << 8);
            let v_blank = raw[offset + 6] as u32 | (((raw[offset + 7] & 0x0F) as u32) << 8);

            info.preferred_width = h_active;
            info.preferred_height = v_active;

            let total_pixels = (h_active + h_blank) * (v_active + v_blank);
            if total_pixels > 0 {
                info.preferred_refresh = (pixel_clock as u32 * 10000) / total_pixels;
            }
        }
    }

    Some(info)
}

fn extract_descriptor_string(data: &[u8]) -> String {
    let mut s = String::new();
    for &b in data {
        if b == 0x0A || b == 0x00 { break; }
        if b >= 0x20 && b < 0x7F {
            s.push(b as char);
        }
    }
    String::from(s.trim())
}
