//! HTML Report Generator — self-contained hardware diagnostic report
//!
//! Generates a single HTML file with embedded CSS that can be opened in any browser.
//! Includes all data from hwdiag + marionet probes.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;

use crate::marionet::probe;

/// Generate a complete HTML report from collected system data
pub fn generate_html_report(data: &probe::SystemData) -> String {
    let dt = crate::rtc::read_rtc();
    let uptime = crate::time::uptime_secs();

    let mut h = String::with_capacity(16384);

    // HTML header with embedded CSS
    h.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    h.push_str("<title>TrustOS Hardware Report</title>\n");
    h.push_str("<style>\n");
    h.push_str(CSS);
    h.push_str("\n</style>\n</head>\n<body>\n");

    // Header
    h.push_str("<div class=\"header\">\n");
    h.push_str("<h1>TrustOS Hardware Report</h1>\n");
    h.push_str("<p class=\"subtitle\">Bare-Metal Hardware Diagnostics</p>\n");
    h.push_str(&format!("<p class=\"meta\">Generated: {} &mdash; Uptime: {}s</p>\n", dt.format(), uptime));
    h.push_str("</div>\n\n");

    // SMBIOS / System Identity
    if let Some(smbios) = super::smbios::get_info() {
        h.push_str("<div class=\"section\">\n<h2>System Identity (SMBIOS/DMI)</h2>\n");
        h.push_str("<table>\n");
        row(&mut h, "Manufacturer", &smbios.system.manufacturer);
        row(&mut h, "Product", &smbios.system.product_name);
        row(&mut h, "Version", &smbios.system.version);
        row(&mut h, "Serial Number", &smbios.system.serial_number);
        row(&mut h, "BIOS Vendor", &smbios.bios.vendor);
        row(&mut h, "BIOS Version", &smbios.bios.version);
        row(&mut h, "BIOS Date", &smbios.bios.release_date);
        row(&mut h, "Board", &format!("{} {}", smbios.baseboard.manufacturer, smbios.baseboard.product));
        row(&mut h, "Chassis", &format!("{} ({})", smbios.chassis.manufacturer,
            super::smbios::chassis_type_name(smbios.chassis.chassis_type)));
        h.push_str("</table>\n</div>\n\n");
    }

    // CPU
    h.push_str("<div class=\"section\">\n<h2>CPU</h2>\n");
    h.push_str("<table>\n");
    row(&mut h, "Brand", &data.cpu.brand);
    row(&mut h, "Vendor", &data.cpu.vendor);
    row(&mut h, "Family/Model/Stepping", &format!("{}/{}/{}", data.cpu.family, data.cpu.model, data.cpu.stepping));
    row(&mut h, "Logical Cores", &format!("{}", data.cpu.cores));
    row(&mut h, "TSC Frequency", &format!("{} MHz", data.cpu.tsc_freq_mhz));
    row(&mut h, "Features", &data.cpu.features.join(", "));
    h.push_str("</table>\n</div>\n\n");

    // Memory
    h.push_str("<div class=\"section\">\n<h2>Memory</h2>\n");
    h.push_str("<table>\n");
    row(&mut h, "Total RAM", &format!("{} MB ({} bytes)", data.memory.total_mb, data.memory.total_bytes));
    row(&mut h, "Heap Used", &format!("{} bytes", data.memory.heap_used));
    row(&mut h, "Heap Free", &format!("{} bytes", data.memory.heap_free));
    row(&mut h, "Heap Total", &format!("{} bytes", data.memory.heap_total));
    h.push_str("</table>\n");

    // SMBIOS memory slots
    if let Some(smbios) = super::smbios::get_info() {
        if !smbios.memory_devices.is_empty() {
            h.push_str("<h3>Memory Slots</h3>\n");
            h.push_str("<table class=\"full\">\n<tr><th>Slot</th><th>Size</th><th>Type</th><th>Speed</th><th>Manufacturer</th><th>Part</th></tr>\n");
            for mem in &smbios.memory_devices {
                if mem.size_mb == 0 { continue; }
                h.push_str(&format!("<tr><td>{}</td><td>{} MB</td><td>{}</td><td>{} MHz</td><td>{}</td><td>{}</td></tr>\n",
                    esc(&mem.locator), mem.size_mb,
                    super::smbios::memory_type_name(mem.memory_type),
                    mem.speed_mhz, esc(&mem.manufacturer), esc(&mem.part_number)));
            }
            h.push_str("</table>\n");
        }
    }
    h.push_str("</div>\n\n");

    // PCI Devices
    h.push_str("<div class=\"section\">\n");
    h.push_str(&format!("<h2>PCI Devices ({})</h2>\n", data.pci_devices.len()));
    h.push_str("<table class=\"full\">\n<tr><th>BDF</th><th>VID:DID</th><th>Vendor</th><th>Class</th><th>IRQ</th></tr>\n");
    for d in &data.pci_devices {
        let irq = if d.irq_line != 0 && d.irq_line != 0xFF {
            format!("{}", d.irq_line)
        } else {
            String::from("-")
        };
        h.push_str(&format!("<tr><td>{:02x}:{:02x}.{}</td><td>{:04x}:{:04x}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            d.bus, d.device, d.function, d.vendor_id, d.device_id,
            esc(&d.vendor_name), esc(&d.class_name), irq));
    }
    h.push_str("</table>\n</div>\n\n");

    // Interrupts
    h.push_str("<div class=\"section\">\n<h2>Interrupts</h2>\n");
    h.push_str("<table>\n");
    row(&mut h, "Local APIC", &format!("0x{:08X}", data.irq.local_apic_addr));
    row(&mut h, "I/O APICs", &format!("{}", data.irq.io_apic_count));
    row(&mut h, "CPUs", &format!("{}", data.irq.cpu_count));
    row(&mut h, "Overrides", &format!("{}", data.irq.override_count));
    h.push_str("</table>\n</div>\n\n");

    // Thermal
    h.push_str("<div class=\"section\">\n<h2>Thermal &amp; Power</h2>\n");
    h.push_str("<table>\n");
    if let Some(t) = data.thermal.cpu_temp {
        let color = if t > 85 { "hot" } else if t > 70 { "warm" } else { "cool" };
        row(&mut h, "CPU Temperature", &format!("<span class=\"{}\">{}°C</span>", color, t));
    }
    row(&mut h, "TjMax", &format!("{}°C", data.thermal.tj_max));
    for detail in &data.thermal.details {
        h.push_str(&format!("<tr><td colspan=\"2\">{}</td></tr>\n", esc(detail)));
    }
    h.push_str("</table>\n</div>\n\n");

    // Storage
    h.push_str("<div class=\"section\">\n<h2>Storage</h2>\n");
    if data.storage.devices.is_empty() {
        h.push_str("<p>(none detected)</p>\n");
    } else {
        h.push_str("<ul>\n");
        for s in &data.storage.devices {
            h.push_str(&format!("<li>{}</li>\n", esc(s)));
        }
        h.push_str("</ul>\n");
    }

    // SMART data
    let smart_data = super::smart::collect_all();
    if !smart_data.is_empty() {
        h.push_str("<h3>SMART Health</h3>\n");
        for sd in &smart_data {
            let health_class = if sd.health_ok { "ok" } else { "fail" };
            h.push_str(&format!("<div class=\"smart-drive\">\n<h4>{} &mdash; <span class=\"{}\">{}</span></h4>\n",
                esc(&sd.drive_name), health_class,
                if sd.health_ok { "PASSED" } else { "FAILING" }));
            h.push_str("<table>\n");
            if let Some(t) = sd.temperature_c { row(&mut h, "Temperature", &format!("{}°C", t)); }
            if let Some(hrs) = sd.power_on_hours { row(&mut h, "Power-On Hours", &format!("{} ({} days)", hrs, hrs / 24)); }
            if let Some(c) = sd.power_cycle_count { row(&mut h, "Power Cycles", &format!("{}", c)); }
            if let Some(r) = sd.reallocated_sectors {
                let cls = if r > 0 { " class=\"warn\"" } else { "" };
                h.push_str(&format!("<tr><td>Reallocated Sectors</td><td{}>{}</td></tr>\n", cls, r));
            }
            if let Some(p) = sd.pending_sectors {
                let cls = if p > 0 { " class=\"warn\"" } else { "" };
                h.push_str(&format!("<tr><td>Pending Sectors</td><td{}>{}</td></tr>\n", cls, p));
            }
            h.push_str("</table>\n</div>\n");
        }
    }
    h.push_str("</div>\n\n");

    // Network
    h.push_str("<div class=\"section\">\n<h2>Network</h2>\n");
    if data.network.interfaces.is_empty() {
        h.push_str("<p>(none detected)</p>\n");
    } else {
        h.push_str("<ul>\n");
        for n in &data.network.interfaces {
            h.push_str(&format!("<li>{}</li>\n", esc(n)));
        }
        h.push_str("</ul>\n");
    }
    h.push_str("</div>\n\n");

    // EFI Variables
    #[cfg(target_arch = "x86_64")]
    {
        let efi = super::efi_vars::collect_efi_info();
        if efi.available {
            h.push_str("<div class=\"section\">\n<h2>EFI/UEFI</h2>\n");
            h.push_str("<table>\n");
            row(&mut h, "Secure Boot", if efi.secure_boot { "Enabled" } else { "Disabled" });
            row(&mut h, "UEFI Version", &efi.uefi_version);
            row(&mut h, "Firmware Vendor", &efi.firmware_vendor);
            if !efi.boot_order.is_empty() {
                row(&mut h, "Boot Order", &efi.boot_order.iter()
                    .map(|x| format!("{:04X}", x)).collect::<Vec<_>>().join(", "));
            }
            h.push_str("</table>\n</div>\n\n");
        }
    }

    // Footer
    h.push_str("<div class=\"footer\">\n");
    h.push_str("<p>Generated by TrustOS HwDbg &mdash; Bare-Metal Hardware Debugger</p>\n");
    h.push_str("<p>Boot from USB, diagnose anything.</p>\n");
    h.push_str("</div>\n\n");

    h.push_str("</body>\n</html>\n");
    h
}

/// Run the HTML report command
pub fn run(args: &[&str]) {
    use super::dbg_out;

    dbg_out!("[HTML] === Generating HTML Hardware Report ===");

    let data = probe::collect_all();
    let html = generate_html_report(&data);

    // Try writing to USB if available
    if args.contains(&"usb") || args.contains(&"--usb") {
        if write_to_usb(&html) {
            dbg_out!("[HTML] Report written to USB");
            return;
        }
        dbg_out!("[HTML] USB write failed, dumping to serial");
    }

    // Dump to serial for capture
    dbg_out!("[HTML] Report size: {} bytes", html.len());
    dbg_out!("[HTML] === BEGIN HTML ===");
    crate::serial_println!("{}", html);
    dbg_out!("[HTML] === END HTML ===");
    dbg_out!("[HTML] Capture the output above from serial (COM1) to save as .html");
}

fn write_to_usb(html: &str) -> bool {
    use alloc::sync::Arc;

    let usb_count = crate::drivers::usb_storage::device_count();
    if usb_count == 0 { return false; }

    let block_dev = match crate::drivers::usb_storage::get_block_device(0) {
        Some(dev) => dev,
        None => return false,
    };

    let fat32 = match crate::vfs::fat32::Fat32Fs::mount(Arc::new(block_dev)) {
        Ok(fs) => fs,
        Err(_) => return false,
    };

    let mount_path = "/usb_report";
    if crate::vfs::mount(mount_path, Arc::new(fat32)).is_err() {
        return false;
    }

    let dt = crate::rtc::read_rtc();
    let filename = format!("{}/TRUSTOS_REPORT_{:04}{:02}{:02}_{:02}{:02}{:02}.html",
        mount_path, dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second);

    if crate::vfs::write_file(&filename, html.as_bytes()).is_err() {
        return false;
    }

    let _ = crate::vfs::sync_all();
    crate::println!("  HTML report written to {}", filename);
    true
}

// ─── Helpers ───────────────────────────────────────────────────────────────

fn row(h: &mut String, label: &str, value: &str) {
    h.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>\n", label, value));
}

/// Escape HTML special characters
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

// ─── Embedded CSS ──────────────────────────────────────────────────────────

const CSS: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: 'Segoe UI', system-ui, -apple-system, sans-serif; background: #0a0e17; color: #c8d6e5; line-height: 1.6; padding: 20px; max-width: 960px; margin: 0 auto; }
.header { text-align: center; padding: 30px 0; border-bottom: 2px solid #1e3a5f; margin-bottom: 30px; }
.header h1 { color: #00d4ff; font-size: 2em; letter-spacing: 2px; }
.subtitle { color: #7f8c8d; font-size: 0.9em; margin-top: 5px; }
.meta { color: #556677; font-size: 0.8em; margin-top: 10px; }
.section { background: #111b2a; border: 1px solid #1e3a5f; border-radius: 8px; padding: 20px; margin-bottom: 20px; }
.section h2 { color: #00d4ff; border-bottom: 1px solid #1e3a5f; padding-bottom: 8px; margin-bottom: 15px; font-size: 1.2em; }
.section h3 { color: #00aacc; margin: 15px 0 10px; font-size: 1em; }
.section h4 { color: #ddd; margin-bottom: 8px; }
table { width: 100%; border-collapse: collapse; }
table td, table th { padding: 6px 12px; text-align: left; border-bottom: 1px solid #1a2a3a; }
table th { background: #0d1620; color: #00d4ff; font-weight: 600; }
table tr:hover { background: #152238; }
table td:first-child { color: #8899aa; white-space: nowrap; width: 200px; }
table.full td:first-child { width: auto; color: inherit; }
ul { list-style: none; }
ul li { padding: 4px 0; padding-left: 15px; position: relative; }
ul li:before { content: "▸"; position: absolute; left: 0; color: #00d4ff; }
.ok { color: #2ecc71; font-weight: bold; }
.fail { color: #e74c3c; font-weight: bold; }
.warn { color: #f39c12; font-weight: bold; }
.hot { color: #e74c3c; font-weight: bold; }
.warm { color: #f39c12; }
.cool { color: #2ecc71; }
.smart-drive { margin: 10px 0; padding: 10px; background: #0d1620; border-radius: 4px; }
.footer { text-align: center; padding: 20px 0; color: #556677; font-size: 0.8em; border-top: 1px solid #1e3a5f; margin-top: 20px; }
@media print { body { background: #fff; color: #333; } .section { border-color: #ccc; background: #fafafa; } .header h1 { color: #0077aa; } }
"#;
