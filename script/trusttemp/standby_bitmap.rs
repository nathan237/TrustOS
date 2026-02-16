//! Version bitmap/PPM du standby screen (TrustOS)
//! Nécessite un fichier standby.ppm généré à partir de l'image d'origine
use crate::framebuffer;

// TODO: Générer standby.ppm à partir de l'image fournie et l'inclure dans le build
// (ou charger dynamiquement si supporté)

pub fn draw_standby_bitmap() {
    // Supposons que le buffer PPM est inclus ou accessible
    // Exemple fictif : framebuffer::draw_image(&STANDBY_PPM, x, y, w, h);
    // Ici, on simule juste un fond gris et le texte
    framebuffer::clear_color(0x222222FF);
    framebuffer::draw_text(15, 15, "[IMAGE STANDBY PPM]", 0xCCCCCCFF);
    framebuffer::draw_text(15, 18, "PLEASE STAND BY", 0xFFFFFFFF);
    framebuffer::swap_buffers();
}
