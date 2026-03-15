












use alloc::string::String;
use alloc::vec::Vec;
use super::{Theme, ThemeColors, TaskbarConfig, TaskbarPosition, WindowConfig, WallpaperConfig, WallpaperMode};


pub fn vee(path: &str) -> Option<Theme> {
    
    let ca = match crate::vfs::lxu(path) {
        Ok(e) => e,
        Err(_) => {
            crate::serial_println!("[THEME] Cannot read file: {}", path);
            return None;
        }
    };
    
    vef(&ca)
}


pub fn vef(ca: &str) -> Option<Theme> {
    let mut theme = Theme::fgk();
    let mut nik = String::new();
    
    for line in ca.ak() {
        let line = line.em();
        
        
        if line.is_empty() || line.cj('#') || line.cj(';') {
            continue;
        }
        
        
        if line.cj('[') && line.pp(']') {
            nik = String::from(&line[1..line.len()-1]);
            continue;
        }
        
        
        if let Some(bzo) = line.du('=') {
            let bs = line[..bzo].em();
            let bn = line[bzo+1..].em();
            
            qjp(&mut theme, &nik, bs, bn);
        }
    }
    
    Some(theme)
}

fn qjp(theme: &mut Theme, ava: &str, bs: &str, bn: &str) {
    match ava {
        "theme" | "general" => match bs {
            "name" => theme.j = String::from(bn),
            "base" => {
                
                match bn {
                    "dark" | "dark_green" => theme.colors = ThemeColors::fgk(),
                    "windows11" | "win11" => theme.colors = ThemeColors::jwx(),
                    "light" => theme.colors = ThemeColors::light(),
                    _ => {}
                }
            }
            _ => {}
        }
        
        "colors" => {
            if let Some(s) = clt(bn) {
                match bs {
                    "background" => theme.colors.cop = s,
                    "foreground" => theme.colors.ivg = s,
                    "accent" => theme.colors.mm = s,
                    "accent_hover" => theme.colors.cof = s,
                    "accent_dark" => theme.colors.iiq = s,
                    "surface" => theme.colors.surface = s,
                    "surface_hover" => theme.colors.dwl = s,
                    "text" | "text_primary" => theme.colors.dcp = s,
                    "text_secondary" => theme.colors.dwr = s,
                    "titlebar" | "titlebar_active" => theme.colors.idr = s,
                    "titlebar_inactive" => theme.colors.jth = s,
                    "border" => theme.colors.acu = s,
                    "border_focused" => theme.colors.dzc = s,
                    "taskbar" | "taskbar_bg" => theme.colors.ida = s,
                    "shadow" => theme.colors.zc = s,
                    "selection" => theme.colors.gry = s,
                    "success" => theme.colors.vx = s,
                    "warning" => theme.colors.ekt = s,
                    "error" => theme.colors.zt = s,
                    _ => {}
                }
            }
        }
        
        "taskbar" => match bs {
            "height" => {
                if let Some(i) = fqj(bn) {
                    theme.bou.ac = i.qp(24, 96);
                }
            }
            "position" => {
                theme.bou.qf = match bn.em() {
                    "top" | "Top" | "TOP" => TaskbarPosition::Jd,
                    "left" | "Left" | "LEFT" => TaskbarPosition::Ap,
                    "right" | "Right" | "RIGHT" => TaskbarPosition::Ca,
                    _ => TaskbarPosition::Hk,
                };
            }
            "centered" | "centered_icons" => {
                theme.bou.gch = lsg(bn);
            }
            "show_clock" => theme.bou.iai = lsg(bn),
            "show_date" => theme.bou.jqa = lsg(bn),
            "transparency" => {
                if let Some(ab) = fqj(bn) {
                    theme.bou.juc = ab.qp(0, 255) as u8;
                }
            }
            _ => {}
        }
        
        "window" | "windows" => match bs {
            "titlebar_height" => {
                if let Some(i) = fqj(bn) {
                    theme.bh.ids = i.qp(20, 48);
                }
            }
            "border_radius" | "radius" => {
                if let Some(m) = fqj(bn) {
                    theme.bh.avh = m.qp(0, 20);
                }
            }
            "shadow_size" | "shadow" => {
                if let Some(e) = fqj(bn) {
                    theme.bh.iac = e.qp(0, 32);
                }
            }
            "shadow_opacity" => {
                if let Some(dkb) = fqj(bn) {
                    theme.bh.dby = dkb.qp(0, 255) as u8;
                }
            }
            "border_width" => {
                if let Some(d) = fqj(bn) {
                    theme.bh.dek = d.qp(0, 4);
                }
            }
            _ => {}
        }
        
        "wallpaper" => match bs {
            "path" | "file" => theme.bsx.path = String::from(bn),
            "mode" => {
                theme.bsx.ev = match bn.em() {
                    "stretch" | "Stretch" | "STRETCH" => WallpaperMode::Uq,
                    "center" | "Center" | "CENTER" => WallpaperMode::Eo,
                    "tile" | "Tile" | "TILE" => WallpaperMode::Azw,
                    "fill" | "Fill" | "FILL" => WallpaperMode::Bhc,
                    "fit" | "Fit" | "FIT" => WallpaperMode::Bhh,
                    "solid" | "Solid" | "SOLID" | "color" | "Color" => WallpaperMode::Aes,
                    _ => WallpaperMode::Uq,
                };
            }
            "color" | "fallback" | "fallback_color" => {
                if let Some(r) = clt(bn) {
                    theme.bsx.hiv = r;
                }
            }
            _ => {}
        }
        
        _ => {}
    }
}


fn clt(e: &str) -> Option<u32> {
    let e = e.em();
    
    
    if e.cj("0x") || e.cj("0X") {
        return u32::wa(&e[2..], 16).bq().map(|r| {
            
            if e.len() <= 8 { 0xFF000000 | r } else { r }
        });
    }
    
    
    if e.cj('#') {
        let nu = &e[1..];
        return match nu.len() {
            3 => {
                
                let m = u8::wa(&nu[0..1], 16).bq()?;
                let at = u8::wa(&nu[1..2], 16).bq()?;
                let o = u8::wa(&nu[2..3], 16).bq()?;
                Some(0xFF000000 | ((m as u32 * 17) << 16) | ((at as u32 * 17) << 8) | (o as u32 * 17))
            }
            6 => {
                u32::wa(nu, 16).bq().map(|r| 0xFF000000 | r)
            }
            8 => {
                u32::wa(nu, 16).bq()
            }
            _ => None
        };
    }
    
    
    e.parse::<u32>().bq()
}


fn lsg(e: &str) -> bool {
    oh!(e.em(), "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "1" | "on" | "On" | "ON")
}


fn fqj(e: &str) -> Option<u32> {
    e.em().parse().bq()
}






pub fn tcp(theme: &Theme) -> String {
    use alloc::format;
    
    let mut bd = String::new();
    
    bd.t("# TrustOS Theme Configuration\n");
    bd.t("# Edit colors and settings, then reload with 'theme reload'\n\n");
    
    bd.t("[theme]\n");
    bd.t(&format!("name = {}\n\n", theme.j));
    
    bd.t("[colors]\n");
    bd.t(&format!("background = 0x{:08X}\n", theme.colors.cop));
    bd.t(&format!("accent = 0x{:08X}\n", theme.colors.mm));
    bd.t(&format!("text_primary = 0x{:08X}\n", theme.colors.dcp));
    bd.t(&format!("text_secondary = 0x{:08X}\n", theme.colors.dwr));
    bd.t(&format!("surface = 0x{:08X}\n", theme.colors.surface));
    bd.t(&format!("titlebar_active = 0x{:08X}\n", theme.colors.idr));
    bd.t(&format!("border = 0x{:08X}\n", theme.colors.acu));
    bd.t(&format!("border_focused = 0x{:08X}\n", theme.colors.dzc));
    bd.t(&format!("taskbar_bg = 0x{:08X}\n", theme.colors.ida));
    bd.t("\n");
    
    bd.t("[taskbar]\n");
    bd.t(&format!("height = {}\n", theme.bou.ac));
    bd.t(&format!("position = {}\n", match theme.bou.qf {
        TaskbarPosition::Jd => "top",
        TaskbarPosition::Hk => "bottom",
        TaskbarPosition::Ap => "left",
        TaskbarPosition::Ca => "right",
    }));
    bd.t(&format!("centered_icons = {}\n", theme.bou.gch));
    bd.t(&format!("show_clock = {}\n", theme.bou.iai));
    bd.t(&format!("transparency = {}\n", theme.bou.juc));
    bd.t("\n");
    
    bd.t("[window]\n");
    bd.t(&format!("titlebar_height = {}\n", theme.bh.ids));
    bd.t(&format!("border_radius = {}\n", theme.bh.avh));
    bd.t(&format!("shadow_size = {}\n", theme.bh.iac));
    bd.t(&format!("border_width = {}\n", theme.bh.dek));
    bd.t("\n");
    
    bd.t("[wallpaper]\n");
    bd.t(&format!("path = {}\n", theme.bsx.path));
    bd.t(&format!("mode = {}\n", match theme.bsx.ev {
        WallpaperMode::Uq => "stretch",
        WallpaperMode::Eo => "center",
        WallpaperMode::Azw => "tile",
        WallpaperMode::Bhc => "fill",
        WallpaperMode::Bhh => "fit",
        WallpaperMode::Aes => "solid",
    }));
    bd.t(&format!("fallback_color = 0x{:08X}\n", theme.bsx.hiv));
    
    bd
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn xea() {
        assert_eq!(clt("0xFF00FF"), Some(0xFFFF00FF));
        assert_eq!(clt("#FF0000"), Some(0xFFFF0000));
        assert_eq!(clt("#F00"), Some(0xFFFF0000));
    }
}
