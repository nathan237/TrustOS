












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


#[derive(Clone)]
pub struct Rk {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
}


pub struct BrowserTab {
    pub url: String,
    pub title: String,
    pub document: Option<Ia>,
    pub scroll_y: i32,
    pub links: Vec<Jy>,
    pub raw_html: String,
    pub show_raw_html: bool,
    pub history: Vec<String>,
    pub history_index: usize,
    pub status: BrowserStatus,
    pub resources: BTreeMap<String, Fx>,
    pub pending_resources: Vec<String>,
}

impl BrowserTab {
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


pub struct Browser {
    pub current_url: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub document: Option<Ia>,
    pub scroll_y: i32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub status: BrowserStatus,
    pub links: Vec<Jy>,
    
    pub raw_html: String,
    
    pub show_raw_html: bool,
    
    pub resources: BTreeMap<String, Fx>,
    
    pub pending_resources: Vec<String>,
    
    pub cookies: Vec<Rk>,
    
    pub tabs: Vec<BrowserTab>,
    
    pub active_tab: usize,
    
    pub bookmarks: Vec<(String, String)>, 
    
    pub js_console: Vec<String>,
    
    pub form_inputs: BTreeMap<String, String>,
    
    pub focused_input: Option<String>,
}


#[derive(Clone)]
pub struct Fx {
    pub content_type: ResourceType,
    pub data: Vec<u8>,
}


#[derive(Clone, PartialEq)]
pub enum ResourceType {
    Image,
    Fd,
    Script,
    Other,
}


#[derive(Clone, PartialEq)]
pub enum BrowserStatus {
    Idle,
    Loading,
    Error(String),
    Ready,
}


#[derive(Clone)]
pub struct Jy {
    pub href: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Browser {
    pub fn new(width: u32, height: u32) -> Self {
        let lwl = BrowserTab::new();
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
            tabs: alloc::vec![lwl],
            active_tab: 0,
            bookmarks: Vec::new(),
            js_console: Vec::new(),
            form_inputs: BTreeMap::new(),
            focused_input: None,
        }
    }
    
    
    pub fn toggle_view_mode(&mut self) {
        self.show_raw_html = !self.show_raw_html;
        crate::serial_println!("[BROWSER] View mode: {}", if self.show_raw_html { "RAW" } else { "PARSED" });
    }
    
    
    pub fn navigate(&mut self, url: &str) -> Result<(), &'static str> {
        self.navigate_inner(url, 0)
    }
    
    
    fn navigate_inner(&mut self, url: &str, depth: u32) -> Result<(), &'static str> {
        if depth > 5 {
            self.status = BrowserStatus::Error(alloc::format!("Too many redirects"));
            return Err("Too many redirects");
        }
        
        self.status = BrowserStatus::Loading;
        
        
        let ti = normalize_url(url, &self.current_url);
        
        crate::serial_println!("[BROWSER] Navigating to: {}", ti);
        
        
        let (status_code, headers, body) = if ti.starts_with("https://") {
            
            match crate::netstack::https::get(&ti) {
                Ok(r) => (r.status_code, r.headers, r.body),
                Err(e) => {
                    crate::serial_println!("[BROWSER] HTTPS error: {}", e);
                    self.status = BrowserStatus::Error(alloc::format!("HTTPS error: {}", e));
                    self.raw_html = alloc::format!(
                        "<html><body><h1>HTTPS Error</h1><p>Could not load {}</p><p>{}</p></body></html>",
                        ti, e
                    );
                    self.document = Some(boe(&self.raw_html));
                    self.current_url = ti;
                    self.scroll_y = 0;
                    return Err("HTTPS error");
                }
            }
        } else {
            
            match crate::netstack::http::get(&ti) {
                Ok(r) => (r.status_code, r.headers, r.body),
                Err(e) => {
                    crate::serial_println!("[BROWSER] Network error: {}", e);
                    self.status = BrowserStatus::Error(alloc::format!("Network error: {}", e));
                    self.raw_html = alloc::format!(
                        "<html><body><h1>Error</h1><p>Could not load {}</p><p>{}</p></body></html>",
                        ti, e
                    );
                    self.document = Some(boe(&self.raw_html));
                    self.current_url = ti;
                    self.scroll_y = 0;
                    return Err(e);
                }
            }
        };
        
        if status_code >= 400 {
            let bk = alloc::format!("HTTP {}", status_code);
            self.status = BrowserStatus::Error(bk.clone());
            self.raw_html = alloc::format!(
                "<html><body><h1>HTTP Error {}</h1><p>The server returned an error for {}</p></body></html>",
                status_code, ti
            );
            self.document = Some(boe(&self.raw_html));
            self.current_url = ti;
            self.scroll_y = 0;
            return Err("HTTP error");
        }
        
        
        if status_code >= 300 && status_code < 400 {
            
            let axx = headers.iter()
                .find(|(k, _)| k.to_lowercase() == "location")
                .map(|(_, v)| v.clone());
            if let Some(loc) = axx {
                crate::serial_println!("[BROWSER] Redirect {} -> {}", status_code, loc);
                return self.navigate_inner(&loc, depth + 1);
            }
        }
        
        
        let ajx = core::str::from_utf8(&body).unwrap_or("");
        self.raw_html = ajx.to_string();
        
        
        self.process_set_cookies(&headers, &ti);
        
        
        self.document = Some(boe(ajx));
        
        
        self.execute_scripts();
        
        
        self.extract_resources(&ti);
        
        
        if self.history_index < self.history.len() {
            self.history.truncate(self.history_index);
        }
        self.history.push(ti.clone());
        self.history_index = self.history.len();
        
        self.current_url = ti;
        self.scroll_y = 0;
        self.status = BrowserStatus::Ready;
        
        Ok(())
    }
    
    
    pub fn extract_resources(&mut self, base_url: &str) {
        self.pending_resources.clear();
        self.resources.clear();
        
        if let Some(ref doc) = self.document {
            let resources = hmu(&doc.nodes);
            for (tag, url) in resources {
                let ti = normalize_url(&url, base_url);
                crate::serial_println!("[BROWSER] Found {} resource: {}", tag, ti);
                self.pending_resources.push(ti);
            }
        }
    }
    
    
    pub fn qnt(&mut self) {
        let ogi: Vec<String> = self.pending_resources.drain(..).collect();
        
        for url in ogi {
            if self.resources.contains_key(&url) {
                continue; 
            }
            
            crate::serial_println!("[BROWSER] Loading resource: {}", url);
            
            match crate::netstack::http::get(&url) {
                Ok(fa) => {
                    if fa.status_code == 200 {
                        let content_type = fa.header("Content-Type").unwrap_or("");
                        let akk = if content_type.contains("image") {
                            ResourceType::Image
                        } else if content_type.contains("css") {
                            ResourceType::Fd
                        } else if content_type.contains("javascript") {
                            ResourceType::Script
                        } else {
                            ResourceType::Other
                        };
                        
                        crate::serial_println!("[BROWSER] Loaded {} ({} bytes)", url, fa.body.len());
                        self.resources.insert(url, Fx {
                            content_type: akk,
                            data: fa.body,
                        });
                    }
                }
                Err(e) => {
                    crate::serial_println!("[BROWSER] Failed to load {}: {}", url, e);
                }
            }
        }
    }
    
    
    pub fn back(&mut self) -> Result<(), &'static str> {
        if self.history_index > 1 {
            self.history_index -= 1;
            let url = self.history[self.history_index - 1].clone();
            self.navigate(&url)
        } else {
            Err("No previous page")
        }
    }
    
    
    pub fn forward(&mut self) -> Result<(), &'static str> {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            let url = self.history[self.history_index - 1].clone();
            self.navigate(&url)
        } else {
            Err("No next page")
        }
    }
    
    
    pub fn refresh(&mut self) -> Result<(), &'static str> {
        let url = self.current_url.clone();
        self.navigate(&url)
    }
    
    
    pub fn scroll(&mut self, mk: i32) {
        self.scroll_y = (self.scroll_y + mk).max(0);
    }
    
    
    pub fn mlr(&self, x: i32, y: i32) -> Option<&str> {
        let hed = y + self.scroll_y;
        for link in &self.links {
            if x >= link.x && x < link.x + link.width as i32 &&
               hed >= link.y && hed < link.y + link.height as i32 {
                return Some(&link.href);
            }
        }
        None
    }

    
    
    

    
    pub fn qpm(&mut self) {
        
        self.save_to_active_tab();
        let tab = BrowserTab::new();
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.load_from_active_tab();
        crate::serial_println!("[BROWSER] New tab #{}", self.active_tab);
    }

    
    pub fn qyg(&mut self, index: usize) {
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        self.save_to_active_tab();
        self.active_tab = index;
        self.load_from_active_tab();
        crate::serial_println!("[BROWSER] Switched to tab #{}", index);
    }

    
    pub fn qaj(&mut self) {
        if self.tabs.len() <= 1 {
            return; 
        }
        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        self.load_from_active_tab();
    }

    
    pub fn qyq(&self) -> usize {
        self.tabs.len()
    }

    
    pub fn qyr(&self) -> Vec<(String, bool)> {
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

    
    
    

    
    pub fn process_set_cookies(&mut self, headers: &[(String, String)], url: &str) {
        let domain = cxk(url);
        for (key, value) in headers {
            if key.to_lowercase() == "set-cookie" {
                if let Some(brd) = nre(value, &domain) {
                    
                    self.cookies.retain(|c| !(c.name == brd.name && c.domain == brd.domain));
                    self.cookies.push(brd);
                }
            }
        }
    }

    
    pub fn qbh(&self, url: &str) -> Option<String> {
        let domain = cxk(url);
        let mtr = url.starts_with("https://");
        let path = ltq(url);

        let ima: Vec<String> = self.cookies.iter()
            .filter(|c| {
                domain.ends_with(&c.domain) &&
                path.starts_with(&c.path) &&
                (!c.secure || mtr)
            })
            .map(|c| alloc::format!("{}={}", c.name, c.value))
            .collect();

        if ima.is_empty() { None } else { Some(ima.join("; ")) }
    }

    
    
    

    
    pub fn pxs(&mut self) {
        let title = self.document.as_ref()
            .map(|d| d.title.clone())
            .unwrap_or_else(|| self.current_url.clone());
        if !self.current_url.is_empty() {
            self.bookmarks.push((self.current_url.clone(), title));
            crate::serial_println!("[BROWSER] Bookmarked: {}", self.current_url);
        }
    }

    
    pub fn qtq(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
        }
    }

    
    pub fn qmd(&self) -> bool {
        self.bookmarks.iter().any(|(url, _)| url == &self.current_url)
    }

    
    
    

    
    pub fn qxs(&mut self, action: &str, aui: &str) -> Result<(), &'static str> {
        let ti = normalize_url(action, &self.current_url);
        
        
        let mut bmf = String::new();
        for (name, value) in &self.form_inputs {
            if !bmf.is_empty() { bmf.push('&'); }
            bmf.push_str(&hau(name));
            bmf.push('=');
            bmf.push_str(&hau(value));
        }

        crate::serial_println!("[BROWSER] Form submit: {} {} data={}", aui, ti, bmf);

        if aui.eq_ignore_ascii_case("post") {
            
            match crate::netstack::http::nwh(&ti, "application/x-www-form-urlencoded", bmf.as_bytes()) {
                Ok(r) => {
                    
                    self.process_set_cookies(&r.headers, &ti);

                    if r.status_code >= 300 && r.status_code < 400 {
                        if let Some(loc) = r.headers.iter()
                            .find(|(k, _)| k.to_lowercase() == "location")
                            .map(|(_, v)| v.clone())
                        {
                            return self.navigate(&loc);
                        }
                    }

                    let ajx = core::str::from_utf8(&r.body).unwrap_or("");
                    self.raw_html = ajx.to_string();
                    self.document = Some(boe(ajx));
                    self.current_url = ti;
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
            
            let pqa = if bmf.is_empty() {
                ti
            } else if ti.contains('?') {
                alloc::format!("{}&{}", ti, bmf)
            } else {
                alloc::format!("{}?{}", ti, bmf)
            };
            self.navigate(&pqa)
        }
    }

    
    pub fn qwb(&mut self, name: &str, value: &str) {
        self.form_inputs.insert(name.to_string(), value.to_string());
    }

    
    pub fn rbc(&mut self, c: char) {
        if let Some(ref name) = self.focused_input.clone() {
            let val = self.form_inputs.entry(name.clone()).or_insert_with(String::new);
            val.push(c);
        }
    }

    
    pub fn pyi(&mut self) {
        if let Some(ref name) = self.focused_input.clone() {
            if let Some(val) = self.form_inputs.get_mut(name) {
                val.pop();
            }
        }
    }

    
    
    

    
    pub fn execute_scripts(&mut self) {
        if let Some(ref doc) = self.document {
            let ddx = hmv(&doc.nodes);
            if !ddx.is_empty() {
                let mut iis = js_engine::JsContext::new();
                for script in ddx {
                    if let Err(e) = iis.execute(&script) {
                        crate::serial_println!("[BROWSER JS ERROR] {}", e);
                    }
                }
                self.js_console.extend(iis.console_output);
            }
        }
    }
}


pub fn normalize_url(url: &str, base: &str) -> String {
    let url = url.trim();
    
    
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    
    
    if url.starts_with("//") {
        return alloc::format!("http:{}", url);
    }
    
    
    
    let mjg = url.contains('.');
    let lwk = url.find('/');
    let lwg = url.find('.');
    let mry = mjg && match (lwg, lwk) {
        (Some(d), Some(j)) => d < j,  
        (Some(_), None) => true,       
        _ => false,
    };
    
    if mry {
        return alloc::format!("http://{}", url);
    }
    
    
    let base = if base.is_empty() { "http://localhost/" } else { base };
    let base = base.strip_prefix("http://").unwrap_or(base);
    
    let (host, cge) = match base.find('/') {
        Some(i) => (&base[..i], &base[i..]),
        None => (base, "/"),
    };
    
    
    if url.starts_with('/') {
        return alloc::format!("http://{}{}", host, url);
    }
    
    
    let dij = match cge.rfind('/') {
        Some(i) => &cge[..=i],
        None => "/",
    };
    
    alloc::format!("http://{}{}{}", host, dij, url)
}



fn hmu(nodes: &[HtmlNode]) -> Vec<(String, String)> {
    let mut resources = Vec::new();
    
    for uf in nodes {
        if let HtmlNode::Element(el) = uf {
            match el.tag.as_str() {
                "img" => {
                    if let Some(src) = el.attr("src") {
                        if !src.is_empty() && !src.starts_with("data:") {
                            resources.push(("img".to_string(), src.to_string()));
                        }
                    }
                }
                "link" => {
                    
                    let ot = el.attr("rel").unwrap_or("");
                    if ot == "stylesheet" {
                        if let Some(href) = el.attr("href") {
                            if !href.is_empty() {
                                resources.push(("css".to_string(), href.to_string()));
                            }
                        }
                    }
                    
                    if ot.contains("icon") {
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
            
            
            resources.extend(hmu(&el.children));
        }
    }
    
    resources
}

use alloc::string::ToString;


fn cxk(url: &str) -> String {
    let cfk = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    cfk.split('/').next().unwrap_or("").split(':').next().unwrap_or("").to_string()
}


fn ltq(url: &str) -> String {
    let cfk = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    match cfk.find('/') {
        Some(i) => cfk[i..].split('?').next().unwrap_or("/").to_string(),
        None => "/".to_string(),
    }
}


fn hau(j: &str) -> String {
    let mut atq = String::new();
    for b in j.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            atq.push(b as char);
        } else if b == b' ' {
            atq.push('+');
        } else {
            atq.push('%');
            atq.push(char::from(b"0123456789ABCDEF"[(b >> 4) as usize]));
            atq.push(char::from(b"0123456789ABCDEF"[(b & 0xF) as usize]));
        }
    }
    atq
}


fn nre(header: &str, default_domain: &str) -> Option<Rk> {
    let mut au = header.split(';');
    let giq = au.next()?.trim();
    let eq = giq.find('=')?;
    let name = giq[..eq].trim().to_string();
    let value = giq[eq + 1..].trim().to_string();
    
    if name.is_empty() { return None; }

    let mut brd = Rk {
        name,
        value,
        domain: default_domain.to_string(),
        path: "/".to_string(),
        secure: false,
        http_only: false,
    };

    for attr in au {
        let attr = attr.trim().to_lowercase();
        if attr.starts_with("domain=") {
            brd.domain = attr[7..].trim_start_matches('.').to_string();
        } else if attr.starts_with("path=") {
            brd.path = attr[5..].to_string();
        } else if attr == "secure" {
            brd.secure = true;
        } else if attr == "httponly" {
            brd.http_only = true;
        }
        
    }

    Some(brd)
}


fn hmv(nodes: &[HtmlNode]) -> Vec<String> {
    let mut ddx = Vec::new();
    for uf in nodes {
        if let HtmlNode::Element(el) = uf {
            if el.tag == "script" && el.attr("src").is_none() {
                let mut text = String::new();
                for pd in &el.children {
                    if let HtmlNode::Text(t) = pd {
                        text.push_str(t);
                    }
                }
                if !text.trim().is_empty() {
                    ddx.push(text);
                }
            }
            ddx.extend(hmv(&el.children));
        }
    }
    ddx
}
