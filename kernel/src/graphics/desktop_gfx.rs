//! Desktop Graphics Integration
//!
//! Provides easy-to-use graphics functions for the desktop manager.
//! Wraps the graphics engine to work with the framebuffer module.

use crate::framebuffer;
use crate::graphics::render2d::{FramebufferTarget, Color2D};
use crate::graphics::gui_renderer;

/// Create a FramebufferTarget from the current backbuffer
/// 
/// SAFETY: This assumes the backbuffer is valid and properly initialized.
/// The returned target is only valid while the backbuffer address remains stable.
pub unsafe fn get_render_target() -> Option<FramebufferTarget> {
    let (ptr, width, height, stride) = framebuffer::get_backbuffer_info()?;
    Some(FramebufferTarget::new(ptr as *mut u32, width, height, stride))
}

// ═══════════════════════════════════════════════════════════════════════════════
// OPTIMIZED DRAWING FUNCTIONS (use new graphics engine)
// ═══════════════════════════════════════════════════════════════════════════════

/// Draw a modern rounded rectangle with optional shadow
pub fn draw_rounded_rect(
    x: i32, y: i32,
    width: u32, height: u32,
    radius: u32,
    color: u32,
    shadow: bool,
) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            let fill_color = Color2D::from_u32(color);
            if shadow {
                gui_renderer::draw_rounded_rect_shadow(
                    &mut target, x, y, width, height, radius,
                    fill_color, 8, Color2D::rgba(0, 0, 0, 100),
                );
            } else {
                gui_renderer::draw_rounded_rect_fast(
                    &mut target, x, y, width, height, radius, fill_color,
                );
            }
        }
    }
}

/// Draw a vertical gradient
pub fn draw_gradient_v(x: i32, y: i32, width: u32, height: u32, top_color: u32, bottom_color: u32) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            gui_renderer::draw_gradient_v(
                &mut target, x, y, width, height,
                Color2D::from_u32(top_color),
                Color2D::from_u32(bottom_color),
            );
        }
    }
}

/// Draw a horizontal gradient
pub fn draw_gradient_h(x: i32, y: i32, width: u32, height: u32, left_color: u32, right_color: u32) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            gui_renderer::draw_gradient_h(
                &mut target, x, y, width, height,
                Color2D::from_u32(left_color),
                Color2D::from_u32(right_color),
            );
        }
    }
}

/// Draw a glow effect
pub fn draw_glow(x: i32, y: i32, width: u32, height: u32, glow_size: u32, color: u32) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            gui_renderer::draw_glow(
                &mut target, x, y, width, height, glow_size,
                Color2D::from_u32(color),
            );
        }
    }
}

/// Draw a modern window frame
pub fn draw_window_frame(
    x: i32, y: i32,
    width: u32, height: u32,
    title_height: u32,
    focused: bool,
) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            let style = gui_renderer::WindowStyle::default();
            gui_renderer::draw_window_frame(
                &mut target, x, y, width, height, title_height, &style, focused,
            );
        }
    }
}

/// Draw a modern button
pub fn draw_modern_button(
    x: i32, y: i32,
    width: u32, height: u32,
    style_type: ButtonType,
    hovered: bool,
    pressed: bool,
) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            let style = match style_type {
                ButtonType::Primary => gui_renderer::ButtonStyle::PRIMARY,
                ButtonType::Secondary => gui_renderer::ButtonStyle::SECONDARY,
                ButtonType::Danger => gui_renderer::ButtonStyle::DANGER,
            };
            gui_renderer::draw_button(&mut target, x, y, width, height, &style, hovered, pressed);
        }
    }
}

/// Button style type
#[derive(Clone, Copy)]
pub enum ButtonType {
    Primary,
    Secondary,
    Danger,
}

/// Draw anti-aliased line
pub fn draw_aa_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            gui_renderer::draw_aa_line(
                &mut target, x0, y0, x1, y1,
                Color2D::from_u32(color),
            );
        }
    }
}

/// Draw anti-aliased circle
pub fn draw_aa_circle(cx: i32, cy: i32, radius: u32, color: u32) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            gui_renderer::draw_aa_circle(
                &mut target, cx, cy, radius,
                Color2D::from_u32(color),
            );
        }
    }
}

/// Draw modern background
pub fn draw_modern_background(width: u32, height: u32) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            let accent = Color2D::rgb(0, 26, 13); // Subtle green accent
            gui_renderer::draw_modern_background(&mut target, width, height, accent);
        }
    }
}

/// Draw vignette effect
pub fn draw_vignette(width: u32, height: u32, strength: f32) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            gui_renderer::draw_vignette(&mut target, width, height, strength);
        }
    }
}

/// Draw pulse animation (for notifications)
pub fn draw_pulse_ring(cx: i32, cy: i32, radius: u32, ring_width: u32, color: u32, phase: f32) {
    unsafe {
        if let Some(mut target) = get_render_target() {
            gui_renderer::draw_pulse_ring(
                &mut target, cx, cy, radius, ring_width,
                Color2D::from_u32(color), phase,
            );
        }
    }
}
