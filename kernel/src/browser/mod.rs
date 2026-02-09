//! TrustOS Web Browser
//!
//! A web browser that can render HTML pages with CSS styling and basic JavaScript.
//! Designed after modern open-source browsers (Servo, Firefox).
//!
//! Features:
//! - HTML5 parsing
//! - CSS3 styling (partial)
//! - JavaScript ES5 (basic)
//! - HTTP/HTTPS networking
//! - Form handling
//! - Image display (basic)

pub mod html_parser;
pub mod css_parser;
pub mod js_engine;
pub mod renderer;
pub mod url;

pub use html_parser::*;
pub use css_parser::*;
pub use js_engine::*;
pub use renderer::*;
pub use url::*;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Browser state
pub struct Browser {
    pub current_url: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub document: Option<HtmlDocument>,
    pub scroll_y: i32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub status: BrowserStatus,
    pub links: Vec<HtmlLink>,
    /// Raw HTML source of current page
    pub raw_html: String,
    /// Toggle between parsed and raw view
    pub show_raw_html: bool,
    /// Cached external resources (URL -> data)
    pub resources: BTreeMap<String, Resource>,
    /// Resources currently being loaded
    pub pending_resources: Vec<String>,
}

/// External resource (image, CSS, etc.)
#[derive(Clone)]
pub struct Resource {
    pub content_type: ResourceType,
    pub data: Vec<u8>,
}

/// Resource types
#[derive(Clone, PartialEq)]
pub enum ResourceType {
    Image,
    Stylesheet,
    Script,
    Other,
}

/// Browser status
#[derive(Clone, PartialEq)]
pub enum BrowserStatus {
    Idle,
    Loading,
    Error(String),
    Ready,
}

/// A clickable link in the rendered page
#[derive(Clone)]
pub struct HtmlLink {
    pub href: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Browser {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            current_url: String::new(),
            history: Vec::new(),
            history_index: 0,
            document: None,
            scroll_y: 0,
            viewport_width: width,
            viewport_height: height,
            status: BrowserStatus::Idle,
            links: Vec::new(),
            raw_html: String::new(),
            show_raw_html: false,
            resources: BTreeMap::new(),
            pending_resources: Vec::new(),
        }
    }
    
    /// Toggle between parsed and raw HTML view
    pub fn toggle_view_mode(&mut self) {
        self.show_raw_html = !self.show_raw_html;
        crate::serial_println!("[BROWSER] View mode: {}", if self.show_raw_html { "RAW" } else { "PARSED" });
    }
    
    /// Navigate to URL
    pub fn navigate(&mut self, url: &str) -> Result<(), &'static str> {
        self.status = BrowserStatus::Loading;
        
        // Normalize URL
        let full_url = normalize_url(url, &self.current_url);
        
        crate::serial_println!("[BROWSER] Navigating to: {}", full_url);
        
        // Fetch the page
        let response = crate::netstack::http::get(&full_url)?;
        
        if response.status_code >= 400 {
            self.status = BrowserStatus::Error(alloc::format!("HTTP {}", response.status_code));
            return Err("HTTP error");
        }
        
        // Handle redirects
        if response.status_code >= 300 && response.status_code < 400 {
            if let Some(location) = response.header("Location") {
                return self.navigate(location);
            }
        }
        
        // Store raw HTML
        let html = response.body_str().unwrap_or("");
        self.raw_html = html.to_string();
        
        // Parse HTML
        self.document = Some(parse_html(html));
        
        // Extract and queue external resources
        self.extract_resources(&full_url);
        
        // Update history
        if self.history_index < self.history.len() {
            self.history.truncate(self.history_index);
        }
        self.history.push(full_url.clone());
        self.history_index = self.history.len();
        
        self.current_url = full_url;
        self.scroll_y = 0;
        self.status = BrowserStatus::Ready;
        
        Ok(())
    }
    
    /// Extract external resources from the document
    fn extract_resources(&mut self, base_url: &str) {
        self.pending_resources.clear();
        self.resources.clear();
        
        if let Some(ref doc) = self.document {
            let resources = collect_resources(&doc.nodes);
            for (tag, url) in resources {
                let full_url = normalize_url(&url, base_url);
                crate::serial_println!("[BROWSER] Found {} resource: {}", tag, full_url);
                self.pending_resources.push(full_url);
            }
        }
    }
    
    /// Load pending external resources (call this periodically or in background)
    pub fn load_pending_resources(&mut self) {
        let resources_to_load: Vec<String> = self.pending_resources.drain(..).collect();
        
        for url in resources_to_load {
            if self.resources.contains_key(&url) {
                continue; // Already loaded
            }
            
            crate::serial_println!("[BROWSER] Loading resource: {}", url);
            
            match crate::netstack::http::get(&url) {
                Ok(response) => {
                    if response.status_code == 200 {
                        let content_type = response.header("Content-Type").unwrap_or("");
                        let resource_type = if content_type.contains("image") {
                            ResourceType::Image
                        } else if content_type.contains("css") {
                            ResourceType::Stylesheet
                        } else if content_type.contains("javascript") {
                            ResourceType::Script
                        } else {
                            ResourceType::Other
                        };
                        
                        crate::serial_println!("[BROWSER] Loaded {} ({} bytes)", url, response.body.len());
                        self.resources.insert(url, Resource {
                            content_type: resource_type,
                            data: response.body,
                        });
                    }
                }
                Err(e) => {
                    crate::serial_println!("[BROWSER] Failed to load {}: {}", url, e);
                }
            }
        }
    }
    
    /// Go back in history
    pub fn back(&mut self) -> Result<(), &'static str> {
        if self.history_index > 1 {
            self.history_index -= 1;
            let url = self.history[self.history_index - 1].clone();
            self.navigate(&url)
        } else {
            Err("No previous page")
        }
    }
    
    /// Go forward in history
    pub fn forward(&mut self) -> Result<(), &'static str> {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            let url = self.history[self.history_index - 1].clone();
            self.navigate(&url)
        } else {
            Err("No next page")
        }
    }
    
    /// Refresh current page
    pub fn refresh(&mut self) -> Result<(), &'static str> {
        let url = self.current_url.clone();
        self.navigate(&url)
    }
    
    /// Scroll the page
    pub fn scroll(&mut self, delta: i32) {
        self.scroll_y = (self.scroll_y + delta).max(0);
    }
    
    /// Check if a point hits a link
    pub fn hit_test(&self, x: i32, y: i32) -> Option<&str> {
        let adjusted_y = y + self.scroll_y;
        for link in &self.links {
            if x >= link.x && x < link.x + link.width as i32 &&
               adjusted_y >= link.y && adjusted_y < link.y + link.height as i32 {
                return Some(&link.href);
            }
        }
        None
    }
}

/// Normalize a URL (handle relative URLs)
pub fn normalize_url(url: &str, base: &str) -> String {
    // Already absolute
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    
    // Protocol-relative
    if url.starts_with("//") {
        return alloc::format!("http:{}", url);
    }
    
    // Get base components
    let base = if base.is_empty() { "http://localhost/" } else { base };
    let base = base.strip_prefix("http://").unwrap_or(base);
    
    let (host, base_path) = match base.find('/') {
        Some(i) => (&base[..i], &base[i..]),
        None => (base, "/"),
    };
    
    // Absolute path
    if url.starts_with('/') {
        return alloc::format!("http://{}{}", host, url);
    }
    
    // Relative path
    let base_dir = match base_path.rfind('/') {
        Some(i) => &base_path[..=i],
        None => "/",
    };
    
    alloc::format!("http://{}{}{}", host, base_dir, url)
}

/// Collect external resources (images, CSS, scripts) from HTML nodes
/// Returns Vec of (tag_name, url)
fn collect_resources(nodes: &[HtmlNode]) -> Vec<(String, String)> {
    let mut resources = Vec::new();
    
    for node in nodes {
        if let HtmlNode::Element(el) = node {
            match el.tag.as_str() {
                "img" => {
                    if let Some(src) = el.attr("src") {
                        if !src.is_empty() && !src.starts_with("data:") {
                            resources.push(("img".to_string(), src.to_string()));
                        }
                    }
                }
                "link" => {
                    // CSS stylesheets
                    let rel = el.attr("rel").unwrap_or("");
                    if rel == "stylesheet" {
                        if let Some(href) = el.attr("href") {
                            if !href.is_empty() {
                                resources.push(("css".to_string(), href.to_string()));
                            }
                        }
                    }
                    // Favicon
                    if rel.contains("icon") {
                        if let Some(href) = el.attr("href") {
                            if !href.is_empty() {
                                resources.push(("icon".to_string(), href.to_string()));
                            }
                        }
                    }
                }
                "script" => {
                    if let Some(src) = el.attr("src") {
                        if !src.is_empty() {
                            resources.push(("script".to_string(), src.to_string()));
                        }
                    }
                }
                _ => {}
            }
            
            // Recurse into children
            resources.extend(collect_resources(&el.children));
        }
    }
    
    resources
}

use alloc::string::ToString;
