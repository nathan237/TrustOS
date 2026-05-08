






























use core::sync::atomic::{AtomicU32, Ordering};






static BHC_: AtomicU32 = AtomicU32::new(1);


const FO_: u32 = 8;
const IE_: u32 = 16;









pub fn opk(ha: u32) {
    let bqy = ha.clamp(1, 3);
    BHC_.store(bqy, Ordering::SeqCst);
    crate::serial_println!("[Scaling] Scale factor set to {}x", bqy);
}


#[inline]
pub fn aqv() -> u32 {
    BHC_.load(Ordering::Relaxed)
}







pub fn jyi(fb_width: u32, fb_height: u32) -> u32 {
    let ha = if fb_width >= 3840 {
        3
    } else if fb_width >= 2560 {
        2
    } else {
        1
    };
    crate::serial_println!(
        "[Scaling] Auto-detected {}x scale for {}x{} framebuffer",
        ha, fb_width, fb_height
    );
    ha
}














pub fn jyj(fb_width: u32, _fb_height: u32) -> (u32, u32) {
    if fb_width >= 3840 {
        (2, 1) 
    } else if fb_width >= 2560 {
        (3, 2) 
    } else {
        (1, 1) 
    }
}


static BKD_: AtomicU32 = AtomicU32::new(1);

static BKC_: AtomicU32 = AtomicU32::new(1);


#[inline]
pub fn ddt(value: u32) -> u32 {
    let ae = BKD_.load(Ordering::Relaxed);
    let d = BKC_.load(Ordering::Relaxed);
    (value * ae + d / 2) / d 
}


pub fn mpq(fb_width: u32, fb_height: u32) {
    let (ae, d) = jyj(fb_width, fb_height);
    BKD_.store(ae, Ordering::SeqCst);
    BKC_.store(d, Ordering::SeqCst);
    crate::serial_println!("[Scaling] UI chrome scale: {}x/{} for {}x{}", ae, d, fb_width, fb_height);
}




pub fn init(fb_width: u32, fb_height: u32) {
    let ha = jyi(fb_width, fb_height);
    opk(ha);
}






#[inline]
pub fn scale(value: u32) -> u32 {
    value * aqv()
}


#[inline]
pub fn qut(value: i32) -> i32 {
    value * aqv() as i32
}


#[inline]
pub fn rbm(physical: u32) -> u32 {
    let f = aqv();
    if f == 0 { physical } else { physical / f }
}


#[inline]
pub fn rbn(physical: i32) -> i32 {
    let f = aqv() as i32;
    if f == 0 { physical } else { physical / f }
}


#[inline]
pub fn agg() -> u32 {
    FO_ * aqv()
}


#[inline]
pub fn cgu() -> u32 {
    IE_ * aqv()
}








#[derive(Clone, Copy, Debug)]
pub struct Uy {
    pub ha: u32,
    pub taskbar_height: u32,
    pub title_bar_height: u32,
    pub window_border_radius: u32,
    pub window_shadow_blur: u32,
    pub bbe: u32,
    pub dock_width: u32,
    pub agg: u32,
    pub cgu: u32,
}

impl Uy {
    
    const SG_: u32 = 40;
    const SH_: u32 = 28;
    const SI_: u32 = 6;
    const SJ_: u32 = 12;
    const SE_: u32 = 24;
    const SF_: u32 = 60;

    
    pub fn current() -> Self {
        let f = aqv();
        Uy {
            ha: f,
            taskbar_height: Self::SG_ * f,
            title_bar_height: Self::SH_ * f,
            window_border_radius: Self::SI_ * f,
            window_shadow_blur: Self::SJ_ * f,
            bbe: Self::SE_ * f,
            dock_width: Self::SF_ * f,
            agg: FO_ * f,
            cgu: IE_ * f,
        }
    }

    
    pub fn rcp(f: u32) -> Self {
        let f = f.clamp(1, 3);
        Uy {
            ha: f,
            taskbar_height: Self::SG_ * f,
            title_bar_height: Self::SH_ * f,
            window_border_radius: Self::SI_ * f,
            window_shadow_blur: Self::SJ_ * f,
            bbe: Self::SE_ * f,
            dock_width: Self::SF_ * f,
            agg: FO_ * f,
            cgu: IE_ * f,
        }
    }
}









pub fn fta(x: u32, y: u32, c: char, color: u32) {
    let ha = aqv();

    
    if ha == 1 {
        crate::framebuffer::px(x, y, c, color);
        return;
    }

    let du = crate::framebuffer::font::ol(c);
    let fb_width = crate::framebuffer::width();
    let fb_height = crate::framebuffer::height();

    
    let aaj = FO_ * ha;
    let sn = IE_ * ha;
    if x >= fb_width || y >= fb_height {
        return;
    }

    
    let duc = fb_width.min(x + aaj);
    let dud = fb_height.min(y + sn);

    for row in 0..IE_ as usize {
        let bits = du[row];
        if bits == 0 {
            continue; 
        }
        let diq = y + (row as u32) * ha;
        if diq >= dud {
            break;
        }

        for col in 0..FO_ as usize {
            if (bits >> (7 - col)) & 1 == 1 {
                let dip = x + (col as u32) * ha;
                if dip >= duc {
                    break;
                }

                
                for ak in 0..ha {
                    let o = diq + ak;
                    if o >= dud {
                        break;
                    }
                    for am in 0..ha {
                        let p = dip + am;
                        if p < duc {
                            crate::framebuffer::put_pixel(p, o, color);
                        }
                    }
                }
            }
        }
    }
}




pub fn ekr(x: i32, y: i32, text: &str, color: u32) {
    let aq = agg() as i32;
    let fb_w = crate::framebuffer::width() as i32;
    let fb_h = crate::framebuffer::height() as i32;

    if y < 0 || y >= fb_h {
        return;
    }

    for (i, c) in text.chars().enumerate() {
        let p = x + (i as i32) * aq;
        if p >= fb_w {
            break; 
        }
        if p + aq <= 0 {
            continue; 
        }
        if p >= 0 {
            fta(p as u32, y as u32, c, color);
        }
    }
}




pub fn aat(x: i32, y: i32, text: &str, color: u32, ha: u32) {
    let ha = ha.clamp(1, 3);
    let aq = (FO_ * ha) as i32;
    let fb_w = crate::framebuffer::width() as i32;
    let fb_h = crate::framebuffer::height() as i32;

    if y < 0 || y >= fb_h {
        return;
    }

    for (i, c) in text.chars().enumerate() {
        let p = x + (i as i32) * aq;
        if p >= fb_w {
            break;
        }
        if p + aq <= 0 {
            continue;
        }
        if p >= 0 {
            lig(p as u32, y as u32, c, color, ha);
        }
    }
}


fn lig(x: u32, y: u32, c: char, color: u32, ha: u32) {
    if ha == 1 {
        crate::framebuffer::px(x, y, c, color);
        return;
    }

    let du = crate::framebuffer::font::ol(c);
    let fb_width = crate::framebuffer::width();
    let fb_height = crate::framebuffer::height();

    let duc = fb_width.min(x + FO_ * ha);
    let dud = fb_height.min(y + IE_ * ha);

    for row in 0..IE_ as usize {
        let bits = du[row];
        if bits == 0 {
            continue;
        }
        let diq = y + (row as u32) * ha;
        if diq >= dud {
            break;
        }

        for col in 0..FO_ as usize {
            if (bits >> (7 - col)) & 1 == 1 {
                let dip = x + (col as u32) * ha;
                if dip >= duc {
                    break;
                }
                for ak in 0..ha {
                    let o = diq + ak;
                    if o >= dud {
                        break;
                    }
                    for am in 0..ha {
                        let p = dip + am;
                        if p < duc {
                            crate::framebuffer::put_pixel(p, o, color);
                        }
                    }
                }
            }
        }
    }
}








pub fn qfr(x: i32, y: i32, logical_w: u32, logical_h: u32, color: u32) {
    let f = aqv();
    let wl = logical_w * f;
    let qc = logical_h * f;

    if x >= 0 && y >= 0 {
        crate::framebuffer::fill_rect(x as u32, y as u32, wl, qc, color);
    }
}


#[inline]
pub fn quu(x: i32, y: i32, w: u32, h: u32) -> (i32, i32, u32, u32) {
    let f = aqv();
    (
        x * f as i32,
        y * f as i32,
        w * f,
        h * f,
    )
}









pub fn qdy(
    cursor_x: i32,
    cursor_y: i32,
    pattern: &[[u8; 12]],
    bob: u32,
    bso: u32,
) {
    let ha = aqv();
    let fb_w = crate::framebuffer::width();
    let fb_h = crate::framebuffer::height();

    for (u, row) in pattern.iter().enumerate() {
        for (cx, &ct) in row.iter().enumerate() {
            if ct == 0 {
                continue;
            }
            let color = match ct {
                1 => bob,
                2 => bso,
                _ => continue,
            };

            
            for ak in 0..ha {
                for am in 0..ha {
                    let p = cursor_x + (cx as u32 * ha + am) as i32;
                    let o = cursor_y + (u as u32 * ha + ak) as i32;
                    if p >= 0 && o >= 0 && (p as u32) < fb_w && (o as u32) < fb_h {
                        crate::framebuffer::put_pixel(p as u32, o as u32, color);
                    }
                }
            }
        }
    }
}






#[inline]
pub fn auh(text: &str) -> u32 {
    text.len() as u32 * agg()
}


#[inline]
pub fn qox() -> u32 {
    cgu()
}


#[inline]
pub fn qoy(text: &str, ha: u32) -> u32 {
    text.len() as u32 * FO_ * ha.clamp(1, 3)
}
