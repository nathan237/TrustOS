








use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use super::{Aw, Az, Event, MouseEvent, Rect, Size, Color, CosmicRenderer, ButtonState, theme};






pub struct Jl {
    pub label: String,
    pub on_press: Option<Az>,
    pub style: ButtonStyle,
    state: ButtonState,
    hovered: bool,
    pressed: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonStyle {
    Standard,
    Suggested,
    Destructive,
    Text,
}

impl Jl {
    pub fn new(label: &str) -> Self {
        Self {
            label: String::from(label),
            on_press: None,
            style: ButtonStyle::Standard,
            state: ButtonState::Normal,
            hovered: false,
            pressed: false,
        }
    }
    
    pub fn on_press(mut self, bk: Az) -> Self {
        self.on_press = Some(bk);
        self
    }
    
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }
    
    fn update_state(&mut self) {
        self.state = if self.pressed {
            ButtonState::Pressed
        } else if self.hovered {
            match self.style {
                ButtonStyle::Suggested => ButtonState::Suggested,
                ButtonStyle::Destructive => ButtonState::Destructive,
                _ => ButtonState::Hovered,
            }
        } else {
            match self.style {
                ButtonStyle::Suggested => ButtonState::Suggested,
                ButtonStyle::Destructive => ButtonState::Destructive,
                _ => ButtonState::Normal,
            }
        };
    }
}

impl Aw for Jl {
    fn size(&self) -> Size {
        
        let ebn = self.label.len() as f32 * 8.0;
        Size::new(ebn + 32.0, 36.0)
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        renderer.draw_button(bounds, &self.label, self.state);
    }
    
    fn on_event(&mut self, event: &Event, bounds: Rect) -> Option<Az> {
        match event {
            Event::Mouse(MouseEvent::Move { x, y }) => {
                self.hovered = bounds.contains(*x, *y);
                self.update_state();
                None
            }
            Event::Mouse(MouseEvent::Press { x, y, .. }) => {
                if bounds.contains(*x, *y) {
                    self.pressed = true;
                    self.update_state();
                }
                None
            }
            Event::Mouse(MouseEvent::Release { x, y, .. }) => {
                if self.pressed && bounds.contains(*x, *y) {
                    self.pressed = false;
                    self.update_state();
                    return self.on_press;
                }
                self.pressed = false;
                self.update_state();
                None
            }
            _ => None,
        }
    }
}






pub struct Br {
    pub text: String,
    pub color: Option<Color>,
    pub size: LabelSize,
}

#[derive(Clone, Copy)]
pub enum LabelSize {
    Small,
    Normal,
    Large,
    Title,
}

impl Br {
    pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
            color: None,
            size: LabelSize::Normal,
        }
    }
    
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    
    pub fn qxb(mut self, j: LabelSize) -> Self {
        self.size = j;
        self
    }
}

impl Aw for Br {
    fn size(&self) -> Size {
        let ew = match self.size {
            LabelSize::Small => 6.0,
            LabelSize::Normal => 8.0,
            LabelSize::Large => 10.0,
            LabelSize::Title => 14.0,
        };
        let height = match self.size {
            LabelSize::Small => 14.0,
            LabelSize::Normal => 18.0,
            LabelSize::Large => 24.0,
            LabelSize::Title => 32.0,
        };
        Size::new(self.text.len() as f32 * ew, height)
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        
        
        let t = theme();
        let color = self.color.unwrap_or(t.text_primary);
        
        
        renderer.draw_line(
            super::Point::new(bounds.x, bounds.y + bounds.height - 2.0),
            super::Point::new(bounds.x + bounds.width, bounds.y + bounds.height - 2.0),
            color.with_alpha(0.3),
            1.0,
        );
    }
    
    fn on_event(&mut self, _event: &Event, _bounds: Rect) -> Option<Az> {
        None 
    }
}






pub struct Xi {
    children: Vec<Box<dyn Aw>>,
    pub padding: f32,
    pub spacing: f32,
    pub direction: Direction,
    pub background: Option<Color>,
    pub border_radius: f32,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
}

impl Xi {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            padding: 12.0,
            spacing: 8.0,
            direction: Direction::Vertical,
            background: None,
            border_radius: 0.0,
        }
    }
    
    pub fn push<W: Aw + 'static>(mut self, akq: W) -> Self {
        self.children.push(Box::new(akq));
        self
    }
    
    pub fn padding(mut self, aa: f32) -> Self {
        self.padding = aa;
        self
    }
    
    pub fn spacing(mut self, j: f32) -> Self {
        self.spacing = j;
        self
    }
    
    pub fn direction(mut self, d: Direction) -> Self {
        self.direction = d;
        self
    }
    
    pub fn background(mut self, c: Color) -> Self {
        self.background = Some(c);
        self
    }
    
    pub fn border_radius(mut self, r: f32) -> Self {
        self.border_radius = r;
        self
    }
}

impl Aw for Xi {
    fn size(&self) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        
        for pd in &self.children {
            let j = pd.size();
            match self.direction {
                Direction::Vertical => {
                    width = width.max(j.width);
                    height += j.height + self.spacing;
                }
                Direction::Horizontal => {
                    width += j.width + self.spacing;
                    height = height.max(j.height);
                }
            }
        }
        
        Size::new(
            width + self.padding * 2.0,
            height + self.padding * 2.0 - self.spacing,
        )
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        
        if let Some(bg) = self.background {
            if self.border_radius > 0.0 {
                renderer.fill_rounded_rect(bounds, self.border_radius, bg);
            } else {
                renderer.fill_rect(bounds, bg);
            }
        }
        
        
        let mut offset = self.padding;
        
        for pd in &self.children {
            let j = pd.size();
            let flo = match self.direction {
                Direction::Vertical => {
                    let b = Rect::new(
                        bounds.x + self.padding,
                        bounds.y + offset,
                        bounds.width - self.padding * 2.0,
                        j.height,
                    );
                    offset += j.height + self.spacing;
                    b
                }
                Direction::Horizontal => {
                    let b = Rect::new(
                        bounds.x + offset,
                        bounds.y + self.padding,
                        j.width,
                        bounds.height - self.padding * 2.0,
                    );
                    offset += j.width + self.spacing;
                    b
                }
            };
            
            pd.draw(renderer, flo);
        }
    }
    
    fn on_event(&mut self, event: &Event, bounds: Rect) -> Option<Az> {
        
        let mut offset = self.padding;
        
        for pd in &mut self.children {
            let j = pd.size();
            let flo = match self.direction {
                Direction::Vertical => {
                    let b = Rect::new(
                        bounds.x + self.padding,
                        bounds.y + offset,
                        bounds.width - self.padding * 2.0,
                        j.height,
                    );
                    offset += j.height + self.spacing;
                    b
                }
                Direction::Horizontal => {
                    let b = Rect::new(
                        bounds.x + offset,
                        bounds.y + self.padding,
                        j.width,
                        bounds.height - self.padding * 2.0,
                    );
                    offset += j.width + self.spacing;
                    b
                }
            };
            
            if let Some(bk) = pd.on_event(event, flo) {
                return Some(bk);
            }
        }
        
        None
    }
}






pub struct Zr {
    pub title: String,
    pub show_controls: bool,
    focused: bool,
}

impl Zr {
    pub fn new(title: &str) -> Self {
        Self {
            title: String::from(title),
            show_controls: true,
            focused: true,
        }
    }
    
    pub fn qvw(&mut self, f: bool) {
        self.focused = f;
    }
}

impl Aw for Zr {
    fn size(&self) -> Size {
        Size::new(400.0, 40.0) 
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        renderer.draw_header(bounds, &self.title, self.focused);
    }
    
    fn on_event(&mut self, event: &Event, bounds: Rect) -> Option<Az> {
        
        if let Event::Mouse(MouseEvent::Press { x, y, .. }) = event {
            let wv = 14.0;
            let ed = bounds.y + (bounds.height - wv) / 2.0;
            
            
            let adl = bounds.x + bounds.width - wv - 12.0;
            if *x >= adl && *x <= adl + wv &&
               *y >= ed && *y <= ed + wv {
                return Some(1); 
            }
            
            
            let aly = adl - wv - 8.0;
            if *x >= aly && *x <= aly + wv &&
               *y >= ed && *y <= ed + wv {
                return Some(2); 
            }
            
            
            let ayg = aly - wv - 8.0;
            if *x >= ayg && *x <= ayg + wv &&
               *y >= ed && *y <= ed + wv {
                return Some(3); 
            }
        }
        
        None
    }
}






pub struct Bs {
    pub height: f32,
}

impl Bs {
    pub fn new() -> Self {
        Self { height: 32.0 }
    }
}

impl Aw for Bs {
    fn size(&self) -> Size {
        Size::new(1280.0, self.height)
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        renderer.draw_panel(bounds);
    }
    
    fn on_event(&mut self, _event: &Event, _bounds: Rect) -> Option<Az> {
        None
    }
}
