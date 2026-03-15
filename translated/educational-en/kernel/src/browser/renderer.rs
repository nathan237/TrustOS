//! HTML Renderer
//!
//! Renders HTML to the framebuffer with basic styling.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;

use super::{HtmlDocument, HtmlNode, HtmlElement, HtmlLink};
use super::css_parser::{self, CssValue, FontWeight, FontStyle as CssFontStyle, Declaration, Stylesheet, Selector, SelectorPart};
use crate::framebuffer;

/// Colors for rendering
const COLOR_BG: u32 = 0xFFFFFFFF;           // White background
const COLOR_TEXT: u32 = 0xFF1A1A1A;         // Dark text
const COLOR_LINK: u32 = 0xFF0066CC;         // Blue links
const COLOR_LINK_VISITED: u32 = 0xFF551A8B; // Purple visited
const COLOR_HEADING: u32 = 0xFF000000;      // Black headings
const COLOR_CODE_BG: u32 = 0xFFF5F5F5;      // Light gray code bg
const COLOR_CODE: u32 = 0xFFD63384;         // Pink code text
const COLOR_HR: u32 = 0xFFCCCCCC;           // Gray horizontal rule
const COLOR_QUOTE_BORDER: u32 = 0xFF0066CC; // Blue quote border
const COLOR_QUOTE_BG: u32 = 0xFFF0F7FF;     // Light blue quote bg

/// Rendering context
pub struct RenderContext {
    pub x: i32,
    pub y: i32,
    pub maximum_width: u32,
    pub line_height: i32,
    pub font_size: FontSize,
    pub bold: bool,
    pub italic: bool,
    pub in_link: Option<String>,
    pub links: Vec<HtmlLink>,
    pub list_depth: i32,
    pub in_pre: bool,
    /// Current text color (overridable by CSS)
    pub text_color: u32,
    /// Current background color (overridable by CSS)
    pub bg_color: Option<u32>,
    /// Opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Text decoration
    pub underline: bool,
    pub strikethrough: bool,
    /// Ordered list counter per depth
    pub list_counters: Vec<i32>,
    /// Is inside <ol> at current depth
    pub in_ordered_list: bool,
}

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum FontSize {
    Small,
    Normal,
    Large,
    H1,
    H2,
    H3,
}

// Implementation block — defines methods for the type above.
impl FontSize {
    fn height(&self) -> i32 {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            FontSize::Small => 12,
            FontSize::Normal => 16,
            FontSize::Large => 18,
            FontSize::H1 => 32,
            FontSize::H2 => 26,
            FontSize::H3 => 20,
        }
    }
}

// Implementation block — defines methods for the type above.
impl RenderContext {
        // Public function — callable from other modules.
pub fn new(width: u32) -> Self {
        Self {
            x: 16,
            y: 16,
            maximum_width: width - 32,
            line_height: 20,
            font_size: FontSize::Normal,
            bold: false,
            italic: false,
            in_link: None,
            links: Vec::new(),
            list_depth: 0,
            in_pre: false,
            text_color: COLOR_TEXT,
            bg_color: None,
            opacity: 1.0,
            underline: false,
            strikethrough: false,
            list_counters: Vec::new(),
            in_ordered_list: false,
        }
    }
    
    fn newline(&mut self) {
        self.x = 16 + (self.list_depth * 24);
        self.y += self.line_height;
    }
    
    fn space(&mut self) {
        self.x += 6;
    }
}

/// Render HTML document to framebuffer
pub fn render_html(
    doc: &HtmlDocument,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scroll_y: i32,
) -> Vec<HtmlLink> {
    // Clear background
    framebuffer::fill_rect(x as u32, y as u32, width, height, COLOR_BG);
    
    let mut context = RenderContext::new(width);
    context.y = y - scroll_y;
    
    // Extract CSS from <style> tags and parse into stylesheet
    let css_text = extract_style_content(&doc.nodes);
    let stylesheet = if !css_text.is_empty() {
        css_parser::parse_stylesheet(&css_text)
    } else {
        Stylesheet { rules: Vec::new() }
    };
    
    // Render all nodes
    for node in &doc.nodes {
        render_node(&mut context, node, &stylesheet, x, y, width, height);
    }
    
    context.links
}

/// Render a single node
fn render_node(
    context: &mut RenderContext,
    node: &HtmlNode,
    stylesheet: &Stylesheet,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
        // Pattern matching — Rust's exhaustive branching construct.
match node {
        HtmlNode::Text(text) => {
            render_text(context, text, clip_x, clip_y, clip_w, clip_h);
        }
        HtmlNode::Element(el) => {
            render_element(context, el, stylesheet, clip_x, clip_y, clip_w, clip_h);
        }
    }
}

/// Render text content
fn render_text(
    context: &mut RenderContext,
    text: &str,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
    let color = if context.in_link.is_some() { COLOR_LINK } else { context.text_color };
    
    // Handle preformatted text
    if context.in_pre {
        for line in text.lines() {
            render_line(context, line, color, clip_x, clip_y, clip_w, clip_h);
            context.newline();
        }
        return;
    }
    
    // Normal text - word wrap
    let words: Vec<&str> = text.split_whitespace().collect();
    
    for word in words {
        let word_width = word.len() as i32 * 8; // Approximate
        
        // Word wrap
        if context.x + word_width > (clip_x + clip_w as i32 - 16) && context.x > 16 + (context.list_depth * 24) {
            context.newline();
        }
        
        render_word(context, word, color, clip_x, clip_y, clip_w, clip_h);
        context.space();
    }
}

fn render_word(
    context: &mut RenderContext,
    word: &str,
    color: u32,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
    // Check if visible
    if context.y + context.line_height < clip_y || context.y > clip_y + clip_h as i32 {
        context.x += word.len() as i32 * 8;
        return;
    }
    
    // Track link bounds
    let link_start_x = context.x;
    
    // Draw each character
    for c in word.chars() {
        if context.x >= clip_x && context.x < clip_x + clip_w as i32 - 8 {
            draw_char(context.x as u32, context.y as u32, c, color);
        }
        context.x += 8;
    }
    
    // Add link if in anchor
    if let Some(href) = &context.in_link {
        context.links.push(HtmlLink {
            href: href.clone(),
            x: link_start_x,
            y: context.y,
            width: (context.x - link_start_x) as u32,
            height: context.line_height as u32,
        });
        
        // Draw underline for links
        if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
            framebuffer::fill_rect(
                link_start_x as u32,
                (context.y + context.line_height - 2) as u32,
                (context.x - link_start_x) as u32,
                1,
                COLOR_LINK,
            );
        }
    }
    
    // Draw underline for <u>/<ins> elements
    if context.underline && context.in_link.is_none() {
        let w = (context.x - link_start_x) as u32;
        if w > 0 && context.y >= clip_y && context.y < clip_y + clip_h as i32 {
            framebuffer::fill_rect(
                link_start_x as u32,
                (context.y + context.line_height - 2) as u32,
                w, 1, color,
            );
        }
    }
    
    // Draw strikethrough for <del>/<s>/<strike>
    if context.strikethrough {
        let w = (context.x - link_start_x) as u32;
        if w > 0 && context.y >= clip_y && context.y < clip_y + clip_h as i32 {
            framebuffer::fill_rect(
                link_start_x as u32,
                (context.y + context.line_height / 2) as u32,
                w, 1, context.text_color,
            );
        }
    }
}

fn render_line(
    context: &mut RenderContext,
    line: &str,
    color: u32,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
    if context.y + context.line_height < clip_y || context.y > clip_y + clip_h as i32 {
        return;
    }
    
    for c in line.chars() {
        if context.x >= clip_x && context.x < clip_x + clip_w as i32 - 8 {
            draw_char(context.x as u32, context.y as u32, c, color);
        }
        context.x += 8;
    }
}

/// Render an HTML element
fn render_element(
    context: &mut RenderContext,
    el: &HtmlElement,
    stylesheet: &Stylesheet,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
    let tag = el.tag.as_str();
    
    // Skip invisible elements
    if matches!(tag, "head" | "script" | "style" | "meta" | "link" | "title") {
        return;
    }
    
    // Save context
    let saved_font = context.font_size;
    let saved_bold = context.bold;
    let saved_link = context.in_link.clone();
    let saved_pre = context.in_pre;
    let saved_text_color = context.text_color;
    let saved_bg_color = context.bg_color;
    let saved_opacity = context.opacity;
    let saved_underline = context.underline;
    let saved_strikethrough = context.strikethrough;
    let saved_ordered = context.in_ordered_list;
    
    // Apply inline CSS styles from style="" attribute
    if let Some(style_str) = el.attribute("style") {
        apply_inline_style(context, style_str);
    }
    
    // Apply stylesheet rules (from <style> tags)
    apply_stylesheet_rules(context, el, stylesheet);
    
    // Apply color from HTML color attribute (legacy support)
    if let Some(color_str) = el.attribute("color") {
        if let Some(c) = parse_html_color(color_str) {
            context.text_color = c;
        }
    }
    if let Some(bgcolor_str) = el.attribute("bgcolor") {
        if let Some(c) = parse_html_color(bgcolor_str) {
            context.bg_color = Some(c);
        }
    }
    
    // Check if element was hidden by display:none
    if context.y >= i32::MAX / 4 {
        context.font_size = saved_font;
        context.line_height = saved_font.height() + 4;
        context.bold = saved_bold;
        context.in_link = saved_link;
        context.in_pre = saved_pre;
        context.text_color = saved_text_color;
        context.bg_color = saved_bg_color;
        context.opacity = saved_opacity;
        context.underline = saved_underline;
        context.strikethrough = saved_strikethrough;
        context.in_ordered_list = saved_ordered;
        return;
    }
    
    // Apply element-specific styling
    match tag {
        // Block elements - newline before
        "html" | "body" | "div" | "section" | "article" | "nav" | "header" | "footer" | "main" | 
        "noscript" | "span" | "form" | "label" | "fieldset" | "legend" | "details" | "summary" |
        "figure" | "figcaption" | "aside" | "dialog" | "abbr" | "address" | "cite" | "dfn" |
        "ruby" | "rt" | "rp" | "data" | "time" | "var" | "samp" | "kbd" | "wbr" | "bdi" | "bdo" => {
            // Container - just render children
        }
        
        "p" => {
            context.newline();
            context.y += 8; // Paragraph spacing
        }
        
        "br" => {
            context.newline();
        }
        
        "hr" => {
            context.newline();
            context.y += 8;
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(
                    (clip_x + 16) as u32,
                    context.y as u32,
                    clip_w - 32,
                    1,
                    COLOR_HR,
                );
            }
            context.y += 16;
            context.x = 16;
        }
        
        "h1" => {
            context.newline();
            context.y += 16;
            context.font_size = FontSize::H1;
            context.line_height = 40;
            context.bold = true;
        }
        
        "h2" => {
            context.newline();
            context.y += 12;
            context.font_size = FontSize::H2;
            context.line_height = 32;
            context.bold = true;
        }
        
        "h3" | "h4" | "h5" | "h6" => {
            context.newline();
            context.y += 8;
            context.font_size = FontSize::H3;
            context.line_height = 26;
            context.bold = true;
        }
        
        "a" => {
            if let Some(href) = el.attribute("href") {
                context.in_link = Some(href.to_string());
            }
        }
        
        "strong" | "b" => {
            context.bold = true;
        }
        
        "em" | "i" => {
            context.italic = true;
        }
        
        "code" => {
            // Inline code styling handled in text
        }
        
        "pre" => {
            context.newline();
            context.y += 8;
            context.in_pre = true;
            // Draw code background
            if context.y >= clip_y {
                framebuffer::fill_rect(
                    (clip_x + 8) as u32,
                    context.y as u32,
                    clip_w - 16,
                    100, // Will be adjusted
                    COLOR_CODE_BG,
                );
            }
        }
        
        "blockquote" => {
            context.newline();
            context.y += 8;
            context.list_depth += 1;
            // Draw quote border
            if context.y >= clip_y {
                framebuffer::fill_rect(
                    (clip_x + 12) as u32,
                    context.y as u32,
                    4,
                    80,
                    COLOR_QUOTE_BORDER,
                );
            }
        }
        
        "ul" | "ol" => {
            context.newline();
            context.list_depth += 1;
            if tag == "ol" {
                context.in_ordered_list = true;
                context.list_counters.push(0);
            }
        }
        
        "li" => {
            context.newline();
            // Draw bullet or number
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                if context.in_ordered_list {
                    // Numbered list
                    if let Some(counter) = context.list_counters.last_mut() {
                        *counter += 1;
                        let number_str = alloc::format!("{}.", counter);
                        let number_x = context.x - 20;
                        for (i, c) in number_str.chars().enumerate() {
                            draw_char((number_x + i as i32 * 8) as u32, context.y as u32, c, COLOR_TEXT);
                        }
                    }
                } else {
                    // Bullet
                    let bullet_x = context.x - 12;
                    framebuffer::fill_rect(
                        bullet_x as u32,
                        (context.y + 6) as u32,
                        4,
                        4,
                        COLOR_TEXT,
                    );
                }
            }
        }
        
        "img" => {
            // Show placeholder for images with proper dimensions
            let alt = el.attribute("alt").unwrap_or("");
            let source = el.attribute("src").unwrap_or("");
            
            // Parse width/height attributes
            let image_w = el.attribute("width")
                .and_then(|w| w.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(120)
                .minimum(clip_w.saturating_sub(32))
                .maximum(40);
            let image_h = el.attribute("height")
                .and_then(|h| h.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(60)
                .minimum(300)
                .maximum(20);
            
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                // Image placeholder background
                framebuffer::fill_rect(context.x as u32, context.y as u32, image_w, image_h, 0xFFF0F0F0);
                // Border
                framebuffer::fill_rect(context.x as u32, context.y as u32, image_w, 1, 0xFFDDDDDD);
                framebuffer::fill_rect(context.x as u32, (context.y + image_h as i32 - 1) as u32, image_w, 1, 0xFFDDDDDD);
                framebuffer::fill_rect(context.x as u32, context.y as u32, 1, image_h, 0xFFDDDDDD);
                framebuffer::fill_rect((context.x + image_w as i32 - 1) as u32, context.y as u32, 1, image_h, 0xFFDDDDDD);
                
                // Draw [IMG] icon in center
                let icon_text = "[IMG]";
                let icon_x = context.x + (image_w as i32 / 2) - 20;
                let icon_y = context.y + (image_h as i32 / 2) - 12;
                if image_h > 24 {
                    for (i, c) in icon_text.chars().enumerate() {
                        draw_char((icon_x + i as i32 * 8) as u32, icon_y as u32, c, 0xFF999999);
                    }
                }
                
                // Alt text or filename below icon
                let display_text = if !alt.is_empty() {
                    alt
                } else if !source.is_empty() {
                    source.rsplit('/').next().unwrap_or(source)
                } else {
                    ""
                };
                if !display_text.is_empty() && image_h > 40 {
                    let maximum_chars = (image_w / 8).saturating_sub(2) as usize;
                    let text_x = context.x + 8;
                    let text_y = context.y + (image_h as i32 / 2) + 4;
                    for (i, c) in display_text.chars().take(maximum_chars).enumerate() {
                        draw_char((text_x + i as i32 * 8) as u32, text_y as u32, c, 0xFF666666);
                    }
                }
            }
            context.y += image_h as i32 + 4;
        }
        
        "table" => {
            context.newline();
            context.y += 8;
        }
        
        "tr" => {
            context.newline();
        }
        
        "td" | "th" => {
            context.x += 16;
        }
        
        "input" => {
            let input_type = el.attribute("type").unwrap_or("text");
                        // Pattern matching — Rust's exhaustive branching construct.
match input_type {
                "hidden" => {
                    return;
                }
                "submit" | "button" => {
                    // Draw submit/button
                    let value = el.attribute("value").unwrap_or("Submit");
                    let button_width = (value.len() as u32 * 8) + 24;
                    let button_height = 28u32;
                    if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                        framebuffer::fill_rect(context.x as u32, context.y as u32, button_width, button_height, 0xFFE8E8E8);
                        // Border
                        framebuffer::fill_rect(context.x as u32, context.y as u32, button_width, 1, 0xFFBBBBBB);
                        framebuffer::fill_rect(context.x as u32, (context.y + button_height as i32 - 1) as u32, button_width, 1, 0xFFBBBBBB);
                        framebuffer::fill_rect(context.x as u32, context.y as u32, 1, button_height, 0xFFBBBBBB);
                        framebuffer::fill_rect((context.x + button_width as i32 - 1) as u32, context.y as u32, 1, button_height, 0xFFBBBBBB);
                        for (i, c) in value.chars().enumerate() {
                            draw_char((context.x + 12 + i as i32 * 8) as u32, (context.y + 6) as u32, c, COLOR_TEXT);
                        }
                    }
                    context.x += button_width as i32 + 8;
                }
                "checkbox" => {
                    if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                        framebuffer::fill_rect(context.x as u32, (context.y + 2) as u32, 14, 14, 0xFFFFFFFF);
                        framebuffer::fill_rect(context.x as u32, (context.y + 2) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(context.x as u32, (context.y + 15) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(context.x as u32, (context.y + 2) as u32, 1, 14, 0xFF999999);
                        framebuffer::fill_rect((context.x + 13) as u32, (context.y + 2) as u32, 1, 14, 0xFF999999);
                        if el.attribute("checked").is_some() {
                            framebuffer::fill_rect((context.x + 3) as u32, (context.y + 8) as u32, 8, 2, COLOR_TEXT);
                            framebuffer::fill_rect((context.x + 3) as u32, (context.y + 5) as u32, 2, 5, COLOR_TEXT);
                        }
                    }
                    context.x += 20;
                }
                "radio" => {
                    if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                        framebuffer::fill_rect(context.x as u32, (context.y + 2) as u32, 14, 14, 0xFFFFFFFF);
                        framebuffer::fill_rect(context.x as u32, (context.y + 2) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(context.x as u32, (context.y + 15) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(context.x as u32, (context.y + 2) as u32, 1, 14, 0xFF999999);
                        framebuffer::fill_rect((context.x + 13) as u32, (context.y + 2) as u32, 1, 14, 0xFF999999);
                        if el.attribute("checked").is_some() {
                            framebuffer::fill_rect((context.x + 4) as u32, (context.y + 6) as u32, 6, 6, COLOR_TEXT);
                        }
                    }
                    context.x += 20;
                }
                _ => {
                    // text, email, search, url, tel, password, number
                    let placeholder = el.attribute("placeholder").unwrap_or("");
                    let value = el.attribute("value").unwrap_or("");
                    let display_text = if value.is_empty() { placeholder } else { value };
                    let input_width = 200u32.minimum(clip_w.saturating_sub(40));
                    let input_height = 26u32;
                    if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                        framebuffer::fill_rect(context.x as u32, context.y as u32, input_width, input_height, 0xFFFFFFFF);
                        framebuffer::fill_rect(context.x as u32, context.y as u32, input_width, 1, 0xFF999999);
                        framebuffer::fill_rect(context.x as u32, (context.y + input_height as i32 - 1) as u32, input_width, 1, 0xFF999999);
                        framebuffer::fill_rect(context.x as u32, context.y as u32, 1, input_height, 0xFF999999);
                        framebuffer::fill_rect((context.x + input_width as i32 - 1) as u32, context.y as u32, 1, input_height, 0xFF999999);
                        let text_color = if value.is_empty() { 0xFF999999 } else { COLOR_TEXT };
                        for (i, c) in display_text.chars().take((input_width / 8 - 2) as usize).enumerate() {
                            draw_char((context.x + 4 + i as i32 * 8) as u32, (context.y + 5) as u32, c, text_color);
                        }
                    }
                    context.x += input_width as i32 + 8;
                }
            }
        }
        
        "button" => {
            // Render button with children as text
            let button_height = 28u32;
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(context.x as u32, context.y as u32, 120, button_height, 0xFFE8E8E8);
                framebuffer::fill_rect(context.x as u32, context.y as u32, 120, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(context.x as u32, (context.y + button_height as i32 - 1) as u32, 120, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(context.x as u32, context.y as u32, 1, button_height, 0xFFBBBBBB);
                framebuffer::fill_rect(119 + context.x as u32, context.y as u32, 1, button_height, 0xFFBBBBBB);
            }
            context.y += 6;
        }
        
        "textarea" => {
            context.newline();
            let ta_width = core::cmp::minimum(clip_w.saturating_sub(32), 400);
            let ta_height = 80u32;
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(context.x as u32, context.y as u32, ta_width, ta_height, 0xFFFFFFFF);
                framebuffer::fill_rect(context.x as u32, context.y as u32, ta_width, 1, 0xFF999999);
                framebuffer::fill_rect(context.x as u32, (context.y + ta_height as i32 - 1) as u32, ta_width, 1, 0xFF999999);
                framebuffer::fill_rect(context.x as u32, context.y as u32, 1, ta_height, 0xFF999999);
                framebuffer::fill_rect((context.x + ta_width as i32 - 1) as u32, context.y as u32, 1, ta_height, 0xFF999999);
            }
            context.y += ta_height as i32 + 8;
        }
        
        "select" => {
            let select_width = 160u32.minimum(clip_w.saturating_sub(32));
            let select_height = 26u32;
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(context.x as u32, context.y as u32, select_width, select_height, 0xFFFFFFFF);
                framebuffer::fill_rect(context.x as u32, context.y as u32, select_width, 1, 0xFF999999);
                framebuffer::fill_rect(context.x as u32, (context.y + select_height as i32 - 1) as u32, select_width, 1, 0xFF999999);
                framebuffer::fill_rect(context.x as u32, context.y as u32, 1, select_height, 0xFF999999);
                framebuffer::fill_rect((context.x + select_width as i32 - 1) as u32, context.y as u32, 1, select_height, 0xFF999999);
                // Dropdown arrow
                framebuffer::fill_rect((context.x + select_width as i32 - 16) as u32, (context.y + 10) as u32, 8, 2, 0xFF666666);
                framebuffer::fill_rect((context.x + select_width as i32 - 14) as u32, (context.y + 12) as u32, 4, 2, 0xFF666666);
            }
            context.x += select_width as i32 + 8;
        }
        
        "option" | "optgroup" => {
            // Skip options display (handled by select)
            return;
        }
        
        "small" | "sub" | "sup" => {
            context.font_size = FontSize::Small;
            context.line_height = FontSize::Small.height() + 4;
        }
        
        "mark" => {
            context.bg_color = Some(0xFFFFFF00); // Yellow highlight
        }
        
        "del" | "s" | "strike" => {
            context.text_color = 0xFF999999;
            context.strikethrough = true;
        }
        
        "u" | "ins" => {
            context.underline = true;
        }
        
        "progress" => {
            // HTML5 progress bar
            let maximum_value: f32 = el.attribute("max").and_then(|v| v.parse().ok()).unwrap_or(1.0);
            let cur_value: f32 = el.attribute("value").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let pct = (cur_value / maximum_value).minimum(1.0).maximum(0.0);
            let bar_w = 200u32.minimum(clip_w.saturating_sub(40));
            let bar_h = 18u32;
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                // Track background
                framebuffer::fill_rect(context.x as u32, context.y as u32, bar_w, bar_h, 0xFFE0E0E0);
                // Fill
                let fill_w = (bar_w as f32 * pct) as u32;
                if fill_w > 0 {
                    framebuffer::fill_rect(context.x as u32, context.y as u32, fill_w, bar_h, 0xFF4CAF50);
                }
                // Border
                framebuffer::fill_rect(context.x as u32, context.y as u32, bar_w, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(context.x as u32, (context.y + bar_h as i32 - 1) as u32, bar_w, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(context.x as u32, context.y as u32, 1, bar_h, 0xFFBBBBBB);
                framebuffer::fill_rect((context.x + bar_w as i32 - 1) as u32, context.y as u32, 1, bar_h, 0xFFBBBBBB);
                // Percentage text
                let pct_str = alloc::format!("{}%", (pct * 100.0) as u32);
                let text_x = context.x + (bar_w as i32 / 2) - (pct_str.len() as i32 * 4);
                for (i, c) in pct_str.chars().enumerate() {
                    draw_char((text_x + i as i32 * 8) as u32, (context.y + 1) as u32, c, COLOR_TEXT);
                }
            }
            context.x += bar_w as i32 + 8;
            return; // progress has no children
        }
        
        "meter" => {
            // HTML5 meter element
            let minimum_value: f32 = el.attribute("min").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let maximum_value: f32 = el.attribute("max").and_then(|v| v.parse().ok()).unwrap_or(1.0);
            let cur_value: f32 = el.attribute("value").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let low: f32 = el.attribute("low").and_then(|v| v.parse().ok()).unwrap_or(minimum_value);
            let high: f32 = el.attribute("high").and_then(|v| v.parse().ok()).unwrap_or(maximum_value);
            let range = maximum_value - minimum_value;
            let pct = if range > 0.0 { ((cur_value - minimum_value) / range).minimum(1.0).maximum(0.0) } else { 0.0 };
            let bar_w = 160u32.minimum(clip_w.saturating_sub(40));
            let bar_h = 16u32;
            // Color depends on value zone
            let fill_color = if cur_value < low {
                0xFFFF5722 // Orange-red (low)
            } else if cur_value > high {
                0xFFFF5722 // Orange-red (high/danger)
            } else {
                0xFF4CAF50 // Green (normal)
            };
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(context.x as u32, context.y as u32, bar_w, bar_h, 0xFFE0E0E0);
                let fill_w = (bar_w as f32 * pct) as u32;
                if fill_w > 0 {
                    framebuffer::fill_rect(context.x as u32, context.y as u32, fill_w, bar_h, fill_color);
                }
                framebuffer::fill_rect(context.x as u32, context.y as u32, bar_w, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(context.x as u32, (context.y + bar_h as i32 - 1) as u32, bar_w, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(context.x as u32, context.y as u32, 1, bar_h, 0xFFBBBBBB);
                framebuffer::fill_rect((context.x + bar_w as i32 - 1) as u32, context.y as u32, 1, bar_h, 0xFFBBBBBB);
            }
            context.x += bar_w as i32 + 8;
            return;
        }
        
        "dl" => {
            context.newline();
        }
        "dt" => {
            context.newline();
            context.bold = true;
        }
        "dd" => {
            context.newline();
            context.x += 40; // indent definition
        }
        
        "details" => {
            context.newline();
            context.y += 4;
            // Draw disclosure triangle
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                let tri_x = context.x;
                let open = el.attribute("open").is_some();
                if open {
                    // Down-pointing triangle
                    framebuffer::fill_rect(tri_x as u32, context.y as u32, 8, 2, COLOR_TEXT);
                    framebuffer::fill_rect((tri_x + 1) as u32, (context.y + 2) as u32, 6, 2, COLOR_TEXT);
                    framebuffer::fill_rect((tri_x + 2) as u32, (context.y + 4) as u32, 4, 2, COLOR_TEXT);
                } else {
                    // Right-pointing triangle
                    framebuffer::fill_rect(tri_x as u32, context.y as u32, 2, 8, COLOR_TEXT);
                    framebuffer::fill_rect((tri_x + 2) as u32, (context.y + 1) as u32, 2, 6, COLOR_TEXT);
                    framebuffer::fill_rect((tri_x + 4) as u32, (context.y + 2) as u32, 2, 4, COLOR_TEXT);
                }
            }
            context.x += 16; // move past triangle
        }
        
        "summary" => {
            context.bold = true;
        }
        
        "figure" => {
            context.newline();
            context.y += 8;
            context.list_depth += 1;
        }
        
        "figcaption" => {
            context.newline();
            context.font_size = FontSize::Small;
            context.line_height = FontSize::Small.height() + 4;
            context.text_color = 0xFF666666;
        }
        
        "nav" => {
            context.bg_color = Some(0xFFF8F8F8);
        }
        
        "footer" => {
            context.newline();
            context.y += 16;
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(
                    (clip_x + 16) as u32, context.y as u32, clip_w - 32, 1, COLOR_HR,
                );
            }
            context.y += 8;
            context.font_size = FontSize::Small;
            context.line_height = FontSize::Small.height() + 4;
            context.text_color = 0xFF666666;
        }
        
        "header" => {
            context.bg_color = Some(0xFFF0F0F0);
        }
        
        "main" | "article" | "section" | "aside" => {
            // Semantic containers — just render children
        }
        
        "video" | "audio" | "canvas" | "svg" | "iframe" | "embed" | "object" => {
            // Media placeholders
            let placeholder_w = el.attribute("width")
                .and_then(|w| w.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(320)
                .minimum(clip_w.saturating_sub(32));
            let placeholder_h = el.attribute("height")
                .and_then(|h| h.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(180)
                .minimum(400);
            if context.y >= clip_y && context.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(context.x as u32, context.y as u32, placeholder_w, placeholder_h, 0xFF2C2C2C);
                // Icon label
                let label = // Pattern matching — Rust's exhaustive branching construct.
match tag {
                    "video" => "[VIDEO]",
                    "audio" => "[AUDIO]",
                    "canvas" => "[CANVAS]",
                    "svg" => "[SVG]",
                    "iframe" => "[IFRAME]",
                    _ => "[EMBED]",
                };
                let lx = context.x + (placeholder_w as i32 / 2) - (label.len() as i32 * 4);
                let ly = context.y + (placeholder_h as i32 / 2) - 8;
                for (i, c) in label.chars().enumerate() {
                    draw_char((lx + i as i32 * 8) as u32, ly as u32, c, 0xFFAAAAAA);
                }
                // Play button triangle for video/audio
                if tag == "video" || tag == "audio" {
                    let cx = context.x + placeholder_w as i32 / 2;
                    let cy = context.y + placeholder_h as i32 / 2 + 12;
                    for row in 0..16 {
                        let w = 16 - row;
                        framebuffer::fill_rect((cx - 4) as u32, (cy + row) as u32, w as u32, 1, 0xFFFFFFFF);
                    }
                }
                // Border
                framebuffer::fill_rect(context.x as u32, context.y as u32, placeholder_w, 1, 0xFF555555);
                framebuffer::fill_rect(context.x as u32, (context.y + placeholder_h as i32 - 1) as u32, placeholder_w, 1, 0xFF555555);
                framebuffer::fill_rect(context.x as u32, context.y as u32, 1, placeholder_h, 0xFF555555);
                framebuffer::fill_rect((context.x + placeholder_w as i32 - 1) as u32, context.y as u32, 1, placeholder_h, 0xFF555555);
            }
            context.y += placeholder_h as i32 + 4;
            return; // no children rendering for media
        }
        
        "center" => {
            context.newline();
            context.x = clip_x + (clip_w as i32 / 4);
        }
        
        _ => {}
    }
    
    // Render children
    for child in &el.children {
        render_node(context, child, stylesheet, clip_x, clip_y, clip_w, clip_h);
    }
    
    // Restore context
    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            context.newline();
            context.y += 8;
        }
        "p" => {
            context.newline();
            context.y += 8;
        }
        "ul" | "ol" => {
            context.list_depth -= 1;
            context.list_counters.pop();
            context.newline();
        }
        "blockquote" => {
            context.list_depth -= 1;
            context.newline();
        }
        "pre" => {
            context.newline();
        }
        "table" => {
            context.newline();
        }
        "dl" => {
            context.newline();
            context.y += 4;
        }
        "dt" | "dd" => {
            context.newline();
        }
        "details" => {
            context.newline();
            context.y += 4;
        }
        "figure" => {
            context.list_depth -= 1;
            context.newline();
            context.y += 8;
        }
        "footer" | "header" | "nav" => {
            context.newline();
        }
        _ => {}
    }
    
    context.font_size = saved_font;
    context.line_height = saved_font.height() + 4;
    context.bold = saved_bold;
    context.in_link = saved_link;
    context.in_pre = saved_pre;
    context.text_color = saved_text_color;
    context.bg_color = saved_bg_color;
    context.opacity = saved_opacity;
    context.underline = saved_underline;
    context.strikethrough = saved_strikethrough;
    context.in_ordered_list = saved_ordered;
}

/// Extract CSS text from <style> elements in the DOM tree
fn extract_style_content(nodes: &[HtmlNode]) -> String {
    let mut css = String::new();
    for node in nodes {
        if let HtmlNode::Element(el) = node {
            if el.tag == "style" {
                // Collect text content inside <style>
                for child in &el.children {
                    if let HtmlNode::Text(text) = child {
                        css.push_str(text);
                        css.push('\n');
                    }
                }
            } else {
                // Recurse into non-style elements (including <head>)
                let child_css = extract_style_content(&el.children);
                if !child_css.is_empty() {
                    css.push_str(&child_css);
                }
            }
        }
    }
    css
}

/// Check if a CSS selector matches an HTML element
fn matches_element(selector: &Selector, element: &HtmlElement) -> bool {
    let parts = &selector.elements;
    if parts.is_empty() {
        return false;
    }
    
    // Find the last simple selector segment (after any combinator)
    // Also handle the parser not inserting Descendant for "div p" (multiple Tags)
    let mut segment_start = 0;
    let mut last_tag_index: Option<usize> = None;
    let mut has_combinator = false;
    
    for (i, part) in parts.iter().enumerate() {
                // Pattern matching — Rust's exhaustive branching construct.
match part {
            SelectorPart::Descendant | SelectorPart::Child | SelectorPart::Adjacent | SelectorPart::Sibling => {
                has_combinator = true;
                segment_start = i + 1;
                last_tag_index = None;
            }
            SelectorPart::Tag(_) => {
                if !has_combinator {
                    if let Some(_previous) = last_tag_index {
                        // Multiple tags without combinator = implicit descendant
                        segment_start = i;
                    }
                }
                last_tag_index = Some(i);
            }
            _ => {}
        }
    }
    
    let segment = &parts[segment_start..];
    
    for part in segment {
                // Pattern matching — Rust's exhaustive branching construct.
match part {
            SelectorPart::Tag(tag) => {
                if element.tag != *tag {
                    return false;
                }
            }
            SelectorPart::Class(class) => {
                let el_classes = element.attribute("class").unwrap_or("");
                if !el_classes.split_whitespace().any(|c| c == class.as_str()) {
                    return false;
                }
            }
            SelectorPart::Id(id) => {
                let el_id = element.attribute("id").unwrap_or("");
                if el_id != id.as_str() {
                    return false;
                }
            }
            SelectorPart::Universal => {}
            SelectorPart::Attribute(attribute, value) => {
                                // Pattern matching — Rust's exhaustive branching construct.
match element.attribute(attribute) {
                    Some(el_value) => {
                        if let Some(expected) = value {
                            if el_value != expected.as_str() {
                                return false;
                            }
                        }
                    }
                    None => return false,
                }
            }
            SelectorPart::Pseudo(_) => {} // Skip pseudo selectors for now
            _ => {} // Skip combinators
        }
    }
    
    !segment.is_empty()
}

/// Apply CSS rules from a stylesheet to an element
fn apply_stylesheet_rules(context: &mut RenderContext, el: &HtmlElement, stylesheet: &Stylesheet) {
    for rule in &stylesheet.rules {
        let matched = rule.selectors.iter().any(|sel| matches_element(sel, el));
        if matched {
            apply_declarations(context, &rule.declarations);
        }
    }
}

/// Apply a list of CSS declarations to the render context
fn apply_declarations(context: &mut RenderContext, declarations: &[Declaration]) {
    for decl in declarations {
                // Pattern matching — Rust's exhaustive branching construct.
match decl.property.as_str() {
            "color" => {
                if let Some(c) = css_value_to_color(&decl.value) {
                    context.text_color = c;
                }
            }
            "background-color" | "background" => {
                if let Some(c) = css_value_to_color(&decl.value) {
                    context.bg_color = Some(c);
                }
            }
            "font-size" => {
                                // Pattern matching — Rust's exhaustive branching construct.
match &decl.value {
                    CssValue::Length(pixel, _) => {
                        if *pixel <= 14.0 {
                            context.font_size = FontSize::Small;
                        } else if *pixel <= 18.0 {
                            context.font_size = FontSize::Normal;
                        } else if *pixel <= 22.0 {
                            context.font_size = FontSize::Large;
                        } else if *pixel <= 28.0 {
                            context.font_size = FontSize::H3;
                        } else if *pixel <= 30.0 {
                            context.font_size = FontSize::H2;
                        } else {
                            context.font_size = FontSize::H1;
                        }
                        context.line_height = context.font_size.height() + 4;
                    }
                    CssValue::Keyword(kw) => {
                                                // Pattern matching — Rust's exhaustive branching construct.
match kw.as_str() {
                            "small" | "x-small" | "xx-small" => context.font_size = FontSize::Small,
                            "medium" => context.font_size = FontSize::Normal,
                            "large" => context.font_size = FontSize::Large,
                            "x-large" | "xx-large" => context.font_size = FontSize::H1,
                            _ => {}
                        }
                        context.line_height = context.font_size.height() + 4;
                    }
                    _ => {}
                }
            }
            "font-weight" => {
                                // Pattern matching — Rust's exhaustive branching construct.
match &decl.value {
                    CssValue::Keyword(kw) if kw == "bold" => context.bold = true,
                    CssValue::Keyword(kw) if kw == "normal" => context.bold = false,
                    CssValue::Number(n) if *n >= 700.0 => context.bold = true,
                    _ => {}
                }
            }
            "font-style" => {
                if let CssValue::Keyword(kw) = &decl.value {
                    context.italic = kw == "italic" || kw == "oblique";
                }
            }
            "display" => {
                if let CssValue::Keyword(kw) = &decl.value {
                    if kw == "none" {
                        context.y = i32::MAX / 2;
                    }
                }
            }
            "text-align" => {
                if let CssValue::Keyword(kw) = &decl.value {
                                        // Pattern matching — Rust's exhaustive branching construct.
match kw.as_str() {
                        "center" => { context.x = context.maximum_width as i32 / 4; }
                        _ => {}
                    }
                }
            }
            "margin-top" | "padding-top" => {
                if let CssValue::Length(pixel, _) = &decl.value {
                    context.y += *pixel as i32;
                }
            }
            "margin-bottom" | "padding-bottom" => {
                // Handled after element rendering
            }
            "visibility" => {
                if let CssValue::Keyword(kw) = &decl.value {
                    if kw == "hidden" || kw == "collapse" {
                        context.y = i32::MAX / 2;
                    }
                }
            }
            "opacity" => {
                                // Pattern matching — Rust's exhaustive branching construct.
match &decl.value {
                    CssValue::Number(n) => {
                        context.opacity = n.maximum(0.0).minimum(1.0) as f32;
                    }
                    _ => {}
                }
            }
            "text-decoration" | "text-decoration-line" => {
                if let CssValue::Keyword(kw) = &decl.value {
                                        // Pattern matching — Rust's exhaustive branching construct.
match kw.as_str() {
                        "underline" => context.underline = true,
                        "line-through" => context.strikethrough = true,
                        "none" => {
                            context.underline = false;
                            context.strikethrough = false;
                        }
                        _ => {}
                    }
                }
            }
            "text-transform" => {
                // Handled during text rendering via ctx state
                if let CssValue::Keyword(kw) = &decl.value {
                                        // Pattern matching — Rust's exhaustive branching construct.
match kw.as_str() {
                        "uppercase" | "lowercase" | "capitalize" | "none" => {
                            // Store for later use in render_word
                        }
                        _ => {}
                    }
                }
            }
            "line-height" => {
                                // Pattern matching — Rust's exhaustive branching construct.
match &decl.value {
                    CssValue::Length(pixel, _) => {
                        context.line_height = *pixel as i32;
                    }
                    CssValue::Number(n) => {
                        context.line_height = (*n as i32) * context.font_size.height();
                    }
                    _ => {}
                }
            }
            "margin-left" | "padding-left" => {
                if let CssValue::Length(pixel, _) = &decl.value {
                    context.x += *pixel as i32;
                }
            }
            "border" | "border-top" | "border-bottom" | "border-left" | "border-right" => {
                // Basic border hints — just parse the color for rendering
            }
            _ => {} // Ignore unsupported properties
        }
    }
}

/// Apply CSS declarations from inline style="" attribute
fn apply_inline_style(context: &mut RenderContext, style_str: &str) {
    let declarations = css_parser::parse_inline_style(style_str);
    apply_declarations(context, &declarations);
}

/// Convert CSS value to ARGB color
fn css_value_to_color(value: &CssValue) -> Option<u32> {
        // Pattern matching — Rust's exhaustive branching construct.
match value {
        CssValue::Color(c) => Some(*c),
        CssValue::Keyword(name) => parse_html_color(name),
        _ => None,
    }
}

/// Parse HTML color names and hex codes to ARGB
fn parse_html_color(s: &str) -> Option<u32> {
    let s = s.trim();
    
    // Hex colors
    if s.starts_with('#') {
        let hex = &s[1..];
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32);
        }
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some(0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32);
        }
        return None;
    }
    
    // Named colors
    match s.to_lowercase().as_str() {
        "black" => Some(0xFF000000),
        "white" => Some(0xFFFFFFFF),
        "red" => Some(0xFFFF0000),
        "green" => Some(0xFF008000),
        "blue" => Some(0xFF0000FF),
        "yellow" => Some(0xFFFFFF00),
        "cyan" | "aqua" => Some(0xFF00FFFF),
        "magenta" | "fuchsia" => Some(0xFFFF00FF),
        "gray" | "grey" => Some(0xFF808080),
        "silver" => Some(0xFFC0C0C0),
        "maroon" => Some(0xFF800000),
        "olive" => Some(0xFF808000),
        "navy" => Some(0xFF000080),
        "teal" => Some(0xFF008080),
        "purple" => Some(0xFF800080),
        "orange" => Some(0xFFFFA500),
        "pink" => Some(0xFFFFC0CB),
        "brown" => Some(0xFFA52A2A),
        "coral" => Some(0xFFFF7F50),
        "crimson" => Some(0xFFDC143C),
        "darkblue" => Some(0xFF00008B),
        "darkgreen" => Some(0xFF006400),
        "darkred" => Some(0xFF8B0000),
        "gold" => Some(0xFFFFD700),
        "indigo" => Some(0xFF4B0082),
        "lime" => Some(0xFF00FF00),
        "tomato" => Some(0xFFFF6347),
        "transparent" => Some(0x00000000),
        _ => None,
    }
}

/// Draw a character (simplified - uses 8x16 font approximation)
fn draw_char(x: u32, y: u32, c: char, color: u32) {
    let font = get_char_bitmap(c);
    
    for (row, byte) in font.iter().enumerate() {
        for bit in 0..8 {
            if (byte >> (7 - bit)) & 1 != 0 {
                framebuffer::put_pixel(x + bit, y + row as u32, color);
            }
        }
    }
}

/// Get 8x16 bitmap for character (simplified font)
fn get_char_bitmap(c: char) -> [u8; 16] {
    // Basic ASCII subset
    match c {
        ' ' => [0x00; 16],
        '!' => [0x00,0x18,0x18,0x18,0x18,0x18,0x18,0x18,0x00,0x18,0x18,0x00,0x00,0x00,0x00,0x00],
        '"' => [0x00,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '#' => [0x00,0x36,0x36,0x7F,0x36,0x36,0x7F,0x36,0x36,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '.' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x00,0x00,0x00,0x00,0x00],
        ',' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x18,0x18,0x08,0x10,0x00,0x00,0x00],
        ':' => [0x00,0x00,0x00,0x18,0x18,0x00,0x00,0x00,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00],
        ';' => [0x00,0x00,0x00,0x18,0x18,0x00,0x00,0x00,0x18,0x18,0x08,0x10,0x00,0x00,0x00,0x00],
        '?' => [0x00,0x3C,0x66,0x06,0x0C,0x18,0x18,0x00,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00],
        '-' => [0x00,0x00,0x00,0x00,0x00,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '_' => [0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x7F,0x00,0x00,0x00,0x00],
        '/' => [0x00,0x02,0x04,0x08,0x10,0x20,0x40,0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '(' => [0x00,0x0C,0x18,0x30,0x30,0x30,0x30,0x30,0x18,0x0C,0x00,0x00,0x00,0x00,0x00,0x00],
        ')' => [0x00,0x30,0x18,0x0C,0x0C,0x0C,0x0C,0x0C,0x18,0x30,0x00,0x00,0x00,0x00,0x00,0x00],
        '[' => [0x00,0x3C,0x30,0x30,0x30,0x30,0x30,0x30,0x30,0x3C,0x00,0x00,0x00,0x00,0x00,0x00],
        ']' => [0x00,0x3C,0x0C,0x0C,0x0C,0x0C,0x0C,0x0C,0x0C,0x3C,0x00,0x00,0x00,0x00,0x00,0x00],
        '<' => [0x00,0x00,0x06,0x0C,0x18,0x30,0x18,0x0C,0x06,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '>' => [0x00,0x00,0x60,0x30,0x18,0x0C,0x18,0x30,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '=' => [0x00,0x00,0x00,0x7E,0x00,0x00,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '+' => [0x00,0x00,0x18,0x18,0x18,0x7E,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '0' => [0x00,0x3C,0x66,0x66,0x6E,0x76,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '1' => [0x00,0x18,0x38,0x18,0x18,0x18,0x18,0x18,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '2' => [0x00,0x3C,0x66,0x06,0x0C,0x18,0x30,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '3' => [0x00,0x3C,0x66,0x06,0x1C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '4' => [0x00,0x0C,0x1C,0x3C,0x6C,0x7E,0x0C,0x0C,0x0C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '5' => [0x00,0x7E,0x60,0x7C,0x06,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '6' => [0x00,0x1C,0x30,0x60,0x7C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '7' => [0x00,0x7E,0x06,0x0C,0x18,0x30,0x30,0x30,0x30,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '8' => [0x00,0x3C,0x66,0x66,0x3C,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        '9' => [0x00,0x3C,0x66,0x66,0x66,0x3E,0x06,0x0C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'A' | 'a' => [0x00,0x18,0x3C,0x66,0x66,0x7E,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'B' | 'b' => [0x00,0x7C,0x66,0x66,0x7C,0x66,0x66,0x66,0x7C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'C' | 'c' => [0x00,0x3C,0x66,0x60,0x60,0x60,0x60,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'D' | 'd' => [0x00,0x78,0x6C,0x66,0x66,0x66,0x66,0x6C,0x78,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'E' | 'e' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'F' | 'f' => [0x00,0x7E,0x60,0x60,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'G' | 'g' => [0x00,0x3C,0x66,0x60,0x60,0x6E,0x66,0x66,0x3E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'H' | 'h' => [0x00,0x66,0x66,0x66,0x7E,0x66,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'I' | 'i' => [0x00,0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'J' | 'j' => [0x00,0x3E,0x0C,0x0C,0x0C,0x0C,0x0C,0x6C,0x38,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'K' | 'k' => [0x00,0x66,0x6C,0x78,0x70,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'L' | 'l' => [0x00,0x60,0x60,0x60,0x60,0x60,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'M' | 'm' => [0x00,0x63,0x77,0x7F,0x6B,0x63,0x63,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'N' | 'n' => [0x00,0x66,0x76,0x7E,0x7E,0x6E,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'O' | 'o' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'P' | 'p' => [0x00,0x7C,0x66,0x66,0x7C,0x60,0x60,0x60,0x60,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Q' | 'q' => [0x00,0x3C,0x66,0x66,0x66,0x66,0x6E,0x3C,0x0E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'R' | 'r' => [0x00,0x7C,0x66,0x66,0x7C,0x78,0x6C,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'S' | 's' => [0x00,0x3C,0x66,0x60,0x3C,0x06,0x06,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'T' | 't' => [0x00,0x7E,0x18,0x18,0x18,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'U' | 'u' => [0x00,0x66,0x66,0x66,0x66,0x66,0x66,0x66,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'V' | 'v' => [0x00,0x66,0x66,0x66,0x66,0x66,0x3C,0x3C,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'W' | 'w' => [0x00,0x63,0x63,0x63,0x6B,0x7F,0x77,0x63,0x63,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'X' | 'x' => [0x00,0x66,0x66,0x3C,0x18,0x3C,0x66,0x66,0x66,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Y' | 'y' => [0x00,0x66,0x66,0x66,0x3C,0x18,0x18,0x18,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        'Z' | 'z' => [0x00,0x7E,0x06,0x0C,0x18,0x30,0x60,0x60,0x7E,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
        _ => [0x00,0x3C,0x42,0x42,0x42,0x42,0x42,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], // Box
    }
}
