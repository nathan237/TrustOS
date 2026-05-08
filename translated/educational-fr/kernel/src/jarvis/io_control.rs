//! JARVIS I/O Control Monitor — Confirm Total I/O Awareness
//!
//! Centralized module that audits every I/O subsystem in TrustOS and confirms
//! which channels JARVIS has visibility and control over.
//!
//! # Purpose
//!
//! When JARVIS propagates across a mesh network, each node must verify it has
//! full I/O control before participating in federated learning. A node with
//! incomplete I/O cannot be trusted to:
//! - Receive training commands (keyboard/serial)
//! - Save/load weights (disk)
//! - Communicate with peers (network)
//! - Report hardware capabilities (CPU/GPU)
//!
//! # Usage
//!
//! ```text
//! let report = io_control::full_audit();
//! let score = io_control::control_score();   // 0..100
//! let ready = io_control::network_ready();   // Can this node join mesh?
//! ```

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// I/O Channel Status
// ═══════════════════════════════════════════════════════════════════════════════

/// Status of a single I/O channel
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum IoStatus {
    /// Channel detected and operational
    Active,
    /// Channel detected but not operational (driver issue)
    Detected,
    /// Channel not detected (hardware absent)
    Absent,
    /// Channel not probed yet
    Unknown,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl IoStatus {
    fn symbol(self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            IoStatus::Active => "[+]",
            IoStatus::Detected => "[~]",
            IoStatus::Absent => "[-]",
            IoStatus::Unknown => "[?]",
        }
    }

    fn is_active(self) -> bool {
        self == IoStatus::Active
    }
}

/// Complete I/O audit report
pub struct IoAudit {
    pub keyboard: IoStatus,
    pub mouse: IoStatus,
    pub serial: IoStatus,
    pub network: IoStatus,
    pub disk: IoStatus,
    pub display: IoStatus,
    pub usb: IoStatus,
    pub audio: IoStatus,
    pub touch: IoStatus,
    pub cpu_smp: IoStatus,
    pub gpu: IoStatus,
    pub storage_ahci: IoStatus,
    pub storage_ata: IoStatus,
    pub storage_nvme: IoStatus,
    pub pci: IoStatus,
}

/// Cached last audit timestamp
static LAST_AUDIT_MOUSE: AtomicU64 = AtomicU64::new(0);

/// Cached audit result
static LAST_AUDIT: Mutex<Option<IoAudit>> = Mutex::new(None);

/// Whether continuous monitoring is enabled
static MONITORING_ENABLED: AtomicBool = AtomicBool::new(false);

/// I/O events observed counter
static IO_EVENTS_TOTAL: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// Core Audit — Probe all I/O channels
// ═══════════════════════════════════════════════════════════════════════════════

/// Perform a full audit of all I/O subsystems.
/// Returns a snapshot of what JARVIS can see and control.
pub fn full_audit() -> IoAudit {
    let audit = IoAudit {
        keyboard: probe_keyboard(),
        mouse: probe_mouse(),
        serial: probe_serial(),
        network: probe_network(),
        disk: probe_disk(),
        display: probe_display(),
        usb: probe_usb(),
        audio: probe_audio(),
        touch: probe_touch(),
        cpu_smp: probe_cpu_smp(),
        gpu: probe_gpu(),
        storage_ahci: probe_ahci(),
        storage_ata: probe_ata(),
        storage_nvme: probe_nvme(),
        pci: probe_pci(),
    };

    // Cache result
    LAST_AUDIT_MOUSE.store(crate::time::uptime_ms(), Ordering::SeqCst);
    *LAST_AUDIT.lock() = Some(IoAudit {
        keyboard: audit.keyboard,
        mouse: audit.mouse,
        serial: audit.serial,
        network: audit.network,
        disk: audit.disk,
        display: audit.display,
        usb: audit.usb,
        audio: audit.audio,
        touch: audit.touch,
        cpu_smp: audit.cpu_smp,
        gpu: audit.gpu,
        storage_ahci: audit.storage_ahci,
        storage_ata: audit.storage_ata,
        storage_nvme: audit.storage_nvme,
        pci: audit.pci,
    });

    IO_EVENTS_TOTAL.fetch_add(1, Ordering::Relaxed);
    audit
}

// ═══════════════════════════════════════════════════════════════════════════════
// Individual I/O Probes
// ═══════════════════════════════════════════════════════════════════════════════

fn probe_keyboard() -> IoStatus {
    if crate::drivers::input::has_keyboard() {
        IoStatus::Active
    } else if crate::keyboard::has_input() {
        IoStatus::Active
    } else {
        // Keyboard is always present on x86 (PS/2 port), just may not have input yet
        IoStatus::Detected
    }
}

fn probe_mouse() -> IoStatus {
    if crate::mouse::is_initialized() {
        if crate::drivers::input::has_mouse() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn probe_serial() -> IoStatus {
    // Serial is always available on x86 (COM1)
    // Test by checking if we can read (non-blocking)
    IoStatus::Active
}

fn probe_network() -> IoStatus {
    if crate::network::is_available() {
        if crate::network::get_mac_address().is_some() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn probe_disk() -> IoStatus {
    if crate::disk::is_available() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn probe_display() -> IoStatus {
    if crate::framebuffer::is_initialized() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn probe_usb() -> IoStatus {
    if crate::drivers::usb::is_initialized() {
        if crate::drivers::usb::has_hid_devices() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn probe_audio() -> IoStatus {
    if crate::drivers::hda::is_initialized() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn probe_touch() -> IoStatus {
    if crate::touch::is_available() {
        if crate::touch::is_initialized() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn probe_cpu_smp() -> IoStatus {
    let count = crate::cpu::smp::cpu_count();
    let ready = crate::cpu::smp::ready_cpu_count();
    if ready > 1 {
        IoStatus::Active
    } else if count > 1 {
        IoStatus::Detected
    } else {
        // Single-core is still functional
        IoStatus::Active
    }
}

fn probe_gpu() -> IoStatus {
    if crate::drivers::amdgpu::is_detected() {
        if crate::drivers::amdgpu::compute::is_ready() {
            IoStatus::Active
        } else {
            IoStatus::Detected
        }
    } else {
        IoStatus::Absent
    }
}

fn probe_ahci() -> IoStatus {
    if crate::drivers::ahci::is_initialized() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn probe_ata() -> IoStatus {
    if crate::drivers::ata::is_initialized() {
        IoStatus::Active
    } else {
        IoStatus::Absent
    }
}

fn probe_nvme() -> IoStatus {
    // NVMe detection through driver framework
    if crate::drivers::has_storage() {
        IoStatus::Detected
    } else {
        IoStatus::Absent
    }
}

fn probe_pci() -> IoStatus {
    // PCI is always scanned at boot on x86
    IoStatus::Active
}

// ═══════════════════════════════════════════════════════════════════════════════
// Scoring — Quantify I/O Control
// ═══════════════════════════════════════════════════════════════════════════════

/// Calculate a control score (0..100) based on I/O coverage.
///
/// Critical channels (keyboard, serial, network, disk, display) are weighted
/// higher because they're essential for JARVIS operation and mesh participation.
pub fn control_score(audit: &IoAudit) -> u8 {
    let mut score: u32 = 0;
    let mut maximum_score: u32 = 0;

    // Critical I/O (weighted 15 each = 75 total)
    let critical = [
        (audit.keyboard, 15u32),
        (audit.serial, 15),
        (audit.network, 15),
        (audit.disk, 15),
        (audit.display, 15),
    ];

    // Optional I/O (weighted 5 each = 25 total)
    let optional = [
        (audit.mouse, 5u32),
        (audit.usb, 5),
        (audit.audio, 5),
        (audit.touch, 5),
        (audit.gpu, 5),
    ];

    for (status, weight) in &critical {
        maximum_score += weight;
                // Correspondance de motifs — branchement exhaustif de Rust.
match status {
            IoStatus::Active => score += weight,
            IoStatus::Detected => score += weight / 2,
            _ => {}
        }
    }

    for (status, weight) in &optional {
        maximum_score += weight;
                // Correspondance de motifs — branchement exhaustif de Rust.
match status {
            IoStatus::Active => score += weight,
            IoStatus::Detected => score += weight / 2,
            _ => {}
        }
    }

    ((score * 100) / maximum_score).min(100) as u8
}

/// Check if this node has enough I/O to participate in the mesh network.
/// Requires: network + serial/keyboard + disk at minimum.
pub fn network_ready(audit: &IoAudit) -> bool {
    audit.network.is_active()
        && (audit.keyboard.is_active() || audit.serial.is_active())
        && audit.disk.is_active()
}

/// Check if this node has full I/O control (maximum propagation capability)
pub fn full_control(audit: &IoAudit) -> bool {
    control_score(audit) >= 75
}

// ═══════════════════════════════════════════════════════════════════════════════
// Human-Readable Report
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate a formatted I/O control report
pub fn format_report(audit: &IoAudit) -> Vec<String> {
    let score = control_score(audit);
    let ready = network_ready(audit);

    let mut lines = Vec::with_capacity(24);
    lines.push(String::from("╔═══════════════════════════════════════════════════╗"));
    lines.push(String::from("║       JARVIS I/O CONTROL AUDIT                   ║"));
    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));

    // Critical I/O
    lines.push(String::from("║ CRITICAL CHANNELS:                                ║"));
    lines.push(format!("║  {} Keyboard    {}", audit.keyboard.symbol(), status_line(audit.keyboard)));
    lines.push(format!("║  {} Serial      {}", audit.serial.symbol(), status_line(audit.serial)));
    lines.push(format!("║  {} Network     {}", audit.network.symbol(), status_line(audit.network)));
    lines.push(format!("║  {} Disk        {}", audit.disk.symbol(), status_line(audit.disk)));
    lines.push(format!("║  {} Display     {}", audit.display.symbol(), status_line(audit.display)));

    lines.push(String::from("║ EXTENDED CHANNELS:                                ║"));
    lines.push(format!("║  {} Mouse       {}", audit.mouse.symbol(), status_line(audit.mouse)));
    lines.push(format!("║  {} USB         {}", audit.usb.symbol(), status_line(audit.usb)));
    lines.push(format!("║  {} Audio       {}", audit.audio.symbol(), status_line(audit.audio)));
    lines.push(format!("║  {} Touch       {}", audit.touch.symbol(), status_line(audit.touch)));
    lines.push(format!("║  {} GPU Compute {}", audit.gpu.symbol(), status_line(audit.gpu)));

    lines.push(String::from("║ STORAGE:                                          ║"));
    lines.push(format!("║  {} AHCI/SATA  {}", audit.storage_ahci.symbol(), status_line(audit.storage_ahci)));
    lines.push(format!("║  {} ATA/IDE    {}", audit.storage_ata.symbol(), status_line(audit.storage_ata)));

    lines.push(String::from("║ COMPUTE:                                          ║"));
    lines.push(format!("║  {} CPU/SMP    {}", audit.cpu_smp.symbol(), status_line(audit.cpu_smp)));
    lines.push(format!("║  {} PCI Bus    {}", audit.pci.symbol(), status_line(audit.pci)));

    lines.push(String::from("╠═══════════════════════════════════════════════════╣"));
    lines.push(format!("║  Control Score: {}%                                 ║",
        if score < 10 { format!(" {}", score) } else { format!("{}", score) }));
    lines.push(format!("║  Network Ready: {}                                ║",
        if ready { "YES ✓" } else { "NO  ✗" }));
    lines.push(format!("║  Full Control:  {}                                ║",
        if full_control(audit) { "YES ✓" } else { "NO  ✗" }));
    lines.push(String::from("╚═══════════════════════════════════════════════════╝"));

    lines
}

/// Format a status value with fixed-width padding
fn status_line(status: IoStatus) -> &'static str {
        // Correspondance de motifs — branchement exhaustif de Rust.
match status {
        IoStatus::Active =>   "Active                          ║",
        IoStatus::Detected => "Detected                        ║",
        IoStatus::Absent =>   "Absent                          ║",
        IoStatus::Unknown =>  "Unknown                         ║",
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mesh Integration — Announce I/O capabilities to peers
// ═══════════════════════════════════════════════════════════════════════════════

/// Encode I/O capabilities as a compact bitmask for mesh announce packets.
/// Each bit represents one I/O channel being active.
///
/// Bit layout:
/// ```text
/// [0] keyboard  [1] mouse   [2] serial   [3] network
/// [4] disk      [5] display  [6] usb      [7] audio
/// [8] touch     [9] gpu     [10] smp     [11] ahci
/// [12] ata      [13] nvme   [14] pci     [15] reserved
/// ```
pub fn capability_bitmask(audit: &IoAudit) -> u16 {
    let mut mask: u16 = 0;
    if audit.keyboard.is_active() { mask |= 1 << 0; }
    if audit.mouse.is_active()    { mask |= 1 << 1; }
    if audit.serial.is_active()   { mask |= 1 << 2; }
    if audit.network.is_active()  { mask |= 1 << 3; }
    if audit.disk.is_active()     { mask |= 1 << 4; }
    if audit.display.is_active()  { mask |= 1 << 5; }
    if audit.usb.is_active()      { mask |= 1 << 6; }
    if audit.audio.is_active()    { mask |= 1 << 7; }
    if audit.touch.is_active()    { mask |= 1 << 8; }
    if audit.gpu.is_active()      { mask |= 1 << 9; }
    if audit.cpu_smp.is_active()  { mask |= 1 << 10; }
    if audit.storage_ahci.is_active() { mask |= 1 << 11; }
    if audit.storage_ata.is_active()  { mask |= 1 << 12; }
    if audit.storage_nvme.is_active() { mask |= 1 << 13; }
    if audit.pci.is_active()      { mask |= 1 << 14; }
    mask
}

/// Decode capability bitmask back to human-readable summary
pub fn describe_capabilities(mask: u16) -> String {
    let mut caps = Vec::new();
    if mask & (1 << 0) != 0 { caps.push("kbd"); }
    if mask & (1 << 1) != 0 { caps.push("mouse"); }
    if mask & (1 << 2) != 0 { caps.push("serial"); }
    if mask & (1 << 3) != 0 { caps.push("net"); }
    if mask & (1 << 4) != 0 { caps.push("disk"); }
    if mask & (1 << 5) != 0 { caps.push("display"); }
    if mask & (1 << 6) != 0 { caps.push("usb"); }
    if mask & (1 << 7) != 0 { caps.push("audio"); }
    if mask & (1 << 8) != 0 { caps.push("touch"); }
    if mask & (1 << 9) != 0 { caps.push("gpu"); }
    if mask & (1 << 10) != 0 { caps.push("smp"); }
    if mask & (1 << 11) != 0 { caps.push("ahci"); }
    if mask & (1 << 12) != 0 { caps.push("ata"); }
    if mask & (1 << 13) != 0 { caps.push("nvme"); }
    if mask & (1 << 14) != 0 { caps.push("pci"); }

    if caps.is_empty() {
        String::from("none")
    } else {
        let mut s = String::new();
        for (i, c) in caps.iter().enumerate() {
            if i > 0 { s.push_str(", "); }
            s.push_str(c);
        }
        s
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Continuous Monitoring
// ═══════════════════════════════════════════════════════════════════════════════

/// Enable continuous I/O monitoring (call poll() from main loop)
pub fn enable_monitoring() {
    MONITORING_ENABLED.store(true, Ordering::SeqCst);
    crate::serial_println!("[IO-CTRL] Continuous I/O monitoring enabled");
}

/// Disable monitoring
pub fn disable_monitoring() {
    MONITORING_ENABLED.store(false, Ordering::SeqCst);
}

/// Poll I/O channels — call periodically from kernel main loop.
/// Re-audits every 10 seconds if monitoring is enabled.
pub fn poll() {
    if !MONITORING_ENABLED.load(Ordering::SeqCst) {
        return;
    }

    let now = crate::time::uptime_ms();
    let last = LAST_AUDIT_MOUSE.load(Ordering::SeqCst);
    if now.wrapping_sub(last) < 10_000 {
        return;
    }

    let audit = full_audit();
    let score = control_score(&audit);

    // Log if score drops below threshold
    if score < 60 {
        crate::serial_println!("[IO-CTRL] WARNING: I/O control score dropped to {}%", score);
    }
}

/// Get total I/O events observed
pub fn events_total() -> u64 {
    IO_EVENTS_TOTAL.load(Ordering::Relaxed)
}

/// Quick summary string for status display
pub fn quick_status() -> String {
    let audit = full_audit();
    let score = control_score(&audit);
    let mask = capability_bitmask(&audit);
    let ready = network_ready(&audit);
    format!("io_score={}% caps=0x{:04X} mesh_ready={}", score, mask, ready)
}
