



use alloc::string::{String, ToString};
use alloc::vec::Vec;


#[derive(Clone, Debug)]
pub struct Url {
    pub scheme: String,     
    pub host: String,       
    pub port: u16,          
    pub path: String,       
    pub query: Option<String>,  
    pub fragment: Option<String>, 
}

impl Url {
    
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        
        
        let (scheme, ef) = if let Some(idx) = url.find("://") {
            (&url[..idx], &url[idx + 3..])
        } else {
            ("http", url)
        };
        
        
        let (ef, fragment) = if let Some(idx) = ef.find('#') {
            (&ef[..idx], Some(ef[idx + 1..].to_string()))
        } else {
            (ef, None)
        };
        
        
        let (ef, query) = if let Some(idx) = ef.find('?') {
            (&ef[..idx], Some(ef[idx + 1..].to_string()))
        } else {
            (ef, None)
        };
        
        
        let (host_port, path) = if let Some(idx) = ef.find('/') {
            (&ef[..idx], ef[idx..].to_string())
        } else {
            (ef, "/".to_string())
        };
        
        
        let (host, port) = if let Some(idx) = host_port.find(':') {
            let bva = &host_port[idx + 1..];
            let port = bva.parse().unwrap_or(80);
            (&host_port[..idx], port)
        } else {
            let bru = if scheme == "https" { 443 } else { 80 };
            (host_port, bru)
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
    
    
    pub fn to_string(&self) -> String {
        let mut j = alloc::format!("{}://{}", self.scheme, self.host);
        
        let bru = if self.scheme == "https" { 443 } else { 80 };
        if self.port != bru {
            j.push(':');
            j.push_str(&alloc::format!("{}", self.port));
        }
        
        j.push_str(&self.path);
        
        if let Some(ref q) = self.query {
            j.push('?');
            j.push_str(q);
        }
        
        if let Some(ref f) = self.fragment {
            j.push('#');
            j.push_str(f);
        }
        
        j
    }
    
    
    pub fn yb(&self, xj: &str) -> Option<Self> {
        let xj = xj.trim();
        
        
        if xj.contains("://") {
            return Self::parse(xj);
        }
        
        
        if xj.starts_with("//") {
            return Self::parse(&alloc::format!("{}:{}", self.scheme, xj));
        }
        
        
        if xj.starts_with('/') {
            let mut new = self.clone();
            new.path = xj.to_string();
            new.query = None;
            new.fragment = None;
            return Some(new);
        }
        
        
        if xj.starts_with('#') {
            let mut new = self.clone();
            new.fragment = Some(xj[1..].to_string());
            return Some(new);
        }
        
        
        if xj.starts_with('?') {
            let mut new = self.clone();
            new.query = Some(xj[1..].to_string());
            new.fragment = None;
            return Some(new);
        }
        
        
        let cge = if let Some(idx) = self.path.rfind('/') {
            &self.path[..idx + 1]
        } else {
            "/"
        };
        
        let mut bcx = alloc::format!("{}{}", cge, xj);
        
        
        bcx = normalize_path(&bcx);
        
        let mut new = self.clone();
        new.path = bcx;
        new.query = None;
        new.fragment = None;
        
        Some(new)
    }
    
    
    pub fn quc(&self) -> String {
        if let Some(ref q) = self.query {
            alloc::format!("{}?{}", self.path, q)
        } else {
            self.path.clone()
        }
    }
}


fn normalize_path(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    
    for segment in path.split('/') {
        match segment {
            "" | "." => {
                
            }
            ".." => {
                
                if !segments.is_empty() {
                    segments.pop();
                }
            }
            j => {
                segments.push(j);
            }
        }
    }
    
    if segments.is_empty() {
        "/".to_string()
    } else {
        let mut result = String::new();
        for gq in segments {
            result.push('/');
            result.push_str(gq);
        }
        result
    }
}


pub fn hau(j: &str) -> String {
    let mut result = String::new();
    
    for c in j.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            ' ' => {
                result.push('+');
            }
            _ => {
                let mut buf = [0u8; 4];
                let atq = c.encode_utf8(&mut buf);
                for byte in atq.bytes() {
                    result.push('%');
                    result.push_str(&alloc::format!("{:02X}", byte));
                }
            }
        }
    }
    
    result
}


pub fn rbt(j: &str) -> String {
    let mut result = String::new();
    let mut chars = j.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let ga: String = chars.by_ref().take(2).collect();
                if ga.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&ga, 16) {
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
    fn qzv() {
        let url = Url::parse("http://example.com/path").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 80);
        assert_eq!(url.path, "/path");
    }
}
