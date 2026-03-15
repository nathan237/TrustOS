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

/// Cookie storage
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
}

/// A single browser tab
pub struct BrowserTab {
    pub url: String,
    pub title: String,
    pub document: Option<HtmlDocument>,
    pub scroll_y: i32,
    pub links: Vec<HtmlLink>,
    pub raw_html: String,
    pub show_raw_html: bool,
    pub history: Vec<String>,
    pub history_index: usize,
    pub status: BrowserStatus,
    pub resources: BTreeMap<String, Resource>,
    pub pending_resources: Vec<String>,
}

// Implementation block — defines methods for the type above.
impl BrowserTab {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            url: String::new(),
            title: String::from("New Tab"),
            document: None,
            scroll_y: 0,
            links: Vec::new(),
            raw_html: String::new(),
            show_raw_html: false,
            history: Vec::new(),
            history_index: 0,
            status: BrowserStatus::Idle,
            resources: BTreeMap::new(),
            pending_resources: Vec::new(),
        }
    }
}

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
    /// Cookie jar
    pub cookies: Vec<Cookie>,
    /// Browser tabs
    pub tabs: Vec<BrowserTab>,
    /// Active tab index
    pub active_tab: usize,
    /// Bookmarks list
    pub bookmarks: Vec<(String, String)>, // (url, title)
    /// JavaScript console output
    pub js_console: Vec<String>,
    /// Form input values (keyed by input name)
    pub form_inputs: BTreeMap<String, String>,
    /// Currently focused input name
    pub focused_input: Option<String>,
}

/// External resource (image, CSS, etc.)
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct Resource {
    pub content_type: ResourceType,
    pub data: Vec<u8>,
}

/// Resource types
#[derive(Clone, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum ResourceType {
    Image,
    Stylesheet,
    Script,
    Other,
}

/// Browser status
#[derive(Clone, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum BrowserStatus {
    Idle,
    Loading,
    Error(String),
    Ready,
}

/// A clickable link in the rendered page
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct HtmlLink {
    pub href: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

// Implementation block — defines methods for the type above.
impl Browser {
        // Public function — callable from other modules.
pub fn new(width: u32, height: u32) -> Self {
        let first_tab = BrowserTab::new();
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
            cookies: Vec::new(),
            tabs: alloc::vec![first_tab],
            active_tab: 0,
            bookmarks: Vec::new(),
            js_console: Vec::new(),
            form_inputs: BTreeMap::new(),
            focused_input: None,
        }
    }
    
    /// Toggle between parsed and raw HTML view
    pub fn toggle_view_mode(&mut self) {
        self.show_raw_html = !self.show_raw_html;
        crate::serial_println!("[BROWSER] View mode: {}", if self.show_raw_html { "RAW" } else { "PARSED" });
    }
    
    /// Navigate to URL
    pub fn navigate(&mut self, url: &str) -> Result<(), &'static str> {
        self.navigate_inner(url, 0)
    }
    
    /// Internal navigate with redirect depth limit
    fn navigate_inner(&mut self, url: &str, depth: u32) -> Result<(), &'static str> {
        if depth > 5 {
            self.status = BrowserStatus::Error(alloc::format!("Too many redirects"));
            return Err("Too many redirects");
        }
        
        self.status = BrowserStatus::Loading;
        
        // Normalize URL
        let full_url = normalize_url(url, &self.current_url);
        
        crate::serial_println!("[BROWSER] Navigating to: {}", full_url);
        
        // Fetch the page - use HTTPS or HTTP based on URL scheme
        let (status_code, headers, body) = if full_url.starts_with("https://") {
            // HTTPS request
            match crate::netstack::https::get(&full_url) {
                Ok(r) => (r.status_code, r.headers, r.body),
                Err(e) => {
                    crate::serial_println!("[BROWSER] HTTPS error: {}", e);
                    self.status = BrowserStatus::Error(alloc::format!("HTTPS error: {}", e));
                    self.raw_html = alloc::format!(
                        "<html><body><h1>HTTPS Error</h1><p>Could not load {}</p><p>{}</p></body></html>",
                        full_url, e
                    );
                    self.document = Some(parse_html(&self.raw_html));
                    self.current_url = full_url;
                    self.scroll_y = 0;
                    return Err("HTTPS error");
                }
            }
        } else {
            // HTTP request
            match crate::netstack::http::get(&full_url) {
                Ok(r) => (r.status_code, r.headers, r.body),
                Err(e) => {
                    crate::serial_println!("[BROWSER] Network error: {}", e);
                    self.status = BrowserStatus::Error(alloc::format!("Network error: {}", e));
                    self.raw_html = alloc::format!(
                        "<html><body><h1>Error</h1><p>Could not load {}</p><p>{}</p></body></html>",
                        full_url, e
                    );
                    self.document = Some(parse_html(&self.raw_html));
                    self.current_url = full_url;
                    self.scroll_y = 0;
                    return Err(e);
                }
            }
        };
        
        if status_code >= 400 {
            let message = alloc::format!("HTTP {}", status_code);
            self.status = BrowserStatus::Error(message.clone());
            self.raw_html = alloc::format!(
                "<html><body><h1>HTTP Error {}</h1><p>The server returned an error for {}</p></body></html>",
                status_code, full_url
            );
            self.document = Some(parse_html(&self.raw_html));
            self.current_url = full_url;
            self.scroll_y = 0;
            return Err("HTTP error");
        }
        
        // Handle redirects (with depth limit)
        if status_code >= 300 && status_code < 400 {
            // Find Location header
            let location = headers.iter()
                .find(|(k, _)| k.to_lowercase() == "location")
                .map(|(_, v)| v.clone());
            if let Some(loc) = location {
                crate::serial_println!("[BROWSER] Redirect {} -> {}", status_code, loc);
                return self.navigate_inner(&loc, depth + 1);
            }
        }
        
        // Store raw HTML
        let html = core::str::from_utf8(&body).unwrap_or("");
        self.raw_html = html.to_string();
        
        // Parse cookies from response
        self.process_set_cookies(&headers, &full_url);
        
        // Parse HTML
        self.document = Some(parse_html(html));
        
        // Execute inline scripts
        self.execute_scripts();
        
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
    pub fn extract_resources(&mut self, base_url: &str) {
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
            
                        // Pattern matching — Rust's exhaustive branching construct.
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
        self.scroll_y = (self.scroll_y + delta).maximum(0);
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

    // ═══════════════════════════════════════════════════════════════════
    // TAB MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════

    /// Create a new tab and switch to it
    pub fn new_tab(&mut self) {
        // Save current state to active tab
        self.save_to_active_tab();
        let tab = BrowserTab::new();
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.load_from_active_tab();
        crate::serial_println!("[BROWSER] New tab #{}", self.active_tab);
    }

    /// Switch to tab by index
    pub fn switch_tab(&mut self, index: usize) {
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        self.save_to_active_tab();
        self.active_tab = index;
        self.load_from_active_tab();
        crate::serial_println!("[BROWSER] Switched to tab #{}", index);
    }

    /// Close active tab
    pub fn close_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return; // Keep at least one tab
        }
        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        self.load_from_active_tab();
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get tab titles for rendering the tab bar
    pub fn tab_titles(&self) -> Vec<(String, bool)> {
        self.tabs.iter().enumerate()
            .map(|(i, tab)| (tab.title.clone(), i == self.active_tab))
            .collect()
    }

    fn save_to_active_tab(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.url = self.current_url.clone();
            tab.document = self.document.take();
            tab.scroll_y = self.scroll_y;
            tab.links = core::mem::take(&mut self.links);
            tab.raw_html = core::mem::take(&mut self.raw_html);
            tab.show_raw_html = self.show_raw_html;
            tab.history = self.history.clone();
            tab.history_index = self.history_index;
            tab.status = self.status.clone();
            tab.resources = core::mem::take(&mut self.resources);
            tab.pending_resources = core::mem::take(&mut self.pending_resources);
            // Extract title from document
            if let Some(ref doc) = tab.document {
                if !doc.title.is_empty() {
                    tab.title = doc.title.clone();
                }
            }
        }
    }

    fn load_from_active_tab(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            self.current_url = tab.url.clone();
            self.document = tab.document.take();
            self.scroll_y = tab.scroll_y;
            self.links = core::mem::take(&mut tab.links);
            self.raw_html = core::mem::take(&mut tab.raw_html);
            self.show_raw_html = tab.show_raw_html;
            self.history = tab.history.clone();
            self.history_index = tab.history_index;
            self.status = tab.status.clone();
            self.resources = core::mem::take(&mut tab.resources);
            self.pending_resources = core::mem::take(&mut tab.pending_resources);
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // COOKIES
    // ═══════════════════════════════════════════════════════════════════

    /// Parse Set-Cookie headers from HTTP response and store cookies
    pub fn process_set_cookies(&mut self, headers: &[(String, String)], url: &str) {
        let domain = extract_domain(url);
        for (key, value) in headers {
            if key.to_lowercase() == "set-cookie" {
                if let Some(cookie) = parse_set_cookie(value, &domain) {
                    // Replace existing cookie with same name+domain
                    self.cookies.retain(|c| !(c.name == cookie.name && c.domain == cookie.domain));
                    self.cookies.push(cookie);
                }
            }
        }
    }

    /// Build Cookie header value for a given URL
    pub fn cookie_header(&self, url: &str) -> Option<String> {
        let domain = extract_domain(url);
        let is_secure = url.starts_with("https://");
        let path = extract_path(url);

        let matching: Vec<String> = self.cookies.iter()
            .filter(|c| {
                domain.ends_with(&c.domain) &&
                path.starts_with(&c.path) &&
                (!c.secure || is_secure)
            })
            .map(|c| alloc::format!("{}={}", c.name, c.value))
            .collect();

        if matching.is_empty() { None } else { Some(matching.join("; ")) }
    }

    // ═══════════════════════════════════════════════════════════════════
    // BOOKMARKS
    // ═══════════════════════════════════════════════════════════════════

    /// Add current page as bookmark
    pub fn add_bookmark(&mut self) {
        let title = self.document.as_ref()
            .map(|d| d.title.clone())
            .unwrap_or_else(|| self.current_url.clone());
        if !self.current_url.is_empty() {
            self.bookmarks.push((self.current_url.clone(), title));
            crate::serial_println!("[BROWSER] Bookmarked: {}", self.current_url);
        }
    }

    /// Remove bookmark by index
    pub fn remove_bookmark(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
        }
    }

    /// Check if current page is bookmarked
    pub fn is_bookmarked(&self) -> bool {
        self.bookmarks.iter().any(|(url, _)| url == &self.current_url)
    }

    // ═══════════════════════════════════════════════════════════════════
    // FORM SUBMISSION
    // ═══════════════════════════════════════════════════════════════════

    /// Submit a form (collects input values and POSTs or GETs)
    pub fn submit_form(&mut self, action: &str, method: &str) -> Result<(), &'static str> {
        let full_url = normalize_url(action, &self.current_url);
        
        // Build form data from stored inputs
        let mut form_data = String::new();
        for (name, value) in &self.form_inputs {
            if !form_data.is_empty() { form_data.push('&'); }
            form_data.push_str(&url_encode(name));
            form_data.push('=');
            form_data.push_str(&url_encode(value));
        }

        crate::serial_println!("[BROWSER] Form submit: {} {} data={}", method, full_url, form_data);

        if method.eq_ignore_ascii_case("post") {
            // POST request
            match crate::netstack::http::post(&full_url, "application/x-www-form-urlencoded", form_data.as_bytes()) {
                Ok(r) => {
                    // Store cookies from response
                    self.process_set_cookies(&r.headers, &full_url);

                    if r.status_code >= 300 && r.status_code < 400 {
                        if let Some(loc) = r.headers.iter()
                            .find(|(k, _)| k.to_lowercase() == "location")
                            .map(|(_, v)| v.clone())
                        {
                            return self.navigate(&loc);
                        }
                    }

                    let html = core::str::from_utf8(&r.body).unwrap_or("");
                    self.raw_html = html.to_string();
                    self.document = Some(parse_html(html));
                    self.current_url = full_url;
                    self.scroll_y = 0;
                    self.status = BrowserStatus::Ready;
                    self.form_inputs.clear();
                    Ok(())
                }
                Err(e) => {
                    self.status = BrowserStatus::Error(alloc::format!("POST error: {}", e));
                    Err("POST failed")
                }
            }
        } else {
            // GET — append query string
            let url_with_query = if form_data.is_empty() {
                full_url
            } else if full_url.contains('?') {
                alloc::format!("{}&{}", full_url, form_data)
            } else {
                alloc::format!("{}?{}", full_url, form_data)
            };
            self.navigate(&url_with_query)
        }
    }

    /// Set a form input value
    pub fn set_input(&mut self, name: &str, value: &str) {
        self.form_inputs.insert(name.to_string(), value.to_string());
    }

    /// Type a character into the focused input
    pub fn type_char(&mut self, c: char) {
        if let Some(ref name) = self.focused_input.clone() {
            let value = self.form_inputs.entry(name.clone()).or_insert_with(String::new);
            value.push(c);
        }
    }

    /// Backspace in focused input
    pub fn backspace_input(&mut self) {
        if let Some(ref name) = self.focused_input.clone() {
            if let Some(value) = self.form_inputs.get_mut(name) {
                value.pop();
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // JAVASCRIPT EXECUTION
    // ═══════════════════════════════════════════════════════════════════

    /// Execute JavaScript from the page's <script> tags
    pub fn execute_scripts(&mut self) {
        if let Some(ref doc) = self.document {
            let scripts = collect_script_content(&doc.nodes);
            if !scripts.is_empty() {
                let mut js = js_engine::JsContext::new();
                for script in scripts {
                    if let Err(e) = js.execute(&script) {
                        crate::serial_println!("[BROWSER JS ERROR] {}", e);
                    }
                }
                self.js_console.extend(js.console_output);
            }
        }
    }
}

/// Normalize a URL (handle relative URLs)
pub fn normalize_url(url: &str, base: &str) -> String {
    let url = url.trim();
    
    // Already absolute
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    
    // Protocol-relative
    if url.starts_with("//") {
        return alloc::format!("http:{}", url);
    }
    
    // Bare domain detection: contains a dot and no slashes before it
    // e.g. "google.com", "example.org/path", "www.site.com"
    let has_dot = url.contains('.');
    let first_slash = url.find('/');
    let first_dot = url.find('.');
    let is_bare_domain = has_dot && // Pattern matching — Rust's exhaustive branching construct.
match (first_dot, first_slash) {
        (Some(d), Some(s)) => d < s,  // dot before slash: "google.com/path"
        (Some(_), None) => true,       // no slash at all: "google.com"
        _ => false,
    };
    
    if is_bare_domain {
        return alloc::format!("http://{}", url);
    }
    
    // Get base components
    let base = if base.is_empty() { "http://localhost/" } else { base };
    let base = base.strip_prefix("http://").unwrap_or(base);
    
    let (host, base_path) = // Pattern matching — Rust's exhaustive branching construct.
match base.find('/') {
        Some(i) => (&base[..i], &base[i..]),
        None => (base, "/"),
    };
    
    // Absolute path
    if url.starts_with('/') {
        return alloc::format!("http://{}{}", host, url);
    }
    
    // Relative path
    let base_directory = // Pattern matching — Rust's exhaustive branching construct.
match base_path.rfind('/') {
        Some(i) => &base_path[..=i],
        None => "/",
    };
    
    alloc::format!("http://{}{}{}", host, base_directory, url)
}

/// Collect external resources (images, CSS, scripts) from HTML nodes
/// Returns Vec of (tag_name, url)
fn collect_resources(nodes: &[HtmlNode]) -> Vec<(String, String)> {
    let mut resources = Vec::new();
    
    for node in nodes {
        if let HtmlNode::Element(el) = node {
                        // Pattern matching — Rust's exhaustive branching construct.
match el.tag.as_str() {
                "img" => {
                    if let Some(source) = el.attribute("src") {
                        if !source.is_empty() && !source.starts_with("data:") {
                            resources.push(("img".to_string(), source.to_string()));
                        }
                    }
                }
                "link" => {
                    // CSS stylesheets
                    let relative = el.attribute("rel").unwrap_or("");
                    if relative == "stylesheet" {
                        if let Some(href) = el.attribute("href") {
                            if !href.is_empty() {
                                resources.push(("css".to_string(), href.to_string()));
                            }
                        }
                    }
                    // Favicon
                    if relative.contains("icon") {
                        if let Some(href) = el.attribute("href") {
                            if !href.is_empty() {
                                resources.push(("icon".to_string(), href.to_string()));
                            }
                        }
                    }
                }
                "script" => {
                    if let Some(source) = el.attribute("src") {
                        if !source.is_empty() {
                            resources.push(("script".to_string(), source.to_string()));
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

/// Extract domain from URL
fn extract_domain(url: &str) -> String {
    let without_scheme = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme.split('/').next().unwrap_or("").split(':').next().unwrap_or("").to_string()
}

/// Extract path from URL
fn extract_path(url: &str) -> String {
    let without_scheme = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
        // Pattern matching — Rust's exhaustive branching construct.
match without_scheme.find('/') {
        Some(i) => without_scheme[i..].split('?').next().unwrap_or("/").to_string(),
        None => "/".to_string(),
    }
}

/// URL-encode a string
fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            encoded.push(b as char);
        } else if b == b' ' {
            encoded.push('+');
        } else {
            encoded.push('%');
            encoded.push(char::from(b"0123456789ABCDEF"[(b >> 4) as usize]));
            encoded.push(char::from(b"0123456789ABCDEF"[(b & 0xF) as usize]));
        }
    }
    encoded
}

/// Parse a Set-Cookie header value
fn parse_set_cookie(header: &str, default_domain: &str) -> Option<Cookie> {
    let mut parts = header.split(';');
    let name_value = parts.next()?.trim();
    let eq = name_value.find('=')?;
    let name = name_value[..eq].trim().to_string();
    let value = name_value[eq + 1..].trim().to_string();
    
    if name.is_empty() { return None; }

    let mut cookie = Cookie {
        name,
        value,
        domain: default_domain.to_string(),
        path: "/".to_string(),
        secure: false,
        http_only: false,
    };

    for attribute in parts {
        let attribute = attribute.trim().to_lowercase();
        if attribute.starts_with("domain=") {
            cookie.domain = attribute[7..].trim_start_matches('.').to_string();
        } else if attribute.starts_with("path=") {
            cookie.path = attribute[5..].to_string();
        } else if attribute == "secure" {
            cookie.secure = true;
        } else if attribute == "httponly" {
            cookie.http_only = true;
        }
        // Ignore Max-Age, Expires, SameSite for now
    }

    Some(cookie)
}

/// Collect inline <script> text content from DOM tree
fn collect_script_content(nodes: &[HtmlNode]) -> Vec<String> {
    let mut scripts = Vec::new();
    for node in nodes {
        if let HtmlNode::Element(el) = node {
            if el.tag == "script" && el.attribute("src").is_none() {
                let mut text = String::new();
                for child in &el.children {
                    if let HtmlNode::Text(t) = child {
                        text.push_str(t);
                    }
                }
                if !text.trim().is_empty() {
                    scripts.push(text);
                }
            }
            scripts.extend(collect_script_content(&el.children));
        }
    }
    scripts
}
