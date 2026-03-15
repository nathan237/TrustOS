//! VM Inspector Panel — Real-time VM Introspection for TrustLab
//!
//! Displays live VM state: registers, exit statistics, memory hexdump,
//! recent VM events, and guest process introspection.
//!
//! This is the world's first integrated VM introspection panel inside
//! a bare-metal OS's own kernel introspection lab.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::trace_bus::{self, EventCategory, LabEvent};

// ── Colors ─────────────────────────────────────────────────────────────────
const COLUMN_TEXT: u32    = 0xFFE6EDF3;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COLUMN_DIM: u32     = 0xFF8B949E;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COLUMN_GREEN: u32   = 0xFF3FB950;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COLUMN_RED: u32     = 0xFFF85149;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COLUMN_YELLOW: u32  = 0xFFD29922;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COLUMN_CYAN: u32    = 0xFF79C0FF;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COLUMN_PURPLE: u32  = 0xFFBC8CFF;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COLUMN_ORANGE: u32  = 0xFFD18616;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COLUMN_HV: u32      = 0xFFFF6B6B; // Hypervisor accent

/// VM Inspector panel state
pub struct VmInspectorState {
    /// Recent HV events (filtered from trace bus)
    pub events: Vec<LabEvent>,
    /// Last read index from trace bus
    pub last_read_index: u64,
    /// Current view tab (0=Overview, 1=Exits, 2=Memory, 3=Registers)
    pub active_tab: usize,
    /// Scroll offset in the event list
    pub scroll: usize,
    /// Frame counter
    pub frame: u64,
    /// Cached VM stats (from last HV event scan)
    pub vm_count: u32,
    pub total_exits: u64,
    pub last_exit_reason: String,
    pub last_guest_rip: u64,
    /// Register snapshot (parsed from last REGS event)
    pub regs_rax: u64,
    pub regs_rbx: u64,
    pub regs_rcx: u64,
    pub regs_rdx: u64,
    pub regs_rip: u64,
    pub regs_rsp: u64,
    /// Exit counters by type
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub npf_exits: u64,
    pub vmcall_exits: u64,
    /// Memory view base address
    pub memory_view_address: u64,
}

// Implementation block — defines methods for the type above.
impl VmInspectorState {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            events: Vec::new(),
            last_read_index: 0,
            active_tab: 0,
            scroll: 0,
            frame: 0,
            vm_count: 0,
            total_exits: 0,
            last_exit_reason: String::from("(none)"),
            last_guest_rip: 0,
            regs_rax: 0,
            regs_rbx: 0,
            regs_rcx: 0,
            regs_rdx: 0,
            regs_rip: 0,
            regs_rsp: 0,
            cpuid_exits: 0,
            io_exits: 0,
            msr_exits: 0,
            hlt_exits: 0,
            npf_exits: 0,
            vmcall_exits: 0,
            memory_view_address: 0x1000,
        }
    }

    /// Update state from trace bus (called each tick)
    pub fn update(&mut self) {
        self.frame += 1;
        if self.frame % 10 != 0 { return; }
        
        let (new_events, new_index) = trace_bus::read_since(self.last_read_index, 200);
        self.last_read_index = new_index;
        
        for event in &new_events {
            if event.category == EventCategory::Hypervisor {
                // Parse event for stats
                self.parse_hv_event(event);
                
                // Keep last 200 HV events
                self.events.push(event.clone());
                if self.events.len() > 200 {
                    self.events.remove(0);
                }
            }
        }
    }

    /// Parse a hypervisor event to extract stats
    fn parse_hv_event(&mut self, event: &LabEvent) {
        let message = &event.message;
        
        // Count exits by type
        if message.contains("EXIT: CPUID") {
            self.cpuid_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("CPUID");
        } else if message.contains("IO ") {
            self.io_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("I/O");
        } else if message.contains("RDMSR") || message.contains("WRMSR") {
            self.msr_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("MSR");
        } else if message.contains("EXIT: HLT") {
            self.hlt_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("HLT");
        } else if message.contains("NPF_VIOLATION") || message.contains("EPT_VIOLATION") {
            self.npf_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("PAGE_FAULT");
        } else if message.contains("EXIT: VMMCALL") || message.contains("EXIT: VMCALL") {
            self.vmcall_exits += 1;
            self.total_exits += 1;
            self.last_exit_reason = String::from("HYPERCALL");
        } else if message.contains("CREATED") {
            self.vm_count += 1;
        } else if message.contains("STOPPED") || message.contains("TRIPLE FAULT") {
            if self.vm_count > 0 {
                self.vm_count -= 1;
            }
        }
        
        // Parse RIP from EXIT messages: "at RIP=0x..."
        if let Some(rip_position) = message.find("RIP=0x") {
            let rip_str = &message[rip_position + 6..];
            let end = rip_str.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(rip_str.len());
            if let Ok(rip) = u64::from_str_radix(&rip_str[..end], 16) {
                self.last_guest_rip = rip;
            }
        }
        
        // Parse REGS events
        if message.contains("REGS ") {
            self.parse_regs(message);
        }
    }
    
    /// Parse register values from a REGS event message
    fn parse_regs(&mut self, message: &str) {
        fn parse_hex(message: &str, prefix: &str) -> Option<u64> {
            let position = message.find(prefix)?;
            let start = position + prefix.len();
            let rest = &message[start..];
            let end = rest.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(rest.len());
            u64::from_str_radix(&rest[..end], 16).ok()
        }
        
        if let Some(v) = parse_hex(message, "RAX=0x") { self.regs_rax = v; }
        if let Some(v) = parse_hex(message, "RBX=0x") { self.regs_rbx = v; }
        if let Some(v) = parse_hex(message, "RCX=0x") { self.regs_rcx = v; }
        if let Some(v) = parse_hex(message, "RDX=0x") { self.regs_rdx = v; }
        if let Some(v) = parse_hex(message, "RIP=0x") { self.regs_rip = v; }
        if let Some(v) = parse_hex(message, "RSP=0x") { self.regs_rsp = v; }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT};
                // Pattern matching — Rust's exhaustive branching construct.
match key {
            k if k == KEY_LEFT => {
                if self.active_tab > 0 { self.active_tab -= 1; }
            }
            k if k == KEY_RIGHT => {
                if self.active_tab < 3 { self.active_tab += 1; }
            }
            k if k == KEY_UP => {
                if self.scroll > 0 { self.scroll -= 1; }
            }
            k if k == KEY_DOWN => {
                self.scroll += 1;
            }
            b'1' => self.active_tab = 0, // Overview
            b'2' => self.active_tab = 1, // Exits
            b'3' => self.active_tab = 2, // Memory map
            b'4' => self.active_tab = 3, // Registers
            _ => {}
        }
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, lx: i32, ly: i32, w: u32, _h: u32) {
        let character = crate::graphics::scaling::char_height() as i32;
        let cw = crate::graphics::scaling::char_width() as i32;
        
        // Tab bar click (first row)
        if ly < character + 4 && cw > 0 {
            let tab_w = w as i32 / 4;
            let tab = (lx / tab_w).minimum(3) as usize;
            self.active_tab = tab;
        }
    }
}

// ── Drawing ────────────────────────────────────────────────────────────────

/// Draw the VM Inspector panel content
pub fn draw(state: &VmInspectorState, x: i32, y: i32, w: u32, h: u32) {
    let character = super::char_h();
    let cw = super::char_w();
    if character <= 0 || cw <= 0 { return; }
    
    // Tab bar
    let tabs = ["Overview", "VM Exits", "Memory", "Registers"];
    let tab_w = w as i32 / 4;
    for (i, tab) in tabs.iter().enumerate() {
        let transmit = x + i as i32 * tab_w;
        let color = if i == state.active_tab { COLUMN_HV } else { COLUMN_DIM };
        let bg = if i == state.active_tab { 0xFF1F2937 } else { 0xFF0D1117 };
        crate::framebuffer::fill_rect(transmit as u32, y as u32, tab_w as u32, character as u32, bg);
        super::draw_lab_text(transmit + 4, y + 2, tab, color);
    }
    // Tab separator
    crate::framebuffer::fill_rect(x as u32, (y + character) as u32, w, 1, COLUMN_HV);
    
    let content_y = y + character + 4;
    let content_h = h.saturating_sub(character as u32 + 4);
    
        // Pattern matching — Rust's exhaustive branching construct.
match state.active_tab {
        0 => draw_overview(state, x, content_y, w, content_h),
        1 => draw_exits(state, x, content_y, w, content_h),
        2 => draw_memory_map(state, x, content_y, w, content_h),
        3 => draw_registers(state, x, content_y, w, content_h),
        _ => {}
    }
}

/// Draw the overview tab
fn draw_overview(state: &VmInspectorState, x: i32, y: i32, w: u32, h: u32) {
    let character = super::char_h();
    let mut cy = y;
    
    // VM Status
    let status_color = if state.vm_count > 0 { COLUMN_GREEN } else { COLUMN_DIM };
    let status = if state.vm_count > 0 { "RUNNING" } else { "NO VM" };
    super::draw_lab_text(x, cy, &format!("Status: {}", status), status_color);
    cy += character;
    
    super::draw_lab_text(x, cy, &format!("VMs Active: {}", state.vm_count), COLUMN_TEXT);
    cy += character;
    
    super::draw_lab_text(x, cy, &format!("Total Exits: {}", state.total_exits), COLUMN_YELLOW);
    cy += character;
    
    super::draw_lab_text(x, cy, &format!("Last Exit: {}", state.last_exit_reason), COLUMN_ORANGE);
    cy += character;
    
    if state.last_guest_rip != 0 {
        super::draw_lab_text(x, cy, &format!("Guest RIP: 0x{:X}", state.last_guest_rip), COLUMN_CYAN);
        cy += character;
    }
    
    // Backend info
    cy += character / 2;
    let backend = crate::hypervisor::backend_information();
    super::draw_lab_text(x, cy, &format!("Backend: {}", backend), COLUMN_PURPLE);
    cy += character;
    
    // Separator
    cy += character / 2;
    crate::framebuffer::fill_rect(x as u32, cy as u32, w, 1, 0xFF30363D);
    cy += 4;
    
    // Recent events (last 10 HV events)
    super::draw_lab_text(x, cy, "Recent VM Events:", COLUMN_HV);
    cy += character;
    
    let maximum_events = ((h as i32 - (cy - y)) / character).maximum(0) as usize;
    let start = if state.events.len() > maximum_events {
        state.events.len() - maximum_events
    } else {
        0
    };
    
    for event in state.events[start..].iter() {
        if cy + character > y + h as i32 { break; }
        
        let ts = event.timestamp_mouse;
        let secs = ts / 1000;
        let mouse = ts % 1000;
        let time_str = format!("{:02}:{:02}.{:03}", secs / 60, secs % 60, mouse);
        
        // Truncate message to fit
        let maximum_chars = ((w as i32 - 80) / super::char_w()).maximum(10) as usize;
        let message = if event.message.len() > maximum_chars {
            &event.message[..maximum_chars]
        } else {
            &event.message
        };
        
        super::draw_lab_text(x, cy, &time_str, COLUMN_DIM);
        super::draw_lab_text(x + 70, cy, message, COLUMN_TEXT);
        cy += character;
    }
}

/// Draw the exit statistics tab
fn draw_exits(state: &VmInspectorState, x: i32, y: i32, w: u32, _h: u32) {
    let character = super::char_h();
    let mut cy = y;
    
    super::draw_lab_text(x, cy, "VM Exit Breakdown:", COLUMN_HV);
    cy += character + 4;
    
    // Exit type bars
    let exits = [
        ("CPUID", state.cpuid_exits, COLUMN_CYAN),
        ("I/O", state.io_exits, COLUMN_GREEN),
        ("MSR", state.msr_exits, COLUMN_PURPLE),
        ("HLT", state.hlt_exits, COLUMN_YELLOW),
        ("NPF/EPT", state.npf_exits, COLUMN_RED),
        ("VMCALL", state.vmcall_exits, COLUMN_ORANGE),
    ];
    
    let maximum_count = exits.iter().map(|(_, c, _)| *c).maximum().unwrap_or(1).maximum(1);
    let bar_maximum_w = w.saturating_sub(120) as u64;
    
    for (name, count, color) in &exits {
        let label = format!("{:<8} {:>8}", name, count);
        super::draw_lab_text(x, cy, &label, COLUMN_TEXT);
        
        // Progress bar
        let bar_w = if *count > 0 {
            ((*count as u64 * bar_maximum_w) / maximum_count).maximum(2)
        } else {
            0
        };
        if bar_w > 0 {
            crate::framebuffer::fill_rect(
                (x + 120) as u32, (cy + 2) as u32,
                bar_w as u32, (character - 4) as u32,
                *color,
            );
        }
        
        cy += character;
    }
    
    // Total
    cy += character / 2;
    super::draw_lab_text(x, cy, &format!("Total: {}", state.total_exits), COLUMN_TEXT);
    cy += character;
    
    // Exits per second (approximate)
    if state.frame > 0 {
        let eps = state.total_exits * 60 / state.frame.maximum(1);
        super::draw_lab_text(x, cy, &format!("Rate: ~{} exits/sec", eps), COLUMN_DIM);
    }
}

/// Draw the memory map tab — uses VMI engine for real guest memory layout
fn draw_memory_map(_state: &VmInspectorState, x: i32, y: i32, w: u32, h: u32) {
    let character = super::char_h();
    let mut cy = y;
    
    super::draw_lab_text(x, cy, "Guest Physical Memory Map:", COLUMN_HV);
    cy += character + 4;
    
    // Use VMI engine to build real memory map
    let regions = crate::hypervisor::vmi::build_guest_memory_map(64);
    
    for region in &regions {
        if cy + character > y + h as i32 { break; }
        let color = // Pattern matching — Rust's exhaustive branching construct.
match region.region_type {
            crate::hypervisor::vmi::MemoryRegionType::Ram => COLUMN_GREEN,
            crate::hypervisor::vmi::MemoryRegionType::Mmio => COLUMN_RED,
            crate::hypervisor::vmi::MemoryRegionType::Rom => COLUMN_YELLOW,
            crate::hypervisor::vmi::MemoryRegionType::Reserved => COLUMN_DIM,
            crate::hypervisor::vmi::MemoryRegionType::AcpiReclaimable => COLUMN_PURPLE,
            crate::hypervisor::vmi::MemoryRegionType::Unmapped => COLUMN_DIM,
        };
        
        let size_keyboard = region.size / 1024;
        let size_str = if size_keyboard >= 1024 {
            format!("{:>4} MB", size_keyboard / 1024)
        } else {
            format!("{:>4} KB", size_keyboard)
        };
        
        let label = format!("0x{:09X} {} {}", region.base, size_str, region.label);
        super::draw_lab_text(x, cy, &label, color);
        cy += character;
    }
    
    // VMI status
    cy += character;
    let vmi_status = if crate::hypervisor::vmi::is_enabled() { "ENABLED" } else { "DISABLED" };
    let vmi_color = if crate::hypervisor::vmi::is_enabled() { COLUMN_GREEN } else { COLUMN_DIM };
    super::draw_lab_text(x, cy, &format!("VMI Engine: {}", vmi_status), vmi_color);
    cy += character;
    
    // List live VMs
    let vms = crate::hypervisor::vmi::list_all_vms();
    if !vms.is_empty() {
        super::draw_lab_text(x, cy, "Live VMs:", COLUMN_HV);
        cy += character;
        for (id, name, state_str) in &vms {
            if cy + character > y + h as i32 { break; }
            let color = // Pattern matching — Rust's exhaustive branching construct.
match *state_str {
                "running" => COLUMN_GREEN,
                "created" => COLUMN_CYAN,
                "paused" => COLUMN_YELLOW,
                "stopped" => COLUMN_RED,
                _ => COLUMN_DIM,
            };
            super::draw_lab_text(x, cy, &format!("  VM #{}: {} [{}]", id, name, state_str), color);
            cy += character;
        }
    } else {
        super::draw_lab_text(x, cy, "No VMs created yet", COLUMN_DIM);
    }
}

/// Draw the registers tab
fn draw_registers(state: &VmInspectorState, x: i32, y: i32, _w: u32, _h: u32) {
    let character = super::char_h();
    let mut cy = y;
    
    super::draw_lab_text(x, cy, "Guest Registers (last snapshot):", COLUMN_HV);
    cy += character + 4;
    
    let half_w = 200i32;
    
    // Left column: general purpose
    let regs_left = [
        ("RAX", state.regs_rax),
        ("RBX", state.regs_rbx),
        ("RCX", state.regs_rcx),
        ("RDX", state.regs_rdx),
    ];
    
    let regs_right = [
        ("RIP", state.regs_rip),
        ("RSP", state.regs_rsp),
        ("Last Exit RIP", state.last_guest_rip),
    ];
    
    for (name, value) in &regs_left {
        let color = if *value != 0 { COLUMN_GREEN } else { COLUMN_DIM };
        super::draw_lab_text(x, cy, &format!("{:<4} = 0x{:016X}", name, value), color);
        cy += character;
    }
    
    cy += character / 2;
    
    for (name, value) in &regs_right {
        let color = if *value != 0 { COLUMN_CYAN } else { COLUMN_DIM };
        super::draw_lab_text(x, cy, &format!("{:<14} = 0x{:016X}", name, value), color);
        cy += character;
    }
    
    // Exit context
    cy += character;
    super::draw_lab_text(x, cy, &format!("Last Exit: {}", state.last_exit_reason), COLUMN_ORANGE);
    cy += character;
    super::draw_lab_text(x, cy, &format!("Total Exits: {}", state.total_exits), COLUMN_YELLOW);
}
