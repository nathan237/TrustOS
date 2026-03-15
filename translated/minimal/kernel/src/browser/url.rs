



use alloc::string::{String, Gd};
use alloc::vec::Vec;


#[derive(Clone, Debug)]
pub struct Url {
    pub eyc: String,     
    pub kh: String,       
    pub port: u16,          
    pub path: String,       
    pub query: Option<String>,  
    pub fja: Option<String>, 
}

impl Url {
    
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.em();
        
        
        let (eyc, kr) = if let Some(w) = url.du("://") {
            (&url[..w], &url[w + 3..])
        } else {
            ("http", url)
        };
        
        
        let (kr, fja) = if let Some(w) = kr.du('#') {
            (&kr[..w], Some(kr[w + 1..].to_string()))
        } else {
            (kr, None)
        };
        
        
        let (kr, query) = if let Some(w) = kr.du('?') {
            (&kr[..w], Some(kr[w + 1..].to_string()))
        } else {
            (kr, None)
        };
        
        
        let (bej, path) = if let Some(w) = kr.du('/') {
            (&kr[..w], kr[w..].to_string())
        } else {
            (kr, "/".to_string())
        };
        
        
        let (kh, port) = if let Some(w) = bej.du(':') {
            let frc = &bej[w + 1..];
            let port = frc.parse().unwrap_or(80);
            (&bej[..w], port)
        } else {
            let eaq = if eyc == "https" { 443 } else { 80 };
            (bej, eaq)
        };
        
        if kh.is_empty() {
            return None;
        }
        
        Some(Self {
            eyc: eyc.to_string(),
            kh: kh.to_string(),
            port,
            path,
            query,
            fja,
        })
    }
    
    
    pub fn to_string(&self) -> String {
        let mut e = alloc::format!("{}://{}", self.eyc, self.kh);
        
        let eaq = if self.eyc == "https" { 443 } else { 80 };
        if self.port != eaq {
            e.push(':');
            e.t(&alloc::format!("{}", self.port));
        }
        
        e.t(&self.path);
        
        if let Some(ref fm) = self.query {
            e.push('?');
            e.t(fm);
        }
        
        if let Some(ref bb) = self.fja {
            e.push('#');
            e.t(bb);
        }
        
        e
    }
    
    
    pub fn ayo(&self, atj: &str) -> Option<Self> {
        let atj = atj.em();
        
        
        if atj.contains("://") {
            return Self::parse(atj);
        }
        
        
        if atj.cj("//") {
            return Self::parse(&alloc::format!("{}:{}", self.eyc, atj));
        }
        
        
        if atj.cj('/') {
            let mut new = self.clone();
            new.path = atj.to_string();
            new.query = None;
            new.fja = None;
            return Some(new);
        }
        
        
        if atj.cj('#') {
            let mut new = self.clone();
            new.fja = Some(atj[1..].to_string());
            return Some(new);
        }
        
        
        if atj.cj('?') {
            let mut new = self.clone();
            new.query = Some(atj[1..].to_string());
            new.fja = None;
            return Some(new);
        }
        
        
        let fdd = if let Some(w) = self.path.bhx('/') {
            &self.path[..w + 1]
        } else {
            "/"
        };
        
        let mut dag = alloc::format!("{}{}", fdd, atj);
        
        
        dag = bro(&dag);
        
        let mut new = self.clone();
        new.path = dag;
        new.query = None;
        new.fja = None;
        
        Some(new)
    }
    
    
    pub fn zju(&self) -> String {
        if let Some(ref fm) = self.query {
            alloc::format!("{}?{}", self.path, fm)
        } else {
            self.path.clone()
        }
    }
}


fn bro(path: &str) -> String {
    let mut jq: Vec<&str> = Vec::new();
    
    for ie in path.adk('/') {
        match ie {
            "" | "." => {
                
            }
            ".." => {
                
                if !jq.is_empty() {
                    jq.pop();
                }
            }
            e => {
                jq.push(e);
            }
        }
    }
    
    if jq.is_empty() {
        "/".to_string()
    } else {
        let mut result = String::new();
        for pk in jq {
            result.push('/');
            result.t(pk);
        }
        result
    }
}


pub fn moh(e: &str) -> String {
    let mut result = String::new();
    
    for r in e.bw() {
        match r {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(r);
            }
            ' ' => {
                result.push('+');
            }
            _ => {
                let mut k = [0u8; 4];
                let ckd = r.hia(&mut k);
                for hf in ckd.bf() {
                    result.push('%');
                    result.t(&alloc::format!("{:02X}", hf));
                }
            }
        }
    }
    
    result
}


pub fn zux(e: &str) -> String {
    let mut result = String::new();
    let mut bw = e.bw().ltk();
    
    while let Some(r) = bw.next() {
        match r {
            '%' => {
                let nu: String = bw.ygv().take(2).collect();
                if nu.len() == 2 {
                    if let Ok(hf) = u8::wa(&nu, 16) {
                        result.push(hf as char);
                    }
                }
            }
            '+' => {
                result.push(' ');
            }
            _ => {
                result.push(r);
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn zsb() {
        let url = Url::parse("http://example.com/path").unwrap();
        assert_eq!(url.eyc, "http");
        assert_eq!(url.kh, "example.com");
        assert_eq!(url.port, 80);
        assert_eq!(url.path, "/path");
    }
}
