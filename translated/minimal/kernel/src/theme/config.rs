












use alloc::string::String;
use alloc::vec::Vec;
use super::{Theme, ThemeColors, TaskbarConfig, TaskbarPosition, WindowConfig, WallpaperConfig, WallpaperMode};


pub fn nrj(path: &str) -> Option<Theme> {
    
    let content = match crate::vfs::gqh(path) {
        Ok(j) => j,
        Err(_) => {
            crate::serial_println!("[THEME] Cannot read file: {}", path);
            return None;
        }
    };
    
    nrk(&content)
}


pub fn nrk(content: &str) -> Option<Theme> {
    let mut theme = Theme::cia();
    let mut hpv = String::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        
        
        if line.starts_with('[') && line.ends_with(']') {
            hpv = String::from(&line[1..line.len()-1]);
            continue;
        }
        
        
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos+1..].trim();
            
            jwx(&mut theme, &hpv, key, value);
        }
    }
    
    Some(theme)
}

fn jwx(theme: &mut Theme, section: &str, key: &str, value: &str) {
    match section {
        "theme" | "general" => match key {
            "name" => theme.name = String::from(value),
            "base" => {
                
                match value {
                    "dark" | "dark_green" => theme.colors = ThemeColors::cia(),
                    "windows11" | "win11" => theme.colors = ThemeColors::ffg(),
                    "light" => theme.colors = ThemeColors::light(),
                    _ => {}
                }
            }
            _ => {}
        }
        
        "colors" => {
            if let Some(color) = aul(value) {
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
                if let Some(h) = cnw(value) {
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
                theme.taskbar.centered_icons = gmg(value);
            }
            "show_clock" => theme.taskbar.show_clock = gmg(value),
            "show_date" => theme.taskbar.show_date = gmg(value),
            "transparency" => {
                if let Some(t) = cnw(value) {
                    theme.taskbar.transparency = t.clamp(0, 255) as u8;
                }
            }
            _ => {}
        }
        
        "window" | "windows" => match key {
            "titlebar_height" => {
                if let Some(h) = cnw(value) {
                    theme.window.titlebar_height = h.clamp(20, 48);
                }
            }
            "border_radius" | "radius" => {
                if let Some(r) = cnw(value) {
                    theme.window.border_radius = r.clamp(0, 20);
                }
            }
            "shadow_size" | "shadow" => {
                if let Some(j) = cnw(value) {
                    theme.window.shadow_size = j.clamp(0, 32);
                }
            }
            "shadow_opacity" => {
                if let Some(ays) = cnw(value) {
                    theme.window.shadow_opacity = ays.clamp(0, 255) as u8;
                }
            }
            "border_width" => {
                if let Some(w) = cnw(value) {
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
                if let Some(c) = aul(value) {
                    theme.wallpaper.fallback_color = c;
                }
            }
            _ => {}
        }
        
        _ => {}
    }
}


fn aul(j: &str) -> Option<u32> {
    let j = j.trim();
    
    
    if j.starts_with("0x") || j.starts_with("0X") {
        return u32::from_str_radix(&j[2..], 16).ok().map(|c| {
            
            if j.len() <= 8 { 0xFF000000 | c } else { c }
        });
    }
    
    
    if j.starts_with('#') {
        let ga = &j[1..];
        return match ga.len() {
            3 => {
                
                let r = u8::from_str_radix(&ga[0..1], 16).ok()?;
                let g = u8::from_str_radix(&ga[1..2], 16).ok()?;
                let b = u8::from_str_radix(&ga[2..3], 16).ok()?;
                Some(0xFF000000 | ((r as u32 * 17) << 16) | ((g as u32 * 17) << 8) | (b as u32 * 17))
            }
            6 => {
                u32::from_str_radix(ga, 16).ok().map(|c| 0xFF000000 | c)
            }
            8 => {
                u32::from_str_radix(ga, 16).ok()
            }
            _ => None
        };
    }
    
    
    j.parse::<u32>().ok()
}


fn gmg(j: &str) -> bool {
    matches!(j.trim(), "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "1" | "on" | "On" | "ON")
}


fn cnw(j: &str) -> Option<u32> {
    j.trim().parse().ok()
}






pub fn mcm(theme: &Theme) -> String {
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
    fn pgk() {
        assert_eq!(aul("0xFF00FF"), Some(0xFFFF00FF));
        assert_eq!(aul("#FF0000"), Some(0xFFFF0000));
        assert_eq!(aul("#F00"), Some(0xFFFF0000));
    }
}
