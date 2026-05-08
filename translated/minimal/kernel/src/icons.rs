



use crate::framebuffer;
use libm::{cosf, sinf};


pub const BX_: u32 = 32;
pub const DUC_: u32 = 16;


pub fn lle(x: u32, y: u32, color: u32, bg: u32) {
    
    let dark = darken(color, 0.6);
    let light = lighten(color, 1.3);
    
    
    framebuffer::fill_rect(x, y, 32, 32, bg);
    framebuffer::fill_rect(x + 2, y + 2, 28, 28, dark);
    
    
    framebuffer::fill_rect(x + 2, y + 2, 28, 6, color);
    
    
    framebuffer::fill_rect(x + 4, y + 4, 2, 2, 0xFFFF5555); 
    framebuffer::fill_rect(x + 8, y + 4, 2, 2, 0xFFFFAA00); 
    framebuffer::fill_rect(x + 12, y + 4, 2, 2, 0xFF55FF55); 
    
    
    framebuffer::fill_rect(x + 4, y + 10, 24, 18, 0xFF0A0A0A);
    
    
    framebuffer::fill_rect(x + 6, y + 14, 2, 6, color);  
    framebuffer::fill_rect(x + 8, y + 17, 2, 2, color);  
    framebuffer::fill_rect(x + 12, y + 20, 8, 2, light); 
}


pub fn liv(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.7);
    let light = lighten(color, 1.2);
    
    
    framebuffer::fill_rect(x + 2, y + 6, 12, 4, light);
    
    
    framebuffer::fill_rect(x + 2, y + 8, 28, 18, color);
    
    
    framebuffer::fill_rect(x + 2, y + 12, 28, 14, dark);
    
    
    framebuffer::fill_rect(x + 2, y + 12, 28, 2, light);
    
    
    framebuffer::fill_rect(x + 4, y + 24, 26, 2, darken(dark, 0.5));
}


pub fn lis(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.8);
    let light = lighten(color, 1.2);
    
    
    framebuffer::fill_rect(x + 4, y + 2, 20, 28, 0xFFEEEEEE);
    
    
    framebuffer::fill_rect(x + 18, y + 2, 6, 6, 0xFFCCCCCC);
    framebuffer::fill_rect(x + 18, y + 2, 1, 6, 0xFFDDDDDD);
    
    
    for i in 0..5 {
        framebuffer::fill_rect(x + 8, y + 10 + i * 4, 12, 2, dark);
    }
    
    
    framebuffer::draw_rect(x + 4, y + 2, 20, 28, color);
}


pub fn lkp(x: u32, y: u32, color: u32, _bg: u32) {
    let cx = x + 16;
    let u = y + 16;
    let light = lighten(color, 1.2);
    
    
    byw(cx, u, 6, color);
    byw(cx, u, 3, 0xFF0A0A0A);
    
    
    let ebi: [(i32, i32); 8] = [
        (0, -10), (7, -7), (10, 0), (7, 7),
        (0, 10), (-7, 7), (-10, 0), (-7, -7),
    ];
    for (dx, ad) in ebi {
        framebuffer::fill_rect(
            (cx as i32 + dx - 2) as u32,
            (u as i32 + ad - 2) as u32,
            5, 5, color
        );
    }
}


pub fn lif(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.6);
    
    
    framebuffer::fill_rect(x + 4, y + 2, 24, 28, dark);
    framebuffer::draw_rect(x + 4, y + 2, 24, 28, color);
    
    
    framebuffer::fill_rect(x + 6, y + 4, 20, 8, 0xFF1A2A1A);
    framebuffer::fill_rect(x + 18, y + 6, 6, 4, color); 
    
    
    for row in 0..4 {
        for col in 0..4 {
            let bx = x + 6 + col * 5;
            let dc = y + 14 + row * 4;
            let djr = if col == 3 { 0xFF44AA44 } else { 0xFF333333 };
            framebuffer::fill_rect(bx, dc, 4, 3, djr);
        }
    }
}


pub fn ljv(x: u32, y: u32, color: u32, _bg: u32) {
    let light = lighten(color, 1.2);
    
    
    draw_circle(x + 16, y + 16, 12, color);
    
    
    framebuffer::fill_rect(x + 6, y + 11, 20, 1, color);
    framebuffer::fill_rect(x + 4, y + 16, 24, 1, color);
    framebuffer::fill_rect(x + 6, y + 21, 20, 1, color);
    
    
    draw_circle(x + 16, y + 16, 6, color);
    
    
    framebuffer::fill_rect(x + 15, y + 4, 2, 24, color);
}


pub fn lhl(x: u32, y: u32, color: u32, _bg: u32) {
    let light = lighten(color, 1.3);
    
    
    byw(x + 16, y + 16, 12, color);
    byw(x + 16, y + 16, 10, 0xFF0A0A0A);
    
    
    framebuffer::fill_rect(x + 14, y + 10, 4, 4, light); 
    framebuffer::fill_rect(x + 14, y + 16, 4, 10, light); 
    framebuffer::fill_rect(x + 12, y + 24, 8, 2, light); 
}


pub fn ljt(x: u32, y: u32, color: u32, _bg: u32) {
    let light = lighten(color, 1.3);
    
    
    byw(x + 10, y + 24, 5, light);
    byw(x + 10, y + 24, 4, color);
    
    framebuffer::fill_rect(x + 14, y + 6, 2, 19, light);
    
    framebuffer::fill_rect(x + 16, y + 6, 2, 4, light);
    framebuffer::fill_rect(x + 18, y + 8, 2, 4, light);
    framebuffer::fill_rect(x + 20, y + 10, 2, 4, light);
}


pub fn fsx(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.7);
    let light = lighten(color, 1.2);
    
    
    framebuffer::fill_rect(x + 4, y + 10, 24, 14, dark);
    framebuffer::fill_rect(x + 2, y + 12, 4, 10, dark);
    framebuffer::fill_rect(x + 26, y + 12, 4, 10, dark);
    
    
    framebuffer::fill_rect(x + 8, y + 14, 2, 6, color);
    framebuffer::fill_rect(x + 6, y + 16, 6, 2, color);
    
    
    framebuffer::fill_rect(x + 22, y + 14, 3, 3, 0xFF55FF55);
    framebuffer::fill_rect(x + 18, y + 17, 3, 3, 0xFFFF5555);
}


pub fn lip(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.7);
    
    
    framebuffer::fill_rect(x + 4, y + 2, 24, 28, 0xFFEEEEEE);
    framebuffer::draw_rect(x + 4, y + 2, 24, 28, dark);
    
    
    framebuffer::fill_rect(x + 8, y + 6, 16, 2, color);
    framebuffer::fill_rect(x + 8, y + 10, 14, 2, dark);
    framebuffer::fill_rect(x + 8, y + 14, 16, 2, dark);
    framebuffer::fill_rect(x + 8, y + 18, 10, 2, dark);
    framebuffer::fill_rect(x + 8, y + 22, 14, 2, dark);
    
    
    framebuffer::fill_rect(x + 10, y + 22, 1, 4, color);
}






fn darken(color: u32, ha: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32 * ha;
    let g = ((color >> 8) & 0xFF) as f32 * ha;
    let b = (color & 0xFF) as f32 * ha;
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}


fn lighten(color: u32, ha: f32) -> u32 {
    let r = (((color >> 16) & 0xFF) as f32 * ha).min(255.0);
    let g = (((color >> 8) & 0xFF) as f32 * ha).min(255.0);
    let b = ((color & 0xFF) as f32 * ha).min(255.0);
    0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}


fn byw(cx: u32, u: u32, r: u32, color: u32) {
    let r = r as i32;
    let cx = cx as i32;
    let u = u as i32;
    
    for ad in -r..=r {
        for dx in -r..=r {
            if dx * dx + ad * ad <= r * r {
                let p = cx + dx;
                let o = u + ad;
                if p >= 0 && o >= 0 {
                    framebuffer::put_pixel(p as u32, o as u32, color);
                }
            }
        }
    }
}


fn draw_circle(cx: u32, u: u32, r: u32, color: u32) {
    let r = r as i32;
    let cx = cx as i32;
    let u = u as i32;
    
    let mut x = r;
    let mut y = 0;
    let mut err = 0;
    
    while x >= y {
        ccx(cx + x, u + y, color);
        ccx(cx + y, u + x, color);
        ccx(cx - y, u + x, color);
        ccx(cx - x, u + y, color);
        ccx(cx - x, u - y, color);
        ccx(cx - y, u - x, color);
        ccx(cx + y, u - x, color);
        ccx(cx + x, u - y, color);
        
        y += 1;
        err += 1 + 2 * y;
        if 2 * (err - x) + 1 > 0 {
            x -= 1;
            err += 1 - 2 * x;
        }
    }
}

fn ccx(x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::put_pixel(x as u32, y as u32, color);
    }
}





#[derive(Clone, Copy, PartialEq)]
pub enum IconType {
    Terminal,
    Folder,
    File,
    Settings,
    Calculator,
    Network,
    About,
    Music,
    Game,
    Editor,
    OpenGL,
    Browser,
    ModelEditor,
    GameBoy,
    GameLab,
    Chess,
}


pub fn qdr(icon_type: IconType, x: u32, y: u32, color: u32, bg: u32) {
    match icon_type {
        IconType::Terminal => lle(x, y, color, bg),
        IconType::Folder => liv(x, y, color, bg),
        IconType::File => lis(x, y, color, bg),
        IconType::Settings => lkp(x, y, color, bg),
        IconType::Calculator => lif(x, y, color, bg),
        IconType::Network => ljv(x, y, color, bg),
        IconType::About => lhl(x, y, color, bg),
        IconType::Music => ljt(x, y, color, bg),
        IconType::Game => fsx(x, y, color, bg),
        IconType::Editor => lip(x, y, color, bg),
        IconType::OpenGL => ljy(x, y, color, bg),
        IconType::Browser => lie(x, y, color, bg),
        IconType::ModelEditor => ljp(x, y, color, bg),
        IconType::GameBoy => fsx(x, y, color, bg), 
        IconType::GameLab => liy(x, y, color, bg),
        IconType::Chess => fsx(x, y, color, bg), 
    }
}


pub fn ljy(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.5);
    let light = lighten(color, 1.4);
    
    
    let cx = x as i32 + 16;
    let u = y as i32 + 18;
    let size: i32 = 8;
    
    
    let os = |p: i32, o: i32, c: u32| {
        if p >= 0 && o >= 0 {
            framebuffer::put_pixel(p as u32, o as u32, c);
        }
    };
    
    
    for i in 0..=size {
        os(cx - size + i, u - size, light);
        os(cx + size, u - size + i, light);
        os(cx + size - i, u + size, light);
        os(cx - size, u + size - i, light);
    }
    
    
    let offset: i32 = 5;
    for i in 0..=size {
        os(cx - size + i + offset, u - size - offset, dark);
        os(cx + size + offset, u - size + i - offset, dark);
        os(cx + size - i + offset, u + size - offset, dark);
        os(cx - size + offset, u + size - i - offset, dark);
    }
    
    
    for i in 0..offset {
        os(cx - size + i, u - size - i, color);
        os(cx + size + i, u - size - i, color);
        os(cx + size + i, u + size - i, color);
        os(cx - size + i, u + size - i, color);
    }
    
    
    let bu = x as i32;
    let ty = y as i32;
    
    os(bu + 10, ty + 26, light);
    os(bu + 11, ty + 26, light);
    os(bu + 12, ty + 26, light);
    os(bu + 9, ty + 27, light);
    os(bu + 9, ty + 28, light);
    os(bu + 10, ty + 29, light);
    os(bu + 11, ty + 29, light);
    os(bu + 12, ty + 29, light);
    os(bu + 12, ty + 28, light);
    
    
    os(bu + 15, ty + 26, light);
    os(bu + 15, ty + 27, light);
    os(bu + 15, ty + 28, light);
    os(bu + 15, ty + 29, light);
    os(bu + 16, ty + 29, light);
    os(bu + 17, ty + 29, light);
}


pub fn lie(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.6);
    let light = lighten(color, 1.3);
    
    let cx = x as i32 + 16;
    let u = y as i32 + 16;
    
    
    for cc in 0..360 {
        let abf = (cc as f32) * 3.14159 / 180.0;
        let p = cx + (cosf(abf) * 12.0) as i32;
        let o = u + (sinf(abf) * 12.0) as i32;
        if p >= 0 && o >= 0 {
            framebuffer::put_pixel(p as u32, o as u32, color);
        }
    }
    
    
    for cc in 0..360 {
        let abf = (cc as f32) * 3.14159 / 180.0;
        let p = cx + (cosf(abf) * 8.0) as i32;
        let o = u + (sinf(abf) * 8.0) as i32;
        if p >= 0 && o >= 0 {
            framebuffer::put_pixel(p as u32, o as u32, dark);
        }
    }
    
    
    for ad in -12i32..=12 {
        let o = u + ad;
        if o >= 0 {
            framebuffer::put_pixel(cx as u32, o as u32, light);
        }
    }
    
    
    for dx in -12i32..=12 {
        let p = cx + dx;
        if p >= 0 {
            framebuffer::put_pixel(p as u32, u as u32, light);
        }
    }
    
    
    for dx in -10i32..=10 {
        let p = cx + dx;
        if p >= 0 {
            framebuffer::put_pixel(p as u32, (u - 6) as u32, dark);
            framebuffer::put_pixel(p as u32, (u + 6) as u32, dark);
        }
    }
    
    
    framebuffer::fill_rect(x + 2, y + 26, 28, 4, dark);
    framebuffer::fill_rect(x + 4, y + 27, 24, 2, 0xFF202020);
}


pub fn ljp(x: u32, y: u32, color: u32, _bg: u32) {
    let dark = darken(color, 0.5);
    let light = lighten(color, 1.4);
    let accent = 0xFF00FFAA; 
    
    let cx = x as i32 + 16;
    let u = y as i32 + 16;
    
    
    let os = |p: i32, o: i32, c: u32| {
        if p >= 0 && o >= 0 {
            framebuffer::put_pixel(p as u32, o as u32, c);
        }
    };
    
    
    let j: i32 = 7;
    for i in 0..=j {
        os(cx - j + i - 2, u - j + 2, accent);
        os(cx + j - 2, u - j + i + 2, accent);
        os(cx + j - i - 2, u + j + 2, accent);
        os(cx - j - 2, u + j - i + 2, accent);
    }
    
    
    let ays: i32 = 5;
    for i in 0..=j {
        os(cx - j + i + ays - 2, u - j - ays + 2, dark);
        os(cx + j + ays - 2, u - j + i - ays + 2, dark);
    }
    
    
    for i in 0..ays {
        os(cx - j + i - 2, u - j - i + 2, light);
        os(cx + j + i - 2, u - j - i + 2, light);
        os(cx + j + i - 2, u + j - i + 2, light);
    }
    
    
    for i in 0..5 {
        os(cx + 8, u + 6 + i, 0xFFFFFFFF);
        os(cx + 6 + i, u + 8, 0xFFFFFFFF);
    }
    
    
    os(cx - j - 2, u - j + 2, 0xFFFFFF00);
    os(cx + j - 2, u + j + 2, 0xFFFFFF00);
    os(cx + j + ays - 2, u - j - ays + 2, 0xFFFFFF00);
}


pub fn liy(x: u32, y: u32, color: u32, _bg: u32) {
    use crate::framebuffer;
    let na = 0xFF00FF88u32;
    let dim = darken(color, 0.5);
    
    
    framebuffer::fill_rect(x + 8, y + 16, 16, 12, dim);
    framebuffer::fill_rect(x + 6, y + 20, 20, 8, dim);
    
    
    framebuffer::fill_rect(x + 13, y + 6, 6, 10, dim);
    
    
    framebuffer::fill_rect(x + 11, y + 4, 10, 2, color);
    
    
    framebuffer::fill_rect(x + 9, y + 22, 14, 4, na);
    framebuffer::fill_rect(x + 10, y + 18, 12, 4, 0xFF00CC66);
    
    
    framebuffer::fill_rect(x + 11, y + 19, 2, 2, 0xFFFFFFFF);
    framebuffer::fill_rect(x + 16, y + 23, 2, 2, 0xFFFFFFFF);
    framebuffer::fill_rect(x + 19, y + 20, 2, 2, 0xFFFFFFFF);
    
    
    framebuffer::fill_rect(x + 7, y + 16, 1, 12, na);
    framebuffer::fill_rect(x + 24, y + 16, 1, 12, na);
    framebuffer::fill_rect(x + 6, y + 28, 20, 1, na);
}

