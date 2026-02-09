//! TrustOS Graphics Engine
//!
//! Provides 2D/3D software rendering and GPU acceleration.
//!
//! Modules:
//! - render2d: 2D rendering with embedded-graphics
//! - render3d: Software 3D wireframe/filled rendering
//! - gui_renderer: Optimized UI primitives and effects
//! - desktop_gfx: Desktop integration helpers
//! - math3d: 3D math (vectors, matrices, transforms)
//! - opengl: OpenGL 1.x style immediate mode API
//! - compositor: TrustGL-based window compositor with effects
//! - fast_render: Optimized software rendering engine
//! - holomatrix: Volumetric 3D via matrix layer superposition
//! - raytracer: Real-time ray tracing engine

pub mod render2d;
pub mod render3d;
pub mod math3d;
pub mod gui_renderer;
pub mod desktop_gfx;
pub mod opengl;
pub mod compositor;
pub mod fast_render;
pub mod simd;
pub mod holomatrix;
pub mod raytracer;
pub mod texture;

pub use render2d::*;
pub use render3d::*;
pub use math3d::*;
pub use gui_renderer::*;
pub use desktop_gfx::*;
pub use opengl::*;
pub use compositor::*;
pub use fast_render::*;
pub use simd::*;
pub use holomatrix::*;
pub use texture::*;
