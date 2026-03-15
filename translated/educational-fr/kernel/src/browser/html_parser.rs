//! HTML Parser
//!
//! Simple HTML parser that extracts structure and text.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;

/// HTML Document
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct HtmlDocument {
    pub title: String,
    pub nodes: Vec<HtmlNode>,
}

/// HTML Node types
#[derive(Debug, Clone)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum HtmlNode {
    Text(String),
    Element(HtmlElement),
}

/// HTML Element
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct HtmlElement {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<HtmlNode>,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl HtmlElement {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_lowercase(),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }
    
    /// Get attribute value
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes.iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Parse HTML string into document
pub fn parse_html(html: &str) -> HtmlDocument {
    let mut doc = HtmlDocument {
        title: String::new(),
        nodes: Vec::new(),
    };
    
    let mut parser = HtmlParser::new(html);
    doc.nodes = parser.parse_nodes();
    
    // Extract title
    doc.title = find_title(&doc.nodes).unwrap_or_else(|| "Untitled".to_string());
    
    doc
}

/// Find <title> content
fn find_title(nodes: &[HtmlNode]) -> Option<String> {
    for node in nodes {
        if let HtmlNode::Element(el) = node {
            if el.tag == "title" {
                return Some(get_text_content(&el.children));
            }
            if let Some(title) = find_title(&el.children) {
                return Some(title);
            }
        }
    }
    None
}

/// Get text content of nodes
fn get_text_content(nodes: &[HtmlNode]) -> String {
    let mut text = String::new();
    for node in nodes {
                // Correspondance de motifs — branchement exhaustif de Rust.
match node {
            HtmlNode::Text(t) => text.push_str(t),
            HtmlNode::Element(el) => text.push_str(&get_text_content(&el.children)),
        }
    }
    text
}

/// HTML Parser
struct HtmlParser<'a> {
    input: &'a str,
    position: usize,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl<'a> HtmlParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }
    
    fn parse_nodes(&mut self) -> Vec<HtmlNode> {
        let mut nodes = Vec::new();
        
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.starts_with("<!--") {
                self.skip_comment();
            } else if self.starts_with("<!") {
                self.skip_doctype();
            } else if self.starts_with("</") {
                // End tag - return to parent
                break;
            } else if self.starts_with("<") {
                if let Some(el) = self.parse_element() {
                    nodes.push(HtmlNode::Element(el));
                }
            } else {
                if let Some(text) = self.parse_text() {
                    if !text.trim().is_empty() {
                        nodes.push(HtmlNode::Text(text));
                    }
                }
            }
        }
        
        nodes
    }
    
    fn parse_element(&mut self) -> Option<HtmlElement> {
        if !self.consume("<") {
            return None;
        }
        
        let tag = self.parse_tag_name();
        if tag.is_empty() {
            return None;
        }
        
        let mut element = HtmlElement::new(&tag);
        
        // Parse attributes
        loop {
            self.skip_whitespace();
            
            if self.starts_with("/>") {
                self.consume("/>");
                return Some(element);
            }
            
            if self.starts_with(">") {
                self.consume(">");
                break;
            }
            
            if let Some((name, value)) = self.parse_attribute() {
                element.attributes.push((name, value));
            } else {
                break;
            }
        }
        
        // Self-closing tags
        let self_closing = matches!(tag.as_str(), 
            "br" | "hr" | "img" | "input" | "meta" | "link" | "area" | "base" | 
            "col" | "embed" | "param" | "source" | "track" | "wbr"
        );
        
        if !self_closing {
            // Parse children
            element.children = self.parse_nodes();
            
            // Consume end tag
            self.skip_whitespace();
            if self.starts_with("</") {
                self.consume("</");
                self.parse_tag_name();
                self.skip_whitespace();
                self.consume(">");
            }
        }
        
        Some(element)
    }
    
    fn parse_tag_name(&mut self) -> String {
        let start = self.position;
        while self.position < self.input.len() {
            let c = self.current_char();
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ':' {
                self.position += 1;
            } else {
                break;
            }
        }
        self.input[start..self.position].to_lowercase()
    }
    
    fn parse_attribute(&mut self) -> Option<(String, String)> {
        let name_start = self.position;
        while self.position < self.input.len() {
            let c = self.current_char();
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ':' {
                self.position += 1;
            } else {
                break;
            }
        }
        
        let name = self.input[name_start..self.position].to_lowercase();
        if name.is_empty() {
            return None;
        }
        
        self.skip_whitespace();
        
        let value = if self.consume("=") {
            self.skip_whitespace();
            self.parse_attribute_value()
        } else {
            String::new()
        };
        
        Some((name, value))
    }
    
    fn parse_attribute_value(&mut self) -> String {
        if self.starts_with("\"") {
            self.consume("\"");
            let value = self.consume_until('"');
            self.consume("\"");
            value
        } else if self.starts_with("'") {
            self.consume("'");
            let value = self.consume_until('\'');
            self.consume("'");
            value
        } else {
            // Unquoted value
            let start = self.position;
            while self.position < self.input.len() {
                let c = self.current_char();
                if c.is_whitespace() || c == '>' || c == '/' {
                    break;
                }
                self.position += 1;
            }
            self.input[start..self.position].to_string()
        }
    }
    
    fn parse_text(&mut self) -> Option<String> {
        let start = self.position;
        while self.position < self.input.len() && !self.starts_with("<") {
            self.position += 1;
        }
        if self.position > start {
            Some(decode_entities(&self.input[start..self.position]))
        } else {
            None
        }
    }
    
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.current_char().is_whitespace() {
            self.position += 1;
        }
    }
    
    fn skip_comment(&mut self) {
        self.consume("<!--");
        while self.position < self.input.len() && !self.starts_with("-->") {
            self.position += 1;
        }
        self.consume("-->");
    }
    
    fn skip_doctype(&mut self) {
        while self.position < self.input.len() && !self.starts_with(">") {
            self.position += 1;
        }
        self.consume(">");
    }
    
    fn current_char(&self) -> char {
        self.input[self.position..].chars().next().unwrap_or('\0')
    }
    
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }
    
    fn consume(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.position += s.len();
            true
        } else {
            false
        }
    }
    
    fn consume_until(&mut self, c: char) -> String {
        let start = self.position;
        while self.position < self.input.len() && self.current_char() != c {
            self.position += 1;
        }
        self.input[start..self.position].to_string()
    }
}

/// Decode HTML entities
fn decode_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '&' {
            let mut entity = String::new();
            while let Some(&c) = chars.peek() {
                if c == ';' {
                    chars.next();
                    break;
                }
                if c.is_alphanumeric() || c == '#' {
                    entity.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            
                        // Correspondance de motifs — branchement exhaustif de Rust.
match entity.as_str() {
                "amp" => result.push('&'),
                "lt" => result.push('<'),
                "gt" => result.push('>'),
                "quot" => result.push('"'),
                "apos" => result.push('\''),
                "nbsp" => result.push(' '),
                "copy" => result.push('©'),
                "reg" => result.push('®'),
                "mdash" => result.push('—'),
                "ndash" => result.push('–'),
                s if s.starts_with('#') => {
                    if let Some(code) = parse_char_code(&s[1..]) {
                        if let Some(c) = char::from_u32(code) {
                            result.push(c);
                        }
                    }
                }
                _ => {
                    result.push('&');
                    result.push_str(&entity);
                    result.push(';');
                }
            }
        } else {
            result.push(c);
        }
    }
    
    result
}

fn parse_char_code(s: &str) -> Option<u32> {
    if s.starts_with('x') || s.starts_with('X') {
        u32::from_str_radix(&s[1..], 16).ok()
    } else {
        s.parse().ok()
    }
}
