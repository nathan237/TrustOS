//! MARIONET Auto-Dump — Detect USB, scan hardware, write report to USB key
//!
//! Usage: `marionet dump` — detects USB storage, runs full probe, writes report file.

use alloc::string::String;
use alloc::format;
use alloc::sync::Arc;

use crate::marionet::probe;

/// Run the auto-dump workflow: detect USB → probe → write report → sync
pub fn cmd_autodump() {
    run_autodump(false);
}

/// Boot-time auto-dump: silent if no USB found, verbose if USB detected
pub fn boot_autodump() {
    run_autodump(true);
}

fn run_autodump(auto_mode: bool) {
    crate::println_color!(crate::framebuffer::COLOR_CYAN,
        "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(crate::framebuffer::COLOR_CYAN,
        "║           MARIONET — USB Auto-Dump                         ║");
    crate::println_color!(crate::framebuffer::COLOR_CYAN,
        "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();

    // Step 1: Detect USB storage
    crate::print_color!(crate::framebuffer::COLOR_YELLOW, "[1/5] ");
    crate::println!("Detecting USB storage...");

    let usb_count = crate::drivers::usb_storage::device_count();
    if usb_count == 0 {
        if auto_mode {
            // Silent skip at boot — no USB, no fuss
            return;
        }
        crate::println_color!(crate::framebuffer::COLOR_RED,
            "  ERROR: No USB storage device detected.");
        crate::println!("  Plug in a USB key and try again.");
        return;
    }

    let usb_list = crate::drivers::usb_storage::list_devices();
    for (i, (name, blocks, bsize)) in usb_list.iter().enumerate() {
        let size_mb = (*blocks as u64) * (*bsize as u64) / 1024 / 1024;
        crate::println_color!(crate::framebuffer::COLOR_GREEN,
            "  USB #{}: {} — {} MB ({} sectors × {} B)",
            i, name, size_mb, blocks, bsize);
    }

    // Step 2: Mount FAT32 on first USB device
    crate::print_color!(crate::framebuffer::COLOR_YELLOW, "[2/5] ");
    crate::println!("Mounting FAT32 filesystem...");

    let block_dev = match crate::drivers::usb_storage::get_block_device(0) {
        Some(dev) => dev,
        None => {
            crate::println_color!(crate::framebuffer::COLOR_RED,
                "  ERROR: Could not open USB device #0.");
            return;
        }
    };

    let fat32 = match crate::vfs::fat32::Fat32Fs::mount(Arc::new(block_dev)) {
        Ok(fs) => fs,
        Err(e) => {
            crate::println_color!(crate::framebuffer::COLOR_RED,
                "  ERROR: FAT32 mount failed: {:?}", e);
            crate::println!("  Make sure the USB key has a FAT32 partition.");
            return;
        }
    };

    let mount_path = "/usb_marionet";
    if let Err(e) = crate::vfs::mount(mount_path, Arc::new(fat32)) {
        crate::println_color!(crate::framebuffer::COLOR_RED,
            "  ERROR: VFS mount failed: {:?}", e);
        return;
    }
    crate::println_color!(crate::framebuffer::COLOR_GREEN, "  Mounted at {}", mount_path);

    // Step 3: Full hardware scan
    crate::print_color!(crate::framebuffer::COLOR_YELLOW, "[3/5] ");
    crate::println!("Running full hardware scan...");

    let data = probe::collect_all();
    crate::println_color!(crate::framebuffer::COLOR_GREEN, "  Scan complete.");

    // Step 4: Generate and write report
    crate::print_color!(crate::framebuffer::COLOR_YELLOW, "[4/5] ");
    crate::println!("Writing report to USB...");

    let dt = crate::rtc::read_rtc();
    let filename = format!("{}/MARIONET_{:04}{:02}{:02}_{:02}{:02}{:02}.txt",
        mount_path,
        dt.year, dt.month, dt.day,
        dt.hour, dt.minute, dt.second);

    let report = generate_report(&data, &dt);

    match crate::vfs::write_file(&filename, report.as_bytes()) {
        Ok(()) => {
            crate::println_color!(crate::framebuffer::COLOR_GREEN,
                "  Written: {} ({} bytes)", filename, report.len());
        }
        Err(e) => {
            crate::println_color!(crate::framebuffer::COLOR_RED,
                "  ERROR: Write failed: {:?}", e);
            // Fallback: dump to serial
            crate::serial_println!("{}", report);
            crate::println!("  Report dumped to serial (COM1) instead.");
            return;
        }
    }

    // Step 5: Sync
    crate::print_color!(crate::framebuffer::COLOR_YELLOW, "[5/5] ");
    crate::println!("Syncing filesystem...");

    if let Err(e) = crate::vfs::sync_all() {
        crate::println_color!(crate::framebuffer::COLOR_RED,
            "  WARN: Sync error: {:?}", e);
    }

    crate::println!();
    crate::println_color!(crate::framebuffer::COLOR_GREEN,
        "  Done! Report saved to USB key.");
    crate::println_color!(crate::framebuffer::COLOR_GRAY,
        "  You can safely remove the USB drive.");
}

/// Generate a full plaintext report from collected system data
fn generate_report(data: &probe::SystemData, dt: &crate::rtc::DateTime) -> String {
    let mut r = String::with_capacity(4096);

    r.push_str("================================================================\n");
    r.push_str("  MARIONET — Hardware Report\n");
    r.push_str("  TrustOS Bare-Metal Diagnostics\n");
    r.push_str(&format!("  Generated: {}\n", dt.format()));
    r.push_str(&format!("  Uptime: {} seconds\n", crate::time::uptime_secs()));
    r.push_str("================================================================\n\n");

    // CPU
    r.push_str("── CPU ─────────────────────────────────────────────────────────\n");
    r.push_str(&format!("  Brand:      {}\n", data.cpu.brand));
    r.push_str(&format!("  Vendor:     {}\n", data.cpu.vendor));
    r.push_str(&format!("  Family:     {}  Model: {}  Stepping: {}\n",
        data.cpu.family, data.cpu.model, data.cpu.stepping));
    r.push_str(&format!("  Cores:      {} logical\n", data.cpu.cores));
    r.push_str(&format!("  TSC:        {} MHz\n", data.cpu.tsc_freq_mhz));
    r.push_str(&format!("  Features:   {}\n", data.cpu.features.join(", ")));
    r.push('\n');

    // Memory
    r.push_str("── MEMORY ──────────────────────────────────────────────────────\n");
    r.push_str(&format!("  Total RAM:  {} MB ({} bytes)\n", data.memory.total_mb, data.memory.total_bytes));
    r.push_str(&format!("  Heap used:  {} bytes\n", data.memory.heap_used));
    r.push_str(&format!("  Heap free:  {} bytes\n", data.memory.heap_free));
    r.push_str(&format!("  Heap total: {} bytes\n", data.memory.heap_total));
    for region in &data.memory.regions {
        r.push_str(&format!("  {}\n", region));
    }
    r.push('\n');

    // PCI
    r.push_str(&format!("── PCI DEVICES ({}) ──────────────────────────────────────────\n",
        data.pci_devices.len()));
    for d in &data.pci_devices {
        r.push_str(&format!("  {:02x}:{:02x}.{} [{:04x}:{:04x}] {} — {}\n",
            d.bus, d.device, d.function, d.vendor_id, d.device_id,
            d.vendor_name, d.class_name));
        if d.irq_line != 0 && d.irq_line != 0xFF {
            r.push_str(&format!("           IRQ line={} pin={}\n", d.irq_line, d.irq_pin));
        }
    }
    r.push('\n');

    // IRQ / Interrupts
    r.push_str("── INTERRUPTS ──────────────────────────────────────────────────\n");
    r.push_str(&format!("  Local APIC: 0x{:08X}\n", data.irq.local_apic_addr));
    r.push_str(&format!("  I/O APICs:  {}\n", data.irq.io_apic_count));
    r.push_str(&format!("  CPUs:       {}\n", data.irq.cpu_count));
    r.push_str(&format!("  Overrides:  {}\n", data.irq.override_count));
    for line in &data.irq.details {
        r.push_str(&format!("  {}\n", line));
    }
    r.push('\n');

    // Thermal
    r.push_str("── THERMAL ─────────────────────────────────────────────────────\n");
    if let Some(t) = data.thermal.cpu_temp {
        r.push_str(&format!("  CPU Temp:   ~{}°C\n", t));
    } else {
        r.push_str("  CPU Temp:   (unavailable)\n");
    }
    r.push_str(&format!("  TjMax:      {}°C\n", data.thermal.tj_max));
    for line in &data.thermal.details {
        r.push_str(&format!("  {}\n", line));
    }
    r.push('\n');

    // Storage
    r.push_str("── STORAGE ─────────────────────────────────────────────────────\n");
    if data.storage.devices.is_empty() {
        r.push_str("  (none detected)\n");
    } else {
        for s in &data.storage.devices {
            r.push_str(&format!("  {}\n", s));
        }
    }
    r.push('\n');

    // Network
    r.push_str("── NETWORK ─────────────────────────────────────────────────────\n");
    if data.network.interfaces.is_empty() {
        r.push_str("  (none detected)\n");
    } else {
        for n in &data.network.interfaces {
            r.push_str(&format!("  {}\n", n));
        }
    }
    r.push('\n');

    r.push_str("================================================================\n");
    r.push_str("  End of MARIONET report\n");
    r.push_str("================================================================\n");

    r
}
