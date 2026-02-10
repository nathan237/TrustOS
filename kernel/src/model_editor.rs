// ═══════════════════════════════════════════════════════════════════════════
// TRUSTEDIT 3D — Interactive Wireframe Model Editor
// A simplified Blender-style modal editor for creating & editing 3D meshes
// Features: orbital camera, vertex/edge tools, preset shapes, save/load
// ═══════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::formula3d::{V3, Mesh, fast_sin, fast_cos};

// ─── Constants ───────────────────────────────────────────────────────────

const GRID_SIZE: f32 = 8.0;
const GRID_DIVISIONS: usize = 16;
const VERTEX_SELECT_RADIUS: f32 = 12.0;
const AXIS_LENGTH: f32 = 2.0;

// Colors
const COL_BG: u32        = 0xFF1A1A2E;
const COL_GRID: u32      = 0xFF2A2A3A;
const COL_GRID_CENTER: u32 = 0xFF3A3A4A;
const COL_AXIS_X: u32    = 0xFFFF4444;
const COL_AXIS_Y: u32    = 0xFF44FF44;
const COL_AXIS_Z: u32    = 0xFF4488FF;
const COL_WIRE: u32      = 0xFF00FFAA;
const COL_WIRE_SEL: u32  = 0xFFFFFF00;
const COL_VERTEX: u32    = 0xFFFFFFFF;
const COL_VERT_SEL: u32  = 0xFFFFAA00;
const COL_VERT_HOVER: u32 = 0xFF88FFFF;
const COL_TOOLBAR: u32   = 0xFF0D0D1A;
const COL_TOOLBAR_BTN: u32 = 0xFF2A2A4A;
const COL_TOOLBAR_ACT: u32 = 0xFF0055AA;
const COL_STATUS: u32    = 0xFF0D0D1A;
const COL_TEXT: u32      = 0xFFCCCCCC;
const COL_TEXT_HI: u32   = 0xFFFFFFFF;
const COL_TEXT_DIM: u32  = 0xFF777777;
const COL_TIP_BG: u32    = 0xCC1A1A2E;
const COL_TIP_BORDER: u32 = 0xFF00AAFF;

// ─── Tool / Mode ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum EditTool {
    Select,     // Click to select vertices/edges
    AddVertex,  // Click to place vertices on the grid plane
    AddEdge,    // Select two vertices to connect
    Move,       // Grab and move selected vertices
    Delete,     // Click to delete vertices/edges
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

// ─── Editor State ────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ModelEditorState {
    // Mesh data
    pub vertices: Vec<V3>,
    pub edges: Vec<(usize, usize)>,
    
    // Camera (orbital)
    pub cam_angle_y: f32,    // horizontal orbit (radians)
    pub cam_angle_x: f32,    // vertical orbit (radians)
    pub cam_dist: f32,       // distance from origin
    pub cam_target: V3,      // look-at point
    
    // Interaction
    pub tool: EditTool,
    pub selected_verts: Vec<usize>,
    pub hover_vert: Option<usize>,
    pub edge_start: Option<usize>, // for AddEdge tool
    
    // Mouse state (relative to viewport)
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_pressed: bool,
    pub mouse_dragging: bool,
    pub drag_start_x: i32,
    pub drag_start_y: i32,
    pub right_dragging: bool,  // orbit camera
    
    // UI
    pub show_grid: bool,
    pub show_axes: bool,
    pub show_tips: bool,
    pub show_vertices: bool,
    pub status_msg: String,
    pub model_name: String,
    
    // Undo
    pub undo_stack: Vec<(Vec<V3>, Vec<(usize, usize)>)>,
    
    // Frame counter for animation
    frame: u32,
    // Cached trig values (recomputed once per frame for perf)
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
            cached_cy: fast_cos(0.6),
            cached_sy: fast_sin(0.6),
            cached_cx: fast_cos(0.4),
            cached_sx: fast_sin(0.4),
        }
    }
    
    /// Load a preset mesh
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
    
    /// Clear the scene
    pub fn clear(&mut self) {
        self.push_undo();
        self.vertices.clear();
        self.edges.clear();
        self.selected_verts.clear();
        self.edge_start = None;
        self.status_msg = String::from("Scene cleared");
    }
    
    /// Push current state to undo stack
    fn push_undo(&mut self) {
        if self.undo_stack.len() > 20 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push((self.vertices.clone(), self.edges.clone()));
    }
    
    /// Undo last action
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
    
    /// Save mesh to TrustFS (ramfs)
    pub fn save(&mut self) {
        // Serialize to a simple text format
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
        
        // Touch (create) the file first, then write
        let _ = crate::ramfs::with_fs(|fs| fs.touch(&path));
        match crate::ramfs::with_fs(|fs| fs.write_file(&path, bytes)) {
            Ok(_) => {
                self.status_msg = format!("Saved to {}", path);
            },
            Err(_) => {
                self.status_msg = String::from("Error: Could not save file");
            }
        }
    }
    
    /// Load mesh from TrustFS
    pub fn load(&mut self, path: &str) {
        let data_result: Result<Vec<u8>, ()> = crate::ramfs::with_fs(|fs| {
            match fs.read_file(path) {
                Ok(bytes) => Ok(bytes.to_vec()),
                Err(_) => Err(()),
            }
        });
        
        match data_result {
            Ok(data) => {
                self.push_undo();
                if let Ok(text) = core::str::from_utf8(&data) {
                    self.parse_t3d(text);
                    // Extract name from path
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
    
    /// Parse .t3d format
    fn parse_t3d(&mut self, text: &str) {
        self.vertices.clear();
        self.edges.clear();
        self.selected_verts.clear();
        
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }
            match parts[0] {
                "v" if parts.len() >= 4 => {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        self.vertices.push(V3 { x, y, z });
                    }
                },
                "e" if parts.len() >= 3 => {
                    if let (Ok(a), Ok(b)) = (
                        parts[1].parse::<usize>(),
                        parts[2].parse::<usize>(),
                    ) {
                        self.edges.push((a, b));
                    }
                },
                _ => {}
            }
        }
    }
    
    /// Export to formula3d Mesh format
    pub fn to_mesh(&self) -> Mesh {
        Mesh {
            vertices: self.vertices.clone(),
            edges: self.edges.clone(),
            edge_colors: None,
        }
    }
    
    // ─── 3D Transform Pipeline ───────────────────────────────────────────
    
    fn project_vertex(&self, v: V3, vw: usize, vh: usize) -> (i32, i32, f32) {
        // Translate relative to camera target
        let v = V3 { 
            x: v.x - self.cam_target.x, 
            y: v.y - self.cam_target.y, 
            z: v.z - self.cam_target.z 
        };
        // Rotate around Y then X (orbit camera) — using cached sin/cos
        let cy = self.cached_cy;
        let sy = self.cached_sy;
        let rx = V3 { x: v.x * cy + v.z * sy, y: v.y, z: -v.x * sy + v.z * cy };
        
        let cx = self.cached_cx;
        let sx = self.cached_sx;
        let ry = V3 { x: rx.x, y: rx.y * cx - rx.z * sx, z: rx.y * sx + rx.z * cx };
        
        // Translate along camera distance
        let tz = ry.z + self.cam_dist;
        
        // Perspective projection
        if tz < 0.1 { return (0, 0, -999.0); }
        let scale = (vw.min(vh) as f32) * 0.45;
        let sx_proj = (ry.x / tz * scale) + (vw as f32 * 0.5);
        let sy_proj = (-ry.y / tz * scale) + (vh as f32 * 0.5);
        (sx_proj as i32, sy_proj as i32, tz)
    }
    
    /// Unproject a screen point to a 3D position on the Y=0 plane
    fn unproject_to_grid(&self, sx: i32, sy: i32, vw: usize, vh: usize) -> V3 {
        // Reverse perspective: get ray direction
        let scale = (vw.min(vh) as f32) * 0.45;
        let rx = (sx as f32 - vw as f32 * 0.5) / scale;
        let ry = -(sy as f32 - vh as f32 * 0.5) / scale;
        
        // Ray direction in camera space
        let dir = V3 { x: rx, y: ry, z: 1.0 };
        
        // Unrotate by camera angles (inverse rotation)
        let cx = fast_cos(-self.cam_angle_x);
        let sx_a = fast_sin(-self.cam_angle_x);
        let d1 = V3 { x: dir.x, y: dir.y * cx - dir.z * sx_a, z: dir.y * sx_a + dir.z * cx };
        
        let cy = fast_cos(-self.cam_angle_y);
        let sy_a = fast_sin(-self.cam_angle_y);
        let d2 = V3 { x: d1.x * cy + d1.z * sy_a, y: d1.y, z: -d1.x * sy_a + d1.z * cy };
        
        // Camera position in world space
        let cam_pos = V3 {
            x: self.cam_target.x - self.cam_dist * fast_sin(self.cam_angle_y) * fast_cos(self.cam_angle_x),
            y: self.cam_target.y + self.cam_dist * fast_sin(self.cam_angle_x),
            z: self.cam_target.z - self.cam_dist * fast_cos(self.cam_angle_y) * fast_cos(self.cam_angle_x),
        };
        
        // Intersect with Y=0 plane
        if d2.y.abs() < 0.001 {
            // Ray parallel to grid — place at distance
            return V3 { x: cam_pos.x + d2.x * 5.0, y: 0.0, z: cam_pos.z + d2.z * 5.0 };
        }
        let t = -cam_pos.y / d2.y;
        if t < 0.0 {
            // Behind camera
            return V3 { x: cam_pos.x + d2.x * 5.0, y: 0.0, z: cam_pos.z + d2.z * 5.0 };
        }
        
        let world = V3 {
            x: cam_pos.x + d2.x * t,
            y: 0.0,
            z: cam_pos.z + d2.z * t,
        };
        
        // Snap to grid (0.5 units)
        V3 {
            x: (world.x * 2.0 + 0.5) as i32 as f32 * 0.5,
            y: 0.0,
            z: (world.z * 2.0 + 0.5) as i32 as f32 * 0.5,
        }
    }
    
    /// Find closest vertex to screen position
    fn find_vertex_at(&self, sx: i32, sy: i32, vw: usize, vh: usize) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, v) in self.vertices.iter().enumerate() {
            let (px, py, z) = self.project_vertex(*v, vw, vh);
            if z < 0.1 { continue; }
            let dx = (px - sx) as f32;
            let dy = (py - sy) as f32;
            let dist = dx * dx + dy * dy;
            if dist < VERTEX_SELECT_RADIUS * VERTEX_SELECT_RADIUS {
                if let Some((_, best_d)) = best {
                    if dist < best_d { best = Some((i, dist)); }
                } else {
                    best = Some((i, dist));
                }
            }
        }
        best.map(|(i, _)| i)
    }
    
    // ─── Input Handling ──────────────────────────────────────────────────
    
    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::*;
        
        match key {
            // Tool selection
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
                    // If already in delete mode, delete selected vertices
                    if !self.selected_verts.is_empty() {
                        self.delete_selected();
                    }
                } else {
                    self.tool = EditTool::Delete;
                    self.edge_start = None;
                    self.status_msg = String::from("Tool: Delete — Click to remove");
                }
            },
            
            // Undo
            b'z' | b'Z' => self.undo(),
            
            // Toggle display
            b'h' | b'H' => {
                self.show_tips = !self.show_tips;
                self.status_msg = format!("Tips: {}", if self.show_tips { "ON" } else { "OFF" });
            },
            b'v' | b'V' => {
                self.show_vertices = !self.show_vertices;
            },
            
            // Presets
            b'1' => self.load_preset("cube"),
            b'2' => self.load_preset("pyramid"),
            b'3' => self.load_preset("diamond"),
            b'4' => self.load_preset("torus"),
            b'5' => self.load_preset("icosphere"),
            b'6' => self.load_preset("grid"),
            
            // Save/Load  
            b'w' | b'W' => self.save(),
            b'l' | b'L' => {
                let path = format!("/{}.t3d", self.model_name);
                self.load(&path);
            },
            
            // Clear scene
            b'c' | b'C' => self.clear(),
            
            // Camera orbit with arrow keys
            KEY_LEFT => self.cam_angle_y -= 0.15,
            KEY_RIGHT => self.cam_angle_y += 0.15,
            KEY_UP => {
                self.cam_angle_x = (self.cam_angle_x - 0.15).max(-1.5);
            },
            KEY_DOWN => {
                self.cam_angle_x = (self.cam_angle_x + 0.15).min(1.5);
            },
            
            // Select all / deselect all
            b'd' | b'D' => {
                self.selected_verts.clear();
                self.edge_start = None;
                self.status_msg = String::from("Deselected all");
            },
            
            // Camera zoom
            b'+' | b'=' => {
                self.cam_dist = (self.cam_dist - 0.5).max(1.5);
            },
            b'-' | b'_' => {
                self.cam_dist = (self.cam_dist + 0.5).min(20.0);
            },
            
            // Camera reset
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
    
    /// Handle mouse click in viewport coordinates
    pub fn handle_click(&mut self, vx: i32, vy: i32, vw: usize, vh: usize, pressed: bool) {
        if pressed {
            self.mouse_pressed = true;
            self.drag_start_x = vx;
            self.drag_start_y = vy;
            self.mouse_dragging = false;
            
            match self.tool {
                EditTool::Select => {
                    if let Some(idx) = self.find_vertex_at(vx, vy, vw, vh) {
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
                    let world = self.unproject_to_grid(vx, vy, vw, vh);
                    self.vertices.push(world);
                    self.status_msg = format!("Added vertex at ({:.1}, {:.1}, {:.1}) — {} total",
                        world.x, world.y, world.z, self.vertices.len());
                },
                EditTool::AddEdge => {
                    if let Some(idx) = self.find_vertex_at(vx, vy, vw, vh) {
                        if let Some(start) = self.edge_start {
                            if start != idx {
                                // Check if edge already exists
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
                    if let Some(idx) = self.find_vertex_at(vx, vy, vw, vh) {
                        if !self.selected_verts.contains(&idx) {
                            self.selected_verts.clear();
                            self.selected_verts.push(idx);
                        }
                        self.push_undo();
                    }
                },
                EditTool::Delete => {
                    if let Some(idx) = self.find_vertex_at(vx, vy, vw, vh) {
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
    
    /// Handle mouse move in viewport coordinates
    pub fn handle_mouse_move(&mut self, vx: i32, vy: i32, vw: usize, vh: usize) {
        let dx = vx - self.mouse_x;
        let dy = vy - self.mouse_y;
        self.mouse_x = vx;
        self.mouse_y = vy;
        
        // Update hover
        self.hover_vert = self.find_vertex_at(vx, vy, vw, vh);
        
        if self.mouse_pressed {
            let total_dx = (vx - self.drag_start_x).abs();
            let total_dy = (vy - self.drag_start_y).abs();
            if total_dx > 3 || total_dy > 3 {
                self.mouse_dragging = true;
            }
            
            if self.mouse_dragging {
                match self.tool {
                    EditTool::Move if !self.selected_verts.is_empty() => {
                        // Move selected vertices
                        let move_scale = 0.01 * self.cam_dist;
                        let move_x = dx as f32 * move_scale;
                        let move_y = -dy as f32 * move_scale;
                        
                        // Move in camera-relative direction
                        let cy = fast_cos(self.cam_angle_y);
                        let sy = fast_sin(self.cam_angle_y);
                        
                        for &idx in &self.selected_verts.clone() {
                            if idx < self.vertices.len() {
                                self.vertices[idx].x += move_x * cy;
                                self.vertices[idx].z += move_x * sy;
                                self.vertices[idx].y += move_y;
                            }
                        }
                    },
                    EditTool::Select | EditTool::AddVertex | EditTool::AddEdge | EditTool::Delete => {
                        // Orbit camera on drag (except Move tool)
                        self.cam_angle_y += dx as f32 * 0.01;
                        self.cam_angle_x = (self.cam_angle_x + dy as f32 * 0.01).clamp(-1.5, 1.5);
                    },
                    _ => {}
                }
            }
        }
    }
    
    /// Handle scroll for zoom
    pub fn handle_scroll(&mut self, delta: i8) {
        if delta > 0 {
            self.cam_dist = (self.cam_dist - 0.5).max(1.5);
        } else {
            self.cam_dist = (self.cam_dist + 0.5).min(20.0);
        }
    }
    
    fn delete_vertex(&mut self, idx: usize) {
        if idx >= self.vertices.len() { return; }
        self.vertices.remove(idx);
        // Remove edges that reference this vertex and fix indices
        self.edges.retain(|&(a, b)| a != idx && b != idx);
        for edge in &mut self.edges {
            if edge.0 > idx { edge.0 -= 1; }
            if edge.1 > idx { edge.1 -= 1; }
        }
        self.selected_verts.retain(|&v| v != idx);
        for sv in &mut self.selected_verts {
            if *sv > idx { *sv -= 1; }
        }
    }
    
    fn delete_selected(&mut self) {
        if self.selected_verts.is_empty() { return; }
        self.push_undo();
        // Sort descending to remove from end first
        let mut to_remove = self.selected_verts.clone();
        to_remove.sort_unstable();
        to_remove.reverse();
        for idx in to_remove {
            if idx < self.vertices.len() {
                self.vertices.remove(idx);
                self.edges.retain(|&(a, b)| a != idx && b != idx);
                for edge in &mut self.edges {
                    if edge.0 > idx { edge.0 -= 1; }
                    if edge.1 > idx { edge.1 -= 1; }
                }
            }
        }
        let count = self.selected_verts.len();
        self.selected_verts.clear();
        self.status_msg = format!("Deleted {} vertices", count);
    }
    
    // ─── Rendering ───────────────────────────────────────────────────────
    
    pub fn render(&mut self, buf: &mut [u32], w: usize, h: usize) {
        self.frame += 1;
        
        // Cache trig values once per frame (avoids recomputing per vertex)
        self.cached_cy = fast_cos(self.cam_angle_y);
        self.cached_sy = fast_sin(self.cam_angle_y);
        self.cached_cx = fast_cos(self.cam_angle_x);
        self.cached_sx = fast_sin(self.cam_angle_x);
        
        let toolbar_h = 32;
        let status_h = 20;
        let viewport_y = toolbar_h;
        let viewport_h = h.saturating_sub(toolbar_h + status_h);
        
        // Clear with background
        for pixel in buf.iter_mut() {
            *pixel = COL_BG;
        }
        
        // Draw toolbar
        self.draw_toolbar(buf, w, h, toolbar_h);
        
        // Draw 3D viewport
        if viewport_h > 10 {
            self.draw_viewport(buf, w, h, 0, viewport_y, w, viewport_h);
        }
        
        // Draw status bar
        self.draw_status_bar(buf, w, h, h - status_h, status_h);
        
        // Draw tips overlay
        if self.show_tips {
            self.draw_tips(buf, w, h, viewport_y, viewport_h);
        }
    }
    
    fn draw_toolbar(&self, buf: &mut [u32], w: usize, h: usize, th: usize) {
        // Background
        for y in 0..th {
            for x in 0..w {
                buf[y * w + x] = COL_TOOLBAR;
            }
        }
        
        // Separator line
        if th < h {
            for x in 0..w {
                buf[th * w + x] = 0xFF333355;
            }
        }
        
        // Tool buttons
        let tools = [EditTool::Select, EditTool::AddVertex, EditTool::AddEdge, EditTool::Move, EditTool::Delete];
        let btn_w = 60;
        let btn_h = 22;
        let btn_y = 5;
        let mut bx = 8;
        
        for tool in &tools {
            let active = self.tool == *tool;
            let bg = if active { COL_TOOLBAR_ACT } else { COL_TOOLBAR_BTN };
            fill_rect(buf, w, h, bx, btn_y, btn_w, btn_h, bg);
            // Border
            draw_rect(buf, w, h, bx, btn_y, btn_w, btn_h, if active { 0xFF00AAFF } else { 0xFF444466 });
            // Label
            let label = format!("{} [{}]", tool.name(), tool.hotkey());
            draw_small_text(buf, w, h, bx + 3, btn_y + 7, &label, 
                if active { COL_TEXT_HI } else { COL_TEXT });
            bx += btn_w + 4;
        }
        
        // Right side: preset buttons
        let presets = ["1:Cube", "2:Pyr", "3:Dia", "4:Tor", "5:Ico", "6:Grid"];
        let mut px = w - 8;
        for preset in presets.iter().rev() {
            let pw = preset.len() * 6 + 8;
            px -= pw;
            fill_rect(buf, w, h, px, btn_y, pw, btn_h, COL_TOOLBAR_BTN);
            draw_rect(buf, w, h, px, btn_y, pw, btn_h, 0xFF444466);
            draw_small_text(buf, w, h, px + 4, btn_y + 7, preset, COL_TEXT_DIM);
            px -= 4;
        }
        
        // Save/Load in middle-right
        let save_x = bx + 20;
        fill_rect(buf, w, h, save_x, btn_y, 50, btn_h, COL_TOOLBAR_BTN);
        draw_rect(buf, w, h, save_x, btn_y, 50, btn_h, 0xFF444466);
        draw_small_text(buf, w, h, save_x + 4, btn_y + 7, "Save[W]", COL_TEXT);
        
        fill_rect(buf, w, h, save_x + 54, btn_y, 50, btn_h, COL_TOOLBAR_BTN);
        draw_rect(buf, w, h, save_x + 54, btn_y, 50, btn_h, 0xFF444466);
        draw_small_text(buf, w, h, save_x + 58, btn_y + 7, "Load[L]", COL_TEXT);
        
        fill_rect(buf, w, h, save_x + 108, btn_y, 50, btn_h, COL_TOOLBAR_BTN);
        draw_rect(buf, w, h, save_x + 108, btn_y, 50, btn_h, 0xFF444466);
        draw_small_text(buf, w, h, save_x + 112, btn_y + 7, "Undo[Z]", COL_TEXT);
    }
    
    fn draw_viewport(&self, buf: &mut [u32], bw: usize, bh: usize,
                     vx: usize, vy: usize, vw: usize, vh: usize) {
        // Draw grid
        if self.show_grid {
            self.draw_grid(buf, bw, bh, vx, vy, vw, vh);
        }
        
        // Draw axes
        if self.show_axes {
            self.draw_axes(buf, bw, bh, vx, vy, vw, vh);
        }
        
        // Draw edges
        for (idx, &(a, b)) in self.edges.iter().enumerate() {
            if a >= self.vertices.len() || b >= self.vertices.len() { continue; }
            let (x0, y0, z0) = self.project_vertex(self.vertices[a], vw, vh);
            let (x1, y1, z1) = self.project_vertex(self.vertices[b], vw, vh);
            if z0 < 0.1 || z1 < 0.1 { continue; }
            
            // Check if either vertex is selected
            let sel = self.selected_verts.contains(&a) || self.selected_verts.contains(&b);
            let is_edge_start = self.edge_start == Some(a) || self.edge_start == Some(b);
            
            let color = if is_edge_start { 0xFFFF8800 } else if sel { COL_WIRE_SEL } else { COL_WIRE };
            
            // Depth fade
            let avg_z = (z0 + z1) * 0.5;
            let fade = (1.0 - (avg_z - 3.0) * 0.1).clamp(0.3, 1.0);
            let color = fade_color(color, fade);
            
            draw_line_buf(buf, bw, bh, 
                vx as i32 + x0, vy as i32 + y0,
                vx as i32 + x1, vy as i32 + y1, color);
        }
        
        // Draw vertices
        if self.show_vertices {
            for (i, v) in self.vertices.iter().enumerate() {
                let (sx, sy, z) = self.project_vertex(*v, vw, vh);
                if z < 0.1 { continue; }
                
                let px = vx as i32 + sx;
                let py = vy as i32 + sy;
                
                let selected = self.selected_verts.contains(&i);
                let hovered = self.hover_vert == Some(i);
                let edge_start = self.edge_start == Some(i);
                
                let color = if edge_start { 0xFFFF8800 }
                    else if selected { COL_VERT_SEL }
                    else if hovered { COL_VERT_HOVER }
                    else { COL_VERTEX };
                
                let size = if selected || hovered || edge_start { 3 } else { 2 };
                
                // Draw vertex dot
                for dy in -size..=size {
                    for dx in -size..=size {
                        if dx * dx + dy * dy <= size * size {
                            put_buf_pixel(buf, bw, bh, px + dx, py + dy, color);
                        }
                    }
                }
                
                // Vertex index label
                if selected || hovered {
                    let label = format!("{}", i);
                    draw_small_text(buf, bw, bh, (px + size + 3) as usize, (py - 3) as usize, &label, color);
                }
            }
        }
        
        // Draw edge-in-progress line
        if let Some(start_idx) = self.edge_start {
            if start_idx < self.vertices.len() {
                let (sx, sy, z) = self.project_vertex(self.vertices[start_idx], vw, vh);
                if z > 0.1 {
                    // Dashed line from start vertex to mouse
                    draw_dashed_line(buf, bw, bh,
                        vx as i32 + sx, vy as i32 + sy,
                        vx as i32 + self.mouse_x, vy as i32 + self.mouse_y,
                        0xFFFF8800);
                }
            }
        }
    }
    
    fn draw_grid(&self, buf: &mut [u32], bw: usize, bh: usize,
                 vx: usize, vy: usize, vw: usize, vh: usize) {
        let half = GRID_SIZE * 0.5;
        let step = GRID_SIZE / GRID_DIVISIONS as f32;
        
        for i in 0..=GRID_DIVISIONS {
            let t = -half + i as f32 * step;
            let is_center = (i == GRID_DIVISIONS / 2);
            let color = if is_center { COL_GRID_CENTER } else { COL_GRID };
            
            // Lines along X
            let (x0, y0, z0) = self.project_vertex(V3 { x: -half, y: 0.0, z: t }, vw, vh);
            let (x1, y1, z1) = self.project_vertex(V3 { x: half, y: 0.0, z: t }, vw, vh);
            if z0 > 0.1 && z1 > 0.1 {
                draw_line_buf(buf, bw, bh, 
                    vx as i32 + x0, vy as i32 + y0,
                    vx as i32 + x1, vy as i32 + y1, color);
            }
            
            // Lines along Z
            let (x0, y0, z0) = self.project_vertex(V3 { x: t, y: 0.0, z: -half }, vw, vh);
            let (x1, y1, z1) = self.project_vertex(V3 { x: t, y: 0.0, z: half }, vw, vh);
            if z0 > 0.1 && z1 > 0.1 {
                draw_line_buf(buf, bw, bh, 
                    vx as i32 + x0, vy as i32 + y0,
                    vx as i32 + x1, vy as i32 + y1, color);
            }
        }
    }
    
    fn draw_axes(&self, buf: &mut [u32], bw: usize, bh: usize,
                 vx: usize, vy: usize, vw: usize, vh: usize) {
        let origin = V3 { x: 0.0, y: 0.0, z: 0.0 };
        let axes = [
            (V3 { x: AXIS_LENGTH, y: 0.0, z: 0.0 }, COL_AXIS_X, "X"),
            (V3 { x: 0.0, y: AXIS_LENGTH, z: 0.0 }, COL_AXIS_Y, "Y"),
            (V3 { x: 0.0, y: 0.0, z: AXIS_LENGTH }, COL_AXIS_Z, "Z"),
        ];
        
        let (ox, oy, oz) = self.project_vertex(origin, vw, vh);
        if oz < 0.1 { return; }
        
        for (end, color, label) in &axes {
            let (ex, ey, ez) = self.project_vertex(*end, vw, vh);
            if ez < 0.1 { continue; }
            
            // Draw thick axis line
            for d in -1..=1i32 {
                draw_line_buf(buf, bw, bh,
                    vx as i32 + ox + d, vy as i32 + oy,
                    vx as i32 + ex + d, vy as i32 + ey, *color);
                draw_line_buf(buf, bw, bh,
                    vx as i32 + ox, vy as i32 + oy + d,
                    vx as i32 + ex, vy as i32 + ey + d, *color);
            }
            
            // Axis label
            draw_small_text(buf, bw, bh, (vx as i32 + ex + 5) as usize, (vy as i32 + ey - 3) as usize,
                label, *color);
        }
    }
    
    fn draw_status_bar(&self, buf: &mut [u32], w: usize, h: usize, sy: usize, sh: usize) {
        // Background
        for y in sy..sy + sh {
            if y >= h { break; }
            for x in 0..w {
                buf[y * w + x] = COL_STATUS;
            }
        }
        // Separator
        if sy > 0 && sy < h {
            for x in 0..w {
                buf[sy * w + x] = 0xFF333355;
            }
        }
        
        // Status message
        draw_small_text(buf, w, h, 8, sy + 6, &self.status_msg, COL_TEXT);
        
        // Right: stats
        let stats = format!("V:{} E:{} | {} | Zoom:{:.1}", 
            self.vertices.len(), self.edges.len(), self.model_name, self.cam_dist);
        let stats_x = w.saturating_sub(stats.len() * 6 + 8);
        draw_small_text(buf, w, h, stats_x, sy + 6, &stats, COL_TEXT_DIM);
    }
    
    fn draw_tips(&self, buf: &mut [u32], w: usize, h: usize, vy: usize, vh: usize) {
        let panel_w = 200;
        let panel_h = 180;
        let panel_x = w.saturating_sub(panel_w + 10);
        let panel_y = vy + 10;
        
        if panel_y + panel_h >= h { return; }
        
        // Semi-transparent background
        for y in panel_y..panel_y + panel_h {
            for x in panel_x..panel_x + panel_w {
                if y < h && x < w {
                    buf[y * w + x] = blend_alpha(buf[y * w + x], COL_TIP_BG);
                }
            }
        }
        
        // Border
        draw_rect(buf, w, h, panel_x, panel_y, panel_w, panel_h, COL_TIP_BORDER);
        
        let mut ty = panel_y + 6;
        let tx = panel_x + 8;
        
        draw_small_text(buf, w, h, tx, ty, "--- Controls ---", COL_TIP_BORDER);
        ty += 12;
        draw_small_text(buf, w, h, tx, ty, "S: Select  A: Add Vtx", COL_TEXT);
        ty += 10;
        draw_small_text(buf, w, h, tx, ty, "E: Add Edge  G: Move", COL_TEXT);
        ty += 10;
        draw_small_text(buf, w, h, tx, ty, "X: Delete  D: Deselect", COL_TEXT);
        ty += 10;
        draw_small_text(buf, w, h, tx, ty, "Z: Undo  C: Clear", COL_TEXT);
        ty += 10;
        draw_small_text(buf, w, h, tx, ty, "Arrows: Orbit camera", COL_TEXT);
        ty += 10;
        draw_small_text(buf, w, h, tx, ty, "+/-: Zoom  R: Reset cam", COL_TEXT);
        ty += 14;
        draw_small_text(buf, w, h, tx, ty, "--- Presets ---", COL_TIP_BORDER);
        ty += 12;
        draw_small_text(buf, w, h, tx, ty, "1:Cube 2:Pyr 3:Dia", COL_TEXT);
        ty += 10;
        draw_small_text(buf, w, h, tx, ty, "4:Torus 5:Ico 6:Grid", COL_TEXT);
        ty += 14;
        draw_small_text(buf, w, h, tx, ty, "--- File ---", COL_TIP_BORDER);
        ty += 12;
        draw_small_text(buf, w, h, tx, ty, "W: Save  L: Load", COL_TEXT);
        ty += 10;
        draw_small_text(buf, w, h, tx, ty, "H: Toggle this panel", COL_TEXT_DIM);
        
        // Current tool tip
        let tip = self.tool.tip();
        if tip.len() > 0 {
            let tip_y = panel_y + panel_h + 5;
            if tip_y + 16 < h {
                let tip_w = tip.len() * 6 + 12;
                fill_rect(buf, w, h, panel_x, tip_y, tip_w, 14, COL_TIP_BG & 0xDDFFFFFF);
                draw_small_text(buf, w, h, panel_x + 6, tip_y + 3, tip, COL_TEXT_HI);
            }
        }
    }
}

// ─── Buffer Drawing Helpers ──────────────────────────────────────────────

fn put_buf_pixel(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
        buf[y as usize * w + x as usize] = color;
    }
}

fn fill_rect(buf: &mut [u32], bw: usize, bh: usize, x: usize, y: usize, w: usize, h: usize, color: u32) {
    for dy in 0..h {
        let py = y + dy;
        if py >= bh { break; }
        for dx in 0..w {
            let px = x + dx;
            if px >= bw { break; }
            buf[py * bw + px] = color;
        }
    }
}

fn draw_rect(buf: &mut [u32], bw: usize, bh: usize, x: usize, y: usize, w: usize, h: usize, color: u32) {
    for dx in 0..w {
        put_buf_pixel(buf, bw, bh, (x + dx) as i32, y as i32, color);
        put_buf_pixel(buf, bw, bh, (x + dx) as i32, (y + h - 1) as i32, color);
    }
    for dy in 0..h {
        put_buf_pixel(buf, bw, bh, x as i32, (y + dy) as i32, color);
        put_buf_pixel(buf, bw, bh, (x + w - 1) as i32, (y + dy) as i32, color);
    }
}

fn draw_line_buf(buf: &mut [u32], w: usize, h: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    // Bresenham's line algorithm
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    
    let max_steps = (dx.abs() + dy.abs()) as usize + 1;
    let max_steps = max_steps.min(4000); // safety limit
    
    for _ in 0..max_steps {
        put_buf_pixel(buf, w, h, x, y, color);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_dashed_line(buf: &mut [u32], w: usize, h: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut step = 0;
    let max_steps = (dx.abs() + dy.abs()) as usize + 1;
    let max_steps = max_steps.min(4000);
    
    for _ in 0..max_steps {
        if step % 8 < 4 {
            put_buf_pixel(buf, w, h, x, y, color);
        }
        step += 1;
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

fn fade_color(color: u32, factor: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32 * factor;
    let g = ((color >> 8) & 0xFF) as f32 * factor;
    let b = (color & 0xFF) as f32 * factor;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn blend_alpha(bg: u32, fg: u32) -> u32 {
    let alpha = ((fg >> 24) & 0xFF) as f32 / 255.0;
    let inv = 1.0 - alpha;
    let r = ((fg >> 16) & 0xFF) as f32 * alpha + ((bg >> 16) & 0xFF) as f32 * inv;
    let g = ((fg >> 8) & 0xFF) as f32 * alpha + ((bg >> 8) & 0xFF) as f32 * inv;
    let b = (fg & 0xFF) as f32 * alpha + (bg & 0xFF) as f32 * inv;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Tiny 5x7 font — renders text into a buffer. Very compact.
fn draw_small_text(buf: &mut [u32], bw: usize, bh: usize, x: usize, y: usize, text: &str, color: u32) {
    let mut cx = x;
    for ch in text.bytes() {
        draw_small_char(buf, bw, bh, cx, y, ch, color);
        cx += 6;
    }
}

/// Minimal 5x7 bitmap font (ASCII 32-127)
fn draw_small_char(buf: &mut [u32], bw: usize, bh: usize, x: usize, y: usize, ch: u8, color: u32) {
    // Simple procedural font for basic ASCII
    let c = ch as char;
    if c == ' ' { return; }
    
    // Use a simplified approach: map common characters
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
        _ => [0b01110, 0b01010, 0b01010, 0b01010, 0b01010, 0b01010, 0b01110], // box for unknown
    };
    
    for row in 0..7 {
        for col in 0..5 {
            if bits[row] & (1 << (4 - col)) != 0 {
                let px = x + col;
                let py = y + row;
                if px < bw && py < bh {
                    buf[py * bw + px] = color;
                }
            }
        }
    }
}
