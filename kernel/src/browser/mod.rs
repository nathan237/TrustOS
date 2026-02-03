//! TrustOS Web Browser
//!
//! A simple web browser that can render basic HTML pages.
//! Supports HTTP (not HTTPS yet due to TLS complexity).

pub mod html_parser;
pub mod renderer;
pub mod url;

pub use html_parser::*;
pub use renderer::*;
pub use url::*;

use alloc::string::String;
use alloc::vec::Vec;

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
        }
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
        
        // Parse HTML
        let html = response.body_str().unwrap_or("");
        self.document = Some(parse_html(html));
        
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

use alloc::string::ToString;
