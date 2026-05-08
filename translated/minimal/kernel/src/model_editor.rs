





use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::formula3d::{V3, Ai, eu, hr};



const AWA_: f32 = 8.0;
const AET_: usize = 16;
const BKM_: f32 = 12.0;
const AAN_: f32 = 2.0;


const IK_: u32        = 0xFF1A1A2E;
const BQL_: u32      = 0xFF2A2A3A;
const BQM_: u32 = 0xFF3A3A4A;
const BQI_: u32    = 0xFFFF4444;
const BQJ_: u32    = 0xFF44FF44;
const BQK_: u32    = 0xFF4488FF;
const BQZ_: u32      = 0xFF00FFAA;
const BRA_: u32  = 0xFFFFFF00;
const BQW_: u32    = 0xFFFFFFFF;
const BQY_: u32  = 0xFFFFAA00;
const BQX_: u32 = 0xFF88FFFF;
const TJ_: u32   = 0xFF0D0D1A;
const NH_: u32 = 0xFF2A2A4A;
const BQU_: u32 = 0xFF0055AA;
const ABW_: u32    = 0xFF0D0D1A;
const P_: u32      = 0xFFCCCCCC;
const AQR_: u32   = 0xFFFFFFFF;
const ABX_: u32  = 0xFF777777;
const AQS_: u32    = 0xCC1A1A2E;
const TI_: u32 = 0xFF00AAFF;



#[derive(Clone, Copy, PartialEq)]
pub enum EditTool {
    Select,     
    AddVertex,  
    AddEdge,    
    Move,       
    Delete,     
}

impl EditTool {
    fn name(self) -> &'static str {
        match self {
            EditTool::Select => "Select",
            EditTool::AddVertex => "Add Vertex",
            EditTool::AddEdge => "Add Edge",
            EditTool::Move => "Move",
            EditTool::Delete => "Delete",
        }
    }
    fn hotkey(self) -> &'static str {
        match self {
            EditTool::Select => "S",
            EditTool::AddVertex => "A",
            EditTool::AddEdge => "E",
            EditTool::Move => "G",
            EditTool::Delete => "X",
        }
    }
    fn tip(self) -> &'static str {
        match self {
            EditTool::Select => "Click vertex to select. Shift+click for multi-select.",
            EditTool::AddVertex => "Click in viewport to place a vertex on the grid.",
            EditTool::AddEdge => "Click two vertices to connect them with an edge.",
            EditTool::Move => "Drag selected vertices. Arrows = fine move.",
            EditTool::Delete => "Click a vertex or edge to remove it.",
        }
    }
}



#[derive(Clone)]
pub struct ModelEditorState {
    
    pub vertices: Vec<V3>,
    pub edges: Vec<(usize, usize)>,
    
    
    pub cam_angle_y: f32,    
    pub cam_angle_x: f32,    
    pub cam_dist: f32,       
    pub cam_target: V3,      
    
    
    pub tool: EditTool,
    pub selected_verts: Vec<usize>,
    pub hover_vert: Option<usize>,
    pub edge_start: Option<usize>, 
    
    
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_pressed: bool,
    pub mouse_dragging: bool,
    pub drag_start_x: i32,
    pub drag_start_y: i32,
    pub right_dragging: bool,  
    
    
    pub show_grid: bool,
    pub show_axes: bool,
    pub show_tips: bool,
    pub show_vertices: bool,
    pub status_msg: String,
    pub model_name: String,
    
    
    pub undo_stack: Vec<(Vec<V3>, Vec<(usize, usize)>)>,
    
    
    frame: u32,
    
    cached_cy: f32,
    cached_sy: f32,
    cached_cx: f32,
    cached_sx: f32,
}

impl ModelEditorState {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            cam_angle_y: 0.6,
            cam_angle_x: 0.4,
            cam_dist: 6.0,
            cam_target: V3 { x: 0.0, y: 0.0, z: 0.0 },
            tool: EditTool::Select,
            selected_verts: Vec::new(),
            hover_vert: None,
            edge_start: None,
            mouse_x: 0,
            mouse_y: 0,
            mouse_pressed: false,
            mouse_dragging: false,
            drag_start_x: 0,
            drag_start_y: 0,
            right_dragging: false,
            show_grid: true,
            show_axes: true,
            show_tips: true,
            show_vertices: true,
            status_msg: String::from("TrustEdit 3D ready — Press H for help"),
            model_name: String::from("untitled"),
            undo_stack: Vec::new(),
            frame: 0,
            cached_cy: hr(0.6),
            cached_sy: eu(0.6),
            cached_cx: hr(0.4),
            cached_sx: eu(0.4),
        }
    }
    
    
    pub fn load_preset(&mut self, name: &str) {
        self.push_undo();
        let mesh = match name {
            "cube" => crate::formula3d::mesh_cube(),
            "pyramid" => crate::formula3d::mesh_pyramid(),
            "diamond" => crate::formula3d::mesh_diamond(),
            "torus" => crate::formula3d::mesh_torus(1.0, 0.4, 16, 8),
            "icosphere" => crate::formula3d::mesh_icosphere(1.2),
            "grid" => crate::formula3d::mesh_grid(2.0, 4),
            _ => return,
        };
        self.vertices = mesh.vertices;
        self.edges = mesh.edges;
        self.selected_verts.clear();
        self.edge_start = None;
        self.status_msg = format!("Loaded preset: {} ({} verts, {} edges)", 
            name, self.vertices.len(), self.edges.len());
    }
    
    
    pub fn clear(&mut self) {
        self.push_undo();
        self.vertices.clear();
        self.edges.clear();
        self.selected_verts.clear();
        self.edge_start = None;
        self.status_msg = String::from("Scene cleared");
    }
    
    
    fn push_undo(&mut self) {
        if self.undo_stack.len() > 20 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push((self.vertices.clone(), self.edges.clone()));
    }
    
    
    pub fn undo(&mut self) {
        if let Some((verts, edges)) = self.undo_stack.pop() {
            self.vertices = verts;
            self.edges = edges;
            self.selected_verts.clear();
            self.edge_start = None;
            self.status_msg = String::from("Undo");
        } else {
            self.status_msg = String::from("Nothing to undo");
        }
    }
    
    
    pub fn save(&mut self) {
        
        let mut data = String::new();
        data.push_str("TRUSTEDIT3D v1\n");
        data.push_str(&format!("vertices {}\n", self.vertices.len()));
        for v in &self.vertices {
            data.push_str(&format!("v {:.4} {:.4} {:.4}\n", v.x, v.y, v.z));
        }
        data.push_str(&format!("edges {}\n", self.edges.len()));
        for &(a, b) in &self.edges {
            data.push_str(&format!("e {} {}\n", a, b));
        }
        
        let path = format!("/{}.t3d", self.model_name);
        let bytes = data.as_bytes();
        
        
        let _ = crate::ramfs::bh(|fs| fs.touch(&path));
        match crate::ramfs::bh(|fs| fs.write_file(&path, bytes)) {
            Ok(_) => {
                self.status_msg = format!("Saved to {}", path);
            },
            Err(_) => {
                self.status_msg = String::from("Error: Could not save file");
            }
        }
    }
    
    
    pub fn load(&mut self, path: &str) {
        let lbq: Result<Vec<u8>, ()> = crate::ramfs::bh(|fs| {
            match fs.read_file(path) {
                Ok(bytes) => Ok(bytes.to_vec()),
                Err(_) => Err(()),
            }
        });
        
        match lbq {
            Ok(data) => {
                self.push_undo();
                if let Ok(text) = core::str::from_utf8(&data) {
                    self.parse_t3d(text);
                    
                    if let Some(name) = path.strip_prefix('/') {
                        if let Some(name) = name.strip_suffix(".t3d") {
                            self.model_name = String::from(name);
                        }
                    }
                    self.status_msg = format!("Loaded {} ({} verts, {} edges)", 
                        path, self.vertices.len(), self.edges.len());
                } else {
                    self.status_msg = String::from("Error: Invalid file data");
                }
            },
            Err(_) => {
                self.status_msg = format!("Error: Could not read {}", path);
            }
        }
    }
    
    
    fn parse_t3d(&mut self, text: &str) {
        self.vertices.clear();
        self.edges.clear();
        self.selected_verts.clear();
        
        for line in text.lines() {
            let au: Vec<&str> = line.split_whitespace().collect();
            if au.is_empty() { continue; }
            match au[0] {
                "v" if au.len() >= 4 => {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        au[1].parse::<f32>(),
                        au[2].parse::<f32>(),
                        au[3].parse::<f32>(),
                    ) {
                        self.vertices.push(V3 { x, y, z });
                    }
                },
                "e" if au.len() >= 3 => {
                    if let (Ok(a), Ok(b)) = (
                        au[1].parse::<usize>(),
                        au[2].parse::<usize>(),
                    ) {
                        self.edges.push((a, b));
                    }
                },
                _ => {}
            }
        }
    }
    
    
    pub fn rak(&self) -> Ai {
        Ai {
            vertices: self.vertices.clone(),
            edges: self.edges.clone(),
            edge_colors: None,
            faces: None,
            face_colors: None,
        }
    }
    
    
    
    fn project_vertex(&self, v: V3, bt: usize, ex: usize) -> (i32, i32, f32) {
        
        let v = V3 { 
            x: v.x - self.cam_target.x, 
            y: v.y - self.cam_target.y, 
            z: v.z - self.cam_target.z 
        };
        
        let u = self.cached_cy;
        let ak = self.cached_sy;
        let da = V3 { x: v.x * u + v.z * ak, y: v.y, z: -v.x * ak + v.z * u };
        
        let cx = self.cached_cx;
        let am = self.cached_sx;
        let cm = V3 { x: da.x, y: da.y * cx - da.z * am, z: da.y * am + da.z * cx };
        
        
        let edb = cm.z + self.cam_dist;
        
        
        if edb < 0.1 { return (0, 0, -999.0); }
        let scale = (bt.min(ex) as f32) * 0.45;
        let ozh = (cm.x / edb * scale) + (bt as f32 * 0.5);
        let ozi = (-cm.y / edb * scale) + (ex as f32 * 0.5);
        (ozh as i32, ozi as i32, edb)
    }
    
    
    fn unproject_to_grid(&self, am: i32, ak: i32, bt: usize, ex: usize) -> V3 {
        
        let scale = (bt.min(ex) as f32) * 0.45;
        let da = (am as f32 - bt as f32 * 0.5) / scale;
        let cm = -(ak as f32 - ex as f32 * 0.5) / scale;
        
        
        let it = V3 { x: da, y: cm, z: 1.0 };
        
        
        let cx = hr(-self.cam_angle_x);
        let jke = eu(-self.cam_angle_x);
        let vh = V3 { x: it.x, y: it.y * cx - it.z * jke, z: it.y * jke + it.z * cx };
        
        let u = hr(-self.cam_angle_y);
        let jkh = eu(-self.cam_angle_y);
        let jq = V3 { x: vh.x * u + vh.z * jkh, y: vh.y, z: -vh.x * jkh + vh.z * u };
        
        
        let cgo = V3 {
            x: self.cam_target.x - self.cam_dist * eu(self.cam_angle_y) * hr(self.cam_angle_x),
            y: self.cam_target.y + self.cam_dist * eu(self.cam_angle_x),
            z: self.cam_target.z - self.cam_dist * hr(self.cam_angle_y) * hr(self.cam_angle_x),
        };
        
        
        if jq.y.abs() < 0.001 {
            
            return V3 { x: cgo.x + jq.x * 5.0, y: 0.0, z: cgo.z + jq.z * 5.0 };
        }
        let t = -cgo.y / jq.y;
        if t < 0.0 {
            
            return V3 { x: cgo.x + jq.x * 5.0, y: 0.0, z: cgo.z + jq.z * 5.0 };
        }
        
        let bkb = V3 {
            x: cgo.x + jq.x * t,
            y: 0.0,
            z: cgo.z + jq.z * t,
        };
        
        
        V3 {
            x: (bkb.x * 2.0 + 0.5) as i32 as f32 * 0.5,
            y: 0.0,
            z: (bkb.z * 2.0 + 0.5) as i32 as f32 * 0.5,
        }
    }
    
    
    fn find_vertex_at(&self, am: i32, ak: i32, bt: usize, ex: usize) -> Option<usize> {
        let mut adj: Option<(usize, f32)> = None;
        for (i, v) in self.vertices.iter().enumerate() {
            let (p, o, z) = self.project_vertex(*v, bt, ex);
            if z < 0.1 { continue; }
            let dx = (p - am) as f32;
            let ad = (o - ak) as f32;
            let em = dx * dx + ad * ad;
            if em < BKM_ * BKM_ {
                if let Some((_, egt)) = adj {
                    if em < egt { adj = Some((i, em)); }
                } else {
                    adj = Some((i, em));
                }
            }
        }
        adj.map(|(i, _)| i)
    }
    
    
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::*;
        
        match key {
            
            b's' | b'S' => {
                self.tool = EditTool::Select;
                self.edge_start = None;
                self.status_msg = String::from("Tool: Select");
            },
            b'a' | b'A' => {
                self.tool = EditTool::AddVertex;
                self.edge_start = None;
                self.status_msg = String::from("Tool: Add Vertex — Click to place");
            },
            b'e' | b'E' => {
                self.tool = EditTool::AddEdge;
                self.edge_start = None;
                self.status_msg = String::from("Tool: Add Edge — Click two vertices");
            },
            b'g' | b'G' => {
                self.tool = EditTool::Move;
                self.edge_start = None;
                self.status_msg = String::from("Tool: Move — Drag vertices");
            },
            b'x' | b'X' => {
                if self.tool == EditTool::Delete {
                    
                    if !self.selected_verts.is_empty() {
                        self.delete_selected();
                    }
                } else {
                    self.tool = EditTool::Delete;
                    self.edge_start = None;
                    self.status_msg = String::from("Tool: Delete — Click to remove");
                }
            },
            
            
            b'z' | b'Z' => self.undo(),
            
            
            b'h' | b'H' => {
                self.show_tips = !self.show_tips;
                self.status_msg = format!("Tips: {}", if self.show_tips { "ON" } else { "OFF" });
            },
            b'v' | b'V' => {
                self.show_vertices = !self.show_vertices;
            },
            
            
            b'1' => self.load_preset("cube"),
            b'2' => self.load_preset("pyramid"),
            b'3' => self.load_preset("diamond"),
            b'4' => self.load_preset("torus"),
            b'5' => self.load_preset("icosphere"),
            b'6' => self.load_preset("grid"),
            
            
            b'w' | b'W' => self.save(),
            b'l' | b'L' => {
                let path = format!("/{}.t3d", self.model_name);
                self.load(&path);
            },
            
            
            b'c' | b'C' => self.clear(),
            
            
            AI_ => self.cam_angle_y -= 0.15,
            AJ_ => self.cam_angle_y += 0.15,
            T_ => {
                self.cam_angle_x = (self.cam_angle_x - 0.15).max(-1.5);
            },
            S_ => {
                self.cam_angle_x = (self.cam_angle_x + 0.15).min(1.5);
            },
            
            
            b'd' | b'D' => {
                self.selected_verts.clear();
                self.edge_start = None;
                self.status_msg = String::from("Deselected all");
            },
            
            
            b'+' | b'=' => {
                self.cam_dist = (self.cam_dist - 0.5).max(1.5);
            },
            b'-' | b'_' => {
                self.cam_dist = (self.cam_dist + 0.5).min(20.0);
            },
            
            
            b'r' | b'R' => {
                self.cam_angle_y = 0.6;
                self.cam_angle_x = 0.4;
                self.cam_dist = 6.0;
                self.cam_target = V3 { x: 0.0, y: 0.0, z: 0.0 };
                self.status_msg = String::from("Camera reset");
            },
            
            _ => {}
        }
    }
    
    
    pub fn handle_click(&mut self, vx: i32, vy: i32, bt: usize, ex: usize, pressed: bool) {
        if pressed {
            self.mouse_pressed = true;
            self.drag_start_x = vx;
            self.drag_start_y = vy;
            self.mouse_dragging = false;
            
            match self.tool {
                EditTool::Select => {
                    if let Some(idx) = self.find_vertex_at(vx, vy, bt, ex) {
                        if self.selected_verts.contains(&idx) {
                            self.selected_verts.retain(|&v| v != idx);
                        } else {
                            self.selected_verts.push(idx);
                        }
                        self.status_msg = format!("Selected {} vertex(es)", self.selected_verts.len());
                    } else {
                        self.selected_verts.clear();
                        self.status_msg = String::from("Selection cleared");
                    }
                },
                EditTool::AddVertex => {
                    self.push_undo();
                    let bkb = self.unproject_to_grid(vx, vy, bt, ex);
                    self.vertices.push(bkb);
                    self.status_msg = format!("Added vertex at ({:.1}, {:.1}, {:.1}) — {} total",
                        bkb.x, bkb.y, bkb.z, self.vertices.len());
                },
                EditTool::AddEdge => {
                    if let Some(idx) = self.find_vertex_at(vx, vy, bt, ex) {
                        if let Some(start) = self.edge_start {
                            if start != idx {
                                
                                let exists = self.edges.iter().any(|&(a, b)| 
                                    (a == start && b == idx) || (a == idx && b == start));
                                if !exists {
                                    self.push_undo();
                                    self.edges.push((start, idx));
                                    self.status_msg = format!("Edge {} -> {} created — {} edges total",
                                        start, idx, self.edges.len());
                                } else {
                                    self.status_msg = String::from("Edge already exists");
                                }
                            }
                            self.edge_start = None;
                        } else {
                            self.edge_start = Some(idx);
                            self.status_msg = format!("Edge start: vertex {} — Click another vertex", idx);
                        }
                    } else {
                        self.edge_start = None;
                        self.status_msg = String::from("No vertex found — click on a vertex");
                    }
                },
                EditTool::Move => {
                    if let Some(idx) = self.find_vertex_at(vx, vy, bt, ex) {
                        if !self.selected_verts.contains(&idx) {
                            self.selected_verts.clear();
                            self.selected_verts.push(idx);
                        }
                        self.push_undo();
                    }
                },
                EditTool::Delete => {
                    if let Some(idx) = self.find_vertex_at(vx, vy, bt, ex) {
                        self.push_undo();
                        self.delete_vertex(idx);
                        self.status_msg = format!("Deleted vertex {} — {} remaining", idx, self.vertices.len());
                    }
                },
            }
        } else {
            self.mouse_pressed = false;
            self.mouse_dragging = false;
        }
    }
    
    
    pub fn handle_mouse_move(&mut self, vx: i32, vy: i32, bt: usize, ex: usize) {
        let dx = vx - self.mouse_x;
        let ad = vy - self.mouse_y;
        self.mouse_x = vx;
        self.mouse_y = vy;
        
        
        self.hover_vert = self.find_vertex_at(vx, vy, bt, ex);
        
        if self.mouse_pressed {
            let fde = (vx - self.drag_start_x).abs();
            let plv = (vy - self.drag_start_y).abs();
            if fde > 3 || plv > 3 {
                self.mouse_dragging = true;
            }
            
            if self.mouse_dragging {
                match self.tool {
                    EditTool::Move if !self.selected_verts.is_empty() => {
                        
                        let iok = 0.01 * self.cam_dist;
                        let iol = dx as f32 * iok;
                        let ngk = -ad as f32 * iok;
                        
                        
                        let u = hr(self.cam_angle_y);
                        let ak = eu(self.cam_angle_y);
                        
                        for &idx in &self.selected_verts.clone() {
                            if idx < self.vertices.len() {
                                self.vertices[idx].x += iol * u;
                                self.vertices[idx].z += iol * ak;
                                self.vertices[idx].y += ngk;
                            }
                        }
                    },
                    EditTool::Select | EditTool::AddVertex | EditTool::AddEdge | EditTool::Delete => {
                        
                        self.cam_angle_y += dx as f32 * 0.01;
                        self.cam_angle_x = (self.cam_angle_x + ad as f32 * 0.01).clamp(-1.5, 1.5);
                    },
                    _ => {}
                }
            }
        }
    }
    
    
    pub fn handle_scroll(&mut self, mk: i8) {
        if mk > 0 {
            self.cam_dist = (self.cam_dist - 0.5).max(1.5);
        } else {
            self.cam_dist = (self.cam_dist + 0.5).min(20.0);
        }
    }
    
    fn delete_vertex(&mut self, idx: usize) {
        if idx >= self.vertices.len() { return; }
        self.vertices.remove(idx);
        
        self.edges.retain(|&(a, b)| a != idx && b != idx);
        for th in &mut self.edges {
            if th.0 > idx { th.0 -= 1; }
            if th.1 > idx { th.1 -= 1; }
        }
        self.selected_verts.retain(|&v| v != idx);
        for amx in &mut self.selected_verts {
            if *amx > idx { *amx -= 1; }
        }
    }
    
    fn delete_selected(&mut self) {
        if self.selected_verts.is_empty() { return; }
        self.push_undo();
        
        let mut aph = self.selected_verts.clone();
        aph.sort_unstable();
        aph.reverse();
        for idx in aph {
            if idx < self.vertices.len() {
                self.vertices.remove(idx);
                self.edges.retain(|&(a, b)| a != idx && b != idx);
                for th in &mut self.edges {
                    if th.0 > idx { th.0 -= 1; }
                    if th.1 > idx { th.1 -= 1; }
                }
            }
        }
        let count = self.selected_verts.len();
        self.selected_verts.clear();
        self.status_msg = format!("Deleted {} vertices", count);
    }
    
    
    
    pub fn render(&mut self, buf: &mut [u32], w: usize, h: usize) {
        self.frame += 1;
        
        
        self.cached_cy = hr(self.cam_angle_y);
        self.cached_sy = eu(self.cam_angle_y);
        self.cached_cx = hr(self.cam_angle_x);
        self.cached_sx = eu(self.cam_angle_x);
        
        let aiy = 32;
        let aej = 20;
        let viewport_y = aiy;
        let hbm = h.saturating_sub(aiy + aej);
        
        
        for ct in buf.iter_mut() {
            *ct = IK_;
        }
        
        
        self.draw_toolbar(buf, w, h, aiy);
        
        
        if hbm > 10 {
            self.draw_viewport(buf, w, h, 0, viewport_y, w, hbm);
        }
        
        
        self.draw_status_bar(buf, w, h, h - aej, aej);
        
        
        if self.show_tips {
            self.draw_tips(buf, w, h, viewport_y, hbm);
        }
    }
    
    fn draw_toolbar(&self, buf: &mut [u32], w: usize, h: usize, bwj: usize) {
        
        for y in 0..bwj {
            for x in 0..w {
                buf[y * w + x] = TJ_;
            }
        }
        
        
        if bwj < h {
            for x in 0..w {
                buf[bwj * w + x] = 0xFF333355;
            }
        }
        
        
        let gzk = [EditTool::Select, EditTool::AddVertex, EditTool::AddEdge, EditTool::Move, EditTool::Delete];
        let gu = 60;
        let hn = 22;
        let ed = 5;
        let mut bx = 8;
        
        for tool in &gzk {
            let active = self.tool == *tool;
            let bg = if active { BQU_ } else { NH_ };
            fill_rect(buf, w, h, bx, ed, gu, hn, bg);
            
            draw_rect(buf, w, h, bx, ed, gu, hn, if active { 0xFF00AAFF } else { 0xFF444466 });
            
            let label = format!("{} [{}]", tool.name(), tool.hotkey());
            aby(buf, w, h, bx + 3, ed + 7, &label, 
                if active { AQR_ } else { P_ });
            bx += gu + 4;
        }
        
        
        let dwu = ["1:Cube", "2:Pyr", "3:Dia", "4:Tor", "5:Ico", "6:Grid"];
        let mut p = w - 8;
        for preset in dwu.iter().rev() {
            let wl = preset.len() * 6 + 8;
            p -= wl;
            fill_rect(buf, w, h, p, ed, wl, hn, NH_);
            draw_rect(buf, w, h, p, ed, wl, hn, 0xFF444466);
            aby(buf, w, h, p + 4, ed + 7, preset, ABX_);
            p -= 4;
        }
        
        
        let bdn = bx + 20;
        fill_rect(buf, w, h, bdn, ed, 50, hn, NH_);
        draw_rect(buf, w, h, bdn, ed, 50, hn, 0xFF444466);
        aby(buf, w, h, bdn + 4, ed + 7, "Save[W]", P_);
        
        fill_rect(buf, w, h, bdn + 54, ed, 50, hn, NH_);
        draw_rect(buf, w, h, bdn + 54, ed, 50, hn, 0xFF444466);
        aby(buf, w, h, bdn + 58, ed + 7, "Load[L]", P_);
        
        fill_rect(buf, w, h, bdn + 108, ed, 50, hn, NH_);
        draw_rect(buf, w, h, bdn + 108, ed, 50, hn, 0xFF444466);
        aby(buf, w, h, bdn + 112, ed + 7, "Undo[Z]", P_);
    }
    
    fn draw_viewport(&self, buf: &mut [u32], fv: usize, ov: usize,
                     vx: usize, vy: usize, bt: usize, ex: usize) {
        
        if self.show_grid {
            self.draw_grid(buf, fv, ov, vx, vy, bt, ex);
        }
        
        
        if self.show_axes {
            self.draw_axes(buf, fv, ov, vx, vy, bt, ex);
        }
        
        
        for (idx, &(a, b)) in self.edges.iter().enumerate() {
            if a >= self.vertices.len() || b >= self.vertices.len() { continue; }
            let (bm, az, z0) = self.project_vertex(self.vertices[a], bt, ex);
            let (x1, y1, po) = self.project_vertex(self.vertices[b], bt, ex);
            if z0 < 0.1 || po < 0.1 { continue; }
            
            
            let sel = self.selected_verts.contains(&a) || self.selected_verts.contains(&b);
            let msl = self.edge_start == Some(a) || self.edge_start == Some(b);
            
            let color = if msl { 0xFFFF8800 } else if sel { BRA_ } else { BQZ_ };
            
            
            let avg_z = (z0 + po) * 0.5;
            let ln = (1.0 - (avg_z - 3.0) * 0.1).clamp(0.3, 1.0);
            let color = lty(color, ln);
            
            draw_line_buf(buf, fv, ov, 
                vx as i32 + bm, vy as i32 + az,
                vx as i32 + x1, vy as i32 + y1, color);
        }
        
        
        if self.show_vertices {
            for (i, v) in self.vertices.iter().enumerate() {
                let (am, ak, z) = self.project_vertex(*v, bt, ex);
                if z < 0.1 { continue; }
                
                let p = vx as i32 + am;
                let o = vy as i32 + ak;
                
                let selected = self.selected_verts.contains(&i);
                let hovered = self.hover_vert == Some(i);
                let edge_start = self.edge_start == Some(i);
                
                let color = if edge_start { 0xFFFF8800 }
                    else if selected { BQY_ }
                    else if hovered { BQX_ }
                    else { BQW_ };
                
                let size = if selected || hovered || edge_start { 3 } else { 2 };
                
                
                for ad in -size..=size {
                    for dx in -size..=size {
                        if dx * dx + ad * ad <= size * size {
                            put_buf_pixel(buf, fv, ov, p + dx, o + ad, color);
                        }
                    }
                }
                
                
                if selected || hovered {
                    let label = format!("{}", i);
                    aby(buf, fv, ov, (p + size + 3) as usize, (o - 3) as usize, &label, color);
                }
            }
        }
        
        
        if let Some(bjj) = self.edge_start {
            if bjj < self.vertices.len() {
                let (am, ak, z) = self.project_vertex(self.vertices[bjj], bt, ex);
                if z > 0.1 {
                    
                    lin(buf, fv, ov,
                        vx as i32 + am, vy as i32 + ak,
                        vx as i32 + self.mouse_x, vy as i32 + self.mouse_y,
                        0xFFFF8800);
                }
            }
        }
    }
    
    fn draw_grid(&self, buf: &mut [u32], fv: usize, ov: usize,
                 vx: usize, vy: usize, bt: usize, ex: usize) {
        let cw = AWA_ * 0.5;
        let step = AWA_ / AET_ as f32;
        
        for i in 0..=AET_ {
            let t = -cw + i as f32 * step;
            let msb = (i == AET_ / 2);
            let color = if msb { BQM_ } else { BQL_ };
            
            
            let (bm, az, z0) = self.project_vertex(V3 { x: -cw, y: 0.0, z: t }, bt, ex);
            let (x1, y1, po) = self.project_vertex(V3 { x: cw, y: 0.0, z: t }, bt, ex);
            if z0 > 0.1 && po > 0.1 {
                draw_line_buf(buf, fv, ov, 
                    vx as i32 + bm, vy as i32 + az,
                    vx as i32 + x1, vy as i32 + y1, color);
            }
            
            
            let (bm, az, z0) = self.project_vertex(V3 { x: t, y: 0.0, z: -cw }, bt, ex);
            let (x1, y1, po) = self.project_vertex(V3 { x: t, y: 0.0, z: cw }, bt, ex);
            if z0 > 0.1 && po > 0.1 {
                draw_line_buf(buf, fv, ov, 
                    vx as i32 + bm, vy as i32 + az,
                    vx as i32 + x1, vy as i32 + y1, color);
            }
        }
    }
    
    fn draw_axes(&self, buf: &mut [u32], fv: usize, ov: usize,
                 vx: usize, vy: usize, bt: usize, ex: usize) {
        let origin = V3 { x: 0.0, y: 0.0, z: 0.0 };
        let jyv = [
            (V3 { x: AAN_, y: 0.0, z: 0.0 }, BQI_, "X"),
            (V3 { x: 0.0, y: AAN_, z: 0.0 }, BQJ_, "Y"),
            (V3 { x: 0.0, y: 0.0, z: AAN_ }, BQK_, "Z"),
        ];
        
        let (fh, hk, evy) = self.project_vertex(origin, bt, ex);
        if evy < 0.1 { return; }
        
        for (end, color, label) in &jyv {
            let (ajq, qz, ez) = self.project_vertex(*end, bt, ex);
            if ez < 0.1 { continue; }
            
            
            for d in -1..=1i32 {
                draw_line_buf(buf, fv, ov,
                    vx as i32 + fh + d, vy as i32 + hk,
                    vx as i32 + ajq + d, vy as i32 + qz, *color);
                draw_line_buf(buf, fv, ov,
                    vx as i32 + fh, vy as i32 + hk + d,
                    vx as i32 + ajq, vy as i32 + qz + d, *color);
            }
            
            
            aby(buf, fv, ov, (vx as i32 + ajq + 5) as usize, (vy as i32 + qz - 3) as usize,
                label, *color);
        }
    }
    
    fn draw_status_bar(&self, buf: &mut [u32], w: usize, h: usize, ak: usize, dw: usize) {
        
        for y in ak..ak + dw {
            if y >= h { break; }
            for x in 0..w {
                buf[y * w + x] = ABW_;
            }
        }
        
        if ak > 0 && ak < h {
            for x in 0..w {
                buf[ak * w + x] = 0xFF333355;
            }
        }
        
        
        aby(buf, w, h, 8, ak + 6, &self.status_msg, P_);
        
        
        let stats = format!("V:{} E:{} | {} | Zoom:{:.1}", 
            self.vertices.len(), self.edges.len(), self.model_name, self.cam_dist);
        let owu = w.saturating_sub(stats.len() * 6 + 8);
        aby(buf, w, h, owu, ak + 6, &stats, ABX_);
    }
    
    fn draw_tips(&self, buf: &mut [u32], w: usize, h: usize, vy: usize, ex: usize) {
        let he = 200;
        let ug = 180;
        let zc = w.saturating_sub(he + 10);
        let xg = vy + 10;
        
        if xg + ug >= h { return; }
        
        
        for y in xg..xg + ug {
            for x in zc..zc + he {
                if y < h && x < w {
                    buf[y * w + x] = cgi(buf[y * w + x], AQS_);
                }
            }
        }
        
        
        draw_rect(buf, w, h, zc, xg, he, ug, TI_);
        
        let mut ty = xg + 6;
        let bu = zc + 8;
        
        aby(buf, w, h, bu, ty, "--- Controls ---", TI_);
        ty += 12;
        aby(buf, w, h, bu, ty, "S: Select  A: Add Vtx", P_);
        ty += 10;
        aby(buf, w, h, bu, ty, "E: Add Edge  G: Move", P_);
        ty += 10;
        aby(buf, w, h, bu, ty, "X: Delete  D: Deselect", P_);
        ty += 10;
        aby(buf, w, h, bu, ty, "Z: Undo  C: Clear", P_);
        ty += 10;
        aby(buf, w, h, bu, ty, "Arrows: Orbit camera", P_);
        ty += 10;
        aby(buf, w, h, bu, ty, "+/-: Zoom  R: Reset cam", P_);
        ty += 14;
        aby(buf, w, h, bu, ty, "--- Presets ---", TI_);
        ty += 12;
        aby(buf, w, h, bu, ty, "1:Cube 2:Pyr 3:Dia", P_);
        ty += 10;
        aby(buf, w, h, bu, ty, "4:Torus 5:Ico 6:Grid", P_);
        ty += 14;
        aby(buf, w, h, bu, ty, "--- File ---", TI_);
        ty += 12;
        aby(buf, w, h, bu, ty, "W: Save  L: Load", P_);
        ty += 10;
        aby(buf, w, h, bu, ty, "H: Toggle this panel", ABX_);
        
        
        let tip = self.tool.tip();
        if tip.len() > 0 {
            let crq = xg + ug + 5;
            if crq + 16 < h {
                let pjx = tip.len() * 6 + 12;
                fill_rect(buf, w, h, zc, crq, pjx, 14, AQS_ & 0xDDFFFFFF);
                aby(buf, w, h, zc + 6, crq + 3, tip, AQR_);
            }
        }
    }
}



use crate::draw_utils::{
    put_pixel as put_buf_pixel,
    fill_rect,
    draw_rect,
    draw_line as draw_line_buf,
};

fn lin(buf: &mut [u32], w: usize, h: usize, bm: i32, az: i32, x1: i32, y1: i32, color: u32) {
    let mut x = bm;
    let mut y = az;
    let dx = (x1 - bm).abs();
    let ad = -(y1 - az).abs();
    let am = if bm < x1 { 1 } else { -1 };
    let ak = if az < y1 { 1 } else { -1 };
    let mut err = dx + ad;
    let mut step = 0;
    let ayd = (dx.abs() + ad.abs()) as usize + 1;
    let ayd = ayd.min(4000);
    
    for _ in 0..ayd {
        if step % 8 < 4 {
            put_buf_pixel(buf, w, h, x, y, color);
        }
        step += 1;
        if x == x1 && y == y1 { break; }
        let pg = 2 * err;
        if pg >= ad { err += ad; x += am; }
        if pg <= dx { err += dx; y += ak; }
    }
}

fn lty(color: u32, ha: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32 * ha;
    let g = ((color >> 8) & 0xFF) as f32 * ha;
    let b = (color & 0xFF) as f32 * ha;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn cgi(bg: u32, fg: u32) -> u32 {
    let alpha = ((fg >> 24) & 0xFF) as f32 / 255.0;
    let ki = 1.0 - alpha;
    let r = ((fg >> 16) & 0xFF) as f32 * alpha + ((bg >> 16) & 0xFF) as f32 * ki;
    let g = ((fg >> 8) & 0xFF) as f32 * alpha + ((bg >> 8) & 0xFF) as f32 * ki;
    let b = (fg & 0xFF) as f32 * alpha + (bg & 0xFF) as f32 * ki;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}


fn aby(buf: &mut [u32], fv: usize, ov: usize, x: usize, y: usize, text: &str, color: u32) {
    let mut cx = x;
    for ch in text.bytes() {
        lkt(buf, fv, ov, cx, y, ch, color);
        cx += 6;
    }
}


fn lkt(buf: &mut [u32], fv: usize, ov: usize, x: usize, y: usize, ch: u8, color: u32) {
    
    let c = ch as char;
    if c == ' ' { return; }
    
    
    let bits: [u8; 7] = match c {
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        'A' | 'a' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' | 'b' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' | 'c' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' | 'd' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' | 'e' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' | 'f' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' | 'g' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' | 'h' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' | 'i' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' | 'j' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' | 'k' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' | 'l' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' | 'm' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' | 'n' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' | 'o' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' | 'p' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' | 'q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' | 'r' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' | 's' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
        'T' | 't' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' | 'u' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' | 'v' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100],
        'W' | 'w' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' | 'x' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' | 'y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' | 'z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        ':' => [0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '=' => [0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        '[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
        ']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
        '>' => [0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000],
        '<' => [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010],
        '|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        '_' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
        '?' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00100, 0b00000, 0b00100],
        '#' => [0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010],
        '*' => [0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000],
        '~' => [0b00000, 0b00000, 0b01000, 0b10101, 0b00010, 0b00000, 0b00000],
        _ => [0b01110, 0b01010, 0b01010, 0b01010, 0b01010, 0b01010, 0b01110], 
    };
    
    for row in 0..7 {
        for col in 0..5 {
            if bits[row] & (1 << (4 - col)) != 0 {
                let p = x + col;
                let o = y + row;
                if p < fv && o < ov {
                    buf[o * fv + p] = color;
                }
            }
        }
    }
}
