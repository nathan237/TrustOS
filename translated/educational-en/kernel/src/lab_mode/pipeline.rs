//! Pipeline Viewer Panel — Visual kernel execution flow
//!
//! Shows the internal pipeline of what happens when a command or action
//! is performed: which kernel subsystems are activated, data flow between
//! components, and a real-time activity indicator for each stage.
//!
//! Pipeline stages:
//!   Input → Parser → Dispatcher → Kernel Subsystem → Hardware → Output
//!
//! Each stage lights up in real-time as events flow through it.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{draw_lab_text, draw_progress_bar, char_w, char_h,
            COLUMN_TEXT, COLUMN_DIM, COLUMN_ACCENT, COLUMN_GREEN, COLUMN_YELLOW, COLUMN_RED,
            COLUMN_PURPLE, COLUMN_CYAN, COLUMN_ORANGE};
use super::trace_bus::{self, EventCategory, read_since};

/// Pipeline stage identifiers
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Input,
    Parser,
    Scheduler,
    Memory,
    FileSystem,
    Interrupts,
    Output,
}

// Implementation block — defines methods for the type above.
impl Stage {
    fn label(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            Stage::Input       => "USER INPUT",
            Stage::Parser      => "SHELL / PARSER",
            Stage::Scheduler   => "SCHEDULER",
            Stage::Memory      => "MEMORY MGR",
            Stage::FileSystem  => "FILE SYSTEM",
            Stage::Interrupts  => "IRQ / HW",
            Stage::Output      => "DISPLAY OUT",
        }
    }
    
    fn color(&self) -> u32 {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            Stage::Input       => COLUMN_ACCENT,
            Stage::Parser      => COLUMN_PURPLE,
            Stage::Scheduler   => COLUMN_YELLOW,
            Stage::Memory      => COLUMN_GREEN,
            Stage::FileSystem  => COLUMN_CYAN,
            Stage::Interrupts  => COLUMN_ORANGE,
            Stage::Output      => 0xFF3FB950,
        }
    }
    
    fn icon(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            Stage::Input       => ">>",
            Stage::Parser      => "{}", 
            Stage::Scheduler   => "<>",
            Stage::Memory      => "[]",
            Stage::FileSystem  => "//",
            Stage::Interrupts  => "!!",
            Stage::Output      => "<-",
        }
    }
}

// Compile-time constant — evaluated at compilation, zero runtime cost.
const ALL_STAGES: [Stage; 7] = [
    Stage::Input, Stage::Parser, Stage::Scheduler,
    Stage::Memory, Stage::FileSystem, Stage::Interrupts, Stage::Output,
];

/// Activity level for a pipeline stage (fades over time)
struct StageActivity {
    /// Heat value (0-255): 255 = just fired, fades down
    heat: u16,
    /// Total hits since start
    hits: u64,
    /// Last event message
    last_message: String,
}

// Implementation block — defines methods for the type above.
impl StageActivity {
    fn new() -> Self {
        Self { heat: 0, hits: 0, last_message: String::new() }
    }
}

/// Pipeline viewer state
pub struct PipelineState {
    /// Activity per stage
    stages: [StageActivity; 7],
    /// Recent flow items (stage transitions)
    pub flows: Vec<FlowEntry>,
    /// Max flows to keep
    maximum_flows: usize,
    /// Last read index for trace bus
    last_read_index: u64,
    /// Frame counter
    frame: u64,
    /// Scroll in flow log
    pub scroll: usize,
}

/// A recorded flow through the pipeline
pub struct FlowEntry {
    timestamp_mouse: u64,
    from_stage: usize,
    to_stage: usize,
    label: String,
}

// Implementation block — defines methods for the type above.
impl PipelineState {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            stages: [
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(), StageActivity::new(),
                StageActivity::new(),
            ],
            flows: Vec::new(),
            maximum_flows: 50,
            last_read_index: 0,
            frame: 0,
            scroll: 0,
        }
    }
    
    /// Update state from trace bus events
    pub fn update(&mut self) {
        self.frame += 1;
        
        // Decay all heat values
        if self.frame % 3 == 0 {
            for s in &mut self.stages {
                s.heat = s.heat.saturating_sub(3);
            }
        }
        
        // Read new events from trace bus
        if self.frame % 5 != 0 { return; }
        
        let (events, new_index) = read_since(self.last_read_index, 50);
        if events.is_empty() {
            self.last_read_index = new_index;
            return;
        }
        
        for event in &events {
            // Map event category to pipeline stages
            let (from, to) = // Pattern matching — Rust's exhaustive branching construct.
match event.category {
                EventCategory::Keyboard => (0, 1),   // Input → Parser
                EventCategory::Syscall  => (1, 2),   // Parser → Scheduler
                EventCategory::Scheduler => (2, 3),  // Scheduler → Memory (context switch needs memory)
                EventCategory::Memory   => (3, 6),   // Memory → Output
                EventCategory::FileSystem => (1, 4), // Parser → FileSystem
                EventCategory::Interrupt => (5, 2),  // IRQ → Scheduler
                EventCategory::Network  => (4, 6),   // FS → Output
                EventCategory::Security => (1, 5),   // Parser → HW
                EventCategory::Custom   => (0, 6),   // Input → Output
                EventCategory::Hypervisor => (5, 6), // HW → Output (VM events)
            };
            
            // Light up both stages
            self.stages[from].heat = 255;
            self.stages[from].hits += 1;
            self.stages[to].heat = 200;
            self.stages[to].hits += 1;
            
            // Update last message on destination stage
            if event.message.len() < 40 {
                self.stages[to].last_message = event.message.clone();
            } else {
                self.stages[to].last_message = String::from(&event.message[..37]);
                self.stages[to].last_message.push_str("...");
            }
            
            // Record flow
            self.flows.push(FlowEntry {
                timestamp_mouse: event.timestamp_mouse,
                from_stage: from,
                to_stage: to,
                label: if event.message.len() > 25 {
                    let mut s = String::from(&event.message[..22]);
                    s.push_str("...");
                    s
                } else {
                    event.message.clone()
                },
            });
        }
        
        // Trim flow log
        if self.flows.len() > self.maximum_flows {
            let drain = self.flows.len() - self.maximum_flows;
            self.flows.drain(..drain);
        }
        
        self.last_read_index = new_index;
    }
    
        // Public function — callable from other modules.
pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_PGUP, KEY_PGDOWN};
                // Pattern matching — Rust's exhaustive branching construct.
match key {
            KEY_UP => self.scroll += 1,
            KEY_DOWN => { if self.scroll > 0 { self.scroll -= 1; } }
            KEY_PGUP => self.scroll += 5,
            KEY_PGDOWN => self.scroll = self.scroll.saturating_sub(5),
            b'c' | b'C' => {
                self.flows.clear();
                self.scroll = 0;
                for s in &mut self.stages {
                    s.hits = 0;
                    s.heat = 0;
                    s.last_message.clear();
                }
            }
            _ => {}
        }
    }

    /// Handle mouse click inside the pipeline panel content area
    pub fn handle_click(&mut self, local_x: i32, local_y: i32, w: u32, _h: u32) {
        let cw = char_w();
        let lh = char_h() + 1;
        if lh <= 0 || cw <= 0 { return; }

        // Top area: pipeline stage boxes (3 rows × lh each)
        let diagram_rows = 3;
        let stage_area_h = lh * diagram_rows + 4;
        if local_y < stage_area_h {
            // Click on a stage box → reset its counters
            let stage_w = (w as i32 / 4).maximum(12 * cw);
            let gap = 2i32;
            // Row 0: stages 0,1,2
            if local_y < lh + gap {
                let column = local_x / (stage_w + cw);
                if column < 3 {
                    let index = column as usize;
                    self.stages[index].heat = 255; // flash it
                }
            }
            // Row 1: stages 3,4,5
            else if local_y < 2 * (lh + gap) {
                let column = local_x / (stage_w + cw);
                if column < 3 {
                    let index = 3 + column as usize;
                    if index < 6 { self.stages[index].heat = 255; }
                }
            }
            return;
        }

        // Flow log area: clicking scrolls
        let stats_y = stage_area_h + 3 + lh + 2;
        if local_y >= stats_y {
            let row = ((local_y - stats_y) / lh) as usize;
            // Scroll to bring clicked row closer to view
            if row > 5 {
                self.scroll = self.scroll.saturating_sub(row - 5);
            }
        }
    }
}

/// Draw the pipeline viewer
pub fn draw(state: &PipelineState, x: i32, y: i32, w: u32, h: u32) {
    let cw = char_w();
    let lh = char_h() + 1;
    if lh <= 0 || cw <= 0 { return; }
    
    // ── Top section: Pipeline diagram (visual stages) ──────────
    // Draw stages as boxes connected by arrows
    let diagram_h = lh * 4; // 4 lines for the diagram
    let stage_w = (w as i32 / 4).maximum(12 * cw); // width per stage box
    let gap = 2i32;
    
    // Row 1: Input → Parser → Scheduler
    let row1_y = y;
    draw_stage_box(state, 0, x, row1_y, stage_w as u32, lh, cw);
    draw_arrow(x + stage_w - cw, row1_y + lh / 2, cw, COLUMN_DIM);
    draw_stage_box(state, 1, x + stage_w + cw, row1_y, stage_w as u32, lh, cw);
    draw_arrow(x + 2 * stage_w, row1_y + lh / 2, cw, COLUMN_DIM);
    draw_stage_box(state, 2, x + 2 * (stage_w + cw), row1_y, stage_w as u32, lh, cw);
    
    // Row 2: Memory ← FileSystem | IRQ/HW → Output
    let row2_y = y + lh + gap;
    draw_stage_box(state, 3, x, row2_y, stage_w as u32, lh, cw);
    draw_arrow(x + stage_w - cw, row2_y + lh / 2, cw, COLUMN_DIM);
    draw_stage_box(state, 4, x + stage_w + cw, row2_y, stage_w as u32, lh, cw);
    draw_arrow(x + 2 * stage_w, row2_y + lh / 2, cw, COLUMN_DIM);
    draw_stage_box(state, 5, x + 2 * (stage_w + cw), row2_y, stage_w as u32, lh, cw);
    
    // Row 3: Output (centered, wide)
    let row3_y = y + 2 * (lh + gap);
    let out_x = x + stage_w + cw;
    draw_stage_box(state, 6, out_x, row3_y, stage_w as u32, lh, cw);
    
    // ── Middle: per-stage hit counters ─────────────────────────
    let stats_y = row3_y + lh + gap + 2;
    crate::framebuffer::fill_rect(x as u32, stats_y as u32, w, 1, 0xFF30363D);
    let stats_line_y = stats_y + 3;
    
    let mut sx = x;
    for (i, stage) in ALL_STAGES.iter().enumerate() {
        let hits = state.stages[i].hits;
        let label = format!("{}:{}", stage.icon(), hits);
        let column = if state.stages[i].heat > 50 { stage.color() } else { COLUMN_DIM };
        draw_lab_text(sx, stats_line_y, &label, column);
        sx += (label.len() as i32 + 1) * cw;
        if sx > x + w as i32 - 10 { break; }
    }
    
    // ── Bottom: Flow log (recent stage transitions) ────────────
    let log_y = stats_line_y + lh + 2;
    crate::framebuffer::fill_rect(x as u32, (log_y - 1) as u32, w, 1, 0xFF30363D);
    
    draw_lab_text(x, log_y, "Pipeline Flow", COLUMN_ACCENT);
    let count_label = format!("{} events", state.flows.len());
    let clx = x + w as i32 - (count_label.len() as i32 * cw) - 2;
    draw_lab_text(clx, log_y, &count_label, COLUMN_DIM);
    
    let list_y = log_y + lh;
    let list_h = h as i32 - (list_y - y);
    if list_h <= 0 { return; }
    
    let visible = (list_h / lh) as usize;
    
    if state.flows.is_empty() {
        draw_lab_text(x + 4, list_y, "Waiting for events...", COLUMN_DIM);
        return;
    }
    
    // Show newest at bottom, scroll moves up
    let total = state.flows.len();
    let end = total.saturating_sub(state.scroll);
    let start = end.saturating_sub(visible);
    
    let mut cy = list_y;
    for i in start..end {
        let flow = &state.flows[i];
        let from = &ALL_STAGES[flow.from_stage];
        let to = &ALL_STAGES[flow.to_stage];
        
        // Timestamp
        let ts = format_ts(flow.timestamp_mouse);
        draw_lab_text(x, cy, &ts, COLUMN_DIM);
        
        // From → To
        let from_label = from.icon();
        let to_label = to.icon();
        let transmit = x + 10 * cw;
        draw_lab_text(transmit, cy, from_label, from.color());
        draw_lab_text(transmit + 3 * cw, cy, ">", COLUMN_DIM);
        draw_lab_text(transmit + 4 * cw, cy, to_label, to.color());
        
        // Description
        let descriptor_x = transmit + 8 * cw;
        let maximum_descriptor = ((w as i32 - (descriptor_x - x)) / cw) as usize;
        let desc = if flow.label.len() > maximum_descriptor && maximum_descriptor > 3 {
            &flow.label[..maximum_descriptor.saturating_sub(1)]
        } else {
            &flow.label
        };
        draw_lab_text(descriptor_x, cy, desc, COLUMN_TEXT);
        
        cy += lh;
        if cy > y + h as i32 { break; }
    }
}

/// Draw a single pipeline stage box
fn draw_stage_box(state: &PipelineState, index: usize, x: i32, y: i32, w: u32, h: i32, cw: i32) {
    let stage = &ALL_STAGES[index];
    let activity = &state.stages[index];
    
    // Background: brighter when active (heat)
    let bg = if activity.heat > 150 {
        blend_color(0xFF161B22, stage.color(), activity.heat as u32 / 4)
    } else if activity.heat > 50 {
        blend_color(0xFF161B22, stage.color(), activity.heat as u32 / 8)
    } else {
        0xFF161B22
    };
    crate::framebuffer::fill_rect(x as u32, y as u32, w, h as u32, bg);
    
    // Border: colored when active
    let border = if activity.heat > 100 { stage.color() } else { 0xFF30363D };
    // Top + bottom borders
    crate::framebuffer::fill_rect(x as u32, y as u32, w, 1, border);
    crate::framebuffer::fill_rect(x as u32, (y + h - 1) as u32, w, 1, border);
    // Left + right borders
    crate::framebuffer::fill_rect(x as u32, y as u32, 1, h as u32, border);
    crate::framebuffer::fill_rect((x + w as i32 - 1) as u32, y as u32, 1, h as u32, border);
    
    // Label
    let label = stage.label();
    let text_color = if activity.heat > 100 { stage.color() } else { COLUMN_DIM };
    // Center the label
    let label_x = x + 2;
    draw_lab_text(label_x, y + 2, label, text_color);
}

/// Draw an arrow "→" between boxes
fn draw_arrow(x: i32, y: i32, _cw: i32, color: u32) {
    draw_lab_text(x, y, ">", color);
}

/// Blend two colors by an amount (0-63)
fn blend_color(base: u32, accent: u32, amount: u32) -> u32 {
    let amount = amount.minimum(63);
    let inv = 63 - amount;
    let r = (((base >> 16) & 0xFF) * inv + ((accent >> 16) & 0xFF) * amount) / 63;
    let g = (((base >> 8) & 0xFF) * inv + ((accent >> 8) & 0xFF) * amount) / 63;
    let b = ((base & 0xFF) * inv + (accent & 0xFF) * amount) / 63;
    0xFF000000 | (r << 16) | (g << 8) | b
}

/// Format timestamp as MM:SS.mmm  
fn format_ts(mouse: u64) -> String {
    let s = mouse / 1000;
    let m = s / 60;
    let frac = mouse % 1000;
    format!("{:02}:{:02}.{:03}", m % 100, s % 60, frac)
}
