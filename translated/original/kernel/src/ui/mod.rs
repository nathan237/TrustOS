//! TrustOS UI Toolkit
//!
//! A modern widget-based UI framework for TrustOS.
//! Inspired by Qt, GTK, and Flutter.
//!
//! Features:
//! - Widget hierarchy (parent-child relationships)
//! - Layout managers (Flex, Grid, Stack)
//! - Theming and styling
//! - Event handling (mouse, keyboard, focus)
//! - Animations (property transitions)
//! - GPU-accelerated rendering via virtio-gpu
//!
//! Design System: Matrix-inspired professional dark theme
//! - No pure white (#FFFFFF)
//! - Green accent hierarchy
//! - Minimal animations

pub mod widgets;

pub use widgets::*;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::any::Any;
use spin::Mutex;

use crate::drivers::virtio_gpu::{GpuSurface, Compositor};

// ═══════════════════════════════════════════════════════════════════════════════
// THEME SYSTEM
// ═══════════════════════════════════════════════════════════════════════════════

/// Color with RGBA components
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    pub const fn from_u32(color: u32) -> Self {
        Self {
            a: ((color >> 24) & 0xFF) as u8,
            r: ((color >> 16) & 0xFF) as u8,
            g: ((color >> 8) & 0xFF) as u8,
            b: (color & 0xFF) as u8,
        }
    }
    
    pub const fn to_u32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
    
    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }
    
    pub fn lighten(self, amount: u8) -> Self {
        Self {
            r: self.r.saturating_add(amount),
            g: self.g.saturating_add(amount),
            b: self.b.saturating_add(amount),
            a: self.a,
        }
    }
    
    pub fn darken(self, amount: u8) -> Self {
        Self {
            r: self.r.saturating_sub(amount),
            g: self.g.saturating_sub(amount),
            b: self.b.saturating_sub(amount),
            a: self.a,
        }
    }
    
    // Predefined colors
    pub const TRANSPARENT: Color = Color::new(0, 0, 0, 0);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);
    pub const YELLOW: Color = Color::rgb(255, 255, 0);
    pub const CYAN: Color = Color::rgb(0, 255, 255);
    pub const MAGENTA: Color = Color::rgb(255, 0, 255);
}

/// Theme for UI elements
#[derive(Clone)]
pub struct Theme {
    // Background colors
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,
    
    // Foreground (text) colors
    pub fg_primary: Color,
    pub fg_secondary: Color,
    pub fg_disabled: Color,
    
    // Accent colors
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    
    // Button colors
    pub button_bg: Color,
    pub button_hover: Color,
    pub button_pressed: Color,
    pub button_disabled: Color,
    
    // Border colors
    pub border: Color,
    pub border_focus: Color,
    
    // Semantic colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // Dimensions
    pub border_radius: u32,
    pub border_width: u32,
    pub padding: u32,
    pub spacing: u32,
    
    // Fonts
    pub font_size: u32,
    pub font_size_small: u32,
    pub font_size_large: u32,
}

impl Theme {
    /// TrustOS dark theme (default)
    pub fn dark() -> Self {
        Self {
            bg_primary: Color::from_u32(0xFF0A0E0B),
            bg_secondary: Color::from_u32(0xFF141A17),
            bg_tertiary: Color::from_u32(0xFF1E2620),
            
            fg_primary: Color::from_u32(0xFF00FF66),
            fg_secondary: Color::from_u32(0xFF00CC55),
            fg_disabled: Color::from_u32(0xFF4A5A4E),
            
            accent: Color::from_u32(0xFF00FF66),
            accent_hover: Color::from_u32(0xFF00CC55),
            accent_pressed: Color::from_u32(0xFF00AA44),
            
            button_bg: Color::from_u32(0xFF1E2620),
            button_hover: Color::from_u32(0xFF2A3630),
            button_pressed: Color::from_u32(0xFF354540),
            button_disabled: Color::from_u32(0xFF1A1A1A),
            
            border: Color::from_u32(0xFF2A3A2F),
            border_focus: Color::from_u32(0xFF00FF66),
            
            success: Color::from_u32(0xFF00FF66),
            warning: Color::from_u32(0xFFFFD166),
            error: Color::from_u32(0xFFFF6B6B),
            info: Color::from_u32(0xFF4ECDC4),
            
            border_radius: 6,
            border_width: 1,
            padding: 12,
            spacing: 8,
            
            font_size: 14,
            font_size_small: 12,
            font_size_large: 18,
        }
    }
    
    /// Light theme
    pub fn light() -> Self {
        Self {
            bg_primary: Color::from_u32(0xFFF5F5F5),
            bg_secondary: Color::from_u32(0xFFFFFFFF),
            bg_tertiary: Color::from_u32(0xFFE8E8E8),
            
            fg_primary: Color::from_u32(0xFF1A1A1A),
            fg_secondary: Color::from_u32(0xFF4A4A4A),
            fg_disabled: Color::from_u32(0xFFAAAAAA),
            
            accent: Color::from_u32(0xFF0066FF),
            accent_hover: Color::from_u32(0xFF0055DD),
            accent_pressed: Color::from_u32(0xFF0044BB),
            
            button_bg: Color::from_u32(0xFFE8E8E8),
            button_hover: Color::from_u32(0xFFDDDDDD),
            button_pressed: Color::from_u32(0xFFCCCCCC),
            button_disabled: Color::from_u32(0xFFF0F0F0),
            
            border: Color::from_u32(0xFFCCCCCC),
            border_focus: Color::from_u32(0xFF0066FF),
            
            success: Color::from_u32(0xFF22BB44),
            warning: Color::from_u32(0xFFFFAA00),
            error: Color::from_u32(0xFFDD3333),
            info: Color::from_u32(0xFF2299DD),
            
            border_radius: 6,
            border_width: 1,
            padding: 12,
            spacing: 8,
            
            font_size: 14,
            font_size_small: 12,
            font_size_large: 18,
        }
    }
}

// Global theme
static CURRENT_THEME: Mutex<Option<Theme>> = Mutex::new(None);

pub fn set_theme(theme: Theme) {
    *CURRENT_THEME.lock() = Some(theme);
}

pub fn get_theme() -> Theme {
    CURRENT_THEME.lock().clone().unwrap_or_else(Theme::dark)
}

// ═══════════════════════════════════════════════════════════════════════════════
// GEOMETRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Point in 2D space
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub const ZERO: Point = Point::new(0, 0);
}

/// Size (width, height)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    
    pub const ZERO: Size = Size::new(0, 0);
}

/// Rectangle
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p1.x - p2.x).unsigned_abs();
        let height = (p1.y - p2.y).unsigned_abs();
        Self { x, y, width, height }
    }
    
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x < self.x + self.width as i32 &&
        point.y >= self.y && point.y < self.y + self.height as i32
    }
    
    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.x + self.width as i32 <= other.x ||
          other.x + other.width as i32 <= self.x ||
          self.y + self.height as i32 <= other.y ||
          other.y + other.height as i32 <= self.y)
    }
    
    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }
    
    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }
    
    pub const ZERO: Rect = Rect::new(0, 0, 0, 0);
}

/// Edge insets (margins/padding)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeInsets {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl EdgeInsets {
    pub const fn all(value: u32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }
    
    pub const fn symmetric(vertical: u32, horizontal: u32) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }
    
    pub const fn only(top: u32, right: u32, bottom: u32, left: u32) -> Self {
        Self { top, right, bottom, left }
    }
    
    pub const ZERO: EdgeInsets = EdgeInsets::all(0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Mouse button
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse event
#[derive(Clone, Debug)]
pub enum MouseEvent {
    Move { x: i32, y: i32 },
    Down { x: i32, y: i32, button: MouseButton },
    Up { x: i32, y: i32, button: MouseButton },
    Click { x: i32, y: i32, button: MouseButton },
    DoubleClick { x: i32, y: i32, button: MouseButton },
    Scroll { x: i32, y: i32, delta: i32 },
    Enter,
    Leave,
}

/// Keyboard event
#[derive(Clone, Debug)]
pub enum KeyEvent {
    Down { key: char, modifiers: u8 },
    Up { key: char, modifiers: u8 },
    Char { c: char },
}

/// Modifier keys
pub mod modifiers {
    pub const SHIFT: u8 = 0x01;
    pub const CTRL: u8 = 0x02;
    pub const ALT: u8 = 0x04;
    pub const META: u8 = 0x08;
}

/// UI Event
#[derive(Clone, Debug)]
pub enum UiEvent {
    Mouse(MouseEvent),
    Key(KeyEvent),
    Focus,
    Blur,
    Resize(Size),
}

// ═══════════════════════════════════════════════════════════════════════════════
// WIDGET SYSTEM
// ═══════════════════════════════════════════════════════════════════════════════

/// Widget state
#[derive(Clone, Copy, Debug, Default)]
pub struct WidgetState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
    pub visible: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self {
            visible: true,
            ..Default::default()
        }
    }
}

/// Widget trait - base for all UI elements
pub trait Widget {
    /// Get widget ID
    fn id(&self) -> u32;
    
    /// Get widget bounds
    fn bounds(&self) -> Rect;
    
    /// Set widget bounds
    fn set_bounds(&mut self, bounds: Rect);
    
    /// Get preferred size
    fn preferred_size(&self) -> Size {
        Size::new(100, 30)
    }
    
    /// Get minimum size
    fn min_size(&self) -> Size {
        Size::new(0, 0)
    }
    
    /// Get maximum size
    fn max_size(&self) -> Size {
        Size::new(u32::MAX, u32::MAX)
    }
    
    /// Get state
    fn state(&self) -> WidgetState;
    
    /// Set state
    fn set_state(&mut self, state: WidgetState);
    
    /// Handle event
    fn handle_event(&mut self, event: &UiEvent) -> bool {
        false // Not handled
    }
    
    /// Render to surface
    fn render(&self, surface: &mut GpuSurface, theme: &Theme);
    
    /// Check if point is inside widget
    fn hit_test(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }
}

/// Widget ID counter
static NEXT_WIDGET_ID: Mutex<u32> = Mutex::new(1);

pub fn next_widget_id() -> u32 {
    let mut id = NEXT_WIDGET_ID.lock();
    let result = *id;
    *id += 1;
    result
}

// ═══════════════════════════════════════════════════════════════════════════════
// BASIC WIDGETS
// ═══════════════════════════════════════════════════════════════════════════════

/// Label widget (static text)
pub struct Label {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub text: String,
    pub color: Option<Color>,
    pub align: TextAlign,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            text: text.into(),
            color: None,
            align: TextAlign::Left,
        }
    }
    
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    
    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

impl Widget for Label {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        // 8 pixels per character (using 8x16 font)
        Size::new((self.text.len() as u32 * 8).max(10), 16)
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let color = self.color.unwrap_or(theme.fg_primary);
        
        // Draw text (simplified - would need font rendering)
        let x = match self.align {
            TextAlign::Left => self.bounds.x,
            TextAlign::Center => self.bounds.x + (self.bounds.width as i32 - self.text.len() as i32 * 8) / 2,
            TextAlign::Right => self.bounds.x + self.bounds.width as i32 - self.text.len() as i32 * 8,
        };
        
        draw_text(surface, x, self.bounds.y, &self.text, color.to_u32());
    }
}

/// Button widget
pub struct Button {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub text: String,
    pub on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            text: text.into(),
            on_click: None,
        }
    }
    
    pub fn on_click<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl Widget for Button {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new((self.text.len() as u32 * 8 + 24).max(80), 32)
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
        
        let bg = if self.state.disabled {
            theme.button_disabled
        } else if self.state.pressed {
            theme.button_pressed
        } else if self.state.hovered {
            theme.button_hover
        } else {
            theme.button_bg
        };
        
        let fg = if self.state.disabled {
            theme.fg_disabled
        } else {
            theme.fg_primary
        };
        
        // Draw button background
        surface.fill_rounded_rect(
            self.bounds.x as u32,
            self.bounds.y as u32,
            self.bounds.width,
            self.bounds.height,
            theme.border_radius,
            bg.to_u32()
        );
        
        // Draw border
        surface.draw_rounded_rect(
            self.bounds.x as u32,
            self.bounds.y as u32,
            self.bounds.width,
            self.bounds.height,
            theme.border_radius,
            if self.state.focused { theme.border_focus.to_u32() } else { theme.border.to_u32() }
        );
        
        // Draw text (centered)
        let text_x = self.bounds.x + (self.bounds.width as i32 - self.text.len() as i32 * 8) / 2;
        let text_y = self.bounds.y + (self.bounds.height as i32 - 16) / 2;
        draw_text(surface, text_x, text_y, &self.text, fg.to_u32());
    }
}

/// Text input widget
pub struct TextInput {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub text: String,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub max_length: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            text: String::new(),
            placeholder: String::new(),
            cursor_pos: 0,
            max_length: 256,
        }
    }
    
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
}

impl Widget for TextInput {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(200, 32)
    }
    
    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Key(KeyEvent::Char { c }) if self.state.focused => {
                if self.text.len() < self.max_length && !c.is_control() {
                    self.text.insert(self.cursor_pos, *c);
                    self.cursor_pos += 1;
                }
                true
            }
            UiEvent::Key(KeyEvent::Down { key, .. }) if self.state.focused => {
                match *key {
                    '\x08' => { // Backspace
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            self.text.remove(self.cursor_pos);
                        }
                        true
                    }
                    _ => false
                }
            }
            UiEvent::Mouse(MouseEvent::Click { .. }) => {
                self.state.focused = true;
                true
            }
            UiEvent::Blur => {
                self.state.focused = false;
                true
            }
            _ => false
        }
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        // Background
        surface.fill_rounded_rect(
            self.bounds.x as u32,
            self.bounds.y as u32,
            self.bounds.width,
            self.bounds.height,
            theme.border_radius,
            theme.bg_secondary.to_u32()
        );
        
        // Border
        let border_color = if self.state.focused {
            theme.border_focus
        } else if self.state.hovered {
            theme.accent_hover
        } else {
            theme.border
        };
        
        surface.draw_rounded_rect(
            self.bounds.x as u32,
            self.bounds.y as u32,
            self.bounds.width,
            self.bounds.height,
            theme.border_radius,
            border_color.to_u32()
        );
        
        // Text or placeholder
        let text_y = self.bounds.y + (self.bounds.height as i32 - 16) / 2;
        let text_x = self.bounds.x + theme.padding as i32;
        
        if self.text.is_empty() {
            draw_text(surface, text_x, text_y, &self.placeholder, theme.fg_disabled.to_u32());
        } else {
            draw_text(surface, text_x, text_y, &self.text, theme.fg_primary.to_u32());
        }
        
        // Cursor
        if self.state.focused {
            let cursor_x = text_x + (self.cursor_pos as i32 * 8);
            surface.fill_rect(
                cursor_x as u32,
                (text_y + 2) as u32,
                2,
                12,
                theme.accent.to_u32()
            );
        }
    }
}

/// Checkbox widget
pub struct Checkbox {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub checked: bool,
    pub label: String,
    pub on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl Checkbox {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            checked: false,
            label: label.into(),
            on_change: None,
        }
    }
    
    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
        self
    }
    
    pub fn on_change<F: Fn(bool) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl Widget for Checkbox {
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
                self.checked = !self.checked;
                if let Some(ref on_change) = self.on_change {
                    on_change(self.checked);
                }
                true
            }
            _ => false
        }
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        // Checkbox box
        let box_size = 20u32;
        let box_x = self.bounds.x as u32;
        let box_y = self.bounds.y as u32 + (self.bounds.height - box_size) / 2;
        
        let bg = if self.checked { theme.accent } else { theme.bg_secondary };
        
        surface.fill_rounded_rect(box_x, box_y, box_size, box_size, 4, bg.to_u32());
        surface.draw_rounded_rect(box_x, box_y, box_size, box_size, 4, theme.border.to_u32());
        
        // Checkmark
        if self.checked {
            let cx = box_x as i32 + 5;
            let cy = box_y as i32 + 10;
            surface.draw_line(cx, cy, cx + 4, cy + 4, theme.bg_primary.to_u32());
            surface.draw_line(cx + 4, cy + 4, cx + 12, cy - 4, theme.bg_primary.to_u32());
        }
        
        // Label
        let text_x = box_x as i32 + box_size as i32 + 8;
        let text_y = self.bounds.y + (self.bounds.height as i32 - 16) / 2;
        draw_text(surface, text_x, text_y, &self.label, theme.fg_primary.to_u32());
    }
}

/// Progress bar widget
pub struct ProgressBar {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub value: f32,  // 0.0 to 1.0
    pub show_text: bool,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            value: value.clamp(0.0, 1.0),
            show_text: true,
        }
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

impl Widget for ProgressBar {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(200, 20)
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.bounds.width;
        let h = self.bounds.height;
        
        // Background
        surface.fill_rounded_rect(x, y, w, h, h / 2, theme.bg_tertiary.to_u32());
        
        // Progress fill
        let fill_width = ((w as f32 * self.value) as u32).max(h);
        if self.value > 0.0 {
            surface.fill_rounded_rect(x, y, fill_width, h, h / 2, theme.accent.to_u32());
        }
        
        // Text
        if self.show_text {
            let percent = (self.value * 100.0) as u32;
            let text = format!("{}%", percent);
            let text_x = x as i32 + (w as i32 - text.len() as i32 * 8) / 2;
            let text_y = y as i32 + (h as i32 - 16) / 2;
            draw_text(surface, text_x, text_y, &text, theme.fg_primary.to_u32());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEXT RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Simple text drawing (uses 8x16 font from framebuffer)
pub fn draw_text(surface: &mut GpuSurface, x: i32, y: i32, text: &str, color: u32) {
    let mut cx = x;
    for c in text.chars() {
        if cx >= 0 && (cx as u32) < surface.width && y >= 0 && (y as u32) < surface.height {
            draw_char(surface, cx, y, c, color);
        }
        cx += 8;
    }
}

/// Draw a single character (8x16 bitmap font)
fn draw_char(surface: &mut GpuSurface, x: i32, y: i32, c: char, color: u32) {
    // Get glyph from framebuffer font
    let glyph = crate::framebuffer::font::get_glyph(c);
    
    for row in 0..16 {
        let bits = glyph[row];
        for col in 0..8 {
            if (bits >> (7 - col)) & 1 == 1 {
                let px = x + col as i32;
                let py = y + row as i32;
                if px >= 0 && py >= 0 {
                    surface.set_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LAYOUT SYSTEM
// ═══════════════════════════════════════════════════════════════════════════════

/// Flex direction
#[derive(Clone, Copy, Debug, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

/// Flex alignment
#[derive(Clone, Copy, Debug, Default)]
pub enum FlexAlign {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Flex container for layout
pub struct FlexLayout {
    pub direction: FlexDirection,
    pub align: FlexAlign,
    pub cross_align: FlexAlign,
    pub gap: u32,
    pub padding: EdgeInsets,
}

impl FlexLayout {
    pub fn row() -> Self {
        Self {
            direction: FlexDirection::Row,
            align: FlexAlign::Start,
            cross_align: FlexAlign::Center,
            gap: 8,
            padding: EdgeInsets::all(8),
        }
    }
    
    pub fn column() -> Self {
        Self {
            direction: FlexDirection::Column,
            align: FlexAlign::Start,
            cross_align: FlexAlign::Start,
            gap: 8,
            padding: EdgeInsets::all(8),
        }
    }
    
    /// Layout children within container bounds
    pub fn layout(&self, container: Rect, children: &mut [&mut dyn Widget]) {
        if children.is_empty() { return; }
        
        let inner_x = container.x + self.padding.left as i32;
        let inner_y = container.y + self.padding.top as i32;
        let inner_w = container.width.saturating_sub(self.padding.left + self.padding.right);
        let inner_h = container.height.saturating_sub(self.padding.top + self.padding.bottom);
        
        // Calculate total size needed
        let mut total_main = 0u32;
        let mut max_cross = 0u32;
        
        for child in children.iter() {
            let pref = child.preferred_size();
            match self.direction {
                FlexDirection::Row => {
                    total_main += pref.width;
                    max_cross = max_cross.max(pref.height);
                }
                FlexDirection::Column => {
                    total_main += pref.height;
                    max_cross = max_cross.max(pref.width);
                }
            }
        }
        total_main += self.gap * (children.len() as u32 - 1);
        
        // Calculate starting position based on alignment
        let main_size = match self.direction {
            FlexDirection::Row => inner_w,
            FlexDirection::Column => inner_h,
        };
        
        let mut pos = match self.align {
            FlexAlign::Start => 0,
            FlexAlign::Center => ((main_size as i32 - total_main as i32) / 2).max(0) as u32,
            FlexAlign::End => main_size.saturating_sub(total_main),
            _ => 0,
        };
        
        // Position each child
        for child in children.iter_mut() {
            let pref = child.preferred_size();
            
            let (x, y, w, h) = match self.direction {
                FlexDirection::Row => {
                    let cross_pos = match self.cross_align {
                        FlexAlign::Start => 0,
                        FlexAlign::Center => ((inner_h as i32 - pref.height as i32) / 2).max(0) as u32,
                        FlexAlign::End => inner_h.saturating_sub(pref.height),
                        _ => 0,
                    };
                    (inner_x + pos as i32, inner_y + cross_pos as i32, pref.width, pref.height)
                }
                FlexDirection::Column => {
                    let cross_pos = match self.cross_align {
                        FlexAlign::Start => 0,
                        FlexAlign::Center => ((inner_w as i32 - pref.width as i32) / 2).max(0) as u32,
                        FlexAlign::End => inner_w.saturating_sub(pref.width),
                        _ => 0,
                    };
                    (inner_x + cross_pos as i32, inner_y + pos as i32, pref.width, pref.height)
                }
            };
            
            child.set_bounds(Rect::new(x, y, w, h));
            
            pos += match self.direction {
                FlexDirection::Row => pref.width + self.gap,
                FlexDirection::Column => pref.height + self.gap,
            };
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INITIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize UI toolkit
pub fn init() {
    set_theme(Theme::dark());
    crate::serial_println!("[UI] Toolkit initialized with dark theme");
}
