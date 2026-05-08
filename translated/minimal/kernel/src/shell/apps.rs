





use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AX_, D_, A_, C_, R_, CF_, DM_, K_};






pub(super) fn koe(args: &[&str]) {
    use crate::wayland::terminal;
    
    let je = args.get(0).copied().unwrap_or("launch");
    
    match je {
        "launch" | "start" | "run" => {
            crate::n!(C_, "+--------------------------------------------------------------+");
            crate::n!(C_, "|           TrustOS Graphical Terminal - Matrix Edition        |");
            crate::n!(C_, "+--------------------------------------------------------------+");
            crate::println!();
            
            
            let _ = crate::wayland::init();
            
            
            let (screen_w, screen_h) = crate::framebuffer::kv();
            
            
            let ebl = (screen_w * 80 / 100) & !7; 
            let ebk = (screen_h * 80 / 100) & !15; 
            
            crate::println!("Initializing terminal {}x{} pixels...", ebl, ebk);
            
            
            match terminal::init(ebl, ebk) {
                Ok(()) => {
                    crate::n!(B_, "[OK] Graphics terminal initialized");
                }
                Err(e) => {
                    crate::n!(D_, "[WARN] {}", e);
                }
            }
            
            
            crate::wayland::bjz(|compositor| {
                let avh = compositor.create_surface();
                
                
                if let Some(buffer) = terminal::render() {
                    let (w, h) = terminal::cyt().unwrap_or((ebl, ebk));
                    
                    
                    if let Some(surface) = compositor.surfaces.get_mut(&avh) {
                        surface.attach(buffer, w, h);
                        surface.set_title("TrustOS Terminal");
                        let x = (screen_w - w) / 2;
                        let y = (screen_h - h) / 2;
                        surface.set_position(x as i32, y as i32);
                        surface.make_toplevel();
                        surface.commit();
                    }
                }
                
                crate::n!(B_, "[OK] Terminal surface created (ID: {})", avh);
            });
            
            
            crate::wayland::cho();
            
            crate::println!();
            crate::n!(B_, "Terminal launched!");
            crate::println!("Use 'gterm demo' for an interactive demo.");
            crate::println!("Use 'gterm fullscreen' for fullscreen mode.");
        },
        
        "demo" => {
            crate::n!(C_, "Starting interactive graphical terminal demo...");
            crate::println!();
            
            
            let _ = crate::wayland::init();
            
            let (screen_w, screen_h) = crate::framebuffer::kv();
            let ebl = (screen_w * 85 / 100) & !7;
            let ebk = (screen_h * 85 / 100) & !15;
            
            
            let _ = terminal::init(ebl, ebk);
            
            
            let avh = crate::wayland::bjz(|compositor| {
                let id = compositor.create_surface();
                
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::cyt() {
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
            
            
            crate::wayland::cho();
            
            
            terminal::write("\x1b[2J\x1b[H"); 
            terminal::write("\x1b[1;32m+----------------------------------------------------------+\r\n");
            terminal::write("|  \x1b[1;97mTrustOS\x1b[1;32m Graphical Terminal Demo                       |\r\n");
            terminal::write("|  Matrix Edition v1.0                                     |\r\n");
            terminal::write("+----------------------------------------------------------+\r\n");
            terminal::write("\x1b[0;32m\r\n");
            terminal::write("Type text and press Enter. Press ESC to exit.\r\n\r\n");
            terminal::write("\x1b[1;32m$ \x1b[0;32m");
            
            
            crate::wayland::bjz(|compositor| {
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::cyt() {
                        if let Some(surface) = compositor.surfaces.get_mut(&avh) {
                            surface.attach(buffer, w, h);
                            surface.commit();
                        }
                    }
                }
            });
            crate::wayland::cho();
            
            
            let mut input_buffer = alloc::string::String::new();
            loop {
                
                if crate::shell::cbc() { break; }
                
                if let Some(key) = crate::keyboard::ya() {
                    let c = key as char;
                    match key {
                        0x1b => {
                            
                            break;
                        }
                        0x0D | 0x0A => {
                            
                            terminal::write("\r\n");
                            
                            if !input_buffer.is_empty() {
                                
                                let fa = alloc::format!("\x1b[0;36mYou typed: \x1b[1;97m{}\x1b[0;32m\r\n", input_buffer);
                                terminal::write(&fa);
                                input_buffer.clear();
                            }
                            
                            terminal::write("\x1b[1;32m$ \x1b[0;32m");
                        }
                        0x08 | 0x7F => {
                            
                            if !input_buffer.is_empty() {
                                input_buffer.pop();
                                terminal::write("\x08 \x08");
                            }
                        }
                        k if k >= 0x20 && k < 0x7F => {
                            
                            input_buffer.push(c);
                            let j = alloc::format!("{}", c);
                            terminal::write(&j);
                        }
                        _ => {}
                    }
                    
                    
                    crate::wayland::bjz(|compositor| {
                        if let Some(buffer) = terminal::render() {
                            if let Some((w, h)) = terminal::cyt() {
                                if let Some(surface) = compositor.surfaces.get_mut(&avh) {
                                    surface.attach(buffer, w, h);
                                    surface.commit();
                                }
                            }
                        }
                    });
                    crate::wayland::cho();
                }
                
                
                for _ in 0..1000 { core::hint::spin_loop(); }
            }
            
            
            crate::framebuffer::clear();
            crate::n!(B_, "Demo ended.");
        },
        
        "fullscreen" | "fs" => {
            crate::n!(C_, "Launching fullscreen terminal...");
            
            
            let (screen_w, screen_h) = crate::framebuffer::kv();
            
            let _ = crate::wayland::init();
            let _ = terminal::init(screen_w, screen_h);
            
            
            crate::wayland::bjz(|compositor| {
                let id = compositor.create_surface();
                
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::cyt() {
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
            
            crate::wayland::cho();
            crate::n!(B_, "[OK] Fullscreen terminal active");
        },
        
        "test" => {
            
            crate::n!(C_, "Testing graphical terminal ANSI support...");
            
            let _ = crate::wayland::init();
            let (w, h) = crate::framebuffer::kv();
            let _ = terminal::init(w * 70 / 100, h * 70 / 100);
            
            
            terminal::write("\x1b[2J\x1b[H"); 
            terminal::write("\x1b[1;32m=== ANSI Escape Code Test ===\x1b[0m\r\n\r\n");
            
            
            terminal::write("\x1b[31mRed \x1b[32mGreen \x1b[33mYellow \x1b[34mBlue \x1b[35mMagenta \x1b[36mCyan\x1b[0m\r\n");
            terminal::write("\x1b[91mBright Red \x1b[92mBright Green \x1b[93mBright Yellow\x1b[0m\r\n\r\n");
            
            
            terminal::write("\x1b[1mBold\x1b[0m \x1b[2mDim\x1b[0m \x1b[4mUnderline\x1b[0m \x1b[7mReverse\x1b[0m\r\n\r\n");
            
            
            terminal::write("\x1b[32m");
            for i in 0..5 {
                for _ in 0..60 {
                    let c = ((i * 7 + 33) % 94 + 33) as u8 as char;
                    let j = alloc::format!("{}", c);
                    terminal::write(&j);
                }
                terminal::write("\r\n");
            }
            terminal::write("\x1b[0m\r\n");
            
            terminal::write("\x1b[1;97mTest complete!\x1b[0m\r\n");
            
            
            crate::wayland::bjz(|compositor| {
                let id = compositor.create_surface();
                if let Some(buffer) = terminal::render() {
                    if let Some((gr, bwj)) = terminal::cyt() {
                        if let Some(surface) = compositor.surfaces.get_mut(&id) {
                            surface.attach(buffer, gr, bwj);
                            surface.set_title("ANSI Test");
                            surface.set_position(
                                ((w - gr) / 2) as i32,
                                ((h - bwj) / 2) as i32
                            );
                            surface.make_toplevel();
                            surface.commit();
                        }
                    }
                }
            });
            crate::wayland::cho();
            
            crate::println!();
            crate::n!(B_, "Press any key to close...");
            loop {
                if crate::keyboard::ya().is_some() {
                    break;
                }
            }
            crate::framebuffer::clear();
        },
        
        _ => {
            crate::n!(C_, "+--------------------------------------------------------------+");
            crate::n!(C_, "|       TrustOS Graphical Terminal - Matrix Edition           |");
            crate::n!(C_, "+--------------------------------------------------------------+");
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


pub(super) fn kua(args: &[&str]) {
    let je = args.get(0).copied().unwrap_or("help");
    
    match je {
        "init" | "start" => {
            crate::n!(C_, "+--------------------------------------------------------------+");
            crate::n!(C_, "|            TrustOS Wayland Compositor                        |");
            crate::n!(C_, "+--------------------------------------------------------------+");
            crate::println!();
            
            match crate::wayland::init() {
                Ok(()) => {
                    crate::n!(B_, "[OK] Wayland compositor initialized");
                    
                    
                    let (width, height) = crate::framebuffer::kv();
                    crate::println!("     Display: {}x{}", width, height);
                    crate::println!();
                    crate::println!("Available globals:");
                    for global in crate::wayland::protocol::fys() {
                        crate::println!("  * {} v{}", global.interface, global.version);
                    }
                }
                Err(e) => {
                    crate::n!(A_, "[ERROR] {}", e);
                }
            }
        },
        
        "demo" => {
            crate::n!(C_, "Starting Wayland demo...");
            
            
            let _ = crate::wayland::init();
            
            
            crate::wayland::bjz(|compositor| {
                
                let avh = compositor.create_surface();
                
                
                let width = 400u32;
                let height = 300u32;
                let mut buffer = alloc::vec![0xFF0A0F0C_u32; (width * height) as usize];
                
                
                for y in 0..height {
                    for x in 0..width {
                        let r = (x * 255 / width) as u8;
                        let g = ((y * 255 / height) as u8) / 2;
                        let b = 0x20_u8;
                        buffer[(y * width + x) as usize] = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
                    }
                }
                
                
                for x in 0..width {
                    buffer[x as usize] = 0xFF00FF66;
                    buffer[((height - 1) * width + x) as usize] = 0xFF00FF66;
                }
                for y in 0..height {
                    buffer[(y * width) as usize] = 0xFF00FF66;
                    buffer[(y * width + width - 1) as usize] = 0xFF00FF66;
                }
                
                
                if let Some(surface) = compositor.surfaces.get_mut(&avh) {
                    surface.attach(buffer, width, height);
                    surface.set_title("Wayland Demo");
                    surface.set_position(200, 150);
                    surface.make_toplevel();
                    surface.commit();
                }
                
                crate::n!(B_, "[OK] Created surface {}", avh);
            });
            
            
            crate::wayland::cho();
            crate::n!(B_, "[OK] Frame composed to framebuffer");
            crate::println!();
            crate::println!("Press any key to close demo...");
            
            
            loop {
                if let Some(_) = crate::keyboard::ya() {
                    break;
                }
            }
            
            
            crate::framebuffer::clear();
        },
        
        "status" => {
            crate::n!(C_, "Wayland Compositor Status");
            crate::n!(C_, "--------------------------");
            
            crate::wayland::bjz(|compositor| {
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
                crate::n!(D_, "Compositor not initialized");
                crate::println!("Run 'wayland init' first");
            });
        },
        
        _ => {
            crate::n!(C_, "TrustOS Wayland Compositor");
            crate::n!(C_, "--------------------------");
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


pub(super) fn kta(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");

    match je {
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
            let source = match crate::ramfs::bh(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
                Ok(data) => match alloc::string::String::from_utf8(data) {
                    Ok(j) => j,
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
            let source = match crate::ramfs::bh(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
                Ok(data) => match alloc::string::String::from_utf8(data) {
                    Ok(j) => j,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", filename); return; }
            };
            match crate::trustlang::cgv(&source) {
                Ok(()) => crate::println!("\x1b[32m?\x1b[0m {} -- no errors", filename),
                Err(e) => crate::println!("\x1b[31m?\x1b[0m {} -- {}", filename, e),
            }
        }
        "eval" => {
            
            let code = args[1..].join(" ");
            let wrapped = alloc::format!("fn main() {{ {} }}", code);
            match crate::trustlang::run(&wrapped) {
                Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
                Err(e) => crate::println!("\x1b[31mError:\x1b[0m {}", e),
            }
        }
        "demo" => {
            
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
            crate::ramfs::bh(|fs| {
                let _ = fs.write_file("/demo.tl", demo.as_bytes());
            });
            crate::println!("Created /demo.tl -- run with: trustlang run demo.tl");
            
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
                if crate::shell::cbc() { break; }
                crate::print!("\x1b[36mtl>\x1b[0m ");
                let line = crate::shell::read_line();
                let jw = line.trim();
                if jw.is_empty() {
                    if crate::shell::cbc() { break; }
                    continue;
                }
                if jw == "exit" || jw == "quit" { break; }
                if jw == "help" {
                    crate::println!("  println(\"hello\")       — print with newline");
                    crate::println!("  let x = 42;            — declare variable");
                    crate::println!("  for i in 0..5 {{ ... }}  — for loop");
                    crate::println!("  fn foo(n: i64) {{ ... }} — define function + fn main()");
                    continue;
                }
                match crate::trustlang::hwq(jw) {
                    Ok(output) => {
                        if !output.is_empty() { crate::print!("{}", output); }
                    }
                    Err(e) => crate::println!("\x1b[31mError:\x1b[0m {}", e),
                }
            }
        }
        "compile" | "native" => {
            
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: trustlang compile <file.tl>"); return; }
            };
            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/{}", filename)
            };
            let source = match crate::ramfs::bh(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
                Ok(data) => match alloc::string::String::from_utf8(data) {
                    Ok(j) => j,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", filename); return; }
            };
            crate::println!("\x1b[36m[TrustLang Native]\x1b[0m Compiling {} to x86_64...", filename);
            fn nhr(id: u8, anl: usize, argv: *const i64) -> i64 {
                
                match id {
                    0 | 1 => { 
                        if anl > 0 {
                            let val = unsafe { *argv };
                            crate::print!("{}", val);
                        }
                        if id == 1 { crate::println!(); }
                        0
                    }
                    _ => 0,
                }
            }
            match crate::trustlang::kwe(&source, nhr) {
                Ok(result) => {
                    crate::println!("\x1b[32m[TrustLang Native]\x1b[0m Program returned: {}", result);
                }
                Err(e) => crate::println!("\x1b[31m[TrustLang Native Error]\x1b[0m {}", e),
            }
        }
        "test" => {
            
            crate::println!("\x1b[1;36m══════ TrustLang Native x86_64 Test Suite ══════\x1b[0m\n");
            let (passed, bv, details) = crate::trustlang::ojd();
            crate::print!("{}", details);
            crate::println!();
            if bv == 0 {
                crate::println!("\x1b[1;32m  ALL {} TESTS PASSED\x1b[0m", passed);
            } else {
                crate::println!("\x1b[1;31m  {}/{} tests failed\x1b[0m", bv, passed + bv);
            }
        }
        "bench" => {
            
            crate::println!("\x1b[1;36m── TrustLang Native Benchmark ──\x1b[0m");
            crate::println!("  Computing fib(25) natively...");
            let (result, cycles) = crate::trustlang::tests::kbi();
            crate::println!("  Result: fib(25) = {}", result);
            crate::println!("  Cycles: {} (~{} µs @ 3GHz)", cycles, cycles / 3000);

            
            crate::println!("  Computing fib(25) via bytecode VM...");
            let psn = "fn fib(n: i64) -> i64 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fn main() { let r = fib(25); println(to_string(r)); }";
            let start = unsafe { core::arch::x86_64::_rdtsc() };
            let _ = crate::trustlang::run(psn);
            let end = unsafe { core::arch::x86_64::_rdtsc() };
            let feo = end - start;
            crate::println!("  VM Cycles: {} (~{} µs @ 3GHz)", feo, feo / 3000);
            if feo > 0 {
                crate::println!("  \x1b[1;32mNative speedup: {:.1}x\x1b[0m", feo as f64 / cycles as f64);
            }
        }
        _ => {
            crate::println!("\x1b[1;36mTrustLang\x1b[0m -- Integrated Programming Language");
            crate::println!("  Rust-inspired syntax, bytecode VM + native x86_64 compiler\n");
            crate::println!("Commands:");
            crate::println!("  trustlang run <file.tl>        Compile & execute (bytecode VM)");
            crate::println!("  trustlang compile <file.tl>    Compile to native x86_64 & execute");
            crate::println!("  trustlang check <file.tl>      Syntax check only");
            crate::println!("  trustlang eval <code>          Evaluate inline code");
            crate::println!("  trustlang repl                 Interactive REPL");
            crate::println!("  trustlang demo                 Create & run demo program");
            crate::println!("  trustlang test                 Run native backend test suite");
            crate::println!("  trustlang bench                Benchmark native vs bytecode VM");
            crate::println!("\nExample:");
            crate::println!("  trustlang eval println(\"Hello TrustOS!\")");
            crate::println!("  trustlang eval \"let x = 42; println(to_string(x * 2))\"");
        }
    }
}



fn pns(line: &str) -> alloc::vec::Vec<u32> {
    let chars: alloc::vec::Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut colors = alloc::vec![0xFFD4D4D4u32; len]; 
    if len == 0 { return colors; }

    
    let hnb = {
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
    let don = hnb.unwrap_or(len);

    
    if let Some(cp) = hnb {
        for i in cp..len {
            colors[i] = 0xFF6A9955;
        }
    }

    
    let mut bcj = false;
    for i in 0..don {
        if chars[i] == '"' {
            colors[i] = 0xFFCE9178;
            bcj = !bcj;
        } else if bcj {
            colors[i] = 0xFFCE9178;
        }
    }

    
    bcj = false;
    let mut i = 0usize;
    while i < don {
        if chars[i] == '"' {
            bcj = !bcj;
            i += 1;
            continue;
        }
        if bcj { i += 1; continue; }

        
        if chars[i].is_ascii_digit() {
            colors[i] = 0xFFB5CEA8;
            i += 1;
            continue;
        }
        
        if matches!(chars[i], '(' | ')' | '{' | '}' | '[' | ']') {
            colors[i] = 0xFFFFD700;
            i += 1;
            continue;
        }
        
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < don && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let fx: alloc::string::String = chars[start..i].iter().collect();

            
            let mut peek = i;
            while peek < don && chars[peek] == ' ' { peek += 1; }
            let gdw = peek < don && chars[peek] == '(';

            
            let bak: alloc::string::String = chars[..start].iter().collect();
            let jw = bak.trim_end();
            let muc = jw.ends_with("let") || jw.ends_with("mut");

            if matches!(fx.as_str(),
                "fn" | "let" | "mut" | "if" | "else" | "while" | "for" | "in" |
                "return" | "loop" | "break" | "continue" | "true" | "false" |
                "struct" | "enum" | "match" | "use" | "pub" | "const" | "static" |
                "impl" | "self" | "type")
            {
                for ay in start..i { colors[ay] = 0xFFFF7B72; } 
            } else if gdw {
                for ay in start..i { colors[ay] = 0xFF79C0FF; } 
            } else if muc {
                for ay in start..i { colors[ay] = 0xFF9CDCFE; } 
            }
            
            continue;
        }
        i += 1;
    }
    colors
}




pub(super) fn ktb() {
    let (dy, dw) = crate::framebuffer::kv();
    let w = dy as usize;
    let h = dw as usize;

    let pu = crate::framebuffer::ajy();
    if !pu {
        crate::framebuffer::adw();
        crate::framebuffer::pr(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    

    let pf = |buf: &mut [u32], w: usize, h: usize, cx: usize, u: usize, c: char, color: u32, scale: usize| {
        let du = crate::framebuffer::font::ol(c);
        for (row, &bits) in du.iter().enumerate() {
            for bf in 0..8u32 {
                if bits & (0x80 >> bf) != 0 {
                    for ak in 0..scale {
                        for am in 0..scale {
                            let p = cx + bf as usize * scale + am;
                            let o = u + row * scale + ak;
                            if p < w && o < h {
                                buf[o * w + p] = color;
                            }
                        }
                    }
                }
            }
        }
    };

    let draw_text_at = |buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            pf(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
        let gr = text.len() * 8 * scale;
        let am = if gr < w { (w - gr) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            pf(buf, w, h, am + i * 8 * scale, y, c, color, scale);
        }
    };

    let ev = |buf: &[u32], w: usize, h: usize| {
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
            let mq = bb_ptr as *mut u32;
            let aeu = bb_stride as usize;
            for y in 0..h.min(bb_h as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(buf[y * w..].as_ptr(), mq.add(y * aeu), w);
                }
            }
        }
        crate::framebuffer::ii();
    };

    let uq = |buf: &mut [u32]| {
        for aa in buf.iter_mut() { *aa = 0xFF000000; }
    };

    
    let mut kk: alloc::vec::Vec<u16> = (0..w / 8 + 1).map(|i| ((i * 37 + 13) % h) as u16).collect();
    let la: alloc::vec::Vec<u8> = (0..w / 8 + 1).map(|i| (((i * 7 + 3) % 4) + 1) as u8).collect();

    let draw_rain = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
        for ct in buf.iter_mut() {
            let g = ((*ct >> 8) & 0xFF) as u32;
            if g > 0 { *ct = 0xFF000000 | (g.saturating_sub(6) << 8); }
        }
        for ci in 0..cols.len() {
            let x = ci * 8;
            if x >= w { continue; }
            cols[ci] = cols[ci].wrapping_add(speeds[ci] as u16);
            if cols[ci] as usize >= h { cols[ci] = 0; }
            let y = cols[ci] as usize;
            let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
            let du = crate::framebuffer::font::ol(c);
            for (row, &bits) in du.iter().enumerate() {
                let o = y + row;
                if o >= h { break; }
                for bf in 0..8u32 {
                    if bits & (0x80 >> bf) != 0 {
                        let p = x + bf as usize;
                        if p < w { buf[o * w + p] = 0xFF00FF44; }
                    }
                }
            }
        }
    };

    
    let vj: u64 = 30;

    
    let vi = |buf: &mut [u32], w: usize, h: usize, blit: &dyn Fn(&[u32], usize, usize)| {
        for _ in 0..78 {
            for p in buf.iter_mut() {
                let r = ((*p >> 16) & 0xFF).saturating_sub(4);
                let g = ((*p >> 8) & 0xFF).saturating_sub(4);
                let b = (*p & 0xFF).saturating_sub(4);
                *p = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
        for aa in buf.iter_mut() { *aa = 0xFF000000; }
        blit(buf, w, h);
        
        crate::cpu::tsc::hq(1400);
    };

    
    
    
    
    
    let faw = |buf: &mut [u32], w: usize, h: usize,
                            kk: &mut [u16], la: &[u8],
                            lines: &[(&str, u32, usize)],
                            ms_per_char: u64,
                            hold_frames: u32| {
        let vu: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
        
        let dqf = (ms_per_char / vj).max(1) as u32;
        let dfz = vu as u32 * dqf;
        let total_frames = dfz + hold_frames;
        let mut frame = 0u32;

        while frame < total_frames {
            
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }

            
            draw_rain(buf, w, h, kk, la, frame);

            
            let hh = (frame / dqf) as usize;

            
            let sn: usize = lines.iter().map(|(_, _, j)| 16 * j + 12).sum();
            let mut y = if sn < h { (h - sn) / 2 } else { 20 };
            let mut abx = 0usize;

            for &(text, color, scale) in lines {
                let gr = text.len() * 8 * scale;
                let am = if gr < w { (w - gr) / 2 } else { 0 };
                for (i, c) in text.chars().enumerate() {
                    if abx + i >= hh { break; }
                    pf(buf, w, h, am + i * 8 * scale, y, c, color, scale);
                }
                
                if hh > abx && hh < abx + text.len() {
                    let ci = hh - abx;
                    let cx = am + ci * 8 * scale;
                    if (frame / 8) % 2 == 0 {
                        for u in y..y + 16 * scale {
                            if u < h && cx + 2 < w {
                                buf[u * w + cx] = 0xFF00FF88;
                                buf[u * w + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }
                }
                abx += text.len();
                y += 16 * scale + 12;
            }

            ev(buf, w, h);
            frame += 1;
            crate::cpu::tsc::hq(vj);
        }
    };

    
    
    
    
    
    
    let ory = |buf: &mut [u32], w: usize, h: usize,
                             kk: &mut [u16], la: &[u8],
                             title: &str,
                             source: &str,
                             pre_msg: &str,
                             _ms_per_char: u64,
                             _hold_frames: u32,
                             output_hold_ms: u64| {
        
        
        
        

        let aon: alloc::vec::Vec<&str> = source.lines().collect();
        let vu: usize = source.len();

        let bcn = 40usize;
        let acc = 50usize;
        let dla = acc + 30;
        let bw = 18usize;
        let cvh = 1usize;

        
        
        let mut ehr: alloc::vec::Vec<(usize, usize, char)> = alloc::vec::Vec::new();
        for (sz, line) in aon.iter().enumerate() {
            for (ci, c) in line.chars().enumerate() {
                ehr.push((sz, ci, c));
            }
            ehr.push((sz, line.len(), '\n')); 
        }

        
        let mut rng_state: u32 = 0xDEAD_BEEF;
        let mut cdk = |state: &mut u32| -> u32 {
            *state ^= *state << 13;
            *state ^= *state >> 17;
            *state ^= *state << 5;
            *state
        };

        
        
        let ppc: alloc::vec::Vec<(usize, char)> = alloc::vec![
            (45, 'w'),     
            (180, 'p'),    
            (350, 'e'),    
            (520, '0'),    
            (700, ';'),    
        ];

        
        
        let mut hh: usize = 0;
        let mut auq: u32 = 0;

        
        let bvh = |buf: &mut [u32], w: usize, h: usize,
                             kk: &mut [u16], la: &[u8],
                             auq: u32,
                             hh: usize,
                             typo_char: Option<(usize, usize, char)>| {
            
            for aa in buf.iter_mut() { *aa = 0xFF0A0A0A; }
            draw_rain(buf, w, h, kk, la, auq);
            for aa in buf.iter_mut() {
                let g = ((*aa >> 8) & 0xFF).min(25);
                *aa = 0xFF000000 | (g << 8);
            }

            
            for y in 0..acc {
                for x in 0..w {
                    buf[y * w + x] = 0xFF111111;
                }
            }
            draw_text_at(buf, w, h, bcn + 20, 15, title, 0xFF00FF88, 2);

            if !pre_msg.is_empty() {
                draw_text_at(buf, w, h, bcn + 20, acc + 5, pre_msg, 0xFF888888, 1);
            }

            
            let zc = bcn;
            let he = w - 2 * bcn;
            let xg = dla;
            let ug = h - dla - 80;
            for o in xg..xg + ug {
                for p in zc..zc + he {
                    if o < h && p < w {
                        buf[o * w + p] = 0xFF0D1117;
                    }
                }
            }
            
            for p in zc..zc + he {
                if xg < h { buf[xg * w + p] = 0xFF00FF44; }
                let age = (xg + ug).min(h) - 1;
                buf[age * w + p] = 0xFF00FF44;
            }
            for o in xg..(xg + ug).min(h) {
                buf[o * w + zc] = 0xFF00FF44;
                let right = (zc + he - 1).min(w - 1);
                buf[o * w + right] = 0xFF00FF44;
            }

            
            let aac = ug.saturating_sub(30) / bw;
            let cursor_line = {
                let mut ci = 0usize;
                let mut dat = 0usize;
                for (sz, line) in aon.iter().enumerate() {
                    if ci + line.len() >= hh {
                        dat = sz;
                        break;
                    }
                    ci += line.len() + 1;
                    dat = sz + 1;
                }
                dat.min(aon.len().saturating_sub(1))
            };
            let scroll_offset = if cursor_line >= aac.saturating_sub(2) {
                cursor_line.saturating_sub(aac.saturating_sub(3))
            } else {
                0
            };

            
            let adm = zc + 42;
            let mut cyu = 0usize;
            
            for sz in 0..scroll_offset.min(aon.len()) {
                cyu += aon[sz].len() + 1; 
            }
            for pt in 0..(aon.len() - scroll_offset.min(aon.len())) {
                let sz = pt + scroll_offset;
                let ly = dla + 15 + pt * bw;
                if ly + 16 > xg + ug { break; }
                let line = aon[sz];

                
                let mzr = alloc::format!("{:>3}", sz + 1);
                draw_text_at(buf, w, h, zc + 8, ly, &mzr, 0xFF555555, cvh);
                let jep = zc + 35;
                for ak in ly..ly + 16 {
                    if ak < h && jep < w { buf[ak * w + jep] = 0xFF333333; }
                }

                
                let myl = pns(line);
                for (ci, c) in line.chars().enumerate() {
                    if cyu >= hh { break; }
                    let color = myl.get(ci).copied().unwrap_or(0xFFD4D4D4);
                    pf(buf, w, h, adm + ci * 8 * cvh, ly, c, color, cvh);
                    cyu += 1;
                }

                
                if let Some((tli, tci, wo)) = typo_char {
                    if sz == tli && cyu == hh {
                        let nbj = dla + 15 + pt * bw;
                        pf(buf, w, h, adm + tci * 8 * cvh, nbj, wo, 0xFFFF4444, cvh);
                    }
                }

                if cyu < hh { cyu += 1; } 
            }

            
            if aon.len() > aac {
                let yc = zc + he - 8;
                let dyj = xg + 2;
                let bdo = ug.saturating_sub(4);
                for o in dyj..dyj + bdo {
                    if o < h && yc < w { buf[o * w + yc] = 0xFF1A1A1A; }
                }
                let zo = ((aac * bdo) / aon.len()).max(10);
                let akn = if aon.len() > 0 { dyj + (scroll_offset * bdo) / aon.len() } else { dyj };
                for o in akn..(akn + zo).min(dyj + bdo) {
                    if o < h && yc < w {
                        buf[o * w + yc] = 0xFF00FF44;
                        if yc + 1 < w { buf[o * w + yc + 1] = 0xFF00FF44; }
                    }
                }
            }

            
            if hh <= vu && cursor_line >= scroll_offset {
                let mut flq = 0usize;
                let mut gyb = 0usize;
                let mut gya = 0usize;
                for (sz, line) in aon.iter().enumerate() {
                    if flq + line.len() >= hh {
                        gyb = sz;
                        gya = hh - flq;
                        break;
                    }
                    flq += line.len() + 1;
                }
                
                let cursor_col = if typo_char.is_some() && typo_char.unwrap().0 == gyb {
                    gya + 1
                } else {
                    gya
                };
                let psi = gyb.saturating_sub(scroll_offset);
                let u = dla + 15 + psi * bw;
                let cx = zc + 42 + cursor_col * 8 * cvh;
                if (auq / 5) % 2 == 0 {
                    for ak in u..u + 16 {
                        if ak < h && cx < w && cx + 2 < w {
                            buf[ak * w + cx] = 0xFF00FF88;
                            buf[ak * w + cx + 1] = 0xFF00FF88;
                        }
                    }
                }
            }

            
            let status_y = h - 40;
            for o in status_y..h {
                for p in 0..w { buf[o * w + p] = 0xFF111111; }
            }
            {
                let lah = {
                    let mut hky = 0usize;
                    let mut dat = 1usize;
                    for (_li, line) in aon.iter().enumerate() {
                        if hky + line.len() >= hh { break; }
                        hky += line.len() + 1;
                        dat += 1;
                    }
                    dat
                };
                let status = alloc::format!("Ln {}  |  {} lines  |  TrustLang", lah, aon.len());
                draw_text_at(buf, w, h, bcn, status_y + 12, &status, 0xFF00CC66, 1);
            }

            ev(buf, w, h);
        };

        
        while hh < vu {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { hh = vu; break; }
            }

            
            let mut hsc = false;
            for &(typo_idx, wrong_c) in ppc.iter() {
                if hh == typo_idx && typo_idx < vu {
                    
                    let mut flr = 0usize;
                    let mut gym = 0usize;
                    let mut gyl = 0usize;
                    for (sz, line) in aon.iter().enumerate() {
                        if flr + line.len() > hh {
                            gym = sz;
                            gyl = hh - flr;
                            break;
                        }
                        flr += line.len() + 1;
                    }

                    
                    bvh(buf, w, h, kk, la, auq,
                        hh, Some((gym, gyl, wrong_c)));
                    auq += 1;
                    crate::cpu::tsc::hq(120);

                    
                    bvh(buf, w, h, kk, la, auq,
                        hh, Some((gym, gyl, wrong_c)));
                    auq += 1;
                    crate::cpu::tsc::hq(400);

                    
                    bvh(buf, w, h, kk, la, auq,
                        hh, None);
                    auq += 1;
                    crate::cpu::tsc::hq(150);

                    
                    hsc = true;
                    break;
                }
            }

            
            let c = ehr.get(hh).map(|&(_, _, c)| c).unwrap_or(' ');
            let njz = ehr.get(hh + 1).map(|&(_, _, c)| c);

            
            let mut aar: u64 = 20;

            
            if c == '\n' {
                aar = 80 + (cdk(&mut rng_state) % 120) as u64; 
            }
            
            else if c == '{' || (njz == Some('}')) {
                aar = 150 + (cdk(&mut rng_state) % 200) as u64;
            }
            
            else if c == '/' {
                aar = 40 + (cdk(&mut rng_state) % 60) as u64;
            }
            
            else if c == ' ' {
                aar = 15 + (cdk(&mut rng_state) % 40) as u64;
            }
            
            else if c == '(' || c == ')' || c == ';' || c == ',' {
                aar = 30 + (cdk(&mut rng_state) % 30) as u64;
            }
            
            else {
                aar = 18 + (cdk(&mut rng_state) % 25) as u64;
            }

            
            if cdk(&mut rng_state) % 100 < 5 {
                aar += 200 + (cdk(&mut rng_state) % 400) as u64;
            }

            
            if hsc {
                aar += 80;
            }

            
            bvh(buf, w, h, kk, la, auq, hh, None);
            auq += 1;

            
            hh += 1;
            crate::cpu::tsc::hq(aar);
        }

        
        bvh(buf, w, h, kk, la, auq, vu, None);
        crate::cpu::tsc::hq(1200);

        
        
        
        {
            
            bvh(buf, w, h, kk, la, auq, vu, None);

            
            let bio = h - 120;
            let npq = 80;
            let bin = bcn;
            let gma = w - 2 * bcn;

            
            for o in bio..bio + npq {
                for p in bin..bin + gma {
                    if o < h && p < w {
                        buf[o * w + p] = 0xFF0A0E14;
                    }
                }
            }
            
            for p in bin..bin + gma {
                if bio < h { buf[bio * w + p] = 0xFF00FF44; }
            }
            
            draw_text_at(buf, w, h, bin + 8, bio + 4, "OUTPUT", 0xFF888888, 1);

            
            draw_text_at(buf, w, h, bin + 8, bio + 22, "$ trustlang compile youtube_dvd.tl", 0xFF00CC66, 1);
            ev(buf, w, h);
            crate::cpu::tsc::hq(800);

            
            draw_text_at(buf, w, h, bin + 8, bio + 38, "Compiling...", 0xFFAABBCC, 1);
            ev(buf, w, h);
            crate::cpu::tsc::hq(1200);

            
            for o in bio + 36..bio + 56 {
                for p in bin + 4..bin + gma - 4 {
                    if o < h && p < w {
                        buf[o * w + p] = 0xFF0A0E14;
                    }
                }
            }
            draw_text_at(buf, w, h, bin + 8, bio + 38, "Compiled successfully in 0.3s  (47 lines, 0 errors)", 0xFF00FF88, 1);
            
            draw_text_at(buf, w, h, bin + 8, bio + 54, "Generated 284 bytecode instructions", 0xFF666666, 1);
            ev(buf, w, h);
            crate::cpu::tsc::hq(2000);
        }

        
        

        
        crate::ramfs::bh(|fs| {
            let _ = fs.write_file("/youtube_dvd.tl", source.as_bytes());
        });

        
        {
            
            for aa in buf.iter_mut() { *aa = 0xFF0A0A0A; }
            draw_rain(buf, w, h, kk, la, auq);
            for aa in buf.iter_mut() {
                let g = ((*aa >> 8) & 0xFF).min(15);
                *aa = 0xFF000000 | (g << 8);
            }

            
            let nw = 30usize;
            let qr = 20usize;
            let ul = w - 60;
            let afy = h - 40;
            
            for o in qr..qr + afy {
                for p in nw..nw + ul {
                    if o < h && p < w {
                        buf[o * w + p] = 0xFF0D0D0D;
                    }
                }
            }
            
            for o in qr..qr + 28 {
                for p in nw..nw + ul {
                    if o < h && p < w {
                        buf[o * w + p] = 0xFF1A1A1A;
                    }
                }
            }
            draw_text_at(buf, w, h, nw + 12, qr + 6, "TrustOS Terminal", 0xFF00FF88, 1);
            
            for p in nw..nw + ul {
                if qr < h { buf[qr * w + p] = 0xFF00FF44; }
                let age = (qr + afy - 1).min(h - 1);
                buf[age * w + p] = 0xFF00FF44;
            }
            for o in qr..qr + afy {
                if o < h {
                    buf[o * w + nw] = 0xFF00FF44;
                    let r = (nw + ul - 1).min(w - 1);
                    buf[o * w + r] = 0xFF00FF44;
                }
            }

            let kd = nw + 16;
            let mut ie = qr + 40;

            
            draw_text_at(buf, w, h, kd, ie, "TrustOS v2.0 - TrustLang Runtime", 0xFF00FF88, 1);
            ie += 20;
            draw_text_at(buf, w, h, kd, ie, "Type 'help' for available commands.", 0xFF666666, 1);
            ie += 28;

            
            
            draw_text_at(buf, w, h, kd, ie, "root", 0xFFFF0000, 1);
            draw_text_at(buf, w, h, kd + 32, ie, "@", 0xFFFFFFFF, 1);
            draw_text_at(buf, w, h, kd + 40, ie, "trustos", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, kd + 96, ie, ":/$ ", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, kd + 128, ie, "trustlang compile youtube_dvd.tl", 0xFFD4D4D4, 1);
            ie += 18;
            draw_text_at(buf, w, h, kd, ie, "Compiled successfully in 0.3s", 0xFF00FF88, 1);
            ie += 18;
            draw_text_at(buf, w, h, kd, ie, "Generated 284 bytecode instructions", 0xFF666666, 1);
            ie += 28;

            
            let dcy = ie;
            draw_text_at(buf, w, h, kd, dcy, "root", 0xFFFF0000, 1);
            draw_text_at(buf, w, h, kd + 32, dcy, "@", 0xFFFFFFFF, 1);
            draw_text_at(buf, w, h, kd + 40, dcy, "trustos", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, kd + 96, dcy, ":/$ ", 0xFF00FF00, 1);
            ev(buf, w, h);
            crate::cpu::tsc::hq(800);

            
            let oiy = "trustlang run youtube_dvd.tl";
            let krw = kd + 128;
            for (ci, c) in oiy.chars().enumerate() {
                pf(buf, w, h, krw + ci * 8, dcy, c, 0xFFD4D4D4, 1);
                ev(buf, w, h);
                let d = 30 + (((ci as u32 * 7 + 13) ^ 0x5A) % 50) as u64;
                crate::cpu::tsc::hq(d);
            }
            crate::cpu::tsc::hq(400);

            
            ie = dcy + 24;
            draw_text_at(buf, w, h, kd, ie, "Running youtube_dvd.tl ...", 0xFFAABBCC, 1);
            ev(buf, w, h);
            crate::cpu::tsc::hq(600);
        }

        
        match crate::trustlang::run(source) {
            Ok(output) => {
                if !output.is_empty() {
                    
                    
                    let noj: alloc::vec::Vec<&str> = output.lines().collect();
                    uq(buf);
                    draw_text_centered(buf, w, h, 25, "OUTPUT", 0xFF00FF88, 3);
                    for (i, line) in noj.iter().enumerate() {
                        let ly = 80 + i * 20;
                        if ly + 16 > h - 40 { break; }
                        let am = if line.len() * 8 < w { (w - line.len() * 8) / 2 } else { 40 };
                        draw_text_at(buf, w, h, am, ly, line, 0xFFCCFFCC, 1);
                    }
                    ev(buf, w, h);
                    crate::cpu::tsc::hq(output_hold_ms);
                }
                if output.is_empty() {
                    
                    
                    if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
                        let mq = bb_ptr as *mut u32;
                        let aeu = bb_stride as usize;
                        for y in 0..h.min(bb_h as usize) {
                            unsafe {
                                core::ptr::copy_nonoverlapping(
                                    mq.add(y * aeu),
                                    buf[y * w..].as_mut_ptr(),
                                    w,
                                );
                            }
                        }
                    }
                    
                    crate::cpu::tsc::hq(output_hold_ms);
                }
            }
            Err(e) => {
                uq(buf);
                draw_text_centered(buf, w, h, h / 2 - 20, "Runtime Error", 0xFFFF4444, 4);
                let lrl = if e.len() > 80 { &e[..80] } else { &e };
                draw_text_centered(buf, w, h, h / 2 + 50, lrl, 0xFFFF8888, 1);
                ev(buf, w, h);
                crate::cpu::tsc::hq(3000);
            }
        }
    };

    
    
    
    

    crate::serial_println!("[TL_SHOWCASE] Starting TrustLang showcase -- YouTube DVD Screensaver");

    
    
    
    uq(&mut buf);
    faw(&mut buf, w, h, &mut kk, &la,
        &[("TrustLang", 0xFF00FF88, 6),
          ("", 0xFF000000, 1),
          ("Live Demo", 0xFF00CC66, 4),
          ("Programming Inside TrustOS", 0xFF008844, 2)],
        90, 200);   
    vi(&mut buf, w, h, &ev);

    
    
    
    uq(&mut buf);
    faw(&mut buf, w, h, &mut kk, &la,
        &[("YouTube DVD Screensaver", 0xFFFF0000, 4),
          ("", 0xFF000000, 1),
          ("A bouncing 3D YouTube logo", 0xFFCCFFCC, 2),
          ("with 'Like & Subscribe' text.", 0xFFCCFFCC, 2),
          ("", 0xFF000000, 1),
          ("Written, compiled, and animated", 0xFF00FF88, 2),
          ("live inside the OS kernel.", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("All in real-time. Zero dependencies.", 0xFF888888, 2)],
        70, 180);   
    vi(&mut buf, w, h, &ev);

    
    
    
    uq(&mut buf);
    faw(&mut buf, w, h, &mut kk, &la,
        &[("How TrustLang Works", 0xFF00FF88, 4),
          ("", 0xFF000000, 1),
          ("tokenize()   Lexer -> Tokens", 0xFFFF7B72, 2),
          ("parse()      Tokens -> AST", 0xFFFFA657, 2),
          ("compile()    AST -> Bytecode", 0xFFA5D6FF, 2),
          ("execute()    Bytecode -> VM", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("pixel() fill_rect() draw_text()", 0xFFFFD700, 2),
          ("flush() sleep() clear_screen()", 0xFFFFD700, 2)],
        60, 170);   
    vi(&mut buf, w, h, &ev);

    
    
    
    uq(&mut buf);
    ory(&mut buf, w, h, &mut kk, &la,
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
        30,     
        80,     
        3000);  
    vi(&mut buf, w, h, &ev);

    
    
    
    uq(&mut buf);
    faw(&mut buf, w, h, &mut kk, &la,
        &[("TrustLang", 0xFF00FF88, 6),
          ("", 0xFF000000, 1),
          ("Lexer > Parser > Compiler > VM", 0xFFAADDAA, 2),
          ("Real-time graphics. Zero deps.", 0xFFAADDAA, 2),
          ("", 0xFF000000, 1),
          ("Built into TrustOS.", 0xFF00CC66, 3),
          ("", 0xFF000000, 1),
          ("github.com/nathan237/TrustOS", 0xFF00FF88, 2)],
        80, 250);   
    vi(&mut buf, w, h, &ev);

    
    uq(&mut buf);
    ev(&buf, w, h);
    if !pu {
        crate::framebuffer::pr(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[TL_SHOWCASE] Showcase complete");
}














pub(super) fn ktc() {
    let (dy, dw) = crate::framebuffer::kv();
    let w = dy as usize;
    let h = dw as usize;

    let pu = crate::framebuffer::ajy();
    if !pu {
        crate::framebuffer::adw();
        crate::framebuffer::pr(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    

    let pf = |buf: &mut [u32], w: usize, h: usize,
                         cx: usize, u: usize, c: char, color: u32, scale: usize| {
        let du = crate::framebuffer::font::ol(c);
        for (row, &bits) in du.iter().enumerate() {
            for bf in 0..8u32 {
                if bits & (0x80 >> bf) != 0 {
                    for ak in 0..scale {
                        for am in 0..scale {
                            let p = cx + bf as usize * scale + am;
                            let o = u + row * scale + ak;
                            if p < w && o < h { buf[o * w + p] = color; }
                        }
                    }
                }
            }
        }
    };

    let draw_text_at = |buf: &mut [u32], w: usize, h: usize,
                        x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            pf(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize,
                              y: usize, text: &str, color: u32, scale: usize| {
        let gr = text.len() * 8 * scale;
        let am = if gr < w { (w - gr) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            pf(buf, w, h, am + i * 8 * scale, y, c, color, scale);
        }
    };

    let ev = |buf: &[u32], w: usize, h: usize| {
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
            let mq = bb_ptr as *mut u32;
            let aeu = bb_stride as usize;
            for y in 0..h.min(bb_h as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buf[y * w..].as_ptr(), mq.add(y * aeu), w);
                }
            }
        }
        crate::framebuffer::ii();
    };

    let uq = |buf: &mut [u32]| {
        for aa in buf.iter_mut() { *aa = 0xFF000000; }
    };

    
    let draw_rect = |buf: &mut [u32], w: usize, h: usize,
                     x: usize, y: usize, lk: usize, pp: usize, color: u32| {
        for ad in 0..pp {
            for dx in 0..lk {
                let p = x + dx;
                let o = y + ad;
                if p < w && o < h { buf[o * w + p] = color; }
            }
        }
    };

    let vj: u64 = 30;

    let vi = |buf: &mut [u32], w: usize, h: usize,
                   blit: &dyn Fn(&[u32], usize, usize)| {
        for _ in 0..40 {
            for p in buf.iter_mut() {
                let r = ((*p >> 16) & 0xFF).saturating_sub(8);
                let g = ((*p >> 8) & 0xFF).saturating_sub(8);
                let b = (*p & 0xFF).saturating_sub(8);
                *p = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
        for aa in buf.iter_mut() { *aa = 0xFF000000; }
        blit(buf, w, h);
        crate::cpu::tsc::hq(400);
    };

    
    

    
    let bqq = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        
        
        let phase = (frame % 160) as u32;
        let kq = if phase < 80 { phase / 2 } else { (160 - phase) / 2 };
        let cnz = ((frame + 40) % 120) as u32;
        let gox = if cnz < 60 { cnz / 2 } else { (120 - cnz) / 2 };
        for y in 0..h {
            let bkj = (y as u32 * 40) / h as u32;
            for x in 0..w {
                let eem = (x as u32 * 10) / w as u32;
                let r = (bkj / 4 + gox / 3).min(40);
                let g = (eem / 3).min(15);
                let b = (bkj + kq + eem / 2).min(80);
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    };

    
    let cub = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        let scroll = (frame as usize * 2) % h;
        for y in 0..h {
            let ak = (y + scroll) % h;
            let gwn = (ak / 4) % 2 == 0;
            for x in 0..w {
                let adi = if gwn { 35u32 } else { 15 };
                let flash = if (ak % 60) < 2 { 30u32 } else { 0 };
                let r = (adi + flash).min(65);
                let g = 2;
                let b = 5;
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    };

    
    let hhp = |buf: &mut [u32], w: usize, h: usize, _frame: u32| {
        for y in 0..h {
            for x in 0..w {
                let nnc = (x % 20 < 2) && (y % 20 < 2);
                let color = if nnc { 0xFF0A1A3A } else { 0xFF060E1E };
                buf[y * w + x] = color;
            }
        }
    };

    
    let cud = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        
        for p in buf.iter_mut() {
            let r = ((*p >> 16) & 0xFF).saturating_sub(8);
            let g = ((*p >> 8) & 0xFF).saturating_sub(12);
            let b = (*p & 0xFF).saturating_sub(8);
            *p = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
        
        for i in 0..24u32 {
            let seed = (i.wrapping_mul(2654435761).wrapping_add(frame.wrapping_mul(37))) as usize;
            let p = (seed.wrapping_mul(7919)) % w;
            let gru = (frame as usize + seed) % h;
            let o = h.saturating_sub(gru);
            let brightness = (50 + (seed % 40)) as u32;
            if p < w && o < h {
                buf[o * w + p] = 0xFF000000 | (brightness / 4 << 16) | (brightness << 8) | (brightness / 3);
                if p + 1 < w { buf[o * w + p + 1] = 0xFF000000 | (brightness << 8); }
            }
        }
    };

    
    let hht = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        for aa in buf.iter_mut() { *aa = 0xFF050510; }
        
        for i in 0..80u32 {
            let am = ((i.wrapping_mul(7919)) as usize) % w;
            let ak = ((i.wrapping_mul(104729)) as usize) % h;
            let haf = ((frame.wrapping_add(i * 17)) % 30) as u32;
            let na = if haf < 15 { 40 + haf * 3 } else { 40 + (30 - haf) * 3 };
            let na = na.min(120);
            if am < w && ak < h {
                buf[ak * w + am] = 0xFF000000 | (na << 16) | (na << 8) | na;
            }
        }
    };

    
    let ctz = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        for aa in buf.iter_mut() { *aa = 0xFF0A0A14; }
        
        let jog = 0xFF0F2818u32;
        let hdw = 0xFF00AA44u32;
        for i in 0..20u32 {
            let ty = ((i.wrapping_mul(7919) as usize) % h) & !3;
            let bu = ((i.wrapping_mul(104729) as usize) % w) & !3;
            
            if ty < h {
                for x in 0..w {
                    buf[ty * w + x] = jog;
                }
            }
            
            if bu < w {
                for y in 0..h {
                    buf[y * w + bu] = jog;
                }
            }
        }
        
        let exh = ((frame as usize * 3) % h) & !3;
        if exh < h {
            let wl = (w / 4).min(120);
            let bdd = (frame as usize * 5) % w;
            for dx in 0..wl {
                let p = (bdd + dx) % w;
                buf[exh * w + p] = hdw;
                if exh + 1 < h { buf[(exh + 1) * w + p] = hdw; }
            }
        }
    };

    
    let djf = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        let cmf = (frame as u32).min(60); 
        for y in 0..h {
            let bkj = y as u32 * 100 / h as u32; 
            let csq = if bkj > 50 { (bkj - 50).min(50) + cmf } else { cmf / 2 };
            let r = (csq * 2).min(90);
            let g = (csq * 3 / 4).min(45);
            let b = (20u32.saturating_sub(csq / 3)).min(30);
            for x in 0..w {
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
        
        let jjs = w / 2;
        let jjt = h - (cmf as usize * h / 200);
        let ceb = 80usize + cmf as usize;
        for ad in 0..ceb {
            for dx in 0..ceb {
                let wz = dx * dx + ad * ad;
                if wz < ceb * ceb {
                    let intensity = (ceb * ceb - wz) * 60 / (ceb * ceb);
                    let intensity = intensity as u32;
                    for (am, ak) in [(jjs + dx, jjt.wrapping_sub(ad)),
                                     (jjs.wrapping_sub(dx), jjt.wrapping_sub(ad))] {
                        if am < w && ak < h {
                            let ku = buf[ak * w + am];
                            let ajp = ((ku >> 16) & 0xFF) + intensity;
                            let cir = ((ku >> 8) & 0xFF) + intensity * 2 / 3;
                            let bsd = (ku & 0xFF) + intensity / 4;
                            buf[ak * w + am] = 0xFF000000
                                | (ajp.min(255) << 16)
                                | (cir.min(255) << 8)
                                | bsd.min(255);
                        }
                    }
                }
            }
        }
    };

    
    let mut kk: alloc::vec::Vec<u16> =
        (0..w / 8 + 1).map(|i| ((i * 37 + 13) % h) as u16).collect();
    let la: alloc::vec::Vec<u8> =
        (0..w / 8 + 1).map(|i| (((i * 7 + 3) % 4) + 1) as u8).collect();

    let draw_rain = |buf: &mut [u32], w: usize, h: usize,
                     cols: &mut [u16], speeds: &[u8], frame: u32| {
        for ct in buf.iter_mut() {
            let g = ((*ct >> 8) & 0xFF) as u32;
            if g > 0 { *ct = 0xFF000000 | (g.saturating_sub(6) << 8); }
        }
        for ci in 0..cols.len() {
            let x = ci * 8;
            if x >= w { continue; }
            cols[ci] = cols[ci].wrapping_add(speeds[ci] as u16);
            if cols[ci] as usize >= h { cols[ci] = 0; }
            let y = cols[ci] as usize;
            let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
            let du = crate::framebuffer::font::ol(c);
            for (row, &bits) in du.iter().enumerate() {
                let o = y + row;
                if o >= h { break; }
                for bf in 0..8u32 {
                    if bits & (0x80 >> bf) != 0 {
                        let p = x + bf as usize;
                        if p < w { buf[o * w + p] = 0xFF00FF44; }
                    }
                }
            }
        }
    };

    
    
    
    
    let boy = |buf: &mut [u32], w: usize, h: usize,
                      kk: &mut [u16], la: &[u8],
                      lines: &[(&str, u32, usize)],
                      ms_per_char: u64, hold_frames: u32, bg_id: u8| {
        let vu: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
        let dqf = (ms_per_char / vj).max(1) as u32;
        let dfz = vu as u32 * dqf;
        let total_frames = dfz + hold_frames;
        let mut frame = 0u32;
        while frame < total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            
            match bg_id {
                1 => bqq(buf, w, h, frame),
                2 => cub(buf, w, h, frame),
                3 => hhp(buf, w, h, frame),
                4 => cud(buf, w, h, frame),
                5 => hht(buf, w, h, frame),
                6 => ctz(buf, w, h, frame),
                7 => djf(buf, w, h, frame),
                8 => draw_rain(buf, w, h, kk, la, frame),
                _ => { for aa in buf.iter_mut() { *aa = 0xFF000000; } }
            }
            let hh = (frame / dqf) as usize;
            let sn: usize = lines.iter().map(|(_, _, j)| 16 * j + 12).sum();
            let mut y = if sn < h { (h - sn) / 2 } else { 20 };
            let mut abx = 0usize;
            for &(text, color, scale) in lines {
                let gr = text.len() * 8 * scale;
                let am = if gr < w { (w - gr) / 2 } else { 0 };
                for (i, c) in text.chars().enumerate() {
                    if abx + i >= hh { break; }
                    pf(buf, w, h, am + i * 8 * scale, y, c, color, scale);
                }
                
                if hh > abx && hh < abx + text.len() {
                    let ci = hh - abx;
                    let cx = am + ci * 8 * scale;
                    if (frame / 8) % 2 == 0 {
                        for u in y..y + 16 * scale {
                            if u < h && cx + 2 < w {
                                buf[u * w + cx] = 0xFFFFFFFF;
                                buf[u * w + cx + 1] = 0xFFFFFFFF;
                            }
                        }
                    }
                }
                abx += text.len();
                y += 16 * scale + 12;
            }
            ev(buf, w, h);
            frame += 1;
            crate::cpu::tsc::hq(vj);
        }
    };

    crate::serial_println!("[FILM] TrustOS Film started");

    
    
    
    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("ACT I", 0xFF88CCFF, 5)],
        50, 30, 1);
    vi(&mut buf, w, h, &ev);

    
    {
        let yr = 2u32;
        let wq = "You use a computer";
        let aiw = "every single day.";
        let vu = wq.len() + aiw.len();
        let total_frames = vu as u32 * yr + 50;
        
        let mut puw: [(i32,i32,usize,usize,u32,i32,i32); 6] = [
            (80, 40, 120, 80, 0xFF3355AA, 2, 1),
            (w as i32 - 220, 90, 100, 70, 0xFF55AA33, -1, 2),
            (180, h as i32 - 180, 130, 85, 0xFFAA5533, 1, -1),
            (w as i32 / 2, 60, 110, 75, 0xFF8844CC, -2, 1),
            (40, h as i32 / 2, 125, 80, 0xFF4488CC, 1, -2),
            (w as i32 - 160, h as i32 / 2 + 40, 100, 65, 0xFFCC8844, -1, -1),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bqq(&mut buf, w, h, frame);
            
            for ld in 0..6 {
                let aw = &mut puw[ld];
                
                if frame < (ld as u32) * 8 { continue; }
                aw.0 += aw.5;
                aw.1 += aw.6;
                if aw.0 < 0 || aw.0 + aw.2 as i32 > w as i32 { aw.5 = -aw.5; aw.0 += aw.5; }
                if aw.1 < 0 || aw.1 + aw.3 as i32 > h as i32 { aw.6 = -aw.6; aw.1 += aw.6; }
                let wx = aw.0.max(0) as usize;
                let wy = aw.1.max(0) as usize;
                let bww = aw.4;
                let aep = ((bww >> 16) & 0xFF) / 3;
                let puk = ((bww >> 8)  & 0xFF) / 3;
                let hcf = (bww & 0xFF) / 3;
                let dim = 0xFF000000 | (aep << 16) | (puk << 8) | hcf;
                draw_rect(&mut buf, w, h, wx, wy, aw.2, aw.3, dim);
                draw_rect(&mut buf, w, h, wx, wy, aw.2, 10, bww);
                
                for sz in 0..3usize {
                    let ly = wy + 16 + sz * 12;
                    if ly + 5 < wy + aw.3 {
                        draw_rect(&mut buf, w, h, wx + 6, ly, aw.2.saturating_sub(12), 5, 0xFF222233);
                    }
                }
            }
            
            let hh = (frame / yr) as usize;
            let scale = 3usize;
            let bw = 16 * scale + 12;
            let y1 = h / 2 - bw;
            let y2 = h / 2 + 4;
            let avm = wq.len() * 8 * scale;
            let wn = if avm < w { (w - avm) / 2 } else { 0 };
            for (i, c) in wq.chars().enumerate() {
                if i >= hh { break; }
                pf(&mut buf, w, h, wn + i * 8 * scale, y1, c, 0xFFFFFFFF, scale);
            }
            if hh > wq.len() {
                let apj = aiw.len() * 8 * scale;
                let tq = if apj < w { (w - apj) / 2 } else { 0 };
                let ua = hh - wq.len();
                for (i, c) in aiw.chars().enumerate() {
                    if i >= ua { break; }
                    pf(&mut buf, w, h, tq + i * 8 * scale, y2, c, 0xFFFFFFFF, scale);
                }
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    {
        let yr = 2u32;
        let wq = "Do you really know";
        let aiw = "what it does?";
        let vu = wq.len() + aiw.len();
        let total_frames = vu as u32 * yr + 60;
        let irs = w / 10;
        let mut dxd: alloc::vec::Vec<i32> = (0..irs).map(|i| -((i * 37 % 200) as i32)).collect();
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bqq(&mut buf, w, h, frame);
            
            let speed = 1 + (frame / 30) as i32;
            for qi in 0..irs {
                dxd[qi] += speed + (qi as i32 % 3);
                if dxd[qi] > h as i32 { dxd[qi] = -(qi as i32 * 13 % 60); }
                if dxd[qi] >= 0 {
                    let p = qi * 10 + 2;
                    let o = dxd[qi] as usize;
                    let na = 0xFF000000 | (0x40 << 16) | (0x60 << 8) | 0xFF;
                    if p < w && o < h {
                        pf(&mut buf, w, h, p, o, '?', na, 1);
                    }
                }
            }
            
            let hh = (frame / yr) as usize;
            let scale = 3usize;
            let y1 = h / 2 - 40;
            let y2 = h / 2 + 20;
            let avm = wq.len() * 8 * scale;
            let wn = if avm < w { (w - avm) / 2 } else { 0 };
            for (i, c) in wq.chars().enumerate() {
                if i >= hh { break; }
                pf(&mut buf, w, h, wn + i * 8 * scale, y1, c, 0xFFCCCCCC, scale);
            }
            if hh > wq.len() {
                let apj = aiw.len() * 8 * scale;
                let tq = if apj < w { (w - apj) / 2 } else { 0 };
                let ua = hh - wq.len();
                for (i, c) in aiw.chars().enumerate() {
                    if i >= ua { break; }
                    pf(&mut buf, w, h, tq + i * 8 * scale, y2, c, 0xFFFF9944, 4);
                }
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    {
        let yr = 2u32;
        let wq = "The honest answer...";
        let aiw = "is no.";
        let bjt = (wq.len() + aiw.len()) as u32 * yr;
        let oro = 50u32;
        let total_frames = bjt + oro;
        
        let kzb: [(i32, i32); 12] = [
            (3,0),(-3,0),(0,3),(0,-3),(2,2),(-2,2),(2,-2),(-2,-2),
            (3,1),(-3,1),(1,-3),(-1,3),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            cub(&mut buf, w, h, frame);
            
            let hh = (frame / yr) as usize;
            let gsp = 3usize;
            let y1 = h / 2 - 60;
            let avm = wq.len() * 8 * gsp;
            let wn = if avm < w { (w - avm) / 2 } else { 0 };
            for (i, c) in wq.chars().enumerate() {
                if i >= hh { break; }
                pf(&mut buf, w, h, wn + i * 8 * gsp, y1, c, 0xFF888888, gsp);
            }
            if hh > wq.len() {
                let gsq = 5usize;
                let apj = aiw.len() * 8 * gsq;
                let tq = if apj < w { (w - apj) / 2 } else { 0 };
                let ua = hh - wq.len();
                for (i, c) in aiw.chars().enumerate() {
                    if i >= ua { break; }
                    pf(&mut buf, w, h, tq + i * 8 * gsq, h / 2, c, 0xFFFF4444, gsq);
                }
            }
            
            if frame > bjt {
                let progress = frame - bjt;
                let cx = w / 2;
                let u = h / 2;
                for &(cdx, cdy) in kzb.iter() {
                    for step in 0..(progress * 4) as i32 {
                        let p = (cx as i32 + cdx * step).max(0) as usize;
                        let o = (u as i32 + cdy * step).max(0) as usize;
                        if p < w && o < h {
                            buf[o * w + p] = 0xFFFFFFFF;
                            if p + 1 < w { buf[o * w + p + 1] = 0xFFFFDDDD; }
                            if o + 1 < h { buf[(o + 1) * w + p] = 0xFFFFDDDD; }
                        }
                    }
                }
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    
    
    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("ACT II", 0xFFFF6644, 5),
          ("", 0xFF000000, 1),
          ("The Problem", 0xFFFF4444, 3)],
        50, 30, 2);
    vi(&mut buf, w, h, &ev);

    
    {
        let yr = 2u32;
        let cmg: [(&str, u32, usize); 5] = [
            ("Your computer runs on", 0xFFCCCCCC, 2),
            ("an operating system.", 0xFFCCCCCC, 2),
            ("", 0xFF000000, 1),
            ("It controls", 0xFFCCCCCC, 2),
            ("EVERYTHING.", 0xFFFF6644, 4),
        ];
        let vu: usize = cmg.iter().map(|(t,_,_)| t.len()).sum();
        let total_frames = vu as u32 * yr + 70;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            cub(&mut buf, w, h, frame);
            
            let lwz = (frame as usize * 3).min(h);
            for hj in 0..lwz {
                if hj >= h { break; }
                
                for fx_step in 0..w / 12 {
                    let dg = fx_step * 12;
                    let seed = (hj.wrapping_mul(7919) + dg.wrapping_mul(104729) + frame as usize * 37) % 100;
                    if seed < 15 {
                        let c = if seed < 8 { '0' } else { '1' };
                        let na = (20 + (seed * 2)) as u32;
                        let color = 0xFF000000 | (na << 16) | ((na / 2) << 8) | (na / 4);
                        pf(&mut buf, w, h, dg, hj, c, color, 1);
                    }
                }
            }
            
            let hh = (frame / yr) as usize;
            let sn: usize = cmg.iter().map(|(_,_,j)| 16 * j + 12).sum();
            let mut y = if sn < h { (h - sn) / 2 } else { 20 };
            let mut abx = 0usize;
            for &(text, color, scale) in cmg.iter() {
                let gr = text.len() * 8 * scale;
                let am = if gr < w { (w - gr) / 2 } else { 0 };
                
                if !text.is_empty() {
                    draw_rect(&mut buf, w, h, am.saturating_sub(4), y.saturating_sub(2),
                        gr + 8, 16 * scale + 4, 0xCC000000);
                }
                for (i, c) in text.chars().enumerate() {
                    if abx + i >= hh { break; }
                    pf(&mut buf, w, h, am + i * 8 * scale, y, c, color, scale);
                }
                abx += text.len();
                y += 16 * scale + 12;
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    {
        let yr = 2u32;
        let cmg: [(&str, u32, usize); 5] = [
            ("But nobody knows", 0xFFCCCCCC, 3),
            ("what's inside it.", 0xFFCCCCCC, 3),
            ("", 0xFF000000, 1),
            ("Not even the people", 0xFFFF4444, 2),
            ("who wrote it.", 0xFFFF4444, 2),
        ];
        let vu: usize = cmg.iter().map(|(t,_,_)| t.len()).sum();
        let bjt = vu as u32 * yr;
        let odu = 60u32;
        let total_frames = bjt + odu;
        
        let dnk: [(&str, usize); 6] = [
            ("Source code: kernel/mm/init.c", 60),
            ("Author: CLASSIFIED", 140),
            ("Memory manager: UNKNOWN", 220),
            ("Security audit: NONE PERFORMED", 300),
            ("Bug count: UNTRACKED", 380),
            ("Last review: NEVER", 460),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            cub(&mut buf, w, h, frame);
            
            let fsq = 30usize;
            for &(line, ad) in dnk.iter() {
                if ad < h {
                    draw_text_at(&mut buf, w, h, fsq, ad, line, 0xFF445566, 1);
                }
            }
            
            if frame > bjt {
                let progress = frame - bjt;
                for (di, &(_line, ad)) in dnk.iter().enumerate() {
                    let delay = di as u32 * 6;
                    if progress > delay {
                        let ek = ((progress - delay) as usize * 12).min(280);
                        if ad < h {
                            draw_rect(&mut buf, w, h, fsq, ad.saturating_sub(2),
                                ek, 14, 0xFF000000);
                            if ek > 80 {
                                draw_text_at(&mut buf, w, h, fsq + 4, ad,
                                    "REDACTED", 0xFFFF2222, 1);
                            }
                        }
                    }
                }
            }
            
            let hh = (frame / yr) as usize;
            let pio = w / 2 + 20;
            let sn: usize = cmg.iter().map(|(_,_,j)| 16 * j + 12).sum();
            let mut y = if sn < h { (h - sn) / 2 } else { 20 };
            let mut abx = 0usize;
            for &(text, color, scale) in cmg.iter() {
                let am = pio;
                for (i, c) in text.chars().enumerate() {
                    if abx + i >= hh { break; }
                    pf(&mut buf, w, h, am + i * 8 * scale, y, c, color, scale);
                }
                abx += text.len();
                y += 16 * scale + 12;
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    {
        let hgp: [(&str, u32, u32); 4] = [
            ("Windows",  50_000_000, 0xFFFF4444),
            ("macOS",    30_000_000, 0xFFFFAA22),
            ("Linux",    28_000_000, 0xFFFF8800),
            ("TrustOS",     120_000, 0xFF00FF88),
        ];
        let sh = 50_000_000u32;
        let ctv = w * 3 / 5;
        let fif = 40usize;
        let hgt = 80usize;
        let start_y = h / 2 - (hgp.len() * hgt) / 2;
        let bhn = 40usize;

        let mut guu: i32 = 0;
        let mut fap: i32 = 0;

        for frame in 0..160u32 {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            hhp(&mut buf, w, h, frame);

            let progress = if frame < 30 { 0u32 }
                else { ((frame - 30) * 100 / 70).min(100) };

            
            if frame > 100 && frame < 130 {
                let seed = frame.wrapping_mul(7919) as i32;
                guu = (seed % 7) - 3;
                fap = ((seed / 7) % 5) - 2;
            } else {
                guu = 0;
                fap = 0;
            }

            draw_text_centered(&mut buf, w, h,
                (30i32 + fap) as usize,
                "Lines of Code per OS", 0xFFFFFFFF, 3);

            for (i, &(name, val, color)) in hgp.iter().enumerate() {
                let y = ((start_y + i * hgt) as i32 + fap).max(0) as usize;
                let heb = (bhn as i32 + guu).max(0) as usize;
                draw_text_at(&mut buf, w, h, heb, y + 10,
                    name, 0xFFFFFFFF, 2);
                let pv = (heb + 170).min(w.saturating_sub(10));
                draw_rect(&mut buf, w, h, pv, y,
                    ctv, fif, 0xFF111122);
                let maj = (val as usize * ctv) / sh as usize;
                let pdi = maj.max(12);
                let cwc = pdi * progress as usize / 100;
                draw_rect(&mut buf, w, h, pv, y,
                    cwc, fif, color);

                
                if i == 3 && frame > 100 && frame < 110 {
                    let flash = 0xFF88FFAA;
                    draw_rect(&mut buf, w, h, pv, y,
                        cwc + 4, fif + 4, flash);
                }

                if frame > 70 {
                    let label = if val >= 1_000_000 {
                        alloc::format!("{}M", val / 1_000_000)
                    } else {
                        alloc::format!("{}K", val / 1000)
                    };
                    draw_text_at(&mut buf, w, h, pv + cwc + 10,
                        y + 10, &label, 0xFFFFFFFF, 2);
                }
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("50 million vs 120 thousand.", 0xFFFFFFFF, 3),
          ("", 0xFF000000, 1),
          ("Like comparing a city", 0xFFCCCCCC, 2),
          ("to a single house.", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("Except the house", 0xFFCCCCCC, 2),
          ("does everything.", 0xFF00FF88, 3)],
        50, 80, 2);
    vi(&mut buf, w, h, &ev);

    
    
    
    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("ACT III", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("The Solution", 0xFF00CC66, 3)],
        50, 30, 4);
    vi(&mut buf, w, h, &ev);

    
    {
        let yr = 2u32;
        let wq = "What if one person";
        let aiw = "could understand ALL of it?";
        let vu = wq.len() + aiw.len();
        let total_frames = vu as u32 * yr + 60;
        
        let obs: [(i32, i32); 16] = [
            (4,0),(-4,0),(0,4),(0,-4),(3,3),(-3,3),(3,-3),(-3,-3),
            (4,1),(4,-1),(-4,1),(-4,-1),(1,4),(1,-4),(-1,4),(-1,-4),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            cud(&mut buf, w, h, frame);
            
            let cx = w / 2;
            let u = h / 2;
            
            let phase = (frame % 40) as u32;
            let kq = if phase < 20 { phase * 4 } else { (40 - phase) * 4 };
            
            let ddd = 40 + (frame / 2) as i32;
            for &(rdx, rdy) in obs.iter() {
                for step in 0..ddd {
                    let p = (cx as i32 + rdx * step).max(0) as usize;
                    let o = (u as i32 + rdy * step).max(0) as usize;
                    if p < w && o < h {
                        let att = (ddd - step) as u32 * 3;
                        let na = (kq + att).min(180);
                        let r = na;
                        let g = (na * 3 / 4).min(140);
                        let b = (na / 3).min(60);
                        let ku = buf[o * w + p];
                        let ajp = ((ku >> 16) & 0xFF) + r;
                        let cir = ((ku >> 8) & 0xFF) + g;
                        let bsd = (ku & 0xFF) + b;
                        buf[o * w + p] = 0xFF000000
                            | (ajp.min(255) << 16)
                            | (cir.min(255) << 8)
                            | bsd.min(255);
                    }
                }
            }
            
            let bsz = 15 + (kq / 4) as usize;
            for ad in 0..bsz {
                for dx in 0..bsz {
                    if dx * dx + ad * ad < bsz * bsz {
                        for &(am, ak) in &[(cx+dx, u+ad),(cx+dx, u.wrapping_sub(ad)),
                                           (cx.wrapping_sub(dx), u+ad),
                                           (cx.wrapping_sub(dx), u.wrapping_sub(ad))] {
                            if am < w && ak < h {
                                buf[ak * w + am] = 0xFFFFFFCC;
                            }
                        }
                    }
                }
            }
            
            let hh = (frame / yr) as usize;
            let scale = 3usize;
            let y1 = h / 4;
            let y2 = y1 + 16 * scale + 12;
            let avm = wq.len() * 8 * scale;
            let wn = if avm < w { (w - avm) / 2 } else { 0 };
            for (i, c) in wq.chars().enumerate() {
                if i >= hh { break; }
                pf(&mut buf, w, h, wn + i * 8 * scale, y1, c, 0xFFFFFFFF, scale);
            }
            if hh > wq.len() {
                let apj = aiw.len() * 8 * scale;
                let tq = if apj < w { (w - apj) / 2 } else { 0 };
                let ua = hh - wq.len();
                for (i, c) in aiw.chars().enumerate() {
                    if i >= ua { break; }
                    pf(&mut buf, w, h, tq + i * 8 * scale, y2, c, 0xFF00FF88, scale);
                }
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    {
        let stats: [(&str, &str, u32, u32); 4] = [
            ("", "lines of code", 120_000, 0xFF00FF88),
            ("", "author", 1, 0xFFFFFFFF),
            ("", "secrets", 0, 0xFFFFFFFF),
            ("100%", "Rust.  0% C.", 0, 0xFFFF7744),
        ];
        let total_frames = 140u32;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            cud(&mut buf, w, h, frame);
            
            draw_text_centered(&mut buf, w, h, 40, "TrustOS", 0xFF00FF88, 6);
            
            let progress = if frame < 20 { 0u32 }
                else { ((frame - 20) * 100 / 80).min(100) };
            let myp = h / 2 - 40;
            for (si, &(nm, asi, target, color)) in stats.iter().enumerate() {
                let y = myp + si * 48;
                let scale = 2usize;
                if target > 0 {
                    let current = (target as u64 * progress as u64 / 100) as u32;
                    let rw = if current >= 1000 {
                        alloc::format!("{},{:03}", current / 1000, current % 1000)
                    } else {
                        alloc::format!("{}", current)
                    };
                    let xo = alloc::format!("{} {}", rw, asi);
                    draw_text_centered(&mut buf, w, h, y, &xo, color, scale);
                } else if !nm.is_empty() {
                    let xo = alloc::format!("{} {}", nm, asi);
                    if frame > 60 + si as u32 * 15 {
                        draw_text_centered(&mut buf, w, h, y, &xo, color, scale);
                    }
                } else {
                    let xo = alloc::format!("0 {}", asi);
                    if frame > 60 + si as u32 * 15 {
                        draw_text_centered(&mut buf, w, h, y, &xo, color, scale);
                    }
                }
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
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
        let cza = 100usize;
        let aju = 40usize;

        for reveal in 0..features.len() {
            for frame in 0..30u32 {
                if let Some(k) = crate::keyboard::kr() {
                    if k == 0x1B { break; }
                }
                hht(&mut buf, w, h, frame + reveal as u32 * 30);
                draw_text_centered(&mut buf, w, h, 30,
                    "All of this in 10 MB:", 0xFFFFFFFF, 3);
                for (axb, &(name, desc, color)) in features.iter().enumerate() {
                    if axb > reveal { break; }
                    let col = axb % cols;
                    let row = axb / cols;
                    let dg = aju + col * (cell_w + 40);
                    let hj = cza + row * (cell_h + 20);

                    
                    let glow = if axb == reveal && frame < 15 {
                        (15 - frame) * 12
                    } else { 0 };
                    let glow = glow as u32;

                    
                    draw_rect(&mut buf, w, h, dg, hj, cell_w, cell_h, 0xFF0E0E1E);

                    
                    if glow > 0 {
                        let bmk = 0xFF000000 | (glow.min(255) << 16) | (glow.min(255) << 8) | glow.min(255);
                        draw_rect(&mut buf, w, h, dg.saturating_sub(2), hj.saturating_sub(2),
                            cell_w + 4, 3, bmk);
                        draw_rect(&mut buf, w, h, dg.saturating_sub(2), hj + cell_h,
                            cell_w + 4, 3, bmk);
                        draw_rect(&mut buf, w, h, dg.saturating_sub(2), hj,
                            3, cell_h, bmk);
                        draw_rect(&mut buf, w, h, dg + cell_w, hj,
                            3, cell_h, bmk);
                    }

                    
                    draw_rect(&mut buf, w, h, dg, hj, cell_w, 3, color);
                    draw_rect(&mut buf, w, h, dg, hj + cell_h - 1, cell_w, 1, 0xFF222244);
                    draw_text_at(&mut buf, w, h, dg + 10, hj + 12,
                        name, color, 2);
                    draw_text_at(&mut buf, w, h, dg + 10, hj + 42,
                        desc, 0xFFAAAAAA, 1);
                }
                ev(&buf, w, h);
                crate::cpu::tsc::hq(vj);
            }
        }
        crate::cpu::tsc::hq(1500);
    }
    vi(&mut buf, w, h, &ev);

    
    
    
    
    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("ACT IV", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("The Proof", 0xFF00CC66, 3)],
        50, 30, 6);
    vi(&mut buf, w, h, &ev);

    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("When you visit a website,", 0xFFCCCCCC, 2),
          ("this is what happens", 0xFFCCCCCC, 2),
          ("inside TrustOS:", 0xFF00FF88, 2)],
        50, 60, 6);
    vi(&mut buf, w, h, &ev);

    
    {
        let stages: [(&str, u32); 5] = [
            ("App",     0xFF4488FF),
            ("TLS 1.3", 0xFFFF4444),
            ("TCP/IP",  0xFFFFAA22),
            ("Driver",  0xFF44FF88),
            ("Wire",    0xFF8888FF),
        ];
        let ae = stages.len();
        let abk = (w.saturating_sub(80)) / (ae + 1);
        let det = 60usize;
        let clx = h / 2 - det / 2;

        for gd in 0..2u32 {
            let label = if gd == 0 { "Sending packet..." }
                        else         { "Response received!" };
            let nvc = if gd == 0 { 0xFF00FF88 } else { 0xFF44DDFF };

            for frame in 0..150u32 {
                if let Some(k) = crate::keyboard::kr() {
                    if k == 0x1B { break; }
                }
                ctz(&mut buf, w, h, frame + gd * 150);

                draw_text_centered(&mut buf, w, h, 30,
                    label, 0xFFFFFFFF, 2);

                for (si, &(name, color)) in stages.iter().enumerate() {
                    let am = 40 + si * abk;
                    let fv = abk.saturating_sub(15);
                    draw_rect(&mut buf, w, h, am, clx, fv, det,
                              0xFF0E1020);
                    draw_rect(&mut buf, w, h, am, clx, fv, 3, color);
                    draw_rect(&mut buf, w, h, am, clx + det - 1, fv, 1, 0xFF222244);
                    let bu = am + fv / 2 - name.len() * 4;
                    draw_text_at(&mut buf, w, h, bu, clx + 22,
                        name, color, 1);
                    if si < ae - 1 {
                        let ax = am + fv;
                        draw_rect(&mut buf, w, h, ax,
                            clx + det / 2 - 1, 15, 3, 0xFF334455);
                        
                        draw_rect(&mut buf, w, h, ax + 12,
                            clx + det / 2 - 3, 3, 7, 0xFF556677);
                    }
                }

                
                let progress = (frame * 100 / 120).min(100) as usize;
                let eck = (ae - 1) * abk;
                let nvd = if gd == 0 {
                    eck * progress / 100
                } else {
                    eck - eck * progress / 100
                };
                let ews = 40 + nvd + abk / 2 - 8;
                let gnf = clx + det + 18;
                
                for wr in 1..6u32 {
                    let bu = if gd == 0 { ews.saturating_sub(wr as usize * 6) }
                             else { ews + wr as usize * 6 };
                    let alpha = (60u32.saturating_sub(wr * 12)).min(255);
                    let wo = 0xFF000000 | (alpha / 4 << 16) | (alpha << 8) | (alpha / 3);
                    draw_rect(&mut buf, w, h, bu, gnf + 2, 8, 12, wo);
                }
                draw_rect(&mut buf, w, h, ews, gnf, 16, 16, nvc);
                draw_text_at(&mut buf, w, h,
                    ews.saturating_sub(16), gnf + 20,
                    "packet", 0xFFCCCCCC, 1);

                ev(&buf, w, h);
                crate::cpu::tsc::hq(vj);
            }
        }
    }
    vi(&mut buf, w, h, &ev);

    
    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("Every step is visible.", 0xFFFFFFFF, 3),
          ("Every byte is readable.", 0xFFFFFFFF, 3),
          ("", 0xFF000000, 1),
          ("Nothing is hidden.", 0xFF00FF88, 4)],
        50, 80, 4);
    vi(&mut buf, w, h, &ev);

    
    
    
    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("ACT V", 0xFFFFDD88, 5),
          ("", 0xFF000000, 1),
          ("The Future", 0xFFFFAA44, 3)],
        50, 30, 7);
    vi(&mut buf, w, h, &ev);

    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("You deserve to understand", 0xFFFFFFFF, 3),
          ("your own machine.", 0xFFFFFFFF, 3)],
        50, 60, 7);
    vi(&mut buf, w, h, &ev);

    
    {
        let yr = 2u32;
        let wq = "Computing is not magic.";
        let aiw = "It's math and logic.";
        let vu = wq.len() + aiw.len();
        let total_frames = vu as u32 * yr + 80;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            djf(&mut buf, w, h, frame);
            
            let oul = (frame * 2).min(200) as usize;
            for si in 0..oul {
                let seed = si.wrapping_mul(2654435761).wrapping_add(frame as usize * 131);
                let am = seed % w;
                let ak = (seed / w) % h;
                
                let kxg = frame > 60;
                let (dg, hj) = if kxg {
                    
                    let cx = w / 2;
                    let u = h / 2;
                    let orl = (si % 4);
                    match orl {
                        0 => {  
                            let fe = cx.saturating_sub(100) + (si * 3) % 200;
                            (fe, u.saturating_sub(60))
                        }
                        1 => {  
                            let fe = cx.saturating_sub(100) + (si * 7) % 200;
                            (fe, u + 60)
                        }
                        2 => {  
                            let ly = u.saturating_sub(60) + (si * 5) % 120;
                            (cx.saturating_sub(100), ly)
                        }
                        _ => {  
                            let ly = u.saturating_sub(60) + (si * 11) % 120;
                            (cx + 100, ly)
                        }
                    }
                } else {
                    (am, ak)
                };
                if dg < w && hj < h {
                    let na = (100 + (seed % 155)) as u32;
                    buf[hj * w + dg] = 0xFF000000 | (na << 16) | (na << 8) | na;
                    if dg + 1 < w { buf[hj * w + dg + 1] = 0xFF000000 | (na << 16) | ((na / 2) << 8); }
                }
            }
            
            let hh = (frame / yr) as usize;
            let scale = 3usize;
            let y1 = h / 4;
            let y2 = y1 + 16 * scale + 16;
            let avm = wq.len() * 8 * scale;
            let wn = if avm < w { (w - avm) / 2 } else { 0 };
            for (i, c) in wq.chars().enumerate() {
                if i >= hh { break; }
                pf(&mut buf, w, h, wn + i * 8 * scale, y1, c, 0xFFCCCCCC, scale);
            }
            if hh > wq.len() {
                let apj = aiw.len() * 8 * scale;
                let tq = if apj < w { (w - apj) / 2 } else { 0 };
                let ua = hh - wq.len();
                for (i, c) in aiw.chars().enumerate() {
                    if i >= ua { break; }
                    pf(&mut buf, w, h, tq + i * 8 * scale, y2, c, 0xFFFFDD88, scale);
                }
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    {
        let yr = 2u32;
        let text = "TrustOS proves it.";
        let bjt = text.len() as u32 * yr;
        let ohd = 60u32;
        let total_frames = bjt + ohd;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::kr() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            djf(&mut buf, w, h, frame);
            let cx = w / 2;
            let u = h / 2;
            
            if frame > bjt / 2 {
                let ohf = frame.saturating_sub(bjt / 2);
                let nlv = 5u32;
                for dk in 0..nlv {
                    let radius = (ohf as usize * 4).saturating_sub(dk as usize * 30);
                    if radius == 0 || radius > w { continue; }
                    let na = (200u32.saturating_sub(dk * 30)).min(255);
                    let ixl = 0xFF000000 | ((na / 3) << 16) | (na << 8) | ((na * 2 / 3).min(255));
                    
                    let ixk = radius * radius;
                    let boi = radius.saturating_sub(3);
                    let exn = boi * boi;
                    let ajb = u.saturating_sub(radius);
                    let bkg = (u + radius).min(h);
                    for o in ajb..bkg {
                        let ad = if o >= u { o - u } else { u - o };
                        let eky = ad * ad;
                        
                        if eky > ixk { continue; }
                        let lmw = ixk - eky;
                        let lmx = if exn > eky { exn - eky } else { 0 };
                        
                        let mut bzb = 0usize;
                        while (bzb + 1) * (bzb + 1) <= lmw { bzb += 1; }
                        let mut dob = 0usize;
                        while (dob + 1) * (dob + 1) <= lmx { dob += 1; }
                        
                        for dx in dob..=bzb {
                            let p = cx + dx;
                            if p < w { buf[o * w + p] = ixl; }
                        }
                        
                        for dx in dob..=bzb {
                            let p = cx.wrapping_sub(dx);
                            if p < w { buf[o * w + p] = ixl; }
                        }
                    }
                }
            }
            
            let hh = (frame / yr) as usize;
            let scale = 4usize;
            let gr = text.len() * 8 * scale;
            let am = if gr < w { (w - gr) / 2 } else { 0 };
            let ty = h / 2 - 8 * scale;
            for (i, c) in text.chars().enumerate() {
                if i >= hh { break; }
                pf(&mut buf, w, h, am + i * 8 * scale, ty, c, 0xFF00FF88, scale);
            }
            ev(&buf, w, h);
            crate::cpu::tsc::hq(vj);
        }
    }
    vi(&mut buf, w, h, &ev);

    
    
    
    uq(&mut buf);
    boy(&mut buf, w, h, &mut kk, &la,
        &[("Trust the code.", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("Rust is the reason.", 0xFFFF7744, 3),
          ("", 0xFF000000, 1),
          ("github.com/nathan237/TrustOS", 0xFFCCCCCC, 2)],
        60, 150, 8);
    vi(&mut buf, w, h, &ev);

    
    uq(&mut buf);
    ev(&buf, w, h);
    if !pu {
        crate::framebuffer::pr(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[FILM] TrustOS Film complete");
}


pub(super) fn ktf(args: &[&str]) {
    if args.is_empty() {
        crate::n!(C_, "TrustView -- Binary Analysis Viewer");
        crate::n!(B_, "Usage: trustview <file>");
        crate::n!(B_, "       tv <file>");
        crate::println!("");
        crate::println!("Opens an ELF binary in the desktop binary viewer.");
        crate::println!("Panels: Navigation | Hex | Disassembly | Info/Xrefs");
        crate::println!("");
        crate::println!("Quick analysis (terminal only):");
        crate::println!("  trustview info <file>  -- Print binary summary");
        return;
    }

    let je = args[0];

    if je == "info" {
        
        let path = args.get(1).copied().unwrap_or("");
        if path.is_empty() {
            crate::n!(A_, "Usage: trustview info <file>");
            return;
        }
        match crate::binary_analysis::hfe(path) {
            Ok(bqp) => {
                crate::n!(C_, "=== TrustView Analysis: {} ===", path);
                crate::println!("{}", bqp.summary());
                crate::println!("");
                
                crate::n!(C_, "Detected Functions ({}):", bqp.xrefs.functions.len());
                for func in bqp.xrefs.functions.iter().take(20) {
                    let name = if func.name.is_empty() {
                        alloc::format!("sub_{:X}", func.entry)
                    } else {
                        func.name.clone()
                    };
                    crate::println!("  0x{:08X} {} ({} insns, {} blocks)", 
                        func.entry, name, func.instruction_count, func.basic_blocks);
                }
                if bqp.xrefs.functions.len() > 20 {
                    crate::println!("  ... and {} more", bqp.xrefs.functions.len() - 20);
                }
            },
            Err(e) => crate::n!(A_, "Error: {}", e),
        }
        return;
    }

    
    let path = je;
    use crate::desktop::S;
    let mut desktop = S.lock();
    match desktop.open_binary_viewer(path) {
        Ok(id) => {
            crate::n!(B_, "TrustView opened: {} (window #{})", path, id);
        },
        Err(e) => {
            crate::n!(A_, "Failed to open '{}': {}", path, e);
        }
    }
}


pub(super) fn kre(args: &[&str]) {
    let path = args.get(0).copied().unwrap_or("");

    if path.is_empty() || path == "help" || path == "-h" {
        crate::n!(C_, "+--------------------------------------------------------------+");
        crate::n!(C_, "|     TrustOS RISC-V Universal Translation Layer               |");
        crate::n!(C_, "|  Run x86_64/ARM64/MIPS binaries via RISC-V IR translation    |");
        crate::n!(C_, "+--------------------------------------------------------------+");
        crate::println!();
        crate::println!("Usage: rv-xlat <elf-binary>");
        crate::println!();
        crate::println!("Supports: x86_64, AArch64, RISC-V (passthrough), MIPS64");
        crate::println!("Pipeline: Source ISA → RISC-V IR → Interpreter → Syscall xlat");
        crate::println!();
        crate::println!("Examples:");
        crate::println!("  rv-xlat /alpine/bin/true    # Run x86_64 binary via RV translation");
        crate::println!("  rv-xlat /bin/hello_arm      # Run ARM64 binary via RV translation");
        crate::println!();
        crate::println!("See also: rv-disasm (show RISC-V IR without executing)");
        return;
    }

    
    let data = match crate::ramfs::bh(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::n!(A_, "Cannot read file: {}", path);
            return;
        }
    };

    
    match crate::riscv_translator::frv(&data) {
        Some(arch) => {
            crate::n!(B_, "[RV-XLAT] Detected: {} binary ({} bytes)", arch.name(), data.len());
        }
        None => {
            crate::n!(A_, "Not a valid ELF binary");
            return;
        }
    }

    
    match crate::riscv_translator::pne(&data) {
        Ok(code) => {
            crate::println!();
            if code == 0 {
                crate::n!(B_, "[RV-XLAT] Process exited successfully (code 0)");
            } else {
                crate::n!(D_, "[RV-XLAT] Process exited with code {}", code);
            }
        }
        Err(e) => {
            crate::n!(A_, "[RV-XLAT] Error: {}", e);
        }
    }
}


pub(super) fn krd(args: &[&str]) {
    let path = args.get(0).copied().unwrap_or("");

    if path.is_empty() || path == "help" {
        crate::println!("Usage: rv-disasm <elf-binary>");
        crate::println!("Shows the RISC-V IR translation of a binary");
        return;
    }

    let data = match crate::ramfs::bh(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::n!(A_, "Cannot read file: {}", path);
            return;
        }
    };

    match crate::riscv_translator::jon(&data) {
        Ok(output) => {
            crate::println!("{}", output);
        }
        Err(e) => {
            crate::n!(A_, "Error: {}", e);
        }
    }
}


pub(super) fn ksz(args: &[&str]) {
    let je = args.get(0).copied().unwrap_or("help");
    
    match je {
        "help" | "-h" | "--help" => {
            crate::n!(C_, "+--------------------------------------------------------------+");
            crate::n!(C_, "|           TrustOS Binary Transpiler                          |");
            crate::n!(C_, "|       Analyze Linux binaries ? Generate Rust code            |");
            crate::n!(C_, "+--------------------------------------------------------------+");
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
            pni();
        }
        
        "analyze" | "a" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            fdo(path, true, true, true);
        }
        
        "disasm" | "d" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            fdo(path, true, false, false);
        }
        
        "rust" | "r" | "gen" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            fdo(path, false, false, true);
        }
        
        "strings" | "s" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            pnk(path);
        }
        
        "syscalls" | "sys" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            pnl(path);
        }
        
        "scan" => {
            pnj();
        }
        
        "batch" => {
            pnh();
        }
        
        "run" | "exec" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/true");
            jop(path, false);
        }
        
        "execute" | "x" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/true");
            jop(path, true);
        }

        "audit" | "stats" => {
            pnf();
        }

        _ => {
            
            if je.starts_with('/') || je.contains('.') {
                fdo(je, true, true, true);
            } else {
                crate::n!(A_, "Unknown subcommand: {}", je);
                crate::println!("Use 'transpile help' for usage");
            }
        }
    }
}

fn fdo(path: &str, show_disasm: bool, show_strings: bool, show_rust: bool) {
    crate::n!(C_, "Analyzing binary: {}", path);
    crate::println!();
    
    
    let data = match crate::ramfs::bh(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::n!(A_, "Cannot read file: {}", path);
            return;
        }
    };
    
    crate::println!("File size: {} bytes ({} KB)", data.len(), data.len() / 1024);
    
    
    match crate::transpiler::bks(&data) {
        Some(analysis) => {
            crate::n!(B_, "ELF analysis successful!");
            crate::println!();
            crate::println!("Entry point: 0x{:x}", analysis.entry_point);
            crate::println!("Functions: {}", analysis.functions.len());
            crate::println!("Syscalls: {:?}", analysis.syscalls_used);
            crate::println!("Strings: {}", analysis.strings.len());
            crate::println!();
            
            if show_disasm {
                if let Some(func) = analysis.functions.first() {
                    crate::n!(D_, "=== Disassembly ({} instructions) ===", func.instructions.len());
                    let transpiler = crate::transpiler::Transpiler::new(func.instructions.clone());
                    crate::println!("{}", transpiler.generate_listing());
                }
            }
            
            if show_strings && !analysis.strings.is_empty() {
                crate::n!(D_, "=== Strings (first 20) ===");
                for (addr, j) in analysis.strings.iter().take(20) {
                    crate::println!("0x{:06x}: \"{}\"", addr, j);
                }
                crate::println!();
            }
            
            if show_rust {
                crate::n!(D_, "=== Generated Rust Code ===");
                crate::println!("{}", analysis.rust_code);
            }
        }
        None => {
            crate::n!(A_, "Not a valid ELF binary");
        }
    }
}

fn pnk(path: &str) {
    let data = match crate::ramfs::bh(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::n!(A_, "Cannot read file: {}", path);
            return;
        }
    };
    
    
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
    
    crate::n!(C_, "Strings in {}: {} found", path, strings.len());
    crate::println!();
    
    for (addr, j) in strings.iter() {
        
        if j.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
            crate::println!("0x{:06x}: {}", addr, j);
        }
    }
}

fn pnl(path: &str) {
    let data = match crate::ramfs::bh(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::n!(A_, "Cannot read file: {}", path);
            return;
        }
    };
    
    match crate::transpiler::bks(&data) {
        Some(analysis) => {
            crate::n!(C_, "Syscalls in {}", path);
            crate::println!();
            
            for func in &analysis.functions {
                if !func.syscalls.is_empty() {
                    crate::println!("Function {} @ 0x{:x}:", func.name, func.address);
                    for dr in &func.syscalls {
                        crate::println!("  0x{:x}: {} (#{})", dr.address, dr.name, dr.number);
                    }
                }
            }
            
            crate::println!();
            crate::println!("Summary: {:?}", analysis.syscalls_used);
        }
        None => {
            crate::n!(A_, "Not a valid ELF binary");
        }
    }
}

fn pnj() {
    crate::n!(C_, "Scanning /alpine/bin for binaries...");
    crate::println!();
    
    let entries = match crate::ramfs::bh(|fs| fs.ls(Some("/alpine/bin"))) {
        Ok(e) => e,
        Err(_) => {
            crate::n!(A_, "Cannot access /alpine/bin - run 'alpine test' first");
            return;
        }
    };
    
    let mut gvf = alloc::vec::Vec::new();
    let mut dlf = alloc::vec::Vec::new();
    
    for (name, _, bek) in entries {
        let path = alloc::format!("/alpine/bin/{}", name);
        
        if let Ok(data) = crate::ramfs::bh(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::bks(&data) {
                let syscall_count = analysis.syscalls_used.len();
                let gcz = analysis.functions.first().map(|f| f.instructions.len()).unwrap_or(0);
                
                if syscall_count <= 3 && gcz < 100 {
                    gvf.push((name.clone(), syscall_count, gcz));
                } else {
                    dlf.push((name.clone(), syscall_count, gcz));
                }
            }
        }
    }
    
    crate::n!(B_, "Simple binaries ({} - easy to transpile):", gvf.len());
    for (name, dr, instr) in &gvf {
        crate::println!("  {} - {} syscalls, {} instructions", name, dr, instr);
    }
    
    crate::println!();
    crate::n!(D_, "Complex binaries ({} - need more work):", dlf.len());
    for (name, dr, instr) in dlf.iter().take(10) {
        crate::println!("  {} - {} syscalls, {} instructions", name, dr, instr);
    }
    if dlf.len() > 10 {
        crate::println!("  ... and {} more", dlf.len() - 10);
    }
}

fn pnh() {
    crate::n!(C_, "Batch transpiling simple binaries...");
    crate::println!();
    
    
    let osz = ["true", "false", "pwd", "whoami", "hostname", "uname", "echo", "yes"];
    
    for name in &osz {
        let path = alloc::format!("/alpine/bin/{}", name);
        
        if let Ok(data) = crate::ramfs::bh(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::bks(&data) {
                crate::n!(B_, "=== {} ===", name);
                crate::println!("Syscalls: {:?}", analysis.syscalls_used);
                crate::println!();
                crate::println!("{}", analysis.rust_code);
                crate::println!();
            } else {
                crate::n!(D_, "{}: not found or not ELF", name);
            }
        } else {
            crate::n!(D_, "{}: not available", name);
        }
    }
}


fn pnf() {
    use alloc::collections::BTreeMap;
    
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::n!(C_, "|         TrustOS Transpiler - Alpine Syscall Audit            |");
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    let entries = match crate::ramfs::bh(|fs| fs.ls(Some("/alpine/bin"))) {
        Ok(e) => e,
        Err(_) => {
            crate::n!(A_, "Cannot access /alpine/bin - run 'linux extract' first");
            return;
        }
    };
    
    
    let mut jkx: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut fce: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut djj = 0;
    let mut hvi = 0;
    let mut jju = 0;
    let mut jnw = 0usize;
    
    crate::println!("Scanning {} files...", entries.len());
    crate::println!();
    
    for (name, _, _) in &entries {
        let path = alloc::format!("/alpine/bin/{}", name);
        djj += 1;
        
        if let Ok(data) = crate::ramfs::bh(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::bks(&data) {
                hvi += 1;
                
                
                for func in &analysis.functions {
                    jnw += func.instructions.len();
                    
                    for dr in &func.syscalls {
                        *jkx.entry(dr.name).or_insert(0) += 1;
                        fce.insert(dr.name, dr.number);
                    }
                }
                
                
                let jus = analysis.syscalls_used.iter().all(|dr| {
                    matches!(*dr, "exit" | "exit_group" | "write" | "read" | "open" | "close" |
                            "getcwd" | "uname" | "getpid" | "getuid" | "getgid" | "geteuid" | "getegid")
                });
                if jus && !analysis.syscalls_used.is_empty() {
                    jju += 1;
                }
            }
        }
    }
    
    crate::n!(B_, "--- Statistics ---");
    crate::println!("Files scanned:      {}", djj);
    crate::println!("Valid ELF binaries: {}", hvi);
    crate::println!("Fully supported:    {}", jju);
    crate::println!("Total instructions: {}", jnw);
    crate::println!();
    
    
    let mut acq: Vec<_> = jkx.iter().collect();
    acq.sort_by(|a, b| b.1.cmp(a.1));
    
    crate::n!(C_, "--- Syscalls by Frequency ---");
    crate::println!("{:<20} {:>8} {:>8} {}", "Syscall", "Count", "Number", "Status");
    crate::println!("{}", "-".repeat(50));
    
    for (name, count) in &acq {
        let num = fce.get(*name).copied().unwrap_or(0);
        let level = crate::transpiler::jla(num);
        let status = match level {
            "Full" => "Full",
            "Partial" => "Partial",
            "Stub" => "Stub",
            _ => "Missing",
        };
        crate::println!("{:<20} {:>8} {:>8} {}", name, count, num, status);
    }
    
    crate::println!();
    crate::n!(D_, "--- Missing Syscalls (need implementation) ---");
    let ghu: Vec<_> = acq.iter()
        .filter(|(name, _)| {
            let num = fce.get(*name).copied().unwrap_or(999);
            crate::transpiler::jla(num) == "None"
        })
        .collect();
    
    for (name, count) in &ghu {
        let num = fce.get(*name).copied().unwrap_or(0);
        crate::println!("  {} (#{}) - used {} times", name, num, count);
    }
    
    if ghu.is_empty() {
        crate::n!(B_, "  All syscalls are at least partially implemented!");
    }
    
    crate::println!();
    crate::n!(C_, "--- Recommendation ---");
    crate::println!("To improve transpiler coverage, implement these syscalls in order:");
    let priority: Vec<_> = ghu.iter().take(5).collect();
    for (i, (name, count)) in priority.iter().enumerate() {
        crate::println!("  {}. {} (used {} times)", i + 1, name, count);
    }
}


pub(super) fn kzm() {
    
    let _ = crate::ramfs::bh(|fs| {
        let _ = fs.mkdir("/alpine");
        let _ = fs.mkdir("/alpine/bin");
        Ok::<(), ()>(())
    });
    
    fn ggm(code: &[u8]) -> alloc::vec::Vec<u8> {
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
    
    let fjd: [(&str, &[u8]); 7] = [
        ("true", &[0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("false", &[0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, 0x0f, 0x05]),
        ("getpid", &[0x48, 0xc7, 0xc0, 0x27, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("getuid", &[0x48, 0xc7, 0xc0, 0x66, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("uname", &[0x48, 0xc7, 0xc0, 0x3f, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("echo", &[0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, 0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, 0x48, 0x31, 0xf6, 0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("pwd", &[0x48, 0xc7, 0xc0, 0x4f, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x48, 0x31, 0xf6, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
    ];
    
    let mut dlq = 0;
    for (name, code) in &fjd {
        let elf = ggm(code);
        let path = alloc::format!("/alpine/bin/{}", name);
        if crate::ramfs::bh(|fs| { let _ = fs.touch(&path); fs.write_file(&path, &elf) }).is_ok() {
            dlq += 1;
        }
    }
    crate::n!(B_, "      Created {} binaries", dlq);
}


pub(super) fn hos() {
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::n!(C_, "|       Creating Test Binaries for Transpiler                  |");
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    let _ = crate::ramfs::bh(|fs| {
        let _ = fs.mkdir("/alpine");
        let _ = fs.mkdir("/alpine/bin");
        Ok::<(), ()>(())
    });
    
    
    fn ggm(code: &[u8]) -> alloc::vec::Vec<u8> {
        let mut elf = alloc::vec![
            
            0x7fu8, 0x45, 0x4c, 0x46,  
            0x02, 0x01, 0x01, 0x00,     
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x02, 0x00,                 
            0x3e, 0x00,                 
            0x01, 0x00, 0x00, 0x00,     
            0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x00, 0x00, 0x00,     
            0x40, 0x00,                 
            0x38, 0x00,                 
            0x01, 0x00,                 
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            
            
            0x01, 0x00, 0x00, 0x00,     
            0x05, 0x00, 0x00, 0x00,     
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        ];
        elf.extend_from_slice(code);
        while elf.len() < 256 {
            elf.push(0);
        }
        elf
    }
    
    
    let fjd: [(&str, &[u8], &str); 7] = [
        ("true", &[
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xff,                          
            0x0f, 0x05,                                
        ], "exit(0)"),
        
        ("false", &[
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  
            0x0f, 0x05,                                
        ], "exit(1)"),
        
        ("getpid", &[
            0x48, 0xc7, 0xc0, 0x27, 0x00, 0x00, 0x00,  
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getpid+exit"),
        
        ("getuid", &[
            0x48, 0xc7, 0xc0, 0x66, 0x00, 0x00, 0x00,  
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getuid+exit"),
        
        ("uname", &[
            0x48, 0xc7, 0xc0, 0x3f, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xff,
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "uname+exit"),
        
        ("echo", &[
            0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,  
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xf6,                          
            0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00,  
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "write+exit"),
        
        ("pwd", &[
            0x48, 0xc7, 0xc0, 0x4f, 0x00, 0x00, 0x00,  
            0x48, 0x31, 0xff,
            0x48, 0x31, 0xf6,
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getcwd+exit"),
    ];
    
    let mut dlq = 0;
    for (name, code, desc) in &fjd {
        let elf = ggm(code);
        let path = alloc::format!("/alpine/bin/{}", name);
        
        let result = crate::ramfs::bh(|fs| {
            let _ = fs.touch(&path);
            fs.write_file(&path, &elf)
        });
        
        match result {
            Ok(_) => {
                crate::n!(B_, "? {} - {}", name, desc);
                dlq += 1;
            }
            Err(_) => {
                crate::n!(A_, "? {} - failed", name);
            }
        }
    }
    
    crate::println!();
    crate::n!(B_, "Created {} test binaries in /alpine/bin", dlq);
    crate::println!();
    crate::println!("Now run:");
    crate::println!("  transpile audit       - Analyze all syscalls");
    crate::println!("  transpile run /alpine/bin/true");
    crate::println!("  transpile analyze /alpine/bin/echo");
}


fn pni() {
    crate::n!(G_, "+--------------------------------------------------------------+");
    crate::n!(G_, "|         TrustOS Transpiler Demo - Built-in Test              |");
    crate::n!(G_, "+--------------------------------------------------------------+");
    crate::println!();
    
    
    
    
    
    
    
    crate::n!(C_, "Creating test binary: exit(0) program");
    crate::println!();
    
    
    
    #[rustfmt::skip]
    let frl: &[u8] = &[
        
        0x7F, b'E', b'L', b'F',  
        0x02,                     
        0x01,                     
        0x01,                     
        0x00,                     
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x02, 0x00,               
        0x3E, 0x00,               
        0x01, 0x00, 0x00, 0x00,   
        0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x00, 0x00, 0x00,   
        0x40, 0x00,               
        0x38, 0x00,               
        0x01, 0x00,               
        0x00, 0x00,               
        0x00, 0x00,               
        0x00, 0x00,               
        
        
        0x01, 0x00, 0x00, 0x00,   
        0x05, 0x00, 0x00, 0x00,   
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  
        
        
        
        
        
        
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00,  
        0x48, 0x31, 0xFF,                          
        0x0F, 0x05,                                
    ];
    
    crate::println!("Demo binary: {} bytes", frl.len());
    crate::println!("Code: mov rax, 60; xor rdi, rdi; syscall");
    crate::println!();
    
    
    match crate::transpiler::bks(&frl) {
        Some(analysis) => {
            crate::n!(B_, "? ELF Analysis Successful!");
            crate::println!();
            
            crate::n!(D_, "--- Binary Info ---");
            crate::println!("Entry point:  0x{:x}", analysis.entry_point);
            crate::println!("Functions:    {}", analysis.functions.len());
            crate::println!("Syscalls:     {:?}", analysis.syscalls_used);
            crate::println!();
            
            
            if let Some(func) = analysis.functions.first() {
                crate::n!(D_, "--- Disassembly ({} instructions) ---", func.instructions.len());
                let transpiler = crate::transpiler::Transpiler::new(func.instructions.clone());
                crate::println!("{}", transpiler.generate_listing());
            }
            
            
            crate::n!(D_, "--- Generated Rust Code ---");
            crate::println!("{}", analysis.rust_code);
            
            crate::n!(G_, "");
            crate::n!(G_, "? Transpiler test PASSED!");
            crate::println!();
            crate::println!("The transpiler successfully:");
            crate::println!("  1. Parsed ELF64 header");
            crate::println!("  2. Found executable segment");
            crate::println!("  3. Disassembled x86_64 code");
            crate::println!("  4. Detected syscall (sys_exit)");
            crate::println!("  5. Generated equivalent Rust code");
        }
        None => {
            crate::n!(A_, "? Failed to analyze demo binary");
        }
    }
    
    
    crate::println!();
    crate::n!(C_, "Saving demo binary to /tmp/demo_exit...");
    let dyg = crate::ramfs::bh(|fs| {
        let _ = fs.mkdir("/tmp");
        let _ = fs.touch("/tmp/demo_exit"); 
        fs.write_file("/tmp/demo_exit", frl)
    });
    match dyg {
        Ok(_) => {
            crate::n!(B_, "? Saved! You can now run:");
            crate::println!("  transpile analyze /tmp/demo_exit");
            crate::println!("  transpile rust /tmp/demo_exit");
        }
        Err(_) => {
            crate::n!(D_, "Could not save demo binary");
        }
    }
}


fn jop(path: &str, csi: bool) {
    use crate::transpiler::{bks, BinaryType};
    
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::n!(C_, "|           TrustOS Transpiler - Execute Binary                |");
    crate::n!(C_, "+--------------------------------------------------------------+");
    crate::println!();
    crate::println!("Binary: {}", path);
    
    
    let data = match super::network::exu(path) {
        Some(d) => d,
        None => {
            crate::n!(A_, "Error: Could not read file");
            return;
        }
    };
    
    if csi {
        crate::println!("Size: {} bytes", data.len());
    }
    
    
    let analysis = match bks(&data) {
        Some(a) => a,
        None => {
            crate::n!(A_, "Error: Not a valid ELF binary");
            return;
        }
    };
    
    if csi {
        crate::println!("Entry point: 0x{:x}", analysis.entry_point);
        crate::println!("Syscalls: {:?}", analysis.syscalls_used);
    }
    
    crate::println!();
    crate::n!(B_, "--- Executing transpiled binary ---");
    crate::println!();
    
    
    let exit_code = lsf(&analysis);
    
    crate::println!();
    crate::n!(C_, "---------------------------------------------------------------");
    crate::println!("Exit code: {}", exit_code);
}


fn lsf(analysis: &crate::transpiler::Hf) -> i32 {
    
    let syscalls = if let Some(func) = analysis.functions.first() {
        &func.syscalls
    } else {
        crate::n!(A_, "No functions found in binary");
        return 1;
    };
    
    
    for syscall in syscalls {
        match syscall.name {
            "exit" | "exit_group" => {
                let code = syscall.args.get(0).copied().unwrap_or(0) as i32;
                return code;
            }
            "write" => {
                let fd = syscall.args.get(0).copied().unwrap_or(1);
                if fd == 1 || fd == 2 {
                    
                    
                    
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
    
    
    0
}


pub(super) fn ktr(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");

    match je {
        "demo" => {
            let aoa = args.get(1).copied().unwrap_or("plasma");
            let fps = 30u64;
            let vj = 1000 / fps;

            crate::println!("=== TrustVideo Demo: {} ===", aoa);
            crate::println!("Rendering in real-time @ {}fps", fps);
            crate::println!("Press Q or ESC to stop");

            
            let dy = crate::framebuffer::width();
            let dw = crate::framebuffer::height();
            let bt = dy.min(640) as u16;
            let ex = dw.min(480) as u16;

            match aoa {
                "plasma" | "fire" | "matrix" | "shader" => {
                    crate::video::player::ofp(aoa, bt, ex, fps as u16);
                }
                _ => {
                    crate::println!("Unknown effect: {}. Available: plasma, fire, matrix, shader", aoa);
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
                        Ok(bk) => crate::println!("{}", bk),
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
                    if let Some(kp) = crate::video::codec::TvHeader::bsv(&data) {
                        crate::println!("=== TrustVideo Info ===");
                        crate::println!("  Format:     TrustVideo v{}", kp.version);
                        crate::println!("  Resolution: {}x{}", kp.width, kp.height);
                        crate::println!("  FPS:        {}", kp.fps);
                        crate::println!("  Frames:     {}", kp.frame_count);
                        crate::println!("  Duration:   {:.1}s", kp.frame_count as f64 / kp.fps as f64);
                        crate::println!("  Keyframe:   every {} frames", kp.keyframe_interval);
                        crate::println!("  File size:  {} bytes ({} KB)", data.len(), data.len() / 1024);
                        let gps = kp.width as usize * kp.height as usize * 4 * kp.frame_count as usize;
                        if gps > 0 {
                            let zi = gps as f64 / data.len() as f64;
                            crate::println!("  Compression: {:.1}x (raw would be {} KB)", zi, gps / 1024);
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


pub(super) fn kpe(args: &[&str]) {
    match args.first().copied().unwrap_or("open") {
        "open" | "" => {
            crate::println!("\x01G[TrustLab]\x01W Opening OS Introspection Laboratory...");
            crate::println!("  6-panel real-time kernel dashboard");
            crate::println!("  Use Tab to cycle panels, arrow keys to navigate");
            crate::desktop::S.lock().open_lab_mode();
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


pub(super) fn kop(args: &[&str]) {
    let output = crate::hwscan::idk(args);
    crate::print!("{}", output);
}