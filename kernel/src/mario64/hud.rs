//! TrustMario64 — HUD (Heads-Up Display)
//! SM64-authentic: power meter, star count, coin count, lives
#![allow(dead_code)]

use super::physics::*;
use super::player::MarioState;
use super::tas::{TasEngine, TasMode};
use crate::math::{fast_sin, fast_cos};

// ======================== Mini Font (3x5 pixels, digits + letters) ========================

const FONT: [[u8; 5]; 36] = [
    // 0-9
    [0b111, 0b101, 0b101, 0b101, 0b111], // 0
    [0b010, 0b110, 0b010, 0b010, 0b111], // 1
    [0b111, 0b001, 0b111, 0b100, 0b111], // 2
    [0b111, 0b001, 0b111, 0b001, 0b111], // 3
    [0b101, 0b101, 0b111, 0b001, 0b001], // 4
    [0b111, 0b100, 0b111, 0b001, 0b111], // 5
    [0b111, 0b100, 0b111, 0b101, 0b111], // 6
    [0b111, 0b001, 0b010, 0b010, 0b010], // 7
    [0b111, 0b101, 0b111, 0b101, 0b111], // 8
    [0b111, 0b101, 0b111, 0b001, 0b111], // 9
    // A-Z
    [0b010, 0b101, 0b111, 0b101, 0b101], // A
    [0b110, 0b101, 0b110, 0b101, 0b110], // B
    [0b011, 0b100, 0b100, 0b100, 0b011], // C
    [0b110, 0b101, 0b101, 0b101, 0b110], // D
    [0b111, 0b100, 0b110, 0b100, 0b111], // E
    [0b111, 0b100, 0b110, 0b100, 0b100], // F
    [0b011, 0b100, 0b101, 0b101, 0b011], // G
    [0b101, 0b101, 0b111, 0b101, 0b101], // H
    [0b111, 0b010, 0b010, 0b010, 0b111], // I
    [0b001, 0b001, 0b001, 0b101, 0b010], // J
    [0b101, 0b110, 0b100, 0b110, 0b101], // K
    [0b100, 0b100, 0b100, 0b100, 0b111], // L
    [0b101, 0b111, 0b111, 0b101, 0b101], // M
    [0b101, 0b111, 0b111, 0b111, 0b101], // N
    [0b010, 0b101, 0b101, 0b101, 0b010], // O
    [0b110, 0b101, 0b110, 0b100, 0b100], // P
    [0b010, 0b101, 0b101, 0b111, 0b011], // Q
    [0b110, 0b101, 0b110, 0b101, 0b101], // R
    [0b011, 0b100, 0b010, 0b001, 0b110], // S
    [0b111, 0b010, 0b010, 0b010, 0b010], // T
    [0b101, 0b101, 0b101, 0b101, 0b111], // U
    [0b101, 0b101, 0b101, 0b101, 0b010], // V
    [0b101, 0b101, 0b111, 0b111, 0b101], // W
    [0b101, 0b101, 0b010, 0b101, 0b101], // X
    [0b101, 0b101, 0b010, 0b010, 0b010], // Y
    [0b111, 0b001, 0b010, 0b100, 0b111], // Z
];

fn char_index(c: u8) -> Option<usize> {
    match c {
        b'0'..=b'9' => Some((c - b'0') as usize),
        b'A'..=b'Z' => Some((c - b'A') as usize + 10),
        b'a'..=b'z' => Some((c - b'a') as usize + 10),
        _ => None,
    }
}

// ======================== Buffer Drawing ========================

fn put_pixel(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
        buf[y as usize * w + x as usize] = color;
    }
}

fn fill_rect_buf(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, rw: i32, rh: i32, color: u32) {
    for dy in 0..rh {
        for dx in 0..rw {
            put_pixel(buf, w, h, x + dx, y + dy, color);
        }
    }
}

fn draw_text(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, text: &[u8], color: u32, scale: i32) {
    let mut cx = x;
    for &ch in text {
        if ch == b' ' {
            cx += 4 * scale;
            continue;
        }
        if ch == b'x' || ch == b'X' {
            // Draw a small x (multiply sign)
            for s in 0..scale {
                put_pixel(buf, w, h, cx + s, y + s, color);
                put_pixel(buf, w, h, cx + 2 * scale - s, y + s, color);
                put_pixel(buf, w, h, cx + s, y + 4 * scale - s, color);
                put_pixel(buf, w, h, cx + 2 * scale - s, y + 4 * scale - s, color);
            }
            cx += 4 * scale;
            continue;
        }
        if let Some(idx) = char_index(ch) {
            let glyph = &FONT[idx];
            for row in 0..5 {
                for col in 0..3 {
                    if glyph[row] & (0b100 >> col) != 0 {
                        fill_rect_buf(buf, w, h, cx + col as i32 * scale, y + row as i32 * scale, scale, scale, color);
                    }
                }
            }
        }
        cx += 4 * scale;
    }
}

// ======================== Draw Number ========================

fn draw_number(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, num: u32, color: u32, scale: i32) {
    let mut digits = [0u8; 10];
    let mut n = num;
    let mut len = 0;
    if n == 0 {
        digits[0] = b'0';
        len = 1;
    } else {
        while n > 0 {
            digits[len] = b'0' + (n % 10) as u8;
            n /= 10;
            len += 1;
        }
        // Reverse
        for i in 0..len / 2 {
            digits.swap(i, len - 1 - i);
        }
    }
    draw_text(buf, w, h, x, y, &digits[..len], color, scale);
}

// ======================== HUD Renderer ========================

/// Draw the complete SM64 HUD
pub fn draw_hud(
    buf: &mut [u32], w: usize, h: usize,
    mario: &MarioState, tas: &TasEngine,
) {
    let s = if w > 400 { 2 } else { 1 }; // scale factor

    // === Power Meter (SM64's health pie chart) — top center ===
    draw_power_meter(buf, w, h, mario.hp, mario.max_hp, s);

    // === Star Counter — top left ===
    {
        let sx = 8 * s;
        let sy = 8 * s;
        // Star icon (small yellow diamond)
        draw_star_icon(buf, w, h, sx, sy, s);
        draw_text(buf, w, h, sx + 12 * s as i32, sy + 2 * s as i32, b"x", COLOR_HUD_WHITE, s);
        draw_number(buf, w, h, sx + 18 * s as i32, sy + 2 * s as i32, mario.stars as u32, COLOR_HUD_WHITE, s);
    }

    // === Coin Counter — top right ===
    {
        let cx = w as i32 - 60 * s as i32;
        let cy = 8 * s;
        // Coin icon (small yellow circle)
        draw_coin_icon(buf, w, h, cx, cy as i32, s);
        draw_text(buf, w, h, cx + 12 * s as i32, cy as i32 + 2 * s as i32, b"x", COLOR_HUD_WHITE, s);
        draw_number(buf, w, h, cx + 18 * s as i32, cy as i32 + 2 * s as i32, mario.coins as u32, COLOR_HUD_YELLOW, s);
    }

    // === Lives — bottom left ===
    {
        let lx = 8 * s as i32;
        let ly = h as i32 - 16 * s as i32;
        // Mario face (red circle)
        draw_mario_face(buf, w, h, lx, ly, s);
        draw_text(buf, w, h, lx + 12 * s as i32, ly + 2 * s as i32, b"x", COLOR_HUD_WHITE, s);
        draw_number(buf, w, h, lx + 18 * s as i32, ly + 2 * s as i32, mario.lives as u32, COLOR_HUD_WHITE, s);
    }

    // === TAS info panel ===
    if tas.show_info_panel {
        draw_tas_info(buf, w, h, mario, tas, s);
    }

    // === TAS mode indicator ===
    match tas.mode {
        TasMode::Normal => {},
        TasMode::Recording => {
            // Red dot blinking
            if (tas.frame / 15) % 2 == 0 {
                fill_rect_buf(buf, w, h, w as i32 / 2 - 30 * s as i32, h as i32 - 14 * s as i32, 6 * s as i32, 6 * s as i32, COLOR_HUD_RED);
            }
            draw_text(buf, w, h, w as i32 / 2 - 22 * s as i32, h as i32 - 12 * s as i32, b"REC", COLOR_HUD_RED, s);
        }
        TasMode::Replaying => {
            draw_text(buf, w, h, w as i32 / 2 - 30 * s as i32, h as i32 - 12 * s as i32, b"REPLAY", 0xFF00CCFF, s);
        }
        TasMode::FrameAdvance => {
            draw_text(buf, w, h, w as i32 / 2 - 30 * s as i32, h as i32 - 12 * s as i32, b"FRAME", COLOR_HUD_YELLOW, s);
            draw_number(buf, w, h, w as i32 / 2 + 5 * s as i32, h as i32 - 12 * s as i32, tas.frame as u32, COLOR_HUD_YELLOW, s);
        }
    }

    // === Input display (TAS standard) ===
    if tas.show_input_display {
        draw_input_display(buf, w, h, &tas.current_input, s);
    }
}

// ======================== SM64 Power Meter ========================

fn draw_power_meter(buf: &mut [u32], w: usize, h: usize, hp: u8, max_hp: u8, s: i32) {
    let cx = w as i32 / 2;
    let cy = 20 * s;
    let r = 10 * s;

    // Background circle
    draw_circle_filled(buf, w, h, cx, cy, r + 1, 0x80000000);

    // Health wedges (8 segments)
    for i in 0..max_hp as i32 {
        let color = if (i as u8) < hp {
            if hp > 5 { COLOR_HEALTH_FULL }
            else if hp > 2 { COLOR_HEALTH_MED }
            else { COLOR_HEALTH_LOW }
        } else {
            0xFF333333  // empty wedge
        };

        let angle_start = (i as f32 / max_hp as f32) * PI * 2.0 - HALF_PI;
        let angle_end = ((i + 1) as f32 / max_hp as f32) * PI * 2.0 - HALF_PI;

        draw_wedge(buf, w, h, cx, cy, r, angle_start, angle_end, color);
    }
}

fn draw_wedge(buf: &mut [u32], w: usize, h: usize, cx: i32, cy: i32, r: i32, a0: f32, a1: f32, color: u32) {
    let steps = 8;
    for step in 0..steps {
        let t0 = a0 + (a1 - a0) * step as f32 / steps as f32;
        let t1 = a0 + (a1 - a0) * (step + 1) as f32 / steps as f32;

        let x0 = cx + (fast_cos(t0) * r as f32) as i32;
        let y0 = cy + (fast_sin(t0) * r as f32) as i32;
        let x1 = cx + (fast_cos(t1) * r as f32) as i32;
        let y1 = cy + (fast_sin(t1) * r as f32) as i32;

        draw_line(buf, w, h, cx, cy, x0, y0, color);
        draw_line(buf, w, h, x0, y0, x1, y1, color);
    }
}

fn draw_circle_filled(buf: &mut [u32], w: usize, h: usize, cx: i32, cy: i32, r: i32, color: u32) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                put_pixel(buf, w, h, cx + dx, cy + dy, color);
            }
        }
    }
}

fn draw_line(buf: &mut [u32], w: usize, h: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        put_pixel(buf, w, h, x, y, color);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

// ======================== Icons ========================

fn draw_star_icon(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, s: i32) {
    // Simple 5-pointed star in yellow
    let pts = [
        (4, 0), (5, 3), (8, 3), (6, 5), (7, 8), (4, 6), (1, 8), (2, 5), (0, 3), (3, 3),
    ];
    for i in 0..5 {
        let (x0, y0) = pts[i * 2];
        let (x1, y1) = pts[i * 2 + 1];
        draw_line(buf, w, h, x + x0 * s, y + y0 * s, x + x1 * s, y + y1 * s, COLOR_STAR);
    }
    fill_rect_buf(buf, w, h, x + 3 * s, y + 2 * s, 3 * s, 4 * s, COLOR_STAR);
}

fn draw_coin_icon(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, s: i32) {
    draw_circle_filled(buf, w, h, x + 4 * s, y + 4 * s, 4 * s, COLOR_COIN);
    draw_circle_filled(buf, w, h, x + 4 * s, y + 4 * s, 2 * s, 0xFFEEAA00);
}

fn draw_mario_face(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, s: i32) {
    draw_circle_filled(buf, w, h, x + 4 * s, y + 4 * s, 4 * s, COLOR_MARIO_SKIN);
    fill_rect_buf(buf, w, h, x + 1 * s, y, 6 * s, 3 * s, COLOR_MARIO_RED); // cap
    fill_rect_buf(buf, w, h, x + 2 * s, y + 5 * s, 4 * s, 1 * s, COLOR_MARIO_BROWN); // mustache
}

// ======================== TAS Overlays ========================

fn draw_input_display(buf: &mut [u32], w: usize, h: usize, input: &super::tas::FrameInput, s: i32) {
    let bx = w as i32 - 50 * s as i32;
    let by = h as i32 - 40 * s as i32;

    // Stick position (analog display)
    let stick_size = 12 * s;
    fill_rect_buf(buf, w, h, bx, by, stick_size, stick_size, 0x60000000);
    let sx = bx + stick_size / 2 + (input.stick_x as i32 * stick_size / 256);
    let sy = by + stick_size / 2 - (input.stick_y as i32 * stick_size / 256);
    draw_circle_filled(buf, w, h, sx, sy, 2 * s, COLOR_HUD_WHITE);

    // Button display
    let btn_y = by + stick_size + 4 * s;
    let btn_s = 4 * s;
    let buttons = [
        (super::tas::BTN_A, b"A", 0xFF00CC00),
        (super::tas::BTN_B, b"B", COLOR_HUD_RED),
        (super::tas::BTN_Z, b"Z", 0xFF8888FF),
    ];
    for (i, &(mask, label, color)) in buttons.iter().enumerate() {
        let bxx = bx + i as i32 * (btn_s + 2 * s);
        let c = if input.buttons & mask != 0 { color } else { 0xFF333333 };
        fill_rect_buf(buf, w, h, bxx, btn_y, btn_s, btn_s, c);
        draw_text(buf, w, h, bxx + s, btn_y + s, label, COLOR_HUD_WHITE, 1);
    }
}

fn draw_tas_info(buf: &mut [u32], w: usize, h: usize, mario: &MarioState, tas: &TasEngine, s: i32) {
    let px = 4 * s as i32;
    let py = h as i32 / 2;
    let lh = 8 * s;

    // Semi-transparent background
    fill_rect_buf(buf, w, h, px - 2, py - 2, 80 * s as i32, 60 * s as i32, 0xA0000000);

    // Frame
    draw_text(buf, w, h, px, py, b"FRAME", COLOR_HUD_WHITE, s);
    draw_number(buf, w, h, px + 30 * s as i32, py, tas.frame as u32, COLOR_HUD_YELLOW, s);

    // Position
    let row1 = py + lh as i32;
    draw_text(buf, w, h, px, row1, b"POS", COLOR_HUD_WHITE, s);
    draw_number(buf, w, h, px + 20 * s as i32, row1, mario.pos.x.abs() as u32, COLOR_HUD_YELLOW, s);
    draw_number(buf, w, h, px + 40 * s as i32, row1, mario.pos.y.abs() as u32, COLOR_HUD_YELLOW, s);

    // Speed
    let row2 = py + lh as i32 * 2;
    draw_text(buf, w, h, px, row2, b"SPD", COLOR_HUD_WHITE, s);
    draw_number(buf, w, h, px + 20 * s as i32, row2, (mario.forward_vel * 10.0).abs() as u32, 0xFF00CCFF, s);

    // Action
    let row3 = py + lh as i32 * 3;
    draw_text(buf, w, h, px, row3, b"ACT", COLOR_HUD_WHITE, s);
    let act_name = match mario.action {
        super::player::Action::Idle => b"IDLE" as &[u8],
        super::player::Action::Walking => b"WALK",
        super::player::Action::Running => b"RUN",
        super::player::Action::Jumping => b"JUMP",
        super::player::Action::DoubleJump => b"DBL",
        super::player::Action::TripleJump => b"TRI",
        super::player::Action::LongJump => b"LONG",
        super::player::Action::Backflip => b"BFLP",
        super::player::Action::WallKick => b"WKIK",
        super::player::Action::GroundPound => b"GP",
        super::player::Action::Freefall => b"FALL",
        super::player::Action::Dive => b"DIVE",
        _ => b"???",
    };
    draw_text(buf, w, h, px + 20 * s as i32, row3, act_name, 0xFF00FF88, s);

    // Slot info
    let row4 = py + lh as i32 * 4;
    draw_text(buf, w, h, px, row4, b"SLOT", COLOR_HUD_WHITE, s);
    draw_number(buf, w, h, px + 25 * s as i32, row4, tas.active_slot as u32, COLOR_HUD_YELLOW, s);

    // Rewind buffer
    let row5 = py + lh as i32 * 5;
    draw_text(buf, w, h, px, row5, b"RWD", COLOR_HUD_WHITE, s);
    draw_number(buf, w, h, px + 20 * s as i32, row5, tas.rewind_count as u32, 0xFFCC88FF, s);
}
