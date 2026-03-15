








extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

use super::Ax;




#[derive(Debug, Clone)]
pub struct JsSandboxConfig {
    
    pub sg: u64,
    
    pub ulz: usize,
    
    pub uma: usize,
    
    pub uks: usize,
    
    pub ulq: usize,
    
    pub jfj: usize,
    
    pub qha: bool,
    
    pub qhd: bool,
    
    pub qhb: bool,
    
    pub qhc: bool,
}

impl Default for JsSandboxConfig {
    fn default() -> Self {
        Self {
            sg: 5000,          
            ulz: 64,
            uma: 65536,  
            uks: 4096,
            ulq: 256,
            jfj: 1024 * 1024, 
            qha: true,
            qhd: false,       
            qhb: false,      
            qhc: true,
        }
    }
}




const BLG_: &[&str] = &[
    "eval(",
    "eval (",
    "new Function(",
    "new Function (",
    "Function(",
    "setTimeout(\"",      
    "setTimeout('",
    "setInterval(\"",
    "setInterval('",
    "__proto__",          
    "constructor[",      
    "constructor.",       
    "import(",           
    "import (",
    "require(",          
    "process.",          
    "global.",           
    "globalThis.",       
    "Reflect.",          
    "Proxy(",            
    "window.",           
    "document.cookie",   
    "document.write",    
    "innerHTML",         
    "outerHTML",
    "insertAdjacentHTML",
    "XMLHttpRequest",    
    "WebSocket",         
    "fetch(",            
    "Worker(",           
    "SharedArrayBuffer", 
    "Atomics.",          
];


pub fn wdt(aj: &str) -> Vec<String> {
    let mut dws = Vec::new();
    let pb = aj.avd();

    for &pattern in BLG_ {
        
        if pb.contains(&pattern.avd()) {
            dws.push(format!("blocked pattern: {}", pattern));
        }
    }

    
    if aj.len() > 100_000 {
        dws.push(String::from("code exceeds 100KB size limit"));
    }

    
    let oly = aj.bw().cqs((0i32, 0i32), |(eo, am), r| {
        match r {
            '{' | '[' | '(' => (eo + 1, core::cmp::am(am, eo + 1)),
            '}' | ']' | ')' => (eo - 1, am),
            _ => (eo, am),
        }
    }).1;
    if oly > 32 {
        dws.push(format!("excessive nesting depth: {}", oly));
    }

    dws
}




#[derive(Debug)]
pub struct Ack {
    
    pub an: Vec<String>,
    
    pub jmj: String,
    
    pub cpn: bool,
    
    pub zt: Option<String>,
    
    pub oz: u64,
}


pub struct JsSandbox {
    pub afh: Ax,
    pub config: JsSandboxConfig,
    
    pub vgv: BTreeMap<String, String>,
    
    pub dzs: Vec<String>,
    
    pub kuh: usize,
    
    pub kec: usize,
}

impl JsSandbox {
    pub fn new(afh: Ax, config: JsSandboxConfig) -> Self {
        Self {
            afh,
            config,
            vgv: BTreeMap::new(),
            dzs: Vec::new(),
            kuh: 0,
            kec: 0,
        }
    }

    
    
    pub fn bna(&mut self, aj: &str) -> Ack {
        self.kuh += 1;

        
        let dws = wdt(aj);
        if !dws.is_empty() {
            self.kec += 1;
            crate::serial_println!("[sandbox:{}] JS BLOCKED: {} threats found",
                self.afh.0, dws.len());
            for ab in &dws {
                crate::serial_println!("[sandbox:{}]   - {}", self.afh.0, ab);
            }
            return Ack {
                an: Vec::new(),
                jmj: String::from("undefined"),
                cpn: false,
                zt: Some(format!("Security: {} threat(s) detected: {}", 
                    dws.len(), dws.fv().unwrap_or(&String::new()))),
                oz: 0,
            };
        }

        
        let ay = crate::time::lc();

        
        let mut be = crate::browser::js_engine::JsContext::new();

        
        let result = be.bna(aj);

        let ez = crate::time::lc().ao(ay);

        
        let an: Vec<String> = be.ffp.iter()
            .map(|e| e.clone())
            .collect();
        
        
        for line in &an {
            self.dzs.push(line.clone());
            
            if self.dzs.len() > 100 {
                self.dzs.remove(0);
            }
        }

        
        if ez > self.config.sg && self.config.sg > 0 {
            return Ack {
                an,
                jmj: String::from("undefined"),
                cpn: false,
                zt: Some(format!("Timeout: execution exceeded {}ms limit", self.config.sg)),
                oz: ez,
            };
        }

        let pcz = match &result {
            Ok(ap) => ap.to_string(),
            Err(aa) => format!("Error: {}", aa),
        };

        Ack {
            an,
            jmj: pcz.clone(),
            cpn: result.is_ok(),
            zt: if result.is_err() { Some(pcz) } else { None },
            oz: ez,
        }
    }

    
    pub fn sop(&mut self, brb: &str) -> Vec<Ack> {
        let mut hd = Vec::new();

        
        let mut jnz = 0;
        loop {
            let pb = brb[jnz..].avd();
            let mhk = match pb.du("<script") {
                Some(u) => u + jnz,
                None => break,
            };

            
            let dzt = match brb[mhk..].du('>') {
                Some(u) => mhk + u + 1,
                None => break,
            };

            
            let ll = &brb[mhk..dzt];
            if ll.avd().contains("src=") {
                
                jnz = dzt;
                continue;
            }

            
            let nqd = match brb[dzt..].avd().du("</script>") {
                Some(u) => dzt + u,
                None => break,
            };

            let pgp = &brb[dzt..nqd];
            if !pgp.em().is_empty() {
                let result = self.bna(pgp.em());
                hd.push(result);
            }

            jnz = nqd + 9; 
        }

        hd
    }

    
    pub fn ffp(&self) -> &[String] {
        &self.dzs
    }

    
    pub fn yii(&mut self) {
        self.dzs.clear();
    }

    
    pub fn cm(&self) -> (usize, usize) {
        (self.kuh, self.kec)
    }
}
