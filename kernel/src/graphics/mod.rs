//! TrustOS Graphics Engine
//!
//! Provides 2D/3D software rendering and GPU acceleration.
//!
//! Modules:
//! - render2d: 2D rendering with embedded-graphics
//! - math3d: 3D math (vectors, matrices, transforms)
//! - opengl: OpenGL 1.x style immediate mode API
//! - compositor: TrustGL-based window compositor with effects
//! - fast_render: Optimized software rendering engine
//! - holomatrix: Volumetric 3D via matrix layer superposition
//! - raytracer: Real-time ray tracing engine

pub mod render2d;
pub mod math3d;
pub mod opengl;
pub mod compositor;
pub mod fast_render;
pub mod simd;
pub mod holomatrix;
pub mod raytracer;
pub mod texture;
pub mod scaling;

pub use render2d::*;
pub use math3d::*;
pub use opengl::*;
pub use compositor::*;
pub use fast_render::*;
pub use simd::*;
pub use holomatrix::*;
pub use texture::*;
