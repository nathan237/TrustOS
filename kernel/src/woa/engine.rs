//! WOA Engine — Fixed 60fps game loop with TSC-based timing
//!
//! Internal resolution: 1280×800, sprites 128×128.

use super::{GameState, renderer, input, camera, sprites, effects};
use renderer::{INTERNAL_W, INTERNAL_H};

/// Target frame time in TSC cycles (computed at init)
const TARGET_FPS: u64 = 60;

/// Player state
struct Player {
    x: i32,
    y: i32,
    speed: i32,
    facing_right: bool,
}

/// Main game loop
pub fn run_game() {
    let mut render = renderer::Renderer::new(INTERNAL_W, INTERNAL_H);
    let mut inp = input::Input::new();
    let mut cam = camera::Camera::new(INTERNAL_W, INTERNAL_H);
    let mut fx = effects::Effects::new();
    let mut state = GameState::Menu;

    // Sprite buffers — 128×128 = 16384 pixels each
    let mut sprite_main = alloc::vec![0u32; sprites::SPRITE_PIXELS];
    sprite_main.copy_from_slice(&sprites::MILITANT_IDLE);

    // Flipped sprite buffer (facing left) — 128×128 = 16384 pixels
    let mut sprite_flipped = alloc::vec![0u32; sprites::SPRITE_PIXELS];
    sprites::flip_sprite_h(&sprite_main, sprites::SPRITE_W, sprites::SPRITE_H, &mut sprite_flipped);

    let mut player = Player {
        x: 576,   // ~centered (1280/2 - 64)
        y: 608,   // on ground (800 - 64 ground - 128 sprite)
        speed: 4,
        facing_right: true,
    };

    // TSC timing setup
    let tsc_freq = crate::cpu::tsc_frequency();
    let ticks_per_frame = if tsc_freq > 0 { tsc_freq / TARGET_FPS } else { 0 };
    let use_tsc = ticks_per_frame > 0;

    let mut frame_count: u64 = 0;
    let mut fps_timer = crate::time::uptime_ms();
    let mut last_fps_frame: u64 = 0;
    let mut fps_display: u32 = 60;

    crate::serial_println!("[WOA] Engine ready. TSC freq={}Hz, ticks/frame={}",
        tsc_freq, ticks_per_frame);
    crate::serial_println!("[WOA] Press any key to start. ESC to quit.");
    crate::shell::clear_interrupted();

    loop {
        let frame_start = if use_tsc { crate::cpu::tsc::read_tsc() } else { 0 };

        // ── Input ──
        inp.poll();

        // Exit check
        if inp.is_pressed(input::SC_ESC) || crate::shell::is_interrupted() {
            crate::shell::clear_interrupted();
            break;
        }

        // ── Update ──
        match state {
            GameState::Menu => {
                if inp.any_pressed() && !inp.is_pressed(input::SC_ESC) {
                    state = GameState::Playing;
                    crate::serial_println!("[WOA] -> Playing");
                }
            }
            GameState::Playing => {
                // Pause
                if inp.is_pressed(input::SC_P) {
                    state = GameState::Paused;
                    crate::serial_println!("[WOA] -> Paused");
                }

                // Movement (WASD + arrows)
                let left = inp.is_held(input::SC_LEFT) || inp.is_held(input::SC_A) || inp.is_held(input::SC_Q);
                let right = inp.is_held(input::SC_RIGHT) || inp.is_held(input::SC_D);
                let up = inp.is_held(input::SC_UP) || inp.is_held(input::SC_W) || inp.is_held(input::SC_Z);
                let down = inp.is_held(input::SC_DOWN) || inp.is_held(input::SC_S);

                // Track facing direction
                if left { player.facing_right = false; }
                if right { player.facing_right = true; }

                if left  { player.x -= player.speed; }
                if right { player.x += player.speed; }
                if up    { player.y -= player.speed; }
                if down  { player.y += player.speed; }

                // Speed boost with shift
                let effective_speed = if inp.is_held(input::SC_LSHIFT) { player.speed * 2 } else { player.speed };
                if left || right || up || down {
                    // Re-apply with correct speed (undo default, apply boost)
                    if effective_speed != player.speed {
                        if left  { player.x -= player.speed; } // apply extra
                        if right { player.x += player.speed; }
                        if up    { player.y -= player.speed; }
                        if down  { player.y += player.speed; }
                    }
                }

                // Clamp to world bounds (24×24 sprite)
                player.x = player.x.clamp(0, (INTERNAL_W - sprites::SPRITE_W) as i32);
                player.y = player.y.clamp(0, (INTERNAL_H - sprites::SPRITE_H) as i32);

                // Effect triggers (demo: space=jump wind, E=slide dust, tab=double-jump halo)
                if inp.is_pressed(input::SC_SPACE) {
                    fx.spawn_jump_wind(player.x, player.y, sprites::SPRITE_W, sprites::SPRITE_H);
                }
                if inp.is_pressed(input::SC_E) {
                    fx.spawn_slide_dust(player.x, player.y, sprites::SPRITE_H, player.facing_right);
                }
                if inp.is_pressed(input::SC_TAB) {
                    fx.spawn_double_jump_halo(player.x, player.y, sprites::SPRITE_W, sprites::SPRITE_H);
                }

                cam.follow(player.x + 64, player.y + 64);
            }
            GameState::Paused => {
                if inp.is_pressed(input::SC_P) || inp.is_pressed(input::SC_SPACE) {
                    state = GameState::Playing;
                    crate::serial_println!("[WOA] -> Playing");
                }
            }
            _ => {}
        }

        // ── Update effects ──
        fx.update();

        // ── Render ──
        render.clear(0xFF1A1A2E); // dark navy background

        match state {
            GameState::Menu => {
                render_menu(&mut render, frame_count);
            }
            GameState::Playing => {
                render_game(&mut render, &player, fps_display, &fx, &sprite_main, &sprite_flipped);
            }
            GameState::Paused => {
                render_game(&mut render, &player, fps_display, &fx, &sprite_main, &sprite_flipped);
                // Darken overlay
                render_pause_overlay(&mut render);
            }
            _ => {}
        }

        // Blit to native framebuffer
        render.present();

        // ── End-of-frame input update ──
        inp.end_frame();
        frame_count += 1;

        // FPS counter (1s intervals via uptime_ms)
        let now_ms = crate::time::uptime_ms();
        if now_ms.saturating_sub(fps_timer) >= 1000 {
            fps_display = (frame_count - last_fps_frame) as u32;
            last_fps_frame = frame_count;
            fps_timer = now_ms;
        }

        // ── Frame timing (TSC busy-wait) ──
        if use_tsc {
            while crate::cpu::tsc::read_tsc().saturating_sub(frame_start) < ticks_per_frame {
                core::hint::spin_loop();
            }
        } else {
            // Fallback: ~16ms via uptime_ms (coarse but functional)
            let ms_start = now_ms;
            while crate::time::uptime_ms().saturating_sub(ms_start) < 16 {
                core::hint::spin_loop();
            }
        }
    }

    crate::serial_println!("[WOA] Stopped. {} frames total.", frame_count);
}

// ── Render helpers ──

fn render_menu(r: &mut renderer::Renderer, frame: u64) {
    // Title — block letters spelling "WOA" (scaled 2×)
    let title_y: u32 = 80;
    let color_title = 0xFFFFCC00; // gold
    let color_sub = 0xFF888888;   // gray

    // "W O A" centered at 1280 width
    draw_block_char(r, 544, title_y, 'W', color_title);
    draw_block_char(r, 608, title_y, 'O', color_title);
    draw_block_char(r, 672, title_y, 'A', color_title);

    // Subtitle — flashing indicator
    if (frame / 30) % 2 == 0 {
        let sub_y = title_y + 100;
        r.fill_rect(560, sub_y, 160, 4, color_sub);
    }

    // Version
    let ver_y = INTERNAL_H - 30;
    r.fill_rect(12, ver_y, 120, 3, 0xFF444444);
}

fn render_game(r: &mut renderer::Renderer, player: &Player, fps: u32, fx: &effects::Effects, sprite_main: &[u32], sprite_flipped: &[u32]) {
    // Ground — 64px tall
    r.fill_rect(0, INTERNAL_H - 64, INTERNAL_W, 64, 0xFF2D5016);
    // Ground detail — grass tufts
    for i in (0..INTERNAL_W).step_by(20) {
        r.fill_rect(i, INTERNAL_H - 70, 3, 6, 0xFF4CAF50);
        r.fill_rect(i + 8, INTERNAL_H - 68, 2, 4, 0xFF66BB6A);
        r.fill_rect(i + 14, INTERNAL_H - 69, 2, 5, 0xFF3D8B26);
    }

    // Player — 128×128 militant ant sprite
    let sprite_data = if player.facing_right {
        &sprite_main[..]
    } else {
        &sprite_flipped[..]
    };
    r.blit_sprite(player.x, player.y, sprites::SPRITE_W, sprites::SPRITE_H, sprite_data);

    // Effects (wind, dust, halo) — drawn on top
    fx.draw(r, sprite_data, sprites::SPRITE_W, sprites::SPRITE_H);

    // FPS indicator — top right, colored bar
    let fps_color = if fps >= 58 { 0xFF44FF44 } else if fps >= 30 { 0xFFFFFF00 } else { 0xFFFF4444 };
    let bar_w = ((fps as u32) * 2).min(120);
    r.fill_rect(INTERNAL_W - 140, 8, 120, 12, 0xFF222222);
    r.fill_rect(INTERNAL_W - 140, 8, bar_w, 12, fps_color);

    // Position indicator — bottom
    let indicator_x = (player.x as u32 * (INTERNAL_W - 16)) / INTERNAL_W;
    r.fill_rect(8, INTERNAL_H - 16, INTERNAL_W - 16, 8, 0xFF222222);
    r.fill_rect(8 + indicator_x, INTERNAL_H - 16, 8, 8, 0xFFFFCC00);
}

fn render_pause_overlay(r: &mut renderer::Renderer) {
    // Semi-transparent dark overlay (checkerboard pattern)
    for y in 0..INTERNAL_H {
        for x in 0..INTERNAL_W {
            if (x + y) % 2 == 0 {
                r.put_pixel(x, y, 0xFF000000);
            }
        }
    }
    // Pause indicator — two vertical bars (centered at 1280×800)
    r.fill_rect(600, 350, 16, 100, 0xFFFFFFFF);
    r.fill_rect(664, 350, 16, 100, 0xFFFFFFFF);
}

/// Draw a block letter (5×7 grid, each cell = 6×6 pixels for 640×400 res)
fn draw_block_char(r: &mut renderer::Renderer, x: u32, y: u32, ch: char, color: u32) {
    let bitmap: [u8; 7] = match ch {
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10010, 0b10001, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'D' => [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        _ => [0; 7],
    };
    for (row, &bits) in bitmap.iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) != 0 {
                let px = x + col * 6;
                let py = y + row as u32 * 6;
                r.fill_rect(px, py, 6, 6, color);
            }
        }
    }
}
