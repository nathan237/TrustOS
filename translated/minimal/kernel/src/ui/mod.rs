

















pub mod widgets;

pub use widgets::*;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::any::Any;
use spin::Mutex;

use crate::drivers::virtio_gpu::{GpuSurface, Compositor};






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
    
    pub fn lighten(self, adg: u8) -> Self {
        Self {
            r: self.r.saturating_add(adg),
            g: self.g.saturating_add(adg),
            b: self.b.saturating_add(adg),
            a: self.a,
        }
    }
    
    pub fn darken(self, adg: u8) -> Self {
        Self {
            r: self.r.saturating_sub(adg),
            g: self.g.saturating_sub(adg),
            b: self.b.saturating_sub(adg),
            a: self.a,
        }
    }
    
    
    pub const TRANSPARENT: Color = Color::new(0, 0, 0, 0);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const Acz: Color = Color::rgb(255, 0, 0);
    pub const Zf: Color = Color::rgb(0, 255, 0);
    pub const Wn: Color = Color::rgb(0, 0, 255);
    pub const Asf: Color = Color::rgb(255, 255, 0);
    pub const Ahy: Color = Color::rgb(0, 255, 255);
    pub const Amm: Color = Color::rgb(255, 0, 255);
}


#[derive(Clone)]
pub struct Theme {
    
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,
    
    
    pub fg_primary: Color,
    pub fg_secondary: Color,
    pub fg_disabled: Color,
    
    
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    
    
    pub button_bg: Color,
    pub button_hover: Color,
    pub button_pressed: Color,
    pub button_disabled: Color,
    
    
    pub border: Color,
    pub border_focus: Color,
    
    
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    
    pub border_radius: u32,
    pub border_width: u32,
    pub padding: u32,
    pub spacing: u32,
    
    
    pub font_size: u32,
    pub font_size_small: u32,
    pub font_size_large: u32,
}

impl Theme {
    
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


static NP_: Mutex<Option<Theme>> = Mutex::new(None);

pub fn set_theme(theme: Theme) {
    *NP_.lock() = Some(theme);
}

pub fn qir() -> Theme {
    NP_.lock().clone().unwrap_or_else(Theme::dark)
}






#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub const Bk: Point = Point::new(0, 0);
}


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    
    pub const Bk: Size = Size::new(0, 0);
}


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
    
    pub fn qgl(gw: Point, gn: Point) -> Self {
        let x = gw.x.min(gn.x);
        let y = gw.y.min(gn.y);
        let width = (gw.x - gn.x).unsigned_abs();
        let height = (gw.y - gn.y).unsigned_abs();
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
    
    pub const Bk: Rect = Rect::new(0, 0, 0, 0);
}


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
    
    pub const fn ozm(vertical: u32, horizontal: u32) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }
    
    pub const fn nnf(top: u32, right: u32, bottom: u32, left: u32) -> Self {
        Self { top, right, bottom, left }
    }
    
    pub const Bk: EdgeInsets = EdgeInsets::all(0);
}






#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}


#[derive(Clone, Debug)]
pub enum MouseEvent {
    Move { x: i32, y: i32 },
    Down { x: i32, y: i32, button: MouseButton },
    Up { x: i32, y: i32, button: MouseButton },
    Click { x: i32, y: i32, button: MouseButton },
    DoubleClick { x: i32, y: i32, button: MouseButton },
    Scroll { x: i32, y: i32, mk: i32 },
    Enter,
    Leave,
}


#[derive(Clone, Debug)]
pub enum KeyEvent {
    Down { key: char, modifiers: u8 },
    Up { key: char, modifiers: u8 },
    Char { c: char },
}


pub mod modifiers {
    pub const Adr: u8 = 0x01;
    pub const Xc: u8 = 0x02;
    pub const Wd: u8 = 0x04;
    pub const Ayu: u8 = 0x08;
}


#[derive(Clone, Debug)]
pub enum UiEvent {
    Mouse(MouseEvent),
    Key(KeyEvent),
    Focus,
    Blur,
    Resize(Size),
}






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


pub trait Aw {
    
    fn id(&self) -> u32;
    
    
    fn bounds(&self) -> Rect;
    
    
    fn set_bounds(&mut self, bounds: Rect);
    
    
    fn preferred_size(&self) -> Size {
        Size::new(100, 30)
    }
    
    
    fn qpb(&self) -> Size {
        Size::new(0, 0)
    }
    
    
    fn max_size(&self) -> Size {
        Size::new(u32::MAX, u32::MAX)
    }
    
    
    fn state(&self) -> WidgetState;
    
    
    fn apc(&mut self, state: WidgetState);
    
    
    fn btb(&mut self, event: &UiEvent) -> bool {
        false 
    }
    
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme);
    
    
    fn mlr(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }
}


static CLG_: Mutex<u32> = Mutex::new(1);

pub fn ama() -> u32 {
    let mut id = CLG_.lock();
    let result = *id;
    *id += 1;
    result
}






pub struct Br {
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

impl Br {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            text: text.into(),
            color: None,
            align: TextAlign::Left,
        }
    }
    
    pub fn rcm(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    
    pub fn rck(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

impl Aw for Br {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        
        Size::new((self.text.len() as u32 * 8).max(10), 16)
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let color = self.color.unwrap_or(theme.fg_primary);
        
        
        let x = match self.align {
            TextAlign::Left => self.bounds.x,
            TextAlign::Center => self.bounds.x + (self.bounds.width as i32 - self.text.len() as i32 * 8) / 2,
            TextAlign::Right => self.bounds.x + self.bounds.width as i32 - self.text.len() as i32 * 8,
        };
        
        draw_text(surface, x, self.bounds.y, &self.text, color.to_u32());
    }
}


pub struct Jl {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub text: String,
    pub on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Jl {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
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

impl Aw for Jl {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new((self.text.len() as u32 * 8 + 24).max(80), 32)
    }
    
    fn btb(&mut self, event: &UiEvent) -> bool {
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
        
        
        surface.fill_rounded_rect(
            self.bounds.x as u32,
            self.bounds.y as u32,
            self.bounds.width,
            self.bounds.height,
            theme.border_radius,
            bg.to_u32()
        );
        
        
        surface.draw_rounded_rect(
            self.bounds.x as u32,
            self.bounds.y as u32,
            self.bounds.width,
            self.bounds.height,
            theme.border_radius,
            if self.state.focused { theme.border_focus.to_u32() } else { theme.border.to_u32() }
        );
        
        
        let kd = self.bounds.x + (self.bounds.width as i32 - self.text.len() as i32 * 8) / 2;
        let ie = self.bounds.y + (self.bounds.height as i32 - 16) / 2;
        draw_text(surface, kd, ie, &self.text, fg.to_u32());
    }
}


pub struct Afg {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub text: String,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub max_length: usize,
}

impl Afg {
    pub fn new() -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            text: String::new(),
            placeholder: String::new(),
            cursor_pos: 0,
            max_length: 256,
        }
    }
    
    pub fn rcs(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
}

impl Aw for Afg {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(200, 32)
    }
    
    fn btb(&mut self, event: &UiEvent) -> bool {
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
                    '\x08' => { 
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
        
        
        surface.fill_rounded_rect(
            self.bounds.x as u32,
            self.bounds.y as u32,
            self.bounds.width,
            self.bounds.height,
            theme.border_radius,
            theme.bg_secondary.to_u32()
        );
        
        
        let ri = if self.state.focused {
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
            ri.to_u32()
        );
        
        
        let ie = self.bounds.y + (self.bounds.height as i32 - 16) / 2;
        let kd = self.bounds.x + theme.padding as i32;
        
        if self.text.is_empty() {
            draw_text(surface, kd, ie, &self.placeholder, theme.fg_disabled.to_u32());
        } else {
            draw_text(surface, kd, ie, &self.text, theme.fg_primary.to_u32());
        }
        
        
        if self.state.focused {
            let cursor_x = kd + (self.cursor_pos as i32 * 8);
            surface.fill_rect(
                cursor_x as u32,
                (ie + 2) as u32,
                2,
                12,
                theme.accent.to_u32()
            );
        }
    }
}


pub struct Xg {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub checked: bool,
    pub label: String,
    pub on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl Xg {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
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

impl Aw for Xg {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(24 + self.label.len() as u32 * 8 + 8, 24)
    }
    
    fn btb(&mut self, event: &UiEvent) -> bool {
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
        
        
        let cug = 20u32;
        let ala = self.bounds.x as u32;
        let agf = self.bounds.y as u32 + (self.bounds.height - cug) / 2;
        
        let bg = if self.checked { theme.accent } else { theme.bg_secondary };
        
        surface.fill_rounded_rect(ala, agf, cug, cug, 4, bg.to_u32());
        surface.draw_rounded_rect(ala, agf, cug, cug, 4, theme.border.to_u32());
        
        
        if self.checked {
            let cx = ala as i32 + 5;
            let u = agf as i32 + 10;
            surface.draw_line(cx, u, cx + 4, u + 4, theme.bg_primary.to_u32());
            surface.draw_line(cx + 4, u + 4, cx + 12, u - 4, theme.bg_primary.to_u32());
        }
        
        
        let kd = ala as i32 + cug as i32 + 8;
        let ie = self.bounds.y + (self.bounds.height as i32 - 16) / 2;
        draw_text(surface, kd, ie, &self.label, theme.fg_primary.to_u32());
    }
}


pub struct Acv {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub value: f32,  
    pub show_text: bool,
}

impl Acv {
    pub fn new(value: f32) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            value: value.clamp(0.0, 1.0),
            show_text: true,
        }
    }
    
    pub fn qwp(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

impl Aw for Acv {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(200, 20)
    }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.bounds.width;
        let h = self.bounds.height;
        
        
        surface.fill_rounded_rect(x, y, w, h, h / 2, theme.bg_tertiary.to_u32());
        
        
        let bzr = ((w as f32 * self.value) as u32).max(h);
        if self.value > 0.0 {
            surface.fill_rounded_rect(x, y, bzr, h, h / 2, theme.accent.to_u32());
        }
        
        
        if self.show_text {
            let bup = (self.value * 100.0) as u32;
            let text = format!("{}%", bup);
            let kd = x as i32 + (w as i32 - text.len() as i32 * 8) / 2;
            let ie = y as i32 + (h as i32 - 16) / 2;
            draw_text(surface, kd, ie, &text, theme.fg_primary.to_u32());
        }
    }
}






pub fn draw_text(surface: &mut GpuSurface, x: i32, y: i32, text: &str, color: u32) {
    let mut cx = x;
    for c in text.chars() {
        if cx >= 0 && (cx as u32) < surface.width && y >= 0 && (y as u32) < surface.height {
            draw_char(surface, cx, y, c, color);
        }
        cx += 8;
    }
}


fn draw_char(surface: &mut GpuSurface, x: i32, y: i32, c: char, color: u32) {
    
    let du = crate::framebuffer::font::ol(c);
    
    for row in 0..16 {
        let bits = du[row];
        for col in 0..8 {
            if (bits >> (7 - col)) & 1 == 1 {
                let p = x + col as i32;
                let o = y + row as i32;
                if p >= 0 && o >= 0 {
                    surface.set_pixel(p as u32, o as u32, color);
                }
            }
        }
    }
}






#[derive(Clone, Copy, Debug, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}


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


pub struct Akg {
    pub direction: FlexDirection,
    pub align: FlexAlign,
    pub cross_align: FlexAlign,
    pub gap: u32,
    pub padding: EdgeInsets,
}

impl Akg {
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
    
    
    pub fn layout(&self, container: Rect, children: &mut [&mut dyn Aw]) {
        if children.is_empty() { return; }
        
        let lp = container.x + self.padding.left as i32;
        let eqr = container.y + self.padding.top as i32;
        let rn = container.width.saturating_sub(self.padding.left + self.padding.right);
        let gcv = container.height.saturating_sub(self.padding.top + self.padding.bottom);
        
        
        let mut ecf = 0u32;
        let mut etz = 0u32;
        
        for pd in children.iter() {
            let amg = pd.preferred_size();
            match self.direction {
                FlexDirection::Row => {
                    ecf += amg.width;
                    etz = etz.max(amg.height);
                }
                FlexDirection::Column => {
                    ecf += amg.height;
                    etz = etz.max(amg.width);
                }
            }
        }
        ecf += self.gap * (children.len() as u32 - 1);
        
        
        let ilq = match self.direction {
            FlexDirection::Row => rn,
            FlexDirection::Column => gcv,
        };
        
        let mut pos = match self.align {
            FlexAlign::Start => 0,
            FlexAlign::Center => ((ilq as i32 - ecf as i32) / 2).max(0) as u32,
            FlexAlign::End => ilq.saturating_sub(ecf),
            _ => 0,
        };
        
        
        for pd in children.iter_mut() {
            let amg = pd.preferred_size();
            
            let (x, y, w, h) = match self.direction {
                FlexDirection::Row => {
                    let fpd = match self.cross_align {
                        FlexAlign::Start => 0,
                        FlexAlign::Center => ((gcv as i32 - amg.height as i32) / 2).max(0) as u32,
                        FlexAlign::End => gcv.saturating_sub(amg.height),
                        _ => 0,
                    };
                    (lp + pos as i32, eqr + fpd as i32, amg.width, amg.height)
                }
                FlexDirection::Column => {
                    let fpd = match self.cross_align {
                        FlexAlign::Start => 0,
                        FlexAlign::Center => ((rn as i32 - amg.width as i32) / 2).max(0) as u32,
                        FlexAlign::End => rn.saturating_sub(amg.width),
                        _ => 0,
                    };
                    (lp + fpd as i32, eqr + pos as i32, amg.width, amg.height)
                }
            };
            
            pd.set_bounds(Rect::new(x, y, w, h));
            
            pos += match self.direction {
                FlexDirection::Row => amg.width + self.gap,
                FlexDirection::Column => amg.height + self.gap,
            };
        }
    }
}






pub fn init() {
    set_theme(Theme::dark());
    crate::serial_println!("[UI] Toolkit initialized with dark theme");
}
