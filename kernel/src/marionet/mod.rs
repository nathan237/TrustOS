//! MARIONET — Bare-Metal Hardware Dashboard
//!
//! Full-screen interactive hardware visualization platform.
//! Renders directly to framebuffer with colored panels, live data,
//! and keyboard navigation between tabs.
//!
//! Commands:
//!   marionet              — Launch interactive dashboard
//!   marionet scan         — Quick scan, print summary to shell
//!   marionet probe <mod>  — Probe specific subsystem (cpu/pci/mem/irq/thermal/storage/net)
//!   marionet export       — Dump full report to serial

pub mod autodump;
pub mod probe;
mod render;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Dashboard tab identifiers
#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Overview,
    Cpu,
    Memory,
    Pci,
    Irq,
    Storage,
    Network,
    Thermal,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Tab::Overview => "OVERVIEW",
            Tab::Cpu      => "CPU",
            Tab::Memory   => "MEMORY",
            Tab::Pci      => "PCI",
            Tab::Irq      => "IRQ",
            Tab::Storage  => "STORAGE",
            Tab::Network  => "NET",
            Tab::Thermal  => "THERMAL",
        }
    }

    fn all() -> &'static [Tab] {
        &[Tab::Overview, Tab::Cpu, Tab::Memory, Tab::Pci,
          Tab::Irq, Tab::Storage, Tab::Network, Tab::Thermal]
    }

    fn next(self) -> Tab {
        let tabs = Self::all();
        let idx = tabs.iter().position(|&t| t == self).unwrap_or(0);
        tabs[(idx + 1) % tabs.len()]
    }

    fn prev(self) -> Tab {
        let tabs = Self::all();
        let idx = tabs.iter().position(|&t| t == self).unwrap_or(0);
        tabs[(idx + tabs.len() - 1) % tabs.len()]
    }
}

/// Main entry point from shell
pub fn handle_command(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("");

    match subcmd {
        "" | "dashboard" | "dash" => run_dashboard(),
        "dump" | "autopsy" | "auto" => autodump::cmd_autodump(),
        "scan" => cmd_scan(args.get(1..).unwrap_or(&[])),
        "probe" => cmd_probe(args.get(1..).unwrap_or(&[])),
        "export" => cmd_export(),
        "help" => cmd_help(),
        _ => {
            // Try as probe shortcut: "marionet cpu" = "marionet probe cpu"
            cmd_probe(args);
        }
    }
}

/// Interactive full-screen dashboard
fn run_dashboard() {
    use crate::framebuffer;

    let (screen_w, screen_h) = framebuffer::get_dimensions();
    if screen_w < 320 || screen_h < 200 {
        crate::println!("Screen too small for Marionet dashboard");
        return;
    }

    // Collect all data upfront
    let data = probe::collect_all();

    let mut current_tab = Tab::Overview;
    let mut scroll: usize = 0;
    let mut running = true;

    // Initial render
    render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll);

    while running {
        if let Some(key) = crate::keyboard::try_read_key() {
            match key {
                // ESC or 'q' to quit
                0x1B | b'q' | b'Q' => running = false,

                // Tab / Right arrow → next tab
                b'\t' | 0x4D => {   // 0x4D = right arrow scancode
                    current_tab = current_tab.next();
                    scroll = 0;
                    render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll);
                }

                // Shift-Tab / Left arrow → prev tab
                0x4B => {           // 0x4B = left arrow scancode
                    current_tab = current_tab.prev();
                    scroll = 0;
                    render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll);
                }

                // Number keys → jump to tab
                b'1' => { current_tab = Tab::Overview; scroll = 0; render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll); }
                b'2' => { current_tab = Tab::Cpu;      scroll = 0; render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll); }
                b'3' => { current_tab = Tab::Memory;   scroll = 0; render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll); }
                b'4' => { current_tab = Tab::Pci;      scroll = 0; render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll); }
                b'5' => { current_tab = Tab::Irq;      scroll = 0; render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll); }
                b'6' => { current_tab = Tab::Storage;   scroll = 0; render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll); }
                b'7' => { current_tab = Tab::Network;   scroll = 0; render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll); }
                b'8' => { current_tab = Tab::Thermal;   scroll = 0; render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll); }

                // Scroll up/down
                0x48 => { // Up arrow
                    if scroll > 0 { scroll -= 1; }
                    render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll);
                }
                0x50 => { // Down arrow
                    scroll += 1;
                    render::draw_dashboard(screen_w, screen_h, current_tab, &data, scroll);
                }

                // 'r' to refresh data
                b'r' | b'R' => {
                    let data_new = probe::collect_all();
                    // We can't reassign captured data easily, so just redraw
                    render::draw_dashboard(screen_w, screen_h, current_tab, &data_new, scroll);
                }

                _ => {}
            }
        }
        // Small sleep to avoid burning CPU
        for _ in 0..5000 { core::hint::spin_loop(); }
    }

    // Restore shell
    framebuffer::clear();
    framebuffer::set_cursor(0, 0);
    crate::println!("Marionet exited.");
}

/// Quick scan — print summary to shell output
fn cmd_scan(args: &[&str]) {
    let deep = args.contains(&"--deep") || args.contains(&"-d");

    crate::println_color!(crate::framebuffer::COLOR_CYAN,
        "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(crate::framebuffer::COLOR_CYAN,
        "║             MARIONET — Hardware Scan Report                 ║");
    crate::println_color!(crate::framebuffer::COLOR_CYAN,
        "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();

    let data = probe::collect_all();

    // CPU
    crate::print_color!(crate::framebuffer::COLOR_GREEN, "  CPU: ");
    crate::println!("{}", data.cpu.brand);
    crate::print_color!(crate::framebuffer::COLOR_GREEN, "       ");
    crate::println!("{} cores | Family {} Model {} Step {}",
        data.cpu.cores, data.cpu.family, data.cpu.model, data.cpu.stepping);

    // Memory
    crate::print_color!(crate::framebuffer::COLOR_GREEN, "  RAM: ");
    crate::println!("{} MB total", data.memory.total_mb);

    // PCI devices
    crate::print_color!(crate::framebuffer::COLOR_GREEN, "  PCI: ");
    crate::println!("{} devices", data.pci_devices.len());
    for dev in &data.pci_devices {
        crate::println!("       {:02x}:{:02x}.{} [{:04x}:{:04x}] {}",
            dev.bus, dev.device, dev.function, dev.vendor_id, dev.device_id, dev.class_name);
    }

    // IRQ
    crate::print_color!(crate::framebuffer::COLOR_GREEN, "  IRQ: ");
    crate::println!("{} I/O APIC(s), {} interrupt overrides",
        data.irq.io_apic_count, data.irq.override_count);

    // Storage
    crate::print_color!(crate::framebuffer::COLOR_GREEN, "  DISK:");
    if data.storage.devices.is_empty() {
        crate::println!(" (none detected)");
    } else {
        crate::println!();
        for s in &data.storage.devices {
            crate::println!("       {}", s);
        }
    }

    // Network
    crate::print_color!(crate::framebuffer::COLOR_GREEN, "  NET: ");
    if data.network.interfaces.is_empty() {
        crate::println!("(none detected)");
    } else {
        crate::println!();
        for n in &data.network.interfaces {
            crate::println!("       {}", n);
        }
    }

    // Thermal
    crate::print_color!(crate::framebuffer::COLOR_GREEN, "  TEMP:");
    if let Some(t) = data.thermal.cpu_temp {
        crate::println!(" CPU ~{}°C (TjMax={}°C)", t, data.thermal.tj_max);
    } else {
        crate::println!(" (unavailable)");
    }

    crate::println!();
    crate::println_color!(crate::framebuffer::COLOR_GRAY,
        "  Use 'marionet' for interactive dashboard");
}

/// Probe a specific subsystem
fn cmd_probe(args: &[&str]) {
    let target = args.first().copied().unwrap_or("help");
    match target {
        "cpu" => {
            let info = probe::collect_cpu();
            crate::println_color!(crate::framebuffer::COLOR_CYAN, "── CPU ──");
            crate::println!("Brand:    {}", info.brand);
            crate::println!("Vendor:   {}", info.vendor);
            crate::println!("Family:   {}  Model: {}  Stepping: {}", info.family, info.model, info.stepping);
            crate::println!("Cores:    {} logical", info.cores);
            crate::println!("Features: {}", info.features.join(", "));
        }
        "pci" => {
            let devs = probe::collect_pci();
            crate::println_color!(crate::framebuffer::COLOR_CYAN, "── PCI Devices ({}) ──", devs.len());
            for d in &devs {
                crate::println!("{:02x}:{:02x}.{} [{:04x}:{:04x}] {} — {}",
                    d.bus, d.device, d.function, d.vendor_id, d.device_id,
                    d.vendor_name, d.class_name);
            }
        }
        "mem" | "memory" => {
            let info = probe::collect_memory();
            crate::println_color!(crate::framebuffer::COLOR_CYAN, "── Memory ──");
            crate::println!("Total:    {} MB ({} bytes)", info.total_mb, info.total_bytes);
            for region in &info.regions {
                crate::println!("  {}", region);
            }
        }
        "irq" | "int" | "interrupts" => {
            let info = probe::collect_irq();
            crate::println_color!(crate::framebuffer::COLOR_CYAN, "── Interrupts ──");
            crate::println!("Local APIC: 0x{:X}", info.local_apic_addr);
            crate::println!("I/O APICs:  {}", info.io_apic_count);
            crate::println!("Overrides:  {}", info.override_count);
            for line in &info.details {
                crate::println!("  {}", line);
            }
        }
        "thermal" | "temp" => {
            let info = probe::collect_thermal();
            crate::println_color!(crate::framebuffer::COLOR_CYAN, "── Thermal ──");
            if let Some(t) = info.cpu_temp {
                crate::println!("CPU Temp:   ~{}°C", t);
            }
            crate::println!("TjMax:      {}°C", info.tj_max);
            for line in &info.details {
                crate::println!("  {}", line);
            }
        }
        "storage" | "disk" => {
            let info = probe::collect_storage();
            crate::println_color!(crate::framebuffer::COLOR_CYAN, "── Storage ──");
            for s in &info.devices {
                crate::println!("  {}", s);
            }
        }
        "net" | "network" => {
            let info = probe::collect_network();
            crate::println_color!(crate::framebuffer::COLOR_CYAN, "── Network ──");
            for n in &info.interfaces {
                crate::println!("  {}", n);
            }
        }
        _ => {
            crate::println!("Unknown probe target: {}", target);
            crate::println!("Available: cpu, pci, mem, irq, thermal, storage, net");
        }
    }
}

/// Export full report to serial
fn cmd_export() {
    crate::serial_println!("=== MARIONET FULL EXPORT ===");
    let data = probe::collect_all();

    crate::serial_println!("[CPU] {}", data.cpu.brand);
    crate::serial_println!("[CPU] Vendor={} Family={} Model={} Step={} Cores={}",
        data.cpu.vendor, data.cpu.family, data.cpu.model, data.cpu.stepping, data.cpu.cores);
    crate::serial_println!("[CPU] Features: {}", data.cpu.features.join(", "));

    crate::serial_println!("[MEM] Total={} MB", data.memory.total_mb);

    for dev in &data.pci_devices {
        crate::serial_println!("[PCI] {:02x}:{:02x}.{} [{:04x}:{:04x}] {} {}",
            dev.bus, dev.device, dev.function, dev.vendor_id, dev.device_id,
            dev.vendor_name, dev.class_name);
    }

    crate::serial_println!("[IRQ] LAPIC=0x{:X} IO-APIC={} Overrides={}",
        data.irq.local_apic_addr, data.irq.io_apic_count, data.irq.override_count);

    if let Some(t) = data.thermal.cpu_temp {
        crate::serial_println!("[THERM] CPU={}°C TjMax={}°C", t, data.thermal.tj_max);
    }

    crate::serial_println!("=== END EXPORT ===");
    crate::println!("Export sent to serial (COM1).");
}

fn cmd_help() {
    crate::println_color!(crate::framebuffer::COLOR_CYAN, "MARIONET — Hardware Dashboard");
    crate::println!();
    crate::println!("Usage:");
    crate::println!("  marionet              Launch interactive dashboard");
    crate::println!("  marionet scan         Quick hardware scan");
    crate::println!("  marionet probe <mod>  Probe module (cpu/pci/mem/irq/thermal/storage/net)");
    crate::println!("  marionet dump         Auto-detect USB, scan, dump report to USB key");
    crate::println!("  marionet export       Export full report to serial");
    crate::println!("  marionet help         This help");
    crate::println!();
    crate::println!("Dashboard controls:");
    crate::println!("  1-8        Jump to tab");
    crate::println!("  Tab/Right  Next tab");
    crate::println!("  Left       Previous tab");
    crate::println!("  Up/Down    Scroll");
    crate::println!("  R          Refresh data");
    crate::println!("  Q/ESC      Quit");
}
