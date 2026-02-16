//! MiniFilm module — TrustOS YouTube presentation film
// Structure: scènes, helpers graphiques, intégration logo

use crate::framebuffer;

/// Affiche une image PPM à l'écran (placeholder, à remplacer par la vraie fonction d'affichage PPM)
fn draw_image_ppm(_path: &str, _x: usize, _y: usize) {
    // TODO: Charger et dessiner le buffer PPM à la position donnée
    framebuffer::draw_text(10, 10, "[STANDBY IMAGE]", 0xCCCCCCFF);
}

/// Effet fade-in (placeholder)
fn fade_in() {
    // TODO: Implémenter un vrai effet de fondu
    crate::cpu::tsc::delay_millis(200);
}

/// Effet glitch (placeholder)
fn glitch_effect() {
    // TODO: Implémenter un vrai effet glitch
    crate::cpu::tsc::delay_millis(120);
}

/// Affiche un logo TrustOS (placeholder, à remplacer par la vraie fonction d'affichage du logo)
fn draw_logo_ppm(_x: usize, _y: usize) {
    framebuffer::draw_text(10, 10, "[TRUSTOS LOGO]", 0xFF00FF00);
}

/// Animation de logs défilants (placeholder)
fn scroll_logs() {
    for i in 0..8 {
        framebuffer::clear();
        for j in 0..i {
            framebuffer::draw_text(5, 10 + j, &format!("log[{}]: kernel_call()", 1000 + j), 0xFF00FF00);
        }
        framebuffer::swap_buffers();
        crate::cpu::tsc::delay_millis(120);
    }
}

/// Affiche des icônes ASCII floutées (placeholder)
fn draw_ascii_icons() {
    framebuffer::draw_text(20, 15, "[WIN]   [APPLE]", 0xFF888888);
}

pub fn minifilm() {
    // 1. HOOK visuel fort : écran standby (bitmap PPM), fade-in, glitch
    framebuffer::clear();
    draw_image_ppm("script/trusttemp/standby.ppm", 0, 0);
    fade_in();
    glitch_effect();
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(1800);

    // 2. Pyramide animée (ASCII)
    framebuffer::clear();
    animate_ascii_pyramid(10, 40, 12, 6, 60);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(800);

    // 3. Texte voix off (hook)
    framebuffer::clear();
    framebuffer::draw_text(8, 20, "Chaque système que tu utilises...", 0xFFFFFFFF);
    framebuffer::draw_text(8, 22, "ne t'appartient pas.", 0xFFFFFFFF);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(1800);

    // 4. Montée paranoïa : texte "1984", logs défilants, icônes floutées (ASCII)
    framebuffer::clear();
    framebuffer::draw_text(20, 10, "1984", 0xFF00FF00);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(900);
    scroll_logs();
    draw_ascii_icons();
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(900);

    // 5. Pivot rationnel : schéma architecture
    framebuffer::clear();
    framebuffer::draw_text(10, 10, "[User]", 0xFF00FF00);
    framebuffer::draw_text(10, 12, "   |", 0xFF00FF00);
    framebuffer::draw_text(10, 13, "[OS]", 0xFF00FF00);
    framebuffer::draw_text(10, 15, "   |", 0xFF00FF00);
    framebuffer::draw_text(10, 16, "[Firmware]", 0xFF00FF00);
    framebuffer::draw_text(10, 18, "   |", 0xFF00FF00);
    framebuffer::draw_text(10, 19, "[Hardware]", 0xFF00FF00);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(1800);

    // 6. Reveal progressif : boot minimal, Rust compile, kernel log
    framebuffer::clear();
    framebuffer::draw_text(5, 10, "Booting TrustOS...", 0xFF888888);
    framebuffer::draw_text(5, 12, "rustc main.rs", 0xFF888888);
    framebuffer::draw_text(5, 14, "[ OK ] Kernel loaded", 0xFF00FF00);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(1200);

    // 7. REVEAL LOGO TrustOS
    framebuffer::clear();
    draw_logo_ppm(0, 0);
    framebuffer::draw_text(8, 20, "100% auditable.", 0xFFFFFFFF);
    framebuffer::draw_text(8, 21, "Pas d'obfuscation.", 0xFFFFFFFF);
    framebuffer::draw_text(8, 22, "Pas de télémétrie cachée.", 0xFFFFFFFF);
    framebuffer::draw_text(8, 24, "TrustOS.", 0xFF00FF00);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(2200);
}
    // 1. HOOK visuel fort : écran standby (bitmap PPM), fade-in
    framebuffer::clear();
    // Supposons draw_image_ppm existe et charge script/trusttemp/standby.ppm
    // draw_image_ppm("script/trusttemp/standby.ppm", x, y, w, h);
    framebuffer::draw_text(10, 10, "[STANDBY IMAGE]", 0xCCCCCCFF); // Placeholder
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(2000);

    // 2. Pyramide animée (ASCII)
    framebuffer::clear();
    animate_ascii_pyramid(10, 40, 12, 6, 60);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(800);

    // 3. Texte voix off (hook)
    framebuffer::clear();
    framebuffer::draw_text(8, 20, "Chaque système que tu utilises...", 0xFFFFFFFF);
    framebuffer::draw_text(8, 22, "ne t'appartient pas.", 0xFFFFFFFF);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(1800);

    // 4. Montée paranoïa : texte "1984", logs défilants, icônes floutées (ASCII)
    framebuffer::clear();
    framebuffer::draw_text(20, 10, "1984", 0xFF00FF00);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(900);
    // TODO: Ajouter scroll_logs(), draw_ascii_icons()...

    // 5. Pivot rationnel : schéma architecture
    framebuffer::clear();
    framebuffer::draw_text(10, 10, "[User]", 0xFF00FF00);
    framebuffer::draw_text(10, 12, "   |", 0xFF00FF00);
    framebuffer::draw_text(10, 13, "[OS]", 0xFF00FF00);
    framebuffer::draw_text(10, 15, "   |", 0xFF00FF00);
    framebuffer::draw_text(10, 16, "[Firmware]", 0xFF00FF00);
    framebuffer::draw_text(10, 18, "   |", 0xFF00FF00);
    framebuffer::draw_text(10, 19, "[Hardware]", 0xFF00FF00);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(1800);

    // 6. Reveal progressif : boot minimal, Rust compile, kernel log
    framebuffer::clear();
    framebuffer::draw_text(5, 10, "Booting TrustOS...", 0xFF888888);
    framebuffer::draw_text(5, 12, "rustc main.rs", 0xFF888888);
    framebuffer::draw_text(5, 14, "[ OK ] Kernel loaded", 0xFF00FF00);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(1200);

    // 7. REVEAL LOGO TrustOS
    framebuffer::clear();
    framebuffer::draw_text(10, 10, "[TRUSTOS LOGO]", 0xFF00FF00); // Placeholder pour draw_logo_ppm
    framebuffer::draw_text(8, 20, "100% auditable.", 0xFFFFFFFF);
    framebuffer::draw_text(8, 21, "Pas d'obfuscation.", 0xFFFFFFFF);
    framebuffer::draw_text(8, 22, "Pas de télémétrie cachée.", 0xFFFFFFFF);
    framebuffer::draw_text(8, 24, "TrustOS.", 0xFF00FF00);
    framebuffer::swap_buffers();
    crate::cpu::tsc::delay_millis(2200);
}
