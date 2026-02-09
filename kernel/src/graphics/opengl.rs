//! TrustGL - OpenGL-like API for TrustOS
//!
//! A simplified OpenGL 1.x style immediate mode API that wraps the software 3D renderer.
//! This provides a familiar interface for 3D graphics programming.
//!
//! # Example
//! ```
//! gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
//! gl_matrix_mode(GL_PROJECTION);
//! gl_load_identity();
//! glu_perspective(60.0, aspect, 0.1, 100.0);
//! gl_matrix_mode(GL_MODELVIEW);
//! gl_load_identity();
//! glu_look_at(0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
//! 
//! gl_begin(GL_TRIANGLES);
//!     gl_color3f(1.0, 0.0, 0.0);
//!     gl_vertex3f(-1.0, -1.0, 0.0);
//!     gl_color3f(0.0, 1.0, 0.0);
//!     gl_vertex3f(1.0, -1.0, 0.0);
//!     gl_color3f(0.0, 0.0, 1.0);
//!     gl_vertex3f(0.0, 1.0, 0.0);
//! gl_end();
//! gl_flush();
//! ```

use alloc::vec::Vec;
use spin::Mutex;
use micromath::F32Ext;

use super::math3d::{Vec3, Vec4, Mat4, deg_to_rad};
use super::render2d::Color2D;
use super::texture;
use crate::framebuffer;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS (OpenGL-style)
// ═══════════════════════════════════════════════════════════════════════════════

// Primitive types
pub const GL_POINTS: u32 = 0x0000;
pub const GL_LINES: u32 = 0x0001;
pub const GL_LINE_STRIP: u32 = 0x0003;
pub const GL_LINE_LOOP: u32 = 0x0002;
pub const GL_TRIANGLES: u32 = 0x0004;
pub const GL_TRIANGLE_STRIP: u32 = 0x0005;
pub const GL_TRIANGLE_FAN: u32 = 0x0006;
pub const GL_QUADS: u32 = 0x0007;

// Clear bits
pub const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
pub const GL_DEPTH_BUFFER_BIT: u32 = 0x0100;

// Matrix modes
pub const GL_MODELVIEW: u32 = 0x1700;
pub const GL_PROJECTION: u32 = 0x1701;

// Enable/Disable caps
pub const GL_DEPTH_TEST: u32 = 0x0B71;
pub const GL_LIGHTING: u32 = 0x0B50;
pub const GL_LIGHT0: u32 = 0x4000;
pub const GL_LIGHT1: u32 = 0x4001;
pub const GL_CULL_FACE: u32 = 0x0B44;
pub const GL_BLEND: u32 = 0x0BE2;

// Light parameters
pub const GL_AMBIENT: u32 = 0x1200;
pub const GL_DIFFUSE: u32 = 0x1201;
pub const GL_POSITION: u32 = 0x1203;

// Shade model
pub const GL_FLAT: u32 = 0x1D00;
pub const GL_SMOOTH: u32 = 0x1D01;

// Polygon modes
pub const GL_FRONT: u32 = 0x0404;
pub const GL_BACK: u32 = 0x0405;
pub const GL_FRONT_AND_BACK: u32 = 0x0408;
pub const GL_LINE: u32 = 0x1B01;
pub const GL_FILL: u32 = 0x1B02;

// Error codes
pub const GL_NO_ERROR: u32 = 0;
pub const GL_INVALID_ENUM: u32 = 0x0500;
pub const GL_INVALID_VALUE: u32 = 0x0501;
pub const GL_INVALID_OPERATION: u32 = 0x0502;

// ═══════════════════════════════════════════════════════════════════════════════
// INTERNAL STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// A vertex in the immediate mode pipeline
#[derive(Clone, Copy, Default)]
struct ImmediateVertex {
    position: Vec3,
    color: Color2D,
    normal: Vec3,
    texcoord: (f32, f32),
}

/// OpenGL-like state machine
struct GlState {
    // Viewport
    viewport_x: i32,
    viewport_y: i32,
    viewport_width: u32,
    viewport_height: u32,

    // Matrix stacks
    modelview_stack: Vec<Mat4>,
    projection_stack: Vec<Mat4>,
    current_matrix_mode: u32,

    // Current state
    current_color: Color2D,
    current_normal: Vec3,
    current_texcoord: (f32, f32),
    clear_color: Color2D,

    // Immediate mode
    primitive_type: u32,
    vertices: Vec<ImmediateVertex>,
    in_begin_end: bool,

    // Capabilities
    depth_test_enabled: bool,
    lighting_enabled: bool,
    cull_face_enabled: bool,
    blend_enabled: bool,
    lights_enabled: [bool; 8],

    // Lights
    light_positions: [Vec4; 8],
    light_ambient: [Color2D; 8],
    light_diffuse: [Color2D; 8],

    // Depth buffer
    depth_buffer: Vec<f32>,

    // Shade model
    shade_model: u32,

    // Polygon mode
    polygon_mode: u32,

    // Error
    last_error: u32,

    // Initialized
    initialized: bool,
}

impl GlState {
    const fn new() -> Self {
        Self {
            viewport_x: 0,
            viewport_y: 0,
            viewport_width: 800,
            viewport_height: 600,
            modelview_stack: Vec::new(),
            projection_stack: Vec::new(),
            current_matrix_mode: GL_MODELVIEW,
            current_color: Color2D::WHITE,
            current_normal: Vec3::Z,
            current_texcoord: (0.0, 0.0),
            clear_color: Color2D::BLACK,
            primitive_type: GL_TRIANGLES,
            vertices: Vec::new(),
            in_begin_end: false,
            depth_test_enabled: false,
            lighting_enabled: false,
            cull_face_enabled: false,
            blend_enabled: false,
            lights_enabled: [false; 8],
            light_positions: [Vec4::new(0.0, 0.0, 1.0, 0.0); 8],
            light_ambient: [Color2D::BLACK; 8],
            light_diffuse: [Color2D::WHITE; 8],
            depth_buffer: Vec::new(),
            shade_model: GL_SMOOTH,
            polygon_mode: GL_FILL,
            last_error: GL_NO_ERROR,
            initialized: false,
        }
    }

    fn current_matrix(&mut self) -> &mut Mat4 {
        let stack = match self.current_matrix_mode {
            GL_PROJECTION => &mut self.projection_stack,
            _ => &mut self.modelview_stack,
        };
        if stack.is_empty() {
            stack.push(Mat4::IDENTITY);
        }
        stack.last_mut().unwrap()
    }

    fn get_mvp(&self) -> Mat4 {
        let mv = self.modelview_stack.last().cloned().unwrap_or(Mat4::IDENTITY);
        let proj = self.projection_stack.last().cloned().unwrap_or(Mat4::IDENTITY);
        proj.mul(&mv)
    }
}

static GL_STATE: Mutex<GlState> = Mutex::new(GlState::new());

// ═══════════════════════════════════════════════════════════════════════════════
// INITIALIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize TrustGL with screen dimensions
pub fn gl_init(width: u32, height: u32) {
    let mut state = GL_STATE.lock();
    state.viewport_width = width;
    state.viewport_height = height;
    state.depth_buffer = alloc::vec![1.0f32; (width * height) as usize];
    state.modelview_stack.push(Mat4::IDENTITY);
    state.projection_stack.push(Mat4::IDENTITY);
    state.initialized = true;
}

// ═══════════════════════════════════════════════════════════════════════════════
// VIEWPORT & MATRICES
// ═══════════════════════════════════════════════════════════════════════════════

/// Set the viewport
pub fn gl_viewport(x: i32, y: i32, width: u32, height: u32) {
    let mut state = GL_STATE.lock();
    state.viewport_x = x;
    state.viewport_y = y;
    state.viewport_width = width;
    state.viewport_height = height;
    // Resize depth buffer
    state.depth_buffer = alloc::vec![1.0f32; (width * height) as usize];
}

/// Set the current matrix mode
pub fn gl_matrix_mode(mode: u32) {
    let mut state = GL_STATE.lock();
    state.current_matrix_mode = mode;
}

/// Load identity matrix
pub fn gl_load_identity() {
    let mut state = GL_STATE.lock();
    *state.current_matrix() = Mat4::IDENTITY;
}

/// Push current matrix onto stack
pub fn gl_push_matrix() {
    let mut state = GL_STATE.lock();
    let current = *state.current_matrix();
    match state.current_matrix_mode {
        GL_PROJECTION => state.projection_stack.push(current),
        _ => state.modelview_stack.push(current),
    }
}

/// Pop matrix from stack
pub fn gl_pop_matrix() {
    let mut state = GL_STATE.lock();
    match state.current_matrix_mode {
        GL_PROJECTION => { 
            if state.projection_stack.len() > 1 {
                state.projection_stack.pop();
            }
        },
        _ => {
            if state.modelview_stack.len() > 1 {
                state.modelview_stack.pop();
            }
        },
    }
}

/// Multiply current matrix by translation matrix
pub fn gl_translatef(x: f32, y: f32, z: f32) {
    let mut state = GL_STATE.lock();
    let translation = Mat4::translation(x, y, z);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&translation);
}

/// Multiply current matrix by rotation matrix (angle in degrees)
pub fn gl_rotatef(angle: f32, x: f32, y: f32, z: f32) {
    let mut state = GL_STATE.lock();
    let axis = Vec3::new(x, y, z).normalize();
    let rad = deg_to_rad(angle);
    let rotation = Mat4::rotation(axis, rad);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&rotation);
}

/// Multiply current matrix by scale matrix
pub fn gl_scalef(x: f32, y: f32, z: f32) {
    let mut state = GL_STATE.lock();
    let scale = Mat4::scale(x, y, z);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&scale);
}

/// Multiply current matrix by arbitrary matrix
pub fn gl_mult_matrixf(m: &[f32; 16]) {
    let mut state = GL_STATE.lock();
    let mat = Mat4::from_array(*m);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&mat);
}

/// Set orthographic projection
pub fn gl_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
    let mut state = GL_STATE.lock();
    let ortho = Mat4::orthographic(left, right, bottom, top, near, far);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&ortho);
}

/// Set frustum (perspective) projection
pub fn gl_frustum(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
    let mut state = GL_STATE.lock();
    let frustum = Mat4::frustum(left, right, bottom, top, near, far);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&frustum);
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLU-STYLE HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Set perspective projection (GLU style)
pub fn glu_perspective(fovy: f32, aspect: f32, near: f32, far: f32) {
    let mut state = GL_STATE.lock();
    let perspective = Mat4::perspective(deg_to_rad(fovy), aspect, near, far);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&perspective);
}

/// Set camera look-at (GLU style)
pub fn glu_look_at(
    eye_x: f32, eye_y: f32, eye_z: f32,
    center_x: f32, center_y: f32, center_z: f32,
    up_x: f32, up_y: f32, up_z: f32
) {
    let mut state = GL_STATE.lock();
    let look_at = Mat4::look_at(
        Vec3::new(eye_x, eye_y, eye_z),
        Vec3::new(center_x, center_y, center_z),
        Vec3::new(up_x, up_y, up_z),
    );
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&look_at);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CLEAR & STATE
// ═══════════════════════════════════════════════════════════════════════════════

/// Set clear color
pub fn gl_clear_color(r: f32, g: f32, b: f32, _a: f32) {
    let mut state = GL_STATE.lock();
    state.clear_color = Color2D::rgb(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    );
}

/// Clear buffers
pub fn gl_clear(mask: u32) {
    let mut state = GL_STATE.lock();
    
    if (mask & GL_COLOR_BUFFER_BIT) != 0 {
        let color = state.clear_color.to_u32();
        let w = state.viewport_width;
        let h = state.viewport_height;
        framebuffer::fill_rect(
            state.viewport_x as u32, 
            state.viewport_y as u32, 
            w, h, color
        );
    }
    
    if (mask & GL_DEPTH_BUFFER_BIT) != 0 {
        state.depth_buffer.fill(1.0);
    }
}

/// Enable a capability
pub fn gl_enable(cap: u32) {
    let mut state = GL_STATE.lock();
    match cap {
        GL_DEPTH_TEST => state.depth_test_enabled = true,
        GL_LIGHTING => state.lighting_enabled = true,
        GL_CULL_FACE => state.cull_face_enabled = true,
        GL_BLEND => state.blend_enabled = true,
        GL_LIGHT0..=0x4007 => {
            let idx = (cap - GL_LIGHT0) as usize;
            if idx < 8 {
                state.lights_enabled[idx] = true;
            }
        }
        _ => state.last_error = GL_INVALID_ENUM,
    }
}

/// Disable a capability
pub fn gl_disable(cap: u32) {
    let mut state = GL_STATE.lock();
    match cap {
        GL_DEPTH_TEST => state.depth_test_enabled = false,
        GL_LIGHTING => state.lighting_enabled = false,
        GL_CULL_FACE => state.cull_face_enabled = false,
        GL_BLEND => state.blend_enabled = false,
        GL_LIGHT0..=0x4007 => {
            let idx = (cap - GL_LIGHT0) as usize;
            if idx < 8 {
                state.lights_enabled[idx] = false;
            }
        }
        _ => state.last_error = GL_INVALID_ENUM,
    }
}

/// Set shade model
pub fn gl_shade_model(mode: u32) {
    let mut state = GL_STATE.lock();
    state.shade_model = mode;
}

/// Set polygon mode
pub fn gl_polygon_mode(face: u32, mode: u32) {
    let mut state = GL_STATE.lock();
    if face == GL_FRONT_AND_BACK || face == GL_FRONT {
        state.polygon_mode = mode;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IMMEDIATE MODE RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Begin primitive
pub fn gl_begin(mode: u32) {
    let mut state = GL_STATE.lock();
    if state.in_begin_end {
        state.last_error = GL_INVALID_OPERATION;
        return;
    }
    state.primitive_type = mode;
    state.vertices.clear();
    state.in_begin_end = true;
}

/// End primitive and render
pub fn gl_end() {
    let mut state = GL_STATE.lock();
    if !state.in_begin_end {
        state.last_error = GL_INVALID_OPERATION;
        return;
    }
    state.in_begin_end = false;

    // Get MVP matrix and render
    let mvp = state.get_mvp();
    let vp_x = state.viewport_x;
    let vp_y = state.viewport_y;
    let vp_w = state.viewport_width as f32;
    let vp_h = state.viewport_height as f32;
    let depth_test = state.depth_test_enabled;
    let lighting = state.lighting_enabled;
    let polygon_mode = state.polygon_mode;
    let shade_model = state.shade_model;
    
    // Clone vertices to process
    let vertices = state.vertices.clone();
    let prim_type = state.primitive_type;
    
    // We need to drop state before calling framebuffer functions
    // to avoid deadlock
    let vp_width = state.viewport_width;
    let depth_buffer = &mut state.depth_buffer as *mut Vec<f32>;
    
    // Check if texturing is enabled
    let texturing_enabled = texture::is_texture_2d_enabled();
    
    // Project vertices - now includes UV coords (x, y, z, color, u, v)
    let mut projected: Vec<Option<(i32, i32, f32, Color2D, f32, f32)>> = Vec::with_capacity(vertices.len());
    for v in &vertices {
        let clip = mvp.transform_vec4(Vec4::from_vec3(v.position, 1.0));
        if clip.w > 0.0 {
            let ndc_x = clip.x / clip.w;
            let ndc_y = clip.y / clip.w;
            let ndc_z = clip.z / clip.w;
            
            let screen_x = vp_x + ((ndc_x + 1.0) * 0.5 * vp_w) as i32;
            let screen_y = vp_y + ((1.0 - ndc_y) * 0.5 * vp_h) as i32;
            
            let mut color = v.color;
            
            // Simple lighting calculation
            if lighting {
                let light_dir = Vec3::new(0.5, -1.0, -0.5).normalize();
                let ndotl = v.normal.dot(-light_dir).max(0.0);
                let ambient = 0.2;
                let intensity = ambient + (1.0 - ambient) * ndotl;
                color = Color2D::rgb(
                    (color.r as f32 * intensity) as u8,
                    (color.g as f32 * intensity) as u8,
                    (color.b as f32 * intensity) as u8,
                );
            }
            
            projected.push(Some((screen_x, screen_y, ndc_z, color, v.texcoord.0, v.texcoord.1)));
        } else {
            projected.push(None);
        }
    }

    // Render based on primitive type
    match prim_type {
        GL_POINTS => {
            for p in &projected {
                if let Some((x, y, z, color, u, v)) = p {
                    // Sample texture if enabled
                    let final_color = if texturing_enabled {
                        if let Some(tex_color) = texture::sample_bound_texture(*u, *v) {
                            tex_color
                        } else {
                            color.to_u32()
                        }
                    } else {
                        color.to_u32()
                    };
                    
                    if *x >= 0 && *y >= 0 && (*x as u32) < vp_width && (*y as u32) < (vp_h as u32) {
                        let idx = (*y as usize) * (vp_width as usize) + (*x as usize);
                        unsafe {
                            let db = &mut *depth_buffer;
                            if !depth_test || *z < db[idx] {
                                db[idx] = *z;
                                framebuffer::put_pixel(*x as u32, *y as u32, final_color);
                            }
                        }
                    }
                }
            }
        }
        
        GL_LINES => {
            for i in (0..projected.len()).step_by(2) {
                if i + 1 < projected.len() {
                    if let (Some((x0, y0, _, c0, _, _)), Some((x1, y1, _, _, _, _))) = 
                        (&projected[i], &projected[i + 1]) 
                    {
                        draw_line(*x0, *y0, *x1, *y1, c0.to_u32());
                    }
                }
            }
        }
        
        GL_LINE_STRIP => {
            for i in 0..projected.len().saturating_sub(1) {
                if let (Some((x0, y0, _, c0, _, _)), Some((x1, y1, _, _, _, _))) = 
                    (&projected[i], &projected[i + 1]) 
                {
                    draw_line(*x0, *y0, *x1, *y1, c0.to_u32());
                }
            }
        }
        
        GL_LINE_LOOP => {
            for i in 0..projected.len() {
                let next = (i + 1) % projected.len();
                if let (Some((x0, y0, _, c0, _, _)), Some((x1, y1, _, _, _, _))) = 
                    (&projected[i], &projected[next]) 
                {
                    draw_line(*x0, *y0, *x1, *y1, c0.to_u32());
                }
            }
        }
        
        GL_TRIANGLES => {
            for i in (0..projected.len()).step_by(3) {
                if i + 2 < projected.len() {
                    if let (Some(p0), Some(p1), Some(p2)) = 
                        (&projected[i], &projected[i + 1], &projected[i + 2]) 
                    {
                        if polygon_mode == GL_LINE {
                            // Wireframe
                            draw_line(p0.0, p0.1, p1.0, p1.1, p0.3.to_u32());
                            draw_line(p1.0, p1.1, p2.0, p2.1, p1.3.to_u32());
                            draw_line(p2.0, p2.1, p0.0, p0.1, p2.3.to_u32());
                        } else {
                            // Filled triangle (with optional texturing)
                            unsafe {
                                let db = &mut *depth_buffer;
                                draw_filled_triangle_textured(
                                    p0.0, p0.1, p0.2, p0.3, p0.4, p0.5,
                                    p1.0, p1.1, p1.2, p1.3, p1.4, p1.5,
                                    p2.0, p2.1, p2.2, p2.3, p2.4, p2.5,
                                    vp_width,
                                    depth_test,
                                    shade_model == GL_SMOOTH,
                                    texturing_enabled,
                                    db,
                                );
                            }
                        }
                    }
                }
            }
        }
        
        GL_QUADS => {
            // Convert quads to triangles
            for i in (0..projected.len()).step_by(4) {
                if i + 3 < projected.len() {
                    if let (Some(p0), Some(p1), Some(p2), Some(p3)) = 
                        (&projected[i], &projected[i + 1], &projected[i + 2], &projected[i + 3]) 
                    {
                        if polygon_mode == GL_LINE {
                            draw_line(p0.0, p0.1, p1.0, p1.1, p0.3.to_u32());
                            draw_line(p1.0, p1.1, p2.0, p2.1, p1.3.to_u32());
                            draw_line(p2.0, p2.1, p3.0, p3.1, p2.3.to_u32());
                            draw_line(p3.0, p3.1, p0.0, p0.1, p3.3.to_u32());
                        } else {
                            unsafe {
                                let db = &mut *depth_buffer;
                                // First triangle
                                draw_filled_triangle_textured(
                                    p0.0, p0.1, p0.2, p0.3, p0.4, p0.5,
                                    p1.0, p1.1, p1.2, p1.3, p1.4, p1.5,
                                    p2.0, p2.1, p2.2, p2.3, p2.4, p2.5,
                                    vp_width,
                                    depth_test,
                                    shade_model == GL_SMOOTH,
                                    texturing_enabled,
                                    db,
                                );
                                // Second triangle
                                draw_filled_triangle_textured(
                                    p0.0, p0.1, p0.2, p0.3, p0.4, p0.5,
                                    p2.0, p2.1, p2.2, p2.3, p2.4, p2.5,
                                    p3.0, p3.1, p3.2, p3.3, p3.4, p3.5,
                                    vp_width,
                                    depth_test,
                                    shade_model == GL_SMOOTH,
                                    texturing_enabled,
                                    db,
                                );
                            }
                        }
                    }
                }
            }
        }
        
        _ => {}
    }
}

/// Set current color (float)
pub fn gl_color3f(r: f32, g: f32, b: f32) {
    let mut state = GL_STATE.lock();
    state.current_color = Color2D::rgb(
        (r.clamp(0.0, 1.0) * 255.0) as u8,
        (g.clamp(0.0, 1.0) * 255.0) as u8,
        (b.clamp(0.0, 1.0) * 255.0) as u8,
    );
}

/// Set current color (u8)
pub fn gl_color3ub(r: u8, g: u8, b: u8) {
    let mut state = GL_STATE.lock();
    state.current_color = Color2D::rgb(r, g, b);
}

/// Set current color with alpha
pub fn gl_color4f(r: f32, g: f32, b: f32, a: f32) {
    let mut state = GL_STATE.lock();
    state.current_color = Color2D::rgba(
        (r.clamp(0.0, 1.0) * 255.0) as u8,
        (g.clamp(0.0, 1.0) * 255.0) as u8,
        (b.clamp(0.0, 1.0) * 255.0) as u8,
        (a.clamp(0.0, 1.0) * 255.0) as u8,
    );
}

/// Set current normal
pub fn gl_normal3f(x: f32, y: f32, z: f32) {
    let mut state = GL_STATE.lock();
    state.current_normal = Vec3::new(x, y, z).normalize();
}

/// Set current texture coordinate
pub fn gl_tex_coord2f(s: f32, t: f32) {
    let mut state = GL_STATE.lock();
    state.current_texcoord = (s, t);
}

/// Add a vertex
pub fn gl_vertex3f(x: f32, y: f32, z: f32) {
    let mut state = GL_STATE.lock();
    if !state.in_begin_end {
        state.last_error = GL_INVALID_OPERATION;
        return;
    }
    
    // Copy state values first to avoid borrow conflicts
    let color = state.current_color;
    let normal = state.current_normal;
    let texcoord = state.current_texcoord;
    
    state.vertices.push(ImmediateVertex {
        position: Vec3::new(x, y, z),
        color,
        normal,
        texcoord,
    });
}

/// Add a vertex (2D, z=0)
pub fn gl_vertex2f(x: f32, y: f32) {
    gl_vertex3f(x, y, 0.0);
}

/// Flush rendering (no-op for immediate mode, kept for API compatibility)
pub fn gl_flush() {
    // Nothing to do in immediate mode
}

/// Swap buffers (for double buffering)
pub fn gl_swap_buffers() {
    framebuffer::swap_buffers();
}

/// Get error
pub fn gl_get_error() -> u32 {
    let mut state = GL_STATE.lock();
    let err = state.last_error;
    state.last_error = GL_NO_ERROR;
    err
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && y >= 0 {
            framebuffer::put_pixel(x as u32, y as u32, color);
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

fn draw_filled_triangle(
    x0: i32, y0: i32, z0: f32, c0: Color2D,
    x1: i32, y1: i32, z1: f32, c1: Color2D,
    x2: i32, y2: i32, z2: f32, c2: Color2D,
    width: u32,
    depth_test: bool,
    smooth_shading: bool,
    depth_buffer: &mut Vec<f32>,
) {
    // Sort vertices by Y
    let mut verts = [(x0, y0, z0, c0), (x1, y1, z1, c1), (x2, y2, z2, c2)];
    verts.sort_by_key(|v| v.1);
    
    let (x0, y0, z0, c0) = verts[0];
    let (x1, y1, z1, c1) = verts[1];
    let (x2, y2, z2, c2) = verts[2];
    
    if y0 == y2 { return; } // Degenerate triangle
    
    // Flat color for flat shading
    let flat_color = c0;
    
    // Scanline rasterization
    for y in y0.max(0)..=y2 {
        if y < 0 { continue; }
        
        let (xa, za, ca, xb, zb, cb) = if y < y1 {
            // Top part
            if y1 == y0 {
                let t = if y2 != y0 { (y - y0) as f32 / (y2 - y0) as f32 } else { 0.0 };
                let xa = x0 + ((x2 - x0) as f32 * t) as i32;
                let za = z0 + (z2 - z0) * t;
                (xa, za, lerp_color(c0, c2, t), xa, za, lerp_color(c0, c2, t))
            } else {
                let t1 = (y - y0) as f32 / (y1 - y0) as f32;
                let t2 = (y - y0) as f32 / (y2 - y0) as f32;
                (
                    x0 + ((x1 - x0) as f32 * t1) as i32,
                    z0 + (z1 - z0) * t1,
                    lerp_color(c0, c1, t1),
                    x0 + ((x2 - x0) as f32 * t2) as i32,
                    z0 + (z2 - z0) * t2,
                    lerp_color(c0, c2, t2),
                )
            }
        } else {
            // Bottom part
            if y2 == y1 {
                (x1, z1, c1, x2, z2, c2)
            } else {
                let t1 = (y - y1) as f32 / (y2 - y1) as f32;
                let t2 = (y - y0) as f32 / (y2 - y0) as f32;
                (
                    x1 + ((x2 - x1) as f32 * t1) as i32,
                    z1 + (z2 - z1) * t1,
                    lerp_color(c1, c2, t1),
                    x0 + ((x2 - x0) as f32 * t2) as i32,
                    z0 + (z2 - z0) * t2,
                    lerp_color(c0, c2, t2),
                )
            }
        };
        
        let (start_x, end_x, start_z, end_z, start_c, end_c) = if xa < xb {
            (xa, xb, za, zb, ca, cb)
        } else {
            (xb, xa, zb, za, cb, ca)
        };
        
        for x in start_x.max(0)..=end_x {
            if x < 0 || x as u32 >= width { continue; }
            
            let t = if end_x != start_x {
                (x - start_x) as f32 / (end_x - start_x) as f32
            } else {
                0.0
            };
            
            let z = start_z + (end_z - start_z) * t;
            let color = if smooth_shading {
                lerp_color(start_c, end_c, t)
            } else {
                flat_color
            };
            
            let idx = (y as usize) * (width as usize) + (x as usize);
            if idx < depth_buffer.len() {
                if !depth_test || z < depth_buffer[idx] {
                    depth_buffer[idx] = z;
                    framebuffer::put_pixel(x as u32, y as u32, color.to_u32());
                }
            }
        }
    }
}

fn lerp_color(c0: Color2D, c1: Color2D, t: f32) -> Color2D {
    Color2D::rgb(
        (c0.r as f32 + (c1.r as f32 - c0.r as f32) * t) as u8,
        (c0.g as f32 + (c1.g as f32 - c0.g as f32) * t) as u8,
        (c0.b as f32 + (c1.b as f32 - c0.b as f32) * t) as u8,
    )
}

/// Draw a filled triangle with optional texture mapping
fn draw_filled_triangle_textured(
    x0: i32, y0: i32, z0: f32, c0: Color2D, u0: f32, v0: f32,
    x1: i32, y1: i32, z1: f32, c1: Color2D, u1: f32, v1: f32,
    x2: i32, y2: i32, z2: f32, c2: Color2D, u2: f32, v2: f32,
    width: u32,
    depth_test: bool,
    smooth_shading: bool,
    texturing: bool,
    depth_buffer: &mut Vec<f32>,
) {
    // Sort vertices by Y (with UV coords)
    let mut verts = [
        (x0, y0, z0, c0, u0, v0), 
        (x1, y1, z1, c1, u1, v1), 
        (x2, y2, z2, c2, u2, v2)
    ];
    verts.sort_by_key(|v| v.1);
    
    let (x0, y0, z0, c0, u0, v0) = verts[0];
    let (x1, y1, z1, c1, u1, v1) = verts[1];
    let (x2, y2, z2, c2, u2, v2) = verts[2];
    
    if y0 == y2 { return; }
    
    let flat_color = c0;
    
    for y in y0.max(0)..=y2 {
        if y < 0 { continue; }
        
        // Calculate scanline endpoints with UV interpolation
        let (xa, za, ca, ua, va, xb, zb, cb, ub, vb) = if y < y1 {
            if y1 == y0 {
                let t = if y2 != y0 { (y - y0) as f32 / (y2 - y0) as f32 } else { 0.0 };
                let xa = x0 + ((x2 - x0) as f32 * t) as i32;
                let za = z0 + (z2 - z0) * t;
                let ua = u0 + (u2 - u0) * t;
                let va = v0 + (v2 - v0) * t;
                (xa, za, lerp_color(c0, c2, t), ua, va, xa, za, lerp_color(c0, c2, t), ua, va)
            } else {
                let t1 = (y - y0) as f32 / (y1 - y0) as f32;
                let t2 = (y - y0) as f32 / (y2 - y0) as f32;
                (
                    x0 + ((x1 - x0) as f32 * t1) as i32,
                    z0 + (z1 - z0) * t1,
                    lerp_color(c0, c1, t1),
                    u0 + (u1 - u0) * t1,
                    v0 + (v1 - v0) * t1,
                    x0 + ((x2 - x0) as f32 * t2) as i32,
                    z0 + (z2 - z0) * t2,
                    lerp_color(c0, c2, t2),
                    u0 + (u2 - u0) * t2,
                    v0 + (v2 - v0) * t2,
                )
            }
        } else {
            if y2 == y1 {
                (x1, z1, c1, u1, v1, x2, z2, c2, u2, v2)
            } else {
                let t1 = (y - y1) as f32 / (y2 - y1) as f32;
                let t2 = (y - y0) as f32 / (y2 - y0) as f32;
                (
                    x1 + ((x2 - x1) as f32 * t1) as i32,
                    z1 + (z2 - z1) * t1,
                    lerp_color(c1, c2, t1),
                    u1 + (u2 - u1) * t1,
                    v1 + (v2 - v1) * t1,
                    x0 + ((x2 - x0) as f32 * t2) as i32,
                    z0 + (z2 - z0) * t2,
                    lerp_color(c0, c2, t2),
                    u0 + (u2 - u0) * t2,
                    v0 + (v2 - v0) * t2,
                )
            }
        };
        
        let (start_x, end_x, start_z, end_z, start_c, end_c, start_u, end_u, start_v, end_v) = 
            if xa < xb {
                (xa, xb, za, zb, ca, cb, ua, ub, va, vb)
            } else {
                (xb, xa, zb, za, cb, ca, ub, ua, vb, va)
            };
        
        for x in start_x.max(0)..=end_x {
            if x < 0 || x as u32 >= width { continue; }
            
            let t = if end_x != start_x {
                (x - start_x) as f32 / (end_x - start_x) as f32
            } else {
                0.0
            };
            
            let z = start_z + (end_z - start_z) * t;
            let u = start_u + (end_u - start_u) * t;
            let v = start_v + (end_v - start_v) * t;
            
            // Get color from texture or interpolation
            let final_color = if texturing {
                if let Some(tex_color) = texture::sample_bound_texture(u, v) {
                    // Modulate texture with vertex color if smooth shading
                    if smooth_shading {
                        let vc = lerp_color(start_c, end_c, t);
                        let tr = ((tex_color >> 16) & 0xFF) as u32;
                        let tg = ((tex_color >> 8) & 0xFF) as u32;
                        let tb = (tex_color & 0xFF) as u32;
                        let r = (tr * vc.r as u32 / 255).min(255);
                        let g = (tg * vc.g as u32 / 255).min(255);
                        let b = (tb * vc.b as u32 / 255).min(255);
                        (0xFF << 24) | (r << 16) | (g << 8) | b
                    } else {
                        tex_color
                    }
                } else {
                    let color = if smooth_shading {
                        lerp_color(start_c, end_c, t)
                    } else {
                        flat_color
                    };
                    color.to_u32()
                }
            } else {
                let color = if smooth_shading {
                    lerp_color(start_c, end_c, t)
                } else {
                    flat_color
                };
                color.to_u32()
            };
            
            let idx = (y as usize) * (width as usize) + (x as usize);
            if idx < depth_buffer.len() {
                if !depth_test || z < depth_buffer[idx] {
                    depth_buffer[idx] = z;
                    framebuffer::put_pixel(x as u32, y as u32, final_color);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENT SHAPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a cube (helper function)
pub fn glut_solid_cube(size: f32) {
    let s = size / 2.0;
    
    gl_begin(GL_QUADS);
    
    // Front face
    gl_normal3f(0.0, 0.0, 1.0);
    gl_vertex3f(-s, -s, s);
    gl_vertex3f(s, -s, s);
    gl_vertex3f(s, s, s);
    gl_vertex3f(-s, s, s);
    
    // Back face
    gl_normal3f(0.0, 0.0, -1.0);
    gl_vertex3f(s, -s, -s);
    gl_vertex3f(-s, -s, -s);
    gl_vertex3f(-s, s, -s);
    gl_vertex3f(s, s, -s);
    
    // Top face
    gl_normal3f(0.0, 1.0, 0.0);
    gl_vertex3f(-s, s, s);
    gl_vertex3f(s, s, s);
    gl_vertex3f(s, s, -s);
    gl_vertex3f(-s, s, -s);
    
    // Bottom face
    gl_normal3f(0.0, -1.0, 0.0);
    gl_vertex3f(-s, -s, -s);
    gl_vertex3f(s, -s, -s);
    gl_vertex3f(s, -s, s);
    gl_vertex3f(-s, -s, s);
    
    // Right face
    gl_normal3f(1.0, 0.0, 0.0);
    gl_vertex3f(s, -s, s);
    gl_vertex3f(s, -s, -s);
    gl_vertex3f(s, s, -s);
    gl_vertex3f(s, s, s);
    
    // Left face
    gl_normal3f(-1.0, 0.0, 0.0);
    gl_vertex3f(-s, -s, -s);
    gl_vertex3f(-s, -s, s);
    gl_vertex3f(-s, s, s);
    gl_vertex3f(-s, s, -s);
    
    gl_end();
}

/// Draw wireframe cube
pub fn glut_wire_cube(size: f32) {
    let s = size / 2.0;
    
    gl_begin(GL_LINE_LOOP);
    // Front face
    gl_vertex3f(-s, -s, s);
    gl_vertex3f(s, -s, s);
    gl_vertex3f(s, s, s);
    gl_vertex3f(-s, s, s);
    gl_end();
    
    gl_begin(GL_LINE_LOOP);
    // Back face
    gl_vertex3f(-s, -s, -s);
    gl_vertex3f(s, -s, -s);
    gl_vertex3f(s, s, -s);
    gl_vertex3f(-s, s, -s);
    gl_end();
    
    gl_begin(GL_LINES);
    // Connecting edges
    gl_vertex3f(-s, -s, s); gl_vertex3f(-s, -s, -s);
    gl_vertex3f(s, -s, s); gl_vertex3f(s, -s, -s);
    gl_vertex3f(s, s, s); gl_vertex3f(s, s, -s);
    gl_vertex3f(-s, s, s); gl_vertex3f(-s, s, -s);
    gl_end();
}

/// Draw a teapot (simplified - actually draws a sphere-ish shape)
pub fn glut_solid_teapot(size: f32) {
    // Simplified - just draw an icosphere
    let subdivisions = 2;
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
    
    // Icosahedron vertices
    let vertices = [
        Vec3::new(-1.0, t, 0.0).normalize().scale(size),
        Vec3::new(1.0, t, 0.0).normalize().scale(size),
        Vec3::new(-1.0, -t, 0.0).normalize().scale(size),
        Vec3::new(1.0, -t, 0.0).normalize().scale(size),
        Vec3::new(0.0, -1.0, t).normalize().scale(size),
        Vec3::new(0.0, 1.0, t).normalize().scale(size),
        Vec3::new(0.0, -1.0, -t).normalize().scale(size),
        Vec3::new(0.0, 1.0, -t).normalize().scale(size),
        Vec3::new(t, 0.0, -1.0).normalize().scale(size),
        Vec3::new(t, 0.0, 1.0).normalize().scale(size),
        Vec3::new(-t, 0.0, -1.0).normalize().scale(size),
        Vec3::new(-t, 0.0, 1.0).normalize().scale(size),
    ];
    
    let faces = [
        (0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
        (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
        (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
        (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1),
    ];
    
    gl_begin(GL_TRIANGLES);
    for (a, b, c) in &faces {
        let va = vertices[*a];
        let vb = vertices[*b];
        let vc = vertices[*c];
        let normal = (vb - va).cross(vc - va).normalize();
        
        gl_normal3f(normal.x, normal.y, normal.z);
        gl_vertex3f(va.x, va.y, va.z);
        gl_vertex3f(vb.x, vb.y, vb.z);
        gl_vertex3f(vc.x, vc.y, vc.z);
    }
    gl_end();
}

/// Draw a textured cube with UV mapping
pub fn glut_textured_cube(size: f32) {
    let s = size / 2.0;
    
    gl_begin(GL_QUADS);
    
    // Front face
    gl_normal3f(0.0, 0.0, 1.0);
    gl_tex_coord2f(0.0, 0.0); gl_vertex3f(-s, -s, s);
    gl_tex_coord2f(1.0, 0.0); gl_vertex3f(s, -s, s);
    gl_tex_coord2f(1.0, 1.0); gl_vertex3f(s, s, s);
    gl_tex_coord2f(0.0, 1.0); gl_vertex3f(-s, s, s);
    
    // Back face
    gl_normal3f(0.0, 0.0, -1.0);
    gl_tex_coord2f(0.0, 0.0); gl_vertex3f(s, -s, -s);
    gl_tex_coord2f(1.0, 0.0); gl_vertex3f(-s, -s, -s);
    gl_tex_coord2f(1.0, 1.0); gl_vertex3f(-s, s, -s);
    gl_tex_coord2f(0.0, 1.0); gl_vertex3f(s, s, -s);
    
    // Top face
    gl_normal3f(0.0, 1.0, 0.0);
    gl_tex_coord2f(0.0, 0.0); gl_vertex3f(-s, s, s);
    gl_tex_coord2f(1.0, 0.0); gl_vertex3f(s, s, s);
    gl_tex_coord2f(1.0, 1.0); gl_vertex3f(s, s, -s);
    gl_tex_coord2f(0.0, 1.0); gl_vertex3f(-s, s, -s);
    
    // Bottom face
    gl_normal3f(0.0, -1.0, 0.0);
    gl_tex_coord2f(0.0, 0.0); gl_vertex3f(-s, -s, -s);
    gl_tex_coord2f(1.0, 0.0); gl_vertex3f(s, -s, -s);
    gl_tex_coord2f(1.0, 1.0); gl_vertex3f(s, -s, s);
    gl_tex_coord2f(0.0, 1.0); gl_vertex3f(-s, -s, s);
    
    // Right face
    gl_normal3f(1.0, 0.0, 0.0);
    gl_tex_coord2f(0.0, 0.0); gl_vertex3f(s, -s, s);
    gl_tex_coord2f(1.0, 0.0); gl_vertex3f(s, -s, -s);
    gl_tex_coord2f(1.0, 1.0); gl_vertex3f(s, s, -s);
    gl_tex_coord2f(0.0, 1.0); gl_vertex3f(s, s, s);
    
    // Left face
    gl_normal3f(-1.0, 0.0, 0.0);
    gl_tex_coord2f(0.0, 0.0); gl_vertex3f(-s, -s, -s);
    gl_tex_coord2f(1.0, 0.0); gl_vertex3f(-s, -s, s);
    gl_tex_coord2f(1.0, 1.0); gl_vertex3f(-s, s, s);
    gl_tex_coord2f(0.0, 1.0); gl_vertex3f(-s, s, -s);
    
    gl_end();
}

/// Demo: Initialize a checkerboard texture
pub fn demo_init_checkerboard_texture(tex_id: &mut u32) {
    texture::gl_gen_textures(1, core::slice::from_mut(tex_id));
    texture::gl_bind_texture(texture::GL_TEXTURE_2D, *tex_id);
    
    let tex_data = texture::create_checkerboard_texture(64, 0xFFFFFFFF, 0xFF404040);
    texture::gl_tex_image_2d(
        texture::GL_TEXTURE_2D, 0, texture::GL_RGBA,
        64, 64, 0, texture::GL_RGBA, 0, &tex_data
    );
    texture::gl_tex_parameteri(texture::GL_TEXTURE_2D, texture::GL_TEXTURE_MAG_FILTER, texture::GL_LINEAR);
    texture::gl_tex_parameteri(texture::GL_TEXTURE_2D, texture::GL_TEXTURE_MIN_FILTER, texture::GL_NEAREST);
}

/// Demo: Render a spinning textured cube
pub fn demo_render_textured_cube(angle: f32, tex_id: u32) {
    // Bind texture
    texture::gl_bind_texture(texture::GL_TEXTURE_2D, tex_id);
    texture::gl_enable_texture(texture::GL_TEXTURE_2D);
    
    // Set white color for texture modulation
    gl_color3f(1.0, 1.0, 1.0);
    
    gl_push_matrix();
    gl_rotatef(angle, 0.5, 1.0, 0.3);
    glut_textured_cube(1.5);
    gl_pop_matrix();
    
    texture::gl_disable_texture(texture::GL_TEXTURE_2D);
}
