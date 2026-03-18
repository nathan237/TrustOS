//! MARIONET Render — Full-screen dashboard drawing
//!
//! Pixel-level framebuffer rendering: header bar, tab strip, data panels,
//! progress bars, section boxes, and color-coded status indicators.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::framebuffer;
use super::probe::SystemData;
use super::Tab;

// ─── Color Palette ─────────────────────────────────────────────────────────

const BG_DARK:       u32 = 0xFF0A0A0F;   // Near-black background
const BG_PANEL:      u32 = 0xFF14141E;   // Panel background
const BG_HEADER:     u32 = 0xFF00CCAA;   // Teal header
const BG_TAB_ACTIVE: u32 = 0xFF00AAFF;   // Active tab
const BG_TAB:        u32 = 0xFF1A1A2E;   // Inactive tab
const FG_TITLE:      u32 = 0xFF000000;   // Black text on header
const FG_WHITE:      u32 = 0xFFFFFFFF;
const FG_GREEN:      u32 = 0xFF00FF88;
const FG_YELLOW:     u32 = 0xFFFFCC00;
const FG_RED:        u32 = 0xFFFF4444;
const FG_CYAN:       u32 = 0xFF00DDFF;
const FG_GRAY:       u32 = 0xFF666677;
const FG_DIM:        u32 = 0xFF444455;
const ACCENT:        u32 = 0xFF00FFAA;   // Green accent
const BAR_BG:        u32 = 0xFF222233;   // Progress bar background
const BAR_FG:        u32 = 0xFF00CC88;   // Progress bar fill
const BAR_WARN:      u32 = 0xFFFF8800;   // Progress bar warning
const BAR_CRIT:      u32 = 0xFFFF2222;   // Progress bar critical
const BORDER:        u32 = 0xFF333344;   // Panel border

const CHAR_W: u32 = 8;
const CHAR_H: u32 = 16;
const PAD: u32 = 8;

/// Draw the complete dashboard
pub fn draw_dashboard(w: u32, h: u32, tab: Tab, data: &SystemData, scroll: usize) {
    // Clear to dark background
    framebuffer::fill_rect(0, 0, w, h, BG_DARK);

    // Header bar (32px)
    draw_header(w);

    // Tab strip (24px)
    draw_tabs(w, tab);

    // Content area
    let content_y = 60u32;
    let content_h = h.saturating_sub(content_y + 24); // 24px for footer

    match tab {
        Tab::Overview => draw_overview(w, content_y, content_h, data, scroll),
        Tab::Cpu      => draw_cpu_tab(w, content_y, content_h, data, scroll),
        Tab::Memory   => draw_memory_tab(w, content_y, content_h, data, scroll),
        Tab::Pci      => draw_pci_tab(w, content_y, content_h, data, scroll),
        Tab::Irq      => draw_irq_tab(w, content_y, content_h, data, scroll),
        Tab::Storage  => draw_storage_tab(w, content_y, content_h, data, scroll),
        Tab::Network  => draw_network_tab(w, content_y, content_h, data, scroll),
        Tab::Thermal  => draw_thermal_tab(w, content_y, content_h, data, scroll),
    }

    // Footer
    draw_footer(w, h);
}

// ─── Header ────────────────────────────────────────────────────────────────

fn draw_header(w: u32) {
    framebuffer::fill_rect(0, 0, w, 30, BG_HEADER);
    framebuffer::draw_text_at("  MARIONET", 4, 7, FG_TITLE, BG_HEADER);

    let version = " v1.0 — TrustOS Hardware Dashboard ";
    let vx = 10 * CHAR_W + 4;
    framebuffer::draw_text_at(version, vx, 7, 0xFF005544, BG_HEADER);

    // Uptime right-aligned
    let up = crate::time::uptime_secs();
    let upstr = format!("up {}:{:02}:{:02} ", up / 3600, (up % 3600) / 60, up % 60);
    let rx = w.saturating_sub(upstr.len() as u32 * CHAR_W + 4);
    framebuffer::draw_text_at(&upstr, rx, 7, FG_TITLE, BG_HEADER);
}

// ─── Tab Strip ─────────────────────────────────────────────────────────────

fn draw_tabs(w: u32, active: Tab) {
    framebuffer::fill_rect(0, 30, w, 26, BG_TAB);
    // Bottom border
    framebuffer::fill_rect(0, 56, w, 1, BORDER);

    let mut x = 4u32;
    for (i, &tab) in Tab::all().iter().enumerate() {
        let label = format!(" {}:{} ", i + 1, tab.label());
        let tw = label.len() as u32 * CHAR_W + 4;

        if tab == active {
            framebuffer::fill_rect(x, 32, tw, 22, BG_TAB_ACTIVE);
            framebuffer::draw_text_at(&label, x + 2, 35, FG_WHITE, BG_TAB_ACTIVE);
        } else {
            framebuffer::draw_text_at(&label, x + 2, 35, FG_GRAY, BG_TAB);
        }
        x += tw + 4;
    }
}

// ─── Footer ────────────────────────────────────────────────────────────────

fn draw_footer(w: u32, h: u32) {
    let fy = h.saturating_sub(22);
    framebuffer::fill_rect(0, fy, w, 22, BG_PANEL);
    framebuffer::fill_rect(0, fy, w, 1, BORDER);
    framebuffer::draw_text_at(
        " [1-8] Tab  [</>] Navigate  [Up/Down] Scroll  [R] Refresh  [Q] Quit ",
        4, fy + 3, FG_DIM, BG_PANEL);
}

// ─── Overview Tab ──────────────────────────────────────────────────────────

fn draw_overview(w: u32, y0: u32, h: u32, data: &SystemData, _scroll: usize) {
    let col_w = (w - 24) / 2; // two columns
    let left_x = 8u32;
    let right_x = left_x + col_w + 8;

    // ── Left Column ──

    // CPU Box
    let mut y = y0 + 4;
    y = draw_panel_box(left_x, y, col_w, "CPU", &[
        (FG_WHITE, format!("{}", data.cpu.brand)),
        (FG_CYAN,  format!("{} | Family {} Model {} Step {}",
            data.cpu.vendor, data.cpu.family, data.cpu.model, data.cpu.stepping)),
        (FG_GREEN, format!("{} logical cores | TSC {} MHz",
            data.cpu.cores, data.cpu.tsc_freq_mhz)),
    ]);

    // Memory Box
    y += 4;
    let heap_pct = if data.memory.heap_total > 0 {
        (data.memory.heap_used * 100) / data.memory.heap_total
    } else { 0 };
    y = draw_panel_box(left_x, y, col_w, "MEMORY", &[
        (FG_WHITE, format!("Physical: {} MB", data.memory.total_mb)),
        (FG_CYAN,  format!("Heap: {} / {} KB ({:.0}%)",
            data.memory.heap_used / 1024,
            data.memory.heap_total / 1024,
            heap_pct)),
    ]);
    // Draw heap usage bar inside the panel
    draw_progress_bar(left_x + PAD, y - 4, col_w - PAD * 2, heap_pct as u32, 100);
    y += 14;

    // Thermal Box
    y += 4;
    let temp_lines: Vec<(u32, String)> = if let Some(t) = data.thermal.cpu_temp {
        let color = if t > 85 { FG_RED } else if t > 70 { FG_YELLOW } else { FG_GREEN };
        let mut v = alloc::vec![(color, format!("CPU Temperature: {}°C (TjMax={}°C)", t, data.thermal.tj_max))];
        for d in &data.thermal.details {
            v.push((FG_DIM, d.clone()));
        }
        v
    } else {
        alloc::vec![(FG_DIM, String::from("Temperature data unavailable"))]
    };
    y = draw_panel_box(left_x, y, col_w, "THERMAL", &temp_lines);

    // ── Right Column ──

    // PCI Summary Box
    let mut ry = y0 + 4;
    let mut pci_lines: Vec<(u32, String)> = Vec::new();
    pci_lines.push((FG_WHITE, format!("{} devices detected", data.pci_devices.len())));
    // Group by class
    let mut class_counts: Vec<(String, usize)> = Vec::new();
    for dev in &data.pci_devices {
        if let Some(entry) = class_counts.iter_mut().find(|(c, _)| *c == dev.class_name) {
            entry.1 += 1;
        } else {
            class_counts.push((dev.class_name.clone(), 1));
        }
    }
    for (cls, cnt) in &class_counts {
        pci_lines.push((FG_CYAN, format!("  {} x{}", cls, cnt)));
    }
    ry = draw_panel_box(right_x, ry, col_w, "PCI DEVICES", &pci_lines);

    // IRQ Box
    ry += 4;
    ry = draw_panel_box(right_x, ry, col_w, "INTERRUPTS", &[
        (FG_WHITE, format!("LAPIC @ 0x{:X}", data.irq.local_apic_addr)),
        (FG_CYAN,  format!("{} I/O APIC(s)  |  {} overrides",
            data.irq.io_apic_count, data.irq.override_count)),
        (FG_GREEN, format!("{} CPU(s) detected", data.irq.cpu_count)),
    ]);

    // Storage Box
    ry += 4;
    let stor_lines: Vec<(u32, String)> = data.storage.devices.iter()
        .map(|s| (FG_CYAN, s.clone()))
        .collect();
    ry = draw_panel_box(right_x, ry, col_w, "STORAGE", &stor_lines);

    // Network Box
    ry += 4;
    let net_lines: Vec<(u32, String)> = data.network.interfaces.iter()
        .map(|s| (FG_CYAN, s.clone()))
        .collect();
    let _ = draw_panel_box(right_x, ry, col_w, "NETWORK", &net_lines);
}

// ─── CPU Tab ───────────────────────────────────────────────────────────────

fn draw_cpu_tab(w: u32, y0: u32, _h: u32, data: &SystemData, _scroll: usize) {
    let panel_w = w - 16;
    let x = 8u32;
    let mut y = y0 + 4;

    // Brand + identity
    y = draw_panel_box(x, y, panel_w, "PROCESSOR", &[
        (FG_WHITE,  data.cpu.brand.clone()),
        (FG_CYAN,   format!("Vendor: {}  Family: {}  Model: {}  Stepping: {}",
            data.cpu.vendor, data.cpu.family, data.cpu.model, data.cpu.stepping)),
        (FG_GREEN,  format!("Logical Cores: {}  |  TSC: {} MHz",
            data.cpu.cores, data.cpu.tsc_freq_mhz)),
    ]);

    // Features in rows of ~8
    y += 4;
    let mut feat_lines: Vec<(u32, String)> = Vec::new();
    let mut line = String::new();
    for (i, f) in data.cpu.features.iter().enumerate() {
        if i > 0 { line.push_str("  "); }
        line.push_str(f);
        if (i + 1) % 8 == 0 {
            feat_lines.push((FG_GREEN, line.clone()));
            line.clear();
        }
    }
    if !line.is_empty() {
        feat_lines.push((FG_GREEN, line));
    }
    let _ = draw_panel_box(x, y, panel_w, "FEATURES", &feat_lines);
}

// ─── Memory Tab ────────────────────────────────────────────────────────────

fn draw_memory_tab(w: u32, y0: u32, _h: u32, data: &SystemData, _scroll: usize) {
    let panel_w = w - 16;
    let x = 8u32;
    let mut y = y0 + 4;

    let heap_pct = if data.memory.heap_total > 0 {
        (data.memory.heap_used * 100) / data.memory.heap_total
    } else { 0 };

    y = draw_panel_box(x, y, panel_w, "PHYSICAL MEMORY", &[
        (FG_WHITE, format!("Total: {} MB  ({} bytes)", data.memory.total_mb, data.memory.total_bytes)),
    ]);

    y += 4;
    y = draw_panel_box(x, y, panel_w, "KERNEL HEAP", &[
        (FG_WHITE, format!("Used:  {} KB / {} KB  ({}%)",
            data.memory.heap_used / 1024,
            data.memory.heap_total / 1024,
            heap_pct)),
        (FG_CYAN, format!("Free:  {} KB", data.memory.heap_free / 1024)),
    ]);
    draw_progress_bar(x + PAD, y - 4, panel_w - PAD * 2, heap_pct as u32, 100);
}

// ─── PCI Tab ───────────────────────────────────────────────────────────────

fn draw_pci_tab(w: u32, y0: u32, h: u32, data: &SystemData, scroll: usize) {
    let panel_w = w - 16;
    let x = 8u32;
    let mut y = y0 + 4;

    // Header
    framebuffer::draw_text_at(
        &format!(" {} PCI/PCIe Devices ", data.pci_devices.len()),
        x, y, ACCENT, BG_DARK);
    y += CHAR_H + 4;

    // Column headers
    framebuffer::fill_rect(x, y, panel_w, CHAR_H + 2, BG_PANEL);
    framebuffer::draw_text_at(" BDF        VEND:DEV  VENDOR       CLASS", x, y + 1, FG_YELLOW, BG_PANEL);
    y += CHAR_H + 4;

    // Device rows
    let max_rows = ((h.saturating_sub(y - y0 + CHAR_H + 8)) / (CHAR_H + 2)) as usize;
    let start = scroll.min(data.pci_devices.len().saturating_sub(1));
    let end = (start + max_rows).min(data.pci_devices.len());

    for (i, dev) in data.pci_devices[start..end].iter().enumerate() {
        let bg = if i % 2 == 0 { BG_DARK } else { BG_PANEL };
        framebuffer::fill_rect(x, y, panel_w, CHAR_H + 2, bg);

        let line = format!(" {:02x}:{:02x}.{}    {:04X}:{:04X}  {:<12} {}",
            dev.bus, dev.device, dev.function,
            dev.vendor_id, dev.device_id,
            dev.vendor_name,
            dev.class_name);
        // Truncate to fit
        let max_chars = (panel_w / CHAR_W) as usize;
        let display = if line.len() > max_chars { &line[..max_chars] } else { &line };
        framebuffer::draw_text_at(display, x, y + 1, FG_WHITE, bg);
        y += CHAR_H + 2;
    }

    // Scroll indicator
    if data.pci_devices.len() > max_rows {
        let indicator = format!(" [{}-{} of {}] ", start + 1, end, data.pci_devices.len());
        framebuffer::draw_text_at(&indicator, x, y + 2, FG_DIM, BG_DARK);
    }
}

// ─── IRQ Tab ───────────────────────────────────────────────────────────────

fn draw_irq_tab(w: u32, y0: u32, _h: u32, data: &SystemData, scroll: usize) {
    let panel_w = w - 16;
    let x = 8u32;
    let mut y = y0 + 4;

    y = draw_panel_box(x, y, panel_w, "INTERRUPT CONTROLLER", &[
        (FG_WHITE, format!("Local APIC: 0x{:08X}", data.irq.local_apic_addr)),
        (FG_CYAN,  format!("I/O APICs: {}  |  IRQ Overrides: {}",
            data.irq.io_apic_count, data.irq.override_count)),
        (FG_GREEN, format!("CPU Count: {}", data.irq.cpu_count)),
    ]);

    y += 4;
    let detail_lines: Vec<(u32, String)> = data.irq.details.iter()
        .skip(scroll)
        .map(|d| {
            let color = if d.contains("Override") { FG_YELLOW }
                else if d.contains("APIC") { FG_CYAN }
                else { FG_GREEN };
            (color, d.clone())
        })
        .collect();
    let _ = draw_panel_box(x, y, panel_w, "DETAILS", &detail_lines);
}

// ─── Storage Tab ───────────────────────────────────────────────────────────

fn draw_storage_tab(w: u32, y0: u32, _h: u32, data: &SystemData, _scroll: usize) {
    let panel_w = w - 16;
    let x = 8u32;
    let y = y0 + 4;

    let lines: Vec<(u32, String)> = data.storage.devices.iter()
        .map(|s| (FG_CYAN, s.clone()))
        .collect();
    let _ = draw_panel_box(x, y, panel_w, "STORAGE CONTROLLERS", &lines);
}

// ─── Network Tab ───────────────────────────────────────────────────────────

fn draw_network_tab(w: u32, y0: u32, _h: u32, data: &SystemData, _scroll: usize) {
    let panel_w = w - 16;
    let x = 8u32;
    let y = y0 + 4;

    let lines: Vec<(u32, String)> = data.network.interfaces.iter()
        .map(|s| (FG_CYAN, s.clone()))
        .collect();
    let _ = draw_panel_box(x, y, panel_w, "NETWORK INTERFACES", &lines);
}

// ─── Thermal Tab ───────────────────────────────────────────────────────────

fn draw_thermal_tab(w: u32, y0: u32, _h: u32, data: &SystemData, _scroll: usize) {
    let panel_w = w - 16;
    let x = 8u32;
    let mut y = y0 + 4;

    // Temperature gauge
    if let Some(temp) = data.thermal.cpu_temp {
        let color = if temp > 85 { FG_RED } else if temp > 70 { FG_YELLOW } else { FG_GREEN };
        y = draw_panel_box(x, y, panel_w, "CPU TEMPERATURE", &[
            (color, format!("  {}°C  /  TjMax {}°C", temp, data.thermal.tj_max)),
        ]);
        // Temperature bar (0-100°C mapped)
        let pct = (temp as u32).min(100);
        draw_progress_bar(x + PAD, y - 4, panel_w - PAD * 2, pct, 100);
        y += 14;
    } else {
        y = draw_panel_box(x, y, panel_w, "CPU TEMPERATURE", &[
            (FG_DIM, String::from("Temperature sensors not available")),
        ]);
    }

    // Details
    y += 4;
    let detail_lines: Vec<(u32, String)> = data.thermal.details.iter()
        .map(|d| {
            let color = if d.contains("PROCHOT") || d.contains("throttl") { FG_RED }
                else { FG_CYAN };
            (color, d.clone())
        })
        .collect();
    if !detail_lines.is_empty() {
        let _ = draw_panel_box(x, y, panel_w, "POWER & PERFORMANCE", &detail_lines);
    }
}

// ─── Drawing Primitives ────────────────────────────────────────────────────

/// Draw a panel box with title and colored text lines.
/// Returns the Y coordinate after the box.
fn draw_panel_box(x: u32, y: u32, w: u32, title: &str, lines: &[(u32, String)]) -> u32 {
    let line_count = lines.len().max(1) as u32;
    let box_h = CHAR_H + (line_count * (CHAR_H + 2)) + PAD * 2;

    // Background
    framebuffer::fill_rect(x, y, w, box_h, BG_PANEL);

    // Border
    framebuffer::draw_rect(x, y, w, box_h, BORDER);

    // Title bar
    framebuffer::fill_rect(x + 1, y + 1, w - 2, CHAR_H + 4, 0xFF1E1E2E);
    let title_str = format!(" {} ", title);
    framebuffer::draw_text_at(&title_str, x + PAD, y + 3, ACCENT, 0xFF1E1E2E);
    // Title underline
    framebuffer::fill_rect(x + 1, y + CHAR_H + 5, w - 2, 1, BORDER);

    // Content lines
    let mut ly = y + CHAR_H + 8;
    for (color, text) in lines {
        let max_chars = ((w - PAD * 2) / CHAR_W) as usize;
        let display = if text.len() > max_chars { &text[..max_chars] } else { text.as_str() };
        framebuffer::draw_text_at(display, x + PAD, ly, *color, BG_PANEL);
        ly += CHAR_H + 2;
    }

    y + box_h
}

/// Draw a horizontal progress bar
fn draw_progress_bar(x: u32, y: u32, w: u32, value: u32, max: u32) {
    let bar_h = 10u32;
    // Background
    framebuffer::fill_rect(x, y, w, bar_h, BAR_BG);

    // Fill
    let pct = if max > 0 { (value * 100) / max } else { 0 };
    let fill_w = (w * pct) / 100;
    let color = if pct > 90 { BAR_CRIT } else if pct > 70 { BAR_WARN } else { BAR_FG };
    if fill_w > 0 {
        framebuffer::fill_rect(x, y, fill_w, bar_h, color);
    }

    // Percentage text centered
    let label = format!("{}%", pct);
    let tx = x + (w / 2).saturating_sub((label.len() as u32 * CHAR_W) / 2);
    // Only draw text if bar is wide enough
    if w > label.len() as u32 * CHAR_W + 4 {
        framebuffer::draw_text(& label, tx, y - 1, FG_WHITE);
    }
}
