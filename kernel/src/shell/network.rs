//! Network Commands  Web browser, sandbox, container, HTML rendering
//!
//! Includes: browse/www, sandbox/websandbox, container/webcontainer,
//! HTML document rendering, file reading helpers.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA, COLOR_GRAY};
/// Web browser command
pub(super) fn cmd_browse(args: &[&str]) {
    if args.is_empty() {
        crate::println!("TrustOS Web Browser");
        crate::println!("Usage: browse <url>");
        crate::println!("  Example: browse http://example.com");
        crate::println!("  Example: browse http://info.cern.ch");
        crate::println!("");
        crate::println!("Note: Only HTTP is supported (no HTTPS yet)");
        return;
    }
    
    let url = args[0];
    let url = if !url.starts_with("http://") && !url.starts_with("https://") {
        alloc::format!("http://{}", url)
    } else {
        String::from(url)
    };
    
    crate::println!("[Browser] Loading {}...", url);
    
    // Create browser instance
    let mut browser = crate::browser::Browser::new(800, 600);
    
    match browser.navigate(&url) {
        Ok(()) => {
            crate::println_color!(COLOR_GREEN, "Page loaded successfully!");
            
            // Get document title
            if let Some(ref doc) = browser.document {
                if !doc.title.is_empty() {
                    crate::println!("Title: {}", doc.title);
                }
                
                crate::println!("");
                
                // Render text content to console
                render_document_text(doc, 0);
            }
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Failed to load page: {}", e);
        }
    }
}

// ===========================================================================
// Web Container -- persistent isolated web service management
// ===========================================================================

pub(super) fn cmd_container(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");
    match subcmd {
        "status" | "info" => {
            if args.len() >= 2 {
                // Show specific container
                let id: u32 = args[1].parse().unwrap_or(0);
                match crate::sandbox::container::container_status(id) {
                    Some(s) => crate::println!("{}", s),
                    None => crate::println_color!(COLOR_RED, "Container #{} not found", id),
                }
            } else {
                // Show daemon status
                crate::println!("{}", crate::sandbox::container::daemon_status());
            }
        }

        "list" | "ls" => {
            let containers = crate::sandbox::container::list_containers();
            if containers.is_empty() {
                crate::println!("No containers.");
            } else {
                crate::println_color!(COLOR_CYAN, "Web Containers:");
                crate::println!("  {:>4}  {:>10}  {:>3}  {}", "ID", "Health", "Def", "Name");
                crate::println!("  {}", "-".repeat(45));
                for (id, name, health, is_default) in &containers {
                    let def = if *is_default { " * " } else { "   " };
                    crate::println!("  {:>4}  {:>10?}  {}  {}", id, health, def, name);
                }
            }
        }

        "create" => {
            // container create [name] [secure|default|dev]
            let preset_str = args.get(1).copied().unwrap_or("default");
            let config = match preset_str {
                "secure" => crate::sandbox::container::ContainerConfig::secure(),
                "dev" => crate::sandbox::container::ContainerConfig::dev(),
                _ => {
                    let mut cfg = crate::sandbox::container::ContainerConfig::default();
                    // If first arg doesn't look like a preset, use it as the name
                    if preset_str != "default" {
                        cfg.name = String::from(preset_str);
                    }
                    cfg
                }
            };
            let mut daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
            let id = daemon.create_container(config);
            match daemon.start_container(id) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Container #{} created and started", id),
                Err(e) => crate::println_color!(COLOR_RED, "Created #{} but failed to start: {:?}", id, e),
            }
        }

        "go" | "navigate" | "open" => {
            // container go <url> [container_id]
            if args.len() < 2 {
                crate::println!("Usage: container go <url> [container_id]");
                return;
            }
            let url = args[1];
            let container_id: Option<u32> = args.get(2).and_then(|s| s.parse().ok());

            crate::println_color!(COLOR_CYAN, "[Container] Fetching {}...", url);
            let mut daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
            match daemon.navigate(container_id, url) {
                Ok(resp) => {
                    crate::println_color!(COLOR_GREEN, "  Status: {} | {} | {} bytes",
                        resp.status_code, resp.content_type, resp.body.len());

                    if resp.is_html() {
                        let html_str = resp.body_string();
                        let doc = crate::browser::html_parser::parse_html(&html_str);
                        if !doc.title.is_empty() {
                            crate::println!("  Title: {}", doc.title);
                        }
                        crate::println!();
                        render_document_text(&doc, 0);
                    } else {
                        let text = resp.body_string();
                        let preview: String = text.chars().take(2000).collect();
                        crate::println!("{}", preview);
                        if text.len() > 2000 {
                            crate::println!("  ... ({} bytes total)", text.len());
                        }
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "  Error: {:?}", e);
                }
            }
        }

        "stop" => {
            if args.len() < 2 {
                crate::println!("Usage: container stop <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let mut daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
            match daemon.stop_container(id) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Container #{} stopped", id),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
            }
        }

        "start" => {
            if args.len() < 2 {
                crate::println!("Usage: container start <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let mut daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
            match daemon.start_container(id) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Container #{} started", id),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
            }
        }

        "restart" => {
            if args.len() < 2 {
                crate::println!("Usage: container restart <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let mut daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
            match daemon.restart_container(id) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Container #{} restarted", id),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
            }
        }

        "destroy" | "rm" => {
            if args.len() < 2 {
                crate::println!("Usage: container destroy <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let mut daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
            match daemon.destroy_container(id) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Container #{} destroyed", id),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
            }
        }

        "allow" => {
            if args.len() < 3 {
                crate::println!("Usage: container allow <id> <domain>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            // Get sandbox ID then drop daemon lock to avoid deadlock
            let sandbox_id = {
                let mut daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
                if let Some(container) = daemon.get_mut(id) {
                    container.config.allowed_domains.push(String::from(domain));
                    container.sandbox_id
                } else {
                    crate::println_color!(COLOR_RED, "Container #{} not found", id);
                    return;
                }
            };
            // Apply to live sandbox outside daemon lock
            if let Some(sid) = sandbox_id {
                let mut mgr = crate::sandbox::SANDBOX_MANAGER.lock();
                if let Some(sb) = mgr.get_mut(sid) {
                    sb.policy.allow_domain(domain);
                }
            }
            crate::println_color!(COLOR_GREEN, "Domain '{}' allowed in container #{}", domain, id);
        }

        "deny" | "block" => {
            if args.len() < 3 {
                crate::println!("Usage: container deny <id> <domain>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            let sandbox_id = {
                let mut daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
                if let Some(container) = daemon.get_mut(id) {
                    container.config.blocked_domains.push(String::from(domain));
                    container.sandbox_id
                } else {
                    crate::println_color!(COLOR_RED, "Container #{} not found", id);
                    return;
                }
            };
            if let Some(sid) = sandbox_id {
                let mut mgr = crate::sandbox::SANDBOX_MANAGER.lock();
                if let Some(sb) = mgr.get_mut(sid) {
                    sb.policy.deny_domain(domain);
                }
            }
            crate::println_color!(COLOR_GREEN, "Domain '{}' blocked in container #{}", domain, id);
        }

        "watchdog" => {
            crate::sandbox::container::watchdog_tick();
            crate::println_color!(COLOR_GREEN, "Watchdog tick executed.");
        }

        "history" => {
            if args.len() < 2 {
                crate::println!("Usage: container history <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let daemon = crate::sandbox::container::CONTAINER_DAEMON.lock();
            if let Some(container) = daemon.get(id) {
                if container.history.is_empty() {
                    crate::println!("No navigation history.");
                } else {
                    crate::println_color!(COLOR_CYAN, "Navigation history for container #{}:", id);
                    for (i, url) in container.history.iter().enumerate() {
                        crate::println!("  {}. {}", i + 1, url);
                    }
                }
            } else {
                crate::println_color!(COLOR_RED, "Container #{} not found", id);
            }
        }

        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Web Container -- persistent isolated web service daemon");
            crate::println!();
            crate::println!("Usage: container <command> [args...]");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Daemon:");
            crate::println!("    status [id]           Show daemon or container status");
            crate::println!("    list                  List all containers");
            crate::println!("    watchdog              Run watchdog health check");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Lifecycle:");
            crate::println!("    create [secure|dev]   Create a new container");
            crate::println!("    start <id>            Start a container");
            crate::println!("    stop <id>             Stop a container");
            crate::println!("    restart <id>          Restart a container");
            crate::println!("    destroy <id>          Remove a container");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Navigation:");
            crate::println!("    go <url> [id]         Navigate through container");
            crate::println!("    history <id>          Show navigation history");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Security:");
            crate::println!("    allow <id> <domain>   Allow domain access");
            crate::println!("    deny <id> <domain>    Block domain access");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "  The default container is auto-started at boot.");
            crate::println_color!(COLOR_YELLOW, "  All web traffic is proxied through the kernel.");
            crate::println_color!(COLOR_YELLOW, "  Isolation: software (capabilities) | hardware (EPT - future)");
        }
    }
}

// ===========================================================================
// Web Sandbox -- capability-gated isolated web execution
// ===========================================================================

pub(super) fn cmd_sandbox(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");
    match subcmd {
        "open" | "navigate" | "go" => {
            // sandbox open <url> [preset]
            if args.len() < 2 {
                crate::println!("Usage: sandbox open <url> [strict|moderate|permissive]");
                return;
            }
            let url = args[1];
            let preset = match args.get(2).copied() {
                Some("strict") => crate::sandbox::policy::PolicyPreset::Strict,
                Some("permissive") => crate::sandbox::policy::PolicyPreset::Permissive,
                _ => crate::sandbox::policy::PolicyPreset::Moderate,
            };

            crate::println_color!(COLOR_CYAN, "[Sandbox] Creating sandbox ({:?})...", preset);
            let id = crate::sandbox::create(preset, Some(url));
            crate::println!("  Sandbox #{} created", id.0);

            crate::println_color!(COLOR_CYAN, "[Sandbox] Navigating to {}...", url);
            match crate::sandbox::navigate(id, url) {
                Ok(resp) => {
                    crate::println_color!(COLOR_GREEN, "  Status: {} | Content-Type: {} | {} bytes",
                        resp.status_code, resp.content_type, resp.body.len());

                    if resp.is_html() {
                        // Parse and render through browser engine
                        let html_str = resp.body_string();
                        let doc = crate::browser::html_parser::parse_html(&html_str);
                        if !doc.title.is_empty() {
                            crate::println!("  Title: {}", doc.title);
                        }
                        crate::println!();
                        render_document_text(&doc, 0);

                        // Execute inline scripts through JS sandbox
                        let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
                        let policy = mgr.get(id).map(|s| s.policy.js_allowed()).unwrap_or(false);
                        drop(mgr);
                        if policy {
                            let mut jss = crate::sandbox::js_sandbox::JsSandbox::new(
                                id, crate::sandbox::js_sandbox::JsSandboxConfig::default()
                            );
                            let results = jss.execute_inline_scripts(&html_str);
                            if !results.is_empty() {
                                crate::println_color!(COLOR_YELLOW, "\n  [JS] {} script(s) processed", results.len());
                                for (i, r) in results.iter().enumerate() {
                                    if r.completed {
                                        crate::println_color!(COLOR_GREEN, "    Script {}: OK ({}ms)", i+1, r.elapsed_ms);
                                    } else {
                                        crate::println_color!(COLOR_RED, "    Script {}: {}", i+1,
                                            r.error.as_deref().unwrap_or("failed"));
                                    }
                                    for line in &r.output {
                                        crate::println!("      > {}", line);
                                    }
                                }
                            }
                        } else {
                            crate::println_color!(COLOR_YELLOW, "  [JS blocked by policy]");
                        }
                    } else {
                        // Non-HTML: show raw body (truncated)
                        let text = resp.body_string();
                        let preview: String = text.chars().take(2000).collect();
                        crate::println!("{}", preview);
                        if text.len() > 2000 {
                            crate::println!("  ... ({} bytes total)", text.len());
                        }
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "  Error: {:?}", e);
                }
            }
        }

        "list" | "ls" => {
            let sandboxes = crate::sandbox::list();
            if sandboxes.is_empty() {
                crate::println!("No active sandboxes.");
            } else {
                crate::println_color!(COLOR_CYAN, "Active sandboxes:");
                crate::println!("  {:>4}  {:>10}  {}", "ID", "State", "Label");
                crate::println!("  {}", "-".repeat(40));
                for (id, label, state) in &sandboxes {
                    crate::println!("  {:>4}  {:>10?}  {}", id.0, state, label);
                }
            }
        }

        "status" | "info" => {
            if args.len() < 2 {
                crate::println!("Usage: sandbox status <id>");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let id = crate::sandbox::SandboxId(id_num);
            match crate::sandbox::status_string(id) {
                Some(s) => crate::println!("{}", s),
                None => crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num),
            }
        }

        "kill" | "destroy" | "close" => {
            if args.len() < 2 {
                crate::println!("Usage: sandbox kill <id|all>");
                return;
            }
            if args[1] == "all" {
                let sandboxes = crate::sandbox::list();
                for (id, _, _) in &sandboxes {
                    let _ = crate::sandbox::destroy(*id);
                }
                crate::println_color!(COLOR_GREEN, "All sandboxes destroyed.");
            } else {
                let id_num: u64 = args[1].parse().unwrap_or(0);
                let id = crate::sandbox::SandboxId(id_num);
                match crate::sandbox::destroy(id) {
                    Ok(()) => crate::println_color!(COLOR_GREEN, "Sandbox #{} destroyed.", id_num),
                    Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                }
            }
        }

        "allow" => {
            // sandbox allow <id> <domain>
            if args.len() < 3 {
                crate::println!("Usage: sandbox allow <id> <domain>");
                crate::println!("  Example: sandbox allow 1 example.com");
                crate::println!("  Example: sandbox allow 1 *.github.com");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            let mut mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get_mut(crate::sandbox::SandboxId(id_num)) {
                sb.policy.allow_domain(domain);
                crate::println_color!(COLOR_GREEN, "Domain '{}' allowed in sandbox #{}", domain, id_num);
            } else {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
            }
        }

        "deny" | "block" => {
            // sandbox deny <id> <domain>
            if args.len() < 3 {
                crate::println!("Usage: sandbox deny <id> <domain>");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            let mut mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get_mut(crate::sandbox::SandboxId(id_num)) {
                sb.policy.deny_domain(domain);
                crate::println_color!(COLOR_GREEN, "Domain '{}' blocked in sandbox #{}", domain, id_num);
            } else {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
            }
        }

        "policy" => {
            if args.len() < 2 {
                crate::println!("Usage: sandbox policy <id>");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get(crate::sandbox::SandboxId(id_num)) {
                crate::println!("{}", sb.policy.summary());
            } else {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
            }
        }

        "fs" => {
            // sandbox fs <id> [ls|tree|write|read|del] [path] [data]
            if args.len() < 3 {
                crate::println!("Usage: sandbox fs <id> <ls|tree|read|write|del> [path] [data]");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let fs_cmd = args[2];
            let mut mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get_mut(crate::sandbox::SandboxId(id_num)) {
                match fs_cmd {
                    "tree" => {
                        crate::println!("{}", sb.filesystem.tree());
                    }
                    "ls" => {
                        let dir = args.get(3).copied().unwrap_or("/");
                        match sb.filesystem.list(dir) {
                            Ok(entries) => {
                                crate::println_color!(COLOR_CYAN, "{}:", dir);
                                for (path, ftype, size) in &entries {
                                    let icon = if *ftype == &crate::sandbox::fs::SandboxFileType::Directory { "[D]" } else { "[F]" };
                                    crate::println!("  {} {} ({} bytes)", icon, path, size);
                                }
                            }
                            Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                        }
                    }
                    "read" => {
                        let path = args.get(3).copied().unwrap_or("/");
                        match sb.filesystem.read(path) {
                            Ok(data) => {
                                let text = core::str::from_utf8(data).unwrap_or("<binary>");
                                crate::println!("{}", text);
                            }
                            Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                        }
                    }
                    "write" => {
                        if args.len() < 5 {
                            crate::println!("Usage: sandbox fs <id> write <path> <data>");
                            return;
                        }
                        let path = args[3];
                        let data = args[4..].join(" ");
                        match sb.filesystem.write(path, data.as_bytes(), "shell") {
                            Ok(()) => crate::println_color!(COLOR_GREEN, "Written {} bytes to {}", data.len(), path),
                            Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                        }
                    }
                    "del" | "rm" => {
                        let path = args.get(3).copied().unwrap_or("");
                        match sb.filesystem.delete(path) {
                            Ok(()) => crate::println_color!(COLOR_GREEN, "Deleted {}", path),
                            Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                        }
                    }
                    _ => crate::println!("Unknown fs command: {}", fs_cmd),
                }
            } else {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
            }
        }

        "js" | "eval" => {
            // sandbox js <id> <code...>
            if args.len() < 3 {
                crate::println!("Usage: sandbox js <id> <code>");
                crate::println!("  Example: sandbox js 1 console.log('hello')");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let code = args[2..].join(" ");
            let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            let exists = mgr.get(crate::sandbox::SandboxId(id_num)).is_some();
            let js_ok = mgr.get(crate::sandbox::SandboxId(id_num))
                .map(|s| s.policy.js_allowed()).unwrap_or(false);
            drop(mgr);

            if !exists {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
                return;
            }
            if !js_ok {
                crate::println_color!(COLOR_RED, "JavaScript is blocked by sandbox policy (use 'moderate' or 'permissive' preset)");
                return;
            }

            let mut jss = crate::sandbox::js_sandbox::JsSandbox::new(
                crate::sandbox::SandboxId(id_num),
                crate::sandbox::js_sandbox::JsSandboxConfig::default(),
            );
            let result = jss.execute(&code);
            if result.completed {
                crate::println_color!(COLOR_GREEN, "= {}", result.return_value);
            } else {
                crate::println_color!(COLOR_RED, "Error: {}", result.error.as_deref().unwrap_or("unknown"));
            }
            for line in &result.output {
                crate::println!("  > {}", line);
            }
            crate::println_color!(COLOR_WHITE, "  ({}ms)", result.elapsed_ms);
        }

        "audit" | "log" => {
            if args.len() < 2 {
                // Show global audit log
                let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
                let log = mgr.audit_log();
                if log.is_empty() {
                    crate::println!("No audit entries.");
                } else {
                    crate::println_color!(COLOR_CYAN, "Audit log ({} entries):", log.len());
                    for entry in log.iter().rev().take(20) {
                        crate::println!("  [{}ms] #{} {:?}: {}",
                            entry.timestamp_ms, entry.sandbox_id.0,
                            entry.action, entry.detail);
                    }
                }
            } else {
                let id_num: u64 = args[1].parse().unwrap_or(0);
                let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
                let entries = mgr.audit_for(crate::sandbox::SandboxId(id_num));
                if entries.is_empty() {
                    crate::println!("No audit entries for sandbox #{}", id_num);
                } else {
                    crate::println_color!(COLOR_CYAN, "Audit for sandbox #{} ({} entries):", id_num, entries.len());
                    for entry in entries.iter().rev().take(20) {
                        crate::println!("  [{}ms] {:?}: {}",
                            entry.timestamp_ms, entry.action, entry.detail);
                    }
                }
            }
        }

        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Web Sandbox -- Secure isolated web execution");
            crate::println!();
            crate::println!("Usage: sandbox <command> [args...]");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Navigation:");
            crate::println!("    open <url> [preset]     Open URL in new sandbox");
            crate::println!("                             Presets: strict, moderate (default), permissive");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Sandbox Management:");
            crate::println!("    list                    List active sandboxes");
            crate::println!("    status <id>             Show sandbox details & stats");
            crate::println!("    kill <id|all>           Destroy sandbox(es)");
            crate::println!("    audit [id]              View audit log");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Security Policy:");
            crate::println!("    allow <id> <domain>     Add domain to allowlist");
            crate::println!("    deny <id> <domain>      Add domain to denylist");
            crate::println!("    policy <id>             Show policy config");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Sandboxed Filesystem:");
            crate::println!("    fs <id> tree             Show filesystem tree");
            crate::println!("    fs <id> ls [dir]         List directory");
            crate::println!("    fs <id> read <path>      Read file");
            crate::println!("    fs <id> write <path> <d> Write data to file");
            crate::println!("    fs <id> del <path>       Delete file");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  JavaScript (sandboxed):");
            crate::println!("    js <id> <code>           Execute JS in sandbox");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "  Security features:");
            crate::println!("    - Capability-gated network (kernel proxy)");
            crate::println!("    - Domain allow/deny lists + SSRF protection");
            crate::println!("    - Rate limiting + response size limits");
            crate::println!("    - JS static analysis (blocks eval, prototype pollution)");
            crate::println!("    - Jailed filesystem with quotas");
            crate::println!("    - Full audit trail");
        }
    }
}

/// Render HTML document as text to console
fn render_document_text(doc: &crate::browser::HtmlDocument, _depth: usize) {
    for node in &doc.nodes {
        render_node_text(node, 0);
    }
}

/// Render a single HTML node as text
fn render_node_text(node: &crate::browser::HtmlNode, depth: usize) {
    match node {
        crate::browser::HtmlNode::Text(text) => {
            let text = text.trim();
            if !text.is_empty() {
                crate::println!("{}", text);
            }
        }
        crate::browser::HtmlNode::Element(el) => {
            let tag = el.tag.as_str();
            
            // Skip invisible elements
            if matches!(tag, "head" | "script" | "style" | "meta" | "link" | "title" | "noscript") {
                return;
            }
            
            // Add formatting based on tag
            match tag {
                "h1" => {
                    crate::println!("");
                    crate::println_color!(COLOR_CYAN, "=== {} ===", get_element_text(el));
                    return;
                }
                "h2" => {
                    crate::println!("");
                    crate::println_color!(COLOR_CYAN, "== {} ==", get_element_text(el));
                    return;
                }
                "h3" | "h4" | "h5" | "h6" => {
                    crate::println!("");
                    crate::println_color!(COLOR_CYAN, "= {} =", get_element_text(el));
                    return;
                }
                "p" => {
                    crate::println!("");
                }
                "br" => {
                    crate::println!("");
                }
                "hr" => {
                    crate::println!("----------------------------------------");
                }
                "a" => {
                    if let Some(href) = el.attr("href") {
                        let text = get_element_text(el);
                        if !text.is_empty() {
                            crate::println_color!(COLOR_BLUE, "[{}] ({})", text, href);
                        }
                    }
                    return;
                }
                "li" => {
                    let indent = "  ".repeat(depth);
                    crate::print!("{}* ", indent);
                }
                "pre" | "code" => {
                    crate::println_color!(COLOR_MAGENTA, "{}", get_element_text(el));
                    return;
                }
                "img" => {
                    if let Some(alt) = el.attr("alt") {
                        crate::println!("[Image: {}]", alt);
                    } else {
                        crate::println!("[Image]");
                    }
                    return;
                }
                _ => {}
            }
            
            // Render children
            for child in &el.children {
                render_node_text(child, depth + 1);
            }
            
            // Newline after block elements
            if matches!(tag, "p" | "div" | "section" | "article" | "ul" | "ol" | "table" | "tr") {
                crate::println!("");
            }
        }
    }
}

/// Extract text content from an element
fn get_element_text(el: &crate::browser::HtmlElement) -> String {
    use alloc::string::ToString;
    let mut result = String::new();
    collect_text(&el.children, &mut result);
    result.trim().to_string()
}

/// Collect text from nodes recursively
fn collect_text(nodes: &[crate::browser::HtmlNode], result: &mut String) {
    use alloc::string::ToString;
    
    for node in nodes {
        match node {
            crate::browser::HtmlNode::Text(t) => {
                result.push_str(t);
                result.push(' ');
            }
            crate::browser::HtmlNode::Element(el) => {
                collect_text(&el.children, result);
            }
        }
    }
}

// Helper function to read file content as String
pub(super) fn read_file_content(path: &str) -> Option<String> {
    match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => {
            let mut buf = [0u8; 4096];
            let n = crate::vfs::read(fd, &mut buf).unwrap_or(0);
            crate::vfs::close(fd).ok();
            Some(String::from(core::str::from_utf8(&buf[..n]).unwrap_or("")))
        }
        Err(_) => None,
    }
}

// Helper function to read file content as bytes
pub(super) fn read_file_bytes(path: &str) -> Option<Vec<u8>> {
    // Try ramfs first (for /tmp files)
    if let Ok(data) = crate::ramfs::with_fs(|fs| {
        fs.read_file(path).map(|slice| slice.to_vec())
    }) {
        return Some(data);
    }
    
    // Then try VFS
    match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match crate::vfs::read(fd, &mut chunk) {
                    Ok(0) => break,
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    Err(_) => break,
                }
            }
            crate::vfs::close(fd).ok();
            Some(buf)
        }
        Err(_) => None,
    }
}