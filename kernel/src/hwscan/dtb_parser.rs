//! Flattened Device Tree (FDT/DTB) Full Parser
//!
//! This is a proper DTB parser that walks the tree structure and extracts
//! all hardware descriptions from the Device Tree Blob. The DTB is the
//! **single source of truth** for what hardware exists on an ARM system.
//!
//! What we extract:
//!   - /model and /compatible: SoC identification
//!   - /memory: RAM base + size
//!   - /chosen/stdout-path: Console UART address
//!   - /chosen/linux,initrd-*: Ramdisk location
//!   - /soc/*: All peripherals with their MMIO addresses
//!   - /reserved-memory: Firmware/TZ carveouts (what we CAN'T touch)
//!   - SimpleFB: /chosen/framebuffer or /simple-framebuffer node
//!
//! Format reference: https://devicetree-specification.readthedocs.io/

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// FDT tokens
const FDT_BEGIN_NODE: u32 = 0x00000001;
const FDT_END_NODE: u32   = 0x00000002;
const FDT_PROP: u32        = 0x00000003;
const FDT_NOP: u32         = 0x00000004;
const FDT_END: u32         = 0x00000009;

/// FDT magic
const FDT_MAGIC: u32 = 0xD00DFEED;

/// FDT header (big-endian on disk)
#[repr(C)]
pub struct FdtHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

/// A single MMIO device discovered from DTB
#[derive(Debug, Clone)]
pub struct DtbDevice {
    /// Node path (e.g., "/soc/uart@9000000")
    pub path: String,
    /// Compatible string (e.g., "arm,pl011")
    pub compatible: String,
    /// MMIO base address
    pub reg_base: u64,
    /// MMIO region size
    pub reg_size: u64,
    /// Interrupt numbers (if any)
    pub interrupts: Vec<u32>,
    /// Status ("okay", "disabled", etc.)
    pub status: String,
}

/// Reserved memory region (TrustZone carveouts, firmware, etc.)
#[derive(Debug, Clone)]
pub struct ReservedRegion {
    pub name: String,
    pub base: u64,
    pub size: u64,
    pub no_map: bool,
}

/// SimpleFB framebuffer info from DTB
#[derive(Debug, Clone)]
pub struct SimpleFbInfo {
    pub base: u64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: String,
}

/// Complete parsed DTB data 
#[derive(Debug, Clone)]
pub struct ParsedDtb {
    /// SoC model string
    pub model: String,
    /// Compatible strings (can have multiple)
    pub compatible: Vec<String>,
    /// Physical RAM regions
    pub memory: Vec<(u64, u64)>,
    /// Stdout path (UART console)
    pub stdout_path: String,
    /// Console UART base derived from stdout_path
    pub uart_base: u64,
    /// All discovered devices with MMIO registers
    pub devices: Vec<DtbDevice>,
    /// Reserved memory regions (TZ, firmware, etc.)
    pub reserved: Vec<ReservedRegion>,
    /// SimpleFB info (if available)
    pub simplefb: Option<SimpleFbInfo>,
    /// Total DTB size in bytes
    pub dtb_size: u32,
    /// Number of nodes parsed
    pub node_count: u32,
    /// Raw chosen/bootargs string
    pub bootargs: String,
}

impl ParsedDtb {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            compatible: Vec::new(),
            memory: Vec::new(),
            stdout_path: String::new(),
            uart_base: 0,
            devices: Vec::new(),
            reserved: Vec::new(),
            simplefb: None,
            dtb_size: 0,
            node_count: 0,
            bootargs: String::new(),
        }
    }
}

/// Read a big-endian u32 from a byte pointer
unsafe fn be32(ptr: *const u8) -> u32 {
    let b = core::slice::from_raw_parts(ptr, 4);
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

/// Read a big-endian u64 from two consecutive be32 words
unsafe fn be64(ptr: *const u8) -> u64 {
    let hi = be32(ptr) as u64;
    let lo = be32(ptr.add(4)) as u64;
    (hi << 32) | lo
}

/// Read a NUL-terminated string from a byte pointer (max len)
unsafe fn read_cstr(ptr: *const u8, max: usize) -> String {
    let mut len = 0;
    while len < max && *ptr.add(len) != 0 {
        len += 1;
    }
    let bytes = core::slice::from_raw_parts(ptr, len);
    String::from_utf8_lossy(bytes).into_owned()
}

/// Align offset up to 4-byte boundary 
fn align4(offset: u32) -> u32 {
    (offset + 3) & !3
}

/// Try to extract a hex address from a node name like "uart@9000000"
fn parse_unit_addr(name: &str) -> Option<u64> {
    if let Some(at_pos) = name.find('@') {
        let hex_str = &name[at_pos + 1..];
        u64::from_str_radix(hex_str, 16).ok()
    } else {
        None
    }
}

/// Parse a complete DTB blob from a physical address
///
/// # Safety
/// `dtb_ptr` must point to a valid FDT blob in readable memory.
pub unsafe fn parse_dtb(dtb_ptr: *const u8) -> Option<ParsedDtb> {
    if dtb_ptr.is_null() {
        return None;
    }

    // Validate magic 
    let magic = be32(dtb_ptr);
    if magic != FDT_MAGIC {
        return None;
    }

    let totalsize = be32(dtb_ptr.add(4));
    let off_struct = be32(dtb_ptr.add(8));
    let off_strings = be32(dtb_ptr.add(12));

    // Sanity checks
    if totalsize > 16 * 1024 * 1024 || off_struct >= totalsize || off_strings >= totalsize {
        return None;
    }

    let struct_base = dtb_ptr.add(off_struct as usize);
    let strings_base = dtb_ptr.add(off_strings as usize);

    let mut result = ParsedDtb::new();
    result.dtb_size = totalsize;

    // Walk the structure block
    let mut offset: u32 = 0;
    let max_offset = totalsize - off_struct;
    let mut path_stack: Vec<String> = Vec::new();
    let mut current_path = String::new();

    // Per-node property tracking
    let mut cur_compatible = String::new();
    let mut cur_reg_base: u64 = 0;
    let mut cur_reg_size: u64 = 0;
    let mut cur_status = String::from("okay");
    let mut cur_interrupts: Vec<u32> = Vec::new();
    let mut in_memory_node = false;
    let mut in_reserved_memory = false;
    let mut in_chosen = false;
    let mut in_simplefb = false;

    // SimpleFB accumulation
    let mut sfb_base: u64 = 0;
    let mut sfb_size: u64 = 0;
    let mut sfb_width: u32 = 0;
    let mut sfb_height: u32 = 0;
    let mut sfb_stride: u32 = 0;
    let mut sfb_format = String::new();

    // address-cells / size-cells for current context
    let mut addr_cells: u32 = 2;
    let mut size_cells: u32 = 1;

    while offset + 4 <= max_offset {
        let token = be32(struct_base.add(offset as usize));
        offset += 4;

        match token {
            FDT_BEGIN_NODE => {
                let name = read_cstr(struct_base.add(offset as usize), 256);
                let name_len = name.len() as u32 + 1; // +1 for NUL
                offset = align4(offset + name_len);

                // Build path
                if name.is_empty() {
                    current_path = String::from("/");
                } else {
                    if current_path == "/" {
                        current_path = format!("/{}", name);
                    } else {
                        current_path = format!("{}/{}", current_path, name);
                    }
                }
                path_stack.push(current_path.clone());
                result.node_count += 1;

                // Reset per-node properties
                cur_compatible = String::new();
                cur_reg_base = 0;
                cur_reg_size = 0;
                cur_status = String::from("okay");
                cur_interrupts = Vec::new();

                // Detect special nodes
                in_memory_node = name.starts_with("memory");
                in_chosen = name == "chosen";
                in_reserved_memory = name == "reserved-memory" || current_path.contains("/reserved-memory/");
                in_simplefb = name.contains("framebuffer") || name.contains("simple-framebuffer");
            }

            FDT_END_NODE => {
                // Before popping, save device info if it has a reg property
                let is_device = cur_reg_base != 0 || cur_reg_size != 0;

                if in_memory_node && (cur_reg_base != 0 || cur_reg_size != 0) {
                    result.memory.push((cur_reg_base, cur_reg_size));
                } else if in_reserved_memory && !current_path.ends_with("reserved-memory") && cur_reg_base != 0 {
                    result.reserved.push(ReservedRegion {
                        name: path_stack.last().cloned().unwrap_or_default(),
                        base: cur_reg_base,
                        size: cur_reg_size,
                        no_map: false, // TODO: check for no-map property
                    });
                } else if in_simplefb {
                    if sfb_base == 0 { sfb_base = cur_reg_base; }
                    if sfb_size == 0 { sfb_size = cur_reg_size; }
                    if sfb_width > 0 && sfb_height > 0 {
                        result.simplefb = Some(SimpleFbInfo {
                            base: sfb_base,
                            size: sfb_size,
                            width: sfb_width,
                            height: sfb_height,
                            stride: sfb_stride,
                            format: sfb_format.clone(),
                        });
                    }
                    in_simplefb = false;
                } else if is_device && !cur_compatible.is_empty() {
                    result.devices.push(DtbDevice {
                        path: current_path.clone(),
                        compatible: cur_compatible.clone(),
                        reg_base: cur_reg_base,
                        reg_size: cur_reg_size,
                        interrupts: cur_interrupts.clone(),
                        status: cur_status.clone(),
                    });
                }

                // Pop path
                path_stack.pop();
                current_path = path_stack.last().cloned().unwrap_or(String::from("/"));
                in_memory_node = false;
                in_chosen = current_path.contains("chosen");
            }

            FDT_PROP => {
                if offset + 8 > max_offset { break; }
                let prop_len = be32(struct_base.add(offset as usize));
                let name_off = be32(struct_base.add(offset as usize + 4));
                offset += 8;

                let prop_name = read_cstr(strings_base.add(name_off as usize), 128);
                let prop_data = struct_base.add(offset as usize);
                offset = align4(offset + prop_len);

                // Handle properties
                match prop_name.as_str() {
                    "model" if path_stack.len() <= 1 => {
                        result.model = read_cstr(prop_data, prop_len as usize);
                    }
                    "compatible" => {
                        let compat = read_cstr(prop_data, prop_len as usize);
                        if path_stack.len() <= 1 {
                            // Root compatible — split by NUL
                            let bytes = core::slice::from_raw_parts(prop_data, prop_len as usize);
                            for chunk in bytes.split(|&b| b == 0) {
                                if !chunk.is_empty() {
                                    result.compatible.push(String::from_utf8_lossy(chunk).into_owned());
                                }
                            }
                        }
                        cur_compatible = compat;
                    }
                    "reg" => {
                        // Parse reg property based on #address-cells / #size-cells
                        if prop_len >= 4 {
                            if addr_cells == 2 && prop_len >= 8 {
                                cur_reg_base = be64(prop_data);
                            } else {
                                cur_reg_base = be32(prop_data) as u64;
                            }

                            let size_offset = (addr_cells * 4) as usize;
                            if (size_offset + 4) <= prop_len as usize {
                                if size_cells == 2 && (size_offset + 8) <= prop_len as usize {
                                    cur_reg_size = be64(prop_data.add(size_offset));
                                } else {
                                    cur_reg_size = be32(prop_data.add(size_offset)) as u64;
                                }
                            }
                        }

                        // SimpleFB reg
                        if in_simplefb {
                            sfb_base = cur_reg_base;
                            sfb_size = cur_reg_size;
                        }
                    }
                    "#address-cells" => {
                        if prop_len >= 4 {
                            addr_cells = be32(prop_data);
                        }
                    }
                    "#size-cells" => {
                        if prop_len >= 4 {
                            size_cells = be32(prop_data);
                        }
                    }
                    "status" => {
                        cur_status = read_cstr(prop_data, prop_len as usize);
                    }
                    "interrupts" | "interrupts-extended" => {
                        let count = prop_len / 4;
                        for i in 0..count {
                            cur_interrupts.push(be32(prop_data.add(i as usize * 4)));
                        }
                    }
                    "stdout-path" if in_chosen => {
                        result.stdout_path = read_cstr(prop_data, prop_len as usize);
                        // Extract UART address from stdout-path
                        // Common formats: "/soc/uart@9000000" or "serial0:115200n8"
                        if let Some(addr) = parse_unit_addr(&result.stdout_path) {
                            result.uart_base = addr;
                        }
                    }
                    "bootargs" if in_chosen => {
                        result.bootargs = read_cstr(prop_data, prop_len as usize);
                    }
                    // SimpleFB properties
                    "width" if in_simplefb => {
                        if prop_len >= 4 { sfb_width = be32(prop_data); }
                    }
                    "height" if in_simplefb => {
                        if prop_len >= 4 { sfb_height = be32(prop_data); }
                    }
                    "stride" if in_simplefb => {
                        if prop_len >= 4 { sfb_stride = be32(prop_data); }
                    }
                    "format" if in_simplefb => {
                        sfb_format = read_cstr(prop_data, prop_len as usize);
                    }
                    _ => {}
                }
            }

            FDT_NOP => { /* skip */ }
            FDT_END => break,
            _ => {
                // Unknown token — likely corrupt, bail
                break;
            }
        }

        // Safety: prevent infinite loop on corrupt DTB
        if offset > max_offset {
            break;
        }
    }

    Some(result)
}

/// Format parsed DTB as human-readable report
pub fn format_dtb_report(dtb: &ParsedDtb) -> String {
    let mut out = String::new();

    out.push_str("\x01C== Device Tree Blob (DTB) Report ==\x01W\n\n");
    out.push_str(&format!("Model: {}\n", if dtb.model.is_empty() { "(unknown)" } else { &dtb.model }));

    if !dtb.compatible.is_empty() {
        out.push_str("Compatible: ");
        for (i, c) in dtb.compatible.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            out.push_str(c);
        }
        out.push('\n');
    }

    out.push_str(&format!("DTB size: {} bytes  |  {} nodes parsed\n", dtb.dtb_size, dtb.node_count));

    if !dtb.bootargs.is_empty() {
        out.push_str(&format!("Bootargs: {}\n", dtb.bootargs));
    }
    if !dtb.stdout_path.is_empty() {
        out.push_str(&format!("Console: {} (UART @ 0x{:X})\n", dtb.stdout_path, dtb.uart_base));
    }

    // Memory
    if !dtb.memory.is_empty() {
        out.push_str("\n\x01Y--- Physical Memory ---\x01W\n");
        for (base, size) in &dtb.memory {
            let mb = size / (1024 * 1024);
            out.push_str(&format!("  0x{:010X} - 0x{:010X}  ({} MB)\n", base, base + size, mb));
        }
    }

    // Reserved memory (TrustZone carveouts!)
    if !dtb.reserved.is_empty() {
        out.push_str("\n\x01R--- Reserved Memory (Firmware / TrustZone) ---\x01W\n");
        for r in &dtb.reserved {
            let kb = r.size / 1024;
            out.push_str(&format!("  0x{:010X} - 0x{:010X}  ({:>6} KB) {}\n",
                r.base, r.base + r.size, kb, r.name));
        }
        out.push_str(&format!("  Total reserved: {} regions\n", dtb.reserved.len()));
    }

    // SimpleFB
    if let Some(ref sfb) = dtb.simplefb {
        out.push_str("\n\x01G--- SimpleFB Framebuffer ---\x01W\n");
        out.push_str(&format!("  Base: 0x{:010X}  Size: {} bytes\n", sfb.base, sfb.size));
        out.push_str(&format!("  Resolution: {}x{}  Stride: {}  Format: {}\n",
            sfb.width, sfb.height, sfb.stride, sfb.format));
    }

    // Devices
    if !dtb.devices.is_empty() {
        out.push_str("\n\x01Y--- Discovered Peripherals ---\x01W\n");
        out.push_str(&format!("{:<40} {:<14} {:<10} {}\n",
            "PATH", "REG BASE", "SIZE", "COMPATIBLE"));
        out.push_str(&format!("{}\n", "-".repeat(90)));

        for dev in &dtb.devices {
            let status_icon = match dev.status.as_str() {
                "okay" | "ok" => "\x01G",
                "disabled" => "\x01R",
                _ => "\x01Y",
            };
            out.push_str(&format!("{}{:<40}\x01W 0x{:010X}  {:<10} {}\n",
                status_icon,
                if dev.path.len() > 39 { &dev.path[dev.path.len()-39..] } else { &dev.path },
                dev.reg_base,
                format!("0x{:X}", dev.reg_size),
                dev.compatible));
        }
        out.push_str(&format!("\nTotal: {} devices ({} enabled)\n",
            dtb.devices.len(),
            dtb.devices.iter().filter(|d| d.status == "okay" || d.status == "ok").count()));
    }

    out
}
