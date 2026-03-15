//! Calculator Application
//!
//! A graphical calculator built with the TrustOS UI toolkit.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

use crate::ui::{
    Widget, WidgetState, Color, Theme, Rect, Point, Size,
    UiEvent, MouseEvent, MouseButton,
    Button, Label, next_widget_id, draw_text,
};
use crate::drivers::virtio_gpu::GpuSurface;

/// Calculator button types
#[derive(Clone, Copy, PartialEq)]
pub enum CalcButton {
    Digit(u8),      // 0-9
    Add,            // +
    Subtract,       // -
    Multiply,       // ×
    Divide,         // ÷
    Equals,         // =
    Clear,          // C
    ClearEntry,     // CE
    Backspace,      // ⌫
    Decimal,        // .
    Negate,         // ±
    Percent,        // %
    MemoryAdd,      // M+
    MemorySub,      // M-
    MemoryRecall,   // MR
    MemoryClear,    // MC
}

impl CalcButton {
    fn label(&self) -> &'static str {
        match self {
            CalcButton::Digit(0) => "0",
            CalcButton::Digit(1) => "1",
            CalcButton::Digit(2) => "2",
            CalcButton::Digit(3) => "3",
            CalcButton::Digit(4) => "4",
            CalcButton::Digit(5) => "5",
            CalcButton::Digit(6) => "6",
            CalcButton::Digit(7) => "7",
            CalcButton::Digit(8) => "8",
            CalcButton::Digit(9) => "9",
            CalcButton::Add => "+",
            CalcButton::Subtract => "-",
            CalcButton::Multiply => "x",
            CalcButton::Divide => "/",
            CalcButton::Equals => "=",
            CalcButton::Clear => "C",
            CalcButton::ClearEntry => "CE",
            CalcButton::Backspace => "<",
            CalcButton::Decimal => ".",
            CalcButton::Negate => "+/-",
            CalcButton::Percent => "%",
            CalcButton::MemoryAdd => "M+",
            CalcButton::MemorySub => "M-",
            CalcButton::MemoryRecall => "MR",
            CalcButton::MemoryClear => "MC",
            _ => "?",
        }
    }
}

/// Calculator state
pub struct Calculator {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    
    // Display
    display: String,
    expression: String,
    
    // Calculation state
    current: f64,
    previous: f64,
    operator: Option<CalcButton>,
    new_entry: bool,
    
    // Memory
    memory: f64,
    
    // UI
    button_grid: Vec<(CalcButton, Rect)>,
    hovered_button: Option<usize>,
    pressed_button: Option<usize>,
    
    // Theme
    use_dark_theme: bool,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            bounds: Rect::ZERO,
            state: WidgetState::new(),
            display: String::from("0"),
            expression: String::new(),
            current: 0.0,
            previous: 0.0,
            operator: None,
            new_entry: true,
            memory: 0.0,
            button_grid: Vec::new(),
            hovered_button: None,
            pressed_button: None,
            use_dark_theme: true,
        }
    }
    
    /// Layout the button grid
    fn layout_buttons(&mut self) {
        self.button_grid.clear();
        
        let x = self.bounds.x as u32 + 10;
        let y = self.bounds.y as u32 + 90; // Leave room for display
        let button_w = 60;
        let button_h = 50;
        let gap = 8;
        
        // Button layout (5 columns, 5 rows)
        let layout: [[CalcButton; 5]; 5] = [
            [CalcButton::MemoryClear, CalcButton::MemoryRecall, CalcButton::MemoryAdd, CalcButton::MemorySub, CalcButton::Backspace],
            [CalcButton::Clear, CalcButton::ClearEntry, CalcButton::Percent, CalcButton::Divide, CalcButton::Digit(7)],
            [CalcButton::Digit(4), CalcButton::Digit(5), CalcButton::Digit(6), CalcButton::Multiply, CalcButton::Digit(8)],
            [CalcButton::Digit(1), CalcButton::Digit(2), CalcButton::Digit(3), CalcButton::Subtract, CalcButton::Digit(9)],
            [CalcButton::Negate, CalcButton::Digit(0), CalcButton::Decimal, CalcButton::Add, CalcButton::Equals],
        ];
        
        // Rearrange to standard calculator layout
        let standard_layout: [[CalcButton; 4]; 5] = [
            [CalcButton::Clear, CalcButton::ClearEntry, CalcButton::Backspace, CalcButton::Divide],
            [CalcButton::Digit(7), CalcButton::Digit(8), CalcButton::Digit(9), CalcButton::Multiply],
            [CalcButton::Digit(4), CalcButton::Digit(5), CalcButton::Digit(6), CalcButton::Subtract],
            [CalcButton::Digit(1), CalcButton::Digit(2), CalcButton::Digit(3), CalcButton::Add],
            [CalcButton::Negate, CalcButton::Digit(0), CalcButton::Decimal, CalcButton::Equals],
        ];
        
        for (row_idx, row) in standard_layout.iter().enumerate() {
            for (col_idx, btn) in row.iter().enumerate() {
                let bx = x + (col_idx as u32) * (button_w + gap);
                let by = y + (row_idx as u32) * (button_h + gap);
                
                self.button_grid.push((*btn, Rect::new(
                    bx as i32, by as i32, button_w, button_h
                )));
            }
        }
    }
    
    fn button_at(&self, x: i32, y: i32) -> Option<usize> {
        for (i, (_, rect)) in self.button_grid.iter().enumerate() {
            if rect.contains(Point::new(x, y)) {
                return Some(i);
            }
        }
        None
    }
    
    fn handle_button(&mut self, btn: CalcButton) {
        match btn {
            CalcButton::Digit(d) => {
                if self.new_entry {
                    self.display = format!("{}", d);
                    self.new_entry = false;
                } else if self.display.len() < 15 {
                    if self.display == "0" {
                        self.display = format!("{}", d);
                    } else {
                        self.display.push((b'0' + d) as char);
                    }
                }
                self.current = self.display.parse().unwrap_or(0.0);
            }
            
            CalcButton::Decimal => {
                if self.new_entry {
                    self.display = String::from("0.");
                    self.new_entry = false;
                } else if !self.display.contains('.') {
                    self.display.push('.');
                }
            }
            
            CalcButton::Clear => {
                self.display = String::from("0");
                self.expression.clear();
                self.current = 0.0;
                self.previous = 0.0;
                self.operator = None;
                self.new_entry = true;
            }
            
            CalcButton::ClearEntry => {
                self.display = String::from("0");
                self.current = 0.0;
                self.new_entry = true;
            }
            
            CalcButton::Backspace => {
                if self.display.len() > 1 {
                    self.display.pop();
                } else {
                    self.display = String::from("0");
                }
                self.current = self.display.parse().unwrap_or(0.0);
            }
            
            CalcButton::Negate => {
                if self.current != 0.0 {
                    self.current = -self.current;
                    if self.display.starts_with('-') {
                        self.display = String::from(&self.display[1..]);
                    } else {
                        self.display = format!("-{}", self.display);
                    }
                }
            }
            
            CalcButton::Percent => {
                self.current = self.current / 100.0;
                self.display = self.format_number(self.current);
            }
            
            CalcButton::Add | CalcButton::Subtract | CalcButton::Multiply | CalcButton::Divide => {
                self.execute_pending();
                self.operator = Some(btn);
                self.previous = self.current;
                self.new_entry = true;
                
                let op_char = match btn {
                    CalcButton::Add => "+",
                    CalcButton::Subtract => "-",
                    CalcButton::Multiply => "×",
                    CalcButton::Divide => "÷",
                    _ => "",
                };
                self.expression = format!("{} {}", self.display, op_char);
            }
            
            CalcButton::Equals => {
                self.execute_pending();
                self.expression.clear();
                self.operator = None;
                self.new_entry = true;
            }
            
            CalcButton::MemoryAdd => {
                self.memory += self.current;
            }
            CalcButton::MemorySub => {
                self.memory -= self.current;
            }
            CalcButton::MemoryRecall => {
                self.current = self.memory;
                self.display = self.format_number(self.memory);
                self.new_entry = true;
            }
            CalcButton::MemoryClear => {
                self.memory = 0.0;
            }
        }
    }
    
    fn execute_pending(&mut self) {
        if let Some(op) = self.operator {
            let result = match op {
                CalcButton::Add => self.previous + self.current,
                CalcButton::Subtract => self.previous - self.current,
                CalcButton::Multiply => self.previous * self.current,
                CalcButton::Divide => {
                    if self.current != 0.0 {
                        self.previous / self.current
                    } else {
                        f64::NAN
                    }
                }
                _ => self.current,
            };
            
            self.current = result;
            self.display = self.format_number(result);
        }
    }
    
    fn format_number(&self, n: f64) -> String {
        if n.is_nan() {
            return String::from("Error");
        }
        if n.is_infinite() {
            return String::from("Infinity");
        }
        
        // Format with reasonable precision
        // Simple floor check: n == (n as i64) as f64
        let is_integer = n == (n as i64) as f64 && n.abs() < 1e15;
        if is_integer {
            format!("{:.0}", n)
        } else {
            let s = format!("{:.10}", n);
            // Trim trailing zeros
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            String::from(s)
        }
    }
    
    fn button_color(&self, btn: &CalcButton, theme: &Theme, is_hovered: bool, is_pressed: bool) -> Color {
        let base = match btn {
            CalcButton::Equals => theme.accent,
            CalcButton::Add | CalcButton::Subtract | CalcButton::Multiply | CalcButton::Divide => {
                Color::new(80, 80, 90, 255)
            }
            CalcButton::Clear | CalcButton::ClearEntry => {
                Color::new(120, 60, 60, 255)
            }
            _ => theme.button_bg,
        };
        
        if is_pressed {
            base.darken(20)
        } else if is_hovered {
            base.lighten(15)
        } else {
            base
        }
    }
}

impl Widget for Calculator {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { 
        self.bounds = bounds;
        self.layout_buttons();
    }
    fn state(&self) -> WidgetState { self.state }
    fn set_state(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(290, 400)
    }
    
    fn handle_event(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Mouse(MouseEvent::Move { x, y }) => {
                self.hovered_button = self.button_at(*x, *y);
                true
            }
            UiEvent::Mouse(MouseEvent::Down { x, y, button: MouseButton::Left }) => {
                self.pressed_button = self.button_at(*x, *y);
                true
            }
            UiEvent::Mouse(MouseEvent::Up { button: MouseButton::Left, .. }) => {
                if let Some(idx) = self.pressed_button {
                    if Some(idx) == self.hovered_button {
                        let btn = self.button_grid[idx].0;
                        self.handle_button(btn);
                    }
                }
                self.pressed_button = None;
                true
            }
            UiEvent::Mouse(MouseEvent::Leave) => {
                self.hovered_button = None;
                self.pressed_button = None;
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
        
        // Window background
        surface.fill_rounded_rect(x, y, w, h, 12, theme.bg_primary.to_u32());
        surface.draw_rounded_rect(x, y, w, h, 12, theme.border.to_u32());
        
        // Title bar
        surface.fill_rect(x + 1, y + 1, w - 2, 28, theme.bg_tertiary.to_u32());
        draw_text(surface, x as i32 + 10, y as i32 + 6, "Calculator", theme.fg_secondary.to_u32());
        
        // Display background
        let disp_x = x + 10;
        let disp_y = y + 35;
        let disp_w = w - 20;
        let disp_h = 50;
        surface.fill_rounded_rect(disp_x, disp_y, disp_w, disp_h, 6, Color::new(30, 30, 35, 255).to_u32());
        
        // Expression (small, top)
        if !self.expression.is_empty() {
            let expr_x = disp_x as i32 + disp_w as i32 - (self.expression.len() as i32 * 8) - 8;
            draw_text(surface, expr_x, disp_y as i32 + 4, &self.expression, theme.fg_secondary.to_u32());
        }
        
        // Main display (large, right-aligned)
        let display_x = disp_x as i32 + disp_w as i32 - (self.display.len() as i32 * 12) - 10;
        let display_y = disp_y as i32 + 22;
        
        // Draw larger display text (2x scale simulation)
        for (i, c) in self.display.chars().enumerate() {
            let cx = display_x + (i as i32 * 12);
            let char_str = alloc::string::String::from(c);
            draw_text(surface, cx, display_y, &char_str, theme.fg_primary.to_u32());
            draw_text(surface, cx + 1, display_y, &char_str, theme.fg_primary.to_u32());
        }
        
        // Memory indicator
        if self.memory != 0.0 {
            surface.fill_rounded_rect(disp_x, disp_y, 20, 16, 3, theme.accent.with_alpha(60).to_u32());
            draw_text(surface, disp_x as i32 + 4, disp_y as i32 + 2, "M", theme.accent.to_u32());
        }
        
        // Render buttons
        for (i, (btn, rect)) in self.button_grid.iter().enumerate() {
            let is_hovered = self.hovered_button == Some(i);
            let is_pressed = self.pressed_button == Some(i);
            
            let bg_color = self.button_color(btn, theme, is_hovered, is_pressed);
            
            surface.fill_rounded_rect(
                rect.x as u32, rect.y as u32,
                rect.width, rect.height,
                8,
                bg_color.to_u32()
            );
            
            // Button border
            surface.draw_rounded_rect(
                rect.x as u32, rect.y as u32,
                rect.width, rect.height,
                8,
                theme.border.to_u32()
            );
            
            // Button label (centered)
            let label = btn.label();
            let text_w = label.len() as i32 * 8;
            let text_x = rect.x + (rect.width as i32 - text_w) / 2;
            let text_y = rect.y + (rect.height as i32 - 16) / 2;
            
            let text_color = if matches!(btn, CalcButton::Equals) {
                Color::WHITE
            } else {
                theme.fg_primary
            };
            
            draw_text(surface, text_x, text_y, label, text_color.to_u32());
        }
    }
}

/// Simple calculator window that can be displayed on the desktop
pub fn create_calculator_window() -> Calculator {
    let mut calc = Calculator::new();
    calc.set_bounds(Rect::new(100, 100, 290, 400));
    calc
}
