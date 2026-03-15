








use crate::framebuffer;






pub mod colors {
    
    pub const DUA_: u32 = 0xFF202020;           
    pub const DUB_: u32 = 0xFF1A1A1A;         
    pub const DCD_: u32 = 0xE0282828;          
    pub const Kw: u32 = 0xFF2D2D2D;             
    pub const JA_: u32 = 0xFF383838;       
    pub const PU_: u32 = 0xFF404040;    
    
    
    pub const EIB_: u32 = 0xFF1F1F1F;    
    pub const EIC_: u32 = 0xFF2B2B2B;  
    
    
    pub const Ge: u32 = 0xFF0078D4;              
    pub const DCA_: u32 = 0xFF60CDFF;        
    pub const DBX_: u32 = 0xFF005A9E;         
    
    
    pub const AC_: u32 = 0xFFFFFFFF;        
    pub const N_: u32 = 0xFFB3B3B3;      
    pub const PZ_: u32 = 0xFF6E6E6E;       
    
    
    pub const ZO_: u32 = 0xFF3D3D3D;       
    pub const DDS_: u32 = 0xFF4D4D4D;      
    pub const DDT_: u32 = 0xFF6B6B6B;       
    
    
    pub const BMN_: u32 = 0xFFC42B1C;         
    pub const DFC_: u32 = 0xFFA31818;       
    pub const DGV_: u32 = 0xFF404040;       
    
    
    pub const EGG_: u32 = 0x40000000;      
    pub const EGH_: u32 = 0x30000000;          
    
    
    pub const CXL_: u32 = 0xF0202020;          
    pub const EHI_: u32 = 0xFF383838;       
    pub const EHH_: u32 = 0xFF0078D4;      
}






pub fn mf(b: i32, c: i32, d: u32, i: u32, dy: u32, s: u32) {
    if d < dy * 2 || i < dy * 2 {
        
        ah(b, c, d, i, s);
        return;
    }
    
    let m = dy as i32;
    let d = d as i32;
    let i = i as i32;
    
    
    ah(b + m, c, (d - m * 2) as u32, i as u32, s);
    ah(b, c + m, m as u32, (i - m * 2) as u32, s);
    ah(b + d - m, c + m, m as u32, (i - m * 2) as u32, s);
    
    
    irr(b + m, c + m, dy, Corner::Dp, s);
    irr(b + d - m - 1, c + m, dy, Corner::Dq, s);
    irr(b + m, c + i - m - 1, dy, Corner::Dt, s);
    irr(b + d - m - 1, c + i - m - 1, dy, Corner::Du, s);
}


pub fn tf(b: i32, c: i32, d: u32, i: u32, dy: u32, s: u32) {
    if dy == 0 {
        if b >= 0 && c >= 0 {
            framebuffer::lx(b as u32, c as u32, d, i, s);
        }
        return;
    }
    
    let m = dy as i32;
    let yi = d as i32;
    let gd = i as i32;
    
    
    zs(b + m, c, (yi - m * 2) as u32, s);
    zs(b + m, c + gd - 1, (yi - m * 2) as u32, s);
    
    
    axt(b, c + m, (gd - m * 2) as u32, s);
    axt(b + yi - 1, c + m, (gd - m * 2) as u32, s);
    
    
    irq(b + m, c + m, dy, Corner::Dp, s);
    irq(b + yi - m - 1, c + m, dy, Corner::Dq, s);
    irq(b + m, c + gd - m - 1, dy, Corner::Dt, s);
    irq(b + yi - m - 1, c + gd - m - 1, dy, Corner::Du, s);
}

#[derive(Clone, Copy)]
enum Corner {
    Dp,
    Dq,
    Dt,
    Du,
}


fn irr(cx: i32, ae: i32, dy: u32, hea: Corner, s: u32) {
    let m = dy as i32;
    
    for bg in 0..=m {
        for dx in 0..=m {
            
            if dx * dx + bg * bg <= m * m {
                let (y, x) = match hea {
                    Corner::Dp => (cx - dx, ae - bg),
                    Corner::Dq => (cx + dx, ae - bg),
                    Corner::Dt => (cx - dx, ae + bg),
                    Corner::Du => (cx + dx, ae + bg),
                };
                draw_pixel(y, x, s);
            }
        }
    }
}


fn irq(cx: i32, ae: i32, dy: u32, hea: Corner, s: u32) {
    let m = dy as i32;
    let mut b = 0;
    let mut c = m;
    let mut bc = 3 - 2 * m;
    
    while b <= c {
        let egw = match hea {
            Corner::Dp => [(cx - b, ae - c), (cx - c, ae - b)],
            Corner::Dq => [(cx + b, ae - c), (cx + c, ae - b)],
            Corner::Dt => [(cx - b, ae + c), (cx - c, ae + b)],
            Corner::Du => [(cx + b, ae + c), (cx + c, ae + b)],
        };
        
        for (y, x) in egw {
            draw_pixel(y, x, s);
        }
        
        if bc < 0 {
            bc += 4 * b + 6;
        } else {
            bc += 4 * (b - c) + 10;
            c -= 1;
        }
        b += 1;
    }
}





#[inline]
fn draw_pixel(b: i32, c: i32, s: u32) {
    if b >= 0 && c >= 0 {
        framebuffer::draw_pixel(b as u32, c as u32, s);
    }
}

#[inline]
fn ah(b: i32, c: i32, d: u32, i: u32, s: u32) {
    if b >= 0 && c >= 0 {
        framebuffer::ah(b as u32, c as u32, d, i, s);
    }
}

#[inline]
fn zs(b: i32, c: i32, len: u32, s: u32) {
    if b >= 0 && c >= 0 {
        framebuffer::zs(b as u32, c as u32, len, s);
    }
}

#[inline]
fn axt(b: i32, c: i32, len: u32, s: u32) {
    if b >= 0 && c >= 0 {
        framebuffer::axt(b as u32, c as u32, len, s);
    }
}
