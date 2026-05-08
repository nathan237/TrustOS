




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AX_, D_, A_, C_, R_, CF_, DM_, K_};

pub(super) fn kme(args: &[&str]) {
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
    
    
    let mut browser = crate::browser::Browser::new(800, 600);
    
    match browser.navigate(&url) {
        Ok(()) => {
            crate::n!(B_, "Page loaded successfully!");
            
            
            if let Some(ref doc) = browser.document {
                if !doc.title.is_empty() {
                    crate::println!("Title: {}", doc.title);
                }
                
                crate::println!("");
                
                
                gra(doc, 0);
            }
        }
        Err(e) => {
            crate::n!(A_, "Failed to load page: {}", e);
        }
    }
}





pub(super) fn kmk(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");
    match je {
        "status" | "info" => {
            if args.len() >= 2 {
                
                let id: u32 = args[1].parse().unwrap_or(0);
                match crate::sandbox::container::kxi(id) {
                    Some(j) => crate::println!("{}", j),
                    None => crate::n!(A_, "Container #{} not found", id),
                }
            } else {
                
                crate::println!("{}", crate::sandbox::container::lbk());
            }
        }

        "list" | "ls" => {
            let containers = crate::sandbox::container::mzb();
            if containers.is_empty() {
                crate::println!("No containers.");
            } else {
                crate::n!(C_, "Web Containers:");
                crate::println!("  {:>4}  {:>10}  {:>3}  {}", "ID", "Health", "Def", "Name");
                crate::println!("  {}", "-".repeat(45));
                for (id, name, health, is_default) in &containers {
                    let lcx = if *is_default { " * " } else { "   " };
                    crate::println!("  {:>4}  {:>10?}  {}  {}", id, health, lcx, name);
                }
            }
        }

        "create" => {
            
            let gnx = args.get(1).copied().unwrap_or("default");
            let config = match gnx {
                "secure" => crate::sandbox::container::ContainerConfig::secure(),
                "dev" => crate::sandbox::container::ContainerConfig::s(),
                _ => {
                    let mut cfg = crate::sandbox::container::ContainerConfig::default();
                    
                    if gnx != "default" {
                        cfg.name = String::from(gnx);
                    }
                    cfg
                }
            };
            let mut agj = crate::sandbox::container::CN_.lock();
            let id = agj.create_container(config);
            match agj.start_container(id) {
                Ok(()) => crate::n!(B_, "Container #{} created and started", id),
                Err(e) => crate::n!(A_, "Created #{} but failed to start: {:?}", id, e),
            }
        }

        "go" | "navigate" | "open" => {
            
            if args.len() < 2 {
                crate::println!("Usage: container go <url> [container_id]");
                return;
            }
            let url = args[1];
            let foh: Option<u32> = args.get(2).and_then(|j| j.parse().ok());

            crate::n!(C_, "[Container] Fetching {}...", url);
            let mut agj = crate::sandbox::container::CN_.lock();
            match agj.navigate(foh, url) {
                Ok(eo) => {
                    crate::n!(B_, "  Status: {} | {} | {} bytes",
                        eo.status_code, eo.content_type, eo.body.len());

                    if eo.is_html() {
                        let epr = eo.body_string();
                        let doc = crate::browser::html_parser::boe(&epr);
                        if !doc.title.is_empty() {
                            crate::println!("  Title: {}", doc.title);
                        }
                        crate::println!();
                        gra(&doc, 0);
                    } else {
                        let text = eo.body_string();
                        let dww: String = text.chars().take(2000).collect();
                        crate::println!("{}", dww);
                        if text.len() > 2000 {
                            crate::println!("  ... ({} bytes total)", text.len());
                        }
                    }
                }
                Err(e) => {
                    crate::n!(A_, "  Error: {:?}", e);
                }
            }
        }

        "stop" => {
            if args.len() < 2 {
                crate::println!("Usage: container stop <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let mut agj = crate::sandbox::container::CN_.lock();
            match agj.stop_container(id) {
                Ok(()) => crate::n!(B_, "Container #{} stopped", id),
                Err(e) => crate::n!(A_, "Error: {:?}", e),
            }
        }

        "start" => {
            if args.len() < 2 {
                crate::println!("Usage: container start <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let mut agj = crate::sandbox::container::CN_.lock();
            match agj.start_container(id) {
                Ok(()) => crate::n!(B_, "Container #{} started", id),
                Err(e) => crate::n!(A_, "Error: {:?}", e),
            }
        }

        "restart" => {
            if args.len() < 2 {
                crate::println!("Usage: container restart <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let mut agj = crate::sandbox::container::CN_.lock();
            match agj.restart_container(id) {
                Ok(()) => crate::n!(B_, "Container #{} restarted", id),
                Err(e) => crate::n!(A_, "Error: {:?}", e),
            }
        }

        "destroy" | "rm" => {
            if args.len() < 2 {
                crate::println!("Usage: container destroy <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let mut agj = crate::sandbox::container::CN_.lock();
            match agj.destroy_container(id) {
                Ok(()) => crate::n!(B_, "Container #{} destroyed", id),
                Err(e) => crate::n!(A_, "Error: {:?}", e),
            }
        }

        "allow" => {
            if args.len() < 3 {
                crate::println!("Usage: container allow <id> <domain>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            
            let sandbox_id = {
                let mut agj = crate::sandbox::container::CN_.lock();
                if let Some(container) = agj.get_mut(id) {
                    container.config.allowed_domains.push(String::from(domain));
                    container.sandbox_id
                } else {
                    crate::n!(A_, "Container #{} not found", id);
                    return;
                }
            };
            
            if let Some(sid) = sandbox_id {
                let mut ng = crate::sandbox::BE_.lock();
                if let Some(cv) = ng.get_mut(sid) {
                    cv.policy.allow_domain(domain);
                }
            }
            crate::n!(B_, "Domain '{}' allowed in container #{}", domain, id);
        }

        "deny" | "block" => {
            if args.len() < 3 {
                crate::println!("Usage: container deny <id> <domain>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            let sandbox_id = {
                let mut agj = crate::sandbox::container::CN_.lock();
                if let Some(container) = agj.get_mut(id) {
                    container.config.blocked_domains.push(String::from(domain));
                    container.sandbox_id
                } else {
                    crate::n!(A_, "Container #{} not found", id);
                    return;
                }
            };
            if let Some(sid) = sandbox_id {
                let mut ng = crate::sandbox::BE_.lock();
                if let Some(cv) = ng.get_mut(sid) {
                    cv.policy.deny_domain(domain);
                }
            }
            crate::n!(B_, "Domain '{}' blocked in container #{}", domain, id);
        }

        "watchdog" => {
            crate::sandbox::container::watchdog_tick();
            crate::n!(B_, "Watchdog tick executed.");
        }

        "history" => {
            if args.len() < 2 {
                crate::println!("Usage: container history <id>");
                return;
            }
            let id: u32 = args[1].parse().unwrap_or(0);
            let agj = crate::sandbox::container::CN_.lock();
            if let Some(container) = agj.get(id) {
                if container.history.is_empty() {
                    crate::println!("No navigation history.");
                } else {
                    crate::n!(C_, "Navigation history for container #{}:", id);
                    for (i, url) in container.history.iter().enumerate() {
                        crate::println!("  {}. {}", i + 1, url);
                    }
                }
            } else {
                crate::n!(A_, "Container #{} not found", id);
            }
        }

        _ => {
            crate::n!(C_, "TrustOS Web Container -- persistent isolated web service daemon");
            crate::println!();
            crate::println!("Usage: container <command> [args...]");
            crate::println!();
            crate::n!(R_, "  Daemon:");
            crate::println!("    status [id]           Show daemon or container status");
            crate::println!("    list                  List all containers");
            crate::println!("    watchdog              Run watchdog health check");
            crate::println!();
            crate::n!(R_, "  Lifecycle:");
            crate::println!("    create [secure|dev]   Create a new container");
            crate::println!("    start <id>            Start a container");
            crate::println!("    stop <id>             Stop a container");
            crate::println!("    restart <id>          Restart a container");
            crate::println!("    destroy <id>          Remove a container");
            crate::println!();
            crate::n!(R_, "  Navigation:");
            crate::println!("    go <url> [id]         Navigate through container");
            crate::println!("    history <id>          Show navigation history");
            crate::println!();
            crate::n!(R_, "  Security:");
            crate::println!("    allow <id> <domain>   Allow domain access");
            crate::println!("    deny <id> <domain>    Block domain access");
            crate::println!();
            crate::n!(D_, "  The default container is auto-started at boot.");
            crate::n!(D_, "  All web traffic is proxied through the kernel.");
            crate::n!(D_, "  Isolation: software (capabilities) | hardware (EPT - future)");
        }
    }
}





pub(super) fn krf(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");
    match je {
        "open" | "navigate" | "go" => {
            
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

            crate::n!(C_, "[Sandbox] Creating sandbox ({:?})...", preset);
            let id = crate::sandbox::create(preset, Some(url));
            crate::println!("  Sandbox #{} created", id.0);

            crate::n!(C_, "[Sandbox] Navigating to {}...", url);
            match crate::sandbox::navigate(id, url) {
                Ok(eo) => {
                    crate::n!(B_, "  Status: {} | Content-Type: {} | {} bytes",
                        eo.status_code, eo.content_type, eo.body.len());

                    if eo.is_html() {
                        
                        let epr = eo.body_string();
                        let doc = crate::browser::html_parser::boe(&epr);
                        if !doc.title.is_empty() {
                            crate::println!("  Title: {}", doc.title);
                        }
                        crate::println!();
                        gra(&doc, 0);

                        
                        let ng = crate::sandbox::BE_.lock();
                        let policy = ng.get(id).map(|j| j.policy.js_allowed()).unwrap_or(false);
                        drop(ng);
                        if policy {
                            let mut gej = crate::sandbox::js_sandbox::JsSandbox::new(
                                id, crate::sandbox::js_sandbox::JsSandboxConfig::default()
                            );
                            let results = gej.execute_inline_scripts(&epr);
                            if !results.is_empty() {
                                crate::n!(D_, "\n  [JS] {} script(s) processed", results.len());
                                for (i, r) in results.iter().enumerate() {
                                    if r.completed {
                                        crate::n!(B_, "    Script {}: OK ({}ms)", i+1, r.elapsed_ms);
                                    } else {
                                        crate::n!(A_, "    Script {}: {}", i+1,
                                            r.error.as_deref().unwrap_or("failed"));
                                    }
                                    for line in &r.output {
                                        crate::println!("      > {}", line);
                                    }
                                }
                            }
                        } else {
                            crate::n!(D_, "  [JS blocked by policy]");
                        }
                    } else {
                        
                        let text = eo.body_string();
                        let dww: String = text.chars().take(2000).collect();
                        crate::println!("{}", dww);
                        if text.len() > 2000 {
                            crate::println!("  ... ({} bytes total)", text.len());
                        }
                    }
                }
                Err(e) => {
                    crate::n!(A_, "  Error: {:?}", e);
                }
            }
        }

        "list" | "ls" => {
            let sandboxes = crate::sandbox::list();
            if sandboxes.is_empty() {
                crate::println!("No active sandboxes.");
            } else {
                crate::n!(C_, "Active sandboxes:");
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
            let xd: u64 = args[1].parse().unwrap_or(0);
            let id = crate::sandbox::Ag(xd);
            match crate::sandbox::status_string(id) {
                Some(j) => crate::println!("{}", j),
                None => crate::n!(A_, "Sandbox #{} not found", xd),
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
                crate::n!(B_, "All sandboxes destroyed.");
            } else {
                let xd: u64 = args[1].parse().unwrap_or(0);
                let id = crate::sandbox::Ag(xd);
                match crate::sandbox::destroy(id) {
                    Ok(()) => crate::n!(B_, "Sandbox #{} destroyed.", xd),
                    Err(e) => crate::n!(A_, "Error: {:?}", e),
                }
            }
        }

        "allow" => {
            
            if args.len() < 3 {
                crate::println!("Usage: sandbox allow <id> <domain>");
                crate::println!("  Example: sandbox allow 1 example.com");
                crate::println!("  Example: sandbox allow 1 *.github.com");
                return;
            }
            let xd: u64 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            let mut ng = crate::sandbox::BE_.lock();
            if let Some(cv) = ng.get_mut(crate::sandbox::Ag(xd)) {
                cv.policy.allow_domain(domain);
                crate::n!(B_, "Domain '{}' allowed in sandbox #{}", domain, xd);
            } else {
                crate::n!(A_, "Sandbox #{} not found", xd);
            }
        }

        "deny" | "block" => {
            
            if args.len() < 3 {
                crate::println!("Usage: sandbox deny <id> <domain>");
                return;
            }
            let xd: u64 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            let mut ng = crate::sandbox::BE_.lock();
            if let Some(cv) = ng.get_mut(crate::sandbox::Ag(xd)) {
                cv.policy.deny_domain(domain);
                crate::n!(B_, "Domain '{}' blocked in sandbox #{}", domain, xd);
            } else {
                crate::n!(A_, "Sandbox #{} not found", xd);
            }
        }

        "policy" => {
            if args.len() < 2 {
                crate::println!("Usage: sandbox policy <id>");
                return;
            }
            let xd: u64 = args[1].parse().unwrap_or(0);
            let ng = crate::sandbox::BE_.lock();
            if let Some(cv) = ng.get(crate::sandbox::Ag(xd)) {
                crate::println!("{}", cv.policy.summary());
            } else {
                crate::n!(A_, "Sandbox #{} not found", xd);
            }
        }

        "fs" => {
            
            if args.len() < 3 {
                crate::println!("Usage: sandbox fs <id> <ls|tree|read|write|del> [path] [data]");
                return;
            }
            let xd: u64 = args[1].parse().unwrap_or(0);
            let iag = args[2];
            let mut ng = crate::sandbox::BE_.lock();
            if let Some(cv) = ng.get_mut(crate::sandbox::Ag(xd)) {
                match iag {
                    "tree" => {
                        crate::println!("{}", cv.filesystem.tree());
                    }
                    "ls" => {
                        let it = args.get(3).copied().unwrap_or("/");
                        match cv.filesystem.list(it) {
                            Ok(entries) => {
                                crate::n!(C_, "{}:", it);
                                for (path, wf, size) in &entries {
                                    let icon = if *wf == &crate::sandbox::fs::SandboxFileType::Directory { "[D]" } else { "[F]" };
                                    crate::println!("  {} {} ({} bytes)", icon, path, size);
                                }
                            }
                            Err(e) => crate::n!(A_, "Error: {:?}", e),
                        }
                    }
                    "read" => {
                        let path = args.get(3).copied().unwrap_or("/");
                        match cv.filesystem.read(path) {
                            Ok(data) => {
                                let text = core::str::from_utf8(data).unwrap_or("<binary>");
                                crate::println!("{}", text);
                            }
                            Err(e) => crate::n!(A_, "Error: {:?}", e),
                        }
                    }
                    "write" => {
                        if args.len() < 5 {
                            crate::println!("Usage: sandbox fs <id> write <path> <data>");
                            return;
                        }
                        let path = args[3];
                        let data = args[4..].join(" ");
                        match cv.filesystem.write(path, data.as_bytes(), "shell") {
                            Ok(()) => crate::n!(B_, "Written {} bytes to {}", data.len(), path),
                            Err(e) => crate::n!(A_, "Error: {:?}", e),
                        }
                    }
                    "del" | "rm" => {
                        let path = args.get(3).copied().unwrap_or("");
                        match cv.filesystem.delete(path) {
                            Ok(()) => crate::n!(B_, "Deleted {}", path),
                            Err(e) => crate::n!(A_, "Error: {:?}", e),
                        }
                    }
                    _ => crate::println!("Unknown fs command: {}", iag),
                }
            } else {
                crate::n!(A_, "Sandbox #{} not found", xd);
            }
        }

        "js" | "eval" => {
            
            if args.len() < 3 {
                crate::println!("Usage: sandbox js <id> <code>");
                crate::println!("  Example: sandbox js 1 console.log('hello')");
                return;
            }
            let xd: u64 = args[1].parse().unwrap_or(0);
            let code = args[2..].join(" ");
            let ng = crate::sandbox::BE_.lock();
            let exists = ng.get(crate::sandbox::Ag(xd)).is_some();
            let mvf = ng.get(crate::sandbox::Ag(xd))
                .map(|j| j.policy.js_allowed()).unwrap_or(false);
            drop(ng);

            if !exists {
                crate::n!(A_, "Sandbox #{} not found", xd);
                return;
            }
            if !mvf {
                crate::n!(A_, "JavaScript is blocked by sandbox policy (use 'moderate' or 'permissive' preset)");
                return;
            }

            let mut gej = crate::sandbox::js_sandbox::JsSandbox::new(
                crate::sandbox::Ag(xd),
                crate::sandbox::js_sandbox::JsSandboxConfig::default(),
            );
            let result = gej.execute(&code);
            if result.completed {
                crate::n!(B_, "= {}", result.return_value);
            } else {
                crate::n!(A_, "Error: {}", result.error.as_deref().unwrap_or("unknown"));
            }
            for line in &result.output {
                crate::println!("  > {}", line);
            }
            crate::n!(R_, "  ({}ms)", result.elapsed_ms);
        }

        "audit" | "log" => {
            if args.len() < 2 {
                
                let ng = crate::sandbox::BE_.lock();
                let log = ng.audit_log();
                if log.is_empty() {
                    crate::println!("No audit entries.");
                } else {
                    crate::n!(C_, "Audit log ({} entries):", log.len());
                    for entry in log.iter().rev().take(20) {
                        crate::println!("  [{}ms] #{} {:?}: {}",
                            entry.timestamp_ms, entry.sandbox_id.0,
                            entry.action, entry.detail);
                    }
                }
            } else {
                let xd: u64 = args[1].parse().unwrap_or(0);
                let ng = crate::sandbox::BE_.lock();
                let entries = ng.audit_for(crate::sandbox::Ag(xd));
                if entries.is_empty() {
                    crate::println!("No audit entries for sandbox #{}", xd);
                } else {
                    crate::n!(C_, "Audit for sandbox #{} ({} entries):", xd, entries.len());
                    for entry in entries.iter().rev().take(20) {
                        crate::println!("  [{}ms] {:?}: {}",
                            entry.timestamp_ms, entry.action, entry.detail);
                    }
                }
            }
        }

        _ => {
            crate::n!(C_, "TrustOS Web Sandbox -- Secure isolated web execution");
            crate::println!();
            crate::println!("Usage: sandbox <command> [args...]");
            crate::println!();
            crate::n!(R_, "  Navigation:");
            crate::println!("    open <url> [preset]     Open URL in new sandbox");
            crate::println!("                             Presets: strict, moderate (default), permissive");
            crate::println!();
            crate::n!(R_, "  Sandbox Management:");
            crate::println!("    list                    List active sandboxes");
            crate::println!("    status <id>             Show sandbox details & stats");
            crate::println!("    kill <id|all>           Destroy sandbox(es)");
            crate::println!("    audit [id]              View audit log");
            crate::println!();
            crate::n!(R_, "  Security Policy:");
            crate::println!("    allow <id> <domain>     Add domain to allowlist");
            crate::println!("    deny <id> <domain>      Add domain to denylist");
            crate::println!("    policy <id>             Show policy config");
            crate::println!();
            crate::n!(R_, "  Sandboxed Filesystem:");
            crate::println!("    fs <id> tree             Show filesystem tree");
            crate::println!("    fs <id> ls [dir]         List directory");
            crate::println!("    fs <id> read <path>      Read file");
            crate::println!("    fs <id> write <path> <d> Write data to file");
            crate::println!("    fs <id> del <path>       Delete file");
            crate::println!();
            crate::n!(R_, "  JavaScript (sandboxed):");
            crate::println!("    js <id> <code>           Execute JS in sandbox");
            crate::println!();
            crate::n!(D_, "  Security features:");
            crate::println!("    - Capability-gated network (kernel proxy)");
            crate::println!("    - Domain allow/deny lists + SSRF protection");
            crate::println!("    - Rate limiting + response size limits");
            crate::println!("    - JS static analysis (blocks eval, prototype pollution)");
            crate::println!("    - Jailed filesystem with quotas");
            crate::println!("    - Full audit trail");
        }
    }
}


fn gra(doc: &crate::browser::Ia, _depth: usize) {
    for uf in &doc.nodes {
        izu(uf, 0);
    }
}


fn izu(uf: &crate::browser::HtmlNode, depth: usize) {
    match uf {
        crate::browser::HtmlNode::Text(text) => {
            let text = text.trim();
            if !text.is_empty() {
                crate::println!("{}", text);
            }
        }
        crate::browser::HtmlNode::Element(el) => {
            let tag = el.tag.as_str();
            
            
            if matches!(tag, "head" | "script" | "style" | "meta" | "link" | "title" | "noscript") {
                return;
            }
            
            
            match tag {
                "h1" => {
                    crate::println!("");
                    crate::n!(C_, "=== {} ===", dqo(el));
                    return;
                }
                "h2" => {
                    crate::println!("");
                    crate::n!(C_, "== {} ==", dqo(el));
                    return;
                }
                "h3" | "h4" | "h5" | "h6" => {
                    crate::println!("");
                    crate::n!(C_, "= {} =", dqo(el));
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
                        let text = dqo(el);
                        if !text.is_empty() {
                            crate::n!(CF_, "[{}] ({})", text, href);
                        }
                    }
                    return;
                }
                "li" => {
                    let axq = "  ".repeat(depth);
                    crate::print!("{}* ", axq);
                }
                "pre" | "code" => {
                    crate::n!(DM_, "{}", dqo(el));
                    return;
                }
                "img" => {
                    if let Some(adf) = el.attr("alt") {
                        crate::println!("[Image: {}]", adf);
                    } else {
                        crate::println!("[Image]");
                    }
                    return;
                }
                _ => {}
            }
            
            
            for pd in &el.children {
                izu(pd, depth + 1);
            }
            
            
            if matches!(tag, "p" | "div" | "section" | "article" | "ul" | "ol" | "table" | "tr") {
                crate::println!("");
            }
        }
    }
}


fn dqo(el: &crate::browser::HtmlElement) -> String {
    use alloc::string::ToString;
    let mut result = String::new();
    hmx(&el.children, &mut result);
    result.trim().to_string()
}


fn hmx(nodes: &[crate::browser::HtmlNode], result: &mut String) {
    use alloc::string::ToString;
    
    for uf in nodes {
        match uf {
            crate::browser::HtmlNode::Text(t) => {
                result.push_str(t);
                result.push(' ');
            }
            crate::browser::HtmlNode::Element(el) => {
                hmx(&el.children, result);
            }
        }
    }
}


pub(super) fn cpa(path: &str) -> Option<String> {
    
    if let Ok(data) = crate::ramfs::bh(|fs| {
        fs.read_file(path).map(|slice| {
            String::from(core::str::from_utf8(slice).unwrap_or(""))
        })
    }) {
        return Some(data);
    }

    
    match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => {
            let mut buf = [0u8; 4096];
            let ae = crate::vfs::read(fd, &mut buf).unwrap_or(0);
            crate::vfs::close(fd).ok();
            Some(String::from(core::str::from_utf8(&buf[..ae]).unwrap_or("")))
        }
        Err(_) => None,
    }
}


pub(super) fn exu(path: &str) -> Option<Vec<u8>> {
    
    if let Ok(data) = crate::ramfs::bh(|fs| {
        fs.read_file(path).map(|slice| slice.to_vec())
    }) {
        return Some(data);
    }
    
    
    match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => {
            let mut buf = Vec::new();
            let mut df = [0u8; 4096];
            loop {
                match crate::vfs::read(fd, &mut df) {
                    Ok(0) => break,
                    Ok(ae) => buf.extend_from_slice(&df[..ae]),
                    Err(_) => break,
                }
            }
            crate::vfs::close(fd).ok();
            Some(buf)
        }
        Err(_) => None,
    }
}