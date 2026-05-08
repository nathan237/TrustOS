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
    pub max_width: Option<f32>,
    pub min_width: Option<f32>,
    pub line_height: f32,
    pub opacity: f32,
    pub overflow: Overflow,
    pub position: Position,
    pub white_space: WhiteSpace,
    pub word_break: WordBreak,
    pub cursor: CursorStyle,
    pub box_shadow: Option<BoxShadow>,
    pub text_transform: TextTransform,
    pub letter_spacing: f32,
    pub list_style_type: ListStyleType,
}

/// Overflow behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// Position property
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// White-space handling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
}

/// Word break
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

/// Cursor style
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    Default,
    Pointer,
    Text,
    Move,
    NotAllowed,
    Crosshair,
    Wait,
}

/// Box shadow
#[derive(Debug, Clone, Copy)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: u32,
}

/// Text transform
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

/// List style type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListStyleType {
    Disc,
    Circle,
    Square,
    Decimal,
    None,
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
            color: 0xFF000000,
            background_color: 0x00000000,
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
            max_width: None,
            min_width: None,
            line_height: 1.2,
            opacity: 1.0,
            overflow: Overflow::Visible,
            position: Position::Static,
            white_space: WhiteSpace::Normal,
            word_break: WordBreak::Normal,
            cursor: CursorStyle::Default,
            box_shadow: None,
            text_transform: TextTransform::None,
            letter_spacing: 0.0,
            list_style_type: ListStyleType::Disc,
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

/// Parse a color value — full CSS3 named color palette + hex + rgb/rgba + hsl/hsla
pub fn parse_color(value: &str) -> Option<u32> {
    let value = value.trim().to_lowercase();
    
    // Full CSS3 named colors (147 colors)
    match value.as_str() {
        "aliceblue" => return Some(0xFFF0F8FF),
        "antiquewhite" => return Some(0xFFFAEBD7),
        "aqua" => return Some(0xFF00FFFF),
        "aquamarine" => return Some(0xFF7FFFD4),
        "azure" => return Some(0xFFF0FFFF),
        "beige" => return Some(0xFFF5F5DC),
        "bisque" => return Some(0xFFFFE4C4),
        "black" => return Some(0xFF000000),
        "blanchedalmond" => return Some(0xFFFFEBCD),
        "blue" => return Some(0xFF0000FF),
        "blueviolet" => return Some(0xFF8A2BE2),
        "brown" => return Some(0xFFA52A2A),
        "burlywood" => return Some(0xFFDEB887),
        "cadetblue" => return Some(0xFF5F9EA0),
        "chartreuse" => return Some(0xFF7FFF00),
        "chocolate" => return Some(0xFFD2691E),
        "coral" => return Some(0xFFFF7F50),
        "cornflowerblue" => return Some(0xFF6495ED),
        "cornsilk" => return Some(0xFFFFF8DC),
        "crimson" => return Some(0xFFDC143C),
        "cyan" => return Some(0xFF00FFFF),
        "darkblue" => return Some(0xFF00008B),
        "darkcyan" => return Some(0xFF008B8B),
        "darkgoldenrod" => return Some(0xFFB8860B),
        "darkgray" | "darkgrey" => return Some(0xFFA9A9A9),
        "darkgreen" => return Some(0xFF006400),
        "darkkhaki" => return Some(0xFFBDB76B),
        "darkmagenta" => return Some(0xFF8B008B),
        "darkolivegreen" => return Some(0xFF556B2F),
        "darkorange" => return Some(0xFFFF8C00),
        "darkorchid" => return Some(0xFF9932CC),
        "darkred" => return Some(0xFF8B0000),
        "darksalmon" => return Some(0xFFE9967A),
        "darkseagreen" => return Some(0xFF8FBC8F),
        "darkslateblue" => return Some(0xFF483D8B),
        "darkslategray" | "darkslategrey" => return Some(0xFF2F4F4F),
        "darkturquoise" => return Some(0xFF00CED1),
        "darkviolet" => return Some(0xFF9400D3),
        "deeppink" => return Some(0xFFFF1493),
        "deepskyblue" => return Some(0xFF00BFFF),
        "dimgray" | "dimgrey" => return Some(0xFF696969),
        "dodgerblue" => return Some(0xFF1E90FF),
        "firebrick" => return Some(0xFFB22222),
        "floralwhite" => return Some(0xFFFFFAF0),
        "forestgreen" => return Some(0xFF228B22),
        "fuchsia" => return Some(0xFFFF00FF),
        "gainsboro" => return Some(0xFFDCDCDC),
        "ghostwhite" => return Some(0xFFF8F8FF),
        "gold" => return Some(0xFFFFD700),
        "goldenrod" => return Some(0xFFDAA520),
        "gray" | "grey" => return Some(0xFF808080),
        "green" => return Some(0xFF008000),
        "greenyellow" => return Some(0xFFADFF2F),
        "honeydew" => return Some(0xFFF0FFF0),
        "hotpink" => return Some(0xFFFF69B4),
        "indianred" => return Some(0xFFCD5C5C),
        "indigo" => return Some(0xFF4B0082),
        "ivory" => return Some(0xFFFFFFF0),
        "khaki" => return Some(0xFFF0E68C),
        "lavender" => return Some(0xFFE6E6FA),
        "lavenderblush" => return Some(0xFFFFF0F5),
        "lawngreen" => return Some(0xFF7CFC00),
        "lemonchiffon" => return Some(0xFFFFFACD),
        "lightblue" => return Some(0xFFADD8E6),
        "lightcoral" => return Some(0xFFF08080),
        "lightcyan" => return Some(0xFFE0FFFF),
        "lightgoldenrodyellow" => return Some(0xFFFAFAD2),
        "lightgray" | "lightgrey" => return Some(0xFFD3D3D3),
        "lightgreen" => return Some(0xFF90EE90),
        "lightpink" => return Some(0xFFFFB6C1),
        "lightsalmon" => return Some(0xFFFFA07A),
        "lightseagreen" => return Some(0xFF20B2AA),
        "lightskyblue" => return Some(0xFF87CEFA),
        "lightslategray" | "lightslategrey" => return Some(0xFF778899),
        "lightsteelblue" => return Some(0xFFB0C4DE),
        "lightyellow" => return Some(0xFFFFFFE0),
        "lime" => return Some(0xFF00FF00),
        "limegreen" => return Some(0xFF32CD32),
        "linen" => return Some(0xFFFAF0E6),
        "magenta" => return Some(0xFFFF00FF),
        "maroon" => return Some(0xFF800000),
        "mediumaquamarine" => return Some(0xFF66CDAA),
        "mediumblue" => return Some(0xFF0000CD),
        "mediumorchid" => return Some(0xFFBA55D3),
        "mediumpurple" => return Some(0xFF9370DB),
        "mediumseagreen" => return Some(0xFF3CB371),
        "mediumslateblue" => return Some(0xFF7B68EE),
        "mediumspringgreen" => return Some(0xFF00FA9A),
        "mediumturquoise" => return Some(0xFF48D1CC),
        "mediumvioletred" => return Some(0xFFC71585),
        "midnightblue" => return Some(0xFF191970),
        "mintcream" => return Some(0xFFF5FFFA),
        "mistyrose" => return Some(0xFFFFE4E1),
        "moccasin" => return Some(0xFFFFE4B5),
        "navajowhite" => return Some(0xFFFFDEAD),
        "navy" => return Some(0xFF000080),
        "oldlace" => return Some(0xFFFDF5E6),
        "olive" => return Some(0xFF808000),
        "olivedrab" => return Some(0xFF6B8E23),
        "orange" => return Some(0xFFFFA500),
        "orangered" => return Some(0xFFFF4500),
        "orchid" => return Some(0xFFDA70D6),
        "palegoldenrod" => return Some(0xFFEEE8AA),
        "palegreen" => return Some(0xFF98FB98),
        "paleturquoise" => return Some(0xFFAFEEEE),
        "palevioletred" => return Some(0xFFDB7093),
        "papayawhip" => return Some(0xFFFFEFD5),
        "peachpuff" => return Some(0xFFFFDAB9),
        "peru" => return Some(0xFFCD853F),
        "pink" => return Some(0xFFFFC0CB),
        "plum" => return Some(0xFFDDA0DD),
        "powderblue" => return Some(0xFFB0E0E6),
        "purple" => return Some(0xFF800080),
        "rebeccapurple" => return Some(0xFF663399),
        "red" => return Some(0xFFFF0000),
        "rosybrown" => return Some(0xFFBC8F8F),
        "royalblue" => return Some(0xFF4169E1),
        "saddlebrown" => return Some(0xFF8B4513),
        "salmon" => return Some(0xFFFA8072),
        "sandybrown" => return Some(0xFFF4A460),
        "seagreen" => return Some(0xFF2E8B57),
        "seashell" => return Some(0xFFFFF5EE),
        "sienna" => return Some(0xFFA0522D),
        "silver" => return Some(0xFFC0C0C0),
        "skyblue" => return Some(0xFF87CEEB),
        "slateblue" => return Some(0xFF6A5ACD),
        "slategray" | "slategrey" => return Some(0xFF708090),
        "snow" => return Some(0xFFFFFAFA),
        "springgreen" => return Some(0xFF00FF7F),
        "steelblue" => return Some(0xFF4682B4),
        "tan" => return Some(0xFFD2B48C),
        "teal" => return Some(0xFF008080),
        "thistle" => return Some(0xFFD8BFD8),
        "tomato" => return Some(0xFFFF6347),
        "turquoise" => return Some(0xFF40E0D0),
        "violet" => return Some(0xFFEE82EE),
        "wheat" => return Some(0xFFF5DEB3),
        "white" => return Some(0xFFFFFFFF),
        "whitesmoke" => return Some(0xFFF5F5F5),
        "yellow" => return Some(0xFFFFFF00),
        "yellowgreen" => return Some(0xFF9ACD32),
        "transparent" => return Some(0x00000000),
        _ => {}
    }
    
    // Hex colors
    if value.starts_with('#') {
        let hex = &value[1..];
        return match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(0xFF000000 | ((r as u32 * 17) << 16) | ((g as u32 * 17) << 8) | (b as u32 * 17))
            }
            4 => {
                // #RGBA
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
                Some(((a as u32 * 17) << 24) | ((r as u32 * 17) << 16) | ((g as u32 * 17) << 8) | (b as u32 * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
            }
            8 => {
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
    
    // hsl() / hsla()
    if value.starts_with("hsl") {
        let start = value.find('(')?;
        let end = value.find(')')?;
        let inner = &value[start + 1..end];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let h: f32 = parts[0].trim().trim_end_matches("deg").parse::<f32>().ok()?;
            let s: f32 = parts[1].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;
            let l: f32 = parts[2].trim().trim_end_matches('%').parse::<f32>().ok()? / 100.0;
            let a: f32 = if parts.len() >= 4 {
                parts[3].trim().parse::<f32>().ok()?
            } else {
                1.0
            };
            let (r, g, b) = hsl_to_rgb(h, s, l);
            let alpha = (a * 255.0) as u8;
            return Some(((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32));
        }
    }
    
    None
}

/// Convert HSL to RGB
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }
    let hue = ((h % 360.0) + 360.0) % 360.0 / 360.0;
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, hue + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, hue);
    let b = hue_to_rgb(p, q, hue - 1.0 / 3.0);
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
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
            // SAFETY: just checked len() == 1
            values.into_iter().next().unwrap_or(CssValue::Keyword(String::new()))
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
