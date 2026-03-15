//! Live Kernel Trace Panel — Real-time scrolling feed of kernel events
//!
//! Reads from the trace_bus event ring and displays a scrollable,
//! color-coded log of every kernel event:
//!   [00:12.345] IRQ   timer tick
//!   [00:12.346] SCHED context switch → task 3
//!   [00:12.347] VFS   open("/etc/config")
//!   [00:12.348] MEM   alloc 4096 bytes
//!
//! Two instances exist: panel 1 (Kernel Trace) and panel 5 (Event Stream).
//! The "live" variant auto-scrolls; the regular one allows manual scroll.

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use super::{draw_lab_text, char_w, char_h,
            COLUMN_TEXT, COLUMN_DIM, COLUMN_ACCENT, COLUMN_PANEL_BG};
use super::trace_bus::{LabEvent, EventCategory, read_recent, total_count, read_since};

/// Kernel trace panel state  
pub struct KernelTraceState {
    /// Cached events snapshot
    pub events: Vec<LabEvent>,
    /// Scroll offset (0 = bottom/newest)
    pub scroll: usize,
    /// Auto-scroll mode
    pub auto_scroll: bool,
    /// Last read index for incremental updates
    pub last_read_index: u64,
    /// Filter: which categories to show (all true by default)
    pub filters: [bool; 9],
    /// Whether in "live" mode (auto-scroll, shows latest)
    pub is_live: bool,
    /// Refresh rate divider
    refresh_counter: u64,
    /// Paused
    pub paused: bool,
    /// Selected event index (for detail view via click)
    pub selected_event: Option<usize>,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl KernelTraceState {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        Self {
            events: Vec::new(),
            scroll: 0,
            auto_scroll: false,
            last_read_index: 0,
            filters: [true; 9],
            is_live: false,
            refresh_counter: 0,
            paused: false,
            selected_event: None,
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn new_live() -> Self {
        Self {
            events: Vec::new(),
            scroll: 0,
            auto_scroll: true,
            last_read_index: 0,
            filters: [true; 9],
            is_live: true,
            refresh_counter: 0,
            paused: false,
            selected_event: None,
        }
    }
    
    /// Update cached events
    pub fn update(&mut self) {
        self.refresh_counter += 1;
        if self.refresh_counter % 10 != 0 || self.paused {
            return;
        }
        
        // Incremental read
        let (new_events, new_index) = read_since(self.last_read_index, 100);
        if !new_events.is_empty() {
            self.events.extend(new_events);
            self.last_read_index = new_index;
            
            // Keep buffer bounded
            if self.events.len() > 500 {
                let drain = self.events.len() - 500;
                self.events.drain(..drain);
            }
            
            // Auto-scroll to bottom
            if self.auto_scroll {
                self.scroll = 0;
            }
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_PGUP, KEY_PGDOWN};
                // Correspondance de motifs — branchement exhaustif de Rust.
match key {
            KEY_UP => {
                self.scroll += 1;
                self.auto_scroll = false;
            }
            KEY_DOWN => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                if self.scroll == 0 {
                    self.auto_scroll = true;
                }
            }
            KEY_PGUP => {
                self.scroll += 10;
                self.auto_scroll = false;
            }
            KEY_PGDOWN => {
                self.scroll = self.scroll.saturating_sub(10);
                if self.scroll == 0 {
                    self.auto_scroll = true;
                }
            }
            // 'p' = toggle pause
            b'p' | b'P' => {
                self.paused = !self.paused;
            }
            // 'c' = clear
            b'c' | b'C' => {
                self.events.clear();
                self.scroll = 0;
            }
            // Number keys 1-9 = toggle category filters
            b'1'..=b'9' => {
                let index = (key - b'1') as usize;
                if index < self.filters.len() {
                    self.filters[index] = !self.filters[index];
                }
            }
            _ => {}
        }
    }

    /// Handle mouse click inside the trace panel content area
    pub fn handle_click(&mut self, local_x: i32, local_y: i32, w: u32, _h: u32) {
        let cw = char_w();
        let lh = char_h() + 1;
        if lh <= 0 || cw <= 0 { return; }

        let status_h = lh;
        let filter_y_start = status_h;
        let log_y_start = filter_y_start + lh + 2;

        // Click on filter bar → toggle category
        if local_y >= filter_y_start && local_y < filter_y_start + lh {
            let cats = [
                EventCategory::Interrupt,
                EventCategory::Scheduler,
                EventCategory::Memory,
                EventCategory::FileSystem,
                EventCategory::Syscall,
                EventCategory::Keyboard,
            ];
            let mut fx = 0i32;
            for (i, cat) in cats.iter().enumerate() {
                let label_length = cat.label().len() as i32 + 1;
                let label_end = fx + label_length * cw;
                if local_x >= fx && local_x < label_end {
                    let index = *cat as usize;
                    if index < self.filters.len() {
                        self.filters[index] = !self.filters[index];
                    }
                    return;
                }
                fx = label_end;
                if fx > w as i32 - 20 { break; }
            }
            return;
        }

        // Click on event log area → select event for detail view
        if local_y >= log_y_start {
            let row = ((local_y - log_y_start) / lh) as usize;
            // Map row to filtered event index
            let filtered: Vec<usize> = self.events.iter().enumerate()
                .filter(|(_, e)| self.filters[e.category as usize])
                .map(|(i, _)| i)
                .collect();
            let total_filtered = filtered.len();
            let end = total_filtered.saturating_sub(self.scroll);
            let visible_lines = 20usize; // approximate
            let start = end.saturating_sub(visible_lines);
            let clicked_index = start + row;
            if clicked_index < end {
                let event_index = filtered[clicked_index];
                self.selected_event = if self.selected_event == Some(event_index) {
                    None // deselect on second click
                } else {
                    Some(event_index)
                };
            }
            return;
        }
    }
}

/// Draw the kernel trace panel
pub fn draw(state: &KernelTraceState, x: i32, y: i32, w: u32, h: u32) {
    let cw = char_w();
    let lh = char_h() + 1;
    
    if lh <= 0 || cw <= 0 { return; }
    
    // Status line at top
    let status_h = lh;
    let total = total_count();
    let status = if state.paused {
        format!("PAUSED | {} events", total)
    } else if state.is_live {
        format!("LIVE | {} events", total)
    } else {
        format!("{} events | scroll: {}", total, state.scroll)
    };
    draw_lab_text(x, y, &status, if state.paused { super::COLUMN_YELLOW } else { COLUMN_DIM });
    
    // Filter bar
    let filter_y = y + status_h;
    let cats = [
        EventCategory::Interrupt,
        EventCategory::Scheduler,
        EventCategory::Memory,
        EventCategory::FileSystem,
        EventCategory::Syscall,
        EventCategory::Keyboard,
    ];
    let mut fx = x;
    for (i, cat) in cats.iter().enumerate() {
        let enabled = state.filters[*cat as usize];
        let color = if enabled { cat.color() } else { 0xFF30363D };
        let label = cat.label();
        draw_lab_text(fx, filter_y, label, color);
        fx += (label.len() as i32 + 1) * cw;
        if fx > x + w as i32 - 20 { break; }
    }
    
    // Event log area
    let log_y = filter_y + lh + 2;
    let log_h = h as i32 - (log_y - y);
    if log_h <= 0 { return; }
    
    let visible_lines = (log_h / lh) as usize;
    
    // Filter events by active categories
    let filtered: Vec<&LabEvent> = state.events.iter()
        .filter(|e| state.filters[e.category as usize])
        .collect();
    
    if filtered.is_empty() {
        draw_lab_text(x + 4, log_y + lh, "Waiting for events...", COLUMN_DIM);
        return;
    }
    
    // Calculate visible range (scroll from bottom)
    let total_filtered = filtered.len();
    let end = total_filtered.saturating_sub(state.scroll);

    // Reserve space for detail view if an event is selected
    let detail_lines = if state.selected_event.is_some() { 4 } else { 0 };
    let main_visible = visible_lines.saturating_sub(detail_lines);
    let start = end.saturating_sub(main_visible);
    
    // Build index mapping for filtered events → original events
    let filtered_indices: Vec<usize> = state.events.iter().enumerate()
        .filter(|(_, e)| state.filters[e.category as usize])
        .map(|(i, _)| i)
        .collect();
    
    let mut cy = log_y;
    for i in start..end {
        let event = filtered[i];
        let orig_index = if i < filtered_indices.len() { filtered_indices[i] } else { i };
        let is_selected = state.selected_event == Some(orig_index);
        
        // Highlight selected row
        if is_selected {
            crate::framebuffer::fill_rect(x as u32, cy as u32, w, lh as u32, 0xFF1F2937);
        }
        
        // Timestamp [MM:SS.mmm]
        let secs = event.timestamp_mouse / 1000;
        let mouse = event.timestamp_mouse % 1000;
        let ts = format!("{:02}:{:02}.{:03}", secs / 60, secs % 60, mouse);
        draw_lab_text(x, cy, &ts, COLUMN_DIM);
        
        // Category badge
        let cat_x = x + (ts.len() as i32 + 1) * cw;
        let cat_label = event.category.label();
        draw_lab_text(cat_x, cy, cat_label, event.category.color());
        
        // Syscall number badge (if present)
        let message_x_base = cat_x + (6 * cw);
        let message_x;
        if let Some(nr) = event.syscall_number {
            let number_label = format!("#{}", nr);
            draw_lab_text(message_x_base, cy, &number_label, super::COLUMN_PURPLE);
            message_x = message_x_base + ((number_label.len() as i32 + 1) * cw);
        } else {
            message_x = message_x_base;
        }
        
        // Message
        let maximum_message_w = w as i32 - (message_x - x);
        let maximum_chars = if cw > 0 { (maximum_message_w / cw) as usize } else { 20 };
        let message = if event.message.len() > maximum_chars && maximum_chars > 3 {
            &event.message[..maximum_chars.saturating_sub(3)]
        } else {
            &event.message
        };
        draw_lab_text(message_x, cy, message, if is_selected { COLUMN_ACCENT } else { COLUMN_TEXT });
        
        cy += lh;
        if cy > y + h as i32 { break; }
    }
    
    // Detail view for selected event (drawn at bottom of panel)
    if let Some(sel_index) = state.selected_event {
        if sel_index < state.events.len() {
            let event = &state.events[sel_index];
            let detail_y = y + h as i32 - (detail_lines as i32 * lh);
            // Separator line
            crate::framebuffer::fill_rect(x as u32, (detail_y - 2) as u32, w, 1, super::COLUMN_ACCENT);
            
            let mut dy = detail_y;
            // Line 1: Category + full message
            let header = format!("[{}] {}", event.category.label(), event.message);
            draw_lab_text(x + 2, dy, &header, event.category.color());
            dy += lh;
            
            // Line 2: Syscall details (if available)
            if let Some(nr) = event.syscall_number {
                let name = super::trace_bus::syscall_name(nr);
                let detail = if let Some(args) = event.syscall_args {
                    format!("Syscall #{} ({}) args=[{:#x}, {:#x}, {:#x}]",
                        nr, name, args[0], args[1], args[2])
                } else {
                    format!("Syscall #{} ({})", nr, name)
                };
                draw_lab_text(x + 2, dy, &detail, super::COLUMN_PURPLE);
                dy += lh;
                
                // Line 3: Return value
                if let Some(return_value) = event.syscall_return_value {
                    let return_value_str = if return_value < 0 {
                        format!("Return: {} (error)", return_value)
                    } else {
                        format!("Return: {} ({:#x})", return_value, return_value)
                    };
                    draw_lab_text(x + 2, dy, &return_value_str, if return_value < 0 { super::COLUMN_RED } else { super::COLUMN_GREEN });
                }
            } else {
                // Non-syscall: show payload
                let payload_str = format!("Payload: {} ({:#x}) | Timestamp: {}ms",
                    event.payload, event.payload, event.timestamp_mouse);
                draw_lab_text(x + 2, dy, &payload_str, COLUMN_DIM);
            }
        }
    }
    
    // Scroll indicator on right edge
    if total_filtered > visible_lines {
        let track_y = log_y;
        let track_h = log_h.maximum(1);
        let thumb_h = ((visible_lines as i32 * track_h) / total_filtered as i32).maximum(8);
        let thumb_position = if total_filtered > visible_lines {
            let scroll_range = total_filtered - visible_lines;
            let position = scroll_range.saturating_sub(state.scroll);
            (position as i32 * (track_h - thumb_h)) / scroll_range.maximum(1) as i32
        } else { 0 };
        
        let sb_x = (x + w as i32 - 3) as u32;
        // Track
        crate::framebuffer::fill_rect(sb_x, track_y as u32, 2, track_h as u32, 0xFF21262D);
        // Thumb
        crate::framebuffer::fill_rect(sb_x, (track_y + thumb_position) as u32, 2, thumb_h as u32, COLUMN_ACCENT);
    }
}
