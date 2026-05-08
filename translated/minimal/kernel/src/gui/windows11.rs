








use crate::framebuffer;






pub mod colors {
    
    pub const DXR_: u32 = 0xFF202020;           
    pub const DXS_: u32 = 0xFF1A1A1A;         
    pub const DFY_: u32 = 0xE0282828;          
    pub const El: u32 = 0xFF2D2D2D;             
    pub const JT_: u32 = 0xFF383838;       
    pub const QR_: u32 = 0xFF404040;    
    
    
    pub const ELS_: u32 = 0xFF1F1F1F;    
    pub const ELT_: u32 = 0xFF2B2B2B;  
    
    
    pub const Ch: u32 = 0xFF0078D4;              
    pub const DFV_: u32 = 0xFF60CDFF;        
    pub const DFS_: u32 = 0xFF005A9E;         
    
    
    pub const AB_: u32 = 0xFFFFFFFF;        
    pub const O_: u32 = 0xFFB3B3B3;      
    pub const QW_: u32 = 0xFF6E6E6E;       
    
    
    pub const AAZ_: u32 = 0xFF3D3D3D;       
    pub const DHM_: u32 = 0xFF4D4D4D;      
    pub const DHN_: u32 = 0xFF6B6B6B;       
    
    
    pub const BPF_: u32 = 0xFFC42B1C;         
    pub const DIV_: u32 = 0xFFA31818;       
    pub const DKO_: u32 = 0xFF404040;       
    
    
    pub const EJZ_: u32 = 0x40000000;      
    pub const EKA_: u32 = 0x30000000;          
    
    
    pub const DBD_: u32 = 0xF0202020;          
    pub const EKZ_: u32 = 0xFF383838;       
    pub const EKY_: u32 = 0xFF0078D4;      
}






pub fn draw_rounded_rect(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w < radius * 2 || h < radius * 2 {
        
        fill_rect(x, y, w, h, color);
        return;
    }
    
    let r = radius as i32;
    let w = w as i32;
    let h = h as i32;
    
    
    fill_rect(x + r, y, (w - r * 2) as u32, h as u32, color);
    fill_rect(x, y + r, r as u32, (h - r * 2) as u32, color);
    fill_rect(x + w - r, y + r, r as u32, (h - r * 2) as u32, color);
    
    
    ekl(x + r, y + r, radius, Corner::TopLeft, color);
    ekl(x + w - r - 1, y + r, radius, Corner::TopRight, color);
    ekl(x + r, y + h - r - 1, radius, Corner::BottomLeft, color);
    ekl(x + w - r - 1, y + h - r - 1, radius, Corner::BottomRight, color);
}


pub fn iu(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if radius == 0 {
        if x >= 0 && y >= 0 {
            framebuffer::draw_rect(x as u32, y as u32, w, h, color);
        }
        return;
    }
    
    let r = radius as i32;
    let ld = w as i32;
    let hi = h as i32;
    
    
    mn(x + r, y, (ld - r * 2) as u32, color);
    mn(x + r, y + hi - 1, (ld - r * 2) as u32, color);
    
    
    zv(x, y + r, (hi - r * 2) as u32, color);
    zv(x + ld - 1, y + r, (hi - r * 2) as u32, color);
    
    
    ekk(x + r, y + r, radius, Corner::TopLeft, color);
    ekk(x + ld - r - 1, y + r, radius, Corner::TopRight, color);
    ekk(x + r, y + hi - r - 1, radius, Corner::BottomLeft, color);
    ekk(x + ld - r - 1, y + hi - r - 1, radius, Corner::BottomRight, color);
}

#[derive(Clone, Copy)]
enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}


fn ekl(cx: i32, u: i32, radius: u32, corner: Corner, color: u32) {
    let r = radius as i32;
    
    for ad in 0..=r {
        for dx in 0..=r {
            
            if dx * dx + ad * ad <= r * r {
                let (p, o) = match corner {
                    Corner::TopLeft => (cx - dx, u - ad),
                    Corner::TopRight => (cx + dx, u - ad),
                    Corner::BottomLeft => (cx - dx, u + ad),
                    Corner::BottomRight => (cx + dx, u + ad),
                };
                draw_pixel(p, o, color);
            }
        }
    }
}


fn ekk(cx: i32, u: i32, radius: u32, corner: Corner, color: u32) {
    let r = radius as i32;
    let mut x = 0;
    let mut y = r;
    let mut d = 3 - 2 * r;
    
    while x <= y {
        let points = match corner {
            Corner::TopLeft => [(cx - x, u - y), (cx - y, u - x)],
            Corner::TopRight => [(cx + x, u - y), (cx + y, u - x)],
            Corner::BottomLeft => [(cx - x, u + y), (cx - y, u + x)],
            Corner::BottomRight => [(cx + x, u + y), (cx + y, u + x)],
        };
        
        for (p, o) in points {
            draw_pixel(p, o, color);
        }
        
        if d < 0 {
            d += 4 * x + 6;
        } else {
            d += 4 * (x - y) + 10;
            y -= 1;
        }
        x += 1;
    }
}





#[inline]
fn draw_pixel(x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::draw_pixel(x as u32, y as u32, color);
    }
}

#[inline]
fn fill_rect(x: i32, y: i32, w: u32, h: u32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::fill_rect(x as u32, y as u32, w, h, color);
    }
}

#[inline]
fn mn(x: i32, y: i32, len: u32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::mn(x as u32, y as u32, len, color);
    }
}

#[inline]
fn zv(x: i32, y: i32, len: u32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::zv(x as u32, y as u32, len, color);
    }
}
