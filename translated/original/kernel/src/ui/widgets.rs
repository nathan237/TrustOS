//! Advanced UI Widgets
//!
//! Additional widgets for the TrustOS UI toolkit:
//! - Slider, Radio, Dropdown
//! - ScrollView, ListView
//! - Modal, Dialog, Tooltip
//! - Panel, Card, Divider

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::{
    Widget, WidgetState, Color, Theme, Rect, Point, Size, EdgeInsets,
    UiEvent, MouseEvent, MouseButton, KeyEvent,
    next_widget_id, draw_text,
};
use crate::drivers::virtio_gpu::GpuSurface;

// ═══════════════════════════════════════════════════════════════════════════════
// SLIDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Slider widget for selecting a value in a range
pub struct Slider {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub value: f32,        // 0.0 to 1.0
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub show_value: bool,
    pub on_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
    dragging: bool,
}

impl Slider {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            value: ((value - min) / (max - min)).clamp(0.0, 1.0),
            min,
            max,
            step: 0.0,
            show_value: true,
            on_change: None,
            dragging: false,
        }
    }
    
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }
    
    pub fn on_change<F: Fn(f32) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
    
    pub fn actual_value(&self) -> f32 {
        self.min + self.value * (self.max - self.min)
    }
    
    fn update_from_x(&mut self, x: i32) {
        let track_start = self.bounds.x + 8;
        let track_end = self.bounds.x + self.bounds.width as i32 - 8;
        let track_width = track_end - track_start;
        
        if track_width > 0 {
            let relative = (x - track_start) as f32 / track_width as f32;
            self.value = relative.clamp(0.0, 1.0);
            
            // Apply step if set
            if self.step > 0.0 {
                let range = self.max - self.min;
                let steps_f = self.value * range / self.step;
                let steps = (steps_f + 0.5) as i32 as f32; // Simple round
                self.value = (steps * self.step / range).clamp(0.0, 1.0);
            }
            
            if let Some(ref on_change) = self.on_change {
                on_change(self.actual_value());
            }
        }
    }
}

impl Widget for Slider {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(200, 24)
    }
    
    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Mouse(MouseEvent::Down { x, button: MouseButton::Left, .. }) => {
                self.dragging = true;
                self.update_from_x(*x);
                true
            }
            UiEvent::Mouse(MouseEvent::Move { x, .. }) if self.dragging => {
                self.update_from_x(*x);
                true
            }
            UiEvent::Mouse(MouseEvent::Up { button: MouseButton::Left, .. }) => {
                self.dragging = false;
                true
            }
            _ => false
        }
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.bounds.width;
        let h = self.bounds.height;
        
        // Track
        let track_y = y + h / 2 - 2;
        surface.fill_rounded_rect(x + 4, track_y, w - 8, 4, 2, theme.bg_tertiary.to_u32());
        
        // Filled portion
        let fill_width = ((w - 16) as f32 * self.value) as u32;
        if fill_width > 0 {
            surface.fill_rounded_rect(x + 4, track_y, fill_width + 4, 4, 2, theme.accent.to_u32());
        }
        
        // Thumb
        let thumb_x = x + 4 + fill_width;
        let thumb_y = y + h / 2 - 8;
        let thumb_color = if self.dragging { theme.accent } else if self.state.hovered { theme.accent_hover } else { theme.fg_primary };
        surface.fill_circle(thumb_x as i32 + 4, thumb_y as i32 + 8, 8, thumb_color.to_u32());
        
        // Value text
        if self.show_value {
            let val = self.actual_value();
            let text = if self.step >= 1.0 {
                format!("{}", val as i32)
            } else {
                format!("{:.1}", val)
            };
            let text_x = x as i32 + w as i32 + 8;
            let text_y = y as i32 + (h as i32 - 16) / 2;
            draw_text(surface, text_x, text_y, &text, theme.fg_secondary.to_u32());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RADIO BUTTON
// ═══════════════════════════════════════════════════════════════════════════════

/// Radio button for exclusive selection
pub struct RadioButton {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub selected: bool,
    pub label: String,
    pub group: u32,
    pub on_select: Option<Box<dyn Fn() + Send + Sync>>,
}

impl RadioButton {
    pub fn new(label: impl Into<String>, group: u32) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            selected: false,
            label: label.into(),
            group,
            on_select: None,
        }
    }
    
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }
    
    pub fn on_select<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
}

impl Widget for RadioButton {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(24 + self.label.len() as u32 * 8 + 8, 24)
    }
    
    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Mouse(MouseEvent::Click { button: MouseButton::Left, .. }) => {
                self.selected = true;
                if let Some(ref on_select) = self.on_select {
                    on_select();
                }
                true
            }
            _ => false
        }
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let circle_x = self.bounds.x as i32 + 10;
        let circle_y = self.bounds.y as i32 + self.bounds.height as i32 / 2;
        
        // Outer circle
        surface.draw_circle(circle_x, circle_y, 8, theme.border.to_u32());
        
        // Inner filled circle if selected
        if self.selected {
            surface.fill_circle(circle_x, circle_y, 5, theme.accent.to_u32());
        }
        
        // Label
        let text_x = self.bounds.x as i32 + 28;
        let text_y = self.bounds.y as i32 + (self.bounds.height as i32 - 16) / 2;
        draw_text(surface, text_x, text_y, &self.label, theme.fg_primary.to_u32());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DROPDOWN
// ═══════════════════════════════════════════════════════════════════════════════

/// Dropdown select widget
pub struct Dropdown {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub expanded: bool,
    pub on_change: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

impl Dropdown {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            options,
            selected_index: 0,
            expanded: false,
            on_change: None,
        }
    }
    
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.options.len().saturating_sub(1));
        self
    }
    
    pub fn on_change<F: Fn(usize) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
    
    pub fn selected_value(&self) -> Option<&String> {
        self.options.get(self.selected_index)
    }
}

impl Widget for Dropdown {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        let max_len = self.options.iter().map(|s| s.len()).max().unwrap_or(10);
        Size::new((max_len as u32 * 8 + 32).max(120), 32)
    }
    
    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Mouse(MouseEvent::Click { x, y, button: MouseButton::Left }) => {
                if self.expanded {
                    // Check if clicking on an option
                    let option_y = self.bounds.y + self.bounds.height as i32;
                    let idx = ((*y - option_y) / 28) as usize;
                    if idx < self.options.len() {
                        self.selected_index = idx;
                        if let Some(ref on_change) = self.on_change {
                            on_change(idx);
                        }
                    }
                    self.expanded = false;
                } else {
                    self.expanded = true;
                }
                true
            }
            UiEvent::Blur => {
                self.expanded = false;
                true
            }
            _ => false
        }
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.bounds.width;
        let h = self.bounds.height;
        
        // Main box
        let bg = if self.state.hovered { theme.button_hover } else { theme.button_bg };
        surface.fill_rounded_rect(x, y, w, h, theme.border_radius, bg.to_u32());
        surface.draw_rounded_rect(x, y, w, h, theme.border_radius, theme.border.to_u32());
        
        // Selected text
        if let Some(text) = self.selected_value() {
            let text_x = x as i32 + 12;
            let text_y = y as i32 + (h as i32 - 16) / 2;
            draw_text(surface, text_x, text_y, text, theme.fg_primary.to_u32());
        }
        
        // Arrow
        let arrow_x = x + w - 20;
        let arrow_y = y + h / 2;
        surface.draw_line(arrow_x as i32, arrow_y as i32 - 2, arrow_x as i32 + 4, arrow_y as i32 + 2, theme.fg_secondary.to_u32());
        surface.draw_line(arrow_x as i32 + 4, arrow_y as i32 + 2, arrow_x as i32 + 8, arrow_y as i32 - 2, theme.fg_secondary.to_u32());
        
        // Dropdown options
        if self.expanded {
            let dropdown_y = y + h + 2;
            let dropdown_h = (self.options.len() as u32 * 28).min(200);
            
            surface.fill_rounded_rect(x, dropdown_y, w, dropdown_h, theme.border_radius, theme.bg_secondary.to_u32());
            surface.draw_rounded_rect(x, dropdown_y, w, dropdown_h, theme.border_radius, theme.border.to_u32());
            
            for (i, option) in self.options.iter().enumerate() {
                let opt_y = dropdown_y + i as u32 * 28;
                
                if i == self.selected_index {
                    surface.fill_rect(x + 2, opt_y + 2, w - 4, 24, theme.accent.with_alpha(40).to_u32());
                }
                
                let text_x = x as i32 + 12;
                let text_y = opt_y as i32 + 6;
                let color = if i == self.selected_index { theme.accent } else { theme.fg_primary };
                draw_text(surface, text_x, text_y, option, color.to_u32());
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCROLL VIEW
// ═══════════════════════════════════════════════════════════════════════════════

/// Scrollable container
pub struct ScrollView {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub content_height: u32,
    pub scroll_y: i32,
    pub scroll_speed: i32,
}

impl ScrollView {
    pub fn new(content_height: u32) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            content_height,
            scroll_y: 0,
            scroll_speed: 20,
        }
    }
    
    pub fn scroll_to(&mut self, y: i32) {
        let max_scroll = (self.content_height as i32 - self.bounds.height as i32).max(0);
        self.scroll_y = y.clamp(0, max_scroll);
    }
    
    pub fn scroll_by(&mut self, delta: i32) {
        self.scroll_to(self.scroll_y + delta);
    }
    
    /// Get visible content area
    pub fn visible_rect(&self) -> Rect {
        Rect::new(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width.saturating_sub(12), // Account for scrollbar
            self.bounds.height,
        )
    }
}

impl Widget for ScrollView {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Mouse(MouseEvent::Scroll { delta, .. }) => {
                self.scroll_by(-delta * self.scroll_speed);
                true
            }
            _ => false
        }
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.bounds.width;
        let h = self.bounds.height;
        
        // Background
        surface.fill_rect(x, y, w, h, theme.bg_primary.to_u32());
        
        // Scrollbar track
        let sb_x = x + w - 8;
        surface.fill_rect(sb_x, y, 8, h, theme.bg_tertiary.to_u32());
        
        // Scrollbar thumb
        if self.content_height > h {
            let visible_ratio = h as f32 / self.content_height as f32;
            let thumb_height = ((h as f32 * visible_ratio) as u32).max(20);
            let scroll_ratio = self.scroll_y as f32 / (self.content_height - h) as f32;
            let thumb_y = y + ((h - thumb_height) as f32 * scroll_ratio) as u32;
            
            surface.fill_rounded_rect(sb_x, thumb_y, 8, thumb_height, 4, theme.fg_secondary.to_u32());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MODAL / DIALOG
// ═══════════════════════════════════════════════════════════════════════════════

/// Modal dialog overlay
pub struct Modal {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
}

impl Modal {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            title: title.into(),
            width,
            height,
            visible: false,
        }
    }
    
    pub fn show(&mut self) {
        self.visible = true;
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    /// Get content area inside modal
    pub fn content_rect(&self) -> Rect {
        Rect::new(
            self.bounds.x + 16,
            self.bounds.y + 48,
            self.width - 32,
            self.height - 64,
        )
    }
}

impl Widget for Modal {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { 
        // Center the modal
        let screen_w = bounds.width;
        let screen_h = bounds.height;
        self.bounds = Rect::new(
            (screen_w as i32 - self.width as i32) / 2,
            (screen_h as i32 - self.height as i32) / 2,
            self.width,
            self.height,
        );
    }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.visible { return; }
        
        let (sw, sh) = (surface.width, surface.height);
        
        // Dim background
        for y in 0..sh {
            for x in 0..sw {
                let existing = surface.get_pixel(x, y);
                let dimmed = ((existing >> 16 & 0xFF) / 2) << 16 
                           | ((existing >> 8 & 0xFF) / 2) << 8 
                           | (existing & 0xFF) / 2
                           | 0xC0000000;
                surface.set_pixel(x, y, dimmed);
            }
        }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.width;
        let h = self.height;
        
        // Modal background with shadow
        surface.fill_rounded_rect(x + 4, y + 4, w, h, 8, 0x40000000); // Shadow
        surface.fill_rounded_rect(x, y, w, h, 8, theme.bg_secondary.to_u32());
        surface.draw_rounded_rect(x, y, w, h, 8, theme.border.to_u32());
        
        // Title bar
        surface.fill_rect(x + 1, y + 1, w - 2, 40, theme.bg_tertiary.to_u32());
        
        // Title
        let title_x = x as i32 + 16;
        let title_y = y as i32 + 12;
        draw_text(surface, title_x, title_y, &self.title, theme.fg_primary.to_u32());
        
        // Close button (X)
        let close_x = x + w - 32;
        let close_y = y + 12;
        surface.fill_rounded_rect(close_x, close_y, 20, 20, 4, theme.error.with_alpha(40).to_u32());
        draw_text(surface, close_x as i32 + 6, close_y as i32 + 2, "×", theme.error.to_u32());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PANEL / CARD
// ═══════════════════════════════════════════════════════════════════════════════

/// Panel container with optional title
pub struct Panel {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub title: Option<String>,
    pub padding: EdgeInsets,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            title: None,
            padding: EdgeInsets::all(12),
        }
    }
    
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    pub fn with_padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }
    
    /// Get content area inside panel
    pub fn content_rect(&self) -> Rect {
        let title_offset = if self.title.is_some() { 32 } else { 0 };
        Rect::new(
            self.bounds.x + self.padding.left as i32,
            self.bounds.y + self.padding.top as i32 + title_offset,
            self.bounds.width - self.padding.left - self.padding.right,
            self.bounds.height - self.padding.top - self.padding.bottom - title_offset as u32,
        )
    }
}

impl Widget for Panel {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.bounds.width;
        let h = self.bounds.height;
        
        // Background
        surface.fill_rounded_rect(x, y, w, h, theme.border_radius, theme.bg_secondary.to_u32());
        surface.draw_rounded_rect(x, y, w, h, theme.border_radius, theme.border.to_u32());
        
        // Title
        if let Some(ref title) = self.title {
            let title_y = y + 8;
            draw_text(surface, x as i32 + 12, title_y as i32, title, theme.fg_secondary.to_u32());
            
            // Separator
            surface.draw_line(x as i32 + 8, (y + 28) as i32, (x + w - 8) as i32, (y + 28) as i32, theme.border.to_u32());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DIVIDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Horizontal or vertical divider
pub struct Divider {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub vertical: bool,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            vertical: false,
        }
    }
    
    pub fn vertical() -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            vertical: true,
        }
    }
}

impl Widget for Divider {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        if self.vertical {
            Size::new(1, 20)
        } else {
            Size::new(20, 1)
        }
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        
        if self.vertical {
            surface.fill_rect(x, y, 1, self.bounds.height, theme.border.to_u32());
        } else {
            surface.fill_rect(x, y, self.bounds.width, 1, theme.border.to_u32());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ICON BUTTON
// ═══════════════════════════════════════════════════════════════════════════════

/// Button with icon (single character or emoji-like symbol)
pub struct IconButton {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub icon: char,
    pub tooltip: Option<String>,
    pub on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl IconButton {
    pub fn new(icon: char) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            icon,
            tooltip: None,
            on_click: None,
        }
    }
    
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
    
    pub fn on_click<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl Widget for IconButton {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(36, 36)
    }
    
    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Mouse(MouseEvent::Enter) => {
                self.state.hovered = true;
                true
            }
            UiEvent::Mouse(MouseEvent::Leave) => {
                self.state.hovered = false;
                self.state.pressed = false;
                true
            }
            UiEvent::Mouse(MouseEvent::Down { button: MouseButton::Left, .. }) => {
                self.state.pressed = true;
                true
            }
            UiEvent::Mouse(MouseEvent::Up { button: MouseButton::Left, .. }) => {
                if self.state.pressed {
                    self.state.pressed = false;
                    if let Some(ref on_click) = self.on_click {
                        on_click();
                    }
                }
                true
            }
            _ => false
        }
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let size = self.bounds.width.min(self.bounds.height);
        
        let bg = if self.state.pressed {
            theme.button_pressed
        } else if self.state.hovered {
            theme.button_hover
        } else {
            Color::TRANSPARENT
        };
        
        if bg.a > 0 {
            surface.fill_rounded_rect(x, y, size, size, size / 2, bg.to_u32());
        }
        
        // Icon (centered)
        let icon_str = alloc::string::String::from(self.icon);
        let text_x = x as i32 + (size as i32 - 8) / 2;
        let text_y = y as i32 + (size as i32 - 16) / 2;
        draw_text(surface, text_x, text_y, &icon_str, theme.fg_primary.to_u32());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOOLTIP
// ═══════════════════════════════════════════════════════════════════════════════

/// Tooltip that appears on hover
pub struct Tooltip {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            x: 0,
            y: 0,
            visible: false,
        }
    }
    
    pub fn show_at(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        self.visible = true;
    }
    
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    pub fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.visible { return; }
        
        let padding = 8;
        let w = (self.text.len() as u32 * 8 + padding * 2) as u32;
        let h = 24;
        
        let x = self.x as u32;
        let y = self.y as u32;
        
        // Background
        surface.fill_rounded_rect(x, y, w, h, 4, theme.bg_tertiary.to_u32());
        surface.draw_rounded_rect(x, y, w, h, 4, theme.border.to_u32());
        
        // Text
        draw_text(surface, x as i32 + padding as i32, y as i32 + 4, &self.text, theme.fg_primary.to_u32());
    }
}
