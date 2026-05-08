



extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;




#[derive(Debug, Clone)]
pub enum PolicyVerdict {
    
    Allow,
    
    Deny(String),
    
    Log,
}




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyPreset {
    
    Strict,
    
    Moderate,
    
    Permissive,
}




#[derive(Debug, Clone)]
pub struct DomainRule {
    
    pub pattern: String,
    
    pub allow: bool,
}

impl DomainRule {
    pub fn allow(pattern: &str) -> Self {
        Self { pattern: String::from(pattern), allow: true }
    }

    pub fn deny(pattern: &str) -> Self {
        Self { pattern: String::from(pattern), allow: false }
    }

    
    pub fn matches(&self, domain: &str) -> bool {
        if self.pattern.starts_with("*.") {
            
            let asi = &self.pattern[1..]; 
            domain.ends_with(asi) || domain == &self.pattern[2..]
        } else {
            domain == self.pattern
        }
    }
}




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFilter {
    
    AllowHtml,
    
    AllowCss,
    
    AllowImages,
    
    AllowJavaScript,
    
    AllowJson,
    
    AllowText,
    
    AllowFonts,
}




const BNZ_: &[u16] = &[
    21,   
    22,   
    23,   
    25,   
    110,  
    143,  
    445,  
    3306, 
    3389, 
    5432, 
    5900, 
    6379, 
    27017, 
];




const BTT_: &[&str] = &[
    "file://",
    "javascript:",
    "data:text/html",
    "blob:",
    "about:",
    "localhost",
    "127.0.0.1",
    "0.0.0.0",
    "[::]",
    "[::1]",
    "169.254.",     
    "10.",          
    "192.168.",     
    "172.16.",      
];




fn msh(url: &str) -> Option<&'static str> {
    let gj = url.to_ascii_lowercase();
    for &pat in BTT_ {
        if gj.contains(pat) {
            return Some(pat);
        }
    }
    
    
    if let Some(host_start) = gj.find("://") {
        let beo = &gj[host_start + 3..];
        if let Some(first) = beo.chars().next() {
            if first.is_ascii_digit() {
                
                
                if beo.starts_with("10.") || beo.starts_with("192.168.") 
                    || beo.starts_with("172.16.") || beo.starts_with("127.")
                    || beo.starts_with("0.") {
                    return Some("private IP address");
                }
            }
        }
    }
    None
}


pub fn cxk(url: &str) -> Option<&str> {
    let cfk = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    
    let host = cfk.split('/').next()?;
    
    let domain = if host.contains(':') {
        host.split(':').next()?
    } else {
        host
    };
    
    let domain = if domain.contains('@') {
        domain.split('@').last()?
    } else {
        domain
    };
    if domain.is_empty() { None } else { Some(domain) }
}


fn ltr(url: &str) -> u16 {
    let gj = url.to_ascii_lowercase();
    let cln = gj.starts_with("https://");
    let bru = if cln { 443 } else { 80 };

    let cfk = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    let host = cfk.split('/').next().unwrap_or("");
    if let Some(ald) = host.rfind(':') {
        host[ald + 1..].parse().unwrap_or(bru)
    } else {
        bru
    }
}




#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub preset: PolicyPreset,
    
    pub domain_rules: Vec<DomainRule>,
    
    pub allowed_content: Vec<ContentFilter>,
    
    pub rate_limit_per_min: u32,
    
    pub max_response_bytes: usize,
    
    pub block_javascript: bool,
    
    pub block_inline_scripts: bool,
    
    pub block_external_scripts: bool,
    
    pub allow_redirects: bool,
    
    pub max_redirect_depth: u8,
    
    pub block_private_ips: bool,
    
    pub log_all: bool,
    
    pub blocked_substrings: Vec<String>,
}

impl SandboxPolicy {
    
    pub fn lzo(preset: PolicyPreset) -> Self {
        match preset {
            PolicyPreset::Strict => Self::oyd(),
            PolicyPreset::Moderate => Self::ngb(),
            PolicyPreset::Permissive => Self::ntr(),
        }
    }

    
    fn oyd() -> Self {
        Self {
            preset: PolicyPreset::Strict,
            domain_rules: Vec::new(), 
            allowed_content: alloc::vec![
                ContentFilter::AllowHtml,
                ContentFilter::AllowCss,
                ContentFilter::AllowImages,
                ContentFilter::AllowText,
            ],
            rate_limit_per_min: 30,
            max_response_bytes: 512 * 1024, 
            block_javascript: true,
            block_inline_scripts: true,
            block_external_scripts: true,
            allow_redirects: true,
            max_redirect_depth: 2,
            block_private_ips: true,
            log_all: true,
            blocked_substrings: Vec::new(),
        }
    }

    
    fn ngb() -> Self {
        Self {
            preset: PolicyPreset::Moderate,
            domain_rules: alloc::vec![
                
                DomainRule::deny("*.doubleclick.net"),
                DomainRule::deny("*.googlesyndication.com"),
                DomainRule::deny("*.analytics.google.com"),
                DomainRule::deny("*.facebook.com"),
                DomainRule::deny("*.fbcdn.net"),
            ],
            allowed_content: alloc::vec![
                ContentFilter::AllowHtml,
                ContentFilter::AllowCss,
                ContentFilter::AllowImages,
                ContentFilter::AllowJson,
                ContentFilter::AllowText,
                ContentFilter::AllowJavaScript,
                ContentFilter::AllowFonts,
            ],
            rate_limit_per_min: 60,
            max_response_bytes: 1024 * 1024, 
            block_javascript: false,
            block_inline_scripts: false,
            block_external_scripts: false,
            allow_redirects: true,
            max_redirect_depth: 5,
            block_private_ips: true,
            log_all: false,
            blocked_substrings: Vec::new(),
        }
    }

    
    fn ntr() -> Self {
        Self {
            preset: PolicyPreset::Permissive,
            domain_rules: Vec::new(), 
            allowed_content: alloc::vec![
                ContentFilter::AllowHtml,
                ContentFilter::AllowCss,
                ContentFilter::AllowImages,
                ContentFilter::AllowJson,
                ContentFilter::AllowText,
                ContentFilter::AllowJavaScript,
                ContentFilter::AllowFonts,
            ],
            rate_limit_per_min: 120,
            max_response_bytes: 4 * 1024 * 1024, 
            block_javascript: false,
            block_inline_scripts: false,
            block_external_scripts: false,
            allow_redirects: true,
            max_redirect_depth: 10,
            block_private_ips: true, 
            log_all: false,
            blocked_substrings: Vec::new(),
        }
    }

    
    pub fn allow_domain(&mut self, pattern: &str) {
        self.domain_rules.insert(0, DomainRule::allow(pattern));
    }

    
    pub fn deny_domain(&mut self, pattern: &str) {
        self.domain_rules.insert(0, DomainRule::deny(pattern));
    }

    
    pub fn evaluate_url(&self, url: &str) -> PolicyVerdict {
        
        if let Some(pattern) = msh(url) {
            return PolicyVerdict::Deny(format!("blocked dangerous pattern: {}", pattern));
        }

        
        let port = ltr(url);
        if BNZ_.contains(&port) {
            return PolicyVerdict::Deny(format!("blocked port: {}", port));
        }

        
        let gj = url.to_ascii_lowercase();
        for blocked in &self.blocked_substrings {
            if gj.contains(&blocked.to_ascii_lowercase()) {
                return PolicyVerdict::Deny(format!("blocked pattern: {}", blocked));
            }
        }

        
        let domain = match cxk(url) {
            Some(d) => d,
            None => return PolicyVerdict::Deny(String::from("cannot extract domain")),
        };

        
        for qo in &self.domain_rules {
            if qo.matches(domain) {
                if qo.allow {
                    return if self.log_all { PolicyVerdict::Log } else { PolicyVerdict::Allow };
                } else {
                    return PolicyVerdict::Deny(format!("domain blocked: {}", domain));
                }
            }
        }

        
        match self.preset {
            PolicyPreset::Strict => {
                
                PolicyVerdict::Deny(format!("domain not in allowlist: {}", domain))
            },
            PolicyPreset::Moderate => {
                
                if self.log_all { PolicyVerdict::Log } else { PolicyVerdict::Allow }
            },
            PolicyPreset::Permissive => {
                PolicyVerdict::Allow
            },
        }
    }

    
    pub fn js_allowed(&self) -> bool {
        !self.block_javascript
    }

    
    pub fn qbg(&self, content_type: &str) -> bool {
        let wb = content_type.to_ascii_lowercase();
        for filter in &self.allowed_content {
            match filter {
                ContentFilter::AllowHtml => if wb.contains("text/html") || wb.contains("application/xhtml") { return true; },
                ContentFilter::AllowCss => if wb.contains("text/css") { return true; },
                ContentFilter::AllowImages => if wb.contains("image/") { return true; },
                ContentFilter::AllowJavaScript => if wb.contains("javascript") || wb.contains("ecmascript") { return true; },
                ContentFilter::AllowJson => if wb.contains("json") || wb.contains("application/xml") { return true; },
                ContentFilter::AllowText => if wb.contains("text/plain") || wb.contains("text/") { return true; },
                ContentFilter::AllowFonts => if wb.contains("font/") || wb.contains("woff") { return true; },
            }
        }
        false
    }

    
    pub fn summary(&self) -> String {
        let jvd: usize = self.domain_rules.iter().filter(|r| r.allow).count();
        let ldf: usize = self.domain_rules.iter().filter(|r| !r.allow).count();
        format!(
            "Policy: {:?}\n  Domain rules: {} allow, {} deny\n  Rate limit: {}/min\n  Max response: {} KB\n  JS: {}\n  Redirects: {} (max depth {})\n  SSRF protection: {}\n  Log all: {}",
            self.preset,
            jvd, ldf,
            self.rate_limit_per_min,
            self.max_response_bytes / 1024,
            if self.block_javascript { "BLOCKED" } else { "sandboxed" },
            if self.allow_redirects { "allowed" } else { "blocked" },
            self.max_redirect_depth,
            if self.block_private_ips { "ON" } else { "OFF" },
            if self.log_all { "yes" } else { "no" },
        )
    }
}
