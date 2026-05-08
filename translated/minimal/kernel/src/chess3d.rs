








use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::formula3d::{V3, eu, hr, ra};
use crate::chess::{
    ChessState, GamePhase, Q,
};





pub struct Chess3DState {
    pub chess: ChessState,
    
    pub cam_angle_y: f32,    
    pub cam_angle_x: f32,    
    pub cam_dist: f32,       
    pub auto_rotate: bool,   
    pub frame: u32,
    
    pub buf_w: usize,
    pub buf_h: usize,
    
    pub hover_sq: Option<usize>,   
    pub anim_piece: Option<usize>, 
    pub anim_t: f32,               
    pub anim_from_3d: V3,          
    pub anim_to_3d: V3,            
    
    pub drag_rotating: bool,       
    pub drag_last_x: i32,          
    pub drag_last_y: i32,          
}

impl Chess3DState {
    pub fn new() -> Self {
        Chess3DState {
            chess: ChessState::new(),
            cam_angle_y: 0.0,       
            cam_angle_x: 0.65,      
            cam_dist: 18.0,         
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
        use crate::keyboard::{T_, S_, AI_, AJ_};

        match key {
            
            b'a' | b'A' => self.cam_angle_y -= 0.12,
            b'd' | b'D' => self.cam_angle_y += 0.12,
            b'w' | b'W' => { self.cam_angle_x = (self.cam_angle_x + 0.08).min(1.40); },
            b's' | b'S' => { self.cam_angle_x = (self.cam_angle_x - 0.08).max(0.15); },
            b'z' | b'Z' => { self.cam_dist = (self.cam_dist - 0.8).max(8.0); },
            b'x' | b'X' => { self.cam_dist = (self.cam_dist + 0.8).min(35.0); },
            b'o' | b'O' => self.auto_rotate = !self.auto_rotate,
            
            _ => self.chess.handle_key(key),
        }
    }

    
    
    pub fn handle_click(&mut self, cg: i32, cr: i32, hy: i32, en: i32) {
        
        let w = hy as usize;
        let h = en as usize;
        if cg < 0 || cr < 0 || cg >= hy || cr >= en {
            return;
        }

        
        
        let mut hhk: Option<usize> = None;
        let mut hhh = 999999.0f32;

        for row in 0..8u32 {
            for col in 0..8u32 {
                let cu = (row * 8 + col) as usize;
                let (am, ak, _) = self.project_square_center(row, col, w, h);
                let dx = (am - cg) as f32;
                let ad = (ak - cr) as f32;
                let em = dx * dx + ad * ad;
                if em < hhh && em < (40.0 * 40.0) {
                    hhh = em;
                    hhk = Some(cu);
                }
            }
        }

        if let Some(cu) = hhk {
            let col = (cu % 8) as i32;
            let row = (cu / 8) as i32;
            self.chess.handle_mouse_click(col, row);
        } else {
            
            self.drag_rotating = true;
            self.drag_last_x = cg;
            self.drag_last_y = cr;
        }
    }

    
    pub fn handle_mouse_move(&mut self, cg: i32, cr: i32) {
        if self.drag_rotating {
            let dx = cg - self.drag_last_x;
            let ad = cr - self.drag_last_y;
            
            self.cam_angle_y += dx as f32 * 0.01;
            
            self.cam_angle_x = (self.cam_angle_x - ad as f32 * 0.008).clamp(0.15, 1.40);
            self.drag_last_x = cg;
            self.drag_last_y = cr;
        }
    }

    
    pub fn handle_mouse_release(&mut self) {
        self.drag_rotating = false;
    }

    
    pub fn handle_scroll(&mut self, mk: i8) {
        if mk > 0 {
            
            self.cam_dist = (self.cam_dist - 0.8).max(8.0);
        } else if mk < 0 {
            
            self.cam_dist = (self.cam_dist + 0.8).min(35.0);
        }
    }

    
    fn project_square_center(&self, row: u32, col: u32, w: usize, h: usize) -> (i32, i32, f32) {
        let bkb = ehc(row as i32, col as i32);
        self.project_point(bkb, w, h)
    }

    
    fn project_point(&self, aa: V3, w: usize, h: usize) -> (i32, i32, f32) {
        
        let ce = hr(self.cam_angle_x);
        let dea = eu(self.cam_angle_x);
        let apz = hr(self.cam_angle_y);
        let acl = eu(self.cam_angle_y);

        
        let dln = self.cam_dist * ce * acl;
        let dlo = self.cam_dist * dea;
        let dlp = self.cam_dist * ce * apz;

        
        let aub = 1.0 / self.cam_dist;
        let cjw = -dln * aub;
        let cjx = -dlo * aub;
        let cjy = -dlp * aub;

        
        let cps = -cjy;
        let cpt = cjw;
        let cpn = ra(cps * cps + cpt * cpt);
        if cpn < 0.0001 {
            return (w as i32 / 2, h as i32 / 2, 0.0);
        }
        let coq = cps / cpn;
        let cor = cpt / cpn;

        
        let haj = -cor * cjx;
        let hak = cor * cjw - coq * cjy;
        let hal = coq * cjx;

        
        let dx = aa.x - dln;
        let ad = aa.y - dlo;
        let dz = aa.z - dlp;

        
        let khc = dx * coq + dz * cor;
        let khd = dx * haj + ad * hak + dz * hal;
        let cgp = dx * cjw + ad * cjx + dz * cjy;

        if cgp < 0.5 {
            return (w as i32 / 2, h as i32 / 2, cgp);
        }

        
        let fov = w.min(h) as f32 * 1.6;
        let am = (khc / cgp * fov) as i32 + w as i32 / 2;
        let ak = (-khd / cgp * fov) as i32 + h as i32 / 2;
        (am, ak, cgp)
    }

    pub fn tick(&mut self) {
        self.frame += 1;
        if self.auto_rotate {
            self.cam_angle_y += 0.005;
        }
        
        if self.anim_piece.is_some() {
            self.anim_t += 0.06;
            if self.anim_t >= 1.0 {
                self.anim_piece = None;
                self.anim_t = 0.0;
            }
        }
    }

    
    pub fn render(&mut self, out_buf: &mut [u32], w: usize, h: usize) {
        if w < 100 || h < 100 { return; }

        
        for p in out_buf.iter_mut() {
            *p = 0xFF050808;
        }
        
        self.buf_w = w;
        self.buf_h = h;

        
        self.render_board_reflection(out_buf, w, h);

        
        self.render_board(out_buf, w, h);

        
        self.render_board_labels(out_buf, w, h);

        
        self.render_piece_shadows(out_buf, w, h);

        
        self.render_pieces(out_buf, w, h);
        
        
        self.render_hud(out_buf, w, h);
    }

    fn render_board(&self, buf: &mut [u32], w: usize, h: usize) {
        
        

        
        
        let mut eae: [(u32, u32, f32); 64] = [(0, 0, 0.0); 64];
        for row in 0..8u32 {
            for col in 0..8u32 {
                let idx = (row * 8 + col) as usize;
                let center = ehc(row as i32, col as i32);
                let fkq = self.to_camera_space(center);
                eae[idx] = (row, col, fkq.z);
            }
        }

        
        for i in 1..64 {
            let mut ay = i;
            while ay > 0 && eae[ay].2 > eae[ay - 1].2 {
                eae.swap(ay, ay - 1);
                ay -= 1;
            }
        }

        for &(row, col, _) in &eae {
            let cu = (row * 8 + col) as usize;
            let bhj = (row + col) % 2 == 0;

            
            let mut qf = if bhj { 0xFF3D6B3D } else { 0xFF1A3E1A };

            
            if self.chess.selected == Some(cu) {
                qf = 0xFF7AAA2A;
            }
            
            if self.chess.valid_moves.contains(&cu) {
                qf = if bhj { 0xFF4AAA4A } else { 0xFF2A8A2A };
            }
            
            if self.chess.last_move_from == Some(cu) || self.chess.last_move_to == Some(cu) {
                qf = if bhj { 0xFF6A8A3A } else { 0xFF4A6A2A };
            }
            
            if self.chess.cursor == cu {
                qf = 0xFF00CC55;
            }

            
            let og = ban(row as i32, col as i32);
            let hw = ban(row as i32, col as i32 + 1);
            let jf = ban(row as i32 + 1, col as i32 + 1);
            let bfc = ban(row as i32 + 1, col as i32);

            
            let (sx0, sy0, _) = self.project_point(og, w, h);
            let (wn, aiu, _) = self.project_point(hw, w, h);
            let (tq, acv, _) = self.project_point(jf, w, h);
            let (sx3, sy3, _) = self.project_point(bfc, w, h);

            
            cjn(buf, w, h, sx0, sy0, wn, aiu, tq, acv, qf);
            cjn(buf, w, h, sx0, sy0, tq, acv, sx3, sy3, qf);

            
            let awx = 0xFF0A1A0A;
            draw_line_buf(buf, w, h, sx0, sy0, wn, aiu, awx);
            draw_line_buf(buf, w, h, wn, aiu, tq, acv, awx);
            draw_line_buf(buf, w, h, tq, acv, sx3, sy3, awx);
            draw_line_buf(buf, w, h, sx3, sy3, sx0, sy0, awx);

            
            if self.chess.valid_moves.contains(&cu) && self.chess.board[cu] == Q {
                let (cx, u, _) = self.project_square_center(row, col, w, h);
                fill_circle_buf(buf, w, h, cx, u, 4, 0xFF00FF66);
            }
        }

        
        self.render_board_edge(buf, w, h);
    }

    
    fn render_board_labels(&self, buf: &mut [u32], w: usize, h: usize) {
        let ace = 0xFF55AA55;
        let bpc = 0.8;

        
        for col in 0..8 {
            let mxy = (b'a' + col as u8) as char;
            let pos = V3 {
                x: (col as f32 - 3.5) * bpc,
                y: -0.05,
                z: (8.0 - 4.0) * bpc + 0.3, 
            };
            let (am, ak, fq) = self.project_point(pos, w, h);
            if fq > 0.5 {
                fsv(buf, w, h, am - 4, ak - 8, mxy, ace);
            }
        }

        
        for row in 0..8 {
            let blu = (b'8' - row as u8) as char; 
            let pos = V3 {
                x: (0.0 - 4.0) * bpc - 0.3, 
                y: -0.05,
                z: (row as f32 - 3.5) * bpc,
            };
            let (am, ak, fq) = self.project_point(pos, w, h);
            if fq > 0.5 {
                fsv(buf, w, h, am - 4, ak - 8, blu, ace);
            }
        }
    }

    
    fn render_piece_shadows(&self, buf: &mut [u32], w: usize, h: usize) {
        let bjd = 0x40000000u32; 
        for cu in 0..64 {
            let piece = self.chess.board[cu];
            if piece == Q { continue; }

            let row = cu / 8;
            let col = cu % 8;
            let center = ehc(row as i32, col as i32);
            let (am, ak, fq) = self.project_point(center, w, h);
            if fq < 0.5 { continue; }

            
            let radius = (w.min(h) as f32 * 0.012 * (8.0 / fq)).max(2.0) as i32;
            let cm = (radius as f32 * 0.5) as i32; 
            for ad in -cm..=cm {
                for dx in -radius..=radius {
                    let nx = dx as f32 / radius as f32;
                    let re = ad as f32 / cm as f32;
                    if nx * nx + re * re > 1.0 { continue; }

                    let p = am + dx;
                    let o = ak + ad;
                    if p >= 0 && o >= 0 && p < w as i32 && o < h as i32 {
                        let idx = o as usize * w + p as usize;
                        if idx < buf.len() {
                            
                            let ku = buf[idx];
                            let r = ((ku >> 16) & 0xFF) * 3 / 4;
                            let g = ((ku >> 8) & 0xFF) * 3 / 4;
                            let b = (ku & 0xFF) * 3 / 4;
                            buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                        }
                    }
                }
            }
        }
    }

    
    fn render_board_reflection(&self, buf: &mut [u32], w: usize, h: usize) {
        let oeb = 0.35; 
        let gqv = 0.15; 

        
        for row in 0..8u32 {
            for col in 0..8u32 {
                let bhj = (row + col) % 2 == 0;
                let qf: u32 = if bhj { 0xFF3D6B3D } else { 0xFF1A3E1A };

                
                let og = ban(row as i32, col as i32);
                let hw = ban(row as i32, col as i32 + 1);
                let jf = ban(row as i32 + 1, col as i32 + 1);
                let bfc = ban(row as i32 + 1, col as i32);

                
                let mirror = |aa: V3| V3 { x: aa.x, y: -oeb - aa.y * 0.3, z: aa.z };

                let (sx0, sy0, _) = self.project_point(mirror(og), w, h);
                let (wn, aiu, _) = self.project_point(mirror(hw), w, h);
                let (tq, acv, _) = self.project_point(mirror(jf), w, h);
                let (sx3, sy3, _) = self.project_point(mirror(bfc), w, h);

                
                let r = (((qf >> 16) & 0xFF) as f32 * gqv) as u32;
                let g = (((qf >> 8) & 0xFF) as f32 * gqv) as u32;
                let b = ((qf & 0xFF) as f32 * gqv) as u32;
                let izb = 0xFF000000 | (r << 16) | (g << 8) | b;

                cjn(buf, w, h, sx0, sy0, wn, aiu, tq, acv, izb);
                cjn(buf, w, h, sx0, sy0, tq, acv, sx3, sy3, izb);
            }
        }
    }

    fn render_board_edge(&self, buf: &mut [u32], w: usize, h: usize) {
        let rh = 0.15;
        let awx = 0xFF0A200A;

        
        let atk = [
            ban(0, 0),
            ban(0, 8),
            ban(8, 8),
            ban(8, 0),
        ];
        let hnx: [V3; 4] = [
            V3 { x: atk[0].x, y: atk[0].y - rh, z: atk[0].z },
            V3 { x: atk[1].x, y: atk[1].y - rh, z: atk[1].z },
            V3 { x: atk[2].x, y: atk[2].y - rh, z: atk[2].z },
            V3 { x: atk[3].x, y: atk[3].y - rh, z: atk[3].z },
        ];

        
        for i in 0..4 {
            let ay = (i + 1) % 4;
            let (t0x, t0y, _) = self.project_point(atk[i], w, h);
            let (t1x, t1y, _) = self.project_point(atk[ay], w, h);
            let (b0x, b0y, _) = self.project_point(hnx[i], w, h);
            let (b1x, b1y, _) = self.project_point(hnx[ay], w, h);

            cjn(buf, w, h, t0x, t0y, t1x, t1y, b1x, b1y, awx);
            cjn(buf, w, h, t0x, t0y, b1x, b1y, b0x, b0y, awx);
        }
    }

    fn render_pieces(&self, buf: &mut [u32], w: usize, h: usize) {
        
        struct Aco {
            cu: usize,
            piece: i8,
            pos: V3,
            depth: f32,
        }

        let mut dcq: Vec<Aco> = Vec::new();

        for cu in 0..64 {
            let piece = self.chess.board[cu];
            if piece == Q { continue; }

            let row = cu / 8;
            let col = cu % 8;
            let pos = ehc(row as i32, col as i32);

            
            let fkq = self.to_camera_space(pos);

            dcq.push(Aco { cu, piece, pos, depth: fkq.z });
        }

        
        for i in 1..dcq.len() {
            let mut ay = i;
            while ay > 0 && dcq[ay].depth > dcq[ay - 1].depth {
                dcq.swap(ay, ay - 1);
                ay -= 1;
            }
        }

        
        for ej in &dcq {
            let xr = ej.piece > 0;
            let jth = if ej.piece < 0 { -ej.piece } else { ej.piece };

            let mesh = mdp(jth);
            let qf = if xr { 0xFFE0D8C8 } else { 0xFF2A2A2A };
            let awx = if xr { 0xFF1A1A1A } else { 0xFF888888 };

            
            self.render_piece_mesh(buf, w, h, &mesh, ej.pos, qf, awx);
        }
    }

    fn render_piece_mesh(&self, buf: &mut [u32], w: usize, h: usize,
                         mesh: &Dh, pos: V3, qf: u32, awx: u32) {
        
        let mut agy: Vec<(i32, i32)> = Vec::with_capacity(mesh.vertices.len());
        let mut ldk: Vec<f32> = Vec::with_capacity(mesh.vertices.len());

        for v in &mesh.vertices {
            let bkb = V3 {
                x: pos.x + v.x,
                y: pos.y + v.y,
                z: pos.z + v.z,
            };
            let (am, ak, fq) = self.project_point(bkb, w, h);
            agy.push((am, ak));
            ldk.push(fq);
        }

        
        if let Some(ref faces) = mesh.faces {
            struct Yr {
                idx: usize,
                avg_z: f32,
                brightness: f32,
            }

            let axv = V3 { x: -0.3, y: -0.8, z: -0.5 };
            let gfk = ra(axv.x * axv.x + axv.y * axv.y + axv.z * axv.z);
            let light = V3 { x: axv.x / gfk, y: axv.y / gfk, z: axv.z / gfk };

            let mut visible: Vec<Yr> = Vec::new();

            for (i, &(a, b, c)) in faces.iter().enumerate() {
                if a >= mesh.vertices.len() || b >= mesh.vertices.len() || c >= mesh.vertices.len() { continue; }

                let va = V3 { x: pos.x + mesh.vertices[a].x, y: pos.y + mesh.vertices[a].y, z: pos.z + mesh.vertices[a].z };
                let apk = V3 { x: pos.x + mesh.vertices[b].x, y: pos.y + mesh.vertices[b].y, z: pos.z + mesh.vertices[b].z };
                let bad = V3 { x: pos.x + mesh.vertices[c].x, y: pos.y + mesh.vertices[c].y, z: pos.z + mesh.vertices[c].z };

                
                let crh = self.to_camera_space(va);
                let aiv = self.to_camera_space(apk);
                let wo = self.to_camera_space(bad);

                let bsc = V3 { x: aiv.x - crh.x, y: aiv.y - crh.y, z: aiv.z - crh.z };
                let pg = V3 { x: wo.x - crh.x, y: wo.y - crh.y, z: wo.z - crh.z };
                let nx = bsc.y * pg.z - bsc.z * pg.y;
                let re = bsc.z * pg.x - bsc.x * pg.z;
                let wi = bsc.x * pg.y - bsc.y * pg.x;
                let dbt = ra(nx * nx + re * re + wi * wi);
                if dbt < 0.0001 { continue; }
                let ae = V3 { x: nx / dbt, y: re / dbt, z: wi / dbt };

                
                if ae.z > 0.0 { continue; }

                let dux = -(ae.x * light.x + ae.y * light.y + ae.z * light.z);
                let brightness = 0.3 + 0.7 * dux.max(0.0);

                let avg_z = (crh.z + aiv.z + wo.z) / 3.0;
                visible.push(Yr { idx: i, avg_z, brightness });
            }

            
            for i in 1..visible.len() {
                let mut ay = i;
                while ay > 0 && visible[ay].avg_z > visible[ay - 1].avg_z {
                    visible.swap(ay, ay - 1);
                    ay -= 1;
                }
            }

            let adi = (qf >> 16) & 0xFF;
            let agd = (qf >> 8) & 0xFF;
            let apu = qf & 0xFF;

            for fd in &visible {
                let (a, b, c) = faces[fd.idx];
                if a >= agy.len() || b >= agy.len() || c >= agy.len() { continue; }
                let (sx0, sy0) = agy[a];
                let (wn, aiu) = agy[b];
                let (tq, acv) = agy[c];

                let r = ((adi as f32 * fd.brightness) as u32).min(255);
                let g = ((agd as f32 * fd.brightness) as u32).min(255);
                let jyw = ((apu as f32 * fd.brightness) as u32).min(255);
                let gur = 0xFF000000 | (r << 16) | (g << 8) | jyw;

                cjn(buf, w, h, sx0, sy0, wn, aiu, tq, acv, gur);
            }
        }

        
        if let Some(ref edges) = mesh.edges {
            for &(a, b) in edges {
                if a >= agy.len() || b >= agy.len() { continue; }
                let (bm, az) = agy[a];
                let (x1, y1) = agy[b];
                draw_line_buf(buf, w, h, bm, az, x1, y1, awx);
            }
        }
    }

    fn to_camera_space(&self, aa: V3) -> V3 {
        let ce = hr(self.cam_angle_x);
        let dea = eu(self.cam_angle_x);
        let apz = hr(self.cam_angle_y);
        let acl = eu(self.cam_angle_y);

        let dln = self.cam_dist * ce * acl;
        let dlo = self.cam_dist * dea;
        let dlp = self.cam_dist * ce * apz;

        let aub = 1.0 / self.cam_dist;
        let cjw = -dln * aub;
        let cjx = -dlo * aub;
        let cjy = -dlp * aub;

        let cps = -cjy;
        let cpt = cjw;
        let cpn = ra(cps * cps + cpt * cpt);
        if cpn < 0.0001 {
            return V3 { x: 0.0, y: 0.0, z: self.cam_dist };
        }
        let coq = cps / cpn;
        let cor = cpt / cpn;

        let haj = -cor * cjx;
        let hak = cor * cjw - coq * cjy;
        let hal = coq * cjx;

        let dx = aa.x - dln;
        let ad = aa.y - dlo;
        let dz = aa.z - dlp;

        V3 {
            x: dx * coq + dz * cor,
            y: dx * haj + ad * hak + dz * hal,
            z: dx * cjw + ad * cjx + dz * cjy,
        }
    }

    
    fn render_hud(&self, buf: &mut [u32], w: usize, h: usize) {
        
        let dfw = if self.chess.white_turn { "WHITE" } else { "BLACK" };
        let iuo = match self.chess.phase {
            GamePhase::Playing => "",
            GamePhase::Check => " CHECK!",
            GamePhase::Checkmate => " CHECKMATE!",
            GamePhase::Stalemate => " STALEMATE",
            GamePhase::Promotion => " PROMOTION",
        };
        
        
        let ecu = if self.chess.white_turn { 0xFFE0E0E0 } else { 0xFF40FF40 };
        drp(buf, w, h, 8, 8, dfw, ecu);
        if !iuo.is_empty() {
            let nud = match self.chess.phase {
                GamePhase::Check => 0xFFFF4444,
                GamePhase::Checkmate => 0xFFFF2222,
                GamePhase::Stalemate => 0xFFFFAA00,
                GamePhase::Promotion => 0xFF4488FF,
                _ => 0xFF40FF40,
            };
            drp(buf, w, h, 8 + dfw.len() as i32 * 8, 8, iuo, nud);
        }
        
        
        let score = self.chess.material_score();
        let dyq = format!("{:+}", score);
        let gsw = if score > 0 { 0xFFE0E0E0 } else if score < 0 { 0xFF40FF40 } else { 0xFF888888 };
        drp(buf, w, h, w as i32 - 60, 8, &dyq, gsw);
        
        
        let hint = "WASD:Cam Scroll/ZX:Zoom O:Rotate Drag:Orbit";
        drp(buf, w, h, 8, h as i32 - 14, hint, 0xFF336633);
        
        
        let history = &self.chess.move_history;
        let start = if history.len() > 5 { history.len() - 5 } else { 0 };
        for (i, mv) in history[start..].iter().enumerate() {
            let hde = 24 + i as i32 * 12;
            drp(buf, w, h, 8, hde, mv, 0xFF448844);
        }
    }
}






fn ehc(row: i32, col: i32) -> V3 {
    let bpc = 0.8;
    V3 {
        x: (col as f32 - 3.5) * bpc,
        y: 0.0,
        z: (row as f32 - 3.5) * bpc,
    }
}


fn ban(row: i32, col: i32) -> V3 {
    let bpc = 0.8;
    V3 {
        x: (col as f32 - 4.0) * bpc,
        y: 0.0,
        z: (row as f32 - 4.0) * bpc,
    }
}





pub struct Dh {
    pub vertices: Vec<V3>,
    pub edges: Option<Vec<(usize, usize)>>,
    pub faces: Option<Vec<(usize, usize, usize)>>,
}


fn mdp(agw: i8) -> Dh {
    match agw {
        1 => ink(),
        2 => neu(),
        3 => neo(),
        4 => nex(),
        5 => nev(),
        6 => ner(),
        _ => ink(),
    }
}



fn dq(y: f32, r: f32, ae: u32) -> Vec<V3> {
    let mut pts = Vec::with_capacity(ae as usize);
    for i in 0..ae {
        let cc = (i as f32 / ae as f32) * 6.2831853;
        pts.push(V3 {
            x: hr(cc) * r,
            y,
            z: eu(cc) * r,
        });
    }
    pts
}



fn sr(aje: usize, ajf: usize, ae: usize) -> (Vec<(usize, usize, usize)>, Vec<(usize, usize)>) {
    let mut faces = Vec::new();
    let mut edges = Vec::new();
    for i in 0..ae {
        let ay = (i + 1) % ae;
        let abn = aje + i;
        let eb = aje + ay;
        let kl = ajf + i;
        let gf = ajf + ay;
        faces.push((abn, eb, gf));
        faces.push((abn, gf, kl));
        edges.push((abn, eb));
        edges.push((abn, kl));
    }
    edges.push((aje, ajf)); 
    (faces, edges)
}


fn baw(base: usize, bxm: usize, ae: usize) -> (Vec<(usize, usize, usize)>, Vec<(usize, usize)>) {
    let mut faces = Vec::new();
    let mut edges = Vec::new();
    for i in 0..ae {
        let ay = (i + 1) % ae;
        faces.push((base + i, base + ay, bxm));
        edges.push((base + i, bxm));
    }
    (faces, edges)
}

const Y: u32 = 8; 
const Ko: f32 = 0.22; 

fn ink() -> Dh {
    let j = Ko;
    let ae = Y as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    
    let aml = dq(0.0, 0.35 * j, Y);     
    let kl = verts.len(); verts.extend_from_slice(&aml);
    let uh = dq(0.08, 0.35 * j, Y);     
    let gf = verts.len(); verts.extend_from_slice(&uh);

    
    let ju = dq(0.10, 0.20 * j, Y);     
    let iq = verts.len(); verts.extend_from_slice(&ju);
    let azf = dq(0.28, 0.15 * j, Y);     
    let sc = verts.len(); verts.extend_from_slice(&azf);

    
    let bve = dq(0.30, 0.20 * j, Y);     
    let abr = verts.len(); verts.extend_from_slice(&bve);
    let bde = dq(0.38, 0.22 * j, Y);     
    let agb = verts.len(); verts.extend_from_slice(&bde);
    let cop = dq(0.44, 0.18 * j, Y);     
    let ber = verts.len(); verts.extend_from_slice(&cop);

    
    let bxm = verts.len();
    verts.push(V3 { x: 0.0, y: 0.48 * 1.0, z: 0.0 });

    
    let (f, e) = sr(kl, gf, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(gf, iq, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(iq, sc, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(sc, abr, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(abr, agb, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(agb, ber, ae); faces.extend(f); edges.extend(e);
    let (f, e) = baw(ber, bxm, ae); faces.extend(f); edges.extend(e);

    
    let kdp = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = baw(kl, kdp, ae); faces.extend(f); edges.extend(e);

    Dh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn neu() -> Dh {
    let j = Ko;
    let ae = Y as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    
    let aml = dq(0.0, 0.38 * j, Y);
    let kl = verts.len(); verts.extend_from_slice(&aml);
    let uh = dq(0.10, 0.35 * j, Y);
    let gf = verts.len(); verts.extend_from_slice(&uh);

    
    let ju = dq(0.20, 0.20 * j, Y);
    let iq = verts.len(); verts.extend_from_slice(&ju);
    let azf = dq(0.35, 0.22 * j, Y);
    let sc = verts.len(); verts.extend_from_slice(&azf);

    
    let mut iei = dq(0.42, 0.20 * j, Y);
    for v in iei.iter_mut() { v.z -= 0.06; } 
    let abr = verts.len(); verts.extend_from_slice(&iei);

    
    let mut ipe = dq(0.47, 0.12 * j, Y);
    for v in ipe.iter_mut() { v.z -= 0.12; }
    let agb = verts.len(); verts.extend_from_slice(&ipe);

    
    let lne = verts.len();
    verts.push(V3 { x: 0.0, y: 0.55, z: -0.02 });

    
    let (f, e) = sr(kl, gf, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(gf, iq, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(iq, sc, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(sc, abr, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(abr, agb, ae); faces.extend(f); edges.extend(e);
    let (f, e) = baw(agb, lne, ae); faces.extend(f); edges.extend(e);

    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = baw(kl, bottom, ae); faces.extend(f); edges.extend(e);

    Dh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn neo() -> Dh {
    let j = Ko;
    let ae = Y as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    
    let aml = dq(0.0, 0.38 * j, Y);
    let kl = verts.len(); verts.extend_from_slice(&aml);
    let uh = dq(0.10, 0.35 * j, Y);
    let gf = verts.len(); verts.extend_from_slice(&uh);

    
    let ju = dq(0.12, 0.18 * j, Y);
    let iq = verts.len(); verts.extend_from_slice(&ju);
    let azf = dq(0.32, 0.15 * j, Y);
    let sc = verts.len(); verts.extend_from_slice(&azf);

    
    let bve = dq(0.34, 0.22 * j, Y);
    let abr = verts.len(); verts.extend_from_slice(&bve);
    let bde = dq(0.42, 0.20 * j, Y);
    let agb = verts.len(); verts.extend_from_slice(&bde);
    let cop = dq(0.50, 0.12 * j, Y);
    let ber = verts.len(); verts.extend_from_slice(&cop);

    
    let bxm = verts.len();
    verts.push(V3 { x: 0.0, y: 0.58, z: 0.0 });

    let (f, e) = sr(kl, gf, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(gf, iq, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(iq, sc, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(sc, abr, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(abr, agb, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(agb, ber, ae); faces.extend(f); edges.extend(e);
    let (f, e) = baw(ber, bxm, ae); faces.extend(f); edges.extend(e);

    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = baw(kl, bottom, ae); faces.extend(f); edges.extend(e);

    Dh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn nex() -> Dh {
    let j = Ko;
    let ae = Y as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    
    let aml = dq(0.0, 0.40 * j, Y);
    let kl = verts.len(); verts.extend_from_slice(&aml);
    let uh = dq(0.10, 0.38 * j, Y);
    let gf = verts.len(); verts.extend_from_slice(&uh);

    
    let ju = dq(0.12, 0.25 * j, Y);
    let iq = verts.len(); verts.extend_from_slice(&ju);
    let azf = dq(0.38, 0.25 * j, Y);
    let sc = verts.len(); verts.extend_from_slice(&azf);

    
    let bve = dq(0.40, 0.32 * j, Y);
    let abr = verts.len(); verts.extend_from_slice(&bve);
    let bde = dq(0.50, 0.32 * j, Y);
    let agb = verts.len(); verts.extend_from_slice(&bde);

    
    let pld = verts.len();
    verts.push(V3 { x: 0.0, y: 0.50, z: 0.0 });

    let (f, e) = sr(kl, gf, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(gf, iq, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(iq, sc, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(sc, abr, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(abr, agb, ae); faces.extend(f); edges.extend(e);
    let (f, e) = baw(agb, pld, ae); faces.extend(f); edges.extend(e);

    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = baw(kl, bottom, ae); faces.extend(f); edges.extend(e);

    Dh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn nev() -> Dh {
    let j = Ko;
    let ae = Y as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    
    let aml = dq(0.0, 0.40 * j, Y);
    let kl = verts.len(); verts.extend_from_slice(&aml);
    let uh = dq(0.10, 0.38 * j, Y);
    let gf = verts.len(); verts.extend_from_slice(&uh);

    
    let ju = dq(0.12, 0.22 * j, Y);
    let iq = verts.len(); verts.extend_from_slice(&ju);
    let azf = dq(0.30, 0.18 * j, Y);
    let sc = verts.len(); verts.extend_from_slice(&azf);

    
    let bve = dq(0.35, 0.25 * j, Y);
    let abr = verts.len(); verts.extend_from_slice(&bve);

    
    let bde = dq(0.42, 0.28 * j, Y);
    let agb = verts.len(); verts.extend_from_slice(&bde);

    
    let cop = dq(0.52, 0.15 * j, Y);
    let ber = verts.len(); verts.extend_from_slice(&cop);

    
    let bxm = verts.len();
    verts.push(V3 { x: 0.0, y: 0.60, z: 0.0 });

    let (f, e) = sr(kl, gf, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(gf, iq, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(iq, sc, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(sc, abr, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(abr, agb, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(agb, ber, ae); faces.extend(f); edges.extend(e);
    let (f, e) = baw(ber, bxm, ae); faces.extend(f); edges.extend(e);

    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = baw(kl, bottom, ae); faces.extend(f); edges.extend(e);

    Dh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}

fn ner() -> Dh {
    let j = Ko;
    let ae = Y as usize;
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    let mut edges = Vec::new();

    
    let aml = dq(0.0, 0.42 * j, Y);
    let kl = verts.len(); verts.extend_from_slice(&aml);
    let uh = dq(0.10, 0.40 * j, Y);
    let gf = verts.len(); verts.extend_from_slice(&uh);

    
    let ju = dq(0.12, 0.24 * j, Y);
    let iq = verts.len(); verts.extend_from_slice(&ju);
    let azf = dq(0.32, 0.20 * j, Y);
    let sc = verts.len(); verts.extend_from_slice(&azf);

    
    let bve = dq(0.35, 0.28 * j, Y);
    let abr = verts.len(); verts.extend_from_slice(&bve);
    let bde = dq(0.42, 0.26 * j, Y);
    let agb = verts.len(); verts.extend_from_slice(&bde);

    
    let cop = dq(0.48, 0.15 * j, Y);
    let ber = verts.len(); verts.extend_from_slice(&cop);

    let (f, e) = sr(kl, gf, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(gf, iq, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(iq, sc, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(sc, abr, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(abr, agb, ae); faces.extend(f); edges.extend(e);
    let (f, e) = sr(agb, ber, ae); faces.extend(f); edges.extend(e);

    
    
    let aap = 0.04;
    let dls = 0.14;
    let brg = 0.50;
    let ja = verts.len();
    verts.push(V3 { x: -aap, y: brg, z: -aap });
    verts.push(V3 { x:  aap, y: brg, z: -aap });
    verts.push(V3 { x:  aap, y: brg + dls, z: -aap });
    verts.push(V3 { x: -aap, y: brg + dls, z: -aap });
    verts.push(V3 { x: -aap, y: brg, z:  aap });
    verts.push(V3 { x:  aap, y: brg, z:  aap });
    verts.push(V3 { x:  aap, y: brg + dls, z:  aap });
    verts.push(V3 { x: -aap, y: brg + dls, z:  aap });

    
    faces.push((ja, ja+1, ja+2)); faces.push((ja, ja+2, ja+3));      
    faces.push((ja+4, ja+6, ja+5)); faces.push((ja+4, ja+7, ja+6));  
    faces.push((ja, ja+3, ja+7)); faces.push((ja, ja+7, ja+4));      
    faces.push((ja+1, ja+5, ja+6)); faces.push((ja+1, ja+6, ja+2));  
    faces.push((ja+3, ja+2, ja+6)); faces.push((ja+3, ja+6, ja+7));  

    
    edges.push((ja, ja+1)); edges.push((ja+1, ja+2)); edges.push((ja+2, ja+3)); edges.push((ja+3, ja));
    edges.push((ja+4, ja+5)); edges.push((ja+5, ja+6)); edges.push((ja+6, ja+7)); edges.push((ja+7, ja+4));
    edges.push((ja, ja+4)); edges.push((ja+1, ja+5)); edges.push((ja+2, ja+6)); edges.push((ja+3, ja+7));

    
    let gk = brg + dls * 0.55;
    let ek = 0.10;
    let km = verts.len();
    verts.push(V3 { x: -ek, y: gk, z: -aap });
    verts.push(V3 { x:  ek, y: gk, z: -aap });
    verts.push(V3 { x:  ek, y: gk + 0.03, z: -aap });
    verts.push(V3 { x: -ek, y: gk + 0.03, z: -aap });
    verts.push(V3 { x: -ek, y: gk, z:  aap });
    verts.push(V3 { x:  ek, y: gk, z:  aap });
    verts.push(V3 { x:  ek, y: gk + 0.03, z:  aap });
    verts.push(V3 { x: -ek, y: gk + 0.03, z:  aap });

    faces.push((km, km+1, km+2)); faces.push((km, km+2, km+3));
    faces.push((km+4, km+6, km+5)); faces.push((km+4, km+7, km+6));
    faces.push((km, km+3, km+7)); faces.push((km, km+7, km+4));
    faces.push((km+1, km+5, km+6)); faces.push((km+1, km+6, km+2));
    faces.push((km+3, km+2, km+6)); faces.push((km+3, km+6, km+7));

    edges.push((km, km+1)); edges.push((km+1, km+2)); edges.push((km+2, km+3)); edges.push((km+3, km));
    edges.push((km+4, km+5)); edges.push((km+5, km+6)); edges.push((km+6, km+7)); edges.push((km+7, km+4));

    
    let pli = verts.len();
    verts.push(V3 { x: 0.0, y: 0.48, z: 0.0 });
    let (f, e) = baw(ber, pli, ae); faces.extend(f); edges.extend(e);

    
    let bottom = verts.len();
    verts.push(V3 { x: 0.0, y: 0.0, z: 0.0 });
    let (f, e) = baw(kl, bottom, ae); faces.extend(f); edges.extend(e);

    Dh { vertices: verts, edges: Some(edges), faces: Some(faces) }
}





fn cjn(buf: &mut [u32], w: usize, h: usize,
                       mut bm: i32, mut az: i32,
                       mut x1: i32, mut y1: i32,
                       mut x2: i32, mut y2: i32,
                       color: u32) {
    
    if az > y1 { core::mem::swap(&mut bm, &mut x1); core::mem::swap(&mut az, &mut y1); }
    if y1 > y2 { core::mem::swap(&mut x1, &mut x2); core::mem::swap(&mut y1, &mut y2); }
    if az > y1 { core::mem::swap(&mut bm, &mut x1); core::mem::swap(&mut az, &mut y1); }

    let sn = y2 - az;
    if sn == 0 { return; }

    let ajb = az.max(0);
    let bkg = y2.min(h as i32 - 1);

    for y in ajb..=bkg {
        let dyv = y >= y1;
        let cqg = if dyv { y2 - y1 } else { y1 - az };

        let gxu = (y - az) as f32 / sn as f32;
        let aja = bm as f32 + (x2 - bm) as f32 * gxu;

        let bkd = if cqg == 0 {
            aja
        } else if dyv {
            let crg = (y - y1) as f32 / cqg as f32;
            x1 as f32 + (x2 - x1) as f32 * crg
        } else {
            let crg = (y - az) as f32 / cqg as f32;
            bm as f32 + (x1 - bm) as f32 * crg
        };

        let mut left = aja as i32;
        let mut right = bkd as i32;
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


fn drp(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, text: &str, color: u32) {
    let mut cx = x;
    for ch in text.chars() {
        fsv(buf, w, h, cx, y, ch, color);
        cx += 8;
    }
}


fn fsv(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, c: char, color: u32) {
    let du = crate::framebuffer::font::ol(c);
    for row in 0..16 {
        let bits = du[row];
        for col in 0..8 {
            if bits & (0x80 >> col) != 0 {
                let p = x + col as i32;
                let o = y + row as i32;
                if p >= 0 && o >= 0 && p < w as i32 && o < h as i32 {
                    let idx = o as usize * w + p as usize;
                    if idx < buf.len() {
                        buf[idx] = color;
                    }
                }
            }
        }
    }
}
