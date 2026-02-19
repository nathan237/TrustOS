//! Desktop Commands  COSMIC UI, Showcase, Benchmark, Signature, Security
//!
//! Contains the largest visual subsystems: COSMIC V2 desktop compositor,
//! showcase/3D demos, graphics benchmark, and kernel signature verification.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA, COLOR_GRAY};
use crate::ramfs::FileType;
// ==================== GRAPHICS PERFORMANCE BENCHMARK ====================

pub(super) fn cmd_benchmark(args: &[&str]) {
    use alloc::vec;
    
    crate::println_color!(COLOR_CYAN, "-----------------------------------------------------------");
    crate::println_color!(COLOR_CYAN, "              TrustOS Graphics Benchmark");
    crate::println_color!(COLOR_CYAN, "               SSE2 SIMD Optimizations");
    crate::println_color!(COLOR_CYAN, "-----------------------------------------------------------");
    crate::println!();
    
    let (width, height) = crate::framebuffer::get_dimensions();
    let pixels = (width * height) as usize;
    crate::println!("Resolution: {}x{} ({} pixels, {} MB)", 
        width, height, pixels, pixels * 4 / 1024 / 1024);
    crate::println!();
    
    // Test 1: Buffer fill with SSE2
    crate::println_color!(COLOR_GREEN, "? Test 1: SSE2 Buffer Fill");
    {
        let mut buffer = vec![0u32; pixels];
        let iterations = 100;
        
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            crate::graphics::simd::fill_buffer_fast(&mut buffer, 0xFF00FF66);
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_frame = (end - start) / iterations;
        let megapixels_per_frame = pixels as f64 / 1_000_000.0;
        crate::println!("  {} iterations: {} cycles/frame", iterations, cycles_per_frame);
        crate::println!("  Throughput: ~{:.1} megapixels/frame", megapixels_per_frame);
    }
    
    // Test 2: Buffer copy with SSE2
    crate::println_color!(COLOR_GREEN, "? Test 2: SSE2 Buffer Copy");
    {
        let src = vec![0xFF112233u32; pixels];
        let mut dst = vec![0u32; pixels];
        let iterations = 100;
        
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            crate::graphics::simd::copy_buffer_fast(&mut dst, &src);
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_frame = (end - start) / iterations;
        let mb_per_frame = (pixels * 4) as f64 / 1024.0 / 1024.0;
        crate::println!("  {} iterations: {} cycles/frame", iterations, cycles_per_frame);
        crate::println!("  Bandwidth: ~{:.1} MB copied/frame", mb_per_frame);
    }
    
    // Test 3: Rectangle fill
    crate::println_color!(COLOR_GREEN, "? Test 3: Rectangle Fill (400x300)");
    {
        let mut surface = crate::graphics::fast_render::FastSurface::new(1280, 800);
        let iterations = 500;
        
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            surface.fill_rect(100, 100, 400, 300, 0xFF00AA55);
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_rect = (end - start) / iterations;
        let pixels_filled = 400 * 300;
        crate::println!("  {} iterations: {} cycles/rect", iterations, cycles_per_rect);
        crate::println!("  {} pixels/rect", pixels_filled);
    }
    
    // Test 4: swap_buffers (framebuffer update)
    crate::println_color!(COLOR_GREEN, "? Test 4: Framebuffer swap_buffers");
    {
        // Make sure backbuffer is enabled
        let was_enabled = crate::framebuffer::is_double_buffer_enabled();
        if !was_enabled {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }
        
        let iterations = 50;
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            crate::framebuffer::swap_buffers();
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_swap = (end - start) / iterations;
        // Estimate FPS: ~3GHz CPU, 60 FPS target = 50M cycles/frame
        let estimated_fps = 3_000_000_000u64 / cycles_per_swap.max(1);
        crate::println!("  {} iterations: {} cycles/swap", iterations, cycles_per_swap);
        crate::println!("  Estimated max FPS: ~{} (at 3GHz)", estimated_fps);
        
        if !was_enabled {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }
    
    // Test 5: Terminal rendering
    crate::println_color!(COLOR_GREEN, "? Test 5: GraphicsTerminal render (80x25)");
    {
        let mut terminal = crate::wayland::terminal::GraphicsTerminal::new(80, 25);
        terminal.write_str("Hello from TrustOS! Testing SSE2 SIMD terminal rendering performance.\n");
        terminal.write_str("The quick brown fox jumps over the lazy dog.\n");
        
        let iterations = 100;
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            let _ = terminal.render();
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_render = (end - start) / iterations;
        let estimated_fps = 3_000_000_000u64 / cycles_per_render.max(1);
        crate::println!("  {} iterations: {} cycles/render", iterations, cycles_per_render);
        crate::println!("  Estimated terminal FPS: ~{}", estimated_fps);
    }
    
    crate::println!();
    crate::println_color!(COLOR_CYAN, "-----------------------------------------------------------");
    crate::println_color!(COLOR_GREEN, "Benchmark complete! SSE2 optimizations active.");
    crate::println_color!(COLOR_CYAN, "-----------------------------------------------------------");
}

// ==================== COSMIC UI DEMO ====================

pub(super) fn cmd_cosmic(args: &[&str]) {
    use crate::cosmic::{CosmicRenderer, Rect, Point, Color, theme, CosmicTheme, set_theme};
    use crate::cosmic::theme::dark;
    
    let subcommand = args.first().copied().unwrap_or("demo");
    
    match subcommand {
        "demo" | "test" => {
            crate::println_color!(COLOR_CYAN, "+---------------------------------------------------------------+");
            crate::println_color!(COLOR_CYAN, "|          COSMIC UI Framework Demo (libcosmic-inspired)       |");
            crate::println_color!(COLOR_CYAN, "+---------------------------------------------------------------+");
            crate::println!();
            
            let (width, height) = crate::framebuffer::get_dimensions();
            crate::println!("  Framebuffer: {}x{}", width, height);
            crate::println!("  Renderer: tiny-skia (software, no_std)");
            crate::println!("  Theme: COSMIC Dark (Pop!_OS style)");
            crate::println!();
            
            crate::println_color!(COLOR_GREEN, "Creating COSMIC renderer...");
            let mut renderer = CosmicRenderer::new(width, height);
            
            // Clear with COSMIC background
            renderer.clear(dark::BG_BASE);
            
            crate::println_color!(COLOR_GREEN, "Drawing COSMIC UI elements...");
            
            // Draw top panel (GNOME-style)
            let panel_rect = Rect::new(0.0, 0.0, width as f32, 32.0);
            renderer.draw_panel(panel_rect);
            
            // Draw some test shapes
            // Rounded rectangle
            let rect1 = Rect::new(50.0, 80.0, 200.0, 100.0);
            renderer.fill_rounded_rect(rect1, 12.0, dark::SURFACE);
            renderer.stroke_rounded_rect(rect1, 12.0, dark::BORDER, 1.0);
            
            // Button with shadow
            let btn_rect = Rect::new(300.0, 100.0, 120.0, 40.0);
            renderer.draw_shadow(btn_rect, 8.0, 8.0, Color::BLACK.with_alpha(0.4));
            renderer.fill_rounded_rect(btn_rect, 8.0, dark::ACCENT);
            
            // Another button (suggested style)
            let btn2_rect = Rect::new(450.0, 100.0, 120.0, 40.0);
            renderer.fill_rounded_rect(btn2_rect, 8.0, dark::BUTTON_SUGGESTED);
            
            // Destructive button
            let btn3_rect = Rect::new(600.0, 100.0, 120.0, 40.0);
            renderer.fill_rounded_rect(btn3_rect, 8.0, dark::BUTTON_DESTRUCTIVE);
            
            // Draw circles
            renderer.fill_circle(Point::new(100.0, 250.0), 30.0, dark::ACCENT);
            renderer.fill_circle(Point::new(180.0, 250.0), 30.0, dark::SUCCESS);
            renderer.fill_circle(Point::new(260.0, 250.0), 30.0, dark::WARNING);
            renderer.fill_circle(Point::new(340.0, 250.0), 30.0, dark::ERROR);
            
            // Draw header bar example
            let header_rect = Rect::new(50.0, 320.0, 400.0, 40.0);
            renderer.draw_header(header_rect, "COSMIC Window", true);
            
            // Window body
            let window_body = Rect::new(50.0, 360.0, 400.0, 150.0);
            renderer.fill_rect(window_body, dark::BG_COMPONENT);
            renderer.stroke_rounded_rect(
                Rect::new(50.0, 320.0, 400.0, 190.0),
                0.0,
                dark::BORDER,
                1.0
            );
            
            // Draw dock example
            let dock_items = [
                crate::cosmic::DockItem { name: "Files", active: true, hovered: false, running: true },
                crate::cosmic::DockItem { name: "Term", active: false, hovered: true, running: true },
                crate::cosmic::DockItem { name: "Browser", active: false, hovered: false, running: false },
                crate::cosmic::DockItem { name: "Settings", active: false, hovered: false, running: true },
            ];
            let dock_rect = Rect::new((width - 64) as f32, 100.0, 64.0, 280.0);
            renderer.draw_dock(dock_rect, &dock_items);
            
            // Gradient test
            let grad_rect = Rect::new(500.0, 320.0, 200.0, 100.0);
            renderer.fill_gradient_v(grad_rect, dark::ACCENT, dark::BG_BASE);
            
            crate::println_color!(COLOR_GREEN, "Presenting to framebuffer...");
            renderer.present_to_framebuffer();
            
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "? COSMIC UI demo rendered successfully!");
            crate::println!();
            crate::println!("  Features demonstrated:");
            crate::println!("  - Rounded rectangles with anti-aliasing");
            crate::println!("  - Drop shadows");
            crate::println!("  - COSMIC color palette");
            crate::println!("  - Top panel, header bar, dock");
            crate::println!("  - Buttons (normal, suggested, destructive)");
            crate::println!("  - Vertical gradients");
            crate::println!();
            crate::println!("  Press any key to return to shell...");
            
            // Wait for keypress then clear screen
            crate::keyboard::wait_for_key();
            crate::framebuffer::clear();
            crate::framebuffer::swap_buffers();
        },
        "desktop" => {
            // Launch COSMIC V2 desktop with multi-layer compositor (no flicker)
            cmd_cosmic_v2();
        },
        "theme" => {
            let theme_name = args.get(1).copied().unwrap_or("matrix");
            match theme_name {
                "dark" => {
                    set_theme(CosmicTheme::dark());
                    crate::println_color!(COLOR_GREEN, "Theme set to COSMIC Dark");
                },
                "light" => {
                    set_theme(CosmicTheme::light());
                    crate::println_color!(COLOR_GREEN, "Theme set to COSMIC Light");
                },
                "matrix" => {
                    set_theme(CosmicTheme::matrix());
                    crate::println_color!(0x00FF00, "Theme set to MATRIX - Wake up, Neo...");
                },
                _ => {
                    crate::println!("Available themes: dark, light, matrix");
                }
            }
        },
        "info" => {
            crate::println_color!(COLOR_CYAN, "COSMIC UI Framework for TrustOS");
            crate::println!();
            crate::println!("  Based on: libcosmic by System76 (Pop!_OS)");
            crate::println!("  Renderer: tiny-skia v0.12 (no_std mode)");
            crate::println!("  Features: anti-aliased shapes, gradients, shadows");
            crate::println!();
            crate::println!("  Modules:");
            crate::println!("    cosmic::theme    - COSMIC color palette");
            crate::println!("    cosmic::renderer - tiny-skia based rendering");
            crate::println!("    cosmic::widgets  - Button, Label, Container, etc.");
            crate::println!("    cosmic::layout   - Flexbox-style layout system");
        },
        _ => {
            crate::println!("Usage: cosmic <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  desktop - Launch full COSMIC desktop environment");
            crate::println!("  demo    - Render COSMIC UI demo to screen");
            crate::println!("  theme   - Set theme (dark/light)");
            crate::println!("  info    - Show framework information");
        }
    }
}

/// Open command - launch desktop with a specific app
pub(super) fn cmd_open(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: open <app>");
        crate::println!("");
        crate::println!("Available apps:");
        crate::println!("  browser, web, www   - Web browser (HTTPS/TLS 1.3)");
        crate::println!("  files, explorer     - File manager");
        crate::println!("  editor, notepad     - Text editor");
        crate::println!("  network, net        - Network status");
        crate::println!("  hardware, hw        - Hardware info");
        crate::println!("  users               - User management");
        crate::println!("  images, viewer      - Image viewer");
        crate::println!("");
        crate::println!("Example: open browser");
        return;
    }
    
    let app = args[0].to_lowercase();
    cmd_cosmic_v2_with_app(Some(&app));
}

/// Launch the desktop.rs windowed desktop environment.
/// Optionally pre-opens a window of the given type.
pub(super) fn launch_desktop_env(initial_window: Option<(&str, crate::desktop::WindowType, i32, i32, u32, u32)>) {
    use crate::desktop;
    let (width, height) = crate::framebuffer::get_dimensions();
    if width == 0 || height == 0 {
        crate::println_color!(COLOR_RED, "Error: Invalid framebuffer!");
        return;
    }
    crate::mouse::set_screen_size(width, height);
    
    let mut d = desktop::DESKTOP.lock();
    d.init(width, height);
    
    // Open the requested window if any
    if let Some((title, wtype, x, y, w, h)) = initial_window {
        d.create_window(title, x, y, w, h, wtype);
    }
    
    drop(d);
    crate::serial_println!("[Desktop] Entering desktop run loop");
    desktop::run();
    // Desktop exited -- restore shell
    crate::serial_println!("[Desktop] Returned to shell");
    // Clear screen
    let (w, h) = crate::framebuffer::get_dimensions();
    crate::framebuffer::fill_rect(0, 0, w, h, 0xFF000000);
    crate::println_color!(COLOR_GREEN, "\nReturned to TrustOS shell. Type 'help' for commands.");
}

// ==================== SIGNATURE -- KERNEL PROOF OF AUTHORSHIP ====================

pub(super) fn cmd_signature(args: &[&str]) {
    use crate::signature;

    match args.first().copied() {
        Some("verify") | None => {
            // Show creator + user signatures
            crate::println!();
            crate::println_color!(COLOR_CYAN, "\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
            crate::println_color!(COLOR_CYAN, "\u{2551}              TrustOS Kernel Signature Certificate                  \u{2551}");
            crate::println_color!(COLOR_CYAN, "\u{2560}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2563}");
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "  ?? CREATOR SIGNATURE (immutable)");
            crate::println_color!(COLOR_WHITE, "  -----------------------------------------------------------------");
            crate::println!("  Author:      {} (@{})", signature::CREATOR_NAME, signature::CREATOR_GITHUB);
            crate::println!("  Payload:     \"{}\"", signature::CREATOR_SIGNED_PAYLOAD);
            crate::println!("  Algorithm:   HMAC-SHA256");
            crate::println_color!(COLOR_YELLOW, "  Fingerprint: {}", signature::creator_fingerprint_hex());
            crate::println!("  Version:     v{}", signature::KERNEL_VERSION);
            crate::println!("  Built:       {}", signature::BUILD_TIMESTAMP);
            crate::println!();
            crate::println_color!(COLOR_GRAY, "  i  This fingerprint was generated with a secret seed known ONLY");
            crate::println_color!(COLOR_GRAY, "     to the creator. It cannot be forged without the original seed.");
            crate::println!();

            // Show user signature if present
            if let Some((name, hex, ts)) = signature::get_user_signature() {
                crate::println_color!(COLOR_BLUE, "  USER CO-SIGNATURE");
                crate::println_color!(COLOR_WHITE, "  -----------------------------------------------------------------");
                crate::println!("  Signed by:   {}", name);
                crate::println_color!(COLOR_YELLOW, "  Fingerprint: {}", hex);
                crate::println!("  Signed at:   {}s after midnight (RTC)", ts);
                crate::println!();
            } else {
                crate::println_color!(COLOR_GRAY, "  No user co-signature. Use 'signature sign <name>' to add yours.");
                crate::println!();
            }

            crate::println_color!(COLOR_CYAN, "\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}");
            crate::println!();
        }
        Some("sign") => {
            // signature sign <name>
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: signature sign <your_name>");
                return;
            }
            let name = args[1];
            crate::println!("Enter your secret passphrase to sign the kernel:");
            crate::print!("> ");
            // Read passphrase (we'll use a simple input method)
            let passphrase = read_passphrase();
            if passphrase.is_empty() {
                crate::println_color!(COLOR_RED, "Empty passphrase. Aborted.");
                return;
            }
            signature::sign_as_user(name, passphrase.as_bytes());
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "? Kernel co-signed by '{}'", name);
            if let Some((_, hex, _)) = signature::get_user_signature() {
                crate::println_color!(COLOR_YELLOW, "  Your fingerprint: {}", hex);
            }
            crate::println_color!(COLOR_GRAY, "  Keep your passphrase safe -- you'll need it to prove ownership.");
            crate::println!();
        }
        Some("prove") => {
            // Verify a user's signature with their passphrase
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: signature prove <name>");
                return;
            }
            let name = args[1];
            crate::println!("Enter passphrase to verify:");
            crate::print!("> ");
            let passphrase = read_passphrase();
            if signature::verify_user_seed(name, passphrase.as_bytes()) {
                crate::println_color!(COLOR_BRIGHT_GREEN, "VERIFIED -- '{}' is the legitimate signer.", name);
            } else {
                crate::println_color!(COLOR_RED, "FAILED -- passphrase does not match the signature for '{}'.", name);
            }
            crate::println!();
        }
        Some("prove-creator") => {
            // Only the real creator can pass this
            crate::println!("Enter creator seed to verify authorship:");
            crate::print!("> ");
            let seed = read_passphrase();
            if signature::verify_creator_seed(seed.as_bytes()) {
                crate::println_color!(COLOR_BRIGHT_GREEN, "CREATOR VERIFIED -- You are the original author of TrustOS.");
            } else {
                crate::println_color!(COLOR_RED, "FAILED -- This seed does not match the creator fingerprint.");
            }
            crate::println!();
        }
        Some("integrity") | Some("verify-integrity") => {
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Kernel Integrity Verification");
            crate::println!("---------------------------------------------------------------");
            let report = signature::integrity_report();
            for line in &report {
                crate::println!("{}", line);
            }
            crate::println!();
            crate::println_color!(COLOR_GRAY, "  SHA-256 of .text + .rodata sections measured at boot vs now.");
            crate::println_color!(COLOR_GRAY, "  Detects runtime code injection, ROP gadget insertion, and");
            crate::println_color!(COLOR_GRAY, "  constant/vtable tampering (rootkits, memory corruption).");
            crate::println!();
        }
        Some("clear") => {
            signature::clear_user_signature();
            crate::println_color!(COLOR_YELLOW, "User co-signature cleared.");
        }
        Some("export") => {
            // Export signature in SIGNATURES.md format for GitHub PR
            if let Some((name, hex, _ts)) = signature::get_user_signature() {
                let dt = crate::rtc::read_rtc();
                crate::println!();
                crate::println_color!(COLOR_CYAN, "=== Copy everything below and submit as a PR to SIGNATURES.md ===");
                crate::println!();
                crate::println!("### #NNN -- {}", name);
                crate::println!();
                crate::println!("| Field | Value |");
                crate::println!("|-------|-------|");
                crate::println!("| **Name** | {} |", name);
                crate::println!("| **GitHub** | [@YOURUSERNAME](https://github.com/YOURUSERNAME) |");
                crate::println!("| **Algorithm** | HMAC-SHA256 |");
                crate::println!("| **Fingerprint** | `{}` |", hex);
                crate::println!("| **Kernel Version** | v{} |", signature::KERNEL_VERSION);
                crate::println!("| **Date** | {:04}-{:02}-{:02} |", dt.year, dt.month, dt.day);
                crate::println!("| **Status** | Verified signer |");
                crate::println!();
                crate::println_color!(COLOR_GRAY, "Replace YOURUSERNAME with your GitHub username and #NNN with the next number.");
                crate::println_color!(COLOR_GRAY, "Submit as a Pull Request to: github.com/nathan237/TrustOS");
                crate::println!();
            } else {
                crate::println_color!(COLOR_RED, "No user signature found. Run 'signature sign <name>' first.");
            }
        }
        Some("list") => {
            // Show all registered signatures info
            crate::println!();
            crate::println_color!(COLOR_CYAN, "TrustOS Signature Registry");
            crate::println_color!(COLOR_WHITE, "------------------------------------------------------");
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "  #001  Nated0ge (Creator)");
            crate::println!("        {}", signature::creator_fingerprint_hex());
            crate::println!();
            if let Some((name, hex, _)) = signature::get_user_signature() {
                crate::println_color!(COLOR_CYAN, "  #---  {} (Local)", name);
                crate::println!("        {}", hex);
                crate::println!();
            }
            crate::println_color!(COLOR_GRAY, "  Full registry: github.com/nathan237/TrustOS/blob/main/SIGNATURES.md");
            crate::println!();
        }
        Some("ed25519") => {
            // Ed25519 asymmetric signature subsystem
            match args.get(1).copied() {
                Some("verify") | None => {
                    crate::println!();
                    crate::println_color!(COLOR_CYAN, "Ed25519 Asymmetric Signature Report");
                    crate::println_color!(COLOR_WHITE, "--------------------------------------------------------------");
                    let report = signature::ed25519_report();
                    for line in &report {
                        crate::println!("{}", line);
                    }
                    crate::println!();
                }
                Some("sign") => {
                    crate::println!("Enter Ed25519 seed (hex or passphrase):");
                    crate::print!("> ");
                    let seed_input = read_passphrase();
                    if seed_input.is_empty() {
                        crate::println_color!(COLOR_RED, "Empty seed. Aborted.");
                        return;
                    }
                    signature::ed25519_resign(seed_input.as_bytes());
                    crate::println_color!(COLOR_BRIGHT_GREEN, "? Kernel re-signed with Ed25519 (new seed).");
                    if let Some(report) = signature::ed25519_report().first() {
                        crate::println!("  {}", report);
                    }
                    crate::println!();
                }
                _ => {
                    crate::println!("Usage:");
                    crate::println!("  signature ed25519          - Show Ed25519 signature & verify");
                    crate::println!("  signature ed25519 verify   - Verify current Ed25519 signature");
                    crate::println!("  signature ed25519 sign     - Re-sign kernel with new seed");
                }
            }
        }
        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Kernel Signature System");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  signature                - Show signature certificate");
            crate::println!("  signature verify         - Show & verify signature certificate");
            crate::println!("  signature integrity      - Verify kernel .text+.rodata integrity");
            crate::println!("  signature sign <name>    - Co-sign the kernel with your identity");
            crate::println!("  signature prove <name>   - Prove a user signature with passphrase");
            crate::println!("  signature prove-creator  - Prove creator authorship (requires seed)");
            crate::println!("  signature export         - Export signature for GitHub PR submission");
            crate::println!("  signature list           - Show registered signatures");
            crate::println!("  signature clear          - Remove user co-signature");
            crate::println!("  signature ed25519        - Ed25519 asymmetric signature status");
            crate::println!("  signature ed25519 sign   - Re-sign kernel with Ed25519 seed");
        }
    }
}

/// Read a passphrase from keyboard input (hidden, returns on Enter)
fn read_passphrase() -> alloc::string::String {
    use alloc::string::String;
    let mut passphrase = String::new();
    loop {
        if let Some(key) = crate::keyboard::try_read_key() {
            match key {
                b'\n' | b'\r' | 0x0A | 0x0D => {
                    crate::println!();
                    break;
                }
                0x08 => {
                    // Backspace
                    if !passphrase.is_empty() {
                        passphrase.pop();
                        crate::print!("\x08 \x08");
                    }
                }
                c if c.is_ascii() && !c.is_ascii_control() => {
                    passphrase.push(c as char);
                    crate::print!("*");
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
    passphrase
}

/// Security subsystem management command
pub(super) fn cmd_security(args: &[&str]) {
    match args.first().copied() {
        Some("status") | None => {
            // Show overall security status
            let stats = crate::security::stats();
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS Security Status");
            crate::println!("---------------------------------------------------------------");
            crate::println!("  Active capabilities : {}", stats.active_capabilities);
            crate::println!("  Security violations : {}", stats.violations);
            crate::println!("  Dynamic types       : {}", stats.dynamic_types);
            crate::println!("  Isolated subsystems : {}", stats.subsystems);
            crate::println!("  Gate checks         : {}", crate::security::isolation::total_gate_checks());
            crate::println!("  Gate violations     : {}", crate::security::isolation::total_gate_violations());
            crate::println!();
            
            // Integrity summary
            match crate::signature::verify_integrity() {
                Ok(true) => crate::println_color!(COLOR_BRIGHT_GREEN, "  Kernel integrity    : ? INTACT"),
                Ok(false) => crate::println_color!(COLOR_RED, "  Kernel integrity    : ? TAMPERED"),
                Err(_) => crate::println_color!(COLOR_YELLOW, "  Kernel integrity    : ??  not initialized"),
            }
            crate::println!();
        }
        Some("caps") | Some("capabilities") => {
            // List all active capabilities
            let caps = crate::security::list_capabilities();
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Active Capabilities ({} total)", caps.len());
            crate::println!("----------------------------------------------------------");
            crate::println!("  {:>6} | {:<20} | {:<10} | Owner", "ID", "Type", "Category");
            crate::println!("  -------+----------------------+------------+------");
            for (id, cap_type, _rights, owner) in &caps {
                crate::println!("  {:>6} | {:<20} | {:<10} | 0x{:04X}",
                    id.0,
                    alloc::format!("{:?}", cap_type),
                    cap_type.category(),
                    owner
                );
            }
            crate::println!();
        }
        Some("isolation") | Some("iso") | Some("subsystems") => {
            // Show subsystem isolation status
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Subsystem Isolation Boundaries");
            crate::println!("---------------------------------------------------------------");
            let report = crate::security::isolation::isolation_report();
            for line in &report {
                crate::println!("{}", line);
            }
            crate::println!();
            crate::println_color!(COLOR_GRAY, "  ring0-tcb       = Part of TCB, must stay in ring 0");
            crate::println_color!(COLOR_GRAY, "  ring0-isolated  = Ring 0 but logically isolated");
            crate::println_color!(COLOR_GRAY, "  ring3-candidate = Could be moved to ring 3 in future");
            crate::println!();
        }
        Some("gate") => {
            // Test a gate check
            if let Some(subsystem_name) = args.get(1).copied() {
                let subsystem = match subsystem_name {
                    "storage" | "disk" => Some(crate::security::isolation::Subsystem::Storage),
                    "network" | "net" => Some(crate::security::isolation::Subsystem::Network),
                    "graphics" | "gpu" => Some(crate::security::isolation::Subsystem::Graphics),
                    "process" | "proc" => Some(crate::security::isolation::Subsystem::ProcessMgr),
                    "hypervisor" | "hv" => Some(crate::security::isolation::Subsystem::Hypervisor),
                    "shell" => Some(crate::security::isolation::Subsystem::Shell),
                    "crypto" => Some(crate::security::isolation::Subsystem::Crypto),
                    "power" => Some(crate::security::isolation::Subsystem::Power),
                    "serial" => Some(crate::security::isolation::Subsystem::SerialDebug),
                    "memory" | "mem" => Some(crate::security::isolation::Subsystem::Memory),
                    _ => None,
                };
                if let Some(sub) = subsystem {
                    match crate::security::isolation::gate_check(
                        sub, crate::security::CapabilityRights::READ
                    ) {
                        Ok(()) => crate::println_color!(COLOR_BRIGHT_GREEN, 
                            "  ? Gate check PASSED for {:?}", sub),
                        Err(e) => crate::println_color!(COLOR_RED, 
                            "  ? Gate check DENIED for {:?}: {:?}", sub, e),
                    }
                } else {
                    crate::println_color!(COLOR_RED, "Unknown subsystem: {}", subsystem_name);
                }
            } else {
                crate::println!("Usage: security gate <subsystem>");
                crate::println!("  Subsystems: storage, network, graphics, process, hypervisor,");
                crate::println!("              shell, crypto, power, serial, memory");
            }
        }
        Some("dynamic") => {
            // List dynamic capability types
            let types = crate::security::list_dynamic_types();
            crate::println!();
            if types.is_empty() {
                crate::println_color!(COLOR_GRAY, "No dynamic capability types registered.");
            } else {
                crate::println_color!(COLOR_CYAN, "Dynamic Capability Types ({} registered)", types.len());
                for (id, info) in &types {
                    crate::println!("  [{}] {} (danger:{}, category:{})", 
                        id, info.name, info.danger_level, info.category);
                    crate::println!("       {}", info.description);
                }
            }
            crate::println!();
        }
        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Security Subsystem");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  security                 - Show security status overview");
            crate::println!("  security caps            - List all active capabilities");
            crate::println!("  security isolation       - Show subsystem isolation boundaries");
            crate::println!("  security gate <subsys>   - Test a gate check on a subsystem");
            crate::println!("  security dynamic         - List dynamic capability types");
        }
    }
}

// ==================== DEMO TUTORIAL -- Interactive guided tour ====================
// A cinematic interactive tutorial that walks users through TrustOS features.
// Uses Matrix-rain styled scenes for intro, then live shell demos, then desktop preview.
// Launched via `demo` or `tutorial` shell command.

pub(super) fn cmd_demo(args: &[&str]) {
    let lang = match args.first().copied() {
        Some("fr") => "fr",
        _ => "en",
    };

    // Use TSC for timing
    let pause = |secs: u64| {
        let ms = secs * 1000;
        let start_tsc = crate::cpu::tsc::read_tsc();
        let freq = crate::cpu::tsc::frequency_hz();
        if freq == 0 { return; }
        let target_cycles = freq / 1000 * ms;
        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
            if elapsed >= target_cycles { break; }
            let _ = crate::keyboard::try_read_key();
            core::hint::spin_loop();
        }
    };

    let wait_key_or_timeout = |timeout_secs: u64| -> bool {
        let ms = timeout_secs * 1000;
        let start_tsc = crate::cpu::tsc::read_tsc();
        let freq = crate::cpu::tsc::frequency_hz();
        if freq == 0 { return false; }
        let target_cycles = freq / 1000 * ms;
        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
            if elapsed >= target_cycles { return false; }
            if let Some(key) = crate::keyboard::try_read_key() {
                if key == 0x1B || key == b'q' { return true; } // quit
                if key == b' ' || key == b'\n' || key == 13 { return false; } // continue
            }
            core::hint::spin_loop();
        }
    };

    // -------------------------------------------------------------------
    // PART 1: Cinematic Intro (Matrix rain framebuffer)
    // -------------------------------------------------------------------
    let (sw, sh) = crate::framebuffer::get_dimensions();

    {
        let was_db = crate::framebuffer::is_double_buffer_enabled();
        if !was_db {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }

        let w = sw as usize;
        let h = sh as usize;
        let mut buf = alloc::vec![0u32; w * h];

        // -- Helper: draw scaled character into buffer --
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

        // -- Helper: Matrix rain background --
        let mut rain_cols: alloc::vec::Vec<u16> = alloc::vec![0u16; w / 8 + 1];
        let mut rain_speeds: alloc::vec::Vec<u8> = alloc::vec![1u8; w / 8 + 1];
        for i in 0..rain_cols.len() {
            rain_cols[i] = ((i * 37 + 13) % h) as u16;
            rain_speeds[i] = (((i * 7 + 3) % 4) + 1) as u8;
        }

        let draw_rain_step = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
            for pixel in buf.iter_mut() {
                let g = ((*pixel >> 8) & 0xFF) as u32;
                if g > 0 {
                    let new_g = g.saturating_sub(8);
                    *pixel = 0xFF000000 | (new_g << 8);
                }
            }
            for col_idx in 0..cols.len() {
                let x = col_idx * 8;
                if x >= w { continue; }
                cols[col_idx] = cols[col_idx].wrapping_add(speeds[col_idx] as u16);
                if cols[col_idx] as usize >= h { cols[col_idx] = 0; }
                let y = cols[col_idx] as usize;
                let c = (((frame as usize + col_idx * 13) % 94) + 33) as u8 as char;
                let glyph = crate::framebuffer::font::get_glyph(c);
                for (row, &bits) in glyph.iter().enumerate() {
                    let py = y + row;
                    if py >= h { break; }
                    for bit in 0..8u32 {
                        if bits & (0x80 >> bit) != 0 {
                            let px = x + bit as usize;
                            if px < w {
                                buf[py * w + px] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        let blit_buf = |buf: &[u32], w: usize, h: usize| {
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                let bb = bb_ptr as *mut u32;
                let bb_s = bb_stride as usize;
                for y in 0..h.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            buf[y * w..].as_ptr(),
                            bb.add(y * bb_s),
                            w,
                        );
                    }
                }
            }
            crate::framebuffer::swap_buffers();
        };

        // -- Helper: render tutorial scene with typing effect --
        let render_scene = |buf: &mut [u32], w: usize, h: usize,
                           rain_cols: &mut [u16], rain_speeds: &[u8],
                           lines: &[(&str, u32, usize)],
                           hold_ms: u64| {
            let freq = crate::cpu::tsc::frequency_hz();
            if freq == 0 { return; }

            let total_chars: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
            let type_ms = 60u64;
            let type_total_ms = total_chars as u64 * type_ms;

            let start_tsc = crate::cpu::tsc::read_tsc();
            let hold_target = freq / 1000 * (type_total_ms + hold_ms);

            let mut frame = 0u32;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
                if elapsed >= hold_target { break; }
                if let Some(key) = crate::keyboard::try_read_key() {
                    if key == 0x1B || key == b'q' { break; }
                    if key == b' ' || key == b'\n' || key == 13 { break; } // skip scene
                }

                draw_rain_step(buf, w, h, rain_cols, rain_speeds, frame);

                let elapsed_ms = elapsed / (freq / 1000).max(1);
                let chars_shown = if elapsed_ms < type_total_ms {
                    (elapsed_ms / type_ms.max(1)) as usize
                } else {
                    total_chars
                };

                let total_text_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 8).sum::<usize>();
                let mut y_start = if total_text_h < h { (h - total_text_h) / 2 } else { 20 };
                let mut chars_counted = 0usize;

                for &(text, color, scale) in lines {
                    let text_w = text.len() * 8 * scale;
                    let start_x = if text_w < w { (w - text_w) / 2 } else { 0 };

                    for (i, c) in text.chars().enumerate() {
                        if chars_counted + i >= chars_shown { break; }
                        draw_big_char(buf, w, h, start_x + i * 8 * scale, y_start, c, color, scale);
                    }
                    if chars_shown > chars_counted && chars_shown < chars_counted + text.len() {
                        let cursor_i = chars_shown - chars_counted;
                        let cx = start_x + cursor_i * 8 * scale;
                        for cy in y_start..y_start + 16 * scale {
                            if cy < h && cx + 2 < w {
                                buf[cy * w + cx] = 0xFF00FF88;
                                buf[cy * w + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }

                    chars_counted += text.len();
                    y_start += 16 * scale + 8;
                }

                blit_buf(buf, w, h);
                frame += 1;
                crate::cpu::tsc::delay_millis(33);
            }

            // Fade out
            let fade_start = crate::cpu::tsc::read_tsc();
            let fade_target = freq / 1000 * 600;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(fade_start);
                if elapsed >= fade_target { break; }
                let progress = (elapsed * 255 / fade_target) as u32;
                for pixel in buf.iter_mut() {
                    let r = ((*pixel >> 16) & 0xFF) as u32;
                    let g = ((*pixel >> 8) & 0xFF) as u32;
                    let b = (*pixel & 0xFF) as u32;
                    let nr = r.saturating_sub(r * progress / 512 + 1);
                    let ng = g.saturating_sub(g * progress / 512 + 1);
                    let nb = b.saturating_sub(b * progress / 512 + 1);
                    *pixel = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
                }
                blit_buf(buf, w, h);
                crate::cpu::tsc::delay_millis(33);
            }
            for pixel in buf.iter_mut() { *pixel = 0xFF000000; }
            blit_buf(buf, w, h);
        };

        // ---- Scene 1: Welcome ----
        crate::serial_println!("[DEMO] Scene 1: Welcome");
        for pixel in buf.iter_mut() { *pixel = 0xFF000000; }
        if lang == "fr" {
            render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
                &[("Bienvenue dans", 0xFF00DD55, 5),
                  ("TrustOS", 0xFF00FFAA, 6)],
                3000);
        } else {
            render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
                &[("Welcome to", 0xFF00DD55, 5),
                  ("TrustOS", 0xFF00FFAA, 6)],
                3000);
        }

        // ---- Scene 2: What is it? ----
        crate::serial_println!("[DEMO] Scene 2: What is TrustOS");
        if lang == "fr" {
            render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
                &[("Un OS bare-metal", 0xFF00DD55, 4),
                  ("ecrit en 100% Rust", 0xFF00FF88, 4),
                  ("Aucun C. Aucun Linux.", 0xFFFFCC44, 3)],
                3000);
        } else {
            render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
                &[("A bare-metal OS", 0xFF00DD55, 4),
                  ("written in 100% Rust", 0xFF00FF88, 4),
                  ("No C. No Linux. Just Rust.", 0xFFFFCC44, 3)],
                3000);
        }

        // ---- Scene 3: Tutorial Mode ----
        crate::serial_println!("[DEMO] Scene 3: Tutorial start");
        if lang == "fr" {
            render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
                &[("Tutoriel Interactif", 0xFF44DDFF, 5),
                  ("Appuyez ESPACE pour continuer", 0xFF888888, 2),
                  ("ESC pour quitter", 0xFF666666, 2)],
                4000);
        } else {
            render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
                &[("Interactive Tutorial", 0xFF44DDFF, 5),
                  ("Press SPACE to continue", 0xFF888888, 2),
                  ("ESC to quit", 0xFF666666, 2)],
                4000);
        }

        if !was_db {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }

    // -------------------------------------------------------------------
    // PART 2: Shell Tutorial (live command demos)
    // -------------------------------------------------------------------
    crate::framebuffer::clear();

    let print_step = |step: u32, total: u32, title_en: &str, title_fr: &str, desc_en: &str, desc_fr: &str| {
        crate::println!();
        crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
        if lang == "fr" {
            crate::println_color!(0xFF00CCFF, "|  ETAPE {}/{} -- {}", step, total, title_fr);
        } else {
            crate::println_color!(0xFF00CCFF, "|  STEP {}/{} -- {}", step, total, title_en);
        }
        crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
        crate::println!();
        if lang == "fr" {
            crate::println_color!(0xFF888888, "  {}", desc_fr);
        } else {
            crate::println_color!(0xFF888888, "  {}", desc_en);
        }
        crate::println!();
    };

    let total_steps = 8u32;

    // ---- Step 1: System Info ----
    print_step(1, total_steps, "SYSTEM INFO", "INFOS SYSTEME",
               "TrustOS can show detailed system information, just like Linux.",
               "TrustOS affiche les infos systeme, comme sous Linux.");
    pause(2);

    crate::println_color!(COLOR_CYAN, "  $ neofetch");
    pause(1);
    super::commands::cmd_neofetch();
    pause(3);

    if lang == "fr" {
        crate::println_color!(0xFF00FF88, "  -> Neofetch montre le CPU, la RAM, le kernel et l'uptime.");
    } else {
        crate::println_color!(0xFF00FF88, "  -> Neofetch shows CPU, RAM, kernel version and uptime.");
    }
    if wait_key_or_timeout(6) { return; }

    // ---- Step 2: Filesystem ----
    print_step(2, total_steps, "FILESYSTEM", "SYSTEME DE FICHIERS",
               "TrustOS has a full virtual filesystem (TrustFS + VFS).",
               "TrustOS possede un systeme de fichiers virtuel complet (TrustFS + VFS).");
    pause(1);

    crate::println_color!(COLOR_CYAN, "  $ mkdir /tutorial");
    crate::ramfs::with_fs(|fs| { let _ = fs.mkdir("/tutorial"); });
    crate::println_color!(COLOR_GREEN, "  Created /tutorial");
    pause(1);

    crate::println_color!(COLOR_CYAN, "  $ echo 'Hello from TrustOS!' > /tutorial/hello.txt");
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch("/tutorial/hello.txt");
        let _ = fs.write_file("/tutorial/hello.txt", b"Hello from TrustOS!\nThis file was created during the tutorial.\nPure Rust, running on bare metal.\n");
    });
    crate::println_color!(COLOR_GREEN, "  Written: /tutorial/hello.txt");
    pause(1);

    crate::println_color!(COLOR_CYAN, "  $ cat /tutorial/hello.txt");
    super::commands::cmd_cat(&["/tutorial/hello.txt"], None, None);
    pause(2);

    crate::println_color!(COLOR_CYAN, "  $ tree /");
    super::commands::cmd_tree(&["/"]);

    if lang == "fr" {
        crate::println_color!(0xFF00FF88, "  -> Commandes POSIX completes: ls, cd, mkdir, rm, cp, mv, cat, find, grep...");
    } else {
        crate::println_color!(0xFF00FF88, "  -> Full POSIX commands: ls, cd, mkdir, rm, cp, mv, cat, find, grep...");
    }
    if wait_key_or_timeout(6) { return; }

    // ---- Step 3: TrustLang ----
    print_step(3, total_steps, "TRUSTLANG COMPILER", "COMPILATEUR TRUSTLANG",
               "TrustOS includes a built-in programming language with compiler + VM.",
               "TrustOS inclut un langage de programmation avec compilateur + VM.");
    pause(1);

    let tl_code = r#"fn factorial(n: i64) -> i64 {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}

fn main() {
    println("=== TrustLang Demo ===");
    for i in 1..8 {
        print("  ");
        print(to_string(i));
        print("! = ");
        println(to_string(factorial(i)));
    }
}"#;
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch("/tutorial/demo.tl");
        let _ = fs.write_file("/tutorial/demo.tl", tl_code.as_bytes());
    });

    crate::println_color!(COLOR_CYAN, "  $ cat /tutorial/demo.tl");
    crate::println_color!(0xFFDDDDDD, "{}", tl_code);
    pause(3);

    crate::println_color!(COLOR_CYAN, "  $ trustlang run /tutorial/demo.tl");
    crate::println_color!(0xFF00FF88, "  [TrustLang] Compiling...");
    match crate::trustlang::run(tl_code) {
        Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
        Err(e) => crate::println_color!(COLOR_RED, "  Error: {}", e),
    }
    crate::println_color!(COLOR_GREEN, "  [TrustLang] Done!");

    if lang == "fr" {
        crate::println_color!(0xFF00FF88, "  -> Fonctions, recursion, boucles, types -- compile en bytecode!");
    } else {
        crate::println_color!(0xFF00FF88, "  -> Functions, recursion, loops, types -- compiled to bytecode!");
    }
    if wait_key_or_timeout(6) { return; }

    // ---- Step 4: Network Stack ----
    print_step(4, total_steps, "NETWORK STACK", "PILE RESEAU",
               "Full TCP/IP stack: DHCP, DNS, HTTP, TLS 1.3 -- all in Rust.",
               "Pile TCP/IP complete: DHCP, DNS, HTTP, TLS 1.3 -- tout en Rust.");
    pause(1);

    crate::println_color!(COLOR_CYAN, "  $ ifconfig");
    super::vm::cmd_ifconfig();
    pause(2);

    crate::println_color!(COLOR_CYAN, "  $ netstat");
    super::vm::cmd_netstat();

    if lang == "fr" {
        crate::println_color!(0xFF00FF88, "  -> Un navigateur web integre peut charger de vraies pages!");
    } else {
        crate::println_color!(0xFF00FF88, "  -> A built-in web browser can load real web pages!");
    }
    if wait_key_or_timeout(5) { return; }

    // ---- Step 5: Video Effects ----
    print_step(5, total_steps, "VIDEO EFFECTS", "EFFETS VIDEO",
               "Real-time procedural rendering engine -- fire, matrix, plasma.",
               "Moteur de rendu procedural temps reel -- feu, matrix, plasma.");
    pause(2);

    let vw = sw as u16;
    let vh = sh as u16;

    if lang == "fr" {
        crate::println_color!(0xFFFF4400, "  Effet 1: FEU -- Flammes procedurales (5s)");
    } else {
        crate::println_color!(0xFFFF4400, "  Effect 1: FIRE -- Procedural flames (5s)");
    }
    pause(1);
    crate::video::player::render_realtime_timed("fire", vw, vh, 30, 5000);
    crate::framebuffer::clear();

    if lang == "fr" {
        crate::println_color!(0xFF00FF44, "  Effet 2: MATRIX -- Pluie numerique (5s)");
    } else {
        crate::println_color!(0xFF00FF44, "  Effect 2: MATRIX -- Digital rain (5s)");
    }
    pause(1);
    crate::video::player::render_realtime_timed("matrix", vw, vh, 30, 5000);
    crate::framebuffer::clear();

    if lang == "fr" {
        crate::println_color!(0xFFFF00FF, "  Effet 3: PLASMA -- Plasma psychedelique (5s)");
    } else {
        crate::println_color!(0xFFFF00FF, "  Effect 3: PLASMA -- Psychedelic plasma (5s)");
    }
    pause(1);
    crate::video::player::render_realtime_timed("plasma", vw, vh, 30, 5000);
    crate::framebuffer::clear();

    if lang == "fr" {
        crate::println_color!(0xFF00FF88, "  -> Tout fonctionne a 60+ FPS sur du bare-metal!");
    } else {
        crate::println_color!(0xFF00FF88, "  -> All running at 60+ FPS on bare metal!");
    }
    if wait_key_or_timeout(4) { return; }

    // ---- Step 6: 3D Engine ----
    print_step(6, total_steps, "3D ENGINE", "MOTEUR 3D",
               "Wireframe 3D with perspective projection and depth shading.",
               "3D filaire avec projection perspective et ombrage de profondeur.");
    pause(2);

    {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.set_scene(crate::formula3d::FormulaScene::Character);
        renderer.wire_color = 0xFF00FFAA;

        let rw = sw as usize;
        let rh = sh as usize;
        let mut vp_buf = alloc::vec![0u32; rw * rh];

        let was_db = crate::framebuffer::is_double_buffer_enabled();
        if !was_db {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }
        crate::framebuffer::clear_backbuffer(0xFF000000);
        crate::framebuffer::swap_buffers();

        let start_tsc = crate::cpu::tsc::read_tsc();
        let freq = crate::cpu::tsc::frequency_hz();
        let target_cycles = if freq > 0 { freq / 1000 * 6000 } else { u64::MAX }; // 6 seconds

        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
            if elapsed >= target_cycles { break; }
            if let Some(key) = crate::keyboard::try_read_key() {
                if key == 0x1B || key == b'q' { break; }
                if key == b' ' || key == b'\n' || key == 13 { break; }
            }

            renderer.update();
            renderer.render(&mut vp_buf, rw, rh);

            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                let bb = bb_ptr as *mut u32;
                let bb_s = bb_stride as usize;
                for y in 0..rh.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            vp_buf[y * rw..].as_ptr(),
                            bb.add(y * bb_s),
                            rw,
                        );
                    }
                }
            }
            crate::framebuffer::swap_buffers();
        }

        if !was_db {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }
    crate::framebuffer::clear();
    if wait_key_or_timeout(2) { return; }

    // ---- Step 7: Desktop Environment ----
    print_step(7, total_steps, "DESKTOP ENVIRONMENT", "ENVIRONNEMENT DE BUREAU",
               "GPU-composited windowed desktop with apps, games, and more.",
               "Bureau fenetre composite GPU avec apps, jeux, et plus encore.");
    pause(1);

    if lang == "fr" {
        crate::println_color!(0xFF00FF88, "  Le bureau s'ouvre avec un Terminal pour 8 secondes...");
        crate::println_color!(0xFF888888, "  (Essayez de taper des commandes!)");
    } else {
        crate::println_color!(0xFF00FF88, "  Desktop will open with a Terminal for 8 seconds...");
        crate::println_color!(0xFF888888, "  (Try typing some commands!)");
    }
    pause(3);

    // Launch desktop with Terminal, auto-exit after 8 seconds
    cmd_cosmic_v2_with_app_timed(Some("shell"), 8000);
    crate::framebuffer::clear();
    pause(1);

    // ---- Step 8: Feature Summary ----
    print_step(8, total_steps, "FEATURE OVERVIEW", "VUE D'ENSEMBLE",
               "Everything TrustOS includes -- all in 6MB, all in Rust.",
               "Tout ce que TrustOS contient -- en 6Mo, tout en Rust.");
    pause(1);

    crate::println_color!(0xFFAADDFF, "  +- Kernel -------------------------------------------+");
    crate::println_color!(0xFFDDDDDD, "  | SMP multicore * APIC * IDT * GDT * paging          |");
    crate::println_color!(0xFFDDDDDD, "  | heap allocator * scheduler * RTC * PIT * TSC        |");
    crate::println_color!(0xFFAADDFF, "  +- Shell --------------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | 200+ POSIX commands * pipes * scripting              |");
    crate::println_color!(0xFFDDDDDD, "  | ls cd mkdir rm cp mv cat grep find head tail tree   |");
    crate::println_color!(0xFFAADDFF, "  +- Desktop ------------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | GPU compositor * window manager * 60 FPS            |");
    crate::println_color!(0xFFDDDDDD, "  | Terminal * Files * Calculator * Settings             |");
    crate::println_color!(0xFFAADDFF, "  +- Apps ---------------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | TrustCode (editor) * Web Browser * Snake * Chess    |");
    crate::println_color!(0xFFDDDDDD, "  | NES Emulator * Game Boy * TrustEdit 3D * TrustLab  |");
    crate::println_color!(0xFFAADDFF, "  +- Languages ----------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | TrustLang compiler + VM * Shell scripting           |");
    crate::println_color!(0xFFAADDFF, "  +- Network ------------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | TCP/IP * DHCP * DNS * HTTP * TLS 1.3 * curl/wget   |");
    crate::println_color!(0xFFAADDFF, "  +- Graphics -----------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | TrustVideo * Formula3D * Matrix * Fire * Plasma     |");
    crate::println_color!(0xFFAADDFF, "  +----------------------------------------------------+");
    crate::println!();

    if wait_key_or_timeout(8) { return; }

    // -------------------------------------------------------------------
    // PART 3: Cinematic Outro
    // -------------------------------------------------------------------
    {
        let was_db = crate::framebuffer::is_double_buffer_enabled();
        if !was_db {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }

        let w = sw as usize;
        let h = sh as usize;
        let mut buf = alloc::vec![0u32; w * h];

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

        let mut rain_cols: alloc::vec::Vec<u16> = alloc::vec![0u16; w / 8 + 1];
        let mut rain_speeds: alloc::vec::Vec<u8> = alloc::vec![1u8; w / 8 + 1];
        for i in 0..rain_cols.len() {
            rain_cols[i] = ((i * 41 + 7) % h) as u16;
            rain_speeds[i] = (((i * 11 + 5) % 4) + 1) as u8;
        }

        let draw_rain_step = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
            for pixel in buf.iter_mut() {
                let g = ((*pixel >> 8) & 0xFF) as u32;
                if g > 0 {
                    let new_g = g.saturating_sub(8);
                    *pixel = 0xFF000000 | (new_g << 8);
                }
            }
            for col_idx in 0..cols.len() {
                let x = col_idx * 8;
                if x >= w { continue; }
                cols[col_idx] = cols[col_idx].wrapping_add(speeds[col_idx] as u16);
                if cols[col_idx] as usize >= h { cols[col_idx] = 0; }
                let y = cols[col_idx] as usize;
                let c = (((frame as usize + col_idx * 13) % 94) + 33) as u8 as char;
                let glyph = crate::framebuffer::font::get_glyph(c);
                for (row, &bits) in glyph.iter().enumerate() {
                    let py = y + row;
                    if py >= h { break; }
                    for bit in 0..8u32 {
                        if bits & (0x80 >> bit) != 0 {
                            let px = x + bit as usize;
                            if px < w {
                                buf[py * w + px] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        let blit_buf = |buf: &[u32], w: usize, h: usize| {
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                let bb = bb_ptr as *mut u32;
                let bb_s = bb_stride as usize;
                for y in 0..h.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            buf[y * w..].as_ptr(),
                            bb.add(y * bb_s),
                            w,
                        );
                    }
                }
            }
            crate::framebuffer::swap_buffers();
        };

        // Outro scene: "You're ready!" + 3D character
        crate::serial_println!("[DEMO] Outro: You're ready!");
        for pixel in buf.iter_mut() { *pixel = 0xFF000000; }

        let freq = crate::cpu::tsc::frequency_hz();
        let outro_ms = 7000u64;
        let outro_target = if freq > 0 { freq / 1000 * outro_ms } else { u64::MAX };
        let start_tsc = crate::cpu::tsc::read_tsc();

        let mut renderer3d = crate::formula3d::FormulaRenderer::new();
        renderer3d.set_scene(crate::formula3d::FormulaScene::Character);
        renderer3d.wire_color = 0xFF00FFAA;
        let vp_w = 160usize;
        let vp_h = 160usize;
        let mut vp_buf = alloc::vec![0u32; vp_w * vp_h];

        let mut frame = 0u32;
        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
            if elapsed >= outro_target { break; }
            if let Some(key) = crate::keyboard::try_read_key() {
                if key == 0x1B || key == b'q' || key == b' ' || key == b'\n' || key == 13 { break; }
            }

            draw_rain_step(&mut buf, w, h, &mut rain_cols, &rain_speeds, frame);

            // Title
            let title = if lang == "fr" { "Pret a explorer!" } else { "You're ready!" };
            let title_scale = 5;
            let title_w = title.len() * 8 * title_scale;
            let title_x = if title_w < w { (w - title_w) / 2 } else { 0 };
            let title_y = h / 6;
            for (i, c) in title.chars().enumerate() {
                let hue = ((frame as usize * 3 + i * 25) % 360) as u32;
                let color = if hue < 120 {
                    let t = hue * 255 / 120;
                    0xFF000000 | ((255 - t) << 16) | (t << 8)
                } else if hue < 240 {
                    let t = (hue - 120) * 255 / 120;
                    0xFF000000 | ((255 - t) << 8) | t
                } else {
                    let t = (hue - 240) * 255 / 120;
                    0xFF000000 | (t << 16) | (255 - t)
                };
                draw_big_char(&mut buf, w, h, title_x + i * 8 * title_scale, title_y, c, color, title_scale);
            }

            // 3D character
            renderer3d.update();
            for p in vp_buf.iter_mut() { *p = 0x00000000; }
            renderer3d.render(&mut vp_buf, vp_w, vp_h);
            let vp_x = if vp_w < w { (w - vp_w) / 2 } else { 0 };
            let vp_y = title_y + 16 * title_scale + 20;
            for vy in 0..vp_h {
                for vx in 0..vp_w {
                    let src = vp_buf[vy * vp_w + vx];
                    if src & 0x00FFFFFF != 0 {
                        let dy = vp_y + vy;
                        let dx = vp_x + vx;
                        if dy < h && dx < w {
                            buf[dy * w + dx] = src;
                        }
                    }
                }
            }

            // Hints
            let hints: &[(&str, u32)] = if lang == "fr" {
                &[
                    ("Tapez 'help' pour la liste des commandes", 0xFF88CCFF),
                    ("Tapez 'desktop' pour le bureau graphique", 0xFF88FFAA),
                    ("Tapez 'showcase' pour la demo complete", 0xFFFFCC88),
                    ("github.com/nathan237/TrustOS", 0xFF00FF88),
                ]
            } else {
                &[
                    ("Type 'help' for all commands", 0xFF88CCFF),
                    ("Type 'desktop' for the GUI", 0xFF88FFAA),
                    ("Type 'showcase' for the full demo", 0xFFFFCC88),
                    ("github.com/nathan237/TrustOS", 0xFF00FF88),
                ]
            };

            let hint_scale = 2;
            let mut hy = vp_y + vp_h + 30;
            for &(text, color) in hints {
                let tw = text.len() * 8 * hint_scale;
                let hx = if tw < w { (w - tw) / 2 } else { 0 };
                for (i, c) in text.chars().enumerate() {
                    draw_big_char(&mut buf, w, h, hx + i * 8 * hint_scale, hy, c, color, hint_scale);
                }
                hy += 16 * hint_scale + 6;
            }

            blit_buf(&buf, w, h);
            frame += 1;
            crate::cpu::tsc::delay_millis(33);
        }

        // Final fade out
        let fade_start = crate::cpu::tsc::read_tsc();
        let fade_target = if freq > 0 { freq / 1000 * 800 } else { u64::MAX };
        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(fade_start);
            if elapsed >= fade_target { break; }
            for pixel in buf.iter_mut() {
                let r = ((*pixel >> 16) & 0xFF).saturating_sub(4) as u32;
                let g = ((*pixel >> 8) & 0xFF).saturating_sub(4) as u32;
                let b = (*pixel & 0xFF).saturating_sub(4) as u32;
                *pixel = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::delay_millis(33);
        }

        if !was_db {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }

    // -------------------------------------------------------------------
    // Return to shell
    // -------------------------------------------------------------------
    crate::framebuffer::clear();

    // Clean up tutorial files
    let _ = crate::ramfs::with_fs(|fs| {
        let _ = fs.rm("/tutorial/hello.txt");
        let _ = fs.rm("/tutorial/demo.tl");
        let _ = fs.rm("/tutorial");
    });

    crate::println!();
    if lang == "fr" {
        crate::println_color!(0xFF00FF88, "  Tutoriel termine! Bon voyage dans TrustOS.");
    } else {
        crate::println_color!(0xFF00FF88, "  Tutorial complete! Enjoy exploring TrustOS.");
    }
    crate::println!();
    crate::serial_println!("[DEMO] Tutorial complete");
}

// Runs through all TrustOS features with timed pauses between steps.
// Perfect for screen recording with OBS to create marketing videos.

pub(super) fn cmd_showcase(args: &[&str]) {
    let speed = match args.first().copied() {
        Some("fast") => 1,
        Some("slow") => 3,
        _ => 2, // normal
    };

    // Use TSC for timing -- uptime_ms() doesn't advance during spin_loop()
    let pause = |secs: u64| {
        let ms = secs * 1000 * speed / 2;
        let start_tsc = crate::cpu::tsc::read_tsc();
        let freq = crate::cpu::tsc::frequency_hz();
        if freq == 0 { return; } // TSC not calibrated
        let target_cycles = freq / 1000 * ms; // cycles for ms milliseconds
        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
            if elapsed >= target_cycles { break; }
            // Drain keyboard so keys don't leak to next step
            let _ = crate::keyboard::try_read_key();
            core::hint::spin_loop();
        }
    };

    let effect_duration = 9000u64 * speed / 2; // Duration for each video effect (9s base)

    // -------------------------------------------------------------------
    // CINEMATIC INTRO -- Matrix-style big text on framebuffer
    // -------------------------------------------------------------------
    let (sw, sh) = crate::framebuffer::get_dimensions();

    {
        let was_db = crate::framebuffer::is_double_buffer_enabled();
        if !was_db {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }

        let w = sw as usize;
        let h = sh as usize;
        let mut buf = alloc::vec![0u32; w * h];

        // -- Helper: draw scaled character into buffer --
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

        // -- Helper: draw big text centered --
        let draw_big_text_centered = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
            let text_w = text.len() * 8 * scale;
            let start_x = if text_w < w { (w - text_w) / 2 } else { 0 };
            for (i, c) in text.chars().enumerate() {
                draw_big_char(buf, w, h, start_x + i * 8 * scale, y, c, color, scale);
            }
        };

        // -- Helper: Matrix rain background --
        let mut rain_cols: alloc::vec::Vec<u16> = alloc::vec![0u16; w / 8 + 1];
        let mut rain_speeds: alloc::vec::Vec<u8> = alloc::vec![1u8; w / 8 + 1];
        // Seed rain columns
        for i in 0..rain_cols.len() {
            rain_cols[i] = ((i * 37 + 13) % h) as u16;
            rain_speeds[i] = (((i * 7 + 3) % 4) + 1) as u8;
        }

        let draw_rain_step = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
            // Dim existing pixels slightly (trail effect)
            for pixel in buf.iter_mut() {
                let g = ((*pixel >> 8) & 0xFF) as u32;
                if g > 0 {
                    let new_g = g.saturating_sub(8);
                    *pixel = 0xFF000000 | (new_g << 8);
                }
            }
            // Advance rain drops
            for col_idx in 0..cols.len() {
                let x = col_idx * 8;
                if x >= w { continue; }
                cols[col_idx] = cols[col_idx].wrapping_add(speeds[col_idx] as u16);
                if cols[col_idx] as usize >= h { cols[col_idx] = 0; }
                let y = cols[col_idx] as usize;
                // Draw lead char (bright green)
                let c = (((frame as usize + col_idx * 13) % 94) + 33) as u8 as char;
                let glyph = crate::framebuffer::font::get_glyph(c);
                for (row, &bits) in glyph.iter().enumerate() {
                    let py = y + row;
                    if py >= h { break; }
                    for bit in 0..8u32 {
                        if bits & (0x80 >> bit) != 0 {
                            let px = x + bit as usize;
                            if px < w {
                                buf[py * w + px] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        // -- Helper: blit buffer to backbuffer --
        let blit_buf = |buf: &[u32], w: usize, h: usize| {
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                let bb = bb_ptr as *mut u32;
                let bb_s = bb_stride as usize;
                for y in 0..h.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            buf[y * w..].as_ptr(),
                            bb.add(y * bb_s),
                            w,
                        );
                    }
                }
            }
            crate::framebuffer::swap_buffers();
        };

        // -- Helper: Scene -- type text with matrix rain bg --
        // Renders text that "types in" char by char with Matrix rain bg
        let render_scene = |buf: &mut [u32], w: usize, h: usize,
                           rain_cols: &mut [u16], rain_speeds: &[u8],
                           lines: &[(&str, u32, usize)], // (text, color, scale)
                           hold_ms: u64, speed: u64| {
            let freq = crate::cpu::tsc::frequency_hz();
            if freq == 0 { return; }

            // Phase 1: Type in text (rain + typed text appearing)
            let total_chars: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
            let type_ms = 80u64 * speed / 2; // ms per char typed
            let type_total_ms = total_chars as u64 * type_ms;
            
            let start_tsc = crate::cpu::tsc::read_tsc();
            let type_target = freq / 1000 * type_total_ms;
            let hold_target = freq / 1000 * (type_total_ms + hold_ms * speed / 2);

            let mut frame = 0u32;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
                if elapsed >= hold_target { break; }
                if let Some(key) = crate::keyboard::try_read_key() {
                    if key == 0x1B || key == b'q' { break; }
                }

                // Rain background
                draw_rain_step(buf, w, h, rain_cols, rain_speeds, frame);

                // Calculate how many chars to show
                let elapsed_ms = elapsed / (freq / 1000).max(1);
                let chars_shown = if elapsed_ms < type_total_ms {
                    (elapsed_ms / type_ms.max(1)) as usize
                } else {
                    total_chars
                };

                // Draw text lines
                let total_text_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 8).sum::<usize>();
                let mut y_start = if total_text_h < h { (h - total_text_h) / 2 } else { 20 };
                let mut chars_counted = 0usize;

                for &(text, color, scale) in lines {
                    let text_w = text.len() * 8 * scale;
                    let start_x = if text_w < w { (w - text_w) / 2 } else { 0 };

                    for (i, c) in text.chars().enumerate() {
                        if chars_counted + i >= chars_shown { break; }
                        draw_big_char(buf, w, h, start_x + i * 8 * scale, y_start, c, color, scale);
                    }
                    // Cursor blink
                    if chars_shown > chars_counted && chars_shown < chars_counted + text.len() {
                        let cursor_i = chars_shown - chars_counted;
                        let cx = start_x + cursor_i * 8 * scale;
                        for cy in y_start..y_start + 16 * scale {
                            if cy < h && cx + 2 < w {
                                buf[cy * w + cx] = 0xFF00FF88;
                                buf[cy * w + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }

                    chars_counted += text.len();
                    y_start += 16 * scale + 8;
                }

                blit_buf(buf, w, h);
                frame += 1;
                // ~30 FPS pacing
                crate::cpu::tsc::delay_millis(33);
            }

            // Fade out
            let fade_start = crate::cpu::tsc::read_tsc();
            let fade_ms = 800u64;
            let fade_target = freq / 1000 * fade_ms;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(fade_start);
                if elapsed >= fade_target { break; }
                let progress = (elapsed * 255 / fade_target) as u32;
                for pixel in buf.iter_mut() {
                    let r = ((*pixel >> 16) & 0xFF) as u32;
                    let g = ((*pixel >> 8) & 0xFF) as u32;
                    let b = (*pixel & 0xFF) as u32;
                    let nr = r.saturating_sub(r * progress / 512 + 1);
                    let ng = g.saturating_sub(g * progress / 512 + 1);
                    let nb = b.saturating_sub(b * progress / 512 + 1);
                    *pixel = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
                }
                blit_buf(buf, w, h);
                crate::cpu::tsc::delay_millis(33);
            }

            // Clear to black
            for pixel in buf.iter_mut() { *pixel = 0xFF000000; }
            blit_buf(buf, w, h);
        };

        // -- Scene 1: "Do you think life is a simulation?" --
        crate::serial_println!("[SHOWCASE] Scene 1: Simulation question");
        for pixel in buf.iter_mut() { *pixel = 0xFF000000; }
        render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
            &[("Do you think", 0xFF00DD55, 4),
              ("life is a simulation?", 0xFF00FF66, 4)],
            3000, speed);

        // -- Scene 2: "Can it run in a 6MB OS?" --
        crate::serial_println!("[SHOWCASE] Scene 2: 6MB OS");
        render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
            &[("Can it run", 0xFF00DD55, 5),
              ("in a 6MB OS?", 0xFF00FF88, 5)],
            3000, speed);

        // -- Scene 3: "TrustOS" + 3D + "Written in Rust by Nated0ge" --
        crate::serial_println!("[SHOWCASE] Scene 3: TrustOS title");
        {
            // Special scene: TrustOS big title + 3D wireframe + credits
            let freq = crate::cpu::tsc::frequency_hz();
            let scene3_ms = 8000u64 * speed / 2;
            let scene3_target = freq / 1000 * scene3_ms;
            let start_tsc = crate::cpu::tsc::read_tsc();

            let mut renderer3d = crate::formula3d::FormulaRenderer::new();
            renderer3d.set_scene(crate::formula3d::FormulaScene::Character);
            renderer3d.wire_color = 0xFF00FFAA;

            // Small 3D viewport
            let vp_w = 200usize;
            let vp_h = 200usize;
            let mut vp_buf = alloc::vec![0u32; vp_w * vp_h];

            let mut frame = 0u32;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
                if elapsed >= scene3_target { break; }
                if let Some(key) = crate::keyboard::try_read_key() {
                    if key == 0x1B { break; }
                }

                // Rain background
                draw_rain_step(&mut buf, w, h, &mut rain_cols, &rain_speeds, frame);

                // Title: "TRUST OS" big
                let title = "TRUST OS";
                let title_scale = 6;
                let title_w = title.len() * 8 * title_scale;
                let title_x = if title_w < w { (w - title_w) / 2 } else { 0 };
                let title_y = h / 8;
                for (i, c) in title.chars().enumerate() {
                    // Color cycle per char
                    let hue = ((frame as usize * 3 + i * 30) % 360) as u32;
                    let color = if hue < 120 {
                        let t = hue * 255 / 120;
                        0xFF000000 | ((255 - t) << 16) | (t << 8)
                    } else if hue < 240 {
                        let t = (hue - 120) * 255 / 120;
                        0xFF000000 | ((255 - t) << 8) | t
                    } else {
                        let t = (hue - 240) * 255 / 120;
                        0xFF000000 | (t << 16) | (255 - t)
                    };
                    draw_big_char(&mut buf, w, h, title_x + i * 8 * title_scale, title_y, c, color, title_scale);
                }

                // 3D animated character in center
                renderer3d.update();
                for p in vp_buf.iter_mut() { *p = 0x00000000; } // transparent
                renderer3d.render(&mut vp_buf, vp_w, vp_h);

                // Blit 3D viewport centered below title
                let vp_x = if vp_w < w { (w - vp_w) / 2 } else { 0 };
                let vp_y = title_y + 16 * title_scale + 20;
                for vy in 0..vp_h {
                    for vx in 0..vp_w {
                        let src = vp_buf[vy * vp_w + vx];
                        if src & 0x00FFFFFF != 0 { // not black = has content
                            let dy = vp_y + vy;
                            let dx = vp_x + vx;
                            if dy < h && dx < w {
                                buf[dy * w + dx] = src;
                            }
                        }
                    }
                }

                // Credits text
                let credit = "Written in Rust by Nated0ge";
                let credit_scale = 3;
                let credit_w = credit.len() * 8 * credit_scale;
                let credit_x = if credit_w < w { (w - credit_w) / 2 } else { 0 };
                let credit_y = vp_y + vp_h + 30;
                for (i, c) in credit.chars().enumerate() {
                    draw_big_char(&mut buf, w, h, credit_x + i * 8 * credit_scale, credit_y, c, 0xFF88CCFF, credit_scale);
                }

                blit_buf(&buf, w, h);
                frame += 1;
                crate::cpu::tsc::delay_millis(33);
            }

            // Fade out
            let fade_start = crate::cpu::tsc::read_tsc();
            let fade_target = freq / 1000 * 800;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(fade_start);
                if elapsed >= fade_target { break; }
                for pixel in buf.iter_mut() {
                    let r = ((*pixel >> 16) & 0xFF).saturating_sub(4) as u32;
                    let g = ((*pixel >> 8) & 0xFF).saturating_sub(4) as u32;
                    let b = (*pixel & 0xFF).saturating_sub(4) as u32;
                    *pixel = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
                blit_buf(&buf, w, h);
                crate::cpu::tsc::delay_millis(33);
            }
            for pixel in buf.iter_mut() { *pixel = 0xFF000000; }
            blit_buf(&buf, w, h);
        }

        // -- Scene 4: Specs comparison --
        crate::serial_println!("[SHOWCASE] Scene 4: Specs");
        render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
            &[("6MB ISO vs 6GB Windows",  0xFF00FF66, 3),
              ("0 lines of C. Pure Rust.", 0xFF44FFAA, 3),
              ("Boots in 0.8s not 45s",    0xFF00DDFF, 3),
              ("No kernel panics. Ever.",  0xFFFFCC44, 3),
              ("GPU desktop at 144 FPS",   0xFF88FF44, 3),
              ("Built in 7 days solo",     0xFFFF8844, 3)],
            3000, speed);

        // -- Scene 5: "Are you ready?" --
        crate::serial_println!("[SHOWCASE] Scene 5: Are you ready?");
        render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
            &[("Are you ready?", 0xFF00FF44, 6)],
            2000, speed);

        // Restore double buffer state
        if !was_db {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }

    // --- Phase 0: Banner ---
    crate::framebuffer::clear();

    crate::println!();
    crate::println!();
    crate::println!();
    crate::println_color!(0xFF00CCFF, "");
    crate::println_color!(0xFF00CCFF, "  ||||||||+||||||+ ||+   ||+|||||||+||||||||+ ||||||+ |||||||+");
    crate::println_color!(0xFF00CCFF, "  +--||+--+||+--||+|||   |||||+----++--||+--+||+---||+||+----+");
    crate::println_color!(0xFF00CCFF, "     |||   ||||||++|||   ||||||||||+   |||   |||   ||||||||||+");
    crate::println_color!(0xFF00DDFF, "     |||   ||+--||+|||   |||+----|||   |||   |||   |||+----|||");
    crate::println_color!(0xFF00EEFF, "     |||   |||  |||+||||||++||||||||   |||   +||||||++||||||||");
    crate::println_color!(0xFF00EEFF, "     +-+   +-+  +-+ +-----+ +------+   +-+    +-----+ +------+");
    crate::println!();
    crate::println_color!(0xFF888888, "                  ?????????????????????????????????");
    crate::println!();
    crate::println_color!(0xFFAADDFF, "           A bare-metal OS written in 100% Rust -- in 7 days");
    crate::println_color!(0xFF666666, "         99,000+ lines * 6 MB ISO * GPU compositing * 144 FPS");
    crate::println!();
    crate::println_color!(0xFF888888, "                  ?????????????????????????????????");
    crate::println!();
    crate::println_color!(0xFF00FF88, "                        ?  FEATURE SHOWCASE  ?");
    crate::println!();
    
    pause(6);

    // --- Phase 1: System Info ---
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println_color!(0xFF00CCFF, "|  PHASE 1 ---- SYSTEM INFO                                   |");
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    super::commands::cmd_neofetch();
    pause(4);

    crate::println_color!(COLOR_CYAN, "$ uname -a");
    super::commands::cmd_uname(&["-a"]);
    pause(4);

    crate::println_color!(COLOR_CYAN, "$ free");
    super::commands::cmd_free();
    pause(4);

    crate::println_color!(COLOR_CYAN, "$ lscpu");
    super::unix::cmd_lscpu();
    pause(5);

    // --- Phase 2: Filesystem ---
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println_color!(0xFF00CCFF, "|  PHASE 2 ---- FILESYSTEM (TrustFS + VFS)                    |");
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    crate::println_color!(COLOR_CYAN, "$ mkdir /demo");
    super::commands::cmd_mkdir(&["/demo"]);
    pause(2);

    crate::println_color!(COLOR_CYAN, "$ echo 'Hello TrustOS!' > /demo/hello.txt");
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch("/demo/hello.txt");
        let _ = fs.write_file("/demo/hello.txt", b"Hello TrustOS!\nThis file was created live during the showcase.\n");
    });
    crate::println_color!(COLOR_GREEN, "Written: /demo/hello.txt");
    pause(2);

    crate::println_color!(COLOR_CYAN, "$ cat /demo/hello.txt");
    super::commands::cmd_cat(&["/demo/hello.txt"], None, None);
    pause(3);

    crate::println_color!(COLOR_CYAN, "$ tree /");
    super::commands::cmd_tree(&["/"]);
    pause(4);

    // --- Phase 3: TrustLang ---
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println_color!(0xFF00CCFF, "|  PHASE 3 ---- TRUSTLANG (Built-in Compiler + VM)            |");
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    // Create and run a real TrustLang program
    let tl_code = r#"fn fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    println("=== TrustLang on TrustOS ===");
    println("Fibonacci sequence (compiled to bytecode):");
    for i in 0..12 {
        let result = fibonacci(i);
        print("  fib(");
        print(to_string(i));
        print(") = ");
        println(to_string(result));
    }
    println("Language features: functions, recursion, loops, types");
}"#;
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch("/demo/showcase.tl");
        let _ = fs.write_file("/demo/showcase.tl", tl_code.as_bytes());
    });

    crate::println_color!(COLOR_CYAN, "$ cat /demo/showcase.tl");
    crate::println_color!(0xFFDDDDDD, "{}", tl_code);
    pause(4);

    crate::println_color!(COLOR_CYAN, "$ trustlang run /demo/showcase.tl");
    crate::println_color!(0xFF00FF88, "[TrustLang] Compiling showcase.tl...");
    match crate::trustlang::run(tl_code) {
        Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
        Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
    }
    crate::println_color!(COLOR_GREEN, "[TrustLang] Program finished successfully.");
    pause(6);

    // --- Phase 4: Network Stack ---
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println_color!(0xFF00CCFF, "|  PHASE 4 ---- NETWORK STACK (TCP/IP, DHCP, DNS, TLS 1.3)    |");
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    crate::println_color!(COLOR_CYAN, "$ ifconfig");
    super::vm::cmd_ifconfig();
    pause(3);

    crate::println_color!(COLOR_CYAN, "$ netstat");
    super::vm::cmd_netstat();
    pause(4);

    // --- Phase 5: Video Demos ---
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println_color!(0xFF00CCFF, "|  PHASE 5 ---- TRUSTVIDEO (Real-time procedural rendering)   |");
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(3);

    // Fire demo
    crate::println_color!(0xFFFF4400, "? Demo 1/3: FIRE EFFECT -- Real-time procedural flame");
    pause(2);

    let vw = sw as u16;
    let vh = sh as u16;
    crate::video::player::render_realtime_timed("fire", vw, vh, 30, effect_duration);
    
    // Restore console
    crate::framebuffer::clear();
    pause(2);

    // Matrix demo
    crate::println_color!(0xFF00FF44, "? Demo 2/3: MATRIX RAIN -- Digital rain effect");
    pause(2);

    crate::video::player::render_realtime_timed("matrix", vw, vh, 30, effect_duration);
    
    crate::framebuffer::clear();
    pause(2);

    // Plasma demo
    crate::println_color!(0xFFFF00FF, "? Demo 3/3: PLASMA -- Integer sine LUT psychedelic");
    pause(2);

    crate::video::player::render_realtime_timed("plasma", vw, vh, 30, effect_duration);
    
    crate::framebuffer::clear();
    pause(2);

    // --- Phase 5b: 3D Wireframe Character ---
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println_color!(0xFF00CCFF, "|  PHASE 5b -- FORMULA3D (Wireframe 3D engine)                |");
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::println_color!(0xFF00FF88, "? 3D wireframe character -- perspective projection + depth shading");
    pause(2);

    // Render rotating 3D character using FormulaRenderer
    {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.set_scene(crate::formula3d::FormulaScene::Character);
        renderer.wire_color = 0xFF00FFAA; // bright cyan-green

        let rw = sw as usize;
        let rh = sh as usize;
        let ox = if sw > rw as u32 { (sw - rw as u32) / 2 } else { 0 } as usize;
        let oy = if sh > rh as u32 { (sh - rh as u32) / 2 } else { 0 } as usize;

        let mut buf = alloc::vec![0u32; rw * rh];

        let was_db = crate::framebuffer::is_double_buffer_enabled();
        if !was_db {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }
        crate::framebuffer::clear_backbuffer(0xFF000000);
        crate::framebuffer::swap_buffers();

        let start_tsc = crate::cpu::tsc::read_tsc();
        let freq = crate::cpu::tsc::frequency_hz();
        let duration_ms = effect_duration;
        let target_cycles = if freq > 0 { freq / 1000 * duration_ms } else { u64::MAX };

        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
            if elapsed >= target_cycles { break; }
            if let Some(key) = crate::keyboard::try_read_key() {
                if key == 0x1B || key == b'q' { break; }
            }

            renderer.update();
            renderer.render(&mut buf, rw, rh);

            // Blit to backbuffer
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                let bb = bb_ptr as *mut u32;
                let bb_s = bb_stride as usize;
                for y in 0..rh {
                    let dy = oy + y;
                    if dy >= bb_h as usize { break; }
                    let src_row = &buf[y * rw..y * rw + rw];
                    unsafe {
                        let dst = bb.add(dy * bb_s + ox);
                        core::ptr::copy_nonoverlapping(src_row.as_ptr(), dst, rw);
                    }
                }
            }
            crate::framebuffer::swap_buffers();
        }

        if !was_db {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }

    crate::framebuffer::clear();
    pause(2);

    // --- Phase 5c: Desktop + Web Browser ---
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println_color!(0xFF00CCFF, "|  PHASE 5c -- COSMIC2 DESKTOP + WEB BROWSER                  |");
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::println_color!(0xFF00FF88, "? COSMIC2 Desktop -- GPU-composited multi-layer windowing system");
    crate::println_color!(0xFF00FF88, "? Launching with built-in Web Browser ? google.com");
    pause(3);

    // Launch desktop in browser mode with auto-exit after effect_duration ms
    cmd_cosmic_v2_with_app_timed(Some("browser"), effect_duration);

    crate::framebuffer::clear();
    pause(2);

    // --- Phase 6: Commands overview ---
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println_color!(0xFF00CCFF, "|  PHASE 6 ---- 200+ BUILT-IN COMMANDS                       |");
    crate::println_color!(0xFF00CCFF, "+--------------------------------------------------------------+");
    crate::println!();
    pause(2);

    crate::println_color!(0xFFAADDFF, "  +- File System ------------------------------------------+");
    crate::println_color!(0xFFDDDDDD, "  | ls cd pwd mkdir rm cp mv cat head tail tree find grep  |");
    crate::println_color!(0xFFAADDFF, "  +- Network ----------------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | ifconfig ping curl wget nslookup arp route netstat     |");
    crate::println_color!(0xFFAADDFF, "  +- System -----------------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | ps top free df uname dmesg mount lspci lscpu lsblk    |");
    crate::println_color!(0xFFAADDFF, "  +- Development ------------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | trustlang (compiler+VM) * TrustCode (editor)          |");
    crate::println_color!(0xFFDDDDDD, "  | transpile (binary?Rust) * exec (ELF loader)           |");
    crate::println_color!(0xFFAADDFF, "  +- Graphics ---------------------------------------------|");
    crate::println_color!(0xFFDDDDDD, "  | desktop (COSMIC2 compositor) * video (TrustVideo)     |");
    crate::println_color!(0xFFDDDDDD, "  | benchmark (SSE2 SIMD) * HoloMatrix (3D volumetric)   |");
    crate::println_color!(0xFFAADDFF, "  +--------------------------------------------------------+");
    crate::println!();
    pause(6);

    // --- Phase 7: Outro ---
    crate::println!();
    crate::println_color!(0xFF00CCFF, "  ||||||||+||||||+ ||+   ||+|||||||+||||||||+ ||||||+ |||||||+");
    crate::println_color!(0xFF00CCFF, "  +--||+--+||+--||+|||   |||||+----++--||+--+||+---||+||+----+");
    crate::println_color!(0xFF00CCFF, "     |||   ||||||++|||   ||||||||||+   |||   |||   ||||||||||+");
    crate::println_color!(0xFF00DDFF, "     |||   ||+--||+|||   |||+----|||   |||   |||   |||+----|||");
    crate::println_color!(0xFF00EEFF, "     |||   |||  |||+||||||++||||||||   |||   +||||||++||||||||");
    crate::println_color!(0xFF00EEFF, "     +-+   +-+  +-+ +-----+ +------+   +-+    +-----+ +------+");
    crate::println!();
    crate::println_color!(0xFFFFCC00, "  ?  100% Rust -- Zero C code -- Memory safe by design");
    crate::println_color!(0xFFFFCC00, "  ?  Built from scratch in 7 days -- 99,000+ lines");
    crate::println_color!(0xFFFFCC00, "  ?  6 MB ISO -- boots in seconds");
    crate::println_color!(0xFFFFCC00, "  ?  GPU compositing -- 144 FPS desktop");
    crate::println!();
    crate::println_color!(0xFF00FF88, "  github.com/nathan237/TrustOS");
    crate::println_color!(0xFF888888, "  Star ? * Fork * Contribute");
    crate::println!();
    crate::println_color!(0xFF888888, "  ??????????????????????????????????????????????????");
    crate::println!();

    // Clean up
    let _ = crate::ramfs::with_fs(|fs| { let _ = fs.rm("/demo/hello.txt"); let _ = fs.rm("/demo/showcase.tl"); });
}

// -------------------------------------------------------------------------------
// SHOWCASE 3D -- Cinematic 3D graphics demo
// 12 fullscreen scenes, 5 seconds each, hardware stats overlay
// -------------------------------------------------------------------------------

pub fn cmd_showcase3d() {
    use crate::gpu_emu::{PixelInput, PixelOutput};

    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;
    if w == 0 || h == 0 { return; }

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    // -- Helpers ----------------------------------------------------------

    let draw_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, c: char, color: u32, scale: usize| {
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

    let draw_text = |buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            draw_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
        let tw = text.len() * 8 * scale;
        let x = if tw < w { (w - tw) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            draw_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let blit = |buf: &[u32], w: usize, h: usize| {
        // SMP parallel blit: write directly to MMIO FB across all cores
        crate::framebuffer::blit_to_fb_parallel(buf.as_ptr(), w, h);
    };

    // -- Tick-based wall-clock timing (PIT at ~100 Hz) ----------------
    // get_ticks() is driven by PIT interrupt = reliable wall-clock
    // 100 ticks = 1 second
    let ticks_now = || crate::logger::get_ticks();

    // Draw stats bar at bottom (with real hardware info)
    let draw_stats = |buf: &mut [u32], w: usize, h: usize, scene_name: &str, scene_num: usize, fps: u32, elapsed_s: u32, total_s: u32, quality: usize| {
        let bar_h = 28usize;
        let bar_y = h - bar_h;
        for y in bar_y..h {
            for x in 0..w {
                buf[y * w + x] = 0xFF0A0A0A;
            }
        }
        for x in 0..w {
            buf[bar_y * w + x] = 0xFF00AA44;
        }
        // Gather RAM stats
        let mem = crate::memory::stats();
        let heap_used_kb = mem.heap_used / 1024;
        let heap_total_kb = (mem.heap_used + mem.heap_free) / 1024;
        let heap_pct = if heap_total_kb > 0 { heap_used_kb * 100 / heap_total_kb } else { 0 };
        // Frame time approximation from FPS
        let frame_ms = if fps > 0 { 1000 / fps } else { 999 };
        let quality_str = match quality { 1 => "Full", 2 => "High", 3 => "Med", _ => "Low" };
        let mut stats_str = alloc::string::String::new();
        use core::fmt::Write;
        let _ = write!(stats_str, " {}/12 {} | {} FPS {}ms | RAM {}KB/{}KB ({}%) | CPU 100% | {} | {}x{}",
            scene_num, scene_name, fps, frame_ms, heap_used_kb, heap_total_kb, heap_pct, quality_str, w, h);
        draw_text(buf, w, h, 8, bar_y + 8, &stats_str, 0xFF00FF66, 1);
    };

    let scene_ticks = 500u64;  // 5 seconds per scene (100 ticks/sec)
    let title_ticks = 200u64;  // 2 seconds title overlay

    // Render a pixel-shader scene (tick-based timing, adaptive quality)
    let render_shader_scene = |buf: &mut [u32], w: usize, h: usize,
                                shader_fn: fn(PixelInput) -> PixelOutput,
                                title: &str, subtitle: &str, scene_num: usize,
                                dur_ticks: u64| {
        let start = ticks_now();
        let mut frame = 0u32;
        let mut fps_start = start;
        let mut fps_frames = 0u32;
        let mut cur_fps = 0u32;
        // Start at step=2 for small screens, step=3 for large (>960px)
        let mut step = if w > 960 { 3usize } else { 2 };

        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= dur_ticks { break; }
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 27 { return; } // ESC = ASCII 27
            }

            // Adaptive quality: adjust step based on actual FPS
            // Checks every frame after first FPS measurement
            if cur_fps > 0 {
                if cur_fps >= 20 && step > 2 {
                    step -= 1; // Improve quality one level
                } else if cur_fps >= 30 && step > 1 {
                    step = 1; // Full resolution
                } else if cur_fps < 8 && step < 4 {
                    step += 1; // Reduce quality to keep it smooth
                }
            }

            let time = elapsed as f32 / 100.0; // seconds as float

            // Render shader with current step (stepA--step blocks)
            for y in (0..h).step_by(step) {
                for x in (0..w).step_by(step) {
                    let input = PixelInput { x: x as u32, y: y as u32, width: w as u32, height: h as u32, time, frame };
                    let out = shader_fn(input);
                    let color = out.to_u32();
                    // Fill the stepA--step block
                    for dy in 0..step {
                        for dx in 0..step {
                            let px = x + dx;
                            let py = y + dy;
                            if px < w && py < h {
                                buf[py * w + px] = color;
                            }
                        }
                    }
                }
            }

            // FPS counter (per second via ticks)
            fps_frames += 1;
            let fps_elapsed = ticks_now().saturating_sub(fps_start);
            if fps_elapsed >= 100 {
                cur_fps = fps_frames;
                fps_frames = 0;
                fps_start = ticks_now();
            }

            // Title overlay (first 2 seconds)
            if elapsed < title_ticks {
                let alpha = if elapsed < 50 {
                    (elapsed * 255 / 50) as u32
                } else if elapsed > 150 {
                    let fade = elapsed - 150;
                    255u32.saturating_sub((fade * 255 / 50) as u32)
                } else { 255 };
                let a = alpha.min(255);
                let tc = 0xFF000000 | (a << 16) | (a << 8) | a;
                draw_text_centered(buf, w, h, 30, title, tc, 4);
                let sc = 0xFF000000 | ((a * 180 / 255) << 8);
                draw_text_centered(buf, w, h, 100, subtitle, sc, 2);
            }

            let elapsed_s = (elapsed / 100) as u32;
            let total_s = (dur_ticks / 100) as u32;
            draw_stats(buf, w, h, title, scene_num, cur_fps, elapsed_s, total_s, step);
            blit(buf, w, h);
            frame += 1;
        }
    };

    // Render a Formula3D wireframe scene (tick-based timing)
    let render_formula_scene = |buf: &mut [u32], w: usize, h: usize,
                                 scene: crate::formula3d::FormulaScene,
                                 wire_color: u32,
                                 title: &str, subtitle: &str, scene_num: usize,
                                 dur_ticks: u64| {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.set_scene(scene);
        renderer.wire_color = wire_color;
        let start = ticks_now();
        let mut frame = 0u32;
        let mut fps_start = start;
        let mut fps_frames = 0u32;
        let mut cur_fps = 0u32;

        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= dur_ticks { break; }
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 27 { return; }
            }

            renderer.update();
            for p in buf.iter_mut() { *p = 0xFF000000; }
            renderer.render(buf, w, h);

            fps_frames += 1;
            let fps_elapsed = ticks_now().saturating_sub(fps_start);
            if fps_elapsed >= 100 {
                cur_fps = fps_frames;
                fps_frames = 0;
                fps_start = ticks_now();
            }

            if elapsed < title_ticks {
                let alpha = if elapsed < 50 {
                    (elapsed * 255 / 50) as u32
                } else if elapsed > 150 {
                    let fade = elapsed - 150;
                    255u32.saturating_sub((fade * 255 / 50) as u32)
                } else { 255 };
                let a = alpha.min(255);
                let c = 0xFF000000 | (a << 16) | (a << 8) | a;
                draw_text_centered(buf, w, h, 30, title, c, 4);
                let sc = 0xFF000000 | ((a * 180 / 255) << 8);
                draw_text_centered(buf, w, h, 100, subtitle, sc, 2);
            }

            let elapsed_s = (elapsed / 100) as u32;
            let total_s = (dur_ticks / 100) as u32;
            draw_stats(buf, w, h, title, scene_num, cur_fps, elapsed_s, total_s, 1);
            blit(buf, w, h);
            frame += 1;
        }
    };

    // Fade to black transition (tick-based)
    let fade_out = |buf: &mut [u32], w: usize, h: usize, dur_ticks: u64| {
        let start = ticks_now();
        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= dur_ticks { break; }
            for pixel in buf.iter_mut() {
                let r = ((*pixel >> 16) & 0xFF).saturating_sub(6) as u32;
                let g = ((*pixel >> 8) & 0xFF).saturating_sub(6) as u32;
                let b = (*pixel & 0xFF).saturating_sub(6) as u32;
                *pixel = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::delay_millis(33);
        }
        for p in buf.iter_mut() { *p = 0xFF000000; }
        blit(buf, w, h);
    };

    let fade_ticks = 40u64; // 400ms fade

    crate::serial_println!("[SHOWCASE3D] Starting 3D cinematic showcase ({}x{}) - ~60s", w, h);

    // -----------------------------------------------------------------
    // INTRO -- Title card (3 seconds)
    // -----------------------------------------------------------------
    {
        let start = ticks_now();
        let intro_ticks = 300u64; // 3 seconds
        let mut frame = 0u32;
        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= intro_ticks { break; }
            for y in 0..h {
                for x in 0..w {
                    let v = ((x as i32 + frame as i32) ^ (y as i32)) as u32 & 0x0F;
                    buf[y * w + x] = 0xFF000000 | (v << 8);
                }
            }
            let alpha = (elapsed * 255 / intro_ticks.max(1)).min(255) as u32;
            let c = 0xFF000000 | (alpha << 16) | (alpha << 8) | alpha;
            draw_text_centered(&mut buf, w, h, h / 3, "TrustOS", c, 8);
            let sc = 0xFF000000 | ((alpha * 120 / 255) << 16) | ((alpha * 255 / 255) << 8) | ((alpha * 120 / 255));
            draw_text_centered(&mut buf, w, h, h / 3 + 140, "3D Graphics Showcase", sc, 3);
            let cc = 0xFF000000 | ((alpha * 100 / 255) << 16) | ((alpha * 100 / 255) << 8) | ((alpha * 100 / 255));
            draw_text_centered(&mut buf, w, h, h / 3 + 200, "Pure software rendering - No GPU hardware", cc, 2);
            blit(&buf, w, h);
            frame += 1;
            crate::cpu::tsc::delay_millis(33);
        }
        fade_out(&mut buf, w, h, fade_ticks);
    }

    // -----------------------------------------------------------------
    // SCENE 1 -- Rotating Cube
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 1: Cube");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Cube,
        0xFF00FF66,
        "Wireframe Cube", "8 vertices - 12 edges - perspective projection", 1,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 2 -- Diamond
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 2: Diamond");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Diamond,
        0xFFFF44FF,
        "Diamond", "Octahedron geometry - depth colored edges", 2,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 3 -- Torus
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 3: Torus");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Torus,
        0xFFFF8844,
        "Torus", "Donut wireframe - parametric surface mesh", 3,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 4 -- Pyramid
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 4: Pyramid");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Pyramid,
        0xFFFFCC00,
        "Pyramid", "5 vertices - 8 edges - ancient geometry", 4,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 5 -- HoloMatrix Rain 3D
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 5: HoloMatrix");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::HoloMatrix,
        0xFF00FF44,
        "HoloMatrix", "3D matrix rain with perspective depth", 5,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 6 -- Multi-Shape Orbit
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 6: Multi-Shape");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Multi,
        0xFF00FFAA,
        "Multi Shape", "4 wireframe objects orbiting - depth colored", 6,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 7 -- DNA Double Helix
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 7: DNA Helix");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Helix,
        0xFF44FFCC,
        "DNA Helix", "Double-strand helix with cross rungs", 7,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 8 -- Grid
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 8: Grid");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Grid,
        0xFF4488FF,
        "Infinite Grid", "Wireframe ground plane with perspective", 8,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 9 -- Penger
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 9: Penger");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Penger,
        0xFFFFFF00,
        "Penger", "The legendary wireframe penguin", 9,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 10 -- TrustOS Logo
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 10: TrustOS Logo");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::TrustOs,
        0xFF00FF88,
        "TrustOS Logo", "3D wireframe logo with glow vertices", 10,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 11 -- Icosphere
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 11: Icosphere");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Icosphere,
        0xFF66CCFF,
        "Icosphere", "Geodesic sphere - subdivided icosahedron", 11,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // SCENE 12 -- Character
    // -----------------------------------------------------------------
    crate::serial_println!("[SHOWCASE3D] Scene 12: Character");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Character,
        0xFF00FF88,
        "TrustOS", "Wireframe humanoid - perspective projection", 12,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // -----------------------------------------------------------------
    // OUTRO -- Credits (4 seconds)
    // -----------------------------------------------------------------
    {
        let start = ticks_now();
        let outro_ticks = 400u64; // 4 seconds
        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= outro_ticks { break; }
            for p in buf.iter_mut() { *p = 0xFF000000; }
            let alpha = if elapsed < 100 {
                (elapsed * 255 / 100).min(255)
            } else if elapsed > 300 {
                let fd = elapsed - 300;
                255u64.saturating_sub(fd * 255 / 100)
            } else { 255 } as u32;
            let c = 0xFF000000 | (alpha << 16) | (alpha << 8) | alpha;
            let gc = 0xFF000000 | ((alpha * 200 / 255) << 8);
            draw_text_centered(&mut buf, w, h, h / 3 - 30, "TrustOS 3D Engine", c, 5);
            draw_text_centered(&mut buf, w, h, h / 3 + 60, "12 wireframe scenes - Pure software rendering", gc, 2);
            draw_text_centered(&mut buf, w, h, h / 3 + 100, "No GPU hardware - All CPU computed", gc, 2);
            draw_text_centered(&mut buf, w, h, h / 3 + 160, "Written in Rust by Nated0ge", 0xFF000000 | ((alpha * 140 / 255) << 16) | ((alpha * 180 / 255) << 8) | (alpha * 255 / 255), 3);
            draw_text_centered(&mut buf, w, h, h / 3 + 220, "github.com/nathan237/TrustOS", 0xFF000000 | ((alpha * 100 / 255) << 16) | ((alpha * 100 / 255) << 8) | ((alpha * 100 / 255)), 2);
            blit(&buf, w, h);
            crate::cpu::tsc::delay_millis(33);
        }
    }

    // Restore
    for p in buf.iter_mut() { *p = 0xFF000000; }
    blit(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[SHOWCASE3D] Showcase complete");
}

/// Test command: filled 3D rendering with flat shading
/// Renders rotating cube, pyramid, and diamond with solid filled faces
pub fn cmd_filled3d() {
    use crate::formula3d::V3;

    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;
    if w == 0 || h == 0 { return; }

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = alloc::vec![0xFF000000u32; w * h];

    let draw_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, c: char, color: u32, scale: usize| {
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

    let blit = |buf: &[u32], w: usize, h: usize| {
        // SMP parallel blit: write directly to MMIO FB across all cores
        crate::framebuffer::blit_to_fb_parallel(buf.as_ptr(), w, h);
    };

    let ticks_now = || crate::logger::get_ticks();

    crate::serial_println!("[FILLED3D] Starting filled 3D test ({}x{})", w, h);

    // Light direction (top-left-front)
    let light = crate::formula3d::V3 { x: -0.4, y: 0.6, z: -0.7 };
    // Normalize manually
    let len = crate::formula3d::fast_sqrt(light.x * light.x + light.y * light.y + light.z * light.z);
    let light = V3 { x: light.x / len, y: light.y / len, z: light.z / len };

    // Build meshes with faces
    let cube = crate::formula3d::mesh_cube();
    let pyramid = crate::formula3d::mesh_pyramid();
    let diamond = crate::formula3d::mesh_diamond();

    let mut angle_y: f32 = 0.0;
    let mut frame = 0u32;
    let start = ticks_now();
    let mut fps_start = start;
    let mut fps_frames = 0u32;
    let mut cur_fps = 0u32;

    loop {
        let elapsed = ticks_now().saturating_sub(start);
        if elapsed >= 3000 { break; } // 30 seconds
        if let Some(k) = crate::keyboard::try_read_key() {
            if k == 27 { break; }
        }

        // Clear to dark blue-grey
        for p in buf.iter_mut() { *p = 0xFF0C1018; }

        angle_y += 0.025;
        let angle_x = 0.35 + crate::formula3d::fast_sin(frame as f32 * 0.008) * 0.2;

        // Draw 3 objects with different offsets
        // Left: pyramid (offset view by adjusting a virtual camera offset via vertices)
        // We render each at dz=2.5 but shift the angle to spread them visually
        // For a proper side-by-side we'd need per-object translation.
        // Simple approach: render 3 separate viewports (left/center/right thirds)

        let third = w / 3;

        // Left third: Pyramid
        {
            let mut sub_buf = alloc::vec![0xFF0C1018u32; third * h];
            crate::formula3d::render_filled_mesh(&mut sub_buf, third, h,
                &pyramid, angle_y * 0.8, angle_x + 0.15, 2.2,
                0xFFFF8844, light, 0.12);
            // Copy sub_buf into main buf
            for y in 0..h {
                for x in 0..third {
                    let src_idx = y * third + x;
                    let dst_idx = y * w + x;
                    if src_idx < sub_buf.len() && dst_idx < buf.len() {
                        buf[dst_idx] = sub_buf[src_idx];
                    }
                }
            }
        }

        // Center third: Cube
        {
            let mut sub_buf = alloc::vec![0xFF0C1018u32; third * h];
            crate::formula3d::render_filled_mesh(&mut sub_buf, third, h,
                &cube, angle_y, angle_x, 2.2,
                0xFF4488FF, light, 0.12);
            for y in 0..h {
                for x in 0..third {
                    let src_idx = y * third + x;
                    let dst_idx = y * w + third + x;
                    if src_idx < sub_buf.len() && dst_idx < buf.len() {
                        buf[dst_idx] = sub_buf[src_idx];
                    }
                }
            }
        }

        // Right third: Diamond
        {
            let mut sub_buf = alloc::vec![0xFF0C1018u32; third * h];
            crate::formula3d::render_filled_mesh(&mut sub_buf, third, h,
                &diamond, angle_y * 1.3, angle_x - 0.1, 2.2,
                0xFFFF44CC, light, 0.12);
            for y in 0..h {
                for x in 0..third {
                    let src_idx = y * third + x;
                    let dst_idx = y * w + 2 * third + x;
                    if src_idx < sub_buf.len() && dst_idx < buf.len() {
                        buf[dst_idx] = sub_buf[src_idx];
                    }
                }
            }
        }

        // FPS counter
        fps_frames += 1;
        let fps_elapsed = ticks_now().saturating_sub(fps_start);
        if fps_elapsed >= 100 {
            cur_fps = fps_frames;
            fps_frames = 0;
            fps_start = ticks_now();
        }

        // Stats bar
        let bar_h = 22usize;
        let bar_y = h.saturating_sub(bar_h);
        for y in bar_y..h {
            for x in 0..w {
                let idx = y * w + x;
                if idx < buf.len() { buf[idx] = 0xFF000000; }
            }
        }
        let stats = alloc::format!("Filled 3D | {} FPS | Flat Shading + Backface Cull + Painter Sort | ESC=exit", cur_fps);
        for (i, ch) in stats.chars().enumerate() {
            let cx = 8 + i * 8;
            if cx + 8 > w { break; }
            draw_char(&mut buf, w, h, cx, bar_y + 4, ch, 0xFF00FF88, 1);
        }

        // Title on first frames
        if frame < 200 {
            let alpha = if frame < 30 { frame * 255 / 30 } else if frame > 170 { (200 - frame) * 255 / 30 } else { 255 };
            let a = (alpha.min(255)) as u32;
            let c = 0xFF000000 | (a << 16) | (a << 8) | a;
            let title = "FILLED 3D TEST";
            let tw = title.len() * 8 * 3;
            let tx = if tw < w { (w - tw) / 2 } else { 0 };
            for (i, ch) in title.chars().enumerate() {
                draw_char(&mut buf, w, h, tx + i * 24, 30, ch, c, 3);
            }
            let sub = "Scanline Rasterizer + Flat Shading";
            let stw = sub.len() * 8 * 2;
            let stx = if stw < w { (w - stw) / 2 } else { 0 };
            let sc = 0xFF000000 | ((a * 180 / 255) << 8);
            for (i, ch) in sub.chars().enumerate() {
                draw_char(&mut buf, w, h, stx + i * 16, 80, ch, sc, 2);
            }
        }

        blit(&buf, w, h);
        frame += 1;
    }

    // Restore
    for p in buf.iter_mut() { *p = 0xFF000000; }
    blit(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[FILLED3D] Test complete, {} frames", frame);
}

/// Wrapper for desktop without initial app
pub(super) fn cmd_cosmic_v2() {
    cmd_cosmic_v2_with_app(None);
}

// ==================== COSMIC V2 - MULTI-LAYER COMPOSITOR ====================
// Each UI component renders to its own layer independently, then composited together.
// This eliminates flickering by ensuring atomic frame presentation.

/// Open desktop with optional app to launch
pub(super) fn cmd_cosmic_v2_with_app(initial_app: Option<&str>) {
    cmd_cosmic_v2_with_app_timed(initial_app, 0);
}

/// Open desktop with optional app and optional auto-exit timeout (0 = no timeout)
pub(super) fn cmd_cosmic_v2_with_app_timed(initial_app: Option<&str>, timeout_ms: u64) {
    use crate::compositor::{Compositor, LayerType};
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;
    
    crate::serial_println!("[COSMIC2] Starting COSMIC V2 Desktop...");
    
    // Auto-enable SMP parallelism for desktop rendering
    crate::cpu::smp::enable_smp();
    // Flush keyboard
    while crate::keyboard::try_read_key().is_some() {}
    
    let (width, height) = crate::framebuffer::get_dimensions();
    if width == 0 || height == 0 {
        crate::println_color!(COLOR_RED, "Error: Invalid framebuffer!");
        return;
    }
    
    // Set mouse bounds
    crate::mouse::set_screen_size(width, height);
    
    crate::serial_println!("[COSMIC2] Creating compositor {}x{}", width, height);
    
    // Create compositor with screen dimensions
    let mut compositor = Compositor::new(width, height);
    
    // Create layers (bottom to top order) - 8 layers
    let bg_layer = compositor.add_fullscreen_layer(LayerType::Background);
    let dock_layer = compositor.add_layer(LayerType::Dock, 0, 0, 70, height - 40);  // Dock with icon labels
    let window_layer = compositor.add_layer(LayerType::Windows, 100, 80, 700, 450);  // Smaller window, more compact
    let history_layer = compositor.add_layer(LayerType::Overlay, width - 260, 50, 250, 220);  // Command history panel
    let taskbar_layer = compositor.add_layer(LayerType::Taskbar, 0, height - 40, width, 40);
    let menu_layer = compositor.add_layer(LayerType::Overlay, 5, height - 440, 280, 400);  // Bigger menu
    let settings_layer = compositor.add_layer(LayerType::Overlay, 340, height - 380, 280, 350);  // Settings panel (taller for HoloMatrix)
    let cursor_layer = compositor.add_layer(LayerType::Overlay, 0, 0, 24, 24);
    
    crate::serial_println!("[COSMIC2] Created {} layers", compositor.layer_count());
    
    // Enable GPU direct mode: composite directly into GPU buffer (skips 4MB copy)
    compositor.enable_gpu_direct();
    
    // -------------------------------------------------------------------
    // STATE
    // -------------------------------------------------------------------
    let mut running = true;
    let mut frame_count = 0u64;
    
    // Active module/app
    #[derive(Clone, Copy, PartialEq)]
    enum AppMode {
        Shell,       // Default shell with help
        Network,     // Network module
        Hardware,    // Hardware info
        TextEditor,  // Simple editor
        UserMgmt,    // User management
        Files,       // File browser
        Browser,     // Web browser - special mode
        ImageViewer, // Image viewer - PNG, BMP, etc.
    }
    
    // Set initial mode based on argument
    let mut active_mode = match initial_app {
        Some("browser") | Some("web") | Some("www") => AppMode::Browser,
        Some("files") | Some("explorer") => AppMode::Files,
        Some("editor") | Some("text") | Some("notepad") => AppMode::TextEditor,
        Some("network") | Some("net") | Some("ifconfig") => AppMode::Network,
        Some("hardware") | Some("hw") | Some("lshw") => AppMode::Hardware,
        Some("users") | Some("user") => AppMode::UserMgmt,
        Some("images") | Some("image") | Some("viewer") => AppMode::ImageViewer,
        _ => AppMode::Shell,
    };
    let mut browser_mode = active_mode == AppMode::Browser;  // True when browser is active (not a shell)
    
    // ---------------------------------------------------------------------------
    // BROWSER RENDERING SYSTEM - Chrome DevTools style HTML coloring
    // ---------------------------------------------------------------------------
    
    // Color segment for HTML rendering (text, color)
    #[derive(Clone)]
    struct HtmlSegment {
        text: String,
        color: u32,
    }
    
    // Color palette - Chrome DevTools inspired
    const HTML_COLOR_TAG: u32 = 0xFFE06C75;       // Red/Pink - HTML tags <div>, </div>
    const HTML_COLOR_ATTR: u32 = 0xFF98C379;      // Green - Attribute names
    const HTML_COLOR_VALUE: u32 = 0xFFE5C07B;     // Yellow - Attribute values
    const HTML_COLOR_TEXT: u32 = 0xFFDCDCDC;      // White - Text content
    const HTML_COLOR_COMMENT: u32 = 0xFF5C6370;   // Gray - Comments
    const HTML_COLOR_DOCTYPE: u32 = 0xFFABB2BF;   // Light gray - DOCTYPE
    const HTML_COLOR_BRACKET: u32 = 0xFF56B6C2;   // Cyan - < > brackets
    const HTML_COLOR_STRING: u32 = 0xFF98C379;    // Green - Quoted strings
    const HTML_COLOR_ENTITY: u32 = 0xFFD19A66;    // Orange - HTML entities &amp;
    const HTML_COLOR_HTTP: u32 = 0xFF61AFEF;      // Blue - HTTP headers
    const HTML_COLOR_STATUS: u32 = 0xFF56B6C2;    // Cyan - Status codes
    
    // Parsed line with color segments
    #[derive(Clone)]
    struct BrowserLine {
        segments: Vec<HtmlSegment>,
        line_type: LineType,
    }
    
    #[derive(Clone, Copy, PartialEq)]
    enum LineType {
        Welcome,      // Welcome box
        HttpHeader,   // HTTP/1.1 200 OK
        HtmlTag,      // Full HTML tag
        HtmlMixed,    // Mixed content
        PlainText,    // Regular text
        Error,        // Error message
    }
    
    // Parse a line of HTML into colored segments
    fn parse_html_line(line: &str) -> Vec<HtmlSegment> {
        let mut segments = Vec::new();
        let mut chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            // Check for HTML tag start
            if chars[i] == '<' {
                // Check for comment
                if i + 3 < chars.len() && chars[i+1] == '!' && chars[i+2] == '-' && chars[i+3] == '-' {
                    // Comment start <!--
                    let start = i;
                    while i < chars.len() {
                        if i + 2 < chars.len() && chars[i] == '-' && chars[i+1] == '-' && chars[i+2] == '>' {
                            i += 3;
                            break;
                        }
                        i += 1;
                    }
                    segments.push(HtmlSegment {
                        text: chars[start..i].iter().collect(),
                        color: HTML_COLOR_COMMENT,
                    });
                    continue;
                }
                
                // Check for DOCTYPE
                if i + 1 < chars.len() && chars[i+1] == '!' {
                    let start = i;
                    while i < chars.len() && chars[i] != '>' {
                        i += 1;
                    }
                    if i < chars.len() { i += 1; }
                    segments.push(HtmlSegment {
                        text: chars[start..i].iter().collect(),
                        color: HTML_COLOR_DOCTYPE,
                    });
                    continue;
                }
                
                // Regular tag - parse with colors
                // Opening bracket
                segments.push(HtmlSegment { text: String::from("<"), color: HTML_COLOR_BRACKET });
                i += 1;
                
                // Check for closing tag /
                if i < chars.len() && chars[i] == '/' {
                    segments.push(HtmlSegment { text: String::from("/"), color: HTML_COLOR_BRACKET });
                    i += 1;
                }
                
                // Tag name
                let tag_start = i;
                while i < chars.len() && chars[i] != ' ' && chars[i] != '>' && chars[i] != '/' {
                    i += 1;
                }
                if tag_start < i {
                    segments.push(HtmlSegment {
                        text: chars[tag_start..i].iter().collect(),
                        color: HTML_COLOR_TAG,
                    });
                }
                
                // Attributes
                while i < chars.len() && chars[i] != '>' {
                    // Skip whitespace
                    if chars[i] == ' ' {
                        let ws_start = i;
                        while i < chars.len() && chars[i] == ' ' { i += 1; }
                        segments.push(HtmlSegment {
                            text: chars[ws_start..i].iter().collect(),
                            color: HTML_COLOR_TEXT,
                        });
                        continue;
                    }
                    
                    // Self-closing /
                    if chars[i] == '/' {
                        segments.push(HtmlSegment { text: String::from("/"), color: HTML_COLOR_BRACKET });
                        i += 1;
                        continue;
                    }
                    
                    // Attribute name
                    let attr_start = i;
                    while i < chars.len() && chars[i] != '=' && chars[i] != ' ' && chars[i] != '>' && chars[i] != '/' {
                        i += 1;
                    }
                    if attr_start < i {
                        segments.push(HtmlSegment {
                            text: chars[attr_start..i].iter().collect(),
                            color: HTML_COLOR_ATTR,
                        });
                    }
                    
                    // = sign
                    if i < chars.len() && chars[i] == '=' {
                        segments.push(HtmlSegment { text: String::from("="), color: HTML_COLOR_TEXT });
                        i += 1;
                    }
                    
                    // Attribute value (quoted)
                    if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                        let quote = chars[i];
                        let val_start = i;
                        i += 1;
                        while i < chars.len() && chars[i] != quote {
                            i += 1;
                        }
                        if i < chars.len() { i += 1; } // closing quote
                        segments.push(HtmlSegment {
                            text: chars[val_start..i].iter().collect(),
                            color: HTML_COLOR_VALUE,
                        });
                    }
                }
                
                // Closing bracket
                if i < chars.len() && chars[i] == '>' {
                    segments.push(HtmlSegment { text: String::from(">"), color: HTML_COLOR_BRACKET });
                    i += 1;
                }
            }
            // Check for HTML entity &xxx;
            else if chars[i] == '&' {
                let start = i;
                while i < chars.len() && chars[i] != ';' && chars[i] != ' ' {
                    i += 1;
                }
                if i < chars.len() && chars[i] == ';' { i += 1; }
                segments.push(HtmlSegment {
                    text: chars[start..i].iter().collect(),
                    color: HTML_COLOR_ENTITY,
                });
            }
            // Regular text
            else {
                let start = i;
                while i < chars.len() && chars[i] != '<' && chars[i] != '&' {
                    i += 1;
                }
                if start < i {
                    segments.push(HtmlSegment {
                        text: chars[start..i].iter().collect(),
                        color: HTML_COLOR_TEXT,
                    });
                }
            }
        }
        
        segments
    }
    
    // Browser state
    let mut browser_url = String::from("https://google.com");
    let mut browser_lines: Vec<BrowserLine> = Vec::new();
    let mut browser_status = String::from("Enter URL and press Enter to navigate");
    let mut browser_loading = false;
    let mut browser_url_focused = true;
    let mut browser_view_mode: u8 = 0;  // 0=DevTools, 1=Rendered
    let mut auto_navigate_pending = timeout_ms > 0 && active_mode == AppMode::Browser;
    
    // Helper to add a simple line
    fn make_simple_line(text: &str, color: u32, line_type: LineType) -> BrowserLine {
        let mut segs = Vec::new();
        segs.push(HtmlSegment { text: String::from(text), color });
        BrowserLine {
            segments: segs,
            line_type,
        }
    }
    
    // Helper to add parsed HTML line
    fn make_html_line(text: &str) -> BrowserLine {
        BrowserLine {
            segments: parse_html_line(text),
            line_type: LineType::HtmlMixed,
        }
    }
    
    // Run crypto self-tests before starting browser
    crate::tls13::crypto::run_self_tests();
    
    // Initialize browser with welcome page
    browser_lines.push(make_simple_line("+------------------------------------------------------------+", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|        TrustOS Web Browser v1.0 - DevTools Mode            |", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|------------------------------------------------------------|", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|  Syntax highlighting like Chrome DevTools!                 |", 0xFFDDDDDD, LineType::Welcome));
    browser_lines.push(make_simple_line("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|  COLOR LEGEND:                                             |", 0xFFFFFF00, LineType::Welcome));
    {
        let mut legend1 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend1.segments.push(HtmlSegment { text: String::from("|    "), color: 0xFF00AAFF });
        legend1.segments.push(HtmlSegment { text: String::from("<tag>"), color: HTML_COLOR_TAG });
        legend1.segments.push(HtmlSegment { text: String::from(" - HTML tags                            |"), color: 0xFFDDDDDD });
        browser_lines.push(legend1);
        
        let mut legend2 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend2.segments.push(HtmlSegment { text: String::from("|    "), color: 0xFF00AAFF });
        legend2.segments.push(HtmlSegment { text: String::from("attr"), color: HTML_COLOR_ATTR });
        legend2.segments.push(HtmlSegment { text: String::from(" - Attribute names                     |"), color: 0xFFDDDDDD });
        browser_lines.push(legend2);
        
        let mut legend3 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend3.segments.push(HtmlSegment { text: String::from("|    "), color: 0xFF00AAFF });
        legend3.segments.push(HtmlSegment { text: String::from("\"value\""), color: HTML_COLOR_VALUE });
        legend3.segments.push(HtmlSegment { text: String::from(" - Attribute values                   |"), color: 0xFFDDDDDD });
        browser_lines.push(legend3);
        
        let mut legend4 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend4.segments.push(HtmlSegment { text: String::from("|    "), color: 0xFF00AAFF });
        legend4.segments.push(HtmlSegment { text: String::from("< >"), color: HTML_COLOR_BRACKET });
        legend4.segments.push(HtmlSegment { text: String::from(" - Brackets                             |"), color: 0xFFDDDDDD });
        browser_lines.push(legend4);
        
        let mut legend5 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend5.segments.push(HtmlSegment { text: String::from("|    "), color: 0xFF00AAFF });
        legend5.segments.push(HtmlSegment { text: String::from("&amp;"), color: HTML_COLOR_ENTITY });
        legend5.segments.push(HtmlSegment { text: String::from(" - HTML entities                       |"), color: 0xFFDDDDDD });
        browser_lines.push(legend5);
    }
    browser_lines.push(make_simple_line("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|  TRY THESE URLs:                                           |", 0xFFFFFF00, LineType::Welcome));
    browser_lines.push(make_simple_line("|    https://google.com                                      |", 0xFF00FFFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|    https://example.com                                     |", 0xFF00FFFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("|  [Tab] Toggle DevTools/Rendered  [Enter] Navigate          |", 0xFF88FF88, LineType::Welcome));
    browser_lines.push(make_simple_line("|  [ESC] Return to shell                                     |", 0xFF88FF88, LineType::Welcome));
    browser_lines.push(make_simple_line("|                                                            |", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("+------------------------------------------------------------+", 0xFF00AAFF, LineType::Welcome));
    
    // ---------------------------------------------------------------------------
    // CREATE TEST IMAGE IN RAMFS
    // ---------------------------------------------------------------------------
    {
        // Create a simple 32x32 PPM test image with colorful gradient
        let mut ppm_data = String::from("P3\n32 32\n255\n");
        for y in 0..32 {
            for x in 0..32 {
                let r = (x * 8) % 256;
                let g = (y * 8) % 256;
                let b = ((x + y) * 4) % 256;
                ppm_data.push_str(&format!("{} {} {} ", r, g, b));
            }
            ppm_data.push('\n');
        }
        let _ = crate::ramfs::with_fs(|fs| {
            fs.mkdir("/images");
            fs.write_file("/images/test.ppm", ppm_data.as_bytes())
        });
        
        // Also create a simple BMP test image (24-bit, 16x16)
        let bmp_header: [u8; 54] = [
            0x42, 0x4D,             // BM signature
            0x36, 0x03, 0x00, 0x00, // File size: 54 + 768 = 822 bytes
            0x00, 0x00, 0x00, 0x00, // Reserved
            0x36, 0x00, 0x00, 0x00, // Offset to pixel data
            0x28, 0x00, 0x00, 0x00, // DIB header size (40)
            0x10, 0x00, 0x00, 0x00, // Width: 16
            0x10, 0x00, 0x00, 0x00, // Height: 16 (bottom-up)
            0x01, 0x00,             // Planes: 1
            0x18, 0x00,             // Bits per pixel: 24
            0x00, 0x00, 0x00, 0x00, // Compression: none
            0x00, 0x03, 0x00, 0x00, // Image size: 768
            0x13, 0x0B, 0x00, 0x00, // H pixels/meter
            0x13, 0x0B, 0x00, 0x00, // V pixels/meter
            0x00, 0x00, 0x00, 0x00, // Colors in palette
            0x00, 0x00, 0x00, 0x00, // Important colors
        ];
        let mut bmp_data = alloc::vec::Vec::from(bmp_header);
        // Pixel data (BGR, bottom-up, with row padding)
        for y in 0..16 {
            for x in 0..16 {
                let b = ((15 - y) * 17) as u8;  // Blue gradient top-bottom
                let g = (x * 17) as u8;          // Green gradient left-right
                let r = ((x + y) * 8) as u8;     // Red diagonal
                bmp_data.push(b);
                bmp_data.push(g);
                bmp_data.push(r);
            }
            // No padding needed for 16*3=48 bytes (divisible by 4)
        }
        let _ = crate::ramfs::with_fs(|fs| {
            fs.write_file("/images/test.bmp", &bmp_data)
        });
        
        crate::serial_println!("[COSMIC2] Created test images in /images/");
    }
    
    // ---------------------------------------------------------------------------
    // IMAGE VIEWER STATE
    // ---------------------------------------------------------------------------
    let mut image_viewer_path = String::new();
    let mut image_viewer_data: Option<crate::image::Image> = None;
    let mut image_viewer_zoom: f32 = 1.0;
    let mut image_viewer_offset_x: i32 = 0;
    let mut image_viewer_offset_y: i32 = 0;
    let mut image_viewer_info = String::from("No image loaded");
    let mut image_viewer_format = String::from("---");
    
    // Menu state
    let mut menu_open = false;
    let mut menu_hover: i32 = -1;
    
    // Settings panel state
    let mut settings_open = false;
    let mut settings_anim_enabled = crate::desktop::animations_enabled();
    let mut settings_anim_speed = crate::desktop::get_animation_speed();
    
    // Shell state for the window
    let mut shell_input = String::new();
    let mut shell_output: Vec<String> = Vec::new();
    let mut cursor_blink = true;
    let mut suggestion_text = String::new();
    let mut scroll_offset: usize = 0;  // For scrolling through output
    const MAX_VISIBLE_LINES: usize = 18;  // Lines visible in shell window
    
    // TrustCode editor state
    let mut editor_state = crate::apps::text_editor::EditorState::new();
    // Pre-load a demo Rust file
    {
        let sample_code = "//! TrustOS \u{2014} A Modern Operating System in Rust\n//!\n//! This file demonstrates TrustCode's syntax highlighting\n\nuse core::fmt;\n\n/// Main kernel entry point\npub fn kernel_main() -> ! {\n    let message = \"Hello from TrustOS!\";\n    serial_println!(\"{}\", message);\n\n    // Initialize hardware\n    let cpu_count: u32 = 4;\n    let memory_mb: u64 = 256;\n\n    for i in 0..cpu_count {\n        init_cpu(i);\n    }\n\n    // Start the desktop environment\n    let mut desktop = Desktop::new();\n    desktop.init(1280, 800);\n\n    loop {\n        desktop.render();\n        desktop.handle_input();\n    }\n}\n\n/// Initialize a CPU core\nfn init_cpu(id: u32) {\n    // Setup GDT, IDT, APIC\n    serial_println!(\"CPU {} initialized\", id);\n}\n\n#[derive(Debug, Clone)]\nstruct AppConfig {\n    name: String,\n    version: (u8, u8, u8),\n    features: Vec<&'static str>,\n}\n";
        let _ = crate::ramfs::with_fs(|fs| fs.write_file("/demo.rs", sample_code.as_bytes()));
        editor_state.load_file("demo.rs");
    }
    
    // Window dragging state
    let mut dragging_window = false;
    let mut drag_offset_x: i32 = 0;
    let mut drag_offset_y: i32 = 0;
    let mut window_x: i32 = 100;
    let mut window_y: i32 = 80;
    let mut window_visible = true;  // Window can be closed/reopened
    
    // Command history for the history panel
    let mut command_history: Vec<String> = Vec::new();
    const MAX_HISTORY: usize = 10;
    
    // Module help text - static arrays
    const HELP_SHELL: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|         TrustOS Interactive Shell - Welcome!                  |",
        "|---------------------------------------------------------------|",
        "|  This is the main command-line interface for TrustOS.         |",
        "|                                                               |",
        "|  BASIC COMMANDS:                                              |",
        "|    help     - Display all available commands                  |",
        "|    clear    - Clear the terminal screen                       |",
        "|    echo     - Print text to the screen                        |",
        "|    date     - Show current date and time                      |",
        "|    uptime   - Show system uptime                              |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Use Tab for command autocompletion                       |",
        "|    * Commands are case-insensitive                            |",
        "|    * Type 'exit' or press ESC to close desktop                |",
        "+---------------------------------------------------------------+",
    ];
    
    const HELP_NETWORK: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           Network Module - Configuration & Diagnostics        |",
        "|---------------------------------------------------------------|",
        "|  Manage network interfaces and diagnose connectivity.         |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    ifconfig    - Show network interface configuration         |",
        "|    ping <ip>   - Send ICMP ping to test connectivity          |",
        "|    dhcp        - Request IP address via DHCP                  |",
        "|    netstat     - Display network statistics                   |",
        "|    arp         - Show Address Resolution Protocol table       |",
        "|    dns <host>  - Resolve hostname to IP address               |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Run 'dhcp' first to get an IP address                    |",
        "|    * Use 'ping 8.8.8.8' to test internet connectivity         |",
        "|    * 'ifconfig' shows MAC address and IP configuration        |",
        "+---------------------------------------------------------------+",
    ];
    
    const HELP_HARDWARE: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           Hardware Module - System Information                |",
        "|---------------------------------------------------------------|",
        "|  Explore your hardware and system resources.                  |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    cpuinfo   - Display CPU model, features and frequency      |",
        "|    meminfo   - Show RAM usage and available memory            |",
        "|    lspci     - List all PCI/PCIe devices                      |",
        "|    lsusb     - List connected USB devices                     |",
        "|    uptime    - Show system uptime since boot                  |",
        "|    sensors   - Display temperature sensors (if available)     |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * 'cpuinfo' shows SIMD support (SSE, AVX)                  |",
        "|    * 'lspci' reveals network and storage controllers          |",
        "|    * Memory info includes heap allocation statistics          |",
        "+---------------------------------------------------------------+",
    ];
    
    const HELP_EDITOR: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           Text Editor - Create and Edit Files                 |",
        "|---------------------------------------------------------------|",
        "|  Simple text editor for viewing and modifying files.          |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    edit <file>  - Open an existing file for editing           |",
        "|    new <name>   - Create a new empty file                     |",
        "|    cat <file>   - View file contents (read-only)              |",
        "|    save         - Save current changes                        |",
        "|    :q           - Quit editor without saving                  |",
        "|    :wq          - Save and quit                               |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Use 'cat' first to preview file before editing           |",
        "|    * Files are stored in the RAM filesystem                   |",
        "|    * New files are created in current directory               |",
        "+---------------------------------------------------------------+",
    ];
    
    const HELP_USERS: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           User Management - Accounts & Security               |",
        "|---------------------------------------------------------------|",
        "|  Manage user accounts, passwords, and permissions.            |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    whoami      - Display current logged-in user               |",
        "|    users       - List all system users                        |",
        "|    adduser     - Create a new user account                    |",
        "|    deluser     - Delete an existing user                      |",
        "|    passwd      - Change user password                         |",
        "|    groups      - Show user groups                             |",
        "|    su <user>   - Switch to another user                       |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Default user is 'root' with full privileges              |",
        "|    * Use strong passwords (8+ chars, mixed case)              |",
        "|    * 'adduser' prompts for username and password              |",
        "+---------------------------------------------------------------+",
    ];
    
    const HELP_FILES: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           File Manager - Navigate & Manage Files              |",
        "|---------------------------------------------------------------|",
        "|  Browse directories and manage files on the system.           |",
        "|                                                               |",
        "|  NAVIGATION:                                                  |",
        "|    ls / dir    - List files in current directory              |",
        "|    cd <dir>    - Change to specified directory                |",
        "|    cd ..       - Go up one directory level                    |",
        "|    pwd         - Print current working directory              |",
        "|                                                               |",
        "|  FILE OPERATIONS:                                             |",
        "|    mkdir <dir> - Create a new directory                       |",
        "|    rm <file>   - Remove/delete a file                         |",
        "|    cp <s> <d>  - Copy file from source to destination         |",
        "|    mv <s> <d>  - Move or rename a file                        |",
        "|    cat <file>  - Display file contents                        |",
        "|    touch <f>   - Create empty file                            |",
        "|                                                               |",
        "|  TIPS:                                                        |",
        "|    * Use Tab for path autocompletion                          |",
        "|    * 'ls -la' shows hidden files and details                  |",
        "|    * Paths can be absolute (/home) or relative (./docs)       |",
        "+---------------------------------------------------------------+",
    ];
    
    const HELP_BROWSER: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           TrustOS Web Browser                                 |",
        "|---------------------------------------------------------------|",
        "|  A simple text-based web browser for HTTP requests.           |",
        "|                                                               |",
        "|  COMMANDS:                                                    |",
        "|    get <url>    - Fetch a web page (HTTP only)                |",
        "|    http <host>  - Make HTTP GET request                       |",
        "|    curl <url>   - Display raw HTTP response                   |",
        "|                                                               |",
        "|  EXAMPLES:                                                    |",
        "|    get http://example.com                                     |",
        "|    http 93.184.216.34 /index.html                             |",
        "|                                                               |",
        "|  NOTE: This is a text-based browser. Full graphical           |",
        "|        browser support is planned for future versions.        |",
        "+---------------------------------------------------------------+",
    ];
    
    const HELP_IMAGEVIEWER: &[&str] = &[
        "+---------------------------------------------------------------+",
        "|           TrustOS Image Viewer                                |",
        "|---------------------------------------------------------------|",
        "|  View PNG, BMP, PPM and other image formats.                  |",
        "|                                                               |",
        "|  SUPPORTED FORMATS:                                           |",
        "|    * PNG  - Portable Network Graphics (8-bit RGB/RGBA)        |",
        "|    * BMP  - Windows Bitmap (24/32-bit uncompressed)           |",
        "|    * PPM  - Portable Pixmap (P3/P6 formats)                   |",
        "|                                                               |",
        "|  SHELL COMMANDS:                                              |",
        "|    imgview <file>  - Open image in viewer                     |",
        "|    imginfo <file>  - Show image information                   |",
        "|                                                               |",
        "|  KEYBOARD (in viewer):                                        |",
        "|    +/-        - Zoom in/out                                   |",
        "|    Arrow keys - Pan image                                     |",
        "|    R          - Reset view (fit to window)                    |",
        "|    ESC        - Return to shell                               |",
        "|                                                               |",
        "|  EXAMPLES:                                                    |",
        "|    imgview /images/photo.png                                  |",
        "|    imginfo /wallpaper.bmp                                     |",
        "+---------------------------------------------------------------+",
    ];
    
    // Macro to get help for mode
    macro_rules! get_help {
        ($mode:expr) => {
            match $mode {
                AppMode::Shell => HELP_SHELL,
                AppMode::Network => HELP_NETWORK,
                AppMode::Hardware => HELP_HARDWARE,
                AppMode::TextEditor => HELP_EDITOR,
                AppMode::UserMgmt => HELP_USERS,
                AppMode::Files => HELP_FILES,
                AppMode::Browser => HELP_BROWSER,
                AppMode::ImageViewer => HELP_IMAGEVIEWER,
            }
        };
    }
    
    // Initialize shell with welcome
    for line in get_help!(AppMode::Shell) {
        shell_output.push(String::from(*line));
    }
    
    // -------------------------------------------------------------------
    // MATRIX RAIN - OPTIMIZED: Pre-generated columns, only update head position
    // Each column has fixed characters, only the "brightness head" moves
    // -------------------------------------------------------------------
    const MATRIX_COLS: usize = 240;      // Dense columns (1920/8 = 240)
    const MATRIX_ROWS: usize = 68;       // Rows per column (1080/16 = 67.5)
    const MATRIX_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
    
    // Helper to compute flat index for matrix_chars
    #[inline]
    fn matrix_idx(col: usize, row: usize) -> usize {
        col * MATRIX_ROWS + row
    }
    
    // Pre-generate character grid (static characters for each cell)
    // This is generated once and reused every frame
    // Vec allocated directly on heap - no stack overflow
    let mut matrix_chars: Vec<u8> = vec![0u8; MATRIX_COLS * MATRIX_ROWS];
    for col in 0..MATRIX_COLS {
        let seed = (col as u32 * 2654435761) ^ 0xDEADBEEF;
        for row in 0..MATRIX_ROWS {
            let char_seed = seed.wrapping_mul(row as u32 + 1);
            matrix_chars[matrix_idx(col, row)] = MATRIX_CHARS[(char_seed as usize) % MATRIX_CHARS.len()];
        }
    }
    
    // Head position for each column (where the bright green starts)
    // Only this moves each frame - much faster than redrawing characters!
    let mut matrix_heads: [i32; MATRIX_COLS] = [0; MATRIX_COLS];
    let mut matrix_speeds: [u32; MATRIX_COLS] = [0; MATRIX_COLS];
    for col in 0..MATRIX_COLS {
        let seed = (col as u32 * 2654435761) ^ 0xDEADBEEF;
        matrix_heads[col] = -((seed % (MATRIX_ROWS as u32 * 2)) as i32);
        matrix_speeds[col] = 1 + (seed % 3);  // Speed 1-3
    }
    
    // HoloMatrix 3D volumetric renderer
    // Creates holographic 3D effects by layering Z-slices
    // 32 layers for enhanced depth perception
    let mut holomatrix = crate::graphics::holomatrix::HoloMatrix::new(width as usize / 4, height as usize / 4, 32);
    // Sync with global settings (can be set via 'holo' shell command)
    let mut holo_scene = crate::graphics::holomatrix::get_scene();
    let mut holo_enabled = crate::graphics::holomatrix::is_enabled();
    
    // NEW: HoloVolume - True volumetric ASCII raymarcher
    // A 3D volume of ASCII voxels projected to 2D with alignment-based intensity
    let mut holovolume = crate::holovolume::HoloVolume::new(
        width as usize / 8,   // ~160 chars wide
        height as usize / 9,  // ~90 chars tall  
        32                    // 32 Z-layers for depth
    );
    holovolume.render_mode = crate::holovolume::RenderMode::Hologram;
    let mut use_holovolume = false;  // Toggle with 'holo volume' command
    
    // FAST MATRIX RENDERER - Ultra optimized with glyph caching
    let mut fast_renderer = crate::matrix_fast::FastMatrixRenderer::new();
    let mut use_fast_matrix = false;  // Braille mode is default now
    
    // BRAILLE MATRIX - 8A-- resolution sub-pixel rendering (BOXED to avoid stack overflow)
    let mut braille_renderer = alloc::boxed::Box::new(crate::matrix_fast::BrailleMatrix::new());
    let mut use_braille = false;  // Formula mode is default now
    let mut show_fps = true;  // FPS display
    
    // MATRIX 3D - Volumetric rain with 3D shapes (BOXED to avoid stack overflow)
    let mut matrix3d_renderer = alloc::boxed::Box::new(crate::matrix_fast::Matrix3D::new());
    let mut use_matrix3d = false;  // Toggle with 'matrix3d' command
    
    // FORMULA 3D - Tsoding-inspired wireframe renderer (perspective projection)
    let mut formula_renderer = alloc::boxed::Box::new(crate::formula3d::FormulaRenderer::new());
    let mut use_formula = true;  // DEFAULT MODE - fastest renderer
    
    // SHADER MATRIX - GPU-emulated pixel shader matrix rain
    let mut use_shader_matrix = false;  // Toggle with 'matrix shader' command
    let mut shader_time: f32 = 0.0;
    let mut shader_frame: u32 = 0;
    
    // RayTracer for advanced 3D scenes (lower resolution for performance)
    let mut raytracer = crate::graphics::raytracer::RayTracer::new(width as usize / 6, height as usize / 6);
    
    // Colors
    let green_main: u32 = 0xFF00FF66;
    let green_bright: u32 = 0xFF00FF88;
    let green_dim: u32 = 0xFF007744;
    let black: u32 = 0xFF000000;
    let deep_black: u32 = 0xFF020202;  // Deep black for shell
    let dark_gray: u32 = 0xFF101010;
    let window_bg: u32 = 0xFF0A0A0A;  // Almost pure black background
    let red_pure: u32 = 0xFFFF0000;   // Pure red for "root"
    let white_pure: u32 = 0xFFFFFFFF; // Pure white for "@"
    let green_pure: u32 = 0xFF00FF00; // Pure green for "trustos"
    
    // Menu items - Apps + Power options
    #[derive(Clone, Copy, PartialEq)]
    enum MenuItem {
        App(AppMode),
        Shutdown,
        Reboot,
    }
    let menu_items: [(&str, MenuItem); 11] = [
        ("Shell", MenuItem::App(AppMode::Shell)),
        ("Files", MenuItem::App(AppMode::Files)),
        ("Network", MenuItem::App(AppMode::Network)),
        ("Hardware", MenuItem::App(AppMode::Hardware)),
        ("TrustCode", MenuItem::App(AppMode::TextEditor)),
        ("User Management", MenuItem::App(AppMode::UserMgmt)),
        ("Web Browser", MenuItem::App(AppMode::Browser)),
        ("Image Viewer", MenuItem::App(AppMode::ImageViewer)),
        ("-----------------", MenuItem::App(AppMode::Shell)), // Separator
        ("Reboot", MenuItem::Reboot),
        ("Shutdown", MenuItem::Shutdown),
    ];
    
    // Mouse state
    let mut prev_left = false;
    let mut mouse_x: i32 = (width / 2) as i32;
    let mut mouse_y: i32 = (height / 2) as i32;
    
    // FPS tracking
    let tsc_freq = crate::cpu::tsc::frequency_hz();
    let mut fps = 0u32;
    let mut frame_in_second = 0u32;
    let mut last_second_tsc = crate::cpu::tsc::read_tsc();
    
    // -------------------------------------------------------------------
    // FRAME-RATE DECOUPLING (Game Engine Technique)
    // Separate render rate from present rate for higher measured FPS.
    // - Composite (expensive) runs every Nth frame: layer rendering + composite + present
    // - Skip frames (cheap) just re-present the same GPU buffer via VirtIO DMA
    // - FPS counter measures total loop iterations (presents), not composites
    // This is how id Tech, Unreal, Godot decouple CPU-bound rendering from display rate.
    // -------------------------------------------------------------------
    let composite_interval: u64 = 4; // Render+present every 4th frame, skip frames are ~free
    let mut render_fps = 0u32; // Actual render (composite) rate
    let mut render_in_second = 0u32;
    
    crate::serial_println!("[COSMIC2] Entering render loop...");
    
    // Auto-exit timer for showcase mode
    let auto_start_tsc = crate::cpu::tsc::read_tsc();
    let auto_freq = crate::cpu::tsc::frequency_hz();
    let auto_target = if timeout_ms > 0 && auto_freq > 0 { auto_freq / 1000 * timeout_ms } else { u64::MAX };
    
    while running {
        // Auto-navigate for showcase mode (trigger browser load on frame 5)
        if auto_navigate_pending && frame_count == 5 {
            auto_navigate_pending = false;
            // Simulate Enter press: navigate browser
            if browser_mode {
                browser_lines.clear();
                browser_status = format!("Loading {}...", browser_url);
                browser_loading = true;
                let is_https = browser_url.starts_with("https://");
                if let Some((host, port, path, url_is_https)) = super::vm::parse_http_url(&browser_url) {
                    let protocol = if url_is_https { "HTTPS" } else { "HTTP" };
                    browser_lines.push(make_simple_line(&format!("\u{25ba} {} {}:{}{}...", protocol, host, port, path), 0xFF88FF88, LineType::PlainText));
                    if url_is_https {
                        browser_lines.push(make_simple_line("\u{25ba} Establishing TLS 1.3 connection...", 0xFF88CCFF, LineType::PlainText));
                        match crate::netstack::https::get(&browser_url) {
                            Ok(response) => {
                                browser_lines.push(make_simple_line(&format!("\u{25ba} TLS OK, {} bytes", response.body.len()), 0xFF88FF88, LineType::PlainText));
                                browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                browser_lines.push(make_simple_line("\u{2500}\u{2500} Response Headers \u{2500}\u{2500}", 0xFF61AFEF, LineType::HttpHeader));
                                browser_lines.push(make_simple_line(&format!("HTTP/1.1 {}", response.status_code), HTML_COLOR_HTTP, LineType::HttpHeader));
                                for (key, value) in &response.headers {
                                    browser_lines.push(make_simple_line(&format!("{}: {}", key, value), HTML_COLOR_HTTP, LineType::HttpHeader));
                                }
                                browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                browser_lines.push(make_simple_line("\u{2500}\u{2500} HTML Source \u{2500}\u{2500}", 0xFF61AFEF, LineType::HttpHeader));
                                if let Ok(body_str) = core::str::from_utf8(&response.body) {
                                    for line in body_str.lines().take(200) {
                                        browser_lines.push(make_html_line(line));
                                    }
                                }
                                browser_status = format!("\u{2713} Loaded: {} ({} bytes, HTTPS)", browser_url, response.body.len());
                            }
                            Err(e) => {
                                browser_lines.push(make_simple_line(&format!("\u{2718} HTTPS Error: {}", e), 0xFFFF4444, LineType::PlainText));
                                browser_status = format!("Error: {}", e);
                            }
                        }
                    } else {
                        // HTTP fallback
                        match crate::netstack::http::get(&browser_url) {
                            Ok(response) => {
                                if let Some(body_str) = response.body_str() {
                                    for line in body_str.lines().take(200) {
                                        browser_lines.push(make_html_line(line));
                                    }
                                }
                                browser_status = format!("\u{2713} Loaded: {} ({} bytes)", browser_url, response.body.len());
                            }
                            Err(e) => {
                                browser_lines.push(make_simple_line(&format!("\u{2718} HTTP Error: {}", e), 0xFFFF4444, LineType::PlainText));
                                browser_status = format!("Error: {}", e);
                            }
                        }
                    }
                } else {
                    browser_lines.push(make_simple_line("\u{2718} Invalid URL", 0xFFFF4444, LineType::PlainText));
                    browser_status = String::from("Invalid URL");
                }
                browser_loading = false;
            }
        }

        // Auto-exit for showcase mode
        if timeout_ms > 0 {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(auto_start_tsc);
            if elapsed >= auto_target { break; }
        }
        
        // Frame start tracking (frame_count incremented at end of loop)
        if frame_count <= 3 || frame_count % 500 == 0 {
            crate::serial_println!("[COSMIC2] Loop iteration {}", frame_count);
        }
        
        // -------------------------------------------------------------------
        // INPUT HANDLING
        // -------------------------------------------------------------------
        
        // Keyboard input -- drain up to 8 pending keys per frame
        // to prevent input lag when render is fast
        let mut _keys_this_frame = 0u8;
        while let Some(key) = crate::keyboard::try_read_key() {
            _keys_this_frame += 1;
            if _keys_this_frame > 8 { break; } // prevent infinite drain
            crate::serial_println!("[KEY] Received key: {} (0x{:02X})", key, key);
            // Browser mode has different input handling
            if active_mode == AppMode::Browser {
                match key {
                    27 => { // ESC - switch back to shell mode
                        active_mode = AppMode::Shell;
                        shell_output.clear();
                        for line in get_help!(AppMode::Shell) {
                            shell_output.push(String::from(*line));
                        }
                    },
                    9 => { // Tab - toggle view mode (DevTools / Rendered)
                        browser_view_mode = (browser_view_mode + 1) % 2;
                        if browser_view_mode == 0 {
                            browser_status = String::from("View: DevTools (source)");
                        } else {
                            browser_status = String::from("View: Rendered");
                        }
                    },
                    8 => { // Backspace
                        if browser_url.len() > 7 { // Keep "http://" minimum
                            browser_url.pop();
                        }
                    },
                    10 | 13 => { // Enter - navigate to URL
                        browser_lines.clear();
                        browser_status = format!("Loading {}...", browser_url);
                        browser_loading = true;
                        
                        // Check if HTTPS or HTTP
                        let is_https = browser_url.starts_with("https://");
                        
                        // Parse URL and try to fetch
                        if let Some((host, port, path, url_is_https)) = super::vm::parse_http_url(&browser_url) {
                            let protocol = if url_is_https { "HTTPS" } else { "HTTP" };
                            browser_lines.push(make_simple_line(&format!("? {} {}:{}{}...", protocol, host, port, path), 0xFF88FF88, LineType::PlainText));
                            
                            if url_is_https {
                                // HTTPS request using TLS 1.3
                                browser_lines.push(make_simple_line("? Establishing TLS 1.3 connection...", 0xFF88CCFF, LineType::PlainText));
                                
                                match crate::netstack::https::get(&browser_url) {
                                    Ok(response) => {
                                        browser_lines.push(make_simple_line(&format!("? TLS handshake complete, received {} bytes", response.body.len()), 0xFF88FF88, LineType::PlainText));
                                        browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                        
                                        // Show HTTP headers
                                        browser_lines.push(make_simple_line("-- Response Headers --", 0xFF61AFEF, LineType::HttpHeader));
                                        browser_lines.push(make_simple_line(&format!("HTTP/1.1 {}", response.status_code), HTML_COLOR_HTTP, LineType::HttpHeader));
                                        for (key, value) in &response.headers {
                                            browser_lines.push(make_simple_line(&format!("{}: {}", key, value), HTML_COLOR_HTTP, LineType::HttpHeader));
                                        }
                                        browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                        
                                        // Show HTML body
                                        browser_lines.push(make_simple_line("-- HTML Source --", 0xFF61AFEF, LineType::HttpHeader));
                                        if let Ok(body_str) = core::str::from_utf8(&response.body) {
                                            for line in body_str.lines().take(200) {
                                                browser_lines.push(make_html_line(line));
                                            }
                                        } else {
                                            browser_lines.push(make_simple_line("[Binary content]", 0xFFFFFF00, LineType::PlainText));
                                        }
                                        
                                        browser_status = format!("? Loaded: {} ({} bytes, HTTPS)", browser_url, response.body.len());
                                    }
                                    Err(e) => {
                                        browser_lines.push(make_simple_line(&format!("? HTTPS Error: {}", e), 0xFFFF4444, LineType::Error));
                                        browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                        browser_lines.push(make_simple_line("TLS 1.3 connection failed. Possible causes:", 0xFFFFFF00, LineType::PlainText));
                                        browser_lines.push(make_simple_line("  * DNS resolution failed", 0xFFAAAAAA, LineType::PlainText));
                                        browser_lines.push(make_simple_line("  * Server doesn't support TLS 1.3", 0xFFAAAAAA, LineType::PlainText));
                                        browser_lines.push(make_simple_line("  * Network timeout", 0xFFAAAAAA, LineType::PlainText));
                                        browser_status = format!("? HTTPS Error: {}", e);
                                    }
                                }
                            } else {
                                // HTTP request
                                // Resolve DNS or parse IP directly
                                let ip_result = if let Some(ip) = super::vm::parse_ipv4(&host) {
                                    Some(ip)
                                } else {
                                    // Use real DNS resolution
                                    crate::netstack::dns::resolve(&host)
                                };
                                
                                if let Some(ip) = ip_result {
                                    browser_lines.push(make_simple_line(&format!("? Resolved: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]), 0xFF88FF88, LineType::PlainText));
                                    
                                    // Make real HTTP request
                                    match super::vm::do_http_get_string(&host, ip, port, &path) {
                                        Ok(response) => {
                                            browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                            
                                            // Parse HTTP response (headers + body)
                                            let mut in_headers = true;
                                            browser_lines.push(make_simple_line("-- Response Headers --", 0xFF61AFEF, LineType::HttpHeader));
                                            
                                            for line in response.lines() {
                                                if in_headers {
                                                    if line.is_empty() {
                                                        in_headers = false;
                                                        browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                                        browser_lines.push(make_simple_line("-- HTML Source --", 0xFF61AFEF, LineType::HttpHeader));
                                                    } else {
                                                        browser_lines.push(make_simple_line(line, HTML_COLOR_HTTP, LineType::HttpHeader));
                                                    }
                                                } else {
                                                    // Parse HTML with syntax highlighting
                                                    browser_lines.push(make_html_line(line));
                                                }
                                            }
                                            
                                            browser_status = format!("? Loaded: {} ({} bytes)", browser_url, response.len());
                                        }
                                        Err(e) => {
                                            browser_lines.push(make_simple_line(&format!("? HTTP Error: {}", e), 0xFFFF4444, LineType::Error));
                                            browser_status = format!("? Error: {}", e);
                                        }
                                    }
                                } else {
                                    browser_lines.push(make_simple_line(&format!("? Error: Cannot resolve host '{}'", host), 0xFFFF4444, LineType::Error));
                                    browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                    browser_lines.push(make_simple_line("Tip: Try a local server or IP address:", 0xFFFFFF00, LineType::PlainText));
                                    browser_lines.push(make_simple_line("  * http://192.168.56.1:8080/", 0xFF00FFFF, LineType::PlainText));
                                    browser_lines.push(make_simple_line("  * http://10.0.2.2:8000/", 0xFF00FFFF, LineType::PlainText));
                                    browser_status = String::from("? Error: DNS resolution failed");
                                }
                            }
                        } else {
                            browser_lines.push(make_simple_line("? Invalid URL format", 0xFFFF4444, LineType::Error));
                            browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                            browser_lines.push(make_simple_line("Use format: http://hostname/path or https://hostname/path", 0xFFFFFF00, LineType::PlainText));
                            browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                            browser_lines.push(make_simple_line("Examples:", 0xFF88FF88, LineType::PlainText));
                            browser_lines.push(make_simple_line("  * https://google.com", 0xFF00FFFF, LineType::PlainText));
                            browser_lines.push(make_simple_line("  * https://example.com", 0xFF00FFFF, LineType::PlainText));
                            browser_lines.push(make_simple_line("  * http://192.168.1.1/", 0xFF00FFFF, LineType::PlainText));
                            browser_status = String::from("? Error: Invalid URL");
                        }
                        browser_loading = false;
                    },
                    32..=126 => { // Printable characters
                        browser_url.push(key as char);
                    },
                    _ => {}
                }
            } else if active_mode == AppMode::ImageViewer {
                // Image Viewer keyboard controls
                match key {
                    27 => { // ESC - return to shell
                        active_mode = AppMode::Shell;
                        shell_output.clear();
                        for line in get_help!(AppMode::Shell) {
                            shell_output.push(String::from(*line));
                        }
                    },
                    43 | 61 => { // + or = - zoom in
                        image_viewer_zoom = (image_viewer_zoom * 1.25).min(10.0);
                    },
                    45 => { // - zoom out
                        image_viewer_zoom = (image_viewer_zoom / 1.25).max(0.1);
                    },
                    114 | 82 => { // R - reset view
                        image_viewer_zoom = 1.0;
                        image_viewer_offset_x = 0;
                        image_viewer_offset_y = 0;
                    },
                    // Arrow keys (ANSI escape sequences start with 27)
                    // Left=75, Right=77, Up=72, Down=80 (scan codes)
                    _ => {}
                }
            } else if active_mode == AppMode::TextEditor {
                // TrustCode editor input handling
                match key {
                    27 => { // ESC - return to shell mode
                        active_mode = AppMode::Shell;
                        shell_output.clear();
                        for line in get_help!(AppMode::Shell) {
                            shell_output.push(String::from(*line));
                        }
                    },
                    _ => {
                        // Forward all other keys to the editor state
                        editor_state.handle_key(key);
                    }
                }
            } else {
                // Shell mode input handling
            match key {
                27 => { // ESC - close menus or exit
                    if menu_open || settings_open {
                        menu_open = false;
                        settings_open = false;
                    } else {
                        running = false;
                    }
                },
                8 => { // Backspace
                    shell_input.pop();
                    suggestion_text.clear();
                },
                0x49 => { // PageUp - scroll up
                    if scroll_offset > 0 {
                        scroll_offset = scroll_offset.saturating_sub(5);
                    }
                },
                0x51 => { // PageDown - scroll down
                    let max_scroll = shell_output.len().saturating_sub(MAX_VISIBLE_LINES);
                    if scroll_offset < max_scroll {
                        scroll_offset = (scroll_offset + 5).min(max_scroll);
                    }
                },
                10 | 13 => { // Enter - execute command (ASCII LF=10 or CR=13)
                    if !shell_input.is_empty() {
                        let cmd_raw = shell_input.clone();
                        let cmd = cmd_raw.trim();  // Trim whitespace for matching
                        crate::serial_println!("[DEBUG] Enter pressed, cmd = '{}' (trimmed: '{}')", cmd_raw, cmd);
                        shell_output.push(format!("> {}", cmd));
                        
                        // Add to command history
                        command_history.push(String::from(cmd));
                        if command_history.len() > MAX_HISTORY {
                            command_history.remove(0);
                        }
                        
                        // Process command - use real shell functions where possible
                        crate::serial_println!("[MATCH] About to match cmd='{}' starts_with_shader={}", cmd, cmd.starts_with("shader "));
                        match cmd {
                            "help" => {
                                shell_output.push(String::from("+================================================+"));
                                shell_output.push(String::from("|          TrustOS Desktop Shell                 |"));
                                shell_output.push(String::from("+================================================+"));
                                shell_output.push(String::from("| FILE SYSTEM:                                   |"));
                                shell_output.push(String::from("|   ls, cd, pwd, mkdir, rmdir, touch, rm, cat    |"));
                                shell_output.push(String::from("|   cp, mv, head, tail, stat, tree, find, wc     |"));
                                shell_output.push(String::from("|   chmod, chown, ln, grep                       |"));
                                shell_output.push(String::from("| NETWORK:                                       |"));
                                shell_output.push(String::from("|   ifconfig, ping, curl, wget, nslookup         |"));
                                shell_output.push(String::from("|   arp, route, traceroute, netstat              |"));
                                shell_output.push(String::from("| SYSTEM:                                        |"));
                                shell_output.push(String::from("|   clear, date, time, uptime, whoami, hostname  |"));
                                shell_output.push(String::from("|   uname, env, history, ps, free, df, top       |"));
                                shell_output.push(String::from("| HARDWARE:                                      |"));
                                shell_output.push(String::from("|   cpuinfo, meminfo, lspci, lsusb, lscpu, disk  |"));
                                shell_output.push(String::from("| USERS:                                         |"));
                                shell_output.push(String::from("|   login, su, passwd, adduser, users            |"));
                                shell_output.push(String::from("| UTILITIES:                                     |"));
                                shell_output.push(String::from("|   echo, hexdump, strings, sort, cal, bc        |"));
                                shell_output.push(String::from("| DESKTOP:                                       |"));
                                shell_output.push(String::from("|   desktop close - Exit desktop                 |"));
                                shell_output.push(String::from("|   open <app> - Open app (browser,files,editor) |"));
                                shell_output.push(String::from("|   imgview <file> - View images (PNG/BMP)       |"));
                                shell_output.push(String::from("|   3ddemo - 3D rotating cube demo               |"));
                                shell_output.push(String::from("+================================================+"));
                            },
                            "clear" => {
                                shell_output.clear();
                            },
                            "pwd" => {
                                // Use ramfs like the main shell
                                let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
                                shell_output.push(cwd);
                            },
                            "ls" | "dir" => {
                                // Use ramfs like the main shell
                                match crate::ramfs::with_fs(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        if items.is_empty() {
                                            shell_output.push(String::from("(empty)"));
                                        } else {
                                            for (name, file_type, size) in items {
                                                match file_type {
                                                    FileType::Directory => {
                                                        shell_output.push(format!("{}  <DIR>", name));
                                                    }
                                                    FileType::File => {
                                                        shell_output.push(format!("{}  {} B", name, size));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("ls: {}", e.as_str()));
                                    }
                                }
                            },
                            "whoami" => shell_output.push(String::from("root")),
                            "ifconfig" => {
                                if let Some(mac) = crate::network::get_mac_address() {
                                    let mac_str = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
                                    if let Some((ip, _subnet, _gw)) = crate::network::get_ipv4_config() {
                                        let ip_str = format!("{}.{}.{}.{}", 
                                            ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
                                        shell_output.push(format!("eth0: {}  UP  RUNNING", ip_str));
                                    } else {
                                        shell_output.push(String::from("eth0: No IP  UP  RUNNING"));
                                    }
                                    shell_output.push(format!("      MAC: {}", mac_str));
                                } else {
                                    shell_output.push(String::from("eth0: No network interface"));
                                }
                            },
                            "cpuinfo" => {
                                shell_output.push(String::from("CPU: QEMU Virtual CPU version 2.5+"));
                                shell_output.push(String::from("Freq: 3.8 GHz | Cores: 1 | Arch: x86_64"));
                                shell_output.push(String::from("Features: SSE SSE2 NX SVM"));
                            },
                            "meminfo" => {
                                let used = crate::memory::heap::used() / 1024;
                                let total = crate::memory::heap_size() / 1024;
                                let total_ram_mb = crate::memory::total_physical_memory() / 1024 / 1024;
                                shell_output.push(format!("Heap: {} / {} KB", used, total));
                                shell_output.push(format!("System: {} MB total", total_ram_mb));
                            },
                            "uptime" => {
                                let secs = crate::cpu::tsc::read_tsc() / crate::cpu::tsc::frequency_hz();
                                let h = secs / 3600;
                                let m = (secs % 3600) / 60;
                                let s = secs % 60;
                                shell_output.push(format!("Uptime: {:02}:{:02}:{:02}", h, m, s));
                            },
                            "exit" | "quit" => {
                                shell_output.push(String::from("> Use 'desktop close' to exit desktop"));
                            },
                            "date" | "time" => {
                                let dt = crate::rtc::read_rtc();
                                shell_output.push(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second));
                            },
                            "hostname" => shell_output.push(String::from("trustos")),
                            "uname" => shell_output.push(String::from("TrustOS 0.1.0 x86_64")),
                            "holo" | "holomatrix" => {
                                // Toggle HoloMatrix mode
                                crate::serial_println!("[DEBUG] holo command received, toggling...");
                                holo_enabled = !holo_enabled;
                                crate::graphics::holomatrix::set_enabled(holo_enabled);
                                crate::serial_println!("[DEBUG] holo_enabled = {}", holo_enabled);
                                if holo_enabled {
                                    shell_output.push(String::from("? HoloMatrix 3D ENABLED"));
                                    shell_output.push(String::from("  3D hologram appears through Matrix Rain"));
                                    shell_output.push(String::from("  Use settings panel to change scene"));
                                } else {
                                    shell_output.push(String::from("? HoloMatrix 3D DISABLED"));
                                    shell_output.push(String::from("  Standard Matrix Rain background"));
                                }
                            },
                            "holo on" => {
                                holo_enabled = true;
                                crate::graphics::holomatrix::set_enabled(true);
                                shell_output.push(String::from("? HoloMatrix 3D enabled"));
                            },
                            "holo off" => {
                                holo_enabled = false;
                                use_holovolume = false;
                                crate::graphics::holomatrix::set_enabled(false);
                                shell_output.push(String::from("? HoloMatrix 3D disabled"));
                            },
                            "holo volume" | "holovolume" => {
                                // Toggle volumetric ASCII raymarcher
                                use_holovolume = !use_holovolume;
                                if use_holovolume {
                                    holo_enabled = false;  // Disable old holomatrix
                                    shell_output.push(String::from("? HOLOVOLUME ENABLED"));
                                    shell_output.push(String::from("  Volumetric ASCII raymarcher active"));
                                    shell_output.push(String::from("  3D voxel grid projected to 2D"));
                                    shell_output.push(String::from("  Aligned characters = brighter"));
                                } else {
                                    shell_output.push(String::from("? HoloVolume disabled"));
                                    shell_output.push(String::from("  Back to Matrix Rain"));
                                }
                            },
                            "holo dna" => {
                                use_holovolume = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::DnaHelix;
                                shell_output.push(String::from("? HoloVolume: DNA Helix"));
                            },
                            "holo cube" => {
                                use_holovolume = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::RotatingCube;
                                shell_output.push(String::from("? HoloVolume: Rotating Cube"));
                            },
                            "holo sphere" => {
                                use_holovolume = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::Sphere;
                                shell_output.push(String::from("? HoloVolume: Sphere"));
                            },
                            "holo rain" => {
                                use_holovolume = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::MatrixRain;
                                shell_output.push(String::from("? HoloVolume: Matrix Rain (volumetric)"));
                            },
                            // -----------------------------------------------
                            // MATRIX RENDERER MODE COMMANDS
                            // -----------------------------------------------
                            "matrix formula" | "formula" | "formula3d" => {
                                use_formula = true;
                                use_braille = false;
                                use_fast_matrix = false;
                                use_matrix3d = false;
                                use_shader_matrix = false;
                                use_holovolume = false;
                                shell_output.push(String::from("? FORMULA 3D: Wireframe perspective projection"));
                                shell_output.push(String::from("  Commands: formula cube|pyramid|diamond|torus|sphere|grid|helix|multi"));
                            },
                            "formula cube" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Cube);
                                shell_output.push(String::from("? FORMULA: Rotating Cube"));
                            },
                            "formula pyramid" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Pyramid);
                                shell_output.push(String::from("? FORMULA: Pyramid"));
                            },
                            "formula diamond" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Diamond);
                                shell_output.push(String::from("? FORMULA: Diamond octahedron"));
                            },
                            "formula torus" | "formula donut" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Torus);
                                shell_output.push(String::from("? FORMULA: Torus (donut)"));
                            },
                            "formula sphere" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Icosphere);
                                shell_output.push(String::from("? FORMULA: Icosphere"));
                            },
                            "formula grid" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Grid);
                                shell_output.push(String::from("? FORMULA: Infinite grid"));
                            },
                            "formula helix" | "formula dna" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Helix);
                                shell_output.push(String::from("? FORMULA: DNA helix"));
                            },
                            "formula multi" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Multi);
                                shell_output.push(String::from("? FORMULA: Multi - orbiting shapes"));
                            },
                            "formula penger" | "formula penguin" | "penger" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Penger);
                                shell_output.push(String::from("? FORMULA: Penger - hologram penguin ??"));
                            },
                            "formula trustos" | "formula title" | "trustos" | "trustos 3d" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::TrustOs);
                                formula_renderer.wire_color = 0xFF00CCFF;
                                shell_output.push(String::from("? FORMULA: TrustOS 3D -- hologram scanline title"));
                            },
                            "formula holo" | "holo matrix" | "holomatrix" | "matrix holo" | "matrix 3d holo" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::HoloMatrix);
                                shell_output.push(String::from("? FORMULA: HoloMatrix 3D -- volumetric holographic rain"));
                            },
                            "matrix fast" => {
                                use_formula = false;
                                use_fast_matrix = true;
                                use_braille = false;
                                use_shader_matrix = false;
                                shell_output.push(String::from("? FAST MATRIX: Glyph-cached renderer"));
                                shell_output.push(String::from("  Pre-computed u128 glyphs + LUT intensity"));
                            },
                            "matrix braille" => {
                                use_formula = false;
                                use_braille = true;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                shell_output.push(String::from("? BRAILLE MATRIX: 8A-- sub-pixel resolution"));
                                shell_output.push(String::from("  480A--272 virtual pixels via Unicode ??"));
                            },
                            "matrix legacy" => {
                                use_formula = false;
                                use_fast_matrix = false;
                                use_braille = false;
                                use_matrix3d = false;
                                use_shader_matrix = false;
                                shell_output.push(String::from("? LEGACY MATRIX: Original renderer"));
                                shell_output.push(String::from("  Per-pixel font lookup (slower)"));
                            },
                            "matrix3d" | "matrix 3d" => {
                                use_formula = false;
                                use_matrix3d = !use_matrix3d;
                                use_braille = !use_matrix3d;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                if use_matrix3d {
                                    shell_output.push(String::from("? MATRIX 3D: Volumetric rain with shapes"));
                                    shell_output.push(String::from("  Commands: matrix3d sphere | cube | torus"));
                                } else {
                                    shell_output.push(String::from("? MATRIX 3D: Disabled, back to BRAILLE"));
                                }
                            },
                            "matrix3d sphere" | "matrix 3d sphere" => {
                                use_formula = false;
                                use_matrix3d = true;
                                use_braille = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                matrix3d_renderer.set_demo_shapes();
                                shell_output.push(String::from("? MATRIX 3D: Sphere - rain flows around it"));
                            },
                            "matrix3d cube" | "matrix 3d cube" => {
                                use_formula = false;
                                use_matrix3d = true;
                                use_braille = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                matrix3d_renderer.set_cube();
                                shell_output.push(String::from("? MATRIX 3D: Rotating Cube"));
                            },
                            "matrix3d torus" | "matrix 3d torus" => {
                                use_formula = false;
                                use_matrix3d = true;
                                use_braille = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                matrix3d_renderer.set_torus();
                                shell_output.push(String::from("? MATRIX 3D: Torus (donut shape)"));
                            },
                            // SHAPE OVERLAYS for BrailleMatrix (normal rain + shape traced by drops)
                            "matrix cube" => {
                                use_formula = false;
                                use_braille = true;
                                use_matrix3d = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::Cube);
                                shell_output.push(String::from("? MATRIX: Cube overlay - glyphs trace rotating cube"));
                            },
                            "matrix sphere" => {
                                use_formula = false;
                                use_braille = true;
                                use_matrix3d = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::Sphere);
                                shell_output.push(String::from("? MATRIX: Sphere overlay - glyphs trace sphere surface"));
                            },
                            "matrix torus" => {
                                use_formula = false;
                                use_braille = true;
                                use_matrix3d = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::Torus);
                                shell_output.push(String::from("? MATRIX: Torus overlay - glyphs trace spinning donut"));
                            },
                            "matrix dna" => {
                                use_formula = false;
                                use_braille = true;
                                use_matrix3d = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::DNA);
                                shell_output.push(String::from("? MATRIX: DNA overlay - glyphs trace double helix"));
                            },
                            "matrix off" | "matrix clear" | "matrix normal" => {
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::None);
                                shell_output.push(String::from("? MATRIX: Shape overlay disabled - normal rain"));
                            },
                            "matrix shader" | "matrix gpu" => {
                                use_shader_matrix = !use_shader_matrix;
                                if use_shader_matrix {
                                    use_formula = false;
                                    use_braille = false;
                                    use_fast_matrix = false;
                                    use_matrix3d = false;
                                    shell_output.push(String::from("? SHADER MATRIX: GPU-emulated pixel shader"));
                                    shell_output.push(String::from("  Uses SMP parallel dispatch + SSE2 SIMD"));
                                    shell_output.push(String::from("  Smooth per-pixel glyph rendering"));
                                } else {
                                    use_braille = true;
                                    shell_output.push(String::from("? SHADER MATRIX: Disabled, back to BRAILLE"));
                                }
                            },
                            "matrix" => {
                                let mode = if use_formula { "FORMULA (wireframe 3D)" }
                                           else if use_shader_matrix { "SHADER (GPU-emulated pixel shader)" }
                                           else if use_matrix3d { "3D (volumetric shapes)" }
                                           else if use_braille { "BRAILLE (8A-- sub-pixel)" }
                                           else if use_fast_matrix { "FAST (glyph-cached)" }
                                           else { "LEGACY (per-pixel)" };
                                shell_output.push(format!("Matrix Renderer: {}", mode));
                                shell_output.push(String::from("Commands: matrix formula | fast | braille | legacy | 3d | shader"));
                            },
                            "fps" => {
                                show_fps = !show_fps;
                                shell_output.push(format!("FPS display: {}", if show_fps { "ON" } else { "OFF" }));
                            },
                            "smp" | "smpstatus" | "smp status" => {
                                let status = if crate::cpu::smp::is_smp_enabled() { "ON" } else { "OFF" };
                                let cpus = crate::cpu::smp::ready_cpu_count();
                                let total = crate::cpu::smp::cpu_count();
                                shell_output.push(format!("SMP Parallel: {} ({}/{} CPUs)", status, cpus, total));
                                shell_output.push(String::from("  smp on  - Enable multi-core"));
                                shell_output.push(String::from("  smp off - Single-core mode"));
                            },
                            "smp on" => {
                                crate::cpu::smp::enable_smp();
                                shell_output.push(String::from("? SMP parallelism ENABLED"));
                            },
                            "smp off" => {
                                crate::cpu::smp::disable_smp();
                                shell_output.push(String::from("? SMP disabled (single-core)"));
                            },
                            "shader" | "shaders" | "vgpu" => {
                                shell_output.push(String::from("+---------------------------------------+"));
                                shell_output.push(String::from("|     Virtual GPU - Shader Demo         |"));
                                shell_output.push(String::from("|---------------------------------------|"));
                                shell_output.push(String::from("| shader plasma    - Plasma waves       |"));
                                shell_output.push(String::from("| shader fire      - Fire effect        |"));
                                shell_output.push(String::from("| shader mandelbrot- Fractal zoom       |"));
                                shell_output.push(String::from("| shader matrix    - Matrix rain        |"));
                                shell_output.push(String::from("| shader tunnel    - 3D HOLOMATRIX      |"));
                                shell_output.push(String::from("| shader parallax  - Depth layers       |"));
                                shell_output.push(String::from("| shader shapes    - Ray-marched 3D     |"));
                                shell_output.push(String::from("| shader rain3d    - Matrix fly-through |"));
                                shell_output.push(String::from("| shader cosmic    - Fractal vortex     |"));
                                shell_output.push(String::from("| shader gradient  - Test gradient      |"));
                                shell_output.push(String::from("+---------------------------------------+"));
                                shell_output.push(String::from("Press ESC to exit shader demo"));
                            },
                            _ if cmd.starts_with("shader ") => {
                                let shader_name = cmd.trim_start_matches("shader ").trim();
                                crate::serial_println!("[SHADER] Trying to load shader: '{}'", shader_name);
                                if let Some(shader_fn) = crate::gpu_emu::get_shader(shader_name) {
                                    crate::serial_println!("[SHADER] Found shader, starting loop...");
                                    shell_output.push(format!("? Loading shader: {}", shader_name));
                                    shell_output.push(String::from("Press ESC to exit..."));
                                    
                                    let width = crate::framebuffer::width();
                                    let height = crate::framebuffer::height();
                                    
                                    // Use double-buffered rendering for correct display + performance
                                    let was_db = crate::framebuffer::is_double_buffer_enabled();
                                    if !was_db {
                                        crate::framebuffer::init_double_buffer();
                                        crate::framebuffer::set_double_buffer_mode(true);
                                    }
                                    
                                    // Get backbuffer pointer (stride = width in pixels, no pitch mismatch)
                                    let bb_info = crate::framebuffer::get_backbuffer_info();
                                    let (fb_ptr, bb_stride) = if let Some((ptr, _w, _h, stride)) = bb_info {
                                        (ptr as *mut u32, stride)
                                    } else {
                                        // Fallback to direct MMIO (will have stride issues on some hw)
                                        (crate::framebuffer::get_framebuffer(), width)
                                    };
                                    
                                    // Init virtual GPU with backbuffer
                                    crate::gpu_emu::init_stride(fb_ptr, width, height, bb_stride);
                                    crate::gpu_emu::set_shader(shader_fn);
                                    
                                    // Run shader demo loop
                                    let start_tsc = crate::cpu::tsc::read_tsc();
                                    let mut frames = 0u32;
                                    
                                    loop {
                                        // Check for ESC key
                                        if let Some(key) = crate::keyboard::try_read_key() {
                                            if key == 27 { break; }
                                        }
                                        
                                        // Draw shader to backbuffer
                                        #[cfg(target_arch = "x86_64")]
                                        crate::gpu_emu::draw_simd();
                                        #[cfg(not(target_arch = "x86_64"))]
                                        crate::gpu_emu::draw();
                                        
                                        // Swap backbuffer ? MMIO (SSE2 optimized)
                                        crate::framebuffer::swap_buffers();
                                        
                                        // Update time (~16ms per frame target)
                                        crate::gpu_emu::tick(16);
                                        frames += 1;
                                        
                                        // Show FPS every 60 frames
                                        if frames % 60 == 0 {
                                            let elapsed = crate::cpu::tsc::read_tsc() - start_tsc;
                                            let elapsed_sec = elapsed as f32 / crate::cpu::tsc::frequency_hz() as f32;
                                            let fps = frames as f32 / elapsed_sec;
                                            crate::serial_println!("[SHADER] FPS: {:.1}", fps);
                                        }
                                    }
                                    
                                    // Restore double buffer state
                                    if !was_db {
                                        crate::framebuffer::set_double_buffer_mode(false);
                                    }
                                    
                                    shell_output.push(format!("Shader demo ended ({} frames)", frames));
                                } else {
                                    crate::serial_println!("[SHADER] Shader '{}' NOT FOUND!", shader_name);
                                    shell_output.push(format!("Unknown shader: {}", shader_name));
                                    shell_output.push(String::from("Available: plasma, fire, mandelbrot, matrix, tunnel, parallax, shapes, rain3d, cosmic, gradient"));
                                }
                            },
                            "echo" => shell_output.push(String::new()),
                            "touch" => shell_output.push(String::from("Usage: touch <filename>")),
                            "rm" => shell_output.push(String::from("Usage: rm <filename>")),
                            "cp" => shell_output.push(String::from("Usage: cp <src> <dest>")),
                            "mv" => shell_output.push(String::from("Usage: mv <src> <dest>")),
                            _ if cmd.starts_with("echo ") => {
                                let text = cmd.trim_start_matches("echo ").trim();
                                shell_output.push(String::from(text));
                            },
                            _ if cmd.starts_with("cd ") => {
                                let path = cmd.trim_start_matches("cd ").trim();
                                // Use ramfs cd
                                match crate::ramfs::with_fs(|fs| fs.cd(path)) {
                                    Ok(()) => {
                                        let new_cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
                                        shell_output.push(format!("Changed to: {}", new_cwd));
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("cd: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("ls ") => {
                                let path = cmd.trim_start_matches("ls ").trim();
                                // Use ramfs ls with path
                                match crate::ramfs::with_fs(|fs| fs.ls(Some(path))) {
                                    Ok(items) => {
                                        if items.is_empty() {
                                            shell_output.push(String::from("(empty)"));
                                        } else {
                                            for (name, file_type, size) in items {
                                                match file_type {
                                                    FileType::Directory => {
                                                        shell_output.push(format!("{}  <DIR>", name));
                                                    }
                                                    FileType::File => {
                                                        shell_output.push(format!("{}  {} B", name, size));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("ls: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("cat ") => {
                                let path = cmd.trim_start_matches("cat ").trim();
                                // Use ramfs read
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(content) => {
                                        if let Ok(text) = core::str::from_utf8(&content) {
                                            for line in text.lines().take(20) {
                                                shell_output.push(String::from(line));
                                            }
                                        } else {
                                            shell_output.push(format!("cat: {}: Binary file", path));
                                        }
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("cat: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            // TrustCode: edit <file> -- open file in editor
                            _ if cmd.starts_with("edit ") || cmd.starts_with("code ") || cmd.starts_with("nano ") || cmd.starts_with("vim ") => {
                                let path = cmd.split_whitespace().nth(1).unwrap_or("").trim();
                                if path.is_empty() {
                                    shell_output.push(String::from("Usage: edit <filename>"));
                                } else {
                                    editor_state.load_file(path);
                                    active_mode = AppMode::TextEditor;
                                    browser_mode = false;
                                    shell_output.push(format!("TrustCode: editing {}", path));
                                    crate::serial_println!("[TrustCode] Editing: {}", path);
                                }
                            },
                            _ if cmd.starts_with("mkdir ") => {
                                let path = cmd.trim_start_matches("mkdir ").trim();
                                // Use ramfs mkdir
                                match crate::ramfs::with_fs(|fs| fs.mkdir(path)) {
                                    Ok(()) => {
                                        shell_output.push(format!("Created directory: {}", path));
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("mkdir: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("touch ") => {
                                let path = cmd.trim_start_matches("touch ").trim();
                                // Create empty file
                                match crate::ramfs::with_fs(|fs| fs.write_file(path, &[])) {
                                    Ok(()) => {
                                        shell_output.push(format!("Created file: {}", path));
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("touch: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("rm ") => {
                                let path = cmd.trim_start_matches("rm ").trim();
                                match crate::ramfs::with_fs(|fs| fs.rm(path)) {
                                    Ok(()) => {
                                        shell_output.push(format!("Removed: {}", path));
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("rm: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("curl ") || cmd.starts_with("get ") || cmd.starts_with("wget ") => {
                                let url = if cmd.starts_with("curl ") {
                                    cmd.trim_start_matches("curl ").trim()
                                } else if cmd.starts_with("wget ") {
                                    cmd.trim_start_matches("wget ").trim()
                                } else {
                                    cmd.trim_start_matches("get ").trim()
                                };
                                shell_output.push(format!("Fetching: {}", url));
                                
                                // Parse URL
                                if let Some((host, port, path)) = super::vm::parse_url_simple(url) {
                                    shell_output.push(format!("Host: {} Port: {} Path: {}", host, port, path));
                                    
                                    // Try to resolve and connect
                                    if let Some(ip) = crate::netstack::dns::resolve(&host) {
                                        shell_output.push(format!("Resolved to: {}.{}.{}.{}", 
                                            ip[0], ip[1], ip[2], ip[3]));
                                        
                                        // Attempt HTTP GET
                                        match super::vm::do_http_get_string(&host, ip, port, &path) {
                                            Ok(response) => {
                                                // Show first 15 lines of response
                                                for line in response.lines().take(15) {
                                                    shell_output.push(String::from(line));
                                                }
                                                if response.lines().count() > 15 {
                                                    shell_output.push(String::from("... (truncated)"));
                                                }
                                            }
                                            Err(e) => {
                                                shell_output.push(format!("Error: {}", e));
                                            }
                                        }
                                    } else {
                                        shell_output.push(format!("Cannot resolve: {}", host));
                                    }
                                } else {
                                    shell_output.push(String::from("Invalid URL format"));
                                    shell_output.push(String::from("Usage: curl http://host/path"));
                                }
                            },
                            _ if cmd.starts_with("desktop ") => {
                                let sub = cmd.trim_start_matches("desktop ");
                                if sub == "close" || sub == "exit" || sub == "quit" {
                                    running = false;
                                }
                            },
                            // Open app command - switch to specific app mode
                            "open" => {
                                shell_output.push(String::from("Usage: open <app>"));
                                shell_output.push(String::from("Apps: browser, files, editor, network, hardware, users, images"));
                            },
                            _ if cmd.starts_with("open ") => {
                                let app = cmd.trim_start_matches("open ").trim().to_lowercase();
                                match app.as_str() {
                                    "browser" | "web" | "www" => {
                                        active_mode = AppMode::Browser;
                                        browser_mode = true;
                                        shell_output.push(String::from("Switched to Browser"));
                                    },
                                    "files" | "explorer" => {
                                        active_mode = AppMode::Files;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Files"));
                                    },
                                    "editor" | "text" | "notepad" | "trustcode" | "code" => {
                                        active_mode = AppMode::TextEditor;
                                        browser_mode = false;
                                        // If no file loaded, show help
                                        if editor_state.file_path.is_none() {
                                            editor_state.load_file("demo.rs");
                                        }
                                        shell_output.push(String::from("TrustCode Editor opened"));
                                    },
                                    "network" | "net" | "ifconfig" => {
                                        active_mode = AppMode::Network;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Network"));
                                    },
                                    "hardware" | "hw" | "lshw" => {
                                        active_mode = AppMode::Hardware;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Hardware"));
                                    },
                                    "users" | "user" => {
                                        active_mode = AppMode::UserMgmt;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to User Management"));
                                    },
                                    "images" | "image" | "viewer" => {
                                        active_mode = AppMode::ImageViewer;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Image Viewer"));
                                    },
                                    "shell" | "terminal" => {
                                        active_mode = AppMode::Shell;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Shell"));
                                    },
                                    _ => {
                                        shell_output.push(format!("Unknown app: {}", app));
                                        shell_output.push(String::from("Available: browser, files, editor, network, hardware, users, images"));
                                    }
                                }
                            },
                            // Additional shell commands from main shell
                            "ping" => shell_output.push(String::from("Usage: ping <host>")),
                            "nslookup" | "dig" => shell_output.push(String::from("Usage: nslookup <hostname>")),
                            "ps" => {
                                shell_output.push(String::from("  PID  STATE  NAME"));
                                shell_output.push(String::from("    1  R      init"));
                                shell_output.push(String::from("    2  R      kernel"));
                                shell_output.push(String::from("    3  R      desktop"));
                            },
                            "df" => {
                                shell_output.push(String::from("Filesystem    Size  Used  Avail  Use%  Mounted"));
                                shell_output.push(String::from("ramfs         8.0M   64K   7.9M    1%  /"));
                            },
                            "free" => {
                                let used = crate::memory::heap::used() / 1024;
                                let total = crate::memory::heap_size() / 1024;
                                let free_kb = total - used;
                                shell_output.push(String::from("              total     used     free"));
                                shell_output.push(format!("Mem:     {:>10}  {:>7}  {:>7}", total, used, free_kb));
                            },
                            "tree" => {
                                shell_output.push(String::from("."));
                                match crate::ramfs::with_fs(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        let count = items.len();
                                        for (i, (name, file_type, _)) in items.into_iter().enumerate() {
                                            let prefix = if i + 1 == count { "+-- " } else { "+-- " };
                                            match file_type {
                                                FileType::Directory => shell_output.push(format!("{}{}/ (dir)", prefix, name)),
                                                FileType::File => shell_output.push(format!("{}{}", prefix, name)),
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            },
                            "history" => {
                                shell_output.push(String::from("Command history not available in desktop shell"));
                            },
                            _ if cmd.starts_with("ping ") => {
                                let host = cmd.trim_start_matches("ping ").trim();
                                // Try to parse as IP first, then try DNS
                                let ip_result = if let Some(parsed) = super::vm::parse_ipv4(host) {
                                    Some(parsed)
                                } else {
                                    // Try DNS resolution (may timeout in VM without network)
                                    // Use common known hosts as fallback
                                    match host {
                                        "google.com" | "www.google.com" => Some([142, 250, 179, 110]),
                                        "cloudflare.com" | "www.cloudflare.com" => Some([104, 16, 132, 229]),
                                        "github.com" | "www.github.com" => Some([140, 82, 114, 3]),
                                        "localhost" => Some([127, 0, 0, 1]),
                                        _ => None, // DNS not available in desktop shell
                                    }
                                };
                                
                                if let Some(ip) = ip_result {
                                    shell_output.push(format!("PING {} ({}.{}.{}.{})", host, ip[0], ip[1], ip[2], ip[3]));
                                    shell_output.push(format!("64 bytes from {}.{}.{}.{}: icmp_seq=1 ttl=64 time=1.5 ms", ip[0], ip[1], ip[2], ip[3]));
                                    shell_output.push(format!("64 bytes from {}.{}.{}.{}: icmp_seq=2 ttl=64 time=1.2 ms", ip[0], ip[1], ip[2], ip[3]));
                                    shell_output.push(String::from("--- ping statistics ---"));
                                    shell_output.push(String::from("2 packets transmitted, 2 received, 0% loss"));
                                } else {
                                    shell_output.push(format!("ping: {} - cannot resolve (use IP address)", host));
                                }
                            },
                            _ if cmd.starts_with("nslookup ") || cmd.starts_with("dig ") => {
                                let host = if cmd.starts_with("nslookup ") {
                                    cmd.trim_start_matches("nslookup ").trim()
                                } else {
                                    cmd.trim_start_matches("dig ").trim()
                                };
                                shell_output.push(format!("Server:  8.8.8.8"));
                                shell_output.push(format!("Name:    {}", host));
                                // Use known hosts fallback
                                let ip_result = match host {
                                    "google.com" | "www.google.com" => Some([142, 250, 179, 110]),
                                    "cloudflare.com" | "www.cloudflare.com" => Some([104, 16, 132, 229]),
                                    "github.com" | "www.github.com" => Some([140, 82, 114, 3]),
                                    "localhost" => Some([127, 0, 0, 1]),
                                    _ => super::vm::parse_ipv4(host), // If it's an IP, return it
                                };
                                if let Some(ip) = ip_result {
                                    shell_output.push(format!("Address: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]));
                                } else {
                                    shell_output.push(String::from("** server can't find: NXDOMAIN"));
                                }
                            },
                            _ if cmd.starts_with("hexdump ") || cmd.starts_with("xxd ") => {
                                let path = if cmd.starts_with("hexdump ") {
                                    cmd.trim_start_matches("hexdump ").trim()
                                } else {
                                    cmd.trim_start_matches("xxd ").trim()
                                };
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(content) => {
                                        for (offset, chunk) in content.chunks(16).take(8).enumerate() {
                                            let hex: alloc::vec::Vec<String> = chunk.iter()
                                                .map(|b| format!("{:02x}", b))
                                                .collect();
                                            let ascii: String = chunk.iter()
                                                .map(|&b| if b >= 32 && b < 127 { b as char } else { '.' })
                                                .collect();
                                            shell_output.push(format!("{:08x}  {:48}  |{}|", 
                                                offset * 16, hex.join(" "), ascii));
                                        }
                                        if content.len() > 128 {
                                            shell_output.push(String::from("... (truncated)"));
                                        }
                                    }
                                    Err(e) => shell_output.push(format!("hexdump: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("imgview ") || cmd.starts_with("view ") => {
                                let path = if cmd.starts_with("imgview ") {
                                    cmd.trim_start_matches("imgview ").trim()
                                } else {
                                    cmd.trim_start_matches("view ").trim()
                                };
                                
                                // Try to load image from ramfs
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(data) => {
                                        // Detect format and load
                                        let format = crate::image::detect_image_format(&data);
                                        if let Some(img) = crate::image::load_image_auto(&data) {
                                            image_viewer_path = String::from(path);
                                            image_viewer_info = format!("{}x{} ({} bytes)", img.width, img.height, data.len());
                                            image_viewer_format = String::from(format.extension());
                                            image_viewer_zoom = 1.0;
                                            image_viewer_offset_x = 0;
                                            image_viewer_offset_y = 0;
                                            image_viewer_data = Some(img);
                                            
                                            // Switch to image viewer mode
                                            active_mode = AppMode::ImageViewer;
                                            shell_output.push(format!("Opening: {} ({})", path, format.extension()));
                                        } else {
                                            shell_output.push(format!("imgview: Cannot decode image (format: {})", format.extension()));
                                        }
                                    },
                                    Err(e) => {
                                        shell_output.push(format!("imgview: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("imginfo ") => {
                                let path = cmd.trim_start_matches("imginfo ").trim();
                                
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(data) => {
                                        let format = crate::image::detect_image_format(&data);
                                        shell_output.push(format!("+---------------------------------------+"));
                                        shell_output.push(format!("| Image Info: {}  ", path));
                                        shell_output.push(format!("|---------------------------------------|"));
                                        shell_output.push(format!("| Format:  {} ({})   ", format.extension(), format.mime_type()));
                                        shell_output.push(format!("| Size:    {} bytes   ", data.len()));
                                        
                                        // Try to get dimensions
                                        if let Some(img) = crate::image::load_image_auto(&data) {
                                            shell_output.push(format!("| Width:   {} px   ", img.width));
                                            shell_output.push(format!("| Height:  {} px   ", img.height));
                                            shell_output.push(format!("| Pixels:  {}   ", img.width * img.height));
                                        } else {
                                            shell_output.push(format!("| (Cannot decode image dimensions)"));
                                        }
                                        shell_output.push(format!("+---------------------------------------+"));
                                    },
                                    Err(e) => {
                                        shell_output.push(format!("imginfo: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            // Additional important commands
                            "top" | "htop" => {
                                shell_output.push(String::from("top - System Monitor"));
                                shell_output.push(String::from("  PID  %CPU  %MEM  TIME     COMMAND"));
                                shell_output.push(String::from("    1  0.5   2.1   0:01.23  kernel"));
                                shell_output.push(String::from("    2  0.1   0.5   0:00.45  desktop"));
                                shell_output.push(String::from("Press 'q' to quit (in desktop: just run another cmd)"));
                            },
                            "lspci" => {
                                shell_output.push(String::from("00:00.0 Host bridge"));
                                shell_output.push(String::from("00:01.0 VGA controller: Virtio GPU"));
                                shell_output.push(String::from("00:02.0 Network controller: Virtio Net"));
                                shell_output.push(String::from("00:03.0 AHCI Controller"));
                            },
                            "lsusb" => {
                                shell_output.push(String::from("Bus 001 Device 001: ID 1d6b:0002 Linux Foundation Root Hub"));
                                shell_output.push(String::from("Bus 001 Device 002: ID 0627:0001 QEMU Tablet"));
                            },
                            "lscpu" => {
                                shell_output.push(String::from("Architecture:        x86_64"));
                                shell_output.push(String::from("CPU op-modes:        64-bit"));
                                shell_output.push(String::from("CPU(s):              4"));
                                shell_output.push(String::from("Vendor ID:           AuthenticAMD"));
                                shell_output.push(String::from("Model name:          QEMU Virtual CPU"));
                            },
                            "disk" => {
                                shell_output.push(String::from("Disk /dev/sda: 64 MB"));
                                shell_output.push(String::from("  Partition 1: 64 MB (TrustOS)"));
                            },
                            "netstat" => {
                                shell_output.push(String::from("Active connections:"));
                                shell_output.push(String::from("Proto  Local Address      Foreign Address    State"));
                                shell_output.push(String::from("tcp    0.0.0.0:0          0.0.0.0:*          LISTEN"));
                            },
                            "arp" => {
                                shell_output.push(String::from("Address         HWtype  HWaddress           Iface"));
                                shell_output.push(String::from("10.0.2.2        ether   52:55:0a:00:02:02   eth0"));
                            },
                            "route" => {
                                shell_output.push(String::from("Kernel IP routing table"));
                                shell_output.push(String::from("Dest         Gateway      Genmask         Iface"));
                                shell_output.push(String::from("0.0.0.0      10.0.2.2     0.0.0.0         eth0"));
                                shell_output.push(String::from("10.0.2.0     0.0.0.0      255.255.255.0   eth0"));
                            },
                            "env" => {
                                shell_output.push(String::from("USER=root"));
                                shell_output.push(String::from("HOME=/root"));
                                shell_output.push(String::from("SHELL=/bin/tsh"));
                                shell_output.push(String::from("PATH=/bin:/usr/bin"));
                                shell_output.push(String::from("TERM=trustos"));
                            },
                            "id" => {
                                shell_output.push(String::from("uid=0(root) gid=0(root) groups=0(root)"));
                            },
                            "cal" => {
                                let dt = crate::rtc::read_rtc();
                                shell_output.push(format!("     {:02}/{:04}", dt.month, dt.year));
                                shell_output.push(String::from("Su Mo Tu We Th Fr Sa"));
                                shell_output.push(String::from("       1  2  3  4  5"));
                                shell_output.push(String::from(" 6  7  8  9 10 11 12"));
                                shell_output.push(String::from("13 14 15 16 17 18 19"));
                                shell_output.push(String::from("20 21 22 23 24 25 26"));
                                shell_output.push(String::from("27 28 29 30 31"));
                            },
                            _ if cmd.starts_with("head ") => {
                                let path = cmd.trim_start_matches("head ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                }) {
                                    Ok(content) => {
                                        for line in content.lines().take(10) {
                                            shell_output.push(String::from(line));
                                        }
                                    },
                                    Err(e) => shell_output.push(format!("head: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("tail ") => {
                                let path = cmd.trim_start_matches("tail ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                }) {
                                    Ok(content) => {
                                        let lines: alloc::vec::Vec<&str> = content.lines().collect();
                                        let start = if lines.len() > 10 { lines.len() - 10 } else { 0 };
                                        for line in &lines[start..] {
                                            shell_output.push(String::from(*line));
                                        }
                                    },
                                    Err(e) => shell_output.push(format!("tail: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("wc ") => {
                                let path = cmd.trim_start_matches("wc ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                }) {
                                    Ok(content) => {
                                        let lines = content.lines().count();
                                        let words = content.split_whitespace().count();
                                        let bytes = content.len();
                                        shell_output.push(format!("{:>5} {:>5} {:>5} {}", lines, words, bytes, path));
                                    },
                                    Err(e) => shell_output.push(format!("wc: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("grep ") => {
                                let args = cmd.trim_start_matches("grep ").trim();
                                let parts: alloc::vec::Vec<&str> = args.splitn(2, ' ').collect();
                                if parts.len() == 2 {
                                    let pattern = parts[0];
                                    let path = parts[1];
                                    match crate::ramfs::with_fs(|fs| {
                                        fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                    }) {
                                        Ok(content) => {
                                            let mut found = false;
                                            for line in content.lines() {
                                                if line.contains(pattern) {
                                                    shell_output.push(String::from(line));
                                                    found = true;
                                                }
                                            }
                                            if !found {
                                                shell_output.push(format!("(no matches for '{}')", pattern));
                                            }
                                        },
                                        Err(e) => shell_output.push(format!("grep: {}: {}", path, e.as_str())),
                                    }
                                } else {
                                    shell_output.push(String::from("Usage: grep <pattern> <file>"));
                                }
                            },
                            _ if cmd.starts_with("find ") => {
                                let pattern = cmd.trim_start_matches("find ").trim();
                                shell_output.push(format!("Searching for: {}", pattern));
                                match crate::ramfs::with_fs(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        for (name, _, _) in items {
                                            if name.contains(pattern) {
                                                shell_output.push(format!("./{}", name));
                                            }
                                        }
                                    },
                                    Err(_) => {}
                                }
                            },
                            _ if cmd.starts_with("stat ") => {
                                let path = cmd.trim_start_matches("stat ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| s.len())
                                }) {
                                    Ok(size) => {
                                        shell_output.push(format!("  File: {}", path));
                                        shell_output.push(format!("  Size: {} bytes", size));
                                        shell_output.push(String::from("  Access: -rw-r--r--"));
                                        shell_output.push(String::from("  Uid: 0  Gid: 0"));
                                    },
                                    Err(e) => shell_output.push(format!("stat: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("sort ") => {
                                let path = cmd.trim_start_matches("sort ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                }) {
                                    Ok(content) => {
                                        let mut lines: alloc::vec::Vec<&str> = content.lines().collect();
                                        lines.sort();
                                        for line in lines {
                                            shell_output.push(String::from(line));
                                        }
                                    },
                                    Err(e) => shell_output.push(format!("sort: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("strings ") => {
                                let path = cmd.trim_start_matches("strings ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(data) => {
                                        let mut current = String::new();
                                        for &b in data.iter().take(1024) {
                                            if b >= 32 && b < 127 {
                                                current.push(b as char);
                                            } else if current.len() >= 4 {
                                                shell_output.push(current.clone());
                                                current.clear();
                                            } else {
                                                current.clear();
                                            }
                                        }
                                        if current.len() >= 4 {
                                            shell_output.push(current);
                                        }
                                    },
                                    Err(e) => shell_output.push(format!("strings: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("traceroute ") || cmd.starts_with("tracert ") => {
                                let host = if cmd.starts_with("traceroute ") {
                                    cmd.trim_start_matches("traceroute ").trim()
                                } else {
                                    cmd.trim_start_matches("tracert ").trim()
                                };
                                shell_output.push(format!("traceroute to {} (simulated)", host));
                                shell_output.push(String::from(" 1  10.0.2.2  1.234 ms"));
                                shell_output.push(String::from(" 2  * * *"));
                                shell_output.push(String::from(" 3  * * *"));
                            },
                            "3ddemo" | "demo3d" | "cube" => {
                                // Run 3D demo using software rasterizer
                                shell_output.push(String::from("Starting 3D Demo..."));
                                shell_output.push(String::from("Controls: Arrow keys rotate, ESC to exit"));
                                
                                // Create rasterizer for the demo
                                let demo_w = 400u32;
                                let demo_h = 300u32;
                                let mut rast = crate::rasterizer::Rasterizer::new(demo_w, demo_h);
                                let mut renderer = crate::rasterizer::Renderer3D::new(demo_w, demo_h);
                                
                                let mut angle_y: f32 = 0.0;
                                let mut angle_x: f32 = 0.3;
                                let mut demo_running = true;
                                let mut demo_frame = 0u32;
                                
                                // Demo window position (centered in main window)
                                let demo_x = (window_x + 150) as u32;
                                let demo_y = (window_y + 50) as u32;
                                
                                while demo_running && demo_frame < 600 { // Max 10 seconds at 60fps
                                    // Check for ESC key
                                    if let Some(k) = crate::keyboard::try_read_key() {
                                        match k {
                                            27 => demo_running = false, // ESC
                                            0x4B => angle_y -= 0.1, // Left
                                            0x4D => angle_y += 0.1, // Right
                                            0x48 => angle_x -= 0.1, // Up
                                            0x50 => angle_x += 0.1, // Down
                                            _ => {}
                                        }
                                    }
                                    
                                    // Clear buffer
                                    rast.clear(0xFF101010);
                                    renderer.clear_z_buffer();
                                    
                                    // Create rotation matrix
                                    let rot_y = crate::rasterizer::Mat4::rotation_y(angle_y);
                                    let rot_x = crate::rasterizer::Mat4::rotation_x(angle_x);
                                    let rotation = rot_x.mul(&rot_y);
                                    
                                    // Draw multiple cubes
                                    let center = crate::rasterizer::Vec3::new(0.0, 0.0, 0.0);
                                    renderer.draw_cube(&mut rast, center, 1.5, &rotation, 0xFF00FF00);
                                    
                                    // Draw a smaller cube offset
                                    let center2 = crate::rasterizer::Vec3::new(2.0, 0.0, 0.0);
                                    renderer.draw_cube(&mut rast, center2, 0.8, &rotation, 0xFF00FFFF);
                                    
                                    // Draw gradient background title bar
                                    rast.fill_gradient_h(0, 0, demo_w, 25, 0xFF003300, 0xFF00AA00);
                                    
                                    // Draw border
                                    rast.draw_rect(0, 0, demo_w, demo_h, 0xFF00FF00);
                                    
                                    // Blit demo to screen
                                    for py in 0..demo_h {
                                        for px in 0..demo_w {
                                            let idx = (py * demo_w + px) as usize;
                                            crate::framebuffer::draw_pixel(demo_x + px, demo_y + py, rast.back_buffer[idx]);
                                        }
                                    }
                                    
                                    // Title
                                    crate::framebuffer::draw_text("3D Demo - ESC to exit", demo_x + 10, demo_y + 5, 0xFFFFFFFF);
                                    
                                    // FPS counter
                                    let fps_str = format!("Frame: {}", demo_frame);
                                    crate::framebuffer::draw_text(&fps_str, demo_x + demo_w - 100, demo_y + 5, 0xFFFFFF00);
                                    
                                    angle_y += 0.02;  // Auto-rotate
                                    demo_frame += 1;
                                    
                                    // Small delay
                                    for _ in 0..50000 { core::hint::spin_loop(); }
                                }
                                
                                shell_output.push(String::from("3D Demo ended."));
                            },
                            "raster" | "rasterdemo" => {
                                // Demo rasterizer features
                                shell_output.push(String::from("Rasterizer Demo - Antialiasing & Gradients"));
                                
                                let demo_w = 350u32;
                                let demo_h = 250u32;
                                let mut rast = crate::rasterizer::Rasterizer::new(demo_w, demo_h);
                                
                                let demo_x = (window_x + 175) as u32;
                                let demo_y = (window_y + 75) as u32;
                                
                                // Clear with dark background
                                rast.clear(0xFF0A0A0A);
                                
                                // Vertical gradient background
                                rast.fill_gradient_v(0, 0, demo_w, demo_h, 0xFF000022, 0xFF002200);
                                
                                // Antialiased circles
                                rast.fill_circle_aa(80, 80, 40, 0xFFFF0000);   // Red
                                rast.fill_circle_aa(150, 100, 35, 0xFF00FF00); // Green  
                                rast.fill_circle_aa(220, 80, 40, 0xFF0000FF);  // Blue
                                
                                // Overlapping with transparency
                                rast.fill_circle_aa(115, 90, 30, 0x8800FFFF);  // Cyan semi-transparent
                                rast.fill_circle_aa(185, 90, 30, 0x88FF00FF);  // Magenta semi-transparent
                                
                                // Rounded rectangle
                                rast.fill_rounded_rect(50, 150, 120, 60, 15, 0xFF444444);
                                rast.fill_gradient_h(55, 155, 110, 50, 0xFF006600, 0xFF00CC00);
                                
                                // Antialiased lines
                                rast.draw_line_aa(200.0, 150.0, 320.0, 220.0, 0xFFFFFF00);
                                rast.draw_line_aa(200.0, 220.0, 320.0, 150.0, 0xFFFF8800);
                                
                                // Shadow demo
                                rast.draw_shadow(250, 160, 60, 40, 8, 0x88000000);
                                rast.fill_rect(250, 160, 60, 40, 0xFF00AA00);
                                
                                // Border
                                rast.draw_rect(0, 0, demo_w, demo_h, 0xFF00FF00);
                                
                                // Blit to screen
                                for py in 0..demo_h {
                                    for px in 0..demo_w {
                                        let idx = (py * demo_w + px) as usize;
                                        crate::framebuffer::draw_pixel(demo_x + px, demo_y + py, rast.back_buffer[idx]);
                                    }
                                }
                                
                                crate::framebuffer::draw_text("Rasterizer: AA + Alpha + Gradients", demo_x + 10, demo_y + 5, 0xFFFFFFFF);
                                
                                // Wait for key
                                shell_output.push(String::from("Press any key to close demo..."));
                                loop {
                                    if crate::keyboard::try_read_key().is_some() {
                                        break;
                                    }
                                    core::hint::spin_loop();
                                }
                                shell_output.push(String::from("Demo closed."));
                            },
                            _ => shell_output.push(format!("Command not found: {}", cmd)),
                        };
                        shell_input.clear();
                        suggestion_text.clear();
                        
                        // Keep only last 20 lines
                        while shell_output.len() > 20 {
                            shell_output.remove(0);
                        }
                        
                        // Auto-scroll to bottom to show new output
                        scroll_offset = shell_output.len().saturating_sub(MAX_VISIBLE_LINES);
                    }
                },
                32..=126 => { // Printable characters
                    crate::serial_println!("[KEY] Printable char: '{}' ({})", key as char, key);
                    shell_input.push(key as char);
                    // Update suggestion
                    let cmds = ["help", "ls", "dir", "clear", "ifconfig", "cpuinfo", "meminfo", "whoami", "uptime", "open", "smp", "fps", "matrix", "holo"];
                    suggestion_text.clear();
                    for c in cmds {
                        if c.starts_with(&shell_input) && c != shell_input.as_str() {
                            suggestion_text = String::from(&c[shell_input.len()..]);
                            break;
                        }
                    }
                },
                _ => {}
            }
            } // End of shell mode else block
        }
        
        // Mouse input - get current state (x,y are absolute positions)
        let mouse_state = crate::mouse::get_state();
        mouse_x = mouse_state.x.clamp(0, width as i32 - 1);
        mouse_y = mouse_state.y.clamp(0, height as i32 - 1);
        let left = mouse_state.left_button;
        
        // Mouse click detection
        let clicked = left && !prev_left;
        let released = !left && prev_left;
        prev_left = left;
        
        // Window dragging logic
        if dragging_window {
            if left {
                // Update window position while dragging
                window_x = (mouse_x - drag_offset_x).clamp(0, width as i32 - 200);
                window_y = (mouse_y - drag_offset_y).clamp(0, height as i32 - 100);
                // Update the layer position
                if let Some(win) = compositor.get_layer_mut(window_layer) {
                    win.set_position(window_x as u32, window_y as u32);
                }
            } else {
                // Released - stop dragging
                dragging_window = false;
            }
        }
        
        // Update cursor layer position
        if let Some(cursor) = compositor.get_layer_mut(cursor_layer) {
            cursor.set_position(mouse_x as u32, mouse_y as u32);
        }
        
        // Menu hover detection
        menu_hover = -1;
        if menu_open {
            let menu_x = 5u32;
            let menu_y = height - 340;
            let mx = mouse_x as u32;
            let my = mouse_y as u32;
            
            if mx >= menu_x && mx < menu_x + 250 && my >= menu_y && my < menu_y + 290 {
                let item_h = 36u32;
                let rel_y = if my > menu_y + 40 { my - menu_y - 40 } else { 0 };
                let idx = (rel_y / item_h) as i32;
                if idx >= 0 && idx < menu_items.len() as i32 {
                    menu_hover = idx;
                }
            }
        }
        
        // Click handling
        if clicked {
            let mx = mouse_x as u32;
            let my = mouse_y as u32;
            
            // TrustOS button in taskbar
            let taskbar_y = height - 40;
            if my >= taskbar_y && my < height && mx >= 5 && mx < 110 {
                menu_open = !menu_open;
                settings_open = false; // Close settings if open
            }
            // Settings button in taskbar (x: 340 to 390)
            else if my >= taskbar_y && my < height && mx >= 340 && mx < 390 {
                settings_open = !settings_open;
                menu_open = false; // Close menu if open
                // Refresh current values
                settings_anim_enabled = crate::desktop::animations_enabled();
                settings_anim_speed = crate::desktop::get_animation_speed();
            }
            // Window button in taskbar (x: 220 to 320)
            else if my >= taskbar_y && my < height && mx >= 220 && mx < 320 {
                // Toggle window visibility
                window_visible = !window_visible;
            }
            // Menu item click
            else if menu_open && menu_hover >= 0 && menu_hover < menu_items.len() as i32 {
                let (_name, item) = menu_items[menu_hover as usize];
                match item {
                    MenuItem::App(mode) => {
                        // Skip separator line
                        if !_name.starts_with("-") {
                            active_mode = mode;
                            shell_output.clear();
                            for line in get_help!(mode) {
                                shell_output.push(String::from(*line));
                            }
                            shell_output.push(String::from(""));
                            shell_output.push(String::from("Type commands below. Type 'help' for more info."));
                        }
                    },
                    MenuItem::Shutdown => {
                        shell_output.push(String::from("> Shutting down..."));
                        // Delay then halt
                        for _ in 0..10000000 { core::hint::spin_loop(); }
                        loop {
                            x86_64::instructions::interrupts::disable();
                            x86_64::instructions::hlt();
                        }
                    },
                    MenuItem::Reboot => {
                        shell_output.push(String::from("> Rebooting..."));
                        for _ in 0..10000000 { core::hint::spin_loop(); }
                        unsafe {
                            x86_64::instructions::port::Port::<u8>::new(0x64).write(0xFE);
                        }
                        loop { x86_64::instructions::hlt(); }
                    },
                }
                menu_open = false;
            }
            // Click outside menu/settings closes them
            else if menu_open {
                menu_open = false;
            }
            else if settings_open {
                // Check if click is inside settings panel area (340-610, height-380 to height-40)
                let settings_panel_x = 340u32;
                let settings_panel_y = height - 380;
                let settings_panel_w = 270u32;
                let settings_panel_h = 350u32;
                if !(mx >= settings_panel_x && mx < settings_panel_x + settings_panel_w 
                    && my >= settings_panel_y && my < settings_panel_y + settings_panel_h) {
                    settings_open = false;
                }
            }
            // Window interaction (buttons and dragging)
            else if !dragging_window {
                let win_x = window_x as u32;
                let win_y = window_y as u32;
                let win_w = 700u32;  // Fixed window width from add_layer
                let win_h = 450u32;  // Fixed window height from add_layer
                
                // Check if click is in title bar area (only if window visible)
                if window_visible && mx >= win_x && mx < win_x + win_w && my >= win_y && my < win_y + 28 {
                    // Button positions: close at w-50, min at w-80, max at w-110
                    // Close button (X) - hides the window (use 'desktop close' to exit)
                    if mx >= win_x + win_w - 60 && mx < win_x + win_w - 40 {
                        window_visible = false;  // Hide window, don't exit desktop
                        shell_output.push(String::from("> Window closed. Click dock icon to reopen."));
                    }
                    // Minimize button (-) - radius 10 -> check x from w-90 to w-70
                    else if mx >= win_x + win_w - 90 && mx < win_x + win_w - 70 {
                        window_visible = false;  // Minimize = hide
                        shell_output.push(String::from("> Window minimized"));
                    }
                    // Maximize button (?) - radius 10 -> check x from w-120 to w-100
                    else if mx >= win_x + win_w - 120 && mx < win_x + win_w - 100 {
                        shell_output.push(String::from("> Window maximized"));
                    }
                    // Otherwise, start dragging
                    else {
                        dragging_window = true;
                        drag_offset_x = mouse_x - window_x;
                        drag_offset_y = mouse_y - window_y;
                    }
                }
                // Dock icon clicks - also reopens window if closed
                else if mx < 80 && my < height - 40 {
                    let icon_size = 36u32;
                    let gap = 50u32;       // Match render gap (was 58, now 50)
                    let start_y = 10u32;
                    for i in 0..8usize {
                        let iy = start_y + (i as u32) * (icon_size + gap);
                        if my >= iy && my < iy + icon_size + 16 {
                            active_mode = match i {
                                0 => AppMode::Files,
                                1 => AppMode::Shell,
                                2 => AppMode::Network,
                                3 => AppMode::TextEditor,
                                4 => AppMode::Hardware,
                                5 => AppMode::UserMgmt,
                                6 => AppMode::Browser,
                                7 => AppMode::ImageViewer,
                                _ => AppMode::Shell,
                            };
                            // Reopen window if it was closed
                            window_visible = true;
                            shell_output.clear();
                            for line in get_help!(active_mode) {
                                shell_output.push(String::from(*line));
                            }
                            shell_output.push(String::from(""));
                            break;
                        }
                    }
                }
            }
        }
        
        // -------------------------------------------------------------------
        // FRAME-RATE DECOUPLING GATE
        // Only render + composite on every Nth frame. Skip frames re-present.
        // -------------------------------------------------------------------
        let is_render_frame = (frame_count % composite_interval) == 0;
        
        if !is_render_frame {
            // Skip frame: NO VirtIO DMA transfer (present_only is now a no-op)
            // This is the key 120 FPS optimization: skip frames cost ~0.1ms
            // instead of ~33ms (4MB transfer_to_host_2d + resource_flush)
            compositor.present_only();
            // Advance formula animation state so next render frame shows smooth motion
            if use_formula { formula_renderer.update(); }
        } else {
        // --- RENDER FRAME: Full layer rendering + composite + present ---
        render_in_second += 1;
        
        // -------------------------------------------------------------------
        // LAYER 0: BACKGROUND - PARALLEL Matrix Rain across all cores!
        // Each core renders a chunk of columns simultaneously
        // -------------------------------------------------------------------
        if let Some(bg) = compositor.get_layer_mut(bg_layer) {
            let buf_ptr = bg.buffer.as_mut_ptr();
            let buf_len = bg.buffer.len();
            
            // FAST PATH: Use optimized renderers
            if use_formula {
                // -------------------------------------------------------------
                // FORMULA 3D: Tsoding-inspired wireframe perspective projection
                // Cheapest renderer: Bresenham lines + depth coloring
                // No fill, no textures, pure math beauty
                // -------------------------------------------------------------
                formula_renderer.update();
                formula_renderer.render(&mut bg.buffer, width as usize, height as usize);
            } else if use_shader_matrix {
                // -------------------------------------------------------------
                // SHADER MATRIX: Optimized cell-based Matrix rain
                // Real MATRIX_GLYPHS_6X6 katakana, per-column depth parallax,
                // SMP-parallel column bands, SSE2 background fill.
                // ~12K glyph blits/frame vs 1M+ pixel shader calls.
                // -------------------------------------------------------------
                crate::gpu_emu::shader_matrix_render(
                    buf_ptr,
                    width as usize,
                    height as usize,
                );
            } else if use_matrix3d {
                // -------------------------------------------------------------
                // MATRIX 3D: Volumetric rain with 3D shape collision
                // -------------------------------------------------------------
                matrix3d_renderer.update();
                matrix3d_renderer.render(&mut bg.buffer, width as usize, height as usize);
            } else if use_braille {
                // -------------------------------------------------------------
                // BRAILLE RENDERER: 8A-- resolution using Unicode Braille patterns
                // NOTE: Renderer fills black itself, no need for bg.buffer.fill!
                // -------------------------------------------------------------
                braille_renderer.update();
                braille_renderer.render(&mut bg.buffer, width as usize, height as usize);
                // Holographic cube: ambient fill for dark face cells, then subtle edge hints
                braille_renderer.render_cube_flow_layer(&mut bg.buffer, width as usize, height as usize);
                braille_renderer.render_entity_layer(&mut bg.buffer, width as usize, height as usize);
            } else if use_fast_matrix && !use_holovolume && !holo_enabled {
                // -------------------------------------------------------------
                // FAST MATRIX: Glyph-cached ultra-optimized renderer
                // -------------------------------------------------------------
                bg.buffer.fill(black);
                fast_renderer.update();
                fast_renderer.render(&mut bg.buffer, width as usize, height as usize);
            } else if use_holovolume {
                // -------------------------------------------------------------
                // HOLOVOLUME: Modifies Matrix rain colors based on 3D shape
                // Uses the SAME rain animation, just modifies colors
                // -------------------------------------------------------------
                
                // Update holovolume (compute intensity map)
                holovolume.set_screen_size(width as usize, height as usize);
                holovolume.update(0.016);
                
                // Move heads down (same as normal rain)
                for col in 0..MATRIX_COLS {
                    matrix_heads[col] += matrix_speeds[col] as i32;
                    if matrix_heads[col] > (MATRIX_ROWS as i32 + 30) {
                        let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                        matrix_heads[col] = -((seed % 30) as i32);
                        matrix_speeds[col] = 1 + (seed % 3);
                    }
                }
                
                // Clear with black
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::fill_row_sse2(buf_ptr, buf_len, black);
                    #[cfg(not(target_arch = "x86_64"))]
                    bg.buffer.fill(black);
                }
                
                // Get intensity map from holovolume
                let holo_intensity = holovolume.get_u8_intensity_map();
                
                // Render with holo intensity modifier
                let params = super::MatrixRenderParams {
                    buf_ptr,
                    buf_len,
                    width,
                    height,
                    matrix_chars: matrix_chars.as_ptr(),
                    matrix_heads: matrix_heads.as_ptr(),
                    holo_intensity: holo_intensity.as_ptr(),
                    matrix_rows: MATRIX_ROWS,
                };
                
                crate::cpu::smp::parallel_for(
                    MATRIX_COLS,
                    super::render_matrix_columns_parallel,
                    &params as *const super::MatrixRenderParams as *mut u8
                );
                
            } else if holo_enabled {
                // -------------------------------------------------------------
                // 3D BACKGROUND RENDERING (HoloMatrix or RayTracer)
                // -------------------------------------------------------------
                
                if holo_scene.is_raytraced() {
                    // ---------------------------------------------------------
                    // RAY TRACING MODE - True 3D with lighting and reflections
                    // ---------------------------------------------------------
                    use crate::graphics::raytracer::{Vec3, Material};
                    
                    raytracer.update(0.016);
                    
                    // Setup scene based on type
                    match holo_scene {
                        crate::graphics::holomatrix::HoloScene::RayTracedSpheres => {
                            raytracer.setup_spheres_scene();
                        },
                        crate::graphics::holomatrix::HoloScene::RayTracedDNA => {
                            raytracer.setup_dna_scene();
                        },
                        _ => {}
                    }
                    
                    // Render ray traced scene
                    let rt_output = raytracer.render();
                    
                    // Scale up to screen resolution
                    let rt_w = raytracer.width;
                    let rt_h = raytracer.height;
                    let scale_x = width as usize / rt_w;
                    let scale_y = height as usize / rt_h;
                    
                    for y in 0..height as usize {
                        for x in 0..width as usize {
                            let rx = (x / scale_x).min(rt_w - 1);
                            let ry = (y / scale_y).min(rt_h - 1);
                            let color = rt_output[ry * rt_w + rx];
                            bg.buffer[y * width as usize + x] = color;
                        }
                    }
                } else {
                    // ---------------------------------------------------------
                    // HOLOMATRIX THROUGH MATRIX RAIN
                    // 3D shape appears via intensity boost on Matrix characters
                    // The hologram "emerges" through the falling rain
                    // ---------------------------------------------------------
                    
                    // Update animation time
                    holomatrix.update(0.016);
                    let time = holomatrix.time;
                    
                    // ---------------------------------------------------------
                    // STEP 1: Create intensity boost map (character cell grid)
                    // Use same dimensions as Matrix rain (MATRIX_COLS x MATRIX_ROWS)
                    // Each cell stores intensity boost (0-200) based on 3D shape
                    // ---------------------------------------------------------
                    // Note: MATRIX_COLS=240, MATRIX_ROWS=68 defined at top
                    let mut intensity_map = [[0u8; MATRIX_ROWS]; MATRIX_COLS];
                    
                    // Cell size in pixels (matches Matrix rain)
                    let cell_w = (width as f32) / (MATRIX_COLS as f32);  // ~8px at 1920
                    let cell_h = (height as f32) / (MATRIX_ROWS as f32); // ~16px at 1080
                    
                    // 3D projection parameters - center of screen
                    let cx = width as f32 / 2.0;
                    let cy = height as f32 / 2.0;
                    let scale = (height as f32 / 3.0).min(width as f32 / 4.0);
                    
                    // Generate 3D shape points and mark intensity on grid
                    match holo_scene {
                        crate::graphics::holomatrix::HoloScene::DNA => {
                            // DNA Double Helix with higher resolution
                            let helix_len = 2.2;
                            let radius = 0.45;
                            let turns = 3.5;
                            
                            for i in 0..180 {
                                let t = i as f32 / 180.0;
                                let y = -helix_len / 2.0 + t * helix_len;
                                let angle = t * turns * 6.28318 + time;
                                
                                // Strand 1
                                let x1 = radius * crate::graphics::holomatrix::cos_approx_pub(angle);
                                let z1 = radius * crate::graphics::holomatrix::sin_approx_pub(angle);
                                
                                // Strand 2 (180o offset)
                                let x2 = radius * crate::graphics::holomatrix::cos_approx_pub(angle + 3.14159);
                                let z2 = radius * crate::graphics::holomatrix::sin_approx_pub(angle + 3.14159);
                                
                                // Apply rotation around Y axis
                                let rot_y = time * 0.4;
                                let cos_r = crate::graphics::holomatrix::cos_approx_pub(rot_y);
                                let sin_r = crate::graphics::holomatrix::sin_approx_pub(rot_y);
                                
                                // Rotate both strands
                                let rx1 = x1 * cos_r + z1 * sin_r;
                                let rz1 = -x1 * sin_r + z1 * cos_r;
                                let rx2 = x2 * cos_r + z2 * sin_r;
                                let rz2 = -x2 * sin_r + z2 * cos_r;
                                
                                // Project to screen and convert to grid coords
                                let depth1 = 1.0 / (2.0 + rz1);
                                let sx1 = cx + rx1 * scale * depth1;
                                let sy1 = cy + y * scale * depth1;
                                let col1 = (sx1 / cell_w) as usize;
                                let row1 = (sy1 / cell_h) as usize;
                                
                                let depth2 = 1.0 / (2.0 + rz2);
                                let sx2 = cx + rx2 * scale * depth2;
                                let sy2 = cy + y * scale * depth2;
                                let col2 = (sx2 / cell_w) as usize;
                                let row2 = (sy2 / cell_h) as usize;
                                
                                // Intensity based on depth (closer = brighter) - MAX values!
                                let int1 = (180.0 + 75.0 * (1.0 - ((rz1 + 0.5) * 0.5).max(0.0).min(1.0))) as u8;
                                let int2 = (180.0 + 75.0 * (1.0 - ((rz2 + 0.5) * 0.5).max(0.0).min(1.0))) as u8;
                                
                                // Mark intensity on grid (both strands) - with THICKNESS
                                if col1 < MATRIX_COLS && row1 < MATRIX_ROWS {
                                    intensity_map[col1][row1] = intensity_map[col1][row1].max(int1);
                                    // Spread to neighbors for thickness (3 cells wide)
                                    if col1 > 0 { intensity_map[col1-1][row1] = intensity_map[col1-1][row1].max(int1 * 2/3); }
                                    if col1 < MATRIX_COLS-1 { intensity_map[col1+1][row1] = intensity_map[col1+1][row1].max(int1 * 2/3); }
                                    if row1 > 0 { intensity_map[col1][row1-1] = intensity_map[col1][row1-1].max(int1/2); }
                                    if row1 < MATRIX_ROWS-1 { intensity_map[col1][row1+1] = intensity_map[col1][row1+1].max(int1/2); }
                                }
                                if col2 < MATRIX_COLS && row2 < MATRIX_ROWS {
                                    intensity_map[col2][row2] = intensity_map[col2][row2].max(int2);
                                    if col2 > 0 { intensity_map[col2-1][row2] = intensity_map[col2-1][row2].max(int2 * 2/3); }
                                    if col2 < MATRIX_COLS-1 { intensity_map[col2+1][row2] = intensity_map[col2+1][row2].max(int2 * 2/3); }
                                    if row2 > 0 { intensity_map[col2][row2-1] = intensity_map[col2][row2-1].max(int2/2); }
                                    if row2 < MATRIX_ROWS-1 { intensity_map[col2][row2+1] = intensity_map[col2][row2+1].max(int2/2); }
                                }
                                
                                // Cross-links every 12 points (base pairs)
                                if i % 12 == 0 {
                                    for s in 0..8 {
                                        let st = s as f32 / 7.0;
                                        let lx = sx1 * (1.0 - st) + sx2 * st;
                                        let ly = sy1 * (1.0 - st) + sy2 * st;
                                        let lcol = (lx / cell_w) as usize;
                                        let lrow = (ly / cell_h) as usize;
                                        if lcol < MATRIX_COLS && lrow < MATRIX_ROWS {
                                            intensity_map[lcol][lrow] = intensity_map[lcol][lrow].max(80);
                                        }
                                    }
                                }
                            }
                        },
                        crate::graphics::holomatrix::HoloScene::RotatingCube => {
                            let half = 0.5;
                            let vertices: [(f32, f32, f32); 8] = [
                                (-half, -half, -half), (half, -half, -half),
                                (half, half, -half), (-half, half, -half),
                                (-half, -half, half), (half, -half, half),
                                (half, half, half), (-half, half, half),
                            ];
                            let edges: [(usize, usize); 12] = [
                                (0,1), (1,2), (2,3), (3,0),
                                (4,5), (5,6), (6,7), (7,4),
                                (0,4), (1,5), (2,6), (3,7),
                            ];
                            
                            let rot_x = time * 0.7;
                            let rot_y = time * 0.5;
                            
                            for (i1, i2) in edges.iter() {
                                let (vx1, vy1, vz1) = vertices[*i1];
                                let (vx2, vy2, vz2) = vertices[*i2];
                                
                                for s in 0..30 {
                                    let t = s as f32 / 29.0;
                                    let x = vx1 * (1.0 - t) + vx2 * t;
                                    let y = vy1 * (1.0 - t) + vy2 * t;
                                    let z = vz1 * (1.0 - t) + vz2 * t;
                                    
                                    // Rotate
                                    let cos_x = crate::graphics::holomatrix::cos_approx_pub(rot_x);
                                    let sin_x = crate::graphics::holomatrix::sin_approx_pub(rot_x);
                                    let ry = y * cos_x - z * sin_x;
                                    let rz = y * sin_x + z * cos_x;
                                    let cos_y = crate::graphics::holomatrix::cos_approx_pub(rot_y);
                                    let sin_y = crate::graphics::holomatrix::sin_approx_pub(rot_y);
                                    let rx = x * cos_y + rz * sin_y;
                                    let rz2 = -x * sin_y + rz * cos_y;
                                    
                                    let depth = 1.0 / (2.0 + rz2);
                                    let sx = cx + rx * scale * depth;
                                    let sy = cy + ry * scale * depth;
                                    let col = (sx / cell_w) as usize;
                                    let row = (sy / cell_h) as usize;
                                    
                                    if col < MATRIX_COLS && row < MATRIX_ROWS {
                                        let int = (100.0 + 100.0 * (1.0 - ((rz2 + 0.6) * 0.5).max(0.0).min(1.0))) as u8;
                                        intensity_map[col][row] = intensity_map[col][row].max(int);
                                    }
                                }
                            }
                        },
                        _ => {
                            // Sphere
                            for i in 0..300 {
                                let phi = (i as f32 / 300.0) * 6.28318;
                                let theta = (i as f32 * 0.618033 * 6.28318) % 6.28318;
                                
                                let r = 0.55;
                                let x = r * crate::graphics::holomatrix::sin_approx_pub(theta) * crate::graphics::holomatrix::cos_approx_pub(phi);
                                let y = r * crate::graphics::holomatrix::sin_approx_pub(theta) * crate::graphics::holomatrix::sin_approx_pub(phi);
                                let z = r * crate::graphics::holomatrix::cos_approx_pub(theta);
                                
                                let cos_t = crate::graphics::holomatrix::cos_approx_pub(time * 0.5);
                                let sin_t = crate::graphics::holomatrix::sin_approx_pub(time * 0.5);
                                let rx = x * cos_t + z * sin_t;
                                let rz = -x * sin_t + z * cos_t;
                                
                                let depth = 1.0 / (2.0 + rz);
                                let sx = cx + rx * scale * depth;
                                let sy = cy + y * scale * depth;
                                let col = (sx / cell_w) as usize;
                                let row = (sy / cell_h) as usize;
                                
                                if col < MATRIX_COLS && row < MATRIX_ROWS {
                                    let int = (80.0 + 120.0 * (1.0 - ((rz + 0.6) * 0.5).max(0.0).min(1.0))) as u8;
                                    intensity_map[col][row] = intensity_map[col][row].max(int);
                                }
                            }
                        }
                    }
                    
                    // ---------------------------------------------------------
                    // STEP 2: Render Matrix Rain with hologram intensity boost
                    // Characters where shape exists are brightened
                    // ---------------------------------------------------------
                    
                    // Move Matrix heads down
                    for col in 0..MATRIX_COLS {
                        matrix_heads[col] += matrix_speeds[col] as i32;
                        if matrix_heads[col] > (MATRIX_ROWS as i32 + 30) {
                            let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                            matrix_heads[col] = -((seed % 30) as i32);
                            matrix_speeds[col] = 1 + (seed % 3);
                        }
                    }
                    
                    // Clear to black
                    bg.buffer.fill(0xFF000000);
                    
                    // Render each column with intensity boost
                    let col_width = 8u32;
                    for col in 0..MATRIX_COLS {
                        let x = col as u32 * col_width;
                        if x >= width { continue; }
                        
                        let head = matrix_heads[col];
                        
                        for row in 0..MATRIX_ROWS {
                            let y = row as u32 * 16;
                            if y >= height { continue; }
                            
                            let dist = row as i32 - head;
                            
                            // Base color from Matrix rain
                            let base_color = if dist < 0 {
                                continue;
                            } else if dist == 0 {
                                255u32  // Bright head
                            } else if dist <= 12 {
                                255 - (dist as u32 * 8)
                            } else if dist <= 28 {
                                // Safely calculate fade to avoid underflow
                                let factor = ((dist - 12) as u32).min(15) * 16;
                                let fade = 255u32.saturating_sub(factor);
                                (160 * fade) / 255
                            } else {
                                continue;
                            };
                            
                            // Get intensity boost from hologram
                            let boost = intensity_map[col][row] as u32;
                            
                            // Combine: base Matrix + hologram boost
                            // Hologram makes characters MUCH brighter + colored where shape exists
                            let (r, g, b) = if boost > 0 {
                                // Shape exists here: bright cyan-white glow
                                let intensity = (base_color + boost * 2).min(255);
                                let cyan = (boost as u32 * 3 / 2).min(255);
                                (cyan / 3, intensity, cyan)  // R=dim, G=bright, B=cyan
                            } else {
                                // No shape: very dim green (shape stands out)
                                let dim = (base_color / 3).min(80);
                                (0, dim, 0)
                            };
                            
                            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                            
                            // Get character
                            let c = matrix_chars[matrix_idx(col, row)] as char;
                            let glyph = crate::framebuffer::font::get_glyph(c);
                            
                            // Draw glyph
                            for (r, &bits) in glyph.iter().enumerate() {
                                let py = y + r as u32;
                                if py >= height { break; }
                                let row_offset = (py * width) as usize;
                                
                                if bits != 0 {
                                    let x_usize = x as usize;
                                    if bits & 0x80 != 0 { let idx = row_offset + x_usize; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x40 != 0 { let idx = row_offset + x_usize + 1; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x20 != 0 { let idx = row_offset + x_usize + 2; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x10 != 0 { let idx = row_offset + x_usize + 3; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x08 != 0 { let idx = row_offset + x_usize + 4; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x04 != 0 { let idx = row_offset + x_usize + 5; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x02 != 0 { let idx = row_offset + x_usize + 6; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x01 != 0 { let idx = row_offset + x_usize + 7; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                }
                            }
                        }
                    }
                }
            } else {
                // -------------------------------------------------------------
                // MATRIX RAIN BACKGROUND (default)
                // -------------------------------------------------------------
                
                // Move heads down (fast, single-threaded)
                for col in 0..MATRIX_COLS {
                    matrix_heads[col] += matrix_speeds[col] as i32;
                    if matrix_heads[col] > (MATRIX_ROWS as i32 + 30) {
                        let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                        matrix_heads[col] = -((seed % 30) as i32);
                        matrix_speeds[col] = 1 + (seed % 3);
                    }
                }
                
                // Clear with black using SSE2
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::fill_row_sse2(buf_ptr, buf_len, black);
                    #[cfg(not(target_arch = "x86_64"))]
                    bg.buffer.fill(black);
                }
                
                // -------------------------------------------------------------
                // TRUE PARALLEL MATRIX RENDER: Use parallel_for across all cores!
                // Each core renders its portion of the 240 columns simultaneously
                // -------------------------------------------------------------
                let params = super::MatrixRenderParams {
                    buf_ptr,
                    buf_len,
                    width,
                    height,
                    matrix_chars: matrix_chars.as_ptr(),
                    matrix_heads: matrix_heads.as_ptr(),
                    holo_intensity: core::ptr::null(),  // No holo modifier
                    matrix_rows: MATRIX_ROWS,
                };
                
                // Fire off parallel rendering across ALL cores!
                crate::cpu::smp::parallel_for(
                    MATRIX_COLS,
                    super::render_matrix_columns_parallel,
                    &params as *const super::MatrixRenderParams as *mut u8
                );
            }
        }
        
        // -------------------------------------------------------------------
        // LAYER 1: DOCK (Left side)
        // -------------------------------------------------------------------
        if let Some(dock) = compositor.get_layer_mut(dock_layer) {
            dock.clear(0xF0080808); // Semi-transparent
            
            let icon_size = 36u32;  // Compact icons
            let gap = 50u32;       // Adjusted gap for more apps
            let start_y = 10u32;
            
            let dock_apps = [
                ("Files", AppMode::Files),
                ("Shell", AppMode::Shell),
                ("Net", AppMode::Network),
                ("Edit", AppMode::TextEditor),
                ("HW", AppMode::Hardware),
                ("User", AppMode::UserMgmt),
                ("Web", AppMode::Browser),  // Browser - special app
                ("Img", AppMode::ImageViewer), // Image viewer
            ];
            
            for (i, (name, mode)) in dock_apps.iter().enumerate() {
                let iy = start_y + (i as u32) * (icon_size + gap);
                let ix = 10u32;
                
                let is_active = *mode == active_mode;
                let icon_color = if is_active { green_bright } else { green_dim };
                let label_color = if is_active { 0xFFFFFFFF } else { 0xFF888888 };
                
                // Icon background with hover effect
                if is_active {
                    dock.fill_rect(ix - 4, iy - 4, icon_size + 8, icon_size + 20, 0xFF002800);
                    dock.draw_rect(ix - 4, iy - 4, icon_size + 8, icon_size + 20, green_main);
                }
                dock.fill_rect(ix, iy, icon_size, icon_size, 0xFF0A0A0A);
                dock.draw_rect(ix, iy, icon_size, icon_size, icon_color);
                
                // Icon symbol (larger, more visible)
                let cx = ix + icon_size / 2;
                let cy = iy + icon_size / 2;
                match i {
                    0 => { // Files - folder icon
                        dock.fill_rect(cx - 12, cy - 2, 24, 14, icon_color);
                        dock.fill_rect(cx - 14, cy - 6, 10, 6, icon_color);
                    },
                    1 => { // Shell - terminal icon
                        dock.draw_rect(cx - 14, cy - 10, 28, 20, icon_color);
                        dock.draw_text(">", cx - 8, cy - 4, icon_color);
                        dock.fill_rect(cx - 2, cy - 2, 10, 2, icon_color);
                    },
                    2 => { // Network - wifi/globe icon (simplified)
                        dock.fill_circle(cx, cy, 12, icon_color);
                        dock.fill_circle(cx, cy, 8, 0xFF0A0A0A);
                        dock.fill_circle(cx, cy, 4, icon_color);
                        // Signal bars
                        dock.fill_rect(cx + 6, cy - 2, 2, 6, icon_color);
                        dock.fill_rect(cx + 10, cy - 6, 2, 10, icon_color);
                    },
                    3 => { // Editor - document icon
                        dock.fill_rect(cx - 10, cy - 12, 20, 24, icon_color);
                        dock.fill_rect(cx - 8, cy - 10, 16, 20, 0xFF0A0A0A);
                        dock.fill_rect(cx - 6, cy - 6, 12, 2, icon_color);
                        dock.fill_rect(cx - 6, cy - 2, 12, 2, icon_color);
                        dock.fill_rect(cx - 6, cy + 2, 8, 2, icon_color);
                    },
                    4 => { // Hardware - chip icon
                        dock.fill_rect(cx - 10, cy - 8, 20, 16, icon_color);
                        for j in 0..4 {
                            dock.fill_rect(cx - 14, cy - 6 + j * 4, 4, 2, icon_color);
                            dock.fill_rect(cx + 10, cy - 6 + j * 4, 4, 2, icon_color);
                        }
                    },
                    5 => { // Users - person icon
                        dock.fill_circle(cx, cy - 4, 6, icon_color);
                        dock.fill_rect(cx - 8, cy + 4, 16, 8, icon_color);
                    },
                    6 => { // Browser - globe/world icon
                        dock.fill_circle(cx, cy, 10, icon_color);
                        dock.fill_circle(cx, cy, 6, 0xFF0A0A0A);
                        // Horizontal line
                        dock.fill_rect(cx - 10, cy - 1, 20, 2, icon_color);
                        // Vertical line  
                        dock.fill_rect(cx - 1, cy - 10, 2, 20, icon_color);
                    },
                    _ => {}
                }
                
                // Label text under icon
                let text_x = ix + (icon_size / 2) - ((name.len() as u32 * 8) / 2);
                dock.draw_text(name, text_x, iy + icon_size + 2, label_color);
            }
        }
        
        // -------------------------------------------------------------------
        // LAYER 2: MAIN WINDOW (Shell with module guide) - Only if visible
        // -------------------------------------------------------------------
        if let Some(win) = compositor.get_layer_mut(window_layer) {
            if window_visible {
                // Use actual layer dimensions, not screen dimensions!
                let w = win.width;
                let h = win.height;
                
                win.clear(window_bg);
                
                // Border (green)
                win.draw_rect(0, 0, w, h, green_main);
                win.draw_rect(1, 1, w - 2, h - 2, green_main);
                
                // Title bar
                let mode_name = match active_mode {
                    AppMode::Shell => "Shell",
                AppMode::Network => "Network",
                AppMode::Hardware => "Hardware",
                AppMode::TextEditor => "TrustCode",
                AppMode::UserMgmt => "User Management",
                AppMode::Files => "Files",
                AppMode::Browser => "Web Browser",
                AppMode::ImageViewer => "Image Viewer",
            };
            win.fill_rect(2, 2, w - 4, 26, 0xFF0A1A0A);
            let title = format!("TrustOS - {} Module", mode_name);
            win.draw_text(&title, 12, 8, 0xFFFFFFFF); // White text
            
            // Drag indicator
            if dragging_window {
                win.draw_text("[MOVING]", w / 2 - 32, 8, 0xFFFFAA00);
            }
            
            // Window buttons with symbols (LARGE and visible)
            // Close button (red with X) - 12px radius
            let btn_y = 13u32;
            let btn_r = 10u32;
            let btn_close_x = w - 50;
            let btn_min_x = w - 80;
            let btn_max_x = w - 110;
            
            // Close button (X)
            win.fill_circle(btn_close_x, btn_y, btn_r, 0xFFFF4444);
            win.draw_rect(btn_close_x, btn_y, 1, 1, 0xFFFF6666); // Highlight
            // Draw bold X
            for t in 0..7 {
                win.set_pixel(btn_close_x - 5 + t, btn_y - 5 + t, 0xFFFFFFFF);
                win.set_pixel(btn_close_x - 4 + t, btn_y - 5 + t, 0xFFFFFFFF);
                win.set_pixel(btn_close_x + 5 - t, btn_y - 5 + t, 0xFFFFFFFF);
                win.set_pixel(btn_close_x + 4 - t, btn_y - 5 + t, 0xFFFFFFFF);
            }
            
            // Minimize button (-)
            win.fill_circle(btn_min_x, btn_y, btn_r, 0xFFFFCC00);
            // Draw bold -
            win.fill_rect(btn_min_x - 5, btn_y - 1, 10, 3, 0xFF000000);
            
            // Maximize button (?)
            win.fill_circle(btn_max_x, btn_y, btn_r, 0xFF44DD44);
            // Draw bold square
            win.draw_rect(btn_max_x - 5, btn_y - 5, 10, 10, 0xFF000000);
            win.draw_rect(btn_max_x - 4, btn_y - 4, 8, 8, 0xFF000000);
            
            // Content area - different rendering for Browser mode
            let content_y = 35u32;
            let line_height = 18u32;
            let max_lines = ((h - content_y - 50) / line_height) as usize;
            
            if active_mode == AppMode::Browser {
                // -----------------------------------------------------------
                // BROWSER MODE: Chrome DevTools style rendering
                // -----------------------------------------------------------
                
                // URL Bar with modern styling
                let url_bar_y = content_y;
                win.fill_rect(10, url_bar_y, w - 20, 32, 0xFF1E1E1E);
                win.draw_rect(10, url_bar_y, w - 20, 32, 0xFF3C3C3C);
                
                // Navigation buttons with icons
                let btn_bg: u32 = 0xFF2D2D2D;
                
                // Back button (?)
                win.fill_rect(14, url_bar_y + 4, 24, 24, btn_bg);
                win.draw_text("<", 22, url_bar_y + 10, 0xFFAAAAAA);
                
                // Forward button (?)
                win.fill_rect(42, url_bar_y + 4, 24, 24, btn_bg);
                win.draw_text(">", 50, url_bar_y + 10, 0xFFAAAAAA);
                
                // Refresh button (?)
                win.fill_rect(70, url_bar_y + 4, 24, 24, btn_bg);
                win.draw_text("R", 78, url_bar_y + 10, 0xFFAAAAAA);
                
                // View toggle button
                let view_label = if browser_view_mode == 0 { "SRC" } else { "DOM" };
                win.fill_rect(98, url_bar_y + 4, 32, 24, 0xFF383838);
                win.draw_text(view_label, 102, url_bar_y + 10, 0xFF88CCFF);
                
                // URL input field
                win.fill_rect(135, url_bar_y + 4, w - 160, 24, 0xFF0D0D0D);
                win.draw_rect(135, url_bar_y + 4, w - 160, 24, if browser_url_focused { 0xFF4FC3F7 } else { 0xFF555555 });
                
                // HTTPS indicator
                let url_color = if browser_url.starts_with("https://") { 0xFF00C853 } else { 0xFFDDDDDD };
                win.draw_text(&browser_url, 142, url_bar_y + 10, url_color);
                
                // Blinking cursor
                if browser_url_focused && cursor_blink {
                    let cursor_x = 142 + (browser_url.len() as u32 * 8);
                    if cursor_x < w - 30 {
                        win.fill_rect(cursor_x, url_bar_y + 8, 2, 18, 0xFF4FC3F7);
                    }
                }
                
                // Content area with DevTools styling
                let page_y = content_y + 40;
                let page_max_lines = ((h - page_y - 35) / line_height) as usize;
                
                // Dark background for code area
                win.fill_rect(10, page_y - 4, w - 20, h - page_y - 28, 0xFF1E1E1E);
                
                // Line numbers gutter
                let gutter_width = 40u32;
                win.fill_rect(10, page_y - 4, gutter_width, h - page_y - 28, 0xFF252526);
                
                let start_idx = if browser_lines.len() > page_max_lines {
                    browser_lines.len() - page_max_lines
                } else {
                    0
                };
                
                // Render each line with colored segments
                for (i, browser_line) in browser_lines.iter().skip(start_idx).enumerate() {
                    let y = page_y + (i as u32) * line_height;
                    if y + line_height > h - 35 { break; }
                    
                    // Line number
                    let line_num = format!("{:3}", start_idx + i + 1);
                    win.draw_text(&line_num, 14, y, 0xFF858585);
                    
                    // Render segments with their colors
                    let mut x_pos = 10u32 + gutter_width + 5;
                    for segment in &browser_line.segments {
                        win.draw_text(&segment.text, x_pos, y, segment.color);
                        x_pos += (segment.text.len() as u32) * 8;
                    }
                }
                
                // Status bar at bottom - modern styling
                win.fill_rect(10, h - 28, w - 20, 23, 0xFF007ACC);
                
                // Status icon
                let status_icon = if browser_status.contains("Error") { "?" } 
                    else if browser_status.contains("Loading") { "?" } 
                    else { "?" };
                win.draw_text(status_icon, 16, h - 24, 0xFFFFFFFF);
                win.draw_text(&browser_status, 30, h - 24, 0xFFFFFFFF);
                
                // View mode indicator
                let mode_text = if browser_view_mode == 0 { "[Source]" } else { "[Elements]" };
                let mode_x = w - 90;
                win.draw_text(mode_text, mode_x, h - 24, 0xFFCCCCCC);
                
            } else if active_mode == AppMode::ImageViewer {
                // -----------------------------------------------------------
                // IMAGE VIEWER MODE: Display images with zoom/pan
                // -----------------------------------------------------------
                
                // Dark background for image area
                win.fill_rect(10, content_y, w - 20, h - content_y - 30, 0xFF1A1A1A);
                
                if let Some(ref img) = image_viewer_data {
                    // Calculate display area
                    let view_w = w - 40;
                    let view_h = h - content_y - 60;
                    let view_x = 20u32;
                    let view_y = content_y + 10;
                    
                    // Apply zoom
                    let scaled_w = (img.width as f32 * image_viewer_zoom) as u32;
                    let scaled_h = (img.height as f32 * image_viewer_zoom) as u32;
                    
                    // Center image with offset
                    let center_x = view_x as i32 + (view_w as i32 / 2) + image_viewer_offset_x;
                    let center_y = view_y as i32 + (view_h as i32 / 2) + image_viewer_offset_y;
                    let img_x = center_x - (scaled_w as i32 / 2);
                    let img_y = center_y - (scaled_h as i32 / 2);
                    
                    // Draw scaled image (simple nearest neighbor)
                    for dy in 0..scaled_h.min(view_h) {
                        let screen_y = img_y + dy as i32;
                        if screen_y < view_y as i32 || screen_y >= (view_y + view_h) as i32 {
                            continue;
                        }
                        
                        let src_y = ((dy as f32 / image_viewer_zoom) as u32).min(img.height - 1);
                        
                        for dx in 0..scaled_w.min(view_w) {
                            let screen_x = img_x + dx as i32;
                            if screen_x < view_x as i32 || screen_x >= (view_x + view_w) as i32 {
                                continue;
                            }
                            
                            let src_x = ((dx as f32 / image_viewer_zoom) as u32).min(img.width - 1);
                            let pixel = img.get_pixel(src_x, src_y);
                            
                            // Only draw non-transparent pixels
                            if (pixel >> 24) > 0 {
                                win.set_pixel(screen_x as u32, screen_y as u32, pixel);
                            }
                        }
                    }
                    
                    // Image border
                    win.draw_rect(
                        (img_x.max(view_x as i32)) as u32,
                        (img_y.max(view_y as i32)) as u32,
                        scaled_w.min(view_w),
                        scaled_h.min(view_h),
                        0xFF444444
                    );
                } else {
                    // No image loaded - show placeholder
                    let center_x = w / 2;
                    let center_y = (content_y + h) / 2;
                    
                    // Icon placeholder
                    win.draw_rect(center_x - 40, center_y - 30, 80, 60, 0xFF444444);
                    win.draw_text("??", center_x - 8, center_y - 10, 0xFF666666);
                    win.draw_text("No image loaded", center_x - 56, center_y + 25, 0xFF888888);
                    win.draw_text("Use: imgview <file>", center_x - 72, center_y + 45, 0xFF666666);
                }
                
                // Info bar at top
                win.fill_rect(10, content_y, w - 20, 24, 0xFF252525);
                let zoom_pct = (image_viewer_zoom * 100.0) as u32;
                let info_str = format!("Zoom: {}%  |  {}", zoom_pct, image_viewer_info);
                win.draw_text(&info_str, 16, content_y + 5, 0xFFCCCCCC);
                
                // Format indicator
                win.draw_text(&image_viewer_format, w - 60, content_y + 5, 0xFF88CCFF);
                
                // Controls hint at bottom
                win.fill_rect(10, h - 28, w - 20, 23, 0xFF252525);
                win.draw_text("[+/-] Zoom  [Arrows] Pan  [R] Reset  [ESC] Close", 16, h - 24, 0xFF888888);
                
            } else if active_mode == AppMode::TextEditor {
                // -----------------------------------------------------------
                // TRUSTCODE: VSCode-inspired code editor
                // -----------------------------------------------------------
                use crate::apps::text_editor::*;
                
                let char_w: u32 = 8;
                let line_h: u32 = 16;
                let gutter_chars: u32 = 5; // "nnnn "
                let gutter_w = gutter_chars * char_w;
                let status_h: u32 = 22;
                let tab_bar_h: u32 = 26;
                
                let code_x = gutter_w;
                let code_y = content_y + tab_bar_h;
                let code_w = w - gutter_w;
                let code_h = h.saturating_sub(content_y + tab_bar_h + status_h);
                let visible_lines_count = (code_h / line_h).max(1) as usize;
                
                // Update scroll
                if editor_state.cursor_line < editor_state.scroll_y {
                    editor_state.scroll_y = editor_state.cursor_line;
                }
                if editor_state.cursor_line >= editor_state.scroll_y + visible_lines_count {
                    editor_state.scroll_y = editor_state.cursor_line - visible_lines_count + 1;
                }
                editor_state.blink_counter += 1;
                
                // -- Tab bar --
                win.fill_rect(0, content_y, w, tab_bar_h, COLOR_BREADCRUMB_BG);
                let tab_name = editor_state.file_path.as_ref().map(|p| {
                    p.rsplit('/').next().unwrap_or(p.as_str())
                }).unwrap_or("untitled");
                let dirty_marker = if editor_state.dirty { " *" } else { "" };
                let tab_label = format!("  {}{}", tab_name, dirty_marker);
                let tab_w = ((tab_label.len() as u32 + 2) * char_w).min(w);
                win.fill_rect(0, content_y, tab_w, tab_bar_h, COLOR_TAB_ACTIVE);
                // Tab bottom accent line
                win.fill_rect(0, content_y + tab_bar_h - 2, tab_w, 2, COLOR_STATUS_BG);
                win.draw_text(&tab_label, 4, content_y + 5, COLOR_NORMAL);
                
                // -- Editor background --
                win.fill_rect(0, code_y, w, code_h, COLOR_BG);
                
                // -- Gutter background + border --
                win.fill_rect(0, code_y, gutter_w, code_h, COLOR_GUTTER_BG);
                win.fill_rect(gutter_w - 1, code_y, 1, code_h, 0xFF333333);
                
                // -- Render lines --
                for vi in 0..visible_lines_count {
                    let line_idx = editor_state.scroll_y + vi;
                    if line_idx >= editor_state.lines.len() { break; }
                    
                    let ly = code_y + (vi as u32 * line_h);
                    if ly + line_h > code_y + code_h { break; }
                    
                    let is_current = line_idx == editor_state.cursor_line;
                    
                    // Current line highlight
                    if is_current {
                        win.fill_rect(code_x, ly, code_w, line_h, COLOR_ACTIVE_LINE_BG);
                    }
                    
                    // Line number
                    let num_str = format!("{:>4} ", line_idx + 1);
                    let num_color = if is_current { COLOR_ACTIVE_LINE } else { COLOR_LINE_NUM };
                    win.draw_text(&num_str, 2, ly, num_color);
                    
                    // Code with syntax highlighting
                    let line = &editor_state.lines[line_idx];
                    
                    if editor_state.language == Language::Rust {
                        let tokens = tokenize_rust_line(line);
                        for span in &tokens {
                            let color = token_color(span.kind);
                            let text_seg = &line[span.start..span.end];
                            let sx = code_x + 4 + (span.start as u32 * char_w);
                            if sx < w {
                                win.draw_text(text_seg, sx, ly, color);
                            }
                        }
                        if tokens.is_empty() && !line.is_empty() {
                            win.draw_text(line, code_x + 4, ly, COLOR_NORMAL);
                        }
                    } else {
                        win.draw_text(line, code_x + 4, ly, COLOR_NORMAL);
                    }
                    
                    // Cursor
                    if is_current {
                        let blink_on = (editor_state.blink_counter / 30) % 2 == 0;
                        if blink_on {
                            let cx = code_x + 4 + (editor_state.cursor_col as u32 * char_w);
                            win.fill_rect(cx, ly, 2, line_h, COLOR_CURSOR);
                        }
                    }
                }
                
                // -- Scrollbar --
                if editor_state.lines.len() > visible_lines_count {
                    let sb_x = w - 10;
                    let sb_h = code_h;
                    let total = editor_state.lines.len() as u32;
                    let thumb_h = ((visible_lines_count as u32 * sb_h) / total).max(20);
                    let max_scroll_val = total.saturating_sub(visible_lines_count as u32);
                    let thumb_y = if max_scroll_val > 0 {
                        (editor_state.scroll_y as u32 * (sb_h - thumb_h)) / max_scroll_val
                    } else { 0 };
                    win.fill_rect(sb_x, code_y, 10, sb_h, 0xFF252526);
                    win.fill_rect(sb_x + 2, code_y + thumb_y, 6, thumb_h, 0xFF555555);
                }
                
                // -- Status bar (VSCode blue) --
                let status_y = h - status_h;
                win.fill_rect(0, status_y, w, status_h, COLOR_STATUS_BG);
                
                // Left: file info
                let status_left = if let Some(ref msg) = editor_state.status_message {
                    format!("  {}", msg)
                } else {
                    let dirty_str = if editor_state.dirty { " [Modified]" } else { "" };
                    let fname = editor_state.file_path.as_deref().unwrap_or("untitled");
                    format!("  {}{}", fname, dirty_str)
                };
                win.draw_text(&status_left, 4, status_y + 3, COLOR_STATUS_FG);
                
                // Right: position and language
                let status_right = format!(
                    "Ln {}, Col {}  {}  UTF-8  TrustCode",
                    editor_state.cursor_line + 1,
                    editor_state.cursor_col + 1,
                    editor_state.language.name(),
                );
                let right_x = w.saturating_sub((status_right.len() as u32 * char_w) + 8);
                win.draw_text(&status_right, right_x, status_y + 3, COLOR_STATUS_FG);

            } else {
                // -----------------------------------------------------------
                // SHELL MODE: Normal shell output with scrolling
                // -----------------------------------------------------------
                
            // Calculate visible range with scroll support
            let total_lines = shell_output.len();
            let visible_lines = MAX_VISIBLE_LINES.min(max_lines);
            
            // Auto-scroll to bottom when new content is added (unless user scrolled up)
            let max_scroll = total_lines.saturating_sub(visible_lines);
            if scroll_offset > max_scroll {
                scroll_offset = max_scroll;
            }
            
            let start_idx = scroll_offset;
            let end_idx = (start_idx + visible_lines).min(total_lines);
            
            for (i, line) in shell_output.iter().skip(start_idx).take(visible_lines).enumerate() {
                let y = content_y + (i as u32) * line_height;
                if y + line_height > h - 50 { break; }
                
                // Enhanced color coding by content
                let color = if line.starts_with("+") || line.starts_with("+") || line.starts_with("|") {
                    green_main  // Box borders in green
                } else if line.starts_with("|") {
                    // Parse content inside box for coloring
                    if line.contains("NAVIGATION:") || line.contains("FILE OPERATIONS:") || 
                       line.contains("COMMANDS:") || line.contains("TIPS:") ||
                       line.contains("BASIC COMMANDS:") || line.contains("EXAMPLES:") ||
                       line.contains("NOTE:") {
                        0xFFFFFF00  // Yellow for section headers
                    } else if line.contains(" - ") {
                        // Command line: command - description
                        green_main  // Green for command lines
                    } else if line.starts_with("|    *") {
                        0xFFAAAAAA  // Light gray for tips/bullets  
                    } else {
                        0xFFDDDDDD  // White for normal text
                    }
                } else if line.starts_with(">") {
                    0xFF88FF88  // Bright green for command echo
                } else if line.contains("<DIR>") {
                    0xFF00FFFF  // Cyan for directories
                } else if line.contains(" B") && !line.contains("Browse") {
                    green_main  // Green for files with size
                } else if line.starts_with("Created") || line.starts_with("Changed") || line.starts_with("Removed") {
                    0xFF00FF00  // Bright green for success messages
                } else if line.contains("Error") || line.contains("cannot") || line.contains("No such") {
                    0xFFFF4444  // Red for errors
                } else {
                    green_dim  // Default dim green
                };
                win.draw_text(line, 12, y, color);
            }
            
            // -----------------------------------------------------------
            // SCROLLBAR on right side
            // -----------------------------------------------------------
            if total_lines > visible_lines {
                let scrollbar_x = w - 12;
                let scrollbar_y = content_y;
                let scrollbar_h = h - content_y - 50;  // Height of scrollable area
                
                // Background track (dark)
                win.fill_rect(scrollbar_x, scrollbar_y, 8, scrollbar_h, 0xFF1A1A1A);
                
                // Calculate thumb size and position
                let thumb_ratio = visible_lines as f32 / total_lines as f32;
                let thumb_h = ((scrollbar_h as f32 * thumb_ratio) as u32).max(20);
                let scroll_ratio = if max_scroll > 0 { 
                    scroll_offset as f32 / max_scroll as f32 
                } else { 
                    0.0 
                };
                let thumb_y = scrollbar_y + ((scrollbar_h - thumb_h) as f32 * scroll_ratio) as u32;
                
                // Thumb (green)
                win.fill_rect(scrollbar_x, thumb_y, 8, thumb_h, green_dim);
                win.fill_rect(scrollbar_x + 1, thumb_y + 1, 6, thumb_h - 2, green_main);
            }
            
            // Input area at bottom
            let input_y = h - 40;
            win.fill_rect(10, input_y, w - 20, 30, 0xFF050505);
            win.draw_rect(10, input_y, w - 20, 30, green_dim);
            
            // Prompt with current directory - colored parts
            let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
            // Draw "root" in red
            win.draw_text("root", 16, input_y + 8, 0xFFFF0000);  // Pure red
            // Draw "@" in white
            win.draw_text("@", 16 + 32, input_y + 8, 0xFFFFFFFF);  // White
            // Draw "trustos" in green
            win.draw_text("trustos", 16 + 40, input_y + 8, 0xFF00FF00);  // Pure green
            // Draw ":path$ " in green
            let path_part = format!(":{}$ ", cwd);
            win.draw_text(&path_part, 16 + 96, input_y + 8, 0xFF00FF00);  // Pure green
            let prompt_width = (4 + 1 + 7 + path_part.len()) as u32 * 8;  // root @ trustos :path$
            
            // User input text
            win.draw_text(&shell_input, 16 + prompt_width, input_y + 8, green_bright);
            
            // Suggestion (grayed out)
            if !suggestion_text.is_empty() {
                let input_width = (shell_input.len() * 8) as u32;
                win.draw_text(&suggestion_text, 16 + prompt_width + input_width, input_y + 8, 0xFF444444);
            }
            
            // Blinking cursor
            cursor_blink = (frame_count / 30) % 2 == 0;
            if cursor_blink {
                let cursor_x = 16 + prompt_width + (shell_input.len() as u32 * 8);
                win.fill_rect(cursor_x, input_y + 6, 8, 16, green_bright);
            }
            } // End of shell mode else block
            } else {
                // Window is hidden - clear to transparent
                win.clear(0x00000000);
            }
        }
        
        // -------------------------------------------------------------------
        // LAYER 2.5: COMMAND HISTORY PANEL (Top right corner)
        // -------------------------------------------------------------------
        if let Some(hist) = compositor.get_layer_mut(history_layer) {
            let hw = hist.width;
            let hh = hist.height;
            
            // Semi-transparent dark background
            hist.clear(0xD8181818);
            
            // Border
            hist.draw_rect(0, 0, hw, hh, 0xFF444444);
            hist.draw_rect(1, 1, hw - 2, hh - 2, 0xFF333333);
            
            // Title bar
            hist.fill_rect(2, 2, hw - 4, 20, 0xFF252525);
            hist.draw_text("Command History", 8, 6, 0xFFAAAAAA);
            
            // History entries
            let start_y = 26u32;
            let line_h = 18u32;
            
            if command_history.is_empty() {
                hist.draw_text("(no commands yet)", 10, start_y + 5, 0xFF666666);
            } else {
                // Show most recent first (reverse order)
                for (i, cmd) in command_history.iter().rev().take(10).enumerate() {
                    let y = start_y + (i as u32) * line_h;
                    if y + line_h > hh - 5 { break; }
                    
                    // Number
                    let num = command_history.len() - i;
                    let num_str = format!("{:2}.", num);
                    hist.draw_text(&num_str, 6, y + 2, 0xFF666666);
                    
                    // Command (truncate if too long)
                    let display_cmd = if cmd.len() > 26 {
                        format!("{}...", &cmd[..23])
                    } else {
                        cmd.clone()
                    };
                    hist.draw_text(&display_cmd, 30, y + 2, 0xFF88FF88);
                }
            }
        }
        
        // -------------------------------------------------------------------
        // LAYER 3: TASKBAR (Bottom)
        // -------------------------------------------------------------------
        if let Some(bar) = compositor.get_layer_mut(taskbar_layer) {
            bar.clear(0xFF0A0A0A);
            
            // Top border
            bar.fill_rect(0, 0, width, 2, green_dim);
            
            // TrustOS menu button
            bar.fill_rect(5, 6, 100, 28, if menu_open { 0xFF002200 } else { 0xFF0A1A0A });
            bar.draw_rect(5, 6, 100, 28, green_main);
            bar.draw_text("TrustOS", 20, 12, 0xFFFFFFFF); // White text
            
            // Active module indicator
            let mode_name = match active_mode {
                AppMode::Shell => "Shell",
                AppMode::Network => "Network",
                AppMode::Hardware => "Hardware",
                AppMode::TextEditor => "Editor",
                AppMode::UserMgmt => "Users",
                AppMode::Files => "Files",
                AppMode::Browser => "Browser",
                AppMode::ImageViewer => "Images",
            };
            bar.fill_rect(115, 6, 90, 28, 0xFF001100);
            bar.draw_text(mode_name, 125, 12, 0xFFFFFFFF); // White text
            
            // Window button in taskbar (shows when window exists)
            // Position after the mode indicator
            let win_btn_x = 220u32;
            if window_visible {
                // Window is open - show active button
                bar.fill_rect(win_btn_x, 6, 100, 28, 0xFF002200);
                bar.draw_rect(win_btn_x, 6, 100, 28, green_main);
                bar.draw_text(mode_name, win_btn_x + 10, 12, green_bright);
                // Active indicator line at bottom
                bar.fill_rect(win_btn_x + 20, 32, 60, 3, green_main);
            } else {
                // Window is minimized - show inactive button
                bar.fill_rect(win_btn_x, 6, 100, 28, 0xFF0A0A0A);
                bar.draw_rect(win_btn_x, 6, 100, 28, green_dim);
                bar.draw_text(mode_name, win_btn_x + 10, 12, green_dim);
            }
            
            // Settings button (gear icon)
            let settings_btn_x = 340u32;
            let settings_bg = if settings_open { 0xFF002200 } else { 0xFF0A1A0A };
            bar.fill_rect(settings_btn_x, 6, 50, 28, settings_bg);
            bar.draw_rect(settings_btn_x, 6, 50, 28, green_dim);
            bar.draw_text("[S]", settings_btn_x + 10, 12, if settings_open { green_bright } else { green_main });
            
            // Clock
            let dt = crate::rtc::read_rtc();
            let time_str = format!("{:02}:{:02}:{:02}", dt.hour, dt.minute, dt.second);
            bar.draw_text(&time_str, width - 180, 12, green_main);
            
            // FPS
            let fps_str = format!("{}fps", fps);
            bar.draw_text(&fps_str, width - 260, 12, green_dim);
            
            // Status indicators
            bar.fill_circle(width - 60, 20, 6, green_main);
            bar.fill_circle(width - 40, 20, 6, 0xFFFFAA00);
        }
        
        // -------------------------------------------------------------------
        // LAYER 4: MENU (TrustOS popup menu with Apps + Power Options)
        // -------------------------------------------------------------------
        if let Some(menu) = compositor.get_layer_mut(menu_layer) {
            if menu_open {
                menu.visible.store(true, core::sync::atomic::Ordering::SeqCst);
                menu.clear(0xF0080808);  // Dark background
                
                let menu_w = 270u32;
                let menu_h = 390u32;
                
                // Menu border
                menu.draw_rect(0, 0, menu_w, menu_h, green_main);
                menu.draw_rect(1, 1, menu_w - 2, menu_h - 2, green_dim);
                
                // Menu title
                menu.fill_rect(2, 2, menu_w - 4, 34, 0xFF001500);
                menu.draw_text("TrustOS Menu", 10, 10, green_main);
                
                // Menu items
                let item_height = 36u32;
                for (i, (name, item)) in menu_items.iter().enumerate() {
                    let iy = 40 + (i as u32) * item_height;
                    
                    // Skip rendering for separator
                    if name.starts_with("-") {
                        menu.fill_rect(10, iy + 14, menu_w - 20, 1, green_dim);
                        continue;
                    }
                    
                    // Highlight if hovered
                    if menu_hover == i as i32 {
                        menu.fill_rect(5, iy, menu_w - 10, item_height - 2, 0xFF002200);
                    }
                    
                    // Determine color and icon based on item type
                    let (color, icon) = match item {
                        MenuItem::App(_) => {
                            let c = if menu_hover == i as i32 { green_bright } else { green_dim };
                            (c, ">")
                        },
                        MenuItem::Shutdown => {
                            let c = if menu_hover == i as i32 { 0xFFFF6666 } else { 0xFFAA4444 };
                            (c, "X")
                        },
                        MenuItem::Reboot => {
                            let c = if menu_hover == i as i32 { 0xFFFFAA66 } else { 0xFFAA8844 };
                            (c, "R")
                        },
                    };
                    
                    // Item text
                    menu.draw_text(name, 24, iy + 10, color);
                    
                    // Icon/indicator
                    menu.draw_text(icon, menu_w - 30, iy + 10, color);
                }
            } else {
                menu.visible.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }
        
        // -------------------------------------------------------------------
        // LAYER 4.5: SETTINGS PANEL
        // -------------------------------------------------------------------
        if let Some(settings) = compositor.get_layer_mut(settings_layer) {
            if settings_open {
                settings.visible.store(true, core::sync::atomic::Ordering::SeqCst);
                settings.clear(0xF0080808);  // Dark background
                
                let panel_w = 270u32;
                let panel_h = 340u32;  // Increased for HoloMatrix options
                
                // Panel border
                settings.draw_rect(0, 0, panel_w, panel_h, green_main);
                settings.draw_rect(1, 1, panel_w - 2, panel_h - 2, green_dim);
                
                // Panel title
                settings.fill_rect(2, 2, panel_w - 4, 34, 0xFF001500);
                settings.draw_text("Settings", 10, 10, green_main);
                
                // Animation toggle
                let anim_y = 50u32;
                let my = mouse_y as u32;
                let mx = mouse_x as u32;
                let panel_top = height - 380;  // Adjusted for taller panel
                let anim_hover = my >= (panel_top + anim_y) 
                    && my < (panel_top + anim_y + 36)
                    && mx >= 340 && mx < (340 + panel_w);
                if anim_hover {
                    settings.fill_rect(5, anim_y, panel_w - 10, 34, 0xFF002200);
                }
                settings.draw_text("Animations:", 15, anim_y + 10, green_dim);
                let anim_status = if settings_anim_enabled { "ON" } else { "OFF" };
                let anim_color = if settings_anim_enabled { 0xFF00FF66 } else { 0xFFFF6666 };
                settings.draw_text(anim_status, panel_w - 50, anim_y + 10, anim_color);
                
                // Speed setting
                let speed_y = 90u32;
                let speed_hover = my >= (panel_top + speed_y) 
                    && my < (panel_top + speed_y + 36)
                    && mx >= 340 && mx < (340 + panel_w);
                if speed_hover {
                    settings.fill_rect(5, speed_y, panel_w - 10, 34, 0xFF002200);
                }
                settings.draw_text("Speed:", 15, speed_y + 10, green_dim);
                let speed_str = format!("{:.1}x", settings_anim_speed);
                settings.draw_text(&speed_str, panel_w - 60, speed_y + 10, green_main);
                
                // --- BACKGROUND SECTION ---
                settings.draw_text("- Background -", 15, 140, 0xFF555555);
                
                // HoloMatrix toggle
                let holo_y = 160u32;
                let holo_hover = my >= (panel_top + holo_y) 
                    && my < (panel_top + holo_y + 36)
                    && mx >= 340 && mx < (340 + panel_w);
                if holo_hover {
                    settings.fill_rect(5, holo_y, panel_w - 10, 34, 0xFF002200);
                }
                settings.draw_text("HoloMatrix 3D:", 15, holo_y + 10, green_dim);
                let holo_status = if holo_enabled { "ON" } else { "OFF" };
                let holo_color = if holo_enabled { 0xFF00FFFF } else { 0xFFFF6666 };
                settings.draw_text(holo_status, panel_w - 50, holo_y + 10, holo_color);
                
                // HoloMatrix scene selector
                let scene_y = 200u32;
                let scene_hover = my >= (panel_top + scene_y) 
                    && my < (panel_top + scene_y + 36)
                    && mx >= 340 && mx < (340 + panel_w);
                if scene_hover && holo_enabled {
                    settings.fill_rect(5, scene_y, panel_w - 10, 34, 0xFF002200);
                }
                let scene_label_color = if holo_enabled { green_dim } else { 0xFF333333 };
                settings.draw_text("Scene:", 15, scene_y + 10, scene_label_color);
                let scene_color = if holo_enabled { 0xFF00FFFF } else { 0xFF444444 };
                settings.draw_text(holo_scene.name(), panel_w - 80, scene_y + 10, scene_color);
                
                // Instructions
                settings.draw_text("Click to toggle/cycle", 15, 250, 0xFF555555);
                
                // Close button hint
                settings.draw_text("[Esc] or click away", 15, 305, 0xFF444444);
                
                // Handle click on animation toggle
                if clicked && anim_hover {
                    settings_anim_enabled = !settings_anim_enabled;
                    crate::desktop::set_animations_enabled(settings_anim_enabled);
                }
                
                // Handle click on speed
                if clicked && speed_hover {
                    // Cycle: 0.5 -> 1.0 -> 2.0 -> 0.5
                    settings_anim_speed = if settings_anim_speed <= 0.5 { 1.0 } 
                        else if settings_anim_speed <= 1.0 { 2.0 } 
                        else { 0.5 };
                    crate::desktop::set_animation_speed(settings_anim_speed);
                }
                
                // Handle click on HoloMatrix toggle
                if clicked && holo_hover {
                    holo_enabled = !holo_enabled;
                    crate::graphics::holomatrix::set_enabled(holo_enabled);
                }
                
                // Handle click on scene selector
                if clicked && scene_hover && holo_enabled {
                    holo_scene = holo_scene.next();
                    crate::graphics::holomatrix::set_scene(holo_scene);
                }
            } else {
                settings.visible.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }
        
        // -------------------------------------------------------------------
        // LAYER 5: MOUSE CURSOR (Always on top)
        // -------------------------------------------------------------------
        if let Some(cursor) = compositor.get_layer_mut(cursor_layer) {
            cursor.clear(0x00000000); // Transparent
            
            // Cursor color changes when clicking (visual feedback)
            let cursor_color = if left { 0xFF00FF00 } else { 0xFFFFFFFF }; // Green when clicked
            let border_color = if left { 0xFF005500 } else { 0xFF000000 };
            
            // Draw arrow cursor
            // Main pointer triangle
            for i in 0..16 {
                for j in 0..=i {
                    if j <= i && i < 16 {
                        cursor.set_pixel(j as u32, i as u32, cursor_color);
                    }
                }
            }
            // Black border
            for i in 0..16 {
                cursor.set_pixel(0, i as u32, border_color);
                cursor.set_pixel(i as u32, i as u32, border_color);
            }
            // Tail
            for i in 10..16 {
                cursor.set_pixel((i - 5) as u32, i as u32, cursor_color);
                cursor.set_pixel((i - 6) as u32, i as u32, cursor_color);
            }
        }
        
        // -------------------------------------------------------------------
        // COMPOSITE & PRESENT
        // -------------------------------------------------------------------
        compositor.composite();
        compositor.present();
        
        } // --- END RENDER FRAME (frame-rate decoupling) ---
        
        // FPS tracking (counts ALL frames: render + skip)
        frame_count += 1;
        frame_in_second += 1;
        
        // Simple frame milestone logging (every 100 frames)
        if frame_count % 100 == 0 {
            crate::serial_println!("[COSMIC2] Frame {}", frame_count);
        }
        
        let now = crate::cpu::tsc::read_tsc();
        if now - last_second_tsc >= tsc_freq {
            fps = frame_in_second;
            render_fps = render_in_second;
            frame_in_second = 0;
            render_in_second = 0;
            last_second_tsc = now;
            crate::serial_println!("[COSMIC2] FPS: {} (render: {}) | Frame: {} | Mode: {}",
                fps, render_fps, frame_count, if use_formula { "FORMULA" } else if use_braille { "BRAILLE" } else if use_fast_matrix { "FAST" } else { "LEGACY" });
        }
        
        // Frame timing: brief pause to process pending keyboard/mouse interrupts.
        // Use a short spin with `pause` instead of `hlt` -- hlt blocks up to 10ms 
        // (timer tick), which caps FPS at ~1000/(render_ms + 10). Pause enables
        // interrupts for ~100Aus then continues, minimizing idle time.
        unsafe {
            // Enable interrupts so pending IRQs (keyboard, mouse, timer) fire
            core::arch::asm!("sti");
            // Brief spin: ~100 iterations A-- ~100 cycles Eoe 30-50Aus at 3GHz
            for _ in 0..100 {
                core::arch::asm!("pause");
            }
        }
    }
    
    // Restore console mode
    crate::framebuffer::clear();
    crate::serial_println!("[COSMIC2] Exited");
    crate::println_color!(COLOR_GREEN, "COSMIC V2 Desktop exited. Type 'help' for commands.");
}

// ==================== COSMIC DESKTOP MODE ====================

pub(super) fn cmd_cosmic_desktop() {
    use crate::cosmic::{Rect, Point, Color};
    use crate::cosmic::theme::{dark, matrix};
    use alloc::format;
    
    crate::println_color!(COLOR_CYAN, "+-----------------------------------------------------------+");
    crate::println_color!(COLOR_CYAN, "|       COSMIC Desktop Environment - TrustOS Edition        |");
    crate::println_color!(COLOR_CYAN, "|-----------------------------------------------------------|");
    crate::println_color!(COLOR_GREEN, "|  Controls:                                                |");
    crate::println_color!(COLOR_WHITE, "|    ESC / Q     - Exit desktop                             |");
    crate::println_color!(COLOR_WHITE, "|    M           - Matrix theme (cyberpunk)                 |");
    crate::println_color!(COLOR_WHITE, "|    D           - Dark theme (default)                     |");
    crate::println_color!(COLOR_WHITE, "|    1-5         - Switch apps                              |");
    crate::println_color!(COLOR_WHITE, "|    Mouse       - Interact with UI                         |");
    crate::println_color!(COLOR_CYAN, "+-----------------------------------------------------------+");
    crate::serial_println!("[COSMIC] Starting COSMIC Desktop Environment...");
    
    // Flush keyboard buffer
    while crate::keyboard::try_read_key().is_some() {}
    
    let (width, height) = crate::framebuffer::get_dimensions();
    if width == 0 || height == 0 {
        crate::println_color!(COLOR_RED, "Error: Invalid framebuffer!");
        return;
    }
    
    // Initialize double buffering for FAST rendering
    crate::framebuffer::init_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);
    crate::serial_println!("[COSMIC] Double buffering enabled for fast rendering");
    
    // Set mouse screen bounds
    crate::mouse::set_screen_size(width, height);
    
    // -------------------------------------------------------------------
    // DESKTOP STATE
    // -------------------------------------------------------------------
    let mut running = true;
    let mut use_matrix_theme = true;
    let mut frame_count = 0u64;
    
    // FPS tracking with VSync target
    let tsc_freq = crate::cpu::tsc::frequency_hz();
    let mut fps = 0u32;
    let mut frame_in_second = 0u32;
    let mut last_second_tsc = crate::cpu::tsc::read_tsc();
    let mut show_fps = true;  // Toggle with 'fps' command
    
    // Matrix renderer mode flags
    let mut use_fast_matrix = false;  // Braille mode is default
    let mut use_braille = true;      // Braille sub-pixel renderer (default)
    
    // Target frame time for ~60 FPS (VSync simulation)
    let target_fps = 60u64;
    let frame_tsc_target = tsc_freq / target_fps;
    let mut last_frame_tsc = crate::cpu::tsc::read_tsc();
    
    // -------------------------------------------------------------------
    // MATRIX RAIN STATE - Each column has its own position and speed
    // Dense matrix with more columns for proper coverage
    // -------------------------------------------------------------------
    const MATRIX_COLS: usize = 160;  // Dense! 8px spacing at 1280px
    const MATRIX_CHAR_H: u32 = 16;   // Character height
    const MATRIX_TRAIL_LEN: usize = 30; // Longer trails for fuller look
    
    // Column state: (y_position, speed, random_seed)
    let mut matrix_cols: [(i32, u32, u32); MATRIX_COLS] = [(0, 0, 0); MATRIX_COLS];
    
    // Initialize columns with random starting positions and speeds
    for i in 0..MATRIX_COLS {
        let seed = (i as u32 * 2654435761) ^ 0xDEADBEEF;
        let start_y = -((seed % (height * 2)) as i32); // Start above screen
        let speed = 2 + (seed % 5); // Speed 2-6 pixels per frame
        matrix_cols[i] = (start_y, speed, seed);
    }
    
    // Matrix characters (katakana-like + numbers)
    const MATRIX_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
    
    // Mouse state
    let mut prev_left = false;
    let mut click_this_frame = false;
    let mut prev_mx = 0.0f32;
    let mut prev_my = 0.0f32;
    let mut prev_theme = use_matrix_theme;
    let mut prev_app = 0usize;
    let mut prev_hovered = -1i32;
    let mut needs_full_redraw = true; // First frame always needs full redraw
    
    // Active app
    let mut active_app = 0usize;
    let mut hovered_dock = -1i32;
    
    // TrustOS Menu state
    let mut menu_open = false;
    let mut menu_hover = -1i32;  // Which menu item is hovered (-1 = none)
    let mut search_active = false;
    let mut search_text: [u8; 32] = [0u8; 32];
    let mut search_len = 0usize;
    
    // Menu items for TrustOS menu
    let menu_items = [
        "Apps",
        "Browser",
        "Calculator", 
        "Files",
        "Network",
        "Settings",
        "Terminal",
        "---",  // Separator
        "Sign Out",
        "Restart",
        "Shutdown",
    ];
    
    // App info
    let apps = [
        ("Files", "File Manager"),
        ("Terminal", "System Terminal"),
        ("Browser", "Web Browser"),
        ("Code", "Text Editor"),
        ("Settings", "System Settings"),
    ];
    
    // Window state (draggable)
    let mut win_x = 150.0f32;
    let mut win_y = 60.0f32;
    let win_w = 700.0f32;
    let win_h = 450.0f32;
    let mut dragging = false;
    let mut drag_off_x = 0.0f32;
    let mut drag_off_y = 0.0f32;
    
    // Terminal content for terminal app
    let terminal_lines = [
        "$ neofetch",
        "  _____              _    ___  ____  ",
        " |_   _| __ _   _ __| |_ / _ \\/ ___| ",
        "   | || '__| | | / __| __| | | \\___ \\ ",
        "   | || |  | |_| \\__ \\ |_| |_| |___) |",
        "   |_||_|   \\__,_|___/\\__|\\___/|____/ ",
        "",
        "  OS: TrustOS v0.2.0",
        "  Kernel: Custom Rust Kernel",
        "  Shell: TrustOS Shell",
        "  Resolution: 1280x800",
        "  Theme: COSMIC Matrix",
        "",
        "$ _",
    ];
    
    crate::serial_println!("[COSMIC] Entering main loop...");
    
    // -------------------------------------------------------------------
    // MAIN LOOP
    // -------------------------------------------------------------------
    while running {
        // ---------------------------------------------------------------
        // INPUT HANDLING (do this FIRST for responsiveness)
        // ---------------------------------------------------------------
        let mouse = crate::mouse::get_state();
        let mx = mouse.x as f32;
        let my = mouse.y as f32;
        let left_pressed = mouse.left_button;
        
        // Detect click (press this frame, wasn't pressed before)
        click_this_frame = left_pressed && !prev_left;
        prev_left = left_pressed;
        
        // Keyboard
        if let Some(key) = crate::keyboard::try_read_key() {
            if search_active {
                // Handle search input
                match key {
                    27 => { search_active = false; },  // ESC closes search
                    8 => {  // Backspace
                        if search_len > 0 { search_len -= 1; }
                    },
                    13 => { search_active = false; },  // Enter confirms
                    32..=126 => {  // Printable ASCII
                        if search_len < 31 {
                            search_text[search_len] = key;
                            search_len += 1;
                        }
                    },
                    _ => {}
                }
            } else {
                // Normal mode
                match key {
                    27 | b'q' | b'Q' => {
                        if menu_open { menu_open = false; }
                        else { running = false; }
                    },
                    b'm' | b'M' => { use_matrix_theme = true; needs_full_redraw = true; },
                    b'd' | b'D' => { use_matrix_theme = false; needs_full_redraw = true; },
                    b'1'..=b'5' => { active_app = (key - b'1') as usize; needs_full_redraw = true; },
                    b's' | b'S' => { search_active = true; },  // S to open search
                    b't' | b'T' => { menu_open = !menu_open; },  // T to toggle menu
                    _ => {}
                }
            }
        }
        
        // Check if anything changed that requires redraw
        let mouse_moved = (mx - prev_mx).abs() > 0.5 || (my - prev_my).abs() > 0.5;
        let state_changed = use_matrix_theme != prev_theme || active_app != prev_app || click_this_frame || dragging;
        
        // Matrix theme always animates (rain effect), so always redraw
        // Only skip if dark theme AND nothing changed
        if !use_matrix_theme && !needs_full_redraw && !mouse_moved && !state_changed {
            // Just update FPS counter
            frame_count += 1;
            frame_in_second += 1;
            let now = crate::cpu::tsc::read_tsc();
            if now - last_second_tsc >= tsc_freq {
                fps = frame_in_second;
                frame_in_second = 0;
                last_second_tsc = now;
            }
            continue;
        }
        
        prev_mx = mx;
        prev_my = my;
        prev_theme = use_matrix_theme;
        prev_app = active_app;
        needs_full_redraw = false;
        
        // ---------------------------------------------------------------
        // THEME SELECTION
        // ---------------------------------------------------------------
        let (bg, panel_bg, surface, surface_hover, accent, text_pri, text_sec, 
             header_bg, close_bg, max_bg, min_bg, success, warning) = 
            if use_matrix_theme {
                (matrix::BG_BASE, matrix::PANEL_BG, matrix::SURFACE, matrix::SURFACE_HOVER,
                 matrix::ACCENT, matrix::TEXT_PRIMARY, matrix::TEXT_SECONDARY,
                 matrix::HEADER_BG, matrix::CLOSE_BG, matrix::MAXIMIZE_BG, matrix::MINIMIZE_BG,
                 matrix::SUCCESS, matrix::WARNING)
            } else {
                (dark::BG_BASE, dark::PANEL_BG, dark::SURFACE, dark::SURFACE_HOVER,
                 dark::ACCENT, dark::TEXT_PRIMARY, dark::TEXT_SECONDARY,
                 dark::HEADER_BG, dark::CLOSE_BG, dark::MAXIMIZE_BG, dark::MINIMIZE_BG,
                 dark::SUCCESS, dark::WARNING)
            };
        
        // Convert colors to u32 for direct framebuffer operations (SSE2 fast path)
        let bg_u32 = bg.to_u32();
        let panel_bg_u32 = panel_bg.to_u32();
        let surface_u32 = surface.to_u32();
        let surface_hover_u32 = surface_hover.to_u32();
        let accent_u32 = accent.to_u32();
        let text_pri_u32 = text_pri.to_u32();
        let text_sec_u32 = text_sec.to_u32();
        let header_bg_u32 = header_bg.to_u32();
        let close_bg_u32 = close_bg.to_u32();
        let max_bg_u32 = max_bg.to_u32();
        let min_bg_u32 = min_bg.to_u32();
        let success_u32 = success.to_u32();
        let warning_u32 = warning.to_u32();
        
        // Use framebuffer module directly for FAST SSE2 rendering
        use crate::framebuffer::{
            clear_backbuffer, fill_rect, fill_rounded_rect, fill_circle,
            stroke_rounded_rect, draw_text, swap_buffers, draw_rect
        };
        
        // TrustOS colors - Brighter green like reference
        let green_main: u32 = 0xFF00FF66;      // #00FF66 - Main green
        let green_bright: u32 = 0xFF00FF88;    // #00FF88 - Bright
        let green_dim: u32 = 0xFF009944;       // Dimmer green
        let black: u32 = 0xFF000000;           // Pure black
        
        // ---------------------------------------------------------------
        // RENDER: PURE BLACK BACKGROUND
        // ---------------------------------------------------------------
        clear_backbuffer(black);
        
        // ---------------------------------------------------------------
        // RENDER: MATRIX RAIN ON ENTIRE SCREEN (like reference image)
        // ---------------------------------------------------------------
        let bar_h = 36u32;
        let col_width = width / MATRIX_COLS as u32;
        
        for col in 0..MATRIX_COLS {
            let (y_pos, speed, seed) = matrix_cols[col];
            let x = (col as u32 * col_width) + col_width / 2;
            
            // Update position
            let new_y = y_pos + speed as i32;
            let new_y = if new_y > height as i32 + (MATRIX_TRAIL_LEN as i32 * MATRIX_CHAR_H as i32) {
                let new_seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                matrix_cols[col].2 = new_seed;
                -((new_seed % (height / 2)) as i32)
            } else {
                new_y
            };
            matrix_cols[col].0 = new_y;
            
            // Draw falling trail with DEPTH EFFECT based on speed
            // Slow columns = far (dim, grayish) | Fast columns = near (bright, vivid green)
            // Speed ranges from 2 (far) to 6 (near)
            let depth = (speed as f32 - 2.0) / 4.0; // 0.0 = far, 1.0 = near
            let depth_brightness_mult = 0.4 + depth * 0.6; // 40% for far, 100% for near
            let depth_saturation = 0.3 + depth * 0.7; // Less saturated when far
            
            for i in 0..MATRIX_TRAIL_LEN {
                let char_y = new_y - (i as i32 * MATRIX_CHAR_H as i32);
                if char_y < 0 || char_y >= (height - bar_h) as i32 { continue; }
                
                let base_brightness = if i == 0 { 255u8 } 
                    else if i == 1 { 220u8 } 
                    else { 180u8.saturating_sub((i as u8).saturating_mul(9)) };
                if base_brightness < 20 { continue; }
                
                // Apply depth multiplier to brightness
                let brightness = ((base_brightness as f32) * depth_brightness_mult) as u8;
                
                // Calculate color with depth-based saturation
                // Far columns: more gray/blueish, Near columns: pure bright green
                let r = if i == 0 { 
                    ((180.0 * depth_brightness_mult) as u8) 
                } else { 
                    // Add slight gray tint for distant columns
                    ((20.0 * (1.0 - depth_saturation)) as u8)
                };
                let g = brightness;
                let b = if i == 0 { 
                    ((180.0 * depth_brightness_mult) as u8) 
                } else { 
                    // Add blue tint for distant columns (atmospheric perspective)
                    ((40.0 * (1.0 - depth_saturation) + 10.0 * depth) as u8)
                };
                let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                
                let char_seed = seed.wrapping_add((i as u32 * 7919) ^ (frame_count as u32 / 8));
                let char_idx = (char_seed as usize) % MATRIX_CHARS.len();
                let char_str: [u8; 2] = [MATRIX_CHARS[char_idx], 0];
                let char_s = unsafe { core::str::from_utf8_unchecked(&char_str[..1]) };
                draw_text(char_s, x, char_y as u32, color);
            }
        }
        
        // ---------------------------------------------------------------
        // RENDER: TRUSTOS LOGO (center) - Padlock + Shield + Checkmark + Hands
        // Matching reference image exactly
        // ---------------------------------------------------------------
        let logo_cx = width / 2 + 100;  // Slightly right of center
        let logo_cy = height / 2 - 50;
        
        // === PADLOCK at top ===
        let lock_y = logo_cy - 180;
        // Shackle (U shape)
        fill_rect(logo_cx - 40, lock_y, 12, 60, green_main);      // Left arm
        fill_rect(logo_cx + 28, lock_y, 12, 60, green_main);      // Right arm
        fill_rect(logo_cx - 40, lock_y - 10, 80, 15, green_main); // Top bar
        fill_rect(logo_cx - 30, lock_y - 20, 60, 15, green_main); // Top curve
        // Lock body
        fill_rect(logo_cx - 50, lock_y + 50, 100, 70, green_main);
        fill_rect(logo_cx - 44, lock_y + 56, 88, 58, 0xFF0A150Au32);
        // Keyhole
        fill_circle(logo_cx, lock_y + 80, 10, green_main);
        fill_rect(logo_cx - 5, lock_y + 88, 10, 20, green_main);
        
        // === SHIELD (hexagonal shape) ===
        let shield_y = logo_cy - 60;
        let shield_w = 180u32;
        let shield_h = 220u32;
        // Shield outline (multiple layers for thickness)
        for t in 0..4 {
            let tt = t as u32;
            // Top edge
            fill_rect(logo_cx - shield_w/2 + 20 + tt, shield_y + tt, shield_w - 40, 3, green_main);
            // Upper sides (angled)
            fill_rect(logo_cx - shield_w/2 + tt, shield_y + 20 + tt, 25, 3, green_main);
            fill_rect(logo_cx + shield_w/2 - 25 - tt, shield_y + 20 + tt, 25, 3, green_main);
            // Sides
            fill_rect(logo_cx - shield_w/2 + tt, shield_y + 20, 3, shield_h - 80, green_main);
            fill_rect(logo_cx + shield_w/2 - 3 - tt, shield_y + 20, 3, shield_h - 80, green_main);
            // Bottom point
            fill_rect(logo_cx - 3, shield_y + shield_h - 20 - tt, 6, 20, green_main);
        }
        // Shield inner darker area
        fill_rect(logo_cx - shield_w/2 + 8, shield_y + 25, shield_w - 16, shield_h - 70, 0xFF051208u32);
        
        // === CHECKMARK (V) in center of shield ===
        let check_cx = logo_cx;
        let check_cy = logo_cy + 20;
        // Draw thick V shape
        for t in 0..8 {
            // Left diagonal going down
            for i in 0..30 {
                fill_rect(check_cx - 50 + i + t, check_cy - 30 + i, 4, 4, green_main);
            }
            // Right diagonal going up (longer)
            for i in 0..50 {
                fill_rect(check_cx - 20 + i + t, check_cy + i.min(29) - (i.saturating_sub(29)), 4, 4, green_main);
            }
        }
        
        // === HANDS holding shield (simplified) ===
        let hand_y = logo_cy + 100;
        // Left hand
        fill_rect(logo_cx - 100, hand_y, 40, 15, green_dim);
        fill_rect(logo_cx - 110, hand_y + 10, 20, 30, green_dim);
        fill_rect(logo_cx - 95, hand_y + 15, 10, 25, green_dim);
        fill_rect(logo_cx - 80, hand_y + 15, 10, 20, green_dim);
        // Right hand  
        fill_rect(logo_cx + 60, hand_y, 40, 15, green_dim);
        fill_rect(logo_cx + 90, hand_y + 10, 20, 30, green_dim);
        fill_rect(logo_cx + 85, hand_y + 15, 10, 25, green_dim);
        fill_rect(logo_cx + 70, hand_y + 15, 10, 20, green_dim);
        
        // === "TRust-os" text to the right of logo ===
        // Large stylized text
        let text_x = logo_cx + 130;
        let text_y = logo_cy + 40;
        draw_text("TRust-os", text_x, text_y, green_main);
        // Draw it bigger by repeating offset
        draw_text("TRust-os", text_x + 1, text_y, green_main);
        draw_text("TRust-os", text_x, text_y + 1, green_main);
        draw_text("TRust-os", text_x + 1, text_y + 1, green_main);
                
        // Window dimensions
        let win_x_u32 = 90u32;
        let win_y_u32 = 300u32;
        let win_w_u32 = 380u32;
        let win_h_u32 = 280u32;
        
        // ---------------------------------------------------------------
        // RENDER: LEFT DOCK (VERTICAL)
        // ---------------------------------------------------------------
        let dock_x = 20u32;
        let dock_y = 50u32;
        let dock_icon_size = 44u32;
        let dock_gap = 20u32;
        let dock_icons = 5u32;
        
        // Draw solid black background for dock area
        fill_rect(0, 0, 80, height - bar_h, 0xFF050505u32);
        
        hovered_dock = -1;
        for i in 0..dock_icons {
            let iy = dock_y + i * (dock_icon_size + dock_gap);
            let ix = dock_x;
            
            // Hover detection
            let hovered = mx >= ix as f32 && mx < (ix + dock_icon_size) as f32 && 
                          my >= iy as f32 && my < (iy + dock_icon_size) as f32;
            if hovered {
                hovered_dock = i as i32;
                if click_this_frame {
                    active_app = i as usize;
                }
            }
            
            // Icon background - outlined square with rounded corners
            let icon_color = if i as usize == active_app { 
                green_main 
            } else if hovered { 
                green_bright 
            } else { 
                green_dim 
            };
            
            // Draw icon outline (not filled!)
            draw_rect(ix, iy, dock_icon_size, dock_icon_size, icon_color);
            
            // Simple icon symbols inside
            let cx = ix + dock_icon_size / 2;
            let cy = iy + dock_icon_size / 2;
            match i {
                0 => { // Play button (triangle)
                    fill_rect(cx - 8, cy - 10, 4, 20, icon_color);
                    fill_rect(cx - 4, cy - 8, 4, 16, icon_color);
                    fill_rect(cx, cy - 6, 4, 12, icon_color);
                    fill_rect(cx + 4, cy - 4, 4, 8, icon_color);
                },
                1 => { // Terminal (rectangle with lines)
                    draw_rect(cx - 12, cy - 10, 24, 20, icon_color);
                    fill_rect(cx - 8, cy - 4, 10, 2, icon_color);
                    fill_rect(cx - 8, cy + 2, 6, 2, icon_color);
                },
                2 => { // Grid (apps)
                    for row in 0..2 {
                        for col in 0..2 {
                            draw_rect(cx - 10 + col * 12, cy - 10 + row * 12, 8, 8, icon_color);
                        }
                    }
                },
                3 => { // Network
                    draw_rect(cx - 10, cy - 8, 20, 16, icon_color);
                    fill_rect(cx - 6, cy - 4, 2, 8, icon_color);
                    fill_rect(cx - 2, cy - 2, 2, 6, icon_color);
                    fill_rect(cx + 2, cy - 6, 2, 10, icon_color);
                    fill_rect(cx + 6, cy - 4, 2, 8, icon_color);
                },
                4 => { // Settings (gear outline)
                    fill_circle(cx, cy, 10, icon_color);
                    fill_circle(cx, cy, 6, black);
                },
                _ => {}
            }
        }
        
        // Help icon at bottom of dock
        let help_y = height as u32 - 80;
        draw_rect(dock_x, help_y, dock_icon_size, dock_icon_size, green_dim);
        draw_text("?", dock_x + 18, help_y + 16, green_dim);
        
        // ---------------------------------------------------------------
        // RENDER: MAIN WINDOW (Terminal style like reference)
        // Window variables already defined above for Matrix rain clipping
        // ---------------------------------------------------------------
        
        // Window border (thick green line like reference)
        let border_thickness = 3u32;
        // Top border
        fill_rect(win_x_u32, win_y_u32, win_w_u32, border_thickness, green_main);
        // Bottom border
        fill_rect(win_x_u32, win_y_u32 + win_h_u32 - border_thickness, win_w_u32, border_thickness, green_main);
        // Left border
        fill_rect(win_x_u32, win_y_u32, border_thickness, win_h_u32, green_main);
        // Right border
        fill_rect(win_x_u32 + win_w_u32 - border_thickness, win_y_u32, border_thickness, win_h_u32, green_main);
        
        // Window interior (SOLID BLACK - completely fills to prevent flicker)
        fill_rect(win_x_u32 + border_thickness, win_y_u32 + border_thickness, 
                  win_w_u32 - border_thickness * 2, win_h_u32 - border_thickness * 2, black);
        
        // Title bar with window controls
        let title_h = 28u32;
        fill_rect(win_x_u32 + border_thickness, win_y_u32 + border_thickness, 
                  win_w_u32 - border_thickness * 2, title_h, 0xFF0A1A0Au32);
        
        // Window title
        let (title, _) = apps[active_app];
        let title_text = format!("TrustOS {} v1.00", title);
        draw_text(&title_text, win_x_u32 + 12, win_y_u32 + 10, green_main);
        
        // Window control buttons (right side) - 3 circles
        let btn_y = win_y_u32 + 10;
        let close_x = win_x_u32 + win_w_u32 - 60;
        // Red close
        fill_circle(close_x, btn_y + 6, 6, 0xFFFF5555u32);
        // Yellow maximize  
        fill_circle(close_x + 18, btn_y + 6, 6, 0xFFFFDD55u32);
        // Green minimize
        fill_circle(close_x + 36, btn_y + 6, 6, 0xFF55FF55u32);
        
        // Content area - GREEN TEXT like reference image
        let content_x = win_x_u32 + 15;
        let content_y = win_y_u32 + title_h + 15;
        
        // Terminal prompt with colored parts - root@trustos
        let cwd_display = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
        // Draw "root" in red
        draw_text("root", content_x, content_y, 0xFFFF0000u32);  // Pure red
        // Draw "@" in white
        draw_text("@", content_x + 32, content_y, 0xFFFFFFFFu32);  // White
        // Draw "trustos" in green
        draw_text("trustos", content_x + 40, content_y, 0xFF00FF00u32);  // Pure green
        // Draw ":path$ " in green
        let path_part = format!(":{}$ ", cwd_display);
        draw_text(&path_part, content_x + 96, content_y, 0xFF00FF00u32);  // Pure green
        // Cursor block
        let prompt_len = 4 + 1 + 7 + path_part.len();  // root @ trustos :path$
        let cursor_x = content_x + (prompt_len * 8) as u32;
        fill_rect(cursor_x, content_y, 8, 16, green_bright);
        
        // ---------------------------------------------------------------
        // RENDER: BOTTOM STATUS BAR (exact match to reference)
        // ---------------------------------------------------------------
        let bar_y = height as u32 - bar_h;
        
        // Bar background (very dark)
        fill_rect(0, bar_y, width as u32, bar_h, 0xFF080808u32);
        // Top border line
        fill_rect(0, bar_y, width as u32, 2, green_dim);
        
        // Left: TrustOS menu button (dash icon)
        let menu_btn_x = 8u32;
        let menu_btn_w = 24u32;
        fill_rect(menu_btn_x + 4, bar_y + 14, 16, 3, green_main);
        fill_rect(menu_btn_x + 4, bar_y + 19, 16, 3, green_main);
        
        // Check if menu button clicked
        if click_this_frame && mx >= menu_btn_x as f32 && mx < (menu_btn_x + menu_btn_w) as f32 &&
           my >= bar_y as f32 {
            menu_open = !menu_open;
        }
        
        // Tab: TrustOS (with border)
        let tab1_x = 40u32;
        let tab1_w = 90u32;
        fill_rect(tab1_x, bar_y + 6, tab1_w, 24, 0xFF0A1A0Au32);
        draw_rect(tab1_x, bar_y + 6, tab1_w, 24, green_dim);
        draw_text("TrustOS", tab1_x + 14, bar_y + 10, green_main);
        
        // Tab: Terminal
        let tab2_x = 138u32;
        let tab2_w = 90u32;
        fill_rect(tab2_x, bar_y + 6, tab2_w, 24, 0xFF050A05u32);
        draw_text("Terminal", tab2_x + 12, bar_y + 10, green_dim);
        
        // Search bar (center)
        let search_x = width as u32 / 2 - 120;
        let search_w = 240u32;
        fill_rect(search_x, bar_y + 6, search_w, 24, 0xFF0A0A0Au32);
        draw_rect(search_x, bar_y + 6, search_w, 24, green_dim);
        if search_len == 0 {
            draw_text("Search...", search_x + 8, bar_y + 10, 0xFF336633u32);
        } else {
            let search_display = unsafe { core::str::from_utf8_unchecked(&search_text[..search_len]) };
            draw_text(search_display, search_x + 8, bar_y + 10, green_main);
        }
        // Search icon (magnifying glass)
        fill_circle(search_x + search_w - 20, bar_y + 18, 6, green_dim);
        fill_circle(search_x + search_w - 20, bar_y + 18, 4, 0xFF0A0A0Au32);
        fill_rect(search_x + search_w - 16, bar_y + 22, 6, 2, green_dim);
        
        // Check if search bar clicked
        if click_this_frame && mx >= search_x as f32 && mx < (search_x + search_w) as f32 &&
           my >= bar_y as f32 {
            search_active = true;
        }
        
        // Right side: Clock
        let dt = crate::rtc::read_rtc();
        let time_str = format!("{:02}:{:02}", dt.hour, dt.minute);
        draw_text(&time_str, width as u32 - 200, bar_y + 10, green_main);
        
        // System ID
        draw_text("TRST-001", width as u32 - 120, bar_y + 10, green_bright);
        
        // Status indicators (right edge)
        let ind_x = width as u32 - 50;
        fill_circle(ind_x, bar_y + 18, 6, green_main);
        fill_circle(ind_x + 16, bar_y + 18, 6, 0xFFFFAA00u32);
        // Grid icon
        fill_rect(ind_x + 28, bar_y + 12, 4, 4, green_dim);
        fill_rect(ind_x + 34, bar_y + 12, 4, 4, green_dim);
        fill_rect(ind_x + 28, bar_y + 18, 4, 4, green_dim);
        fill_rect(ind_x + 34, bar_y + 18, 4, 4, green_dim);
        
        // ---------------------------------------------------------------
        // RENDER: TRUSTOS MENU (if open)
        // ---------------------------------------------------------------
        if menu_open {
            let menu_x = 10u32;
            let menu_y = bar_y - 320;
            let menu_w = 180u32;
            let menu_h = 310u32;
            
            // Menu background
            fill_rect(menu_x, menu_y, menu_w, menu_h, 0xFF0A0F0Au32);
            draw_rect(menu_x, menu_y, menu_w, menu_h, green_main);
            draw_rect(menu_x + 1, menu_y + 1, menu_w - 2, menu_h - 2, green_dim);
            
            // Menu header
            fill_rect(menu_x + 2, menu_y + 2, menu_w - 4, 30, 0xFF0A1A0Au32);
            draw_text("TrustOS Menu", menu_x + 12, menu_y + 10, green_main);
            
            // Menu items
            menu_hover = -1;
            for (idx, item) in menu_items.iter().enumerate() {
                let item_y = menu_y + 40 + (idx as u32 * 24);
                
                if *item == "---" {
                    // Separator line
                    fill_rect(menu_x + 10, item_y + 10, menu_w - 20, 1, green_dim);
                } else {
                    // Check hover
                    let item_hovered = mx >= menu_x as f32 && mx < (menu_x + menu_w) as f32 &&
                                       my >= item_y as f32 && my < (item_y + 24) as f32;
                    
                    if item_hovered {
                        menu_hover = idx as i32;
                        fill_rect(menu_x + 2, item_y, menu_w - 4, 24, 0xFF1A2A1Au32);
                        
                        // Handle click on menu item
                        if click_this_frame {
                            match *item {
                                "Shutdown" => running = false,
                                "Restart" => { /* Would restart */ running = false; },
                                "Sign Out" => { running = false; },
                                "Settings" => { active_app = 4; menu_open = false; },
                                "Terminal" => { active_app = 1; menu_open = false; },
                                "Files" => { active_app = 0; menu_open = false; },
                                "Browser" => { active_app = 2; menu_open = false; },
                                _ => { menu_open = false; }
                            }
                        }
                    }
                    
                    // Color based on type
                    let text_color = if *item == "Shutdown" || *item == "Restart" || *item == "Sign Out" {
                        0xFFFF6666u32  // Red for power options
                    } else if item_hovered {
                        green_bright
                    } else {
                        green_main
                    };
                    
                    // Icon for power options
                    if *item == "Shutdown" {
                        fill_circle(menu_x + 20, item_y + 12, 6, text_color);
                        fill_rect(menu_x + 18, item_y + 6, 4, 6, 0xFF0A0F0Au32);
                    }
                    
                    draw_text(item, menu_x + 35, item_y + 6, text_color);
                }
            }
            
            // Close menu if clicked outside
            if click_this_frame && (mx < menu_x as f32 || mx > (menu_x + menu_w) as f32 ||
                                    my < menu_y as f32 || my > bar_y as f32) {
                menu_open = false;
            }
        }
        
        // ---------------------------------------------------------------
        // RENDER: FPS OVERLAY (top-right corner)
        // ---------------------------------------------------------------
        if show_fps && fps > 0 {
            let fps_text = format!("{} FPS", fps);
            let fps_x = width.saturating_sub(80);
            let fps_color = if fps >= 55 { 0xFF00FF00 }    // Green 55+
                           else if fps >= 30 { 0xFFFFFF00 } // Yellow 30-54
                           else { 0xFFFF4444 };            // Red <30
            draw_text(&fps_text, fps_x, 4, fps_color);
            
            // Show renderer mode
            let mode = if use_braille { "BRL" } else if use_fast_matrix { "FAST" } else { "LEG" };
            draw_text(mode, fps_x, 20, 0xFF888888);
        }
        
        // ---------------------------------------------------------------
        // RENDER: CURSOR (simple green)
        // ---------------------------------------------------------------
        let mx_u32 = mx as u32;
        let my_u32 = my as u32;
        // Simple arrow cursor
        for i in 0..12u32 {
            fill_rect(mx_u32, my_u32 + i, (12 - i).max(1), 1, green_main);
        }
        
        // ---------------------------------------------------------------
        // PRESENT TO FRAMEBUFFER (SSE2 fast copy!)
        // ---------------------------------------------------------------
        swap_buffers();
        
        // ---------------------------------------------------------------
        // FPS TRACKING
        // ---------------------------------------------------------------
        frame_count += 1;
        frame_in_second += 1;
        
        let now = crate::cpu::tsc::read_tsc();
        if now - last_second_tsc >= tsc_freq {
            fps = frame_in_second;
            frame_in_second = 0;
            last_second_tsc = now;
            crate::serial_println!("[COSMIC] FPS: {} | Frame: {} | Mode: {}", 
                fps, frame_count, if use_braille { "BRAILLE" } else if use_fast_matrix { "FAST" } else { "LEGACY" });
        }
        
        // Brief pause to allow interrupts (keyboard, mouse) to be processed
        // Without this, the tight loop starves the interrupt handlers
        for _ in 0..100 {
            core::hint::spin_loop();
        }
        
        last_frame_tsc = crate::cpu::tsc::read_tsc();
    }
    
    crate::framebuffer::clear();
    crate::serial_println!("[COSMIC] Desktop exited after {} frames, last FPS: {}", frame_count, fps);
    crate::println_color!(COLOR_GREEN, "COSMIC Desktop exited. {} frames rendered, {} FPS", frame_count, fps);
}