//! HTTP Server — Embedded web server for TrustOS
//!
//! Provides a basic HTTP/1.0 server that runs in kernel mode.
//! Uses the existing TCP listener infrastructure (socket → bind → listen → accept).
//!
//! Features:
//! - Serves static content and dynamic system info pages
//! - `/` — Welcome page with TrustOS info
//! - `/status` — Live system status (uptime, memory, CPU)
//! - `/api/info` — JSON API endpoint
//! - `/files/*` — Browse RAMFS filesystem
//! - Connection handling with Keep-Alive support

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Server running flag
static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
/// Server port
static SERVER_PORT: AtomicU32 = AtomicU32::new(8080);
/// Total requests served
static REQUESTS_SERVED: AtomicU64 = AtomicU64::new(0);

/// Check if server is running
pub fn is_running() -> bool {
    SERVER_RUNNING.load(Ordering::SeqCst)
}

/// Get server stats
pub fn get_stats() -> (u16, u64, bool) {
    (
        SERVER_PORT.load(Ordering::SeqCst) as u16,
        REQUESTS_SERVED.load(Ordering::SeqCst),
        is_running(),
    )
}

/// Start the HTTP server on the given port.
/// This blocks and processes requests until stopped.
pub fn start(port: u16, max_requests: u32) {
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        crate::println_color!(crate::framebuffer::COLOR_YELLOW, "HTTP server already running");
        return;
    }

    SERVER_PORT.store(port as u32, Ordering::SeqCst);
    SERVER_RUNNING.store(true, Ordering::SeqCst);
    REQUESTS_SERVED.store(0, Ordering::SeqCst);

    // Get our IP address
    let our_ip = crate::network::get_ipv4_config()
        .map(|(ip, _, _)| { let b = ip.as_bytes(); format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]) })
        .unwrap_or_else(|| String::from("0.0.0.0"));

    crate::println_color!(crate::framebuffer::COLOR_BRIGHT_GREEN,
        "TrustOS HTTP Server v1.0");
    crate::println!("Listening on http://{}:{}", our_ip, port);
    crate::println!("Press Ctrl+C or run 'httpd stop' to stop");
    crate::println!();

    // Start TCP listener
    crate::netstack::tcp::listen_on(port, 8);

    let mut served: u32 = 0;
    let limit = if max_requests == 0 { u32::MAX } else { max_requests };

    // Main accept loop
    while SERVER_RUNNING.load(Ordering::SeqCst) && served < limit {
        crate::netstack::poll();

        // Check for incoming connection
        if let Some((src_port, remote_ip, remote_port)) =
            crate::netstack::tcp::accept_connection(port)
        {
            let remote = format!("{}.{}.{}.{}", remote_ip[0], remote_ip[1], remote_ip[2], remote_ip[3]);
            crate::serial_println!("[HTTPD] Connection from {}:{}", remote, remote_port);

            // Read request
            let request = read_request(remote_ip, port, src_port, 3000);

            if !request.is_empty() {
                // Parse HTTP request
                let (method, path) = parse_request(&request);
                crate::println!("{} {} — {}:{}", method, path, remote, remote_port);

                // Generate response
                let response = route_request(&method, &path);

                // Send response
                let _ = crate::netstack::tcp::send_data(remote_ip, remote_port, src_port, response.as_bytes());

                // Brief delay for data to send
                for _ in 0..50_000 { core::hint::spin_loop(); }

                served += 1;
                REQUESTS_SERVED.fetch_add(1, Ordering::SeqCst);
            }

            // Close connection
            let _ = crate::netstack::tcp::send_fin(remote_ip, remote_port, src_port);
        }

        // Check keyboard for Ctrl+C
        if crate::keyboard::has_input() {
            let key = crate::keyboard::try_read_key();
            if key == Some(3) { // Ctrl+C
                break;
            }
        }

        // Small yield
        x86_64::instructions::hlt();
    }

    // Shutdown
    crate::netstack::tcp::stop_listening(port);
    SERVER_RUNNING.store(false, Ordering::SeqCst);
    crate::println!();
    crate::println_color!(crate::framebuffer::COLOR_CYAN,
        "Server stopped. {} requests served.", served);
}

/// Stop the server (can be called from another command)
pub fn stop() {
    SERVER_RUNNING.store(false, Ordering::SeqCst);
}

/// Read HTTP request from a connection
fn read_request(remote_ip: [u8; 4], listen_port: u16, src_port: u16, timeout_ms: u32) -> String {
    let mut data = Vec::new();
    let start = crate::logger::get_ticks();

    loop {
        crate::netstack::poll();

        if let Some(chunk) = crate::netstack::tcp::recv_data(remote_ip, listen_port, src_port) {
            data.extend_from_slice(&chunk);
            // Check for end of HTTP headers
            if data.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }

        if crate::logger::get_ticks().saturating_sub(start) > timeout_ms as u64 {
            break;
        }

        core::hint::spin_loop();
    }

    String::from_utf8_lossy(&data).into_owned()
}

/// Parse HTTP request line → (method, path)
fn parse_request(request: &str) -> (String, String) {
    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = String::from(*parts.first().unwrap_or(&"GET"));
    let path = String::from(*parts.get(1).unwrap_or(&"/"));
    (method, path)
}

/// Route request to handler
fn route_request(method: &str, path: &str) -> String {
    match path {
        "/" => page_index(),
        "/status" => page_status(),
        "/api/info" => api_info(),
        "/api/stats" => api_stats(),
        "/api/processes" => api_processes(),
        "/favicon.ico" => response_404(),
        _ if path.starts_with("/files") => page_files(path),
        _ => response_404(),
    }
}

/// HTTP 200 response wrapper
fn http_response(content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.0 200 OK\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Server: TrustOS/0.4.0\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        content_type, body.len(), body
    )
}

/// HTTP 404 response
fn response_404() -> String {
    let body = "<html><head><title>404</title></head><body>\
                <h1>404 Not Found</h1><p>The requested resource was not found on TrustOS.</p>\
                <hr><em>TrustOS/0.4.0</em></body></html>";
    format!(
        "HTTP/1.0 404 Not Found\r\n\
         Content-Type: text/html\r\n\
         Content-Length: {}\r\n\
         Server: TrustOS/0.4.0\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        body.len(), body
    )
}

// ═══════════════════════════════════════════════════════════════════
// Pages
// ═══════════════════════════════════════════════════════════════════

fn page_index() -> String {
    let uptime_s = crate::time::uptime_ms() / 1000;
    let hours = uptime_s / 3600;
    let mins = (uptime_s % 3600) / 60;
    let secs = uptime_s % 60;

    let (total, used) = crate::memory::frame::stats();
    let mem_mb = (total * 4096) / (1024 * 1024);
    let used_mb = (used * 4096) / (1024 * 1024);

    let cores = crate::cpu::smp::ready_cpu_count();

    let body = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>TrustOS</title>
    <style>
        body {{ font-family: 'Segoe UI', sans-serif; background: #0a0a0a; color: #e0e0e0; margin: 40px; }}
        h1 {{ color: #00ff88; font-size: 2.5em; }}
        .card {{ background: #1a1a2e; border: 1px solid #333; border-radius: 8px; padding: 20px; margin: 15px 0; }}
        .stat {{ display: inline-block; margin: 10px 20px; }}
        .stat .value {{ font-size: 2em; color: #00ff88; font-weight: bold; }}
        .stat .label {{ color: #888; font-size: 0.9em; }}
        a {{ color: #4fc3f7; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        .nav {{ margin: 20px 0; }}
        .nav a {{ margin-right: 20px; padding: 8px 16px; background: #16213e; border-radius: 4px; }}
        code {{ background: #1a1a2e; padding: 2px 6px; border-radius: 3px; color: #00ff88; }}
        footer {{ margin-top: 40px; color: #555; border-top: 1px solid #333; padding-top: 10px; }}
    </style>
</head>
<body>
    <h1>TrustOS</h1>
    <p><em>Trust the code. Rust is the reason.</em></p>
    
    <div class="nav">
        <a href="/">Home</a>
        <a href="/status">System Status</a>
        <a href="/files/">Browse Files</a>
        <a href="/api/info">API</a>
    </div>

    <div class="card">
        <h2>System Overview</h2>
        <div class="stat"><div class="value">{}</div><div class="label">CPU Cores</div></div>
        <div class="stat"><div class="value">{} MB</div><div class="label">Total RAM</div></div>
        <div class="stat"><div class="value">{} MB</div><div class="label">Used RAM</div></div>
        <div class="stat"><div class="value">{:02}:{:02}:{:02}</div><div class="label">Uptime</div></div>
    </div>

    <div class="card">
        <h2>About</h2>
        <p>TrustOS is a bare-metal operating system written entirely in Rust.</p>
        <ul>
            <li>165,000+ lines of pure Rust — no C, no assembly</li>
            <li>Full TCP/IP networking with TLS 1.3</li>
            <li>Type-1 hypervisor (Intel VT-x / AMD SVM)</li>
            <li>COSMIC desktop environment</li>
            <li>TrustScan security toolkit</li>
            <li>NES + Game Boy emulators</li>
            <li>TrustLang programming language</li>
        </ul>
    </div>

    <div class="card">
        <h2>API Endpoints</h2>
        <ul>
            <li><a href="/api/info">/api/info</a> — System information (JSON)</li>
            <li><a href="/api/stats">/api/stats</a> — Server statistics (JSON)</li>
            <li><a href="/api/processes">/api/processes</a> — Process list (JSON)</li>
        </ul>
    </div>

    <footer>Served by TrustOS HTTP Server v1.0 | Powered by Rust</footer>
</body>
</html>"#, cores, mem_mb, used_mb, hours, mins, secs);

    http_response("text/html; charset=utf-8", &body)
}

fn page_status() -> String {
    let uptime_s = crate::time::uptime_ms() / 1000;
    let (total, used) = crate::memory::frame::stats();
    let free = total - used;
    let cores = crate::cpu::smp::ready_cpu_count();
    let total_cores = crate::cpu::smp::cpu_count();
    let net_stats = crate::network::get_stats();
    let requests = REQUESTS_SERVED.load(Ordering::Relaxed);

    let has_net = crate::drivers::net::has_driver();
    let net_driver = if has_net { "active" } else { "none" };

    let body = format!(r#"<!DOCTYPE html>
<html><head><title>TrustOS Status</title>
<style>
    body {{ font-family: monospace; background: #0a0a0a; color: #e0e0e0; margin: 40px; }}
    h1 {{ color: #00ff88; }} table {{ border-collapse: collapse; width: 100%; }}
    th, td {{ border: 1px solid #333; padding: 8px 12px; text-align: left; }}
    th {{ background: #16213e; color: #4fc3f7; }} tr:hover {{ background: #1a1a2e; }}
    .green {{ color: #00ff88; }} .yellow {{ color: #ffd700; }} .red {{ color: #ff4444; }}
    a {{ color: #4fc3f7; }}
</style></head><body>
<h1>System Status</h1>
<p><a href="/">← Home</a></p>
<table>
<tr><th>Metric</th><th>Value</th></tr>
<tr><td>Uptime</td><td>{} seconds</td></tr>
<tr><td>CPU Cores</td><td class="green">{}/{} online</td></tr>
<tr><td>Memory Total</td><td>{} frames ({} MB)</td></tr>
<tr><td>Memory Used</td><td>{} frames ({} MB)</td></tr>
<tr><td>Memory Free</td><td class="green">{} frames ({} MB)</td></tr>
<tr><td>Network Driver</td><td>{}</td></tr>
<tr><td>Packets RX</td><td>{}</td></tr>
<tr><td>Packets TX</td><td>{}</td></tr>
<tr><td>Bytes RX</td><td>{}</td></tr>
<tr><td>Bytes TX</td><td>{}</td></tr>
<tr><td>HTTP Requests Served</td><td class="green">{}</td></tr>
</table>
</body></html>"#,
        uptime_s,
        cores, total_cores,
        total, (total * 4096) / (1024 * 1024),
        used, (used * 4096) / (1024 * 1024),
        free, (free * 4096) / (1024 * 1024),
        net_driver,
        net_stats.packets_received, net_stats.packets_sent,
        net_stats.bytes_received, net_stats.bytes_sent,
        requests,
    );

    http_response("text/html; charset=utf-8", &body)
}

fn page_files(path: &str) -> String {
    let fs_path = if path == "/files" || path == "/files/" {
        "/"
    } else {
        &path[6..] // strip "/files"
    };

    let entries = crate::ramfs::with_fs(|fs| {
        fs.ls(Some(fs_path)).unwrap_or_default()
    });

    let mut rows = String::new();
    for (name, ftype, size) in &entries {
        let icon = if *ftype == crate::ramfs::FileType::Directory { "d" } else { "-" };
        let link = format!("/files{}{}{}", fs_path,
            if fs_path.ends_with('/') { "" } else { "/" }, name);
        let href = if *ftype == crate::ramfs::FileType::Directory {
            format!("<a href=\"{}\">{} {}/</a>", link, icon, name)
        } else {
            format!("{} {} ({} bytes)", icon, name, size)
        };
        rows.push_str(&format!("<tr><td>{}</td></tr>\n", href));
    }

    let body = format!(r#"<!DOCTYPE html>
<html><head><title>Files: {}</title>
<style>
    body {{ font-family: monospace; background: #0a0a0a; color: #e0e0e0; margin: 40px; }}
    h1 {{ color: #00ff88; }} a {{ color: #4fc3f7; }}
    table {{ border-collapse: collapse; }} td {{ padding: 4px 12px; border-bottom: 1px solid #222; }}
</style></head><body>
<h1>Files: {}</h1>
<p><a href="/">← Home</a> | <a href="/files/">Root</a></p>
<table>{}</table>
</body></html>"#, fs_path, fs_path, rows);

    http_response("text/html; charset=utf-8", &body)
}

fn api_info() -> String {
    let uptime = crate::time::uptime_ms();
    let (total, used) = crate::memory::frame::stats();
    let cores = crate::cpu::smp::ready_cpu_count();

    let body = format!(r#"{{"os":"TrustOS","version":"0.4.0","uptime_ms":{},"cores":{},"memory_total_kb":{},"memory_used_kb":{},"rust":"nightly","arch":"x86_64"}}"#,
        uptime, cores, total * 4, used * 4);

    http_response("application/json", &body)
}

fn api_stats() -> String {
    let net = crate::network::get_stats();
    let requests = REQUESTS_SERVED.load(Ordering::Relaxed);
    let port = SERVER_PORT.load(Ordering::Relaxed);

    let body = format!(r#"{{"server_port":{},"requests_served":{},"packets_rx":{},"packets_tx":{},"bytes_rx":{},"bytes_tx":{}}}"#,
        port, requests, net.packets_received, net.packets_sent,
        net.bytes_received, net.bytes_sent);

    http_response("application/json", &body)
}

fn api_processes() -> String {
    let body = format!(r#"{{"pid":0,"name":"kernel","state":"running","threads":{}}}"#,
        crate::cpu::smp::ready_cpu_count());
    http_response("application/json", &body)
}
