//! URL utilities
//!
//! Parsing and manipulation of URLs.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Parsed URL
#[derive(Clone, Debug)]
pub struct Url {
    pub scheme: String,     // "http" or "https"
    pub host: String,       // "example.com"
    pub port: u16,          // 80, 443, etc.
    pub path: String,       // "/page.html"
    pub query: Option<String>,  // "key=value"
    pub fragment: Option<String>, // "#section"
}

impl Url {
    /// Parse a URL string
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        
        // Extract scheme
        let (scheme, rest) = if let Some(idx) = url.find("://") {
            (&url[..idx], &url[idx + 3..])
        } else {
            ("http", url)
        };
        
        // Extract fragment
        let (rest, fragment) = if let Some(idx) = rest.find('#') {
            (&rest[..idx], Some(rest[idx + 1..].to_string()))
        } else {
            (rest, None)
        };
        
        // Extract query
        let (rest, query) = if let Some(idx) = rest.find('?') {
            (&rest[..idx], Some(rest[idx + 1..].to_string()))
        } else {
            (rest, None)
        };
        
        // Extract path
        let (host_port, path) = if let Some(idx) = rest.find('/') {
            (&rest[..idx], rest[idx..].to_string())
        } else {
            (rest, "/".to_string())
        };
        
        // Extract port
        let (host, port) = if let Some(idx) = host_port.find(':') {
            let port_str = &host_port[idx + 1..];
            let port = port_str.parse().unwrap_or(80);
            (&host_port[..idx], port)
        } else {
            let default_port = if scheme == "https" { 443 } else { 80 };
            (host_port, default_port)
        };
        
        if host.is_empty() {
            return None;
        }
        
        Some(Self {
            scheme: scheme.to_string(),
            host: host.to_string(),
            port,
            path,
            query,
            fragment,
        })
    }
    
    /// Convert back to string
    pub fn to_string(&self) -> String {
        let mut s = alloc::format!("{}://{}", self.scheme, self.host);
        
        let default_port = if self.scheme == "https" { 443 } else { 80 };
        if self.port != default_port {
            s.push(':');
            s.push_str(&alloc::format!("{}", self.port));
        }
        
        s.push_str(&self.path);
        
        if let Some(ref q) = self.query {
            s.push('?');
            s.push_str(q);
        }
        
        if let Some(ref f) = self.fragment {
            s.push('#');
            s.push_str(f);
        }
        
        s
    }
    
    /// Resolve a relative URL against this base
    pub fn resolve(&self, relative: &str) -> Option<Self> {
        let relative = relative.trim();
        
        // Absolute URL
        if relative.contains("://") {
            return Self::parse(relative);
        }
        
        // Protocol-relative
        if relative.starts_with("//") {
            return Self::parse(&alloc::format!("{}:{}", self.scheme, relative));
        }
        
        // Absolute path
        if relative.starts_with('/') {
            let mut new = self.clone();
            new.path = relative.to_string();
            new.query = None;
            new.fragment = None;
            return Some(new);
        }
        
        // Fragment only
        if relative.starts_with('#') {
            let mut new = self.clone();
            new.fragment = Some(relative[1..].to_string());
            return Some(new);
        }
        
        // Query only
        if relative.starts_with('?') {
            let mut new = self.clone();
            new.query = Some(relative[1..].to_string());
            new.fragment = None;
            return Some(new);
        }
        
        // Relative path
        let base_path = if let Some(idx) = self.path.rfind('/') {
            &self.path[..idx + 1]
        } else {
            "/"
        };
        
        let mut new_path = alloc::format!("{}{}", base_path, relative);
        
        // Normalize path (remove ./ and ../)
        new_path = normalize_path(&new_path);
        
        let mut new = self.clone();
        new.path = new_path;
        new.query = None;
        new.fragment = None;
        
        Some(new)
    }
    
    /// Get full request path (path + query)
    pub fn request_path(&self) -> String {
        if let Some(ref q) = self.query {
            alloc::format!("{}?{}", self.path, q)
        } else {
            self.path.clone()
        }
    }
}

/// Normalize a path (remove . and ..)
fn normalize_path(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    
    for segment in path.split('/') {
        match segment {
            "" | "." => {
                // Skip empty and current directory
            }
            ".." => {
                // Go up one level
                if !segments.is_empty() {
                    segments.pop();
                }
            }
            s => {
                segments.push(s);
            }
        }
    }
    
    if segments.is_empty() {
        "/".to_string()
    } else {
        let mut result = String::new();
        for seg in segments {
            result.push('/');
            result.push_str(seg);
        }
        result
    }
}

/// URL encode a string
pub fn url_encode(s: &str) -> String {
    let mut result = String::new();
    
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            ' ' => {
                result.push('+');
            }
            _ => {
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                for byte in encoded.bytes() {
                    result.push('%');
                    result.push_str(&alloc::format!("{:02X}", byte));
                }
            }
        }
    }
    
    result
}

/// URL decode a string
pub fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                    }
                }
            }
            '+' => {
                result.push(' ');
            }
            _ => {
                result.push(c);
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_url() {
        let url = Url::parse("http://example.com/path").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 80);
        assert_eq!(url.path, "/path");
    }
}
