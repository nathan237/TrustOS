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
    pub max_width: u32,
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
}

#[derive(Clone, Copy, PartialEq)]
pub enum FontSize {
    Small,
    Normal,
    Large,
    H1,
    H2,
    H3,
}

impl FontSize {
    fn height(&self) -> i32 {
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

impl RenderContext {
    pub fn new(width: u32) -> Self {
        Self {
            x: 16,
            y: 16,
            max_width: width - 32,
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
    
    let mut ctx = RenderContext::new(width);
    ctx.y = y - scroll_y;
    
    // Extract CSS from <style> tags and parse into stylesheet
    let css_text = extract_style_content(&doc.nodes);
    let stylesheet = if !css_text.is_empty() {
        css_parser::parse_stylesheet(&css_text)
    } else {
        Stylesheet { rules: Vec::new() }
    };
    
    // Render all nodes
    for node in &doc.nodes {
        render_node(&mut ctx, node, &stylesheet, x, y, width, height);
    }
    
    ctx.links
}

/// Render a single node
fn render_node(
    ctx: &mut RenderContext,
    node: &HtmlNode,
    stylesheet: &Stylesheet,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
    match node {
        HtmlNode::Text(text) => {
            render_text(ctx, text, clip_x, clip_y, clip_w, clip_h);
        }
        HtmlNode::Element(el) => {
            render_element(ctx, el, stylesheet, clip_x, clip_y, clip_w, clip_h);
        }
    }
}

/// Render text content
fn render_text(
    ctx: &mut RenderContext,
    text: &str,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
    let color = if ctx.in_link.is_some() { COLOR_LINK } else { ctx.text_color };
    
    // Handle preformatted text
    if ctx.in_pre {
        for line in text.lines() {
            render_line(ctx, line, color, clip_x, clip_y, clip_w, clip_h);
            ctx.newline();
        }
        return;
    }
    
    // Normal text - word wrap
    let words: Vec<&str> = text.split_whitespace().collect();
    
    for word in words {
        let word_width = word.len() as i32 * 8; // Approximate
        
        // Word wrap
        if ctx.x + word_width > (clip_x + clip_w as i32 - 16) && ctx.x > 16 + (ctx.list_depth * 24) {
            ctx.newline();
        }
        
        render_word(ctx, word, color, clip_x, clip_y, clip_w, clip_h);
        ctx.space();
    }
}

fn render_word(
    ctx: &mut RenderContext,
    word: &str,
    color: u32,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
    // Check if visible
    if ctx.y + ctx.line_height < clip_y || ctx.y > clip_y + clip_h as i32 {
        ctx.x += word.len() as i32 * 8;
        return;
    }
    
    // Track link bounds
    let link_start_x = ctx.x;
    
    // Draw each character
    for c in word.chars() {
        if ctx.x >= clip_x && ctx.x < clip_x + clip_w as i32 - 8 {
            draw_char(ctx.x as u32, ctx.y as u32, c, color);
        }
        ctx.x += 8;
    }
    
    // Add link if in anchor
    if let Some(href) = &ctx.in_link {
        ctx.links.push(HtmlLink {
            href: href.clone(),
            x: link_start_x,
            y: ctx.y,
            width: (ctx.x - link_start_x) as u32,
            height: ctx.line_height as u32,
        });
        
        // Draw underline for links
        if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
            framebuffer::fill_rect(
                link_start_x as u32,
                (ctx.y + ctx.line_height - 2) as u32,
                (ctx.x - link_start_x) as u32,
                1,
                COLOR_LINK,
            );
        }
    }
}

fn render_line(
    ctx: &mut RenderContext,
    line: &str,
    color: u32,
    clip_x: i32,
    clip_y: i32,
    clip_w: u32,
    clip_h: u32,
) {
    if ctx.y + ctx.line_height < clip_y || ctx.y > clip_y + clip_h as i32 {
        return;
    }
    
    for c in line.chars() {
        if ctx.x >= clip_x && ctx.x < clip_x + clip_w as i32 - 8 {
            draw_char(ctx.x as u32, ctx.y as u32, c, color);
        }
        ctx.x += 8;
    }
}

/// Render an HTML element
fn render_element(
    ctx: &mut RenderContext,
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
    let saved_font = ctx.font_size;
    let saved_bold = ctx.bold;
    let saved_link = ctx.in_link.clone();
    let saved_pre = ctx.in_pre;
    let saved_text_color = ctx.text_color;
    let saved_bg_color = ctx.bg_color;
    
    // Apply inline CSS styles from style="" attribute
    if let Some(style_str) = el.attr("style") {
        apply_inline_style(ctx, style_str);
    }
    
    // Apply stylesheet rules (from <style> tags)
    apply_stylesheet_rules(ctx, el, stylesheet);
    
    // Apply color from HTML color attribute (legacy support)
    if let Some(color_str) = el.attr("color") {
        if let Some(c) = parse_html_color(color_str) {
            ctx.text_color = c;
        }
    }
    if let Some(bgcolor_str) = el.attr("bgcolor") {
        if let Some(c) = parse_html_color(bgcolor_str) {
            ctx.bg_color = Some(c);
        }
    }
    
    // Check if element was hidden by display:none
    if ctx.y >= i32::MAX / 4 {
        ctx.font_size = saved_font;
        ctx.line_height = saved_font.height() + 4;
        ctx.bold = saved_bold;
        ctx.in_link = saved_link;
        ctx.in_pre = saved_pre;
        ctx.text_color = saved_text_color;
        ctx.bg_color = saved_bg_color;
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
            ctx.newline();
            ctx.y += 8; // Paragraph spacing
        }
        
        "br" => {
            ctx.newline();
        }
        
        "hr" => {
            ctx.newline();
            ctx.y += 8;
            if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(
                    (clip_x + 16) as u32,
                    ctx.y as u32,
                    clip_w - 32,
                    1,
                    COLOR_HR,
                );
            }
            ctx.y += 16;
            ctx.x = 16;
        }
        
        "h1" => {
            ctx.newline();
            ctx.y += 16;
            ctx.font_size = FontSize::H1;
            ctx.line_height = 40;
            ctx.bold = true;
        }
        
        "h2" => {
            ctx.newline();
            ctx.y += 12;
            ctx.font_size = FontSize::H2;
            ctx.line_height = 32;
            ctx.bold = true;
        }
        
        "h3" | "h4" | "h5" | "h6" => {
            ctx.newline();
            ctx.y += 8;
            ctx.font_size = FontSize::H3;
            ctx.line_height = 26;
            ctx.bold = true;
        }
        
        "a" => {
            if let Some(href) = el.attr("href") {
                ctx.in_link = Some(href.to_string());
            }
        }
        
        "strong" | "b" => {
            ctx.bold = true;
        }
        
        "em" | "i" => {
            ctx.italic = true;
        }
        
        "code" => {
            // Inline code styling handled in text
        }
        
        "pre" => {
            ctx.newline();
            ctx.y += 8;
            ctx.in_pre = true;
            // Draw code background
            if ctx.y >= clip_y {
                framebuffer::fill_rect(
                    (clip_x + 8) as u32,
                    ctx.y as u32,
                    clip_w - 16,
                    100, // Will be adjusted
                    COLOR_CODE_BG,
                );
            }
        }
        
        "blockquote" => {
            ctx.newline();
            ctx.y += 8;
            ctx.list_depth += 1;
            // Draw quote border
            if ctx.y >= clip_y {
                framebuffer::fill_rect(
                    (clip_x + 12) as u32,
                    ctx.y as u32,
                    4,
                    80,
                    COLOR_QUOTE_BORDER,
                );
            }
        }
        
        "ul" | "ol" => {
            ctx.newline();
            ctx.list_depth += 1;
        }
        
        "li" => {
            ctx.newline();
            // Draw bullet
            if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                let bullet_x = ctx.x - 12;
                framebuffer::fill_rect(
                    bullet_x as u32,
                    (ctx.y + 6) as u32,
                    4,
                    4,
                    COLOR_TEXT,
                );
            }
        }
        
        "img" => {
            // Show placeholder for images with proper dimensions
            let alt = el.attr("alt").unwrap_or("");
            let src = el.attr("src").unwrap_or("");
            
            // Parse width/height attributes
            let img_w = el.attr("width")
                .and_then(|w| w.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(120)
                .min(clip_w.saturating_sub(32))
                .max(40);
            let img_h = el.attr("height")
                .and_then(|h| h.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(60)
                .min(300)
                .max(20);
            
            if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                // Image placeholder background
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, img_w, img_h, 0xFFF0F0F0);
                // Border
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, img_w, 1, 0xFFDDDDDD);
                framebuffer::fill_rect(ctx.x as u32, (ctx.y + img_h as i32 - 1) as u32, img_w, 1, 0xFFDDDDDD);
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, 1, img_h, 0xFFDDDDDD);
                framebuffer::fill_rect((ctx.x + img_w as i32 - 1) as u32, ctx.y as u32, 1, img_h, 0xFFDDDDDD);
                
                // Draw [IMG] icon in center
                let icon_text = "[IMG]";
                let icon_x = ctx.x + (img_w as i32 / 2) - 20;
                let icon_y = ctx.y + (img_h as i32 / 2) - 12;
                if img_h > 24 {
                    for (i, c) in icon_text.chars().enumerate() {
                        draw_char((icon_x + i as i32 * 8) as u32, icon_y as u32, c, 0xFF999999);
                    }
                }
                
                // Alt text or filename below icon
                let display_text = if !alt.is_empty() {
                    alt
                } else if !src.is_empty() {
                    src.rsplit('/').next().unwrap_or(src)
                } else {
                    ""
                };
                if !display_text.is_empty() && img_h > 40 {
                    let max_chars = (img_w / 8).saturating_sub(2) as usize;
                    let text_x = ctx.x + 8;
                    let text_y = ctx.y + (img_h as i32 / 2) + 4;
                    for (i, c) in display_text.chars().take(max_chars).enumerate() {
                        draw_char((text_x + i as i32 * 8) as u32, text_y as u32, c, 0xFF666666);
                    }
                }
            }
            ctx.y += img_h as i32 + 4;
        }
        
        "table" => {
            ctx.newline();
            ctx.y += 8;
        }
        
        "tr" => {
            ctx.newline();
        }
        
        "td" | "th" => {
            ctx.x += 16;
        }
        
        "input" => {
            let input_type = el.attr("type").unwrap_or("text");
            match input_type {
                "hidden" => {
                    return;
                }
                "submit" | "button" => {
                    // Draw submit/button
                    let value = el.attr("value").unwrap_or("Submit");
                    let btn_width = (value.len() as u32 * 8) + 24;
                    let btn_height = 28u32;
                    if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                        framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, btn_width, btn_height, 0xFFE8E8E8);
                        // Border
                        framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, btn_width, 1, 0xFFBBBBBB);
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + btn_height as i32 - 1) as u32, btn_width, 1, 0xFFBBBBBB);
                        framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, 1, btn_height, 0xFFBBBBBB);
                        framebuffer::fill_rect((ctx.x + btn_width as i32 - 1) as u32, ctx.y as u32, 1, btn_height, 0xFFBBBBBB);
                        for (i, c) in value.chars().enumerate() {
                            draw_char((ctx.x + 12 + i as i32 * 8) as u32, (ctx.y + 6) as u32, c, COLOR_TEXT);
                        }
                    }
                    ctx.x += btn_width as i32 + 8;
                }
                "checkbox" => {
                    if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + 2) as u32, 14, 14, 0xFFFFFFFF);
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + 2) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + 15) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + 2) as u32, 1, 14, 0xFF999999);
                        framebuffer::fill_rect((ctx.x + 13) as u32, (ctx.y + 2) as u32, 1, 14, 0xFF999999);
                        if el.attr("checked").is_some() {
                            framebuffer::fill_rect((ctx.x + 3) as u32, (ctx.y + 8) as u32, 8, 2, COLOR_TEXT);
                            framebuffer::fill_rect((ctx.x + 3) as u32, (ctx.y + 5) as u32, 2, 5, COLOR_TEXT);
                        }
                    }
                    ctx.x += 20;
                }
                "radio" => {
                    if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + 2) as u32, 14, 14, 0xFFFFFFFF);
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + 2) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + 15) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + 2) as u32, 1, 14, 0xFF999999);
                        framebuffer::fill_rect((ctx.x + 13) as u32, (ctx.y + 2) as u32, 1, 14, 0xFF999999);
                        if el.attr("checked").is_some() {
                            framebuffer::fill_rect((ctx.x + 4) as u32, (ctx.y + 6) as u32, 6, 6, COLOR_TEXT);
                        }
                    }
                    ctx.x += 20;
                }
                _ => {
                    // text, email, search, url, tel, password, number
                    let placeholder = el.attr("placeholder").unwrap_or("");
                    let value = el.attr("value").unwrap_or("");
                    let display_text = if value.is_empty() { placeholder } else { value };
                    let input_width = 200u32.min(clip_w.saturating_sub(40));
                    let input_height = 26u32;
                    if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                        framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, input_width, input_height, 0xFFFFFFFF);
                        framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, input_width, 1, 0xFF999999);
                        framebuffer::fill_rect(ctx.x as u32, (ctx.y + input_height as i32 - 1) as u32, input_width, 1, 0xFF999999);
                        framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, 1, input_height, 0xFF999999);
                        framebuffer::fill_rect((ctx.x + input_width as i32 - 1) as u32, ctx.y as u32, 1, input_height, 0xFF999999);
                        let text_color = if value.is_empty() { 0xFF999999 } else { COLOR_TEXT };
                        for (i, c) in display_text.chars().take((input_width / 8 - 2) as usize).enumerate() {
                            draw_char((ctx.x + 4 + i as i32 * 8) as u32, (ctx.y + 5) as u32, c, text_color);
                        }
                    }
                    ctx.x += input_width as i32 + 8;
                }
            }
        }
        
        "button" => {
            // Render button with children as text
            let btn_height = 28u32;
            if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, 120, btn_height, 0xFFE8E8E8);
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, 120, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(ctx.x as u32, (ctx.y + btn_height as i32 - 1) as u32, 120, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, 1, btn_height, 0xFFBBBBBB);
                framebuffer::fill_rect(119 + ctx.x as u32, ctx.y as u32, 1, btn_height, 0xFFBBBBBB);
            }
            ctx.y += 6;
        }
        
        "textarea" => {
            ctx.newline();
            let ta_width = core::cmp::min(clip_w.saturating_sub(32), 400);
            let ta_height = 80u32;
            if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, ta_width, ta_height, 0xFFFFFFFF);
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, ta_width, 1, 0xFF999999);
                framebuffer::fill_rect(ctx.x as u32, (ctx.y + ta_height as i32 - 1) as u32, ta_width, 1, 0xFF999999);
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, 1, ta_height, 0xFF999999);
                framebuffer::fill_rect((ctx.x + ta_width as i32 - 1) as u32, ctx.y as u32, 1, ta_height, 0xFF999999);
            }
            ctx.y += ta_height as i32 + 8;
        }
        
        "select" => {
            let select_width = 160u32.min(clip_w.saturating_sub(32));
            let select_height = 26u32;
            if ctx.y >= clip_y && ctx.y < clip_y + clip_h as i32 {
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, select_width, select_height, 0xFFFFFFFF);
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, select_width, 1, 0xFF999999);
                framebuffer::fill_rect(ctx.x as u32, (ctx.y + select_height as i32 - 1) as u32, select_width, 1, 0xFF999999);
                framebuffer::fill_rect(ctx.x as u32, ctx.y as u32, 1, select_height, 0xFF999999);
                framebuffer::fill_rect((ctx.x + select_width as i32 - 1) as u32, ctx.y as u32, 1, select_height, 0xFF999999);
                // Dropdown arrow
                framebuffer::fill_rect((ctx.x + select_width as i32 - 16) as u32, (ctx.y + 10) as u32, 8, 2, 0xFF666666);
                framebuffer::fill_rect((ctx.x + select_width as i32 - 14) as u32, (ctx.y + 12) as u32, 4, 2, 0xFF666666);
            }
            ctx.x += select_width as i32 + 8;
        }
        
        "option" | "optgroup" => {
            // Skip options display (handled by select)
            return;
        }
        
        "small" | "sub" | "sup" => {
            ctx.font_size = FontSize::Small;
            ctx.line_height = FontSize::Small.height() + 4;
        }
        
        "mark" => {
            ctx.bg_color = Some(0xFFFFFF00); // Yellow highlight
        }
        
        "del" | "s" | "strike" => {
            ctx.text_color = 0xFF999999;
        }
        
        "center" => {
            ctx.newline();
            ctx.x = clip_x + (clip_w as i32 / 4);
        }
        
        _ => {}
    }
    
    // Render children
    for child in &el.children {
        render_node(ctx, child, stylesheet, clip_x, clip_y, clip_w, clip_h);
    }
    
    // Restore context
    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            ctx.newline();
            ctx.y += 8;
        }
        "p" => {
            ctx.newline();
            ctx.y += 8;
        }
        "ul" | "ol" => {
            ctx.list_depth -= 1;
            ctx.newline();
        }
        "blockquote" => {
            ctx.list_depth -= 1;
            ctx.newline();
        }
        "pre" => {
            ctx.newline();
        }
        "table" => {
            ctx.newline();
        }
        _ => {}
    }
    
    ctx.font_size = saved_font;
    ctx.line_height = saved_font.height() + 4;
    ctx.bold = saved_bold;
    ctx.in_link = saved_link;
    ctx.in_pre = saved_pre;
    ctx.text_color = saved_text_color;
    ctx.bg_color = saved_bg_color;
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
    let mut last_tag_idx: Option<usize> = None;
    let mut has_combinator = false;
    
    for (i, part) in parts.iter().enumerate() {
        match part {
            SelectorPart::Descendant | SelectorPart::Child | SelectorPart::Adjacent | SelectorPart::Sibling => {
                has_combinator = true;
                segment_start = i + 1;
                last_tag_idx = None;
            }
            SelectorPart::Tag(_) => {
                if !has_combinator {
                    if let Some(_prev) = last_tag_idx {
                        // Multiple tags without combinator = implicit descendant
                        segment_start = i;
                    }
                }
                last_tag_idx = Some(i);
            }
            _ => {}
        }
    }
    
    let segment = &parts[segment_start..];
    
    for part in segment {
        match part {
            SelectorPart::Tag(tag) => {
                if element.tag != *tag {
                    return false;
                }
            }
            SelectorPart::Class(class) => {
                let el_classes = element.attr("class").unwrap_or("");
                if !el_classes.split_whitespace().any(|c| c == class.as_str()) {
                    return false;
                }
            }
            SelectorPart::Id(id) => {
                let el_id = element.attr("id").unwrap_or("");
                if el_id != id.as_str() {
                    return false;
                }
            }
            SelectorPart::Universal => {}
            SelectorPart::Attribute(attr, val) => {
                match element.attr(attr) {
                    Some(el_val) => {
                        if let Some(expected) = val {
                            if el_val != expected.as_str() {
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
fn apply_stylesheet_rules(ctx: &mut RenderContext, el: &HtmlElement, stylesheet: &Stylesheet) {
    for rule in &stylesheet.rules {
        let matched = rule.selectors.iter().any(|sel| matches_element(sel, el));
        if matched {
            apply_declarations(ctx, &rule.declarations);
        }
    }
}

/// Apply a list of CSS declarations to the render context
fn apply_declarations(ctx: &mut RenderContext, declarations: &[Declaration]) {
    for decl in declarations {
        match decl.property.as_str() {
            "color" => {
                if let Some(c) = css_value_to_color(&decl.value) {
                    ctx.text_color = c;
                }
            }
            "background-color" | "background" => {
                if let Some(c) = css_value_to_color(&decl.value) {
                    ctx.bg_color = Some(c);
                }
            }
            "font-size" => {
                match &decl.value {
                    CssValue::Length(px, _) => {
                        if *px <= 14.0 {
                            ctx.font_size = FontSize::Small;
                        } else if *px <= 18.0 {
                            ctx.font_size = FontSize::Normal;
                        } else if *px <= 22.0 {
                            ctx.font_size = FontSize::Large;
                        } else if *px <= 28.0 {
                            ctx.font_size = FontSize::H3;
                        } else if *px <= 30.0 {
                            ctx.font_size = FontSize::H2;
                        } else {
                            ctx.font_size = FontSize::H1;
                        }
                        ctx.line_height = ctx.font_size.height() + 4;
                    }
                    CssValue::Keyword(kw) => {
                        match kw.as_str() {
                            "small" | "x-small" | "xx-small" => ctx.font_size = FontSize::Small,
                            "medium" => ctx.font_size = FontSize::Normal,
                            "large" => ctx.font_size = FontSize::Large,
                            "x-large" | "xx-large" => ctx.font_size = FontSize::H1,
                            _ => {}
                        }
                        ctx.line_height = ctx.font_size.height() + 4;
                    }
                    _ => {}
                }
            }
            "font-weight" => {
                match &decl.value {
                    CssValue::Keyword(kw) if kw == "bold" => ctx.bold = true,
                    CssValue::Keyword(kw) if kw == "normal" => ctx.bold = false,
                    CssValue::Number(n) if *n >= 700.0 => ctx.bold = true,
                    _ => {}
                }
            }
            "font-style" => {
                if let CssValue::Keyword(kw) = &decl.value {
                    ctx.italic = kw == "italic" || kw == "oblique";
                }
            }
            "display" => {
                if let CssValue::Keyword(kw) = &decl.value {
                    if kw == "none" {
                        ctx.y = i32::MAX / 2;
                    }
                }
            }
            "text-align" => {
                if let CssValue::Keyword(kw) = &decl.value {
                    match kw.as_str() {
                        "center" => { ctx.x = ctx.max_width as i32 / 4; }
                        _ => {}
                    }
                }
            }
            "margin-top" | "padding-top" => {
                if let CssValue::Length(px, _) = &decl.value {
                    ctx.y += *px as i32;
                }
            }
            "margin-bottom" | "padding-bottom" => {
                // Handled after element rendering
            }
            "visibility" => {
                if let CssValue::Keyword(kw) = &decl.value {
                    if kw == "hidden" || kw == "collapse" {
                        ctx.y = i32::MAX / 2;
                    }
                }
            }
            _ => {} // Ignore unsupported properties
        }
    }
}

/// Apply CSS declarations from inline style="" attribute
fn apply_inline_style(ctx: &mut RenderContext, style_str: &str) {
    let declarations = css_parser::parse_inline_style(style_str);
    apply_declarations(ctx, &declarations);
}

/// Convert CSS value to ARGB color
fn css_value_to_color(value: &CssValue) -> Option<u32> {
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
