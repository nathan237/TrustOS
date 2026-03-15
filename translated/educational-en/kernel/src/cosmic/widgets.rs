//! COSMIC Widgets - UI components inspired by libcosmic
//!
//! Implements common widgets:
//! - Button
//! - Label
//! - Container
//! - Header
//! - TextInput

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use super::{Widget, Message, Event, MouseEvent, Rect, Size, Color, CosmicRenderer, ButtonState, theme};

// ═══════════════════════════════════════════════════════════════════════════════
// BUTTON WIDGET
// ═══════════════════════════════════════════════════════════════════════════════

/// A COSMIC-style button
pub struct Button {
    pub label: String,
    pub on_press: Option<Message>,
    pub style: ButtonStyle,
    state: ButtonState,
    hovered: bool,
    pressed: bool,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum ButtonStyle {
    Standard,
    Suggested,
    Destructive,
    Text,
}

// Implementation block — defines methods for the type above.
impl Button {
        // Public function — callable from other modules.
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
    
        // Public function — callable from other modules.
pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }
    
        // Public function — callable from other modules.
pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }
    
    fn update_state(&mut self) {
        self.state = if self.pressed {
            ButtonState::Pressed
        } else if self.hovered {
                        // Pattern matching — Rust's exhaustive branching construct.
match self.style {
                ButtonStyle::Suggested => ButtonState::Suggested,
                ButtonStyle::Destructive => ButtonState::Destructive,
                _ => ButtonState::Hovered,
            }
        } else {
                        // Pattern matching — Rust's exhaustive branching construct.
match self.style {
                ButtonStyle::Suggested => ButtonState::Suggested,
                ButtonStyle::Destructive => ButtonState::Destructive,
                _ => ButtonState::Normal,
            }
        };
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Widget for Button {
    fn size(&self) -> Size {
        // Calculate size based on label length + padding
        let text_width = self.label.len() as f32 * 8.0;
        Size::new(text_width + 32.0, 36.0)
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        renderer.draw_button(bounds, &self.label, self.state);
    }
    
    fn on_event(&mut self, event: &Event, bounds: Rect) -> Option<Message> {
                // Pattern matching — Rust's exhaustive branching construct.
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

// ═══════════════════════════════════════════════════════════════════════════════
// LABEL WIDGET
// ═══════════════════════════════════════════════════════════════════════════════

/// A text label
pub struct Label {
    pub text: String,
    pub color: Option<Color>,
    pub size: LabelSize,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
// Enumeration — a type that can be one of several variants.
pub enum LabelSize {
    Small,
    Normal,
    Large,
    Title,
}

// Implementation block — defines methods for the type above.
impl Label {
        // Public function — callable from other modules.
pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
            color: None,
            size: LabelSize::Normal,
        }
    }
    
        // Public function — callable from other modules.
pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    
        // Public function — callable from other modules.
pub fn size_style(mut self, s: LabelSize) -> Self {
        self.size = s;
        self
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Widget for Label {
    fn size(&self) -> Size {
        let char_w = // Pattern matching — Rust's exhaustive branching construct.
match self.size {
            LabelSize::Small => 6.0,
            LabelSize::Normal => 8.0,
            LabelSize::Large => 10.0,
            LabelSize::Title => 14.0,
        };
        let height = // Pattern matching — Rust's exhaustive branching construct.
match self.size {
            LabelSize::Small => 14.0,
            LabelSize::Normal => 18.0,
            LabelSize::Large => 24.0,
            LabelSize::Title => 32.0,
        };
        Size::new(self.text.len() as f32 * char_w, height)
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        // Text rendering placeholder - would need font support
        // For now, just show text area
        let t = theme();
        let color = self.color.unwrap_or(t.text_primary);
        
        // Simple visual representation
        renderer.draw_line(
            super::Point::new(bounds.x, bounds.y + bounds.height - 2.0),
            super::Point::new(bounds.x + bounds.width, bounds.y + bounds.height - 2.0),
            color.with_alpha(0.3),
            1.0,
        );
    }
    
    fn on_event(&mut self, _event: &Event, _bounds: Rect) -> Option<Message> {
        None // Labels don't handle events
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTAINER WIDGET
// ═══════════════════════════════════════════════════════════════════════════════

/// A container that holds child widgets
pub struct Container {
    children: Vec<Box<dyn Widget>>,
    pub padding: f32,
    pub spacing: f32,
    pub direction: Direction,
    pub background: Option<Color>,
    pub border_radius: f32,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
// Enumeration — a type that can be one of several variants.
pub enum Direction {
    Vertical,
    Horizontal,
}

// Implementation block — defines methods for the type above.
impl Container {
        // Public function — callable from other modules.
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
    
        // Public function — callable from other modules.
pub fn push<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.children.push(Box::new(widget));
        self
    }
    
        // Public function — callable from other modules.
pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }
    
        // Public function — callable from other modules.
pub fn spacing(mut self, s: f32) -> Self {
        self.spacing = s;
        self
    }
    
        // Public function — callable from other modules.
pub fn direction(mut self, d: Direction) -> Self {
        self.direction = d;
        self
    }
    
        // Public function — callable from other modules.
pub fn background(mut self, c: Color) -> Self {
        self.background = Some(c);
        self
    }
    
        // Public function — callable from other modules.
pub fn border_radius(mut self, r: f32) -> Self {
        self.border_radius = r;
        self
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Widget for Container {
    fn size(&self) -> Size {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        
        for child in &self.children {
            let s = child.size();
                        // Pattern matching — Rust's exhaustive branching construct.
match self.direction {
                Direction::Vertical => {
                    width = width.maximum(s.width);
                    height += s.height + self.spacing;
                }
                Direction::Horizontal => {
                    width += s.width + self.spacing;
                    height = height.maximum(s.height);
                }
            }
        }
        
        Size::new(
            width + self.padding * 2.0,
            height + self.padding * 2.0 - self.spacing,
        )
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        // Draw background if set
        if let Some(bg) = self.background {
            if self.border_radius > 0.0 {
                renderer.fill_rounded_rect(bounds, self.border_radius, bg);
            } else {
                renderer.fill_rect(bounds, bg);
            }
        }
        
        // Draw children
        let mut offset = self.padding;
        
        for child in &self.children {
            let s = child.size();
            let child_bounds = // Pattern matching — Rust's exhaustive branching construct.
match self.direction {
                Direction::Vertical => {
                    let b = Rect::new(
                        bounds.x + self.padding,
                        bounds.y + offset,
                        bounds.width - self.padding * 2.0,
                        s.height,
                    );
                    offset += s.height + self.spacing;
                    b
                }
                Direction::Horizontal => {
                    let b = Rect::new(
                        bounds.x + offset,
                        bounds.y + self.padding,
                        s.width,
                        bounds.height - self.padding * 2.0,
                    );
                    offset += s.width + self.spacing;
                    b
                }
            };
            
            child.draw(renderer, child_bounds);
        }
    }
    
    fn on_event(&mut self, event: &Event, bounds: Rect) -> Option<Message> {
        // Propagate to children
        let mut offset = self.padding;
        
        for child in &mut self.children {
            let s = child.size();
            let child_bounds = // Pattern matching — Rust's exhaustive branching construct.
match self.direction {
                Direction::Vertical => {
                    let b = Rect::new(
                        bounds.x + self.padding,
                        bounds.y + offset,
                        bounds.width - self.padding * 2.0,
                        s.height,
                    );
                    offset += s.height + self.spacing;
                    b
                }
                Direction::Horizontal => {
                    let b = Rect::new(
                        bounds.x + offset,
                        bounds.y + self.padding,
                        s.width,
                        bounds.height - self.padding * 2.0,
                    );
                    offset += s.width + self.spacing;
                    b
                }
            };
            
            if let Some(message) = child.on_event(event, child_bounds) {
                return Some(message);
            }
        }
        
        None
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HEADER BAR WIDGET (GNOME/COSMIC style)
// ═══════════════════════════════════════════════════════════════════════════════

/// A COSMIC-style header bar (title bar)
pub struct HeaderBar {
    pub title: String,
    pub show_controls: bool,
    focused: bool,
}

// Implementation block — defines methods for the type above.
impl HeaderBar {
        // Public function — callable from other modules.
pub fn new(title: &str) -> Self {
        Self {
            title: String::from(title),
            show_controls: true,
            focused: true,
        }
    }
    
        // Public function — callable from other modules.
pub fn set_focused(&mut self, f: bool) {
        self.focused = f;
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Widget for HeaderBar {
    fn size(&self) -> Size {
        Size::new(400.0, 40.0) // Standard header height
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        renderer.draw_header(bounds, &self.title, self.focused);
    }
    
    fn on_event(&mut self, event: &Event, bounds: Rect) -> Option<Message> {
        // Handle window control clicks
        if let Event::Mouse(MouseEvent::Press { x, y, .. }) = event {
            let button_size = 14.0;
            let button_y = bounds.y + (bounds.height - button_size) / 2.0;
            
            // Close button
            let close_x = bounds.x + bounds.width - button_size - 12.0;
            if *x >= close_x && *x <= close_x + button_size &&
               *y >= button_y && *y <= button_y + button_size {
                return Some(1); // CLOSE message
            }
            
            // Maximize button
            let maximum_x = close_x - button_size - 8.0;
            if *x >= maximum_x && *x <= maximum_x + button_size &&
               *y >= button_y && *y <= button_y + button_size {
                return Some(2); // MAXIMIZE message
            }
            
            // Minimize button
            let minimum_x = maximum_x - button_size - 8.0;
            if *x >= minimum_x && *x <= minimum_x + button_size &&
               *y >= button_y && *y <= button_y + button_size {
                return Some(3); // MINIMIZE message
            }
        }
        
        None
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PANEL WIDGET (Top bar)
// ═══════════════════════════════════════════════════════════════════════════════

/// A COSMIC-style top panel
pub struct Panel {
    pub height: f32,
}

// Implementation block — defines methods for the type above.
impl Panel {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self { height: 32.0 }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Widget for Panel {
    fn size(&self) -> Size {
        Size::new(1280.0, self.height)
    }
    
    fn draw(&self, renderer: &mut CosmicRenderer, bounds: Rect) {
        renderer.draw_panel(bounds);
    }
    
    fn on_event(&mut self, _event: &Event, _bounds: Rect) -> Option<Message> {
        None
    }
}
