//! WOA Effects — Visual particle effects for movement actions
//!
//! Three effect systems:
//! - Wind lines (jump) — animated swoosh lines rising from player
//! - Dust puffs (slide/land) — small particle clouds at feet
//! - Ghost halo (double jump) — transparent afterimage + halo ring

use alloc::vec::Vec;

/// Maximum particles alive at once (keeps heap bounded)
const MAX_PARTICLES: usize = 48;

/// Single particle
#[derive(Clone, Copy)]
struct Particle {
    x: i32,
    y: i32,
    vx: i32,      // velocity ×16 (fixed-point 4.4)
    vy: i32,
    life: u8,     // frames remaining
    max_life: u8, // initial life (for fade calc)
    color: u32,
    kind: ParticleKind,
}

#[derive(Clone, Copy, PartialEq)]
enum ParticleKind {
    WindLine,
    Dust,
    Halo,
}

/// Effect manager — call spawn_* then update()+draw() each frame
pub struct Effects {
    particles: Vec<Particle>,
    /// Ghost afterimage data for double-jump (stores last position)
    ghost_x: i32,
    ghost_y: i32,
    ghost_timer: u8, // frames remaining for ghost display
}

impl Effects {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
            ghost_x: 0,
            ghost_y: 0,
            ghost_timer: 0,
        }
    }

    // ── Jump: Wind Lines ──
    // Thin vertical/diagonal streaks rising from below player
    pub fn spawn_jump_wind(&mut self, player_x: i32, player_y: i32, sprite_w: u32, sprite_h: u32) {
        let cx = player_x + (sprite_w / 2) as i32;
        let bottom = player_y + sprite_h as i32;

        // 6 wind lines in arc below player
        for i in 0..6 {
            if self.particles.len() >= MAX_PARTICLES { break; }
            let offset_x = (i as i32 - 3) * 3;
            // Small random-ish variation based on position
            let vary = ((cx + i as i32 * 7) % 3) as i32 - 1;
            self.particles.push(Particle {
                x: (cx + offset_x + vary) * 16, // fixed-point
                y: bottom * 16,
                vx: offset_x * 2,  // spread outward
                vy: -24 - (i as i32 % 3) * 8, // upward, varied speed
                life: 10 + (i % 4) as u8,
                max_life: 14,
                color: 0xFFCCDDFF, // pale blue-white
                kind: ParticleKind::WindLine,
            });
        }
    }

    // ── Slide: Dust Puffs ──
    // Small brown particles kicked behind player
    pub fn spawn_slide_dust(&mut self, player_x: i32, player_y: i32, sprite_h: u32, facing_right: bool) {
        let bottom = player_y + sprite_h as i32;
        let spawn_x = if facing_right { player_x } else { player_x + 20 };

        for i in 0..4 {
            if self.particles.len() >= MAX_PARTICLES { break; }
            let vary = ((spawn_x + i as i32 * 13) % 5) as i32 - 2;
            let dir = if facing_right { -1 } else { 1 };
            self.particles.push(Particle {
                x: spawn_x * 16,
                y: (bottom - 2 + vary) * 16,
                vx: dir * (12 + (i as i32 % 3) * 6),
                vy: -8 - (i as i32 % 2) * 8,
                life: 8 + (i % 3) as u8,
                max_life: 11,
                color: if i % 2 == 0 { 0xFF8B7355 } else { 0xFF6B5335 }, // brown tones
                kind: ParticleKind::Dust,
            });
        }
    }

    // ── Land: Dust burst (same as slide but symmetrical) ──
    pub fn spawn_land_dust(&mut self, player_x: i32, player_y: i32, sprite_w: u32, sprite_h: u32) {
        let cx = player_x + (sprite_w / 2) as i32;
        let bottom = player_y + sprite_h as i32;

        for i in 0..6 {
            if self.particles.len() >= MAX_PARTICLES { break; }
            let dir = if i % 2 == 0 { 1 } else { -1 };
            let spread = (i as i32 / 2 + 1) * 8;
            self.particles.push(Particle {
                x: cx * 16,
                y: bottom * 16,
                vx: dir * spread,
                vy: -12 - (i as i32 % 3) * 4,
                life: 8 + (i % 4) as u8,
                max_life: 12,
                color: if i % 3 == 0 { 0xFF9B8365 } else { 0xFF7B6345 },
                kind: ParticleKind::Dust,
            });
        }
    }

    // ── Double Jump: Ghost Halo ──
    // Transparent afterimage at current position + ring burst
    pub fn spawn_double_jump_halo(&mut self, player_x: i32, player_y: i32, sprite_w: u32, sprite_h: u32) {
        // Store ghost position
        self.ghost_x = player_x;
        self.ghost_y = player_y;
        self.ghost_timer = 12; // show ghost for 12 frames

        let cx = player_x + (sprite_w / 2) as i32;
        let cy = player_y + (sprite_h / 2) as i32;

        // Ring of particles expanding outward
        for i in 0..8 {
            if self.particles.len() >= MAX_PARTICLES { break; }
            // 8 directions (0, 45, 90, 135, 180, 225, 270, 315 degrees)
            let (dx, dy) = match i {
                0 => (16, 0),
                1 => (12, -12),
                2 => (0, -16),
                3 => (-12, -12),
                4 => (-16, 0),
                5 => (-12, 12),
                6 => (0, 16),
                7 => (12, 12),
                _ => (0, 0),
            };
            self.particles.push(Particle {
                x: cx * 16,
                y: cy * 16,
                vx: dx,
                vy: dy,
                life: 10,
                max_life: 10,
                color: 0xFF88CCFF, // light cyan-blue
                kind: ParticleKind::Halo,
            });
        }
    }

    /// Update all particles — call once per frame
    pub fn update(&mut self) {
        // Update ghost timer
        if self.ghost_timer > 0 {
            self.ghost_timer -= 1;
        }

        // Update particles
        for p in self.particles.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;

            // Gravity for dust
            if p.kind == ParticleKind::Dust {
                p.vy += 2; // gentle gravity
            }

            // Wind lines decelerate
            if p.kind == ParticleKind::WindLine {
                p.vy += 1; // slow down upward motion
            }

            if p.life > 0 {
                p.life -= 1;
            }
        }

        // Remove dead particles
        self.particles.retain(|p| p.life > 0);
    }

    /// Draw all effects to the renderer
    pub fn draw(&self, r: &mut super::renderer::Renderer, sprite_data: &[u32], sprite_w: u32, sprite_h: u32) {
        // Draw ghost afterimage (double jump)
        if self.ghost_timer > 0 {
            let alpha_factor = self.ghost_timer as u32;
            // Draw a faded version of the sprite
            for sy in 0..sprite_h {
                let dy = self.ghost_y + sy as i32;
                if dy < 0 || dy >= r.height() as i32 { continue; }
                for sx in 0..sprite_w {
                    let dx = self.ghost_x + sx as i32;
                    if dx < 0 || dx >= r.width() as i32 { continue; }
                    let idx = (sy * sprite_w + sx) as usize;
                    if idx < sprite_data.len() {
                        let px = sprite_data[idx];
                        if px & 0xFF000000 != 0 {
                            // Tint cyan-blue + fade based on timer
                            let pr = ((px >> 16) & 0xFF) * alpha_factor / 16;
                            let pg = ((px >> 8) & 0xFF) * alpha_factor / 16;
                            let pb = (px & 0xFF) * alpha_factor / 16;
                            // Shift toward blue
                            let br = (pr / 2) as u32;
                            let bg = (pg + 40 * alpha_factor / 12) as u32;
                            let bb = (pb + 80 * alpha_factor / 12) as u32;
                            let color = 0xFF000000
                                | (br.min(255) << 16)
                                | (bg.min(255) << 8)
                                | bb.min(255);
                            r.put_pixel(dx as u32, dy as u32, color);
                        }
                    }
                }
            }
        }

        // Draw particles
        for p in &self.particles {
            let px = p.x / 16;
            let py = p.y / 16;

            // Fade based on remaining life
            let fade = if p.max_life > 0 { (p.life as u32 * 255) / p.max_life as u32 } else { 255 };
            let cr = ((p.color >> 16) & 0xFF) * fade / 255;
            let cg = ((p.color >> 8) & 0xFF) * fade / 255;
            let cb = (p.color & 0xFF) * fade / 255;
            let faded = 0xFF000000 | (cr << 16) | (cg << 8) | cb;

            match p.kind {
                ParticleKind::WindLine => {
                    // Vertical 1×3 line
                    r.put_pixel(px as u32, py as u32, faded);
                    r.put_pixel(px as u32, (py + 1) as u32, faded);
                    r.put_pixel(px as u32, (py + 2) as u32, faded);
                }
                ParticleKind::Dust => {
                    // 2×2 puff
                    r.put_pixel(px as u32, py as u32, faded);
                    r.put_pixel((px + 1) as u32, py as u32, faded);
                    r.put_pixel(px as u32, (py + 1) as u32, faded);
                    r.put_pixel((px + 1) as u32, (py + 1) as u32, faded);
                }
                ParticleKind::Halo => {
                    // Single bright pixel
                    r.put_pixel(px as u32, py as u32, faded);
                }
            }
        }
    }

    /// Are there any active effects?
    pub fn is_active(&self) -> bool {
        !self.particles.is_empty() || self.ghost_timer > 0
    }
}
