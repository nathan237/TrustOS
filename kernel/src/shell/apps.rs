//! Application Commands  Graphical terminal, Wayland, TrustLang, Film, Transpile, Video, Lab
//!
//! Larger application-level commands: graphical terminal (gterm), Wayland compositor,
//! TrustLang compiler/runtime, animated showcases and film, binary transpiler,
//! video codec, TrustView binary analyzer, and TrustLab introspection.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA, COLOR_GRAY};

// -------------------------------------------------------------------------------
// GRAPHICAL TERMINAL COMMAND
// -------------------------------------------------------------------------------

/// Graphical Terminal - Matrix Edition
pub(super) fn cmd_gterm(args: &[&str]) {
    use crate::wayland::terminal;
    
    let subcmd = args.get(0).copied().unwrap_or("launch");
    
    match subcmd {
        "launch" | "start" | "run" => {
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println_color!(COLOR_CYAN, "|           TrustOS Graphical Terminal - Matrix Edition        |");
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println!();
            
            // Initialize Wayland compositor first
            let _ = crate::wayland::init();
            
            // Get display size
            let (screen_w, screen_h) = crate::framebuffer::get_dimensions();
            
            // Calculate terminal window size (80% of screen)
            let term_w = (screen_w * 80 / 100) & !7; // Align to 8 pixels
            let term_h = (screen_h * 80 / 100) & !15; // Align to 16 pixels
            
            crate::println!("Initializing terminal {}x{} pixels...", term_w, term_h);
            
            // Initialize the graphical terminal
            match terminal::init(term_w, term_h) {
                Ok(()) => {
                    crate::println_color!(COLOR_GREEN, "[OK] Graphics terminal initialized");
                }
                Err(e) => {
                    crate::println_color!(COLOR_YELLOW, "[WARN] {}", e);
                }
            }
            
            // Create Wayland surface for the terminal
            crate::wayland::with_compositor(|compositor| {
                let surface_id = compositor.create_surface();
                
                // Get rendered terminal buffer
                if let Some(buffer) = terminal::render() {
                    let (w, h) = terminal::get_size().unwrap_or((term_w, term_h));
                    
                    // Configure surface
                    if let Some(surface) = compositor.surfaces.get_mut(&surface_id) {
                        surface.attach(buffer, w, h);
                        surface.set_title("TrustOS Terminal");
                        let x = (screen_w - w) / 2;
                        let y = (screen_h - h) / 2;
                        surface.set_position(x as i32, y as i32);
                        surface.make_toplevel();
                        surface.commit();
                    }
                }
                
                crate::println_color!(COLOR_GREEN, "[OK] Terminal surface created (ID: {})", surface_id);
            });
            
            // Initial render
            crate::wayland::compose_frame();
            
            crate::println!();
            crate::println_color!(COLOR_GREEN, "Terminal launched!");
            crate::println!("Use 'gterm demo' for an interactive demo.");
            crate::println!("Use 'gterm fullscreen' for fullscreen mode.");
        },
        
        "demo" => {
            crate::println_color!(COLOR_CYAN, "Starting interactive graphical terminal demo...");
            crate::println!();
            
            // Initialize everything
            let _ = crate::wayland::init();
            
            let (screen_w, screen_h) = crate::framebuffer::get_dimensions();
            let term_w = (screen_w * 85 / 100) & !7;
            let term_h = (screen_h * 85 / 100) & !15;
            
            // Initialize terminal (ignore if already initialized)
            let _ = terminal::init(term_w, term_h);
            
            // Create surface
            let surface_id = crate::wayland::with_compositor(|compositor| {
                let id = compositor.create_surface();
                
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::get_size() {
                        if let Some(surface) = compositor.surfaces.get_mut(&id) {
                            surface.attach(buffer, w, h);
                            surface.set_title("TrustOS Terminal Demo");
                            surface.set_position(
                                ((screen_w - w) / 2) as i32,
                                ((screen_h - h) / 2) as i32
                            );
                            surface.make_toplevel();
                            surface.commit();
                        }
                    }
                }
                id
            }).unwrap_or(0);
            
            // Initial compose
            crate::wayland::compose_frame();
            
            // Write welcome message
            terminal::write("\x1b[2J\x1b[H"); // Clear and home
            terminal::write("\x1b[1;32m+----------------------------------------------------------+\r\n");
            terminal::write("|  \x1b[1;97mTrustOS\x1b[1;32m Graphical Terminal Demo                       |\r\n");
            terminal::write("|  Matrix Edition v1.0                                     |\r\n");
            terminal::write("+----------------------------------------------------------+\r\n");
            terminal::write("\x1b[0;32m\r\n");
            terminal::write("Type text and press Enter. Press ESC to exit.\r\n\r\n");
            terminal::write("\x1b[1;32m$ \x1b[0;32m");
            
            // Render after writing
            crate::wayland::with_compositor(|compositor| {
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::get_size() {
                        if let Some(surface) = compositor.surfaces.get_mut(&surface_id) {
                            surface.attach(buffer, w, h);
                            surface.commit();
                        }
                    }
                }
            });
            crate::wayland::compose_frame();
            
            // Interactive loop
            let mut input_buffer = alloc::string::String::new();
            loop {
                // Check for keyboard input
                if let Some(key) = crate::keyboard::read_char() {
                    let c = key as char;
                    match key {
                        0x1b => {
                            // ESC - exit
                            break;
                        }
                        0x0D | 0x0A => {
                            // Enter - process command
                            terminal::write("\r\n");
                            
                            if !input_buffer.is_empty() {
                                // Echo the command result
                                let response = alloc::format!("\x1b[0;36mYou typed: \x1b[1;97m{}\x1b[0;32m\r\n", input_buffer);
                                terminal::write(&response);
                                input_buffer.clear();
                            }
                            
                            terminal::write("\x1b[1;32m$ \x1b[0;32m");
                        }
                        0x08 | 0x7F => {
                            // Backspace
                            if !input_buffer.is_empty() {
                                input_buffer.pop();
                                terminal::write("\x08 \x08");
                            }
                        }
                        k if k >= 0x20 && k < 0x7F => {
                            // Printable character
                            input_buffer.push(c);
                            let s = alloc::format!("{}", c);
                            terminal::write(&s);
                        }
                        _ => {}
                    }
                    
                    // Re-render after each keystroke
                    crate::wayland::with_compositor(|compositor| {
                        if let Some(buffer) = terminal::render() {
                            if let Some((w, h)) = terminal::get_size() {
                                if let Some(surface) = compositor.surfaces.get_mut(&surface_id) {
                                    surface.attach(buffer, w, h);
                                    surface.commit();
                                }
                            }
                        }
                    });
                    crate::wayland::compose_frame();
                }
                
                // Small delay to prevent busy loop
                for _ in 0..1000 { core::hint::spin_loop(); }
            }
            
            // Cleanup
            crate::framebuffer::clear();
            crate::println_color!(COLOR_GREEN, "Demo ended.");
        },
        
        "fullscreen" | "fs" => {
            crate::println_color!(COLOR_CYAN, "Launching fullscreen terminal...");
            
            // Use full screen dimensions
            let (screen_w, screen_h) = crate::framebuffer::get_dimensions();
            
            let _ = crate::wayland::init();
            let _ = terminal::init(screen_w, screen_h);
            
            // Create fullscreen surface
            crate::wayland::with_compositor(|compositor| {
                let id = compositor.create_surface();
                
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::get_size() {
                        if let Some(surface) = compositor.surfaces.get_mut(&id) {
                            surface.attach(buffer, w, h);
                            surface.set_title("TrustOS Terminal");
                            surface.set_position(0, 0);
                            surface.make_toplevel();
                            surface.state.fullscreen = true;
                            surface.commit();
                        }
                    }
                }
            });
            
            crate::wayland::compose_frame();
            crate::println_color!(COLOR_GREEN, "[OK] Fullscreen terminal active");
        },
        
        "test" => {
            // Test ANSI escape codes
            crate::println_color!(COLOR_CYAN, "Testing graphical terminal ANSI support...");
            
            let _ = crate::wayland::init();
            let (w, h) = crate::framebuffer::get_dimensions();
            let _ = terminal::init(w * 70 / 100, h * 70 / 100);
            
            // Test various escape sequences
            terminal::write("\x1b[2J\x1b[H"); // Clear
            terminal::write("\x1b[1;32m=== ANSI Escape Code Test ===\x1b[0m\r\n\r\n");
            
            // Colors
            terminal::write("\x1b[31mRed \x1b[32mGreen \x1b[33mYellow \x1b[34mBlue \x1b[35mMagenta \x1b[36mCyan\x1b[0m\r\n");
            terminal::write("\x1b[91mBright Red \x1b[92mBright Green \x1b[93mBright Yellow\x1b[0m\r\n\r\n");
            
            // Attributes
            terminal::write("\x1b[1mBold\x1b[0m \x1b[2mDim\x1b[0m \x1b[4mUnderline\x1b[0m \x1b[7mReverse\x1b[0m\r\n\r\n");
            
            // Matrix rain effect preview
            terminal::write("\x1b[32m");
            for i in 0..5 {
                for _ in 0..60 {
                    let c = ((i * 7 + 33) % 94 + 33) as u8 as char;
                    let s = alloc::format!("{}", c);
                    terminal::write(&s);
                }
                terminal::write("\r\n");
            }
            terminal::write("\x1b[0m\r\n");
            
            terminal::write("\x1b[1;97mTest complete!\x1b[0m\r\n");
            
            // Render
            crate::wayland::with_compositor(|compositor| {
                let id = compositor.create_surface();
                if let Some(buffer) = terminal::render() {
                    if let Some((tw, th)) = terminal::get_size() {
                        if let Some(surface) = compositor.surfaces.get_mut(&id) {
                            surface.attach(buffer, tw, th);
                            surface.set_title("ANSI Test");
                            surface.set_position(
                                ((w - tw) / 2) as i32,
                                ((h - th) / 2) as i32
                            );
                            surface.make_toplevel();
                            surface.commit();
                        }
                    }
                }
            });
            crate::wayland::compose_frame();
            
            crate::println!();
            crate::println_color!(COLOR_GREEN, "Press any key to close...");
            loop {
                if crate::keyboard::read_char().is_some() {
                    break;
                }
            }
            crate::framebuffer::clear();
        },
        
        _ => {
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println_color!(COLOR_CYAN, "|       TrustOS Graphical Terminal - Matrix Edition           |");
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println!();
            crate::println!("A native graphical terminal emulator with VT100/ANSI support.");
            crate::println!("Inspired by Smithay, built from scratch for TrustOS.");
            crate::println!();
            crate::println!("Usage: gterm <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  launch     - Open the graphical terminal window");
            crate::println!("  demo       - Interactive demo (type text, ESC to exit)");
            crate::println!("  fullscreen - Open fullscreen terminal");
            crate::println!("  test       - Test ANSI escape code rendering");
            crate::println!();
            crate::println!("Features:");
            crate::println!("  * Matrix-style green phosphor theme");
            crate::println!("  * VT100/ANSI escape code support");
            crate::println!("  * 256-color and 24-bit RGB colors");
            crate::println!("  * Scrollback buffer (1000 lines)");
            crate::println!("  * Phosphor glow effect");
            crate::println!("  * CRT scanline effect");
        }
    }
}

/// Wayland compositor command
pub(super) fn cmd_wayland(args: &[&str]) {
    let subcmd = args.get(0).copied().unwrap_or("help");
    
    match subcmd {
        "init" | "start" => {
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println_color!(COLOR_CYAN, "|            TrustOS Wayland Compositor                        |");
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println!();
            
            match crate::wayland::init() {
                Ok(()) => {
                    crate::println_color!(COLOR_GREEN, "[OK] Wayland compositor initialized");
                    
                    // Get screen info
                    let (width, height) = crate::framebuffer::get_dimensions();
                    crate::println!("     Display: {}x{}", width, height);
                    crate::println!();
                    crate::println!("Available globals:");
                    for global in crate::wayland::protocol::get_globals() {
                        crate::println!("  * {} v{}", global.interface, global.version);
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "[ERROR] {}", e);
                }
            }
        },
        
        "demo" => {
            crate::println_color!(COLOR_CYAN, "Starting Wayland demo...");
            
            // Initialize if not already done
            let _ = crate::wayland::init();
            
            // Create a test surface
            crate::wayland::with_compositor(|compositor| {
                // Create a surface
                let surface_id = compositor.create_surface();
                
                // Create some test content
                let width = 400u32;
                let height = 300u32;
                let mut buffer = alloc::vec![0xFF0A0F0C_u32; (width * height) as usize];
                
                // Draw a gradient
                for y in 0..height {
                    for x in 0..width {
                        let r = (x * 255 / width) as u8;
                        let g = ((y * 255 / height) as u8) / 2;
                        let b = 0x20_u8;
                        buffer[(y * width + x) as usize] = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
                    }
                }
                
                // Draw border
                for x in 0..width {
                    buffer[x as usize] = 0xFF00FF66;
                    buffer[((height - 1) * width + x) as usize] = 0xFF00FF66;
                }
                for y in 0..height {
                    buffer[(y * width) as usize] = 0xFF00FF66;
                    buffer[(y * width + width - 1) as usize] = 0xFF00FF66;
                }
                
                // Attach and commit
                if let Some(surface) = compositor.surfaces.get_mut(&surface_id) {
                    surface.attach(buffer, width, height);
                    surface.set_title("Wayland Demo");
                    surface.set_position(200, 150);
                    surface.make_toplevel();
                    surface.commit();
                }
                
                crate::println_color!(COLOR_GREEN, "[OK] Created surface {}", surface_id);
            });
            
            // Compose and render
            crate::wayland::compose_frame();
            crate::println_color!(COLOR_GREEN, "[OK] Frame composed to framebuffer");
            crate::println!();
            crate::println!("Press any key to close demo...");
            
            // Wait for key
            loop {
                if let Some(_) = crate::keyboard::read_char() {
                    break;
                }
            }
            
            // Clear screen
            crate::framebuffer::clear();
        },
        
        "status" => {
            crate::println_color!(COLOR_CYAN, "Wayland Compositor Status");
            crate::println_color!(COLOR_CYAN, "--------------------------");
            
            crate::wayland::with_compositor(|compositor| {
                let (w, h) = (compositor.width, compositor.height);
                crate::println!("Display: {}x{}", w, h);
                crate::println!("Surfaces: {}", compositor.surfaces.len());
                crate::println!("SHM Pools: {}", compositor.shm_pools.len());
                crate::println!("Frame: {}", compositor.frame_number);
                crate::println!("Pointer: ({}, {})", compositor.pointer_x, compositor.pointer_y);
                
                if !compositor.surfaces.is_empty() {
                    crate::println!();
                    crate::println!("Surfaces:");
                    for (&id, surface) in &compositor.surfaces {
                        let title = if surface.title.is_empty() { "<untitled>" } else { &surface.title };
                        crate::println!("  #{}: {} @ ({},{}) {}x{}", 
                            id, title, surface.x, surface.y, surface.width, surface.height);
                    }
                }
            }).unwrap_or_else(|| {
                crate::println_color!(COLOR_YELLOW, "Compositor not initialized");
                crate::println!("Run 'wayland init' first");
            });
        },
        
        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Wayland Compositor");
            crate::println_color!(COLOR_CYAN, "--------------------------");
            crate::println!();
            crate::println!("A native Wayland display server for TrustOS.");
            crate::println!();
            crate::println!("Usage: wayland <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  init    - Initialize the Wayland compositor");
            crate::println!("  demo    - Run a visual demo");
            crate::println!("  status  - Show compositor status");
            crate::println!();
            crate::println!("Protocol support:");
            crate::println!("  * wl_compositor v5 - Surface creation");
            crate::println!("  * wl_shm v1        - Shared memory buffers");
            crate::println!("  * wl_seat v8       - Input devices");
            crate::println!("  * xdg_wm_base v5   - Window management");
        }
    }
}

/// TrustLang command: compile and run TrustLang programs
pub(super) fn cmd_trustlang(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "run" => {
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: trustlang run <file.tl>"); return; }
            };
            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/{}", filename)
            };
            let source = match crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
                Ok(data) => match alloc::string::String::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", filename); return; }
            };
            crate::println!("\x1b[36m[TrustLang]\x1b[0m Compiling {}...", filename);
            match crate::trustlang::run(&source) {
                Ok(output) => {
                    if !output.is_empty() { crate::print!("{}", output); }
                    crate::println!("\x1b[32m[TrustLang]\x1b[0m Program finished successfully.");
                }
                Err(e) => crate::println!("\x1b[31m[TrustLang Error]\x1b[0m {}", e),
            }
        }
        "check" => {
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: trustlang check <file.tl>"); return; }
            };
            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/{}", filename)
            };
            let source = match crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
                Ok(data) => match alloc::string::String::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", filename); return; }
            };
            match crate::trustlang::check(&source) {
                Ok(()) => crate::println!("\x1b[32m?\x1b[0m {} -- no errors", filename),
                Err(e) => crate::println!("\x1b[31m?\x1b[0m {} -- {}", filename, e),
            }
        }
        "eval" => {
            // Inline eval: wrap in main()
            let code = args[1..].join(" ");
            let wrapped = alloc::format!("fn main() {{ {} }}", code);
            match crate::trustlang::run(&wrapped) {
                Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
                Err(e) => crate::println!("\x1b[31mError:\x1b[0m {}", e),
            }
        }
        "demo" => {
            // Create a demo TrustLang file
            let demo = r#"// TrustLang Demo -- Fibonacci
fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    println("=== TrustLang on TrustOS ===");
    println("Computing Fibonacci sequence:");
    for i in 0..15 {
        let result = fibonacci(i);
        print("  fib(");
        print(to_string(i));
        print(") = ");
        println(to_string(result));
    }
    println("Done!");
}
"#;
            crate::ramfs::with_fs(|fs| {
                let _ = fs.write_file("/demo.tl", demo.as_bytes());
            });
            crate::println!("Created /demo.tl -- run with: trustlang run demo.tl");
            // Also execute it
            match crate::trustlang::run(demo) {
                Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
                Err(e) => crate::println!("\x1b[31mError:\x1b[0m {}", e),
            }
        }
        "repl" => {
            crate::println!("\x1b[1;36mTrustLang REPL\x1b[0m v1.0 — type 'exit' or 'quit' to leave");
            crate::println!("  Expressions are auto-wrapped in fn main() {{ ... }}");
            crate::println!("  Available: print/println, math ops, if/else, for, while");
            crate::println!();
            loop {
                crate::print!("\x1b[36mtl>\x1b[0m ");
                let line = crate::shell::read_line();
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if trimmed == "exit" || trimmed == "quit" { break; }
                if trimmed == "help" {
                    crate::println!("  println(\"hello\")       — print with newline");
                    crate::println!("  let x = 42;            — declare variable");
                    crate::println!("  for i in 0..5 {{ ... }}  — for loop");
                    crate::println!("  fn foo(n: i64) {{ ... }} — define function + fn main()");
                    continue;
                }
                match crate::trustlang::eval_line(trimmed) {
                    Ok(output) => {
                        if !output.is_empty() { crate::print!("{}", output); }
                    }
                    Err(e) => crate::println!("\x1b[31mError:\x1b[0m {}", e),
                }
            }
        }
        _ => {
            crate::println!("\x1b[1;36mTrustLang\x1b[0m -- Integrated Programming Language");
            crate::println!("  Rust-inspired syntax, bytecode VM, zero dependencies\n");
            crate::println!("Commands:");
            crate::println!("  trustlang run <file.tl>    Compile & execute a file");
            crate::println!("  trustlang check <file.tl>  Syntax check only");
            crate::println!("  trustlang eval <code>      Evaluate inline code");
            crate::println!("  trustlang repl             Interactive REPL");
            crate::println!("  trustlang demo             Create & run demo program");
            crate::println!("\nExample:");
            crate::println!("  trustlang eval println(\"Hello TrustOS!\")");
            crate::println!("  trustlang eval \"let x = 42; println(to_string(x * 2))\"");
        }
    }
}

/// Compute syntax highlighting colors for one line of TrustLang code.
/// Returns a Vec of ARGB colors, one per character.
fn trustlang_syntax_colors(line: &str) -> alloc::vec::Vec<u32> {
    let chars: alloc::vec::Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut colors = alloc::vec![0xFFD4D4D4u32; len]; // default: white-gray
    if len == 0 { return colors; }

    // 1) Find comment start (char index, not byte)
    let comment_ci = {
        let mut ci = None;
        let bytes = line.as_bytes();
        for i in 0..bytes.len().saturating_sub(1) {
            if bytes[i] == b'/' && bytes[i + 1] == b'/' {
                ci = Some(line[..i].chars().count());
                break;
            }
        }
        ci
    };
    let effective_len = comment_ci.unwrap_or(len);

    // Color comments green
    if let Some(cp) = comment_ci {
        for i in cp..len {
            colors[i] = 0xFF6A9955;
        }
    }

    // 2) Strings -- track in_string, color everything inside "" as orange-brown
    let mut in_string = false;
    for i in 0..effective_len {
        if chars[i] == '"' {
            colors[i] = 0xFFCE9178;
            in_string = !in_string;
        } else if in_string {
            colors[i] = 0xFFCE9178;
        }
    }

    // 3) Keywords, function calls, variables, numbers, brackets (outside strings & comments)
    in_string = false;
    let mut i = 0usize;
    while i < effective_len {
        if chars[i] == '"' {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if in_string { i += 1; continue; }

        // Numbers
        if chars[i].is_ascii_digit() {
            colors[i] = 0xFFB5CEA8;
            i += 1;
            continue;
        }
        // Brackets
        if matches!(chars[i], '(' | ')' | '{' | '}' | '[' | ']') {
            colors[i] = 0xFFFFD700;
            i += 1;
            continue;
        }
        // Identifiers / keywords
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < effective_len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: alloc::string::String = chars[start..i].iter().collect();

            // Is this a function call? (followed by '(')
            let mut peek = i;
            while peek < effective_len && chars[peek] == ' ' { peek += 1; }
            let is_fn_call = peek < effective_len && chars[peek] == '(';

            // Is this a variable declaration? (preceded by "let" or "mut")
            let before: alloc::string::String = chars[..start].iter().collect();
            let trimmed = before.trim_end();
            let is_var_decl = trimmed.ends_with("let") || trimmed.ends_with("mut");

            if matches!(word.as_str(),
                "fn" | "let" | "mut" | "if" | "else" | "while" | "for" | "in" |
                "return" | "loop" | "break" | "continue" | "true" | "false" |
                "struct" | "enum" | "match" | "use" | "pub" | "const" | "static" |
                "impl" | "self" | "type")
            {
                for j in start..i { colors[j] = 0xFFFF7B72; } // red keywords
            } else if is_fn_call {
                for j in start..i { colors[j] = 0xFF79C0FF; } // blue function calls
            } else if is_var_decl {
                for j in start..i { colors[j] = 0xFF9CDCFE; } // cyan variable names
            }
            // else keep default white
            continue;
        }
        i += 1;
    }
    colors
}

/// TrustLang Showcase -- Animated walkthrough demonstrating the full pipeline
/// ~90 seconds of automated cinematic demo -- descriptions, code typing, compilation, execution
/// Uses frame-counting + delay_millis for reliable timing (no TSC target comparison)
pub(super) fn cmd_trustlang_showcase() {
    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    // --------------- HELPER CLOSURES ---------------

    let draw_big_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, c: char, color: u32, scale: usize| {
        let glyph = crate::framebuffer::font::get_glyph(c);
        for (row, &bits) in glyph.iter().enumerate() {
            for bit in 0..8u32 {
                if bits & (0x80 >> bit) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cx + bit as usize * scale + sx;
                            let py = cy + row * scale + sy;
                            if px < w && py < h {
                                buf[py * w + px] = color;
                            }
                        }
                    }
                }
            }
        }
    };

    let draw_text_at = |buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            draw_big_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
        let tw = text.len() * 8 * scale;
        let sx = if tw < w { (w - tw) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
        }
    };

    let blit_buf = |buf: &[u32], w: usize, h: usize| {
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
            let bb = bb_ptr as *mut u32;
            let bb_s = bb_stride as usize;
            for y in 0..h.min(bb_h as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(buf[y * w..].as_ptr(), bb.add(y * bb_s), w);
                }
            }
        }
        crate::framebuffer::swap_buffers();
    };

    let clear_buf = |buf: &mut [u32]| {
        for p in buf.iter_mut() { *p = 0xFF000000; }
    };

    // Matrix rain state
    let mut rain_cols: alloc::vec::Vec<u16> = (0..w / 8 + 1).map(|i| ((i * 37 + 13) % h) as u16).collect();
    let rain_speeds: alloc::vec::Vec<u8> = (0..w / 8 + 1).map(|i| (((i * 7 + 3) % 4) + 1) as u8).collect();

    let draw_rain = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
        for pixel in buf.iter_mut() {
            let g = ((*pixel >> 8) & 0xFF) as u32;
            if g > 0 { *pixel = 0xFF000000 | (g.saturating_sub(6) << 8); }
        }
        for ci in 0..cols.len() {
            let x = ci * 8;
            if x >= w { continue; }
            cols[ci] = cols[ci].wrapping_add(speeds[ci] as u16);
            if cols[ci] as usize >= h { cols[ci] = 0; }
            let y = cols[ci] as usize;
            let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
            let glyph = crate::framebuffer::font::get_glyph(c);
            for (row, &bits) in glyph.iter().enumerate() {
                let py = y + row;
                if py >= h { break; }
                for bit in 0..8u32 {
                    if bits & (0x80 >> bit) != 0 {
                        let px = x + bit as usize;
                        if px < w { buf[py * w + px] = 0xFF00FF44; }
                    }
                }
            }
        }
    };

    // -- FRAME DELAY: ~30ms per frame --
    let frame_ms: u64 = 30;

    // -- Fade out: gradually darken buffer over ~78 frames (~2.3s) --
    let do_fade = |buf: &mut [u32], w: usize, h: usize, blit: &dyn Fn(&[u32], usize, usize)| {
        for _ in 0..78 {
            for px in buf.iter_mut() {
                let r = ((*px >> 16) & 0xFF).saturating_sub(4);
                let g = ((*px >> 8) & 0xFF).saturating_sub(4);
                let b = (*px & 0xFF).saturating_sub(4);
                *px = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
        for p in buf.iter_mut() { *p = 0xFF000000; }
        blit(buf, w, h);
        // Black pause between scenes (~1.4s)
        crate::cpu::tsc::pit_delay_ms(1400);
    };

    // ---------------------------------------------------------------
    // DESCRIPTION SCREEN -- text types in on Matrix rain background
    // ms_per_char: how fast each character appears (higher = slower)
    // hold_frames: how many frames to HOLD after all text is typed
    // ---------------------------------------------------------------
    let show_description = |buf: &mut [u32], w: usize, h: usize,
                            rain_cols: &mut [u16], rain_speeds: &[u8],
                            lines: &[(&str, u32, usize)],
                            ms_per_char: u64,
                            hold_frames: u32| {
        let total_chars: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
        // frames_per_char: how many render frames per character typed
        let frames_per_char = (ms_per_char / frame_ms).max(1) as u32;
        let typing_frames = total_chars as u32 * frames_per_char;
        let total_frames = typing_frames + hold_frames;
        let mut frame = 0u32;

        while frame < total_frames {
            // ESC to quit, Space/Enter to skip
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }

            // Rain background
            draw_rain(buf, w, h, rain_cols, rain_speeds, frame);

            // How many chars to show based on frame count
            let chars_shown = (frame / frames_per_char) as usize;

            // Compute vertical centering
            let total_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 12).sum();
            let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
            let mut counted = 0usize;

            for &(text, color, scale) in lines {
                let tw = text.len() * 8 * scale;
                let sx = if tw < w { (w - tw) / 2 } else { 0 };
                for (i, c) in text.chars().enumerate() {
                    if counted + i >= chars_shown { break; }
                    draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
                }
                // Cursor blink during typing
                if chars_shown > counted && chars_shown < counted + text.len() {
                    let ci = chars_shown - counted;
                    let cx = sx + ci * 8 * scale;
                    if (frame / 8) % 2 == 0 {
                        for cy in y..y + 16 * scale {
                            if cy < h && cx + 2 < w {
                                buf[cy * w + cx] = 0xFF00FF88;
                                buf[cy * w + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }
                }
                counted += text.len();
                y += 16 * scale + 12;
            }

            blit_buf(buf, w, h);
            frame += 1;
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    };

    // ---------------------------------------------------------------
    // CODE EDITOR SCREEN -- code types in char by char, then compiles
    // ms_per_char: typing speed for code
    // hold_frames: frames to wait after typing before "COMPILING"
    // output_hold_ms: MILLISECONDS to show output after execution (direct delay)
    // ---------------------------------------------------------------
    let show_code_and_run = |buf: &mut [u32], w: usize, h: usize,
                             rain_cols: &mut [u16], rain_speeds: &[u8],
                             title: &str,
                             source: &str,
                             pre_msg: &str,
                             _ms_per_char: u64,
                             _hold_frames: u32,
                             output_hold_ms: u64| {
        // ----------------------------------------------
        // Human-like typing: variable speed per character
        // with random pauses and scripted typos
        // ----------------------------------------------

        let lines_vec: alloc::vec::Vec<&str> = source.lines().collect();
        let total_chars: usize = source.len();

        let margin_x = 40usize;
        let header_h = 50usize;
        let code_y_start = header_h + 30;
        let line_h = 18usize;
        let code_scale = 1usize;

        // Build a flat char list with position info
        // Each entry: (line_idx, col_idx, char)
        let mut char_list: alloc::vec::Vec<(usize, usize, char)> = alloc::vec::Vec::new();
        for (li, line) in lines_vec.iter().enumerate() {
            for (ci, c) in line.chars().enumerate() {
                char_list.push((li, ci, c));
            }
            char_list.push((li, line.len(), '\n')); // newline marker
        }

        // Simple deterministic PRNG (no rand crate in kernel)
        let mut rng_state: u32 = 0xDEAD_BEEF;
        let mut rng_next = |state: &mut u32| -> u32 {
            *state ^= *state << 13;
            *state ^= *state >> 17;
            *state ^= *state << 5;
            *state
        };

        // Typo schedule: at these character indices, type wrong char, pause, backspace, retype
        // Format: (char_index, wrong_char)
        let typo_schedule: alloc::vec::Vec<(usize, char)> = alloc::vec![
            (45, 'w'),     // early typo on a variable
            (180, 'p'),    // mid-code typo
            (350, 'e'),    // in a function name
            (520, '0'),    // number typo
            (700, ';'),    // punctuation slip
        ];

        // Current "typed buffer" -- what's visible on screen
        // We track how many real chars are shown
        let mut chars_shown: usize = 0;
        let mut rain_frame: u32 = 0;

        // Render function (inline) -- draws the current state of the editor
        let render_editor = |buf: &mut [u32], w: usize, h: usize,
                             rain_cols: &mut [u16], rain_speeds: &[u8],
                             rain_frame: u32,
                             chars_shown: usize,
                             typo_char: Option<(usize, usize, char)>| {
            // Dark background with dimmed Matrix rain
            for p in buf.iter_mut() { *p = 0xFF0A0A0A; }
            draw_rain(buf, w, h, rain_cols, rain_speeds, rain_frame);
            for p in buf.iter_mut() {
                let g = ((*p >> 8) & 0xFF).min(25);
                *p = 0xFF000000 | (g << 8);
            }

            // Title bar
            for y in 0..header_h {
                for x in 0..w {
                    buf[y * w + x] = 0xFF111111;
                }
            }
            draw_text_at(buf, w, h, margin_x + 20, 15, title, 0xFF00FF88, 2);

            if !pre_msg.is_empty() {
                draw_text_at(buf, w, h, margin_x + 20, header_h + 5, pre_msg, 0xFF888888, 1);
            }

            // Code panel
            let panel_x = margin_x;
            let panel_w = w - 2 * margin_x;
            let panel_y = code_y_start;
            let panel_h = h - code_y_start - 80;
            for py in panel_y..panel_y + panel_h {
                for px in panel_x..panel_x + panel_w {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF0D1117;
                    }
                }
            }
            // Panel border (green)
            for px in panel_x..panel_x + panel_w {
                if panel_y < h { buf[panel_y * w + px] = 0xFF00FF44; }
                let bot = (panel_y + panel_h).min(h) - 1;
                buf[bot * w + px] = 0xFF00FF44;
            }
            for py in panel_y..(panel_y + panel_h).min(h) {
                buf[py * w + panel_x] = 0xFF00FF44;
                let right = (panel_x + panel_w - 1).min(w - 1);
                buf[py * w + right] = 0xFF00FF44;
            }

            // Compute scroll offset so cursor line stays visible
            let max_visible = panel_h.saturating_sub(30) / line_h;
            let cursor_line = {
                let mut ci = 0usize;
                let mut ln = 0usize;
                for (li, line) in lines_vec.iter().enumerate() {
                    if ci + line.len() >= chars_shown {
                        ln = li;
                        break;
                    }
                    ci += line.len() + 1;
                    ln = li + 1;
                }
                ln.min(lines_vec.len().saturating_sub(1))
            };
            let scroll_offset = if cursor_line >= max_visible.saturating_sub(2) {
                cursor_line.saturating_sub(max_visible.saturating_sub(3))
            } else {
                0
            };

            // Draw typed code (with scroll + syntax highlighting)
            let code_x = panel_x + 42;
            let mut global_idx = 0usize;
            // Skip chars in lines above scroll_offset
            for li in 0..scroll_offset.min(lines_vec.len()) {
                global_idx += lines_vec[li].len() + 1; // +1 for \n
            }
            for vi in 0..(lines_vec.len() - scroll_offset.min(lines_vec.len())) {
                let li = vi + scroll_offset;
                let ly = code_y_start + 15 + vi * line_h;
                if ly + 16 > panel_y + panel_h { break; }
                let line = lines_vec[li];

                // Line number
                let ln_str = alloc::format!("{:>3}", li + 1);
                draw_text_at(buf, w, h, panel_x + 8, ly, &ln_str, 0xFF555555, code_scale);
                let sep_x = panel_x + 35;
                for sy in ly..ly + 16 {
                    if sy < h && sep_x < w { buf[sy * w + sep_x] = 0xFF333333; }
                }

                // Syntax-highlighted characters
                let line_colors = trustlang_syntax_colors(line);
                for (ci, c) in line.chars().enumerate() {
                    if global_idx >= chars_shown { break; }
                    let color = line_colors.get(ci).copied().unwrap_or(0xFFD4D4D4);
                    draw_big_char(buf, w, h, code_x + ci * 8 * code_scale, ly, c, color, code_scale);
                    global_idx += 1;
                }

                // If there's a typo char visible at the cursor position, draw it in red
                if let Some((tli, tci, tc)) = typo_char {
                    if li == tli && global_idx == chars_shown {
                        let ly2 = code_y_start + 15 + vi * line_h;
                        draw_big_char(buf, w, h, code_x + tci * 8 * code_scale, ly2, tc, 0xFFFF4444, code_scale);
                    }
                }

                if global_idx < chars_shown { global_idx += 1; } // \n
            }

            // Scrollbar indicator (when content overflows)
            if lines_vec.len() > max_visible {
                let sb_x = panel_x + panel_w - 8;
                let sb_y = panel_y + 2;
                let sb_h = panel_h.saturating_sub(4);
                for py in sb_y..sb_y + sb_h {
                    if py < h && sb_x < w { buf[py * w + sb_x] = 0xFF1A1A1A; }
                }
                let thumb_h = ((max_visible * sb_h) / lines_vec.len()).max(10);
                let thumb_y = if lines_vec.len() > 0 { sb_y + (scroll_offset * sb_h) / lines_vec.len() } else { sb_y };
                for py in thumb_y..(thumb_y + thumb_h).min(sb_y + sb_h) {
                    if py < h && sb_x < w {
                        buf[py * w + sb_x] = 0xFF00FF44;
                        if sb_x + 1 < w { buf[py * w + sb_x + 1] = 0xFF00FF44; }
                    }
                }
            }

            // Cursor blink (always on during typing for visibility)
            if chars_shown <= total_chars && cursor_line >= scroll_offset {
                let mut ci2 = 0usize;
                let mut target_line = 0usize;
                let mut target_col = 0usize;
                for (li, line) in lines_vec.iter().enumerate() {
                    if ci2 + line.len() >= chars_shown {
                        target_line = li;
                        target_col = chars_shown - ci2;
                        break;
                    }
                    ci2 += line.len() + 1;
                }
                // Offset cursor past typo char if visible
                let cursor_col = if typo_char.is_some() && typo_char.unwrap().0 == target_line {
                    target_col + 1
                } else {
                    target_col
                };
                let vis_line = target_line.saturating_sub(scroll_offset);
                let cy = code_y_start + 15 + vis_line * line_h;
                let cx = panel_x + 42 + cursor_col * 8 * code_scale;
                if (rain_frame / 5) % 2 == 0 {
                    for sy in cy..cy + 16 {
                        if sy < h && cx < w && cx + 2 < w {
                            buf[sy * w + cx] = 0xFF00FF88;
                            buf[sy * w + cx + 1] = 0xFF00FF88;
                        }
                    }
                }
            }

            // Status bar
            let status_y = h - 40;
            for py in status_y..h {
                for px in 0..w { buf[py * w + px] = 0xFF111111; }
            }
            {
                let cur_line = {
                    let mut ci3 = 0usize;
                    let mut ln = 1usize;
                    for (_li, line) in lines_vec.iter().enumerate() {
                        if ci3 + line.len() >= chars_shown { break; }
                        ci3 += line.len() + 1;
                        ln += 1;
                    }
                    ln
                };
                let status = alloc::format!("Ln {}  |  {} lines  |  TrustLang", cur_line, lines_vec.len());
                draw_text_at(buf, w, h, margin_x, status_y + 12, &status, 0xFF00CC66, 1);
            }

            blit_buf(buf, w, h);
        };

        // -- Phase 1: Human-like typing animation --
        while chars_shown < total_chars {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { chars_shown = total_chars; break; }
            }

            // Check if we hit a typo point
            let mut did_typo = false;
            for &(typo_idx, wrong_c) in typo_schedule.iter() {
                if chars_shown == typo_idx && typo_idx < total_chars {
                    // Find which line/col we're at
                    let mut ci4 = 0usize;
                    let mut tgt_line = 0usize;
                    let mut tgt_col = 0usize;
                    for (li, line) in lines_vec.iter().enumerate() {
                        if ci4 + line.len() > chars_shown {
                            tgt_line = li;
                            tgt_col = chars_shown - ci4;
                            break;
                        }
                        ci4 += line.len() + 1;
                    }

                    // Type wrong char
                    render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame,
                        chars_shown, Some((tgt_line, tgt_col, wrong_c)));
                    rain_frame += 1;
                    crate::cpu::tsc::pit_delay_ms(120);

                    // Pause -- "notice the mistake"
                    render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame,
                        chars_shown, Some((tgt_line, tgt_col, wrong_c)));
                    rain_frame += 1;
                    crate::cpu::tsc::pit_delay_ms(400);

                    // Backspace -- remove wrong char (render without it)
                    render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame,
                        chars_shown, None);
                    rain_frame += 1;
                    crate::cpu::tsc::pit_delay_ms(150);

                    // Now type the correct char (fall through to normal typing below)
                    did_typo = true;
                    break;
                }
            }

            // Determine delay for this character
            let c = char_list.get(chars_shown).map(|&(_, _, c)| c).unwrap_or(' ');
            let next_c = char_list.get(chars_shown + 1).map(|&(_, _, c)| c);

            // Base typing speed: ~20ms per char (fast typist)
            let mut delay_ms: u64 = 20;

            // Newline: longer pause (thinking about next line)
            if c == '\n' {
                delay_ms = 80 + (rng_next(&mut rng_state) % 120) as u64; // 80-200ms
            }
            // After {  or before } : thinking pause
            else if c == '{' || (next_c == Some('}')) {
                delay_ms = 150 + (rng_next(&mut rng_state) % 200) as u64;
            }
            // After // comment start: slightly slower (typing words)
            else if c == '/' {
                delay_ms = 40 + (rng_next(&mut rng_state) % 60) as u64;
            }
            // Space: brief pause
            else if c == ' ' {
                delay_ms = 15 + (rng_next(&mut rng_state) % 40) as u64;
            }
            // Punctuation: slightly slower
            else if c == '(' || c == ')' || c == ';' || c == ',' {
                delay_ms = 30 + (rng_next(&mut rng_state) % 30) as u64;
            }
            // Regular chars: some variance
            else {
                delay_ms = 18 + (rng_next(&mut rng_state) % 25) as u64;
            }

            // Occasional random longer pause (~5% chance, "thinking")
            if rng_next(&mut rng_state) % 100 < 5 {
                delay_ms += 200 + (rng_next(&mut rng_state) % 400) as u64;
            }

            // After a typo correction, slight hesitation
            if did_typo {
                delay_ms += 80;
            }

            // Render current state
            render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame, chars_shown, None);
            rain_frame += 1;

            // Advance
            chars_shown += 1;
            crate::cpu::tsc::pit_delay_ms(delay_ms);
        }

        // Show complete code for a moment
        render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame, total_chars, None);
        crate::cpu::tsc::pit_delay_ms(1200);

        // -- Phase 2: In-editor compilation (bottom output pane) --
        // We re-render the editor with a small output pane at the bottom
        // showing "Compiling..." then "Compiled in 0.3s"
        {
            // Render the full editor one more time, then draw output pane on top
            render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame, total_chars, None);

            // Output pane: sits between code panel bottom and status bar
            let pane_y = h - 120;
            let pane_h = 80;
            let pane_x = margin_x;
            let pane_w = w - 2 * margin_x;

            // Dark output pane background
            for py in pane_y..pane_y + pane_h {
                for px in pane_x..pane_x + pane_w {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF0A0E14;
                    }
                }
            }
            // Pane border
            for px in pane_x..pane_x + pane_w {
                if pane_y < h { buf[pane_y * w + px] = 0xFF00FF44; }
            }
            // "OUTPUT" label
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 4, "OUTPUT", 0xFF888888, 1);

            // "$ trustlang compile youtube_dvd.tl"
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 22, "$ trustlang compile youtube_dvd.tl", 0xFF00CC66, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(800);

            // "Compiling..." appears
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 38, "Compiling...", 0xFFAABBCC, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(1200);

            // Replace "Compiling..." with success message
            for py in pane_y + 36..pane_y + 56 {
                for px in pane_x + 4..pane_x + pane_w - 4 {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF0A0E14;
                    }
                }
            }
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 38, "Compiled successfully in 0.3s  (47 lines, 0 errors)", 0xFF00FF88, 1);
            // Also show bytecode info
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 54, "Generated 284 bytecode instructions", 0xFF666666, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(2000);
        }

        // -- Phase 3: Transition to shell & execute --
        // Fake the TrustOS shell, type "trustlang run youtube_dvd.tl", then actually execute

        // Actually compile the program in ramfs first
        crate::ramfs::with_fs(|fs| {
            let _ = fs.write_file("/youtube_dvd.tl", source.as_bytes());
        });

        // Draw a fake shell screen
        {
            // Dark background with subtle rain
            for p in buf.iter_mut() { *p = 0xFF0A0A0A; }
            draw_rain(buf, w, h, rain_cols, rain_speeds, rain_frame);
            for p in buf.iter_mut() {
                let g = ((*p >> 8) & 0xFF).min(15);
                *p = 0xFF000000 | (g << 8);
            }

            // Shell window frame
            let win_x = 30usize;
            let win_y = 20usize;
            let win_w = w - 60;
            let win_h = h - 40;
            // Window background
            for py in win_y..win_y + win_h {
                for px in win_x..win_x + win_w {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF0D0D0D;
                    }
                }
            }
            // Title bar
            for py in win_y..win_y + 28 {
                for px in win_x..win_x + win_w {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF1A1A1A;
                    }
                }
            }
            draw_text_at(buf, w, h, win_x + 12, win_y + 6, "TrustOS Terminal", 0xFF00FF88, 1);
            // Border
            for px in win_x..win_x + win_w {
                if win_y < h { buf[win_y * w + px] = 0xFF00FF44; }
                let bot = (win_y + win_h - 1).min(h - 1);
                buf[bot * w + px] = 0xFF00FF44;
            }
            for py in win_y..win_y + win_h {
                if py < h {
                    buf[py * w + win_x] = 0xFF00FF44;
                    let r = (win_x + win_w - 1).min(w - 1);
                    buf[py * w + r] = 0xFF00FF44;
                }
            }

            let text_x = win_x + 16;
            let mut text_y = win_y + 40;

            // Show some previous shell output (fake history)
            draw_text_at(buf, w, h, text_x, text_y, "TrustOS v2.0 - TrustLang Runtime", 0xFF00FF88, 1);
            text_y += 20;
            draw_text_at(buf, w, h, text_x, text_y, "Type 'help' for available commands.", 0xFF666666, 1);
            text_y += 28;

            // Previous command: trustlang compile
            // Prompt: root@trustos:/$ 
            draw_text_at(buf, w, h, text_x, text_y, "root", 0xFFFF0000, 1);
            draw_text_at(buf, w, h, text_x + 32, text_y, "@", 0xFFFFFFFF, 1);
            draw_text_at(buf, w, h, text_x + 40, text_y, "trustos", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, text_x + 96, text_y, ":/$ ", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, text_x + 128, text_y, "trustlang compile youtube_dvd.tl", 0xFFD4D4D4, 1);
            text_y += 18;
            draw_text_at(buf, w, h, text_x, text_y, "Compiled successfully in 0.3s", 0xFF00FF88, 1);
            text_y += 18;
            draw_text_at(buf, w, h, text_x, text_y, "Generated 284 bytecode instructions", 0xFF666666, 1);
            text_y += 28;

            // New prompt where we'll type the run command
            let prompt_y = text_y;
            draw_text_at(buf, w, h, text_x, prompt_y, "root", 0xFFFF0000, 1);
            draw_text_at(buf, w, h, text_x + 32, prompt_y, "@", 0xFFFFFFFF, 1);
            draw_text_at(buf, w, h, text_x + 40, prompt_y, "trustos", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, text_x + 96, prompt_y, ":/$ ", 0xFF00FF00, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(800);

            // Type "trustlang run youtube_dvd.tl" character by character
            let run_cmd = "trustlang run youtube_dvd.tl";
            let cmd_start_x = text_x + 128;
            for (ci, c) in run_cmd.chars().enumerate() {
                draw_big_char(buf, w, h, cmd_start_x + ci * 8, prompt_y, c, 0xFFD4D4D4, 1);
                blit_buf(buf, w, h);
                let d = 30 + (((ci as u32 * 7 + 13) ^ 0x5A) % 50) as u64;
                crate::cpu::tsc::pit_delay_ms(d);
            }
            crate::cpu::tsc::pit_delay_ms(400);

            // Show "Enter" -- command execution
            text_y = prompt_y + 24;
            draw_text_at(buf, w, h, text_x, text_y, "Running youtube_dvd.tl ...", 0xFFAABBCC, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(600);
        }

        // Actually execute the TrustLang program
        match crate::trustlang::run(source) {
            Ok(output) => {
                if !output.is_empty() {
                    // Text output -- should not happen for this graphics demo,
                    // but handle it just in case
                    let out_lines: alloc::vec::Vec<&str> = output.lines().collect();
                    clear_buf(buf);
                    draw_text_centered(buf, w, h, 25, "OUTPUT", 0xFF00FF88, 3);
                    for (i, line) in out_lines.iter().enumerate() {
                        let ly = 80 + i * 20;
                        if ly + 16 > h - 40 { break; }
                        let sx = if line.len() * 8 < w { (w - line.len() * 8) / 2 } else { 40 };
                        draw_text_at(buf, w, h, sx, ly, line, 0xFFCCFFCC, 1);
                    }
                    blit_buf(buf, w, h);
                    crate::cpu::tsc::pit_delay_ms(output_hold_ms);
                }
                if output.is_empty() {
                    // Graphics program -- result is already on framebuffer
                    // Read back FB into buf for fade later
                    if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                        let bb = bb_ptr as *mut u32;
                        let bb_s = bb_stride as usize;
                        for y in 0..h.min(bb_h as usize) {
                            unsafe {
                                core::ptr::copy_nonoverlapping(
                                    bb.add(y * bb_s),
                                    buf[y * w..].as_mut_ptr(),
                                    w,
                                );
                            }
                        }
                    }
                    // Hold on graphics result
                    crate::cpu::tsc::pit_delay_ms(output_hold_ms);
                }
            }
            Err(e) => {
                clear_buf(buf);
                draw_text_centered(buf, w, h, h / 2 - 20, "Runtime Error", 0xFFFF4444, 4);
                let err_short = if e.len() > 80 { &e[..80] } else { &e };
                draw_text_centered(buf, w, h, h / 2 + 50, err_short, 0xFFFF8888, 1);
                blit_buf(buf, w, h);
                crate::cpu::tsc::pit_delay_ms(3000);
            }
        }
    };

    // ---------------------------------------------------------------
    //  THE SHOWCASE -- YouTube DVD Screensaver Demo
    //  Single TrustLang program: bouncing 3D YouTube logo
    // ---------------------------------------------------------------

    crate::serial_println!("[TL_SHOWCASE] Starting TrustLang showcase -- YouTube DVD Screensaver");

    // ------------------------------------------------
    // INTRO: Title Screen                    (~8s)
    // ------------------------------------------------
    clear_buf(&mut buf);
    show_description(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("TrustLang", 0xFF00FF88, 6),
          ("", 0xFF000000, 1),
          ("Live Demo", 0xFF00CC66, 4),
          ("Programming Inside TrustOS", 0xFF008844, 2)],
        90, 200);   // 90ms/char ? slow dramatic typing, hold 200 frames (~6s)
    do_fade(&mut buf, w, h, &blit_buf);

    // ------------------------------------------------
    // CONCEPT: What we're building            (~10s)
    // ------------------------------------------------
    clear_buf(&mut buf);
    show_description(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("YouTube DVD Screensaver", 0xFFFF0000, 4),
          ("", 0xFF000000, 1),
          ("A bouncing 3D YouTube logo", 0xFFCCFFCC, 2),
          ("with 'Like & Subscribe' text.", 0xFFCCFFCC, 2),
          ("", 0xFF000000, 1),
          ("Written, compiled, and animated", 0xFF00FF88, 2),
          ("live inside the OS kernel.", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("All in real-time. Zero dependencies.", 0xFF888888, 2)],
        70, 180);   // 70ms/char, hold 180 frames (~5.4s)
    do_fade(&mut buf, w, h, &blit_buf);

    // ------------------------------------------------
    // ARCHITECTURE: TrustLang pipeline         (~5s)
    // ------------------------------------------------
    clear_buf(&mut buf);
    show_description(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("How TrustLang Works", 0xFF00FF88, 4),
          ("", 0xFF000000, 1),
          ("tokenize()   Lexer -> Tokens", 0xFFFF7B72, 2),
          ("parse()      Tokens -> AST", 0xFFFFA657, 2),
          ("compile()    AST -> Bytecode", 0xFFA5D6FF, 2),
          ("execute()    Bytecode -> VM", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("pixel() fill_rect() draw_text()", 0xFFFFD700, 2),
          ("flush() sleep() clear_screen()", 0xFFFFD700, 2)],
        60, 170);   // 60ms/char, hold 170 frames (~5s)
    do_fade(&mut buf, w, h, &blit_buf);

    // ------------------------------------------------
    // THE CODE + EXECUTION                    (~50s)
    // ------------------------------------------------
    clear_buf(&mut buf);
    show_code_and_run(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        "TrustCode  -  youtube_dvd.tl",
        r#"// YouTube DVD Screensaver in TrustLang!
fn main() {
    let screen_width = screen_w();
    let screen_height = screen_h();
    // Logo dimensions
    let logo_width = 200;
    let logo_height = 140;
    let total_height = logo_height + 70;
    // Starting position & velocity
    let mut pos_x = screen_width / 4;
    let mut pos_y = screen_height / 4;
    let mut speed_x = 4;
    let mut speed_y = 3;
    // Animate 300 frames (~10 seconds)
    let mut frame = 0;
    while frame < 300 {
        clear_screen(0, 0, 0);
        // 3D shadow offset
        fill_rect(pos_x + 8, pos_y + 8, logo_width, logo_height, 50, 0, 0);
        // Red YouTube rectangle
        fill_rect(pos_x, pos_y, logo_width, logo_height, 230, 0, 0);
        // 3D highlight on top + dark edge on bottom
        fill_rect(pos_x, pos_y, logo_width, 3, 255, 60, 60);
        fill_rect(pos_x, pos_y + logo_height - 3, logo_width, 3, 150, 0, 0);
        // Play button triangle (white)
        let center_x = pos_x + logo_width / 2;
        let center_y = pos_y + logo_height / 2;
        let mut row = 0;
        while row < 70 {
            let offset = row - 35;
            let mut dist = offset;
            if dist < 0 { dist = 0 - dist; }
            let bar_width = 40 * (35 - dist) / 35;
            if bar_width > 0 {
                fill_rect(center_x - 12, center_y - 35 + row, bar_width, 1, 255, 255, 255);
            }
            row = row + 1;
        }
        // Bouncing text below logo
        draw_text("LIKE AND", pos_x + 18, pos_y + logo_height + 12, 255, 255, 255, 2);
        draw_text("SUBSCRIBE!", pos_x + 5, pos_y + logo_height + 42, 255, 80, 80, 2);
        flush();
        sleep(33);
        // Move & bounce off screen edges
        pos_x = pos_x + speed_x;
        pos_y = pos_y + speed_y;
        if pos_x + logo_width > screen_width { speed_x = 0 - speed_x; }
        if pos_x < 0 { speed_x = 0 - speed_x; }
        if pos_y + total_height > screen_height { speed_y = 0 - speed_y; }
        if pos_y < 0 { speed_y = 0 - speed_y; }
        frame = frame + 1;
    }
}"#,
        "New file: /youtube_dvd.tl",
        30,     // unused (human typing now)
        80,     // unused (human typing now)
        3000);  // hold 3s on final frame after animation ends
    do_fade(&mut buf, w, h, &blit_buf);

    // ------------------------------------------------
    // OUTRO                                   (~8s)
    // ------------------------------------------------
    clear_buf(&mut buf);
    show_description(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("TrustLang", 0xFF00FF88, 6),
          ("", 0xFF000000, 1),
          ("Lexer > Parser > Compiler > VM", 0xFFAADDAA, 2),
          ("Real-time graphics. Zero deps.", 0xFFAADDAA, 2),
          ("", 0xFF000000, 1),
          ("Built into TrustOS.", 0xFF00CC66, 3),
          ("", 0xFF000000, 1),
          ("github.com/nathan237/TrustOS", 0xFF00FF88, 2)],
        80, 250);   // slow dramatic outro, hold 250 frames (~7.5s)
    do_fade(&mut buf, w, h, &blit_buf);

    // Restore framebuffer state
    clear_buf(&mut buf);
    blit_buf(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[TL_SHOWCASE] Showcase complete");
}

/// TrustOS Film -- Animated cinematic explainer for non-technical audiences
/// Each scene uses a unique background for maximum visual retention:
///   ACT I:   Slow pulsing deep-blue gradient  (mystery, intrigue)
///   ACT II:  Red scan-lines / warning stripes  (urgency, alarm)
///   Bars:    Blueprint dot-grid                (data, precision)
///   ACT III: Rising green particle sparks      (hope, energy)
///   Grid:    Deep-space starfield              (scale, wonder)
///   ACT IV:  Circuit-board trace lines         (tech, proof)
///   ACT V:   Sunrise gradient warm glow        (inspiration)
///   Outro:   Matrix rain callback              (signature)
/// Retention techniques applied: pattern interrupts every scene,
/// high-contrast color shifts, kinetic text, less text per screen,
/// constant subtle motion, under-2-minute total runtime.
pub(super) fn cmd_trustos_film() {
    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    // --------------- HELPER CLOSURES ---------------

    let draw_big_char = |buf: &mut [u32], w: usize, h: usize,
                         cx: usize, cy: usize, c: char, color: u32, scale: usize| {
        let glyph = crate::framebuffer::font::get_glyph(c);
        for (row, &bits) in glyph.iter().enumerate() {
            for bit in 0..8u32 {
                if bits & (0x80 >> bit) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cx + bit as usize * scale + sx;
                            let py = cy + row * scale + sy;
                            if px < w && py < h { buf[py * w + px] = color; }
                        }
                    }
                }
            }
        }
    };

    let draw_text_at = |buf: &mut [u32], w: usize, h: usize,
                        x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            draw_big_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize,
                              y: usize, text: &str, color: u32, scale: usize| {
        let tw = text.len() * 8 * scale;
        let sx = if tw < w { (w - tw) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
        }
    };

    let blit_buf = |buf: &[u32], w: usize, h: usize| {
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
            let bb = bb_ptr as *mut u32;
            let bb_s = bb_stride as usize;
            for y in 0..h.min(bb_h as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buf[y * w..].as_ptr(), bb.add(y * bb_s), w);
                }
            }
        }
        crate::framebuffer::swap_buffers();
    };

    let clear_buf = |buf: &mut [u32]| {
        for p in buf.iter_mut() { *p = 0xFF000000; }
    };

    // Filled rectangle helper
    let draw_rect = |buf: &mut [u32], w: usize, h: usize,
                     x: usize, y: usize, rw: usize, rh: usize, color: u32| {
        for dy in 0..rh {
            for dx in 0..rw {
                let px = x + dx;
                let py = y + dy;
                if px < w && py < h { buf[py * w + px] = color; }
            }
        }
    };

    let frame_ms: u64 = 30;

    let do_fade = |buf: &mut [u32], w: usize, h: usize,
                   blit: &dyn Fn(&[u32], usize, usize)| {
        for _ in 0..40 {
            for px in buf.iter_mut() {
                let r = ((*px >> 16) & 0xFF).saturating_sub(8);
                let g = ((*px >> 8) & 0xFF).saturating_sub(8);
                let b = (*px & 0xFF).saturating_sub(8);
                *px = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
        for p in buf.iter_mut() { *p = 0xFF000000; }
        blit(buf, w, h);
        crate::cpu::tsc::pit_delay_ms(400);
    };

    // --------------- BACKGROUND GENERATORS ---------------
    // Each produces a unique animated background per-frame.

    // BG1: Pulsing deep-blue/purple nebula gradient (ACT I -- mystery)
    let bg_pulse = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        // Integer-based sine approximation (no libm in no_std)
        // Triangle wave oscillating 0..40..0 over ~160 frames
        let phase = (frame % 160) as u32;
        let pulse = if phase < 80 { phase / 2 } else { (160 - phase) / 2 };
        let phase2 = ((frame + 40) % 120) as u32;
        let pulse2 = if phase2 < 60 { phase2 / 2 } else { (120 - phase2) / 2 };
        for y in 0..h {
            let yf = (y as u32 * 40) / h as u32;
            for x in 0..w {
                let xf = (x as u32 * 10) / w as u32;
                let r = (yf / 4 + pulse2 / 3).min(40);
                let g = (xf / 3).min(15);
                let b = (yf + pulse + xf / 2).min(80);
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    };

    // BG2: Red warning scan-lines (ACT II -- danger/urgency)
    let bg_scanlines = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        let scroll = (frame as usize * 2) % h;
        for y in 0..h {
            let sy = (y + scroll) % h;
            let stripe = (sy / 4) % 2 == 0;
            for x in 0..w {
                let base_r = if stripe { 35u32 } else { 15 };
                let flash = if (sy % 60) < 2 { 30u32 } else { 0 };
                let r = (base_r + flash).min(65);
                let g = 2;
                let b = 5;
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    };

    // BG3: Blueprint dot-grid (comparison bars -- precision)
    let bg_dotgrid = |buf: &mut [u32], w: usize, h: usize, _frame: u32| {
        for y in 0..h {
            for x in 0..w {
                let on_grid = (x % 20 < 2) && (y % 20 < 2);
                let color = if on_grid { 0xFF0A1A3A } else { 0xFF060E1E };
                buf[y * w + x] = color;
            }
        }
    };

    // BG4: Rising green sparks / particles (ACT III -- hope)
    let bg_sparks = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        // Dim previous frame for trailing effect
        for px in buf.iter_mut() {
            let r = ((*px >> 16) & 0xFF).saturating_sub(8);
            let g = ((*px >> 8) & 0xFF).saturating_sub(12);
            let b = (*px & 0xFF).saturating_sub(8);
            *px = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
        // Spawn particles rising from bottom
        for i in 0..24u32 {
            let seed = (i.wrapping_mul(2654435761).wrapping_add(frame.wrapping_mul(37))) as usize;
            let px = (seed.wrapping_mul(7919)) % w;
            let rise = (frame as usize + seed) % h;
            let py = h.saturating_sub(rise);
            let brightness = (50 + (seed % 40)) as u32;
            if px < w && py < h {
                buf[py * w + px] = 0xFF000000 | (brightness / 4 << 16) | (brightness << 8) | (brightness / 3);
                if px + 1 < w { buf[py * w + px + 1] = 0xFF000000 | (brightness << 8); }
            }
        }
    };

    // BG5: Deep-space starfield (feature grid -- wonder/scale)
    let bg_stars = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        for p in buf.iter_mut() { *p = 0xFF050510; }
        // Fixed stars with twinkle
        for i in 0..80u32 {
            let sx = ((i.wrapping_mul(7919)) as usize) % w;
            let sy = ((i.wrapping_mul(104729)) as usize) % h;
            let twinkle = ((frame.wrapping_add(i * 17)) % 30) as u32;
            let bright = if twinkle < 15 { 40 + twinkle * 3 } else { 40 + (30 - twinkle) * 3 };
            let bright = bright.min(120);
            if sx < w && sy < h {
                buf[sy * w + sx] = 0xFF000000 | (bright << 16) | (bright << 8) | bright;
            }
        }
    };

    // BG6: Circuit-board traces (ACT IV -- technical proof)
    let bg_circuit = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        for p in buf.iter_mut() { *p = 0xFF0A0A14; }
        // Horizontal and vertical traces
        let trace_color = 0xFF0F2818u32;
        let active_color = 0xFF00AA44u32;
        for i in 0..20u32 {
            let ty = ((i.wrapping_mul(7919) as usize) % h) & !3;
            let tx = ((i.wrapping_mul(104729) as usize) % w) & !3;
            // Horizontal lines
            if ty < h {
                for x in 0..w {
                    buf[ty * w + x] = trace_color;
                }
            }
            // Vertical lines
            if tx < w {
                for y in 0..h {
                    buf[y * w + tx] = trace_color;
                }
            }
        }
        // Animated pulse along a trace
        let pulse_y = ((frame as usize * 3) % h) & !3;
        if pulse_y < h {
            let pw = (w / 4).min(120);
            let px_start = (frame as usize * 5) % w;
            for dx in 0..pw {
                let px = (px_start + dx) % w;
                buf[pulse_y * w + px] = active_color;
                if pulse_y + 1 < h { buf[(pulse_y + 1) * w + px] = active_color; }
            }
        }
    };

    // BG7: Sunrise warm gradient (ACT V -- inspiration)
    let bg_sunrise = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        let lift = (frame as u32).min(60); // sun rises over 60 frames
        for y in 0..h {
            let yf = y as u32 * 100 / h as u32; // 0=top, 100=bottom
            let warmth = if yf > 50 { (yf - 50).min(50) + lift } else { lift / 2 };
            let r = (warmth * 2).min(90);
            let g = (warmth * 3 / 4).min(45);
            let b = (20u32.saturating_sub(warmth / 3)).min(30);
            for x in 0..w {
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
        // Sun glow at bottom center
        let sun_cx = w / 2;
        let sun_cy = h - (lift as usize * h / 200);
        let sun_r = 80usize + lift as usize;
        for dy in 0..sun_r {
            for dx in 0..sun_r {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < sun_r * sun_r {
                    let intensity = (sun_r * sun_r - dist_sq) * 60 / (sun_r * sun_r);
                    let intensity = intensity as u32;
                    for (sx, sy) in [(sun_cx + dx, sun_cy.wrapping_sub(dy)),
                                     (sun_cx.wrapping_sub(dx), sun_cy.wrapping_sub(dy))] {
                        if sx < w && sy < h {
                            let existing = buf[sy * w + sx];
                            let er = ((existing >> 16) & 0xFF) + intensity;
                            let eg = ((existing >> 8) & 0xFF) + intensity * 2 / 3;
                            let eb = (existing & 0xFF) + intensity / 4;
                            buf[sy * w + sx] = 0xFF000000
                                | (er.min(255) << 16)
                                | (eg.min(255) << 8)
                                | eb.min(255);
                        }
                    }
                }
            }
        }
    };

    // BG8: Matrix rain -- signature callback for outro only
    let mut rain_cols: alloc::vec::Vec<u16> =
        (0..w / 8 + 1).map(|i| ((i * 37 + 13) % h) as u16).collect();
    let rain_speeds: alloc::vec::Vec<u8> =
        (0..w / 8 + 1).map(|i| (((i * 7 + 3) % 4) + 1) as u8).collect();

    let draw_rain = |buf: &mut [u32], w: usize, h: usize,
                     cols: &mut [u16], speeds: &[u8], frame: u32| {
        for pixel in buf.iter_mut() {
            let g = ((*pixel >> 8) & 0xFF) as u32;
            if g > 0 { *pixel = 0xFF000000 | (g.saturating_sub(6) << 8); }
        }
        for ci in 0..cols.len() {
            let x = ci * 8;
            if x >= w { continue; }
            cols[ci] = cols[ci].wrapping_add(speeds[ci] as u16);
            if cols[ci] as usize >= h { cols[ci] = 0; }
            let y = cols[ci] as usize;
            let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
            let glyph = crate::framebuffer::font::get_glyph(c);
            for (row, &bits) in glyph.iter().enumerate() {
                let py = y + row;
                if py >= h { break; }
                for bit in 0..8u32 {
                    if bits & (0x80 >> bit) != 0 {
                        let px = x + bit as usize;
                        if px < w { buf[py * w + px] = 0xFF00FF44; }
                    }
                }
            }
        }
    };

    // --------------- SCENE RENDERER ---------------
    // Generic scene: animate background + type text on top.
    // bg_id: 1=pulse, 2=scanlines, 3=dotgrid, 4=sparks, 5=stars,
    //        6=circuit, 7=sunrise, 8=rain
    let show_scene = |buf: &mut [u32], w: usize, h: usize,
                      rain_cols: &mut [u16], rain_speeds: &[u8],
                      lines: &[(&str, u32, usize)],
                      ms_per_char: u64, hold_frames: u32, bg_id: u8| {
        let total_chars: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
        let frames_per_char = (ms_per_char / frame_ms).max(1) as u32;
        let typing_frames = total_chars as u32 * frames_per_char;
        let total_frames = typing_frames + hold_frames;
        let mut frame = 0u32;
        while frame < total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            // Draw the per-scene background
            match bg_id {
                1 => bg_pulse(buf, w, h, frame),
                2 => bg_scanlines(buf, w, h, frame),
                3 => bg_dotgrid(buf, w, h, frame),
                4 => bg_sparks(buf, w, h, frame),
                5 => bg_stars(buf, w, h, frame),
                6 => bg_circuit(buf, w, h, frame),
                7 => bg_sunrise(buf, w, h, frame),
                8 => draw_rain(buf, w, h, rain_cols, rain_speeds, frame),
                _ => { for p in buf.iter_mut() { *p = 0xFF000000; } }
            }
            let chars_shown = (frame / frames_per_char) as usize;
            let total_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 12).sum();
            let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
            let mut counted = 0usize;
            for &(text, color, scale) in lines {
                let tw = text.len() * 8 * scale;
                let sx = if tw < w { (w - tw) / 2 } else { 0 };
                for (i, c) in text.chars().enumerate() {
                    if counted + i >= chars_shown { break; }
                    draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
                }
                // Blinking cursor during typing
                if chars_shown > counted && chars_shown < counted + text.len() {
                    let ci = chars_shown - counted;
                    let cx = sx + ci * 8 * scale;
                    if (frame / 8) % 2 == 0 {
                        for cy in y..y + 16 * scale {
                            if cy < h && cx + 2 < w {
                                buf[cy * w + cx] = 0xFFFFFFFF;
                                buf[cy * w + cx + 1] = 0xFFFFFFFF;
                            }
                        }
                    }
                }
                counted += text.len();
                y += 16 * scale + 12;
            }
            blit_buf(buf, w, h);
            frame += 1;
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    };

    crate::serial_println!("[FILM] TrustOS Film started");

    // ---------------------------------------------------------------
    //  ACT I  --  THE QUESTION  (unique animations per scene)
    // ---------------------------------------------------------------
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT I", 0xFF88CCFF, 5)],
        50, 30, 1);
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 1: Floating Windows -- "You use a computer every day" --
    {
        let fpc = 2u32;
        let text1 = "You use a computer";
        let text2 = "every single day.";
        let total_chars = text1.len() + text2.len();
        let total_frames = total_chars as u32 * fpc + 50;
        // Window state: x, y, width, height, color, dx, dy
        let mut wins: [(i32,i32,usize,usize,u32,i32,i32); 6] = [
            (80, 40, 120, 80, 0xFF3355AA, 2, 1),
            (w as i32 - 220, 90, 100, 70, 0xFF55AA33, -1, 2),
            (180, h as i32 - 180, 130, 85, 0xFFAA5533, 1, -1),
            (w as i32 / 2, 60, 110, 75, 0xFF8844CC, -2, 1),
            (40, h as i32 / 2, 125, 80, 0xFF4488CC, 1, -2),
            (w as i32 - 160, h as i32 / 2 + 40, 100, 65, 0xFFCC8844, -1, -1),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_pulse(&mut buf, w, h, frame);
            // Animate floating windows
            for wi in 0..6 {
                let win = &mut wins[wi];
                // Only show window after a staggered delay
                if frame < (wi as u32) * 8 { continue; }
                win.0 += win.5;
                win.1 += win.6;
                if win.0 < 0 || win.0 + win.2 as i32 > w as i32 { win.5 = -win.5; win.0 += win.5; }
                if win.1 < 0 || win.1 + win.3 as i32 > h as i32 { win.6 = -win.6; win.1 += win.6; }
                let wx = win.0.max(0) as usize;
                let wy = win.1.max(0) as usize;
                let wc = win.4;
                let wr = ((wc >> 16) & 0xFF) / 3;
                let wg = ((wc >> 8)  & 0xFF) / 3;
                let wb = (wc & 0xFF) / 3;
                let dim = 0xFF000000 | (wr << 16) | (wg << 8) | wb;
                draw_rect(&mut buf, w, h, wx, wy, win.2, win.3, dim);
                draw_rect(&mut buf, w, h, wx, wy, win.2, 10, wc);
                // Fake content lines inside window
                for li in 0..3usize {
                    let ly = wy + 16 + li * 12;
                    if ly + 5 < wy + win.3 {
                        draw_rect(&mut buf, w, h, wx + 6, ly, win.2.saturating_sub(12), 5, 0xFF222233);
                    }
                }
            }
            // Type text on top
            let chars_shown = (frame / fpc) as usize;
            let scale = 3usize;
            let line_h = 16 * scale + 12;
            let y1 = h / 2 - line_h;
            let y2 = h / 2 + 4;
            let tw1 = text1.len() * 8 * scale;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale, y1, c, 0xFFFFFFFF, scale);
            }
            if chars_shown > text1.len() {
                let tw2 = text2.len() * 8 * scale;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale, y2, c, 0xFFFFFFFF, scale);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 2: Question Marks Rain -- "Do you really know?" --
    {
        let fpc = 2u32;
        let text1 = "Do you really know";
        let text2 = "what it does?";
        let total_chars = text1.len() + text2.len();
        let total_frames = total_chars as u32 * fpc + 60;
        let num_qcols = w / 10;
        let mut qy: alloc::vec::Vec<i32> = (0..num_qcols).map(|i| -((i * 37 % 200) as i32)).collect();
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_pulse(&mut buf, w, h, frame);
            // Rain question marks with acceleration
            let speed = 1 + (frame / 30) as i32;
            for qi in 0..num_qcols {
                qy[qi] += speed + (qi as i32 % 3);
                if qy[qi] > h as i32 { qy[qi] = -(qi as i32 * 13 % 60); }
                if qy[qi] >= 0 {
                    let px = qi * 10 + 2;
                    let py = qy[qi] as usize;
                    let bright = 0xFF000000 | (0x40 << 16) | (0x60 << 8) | 0xFF;
                    if px < w && py < h {
                        draw_big_char(&mut buf, w, h, px, py, '?', bright, 1);
                    }
                }
            }
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale = 3usize;
            let y1 = h / 2 - 40;
            let y2 = h / 2 + 20;
            let tw1 = text1.len() * 8 * scale;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale, y1, c, 0xFFCCCCCC, scale);
            }
            if chars_shown > text1.len() {
                let tw2 = text2.len() * 8 * scale;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale, y2, c, 0xFFFF9944, 4);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 3: Screen Shatter -- "The honest answer... is no." --
    {
        let fpc = 2u32;
        let text1 = "The honest answer...";
        let text2 = "is no.";
        let type_frames = (text1.len() + text2.len()) as u32 * fpc;
        let shatter_frames = 50u32;
        let total_frames = type_frames + shatter_frames;
        // Crack directions: dx, dy pairs radiating from center
        let cracks: [(i32, i32); 12] = [
            (3,0),(-3,0),(0,3),(0,-3),(2,2),(-2,2),(2,-2),(-2,-2),
            (3,1),(-3,1),(1,-3),(-1,3),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_scanlines(&mut buf, w, h, frame);
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale1 = 3usize;
            let y1 = h / 2 - 60;
            let tw1 = text1.len() * 8 * scale1;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale1, y1, c, 0xFF888888, scale1);
            }
            if chars_shown > text1.len() {
                let scale2 = 5usize;
                let tw2 = text2.len() * 8 * scale2;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale2, h / 2, c, 0xFFFF4444, scale2);
                }
            }
            // Shatter effect after typing
            if frame > type_frames {
                let progress = frame - type_frames;
                let cx = w / 2;
                let cy = h / 2;
                for &(cdx, cdy) in cracks.iter() {
                    for step in 0..(progress * 4) as i32 {
                        let px = (cx as i32 + cdx * step).max(0) as usize;
                        let py = (cy as i32 + cdy * step).max(0) as usize;
                        if px < w && py < h {
                            buf[py * w + px] = 0xFFFFFFFF;
                            if px + 1 < w { buf[py * w + px + 1] = 0xFFFFDDDD; }
                            if py + 1 < h { buf[(py + 1) * w + px] = 0xFFFFDDDD; }
                        }
                    }
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ---------------------------------------------------------------
    //  ACT II  --  THE PROBLEM  (binary flood + redacted bars)
    // ---------------------------------------------------------------
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT II", 0xFFFF6644, 5),
          ("", 0xFF000000, 1),
          ("The Problem", 0xFFFF4444, 3)],
        50, 30, 2);
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 4: Binary Flood -- "It controls EVERYTHING" --
    {
        let fpc = 2u32;
        let lines_txt: [(&str, u32, usize); 5] = [
            ("Your computer runs on", 0xFFCCCCCC, 2),
            ("an operating system.", 0xFFCCCCCC, 2),
            ("", 0xFF000000, 1),
            ("It controls", 0xFFCCCCCC, 2),
            ("EVERYTHING.", 0xFFFF6644, 4),
        ];
        let total_chars: usize = lines_txt.iter().map(|(t,_,_)| t.len()).sum();
        let total_frames = total_chars as u32 * fpc + 70;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_scanlines(&mut buf, w, h, frame);
            // Binary flood: 0s and 1s cascading from top
            let flood_rows = (frame as usize * 3).min(h);
            for fy in 0..flood_rows {
                if fy >= h { break; }
                // Sparse binary chars
                for fx_step in 0..w / 12 {
                    let fx = fx_step * 12;
                    let seed = (fy.wrapping_mul(7919) + fx.wrapping_mul(104729) + frame as usize * 37) % 100;
                    if seed < 15 {
                        let c = if seed < 8 { '0' } else { '1' };
                        let bright = (20 + (seed * 2)) as u32;
                        let color = 0xFF000000 | (bright << 16) | ((bright / 2) << 8) | (bright / 4);
                        draw_big_char(&mut buf, w, h, fx, fy, c, color, 1);
                    }
                }
            }
            // Type text on top with dark backdrop
            let chars_shown = (frame / fpc) as usize;
            let total_h: usize = lines_txt.iter().map(|(_,_,s)| 16 * s + 12).sum();
            let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
            let mut counted = 0usize;
            for &(text, color, scale) in lines_txt.iter() {
                let tw = text.len() * 8 * scale;
                let sx = if tw < w { (w - tw) / 2 } else { 0 };
                // Dark background behind text
                if !text.is_empty() {
                    draw_rect(&mut buf, w, h, sx.saturating_sub(4), y.saturating_sub(2),
                        tw + 8, 16 * scale + 4, 0xCC000000);
                }
                for (i, c) in text.chars().enumerate() {
                    if counted + i >= chars_shown { break; }
                    draw_big_char(&mut buf, w, h, sx + i * 8 * scale, y, c, color, scale);
                }
                counted += text.len();
                y += 16 * scale + 12;
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 5: Redacted Bars -- "Nobody knows what's inside" --
    {
        let fpc = 2u32;
        let lines_txt: [(&str, u32, usize); 5] = [
            ("But nobody knows", 0xFFCCCCCC, 3),
            ("what's inside it.", 0xFFCCCCCC, 3),
            ("", 0xFF000000, 1),
            ("Not even the people", 0xFFFF4444, 2),
            ("who wrote it.", 0xFFFF4444, 2),
        ];
        let total_chars: usize = lines_txt.iter().map(|(t,_,_)| t.len()).sum();
        let type_frames = total_chars as u32 * fpc;
        let redact_frames = 60u32;
        let total_frames = type_frames + redact_frames;
        // Fake document lines to redact
        let doc_lines: [(&str, usize); 6] = [
            ("Source code: kernel/mm/init.c", 60),
            ("Author: CLASSIFIED", 140),
            ("Memory manager: UNKNOWN", 220),
            ("Security audit: NONE PERFORMED", 300),
            ("Bug count: UNTRACKED", 380),
            ("Last review: NEVER", 460),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_scanlines(&mut buf, w, h, frame);
            // Draw fake document on left side
            let doc_x = 30usize;
            for &(line, dy) in doc_lines.iter() {
                if dy < h {
                    draw_text_at(&mut buf, w, h, doc_x, dy, line, 0xFF445566, 1);
                }
            }
            // Redact bars sliding in after typing completes
            if frame > type_frames {
                let progress = frame - type_frames;
                for (di, &(_line, dy)) in doc_lines.iter().enumerate() {
                    let delay = di as u32 * 6;
                    if progress > delay {
                        let bar_w = ((progress - delay) as usize * 12).min(280);
                        if dy < h {
                            draw_rect(&mut buf, w, h, doc_x, dy.saturating_sub(2),
                                bar_w, 14, 0xFF000000);
                            if bar_w > 80 {
                                draw_text_at(&mut buf, w, h, doc_x + 4, dy,
                                    "REDACTED", 0xFFFF2222, 1);
                            }
                        }
                    }
                }
            }
            // Main text on right side
            let chars_shown = (frame / fpc) as usize;
            let text_x_base = w / 2 + 20;
            let total_h: usize = lines_txt.iter().map(|(_,_,s)| 16 * s + 12).sum();
            let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
            let mut counted = 0usize;
            for &(text, color, scale) in lines_txt.iter() {
                let sx = text_x_base;
                for (i, c) in text.chars().enumerate() {
                    if counted + i >= chars_shown { break; }
                    draw_big_char(&mut buf, w, h, sx + i * 8 * scale, y, c, color, scale);
                }
                counted += text.len();
                y += 16 * scale + 12;
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 6: Bar Chart with Earthquake Shake --
    {
        let bar_data: [(&str, u32, u32); 4] = [
            ("Windows",  50_000_000, 0xFFFF4444),
            ("macOS",    30_000_000, 0xFFFFAA22),
            ("Linux",    28_000_000, 0xFFFF8800),
            ("TrustOS",     120_000, 0xFF00FF88),
        ];
        let max_val = 50_000_000u32;
        let bar_max_w = w * 3 / 5;
        let bar_h_px = 40usize;
        let bar_spacing = 80usize;
        let start_y = h / 2 - (bar_data.len() * bar_spacing) / 2;
        let label_x = 40usize;

        let mut shake_x: i32 = 0;
        let mut shake_y: i32 = 0;

        for frame in 0..160u32 {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_dotgrid(&mut buf, w, h, frame);

            let progress = if frame < 30 { 0u32 }
                else { ((frame - 30) * 100 / 70).min(100) };

            // Earthquake shake when TrustOS bar appears (frame 100+)
            if frame > 100 && frame < 130 {
                let seed = frame.wrapping_mul(7919) as i32;
                shake_x = (seed % 7) - 3;
                shake_y = ((seed / 7) % 5) - 2;
            } else {
                shake_x = 0;
                shake_y = 0;
            }

            draw_text_centered(&mut buf, w, h,
                (30i32 + shake_y) as usize,
                "Lines of Code per OS", 0xFFFFFFFF, 3);

            for (i, &(name, val, color)) in bar_data.iter().enumerate() {
                let y = ((start_y + i * bar_spacing) as i32 + shake_y).max(0) as usize;
                let adj_label_x = (label_x as i32 + shake_x).max(0) as usize;
                draw_text_at(&mut buf, w, h, adj_label_x, y + 10,
                    name, 0xFFFFFFFF, 2);
                let bar_x = (adj_label_x + 170).min(w.saturating_sub(10));
                draw_rect(&mut buf, w, h, bar_x, y,
                    bar_max_w, bar_h_px, 0xFF111122);
                let full_w = (val as usize * bar_max_w) / max_val as usize;
                let target_w = full_w.max(12);
                let current_w = target_w * progress as usize / 100;
                draw_rect(&mut buf, w, h, bar_x, y,
                    current_w, bar_h_px, color);

                // Flash effect on TrustOS bar appearance
                if i == 3 && frame > 100 && frame < 110 {
                    let flash = 0xFF88FFAA;
                    draw_rect(&mut buf, w, h, bar_x, y,
                        current_w + 4, bar_h_px + 4, flash);
                }

                if frame > 70 {
                    let label = if val >= 1_000_000 {
                        alloc::format!("{}M", val / 1_000_000)
                    } else {
                        alloc::format!("{}K", val / 1000)
                    };
                    draw_text_at(&mut buf, w, h, bar_x + current_w + 10,
                        y + 10, &label, 0xFFFFFFFF, 2);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // Contrast emphasis
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("50 million vs 120 thousand.", 0xFFFFFFFF, 3),
          ("", 0xFF000000, 1),
          ("Like comparing a city", 0xFFCCCCCC, 2),
          ("to a single house.", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("Except the house", 0xFFCCCCCC, 2),
          ("does everything.", 0xFF00FF88, 3)],
        50, 80, 2);
    do_fade(&mut buf, w, h, &blit_buf);

    // ---------------------------------------------------------------
    //  ACT III  --  THE SOLUTION  (light burst + odometer counter)
    // ---------------------------------------------------------------
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT III", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("The Solution", 0xFF00CC66, 3)],
        50, 30, 4);
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 7: Light Burst -- "What if one person could understand ALL of it?" --
    {
        let fpc = 2u32;
        let text1 = "What if one person";
        let text2 = "could understand ALL of it?";
        let total_chars = text1.len() + text2.len();
        let total_frames = total_chars as u32 * fpc + 60;
        // 8 ray directions (dx, dy)
        let rays: [(i32, i32); 16] = [
            (4,0),(-4,0),(0,4),(0,-4),(3,3),(-3,3),(3,-3),(-3,-3),
            (4,1),(4,-1),(-4,1),(-4,-1),(1,4),(1,-4),(-1,4),(-1,-4),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_sparks(&mut buf, w, h, frame);
            // Light burst: pulsing center with star rays
            let cx = w / 2;
            let cy = h / 2;
            // Pulsing intensity
            let phase = (frame % 40) as u32;
            let pulse = if phase < 20 { phase * 4 } else { (40 - phase) * 4 };
            // Draw rays
            let ray_len = 40 + (frame / 2) as i32;
            for &(rdx, rdy) in rays.iter() {
                for step in 0..ray_len {
                    let px = (cx as i32 + rdx * step).max(0) as usize;
                    let py = (cy as i32 + rdy * step).max(0) as usize;
                    if px < w && py < h {
                        let falloff = (ray_len - step) as u32 * 3;
                        let bright = (pulse + falloff).min(180);
                        let r = bright;
                        let g = (bright * 3 / 4).min(140);
                        let b = (bright / 3).min(60);
                        let existing = buf[py * w + px];
                        let er = ((existing >> 16) & 0xFF) + r;
                        let eg = ((existing >> 8) & 0xFF) + g;
                        let eb = (existing & 0xFF) + b;
                        buf[py * w + px] = 0xFF000000
                            | (er.min(255) << 16)
                            | (eg.min(255) << 8)
                            | eb.min(255);
                    }
                }
            }
            // Center glow
            let glow_r = 15 + (pulse / 4) as usize;
            for dy in 0..glow_r {
                for dx in 0..glow_r {
                    if dx * dx + dy * dy < glow_r * glow_r {
                        for &(sx, sy) in &[(cx+dx, cy+dy),(cx+dx, cy.wrapping_sub(dy)),
                                           (cx.wrapping_sub(dx), cy+dy),
                                           (cx.wrapping_sub(dx), cy.wrapping_sub(dy))] {
                            if sx < w && sy < h {
                                buf[sy * w + sx] = 0xFFFFFFCC;
                            }
                        }
                    }
                }
            }
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale = 3usize;
            let y1 = h / 4;
            let y2 = y1 + 16 * scale + 12;
            let tw1 = text1.len() * 8 * scale;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale, y1, c, 0xFFFFFFFF, scale);
            }
            if chars_shown > text1.len() {
                let tw2 = text2.len() * 8 * scale;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale, y2, c, 0xFF00FF88, scale);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 8: Odometer Counter -- TrustOS stats --
    {
        let stats: [(&str, &str, u32, u32); 4] = [
            ("", "lines of code", 120_000, 0xFF00FF88),
            ("", "author", 1, 0xFFFFFFFF),
            ("", "secrets", 0, 0xFFFFFFFF),
            ("100%", "Rust.  0% C.", 0, 0xFFFF7744),
        ];
        let total_frames = 140u32;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_sparks(&mut buf, w, h, frame);
            // Title
            draw_text_centered(&mut buf, w, h, 40, "TrustOS", 0xFF00FF88, 6);
            // Odometer: numbers roll from 0 to target
            let progress = if frame < 20 { 0u32 }
                else { ((frame - 20) * 100 / 80).min(100) };
            let line_y_start = h / 2 - 40;
            for (si, &(prefix, suffix, target, color)) in stats.iter().enumerate() {
                let y = line_y_start + si * 48;
                let scale = 2usize;
                if target > 0 {
                    let current = (target as u64 * progress as u64 / 100) as u32;
                    let num_str = if current >= 1000 {
                        alloc::format!("{},{:03}", current / 1000, current % 1000)
                    } else {
                        alloc::format!("{}", current)
                    };
                    let full = alloc::format!("{} {}", num_str, suffix);
                    draw_text_centered(&mut buf, w, h, y, &full, color, scale);
                } else if !prefix.is_empty() {
                    let full = alloc::format!("{} {}", prefix, suffix);
                    if frame > 60 + si as u32 * 15 {
                        draw_text_centered(&mut buf, w, h, y, &full, color, scale);
                    }
                } else {
                    let full = alloc::format!("0 {}", suffix);
                    if frame > 60 + si as u32 * 15 {
                        draw_text_centered(&mut buf, w, h, y, &full, color, scale);
                    }
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 9: Feature Grid with Glow Pulse --
    {
        let features: [(&str, &str, u32); 8] = [
            ("Network",   "TCP/IP, DNS, HTTP, DHCP",       0xFF4488FF),
            ("Security",  "TLS 1.3, Ed25519, SHA-256",     0xFFFF4444),
            ("Language",  "TrustLang: full compiler",      0xFF44FF88),
            ("GUI",       "Windows, themes, animations",   0xFFFFAA22),
            ("Storage",   "FAT32, persistence, AHCI",      0xFF8888FF),
            ("Browser",   "HTTP + HTML in the kernel",     0xFF44DDFF),
            ("Games",     "Snake, Tetris, Pong, Chess",    0xFFFF88FF),
            ("Video",     "Built-in video codec",          0xFFFFDD44),
        ];
        let cols = 2usize;
        let cell_w = (w.saturating_sub(120)) / cols;
        let cell_h = 70usize;
        let grid_y = 100usize;
        let grid_x = 40usize;

        for reveal in 0..features.len() {
            for frame in 0..30u32 {
                if let Some(k) = crate::keyboard::try_read_key() {
                    if k == 0x1B { break; }
                }
                bg_stars(&mut buf, w, h, frame + reveal as u32 * 30);
                draw_text_centered(&mut buf, w, h, 30,
                    "All of this in 10 MB:", 0xFFFFFFFF, 3);
                for (fi, &(name, desc, color)) in features.iter().enumerate() {
                    if fi > reveal { break; }
                    let col = fi % cols;
                    let row = fi / cols;
                    let fx = grid_x + col * (cell_w + 40);
                    let fy = grid_y + row * (cell_h + 20);

                    // Glow pulse: bright border that fades after reveal
                    let glow = if fi == reveal && frame < 15 {
                        (15 - frame) * 12
                    } else { 0 };
                    let glow = glow as u32;

                    // Card background
                    draw_rect(&mut buf, w, h, fx, fy, cell_w, cell_h, 0xFF0E0E1E);

                    // Glow border (all 4 sides)
                    if glow > 0 {
                        let gc = 0xFF000000 | (glow.min(255) << 16) | (glow.min(255) << 8) | glow.min(255);
                        draw_rect(&mut buf, w, h, fx.saturating_sub(2), fy.saturating_sub(2),
                            cell_w + 4, 3, gc);
                        draw_rect(&mut buf, w, h, fx.saturating_sub(2), fy + cell_h,
                            cell_w + 4, 3, gc);
                        draw_rect(&mut buf, w, h, fx.saturating_sub(2), fy,
                            3, cell_h, gc);
                        draw_rect(&mut buf, w, h, fx + cell_w, fy,
                            3, cell_h, gc);
                    }

                    // Top color bar
                    draw_rect(&mut buf, w, h, fx, fy, cell_w, 3, color);
                    draw_rect(&mut buf, w, h, fx, fy + cell_h - 1, cell_w, 1, 0xFF222244);
                    draw_text_at(&mut buf, w, h, fx + 10, fy + 12,
                        name, color, 2);
                    draw_text_at(&mut buf, w, h, fx + 10, fy + 42,
                        desc, 0xFFAAAAAA, 1);
                }
                blit_buf(&buf, w, h);
                crate::cpu::tsc::pit_delay_ms(frame_ms);
            }
        }
        crate::cpu::tsc::pit_delay_ms(1500);
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ---------------------------------------------------------------
    //  ACT IV  --  THE PROOF  (bg: circuit-board traces -- technical)
    //  Retention: interactive-feeling animation, pattern interrupt
    // ---------------------------------------------------------------
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT IV", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("The Proof", 0xFF00CC66, 3)],
        50, 30, 6);
    do_fade(&mut buf, w, h, &blit_buf);

    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("When you visit a website,", 0xFFCCCCCC, 2),
          ("this is what happens", 0xFFCCCCCC, 2),
          ("inside TrustOS:", 0xFF00FF88, 2)],
        50, 60, 6);
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Packet journey animation (on circuit bg) --
    {
        let stages: [(&str, u32); 5] = [
            ("App",     0xFF4488FF),
            ("TLS 1.3", 0xFFFF4444),
            ("TCP/IP",  0xFFFFAA22),
            ("Driver",  0xFF44FF88),
            ("Wire",    0xFF8888FF),
        ];
        let n = stages.len();
        let stage_w = (w.saturating_sub(80)) / (n + 1);
        let stage_h = 60usize;
        let lane_y = h / 2 - stage_h / 2;

        for pass in 0..2u32 {
            let label = if pass == 0 { "Sending packet..." }
                        else         { "Response received!" };
            let pkt_color = if pass == 0 { 0xFF00FF88 } else { 0xFF44DDFF };

            for frame in 0..150u32 {
                if let Some(k) = crate::keyboard::try_read_key() {
                    if k == 0x1B { break; }
                }
                bg_circuit(&mut buf, w, h, frame + pass * 150);

                draw_text_centered(&mut buf, w, h, 30,
                    label, 0xFFFFFFFF, 2);

                for (si, &(name, color)) in stages.iter().enumerate() {
                    let sx = 40 + si * stage_w;
                    let bw = stage_w.saturating_sub(15);
                    draw_rect(&mut buf, w, h, sx, lane_y, bw, stage_h,
                              0xFF0E1020);
                    draw_rect(&mut buf, w, h, sx, lane_y, bw, 3, color);
                    draw_rect(&mut buf, w, h, sx, lane_y + stage_h - 1, bw, 1, 0xFF222244);
                    let tx = sx + bw / 2 - name.len() * 4;
                    draw_text_at(&mut buf, w, h, tx, lane_y + 22,
                        name, color, 1);
                    if si < n - 1 {
                        let ax = sx + bw;
                        draw_rect(&mut buf, w, h, ax,
                            lane_y + stage_h / 2 - 1, 15, 3, 0xFF334455);
                        // Arrow head
                        draw_rect(&mut buf, w, h, ax + 12,
                            lane_y + stage_h / 2 - 3, 3, 7, 0xFF556677);
                    }
                }

                // Animated packet with trail
                let progress = (frame * 100 / 120).min(100) as usize;
                let total_travel = (n - 1) * stage_w;
                let pkt_off = if pass == 0 {
                    total_travel * progress / 100
                } else {
                    total_travel - total_travel * progress / 100
                };
                let pkt_x = 40 + pkt_off + stage_w / 2 - 8;
                let pkt_y = lane_y + stage_h + 18;
                // Trail glow
                for trail in 1..6u32 {
                    let tx = if pass == 0 { pkt_x.saturating_sub(trail as usize * 6) }
                             else { pkt_x + trail as usize * 6 };
                    let alpha = (60u32.saturating_sub(trail * 12)).min(255);
                    let tc = 0xFF000000 | (alpha / 4 << 16) | (alpha << 8) | (alpha / 3);
                    draw_rect(&mut buf, w, h, tx, pkt_y + 2, 8, 12, tc);
                }
                draw_rect(&mut buf, w, h, pkt_x, pkt_y, 16, 16, pkt_color);
                draw_text_at(&mut buf, w, h,
                    pkt_x.saturating_sub(16), pkt_y + 20,
                    "packet", 0xFFCCCCCC, 1);

                blit_buf(&buf, w, h);
                crate::cpu::tsc::pit_delay_ms(frame_ms);
            }
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // Post-demo -- emotional beat
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("Every step is visible.", 0xFFFFFFFF, 3),
          ("Every byte is readable.", 0xFFFFFFFF, 3),
          ("", 0xFF000000, 1),
          ("Nothing is hidden.", 0xFF00FF88, 4)],
        50, 80, 4);
    do_fade(&mut buf, w, h, &blit_buf);

    // ---------------------------------------------------------------
    //  ACT V  --  THE FUTURE  (sparkle dissolve + expanding rings)
    // ---------------------------------------------------------------
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT V", 0xFFFFDD88, 5),
          ("", 0xFF000000, 1),
          ("The Future", 0xFFFFAA44, 3)],
        50, 30, 7);
    do_fade(&mut buf, w, h, &blit_buf);

    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("You deserve to understand", 0xFFFFFFFF, 3),
          ("your own machine.", 0xFFFFFFFF, 3)],
        50, 60, 7);
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 10: Sparkle Dissolve -- "Computing is not magic" --
    {
        let fpc = 2u32;
        let text1 = "Computing is not magic.";
        let text2 = "It's math and logic.";
        let total_chars = text1.len() + text2.len();
        let total_frames = total_chars as u32 * fpc + 80;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_sunrise(&mut buf, w, h, frame);
            // Sparkles: random bright points that form geometric shapes
            let sparkle_count = (frame * 2).min(200) as usize;
            for si in 0..sparkle_count {
                let seed = si.wrapping_mul(2654435761).wrapping_add(frame as usize * 131);
                let sx = seed % w;
                let sy = (seed / w) % h;
                // In later frames, sparkles concentrate into geometric shapes
                let constrained = frame > 60;
                let (fx, fy) = if constrained {
                    // Form rectangles/diamond shapes around center
                    let cx = w / 2;
                    let cy = h / 2;
                    let shape_phase = (si % 4);
                    match shape_phase {
                        0 => {  // Top horizontal line
                            let lx = cx.saturating_sub(100) + (si * 3) % 200;
                            (lx, cy.saturating_sub(60))
                        }
                        1 => {  // Bottom horizontal line
                            let lx = cx.saturating_sub(100) + (si * 7) % 200;
                            (lx, cy + 60)
                        }
                        2 => {  // Left vertical line
                            let ly = cy.saturating_sub(60) + (si * 5) % 120;
                            (cx.saturating_sub(100), ly)
                        }
                        _ => {  // Right vertical line
                            let ly = cy.saturating_sub(60) + (si * 11) % 120;
                            (cx + 100, ly)
                        }
                    }
                } else {
                    (sx, sy)
                };
                if fx < w && fy < h {
                    let bright = (100 + (seed % 155)) as u32;
                    buf[fy * w + fx] = 0xFF000000 | (bright << 16) | (bright << 8) | bright;
                    if fx + 1 < w { buf[fy * w + fx + 1] = 0xFF000000 | (bright << 16) | ((bright / 2) << 8); }
                }
            }
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale = 3usize;
            let y1 = h / 4;
            let y2 = y1 + 16 * scale + 16;
            let tw1 = text1.len() * 8 * scale;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale, y1, c, 0xFFCCCCCC, scale);
            }
            if chars_shown > text1.len() {
                let tw2 = text2.len() * 8 * scale;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale, y2, c, 0xFFFFDD88, scale);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Scene 11: Expanding Rings -- "TrustOS proves it." --
    {
        let fpc = 2u32;
        let text = "TrustOS proves it.";
        let type_frames = text.len() as u32 * fpc;
        let ring_frames = 60u32;
        let total_frames = type_frames + ring_frames;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_sunrise(&mut buf, w, h, frame);
            let cx = w / 2;
            let cy = h / 2;
            // Expanding concentric rings (shockwave)
            if frame > type_frames / 2 {
                let ring_progress = frame.saturating_sub(type_frames / 2);
                let num_rings = 5u32;
                for ri in 0..num_rings {
                    let radius = (ring_progress as usize * 4).saturating_sub(ri as usize * 30);
                    if radius == 0 || radius > w { continue; }
                    let bright = (200u32.saturating_sub(ri * 30)).min(255);
                    let r_color = 0xFF000000 | ((bright / 3) << 16) | (bright << 8) | ((bright * 2 / 3).min(255));
                    // Draw ring using distance check in a bounding box
                    let r2_outer = radius * radius;
                    let r_inner = radius.saturating_sub(3);
                    let r2_inner = r_inner * r_inner;
                    let y_start = cy.saturating_sub(radius);
                    let y_end = (cy + radius).min(h);
                    for py in y_start..y_end {
                        let dy = if py >= cy { py - cy } else { cy - py };
                        let dy2 = dy * dy;
                        // Solve for x range where r_inner^2 <= dx^2+dy^2 <= r_outer^2
                        if dy2 > r2_outer { continue; }
                        let dx_max_sq = r2_outer - dy2;
                        let dx_min_sq = if r2_inner > dy2 { r2_inner - dy2 } else { 0 };
                        // Integer sqrt approximation
                        let mut dx_max = 0usize;
                        while (dx_max + 1) * (dx_max + 1) <= dx_max_sq { dx_max += 1; }
                        let mut dx_min = 0usize;
                        while (dx_min + 1) * (dx_min + 1) <= dx_min_sq { dx_min += 1; }
                        // Draw right arc
                        for dx in dx_min..=dx_max {
                            let px = cx + dx;
                            if px < w { buf[py * w + px] = r_color; }
                        }
                        // Draw left arc
                        for dx in dx_min..=dx_max {
                            let px = cx.wrapping_sub(dx);
                            if px < w { buf[py * w + px] = r_color; }
                        }
                    }
                }
            }
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale = 4usize;
            let tw = text.len() * 8 * scale;
            let sx = if tw < w { (w - tw) / 2 } else { 0 };
            let ty = h / 2 - 8 * scale;
            for (i, c) in text.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx + i * 8 * scale, ty, c, 0xFF00FF88, scale);
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ---------------------------------------------------------------
    //  OUTRO  (Matrix rain callback -- signature TrustOS feel)
    // ---------------------------------------------------------------
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("Trust the code.", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("Rust is the reason.", 0xFFFF7744, 3),
          ("", 0xFF000000, 1),
          ("github.com/nathan237/TrustOS", 0xFFCCCCCC, 2)],
        60, 150, 8);
    do_fade(&mut buf, w, h, &blit_buf);

    // -- Cleanup --
    clear_buf(&mut buf);
    blit_buf(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[FILM] TrustOS Film complete");
}

/// TrustView -- open binary file in Ghidra-style desktop viewer
pub(super) fn cmd_trustview(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustView -- Binary Analysis Viewer");
        crate::println_color!(COLOR_GREEN, "Usage: trustview <file>");
        crate::println_color!(COLOR_GREEN, "       tv <file>");
        crate::println!("");
        crate::println!("Opens an ELF binary in the desktop binary viewer.");
        crate::println!("Panels: Navigation | Hex | Disassembly | Info/Xrefs");
        crate::println!("");
        crate::println!("Quick analysis (terminal only):");
        crate::println!("  trustview info <file>  -- Print binary summary");
        return;
    }

    let subcmd = args[0];

    if subcmd == "info" {
        // Terminal-only quick summary
        let path = args.get(1).copied().unwrap_or("");
        if path.is_empty() {
            crate::println_color!(COLOR_RED, "Usage: trustview info <file>");
            return;
        }
        match crate::binary_analysis::analyze_path(path) {
            Ok(bf) => {
                crate::println_color!(COLOR_CYAN, "=== TrustView Analysis: {} ===", path);
                crate::println!("{}", bf.summary());
                crate::println!("");
                // Show detected functions
                crate::println_color!(COLOR_CYAN, "Detected Functions ({}):", bf.xrefs.functions.len());
                for func in bf.xrefs.functions.iter().take(20) {
                    let name = if func.name.is_empty() {
                        alloc::format!("sub_{:X}", func.entry)
                    } else {
                        func.name.clone()
                    };
                    crate::println!("  0x{:08X} {} ({} insns, {} blocks)", 
                        func.entry, name, func.instruction_count, func.basic_blocks);
                }
                if bf.xrefs.functions.len() > 20 {
                    crate::println!("  ... and {} more", bf.xrefs.functions.len() - 20);
                }
            },
            Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
        }
        return;
    }

    // Open in desktop viewer
    let path = subcmd;
    use crate::desktop::DESKTOP;
    let mut desktop = DESKTOP.lock();
    match desktop.open_binary_viewer(path) {
        Ok(id) => {
            crate::println_color!(COLOR_GREEN, "TrustView opened: {} (window #{})", path, id);
        },
        Err(e) => {
            crate::println_color!(COLOR_RED, "Failed to open '{}': {}", path, e);
        }
    }
}

/// Transpile command: analyze and convert Linux binaries to Rust
pub(super) fn cmd_transpile(args: &[&str]) {
    let subcmd = args.get(0).copied().unwrap_or("help");
    
    match subcmd {
        "help" | "-h" | "--help" => {
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println_color!(COLOR_CYAN, "|           TrustOS Binary Transpiler                          |");
            crate::println_color!(COLOR_CYAN, "|       Analyze Linux binaries ? Generate Rust code            |");
            crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
            crate::println!();
            crate::println!("Usage: transpile <subcommand> [file]");
            crate::println!();
            crate::println!("Subcommands:");
            crate::println!("  test             - Run demo with built-in test binary");
            crate::println!("  analyze <file>   - Analyze ELF binary and show disassembly");
            crate::println!("  disasm <file>    - Show disassembly only");
            crate::println!("  rust <file>      - Generate Rust code from binary");
            crate::println!("  strings <file>   - Extract strings from binary");
            crate::println!("  syscalls <file>  - List detected syscalls");
            crate::println!("  scan             - Scan /alpine/bin for binaries to transpile");
            crate::println!("  batch            - Transpile all simple binaries");
            crate::println!("  audit            - Full syscall audit of all Alpine binaries");
            crate::println!("  run <file>       - Execute transpiled binary natively");
            crate::println!("  execute <file>   - Same as run, with verbose output");
            crate::println!();
            crate::println!("Example:");
            crate::println!("  transpile test              # Test with demo binary");
            crate::println!("  transpile run /alpine/bin/true");
            crate::println!("  transpile analyze /alpine/bin/ls");
            crate::println!("  transpile rust /alpine/bin/pwd");
        }
        
        "test" | "demo" => {
            transpile_demo();
        }
        
        "analyze" | "a" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_analyze(path, true, true, true);
        }
        
        "disasm" | "d" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_analyze(path, true, false, false);
        }
        
        "rust" | "r" | "gen" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_analyze(path, false, false, true);
        }
        
        "strings" | "s" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_strings(path);
        }
        
        "syscalls" | "sys" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_syscalls(path);
        }
        
        "scan" => {
            transpile_scan_binaries();
        }
        
        "batch" => {
            transpile_batch();
        }
        
        "run" | "exec" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/true");
            transpile_run(path, false);
        }
        
        "execute" | "x" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/true");
            transpile_run(path, true);
        }

        "audit" | "stats" => {
            transpile_audit_alpine();
        }

        _ => {
            // If first arg looks like a path, analyze it
            if subcmd.starts_with('/') || subcmd.contains('.') {
                transpile_analyze(subcmd, true, true, true);
            } else {
                crate::println_color!(COLOR_RED, "Unknown subcommand: {}", subcmd);
                crate::println!("Use 'transpile help' for usage");
            }
        }
    }
}

fn transpile_analyze(path: &str, show_disasm: bool, show_strings: bool, show_rust: bool) {
    crate::println_color!(COLOR_CYAN, "Analyzing binary: {}", path);
    crate::println!();
    
    // Read file from ramfs
    let data = match crate::ramfs::with_fs(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot read file: {}", path);
            return;
        }
    };
    
    crate::println!("File size: {} bytes ({} KB)", data.len(), data.len() / 1024);
    
    // Analyze ELF
    match crate::transpiler::analyze_elf(&data) {
        Some(analysis) => {
            crate::println_color!(COLOR_GREEN, "ELF analysis successful!");
            crate::println!();
            crate::println!("Entry point: 0x{:x}", analysis.entry_point);
            crate::println!("Functions: {}", analysis.functions.len());
            crate::println!("Syscalls: {:?}", analysis.syscalls_used);
            crate::println!("Strings: {}", analysis.strings.len());
            crate::println!();
            
            if show_disasm {
                if let Some(func) = analysis.functions.first() {
                    crate::println_color!(COLOR_YELLOW, "=== Disassembly ({} instructions) ===", func.instructions.len());
                    let transpiler = crate::transpiler::Transpiler::new(func.instructions.clone());
                    crate::println!("{}", transpiler.generate_listing());
                }
            }
            
            if show_strings && !analysis.strings.is_empty() {
                crate::println_color!(COLOR_YELLOW, "=== Strings (first 20) ===");
                for (addr, s) in analysis.strings.iter().take(20) {
                    crate::println!("0x{:06x}: \"{}\"", addr, s);
                }
                crate::println!();
            }
            
            if show_rust {
                crate::println_color!(COLOR_YELLOW, "=== Generated Rust Code ===");
                crate::println!("{}", analysis.rust_code);
            }
        }
        None => {
            crate::println_color!(COLOR_RED, "Not a valid ELF binary");
        }
    }
}

fn transpile_strings(path: &str) {
    let data = match crate::ramfs::with_fs(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot read file: {}", path);
            return;
        }
    };
    
    // Extract strings manually (min 4 chars)
    let mut strings = alloc::vec::Vec::new();
    let mut current = alloc::string::String::new();
    let mut start = 0usize;
    
    for (i, &b) in data.iter().enumerate() {
        if b >= 0x20 && b < 0x7F {
            if current.is_empty() {
                start = i;
            }
            current.push(b as char);
        } else {
            if current.len() >= 4 {
                strings.push((start, current.clone()));
            }
            current.clear();
        }
    }
    
    crate::println_color!(COLOR_CYAN, "Strings in {}: {} found", path, strings.len());
    crate::println!();
    
    for (addr, s) in strings.iter() {
        // Skip binary garbage
        if s.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
            crate::println!("0x{:06x}: {}", addr, s);
        }
    }
}

fn transpile_syscalls(path: &str) {
    let data = match crate::ramfs::with_fs(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot read file: {}", path);
            return;
        }
    };
    
    match crate::transpiler::analyze_elf(&data) {
        Some(analysis) => {
            crate::println_color!(COLOR_CYAN, "Syscalls in {}", path);
            crate::println!();
            
            for func in &analysis.functions {
                if !func.syscalls.is_empty() {
                    crate::println!("Function {} @ 0x{:x}:", func.name, func.address);
                    for sc in &func.syscalls {
                        crate::println!("  0x{:x}: {} (#{})", sc.address, sc.name, sc.number);
                    }
                }
            }
            
            crate::println!();
            crate::println!("Summary: {:?}", analysis.syscalls_used);
        }
        None => {
            crate::println_color!(COLOR_RED, "Not a valid ELF binary");
        }
    }
}

fn transpile_scan_binaries() {
    crate::println_color!(COLOR_CYAN, "Scanning /alpine/bin for binaries...");
    crate::println!();
    
    let entries = match crate::ramfs::with_fs(|fs| fs.ls(Some("/alpine/bin"))) {
        Ok(e) => e,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot access /alpine/bin - run 'alpine test' first");
            return;
        }
    };
    
    let mut simple = alloc::vec::Vec::new();
    let mut complex = alloc::vec::Vec::new();
    
    for (name, _, _size) in entries {
        let path = alloc::format!("/alpine/bin/{}", name);
        
        if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::analyze_elf(&data) {
                let syscall_count = analysis.syscalls_used.len();
                let instr_count = analysis.functions.first().map(|f| f.instructions.len()).unwrap_or(0);
                
                if syscall_count <= 3 && instr_count < 100 {
                    simple.push((name.clone(), syscall_count, instr_count));
                } else {
                    complex.push((name.clone(), syscall_count, instr_count));
                }
            }
        }
    }
    
    crate::println_color!(COLOR_GREEN, "Simple binaries ({} - easy to transpile):", simple.len());
    for (name, sc, instr) in &simple {
        crate::println!("  {} - {} syscalls, {} instructions", name, sc, instr);
    }
    
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "Complex binaries ({} - need more work):", complex.len());
    for (name, sc, instr) in complex.iter().take(10) {
        crate::println!("  {} - {} syscalls, {} instructions", name, sc, instr);
    }
    if complex.len() > 10 {
        crate::println!("  ... and {} more", complex.len() - 10);
    }
}

fn transpile_batch() {
    crate::println_color!(COLOR_CYAN, "Batch transpiling simple binaries...");
    crate::println!();
    
    // For now, just demonstrate the concept
    let simple_bins = ["true", "false", "pwd", "whoami", "hostname", "uname", "echo", "yes"];
    
    for name in &simple_bins {
        let path = alloc::format!("/alpine/bin/{}", name);
        
        if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::analyze_elf(&data) {
                crate::println_color!(COLOR_GREEN, "=== {} ===", name);
                crate::println!("Syscalls: {:?}", analysis.syscalls_used);
                crate::println!();
                crate::println!("{}", analysis.rust_code);
                crate::println!();
            } else {
                crate::println_color!(COLOR_YELLOW, "{}: not found or not ELF", name);
            }
        } else {
            crate::println_color!(COLOR_YELLOW, "{}: not available", name);
        }
    }
}

/// Audit all Alpine binaries and show syscall statistics
fn transpile_audit_alpine() {
    use alloc::collections::BTreeMap;
    
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|         TrustOS Transpiler - Alpine Syscall Audit            |");
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println!();
    
    let entries = match crate::ramfs::with_fs(|fs| fs.ls(Some("/alpine/bin"))) {
        Ok(e) => e,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot access /alpine/bin - run 'linux extract' first");
            return;
        }
    };
    
    // Collect all syscalls with their frequency
    let mut syscall_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut syscall_numbers: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut binary_count = 0;
    let mut elf_count = 0;
    let mut supported_count = 0;
    let mut total_instructions = 0usize;
    
    crate::println!("Scanning {} files...", entries.len());
    crate::println!();
    
    for (name, _, _) in &entries {
        let path = alloc::format!("/alpine/bin/{}", name);
        binary_count += 1;
        
        if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::analyze_elf(&data) {
                elf_count += 1;
                
                // Count instructions
                for func in &analysis.functions {
                    total_instructions += func.instructions.len();
                    
                    for sc in &func.syscalls {
                        *syscall_counts.entry(sc.name).or_insert(0) += 1;
                        syscall_numbers.insert(sc.name, sc.number);
                    }
                }
                
                // Check if fully supported
                let all_supported = analysis.syscalls_used.iter().all(|sc| {
                    matches!(*sc, "exit" | "exit_group" | "write" | "read" | "open" | "close" |
                            "getcwd" | "uname" | "getpid" | "getuid" | "getgid" | "geteuid" | "getegid")
                });
                if all_supported && !analysis.syscalls_used.is_empty() {
                    supported_count += 1;
                }
            }
        }
    }
    
    crate::println_color!(COLOR_GREEN, "--- Statistics ---");
    crate::println!("Files scanned:      {}", binary_count);
    crate::println!("Valid ELF binaries: {}", elf_count);
    crate::println!("Fully supported:    {}", supported_count);
    crate::println!("Total instructions: {}", total_instructions);
    crate::println!();
    
    // Sort syscalls by frequency
    let mut sorted: Vec<_> = syscall_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    
    crate::println_color!(COLOR_CYAN, "--- Syscalls by Frequency ---");
    crate::println!("{:<20} {:>8} {:>8} {}", "Syscall", "Count", "Number", "Status");
    crate::println!("{}", "-".repeat(50));
    
    for (name, count) in &sorted {
        let num = syscall_numbers.get(*name).copied().unwrap_or(0);
        let level = crate::transpiler::syscall_support_level(num);
        let status = match level {
            "Full" => "Full",
            "Partial" => "Partial",
            "Stub" => "Stub",
            _ => "Missing",
        };
        crate::println!("{:<20} {:>8} {:>8} {}", name, count, num, status);
    }
    
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "--- Missing Syscalls (need implementation) ---");
    let missing: Vec<_> = sorted.iter()
        .filter(|(name, _)| {
            let num = syscall_numbers.get(*name).copied().unwrap_or(999);
            crate::transpiler::syscall_support_level(num) == "None"
        })
        .collect();
    
    for (name, count) in &missing {
        let num = syscall_numbers.get(*name).copied().unwrap_or(0);
        crate::println!("  {} (#{}) - used {} times", name, num, count);
    }
    
    if missing.is_empty() {
        crate::println_color!(COLOR_GREEN, "  All syscalls are at least partially implemented!");
    }
    
    crate::println!();
    crate::println_color!(COLOR_CYAN, "--- Recommendation ---");
    crate::println!("To improve transpiler coverage, implement these syscalls in order:");
    let priority: Vec<_> = missing.iter().take(5).collect();
    for (i, (name, count)) in priority.iter().enumerate() {
        crate::println!("  {}. {} (used {} times)", i + 1, name, count);
    }
}

/// Create test ELF binaries silently (used by alpine test)
pub(super) fn create_test_binaries_silent() {
    // Create directory structure
    let _ = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/alpine");
        let _ = fs.mkdir("/alpine/bin");
        Ok::<(), ()>(())
    });
    
    fn make_elf(code: &[u8]) -> alloc::vec::Vec<u8> {
        let mut elf = alloc::vec![
            0x7fu8, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x3e, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x38, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        elf.extend_from_slice(code);
        while elf.len() < 256 { elf.push(0); }
        elf
    }
    
    let binaries: [(&str, &[u8]); 7] = [
        ("true", &[0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("false", &[0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, 0x0f, 0x05]),
        ("getpid", &[0x48, 0xc7, 0xc0, 0x27, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("getuid", &[0x48, 0xc7, 0xc0, 0x66, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("uname", &[0x48, 0xc7, 0xc0, 0x3f, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("echo", &[0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, 0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, 0x48, 0x31, 0xf6, 0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("pwd", &[0x48, 0xc7, 0xc0, 0x4f, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x48, 0x31, 0xf6, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
    ];
    
    let mut created = 0;
    for (name, code) in &binaries {
        let elf = make_elf(code);
        let path = alloc::format!("/alpine/bin/{}", name);
        if crate::ramfs::with_fs(|fs| { let _ = fs.touch(&path); fs.write_file(&path, &elf) }).is_ok() {
            created += 1;
        }
    }
    crate::println_color!(COLOR_GREEN, "      Created {} binaries", created);
}

/// Create test ELF binaries in /alpine/bin for transpiler testing
pub(super) fn create_test_binaries() {
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|       Creating Test Binaries for Transpiler                  |");
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println!();
    
    // Create directory structure
    let _ = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/alpine");
        let _ = fs.mkdir("/alpine/bin");
        Ok::<(), ()>(())
    });
    
    // Use the same ELF template as transpile_demo
    fn make_elf(code: &[u8]) -> alloc::vec::Vec<u8> {
        let mut elf = alloc::vec![
            // ELF header (64 bytes)
            0x7fu8, 0x45, 0x4c, 0x46,  // Magic: \x7fELF
            0x02, 0x01, 0x01, 0x00,     // 64-bit, little-endian
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Padding
            0x02, 0x00,                 // Executable
            0x3e, 0x00,                 // x86_64
            0x01, 0x00, 0x00, 0x00,     // Version
            0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Entry: 0x400078
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // PH offset: 64
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // SH offset
            0x00, 0x00, 0x00, 0x00,     // Flags
            0x40, 0x00,                 // ELF header size: 64
            0x38, 0x00,                 // PH entry size: 56
            0x01, 0x00,                 // PH count: 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // SH stuff
            
            // Program header (56 bytes)
            0x01, 0x00, 0x00, 0x00,     // PT_LOAD
            0x05, 0x00, 0x00, 0x00,     // Flags: R+X
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Offset
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Vaddr: 0x400000
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Paddr
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Filesz
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Memsz
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Align
        ];
        elf.extend_from_slice(code);
        while elf.len() < 256 {
            elf.push(0);
        }
        elf
    }
    
    // Different syscall programs
    let binaries: [(&str, &[u8], &str); 7] = [
        ("true", &[
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60
            0x48, 0x31, 0xff,                          // xor rdi, rdi
            0x0f, 0x05,                                // syscall
        ], "exit(0)"),
        
        ("false", &[
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  // mov rdi, 1
            0x0f, 0x05,                                // syscall
        ], "exit(1)"),
        
        ("getpid", &[
            0x48, 0xc7, 0xc0, 0x27, 0x00, 0x00, 0x00,  // mov rax, 39
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getpid+exit"),
        
        ("getuid", &[
            0x48, 0xc7, 0xc0, 0x66, 0x00, 0x00, 0x00,  // mov rax, 102
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getuid+exit"),
        
        ("uname", &[
            0x48, 0xc7, 0xc0, 0x3f, 0x00, 0x00, 0x00,  // mov rax, 63
            0x48, 0x31, 0xff,
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "uname+exit"),
        
        ("echo", &[
            0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,  // mov rax, 1 (write)
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  // mov rdi, 1
            0x48, 0x31, 0xf6,                          // xor rsi, rsi
            0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00,  // mov rdx, 5
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "write+exit"),
        
        ("pwd", &[
            0x48, 0xc7, 0xc0, 0x4f, 0x00, 0x00, 0x00,  // mov rax, 79 (getcwd)
            0x48, 0x31, 0xff,
            0x48, 0x31, 0xf6,
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getcwd+exit"),
    ];
    
    let mut created = 0;
    for (name, code, desc) in &binaries {
        let elf = make_elf(code);
        let path = alloc::format!("/alpine/bin/{}", name);
        
        let result = crate::ramfs::with_fs(|fs| {
            let _ = fs.touch(&path);
            fs.write_file(&path, &elf)
        });
        
        match result {
            Ok(_) => {
                crate::println_color!(COLOR_GREEN, "? {} - {}", name, desc);
                created += 1;
            }
            Err(_) => {
                crate::println_color!(COLOR_RED, "? {} - failed", name);
            }
        }
    }
    
    crate::println!();
    crate::println_color!(COLOR_GREEN, "Created {} test binaries in /alpine/bin", created);
    crate::println!();
    crate::println!("Now run:");
    crate::println!("  transpile audit       - Analyze all syscalls");
    crate::println!("  transpile run /alpine/bin/true");
    crate::println!("  transpile analyze /alpine/bin/echo");
}

/// Demo the transpiler with a built-in test binary (no external files needed)
fn transpile_demo() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "+--------------------------------------------------------------+");
    crate::println_color!(COLOR_BRIGHT_GREEN, "|         TrustOS Transpiler Demo - Built-in Test              |");
    crate::println_color!(COLOR_BRIGHT_GREEN, "+--------------------------------------------------------------+");
    crate::println!();
    
    // Create a minimal x86_64 ELF binary that does:
    //   mov rax, 60    ; sys_exit
    //   xor rdi, rdi   ; exit code 0  
    //   syscall
    // This is the simplest possible Linux program
    
    crate::println_color!(COLOR_CYAN, "Creating test binary: exit(0) program");
    crate::println!();
    
    // Minimal ELF64 executable that calls exit(0)
    // Header (64 bytes) + Program Header (56 bytes) + Code (12 bytes) = 132 bytes
    #[rustfmt::skip]
    let demo_elf: &[u8] = &[
        // ELF Header (64 bytes)
        0x7F, b'E', b'L', b'F',  // Magic
        0x02,                     // 64-bit
        0x01,                     // Little endian
        0x01,                     // ELF version
        0x00,                     // System V ABI
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Padding (8 bytes)
        0x02, 0x00,               // Executable
        0x3E, 0x00,               // x86-64
        0x01, 0x00, 0x00, 0x00,   // ELF version
        0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Entry point: 0x400078
        0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Program header offset: 64
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Section header offset
        0x00, 0x00, 0x00, 0x00,   // Flags
        0x40, 0x00,               // ELF header size: 64
        0x38, 0x00,               // Program header size: 56
        0x01, 0x00,               // Number of program headers: 1
        0x00, 0x00,               // Section header size
        0x00, 0x00,               // Number of section headers
        0x00, 0x00,               // Section name string table index
        
        // Program Header (56 bytes) - PT_LOAD
        0x01, 0x00, 0x00, 0x00,   // PT_LOAD
        0x05, 0x00, 0x00, 0x00,   // Flags: R+X
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Offset in file: 0
        0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Virtual address: 0x400000
        0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Physical address: 0x400000
        0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // File size: 132 bytes
        0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Memory size: 132 bytes
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Alignment: 0x1000
        
        // Code section (offset 0x78 = 120, 12 bytes)
        // _start:
        //   mov rax, 60       ; 48 c7 c0 3c 00 00 00
        //   xor rdi, rdi      ; 48 31 ff  
        //   syscall           ; 0f 05
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00,  // mov rax, 60 (sys_exit)
        0x48, 0x31, 0xFF,                          // xor rdi, rdi (exit code 0)
        0x0F, 0x05,                                // syscall
    ];
    
    crate::println!("Demo binary: {} bytes", demo_elf.len());
    crate::println!("Code: mov rax, 60; xor rdi, rdi; syscall");
    crate::println!();
    
    // Analyze it
    match crate::transpiler::analyze_elf(&demo_elf) {
        Some(analysis) => {
            crate::println_color!(COLOR_GREEN, "? ELF Analysis Successful!");
            crate::println!();
            
            crate::println_color!(COLOR_YELLOW, "--- Binary Info ---");
            crate::println!("Entry point:  0x{:x}", analysis.entry_point);
            crate::println!("Functions:    {}", analysis.functions.len());
            crate::println!("Syscalls:     {:?}", analysis.syscalls_used);
            crate::println!();
            
            // Show disassembly
            if let Some(func) = analysis.functions.first() {
                crate::println_color!(COLOR_YELLOW, "--- Disassembly ({} instructions) ---", func.instructions.len());
                let transpiler = crate::transpiler::Transpiler::new(func.instructions.clone());
                crate::println!("{}", transpiler.generate_listing());
            }
            
            // Show generated Rust
            crate::println_color!(COLOR_YELLOW, "--- Generated Rust Code ---");
            crate::println!("{}", analysis.rust_code);
            
            crate::println_color!(COLOR_BRIGHT_GREEN, "");
            crate::println_color!(COLOR_BRIGHT_GREEN, "? Transpiler test PASSED!");
            crate::println!();
            crate::println!("The transpiler successfully:");
            crate::println!("  1. Parsed ELF64 header");
            crate::println!("  2. Found executable segment");
            crate::println!("  3. Disassembled x86_64 code");
            crate::println!("  4. Detected syscall (sys_exit)");
            crate::println!("  5. Generated equivalent Rust code");
        }
        None => {
            crate::println_color!(COLOR_RED, "? Failed to analyze demo binary");
        }
    }
    
    // Also save to ramfs for further testing
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Saving demo binary to /tmp/demo_exit...");
    let save_result = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/tmp");
        let _ = fs.touch("/tmp/demo_exit"); // Create file first
        fs.write_file("/tmp/demo_exit", demo_elf)
    });
    match save_result {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "? Saved! You can now run:");
            crate::println!("  transpile analyze /tmp/demo_exit");
            crate::println!("  transpile rust /tmp/demo_exit");
        }
        Err(_) => {
            crate::println_color!(COLOR_YELLOW, "Could not save demo binary");
        }
    }
}

/// Execute a transpiled binary directly in TrustOS
fn transpile_run(path: &str, verbose: bool) {
    use crate::transpiler::{analyze_elf, BinaryType};
    
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|           TrustOS Transpiler - Execute Binary                |");
    crate::println_color!(COLOR_CYAN, "+--------------------------------------------------------------+");
    crate::println!();
    crate::println!("Binary: {}", path);
    
    // Read the binary
    let data = match super::network::read_file_bytes(path) {
        Some(d) => d,
        None => {
            crate::println_color!(COLOR_RED, "Error: Could not read file");
            return;
        }
    };
    
    if verbose {
        crate::println!("Size: {} bytes", data.len());
    }
    
    // Analyze it
    let analysis = match analyze_elf(&data) {
        Some(a) => a,
        None => {
            crate::println_color!(COLOR_RED, "Error: Not a valid ELF binary");
            return;
        }
    };
    
    if verbose {
        crate::println!("Entry point: 0x{:x}", analysis.entry_point);
        crate::println!("Syscalls: {:?}", analysis.syscalls_used);
    }
    
    crate::println!();
    crate::println_color!(COLOR_GREEN, "--- Executing transpiled binary ---");
    crate::println!();
    
    // Execute based on detected type and syscalls
    let exit_code = execute_transpiled_binary(&analysis);
    
    crate::println!();
    crate::println_color!(COLOR_CYAN, "---------------------------------------------------------------");
    crate::println!("Exit code: {}", exit_code);
}

/// Execute a transpiled binary based on its analysis
fn execute_transpiled_binary(analysis: &crate::transpiler::BinaryAnalysis) -> i32 {
    // Get syscalls from the first function
    let syscalls = if let Some(func) = analysis.functions.first() {
        &func.syscalls
    } else {
        crate::println_color!(COLOR_RED, "No functions found in binary");
        return 1;
    };
    
    // Execute syscalls in sequence
    for syscall in syscalls {
        match syscall.name {
            "exit" | "exit_group" => {
                let code = syscall.args.get(0).copied().unwrap_or(0) as i32;
                return code;
            }
            "write" => {
                let fd = syscall.args.get(0).copied().unwrap_or(1);
                if fd == 1 || fd == 2 {
                    // Writing to stdout/stderr
                    // In a real implementation, we'd need to extract the actual string
                    // For now, just indicate a write happened
                    crate::print!("[write to fd {}]", fd);
                }
            }
            "getcwd" => {
                crate::println!("/");
            }
            "uname" => {
                crate::println!("TrustOS trustos 1.0.0-transpiled #1 SMP x86_64");
            }
            "getpid" => {
                crate::println!("1");
            }
            "getuid" | "geteuid" => {
                crate::println!("0");
            }
            "getgid" | "getegid" => {
                crate::println!("0");
            }
            _ => {
                crate::println!("[syscall: {} not implemented]", syscall.name);
            }
        }
    }
    
    // Default exit code if no exit syscall was found
    0
}

/// TrustVideo command: video codec, player, and demo animations
pub(super) fn cmd_video(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "demo" => {
            let effect = args.get(1).copied().unwrap_or("plasma");
            let fps = 30u64;
            let frame_ms = 1000 / fps;

            crate::println!("=== TrustVideo Demo: {} ===", effect);
            crate::println!("Rendering in real-time @ {}fps", fps);
            crate::println!("Press Q or ESC to stop");

            // Real-time streaming render -- no file accumulation
            let sw = crate::framebuffer::width();
            let sh = crate::framebuffer::height();
            let vw = sw.min(640) as u16;
            let vh = sh.min(480) as u16;

            match effect {
                "plasma" | "fire" | "matrix" | "shader" => {
                    crate::video::player::render_realtime(effect, vw, vh, fps as u16);
                }
                _ => {
                    crate::println!("Unknown effect: {}. Available: plasma, fire, matrix, shader", effect);
                }
            }
        }

        "play" => {
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: video play <file.tv>"); return; }
            };

            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/home/{}", filename)
            };

            match crate::vfs::read_file(&path) {
                Ok(data) => {
                    crate::println!("Playing {}...", filename);
                    let mut player = crate::video::player::VideoPlayer::new();
                    match player.play_data(data) {
                        Ok(msg) => crate::println!("{}", msg),
                        Err(e) => crate::println!("Error: {}", e),
                    }
                }
                Err(_) => crate::println!("File not found: {}", path),
            }
        }

        "info" => {
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: video info <file.tv>"); return; }
            };

            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/home/{}", filename)
            };

            match crate::vfs::read_file(&path) {
                Ok(data) => {
                    if let Some(hdr) = crate::video::codec::TvHeader::from_bytes(&data) {
                        crate::println!("=== TrustVideo Info ===");
                        crate::println!("  Format:     TrustVideo v{}", hdr.version);
                        crate::println!("  Resolution: {}x{}", hdr.width, hdr.height);
                        crate::println!("  FPS:        {}", hdr.fps);
                        crate::println!("  Frames:     {}", hdr.frame_count);
                        crate::println!("  Duration:   {:.1}s", hdr.frame_count as f64 / hdr.fps as f64);
                        crate::println!("  Keyframe:   every {} frames", hdr.keyframe_interval);
                        crate::println!("  File size:  {} bytes ({} KB)", data.len(), data.len() / 1024);
                        let raw_size = hdr.width as usize * hdr.height as usize * 4 * hdr.frame_count as usize;
                        if raw_size > 0 {
                            let ratio = raw_size as f64 / data.len() as f64;
                            crate::println!("  Compression: {:.1}x (raw would be {} KB)", ratio, raw_size / 1024);
                        }
                    } else {
                        crate::println!("Not a valid TrustVideo file");
                    }
                }
                Err(_) => crate::println!("File not found: {}", path),
            }
        }

        _ => {
            crate::println!("TrustVideo -- Custom video codec for TrustOS");
            crate::println!("");
            crate::println!("Usage: video <command> [args]");
            crate::println!("");
            crate::println!("Commands:");
            crate::println!("  demo [effect]  Generate & play a demo animation");
            crate::println!("                 Effects: plasma, fire, matrix, shader");
            crate::println!("  play <file>    Play a .tv video file");
            crate::println!("  info <file>    Show video file info");
            crate::println!("");
            crate::println!("Controls during playback:");
            crate::println!("  Q / ESC        Stop playback");
            crate::println!("  Space          Pause / Resume");
        }
    }
}

/// TrustLab -- OS introspection laboratory
pub(super) fn cmd_lab(args: &[&str]) {
    match args.first().copied().unwrap_or("open") {
        "open" | "" => {
            crate::println!("\x01G[TrustLab]\x01W Opening OS Introspection Laboratory...");
            crate::println!("  6-panel real-time kernel dashboard");
            crate::println!("  Use Tab to cycle panels, arrow keys to navigate");
            crate::desktop::DESKTOP.lock().open_lab_mode();
        }
        "help" | "--help" | "-h" => {
            crate::println!("\x01B+------------------------------------------+");
            crate::println!("\x01B|  \x01WTrustLab -- OS Introspection Laboratory\x01B  |");
            crate::println!("\x01B+------------------------------------------+");
            crate::println!("");
            crate::println!("\x01YUsage:\x01W lab [command]");
            crate::println!("");
            crate::println!("\x01YCommands:\x01W");
            crate::println!("  open     Open the Lab window (default)");
            crate::println!("  help     Show this help");
            crate::println!("");
            crate::println!("\x01YPanels:\x01W");
            crate::println!("  1. \x01GHardware Status\x01W  -- CPU, memory, IRQ stats");
            crate::println!("  2. \x01YLive Kernel Trace\x01W -- Real-time event log");
            crate::println!("  3. \x01BCommand Guide\x01W    -- Searchable command reference");
            crate::println!("  4. \x01CFile System Tree\x01W -- VFS browser");
            crate::println!("  5. \x01MTrustLang Editor\x01W -- Inline code editor");
            crate::println!("  6. \x01YEvent Stream\x01W     -- Filtered event feed");
            crate::println!("");
            crate::println!("\x01YControls:\x01W");
            crate::println!("  Tab        Cycle focused panel");
            crate::println!("  Arrows     Navigate within panel");
            crate::println!("  PgUp/Down  Scroll faster");
            crate::println!("  F5         Run code (in editor panel)");
        }
        other => {
            crate::println!("\x01RUnknown lab command: {}\x01W", other);
            crate::println!("Type 'lab help' for usage.");
        }
    }
}