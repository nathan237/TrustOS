



use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;

use super::{Ia, HtmlNode, HtmlElement, Jy};
use super::css_parser::{self, CssValue, FontWeight, FontStyle as CssFontStyle, Ho, Fd, Kq, SelectorPart};
use crate::framebuffer;


const ND_: u32 = 0xFFFFFFFF;           
const CM_: u32 = 0xFF1A1A1A;         
const AQI_: u32 = 0xFF0066CC;         
const DJD_: u32 = 0xFF551A8B; 
const DJB_: u32 = 0xFF000000;      
const BPR_: u32 = 0xFFF5F5F5;      
const DIZ_: u32 = 0xFFD63384;         
const AQF_: u32 = 0xFFCCCCCC;           
const BQD_: u32 = 0xFF0066CC; 
const DJJ_: u32 = 0xFFF0F7FF;     


pub struct RenderContext {
    pub x: i32,
    pub y: i32,
    pub max_width: u32,
    pub line_height: i32,
    pub font_size: FontSize,
    pub bold: bool,
    pub italic: bool,
    pub in_link: Option<String>,
    pub links: Vec<Jy>,
    pub list_depth: i32,
    pub in_pre: bool,
    
    pub text_color: u32,
    
    pub bg_color: Option<u32>,
    
    pub opacity: f32,
    
    pub underline: bool,
    pub strikethrough: bool,
    
    pub list_counters: Vec<i32>,
    
    pub in_ordered_list: bool,
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
            text_color: CM_,
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


pub fn ofl(
    doc: &Ia,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scroll_y: i32,
) -> Vec<Jy> {
    
    framebuffer::fill_rect(x as u32, y as u32, width, height, ND_);
    
    let mut ab = RenderContext::new(width);
    ab.y = y - scroll_y;
    
    
    let hpa = hxn(&doc.nodes);
    let cea = if !hpa.is_empty() {
        css_parser::parse_stylesheet(&hpa)
    } else {
        Fd { rules: Vec::new() }
    };
    
    
    for uf in &doc.nodes {
        izt(&mut ab, uf, &cea, x, y, width, height);
    }
    
    ab.links
}


fn izt(
    ab: &mut RenderContext,
    uf: &HtmlNode,
    cea: &Fd,
    aex: i32,
    lg: i32,
    zt: u32,
    ur: u32,
) {
    match uf {
        HtmlNode::Text(text) => {
            ofr(ab, text, aex, lg, zt, ur);
        }
        HtmlNode::Element(el) => {
            ofj(ab, el, cea, aex, lg, zt, ur);
        }
    }
}


fn ofr(
    ab: &mut RenderContext,
    text: &str,
    aex: i32,
    lg: i32,
    zt: u32,
    ur: u32,
) {
    let color = if ab.in_link.is_some() { AQI_ } else { ab.text_color };
    
    
    if ab.in_pre {
        for line in text.lines() {
            ofm(ab, line, color, aex, lg, zt, ur);
            ab.newline();
        }
        return;
    }
    
    
    let um: Vec<&str> = text.split_whitespace().collect();
    
    for fx in um {
        let puz = fx.len() as i32 * 8; 
        
        
        if ab.x + puz > (aex + zt as i32 - 16) && ab.x > 16 + (ab.list_depth * 24) {
            ab.newline();
        }
        
        ofw(ab, fx, color, aex, lg, zt, ur);
        ab.space();
    }
}

fn ofw(
    ab: &mut RenderContext,
    fx: &str,
    color: u32,
    aex: i32,
    lg: i32,
    zt: u32,
    ur: u32,
) {
    
    if ab.y + ab.line_height < lg || ab.y > lg + ur as i32 {
        ab.x += fx.len() as i32 * 8;
        return;
    }
    
    
    let cbk = ab.x;
    
    
    for c in fx.chars() {
        if ab.x >= aex && ab.x < aex + zt as i32 - 8 {
            draw_char(ab.x as u32, ab.y as u32, c, color);
        }
        ab.x += 8;
    }
    
    
    if let Some(href) = &ab.in_link {
        ab.links.push(Jy {
            href: href.clone(),
            x: cbk,
            y: ab.y,
            width: (ab.x - cbk) as u32,
            height: ab.line_height as u32,
        });
        
        
        if ab.y >= lg && ab.y < lg + ur as i32 {
            framebuffer::fill_rect(
                cbk as u32,
                (ab.y + ab.line_height - 2) as u32,
                (ab.x - cbk) as u32,
                1,
                AQI_,
            );
        }
    }
    
    
    if ab.underline && ab.in_link.is_none() {
        let w = (ab.x - cbk) as u32;
        if w > 0 && ab.y >= lg && ab.y < lg + ur as i32 {
            framebuffer::fill_rect(
                cbk as u32,
                (ab.y + ab.line_height - 2) as u32,
                w, 1, color,
            );
        }
    }
    
    
    if ab.strikethrough {
        let w = (ab.x - cbk) as u32;
        if w > 0 && ab.y >= lg && ab.y < lg + ur as i32 {
            framebuffer::fill_rect(
                cbk as u32,
                (ab.y + ab.line_height / 2) as u32,
                w, 1, ab.text_color,
            );
        }
    }
}

fn ofm(
    ab: &mut RenderContext,
    line: &str,
    color: u32,
    aex: i32,
    lg: i32,
    zt: u32,
    ur: u32,
) {
    if ab.y + ab.line_height < lg || ab.y > lg + ur as i32 {
        return;
    }
    
    for c in line.chars() {
        if ab.x >= aex && ab.x < aex + zt as i32 - 8 {
            draw_char(ab.x as u32, ab.y as u32, c, color);
        }
        ab.x += 8;
    }
}


fn ofj(
    ab: &mut RenderContext,
    el: &HtmlElement,
    cea: &Fd,
    aex: i32,
    lg: i32,
    zt: u32,
    ur: u32,
) {
    let tag = el.tag.as_str();
    
    
    if matches!(tag, "head" | "script" | "style" | "meta" | "link" | "title") {
        return;
    }
    
    
    let ezk = ab.font_size;
    let jcq = ab.bold;
    let jcr = ab.in_link.clone();
    let jcu = ab.in_pre;
    let jcw = ab.text_color;
    let jcp = ab.bg_color;
    let jcs = ab.opacity;
    let jcx = ab.underline;
    let jcv = ab.strikethrough;
    let jct = ab.in_ordered_list;
    
    
    if let Some(style_str) = el.attr("style") {
        jxd(ab, style_str);
    }
    
    
    jxf(ab, el, cea);
    
    
    if let Some(fnu) = el.attr("color") {
        if let Some(c) = gmk(fnu) {
            ab.text_color = c;
        }
    }
    if let Some(bgcolor_str) = el.attr("bgcolor") {
        if let Some(c) = gmk(bgcolor_str) {
            ab.bg_color = Some(c);
        }
    }
    
    
    if ab.y >= i32::MAX / 4 {
        ab.font_size = ezk;
        ab.line_height = ezk.height() + 4;
        ab.bold = jcq;
        ab.in_link = jcr;
        ab.in_pre = jcu;
        ab.text_color = jcw;
        ab.bg_color = jcp;
        ab.opacity = jcs;
        ab.underline = jcx;
        ab.strikethrough = jcv;
        ab.in_ordered_list = jct;
        return;
    }
    
    
    match tag {
        
        "html" | "body" | "div" | "section" | "article" | "nav" | "header" | "footer" | "main" | 
        "noscript" | "span" | "form" | "label" | "fieldset" | "legend" | "details" | "summary" |
        "figure" | "figcaption" | "aside" | "dialog" | "abbr" | "address" | "cite" | "dfn" |
        "ruby" | "rt" | "rp" | "data" | "time" | "var" | "samp" | "kbd" | "wbr" | "bdi" | "bdo" => {
            
        }
        
        "p" => {
            ab.newline();
            ab.y += 8; 
        }
        
        "br" => {
            ab.newline();
        }
        
        "hr" => {
            ab.newline();
            ab.y += 8;
            if ab.y >= lg && ab.y < lg + ur as i32 {
                framebuffer::fill_rect(
                    (aex + 16) as u32,
                    ab.y as u32,
                    zt - 32,
                    1,
                    AQF_,
                );
            }
            ab.y += 16;
            ab.x = 16;
        }
        
        "h1" => {
            ab.newline();
            ab.y += 16;
            ab.font_size = FontSize::H1;
            ab.line_height = 40;
            ab.bold = true;
        }
        
        "h2" => {
            ab.newline();
            ab.y += 12;
            ab.font_size = FontSize::H2;
            ab.line_height = 32;
            ab.bold = true;
        }
        
        "h3" | "h4" | "h5" | "h6" => {
            ab.newline();
            ab.y += 8;
            ab.font_size = FontSize::H3;
            ab.line_height = 26;
            ab.bold = true;
        }
        
        "a" => {
            if let Some(href) = el.attr("href") {
                ab.in_link = Some(href.to_string());
            }
        }
        
        "strong" | "b" => {
            ab.bold = true;
        }
        
        "em" | "i" => {
            ab.italic = true;
        }
        
        "code" => {
            
        }
        
        "pre" => {
            ab.newline();
            ab.y += 8;
            ab.in_pre = true;
            
            if ab.y >= lg {
                framebuffer::fill_rect(
                    (aex + 8) as u32,
                    ab.y as u32,
                    zt - 16,
                    100, 
                    BPR_,
                );
            }
        }
        
        "blockquote" => {
            ab.newline();
            ab.y += 8;
            ab.list_depth += 1;
            
            if ab.y >= lg {
                framebuffer::fill_rect(
                    (aex + 12) as u32,
                    ab.y as u32,
                    4,
                    80,
                    BQD_,
                );
            }
        }
        
        "ul" | "ol" => {
            ab.newline();
            ab.list_depth += 1;
            if tag == "ol" {
                ab.in_ordered_list = true;
                ab.list_counters.push(0);
            }
        }
        
        "li" => {
            ab.newline();
            
            if ab.y >= lg && ab.y < lg + ur as i32 {
                if ab.in_ordered_list {
                    
                    if let Some(counter) = ab.list_counters.last_mut() {
                        *counter += 1;
                        let rw = alloc::format!("{}.", counter);
                        let nlz = ab.x - 20;
                        for (i, c) in rw.chars().enumerate() {
                            draw_char((nlz + i as i32 * 8) as u32, ab.y as u32, c, CM_);
                        }
                    }
                } else {
                    
                    let kge = ab.x - 12;
                    framebuffer::fill_rect(
                        kge as u32,
                        (ab.y + 6) as u32,
                        4,
                        4,
                        CM_,
                    );
                }
            }
        }
        
        "img" => {
            
            let adf = el.attr("alt").unwrap_or("");
            let src = el.attr("src").unwrap_or("");
            
            
            let czn = el.attr("width")
                .and_then(|w| w.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(120)
                .min(zt.saturating_sub(32))
                .max(40);
            let bti = el.attr("height")
                .and_then(|h| h.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(60)
                .min(300)
                .max(20);
            
            if ab.y >= lg && ab.y < lg + ur as i32 {
                
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, czn, bti, 0xFFF0F0F0);
                
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, czn, 1, 0xFFDDDDDD);
                framebuffer::fill_rect(ab.x as u32, (ab.y + bti as i32 - 1) as u32, czn, 1, 0xFFDDDDDD);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, bti, 0xFFDDDDDD);
                framebuffer::fill_rect((ab.x + czn as i32 - 1) as u32, ab.y as u32, 1, bti, 0xFFDDDDDD);
                
                
                let mni = "[IMG]";
                let adt = ab.x + (czn as i32 / 2) - 20;
                let adu = ab.y + (bti as i32 / 2) - 12;
                if bti > 24 {
                    for (i, c) in mni.chars().enumerate() {
                        draw_char((adt + i as i32 * 8) as u32, adu as u32, c, 0xFF999999);
                    }
                }
                
                
                let bga = if !adf.is_empty() {
                    adf
                } else if !src.is_empty() {
                    src.rsplit('/').next().unwrap_or(src)
                } else {
                    ""
                };
                if !bga.is_empty() && bti > 40 {
                    let nd = (czn / 8).saturating_sub(2) as usize;
                    let kd = ab.x + 8;
                    let ie = ab.y + (bti as i32 / 2) + 4;
                    for (i, c) in bga.chars().take(nd).enumerate() {
                        draw_char((kd + i as i32 * 8) as u32, ie as u32, c, 0xFF666666);
                    }
                }
            }
            ab.y += bti as i32 + 4;
        }
        
        "table" => {
            ab.newline();
            ab.y += 8;
        }
        
        "tr" => {
            ab.newline();
        }
        
        "td" | "th" => {
            ab.x += 16;
        }
        
        "input" => {
            let mqo = el.attr("type").unwrap_or("text");
            match mqo {
                "hidden" => {
                    return;
                }
                "submit" | "button" => {
                    
                    let value = el.attr("value").unwrap_or("Submit");
                    let dju = (value.len() as u32 * 8) + 24;
                    let bqw = 28u32;
                    if ab.y >= lg && ab.y < lg + ur as i32 {
                        framebuffer::fill_rect(ab.x as u32, ab.y as u32, dju, bqw, 0xFFE8E8E8);
                        
                        framebuffer::fill_rect(ab.x as u32, ab.y as u32, dju, 1, 0xFFBBBBBB);
                        framebuffer::fill_rect(ab.x as u32, (ab.y + bqw as i32 - 1) as u32, dju, 1, 0xFFBBBBBB);
                        framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, bqw, 0xFFBBBBBB);
                        framebuffer::fill_rect((ab.x + dju as i32 - 1) as u32, ab.y as u32, 1, bqw, 0xFFBBBBBB);
                        for (i, c) in value.chars().enumerate() {
                            draw_char((ab.x + 12 + i as i32 * 8) as u32, (ab.y + 6) as u32, c, CM_);
                        }
                    }
                    ab.x += dju as i32 + 8;
                }
                "checkbox" => {
                    if ab.y >= lg && ab.y < lg + ur as i32 {
                        framebuffer::fill_rect(ab.x as u32, (ab.y + 2) as u32, 14, 14, 0xFFFFFFFF);
                        framebuffer::fill_rect(ab.x as u32, (ab.y + 2) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(ab.x as u32, (ab.y + 15) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(ab.x as u32, (ab.y + 2) as u32, 1, 14, 0xFF999999);
                        framebuffer::fill_rect((ab.x + 13) as u32, (ab.y + 2) as u32, 1, 14, 0xFF999999);
                        if el.attr("checked").is_some() {
                            framebuffer::fill_rect((ab.x + 3) as u32, (ab.y + 8) as u32, 8, 2, CM_);
                            framebuffer::fill_rect((ab.x + 3) as u32, (ab.y + 5) as u32, 2, 5, CM_);
                        }
                    }
                    ab.x += 20;
                }
                "radio" => {
                    if ab.y >= lg && ab.y < lg + ur as i32 {
                        framebuffer::fill_rect(ab.x as u32, (ab.y + 2) as u32, 14, 14, 0xFFFFFFFF);
                        framebuffer::fill_rect(ab.x as u32, (ab.y + 2) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(ab.x as u32, (ab.y + 15) as u32, 14, 1, 0xFF999999);
                        framebuffer::fill_rect(ab.x as u32, (ab.y + 2) as u32, 1, 14, 0xFF999999);
                        framebuffer::fill_rect((ab.x + 13) as u32, (ab.y + 2) as u32, 1, 14, 0xFF999999);
                        if el.attr("checked").is_some() {
                            framebuffer::fill_rect((ab.x + 4) as u32, (ab.y + 6) as u32, 6, 6, CM_);
                        }
                    }
                    ab.x += 20;
                }
                _ => {
                    
                    let placeholder = el.attr("placeholder").unwrap_or("");
                    let value = el.attr("value").unwrap_or("");
                    let bga = if value.is_empty() { placeholder } else { value };
                    let cax = 200u32.min(zt.saturating_sub(40));
                    let eqt = 26u32;
                    if ab.y >= lg && ab.y < lg + ur as i32 {
                        framebuffer::fill_rect(ab.x as u32, ab.y as u32, cax, eqt, 0xFFFFFFFF);
                        framebuffer::fill_rect(ab.x as u32, ab.y as u32, cax, 1, 0xFF999999);
                        framebuffer::fill_rect(ab.x as u32, (ab.y + eqt as i32 - 1) as u32, cax, 1, 0xFF999999);
                        framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, eqt, 0xFF999999);
                        framebuffer::fill_rect((ab.x + cax as i32 - 1) as u32, ab.y as u32, 1, eqt, 0xFF999999);
                        let text_color = if value.is_empty() { 0xFF999999 } else { CM_ };
                        for (i, c) in bga.chars().take((cax / 8 - 2) as usize).enumerate() {
                            draw_char((ab.x + 4 + i as i32 * 8) as u32, (ab.y + 5) as u32, c, text_color);
                        }
                    }
                    ab.x += cax as i32 + 8;
                }
            }
        }
        
        "button" => {
            
            let bqw = 28u32;
            if ab.y >= lg && ab.y < lg + ur as i32 {
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 120, bqw, 0xFFE8E8E8);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 120, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(ab.x as u32, (ab.y + bqw as i32 - 1) as u32, 120, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, bqw, 0xFFBBBBBB);
                framebuffer::fill_rect(119 + ab.x as u32, ab.y as u32, 1, bqw, 0xFFBBBBBB);
            }
            ab.y += 6;
        }
        
        "textarea" => {
            ab.newline();
            let fch = core::cmp::min(zt.saturating_sub(32), 400);
            let ebd = 80u32;
            if ab.y >= lg && ab.y < lg + ur as i32 {
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, fch, ebd, 0xFFFFFFFF);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, fch, 1, 0xFF999999);
                framebuffer::fill_rect(ab.x as u32, (ab.y + ebd as i32 - 1) as u32, fch, 1, 0xFF999999);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, ebd, 0xFF999999);
                framebuffer::fill_rect((ab.x + fch as i32 - 1) as u32, ab.y as u32, 1, ebd, 0xFF999999);
            }
            ab.y += ebd as i32 + 8;
        }
        
        "select" => {
            let cqh = 160u32.min(zt.saturating_sub(32));
            let ezz = 26u32;
            if ab.y >= lg && ab.y < lg + ur as i32 {
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, cqh, ezz, 0xFFFFFFFF);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, cqh, 1, 0xFF999999);
                framebuffer::fill_rect(ab.x as u32, (ab.y + ezz as i32 - 1) as u32, cqh, 1, 0xFF999999);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, ezz, 0xFF999999);
                framebuffer::fill_rect((ab.x + cqh as i32 - 1) as u32, ab.y as u32, 1, ezz, 0xFF999999);
                
                framebuffer::fill_rect((ab.x + cqh as i32 - 16) as u32, (ab.y + 10) as u32, 8, 2, 0xFF666666);
                framebuffer::fill_rect((ab.x + cqh as i32 - 14) as u32, (ab.y + 12) as u32, 4, 2, 0xFF666666);
            }
            ab.x += cqh as i32 + 8;
        }
        
        "option" | "optgroup" => {
            
            return;
        }
        
        "small" | "sub" | "sup" => {
            ab.font_size = FontSize::Small;
            ab.line_height = FontSize::Small.height() + 4;
        }
        
        "mark" => {
            ab.bg_color = Some(0xFFFFFF00); 
        }
        
        "del" | "s" | "strike" => {
            ab.text_color = 0xFF999999;
            ab.strikethrough = true;
        }
        
        "u" | "ins" => {
            ab.underline = true;
        }
        
        "progress" => {
            
            let sh: f32 = el.attr("max").and_then(|v| v.parse().ok()).unwrap_or(1.0);
            let dlw: f32 = el.attr("value").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let aed = (dlw / sh).min(1.0).max(0.0);
            let ek = 200u32.min(zt.saturating_sub(40));
            let hs = 18u32;
            if ab.y >= lg && ab.y < lg + ur as i32 {
                
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, ek, hs, 0xFFE0E0E0);
                
                let rb = (ek as f32 * aed) as u32;
                if rb > 0 {
                    framebuffer::fill_rect(ab.x as u32, ab.y as u32, rb, hs, 0xFF4CAF50);
                }
                
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, ek, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(ab.x as u32, (ab.y + hs as i32 - 1) as u32, ek, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, hs, 0xFFBBBBBB);
                framebuffer::fill_rect((ab.x + ek as i32 - 1) as u32, ab.y as u32, 1, hs, 0xFFBBBBBB);
                
                let ewk = alloc::format!("{}%", (aed * 100.0) as u32);
                let kd = ab.x + (ek as i32 / 2) - (ewk.len() as i32 * 4);
                for (i, c) in ewk.chars().enumerate() {
                    draw_char((kd + i as i32 * 8) as u32, (ab.y + 1) as u32, c, CM_);
                }
            }
            ab.x += ek as i32 + 8;
            return; 
        }
        
        "meter" => {
            
            let duj: f32 = el.attr("min").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let sh: f32 = el.attr("max").and_then(|v| v.parse().ok()).unwrap_or(1.0);
            let dlw: f32 = el.attr("value").and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let low: f32 = el.attr("low").and_then(|v| v.parse().ok()).unwrap_or(duj);
            let high: f32 = el.attr("high").and_then(|v| v.parse().ok()).unwrap_or(sh);
            let range = sh - duj;
            let aed = if range > 0.0 { ((dlw - duj) / range).min(1.0).max(0.0) } else { 0.0 };
            let ek = 160u32.min(zt.saturating_sub(40));
            let hs = 16u32;
            
            let bso = if dlw < low {
                0xFFFF5722 
            } else if dlw > high {
                0xFFFF5722 
            } else {
                0xFF4CAF50 
            };
            if ab.y >= lg && ab.y < lg + ur as i32 {
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, ek, hs, 0xFFE0E0E0);
                let rb = (ek as f32 * aed) as u32;
                if rb > 0 {
                    framebuffer::fill_rect(ab.x as u32, ab.y as u32, rb, hs, bso);
                }
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, ek, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(ab.x as u32, (ab.y + hs as i32 - 1) as u32, ek, 1, 0xFFBBBBBB);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, hs, 0xFFBBBBBB);
                framebuffer::fill_rect((ab.x + ek as i32 - 1) as u32, ab.y as u32, 1, hs, 0xFFBBBBBB);
            }
            ab.x += ek as i32 + 8;
            return;
        }
        
        "dl" => {
            ab.newline();
        }
        "dt" => {
            ab.newline();
            ab.bold = true;
        }
        "dd" => {
            ab.newline();
            ab.x += 40; 
        }
        
        "details" => {
            ab.newline();
            ab.y += 4;
            
            if ab.y >= lg && ab.y < lg + ur as i32 {
                let dfu = ab.x;
                let open = el.attr("open").is_some();
                if open {
                    
                    framebuffer::fill_rect(dfu as u32, ab.y as u32, 8, 2, CM_);
                    framebuffer::fill_rect((dfu + 1) as u32, (ab.y + 2) as u32, 6, 2, CM_);
                    framebuffer::fill_rect((dfu + 2) as u32, (ab.y + 4) as u32, 4, 2, CM_);
                } else {
                    
                    framebuffer::fill_rect(dfu as u32, ab.y as u32, 2, 8, CM_);
                    framebuffer::fill_rect((dfu + 2) as u32, (ab.y + 1) as u32, 2, 6, CM_);
                    framebuffer::fill_rect((dfu + 4) as u32, (ab.y + 2) as u32, 2, 4, CM_);
                }
            }
            ab.x += 16; 
        }
        
        "summary" => {
            ab.bold = true;
        }
        
        "figure" => {
            ab.newline();
            ab.y += 8;
            ab.list_depth += 1;
        }
        
        "figcaption" => {
            ab.newline();
            ab.font_size = FontSize::Small;
            ab.line_height = FontSize::Small.height() + 4;
            ab.text_color = 0xFF666666;
        }
        
        "nav" => {
            ab.bg_color = Some(0xFFF8F8F8);
        }
        
        "footer" => {
            ab.newline();
            ab.y += 16;
            if ab.y >= lg && ab.y < lg + ur as i32 {
                framebuffer::fill_rect(
                    (aex + 16) as u32, ab.y as u32, zt - 32, 1, AQF_,
                );
            }
            ab.y += 8;
            ab.font_size = FontSize::Small;
            ab.line_height = FontSize::Small.height() + 4;
            ab.text_color = 0xFF666666;
        }
        
        "header" => {
            ab.bg_color = Some(0xFFF0F0F0);
        }
        
        "main" | "article" | "section" | "aside" => {
            
        }
        
        "video" | "audio" | "canvas" | "svg" | "iframe" | "embed" | "object" => {
            
            let dcs = el.attr("width")
                .and_then(|w| w.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(320)
                .min(zt.saturating_sub(32));
            let cof = el.attr("height")
                .and_then(|h| h.trim_end_matches("px").parse::<u32>().ok())
                .unwrap_or(180)
                .min(400);
            if ab.y >= lg && ab.y < lg + ur as i32 {
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, dcs, cof, 0xFF2C2C2C);
                
                let label = match tag {
                    "video" => "[VIDEO]",
                    "audio" => "[AUDIO]",
                    "canvas" => "[CANVAS]",
                    "svg" => "[SVG]",
                    "iframe" => "[IFRAME]",
                    _ => "[EMBED]",
                };
                let fe = ab.x + (dcs as i32 / 2) - (label.len() as i32 * 4);
                let ly = ab.y + (cof as i32 / 2) - 8;
                for (i, c) in label.chars().enumerate() {
                    draw_char((fe + i as i32 * 8) as u32, ly as u32, c, 0xFFAAAAAA);
                }
                
                if tag == "video" || tag == "audio" {
                    let cx = ab.x + dcs as i32 / 2;
                    let u = ab.y + cof as i32 / 2 + 12;
                    for row in 0..16 {
                        let w = 16 - row;
                        framebuffer::fill_rect((cx - 4) as u32, (u + row) as u32, w as u32, 1, 0xFFFFFFFF);
                    }
                }
                
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, dcs, 1, 0xFF555555);
                framebuffer::fill_rect(ab.x as u32, (ab.y + cof as i32 - 1) as u32, dcs, 1, 0xFF555555);
                framebuffer::fill_rect(ab.x as u32, ab.y as u32, 1, cof, 0xFF555555);
                framebuffer::fill_rect((ab.x + dcs as i32 - 1) as u32, ab.y as u32, 1, cof, 0xFF555555);
            }
            ab.y += cof as i32 + 4;
            return; 
        }
        
        "center" => {
            ab.newline();
            ab.x = aex + (zt as i32 / 4);
        }
        
        _ => {}
    }
    
    
    for pd in &el.children {
        izt(ab, pd, cea, aex, lg, zt, ur);
    }
    
    
    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            ab.newline();
            ab.y += 8;
        }
        "p" => {
            ab.newline();
            ab.y += 8;
        }
        "ul" | "ol" => {
            ab.list_depth -= 1;
            ab.list_counters.pop();
            ab.newline();
        }
        "blockquote" => {
            ab.list_depth -= 1;
            ab.newline();
        }
        "pre" => {
            ab.newline();
        }
        "table" => {
            ab.newline();
        }
        "dl" => {
            ab.newline();
            ab.y += 4;
        }
        "dt" | "dd" => {
            ab.newline();
        }
        "details" => {
            ab.newline();
            ab.y += 4;
        }
        "figure" => {
            ab.list_depth -= 1;
            ab.newline();
            ab.y += 8;
        }
        "footer" | "header" | "nav" => {
            ab.newline();
        }
        _ => {}
    }
    
    ab.font_size = ezk;
    ab.line_height = ezk.height() + 4;
    ab.bold = jcq;
    ab.in_link = jcr;
    ab.in_pre = jcu;
    ab.text_color = jcw;
    ab.bg_color = jcp;
    ab.opacity = jcs;
    ab.underline = jcx;
    ab.strikethrough = jcv;
    ab.in_ordered_list = jct;
}


fn hxn(nodes: &[HtmlNode]) -> String {
    let mut blj = String::new();
    for uf in nodes {
        if let HtmlNode::Element(el) = uf {
            if el.tag == "style" {
                
                for pd in &el.children {
                    if let HtmlNode::Text(text) = pd {
                        blj.push_str(text);
                        blj.push('\n');
                    }
                }
            } else {
                
                let hku = hxn(&el.children);
                if !hku.is_empty() {
                    blj.push_str(&hku);
                }
            }
        }
    }
    blj
}


fn ncj(selector: &Kq, bse: &HtmlElement) -> bool {
    let au = &selector.elements;
    if au.is_empty() {
        return false;
    }
    
    
    
    let mut gto = 0;
    let mut gfa: Option<usize> = None;
    let mut idm = false;
    
    for (i, jn) in au.iter().enumerate() {
        match jn {
            SelectorPart::Descendant | SelectorPart::Child | SelectorPart::Adjacent | SelectorPart::Sibling => {
                idm = true;
                gto = i + 1;
                gfa = None;
            }
            SelectorPart::Tag(_) => {
                if !idm {
                    if let Some(_prev) = gfa {
                        
                        gto = i;
                    }
                }
                gfa = Some(i);
            }
            _ => {}
        }
    }
    
    let segment = &au[gto..];
    
    for jn in segment {
        match jn {
            SelectorPart::Tag(tag) => {
                if bse.tag != *tag {
                    return false;
                }
            }
            SelectorPart::Class(class) => {
                let los = bse.attr("class").unwrap_or("");
                if !los.split_whitespace().any(|c| c == class.as_str()) {
                    return false;
                }
            }
            SelectorPart::Id(id) => {
                let lot = bse.attr("id").unwrap_or("");
                if lot != id.as_str() {
                    return false;
                }
            }
            SelectorPart::Universal => {}
            SelectorPart::Attribute(attr, val) => {
                match bse.attr(attr) {
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
            SelectorPart::Pseudo(_) => {} 
            _ => {} 
        }
    }
    
    !segment.is_empty()
}


fn jxf(ab: &mut RenderContext, el: &HtmlElement, cea: &Fd) {
    for qo in &cea.rules {
        let nci = qo.selectors.iter().any(|sel| ncj(sel, el));
        if nci {
            hfn(ab, &qo.declarations);
        }
    }
}


fn hfn(ab: &mut RenderContext, declarations: &[Ho]) {
    for decl in declarations {
        match decl.property.as_str() {
            "color" => {
                if let Some(c) = hpb(&decl.value) {
                    ab.text_color = c;
                }
            }
            "background-color" | "background" => {
                if let Some(c) = hpb(&decl.value) {
                    ab.bg_color = Some(c);
                }
            }
            "font-size" => {
                match &decl.value {
                    CssValue::Length(p, _) => {
                        if *p <= 14.0 {
                            ab.font_size = FontSize::Small;
                        } else if *p <= 18.0 {
                            ab.font_size = FontSize::Normal;
                        } else if *p <= 22.0 {
                            ab.font_size = FontSize::Large;
                        } else if *p <= 28.0 {
                            ab.font_size = FontSize::H3;
                        } else if *p <= 30.0 {
                            ab.font_size = FontSize::H2;
                        } else {
                            ab.font_size = FontSize::H1;
                        }
                        ab.line_height = ab.font_size.height() + 4;
                    }
                    CssValue::Keyword(li) => {
                        match li.as_str() {
                            "small" | "x-small" | "xx-small" => ab.font_size = FontSize::Small,
                            "medium" => ab.font_size = FontSize::Normal,
                            "large" => ab.font_size = FontSize::Large,
                            "x-large" | "xx-large" => ab.font_size = FontSize::H1,
                            _ => {}
                        }
                        ab.line_height = ab.font_size.height() + 4;
                    }
                    _ => {}
                }
            }
            "font-weight" => {
                match &decl.value {
                    CssValue::Keyword(li) if li == "bold" => ab.bold = true,
                    CssValue::Keyword(li) if li == "normal" => ab.bold = false,
                    CssValue::Number(ae) if *ae >= 700.0 => ab.bold = true,
                    _ => {}
                }
            }
            "font-style" => {
                if let CssValue::Keyword(li) = &decl.value {
                    ab.italic = li == "italic" || li == "oblique";
                }
            }
            "display" => {
                if let CssValue::Keyword(li) = &decl.value {
                    if li == "none" {
                        ab.y = i32::MAX / 2;
                    }
                }
            }
            "text-align" => {
                if let CssValue::Keyword(li) = &decl.value {
                    match li.as_str() {
                        "center" => { ab.x = ab.max_width as i32 / 4; }
                        _ => {}
                    }
                }
            }
            "margin-top" | "padding-top" => {
                if let CssValue::Length(p, _) = &decl.value {
                    ab.y += *p as i32;
                }
            }
            "margin-bottom" | "padding-bottom" => {
                
            }
            "visibility" => {
                if let CssValue::Keyword(li) = &decl.value {
                    if li == "hidden" || li == "collapse" {
                        ab.y = i32::MAX / 2;
                    }
                }
            }
            "opacity" => {
                match &decl.value {
                    CssValue::Number(ae) => {
                        ab.opacity = ae.max(0.0).min(1.0) as f32;
                    }
                    _ => {}
                }
            }
            "text-decoration" | "text-decoration-line" => {
                if let CssValue::Keyword(li) = &decl.value {
                    match li.as_str() {
                        "underline" => ab.underline = true,
                        "line-through" => ab.strikethrough = true,
                        "none" => {
                            ab.underline = false;
                            ab.strikethrough = false;
                        }
                        _ => {}
                    }
                }
            }
            "text-transform" => {
                
                if let CssValue::Keyword(li) = &decl.value {
                    match li.as_str() {
                        "uppercase" | "lowercase" | "capitalize" | "none" => {
                            
                        }
                        _ => {}
                    }
                }
            }
            "line-height" => {
                match &decl.value {
                    CssValue::Length(p, _) => {
                        ab.line_height = *p as i32;
                    }
                    CssValue::Number(ae) => {
                        ab.line_height = (*ae as i32) * ab.font_size.height();
                    }
                    _ => {}
                }
            }
            "margin-left" | "padding-left" => {
                if let CssValue::Length(p, _) = &decl.value {
                    ab.x += *p as i32;
                }
            }
            "border" | "border-top" | "border-bottom" | "border-left" | "border-right" => {
                
            }
            _ => {} 
        }
    }
}


fn jxd(ab: &mut RenderContext, style_str: &str) {
    let declarations = css_parser::nqq(style_str);
    hfn(ab, &declarations);
}


fn hpb(value: &CssValue) -> Option<u32> {
    match value {
        CssValue::Color(c) => Some(*c),
        CssValue::Keyword(name) => gmk(name),
        _ => None,
    }
}


fn gmk(j: &str) -> Option<u32> {
    let j = j.trim();
    
    
    if j.starts_with('#') {
        let ga = &j[1..];
        if ga.len() == 6 {
            let r = u8::from_str_radix(&ga[0..2], 16).ok()?;
            let g = u8::from_str_radix(&ga[2..4], 16).ok()?;
            let b = u8::from_str_radix(&ga[4..6], 16).ok()?;
            return Some(0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32);
        }
        if ga.len() == 3 {
            let r = u8::from_str_radix(&ga[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&ga[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&ga[2..3], 16).ok()? * 17;
            return Some(0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32);
        }
        return None;
    }
    
    
    match j.to_lowercase().as_str() {
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


fn draw_char(x: u32, y: u32, c: char, color: u32) {
    let font = mcs(c);
    
    for (row, byte) in font.iter().enumerate() {
        for bf in 0..8 {
            if (byte >> (7 - bf)) & 1 != 0 {
                framebuffer::put_pixel(x + bf, y + row as u32, color);
            }
        }
    }
}


fn mcs(c: char) -> [u8; 16] {
    
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
        _ => [0x00,0x3C,0x42,0x42,0x42,0x42,0x42,0x3C,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00], 
    }
}
