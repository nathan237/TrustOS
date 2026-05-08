



























use alloc::string::String;
use alloc::vec::Vec;

use crate::framebuffer;





const SM_: u32 = 0xFF050606;
const DW_: u32 = 0xFF070B09;
const EOM_: u32 = 0xFF0A0F0C;
const EOL_: u32 = 0xFF0D1310;

const I_: u32 = 0xFF00FF66;
const AH_: u32 = 0xFF00CC55;
const BM_: u32 = 0xFF00AA44;
const Y_: u32 = 0xFF008844;
const BJ_: u32 = 0xFF006633;
const Q_: u32 = 0xFF003B1A;

const EP_: u32 = 0xFFB0B2B0;
const GR_: u32 = 0xFF8C8E8C;
const AW_: u32 = 0xFF606260;
const AP_: u32 = 0xFF3A3C3A;

const EOJ_: u32 = 0xFFFFD166;
const DJ_: u32 = 0xFFFF5555;
const EOK_: u32 = 0xFF4ECDC4;

const AB_: u32 = 0xFFE0E8E4;
const O_: u32 = 0xFF8A9890;
const EOY_: u32 = 0xFF00CC55;






pub const DVQ_: u32 = 1179;
pub const DVN_: u32 = 2556;

pub const DVP_: u32 = 393;
pub const DVO_: u32 = 852;

pub const CFX_: u32 = 195;
pub const CFW_: u32 = 90;




pub const DDZ_: u32 = 498;
pub const DDY_: u32 = 1080;





const AKB_: u32 = 44;
const GY_: u32 = 90;
const IW_: u32 = 5;
const BZX_: u32 = 134;


const EX_: u32 = 3;
const BX_: u32 = 72;
const OP_: u32 = 18;
const CEQ_: u32 = 18;
const VJ_: u32 = BX_ + CEQ_ + 24; 
const UY_: u32 = 20; 


const IP_: usize = 5;
const BV_: u32 = 52;
const ACT_: u32 = 20;
const TZ_: u32 = 12;


pub const ID_: u32 = 36;


const ABJ_: u32 = 20;


const BIV_: u32 = 16;
const BIW_: u32 = 14;





#[derive(Clone, Copy)]
pub struct Dq {
    pub name: &'static str,
    pub icon_idx: u8,
    pub accent: u32,
}


const JF_: &[Dq] = &[
    Dq { name: "Terminal",  icon_idx: 0,  accent: 0xFF20CC60 },
    Dq { name: "Files",     icon_idx: 1,  accent: 0xFFDDAA30 },
    Dq { name: "Editor",    icon_idx: 2,  accent: 0xFF5090E0 },
    Dq { name: "Calc",      icon_idx: 3,  accent: 0xFFCC6633 },
    Dq { name: "Network",   icon_idx: 4,  accent: 0xFF40AADD },
    Dq { name: "Games",     icon_idx: 5,  accent: 0xFFCC4444 },
    Dq { name: "Browser",   icon_idx: 6,  accent: 0xFF4488DD },
    Dq { name: "TrustEd",   icon_idx: 7,  accent: 0xFF9060D0 },
    Dq { name: "Settings",  icon_idx: 8,  accent: 0xFF9988BB },
    Dq { name: "About",     icon_idx: 9,  accent: 0xFF40CC80 },
    Dq { name: "Music",     icon_idx: 10, accent: 0xFFFF6090 },
    Dq { name: "Chess",     icon_idx: 11, accent: 0xFFD4A854 },
];


const ASP_: [usize; IP_] = [0, 1, 6, 10, 8];





#[derive(Clone, Copy, PartialEq)]
pub enum MobileView {
    Home,
    AppFullscreen,
    AppSwitcher,
    ControlCenter,
}

#[derive(Clone)]
pub struct MobileState {
    pub active: bool,
    pub view: MobileView,
    pub active_app_id: Option<u32>,
    pub home_scroll_x: i32,
    pub home_page: i32,
    pub switcher_scroll_x: i32,
    pub cc_progress: u8,
    pub notif_progress: u8,
    pub gesture_start_x: i32,
    pub gesture_start_y: i32,
    pub gesture_active: bool,
    pub gesture_from_bottom: bool,
    pub gesture_from_top: bool,
    pub anim_frame: u64,
    pub highlighted_icon: i32,
    pub ezu: String,
    pub closing_cards: Vec<(u32, u8)>,
    pub time_str: String,
    
    pub vp_x: i32,
    pub vp_y: i32,
    pub vp_w: u32,
    pub vp_h: u32,
    
    pub music_dropdown_open: bool,
    
    pub music_viz_mode: u8,
    
    pub term_lines: Vec<String>,
    
    pub term_input: String,
    
    pub calc_display: String,
    
    pub calc_op: u8,
    
    pub calc_operand: i64,
    
    pub calc_fresh: bool,
    
    pub files_selected: i32,
    
    pub files_depth: u8,
    
    pub settings_selected: i32,
    
    pub settings_toggles: [bool; 6],
    
    pub games_selected: i32,
    
    pub browser_page: u8,
    
    pub editor_cursor_line: u32,
    
    pub editor_tab: u8,
    
    pub chess_selected: i32,
    
    pub chess_turn: u8,
}

impl MobileState {
    pub const fn new() -> Self {
        Self {
            active: false,
            view: MobileView::Home,
            active_app_id: None,
            home_scroll_x: 0,
            home_page: 0,
            switcher_scroll_x: 0,
            cc_progress: 0,
            notif_progress: 0,
            gesture_start_x: 0,
            gesture_start_y: 0,
            gesture_active: false,
            gesture_from_bottom: false,
            gesture_from_top: false,
            anim_frame: 0,
            highlighted_icon: -1,
            ezu: String::new(),
            closing_cards: Vec::new(),
            time_str: String::new(),
            vp_x: 0,
            vp_y: 0,
            vp_w: DDZ_,
            vp_h: DDY_,
            music_dropdown_open: false,
            music_viz_mode: 0,
            term_lines: Vec::new(),
            term_input: String::new(),
            calc_display: String::new(),
            calc_op: 0,
            calc_operand: 0,
            calc_fresh: false,
            files_selected: -1,
            files_depth: 0,
            settings_selected: -1,
            settings_toggles: [true, false, false, false, true, true],
            games_selected: -1,
            browser_page: 0,
            editor_cursor_line: 0,
            editor_tab: 0,
            chess_selected: -1,
            chess_turn: 0,
        }
    }
}



pub fn hjt(screen_w: u32, screen_h: u32) -> (i32, i32, u32, u32) {
    
    let vp_h = screen_h;
    let vp_w = (screen_h * CFW_ / CFX_).min(screen_w);
    let vx = ((screen_w.saturating_sub(vp_w)) / 2) as i32;
    let vy = 0i32;
    (vx, vy, vp_w, vp_h)
}





fn draw_text(x: i32, y: i32, text: &str, color: u32) {
    crate::graphics::scaling::ekr(x, y, text, color);
}

fn draw_text_centered(cx: i32, y: i32, text: &str, color: u32) {
    let w = crate::graphics::scaling::auh(text) as i32;
    draw_text(cx - w / 2, y, text, color);
}

fn agg() -> u32 {
    crate::graphics::scaling::agg()
}


fn draw_rounded_rect(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);
    if r == 0 {
        if x >= 0 && y >= 0 { framebuffer::fill_rect(x as u32, y as u32, w, h, color); }
        return;
    }
    let ld = w as i32;
    let hi = h as i32;
    let dk = r as i32;
    
    cjl(x, y + dk, ld, hi - dk * 2, color);
    cjl(x + dk, y, ld - dk * 2, dk, color);
    cjl(x + dk, y + hi - dk, ld - dk * 2, dk, color);
    
    let ju = dk * dk;
    for ad in 0..dk {
        let dx = emg(ju - ad * ad);
        cjl(x + dk - dx, y + dk - ad - 1, dx, 1, color);
        cjl(x + ld - dk, y + dk - ad - 1, dx, 1, color);
        cjl(x + dk - dx, y + hi - dk + ad, dx, 1, color);
        cjl(x + ld - dk, y + hi - dk + ad, dx, 1, color);
    }
}


fn iu(x: i32, y: i32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    let r = radius.min(w / 2).min(h / 2);
    let ld = w as i32;
    let hi = h as i32;
    let dk = r as i32;
    if r == 0 {
        if x >= 0 && y >= 0 { framebuffer::draw_rect(x as u32, y as u32, w, h, color); }
        return;
    }
    
    for p in (x + dk)..(x + ld - dk) {
        aag(p, y, color);
        aag(p, y + hi - 1, color);
    }
    for o in (y + dk)..(y + hi - dk) {
        aag(x, o, color);
        aag(x + ld - 1, o, color);
    }
    
    let ju = dk * dk;
    let mut esj = dk;
    for ad in 0..=dk {
        let dx = emg(ju - ad * ad);
        
        for ax in dx..=esj {
            
            aag(x + dk - ax, y + dk - ad, color);
            
            aag(x + ld - 1 - dk + ax, y + dk - ad, color);
            
            aag(x + dk - ax, y + hi - 1 - dk + ad, color);
            
            aag(x + ld - 1 - dk + ax, y + hi - 1 - dk + ad, color);
        }
        esj = dx;
    }
}


fn cjl(x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 || x + w <= 0 || y + h <= 0 { return; }
    let bm = x.max(0) as u32;
    let az = y.max(0) as u32;
    let x1 = (x + w).max(0) as u32;
    let y1 = (y + h).max(0) as u32;
    if x1 > bm && y1 > az {
        framebuffer::fill_rect(bm, az, x1 - bm, y1 - az, color);
    }
}

fn aag(x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        framebuffer::put_pixel(x as u32, y as u32, color);
    }
}

fn emg(v: i32) -> i32 {
    if v <= 0 { return 0; }
    let mut x = v;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + v / x) / 2; }
    x
}


fn jxw(y: i32, x: i32) -> u32 {
    let ax = if x < 0 { -x } else { x };
    let aet = if y < 0 { -y } else { y };
    
    let amb = if x >= 0 {
        if y >= 0 {
            if ax >= aet { 0 } else { 1 }
        } else {
            if ax >= aet { 7 } else { 6 }
        }
    } else {
        if y >= 0 {
            if ax >= aet { 3 } else { 2 }
        } else {
            if ax >= aet { 4 } else { 5 }
        }
    };
    amb
}






fn ekn(x: i32, y: i32, w: u32, h: u32, _radius: u32, opacity: u32) {
    if w == 0 || h == 0 { return; }
    let hdb = x.max(0) as u32;
    let hdf = y.max(0) as u32;
    
    framebuffer::co(hdb, hdf, w, h, 0x000000, (opacity * 7 / 10).min(255));
    
    framebuffer::co(hdb, hdf, w, h, 0x001A0A, (opacity * 2 / 10).min(255));
    
    let guv = h / 4;
    for row in 0..guv {
        let alpha = ((guv - row) * 18 / guv).min(255);
        if alpha > 0 {
            framebuffer::co(hdb, hdf + row, w, 1, 0xFFFFFF, alpha);
        }
    }
    
    iu(x, y, w, h, _radius, AP_);
}





pub fn draw_status_bar(vx: i32, vy: i32, bt: u32, _vh: u32, time_str: &str, _frame: u64) {
    let x = vx;
    let y = vy;
    let w = bt;
    let h = AKB_;

    
    framebuffer::co(x.max(0) as u32, y.max(0) as u32, w, h, 0x040A06, 180);
    framebuffer::co(x.max(0) as u32, y.max(0) as u32, w, h, 0x00AA44, 8);

    
    framebuffer::fill_rect(x.max(0) as u32, (y + h as i32 - 1).max(0) as u32, w, 1, AP_);

    
    let aq = agg();
    draw_text_centered(x + w as i32 / 2, y + 14, time_str, EP_);

    
    draw_text(x + 12, y + 14, "TrustOS", AH_);

    
    let da = x + w as i32 - 12;
    
    let bx = (da - 24).max(0) as u32;
    let dc = (y + 16).max(0) as u32;
    framebuffer::draw_rect(bx, dc, 18, 10, AW_);
    framebuffer::fill_rect(bx + 18, dc + 3, 2, 4, AW_);
    framebuffer::fill_rect(bx + 2, dc + 2, 14, 6, AH_); 

    
    let wx = (da - 50) as i32;
    let wy = y + 24;
    for dq in 0..3u32 {
        let r = 2 + dq * 3;
        let ju = (r * r) as i32;
        let dk = r.saturating_sub(1);
        let gpi = (dk * dk) as i32;
        for ad in 0..=r as i32 {
            for dx in -(r as i32)..=(r as i32) {
                let jq = dx * dx + ad * ad;
                if jq <= ju && jq >= gpi && ad <= 0 {
                    let col = if dq == 0 { AH_ } else { Y_ };
                    aag(wx + dx, wy + ad, col);
                }
            }
        }
    }
    aag(wx, wy + 1, AH_); 
}





pub fn htk(
    vx: i32, vy: i32, bt: u32, ex: u32,
    highlighted: i32, _frame: u64,
) {
    let ae = JF_.len() as u32;
    let rows = (ae + EX_ - 1) / EX_;

    
    
    const AHN_: u32 = 138;
    let eiv = vy + AKB_ as i32;
    let eiu = vy + ex as i32 - GY_ as i32 - IW_ as i32 - AHN_ as i32;
    let en = (eiu - eiv).max(0) as u32;

    
    let cqd = 36u32;
    let jdu = 14u32;
    let acm = bt.saturating_sub(jdu * 2);
    let ava = vx + jdu as i32;
    let agz = eiv + 10;
    
    draw_rounded_rect(ava, agz, acm, cqd, 12, 0xFF0A120E);
    iu(ava, agz, acm, cqd, 12, AP_);
    
    let cmn = ava + 12;
    let cmo = agz + 8;
    
    for ad in -4i32..=4 {
        for dx in -4i32..=4 {
            let jq = dx * dx + ad * ad;
            if jq >= 9 && jq <= 16 {
                aag(cmn + dx, cmo + ad, AW_);
            }
        }
    }
    
    aag(cmn + 4, cmo + 4, AW_);
    aag(cmn + 5, cmo + 5, AW_);
    aag(cmn + 6, cmo + 6, AW_);
    
    draw_text(ava + 26, agz + 10, "Search", O_);

    
    let dra = agz + cqd as i32 + 10;
    let cah = (eiu - dra).max(0) as u32;

    
    let fzm = bt.saturating_sub(UY_ * 2);
    
    let cell_w = fzm / EX_; 
    let fzp = vx + UY_ as i32;

    let gzp = rows * VJ_;
    let bmq = dra + (cah as i32 - gzp as i32) / 2;

    for i in 0..ae {
        let col = i % EX_;
        let row = i / EX_;
        let afz = &JF_[i as usize];

        
        let bi = fzp + (col * cell_w) as i32 + (cell_w as i32 - BX_ as i32) / 2;
        let gg = bmq + (row * VJ_) as i32;
        let erm = highlighted == i as i32;

        
        
        draw_rounded_rect(bi, gg, BX_, BX_, OP_, 0xFF060A06);
        if erm {
            iu(bi, gg, BX_, BX_, OP_, afz.accent);
            
            iu(bi - 1, gg - 1, BX_ + 2, BX_ + 2, OP_ + 1, Q_);
        } else {
            iu(bi, gg, BX_, BX_, OP_, AP_);
        }

        
        let oy = if erm { afz.accent } else { BJ_ };
        let cx = bi + BX_ as i32 / 2;
        let u = gg + BX_ as i32 / 2;
        htl(cx, u, afz.icon_idx, oy, erm);

        
        let ace = if erm { I_ } else { O_ };
        let fe = bi + BX_ as i32 / 2;
        let ly = gg + BX_ as i32 + 2;
        draw_text_centered(fe, ly, afz.name, ace);
    }
}





pub fn draw_dock(vx: i32, vy: i32, bt: u32, ex: u32, highlighted_dock: i32, _frame: u64) {
    let byv = vy + ex as i32 - GY_ as i32 - IW_ as i32;
    let bsb = vx + TZ_ as i32;
    let dnl = bt - TZ_ * 2;
    let atn = GY_ - 10; 

    
    ekn(bsb, byv, dnl, atn, ACT_, 200);
    
    framebuffer::co(
        (bsb + ACT_ as i32).max(0) as u32,
        byv.max(0) as u32,
        dnl.saturating_sub(ACT_ * 2), 1,
        AW_, 120,
    );

    
    let aaj = IP_ as u32 * BV_ + (IP_ as u32 - 1) * 12;
    let start_x = bsb + (dnl as i32 - aaj as i32) / 2;
    let adu = byv + (atn as i32 - BV_ as i32) / 2;

    for (di, &awc) in ASP_.iter().enumerate() {
        let afz = &JF_[awc];
        let bi = start_x + (di as u32 * (BV_ + 12)) as i32;
        let clm = highlighted_dock == di as i32;

        
        draw_rounded_rect(bi, adu, BV_, BV_, 12, 0xFF060A06);
        if clm {
            iu(bi, adu, BV_, BV_, 12, afz.accent);
        } else {
            iu(bi, adu, BV_, BV_, 12, AP_);
        }

        
        let lii = if clm { afz.accent } else { BJ_ };
        let cx = bi + BV_ as i32 / 2;
        let u = adu + BV_ as i32 / 2;
        htl(cx, u, afz.icon_idx, lii, clm);

        
        let ace = if clm { I_ } else { O_ };
        let fe = bi + BV_ as i32 / 2;
        let ly = adu + BV_ as i32 + 2;
        draw_text_centered(fe, ly, afz.name, ace);
    }
}





pub fn ekm(vx: i32, vy: i32, bt: u32, ex: u32) {
    let ek = BZX_;
    let hs = IW_;
    let bx = vx + (bt as i32 - ek as i32) / 2;
    let dc = vy + ex as i32 - hs as i32 - 4;
    draw_rounded_rect(bx, dc, ek, hs, 3, EP_);
}





pub fn lhn(vx: i32, vy: i32, bt: u32, title: &str, _frame: u64) {
    let h = ID_;
    
    framebuffer::co(vx.max(0) as u32, vy.max(0) as u32, bt, h, 0x0A1A0A, 220);
    
    framebuffer::fill_rect((vx).max(0) as u32, (vy + h as i32 - 1).max(0) as u32, bt, 1, AW_);
    
    framebuffer::co(vx.max(0) as u32, vy.max(0) as u32, bt, 1, 0x00FF66, 15);
    
    draw_text_centered(vx + bt as i32 / 2, vy + 10, title, AB_);
    
    draw_text(vx + 10, vy + 10, "<", AH_);
}





pub fn lhx(
    vx: i32, vy: i32, bt: u32, ex: u32,
    windows: &[(u32, &str)], 
    scroll_x: i32, _frame: u64,
) {
    
    framebuffer::co(vx.max(0) as u32, vy.max(0) as u32, bt, ex, 0x000000, 180);

    if windows.is_empty() {
        draw_text_centered(vx + bt as i32 / 2, vy + ex as i32 / 2, "No apps open", O_);
        return;
    }

    
    let aji = (bt * 7 / 10).min(400);
    let aev = (ex * 5 / 10).min(600);
    let cuv = vy + (ex as i32 - aev as i32) / 2;

    let plm = windows.len() as u32 * aji + (windows.len() as u32).saturating_sub(1) * BIV_;
    let start_x = vx + (bt as i32 - plm as i32) / 2 - scroll_x;

    for (i, &(sa, title)) in windows.iter().enumerate() {
        let cx = start_x + (i as u32 * (aji + BIV_)) as i32;
        
        draw_rounded_rect(cx, cuv, aji, aev, BIW_, 0xFF0A0A0A);
        iu(cx, cuv, aji, aev, BIW_, AW_);
        
        framebuffer::co(cx.max(0) as u32, cuv.max(0) as u32, aji, 28, 0x0A1A0A, 220);
        draw_text(cx + 10, cuv + 6, title, AB_);
        
        let adl = cx + aji as i32 - 20;
        draw_text(adl, cuv + 6, "X", DJ_);
        
        draw_text_centered(cx + aji as i32 / 2, cuv + aev as i32 / 2, title, O_);
    }
}





pub fn lik(vx: i32, vy: i32, bt: u32, _vh: u32, progress: u8, _frame: u64) {
    if progress == 0 { return; }
    let h = (340u32 * progress as u32 / 100).max(1);
    let w = bt.saturating_sub(24);
    let x = vx + 12;
    let y = vy;

    
    ekn(x, y, w, h, ABJ_, 230);
    
    framebuffer::co(
        (x + ABJ_ as i32).max(0) as u32, y.max(0) as u32,
        w.saturating_sub(ABJ_ * 2), 1, EP_, 60,
    );

    if h < 100 { return; } 

    let cx = x + w as i32 / 2;
    let mut ty = y + 20;

    
    draw_text(x + 16, ty, "Brightness", AB_);
    ty += 20;
    let ek = w - 40;
    framebuffer::fill_rect((x + 20).max(0) as u32, ty.max(0) as u32, ek, 6, AP_);
    framebuffer::fill_rect((x + 20).max(0) as u32, ty.max(0) as u32, ek * 7 / 10, 6, AH_);
    ty += 24;

    
    draw_text(x + 16, ty, "Volume", AB_);
    ty += 20;
    framebuffer::fill_rect((x + 20).max(0) as u32, ty.max(0) as u32, ek, 6, AP_);
    framebuffer::fill_rect((x + 20).max(0) as u32, ty.max(0) as u32, ek * 5 / 10, 6, AH_);
    ty += 24;

    
    let tile_size = 50u32;
    let cek = 10u32;
    let gyt = ["WiFi", "BT", "Air", "DND"];
    let jmn = [true, false, false, false];
    let pmg = gyt.len() as u32 * tile_size + (gyt.len() as u32 - 1) * cek;
    let pjk = x + (w as i32 - pmg as i32) / 2;

    for (i, &label) in gyt.iter().enumerate() {
        let bu = pjk + (i as u32 * (tile_size + cek)) as i32;
        let bg = if jmn[i] { Q_ } else { SM_ };
        let border = if jmn[i] { AH_ } else { AP_ };
        draw_rounded_rect(bu, ty, tile_size, tile_size, 10, bg);
        iu(bu, ty, tile_size, tile_size, 10, border);
        draw_text_centered(bu + tile_size as i32 / 2, ty + tile_size as i32 / 2 - 7, label, AB_);
    }
}





pub fn lkk(vx: i32, vy: i32, bt: u32, ex: u32) {
    
    iu(vx - 3, vy - 3, bt + 6, ex + 6, 18, Q_);
    
    iu(vx - 2, vy - 2, bt + 4, ex + 4, 16, EP_);
    
    iu(vx - 1, vy - 1, bt + 2, ex + 2, 14, AW_);
}





#[derive(Clone, Copy, PartialEq)]
pub enum MobileAction {
    None,
    GoHome,
    OpenSwitcher,
    OpenControlCenter,
    CloseControlCenter,
    LaunchApp(u8),      
    LaunchDockApp(u8),  
    BackFromApp,
    CloseSwitcherCard(u32),
    MusicTogglePlay,
    MusicStop,
    MusicToggleDropdown,
    MusicSetVizMode(u8),
    
    CalcButton(u8),
    
    FilesTap(u8),
    
    FilesBack,
    
    SettingsTap(u8),
    
    GamesTap(u8),
    
    BrowserNav(u8),
    
    EditorTap(u8),
    
    EditorSwitchTab(u8),
    
    ChessTap(u8),
    
    MusicAppToggle,
    
    TermSubmit,
}



pub fn mhv(
    state: &mut MobileState,
    event: GestureEvent,
) -> MobileAction {
    match event {
        GestureEvent::TapDown(x, y) => {
            state.gesture_start_x = x;
            state.gesture_start_y = y;
            state.gesture_active = true;
            state.gesture_from_bottom = y > state.vp_h as i32 - 60;
            state.gesture_from_top = y < 44;
            
            if state.view == MobileView::Home {
                state.highlighted_icon = iew(x, y, state.vp_w, state.vp_h);
            }
            MobileAction::None
        }
        GestureEvent::TapUp(x, y) => {
            state.gesture_active = false;
            let dx = x - state.gesture_start_x;
            let ad = y - state.gesture_start_y;
            let em = emg(dx * dx + ad * ad);

            if em < 15 {
                
                return miq(state, x, y);
            }

            
            if ad.abs() > dx.abs() && ad.abs() > 30 {
                if ad < 0 {
                    
                    if state.gesture_from_bottom {
                        
                        if state.view == MobileView::AppFullscreen {
                            return MobileAction::GoHome;
                        } else if state.view == MobileView::Home {
                            return MobileAction::OpenSwitcher;
                        }
                    }
                    if state.view == MobileView::ControlCenter {
                        return MobileAction::CloseControlCenter;
                    }
                } else {
                    
                    if state.gesture_from_top && state.view == MobileView::Home {
                        return MobileAction::OpenControlCenter;
                    }
                }
            }

            state.highlighted_icon = -1;
            MobileAction::None
        }
        GestureEvent::Move(_x, _y) => {
            MobileAction::None
        }
    }
}

fn miq(state: &mut MobileState, x: i32, y: i32) -> MobileAction {
    match state.view {
        MobileView::Home => {
            
            let ipb = mlt(x, y, state.vp_w, state.vp_h, state.music_dropdown_open);
            if ipb != MobileAction::None {
                return ipb;
            }
            
            let idx = iew(x, y, state.vp_w, state.vp_h);
            if idx >= 0 && (idx as usize) < JF_.len() {
                state.highlighted_icon = -1;
                return MobileAction::LaunchApp(idx as u8);
            }
            
            let htc = mls(x, y, state.vp_w, state.vp_h);
            if htc >= 0 {
                return MobileAction::LaunchDockApp(htc as u8);
            }
            MobileAction::None
        }
        MobileView::AppFullscreen => {
            
            if y < ID_ as i32 && x < 40 {
                return MobileAction::BackFromApp;
            }
            
            if let Some(awc) = state.active_app_id {
                let ta = y - ID_ as i32;
                return mhh(state, awc, x, ta);
            }
            MobileAction::None
        }
        MobileView::AppSwitcher => {
            
            MobileAction::GoHome
        }
        MobileView::ControlCenter => {
            MobileAction::None
        }
    }
}



fn mhh(state: &MobileState, awc: u32, x: i32, y: i32) -> MobileAction {
    let bt = state.vp_w;
    let ex = state.vp_h;
    let pad = 14i32;

    match awc {
        
        3 => {
            let bi = pad;
            let oo = (bt as i32 - pad * 2) as u32;
            let atm = 60i32;
            let hn = 44i32;
            let rj = 6i32;
            let gu = ((oo - 6 * 3) / 4) as i32;
            let icx = 10 + atm + 14;
            
            if y >= icx {
                let row = (y - icx) / (hn + rj);
                let col = (x - bi) / (gu + rj);
                if row >= 0 && row < 5 && col >= 0 && col < 4 {
                    
                    let kej: [[u8; 4]; 5] = [
                        [16, 17, 18, 14], 
                        [7,  8,  9,  13], 
                        [4,  5,  6,  12], 
                        [1,  2,  3,  11], 
                        [0,  10, 15, 255],
                    ];
                    let code = kej[row as usize][col as usize];
                    if code != 255 {
                        return MobileAction::CalcButton(code);
                    }
                }
            }
            MobileAction::None
        }
        
        1 => {
            let ep = 40i32;
            let dpv = 32;
            if y >= dpv {
                let idx = (y - dpv) / ep;
                if idx >= 0 && idx < 8 {
                    return MobileAction::FilesTap(idx as u8);
                }
            }
            
            if y < 28 && x < 80 && state.files_depth > 0 {
                return MobileAction::FilesBack;
            }
            MobileAction::None
        }
        
        8 => {
            let ep = 52i32;
            let dpv = 10;
            if y >= dpv {
                let idx = (y - dpv) / ep;
                if idx >= 0 && idx < 6 {
                    return MobileAction::SettingsTap(idx as u8);
                }
            }
            MobileAction::None
        }
        
        5 => {
            let aev = 56i32;
            let fkx = 8i32;
            let hzc = 34;
            if y >= hzc {
                let idx = (y - hzc) / (aev + fkx);
                if idx >= 0 && idx < 5 {
                    return MobileAction::GamesTap(idx as u8);
                }
            }
            MobileAction::None
        }
        
        6 => {
            let arr = 40;
            
            if y >= 4 && y < 34 {
                return MobileAction::BrowserNav(0);
            }
            if state.browser_page == 0 {
                
                let cbm = arr + 84;
                let ikj = 20;
                if y >= cbm && y < cbm + ikj * 3 {
                    let idx = (y - cbm) / ikj;
                    return MobileAction::BrowserNav(idx as u8 + 1);
                }
            } else {
                
                if y >= arr {
                    return MobileAction::BrowserNav(0);
                }
            }
            MobileAction::None
        }
        
        2 => {
            
            if y < 26 {
                if x < 80 {
                    return MobileAction::EditorSwitchTab(0);
                } else {
                    return MobileAction::EditorSwitchTab(1);
                }
            }
            
            let bw = 16;
            let code_start = 30;
            if y >= code_start {
                let line = (y - code_start) / bw;
                if line >= 0 && line < 12 {
                    return MobileAction::EditorTap(line as u8);
                }
            }
            MobileAction::None
        }
        
        11 => {
            let tg = (bt as i32 - 16).min((ex.saturating_sub(ID_ + 80)) as i32).min(400);
            let cell = tg / 8;
            let un = (bt as i32 - tg) / 2;
            let ve = 10;
            if x >= un && x < un + tg && y >= ve && y < ve + tg {
                let col = (x - un) / cell;
                let row = (y - ve) / cell;
                let cu = (row * 8 + col) as u8;
                return MobileAction::ChessTap(cu);
            }
            MobileAction::None
        }
        
        10 => {
            let oo = (bt as i32 - pad * 2) as u32;
            let anm = oo.min(200);
            
            let aqc = anm as i32 + 16 + 20 + 28 + 20 + 14;
            let gu = 48i32;
            let hn = 36i32;
            let rj = 10i32;
            let aaj = 5 * gu + 4 * rj;
            let fjx = pad + ((oo as i32 - aaj) / 2);
            if y >= aqc && y < aqc + hn {
                for i in 0..5 {
                    let bx = fjx + i * (gu + rj);
                    if x >= bx && x < bx + gu {
                        return MobileAction::MusicAppToggle;
                    }
                }
            }
            MobileAction::None
        }
        
        0 => {
            
            let en = ex.saturating_sub(ID_ + 20);
            if y > en as i32 - 40 {
                return MobileAction::TermSubmit;
            }
            MobileAction::None
        }
        
        _ => MobileAction::None,
    }
}

#[derive(Clone, Copy)]
pub enum GestureEvent {
    TapDown(i32, i32),
    TapUp(i32, i32),
    Move(i32, i32),
}





fn iew(x: i32, y: i32, bt: u32, ex: u32) -> i32 {
    let ae = JF_.len() as u32;
    let rows = (ae + EX_ - 1) / EX_;
    const AHN_: u32 = 138;
    let eiv = AKB_ as i32;
    let eiu = ex as i32 - GY_ as i32 - IW_ as i32 - AHN_ as i32;
    
    let dra = eiv + 56;
    let cah = (eiu - dra).max(0) as u32;

    let fzm = bt.saturating_sub(UY_ * 2);
    let cell_w = fzm / EX_;
    let fzp = UY_ as i32;

    let gzp = rows * VJ_;
    let bmq = dra + (cah as i32 - gzp as i32) / 2;

    for i in 0..ae {
        let col = i % EX_;
        let row = i / EX_;
        let bi = fzp + (col * cell_w) as i32 + (cell_w as i32 - BX_ as i32) / 2;
        let gg = bmq + (row * VJ_) as i32;
        if x >= bi && x < bi + BX_ as i32 && y >= gg && y < gg + BX_ as i32 {
            return i as i32;
        }
    }
    -1
}



fn mlt(x: i32, y: i32, bt: u32, ex: u32, dropdown_open: bool) -> MobileAction {
    const MI_: u32 = 130;
    const RL_: u32 = 14;
    const GZ_: u32 = 26;
    let csu = bt.saturating_sub(RL_ * 2);
    let cfd = RL_ as i32;
    let dvo = crate::visualizer::JJ_ as u32;
    let elx = if dropdown_open { dvo * GZ_ + 8 } else { 0 };
    let cfe = ex as i32 - GY_ as i32 - IW_ as i32 - MI_ as i32 - 8 - elx as i32;

    let sn = MI_ + elx;

    
    if x < cfd || x > cfd + csu as i32 || y < cfe || y > cfe + sn as i32 {
        return MobileAction::None;
    }

    
    let apg = cfe + 12;
    if y >= apg && y < apg + 18 && x > cfd + csu as i32 / 2 {
        return MobileAction::MusicToggleDropdown;
    }

    
    let pad = 14u32;
    let bi = cfd + pad as i32;
    let oo = csu.saturating_sub(pad * 2);
    let aqc = cfe + 92;
    let gu = 40i32;
    let hn = 20i32;
    let rj = 6i32;
    let dut = 5i32;
    let gzm = dut * gu + (dut - 1) * rj;
    let fjy = bi + (oo as i32 - gzm) / 2;

    if y >= aqc && y <= aqc + hn {
        for bal in 0..5 {
            let bx = fjy + bal * (gu + rj);
            if x >= bx && x < bx + gu {
                return MobileAction::MusicTogglePlay;
            }
        }
    }

    
    if dropdown_open {
        let lby = cfe + MI_ as i32 + 4;
        for mi in 0..dvo {
            let ru = lby + (mi * GZ_) as i32;
            if y >= ru && y < ru + GZ_ as i32 {
                return MobileAction::MusicSetVizMode(mi as u8);
            }
        }
    }

    MobileAction::None
}

fn mls(x: i32, y: i32, bt: u32, ex: u32) -> i32 {
    let byv = ex as i32 - GY_ as i32 - IW_ as i32;
    let bsb = TZ_ as i32;
    let dnl = bt - TZ_ * 2;
    let atn = GY_ - 10;

    if y < byv || y > byv + atn as i32 { return -1; }

    let aaj = IP_ as u32 * BV_ + (IP_ as u32 - 1) * 12;
    let start_x = bsb + (dnl as i32 - aaj as i32) / 2;

    for di in 0..IP_ {
        let bi = start_x + (di as u32 * (BV_ + 12)) as i32;
        if x >= bi && x < bi + BV_ as i32 {
            return di as i32;
        }
    }
    -1
}





pub fn pjf(state: &mut MobileState) {
    state.anim_frame = state.anim_frame.wrapping_add(1);

    
    if state.view == MobileView::ControlCenter && state.cc_progress < 100 {
        state.cc_progress = (state.cc_progress + 8).min(100);
    }
    if state.view != MobileView::ControlCenter && state.cc_progress > 0 {
        state.cc_progress = state.cc_progress.saturating_sub(8);
    }

    
    state.closing_cards.retain_mut(|(_id, ln)| {
        *ln = ln.saturating_sub(4);
        *ln > 0
    });
}





fn htl(cx: i32, u: i32, icon_idx: u8, color: u32, _highlighted: bool) {
    match icon_idx {
        0 => { 
            iu(cx - 14, u - 10, 28, 20, 3, color);
            framebuffer::fill_rect((cx - 13).max(0) as u32, (u - 9).max(0) as u32, 26, 3, color);
            
            framebuffer::fill_rect((cx - 11).max(0) as u32, (u - 8).max(0) as u32, 2, 1, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (u - 8).max(0) as u32, 2, 1, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 5).max(0) as u32, (u - 8).max(0) as u32, 2, 1, 0xFF0A0A0A);
            
            draw_text(cx - 8, u - 2, ">", color);
            framebuffer::fill_rect((cx - 2).max(0) as u32, u.max(0) as u32, 8, 2, color);
        }
        1 => { 
            framebuffer::fill_rect((cx - 14).max(0) as u32, (u - 8).max(0) as u32, 12, 5, color);
            framebuffer::fill_rect((cx - 14).max(0) as u32, (u - 3).max(0) as u32, 28, 15, color);
            framebuffer::fill_rect((cx - 12).max(0) as u32, (u - 1).max(0) as u32, 24, 11, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (u + 2).max(0) as u32, 16, 1, 0xFF303020);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (u + 5).max(0) as u32, 12, 1, 0xFF303020);
        }
        2 => { 
            framebuffer::fill_rect((cx - 10).max(0) as u32, (u - 12).max(0) as u32, 20, 24, color);
            framebuffer::fill_rect((cx + 4).max(0) as u32, (u - 12).max(0) as u32, 6, 6, 0xFF0A0A0A);
            framebuffer::fill_rect((cx + 4).max(0) as u32, (u - 12).max(0) as u32, 1, 6, color);
            framebuffer::fill_rect((cx + 4).max(0) as u32, (u - 7).max(0) as u32, 6, 1, color);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (u - 6).max(0) as u32, 16, 16, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (u - 4).max(0) as u32, 6, 1, 0xFF6688CC);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (u - 2).max(0) as u32, 10, 1, color);
            framebuffer::fill_rect((cx - 6).max(0) as u32, u.max(0) as u32, 8, 1, 0xFFCC8844);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (u + 2).max(0) as u32, 12, 1, color);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (u + 4).max(0) as u32, 4, 1, 0xFF88BB44);
            framebuffer::fill_rect((cx - 6).max(0) as u32, (u + 6).max(0) as u32, 9, 1, color);
        }
        3 => { 
            iu(cx - 10, u - 12, 20, 24, 2, color);
            framebuffer::fill_rect((cx - 8).max(0) as u32, (u - 10).max(0) as u32, 16, 6, 0xFF1A3320);
            draw_text(cx - 4, u - 10, "42", 0xFF40FF40);
            for row in 0..3u32 {
                for col in 0..3u32 {
                    let bx = (cx as u32).wrapping_sub(8) + col * 6;
                    let dc = (u as u32).wrapping_sub(1) + row * 5;
                    let ahl = if row == 2 && col == 2 { 0xFFCC6633 } else { color };
                    framebuffer::fill_rect(bx.max(0), dc.max(0), 4, 3, ahl);
                }
            }
        }
        4 => { 
            let jts = cx;
            let jtt = u + 4;
            for dq in 0..3u32 {
                let r = 4 + dq * 4;
                let ju = (r * r) as i32;
                let dk = r.saturating_sub(2);
                let gpi = (dk * dk) as i32;
                for ad in 0..=r as i32 {
                    for dx in -(r as i32)..=(r as i32) {
                        let jq = dx * dx + ad * ad;
                        if jq <= ju && jq >= gpi && ad <= 0 {
                            let ln = if dq == 0 { color } else { Q_ };
                            aag(jts + dx, jtt + ad, ln);
                        }
                    }
                }
            }
            framebuffer::fill_rect((cx - 1).max(0) as u32, (u + 3).max(0) as u32, 3, 3, color);
        }
        5 => { 
            framebuffer::fill_rect((cx - 12).max(0) as u32, (u - 4).max(0) as u32, 24, 12, color);
            framebuffer::fill_rect((cx - 14).max(0) as u32, (u - 2).max(0) as u32, 4, 8, color);
            framebuffer::fill_rect((cx + 10).max(0) as u32, (u - 2).max(0) as u32, 4, 8, color);
            framebuffer::fill_rect((cx - 11).max(0) as u32, (u - 3).max(0) as u32, 22, 10, 0xFF0A0A0A);
            framebuffer::fill_rect((cx - 9).max(0) as u32, (u - 1).max(0) as u32, 5, 1, color);
            framebuffer::fill_rect((cx - 7).max(0) as u32, (u - 3).max(0) as u32, 1, 5, color);
            framebuffer::fill_rect((cx + 5).max(0) as u32, (u - 2).max(0) as u32, 2, 2, 0xFF4488DD);
            framebuffer::fill_rect((cx + 8).max(0) as u32, (u - 1).max(0) as u32, 2, 2, DJ_);
            framebuffer::fill_rect((cx + 5).max(0) as u32, (u + 1).max(0) as u32, 2, 2, 0xFF44DD44);
            framebuffer::fill_rect((cx + 8).max(0) as u32, (u + 2).max(0) as u32, 2, 2, 0xFFDDDD44);
        }
        6 => { 
            for ad in -8i32..=8 {
                for dx in -8i32..=8 {
                    let jq = dx * dx + ad * ad;
                    if jq <= 64 && jq >= 49 {
                        aag(cx + dx, u + ad, color);
                    }
                }
            }
            
            framebuffer::fill_rect((cx - 1).max(0) as u32, (u - 7).max(0) as u32, 2, 14, color);
            framebuffer::fill_rect((cx - 7).max(0) as u32, (u - 1).max(0) as u32, 14, 2, color);
        }
        7 => { 
            
            iu(cx - 8, u - 6, 16, 12, 1, color);
            
            iu(cx - 4, u - 10, 16, 12, 1, Q_);
            
            aag(cx - 8, u - 6, color); aag(cx - 4, u - 10, color);
            aag(cx + 7, u - 6, color); aag(cx + 11, u - 10, color);
        }
        8 => { 
            for ad in 0..18u32 {
                for dx in 0..18u32 {
                    let lh = dx as i32 - 9;
                    let kf = ad as i32 - 9;
                    let bgb = lh * lh + kf * kf;
                    
                    if bgb >= 36 && bgb <= 81 {
                        let cc = jxw(kf, lh);
                        if bgb > 56 {
                            
                            if cc % 2 == 0 { aag(cx - 9 + dx as i32, u - 9 + ad as i32, color); }
                        } else {
                            aag(cx - 9 + dx as i32, u - 9 + ad as i32, color);
                        }
                    }
                    
                    if bgb <= 9 {
                        aag(cx - 9 + dx as i32, u - 9 + ad as i32, 0xFF0A0A0A);
                    }
                }
            }
        }
        9 => { 
            for ad in -8i32..=8 {
                for dx in -8i32..=8 {
                    let jq = dx * dx + ad * ad;
                    if jq <= 64 && jq >= 49 { aag(cx + dx, u + ad, color); }
                }
            }
            draw_text(cx - 2, u - 6, "i", color);
        }
        10 => { 
            
            framebuffer::fill_rect((cx - 3).max(0) as u32, (u + 2).max(0) as u32, 6, 4, color);
            
            framebuffer::fill_rect((cx + 2).max(0) as u32, (u - 8).max(0) as u32, 2, 12, color);
            
            framebuffer::fill_rect((cx + 3).max(0) as u32, (u - 8).max(0) as u32, 4, 2, color);
            framebuffer::fill_rect((cx + 5).max(0) as u32, (u - 6).max(0) as u32, 2, 2, color);
        }
        11 => { 
            
            framebuffer::fill_rect((cx - 1).max(0) as u32, (u - 10).max(0) as u32, 2, 6, color);
            framebuffer::fill_rect((cx - 3).max(0) as u32, (u - 8).max(0) as u32, 6, 2, color);
            
            framebuffer::fill_rect((cx - 4).max(0) as u32, (u - 4).max(0) as u32, 8, 10, color);
            framebuffer::fill_rect((cx - 3).max(0) as u32, (u - 3).max(0) as u32, 6, 8, 0xFF0A0A0A);
            
            framebuffer::fill_rect((cx - 6).max(0) as u32, (u + 5).max(0) as u32, 12, 3, color);
        }
        _ => {
            
            draw_rounded_rect(cx - 12, u - 12, 24, 24, 6, color);
            draw_text(cx - 3, u - 6, "?", 0xFF0A0A0A);
        }
    }
}





pub fn jwv() -> usize { JF_.len() }
pub fn fhe(idx: usize) -> &'static str { JF_[idx].name }
pub fn lgq(slot: usize) -> usize { ASP_[slot] }
pub fn qdg() -> usize { IP_ }






pub struct Mr {
    pub playing: bool,
    pub beat: f32,
    pub energy: f32,
    pub sub_bass: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub frame: u64,
}




pub fn ljo(
    vx: i32, vy: i32, bt: u32, ex: u32,
    audio: &Mr,
    dropdown_open: bool,
    edu: u8,
) {
    const MI_: u32 = 130;
    const RL_: u32 = 14;
    const DFE_: u32 = 16;
    const GZ_: u32 = 26;

    let csu = bt.saturating_sub(RL_ * 2);
    let cfd = vx + RL_ as i32;
    
    let elx = if dropdown_open { crate::visualizer::JJ_ as u32 * GZ_ + 8 } else { 0 };
    let cfe = vy + ex as i32 - GY_ as i32 - IW_ as i32 - MI_ as i32 - 8 - elx as i32;

    
    ekn(cfd, cfe, csu, MI_, DFE_, 210);

    let pad = 14u32;
    let bi = cfd + pad as i32; 
    let oo = csu.saturating_sub(pad * 2); 
    let mut u = cfe + 12;

    
    let title = if audio.playing { "Now Playing" } else { "Music" };
    let bwl = if audio.playing { I_ } else { GR_ };
    draw_text(bi, u, title, bwl);

    
    let bcu = if (edu as usize) < crate::visualizer::PE_.len() {
        crate::visualizer::PE_[edu as usize]
    } else { "Sphere" };
    let iog = crate::graphics::scaling::auh(bcu) as i32;
    let jxt = if dropdown_open { "^" } else { "v" };
    let cfx = bi + oo as i32 - iog - 14;
    draw_text(cfx, u, bcu, BM_);
    draw_text(cfx + iog + 4, u, jxt, I_);

    u += 20;

    
    let hs = 8u32;
    let hgq = 4u32;
    let ek = (oo - hgq * 3) / 4;
    let apt: [(f32, u32, &str); 4] = [
        (audio.sub_bass, 0xFF00FF44, "SB"),
        (audio.bass,     0xFF00CC88, "BA"),
        (audio.mid,      0xFF00AACC, "MD"),
        (audio.treble,   0xFF8866FF, "TR"),
    ];
    for (bal, &(level, color, label)) in apt.iter().enumerate() {
        let bx = bi + (bal as u32 * (ek + hgq)) as i32;
        
        framebuffer::co(bx.max(0) as u32, u.max(0) as u32, ek, hs, 0x112211, 150);
        
        let fill = if audio.playing {
            (level.min(1.0) * ek as f32) as u32
        } else { 0 };
        if fill > 0 {
            framebuffer::fill_rect(bx.max(0) as u32, u.max(0) as u32, fill, hs, color);
            framebuffer::co(bx.max(0) as u32, u.max(0) as u32, fill, hs, 0xFFFFFF, 15);
        }
        
        let mo = crate::graphics::scaling::auh(label) as i32;
        draw_text(bx + (ek as i32 - mo) / 2, u - 1, label, 0xFFAABBAA);
    }
    u += hs as i32 + 8;

    
    let beh = 36u32;
    framebuffer::co(bi.max(0) as u32, u.max(0) as u32, oo, beh, 0x030908, 160);
    
    framebuffer::co(bi.max(0) as u32, u.max(0) as u32, oo, 1, 0x00FF66, 25);
    framebuffer::co(bi.max(0) as u32, (u + beh as i32 - 1).max(0) as u32, oo, 1, 0x00FF66, 15);

    let ags = u + beh as i32 / 2;
    let kh = (beh / 2 - 2) as f32;

    if audio.playing {
        
        let eus = oo.min(256) as usize;
        for i in 0..eus {
            let t = i as f32 / eus as f32;
            let phase = audio.frame as f32 * 0.06;
            
            let afq = libm::sinf(t * 12.0 + phase) * audio.energy;
            let azn = libm::sinf(t * 28.0 + phase * 1.4) * audio.treble * 0.5;
            let dyf = libm::sinf(t * 5.0 + phase * 0.7) * audio.bass * 0.7;
            let kay = 1.0 + audio.beat * 0.6;
            let ank = ((afq + azn + dyf) * kay).max(-1.0).min(1.0);
            let hdc = (ank * kh) as i32;

            let p = (bi + i as i32).max(0) as u32;
            let o = ((ags + hdc).max(u + 2).min(u + beh as i32 - 3)) as u32;

            
            let cyh = 0xCC;
            let awd = (audio.beat * 160.0).min(255.0) as u32;
            let oau = (audio.energy * 50.0).min(50.0) as u32;
            let color = 0xFF000000 | (oau << 16) | (cyh << 8) | awd;
            framebuffer::put_pixel(p, o, color);
            
            framebuffer::put_pixel(p, o, 0xFF00FFCC);
        }
        
        if audio.beat > 0.4 {
            let flash = ((audio.beat - 0.4) * 40.0).min(30.0) as u32;
            framebuffer::co(bi.max(0) as u32, u.max(0) as u32, oo, beh, 0x00FF88, flash);
        }
    } else {
        
        framebuffer::fill_rect((bi + 4).max(0) as u32, ags.max(0) as u32, oo.saturating_sub(8), 1, 0xFF334433);
        draw_text_centered(bi + oo as i32 / 2, ags - 6, "---", 0xFF445544);
    }
    u += beh as i32 + 8;

    
    let hpf: &[&str] = &["|<", "<<", if audio.playing { "||" } else { ">" }, ">>", ">|"];
    let dut = hpf.len() as u32;
    let gu = 40u32;
    let hn = 20u32;
    let rj = 6u32;
    let gzm = dut * gu + (dut - 1) * rj;
    let fjy = bi + (oo as i32 - gzm as i32) / 2;

    for (bal, &label) in hpf.iter().enumerate() {
        let bx = fjy + (bal as u32 * (gu + rj)) as i32;
        let dss = bal == 2;
        let bg = if dss {
            if audio.playing { 0x00AA55u32 } else { 0x005533u32 }
        } else {
            0x1A2A1Au32
        };
        framebuffer::co(bx.max(0) as u32, u.max(0) as u32, gu, hn, bg, 190);
        
        framebuffer::co(bx.max(0) as u32, u.max(0) as u32, gu, 1, 0x00FF88, 30);
        let mo = crate::graphics::scaling::auh(label) as i32;
        let dtg = if dss { 0xFF00FFAA } else { EP_ };
        draw_text(bx + (gu as i32 - mo) / 2, u + 4, label, dtg);
    }
    u += hn as i32 + 4;

    
    if dropdown_open {
        let fqx = cfd + 8;
        let hqt = csu - 16;
        let dvo = crate::visualizer::JJ_ as u32;
        let lbx = dvo * GZ_ + 8;
        
        ekn(fqx, u, hqt, lbx, 12, 230);
        let mut ad = u + 4;
        for mi in 0..dvo {
            let name = crate::visualizer::PE_[mi as usize];
            let hd = mi as u8 == edu;
            if hd {
                framebuffer::co((fqx + 4).max(0) as u32, ad.max(0) as u32, hqt - 8, GZ_, 0x00FF66, 30);
            }
            let muq = if hd { I_ } else { O_ };
            let cgv = if hd { "> " } else { "  " };
            use alloc::format;
            let label = format!("{}{}", cgv, name);
            draw_text(fqx + 12, ad + 6, &label, muq);
            ad += GZ_ as i32;
        }
    }
}







pub fn ljn(
    vx: i32, vy: i32, bt: u32, ex: u32,
    awc: u32, frame: u64, audio: &Mr,
    state: &MobileState,
) {
    let bn = vy + ID_ as i32;
    let en = ex.saturating_sub(ID_ + 20);
    
    framebuffer::co(vx.max(0) as u32, bn.max(0) as u32, bt, en, 0x050A06, 220);

    match awc {
        0 => lhy(vx, bn, bt, en, frame, state),
        1 => lhs(vx, bn, bt, en, state),
        2 => lhr(vx, bn, bt, en, frame, state),
        3 => lhp(vx, bn, bt, en, frame, state),
        4 => lhv(vx, bn, bt, en, frame),
        5 => lht(vx, bn, bt, en, state),
        6 => lho(vx, bn, bt, en, state),
        7 => lhz(vx, bn, bt, en, frame),
        8 => lhw(vx, bn, bt, en, state),
        9 => lhm(vx, bn, bt, en, frame),
        10 => lhu(vx, bn, bt, en, frame, audio),
        11 => lhq(vx, bn, bt, en, state),
        _ => {
            draw_text_centered(vx + bt as i32 / 2, bn + en as i32 / 2, "Unknown App", O_);
        }
    }
}


fn lhy(vx: i32, u: i32, bt: u32, ch: u32, frame: u64, state: &MobileState) {
    let pad = 10i32;
    let bi = vx + pad;

    
    framebuffer::co(vx.max(0) as u32, u.max(0) as u32, bt, 24, 0x0A1A0A, 200);
    draw_text(bi, u + 4, "trustos@mobile:~$", I_);

    let bw = 16i32;
    let mut ly = u + 28;

    
    if state.term_lines.is_empty() {
        let puj = [
            "TrustOS v2.0 — Mobile Shell",
            "Type 'help' for available commands.",
            "",
        ];
        for line in &puj {
            if ly + bw > u + ch as i32 - 40 { break; }
            let color = if line.starts_with("TrustOS") { AH_ } else { O_ };
            draw_text(bi, ly, line, color);
            ly += bw;
        }
    }

    
    let aac = ((ch as i32 - 68) / bw).max(1) as usize;
    let start = if state.term_lines.len() > aac { state.term_lines.len() - aac } else { 0 };
    for line in &state.term_lines[start..] {
        if ly + bw > u + ch as i32 - 40 { break; }
        let color = if line.starts_with("$") { I_ }
                    else if line.starts_with("TrustOS") { AH_ }
                    else { O_ };
        draw_text(bi, ly, line, color);
        ly += bw;
    }

    
    let sv = u + ch as i32 - 36;
    framebuffer::co(vx.max(0) as u32, sv.max(0) as u32, bt, 32, 0x0A1A0A, 200);
    let nh = alloc::format!("$ {}", state.term_input);
    draw_text(bi, sv + 8, &nh, I_);
    
    if (frame / 30) % 2 == 0 {
        let cursor_x = bi + crate::graphics::scaling::auh(&nh) as i32 + 2;
        framebuffer::fill_rect(cursor_x.max(0) as u32, (sv + 8).max(0) as u32, 8, 14, I_);
    }
    
    draw_text_centered(vx + bt as i32 / 2, sv - 14, "Tap here to run a command", AW_);
}


fn lhs(vx: i32, u: i32, bt: u32, ch: u32, state: &MobileState) {
    let pad = 10i32;
    let bi = vx + pad;

    
    framebuffer::co(vx.max(0) as u32, u.max(0) as u32, bt, 28, 0x0A120E, 200);
    let gmp = if state.files_depth == 0 { "/home/user/" } else { "/home/user/Documents/" };
    if state.files_depth > 0 {
        draw_text(bi, u + 6, "< Back", I_);
        let wl = crate::graphics::scaling::auh(gmp) as i32;
        draw_text(vx + bt as i32 - pad - wl, u + 6, gmp, AH_);
    } else {
        draw_text(bi, u + 6, gmp, AH_);
    }

    
    let lqm: [(&str, &str, u32); 8] = [
        ("Documents", "DIR", 0xFFDDAA30),
        ("Downloads", "DIR", 0xFFDDAA30),
        ("Pictures",  "DIR", 0xFFDDAA30),
        ("Music",     "DIR", 0xFF4488DD),
        ("readme.md", "4KB", O_),
        ("config.toml","2KB", O_),
        ("notes.txt", "1KB", O_),
        ("photo.png", "3MB", 0xFF9060D0),
    ];
    let lqn: [(&str, &str, u32); 6] = [
        ("project.rs", "12KB", 0xFF6688CC),
        ("report.pdf", "2MB", 0xFFCC4444),
        ("budget.csv", "8KB", 0xFF40CC80),
        ("slides.md",  "6KB", O_),
        ("backup.zip", "45MB", 0xFF9060D0),
        ("todo.txt",   "1KB", O_),
    ];

    let ep = 40u32;
    let mut qz = u + 32;

    if state.files_depth == 0 {
        for (i, &(name, size, color)) in lqm.iter().enumerate() {
            if qz + ep as i32 > u + ch as i32 { break; }
            let hd = state.files_selected == i as i32;
            let bg = if hd { 0x0A2A15 } else { 0x060A08 };
            framebuffer::co(vx.max(0) as u32, qz.max(0) as u32, bt, ep, bg, 180);
            framebuffer::fill_rect((vx + 8).max(0) as u32, (qz + ep as i32 - 1).max(0) as u32, bt.saturating_sub(16), 1, AP_);
            let mng = if size == "DIR" { ">" } else { "-" };
            let ayi = if hd { I_ } else { color };
            draw_text(bi, qz + 12, mng, ayi);
            draw_text(bi + 16, qz + 12, name, ayi);
            let dy = crate::graphics::scaling::auh(size) as i32;
            draw_text(vx + bt as i32 - pad - dy, qz + 12, size, AW_);
            qz += ep as i32;
        }
    } else {
        for (i, &(name, size, color)) in lqn.iter().enumerate() {
            if qz + ep as i32 > u + ch as i32 { break; }
            let hd = state.files_selected == i as i32;
            let bg = if hd { 0x0A2A15 } else { 0x060A08 };
            framebuffer::co(vx.max(0) as u32, qz.max(0) as u32, bt, ep, bg, 180);
            framebuffer::fill_rect((vx + 8).max(0) as u32, (qz + ep as i32 - 1).max(0) as u32, bt.saturating_sub(16), 1, AP_);
            let ayi = if hd { I_ } else { color };
            draw_text(bi, qz + 12, "-", ayi);
            draw_text(bi + 16, qz + 12, name, ayi);
            let dy = crate::graphics::scaling::auh(size) as i32;
            draw_text(vx + bt as i32 - pad - dy, qz + 12, size, AW_);
            qz += ep as i32;
        }
    }
}


fn lhr(vx: i32, u: i32, bt: u32, ch: u32, frame: u64, state: &MobileState) {
    let pad = 10i32;
    let bi = vx + pad;

    
    framebuffer::co(vx.max(0) as u32, u.max(0) as u32, bt, 26, 0x0A1A10, 200);
    let pcn = if state.editor_tab == 0 { I_ } else { AW_ };
    let pco = if state.editor_tab == 1 { I_ } else { AW_ };
    draw_text(bi, u + 6, "main.rs", pcn);
    draw_text(bi + 80, u + 6, "lib.rs", pco);
    
    let ppj = if state.editor_tab == 0 { bi } else { bi + 80 };
    framebuffer::fill_rect(ppj.max(0) as u32, (u + 24).max(0) as u32, 50, 2, I_);

    
    let kuu = [
        (1, "fn main() {"),
        (2, "    let os = TrustOS::new();"),
        (3, "    os.init_hardware();"),
        (4, "    os.start_desktop();"),
        (5, ""),
        (6, "    // Mobile mode"),
        (7, "    if os.is_mobile() {"),
        (8, "        os.launch_mobile();"),
        (9, "    }"),
        (10, ""),
        (11, "    os.run_forever();"),
        (12, "}"),
    ];
    let kuv = [
        (1, "pub mod kernel;"),
        (2, "pub mod desktop;"),
        (3, "pub mod mobile;"),
        (4, "pub mod audio;"),
        (5, "pub mod visualizer;"),
        (6, "pub mod network;"),
        (7, ""),
        (8, "pub fn init() {"),
        (9, "    kernel::start();"),
        (10, "}"),
        (11, ""),
        (12, ""),
    ];

    let kuq = if state.editor_tab == 0 { &kuu } else { &kuv };

    let bw = 16i32;
    let mut ly = u + 30;
    for &(num, line) in kuq.iter() {
        if ly + bw > u + ch as i32 { break; }
        use alloc::format;
        let rw = format!("{:3}", num);
        
        let is_current = (num - 1) as u32 == state.editor_cursor_line;
        if is_current {
            framebuffer::co(vx.max(0) as u32, ly.max(0) as u32, bt, bw as u32, 0x1A2A1A, 120);
        }
        draw_text(bi, ly, &rw, if is_current { I_ } else { AW_ });
        
        let color = if line.contains("fn ") { 0xFF6688CC }
            else if line.contains("let ") || line.contains("pub ") { 0xFF8866FF }
            else if line.contains("//") { 0xFF556655 }
            else if line.contains("TrustOS") || line.contains("mod ") { I_ }
            else { O_ };
        draw_text(bi + 30, ly, line, color);
        ly += bw;
    }
    
    if (frame / 30) % 2 == 0 {
        let cursor_y = u + 30 + state.editor_cursor_line as i32 * bw;
        if cursor_y >= u + 30 && cursor_y < u + ch as i32 {
            framebuffer::fill_rect((bi + 30).max(0) as u32, cursor_y.max(0) as u32, 2, 14, I_);
        }
    }
}


fn lhp(vx: i32, u: i32, bt: u32, ch: u32, _frame: u64, state: &MobileState) {
    let pad = 14i32;
    let bi = vx + pad;
    let oo = (bt as i32 - pad * 2) as u32;

    
    let atm = 60u32;
    framebuffer::co(bi.max(0) as u32, (u + 10).max(0) as u32, oo, atm, 0x0A1A10, 220);
    iu(bi, u + 10, oo, atm, 8, AP_);
    let hsm = if state.calc_display.is_empty() { "0" } else { &state.calc_display };
    let gr = crate::graphics::scaling::auh(hsm) as i32;
    draw_text(bi + oo as i32 - gr - 10, u + 30, hsm, I_);

    
    let kei = [
        ["C", "+/-", "%", "/"],
        ["7", "8", "9", "x"],
        ["4", "5", "6", "-"],
        ["1", "2", "3", "+"],
        ["0", ".", "=", ""],
    ];
    let hn = 44u32;
    let rj = 6u32;
    let gu = (oo - rj * 3) / 4;
    let mut dc = u + 10 + atm as i32 + 14;

    for row in &kei {
        let mut bx = bi;
        for &label in row {
            if label.is_empty() { bx += (gu + rj) as i32; continue; }
            let iid = matches!(label, "/" | "x" | "-" | "+" | "=");
            let mso = matches!(label, "C" | "+/-" | "%");
            let bg = if iid { 0xFF008844u32 }
                     else if mso { 0xFF333833 }
                     else { 0xFF1A221A };
            draw_rounded_rect(bx, dc, gu, hn, 8, bg);
            iu(bx, dc, gu, hn, 8, AP_);
            let dtg = if iid { I_ } else { AB_ };
            draw_text_centered(bx + gu as i32 / 2, dc + 14, label, dtg);
            bx += (gu + rj) as i32;
        }
        dc += (hn + rj) as i32;
    }
}


fn lhv(vx: i32, u: i32, bt: u32, ch: u32, frame: u64) {
    let pad = 12i32;
    let bi = vx + pad;
    let oo = bt as i32 - pad * 2;

    let mut ly = u + 10;
    let oms = 28i32;

    
    draw_text(bi, ly, "WiFi", I_);
    draw_text(bi + oo - 24, ly, "ON", AH_);
    ly += oms;
    framebuffer::co(bi.max(0) as u32, ly.max(0) as u32, oo as u32, 40, 0x0A120E, 180);
    draw_text(bi + 8, ly + 12, "TrustNet-5G", AB_);
    draw_text(bi + oo - 80, ly + 12, "Connected", AH_);
    ly += 48;

    
    draw_text(bi, ly, "Network Info", I_);
    ly += 22;
    let info = [
        ("IP Address:", "192.168.1.42"),
        ("Subnet:", "255.255.255.0"),
        ("Gateway:", "192.168.1.1"),
        ("DNS:", "8.8.8.8"),
        ("MAC:", "AA:BB:CC:DD:EE:FF"),
    ];
    for &(label, value) in &info {
        if ly + 18 > u + ch as i32 { break; }
        draw_text(bi + 8, ly, label, O_);
        let hbz = crate::graphics::scaling::auh(value) as i32;
        draw_text(bi + oo - hbz - 8, ly, value, AB_);
        ly += 20;
    }
    ly += 10;

    
    draw_text(bi, ly, "Signal", I_);
    ly += 20;
    let ek = (oo - 16) as u32;
    framebuffer::fill_rect((bi + 8).max(0) as u32, ly.max(0) as u32, ek, 8, AP_);
    let ash = ((frame % 100) as u32 * ek / 100).max(ek * 7 / 10);
    framebuffer::fill_rect((bi + 8).max(0) as u32, ly.max(0) as u32, ash, 8, AH_);
}


fn lht(vx: i32, u: i32, bt: u32, ch: u32, state: &MobileState) {
    let pad = 12i32;
    let bi = vx + pad;
    let oo = (bt as i32 - pad * 2) as u32;

    draw_text(bi, u + 10, "Games Library", I_);

    let mbc = [
        ("Snake", "Classic arcade", 0xFF44DD44),
        ("Chess", "Strategy board game", 0xFFD4A854),
        ("3D FPS", "Raycasting demo", 0xFF4488DD),
        ("GameBoy", "GB emulator", 0xFF9060D0),
        ("NES", "NES emulator", 0xFFCC4444),
    ];

    let aev = 56u32;
    let fkx = 8u32;
    let mut jh = u + 34;

    for (i, &(name, desc, accent)) in mbc.iter().enumerate() {
        if jh + aev as i32 > u + ch as i32 { break; }
        let hd = state.games_selected == i as i32;
        let bg = if hd { 0xFF0C1610 } else { 0xFF080C0A };
        let border = if hd { I_ } else { AP_ };
        draw_rounded_rect(bi, jh, oo, aev, 10, bg);
        iu(bi, jh, oo, aev, 10, border);
        
        framebuffer::fill_rect(bi.max(0) as u32, (jh + 8).max(0) as u32, 3, aev - 16, accent);
        
        draw_text(bi + 14, jh + 10, name, accent);
        
        draw_text(bi + 14, jh + 28, desc, O_);
        
        let kek = if hd { ">>>" } else { ">" };
        let djr = if hd { I_ } else { AW_ };
        draw_text(vx + bt as i32 - pad - 30, jh + 16, kek, djr);
        jh += (aev + fkx) as i32;
    }
}


fn lho(vx: i32, u: i32, bt: u32, ch: u32, state: &MobileState) {
    let pad = 8i32;
    let bi = vx + pad;
    let oo = (bt as i32 - pad * 2) as u32;

    
    let jpk = 30u32;
    draw_rounded_rect(bi, u + 4, oo, jpk, 10, 0xFF0A120E);
    iu(bi, u + 4, oo, jpk, 10, AP_);
    let url = match state.browser_page {
        0 => "https://trustos.local",
        1 => "https://trustos.local/docs",
        2 => "https://trustos.local/source",
        3 => "https://trustos.local/downloads",
        _ => "https://trustos.local",
    };
    draw_text(bi + 10, u + 12, url, O_);

    
    let arr = u + 40;
    framebuffer::co(vx.max(0) as u32, arr.max(0) as u32, bt, ch - 40, 0x0C140E, 200);

    let mut ly = arr + 10;
    match state.browser_page {
        0 => {
            draw_text(bi + 4, ly, "Welcome to TrustOS", I_); ly += 24;
            draw_text(bi + 4, ly, "A secure, minimal operating system", O_); ly += 20;
            draw_text(bi + 4, ly, "built with Rust.", O_); ly += 30;
            draw_text(bi + 4, ly, "> Documentation", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "> Source Code", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "> Downloads", 0xFF4488DD);
        }
        1 => {
            draw_text(bi + 4, ly, "Documentation", I_); ly += 24;
            draw_text(bi + 4, ly, "1. Getting Started", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "   Install TrustOS on bare metal", O_); ly += 16;
            draw_text(bi + 4, ly, "   or run in QEMU/VirtualBox.", O_); ly += 24;
            draw_text(bi + 4, ly, "2. Mobile Mode", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "   Portrait UI for small screens.", O_); ly += 24;
            draw_text(bi + 4, ly, "3. Desktop Mode", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "   Full windowed environment.", O_); ly += 24;
            draw_text(bi + 4, ly, "4. Audio System", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "   HD Audio with DMA.", O_); ly += 30;
            draw_text(bi + 4, ly, "< Back to Home", AH_);
        }
        2 => {
            draw_text(bi + 4, ly, "Source Code", I_); ly += 24;
            draw_text(bi + 4, ly, "Repository:", O_); ly += 20;
            draw_text(bi + 4, ly, "  github.com/trustos/kernel", 0xFF4488DD); ly += 24;
            draw_text(bi + 4, ly, "Language: Rust (no_std)", O_); ly += 20;
            draw_text(bi + 4, ly, "LOC: ~25,000", O_); ly += 20;
            draw_text(bi + 4, ly, "License: MIT", O_); ly += 30;
            draw_text(bi + 4, ly, "< Back to Home", AH_);
        }
        3 => {
            draw_text(bi + 4, ly, "Downloads", I_); ly += 24;
            draw_text(bi + 4, ly, "TrustOS v2.0 ISO", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "  Size: 12 MB | x86_64", O_); ly += 24;
            draw_text(bi + 4, ly, "TrustOS v2.0 aarch64", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "  Size: 14 MB | ARM64", O_); ly += 24;
            draw_text(bi + 4, ly, "VBox Appliance (.ova)", 0xFF4488DD); ly += 20;
            draw_text(bi + 4, ly, "  Size: 50 MB | Pre-configured", O_); ly += 30;
            draw_text(bi + 4, ly, "< Back to Home", AH_);
        }
        _ => {
            draw_text(bi + 4, ly, "Page not found", O_);
        }
    }

    
    ly = u + ch as i32 - 18;
    framebuffer::fill_rect((bi + 4).max(0) as u32, (ly - 4).max(0) as u32, oo - 8, 1, AP_);
    draw_text(bi + 4, ly, "TrustOS Browser v1.0", AW_);
}


fn lhz(vx: i32, u: i32, bt: u32, ch: u32, frame: u64) {
    let pad = 10i32;
    let bi = vx + pad;
    let oo = (bt as i32 - pad * 2) as u32;

    
    framebuffer::co(vx.max(0) as u32, u.max(0) as u32, bt, 28, 0x0A120E, 200);
    let gzk = ["Move", "Rot", "Scale", "Add"];
    let mut bu = bi;
    for tool in &gzk {
        draw_text(bu, u + 7, tool, GR_);
        bu += 50;
    }

    
    let bws = u + 32;
    let aak = ch.saturating_sub(60);
    framebuffer::co(vx.max(0) as u32, bws.max(0) as u32, bt, aak, 0x030806, 220);

    
    let hpz = vx + bt as i32 / 2;
    let hqb = bws + aak as i32 / 2;
    for i in 0..8u32 {
        let offset = (i as i32 - 4) * 20;
        framebuffer::co(vx.max(0) as u32, (hqb + offset).max(0) as u32, bt, 1, 0x002A15, 40);
        framebuffer::co((hpz + offset).max(0) as u32, bws.max(0) as u32, 1, aak, 0x002A15, 40);
    }

    
    let t = frame as f32 * 0.03;
    let cqr = libm::sinf(t);
    let chs = libm::cosf(t);
    let j = 40.0f32;
    let hpi: [(f32, f32, f32); 8] = [
        (-j, -j, -j), (j, -j, -j), (j, j, -j), (-j, j, -j),
        (-j, -j,  j), (j, -j,  j), (j, j,  j), (-j, j,  j),
    ];
    let edges: [(usize, usize); 12] = [
        (0,1),(1,2),(2,3),(3,0), (4,5),(5,6),(6,7),(7,4),
        (0,4),(1,5),(2,6),(3,7),
    ];
    let project = |aa: (f32, f32, f32)| -> (i32, i32) {
        let da = aa.0 * chs - aa.2 * cqr;
        let qp = aa.0 * cqr + aa.2 * chs;
        let cm = aa.1 * libm::cosf(t * 0.7) - qp * libm::sinf(t * 0.7);
        (hpz + da as i32, hqb + cm as i32)
    };
    for &(a, b) in &edges {
        let (x1, y1) = project(hpi[a]);
        let (x2, y2) = project(hpi[b]);
        draw_line(x1, y1, x2, y2, AH_);
    }

    
    let btj = bws + aak as i32 - 20;
    draw_text(bi, btj, "Vertices: 8  Faces: 6  Edges: 12", AW_);
}


fn lhw(vx: i32, u: i32, bt: u32, ch: u32, state: &MobileState) {
    let pad = 12i32;
    let bi = vx + pad;
    let oo = (bt as i32 - pad * 2) as u32;

    let abj = [
        ("WiFi", "Wireless connection", 0xFF40CC80),
        ("Bluetooth", "Paired devices", 0xFF4488DD),
        ("Airplane", "Radio off", 0xFFCC8844),
        ("Do Not Disturb", "Silence alerts", 0xFFFF6090),
        ("Dark Mode", "Display theme", 0xFF9988BB),
        ("Notifications", "Push alerts", 0xFF40AADD),
    ];

    let ep = 52u32;
    let mut cm = u + 10;
    for (i, &(title, desc, accent)) in abj.iter().enumerate() {
        if cm + ep as i32 > u + ch as i32 { break; }
        let hd = state.settings_selected == i as i32;
        let bg = if hd { 0x0A1A12 } else { 0x080C0A };
        framebuffer::co(bi.max(0) as u32, cm.max(0) as u32, oo, ep, bg, 180);
        framebuffer::fill_rect((bi + 4).max(0) as u32, (cm + ep as i32 - 1).max(0) as u32, oo - 8, 1, AP_);
        
        framebuffer::fill_rect((bi + 8).max(0) as u32, (cm + 20).max(0) as u32, 4, 4, accent);
        draw_text(bi + 20, cm + 10, title, AB_);
        draw_text(bi + 20, cm + 28, desc, O_);
        
        let gzh = i < state.settings_toggles.len() && state.settings_toggles[i];
        let gzi = vx + bt as i32 - pad - 44;
        let pkr = if gzh { 0xFF008844 } else { 0xFF333833 };
        draw_rounded_rect(gzi, cm + 14, 40, 22, 11, pkr);
        
        let cbf = if gzh { gzi + 20 } else { gzi + 2 };
        draw_rounded_rect(cbf, cm + 16, 18, 18, 9, if gzh { I_ } else { AW_ });
        cm += ep as i32;
    }
}


fn lhm(vx: i32, u: i32, bt: u32, ch: u32, _frame: u64) {
    let pad = 14i32;
    let bi = vx + pad;

    let center_x = vx + bt as i32 / 2;
    let mut ly = u + 20;

    
    draw_text_centered(center_x, ly, "TrustOS", I_);
    ly += 24;
    draw_text_centered(center_x, ly, "v2.0.0", AH_);
    ly += 30;

    
    framebuffer::fill_rect((bi + 20).max(0) as u32, ly.max(0) as u32, bt.saturating_sub(68), 1, AP_);
    ly += 16;

    let info = [
        ("Kernel:", "TrustOS Microkernel"),
        ("Arch:", "x86_64 / aarch64"),
        ("License:", "MIT"),
        ("Desktop:", "TrustOS Desktop v2"),
        ("Browser:", "TrustBrowser v1.0"),
        ("Audio:", "HD Audio + DMA"),
        ("Graphics:", "COSMIC Renderer"),
        ("Uptime:", "4h 23m"),
    ];

    for &(label, value) in &info {
        if ly + 20 > u + ch as i32 { break; }
        draw_text(bi, ly, label, O_);
        let hbz = crate::graphics::scaling::auh(value) as i32;
        draw_text(vx + bt as i32 - pad - hbz, ly, value, AB_);
        ly += 22;
    }
}


fn lhu(vx: i32, u: i32, bt: u32, ch: u32, frame: u64, audio: &Mr) {
    let pad = 14i32;
    let bi = vx + pad;
    let oo = (bt as i32 - pad * 2) as u32;

    
    let anm = oo.min(200);
    let dhx = vx + (bt as i32 - anm as i32) / 2;
    let mut ly = u + 14;
    draw_rounded_rect(dhx, ly, anm, anm, 14, 0xFF0A1A10);
    iu(dhx, ly, anm, anm, 14, AP_);

    
    if audio.playing {
        let arn = dhx + anm as i32 / 2;
        let ags = ly + anm as i32 / 2;
        let gim = 16u32;
        let ek = anm / (gim * 2);
        for i in 0..gim {
            let t = i as f32 / gim as f32;
            let phase = frame as f32 * 0.08 + t * 6.28;
            let ank = (libm::sinf(phase) * audio.energy + audio.bass * 0.5).max(0.1).min(1.0);
            let h = (ank * (anm as f32 * 0.4)) as u32;
            let bx = dhx + 10 + (i * (ek * 2)) as i32;
            let dc = ly + anm as i32 / 2 + (anm as i32 / 4 - h as i32).max(0);
            let g = (128.0 + ank * 127.0).min(255.0) as u32;
            framebuffer::fill_rect(bx.max(0) as u32, dc.max(0) as u32, ek, h, 0xFF000000 | (g << 8) | 0x40);
        }
    } else {
        draw_text_centered(dhx + anm as i32 / 2, ly + anm as i32 / 2 - 6, "No Track", O_);
    }
    ly += anm as i32 + 16;

    
    let title = if audio.playing { "Untitled (2) - Lo-Fi" } else { "No Track Playing" };
    draw_text_centered(vx + bt as i32 / 2, ly, title, AB_);
    ly += 20;
    draw_text_centered(vx + bt as i32 / 2, ly, "TrustOS Audio", O_);
    ly += 28;

    
    let ek = oo - 20;
    framebuffer::fill_rect((bi + 10).max(0) as u32, ly.max(0) as u32, ek, 4, AP_);
    if audio.playing {
        let progress = (frame % 300) as u32 * ek / 300;
        framebuffer::fill_rect((bi + 10).max(0) as u32, ly.max(0) as u32, progress, 4, I_);
    }
    ly += 20;

    
    let kzz = ["|<", "<<", if audio.playing { "||" } else { ">" }, ">>", ">|"];
    let gu = 48u32;
    let hn = 36u32;
    let rj = 10u32;
    let aaj = 5 * gu + 4 * rj;
    let fjx = bi + (oo as i32 - aaj as i32) / 2;
    for (i, &label) in kzz.iter().enumerate() {
        let bx = fjx + (i as u32 * (gu + rj)) as i32;
        let dss = i == 2;
        let bg = if dss { if audio.playing { 0xFF005533 } else { 0xFF003322 } } else { 0xFF1A2A1A };
        draw_rounded_rect(bx, ly, gu, hn, 10, bg);
        let dtg = if dss { I_ } else { EP_ };
        draw_text_centered(bx + gu as i32 / 2, ly + 10, label, dtg);
    }
    ly += hn as i32 + 16;

    
    let jzh = ["Sub", "Bass", "Mid", "Treble"];
    let jzj = [audio.sub_bass, audio.bass, audio.mid, audio.treble];
    let jzg: [u32; 4] = [0xFF00FF44, 0xFF00CC88, 0xFF00AACC, 0xFF8866FF];
    let efy = (oo - 12) / 4;
    for (i, (&name, &val)) in jzh.iter().zip(jzj.iter()).enumerate() {
        let bx = bi + (i as u32 * (efy + 4)) as i32;
        framebuffer::co(bx.max(0) as u32, ly.max(0) as u32, efy, 10, 0x112211, 150);
        let fill = if audio.playing { (val.min(1.0) * efy as f32) as u32 } else { 0 };
        if fill > 0 {
            framebuffer::fill_rect(bx.max(0) as u32, ly.max(0) as u32, fill, 10, jzg[i]);
        }
        draw_text_centered(bx + efy as i32 / 2, ly + 12, name, AW_);
    }
}


fn lhq(vx: i32, u: i32, bt: u32, ch: u32, state: &MobileState) {
    let pad = 8i32;

    
    let tg = (bt as i32 - pad * 2).min(ch as i32 - 60).min(400);
    let cell = tg / 8;
    let un = vx + (bt as i32 - tg) / 2;
    let ve = u + 10;

    let onj = if state.chess_selected >= 0 { state.chess_selected / 8 } else { -1 };
    let oni = if state.chess_selected >= 0 { state.chess_selected % 8 } else { -1 };

    
    for row in 0..8u32 {
        for col in 0..8u32 {
            let bhj = (row + col) % 2 == 0;
            let hd = row as i32 == onj && col as i32 == oni;
            let color = if hd { 0xFF2A5A2A }
                       else if bhj { 0xFF2A3A2A }
                       else { 0xFF0A140A };
            let cx = un + (col * cell as u32) as i32;
            let cm = ve + (row * cell as u32) as i32;
            framebuffer::fill_rect(cx.max(0) as u32, cm.max(0) as u32, cell as u32, cell as u32, color);
            
            if hd {
                
                framebuffer::fill_rect(cx.max(0) as u32, cm.max(0) as u32, cell as u32, 2, I_);
                framebuffer::fill_rect(cx.max(0) as u32, (cm + cell - 2).max(0) as u32, cell as u32, 2, I_);
                
                framebuffer::fill_rect(cx.max(0) as u32, cm.max(0) as u32, 2, cell as u32, I_);
                framebuffer::fill_rect((cx + cell - 2).max(0) as u32, cm.max(0) as u32, 2, cell as u32, I_);
            }
        }
    }

    
    iu(un - 1, ve - 1, tg as u32 + 2, tg as u32 + 2, 2, AW_);

    
    let iuv = ["R", "N", "B", "Q", "K", "B", "N", "R"];
    for col in 0..8 {
        let cx = un + col * cell + cell / 2;
        
        draw_text_centered(cx, ve + 7 * cell + cell / 2 - 6, iuv[col as usize], 0xFFDDDDDD);
        draw_text_centered(cx, ve + 6 * cell + cell / 2 - 6, "P", 0xFFDDDDDD);
        
        draw_text_centered(cx, ve + cell / 2 - 6, iuv[col as usize], 0xFFD4A854);
        draw_text_centered(cx, ve + cell + cell / 2 - 6, "P", 0xFFD4A854);
    }

    
    let status_y = ve + tg + 8;
    let dfw = if state.chess_turn == 0 { "White to move" } else { "Black to move" };
    let ecu = if state.chess_turn == 0 { 0xFFDDDDDD } else { 0xFFD4A854 };
    draw_text_centered(vx + bt as i32 / 2, status_y, dfw, ecu);

    if state.chess_selected >= 0 {
        let ovf = alloc::format!("Selected: {}{}", 
            (b'a' + (state.chess_selected % 8) as u8) as char,
            8 - state.chess_selected / 8);
        draw_text_centered(vx + bt as i32 / 2, status_y + 18, &ovf, AH_);
    }
}





fn draw_line(bm: i32, az: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - bm).abs();
    let ad = -(y1 - az).abs();
    let am = if bm < x1 { 1 } else { -1 };
    let ak = if az < y1 { 1 } else { -1 };
    let mut err = dx + ad;
    let mut cx = bm;
    let mut u = az;
    loop {
        aag(cx, u, color);
        if cx == x1 && u == y1 { break; }
        let pg = 2 * err;
        if pg >= ad {
            if cx == x1 { break; }
            err += ad;
            cx += am;
        }
        if pg <= dx {
            if u == y1 { break; }
            err += dx;
            u += ak;
        }
    }
}
