//! TrustChess 3D — Matrix-style 3D chess board renderer
//!
//! Features:
//! - Isometric 3D board with perspective projection
//! - Low-poly 3D chess pieces (filled + wireframe outline)
//! - Orbital camera with auto-rotation
//! - Move history panel + material score bar
//! - Reuses ChessState engine from chess.rs for all game logic

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::formula3d::{V3, fast_sin, fast_cos, fast_sqrt};
use crate::chess::{
    ChessState, GamePhase, EMPTY,
};

// ═══════════════════════════════════════════════════════════════
// 3D CHESS STATE
// ═══════════════════════════════════════════════════════════════

pub struct Chess3DState {
    pub chess: ChessState,
    // Camera
    pub cam_angle_y: f32,    // Orbital angle around Y axis
    pub cam_angle_x: f32,    // Tilt angle
    pub cam_dist: f32,       // Distance from center
    pub auto_rotate: bool,   // Auto-rotate camera
    pub frame: u32,
    // Render dimensions
    pub buf_w: usize,
    pub buf_h: usize,
    // Interaction
    pub hover_sq: Option<usize>,   // Square under mouse
    pub anim_piece: Option<usize>, // Piece being animated
    pub anim_t: f32,               // Animation progress 0..1
    pub anim_from_3d: V3,          // Animation start position
    pub anim_to_3d: V3,            // Animation end position
    // Mouse drag rotation
    pub drag_rotating: bool,       // Currently dragging to rotate camera
    pub drag_last_x: i32,          // Last mouse X during drag
    pub drag_last_y: i32,          // Last mouse Y during drag
}

impl Chess3DState {
    pub fn new() -> Self {
        Chess3DState {
            chess: ChessState::new(),
            cam_angle_y: 0.0,       // Azimuth: 0 = behind white's side
            cam_angle_x: 0.65,      // Elevation: ~37° — classic 3D chess view
            cam_dist: 18.0,         // Telephoto distance — minimal distortion
            auto_rotate: false,
            frame: 0,
            buf_w: 0,
            buf_h: 0,
            hover_sq: None,
            anim_piece: None,
            anim_t: 0.0,
            anim_from_3d: V3 { x: 0.0, y: 0.0, z: 0.0 },
            anim_to_3d: V3 { x: 0.0, y: 0.0, z: 0.0 },
            drag_rotating: false,
            drag_last_x: 0,
            drag_last_y: 0,
        }
    }

    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT};

        match key {
            // Camera controls
            b'a' | b'A' => self.cam_angle_y -= 0.12,
            b'd' | b'D' => self.cam_angle_y += 0.12,
            b'w' | b'W' => { self.cam_angle_x = (self.cam_angle_x + 0.08).min(1.40); },
            b's' | b'S' => { self.cam_angle_x = (self.cam_angle_x - 0.08).max(0.15); },
            b'z' | b'Z' => { self.cam_dist = (self.cam_dist - 0.8).max(8.0); },
            b'x' | b'X' => { self.cam_dist = (self.cam_dist + 0.8).min(35.0); },
            b'o' | b'O' => self.auto_rotate = !self.auto_rotate,
            // Forward chess keys
            _ => self.chess.handle_key(key),
        }
    }

    /// Handle mouse click — either select a piece or start drag rotation
    /// mx, my are relative to the content area, w/h are content dimensions
    pub fn handle_click(&mut self, mx: i32, my: i32, content_w: i32, content_h: i32) {
        // The render fills the entire content area
        let w = content_w as usize;
        let h = content_h as usize;
        if mx < 0 || my < 0 || mx >= content_w || my >= content_h {
            return;
        }

        // Try to find which square was clicked by projecting all 64 squares
        // and finding the closest one to the click point
        let mut best_sq: Option<usize> = None;
        let mut best_dist = 999999.0f32;

        for row in 0..8u32 {
            for col in 0..8u32 {
                let sq = (row * 8 + col) as usize;
                let (sx, sy, _) = self.project_square_center(row, col, w, h);
                let dx = (sx - mx) as f32;
                let dy = (sy - my) as f32;
                let dist = dx * dx + dy * dy;
                if dist < best_dist && dist < (40.0 * 40.0) {
                    best_dist = dist;
                    best_sq = Some(sq);
                }
            }
        }

        if let Some(sq) = best_sq {
            let col = (sq % 8) as i32;
            let row = (sq / 8) as i32;
            self.chess.handle_mouse_click(col, row);
        } else {
            // Clicked outside board — start drag rotation
            self.drag_rotating = true;
            self.drag_last_x = mx;
            self.drag_last_y = my;
        }
    }

    /// Handle mouse move — rotate camera if dragging
    pub fn handle_mouse_move(&mut self, mx: i32, my: i32) {
        if self.drag_rotating {
            let dx = mx - self.drag_last_x;
            let dy = my - self.drag_last_y;
            // Horizontal drag → rotate around Y axis (left = board turns left)
            self.cam_angle_y += dx as f32 * 0.01;
            // Vertical drag → tilt angle (up = look more from above)
            self.cam_angle_x = (self.cam_angle_x - dy as f32 * 0.008).clamp(0.15, 1.40);
            self.drag_last_x = mx;
            self.drag_last_y = my;
        }
    }

    /// Handle mouse release — stop drag rotation
    pub fn handle_mouse_release(&mut self) {
        self.drag_rotating = false;
    }

    /// Handle scroll wheel — zoom in/out
    pub fn handle_scroll(&mut self, delta: i8) {
        if delta > 0 {
            // Scroll up → zoom in
            self.cam_dist = (self.cam_dist - 0.8).max(8.0);
        } else if delta < 0 {
            // Scroll down → zoom out
            self.cam_dist = (self.cam_dist + 0.8).min(35.0);
        }
    }

    /// Project a board square center to screen coordinates
    fn project_square_center(&self, row: u32, col: u32, w: usize, h: usize) -> (i32, i32, f32) {
        let world = board_square_to_world(row as i32, col as i32);
        self.project_point(world, w, h)
    }

    /// Project a 3D point to screen coordinates using proper look-at camera
    fn project_point(&self, p: V3, w: usize, h: usize) -> (i32, i32, f32) {
        // Camera in spherical coords: elevation=cam_angle_x, azimuth=cam_angle_y
        let ce = fast_cos(self.cam_angle_x);
        let se = fast_sin(self.cam_angle_x);
        let ca = fast_cos(self.cam_angle_y);
        let sa = fast_sin(self.cam_angle_y);

        // Camera world position
        let cpx = self.cam_dist * ce * sa;
        let cpy = self.cam_dist * se;
        let cpz = self.cam_dist * ce * ca;

        // Forward = normalize(origin - camera) = -camera/dist
        let inv_d = 1.0 / self.cam_dist;
        let fw_x = -cpx * inv_d;
        let fw_y = -cpy * inv_d;
        let fw_z = -cpz * inv_d;

        // Right = normalize(cross(forward, world_up(0,1,0))) = normalize(-fw_z, 0, fw_x)
        let rr_x = -fw_z;
        let rr_z = fw_x;
        let rlen = fast_sqrt(rr_x * rr_x + rr_z * rr_z);
        if rlen < 0.0001 {
            return (w as i32 / 2, h as i32 / 2, 0.0);
        }
        let r_x = rr_x / rlen;
        let r_z = rr_z / rlen;

        // Up = cross(right, forward)
        let u_x = -r_z * fw_y;
        let u_y = r_z * fw_x - r_x * fw_z;
        let u_z = r_x * fw_y;

        // Point relative to camera
        let dx = p.x - cpx;
        let dy = p.y - cpy;
        let dz = p.z - cpz;

        // Camera-space coordinates
        let cam_x = dx * r_x + dz * r_z;
        let cam_y = dx * u_x + dy * u_y + dz * u_z;
        let cam_z = dx * fw_x + dy * fw_y + dz * fw_z;

        if cam_z < 0.5 {
            return (w as i32 / 2, h as i32 / 2, cam_z);
        }

        // Perspective projection (telephoto FOV)
        let fov = w.min(h) as f32 * 1.6;
        let sx = (cam_x / cam_z * fov) as i32 + w as i32 / 2;
        let sy = (-cam_y / cam_z * fov) as i32 + h as i32 / 2;
        (sx, sy, cam_z)
    }

    pub fn tick(&mut self) {
        self.frame += 1;
        if self.auto_rotate {
            self.cam_angle_y += 0.005;
        }
        // Advance piece animation
        if self.anim_piece.is_some() {
            self.anim_t += 0.06;
            if self.anim_t >= 1.0 {
                self.anim_piece = None;
                self.anim_t = 0.0;
            }
        }
    }

    /// Main render function — renders the 3D chess scene into a buffer
    pub fn render(&mut self, out_buf: &mut [u32], w: usize, h: usize) {
        if w < 100 || h < 100 { return; }

        // Clear output buffer — dark matrix green
        for px in out_buf.iter_mut() {
            *px = 0xFF050808;
        }
        
        self.buf_w = w;
        self.buf_h = h;

        // ── Render board reflection (below board) ──
        self.render_board_reflection(out_buf, w, h);

        // ── Render board ──
        self.render_board(out_buf, w, h);

        // ── Board labels a-h / 1-8 ──
        self.render_board_labels(out_buf, w, h);

        // ── Shadows under pieces ──
        self.render_piece_shadows(out_buf, w, h);

        // ── Render pieces ──
        self.render_pieces(out_buf, w, h);
        
        // ── HUD overlay ──
        self.render_hud(out_buf, w, h);
    }

    fn render_board(&self, buf: &mut [u32], w: usize, h: usize) {
        // Draw the 8x8 board as filled quads (two triangles per square)
        // Render back-to-front (painter's algorithm) based on camera angle

        // Determine render order based on camera direction
        // Sort squares by depth for painter's algorithm
        let mut squares: [(u32, u32, f32); 64] = [(0, 0, 0.0); 64];
        for row in 0..8u32 {
            for col in 0..8u32 {
                let idx = (row * 8 + col) as usize;
                let center = board_square_to_world(row as i32, col as i32);
                let cam_space = self.to_camera_space(center);
                squares[idx] = (row, col, cam_space.z);
            }
        }

        // Sort by Z (far first for painter's)
        for i in 1..64 {
            let mut j = i;
            while j > 0 && squares[j].2 > squares[j - 1].2 {
                squares.swap(j, j - 1);
                j -= 1;
            }
        }

        for &(row, col, _) in &squares {
            let sq = (row * 8 + col) as usize;
            let is_light = (row + col) % 2 == 0;

            // Determine square color
            let mut base_color = if is_light { 0xFF3D6B3D } else { 0xFF1A3E1A };

            // Highlight selected piece
            if self.chess.selected == Some(sq) {
                base_color = 0xFF7AAA2A;
            }
            // Highlight valid moves
            if self.chess.valid_moves.contains(&sq) {
                base_color = if is_light { 0xFF4AAA4A } else { 0xFF2A8A2A };
            }
            // Highlight last move
            if self.chess.last_move_from == Some(sq) || self.chess.last_move_to == Some(sq) {
                base_color = if is_light { 0xFF6A8A3A } else { 0xFF4A6A2A };
            }
            // Highlight cursor
            if self.chess.cursor == sq {
                base_color = 0xFF00CC55;
            }

            // Get 4 corners of the square in 3D
            let c0 = board_corner_to_world(row as i32, col as i32);
            let c1 = board_corner_to_world(row as i32, col as i32 + 1);
            let c2 = board_corner_to_world(row as i32 + 1, col as i32 + 1);
            let c3 = board_corner_to_world(row as i32 + 1, col as i32);

            // Project to screen
            let (sx0, sy0, _) = self.project_point(c0, w, h);
            let (sx1, sy1, _) = self.project_point(c1, w, h);
            let (sx2, sy2, _) = self.project_point(c2, w, h);
            let (sx3, sy3, _) = self.project_point(c3, w, h);

            // Draw as two triangles
            fill_triangle_solid(buf, w, h, sx0, sy0, sx1, sy1, sx2, sy2, base_color);
            fill_triangle_solid(buf, w, h, sx0, sy0, sx2, sy2, sx3, sy3, base_color);

            // Draw grid lines (darker)
            let edge_color = 0xFF0A1A0A;
            draw_line_buf(buf, w, h, sx0, sy0, sx1, sy1, edge_color);
            draw_line_buf(buf, w, h, sx1, sy1, sx2, sy2, edge_color);
            draw_line_buf(buf, w, h, sx2, sy2, sx3, sy3, edge_color);
            draw_line_buf(buf, w, h, sx3, sy3, sx0, sy0, edge_color);

            // Valid move indicator dot
            if self.chess.valid_moves.contains(&sq) && self.chess.board[sq] == EMPTY {
                let (cx, cy, _) = self.project_square_center(row, col, w, h);
                fill_circle_buf(buf, w, h, cx, cy, 4, 0xFF00FF66);
            }
        }

        // Draw board edge (thickness) — draw side faces for depth
        self.render_board_edge(buf, w, h);
    }

    /// Render board labels (a-h along columns, 1-8 along rows)
    fn render_board_labels(&self, buf: &mut [u32], w: usize, h: usize) {
        let label_color = 0xFF55AA55;
        let sq_size = 0.8;

        // Column labels: a-h below the board (row = 8 edge, slightly below)
        for col in 0..8 {
            let letter = (b'a' + col as u8) as char;
            let pos = V3 {
                x: (col as f32 - 3.5) * sq_size,
                y: -0.05,
                z: (8.0 - 4.0) * sq_size + 0.3, // just past row 8 edge
            };
            let (sx, sy, sz) = self.project_point(pos, w, h);
            if sz > 0.5 {
                draw_char_buf(buf, w, h, sx - 4, sy - 8, letter, label_color);
            }
        }

        // Row labels: 1-8 along the left edge of the board
        for row in 0..8 {
            let digit = (b'8' - row as u8) as char; // row 0 = rank 8
            let pos = V3 {
                x: (0.0 - 4.0) * sq_size - 0.3, // just past col 0 edge
                y: -0.05,
                z: (row as f32 - 3.5) * sq_size,
            };
            let (sx, sy, sz) = self.project_point(pos, w, h);
            if sz > 0.5 {
                draw_char_buf(buf, w, h, sx - 4, sy - 8, digit, label_color);
            }
        }
    }

    /// Render shadows (dark ellipses) under each piece on the board
    fn render_piece_shadows(&self, buf: &mut [u32], w: usize, h: usize) {
        let shadow_color = 0x40000000u32; // Semi-transparent black idea
        for sq in 0..64 {
            let piece = self.chess.board[sq];
            if piece == EMPTY { continue; }

            let row = sq / 8;
            let col = sq % 8;
            let center = board_square_to_world(row as i32, col as i32);
            let (sx, sy, sz) = self.project_point(center, w, h);
            if sz < 0.5 { continue; }

            // Draw shadow as a small ellipse (flattened circle) darkening existing pixels
            let radius = (w.min(h) as f32 * 0.012 * (8.0 / sz)).max(2.0) as i32;
            let ry = (radius as f32 * 0.5) as i32; // Flatten vertically for perspective
            for dy in -ry..=ry {
                for dx in -radius..=radius {
                    let nx = dx as f32 / radius as f32;
                    let ny = dy as f32 / ry as f32;
                    if nx * nx + ny * ny > 1.0 { continue; }

                    let px = sx + dx;
                    let py = sy + dy;
                    if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                        let idx = py as usize * w + px as usize;
                        if idx < buf.len() {
                            // Darken existing pixel
                            let existing = buf[idx];
                            let r = ((existing >> 16) & 0xFF) * 3 / 4;
                            let g = ((existing >> 8) & 0xFF) * 3 / 4;
                            let b = (existing & 0xFF) * 3 / 4;
                            buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                        }
                    }
                }
            }
        }
    }

    /// Render a subtle reflection of the board below it
    fn render_board_reflection(&self, buf: &mut [u32], w: usize, h: usize) {
        let reflect_depth = 0.35; // How far below the board the reflection extends
        let reflect_alpha = 0.15; // Reflection brightness factor (very subtle)

        // Render reflected board squares (mirrored below y=0)
        for row in 0..8u32 {
            for col in 0..8u32 {
                let is_light = (row + col) % 2 == 0;
                let base_color: u32 = if is_light { 0xFF3D6B3D } else { 0xFF1A3E1A };

                // Get corners, mirror Y to negative and fade
                let c0 = board_corner_to_world(row as i32, col as i32);
                let c1 = board_corner_to_world(row as i32, col as i32 + 1);
                let c2 = board_corner_to_world(row as i32 + 1, col as i32 + 1);
                let c3 = board_corner_to_world(row as i32 + 1, col as i32);

                // Mirror below the board surface
                let mirror = |p: V3| V3 { x: p.x, y: -reflect_depth - p.y * 0.3, z: p.z };

                let (sx0, sy0, _) = self.project_point(mirror(c0), w, h);
                let (sx1, sy1, _) = self.project_point(mirror(c1), w, h);
                let (sx2, sy2, _) = self.project_point(mirror(c2), w, h);
                let (sx3, sy3, _) = self.project_point(mirror(c3), w, h);

                // Dim the color significantly for reflection
                let r = (((base_color >> 16) & 0xFF) as f32 * reflect_alpha) as u32;
                let g = (((base_color >> 8) & 0xFF) as f32 * reflect_alpha) as u32;
                let b = ((base_color & 0xFF) as f32 * reflect_alpha) as u32;
                let refl_color = 0xFF000000 | (r << 16) | (g << 8) | b;

                fill_triangle_solid(buf, w, h, sx0, sy0, sx1, sy1, sx2, sy2, refl_color);
                fill_triangle_solid(buf, w, h, sx0, sy0, sx2, sy2, sx3, sy3, refl_color);
            }
        }
    }

    fn render_board_edge(&self, buf: &mut [u32], w: usize, h: usize) {
        let thickness = 0.15;
        let edge_color = 0xFF0A200A;

        // Four edges of the board — project the bottom edges
        let corners_top = [
            board_corner_to_world(0, 0),
            board_corner_to_world(0, 8),
            board_corner_to_world(8, 8),
            board_corner_to_world(8, 0),
        ];
        let corners_bot: [V3; 4] = [
            V3 { x: corners_top[0].x, y: corners_top[0].y - thickness, z: corners_top[0].z },
            V3 { x: corners_top[1].x, y: corners_top[1].y - thickness, z: corners_top[1].z },
            V3 { x: corners_top[2].x, y: corners_top[2].y - thickness, z: corners_top[2].z },
            V3 { x: corners_top[3].x, y: corners_top[3].y - thickness, z: corners_top[3].z },
        ];

        // Draw 4 side faces
        for i in 0..4 {
            let j = (i + 1) % 4;
            let (t0x, t0y, _) = self.project_point(corners_top[i], w, h);
            let (t1x, t1y, _) = self.project_point(corners_top[j], w, h);
            let (b0x, b0y, _) = self.project_point(corners_bot[i], w, h);
            let (b1x, b1y, _) = self.project_point(corners_bot[j], w, h);

            fill_triangle_solid(buf, w, h, t0x, t0y, t1x, t1y, b1x, b1y, edge_color);
            fill_triangle_solid(buf, w, h, t0x, t0y, b1x, b1y, b0x, b0y, edge_color);
        }
    }

    fn render_pieces(&self, buf: &mut [u32], w: usize, h: usize) {
        // Collect all pieces with their depth for painter's sort
        struct PieceRender {
            sq: usize,
            piece: i8,
            pos: V3,
            depth: f32,
        }

        let mut pieces: Vec<PieceRender> = Vec::new();

        for sq in 0..64 {
            let piece = self.chess.board[sq];
            if piece == EMPTY { continue; }

            let row = sq / 8;
            let col = sq % 8;
            let pos = board_square_to_world(row as i32, col as i32);

            // Compute depth using look-at camera
            let cam_space = self.to_camera_space(pos);

            pieces.push(PieceRender { sq, piece, pos, depth: cam_space.z });
        }

        // Sort far-to-near
        for i in 1..pieces.len() {
            let mut j = i;
            while j > 0 && pieces[j].depth > pieces[j - 1].depth {
                pieces.swap(j, j - 1);
                j -= 1;
            }
        }

        // Render each piece
        for pr in &pieces {
            let is_white = pr.piece > 0;
            let abs_type = if pr.piece < 0 { -pr.piece } else { pr.piece };

            let mesh = get_piece_mesh(abs_type);
            let base_color = if is_white { 0xFFE0D8C8 } else { 0xFF2A2A2A };
            let edge_color = if is_white { 0xFF1A1A1A } else { 0xFF888888 };

            // Render filled piece with flat shading
            self.render_piece_mesh(buf, w, h, &mesh, pr.pos, base_color, edge_color);
        }
    }

    fn render_piece_mesh(&self, buf: &mut [u32], w: usize, h: usize,
                         mesh: &PieceMesh, pos: V3, base_color: u32, edge_color: u32) {
        // Transform all vertices: translate to board position, then camera transform
        let mut screen_pts: Vec<(i32, i32)> = Vec::with_capacity(mesh.vertices.len());
        let mut depths: Vec<f32> = Vec::with_capacity(mesh.vertices.len());

        for v in &mesh.vertices {
            let world = V3 {
                x: pos.x + v.x,
                y: pos.y + v.y,
                z: pos.z + v.z,
            };
            let (sx, sy, sz) = self.project_point(world, w, h);
            screen_pts.push((sx, sy));
            depths.push(sz);
        }

        // Render faces with flat shading (painter's sort)
        if let Some(ref faces) = mesh.faces {
            struct FaceDepth {
                idx: usize,
                avg_z: f32,
                brightness: f32,
            }

            let light_dir = V3 { x: -0.3, y: -0.8, z: -0.5 };
            let light_len = fast_sqrt(light_dir.x * light_dir.x + light_dir.y * light_dir.y + light_dir.z * light_dir.z);
            let light = V3 { x: light_dir.x / light_len, y: light_dir.y / light_len, z: light_dir.z / light_len };

            let mut visible: Vec<FaceDepth> = Vec::new();

            for (i, &(a, b, c)) in faces.iter().enumerate() {
                if a >= mesh.vertices.len() || b >= mesh.vertices.len() || c >= mesh.vertices.len() { continue; }

                let va = V3 { x: pos.x + mesh.vertices[a].x, y: pos.y + mesh.vertices[a].y, z: pos.z + mesh.vertices[a].z };
                let vb = V3 { x: pos.x + mesh.vertices[b].x, y: pos.y + mesh.vertices[b].y, z: pos.z + mesh.vertices[b].z };
                let vc = V3 { x: pos.x + mesh.vertices[c].x, y: pos.y + mesh.vertices[c].y, z: pos.z + mesh.vertices[c].z };

                // Camera-space transform for normal calculation
                let ta = self.to_camera_space(va);
                let tb = self.to_camera_space(vb);
                let tc = self.to_camera_space(vc);

                let e1 = V3 { x: tb.x - ta.x, y: tb.y - ta.y, z: tb.z - ta.z };
                let e2 = V3 { x: tc.x - ta.x, y: tc.y - ta.y, z: tc.z - ta.z };
                let nx = e1.y * e2.z - e1.z * e2.y;
                let ny = e1.z * e2.x - e1.x * e2.z;
                let nz = e1.x * e2.y - e1.y * e2.x;
                let nlen = fast_sqrt(nx * nx + ny * ny + nz * nz);
                if nlen < 0.0001 { continue; }
                let n = V3 { x: nx / nlen, y: ny / nlen, z: nz / nlen };

                // Backface cull
                if n.z > 0.0 { continue; }

                let ndotl = -(n.x * light.x + n.y * light.y + n.z * light.z);
                let brightness = 0.3 + 0.7 * ndotl.max(0.0);

                let avg_z = (ta.z + tb.z + tc.z) / 3.0;
                visible.push(FaceDepth { idx: i, avg_z, brightness });
            }

            // Far first
            for i in 1..visible.len() {
                let mut j = i;
                while j > 0 && visible[j].avg_z > visible[j - 1].avg_z {
                    visible.swap(j, j - 1);
                    j -= 1;
                }
            }

            let base_r = (base_color >> 16) & 0xFF;
            let base_g = (base_color >> 8) & 0xFF;
            let base_b = base_color & 0xFF;

            for fd in &visible {
                let (a, b, c) = faces[fd.idx];
                if a >= screen_pts.len() || b >= screen_pts.len() || c >= screen_pts.len() { continue; }
                let (sx0, sy0) = screen_pts[a];
                let (sx1, sy1) = screen_pts[b];
                let (sx2, sy2) = screen_pts[c];

                let r = ((base_r as f32 * fd.brightness) as u32).min(255);
                let g = ((base_g as f32 * fd.brightness) as u32).min(255);
                let b_c = ((base_b as f32 * fd.brightness) as u32).min(255);
                let shaded = 0xFF000000 | (r << 16) | (g << 8) | b_c;

                fill_triangle_solid(buf, w, h, sx0, sy0, sx1, sy1, sx2, sy2, shaded);
            }
        }

        // Wireframe edges on top
        if let Some(ref edges) = mesh.edges {
            for &(a, b) in edges {
                if a >= screen_pts.len() || b >= screen_pts.len() { continue; }
                let (x0, y0) = screen_pts[a];
                let (x1, y1) = screen_pts[b];
                draw_line_buf(buf, w, h, x0, y0, x1, y1, edge_color);
            }
        }
    }

    fn to_camera_space(&self, p: V3) -> V3 {
        let ce = fast_cos(self.cam_angle_x);
        let se = fast_sin(self.cam_angle_x);
        let ca = fast_cos(self.cam_angle_y);
        let sa = fast_sin(self.cam_angle_y);

        let cpx = self.cam_dist * ce * sa;
        let cpy = self.cam_dist * se;
        let cpz = self.cam_dist * ce * ca;

        let inv_d = 1.0 / self.cam_dist;
        let fw_x = -cpx * inv_d;
        let fw_y = -cpy * inv_d;
        let fw_z = -cpz * inv_d;

        let rr_x = -fw_z;
        let rr_z = fw_x;
        let rlen = fast_sqrt(rr_x * rr_x + rr_z * rr_z);
        if rlen < 0.0001 {
            return V3 { x: 0.0, y: 0.0, z: self.cam_dist };
        }
        let r_x = rr_x / rlen;
        let r_z = rr_z / rlen;

        let u_x = -r_z * fw_y;
        let u_y = r_z * fw_x - r_x * fw_z;
        let u_z = r_x * fw_y;

        let dx = p.x - cpx;
        let dy = p.y - cpy;
        let dz = p.z - cpz;

        V3 {
            x: dx * r_x + dz * r_z,
            y: dx * u_x + dy * u_y + dz * u_z,
            z: dx * fw_x + dy * fw_y + dz * fw_z,
        }
    }

    /// Render HUD overlay (turn indicator, score, controls)
    fn render_hud(&self, buf: &mut [u32], w: usize, h: usize) {
        // Turn indicator
        let turn_text = if self.chess.white_turn { "WHITE" } else { "BLACK" };
        let phase_text = match self.chess.phase {
            GamePhase::Playing => "",
            GamePhase::Check => " CHECK!",
            GamePhase::Checkmate => " CHECKMATE!",
            GamePhase::Stalemate => " STALEMATE",
            GamePhase::Promotion => " PROMOTION",
        };
        
        // Draw turn text at top-left
        let turn_color = if self.chess.white_turn { 0xFFE0E0E0 } else { 0xFF40FF40 };
        hud_text(buf, w, h, 8, 8, turn_text, turn_color);
        if !phase_text.is_empty() {
            let phase_color = match self.chess.phase {
                GamePhase::Check => 0xFFFF4444,
                GamePhase::Checkmate => 0xFFFF2222,
                GamePhase::Stalemate => 0xFFFFAA00,
                GamePhase::Promotion => 0xFF4488FF,
                _ => 0xFF40FF40,
            };
            hud_text(buf, w, h, 8 + turn_text.len() as i32 * 8, 8, phase_text, phase_color);
        }
        
        // Score bar at top-right
        let score = self.chess.material_score();
        let score_str = format!("{:+}", score);
        let score_color = if score > 0 { 0xFFE0E0E0 } else if score < 0 { 0xFF40FF40 } else { 0xFF888888 };
        hud_text(buf, w, h, w as i32 - 60, 8, &score_str, score_color);
        
        // Controls hint at bottom
        let hint = "WASD:Cam Scroll/ZX:Zoom O:Rotate Drag:Orbit";
        hud_text(buf, w, h, 8, h as i32 - 14, hint, 0xFF336633);
        
        // Move history (last 5 moves)
        let history = &self.chess.move_history;
        let start = if history.len() > 5 { history.len() - 5 } else { 0 };
        for (i, mv) in history[start..].iter().enumerate() {
            let y_pos = 24 + i as i32 * 12;
            hud_text(buf, w, h, 8, y_pos, mv, 0xFF448844);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// COORDINATE HELPERS
// ═══════════════════════════════════════════════════════════════

/// Convert board row/col (0-7) to world XZ position (centered on origin)
fn board_square_to_world(row: i32, col: i32) -> V3 {
    let sq_size = 0.8;
    V3 {
        x: (col as f32 - 3.5) * sq_size,
        y: 0.0,
        z: (row as f32 - 3.5) * sq_size,
    }
}

/// Convert board corner to world position (corners are row 0-8, col 0-8)
fn board_corner_to_world(row: i32, col: i32) -> V3 {
    let sq_size = 0.8;
    V3 {
        x: (col as f32 - 4.0) * sq_size,
        y: 0.0,
        z: (row as f32 - 4.0) * sq_size,
    }
}

// ═══════════════════════════════════════════════════════════════
// PIECE MESHES — Low-poly 3D models
// ═══════════════════════════════════════════════════════════════

pub struct PieceMesh {
    pub vertices: Vec<V3>,
    pub edges: Option<Vec<(usize, usize)>>,
    pub faces: Option<Vec<(usize, usize, usize)>>,
}

/// Generate a low-poly piece mesh for a given piece type (1-6)
fn get_piece_mesh(piece_type: i8) -> PieceMesh {
    match piece_type {
        1 => mesh_pawn(),
        2 => mesh_knight(),
        3 => mesh_bishop(),
        4 => mesh_rook(),
        5 => mesh_queen(),
        6 => mesh_king(),
        _ => mesh_pawn(),
    }
}

/// Generate a cylinder/cone segment as vertices
/// Returns vertices forming a ring at height y with radius r, n_sides segments
fn ring(y: f32, r: f32, n: u32) -> Vec<V3> {
    let mut pts = Vec::with_capacity(n as usize);
    for i in 0..n {
        let angle = (i as f32 / n as f32) * 6.2831853;
        pts.push(V3 {
            x: fast_cos(angle) * r,
            y,
            z: fast_sin(angle) * r,
        });
    }
    pts
}

/// Create faces connecting two rings (each with `n` vertices)
/// Ring 0 starts at vertex index `base0`, ring 1 at `base1`
fn connect_rings(base0: usize, base1: usize, n: usize) -> (Vec<(usize, usize, usize)>, Vec<(usize, usize)>) {
    let mut faces = Vec::new();
    let mut edges = Vec::new();
    for i in 0..n {
        let j = (i + 1) % n;
        let a0 = base0 + i;
        let a1 = base0 + j;
        let b0 = base1 + i;
        let b1 = base1 + j;
        faces.push((a0, a1, b1));
        faces.push((a0, b1, b0));
        edges.push((a0, a1));
        edges.push((a0, b0));
    }
    edges.push((base0, base1)); // Close vertical edge
    (faces, edges)
}

/// Create faces connecting a ring to a single point (cone top/bottom)
fn cone_cap(base: usize, apex: usize, n: usize) -> (Vec<(usize, usize, usize)>, Vec<(usize, usize)>) {
    let mut faces = Vec::new();
    let mut edges = Vec::new();
    for i in 0..n {
        let j = (i + 1) % n;
        faces.push((base + i, base + j, apex));
        edges.push((base + i, apex));
    }
    (faces, edges)
}

const SIDES: u32 = 8; // Octagonal cross-section for low-poly look
const SCALE: f32 = 0.22; // Piece scale relative to board square

fn mesh_pawn() -> PieceMesh {
    let s = SCALE;
    let n = SIDES as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    // Base
    let r0 = ring(0.0, 0.35 * s, SIDES);     // base bottom
    let b0 = verts.len(); verts.extend_from_slice(&r0);
    let r1 = ring(0.08, 0.35 * s, SIDES);     // base top
    let b1 = verts.len(); verts.extend_from_slice(&r1);

    // Stem
    let r2 = ring(0.10, 0.20 * s, SIDES);     // stem bottom
    let b2 = verts.len(); verts.extend_from_slice(&r2);
    let r3 = ring(0.28, 0.15 * s, SIDES);     // stem top
    let b3 = verts.len(); verts.extend_from_slice(&r3);

    // Head (sphere approximation)
    let r4 = ring(0.30, 0.20 * s, SIDES);     // head bottom
    let b4 = verts.len(); verts.extend_from_slice(&r4);
    let r5 = ring(0.38, 0.22 * s, SIDES);     // head mid
    let b5 = verts.len(); verts.extend_from_slice(&r5);
    let r6 = ring(0.44, 0.18 * s, SIDES);     // head top
    let b6 = verts.len(); verts.extend_from_slice(&r6);

    // Top point
    let apex = verts.len();
    verts.push(V3 { x: 0.0, y: 0.48 * 1.0, z: 0.0 });

    // Connect rings
    let (f, e) = connect_rings(b0, b1, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b1, b2, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b2, b3, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b3, b4, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b4, b5, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b5, b6, n); faces.extend(f); edges.extend(e);
    let (f, e) = cone_cap(b6, apex, n); faces.extend(f); edges.extend(e);

    // Bottom cap
    let bottom_center = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = cone_cap(b0, bottom_center, n); faces.extend(f); edges.extend(e);

    PieceMesh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn mesh_knight() -> PieceMesh {
    let s = SCALE;
    let n = SIDES as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    // Base
    let r0 = ring(0.0, 0.38 * s, SIDES);
    let b0 = verts.len(); verts.extend_from_slice(&r0);
    let r1 = ring(0.10, 0.35 * s, SIDES);
    let b1 = verts.len(); verts.extend_from_slice(&r1);

    // Neck
    let r2 = ring(0.20, 0.20 * s, SIDES);
    let b2 = verts.len(); verts.extend_from_slice(&r2);
    let r3 = ring(0.35, 0.22 * s, SIDES);
    let b3 = verts.len(); verts.extend_from_slice(&r3);

    // Head (offset forward for horse profile)
    let mut head_ring = ring(0.42, 0.20 * s, SIDES);
    for v in head_ring.iter_mut() { v.z -= 0.06; } // offset forward
    let b4 = verts.len(); verts.extend_from_slice(&head_ring);

    // Muzzle (extended forward)
    let mut muzzle = ring(0.47, 0.12 * s, SIDES);
    for v in muzzle.iter_mut() { v.z -= 0.12; }
    let b5 = verts.len(); verts.extend_from_slice(&muzzle);

    // Ear tip
    let ear = verts.len();
    verts.push(V3 { x: 0.0, y: 0.55, z: -0.02 });

    // Connect
    let (f, e) = connect_rings(b0, b1, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b1, b2, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b2, b3, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b3, b4, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b4, b5, n); faces.extend(f); edges.extend(e);
    let (f, e) = cone_cap(b5, ear, n); faces.extend(f); edges.extend(e);

    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = cone_cap(b0, bottom, n); faces.extend(f); edges.extend(e);

    PieceMesh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn mesh_bishop() -> PieceMesh {
    let s = SCALE;
    let n = SIDES as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    // Base
    let r0 = ring(0.0, 0.38 * s, SIDES);
    let b0 = verts.len(); verts.extend_from_slice(&r0);
    let r1 = ring(0.10, 0.35 * s, SIDES);
    let b1 = verts.len(); verts.extend_from_slice(&r1);

    // Column
    let r2 = ring(0.12, 0.18 * s, SIDES);
    let b2 = verts.len(); verts.extend_from_slice(&r2);
    let r3 = ring(0.32, 0.15 * s, SIDES);
    let b3 = verts.len(); verts.extend_from_slice(&r3);

    // Mitre bulge
    let r4 = ring(0.34, 0.22 * s, SIDES);
    let b4 = verts.len(); verts.extend_from_slice(&r4);
    let r5 = ring(0.42, 0.20 * s, SIDES);
    let b5 = verts.len(); verts.extend_from_slice(&r5);
    let r6 = ring(0.50, 0.12 * s, SIDES);
    let b6 = verts.len(); verts.extend_from_slice(&r6);

    // Point
    let apex = verts.len();
    verts.push(V3 { x: 0.0, y: 0.58, z: 0.0 });

    let (f, e) = connect_rings(b0, b1, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b1, b2, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b2, b3, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b3, b4, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b4, b5, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b5, b6, n); faces.extend(f); edges.extend(e);
    let (f, e) = cone_cap(b6, apex, n); faces.extend(f); edges.extend(e);

    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = cone_cap(b0, bottom, n); faces.extend(f); edges.extend(e);

    PieceMesh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn mesh_rook() -> PieceMesh {
    let s = SCALE;
    let n = SIDES as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    // Base
    let r0 = ring(0.0, 0.40 * s, SIDES);
    let b0 = verts.len(); verts.extend_from_slice(&r0);
    let r1 = ring(0.10, 0.38 * s, SIDES);
    let b1 = verts.len(); verts.extend_from_slice(&r1);

    // Body (tall cylinder)
    let r2 = ring(0.12, 0.25 * s, SIDES);
    let b2 = verts.len(); verts.extend_from_slice(&r2);
    let r3 = ring(0.38, 0.25 * s, SIDES);
    let b3 = verts.len(); verts.extend_from_slice(&r3);

    // Top rim (wider for crenellation effect)
    let r4 = ring(0.40, 0.32 * s, SIDES);
    let b4 = verts.len(); verts.extend_from_slice(&r4);
    let r5 = ring(0.50, 0.32 * s, SIDES);
    let b5 = verts.len(); verts.extend_from_slice(&r5);

    // Top cap flat
    let top_center = verts.len();
    verts.push(V3 { x: 0.0, y: 0.50, z: 0.0 });

    let (f, e) = connect_rings(b0, b1, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b1, b2, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b2, b3, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b3, b4, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b4, b5, n); faces.extend(f); edges.extend(e);
    let (f, e) = cone_cap(b5, top_center, n); faces.extend(f); edges.extend(e);

    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = cone_cap(b0, bottom, n); faces.extend(f); edges.extend(e);

    PieceMesh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn mesh_queen() -> PieceMesh {
    let s = SCALE;
    let n = SIDES as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    // Base
    let r0 = ring(0.0, 0.40 * s, SIDES);
    let b0 = verts.len(); verts.extend_from_slice(&r0);
    let r1 = ring(0.10, 0.38 * s, SIDES);
    let b1 = verts.len(); verts.extend_from_slice(&r1);

    // Body
    let r2 = ring(0.12, 0.22 * s, SIDES);
    let b2 = verts.len(); verts.extend_from_slice(&r2);
    let r3 = ring(0.30, 0.18 * s, SIDES);
    let b3 = verts.len(); verts.extend_from_slice(&r3);

    // Body widening
    let r4 = ring(0.35, 0.25 * s, SIDES);
    let b4 = verts.len(); verts.extend_from_slice(&r4);

    // Crown rim
    let r5 = ring(0.42, 0.28 * s, SIDES);
    let b5 = verts.len(); verts.extend_from_slice(&r5);

    // Crown narrowing
    let r6 = ring(0.52, 0.15 * s, SIDES);
    let b6 = verts.len(); verts.extend_from_slice(&r6);

    // Crown jewel
    let apex = verts.len();
    verts.push(V3 { x: 0.0, y: 0.60, z: 0.0 });

    let (f, e) = connect_rings(b0, b1, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b1, b2, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b2, b3, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b3, b4, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b4, b5, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b5, b6, n); faces.extend(f); edges.extend(e);
    let (f, e) = cone_cap(b6, apex, n); faces.extend(f); edges.extend(e);

    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = cone_cap(b0, bottom, n); faces.extend(f); edges.extend(e);

    PieceMesh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn mesh_king() -> PieceMesh {
    let s = SCALE;
    let n = SIDES as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    // Base
    let r0 = ring(0.0, 0.42 * s, SIDES);
    let b0 = verts.len(); verts.extend_from_slice(&r0);
    let r1 = ring(0.10, 0.40 * s, SIDES);
    let b1 = verts.len(); verts.extend_from_slice(&r1);

    // Body
    let r2 = ring(0.12, 0.24 * s, SIDES);
    let b2 = verts.len(); verts.extend_from_slice(&r2);
    let r3 = ring(0.32, 0.20 * s, SIDES);
    let b3 = verts.len(); verts.extend_from_slice(&r3);

    // Crown rim
    let r4 = ring(0.35, 0.28 * s, SIDES);
    let b4 = verts.len(); verts.extend_from_slice(&r4);
    let r5 = ring(0.42, 0.26 * s, SIDES);
    let b5 = verts.len(); verts.extend_from_slice(&r5);

    // Crown narrowing
    let r6 = ring(0.48, 0.15 * s, SIDES);
    let b6 = verts.len(); verts.extend_from_slice(&r6);

    let (f, e) = connect_rings(b0, b1, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b1, b2, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b2, b3, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b3, b4, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b4, b5, n); faces.extend(f); edges.extend(e);
    let (f, e) = connect_rings(b5, b6, n); faces.extend(f); edges.extend(e);

    // Cross on top of king
    // Vertical bar
    let cross_w = 0.04;
    let cross_h = 0.14;
    let cross_y = 0.50;
    let cv0 = verts.len();
    verts.push(V3 { x: -cross_w, y: cross_y, z: -cross_w });
    verts.push(V3 { x:  cross_w, y: cross_y, z: -cross_w });
    verts.push(V3 { x:  cross_w, y: cross_y + cross_h, z: -cross_w });
    verts.push(V3 { x: -cross_w, y: cross_y + cross_h, z: -cross_w });
    verts.push(V3 { x: -cross_w, y: cross_y, z:  cross_w });
    verts.push(V3 { x:  cross_w, y: cross_y, z:  cross_w });
    verts.push(V3 { x:  cross_w, y: cross_y + cross_h, z:  cross_w });
    verts.push(V3 { x: -cross_w, y: cross_y + cross_h, z:  cross_w });

    // Cross vertical bar faces
    faces.push((cv0, cv0+1, cv0+2)); faces.push((cv0, cv0+2, cv0+3));      // front
    faces.push((cv0+4, cv0+6, cv0+5)); faces.push((cv0+4, cv0+7, cv0+6));  // back
    faces.push((cv0, cv0+3, cv0+7)); faces.push((cv0, cv0+7, cv0+4));      // left
    faces.push((cv0+1, cv0+5, cv0+6)); faces.push((cv0+1, cv0+6, cv0+2));  // right
    faces.push((cv0+3, cv0+2, cv0+6)); faces.push((cv0+3, cv0+6, cv0+7));  // top

    // Edges for cross
    edges.push((cv0, cv0+1)); edges.push((cv0+1, cv0+2)); edges.push((cv0+2, cv0+3)); edges.push((cv0+3, cv0));
    edges.push((cv0+4, cv0+5)); edges.push((cv0+5, cv0+6)); edges.push((cv0+6, cv0+7)); edges.push((cv0+7, cv0+4));
    edges.push((cv0, cv0+4)); edges.push((cv0+1, cv0+5)); edges.push((cv0+2, cv0+6)); edges.push((cv0+3, cv0+7));

    // Horizontal crossbar
    let bar_y = cross_y + cross_h * 0.55;
    let bar_w = 0.10;
    let ch0 = verts.len();
    verts.push(V3 { x: -bar_w, y: bar_y, z: -cross_w });
    verts.push(V3 { x:  bar_w, y: bar_y, z: -cross_w });
    verts.push(V3 { x:  bar_w, y: bar_y + 0.03, z: -cross_w });
    verts.push(V3 { x: -bar_w, y: bar_y + 0.03, z: -cross_w });
    verts.push(V3 { x: -bar_w, y: bar_y, z:  cross_w });
    verts.push(V3 { x:  bar_w, y: bar_y, z:  cross_w });
    verts.push(V3 { x:  bar_w, y: bar_y + 0.03, z:  cross_w });
    verts.push(V3 { x: -bar_w, y: bar_y + 0.03, z:  cross_w });

    faces.push((ch0, ch0+1, ch0+2)); faces.push((ch0, ch0+2, ch0+3));
    faces.push((ch0+4, ch0+6, ch0+5)); faces.push((ch0+4, ch0+7, ch0+6));
    faces.push((ch0, ch0+3, ch0+7)); faces.push((ch0, ch0+7, ch0+4));
    faces.push((ch0+1, ch0+5, ch0+6)); faces.push((ch0+1, ch0+6, ch0+2));
    faces.push((ch0+3, ch0+2, ch0+6)); faces.push((ch0+3, ch0+6, ch0+7));

    edges.push((ch0, ch0+1)); edges.push((ch0+1, ch0+2)); edges.push((ch0+2, ch0+3)); edges.push((ch0+3, ch0));
    edges.push((ch0+4, ch0+5)); edges.push((ch0+5, ch0+6)); edges.push((ch0+6, ch0+7)); edges.push((ch0+7, ch0+4));

    // Top cap
    let top_ring_center = verts.len();
    verts.push(V3 { x: 0.0, y: 0.48, z: 0.0 });
    let (f, e) = cone_cap(b6, top_ring_center, n); faces.extend(f); edges.extend(e);

    // Bottom cap
    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = cone_cap(b0, bottom, n); faces.extend(f); edges.extend(e);

    PieceMesh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

// ═══════════════════════════════════════════════════════════════
// DRAWING HELPERS (local to chess3d, operate on buffer)
// ═══════════════════════════════════════════════════════════════

fn fill_triangle_solid(buf: &mut [u32], w: usize, h: usize,
                       mut x0: i32, mut y0: i32,
                       mut x1: i32, mut y1: i32,
                       mut x2: i32, mut y2: i32,
                       color: u32) {
    // Sort by Y
    if y0 > y1 { core::mem::swap(&mut x0, &mut x1); core::mem::swap(&mut y0, &mut y1); }
    if y1 > y2 { core::mem::swap(&mut x1, &mut x2); core::mem::swap(&mut y1, &mut y2); }
    if y0 > y1 { core::mem::swap(&mut x0, &mut x1); core::mem::swap(&mut y0, &mut y1); }

    let total_h = y2 - y0;
    if total_h == 0 { return; }

    let y_start = y0.max(0);
    let y_end = y2.min(h as i32 - 1);

    for y in y_start..=y_end {
        let second_half = y >= y1;
        let seg_h = if second_half { y2 - y1 } else { y1 - y0 };

        let t_total = (y - y0) as f32 / total_h as f32;
        let xa = x0 as f32 + (x2 - x0) as f32 * t_total;

        let xb = if seg_h == 0 {
            xa
        } else if second_half {
            let t_seg = (y - y1) as f32 / seg_h as f32;
            x1 as f32 + (x2 - x1) as f32 * t_seg
        } else {
            let t_seg = (y - y0) as f32 / seg_h as f32;
            x0 as f32 + (x1 - x0) as f32 * t_seg
        };

        let mut left = xa as i32;
        let mut right = xb as i32;
        if left > right { core::mem::swap(&mut left, &mut right); }

        left = left.max(0);
        right = right.min(w as i32 - 1);

        let row = y as usize * w;
        for x in left..=right {
            let idx = row + x as usize;
            if idx < buf.len() {
                buf[idx] = color;
            }
        }
    }
}

use crate::draw_utils::{draw_line as draw_line_buf, fill_circle as fill_circle_buf};

/// Draw text into a buffer using 8x16 bitmap font
fn hud_text(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, text: &str, color: u32) {
    let mut cx = x;
    for ch in text.chars() {
        draw_char_buf(buf, w, h, cx, y, ch, color);
        cx += 8;
    }
}

/// Simple 8x16 character renderer into buffer
fn draw_char_buf(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, c: char, color: u32) {
    let glyph = crate::framebuffer::font::get_glyph(c);
    for row in 0..16 {
        let bits = glyph[row];
        for col in 0..8 {
            if bits & (0x80 >> col) != 0 {
                let px = x + col as i32;
                let py = y + row as i32;
                if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                    let idx = py as usize * w + px as usize;
                    if idx < buf.len() {
                        buf[idx] = color;
                    }
                }
            }
        }
    }
}
