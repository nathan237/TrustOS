








extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

use super::Ag;




#[derive(Debug, Clone)]
pub struct JsSandboxConfig {
    
    pub timeout_ms: u64,
    
    pub max_stack_depth: usize,
    
    pub max_string_length: usize,
    
    pub max_array_length: usize,
    
    pub max_object_props: usize,
    
    pub max_memory_bytes: usize,
    
    pub allow_console: bool,
    
    pub allow_timers: bool,
    
    pub allow_network: bool,
    
    pub allow_storage: bool,
}

impl Default for JsSandboxConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,          
            max_stack_depth: 64,
            max_string_length: 65536,  
            max_array_length: 4096,
            max_object_props: 256,
            max_memory_bytes: 1024 * 1024, 
            allow_console: true,
            allow_timers: false,       
            allow_network: false,      
            allow_storage: true,
        }
    }
}




const BNY_: &[&str] = &[
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


pub fn olb(code: &str) -> Vec<String> {
    let mut bpj = Vec::new();
    let gj = code.to_ascii_lowercase();

    for &pattern in BNY_ {
        
        if gj.contains(&pattern.to_ascii_lowercase()) {
            bpj.push(format!("blocked pattern: {}", pattern));
        }
    }

    
    if code.len() > 100_000 {
        bpj.push(String::from("code exceeds 100KB size limit"));
    }

    
    let imo = code.chars().fold((0i32, 0i32), |(depth, max), c| {
        match c {
            '{' | '[' | '(' => (depth + 1, core::cmp::max(max, depth + 1)),
            '}' | ']' | ')' => (depth - 1, max),
            _ => (depth, max),
        }
    }).1;
    if imo > 32 {
        bpj.push(format!("excessive nesting depth: {}", imo));
    }

    bpj
}




#[derive(Debug)]
pub struct Mi {
    
    pub output: Vec<String>,
    
    pub return_value: String,
    
    pub completed: bool,
    
    pub error: Option<String>,
    
    pub elapsed_ms: u64,
}


pub struct JsSandbox {
    pub sandbox_id: Ag,
    pub config: JsSandboxConfig,
    
    pub persistent_vars: BTreeMap<String, String>,
    
    pub console_buffer: Vec<String>,
    
    pub exec_count: usize,
    
    pub blocked_count: usize,
}

impl JsSandbox {
    pub fn new(sandbox_id: Ag, config: JsSandboxConfig) -> Self {
        Self {
            sandbox_id,
            config,
            persistent_vars: BTreeMap::new(),
            console_buffer: Vec::new(),
            exec_count: 0,
            blocked_count: 0,
        }
    }

    
    
    pub fn execute(&mut self, code: &str) -> Mi {
        self.exec_count += 1;

        
        let bpj = olb(code);
        if !bpj.is_empty() {
            self.blocked_count += 1;
            crate::serial_println!("[sandbox:{}] JS BLOCKED: {} threats found",
                self.sandbox_id.0, bpj.len());
            for t in &bpj {
                crate::serial_println!("[sandbox:{}]   - {}", self.sandbox_id.0, t);
            }
            return Mi {
                output: Vec::new(),
                return_value: String::from("undefined"),
                completed: false,
                error: Some(format!("Security: {} threat(s) detected: {}", 
                    bpj.len(), bpj.first().unwrap_or(&String::new()))),
                elapsed_ms: 0,
            };
        }

        
        let start = crate::time::uptime_ms();

        
        let mut ab = crate::browser::js_engine::JsContext::new();

        
        let result = ab.execute(code);

        let bb = crate::time::uptime_ms().saturating_sub(start);

        
        let output: Vec<String> = ab.console_output.iter()
            .map(|j| j.clone())
            .collect();
        
        
        for line in &output {
            self.console_buffer.push(line.clone());
            
            if self.console_buffer.len() > 100 {
                self.console_buffer.remove(0);
            }
        }

        
        if bb > self.config.timeout_ms && self.config.timeout_ms > 0 {
            return Mi {
                output,
                return_value: String::from("undefined"),
                completed: false,
                error: Some(format!("Timeout: execution exceeded {}ms limit", self.config.timeout_ms)),
                elapsed_ms: bb,
            };
        }

        let jao = match &result {
            Ok(val) => val.to_string(),
            Err(e) => format!("Error: {}", e),
        };

        Mi {
            output,
            return_value: jao.clone(),
            completed: result.is_ok(),
            error: if result.is_err() { Some(jao) } else { None },
            elapsed_ms: bb,
        }
    }

    
    pub fn execute_inline_scripts(&mut self, ajx: &str) -> Vec<Mi> {
        let mut results = Vec::new();

        
        let mut ezt = 0;
        loop {
            let gj = ajx[ezt..].to_ascii_lowercase();
            let gwe = match gj.find("<script") {
                Some(pos) => pos + ezt,
                None => break,
            };

            
            let brc = match ajx[gwe..].find('>') {
                Some(pos) => gwe + pos + 1,
                None => break,
            };

            
            let tag = &ajx[gwe..brc];
            if tag.to_ascii_lowercase().contains("src=") {
                
                ezt = brc;
                continue;
            }

            
            let hvw = match ajx[brc..].to_ascii_lowercase().find("</script>") {
                Some(pos) => brc + pos,
                None => break,
            };

            let jdm = &ajx[brc..hvw];
            if !jdm.trim().is_empty() {
                let result = self.execute(jdm.trim());
                results.push(result);
            }

            ezt = hvw + 9; 
        }

        results
    }

    
    pub fn console_output(&self) -> &[String] {
        &self.console_buffer
    }

    
    pub fn pzw(&mut self) {
        self.console_buffer.clear();
    }

    
    pub fn stats(&self) -> (usize, usize) {
        (self.exec_count, self.blocked_count)
    }
}
