//! TrustVM Branding and Display
//!
//! Visual branding elements for TrustVM:
//! - ASCII art logos
//! - Status displays
//! - Version information
//! - VM dashboard rendering

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

// ============================================================================
// VERSION INFO
// ============================================================================

pub const TRUSTVM_NAME: &str = "TrustVM";
pub const TRUSTVM_VERSION: &str = "1.0.0";
pub const TRUSTVM_CODENAME: &str = "Phoenix";
pub const TRUSTVM_COPYRIGHT: &str = "(c) 2026 TrustOS Project";

/// Full version string
pub fn version_string() -> String {
    format!("{} v{} \"{}\"", TRUSTVM_NAME, TRUSTVM_VERSION, TRUSTVM_CODENAME)
}

// ============================================================================
// ASCII ART LOGOS
// ============================================================================

/// Small TrustVM logo (5 lines)
pub const LOGO_SMALL: &str = r#"
 _____              _  __   ____  __ 
|_   _| __ _   _ ___| |_\ \ / /  \/  |
  | || '__| | | / __| __\ V /| |\/| |
  | || |  | |_| \__ \ |_ | | | |  | |
  |_||_|   \__,_|___/\__||_| |_|  |_|
"#;

/// Large TrustVM logo (7 lines)
pub const LOGO_LARGE: &str = r#"
████████╗██████╗ ██╗   ██╗███████╗████████╗██╗   ██╗███╗   ███╗
╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██║   ██║████╗ ████║
   ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║██╔████╔██║
   ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║██║╚██╔╝██║
   ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝██║ ╚═╝ ██║
   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚═╝     ╚═╝
"#;

/// Minimal TrustVM badge (1 line)
pub const LOGO_BADGE: &str = "[TrustVM]";

/// Shield icon for security status
pub const ICON_SHIELD: &str = r#"
  ████
 ██████
██  ✓ ██
 ██████
  ████
   ██
"#;

/// VM icon
pub const ICON_VM: &str = r#"
┌──────┐
│ ▓▓▓▓ │
│ ░░░░ │
└──────┘
"#;

// ============================================================================
// STATUS BAR ELEMENTS
// ============================================================================

/// Status indicator for enabled/active
pub const STATUS_ACTIVE: &str = "● ";
/// Status indicator for disabled/inactive  
pub const STATUS_INACTIVE: &str = "○ ";
/// Status indicator for warning
pub const STATUS_WARNING: &str = "▲ ";
/// Status indicator for error
pub const STATUS_ERROR: &str = "✗ ";
/// Status indicator for success
pub const STATUS_OK: &str = "✓ ";

/// Format uptime for display
pub fn format_uptime(ms: u64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours % 24, mins % 60)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, mins % 60, secs % 60)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

/// Format memory size for display
pub fn format_memory(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{} GB", bytes / (1024 * 1024 * 1024))
    } else if bytes >= 1024 * 1024 {
        format!("{} MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{} B", bytes)
    }
}

// ============================================================================
// DASHBOARD RENDERING
// ============================================================================

/// VM status row for dashboard
#[derive(Debug, Clone)]
pub struct VmStatusRow {
    pub id: u64,
    pub name: String,
    pub state: &'static str,
    pub memory_mb: usize,
    pub cpu_percent: u8,
    pub exits: u64,
}

impl VmStatusRow {
    pub fn render(&self) -> String {
        let state_indicator = match self.state {
            "Running" => STATUS_ACTIVE,
            "Stopped" => STATUS_INACTIVE,
            "Crashed" => STATUS_ERROR,
            "Paused" => STATUS_WARNING,
            _ => "  ",
        };
        
        format!(
            "│ {:>3} │ {}{:<12} │ {:>4} MB │ {:>3}% │ {:>8} │",
            self.id,
            state_indicator,
            self.state,
            self.memory_mb,
            self.cpu_percent,
            self.exits
        )
    }
}

/// Render VM dashboard header
pub fn dashboard_header() -> &'static str {
    r#"
┌─────┬────────────────┬─────────┬──────┬──────────┐
│ ID  │ State          │ Memory  │ CPU  │ Exits    │
├─────┼────────────────┼─────────┼──────┼──────────┤"#
}

/// Render VM dashboard footer
pub fn dashboard_footer() -> &'static str {
    "└─────┴────────────────┴─────────┴──────┴──────────┘"
}

/// Render full dashboard
pub fn render_dashboard(vms: &[VmStatusRow]) -> String {
    let mut output = String::new();
    
    output.push_str(&version_string());
    output.push('\n');
    output.push_str(dashboard_header());
    output.push('\n');
    
    if vms.is_empty() {
        output.push_str("│     │ (no VMs)       │         │      │          │\n");
    } else {
        for vm in vms {
            output.push_str(&vm.render());
            output.push('\n');
        }
    }
    
    output.push_str(dashboard_footer());
    output
}

// ============================================================================
// CAPABILITY DISPLAY
// ============================================================================

/// Render capabilities as a formatted string
pub fn render_capabilities(caps: u64) -> String {
    let mut output = String::from("TrustVM Capabilities:\n");
    
    let features = [
        (1 << 0, "VMX Enabled", "Intel VT-x hardware virtualization"),
        (1 << 1, "EPT", "Extended Page Tables for memory isolation"),
        (1 << 2, "VPID", "Virtual Processor IDs for TLB optimization"),
        (1 << 3, "Unrestricted Guest", "Real mode and unpaged protected mode"),
        (1 << 4, "VMCS Shadowing", "Nested virtualization support"),
        (1 << 5, "Posted Interrupts", "Efficient interrupt delivery"),
        (1 << 6, "EPT A/D Bits", "Accessed/Dirty tracking"),
        (1 << 7, "Virtual Console", "Virtual serial console"),
        (1 << 8, "Shared Filesystem", "VirtFS host-guest sharing"),
    ];
    
    for (bit, name, desc) in features {
        let status = if caps & bit != 0 { STATUS_OK } else { STATUS_INACTIVE };
        output.push_str(&format!("  {} {:<20} {}\n", status, name, desc));
    }
    
    output
}

// ============================================================================
// SECURITY STATUS
// ============================================================================

/// Security status display
pub fn render_security_status(
    vpid_enabled: bool,
    ept_enabled: bool,
    nx_enabled: bool,
    violations: u64,
) -> String {
    let mut output = String::from("Security Status:\n");
    
    // VPID
    let vpid_status = if vpid_enabled { STATUS_OK } else { STATUS_WARNING };
    output.push_str(&format!("  {} VPID Isolation: {}\n", vpid_status, 
        if vpid_enabled { "Active" } else { "Disabled" }));
    
    // EPT
    let ept_status = if ept_enabled { STATUS_OK } else { STATUS_ERROR };
    output.push_str(&format!("  {} EPT Protection: {}\n", ept_status,
        if ept_enabled { "Active" } else { "Disabled" }));
    
    // NX
    let nx_status = if nx_enabled { STATUS_OK } else { STATUS_WARNING };
    output.push_str(&format!("  {} NX Enforcement: {}\n", nx_status,
        if nx_enabled { "Active" } else { "Disabled" }));
    
    // Violations
    let viol_status = if violations == 0 { STATUS_OK } else { STATUS_WARNING };
    output.push_str(&format!("  {} EPT Violations: {}\n", viol_status, violations));
    
    output
}

// ============================================================================
// BANNER MESSAGES
// ============================================================================

/// Startup banner
pub fn startup_banner() -> String {
    format!(r#"
{}
────────────────────────────────────────────────
  {} - Secure Hardware Virtualization
  {}
────────────────────────────────────────────────
"#, LOGO_SMALL.trim(), version_string(), TRUSTVM_COPYRIGHT)
}

/// Welcome message after initialization
pub fn welcome_message(vpid: bool, ept: bool) -> String {
    format!(r#"
{} Hypervisor Ready

Features:
  {} VPID (TLB Isolation)
  {} EPT (Memory Protection)
  {} Virtual Console
  {} Shared Filesystem

Type 'hv help' for commands or 'vm run hello' to start a guest.
"#,
        TRUSTVM_NAME,
        if vpid { STATUS_OK } else { STATUS_INACTIVE },
        if ept { STATUS_OK } else { STATUS_INACTIVE },
        STATUS_OK,
        STATUS_OK
    )
}

// ============================================================================
// PROGRESS INDICATORS
// ============================================================================

/// Render a progress bar
pub fn progress_bar(progress: u8, width: usize) -> String {
    let filled = (progress as usize * width) / 100;
    let empty = width - filled;
    
    format!("[{}{}] {}%", 
        "█".repeat(filled),
        "░".repeat(empty),
        progress
    )
}

/// Render a spinner frame
pub fn spinner_frame(frame: usize) -> char {
    const FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    FRAMES[frame % FRAMES.len()]
}
