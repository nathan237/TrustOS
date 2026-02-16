//! Version ASCII/vectorielle du standby screen (TrustOS)
use crate::framebuffer;

pub fn draw_standby_ascii() {
    framebuffer::clear();
    // Cercle central (approximation ASCII)
    let circle = [
        "      _____      ",
        "   .-'     '-.   ",
        "  /           \  ",
        " |             | ",
        " |             | ",
        "  \           /  ",
        "   '-.___.-'     "
    ];
    let y0 = 10;
    for (i, line) in circle.iter().enumerate() {
        framebuffer::draw_text(20, y0 + i, line, 0xCCCCCCFF);
    }
    // Graduations principales (lignes)
    framebuffer::draw_text(27, y0 + 1, "|", 0xCCCCCCFF);
    framebuffer::draw_text(27, y0 + 5, "|", 0xCCCCCCFF);
    framebuffer::draw_text(22, y0 + 3, "-", 0xCCCCCCFF);
    framebuffer::draw_text(32, y0 + 3, "-", 0xCCCCCCFF);
    // Texte central
    framebuffer::draw_text(15, y0 + 8, "PLEASE STAND BY", 0xFFFFFFFF);
    framebuffer::swap_buffers();
}
