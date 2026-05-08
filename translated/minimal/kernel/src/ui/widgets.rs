







use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::{
    Aw, WidgetState, Color, Theme, Rect, Point, Size, EdgeInsets,
    UiEvent, MouseEvent, MouseButton, KeyEvent,
    ama, draw_text,
};
use crate::drivers::virtio_gpu::GpuSurface;






pub struct Aex {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub value: f32,        
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub show_value: bool,
    pub on_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
    dragging: bool,
}

impl Aex {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
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
    
    pub fn rct(mut self, step: f32) -> Self {
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
        let joi = self.bounds.x + 8;
        let pmv = self.bounds.x + self.bounds.width as i32 - 8;
        let joj = pmv - joi;
        
        if joj > 0 {
            let xj = (x - joi) as f32 / joj as f32;
            self.value = xj.clamp(0.0, 1.0);
            
            
            if self.step > 0.0 {
                let range = self.max - self.min;
                let oxf = self.value * range / self.step;
                let steps = (oxf + 0.5) as i32 as f32; 
                self.value = (steps * self.step / range).clamp(0.0, 1.0);
            }
            
            if let Some(ref on_change) = self.on_change {
                on_change(self.actual_value());
            }
        }
    }
}

impl Aw for Aex {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(200, 24)
    }
    
    fn btb(&mut self, event: &UiEvent) -> bool {
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
        
        
        let bwn = y + h / 2 - 2;
        surface.fill_rounded_rect(x + 4, bwn, w - 8, 4, 2, theme.bg_tertiary.to_u32());
        
        
        let bzr = ((w - 16) as f32 * self.value) as u32;
        if bzr > 0 {
            surface.fill_rounded_rect(x + 4, bwn, bzr + 4, 4, 2, theme.accent.to_u32());
        }
        
        
        let pje = x + 4 + bzr;
        let akn = y + h / 2 - 8;
        let pjc = if self.dragging { theme.accent } else if self.state.hovered { theme.accent_hover } else { theme.fg_primary };
        surface.fill_circle(pje as i32 + 4, akn as i32 + 8, 8, pjc.to_u32());
        
        
        if self.show_value {
            let val = self.actual_value();
            let text = if self.step >= 1.0 {
                format!("{}", val as i32)
            } else {
                format!("{:.1}", val)
            };
            let kd = x as i32 + w as i32 + 8;
            let ie = y as i32 + (h as i32 - 16) / 2;
            draw_text(surface, kd, ie, &text, theme.fg_secondary.to_u32());
        }
    }
}






pub struct Adi {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub selected: bool,
    pub label: String,
    pub bbz: u32,
    pub on_select: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Adi {
    pub fn new(label: impl Into<String>, bbz: u32) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            selected: false,
            label: label.into(),
            bbz,
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

impl Aw for Adi {
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
        
        let hla = self.bounds.x as i32 + 10;
        let hlb = self.bounds.y as i32 + self.bounds.height as i32 / 2;
        
        
        surface.draw_circle(hla, hlb, 8, theme.border.to_u32());
        
        
        if self.selected {
            surface.fill_circle(hla, hlb, 5, theme.accent.to_u32());
        }
        
        
        let kd = self.bounds.x as i32 + 28;
        let ie = self.bounds.y as i32 + (self.bounds.height as i32 - 16) / 2;
        draw_text(surface, kd, ie, &self.label, theme.fg_primary.to_u32());
    }
}






pub struct Xy {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub expanded: bool,
    pub on_change: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

impl Xy {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
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

impl Aw for Xy {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        let aoo = self.options.iter().map(|j| j.len()).max().unwrap_or(10);
        Size::new((aoo as u32 * 8 + 32).max(120), 32)
    }
    
    fn btb(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Mouse(MouseEvent::Click { x, y, button: MouseButton::Left }) => {
                if self.expanded {
                    
                    let nnr = self.bounds.y + self.bounds.height as i32;
                    let idx = ((*y - nnr) / 28) as usize;
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
        
        
        let bg = if self.state.hovered { theme.button_hover } else { theme.button_bg };
        surface.fill_rounded_rect(x, y, w, h, theme.border_radius, bg.to_u32());
        surface.draw_rounded_rect(x, y, w, h, theme.border_radius, theme.border.to_u32());
        
        
        if let Some(text) = self.selected_value() {
            let kd = x as i32 + 12;
            let ie = y as i32 + (h as i32 - 16) / 2;
            draw_text(surface, kd, ie, text, theme.fg_primary.to_u32());
        }
        
        
        let cfx = x + w - 20;
        let efp = y + h / 2;
        surface.draw_line(cfx as i32, efp as i32 - 2, cfx as i32 + 4, efp as i32 + 2, theme.fg_secondary.to_u32());
        surface.draw_line(cfx as i32 + 4, efp as i32 + 2, cfx as i32 + 8, efp as i32 - 2, theme.fg_secondary.to_u32());
        
        
        if self.expanded {
            let ftf = y + h + 2;
            let htu = (self.options.len() as u32 * 28).min(200);
            
            surface.fill_rounded_rect(x, ftf, w, htu, theme.border_radius, theme.bg_secondary.to_u32());
            surface.draw_rounded_rect(x, ftf, w, htu, theme.border_radius, theme.border.to_u32());
            
            for (i, option) in self.options.iter().enumerate() {
                let isl = ftf + i as u32 * 28;
                
                if i == self.selected_index {
                    surface.fill_rect(x + 2, isl + 2, w - 4, 24, theme.accent.with_alpha(40).to_u32());
                }
                
                let kd = x as i32 + 12;
                let ie = isl as i32 + 6;
                let color = if i == self.selected_index { theme.accent } else { theme.fg_primary };
                draw_text(surface, kd, ie, option, color.to_u32());
            }
        }
    }
}






pub struct Aeo {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub content_height: u32,
    pub scroll_y: i32,
    pub scroll_speed: i32,
}

impl Aeo {
    pub fn new(content_height: u32) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            content_height,
            scroll_y: 0,
            scroll_speed: 20,
        }
    }
    
    pub fn scroll_to(&mut self, y: i32) {
        let aab = (self.content_height as i32 - self.bounds.height as i32).max(0);
        self.scroll_y = y.clamp(0, aab);
    }
    
    pub fn scroll_by(&mut self, mk: i32) {
        self.scroll_to(self.scroll_y + mk);
    }
    
    
    pub fn rca(&self) -> Rect {
        Rect::new(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width.saturating_sub(12), 
            self.bounds.height,
        )
    }
}

impl Aw for Aeo {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn btb(&mut self, event: &UiEvent) -> bool {
        match event {
            UiEvent::Mouse(MouseEvent::Scroll { mk, .. }) => {
                self.scroll_by(-mk * self.scroll_speed);
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
        
        
        surface.fill_rect(x, y, w, h, theme.bg_primary.to_u32());
        
        
        let yc = x + w - 8;
        surface.fill_rect(yc, y, 8, h, theme.bg_tertiary.to_u32());
        
        
        if self.content_height > h {
            let psk = h as f32 / self.content_height as f32;
            let jmj = ((h as f32 * psk) as u32).max(20);
            let gsz = self.scroll_y as f32 / (self.content_height - h) as f32;
            let akn = y + ((h - jmj) as f32 * gsz) as u32;
            
            surface.fill_rounded_rect(yc, akn, 8, jmj, 4, theme.fg_secondary.to_u32());
        }
    }
}






pub struct Abk {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
}

impl Abk {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            title: title.into(),
            width,
            height,
            visible: false,
        }
    }
    
    pub fn dzi(&mut self) {
        self.visible = true;
    }
    
    pub fn mlj(&mut self) {
        self.visible = false;
    }
    
    
    pub fn kxm(&self) -> Rect {
        Rect::new(
            self.bounds.x + 16,
            self.bounds.y + 48,
            self.width - 32,
            self.height - 64,
        )
    }
}

impl Aw for Abk {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { 
        
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
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.visible { return; }
        
        let (dy, dw) = (surface.width, surface.height);
        
        
        for y in 0..dw {
            for x in 0..dy {
                let ku = surface.get_pixel(x, y);
                let ler = ((ku >> 16 & 0xFF) / 2) << 16 
                           | ((ku >> 8 & 0xFF) / 2) << 8 
                           | (ku & 0xFF) / 2
                           | 0xC0000000;
                surface.set_pixel(x, y, ler);
            }
        }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.width;
        let h = self.height;
        
        
        surface.fill_rounded_rect(x + 4, y + 4, w, h, 8, 0x40000000); 
        surface.fill_rounded_rect(x, y, w, h, 8, theme.bg_secondary.to_u32());
        surface.draw_rounded_rect(x, y, w, h, 8, theme.border.to_u32());
        
        
        surface.fill_rect(x + 1, y + 1, w - 2, 40, theme.bg_tertiary.to_u32());
        
        
        let avk = x as i32 + 16;
        let apg = y as i32 + 12;
        draw_text(surface, avk, apg, &self.title, theme.fg_primary.to_u32());
        
        
        let adl = x + w - 32;
        let hlp = y + 12;
        surface.fill_rounded_rect(adl, hlp, 20, 20, 4, theme.error.with_alpha(40).to_u32());
        draw_text(surface, adl as i32 + 6, hlp as i32 + 2, "×", theme.error.to_u32());
    }
}






pub struct Bs {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub title: Option<String>,
    pub padding: EdgeInsets,
}

impl Bs {
    pub fn new() -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            title: None,
            padding: EdgeInsets::all(12),
        }
    }
    
    pub fn rcu(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    pub fn rcq(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }
    
    
    pub fn kxm(&self) -> Rect {
        let jmv = if self.title.is_some() { 32 } else { 0 };
        Rect::new(
            self.bounds.x + self.padding.left as i32,
            self.bounds.y + self.padding.top as i32 + jmv,
            self.bounds.width - self.padding.left - self.padding.right,
            self.bounds.height - self.padding.top - self.padding.bottom - jmv as u32,
        )
    }
}

impl Aw for Bs {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.state.visible { return; }
        
        let x = self.bounds.x as u32;
        let y = self.bounds.y as u32;
        let w = self.bounds.width;
        let h = self.bounds.height;
        
        
        surface.fill_rounded_rect(x, y, w, h, theme.border_radius, theme.bg_secondary.to_u32());
        surface.draw_rounded_rect(x, y, w, h, theme.border_radius, theme.border.to_u32());
        
        
        if let Some(ref title) = self.title {
            let apg = y + 8;
            draw_text(surface, x as i32 + 12, apg as i32, title, theme.fg_secondary.to_u32());
            
            
            surface.draw_line(x as i32 + 8, (y + 28) as i32, (x + w - 8) as i32, (y + 28) as i32, theme.border.to_u32());
        }
    }
}






pub struct Xv {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub vertical: bool,
}

impl Xv {
    pub fn horizontal() -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            vertical: false,
        }
    }
    
    pub fn vertical() -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            vertical: true,
        }
    }
}

impl Aw for Xv {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
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






pub struct Aab {
    id: u32,
    bounds: Rect,
    state: WidgetState,
    pub icon: char,
    pub tooltip: Option<String>,
    pub on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Aab {
    pub fn new(icon: char) -> Self {
        Self {
            id: ama(),
            bounds: Rect::Bk,
            state: WidgetState::new(),
            icon,
            tooltip: None,
            on_click: None,
        }
    }
    
    pub fn rcv(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
    
    pub fn on_click<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl Aw for Aab {
    fn id(&self) -> u32 { self.id }
    fn bounds(&self) -> Rect { self.bounds }
    fn set_bounds(&mut self, bounds: Rect) { self.bounds = bounds; }
    fn state(&self) -> WidgetState { self.state }
    fn apc(&mut self, state: WidgetState) { self.state = state; }
    
    fn preferred_size(&self) -> Size {
        Size::new(36, 36)
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
        
        
        let drr = alloc::string::String::from(self.icon);
        let kd = x as i32 + (size as i32 - 8) / 2;
        let ie = y as i32 + (size as i32 - 16) / 2;
        draw_text(surface, kd, ie, &drr, theme.fg_primary.to_u32());
    }
}






pub struct Aqr {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
}

impl Aqr {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            x: 0,
            y: 0,
            visible: false,
        }
    }
    
    pub fn qwv(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        self.visible = true;
    }
    
    pub fn mlj(&mut self) {
        self.visible = false;
    }
    
    pub fn render(&self, surface: &mut GpuSurface, theme: &Theme) {
        if !self.visible { return; }
        
        let padding = 8;
        let w = (self.text.len() as u32 * 8 + padding * 2) as u32;
        let h = 24;
        
        let x = self.x as u32;
        let y = self.y as u32;
        
        
        surface.fill_rounded_rect(x, y, w, h, 4, theme.bg_tertiary.to_u32());
        surface.draw_rounded_rect(x, y, w, h, 4, theme.border.to_u32());
        
        
        draw_text(surface, x as i32 + padding as i32, y as i32 + 4, &self.text, theme.fg_primary.to_u32());
    }
}
