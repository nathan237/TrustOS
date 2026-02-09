//! CSS Parser
//!
//! Parses CSS stylesheets and inline styles.
//! Based on the CSS 2.1 specification with modern extensions.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Parsed CSS Stylesheet
#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

/// A CSS rule (selector + declarations)
#[derive(Debug, Clone)]
pub struct CssRule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

/// CSS Selector
#[derive(Debug, Clone)]
pub struct Selector {
    pub elements: Vec<SelectorPart>,
}

/// Part of a selector
#[derive(Debug, Clone)]
pub enum SelectorPart {
    Tag(String),           // div, p, span
    Class(String),         // .class
    Id(String),            // #id
    Universal,             // *
    Descendant,            // space
    Child,                 // >
    Adjacent,              // +
    Sibling,               // ~
    Pseudo(String),        // :hover, :first-child
    Attribute(String, Option<String>), // [attr], [attr="value"]
}

/// CSS Declaration (property: value)
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: CssValue,
    pub important: bool,
}

/// CSS Value types
#[derive(Debug, Clone)]
pub enum CssValue {
    Keyword(String),              // auto, none, block
    Color(u32),                   // ARGB color
    Length(f32, LengthUnit),      // 16px, 1.5em, 50%
    Number(f32),                  // 1.5
    String(String),               // "Helvetica"
    Url(String),                  // url(...)
    Multiple(Vec<CssValue>),      // margin: 10px 20px
}

/// CSS Length units
#[derive(Debug, Clone, Copy)]
pub enum LengthUnit {
    Px,     // pixels
    Em,     // relative to font-size
    Rem,    // relative to root font-size
    Percent,
    Vw,     // viewport width
    Vh,     // viewport height
    Pt,     // points
}

/// Computed styles for an element
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: u32,
    pub background_color: u32,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub text_decoration: TextDecoration,
    pub text_align: TextAlign,
    pub margin: EdgeSizes,
    pub padding: EdgeSizes,
    pub border_width: EdgeSizes,
    pub border_color: u32,
    pub border_radius: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub line_height: f32,
}

/// Display modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
    None,
    Flex,
    Grid,
}

/// Font weight
#[derive(Debug, Clone, Copy)]
pub enum FontWeight {
    Normal,
    Bold,
    Numeric(u16), // 100-900
}

/// Font style
#[derive(Debug, Clone, Copy)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Text decoration
#[derive(Debug, Clone, Copy)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
}

/// Text alignment
#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

/// Edge sizes (for margin/padding/border)
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Block,
            color: 0xFF000000,           // Black
            background_color: 0x00000000, // Transparent
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            text_decoration: TextDecoration::None,
            text_align: TextAlign::Left,
            margin: EdgeSizes::default(),
            padding: EdgeSizes::default(),
            border_width: EdgeSizes::default(),
            border_color: 0xFF000000,
            border_radius: 0.0,
            width: None,
            height: None,
            line_height: 1.2,
        }
    }
}

/// Parse a CSS stylesheet
pub fn parse_stylesheet(css: &str) -> Stylesheet {
    let mut parser = CssParser::new(css);
    parser.parse_stylesheet()
}

/// Parse inline style attribute
pub fn parse_inline_style(style: &str) -> Vec<Declaration> {
    let mut parser = CssParser::new(style);
    parser.parse_declarations()
}

/// Parse a color value
pub fn parse_color(value: &str) -> Option<u32> {
    let value = value.trim().to_lowercase();
    
    // Named colors
    match value.as_str() {
        "black" => return Some(0xFF000000),
        "white" => return Some(0xFFFFFFFF),
        "red" => return Some(0xFFFF0000),
        "green" => return Some(0xFF00FF00),
        "blue" => return Some(0xFF0000FF),
        "yellow" => return Some(0xFFFFFF00),
        "cyan" | "aqua" => return Some(0xFF00FFFF),
        "magenta" | "fuchsia" => return Some(0xFFFF00FF),
        "gray" | "grey" => return Some(0xFF808080),
        "silver" => return Some(0xFFC0C0C0),
        "maroon" => return Some(0xFF800000),
        "olive" => return Some(0xFF808000),
        "navy" => return Some(0xFF000080),
        "teal" => return Some(0xFF008080),
        "purple" => return Some(0xFF800080),
        "orange" => return Some(0xFFFFA500),
        "pink" => return Some(0xFFFFC0CB),
        "transparent" => return Some(0x00000000),
        _ => {}
    }
    
    // Hex colors
    if value.starts_with('#') {
        let hex = &value[1..];
        return match hex.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(0xFF000000 | ((r as u32 * 17) << 16) | ((g as u32 * 17) << 8) | (b as u32 * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
            }
            8 => {
                // #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
            }
            _ => None,
        };
    }
    
    // rgb() / rgba()
    if value.starts_with("rgb") {
        let start = value.find('(')?;
        let end = value.find(')')?;
        let inner = &value[start + 1..end];
        let parts: Vec<&str> = inner.split(',').collect();
        
        if parts.len() >= 3 {
            let r: u8 = parts[0].trim().parse().ok()?;
            let g: u8 = parts[1].trim().parse().ok()?;
            let b: u8 = parts[2].trim().parse().ok()?;
            let a: u8 = if parts.len() >= 4 {
                let alpha: f32 = parts[3].trim().parse().ok()?;
                (alpha * 255.0) as u8
            } else {
                255
            };
            return Some(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32));
        }
    }
    
    None
}

/// CSS Parser
struct CssParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> CssParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
    
    fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut rules = Vec::new();
        
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.input.len() {
                break;
            }
            
            // Skip @-rules for now
            if self.current_char() == Some('@') {
                self.skip_at_rule();
                continue;
            }
            
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            }
        }
        
        Stylesheet { rules }
    }
    
    fn parse_rule(&mut self) -> Option<CssRule> {
        let selectors = self.parse_selectors();
        if selectors.is_empty() {
            return None;
        }
        
        self.skip_whitespace();
        if !self.consume_char('{') {
            return None;
        }
        
        let declarations = self.parse_declarations();
        
        self.skip_whitespace();
        self.consume_char('}');
        
        Some(CssRule { selectors, declarations })
    }
    
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        
        loop {
            self.skip_whitespace();
            if let Some(selector) = self.parse_selector() {
                selectors.push(selector);
            }
            
            self.skip_whitespace();
            if !self.consume_char(',') {
                break;
            }
        }
        
        selectors
    }
    
    fn parse_selector(&mut self) -> Option<Selector> {
        let mut elements = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            match self.current_char()? {
                '{' | ',' => break,
                '*' => {
                    self.pos += 1;
                    elements.push(SelectorPart::Universal);
                }
                '.' => {
                    self.pos += 1;
                    let class = self.parse_identifier();
                    elements.push(SelectorPart::Class(class));
                }
                '#' => {
                    self.pos += 1;
                    let id = self.parse_identifier();
                    elements.push(SelectorPart::Id(id));
                }
                ':' => {
                    self.pos += 1;
                    if self.current_char() == Some(':') {
                        self.pos += 1; // ::pseudo-element
                    }
                    let pseudo = self.parse_identifier();
                    elements.push(SelectorPart::Pseudo(pseudo));
                }
                '[' => {
                    self.pos += 1;
                    let attr = self.parse_attribute_selector();
                    elements.push(attr);
                }
                '>' => {
                    self.pos += 1;
                    elements.push(SelectorPart::Child);
                }
                '+' => {
                    self.pos += 1;
                    elements.push(SelectorPart::Adjacent);
                }
                '~' => {
                    self.pos += 1;
                    elements.push(SelectorPart::Sibling);
                }
                c if c.is_alphabetic() || c == '-' || c == '_' => {
                    let tag = self.parse_identifier();
                    elements.push(SelectorPart::Tag(tag));
                }
                _ => break,
            }
        }
        
        if elements.is_empty() {
            None
        } else {
            Some(Selector { elements })
        }
    }
    
    fn parse_attribute_selector(&mut self) -> SelectorPart {
        let attr_name = self.parse_identifier();
        self.skip_whitespace();
        
        if self.consume_char(']') {
            return SelectorPart::Attribute(attr_name, None);
        }
        
        // Skip operator (=, ~=, ^=, etc.)
        while let Some(c) = self.current_char() {
            if c == '"' || c == '\'' || c.is_alphanumeric() {
                break;
            }
            self.pos += 1;
        }
        
        let value = self.parse_string_or_ident();
        self.skip_whitespace();
        self.consume_char(']');
        
        SelectorPart::Attribute(attr_name, Some(value))
    }
    
    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            if self.current_char() == Some('}') || self.pos >= self.input.len() {
                break;
            }
            
            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            }
            
            self.skip_whitespace();
            self.consume_char(';');
        }
        
        declarations
    }
    
    fn parse_declaration(&mut self) -> Option<Declaration> {
        self.skip_whitespace();
        
        let property = self.parse_identifier();
        if property.is_empty() {
            return None;
        }
        
        self.skip_whitespace();
        if !self.consume_char(':') {
            return None;
        }
        
        self.skip_whitespace();
        let (value, important) = self.parse_value();
        
        Some(Declaration { property, value, important })
    }
    
    fn parse_value(&mut self) -> (CssValue, bool) {
        self.skip_whitespace();
        
        let mut values = Vec::new();
        let mut important = false;
        
        loop {
            self.skip_whitespace();
            
            match self.current_char() {
                None | Some(';') | Some('}') => break,
                Some('!') => {
                    self.pos += 1;
                    let word = self.parse_identifier();
                    if word == "important" {
                        important = true;
                    }
                    break;
                }
                Some(_) => {
                    if let Some(v) = self.parse_single_value() {
                        values.push(v);
                    } else {
                        break;
                    }
                }
            }
        }
        
        let value = if values.len() == 1 {
            values.into_iter().next().unwrap()
        } else {
            CssValue::Multiple(values)
        };
        
        (value, important)
    }
    
    fn parse_single_value(&mut self) -> Option<CssValue> {
        self.skip_whitespace();
        
        match self.current_char()? {
            '#' => {
                // Color
                let start = self.pos;
                self.pos += 1;
                while let Some(c) = self.current_char() {
                    if c.is_ascii_hexdigit() {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let color_str = &self.input[start..self.pos];
                if let Some(color) = parse_color(color_str) {
                    Some(CssValue::Color(color))
                } else {
                    None
                }
            }
            '"' | '\'' => {
                // String
                let s = self.parse_quoted_string();
                Some(CssValue::String(s))
            }
            c if c.is_ascii_digit() || c == '-' || c == '.' => {
                // Number or length
                let (num, unit) = self.parse_number_with_unit();
                if let Some(unit) = unit {
                    Some(CssValue::Length(num, unit))
                } else {
                    Some(CssValue::Number(num))
                }
            }
            c if c.is_alphabetic() => {
                let start = self.pos;
                let word = self.parse_identifier();
                
                // Check for url()
                if word == "url" && self.consume_char('(') {
                    self.skip_whitespace();
                    let url = if self.current_char() == Some('"') || self.current_char() == Some('\'') {
                        self.parse_quoted_string()
                    } else {
                        let start = self.pos;
                        while let Some(c) = self.current_char() {
                            if c == ')' { break; }
                            self.pos += 1;
                        }
                        self.input[start..self.pos].to_string()
                    };
                    self.skip_whitespace();
                    self.consume_char(')');
                    return Some(CssValue::Url(url));
                }
                
                // Check for color keywords
                if let Some(color) = parse_color(&word) {
                    return Some(CssValue::Color(color));
                }
                
                // Check for rgb/rgba
                if (word == "rgb" || word == "rgba") && self.consume_char('(') {
                    let func_start = start;
                    while let Some(c) = self.current_char() {
                        if c == ')' { break; }
                        self.pos += 1;
                    }
                    self.consume_char(')');
                    let func_str = &self.input[func_start..self.pos];
                    if let Some(color) = parse_color(func_str) {
                        return Some(CssValue::Color(color));
                    }
                }
                
                Some(CssValue::Keyword(word))
            }
            _ => None,
        }
    }
    
    fn parse_number_with_unit(&mut self) -> (f32, Option<LengthUnit>) {
        let start = self.pos;
        
        // Parse sign
        if self.current_char() == Some('-') {
            self.pos += 1;
        }
        
        // Parse digits and decimal
        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() || c == '.' {
                self.pos += 1;
            } else {
                break;
            }
        }
        
        let num_str = &self.input[start..self.pos];
        let num: f32 = num_str.parse().unwrap_or(0.0);
        
        // Parse unit
        let unit_start = self.pos;
        while let Some(c) = self.current_char() {
            if c.is_alphabetic() || c == '%' {
                self.pos += 1;
            } else {
                break;
            }
        }
        
        let unit_str = &self.input[unit_start..self.pos];
        let unit = match unit_str {
            "px" => Some(LengthUnit::Px),
            "em" => Some(LengthUnit::Em),
            "rem" => Some(LengthUnit::Rem),
            "%" => Some(LengthUnit::Percent),
            "vw" => Some(LengthUnit::Vw),
            "vh" => Some(LengthUnit::Vh),
            "pt" => Some(LengthUnit::Pt),
            "" => None,
            _ => None,
        };
        
        (num, unit)
    }
    
    fn parse_identifier(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.current_char() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }
    
    fn parse_quoted_string(&mut self) -> String {
        let quote = self.current_char().unwrap_or('"');
        self.pos += 1;
        
        let start = self.pos;
        while let Some(c) = self.current_char() {
            if c == quote {
                break;
            }
            if c == '\\' {
                self.pos += 2; // Skip escape
            } else {
                self.pos += 1;
            }
        }
        let s = self.input[start..self.pos].to_string();
        self.consume_char(quote);
        s
    }
    
    fn parse_string_or_ident(&mut self) -> String {
        match self.current_char() {
            Some('"') | Some('\'') => self.parse_quoted_string(),
            _ => self.parse_identifier(),
        }
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            
            if self.starts_with("/*") {
                // C-style comment
                self.pos += 2;
                while self.pos < self.input.len() {
                    if self.starts_with("*/") {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }
    
    fn skip_at_rule(&mut self) {
        // Skip @import, @media, etc.
        while let Some(c) = self.current_char() {
            if c == ';' {
                self.pos += 1;
                break;
            }
            if c == '{' {
                // Skip block
                let mut depth = 1;
                self.pos += 1;
                while self.pos < self.input.len() && depth > 0 {
                    match self.input.chars().nth(self.pos) {
                        Some('{') => depth += 1,
                        Some('}') => depth -= 1,
                        _ => {}
                    }
                    self.pos += 1;
                }
                break;
            }
            self.pos += 1;
        }
    }
    
    fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }
    
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }
    
    fn consume_char(&mut self, expected: char) -> bool {
        if self.current_char() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("#fff"), Some(0xFFFFFFFF));
        assert_eq!(parse_color("#000"), Some(0xFF000000));
        assert_eq!(parse_color("#ff0000"), Some(0xFFFF0000));
        assert_eq!(parse_color("red"), Some(0xFFFF0000));
        assert_eq!(parse_color("transparent"), Some(0x00000000));
    }
}
