//! TrustLab — Real-time Educational OS Introspection Laboratory
//!
//! A 6-panel live dashboard that lets users observe kernel internals in real time:
//!   ┌──────────────┬──────────────┬──────────────┐
//!   │  Hardware    │  Live Kernel │   Command    │
//!   │  Status      │  Trace       │   Guide      │
//!   │              ├──────────────┤              │
//!   │              │  Pipeline    │              │
//!   │              │  View        │              │
//!   ├──────────────┼──────────────┼──────────────┤
//!   │  File System │  TrustLang   │  Hex Editor  │
//!   │  Tree        │  Editor      │              │
//!   └──────────────┴──────────────┴──────────────┘
//!
//! No other bare-metal OS has a feature like this.

extern crate alloc;

pub mod trace_bus;
pub mod hardware;
pub mod guide;
pub mod filetree;
pub mod editor;
pub mod kernel_trace;
pub mod pipeline;
pub mod hex_editor;
pub mod demo;
pub mod ux_test;
pub mod vm_inspector;
pub mod network_panel;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_PGUP, KEY_PGDOWN};

// ── Layout constants ───────────────────────────────────────────────────────
const TITLE_BAR_HEIGHT: u32 = 28;
const PANEL_BORDER: u32 = 1;
const PANEL_PADDING: u32 = 6;
const PANEL_HEADER_H: u32 = 22;
const SHELL_BAR_H: u32 = 28;

// ── Colors (dark theme, educational) ───────────────────────────────────────
const COL_BG: u32        = 0xFF0D1117;  // Deep dark background
const COL_PANEL_BG: u32  = 0xFF161B22;  // Panel background
const COL_PANEL_BORDER: u32 = 0xFF30363D; // Panel border
const COL_HEADER_BG: u32 = 0xFF1C2128;  // Panel header
const COL_TEXT: u32       = 0xFFE6EDF3;  // Normal text
const COL_DIM: u32        = 0xFF8B949E;  // Dimmed text
const COL_ACCENT: u32     = 0xFF58A6FF;  // Blue accent
const COL_GREEN: u32      = 0xFF3FB950;  // Green (good/alloc)
const COL_YELLOW: u32     = 0xFFD29922;  // Yellow (warning)
const COL_RED: u32        = 0xFFF85149;  // Red (error/dealloc)
const COL_PURPLE: u32     = 0xFFBC8CFF;  // Purple (syscall)
const COL_CYAN: u32       = 0xFF79C0FF;  // Cyan (filesystem)
const COL_ORANGE: u32     = 0xFFD18616;  // Orange (interrupt)
const COL_SHELL_BG: u32   = 0xFF0D1117;  // Shell bar
const COL_SHELL_PROMPT: u32 = 0xFF3FB950;
const COL_SELECTED: u32   = 0xFF1F6FEB;  // Selected panel highlight

/// Global flag: is Lab Mode active? (checked by trace hooks)
pub static LAB_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Frame counter for animations
static LAB_FRAME: AtomicU64 = AtomicU64::new(0);

/// Which panel is currently focused (0-5)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanelId {
    HardwareStatus = 0,
    KernelTrace = 1,
    CommandGuide = 2,
    FileTree = 3,
    TrustLangEditor = 4,
    Pipeline = 5,
    HexEditor = 6,
    VmInspector = 7,
    NetworkDashboard = 8,
}

impl PanelId {
    pub(crate) fn from_index(i: usize) -> Self {
        match i {
            0 => PanelId::HardwareStatus,
            1 => PanelId::KernelTrace,
            2 => PanelId::CommandGuide,
            3 => PanelId::FileTree,
            4 => PanelId::TrustLangEditor,
            5 => PanelId::Pipeline,
            6 => PanelId::HexEditor,
            7 => PanelId::VmInspector,
            _ => PanelId::NetworkDashboard,
        }
    }
    
    fn title(&self) -> &'static str {
        match self {
            PanelId::HardwareStatus => "⚙ Hardware Status",
            PanelId::KernelTrace => "◈ Live Kernel Trace",
            PanelId::CommandGuide => "📖 Command Guide",
            PanelId::FileTree => "📁 File System Tree",
            PanelId::TrustLangEditor => "⌨ TrustLang Editor",
            PanelId::Pipeline => "⚙ Pipeline View",
            PanelId::HexEditor => "🔍 Hex Editor",
            PanelId::VmInspector => "🖥 VM Inspector",
            PanelId::NetworkDashboard => "🌐 Network",
        }
    }
    
    fn icon_color(&self) -> u32 {
        match self {
            PanelId::HardwareStatus => COL_GREEN,
            PanelId::KernelTrace => COL_ORANGE,
            PanelId::CommandGuide => COL_ACCENT,
            PanelId::FileTree => COL_CYAN,
            PanelId::TrustLangEditor => COL_PURPLE,
            PanelId::Pipeline => COL_YELLOW,
            PanelId::HexEditor => COL_RED,
            PanelId::VmInspector => 0xFFFF6B6B,
            PanelId::NetworkDashboard => 0xFF00CED1,
        }
    }

    /// All module types in order
    pub fn all() -> [PanelId; 9] {
        [
            PanelId::HardwareStatus, PanelId::KernelTrace, PanelId::CommandGuide,
            PanelId::FileTree, PanelId::TrustLangEditor, PanelId::Pipeline,
            PanelId::HexEditor, PanelId::VmInspector, PanelId::NetworkDashboard,
        ]
    }

    /// Short display name for switcher UI
    pub fn short_name(&self) -> &'static str {
        match self {
            PanelId::HardwareStatus => "Hardware",
            PanelId::KernelTrace => "Kernel Trace",
            PanelId::CommandGuide => "Cmd Guide",
            PanelId::FileTree => "File Tree",
            PanelId::TrustLangEditor => "TrustLang",
            PanelId::Pipeline => "Pipeline",
            PanelId::HexEditor => "Hex Editor",
            PanelId::VmInspector => "VM Inspector",
            PanelId::NetworkDashboard => "Network",
        }
    }

    /// Category label for switcher UI
    pub fn category(&self) -> &'static str {
        match self {
            PanelId::VmInspector => "Hypervisor",
            PanelId::NetworkDashboard => "Network",
            _ => "Core",
        }
    }
}

// ── Module Switcher ────────────────────────────────────────────────────────

/// Module switcher overlay (appears when user wants to swap a module in a slot)
pub struct ModuleSwitcher {
    /// Whether the switcher is currently visible
    pub open: bool,
    /// Which slot position we're swapping
    pub target_slot: usize,
    /// Currently highlighted entry in the module list
    pub selected: usize,
}

impl ModuleSwitcher {
    pub fn new() -> Self {
        Self { open: false, target_slot: 0, selected: 0 }
    }
}

// ── Layout Presets ─────────────────────────────────────────────────────────

/// Default layout (original TrustLab arrangement)
pub const LAYOUT_DEFAULT: [PanelId; 7] = [
    PanelId::HardwareStatus, PanelId::KernelTrace, PanelId::CommandGuide,
    PanelId::FileTree, PanelId::TrustLangEditor, PanelId::Pipeline, PanelId::HexEditor,
];

/// Developer-focused layout
const LAYOUT_DEV: [PanelId; 7] = [
    PanelId::KernelTrace, PanelId::TrustLangEditor, PanelId::CommandGuide,
    PanelId::FileTree, PanelId::HexEditor, PanelId::Pipeline, PanelId::HardwareStatus,
];

/// Monitoring-focused layout
const LAYOUT_MONITOR: [PanelId; 7] = [
    PanelId::HardwareStatus, PanelId::KernelTrace, PanelId::Pipeline,
    PanelId::HexEditor, PanelId::FileTree, PanelId::KernelTrace, PanelId::CommandGuide,
];

/// Slot position names (for user reference)
const SLOT_NAMES: [&str; 7] = [
    "Top-Left", "Mid-Top", "Top-Right", "Bot-Left", "Mid-Bot", "Mid-Embed", "Bot-Right",
];

/// TrustLab state (one per window)
pub struct LabState {
    /// Which slot position is currently focused (0-6)
    pub focused_slot: usize,
    /// Module assignment: which module type is loaded in each slot
    pub slot_assignment: [PanelId; 7],
    /// Module switcher overlay state
    pub switcher: ModuleSwitcher,
    /// Shell command input buffer
    pub shell_input: String,
    /// Shell cursor position
    pub shell_cursor: usize,
    /// Sub-states per module type (all always alive, slots just select which to display)
    pub hw_state: hardware::HardwareState,
    pub trace_state: kernel_trace::KernelTraceState,
    pub guide_state: guide::GuideState,
    pub tree_state: filetree::FileTreeState,
    pub editor_state: editor::EditorState,
    pub pipeline_state: pipeline::PipelineState,
    pub hex_state: hex_editor::HexEditorState,
    pub vm_inspector_state: vm_inspector::VmInspectorState,
    pub network_panel_state: network_panel::NetworkPanelState,
    pub demo_state: demo::DemoState,
    /// Frame counter
    pub frame: u64,
    /// Whether to auto-scroll trace panels
    pub auto_scroll: bool,
}

impl LabState {
    pub fn new() -> Self {
        LAB_ACTIVE.store(true, Ordering::SeqCst);
        Self {
            focused_slot: 0,
            slot_assignment: LAYOUT_DEFAULT,
            switcher: ModuleSwitcher::new(),
            shell_input: String::new(),
            shell_cursor: 0,
            hw_state: hardware::HardwareState::new(),
            trace_state: kernel_trace::KernelTraceState::new(),
            guide_state: guide::GuideState::new(),
            tree_state: filetree::FileTreeState::new(),
            editor_state: editor::EditorState::new(),
            pipeline_state: pipeline::PipelineState::new(),
            hex_state: hex_editor::HexEditorState::new(),
            vm_inspector_state: vm_inspector::VmInspectorState::new(),
            network_panel_state: network_panel::NetworkPanelState::new(),
            demo_state: demo::DemoState::new(),
            frame: 0,
            auto_scroll: true,
        }
    }
    
    /// Handle keyboard input
    pub fn handle_key(&mut self, key: u8) {
        // Module switcher intercepts all keys when open
        if self.switcher.open {
            self.handle_switcher_key(key);
            return;
        }

        // If demo is running, intercept keys
        if self.demo_state.active {
            self.demo_state.handle_key(key);
            return;
        }

        // Tab = cycle focused slot (all 7 slots are now independent)
        if key == 0x09 {
            self.focused_slot = (self.focused_slot + 1) % 7;
            return;
        }
        
        // Enter in shell bar → execute command 
        // (but if editor or filetree is focused, let them handle Enter)
        if key == 0x0D || key == 0x0A {
            if self.focused_module() == PanelId::TrustLangEditor {
                self.editor_state.handle_key(key);
                return;
            }
            if self.focused_module() == PanelId::FileTree {
                self.tree_state.handle_key(key);
                return;
            }
            if !self.shell_input.is_empty() {
                self.execute_shell_command();
                return;
            }
        }
        
        // Backspace in shell bar 
        if key == 0x08 {
            // If focused on editor, let it handle backspace
            if self.focused_module() == PanelId::TrustLangEditor {
                self.editor_state.handle_key(key);
                return;
            }
            // Otherwise treat as shell bar backspace
            if self.shell_cursor > 0 {
                self.shell_cursor -= 1;
                self.shell_input.remove(self.shell_cursor);
            }
            return;
        }
        
        // Dispatch to focused module
        self.dispatch_key(key);
    }

    /// Dispatch a key press to the currently focused module
    fn dispatch_key(&mut self, key: u8) {
        match self.focused_module() {
            PanelId::HardwareStatus => self.hw_state.handle_key(key),
            PanelId::KernelTrace => self.trace_state.handle_key(key),
            PanelId::CommandGuide => self.guide_state.handle_key(key),
            PanelId::FileTree => self.tree_state.handle_key(key),
            PanelId::TrustLangEditor => self.editor_state.handle_key(key),
            PanelId::Pipeline => self.pipeline_state.handle_key(key),
            PanelId::HexEditor => self.hex_state.handle_key(key),
            PanelId::VmInspector => self.vm_inspector_state.handle_key(key),
            PanelId::NetworkDashboard => self.network_panel_state.handle_key(key),
        }
    }
    
    /// Handle character input (printable)
    pub fn handle_char(&mut self, ch: char) {
        if self.switcher.open { return; }

        if self.demo_state.active {
            // Forward space to demo as key skip
            if ch == ' ' {
                self.demo_state.handle_key(0x20);
            }
            return;
        }

        match self.focused_module() {
            PanelId::TrustLangEditor => self.editor_state.handle_char(ch),
            PanelId::CommandGuide => self.guide_state.handle_char(ch),
            _ => {
                // Route to shell bar for all other panels
                self.shell_input.insert(self.shell_cursor, ch);
                self.shell_cursor += 1;
            }
        }
    }
    
    /// Execute a shell bar command
    pub(crate) fn execute_shell_command(&mut self) {
        let raw: String = self.shell_input.trim().chars().collect();
        let cmd: String = raw.chars().map(|c| c.to_ascii_lowercase()).collect();
        self.shell_input.clear();
        self.shell_cursor = 0;
        
        match cmd.as_str() {
            "hw" | "hardware" | "cpu" => {
                self.focus_module(PanelId::HardwareStatus);
            }
            "trace" | "log" | "events" => {
                self.focus_module(PanelId::KernelTrace);
            }
            "help" | "guide" | "commands" | "cmd" => {
                self.focus_module(PanelId::CommandGuide);
            }
            "fs" | "files" | "tree" | "ls" => {
                self.focus_module(PanelId::FileTree);
                self.tree_state.dirty = true;
                self.tree_state.handle_key(b'R'); // force refresh
            }
            "edit" | "editor" | "code" | "trustlang" => {
                self.focus_module(PanelId::TrustLangEditor);
            }
            "live" | "stream" | "bus" | "pipeline" | "pipe" => {
                self.focus_module(PanelId::Pipeline);
            }
            "hex" | "hexedit" | "hexdump" => {
                self.focus_module(PanelId::HexEditor);
            }
            "vm" | "vmi" | "inspector" | "hypervisor" => {
                self.focus_module(PanelId::VmInspector);
            }
            "net" | "network" | "tcp" | "packets" => {
                self.focus_module(PanelId::NetworkDashboard);
            }
            _ if cmd.starts_with("hex ") => {
                let path = raw[4..].trim();
                if !path.is_empty() {
                    self.hex_state.load_file(path);
                    self.focus_module(PanelId::HexEditor);
                }
            }
            "clear" | "cls" => {
                self.trace_state.events.clear();
                self.pipeline_state.flows.clear();
            }
            "demo" | "showcase" | "present" => {
                self.demo_state.start();
            }
            "refresh" | "r" => {
                self.tree_state.handle_key(b'R');
                self.hw_state.force_refresh();
            }
            "run" | "f5" => {
                self.editor_state.run_code();
                self.focus_module(PanelId::TrustLangEditor);
            }
            "test" | "labtest" | "uxtest" => {
                ux_test::run_ux_tests(self);
            }
            // ── Module Switcher commands ────────────────────────────
            "swap" | "module" | "switch" => {
                self.open_switcher(self.focused_slot);
            }
            // ── Layout preset commands ──────────────────────────────
            _ if cmd.starts_with("layout ") => {
                let preset = cmd[7..].trim();
                match preset {
                    "default" | "reset" => {
                        self.slot_assignment = LAYOUT_DEFAULT;
                        trace_bus::emit_static(trace_bus::EventCategory::Custom, "Layout: default", 0);
                    }
                    "dev" | "developer" => {
                        self.slot_assignment = LAYOUT_DEV;
                        trace_bus::emit_static(trace_bus::EventCategory::Custom, "Layout: developer", 0);
                    }
                    "monitor" | "mon" => {
                        self.slot_assignment = LAYOUT_MONITOR;
                        trace_bus::emit_static(trace_bus::EventCategory::Custom, "Layout: monitor", 0);
                    }
                    _ => {
                        trace_bus::emit_static(trace_bus::EventCategory::Custom, "Unknown layout (try: default, dev, monitor)", 0);
                    }
                }
            }
            "slots" | "layout" => {
                // Show current slot assignment in trace
                for (i, module) in self.slot_assignment.iter().enumerate() {
                    let msg = format!("Slot {} [{}]: {}", i, SLOT_NAMES[i], module.short_name());
                    trace_bus::emit(trace_bus::EventCategory::Custom, msg, i as u64);
                }
                self.focus_module(PanelId::KernelTrace);
            }
            _ => {
                // Unknown command — show in trace
                trace_bus::emit_static(
                    trace_bus::EventCategory::Custom,
                    "lab> unknown command",
                    0,
                );
            }
        }
    }
    
    /// Handle mouse click (coordinates relative to window content area)
    pub fn handle_click(&mut self, rx: i32, ry: i32, ww: u32, wh: u32) {
        // If switcher is open, handle click on switcher or close it
        if self.switcher.open {
            self.handle_switcher_click(rx, ry, ww, wh);
            return;
        }

        let cx = 2i32;
        let cy = TITLE_BAR_HEIGHT as i32 + 2;
        let cw = ww.saturating_sub(4);
        let ch = wh.saturating_sub(TITLE_BAR_HEIGHT + 4);
        if cw < 200 || ch < 100 { return; }

        let panels = compute_panels(cx, cy, cw, ch);

        // Check if click is inside a panel
        for (i, pr) in panels.iter().enumerate() {
            if rx >= pr.x && rx < pr.x + pr.w as i32
                && ry >= pr.y && ry < pr.y + pr.h as i32
            {
                self.focused_slot = i;
                let pid = self.slot_assignment[i];

                // Check if click is on the ▼ swap button in header
                let swap_btn_x = pr.x + pr.w as i32 - 24;
                if ry < pr.y + PANEL_HEADER_H as i32 && rx >= swap_btn_x {
                    self.open_switcher(i);
                    return;
                }

                // Content area coordinates (same as draw_lab)
                let content_x = pr.x + PANEL_PADDING as i32;
                let content_y = pr.y + PANEL_HEADER_H as i32 + PANEL_PADDING as i32;
                let content_w = pr.w.saturating_sub(PANEL_PADDING * 2);
                let content_h = pr.h.saturating_sub(PANEL_HEADER_H + PANEL_PADDING * 2);
                let local_x = rx - content_x;
                let local_y = ry - content_y;

                // Dispatch click to the module loaded in this slot
                self.dispatch_click(pid, local_x, local_y, content_w, content_h);
                return;
            }
        }

        // Click on shell bar — focus stays, position cursor
        let gap = 4u32;
        let shell_y = cy + (ch - SHELL_BAR_H) as i32;
        if ry >= shell_y && ry < shell_y + SHELL_BAR_H as i32 {
            let cw_px = char_w();
            if cw_px > 0 {
                let prompt_len = 5; // "lab> "
                let input_x = cx + 8 + prompt_len * cw_px;
                let click_col = ((rx - input_x) / cw_px).max(0) as usize;
                self.shell_cursor = click_col.min(self.shell_input.len());
            }
        }
    }

    /// Dispatch a click to the appropriate module state
    fn dispatch_click(&mut self, pid: PanelId, lx: i32, ly: i32, w: u32, h: u32) {
        match pid {
            PanelId::HardwareStatus => self.hw_state.handle_click(lx, ly, w, h),
            PanelId::KernelTrace => self.trace_state.handle_click(lx, ly, w, h),
            PanelId::CommandGuide => self.guide_state.handle_click(lx, ly, w, h),
            PanelId::FileTree => self.tree_state.handle_click(lx, ly, w, h),
            PanelId::TrustLangEditor => self.editor_state.handle_click(lx, ly, w, h),
            PanelId::Pipeline => self.pipeline_state.handle_click(lx, ly, w, h),
            PanelId::HexEditor => self.hex_state.handle_click(lx, ly, w, h),
            PanelId::VmInspector => self.vm_inspector_state.handle_click(lx, ly, w, h),
            PanelId::NetworkDashboard => self.network_panel_state.handle_click(lx, ly, w, h),
        }
    }

    /// Update per-frame state
    pub fn tick(&mut self) {
        self.frame += 1;
        self.hw_state.update();
        self.trace_state.update();
        self.pipeline_state.update();
        self.vm_inspector_state.update();
        // Demo tick: auto-focus panels
        if let Some(panel_idx) = self.demo_state.tick() {
            // Demo uses original slot indices
            self.focused_slot = panel_idx.min(6);
        }
    }

    // ── Slot helpers ───────────────────────────────────────────────────

    /// Get the module type loaded in the currently focused slot
    pub fn focused_module(&self) -> PanelId {
        self.slot_assignment[self.focused_slot]
    }

    /// Find the first slot containing a given module type
    pub fn slot_of(&self, module: PanelId) -> Option<usize> {
        self.slot_assignment.iter().position(|m| *m == module)
    }

    /// Focus on a specific module type (finds its slot)
    pub fn focus_module(&mut self, module: PanelId) {
        if let Some(slot) = self.slot_of(module) {
            self.focused_slot = slot;
        }
    }

    /// Open the module switcher for a given slot
    pub fn open_switcher(&mut self, slot: usize) {
        self.switcher.open = true;
        self.switcher.target_slot = slot;
        let current = self.slot_assignment[slot];
        self.switcher.selected = PanelId::all().iter().position(|p| *p == current).unwrap_or(0);
    }

    /// Handle keyboard input when the module switcher is open
    fn handle_switcher_key(&mut self, key: u8) {
        match key {
            0x1B => {
                // Esc: close switcher
                self.switcher.open = false;
            }
            0x0D | 0x0A => {
                // Enter: apply selection
                let all = PanelId::all();
                if self.switcher.selected < all.len() {
                    let module = all[self.switcher.selected];
                    let slot = self.switcher.target_slot;
                    self.slot_assignment[slot] = module;
                    let msg = format!("Slot {} [{}] -> {}",
                        slot, SLOT_NAMES[slot], module.short_name());
                    trace_bus::emit(trace_bus::EventCategory::Custom, msg, 0);
                }
                self.switcher.open = false;
            }
            k if k == KEY_UP => {
                if self.switcher.selected > 0 {
                    self.switcher.selected -= 1;
                }
            }
            k if k == KEY_DOWN => {
                let max = PanelId::all().len() - 1;
                if self.switcher.selected < max {
                    self.switcher.selected += 1;
                }
            }
            _ => {}
        }
    }

    /// Handle click on module switcher overlay
    fn handle_switcher_click(&mut self, rx: i32, ry: i32, ww: u32, wh: u32) {
        let cx = 2i32;
        let cy = TITLE_BAR_HEIGHT as i32 + 2;
        let cw = ww.saturating_sub(4);
        let ch = wh.saturating_sub(TITLE_BAR_HEIGHT + 4);
        if cw < 200 || ch < 100 { self.switcher.open = false; return; }

        let panels = compute_panels(cx, cy, cw, ch);
        let slot = self.switcher.target_slot;
        if slot >= panels.len() { self.switcher.open = false; return; }
        let pr = &panels[slot];

        // Check if click is inside the switcher overlay area
        let pad = 4i32;
        let ox = pr.x + pad;
        let oy = pr.y + pad;
        let ow = pr.w.saturating_sub(8);
        let oh = pr.h.saturating_sub(8);

        if rx >= ox && rx < ox + ow as i32 && ry >= oy && ry < oy + oh as i32 {
            // Calculate which entry was clicked
            let list_y_start = oy + 24; // after title + separator
            let row_h = char_h();
            if row_h > 0 && ry >= list_y_start {
                let entry = ((ry - list_y_start) / row_h) as usize;
                if entry < PanelId::all().len() {
                    self.switcher.selected = entry;
                    // Double-click = select (single click just highlights)
                    // For simplicity, single click selects and applies
                    let module = PanelId::all()[entry];
                    self.slot_assignment[slot] = module;
                    let msg = format!("Slot {} [{}] -> {}",
                        slot, SLOT_NAMES[slot], module.short_name());
                    trace_bus::emit(trace_bus::EventCategory::Custom, msg, 0);
                    self.switcher.open = false;
                }
            }
        } else {
            // Click outside overlay = close
            self.switcher.open = false;
        }
    }
}

impl Drop for LabState {
    fn drop(&mut self) {
        // If no more lab windows, deactivate
        LAB_ACTIVE.store(false, Ordering::SeqCst);
    }
}

// ── Drawing ────────────────────────────────────────────────────────────────

/// Compute the 6 panel rects given window content area
pub(crate) struct PanelRect {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: u32,
    pub(crate) h: u32,
}

pub(crate) fn compute_panels(cx: i32, cy: i32, cw: u32, ch: u32) -> [PanelRect; 7] {
    let gap = 4u32;
    // Reserve bottom for shell bar
    let content_h = ch.saturating_sub(SHELL_BAR_H + gap);
    
    let col_w = (cw.saturating_sub(gap * 2)) / 3;
    let row_h = (content_h.saturating_sub(gap)) / 2;
    
    let x0 = cx;
    let x1 = cx + col_w as i32 + gap as i32;
    let x2 = cx + (col_w as i32 + gap as i32) * 2;
    let y0 = cy;
    let y1 = cy + row_h as i32 + gap as i32;
    
    // Right column: clamp width so it doesn't overflow the window
    let col2_w = (cw as i32 - (x2 - cx)).max(40) as u32;
    
    // Top-middle is split in half vertically: Trace (top) + Pipeline (bottom)
    let trace_h = row_h.saturating_sub(gap) / 2;
    let pipe_h = row_h.saturating_sub(trace_h + gap);
    let pipe_y = y0 + trace_h as i32 + gap as i32;
    
    [
        PanelRect { x: x0, y: y0, w: col_w, h: row_h },          // 0: Hardware Status
        PanelRect { x: x1, y: y0, w: col_w, h: trace_h },        // 1: Kernel Trace (top half)
        PanelRect { x: x2, y: y0, w: col2_w, h: row_h },         // 2: Command Guide
        PanelRect { x: x0, y: y1, w: col_w, h: row_h },          // 3: File Tree
        PanelRect { x: x1, y: y1, w: col_w, h: row_h },          // 4: TrustLang Editor
        PanelRect { x: x1, y: pipe_y, w: col_w, h: pipe_h },     // 5: Pipeline View (bottom half)
        PanelRect { x: x2, y: y1, w: col2_w, h: row_h },         // 6: Hex Editor
    ]
}

/// Draw the entire TrustLab interface
pub fn draw_lab(state: &LabState, wx: i32, wy: i32, ww: u32, wh: u32) {
    let cx = wx + 2;
    let cy = wy + TITLE_BAR_HEIGHT as i32 + 2;
    let cw = ww.saturating_sub(4);
    let ch = wh.saturating_sub(TITLE_BAR_HEIGHT + 4);
    
    if cw < 200 || ch < 100 {
        return;
    }
    
    // Background
    crate::framebuffer::fill_rect(cx as u32, cy as u32, cw, ch, COL_BG);
    
    // Compute panel layout
    let panels = compute_panels(cx, cy, cw, ch);
    
    // Draw each panel (module type comes from slot_assignment)
    for (i, pr) in panels.iter().enumerate() {
        let pid = state.slot_assignment[i];
        let focused = i == state.focused_slot;
        draw_panel_frame(pr, pid, focused);
        
        // Content area inside panel
        let content_x = pr.x + PANEL_PADDING as i32;
        let content_y = pr.y + PANEL_HEADER_H as i32 + PANEL_PADDING as i32;
        let content_w = pr.w.saturating_sub(PANEL_PADDING * 2);
        let content_h = pr.h.saturating_sub(PANEL_HEADER_H + PANEL_PADDING * 2);
        
        draw_module_content(state, pid, content_x, content_y, content_w, content_h);
    }
    
    // Draw shell bar at bottom
    let gap = 4u32;
    let shell_y = cy + (ch - SHELL_BAR_H) as i32;
    draw_shell_bar(state, cx, shell_y, cw, SHELL_BAR_H);

    // Module switcher overlay (drawn on top of panels)
    if state.switcher.open {
        draw_module_switcher(state, &panels);
    }

    // Demo overlay (drawn on top of everything)
    if state.demo_state.active {
        demo::draw_overlay(&state.demo_state, wx, wy, ww, wh);
    }
}

/// Draw a panel frame (border + header + title + swap button)
fn draw_panel_frame(pr: &PanelRect, pid: PanelId, focused: bool) {
    // Background
    crate::framebuffer::fill_rect(pr.x as u32, pr.y as u32, pr.w, pr.h, COL_PANEL_BG);
    
    // Border (highlight if focused)
    let border_color = if focused { COL_SELECTED } else { COL_PANEL_BORDER };
    draw_rect_border(pr.x, pr.y, pr.w, pr.h, border_color);
    
    // Header bar
    let hdr_bg = if focused { 0xFF1F2937 } else { COL_HEADER_BG };
    crate::framebuffer::fill_rect(
        (pr.x + 1) as u32, (pr.y + 1) as u32,
        pr.w.saturating_sub(2), PANEL_HEADER_H - 1,
        hdr_bg,
    );
    
    // Colored accent line at top
    crate::framebuffer::fill_rect(
        (pr.x + 1) as u32, (pr.y + 1) as u32,
        pr.w.saturating_sub(2), 2,
        pid.icon_color(),
    );
    
    // Title
    let title = pid.title();
    draw_lab_text(pr.x + 8, pr.y + 6, title, COL_TEXT);
    
    // Swap button (v) at right side of header
    let btn_x = pr.x + pr.w as i32 - 22;
    let btn_color = if focused { COL_ACCENT } else { COL_DIM };
    draw_lab_text(btn_x, pr.y + 6, "\u{25BC}", btn_color);
}

/// Draw the content of a module into a given area
fn draw_module_content(state: &LabState, pid: PanelId, x: i32, y: i32, w: u32, h: u32) {
    match pid {
        PanelId::HardwareStatus => hardware::draw(&state.hw_state, x, y, w, h),
        PanelId::KernelTrace => kernel_trace::draw(&state.trace_state, x, y, w, h),
        PanelId::CommandGuide => guide::draw(&state.guide_state, x, y, w, h),
        PanelId::FileTree => filetree::draw(&state.tree_state, x, y, w, h),
        PanelId::TrustLangEditor => editor::draw(&state.editor_state, x, y, w, h),
        PanelId::Pipeline => pipeline::draw(&state.pipeline_state, x, y, w, h),
        PanelId::HexEditor => hex_editor::draw(&state.hex_state, x, y, w, h),
        PanelId::VmInspector => vm_inspector::draw(&state.vm_inspector_state, x, y, w, h),
        PanelId::NetworkDashboard => network_panel::draw(&state.network_panel_state, x, y, w, h),
    }
}

/// Draw the module switcher overlay on top of the target slot
fn draw_module_switcher(state: &LabState, panels: &[PanelRect; 7]) {
    let slot = state.switcher.target_slot;
    if slot >= panels.len() { return; }
    let pr = &panels[slot];

    // Dark overlay background
    let pad = 2;
    let ox = pr.x + pad;
    let oy = pr.y + pad;
    let ow = pr.w.saturating_sub(4);
    let oh = pr.h.saturating_sub(4);
    crate::framebuffer::fill_rect(ox as u32, oy as u32, ow, oh, 0xFF0D1117);
    draw_rect_border(ox, oy, ow, oh, COL_ACCENT);

    // Title
    let title = format!("Select Module (Slot {})", slot);
    draw_lab_text(ox + 8, oy + 4, &title, COL_ACCENT);

    // Separator
    let sep_y = oy + 22;
    crate::framebuffer::fill_rect((ox + 1) as u32, sep_y as u32, ow.saturating_sub(2), 1, COL_PANEL_BORDER);

    // Module list
    let ch = char_h();
    let all = PanelId::all();
    let current = state.slot_assignment[slot];
    let mut row_y = sep_y + 4;

    for (i, module) in all.iter().enumerate() {
        if row_y + ch > oy + oh as i32 - ch { break; } // don't overflow

        let is_selected = i == state.switcher.selected;
        let is_current = *module == current;

        // Highlight selected row
        if is_selected {
            crate::framebuffer::fill_rect(
                (ox + 2) as u32, row_y as u32,
                ow.saturating_sub(4), ch as u32,
                0xFF1F6FEB,
            );
        }

        // Icon + label
        let icon = if is_current { "*" } else { " " };
        let suffix = if is_current { " [active]" } else { "" };
        let label = format!("{} {} {}{}", icon, module.short_name(), module.category(), suffix);
        let color = if is_selected { COL_TEXT } else if is_current { COL_GREEN } else { COL_DIM };
        draw_lab_text(ox + 8, row_y + 2, &label, color);

        // Color dot for module
        crate::framebuffer::fill_rect(
            (ox + ow as i32 - 16) as u32, (row_y + 4) as u32,
            8, 8, module.icon_color(),
        );

        row_y += ch;
    }

    // Bottom hint
    let hint_y = oy + oh as i32 - ch;
    draw_lab_text(ox + 8, hint_y, "Up/Down Enter Esc", COL_DIM);
}

/// Draw the bottom shell bar
fn draw_shell_bar(state: &LabState, x: i32, y: i32, w: u32, h: u32) {
    crate::framebuffer::fill_rect(x as u32, y as u32, w, h, COL_SHELL_BG);
    // Top border
    crate::framebuffer::fill_rect(x as u32, y as u32, w, 1, COL_PANEL_BORDER);
    
    // Prompt
    let prompt = "lab> ";
    draw_lab_text(x + 8, y + 7, prompt, COL_SHELL_PROMPT);
    
    // Input
    let input_x = x + 8 + (prompt.len() as i32 * char_w());
    if state.shell_input.is_empty() {
        draw_lab_text(input_x, y + 7, "hw|trace|fs|edit|hex|vm|net|swap|layout|run|test", COL_DIM);
    } else {
        draw_lab_text(input_x, y + 7, &state.shell_input, COL_TEXT);
    }
    
    // Cursor blink
    if (state.frame / 30) % 2 == 0 {
        let cursor_x = input_x + (state.shell_cursor as i32 * char_w());
        crate::framebuffer::fill_rect(cursor_x as u32, (y + 6) as u32, 2, 14, COL_ACCENT);
    }
    
    // Tab hint + active module on right side
    let panel_name = state.focused_module().title();
    let hint = format!("[Tab] cycle | swap | layout | {}", panel_name);
    let hint_x = x + w as i32 - (hint.len() as i32 * char_w()) - 8;
    draw_lab_text(hint_x, y + 7, &hint, COL_DIM);
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Draw text using the kernel's scaled text renderer
pub fn draw_lab_text(x: i32, y: i32, text: &str, color: u32) {
    crate::graphics::scaling::draw_scaled_text(x, y, text, color);
}

/// Character width for current scale
fn char_w() -> i32 {
    crate::graphics::scaling::char_width() as i32
}

/// Character height for current scale
fn char_h() -> i32 {
    16 * crate::graphics::scaling::get_scale_factor() as i32
}

/// Draw a 1px rect border (outline)
fn draw_rect_border(x: i32, y: i32, w: u32, h: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    // Top
    crate::framebuffer::fill_rect(x as u32, y as u32, w, 1, color);
    // Bottom
    crate::framebuffer::fill_rect(x as u32, (y + h as i32 - 1) as u32, w, 1, color);
    // Left
    crate::framebuffer::fill_rect(x as u32, y as u32, 1, h, color);
    // Right
    crate::framebuffer::fill_rect((x + w as i32 - 1) as u32, y as u32, 1, h, color);
}

/// Draw a horizontal progress bar
pub fn draw_progress_bar(x: i32, y: i32, w: u32, h: u32, pct: u32, fg: u32, bg: u32) {
    crate::framebuffer::fill_rect(x as u32, y as u32, w, h, bg);
    let fill_w = (w as u64 * pct.min(100) as u64 / 100) as u32;
    if fill_w > 0 {
        crate::framebuffer::fill_rect(x as u32, y as u32, fill_w, h, fg);
    }
}

/// Truncate a string to fit in pixel width
pub fn truncate_to_width(s: &str, max_w: u32) -> &str {
    let cw = char_w() as u32;
    if cw == 0 { return s; }
    let max_chars = (max_w / cw) as usize;
    if s.len() <= max_chars {
        s
    } else if max_chars > 3 {
        &s[..max_chars - 3]
    } else {
        &s[..max_chars.min(s.len())]
    }
}
