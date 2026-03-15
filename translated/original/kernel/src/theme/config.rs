// ═══════════════════════════════════════════════════════════════════════════════
// Theme Configuration Parser
// ═══════════════════════════════════════════════════════════════════════════════
//
// Simple INI-style parser for theme configuration files
//
// Format:
// [section]
// key = value
// # comments
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use alloc::vec::Vec;
use super::{Theme, ThemeColors, TaskbarConfig, TaskbarPosition, WindowConfig, WallpaperConfig, WallpaperMode};

/// Parse a theme configuration file
pub fn parse_theme_file(path: &str) -> Option<Theme> {
    // Read file from VFS
    let content = match crate::vfs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            crate::serial_println!("[THEME] Cannot read file: {}", path);
            return None;
        }
    };
    
    parse_theme_string(&content)
}

/// Parse theme from string content
pub fn parse_theme_string(content: &str) -> Option<Theme> {
    let mut theme = Theme::dark_green();
    let mut current_section = String::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        
        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            current_section = String::from(&line[1..line.len()-1]);
            continue;
        }
        
        // Key = Value
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos+1..].trim();
            
            apply_config(&mut theme, &current_section, key, value);
        }
    }
    
    Some(theme)
}

fn apply_config(theme: &mut Theme, section: &str, key: &str, value: &str) {
    match section {
        "theme" | "general" => match key {
            "name" => theme.name = String::from(value),
            "base" => {
                // Load base theme first
                match value {
                    "dark" | "dark_green" => theme.colors = ThemeColors::dark_green(),
                    "windows11" | "win11" => theme.colors = ThemeColors::windows11_dark(),
                    "light" => theme.colors = ThemeColors::light(),
                    _ => {}
                }
            }
            _ => {}
        }
        
        "colors" => {
            if let Some(color) = parse_color(value) {
                match key {
                    "background" => theme.colors.background = color,
                    "foreground" => theme.colors.foreground = color,
                    "accent" => theme.colors.accent = color,
                    "accent_hover" => theme.colors.accent_hover = color,
                    "accent_dark" => theme.colors.accent_dark = color,
                    "surface" => theme.colors.surface = color,
                    "surface_hover" => theme.colors.surface_hover = color,
                    "text" | "text_primary" => theme.colors.text_primary = color,
                    "text_secondary" => theme.colors.text_secondary = color,
                    "titlebar" | "titlebar_active" => theme.colors.titlebar_active = color,
                    "titlebar_inactive" => theme.colors.titlebar_inactive = color,
                    "border" => theme.colors.border = color,
                    "border_focused" => theme.colors.border_focused = color,
                    "taskbar" | "taskbar_bg" => theme.colors.taskbar_bg = color,
                    "shadow" => theme.colors.shadow = color,
                    "selection" => theme.colors.selection = color,
                    "success" => theme.colors.success = color,
                    "warning" => theme.colors.warning = color,
                    "error" => theme.colors.error = color,
                    _ => {}
                }
            }
        }
        
        "taskbar" => match key {
            "height" => {
                if let Some(h) = parse_u32(value) {
                    theme.taskbar.height = h.clamp(24, 96);
                }
            }
            "position" => {
                theme.taskbar.position = match value.trim() {
                    "top" | "Top" | "TOP" => TaskbarPosition::Top,
                    "left" | "Left" | "LEFT" => TaskbarPosition::Left,
                    "right" | "Right" | "RIGHT" => TaskbarPosition::Right,
                    _ => TaskbarPosition::Bottom,
                };
            }
            "centered" | "centered_icons" => {
                theme.taskbar.centered_icons = parse_bool(value);
            }
            "show_clock" => theme.taskbar.show_clock = parse_bool(value),
            "show_date" => theme.taskbar.show_date = parse_bool(value),
            "transparency" => {
                if let Some(t) = parse_u32(value) {
                    theme.taskbar.transparency = t.clamp(0, 255) as u8;
                }
            }
            _ => {}
        }
        
        "window" | "windows" => match key {
            "titlebar_height" => {
                if let Some(h) = parse_u32(value) {
                    theme.window.titlebar_height = h.clamp(20, 48);
                }
            }
            "border_radius" | "radius" => {
                if let Some(r) = parse_u32(value) {
                    theme.window.border_radius = r.clamp(0, 20);
                }
            }
            "shadow_size" | "shadow" => {
                if let Some(s) = parse_u32(value) {
                    theme.window.shadow_size = s.clamp(0, 32);
                }
            }
            "shadow_opacity" => {
                if let Some(o) = parse_u32(value) {
                    theme.window.shadow_opacity = o.clamp(0, 255) as u8;
                }
            }
            "border_width" => {
                if let Some(w) = parse_u32(value) {
                    theme.window.border_width = w.clamp(0, 4);
                }
            }
            _ => {}
        }
        
        "wallpaper" => match key {
            "path" | "file" => theme.wallpaper.path = String::from(value),
            "mode" => {
                theme.wallpaper.mode = match value.trim() {
                    "stretch" | "Stretch" | "STRETCH" => WallpaperMode::Stretch,
                    "center" | "Center" | "CENTER" => WallpaperMode::Center,
                    "tile" | "Tile" | "TILE" => WallpaperMode::Tile,
                    "fill" | "Fill" | "FILL" => WallpaperMode::Fill,
                    "fit" | "Fit" | "FIT" => WallpaperMode::Fit,
                    "solid" | "Solid" | "SOLID" | "color" | "Color" => WallpaperMode::Solid,
                    _ => WallpaperMode::Stretch,
                };
            }
            "color" | "fallback" | "fallback_color" => {
                if let Some(c) = parse_color(value) {
                    theme.wallpaper.fallback_color = c;
                }
            }
            _ => {}
        }
        
        _ => {}
    }
}

/// Parse a color value (supports: 0xRRGGBB, #RRGGBB, #RGB)
fn parse_color(s: &str) -> Option<u32> {
    let s = s.trim();
    
    // 0xAARRGGBB or 0xRRGGBB format
    if s.starts_with("0x") || s.starts_with("0X") {
        return u32::from_str_radix(&s[2..], 16).ok().map(|c| {
            // Add alpha if not present
            if s.len() <= 8 { 0xFF000000 | c } else { c }
        });
    }
    
    // #AARRGGBB, #RRGGBB or #RGB format
    if s.starts_with('#') {
        let hex = &s[1..];
        return match hex.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(0xFF000000 | ((r as u32 * 17) << 16) | ((g as u32 * 17) << 8) | (b as u32 * 17))
            }
            6 => {
                u32::from_str_radix(hex, 16).ok().map(|c| 0xFF000000 | c)
            }
            8 => {
                u32::from_str_radix(hex, 16).ok()
            }
            _ => None
        };
    }
    
    // Try decimal
    s.parse::<u32>().ok()
}

/// Parse a boolean value
fn parse_bool(s: &str) -> bool {
    matches!(s.trim(), "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "1" | "on" | "On" | "ON")
}

/// Parse a u32 value
fn parse_u32(s: &str) -> Option<u32> {
    s.trim().parse().ok()
}

// ═══════════════════════════════════════════════════════════════════════════════
// THEME FILE WRITER
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate a theme configuration string
pub fn generate_theme_config(theme: &Theme) -> String {
    use alloc::format;
    
    let mut out = String::new();
    
    out.push_str("# TrustOS Theme Configuration\n");
    out.push_str("# Edit colors and settings, then reload with 'theme reload'\n\n");
    
    out.push_str("[theme]\n");
    out.push_str(&format!("name = {}\n\n", theme.name));
    
    out.push_str("[colors]\n");
    out.push_str(&format!("background = 0x{:08X}\n", theme.colors.background));
    out.push_str(&format!("accent = 0x{:08X}\n", theme.colors.accent));
    out.push_str(&format!("text_primary = 0x{:08X}\n", theme.colors.text_primary));
    out.push_str(&format!("text_secondary = 0x{:08X}\n", theme.colors.text_secondary));
    out.push_str(&format!("surface = 0x{:08X}\n", theme.colors.surface));
    out.push_str(&format!("titlebar_active = 0x{:08X}\n", theme.colors.titlebar_active));
    out.push_str(&format!("border = 0x{:08X}\n", theme.colors.border));
    out.push_str(&format!("border_focused = 0x{:08X}\n", theme.colors.border_focused));
    out.push_str(&format!("taskbar_bg = 0x{:08X}\n", theme.colors.taskbar_bg));
    out.push_str("\n");
    
    out.push_str("[taskbar]\n");
    out.push_str(&format!("height = {}\n", theme.taskbar.height));
    out.push_str(&format!("position = {}\n", match theme.taskbar.position {
        TaskbarPosition::Top => "top",
        TaskbarPosition::Bottom => "bottom",
        TaskbarPosition::Left => "left",
        TaskbarPosition::Right => "right",
    }));
    out.push_str(&format!("centered_icons = {}\n", theme.taskbar.centered_icons));
    out.push_str(&format!("show_clock = {}\n", theme.taskbar.show_clock));
    out.push_str(&format!("transparency = {}\n", theme.taskbar.transparency));
    out.push_str("\n");
    
    out.push_str("[window]\n");
    out.push_str(&format!("titlebar_height = {}\n", theme.window.titlebar_height));
    out.push_str(&format!("border_radius = {}\n", theme.window.border_radius));
    out.push_str(&format!("shadow_size = {}\n", theme.window.shadow_size));
    out.push_str(&format!("border_width = {}\n", theme.window.border_width));
    out.push_str("\n");
    
    out.push_str("[wallpaper]\n");
    out.push_str(&format!("path = {}\n", theme.wallpaper.path));
    out.push_str(&format!("mode = {}\n", match theme.wallpaper.mode {
        WallpaperMode::Stretch => "stretch",
        WallpaperMode::Center => "center",
        WallpaperMode::Tile => "tile",
        WallpaperMode::Fill => "fill",
        WallpaperMode::Fit => "fit",
        WallpaperMode::Solid => "solid",
    }));
    out.push_str(&format!("fallback_color = 0x{:08X}\n", theme.wallpaper.fallback_color));
    
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("0xFF00FF"), Some(0xFFFF00FF));
        assert_eq!(parse_color("#FF0000"), Some(0xFFFF0000));
        assert_eq!(parse_color("#F00"), Some(0xFFFF0000));
    }
}
