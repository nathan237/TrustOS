








use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use micromath::F32Ext;

use super::math3d::{Vec3, Vec4, Mat4};
use super::render2d::Color2D;
use super::opengl::*;
use crate::framebuffer;






#[derive(Clone, Copy, PartialEq)]
pub enum CompositorTheme {
    
    Flat,
    
    Modern,
    
    Glass,
    
    Neon,
    
    Minimal,
}


#[derive(Clone, Copy)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}


#[derive(Clone)]
pub struct WindowSurface {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub opacity: f32,
    pub scale: f32,
    pub rotation: f32,
    pub z_order: i32,
    pub visible: bool,
    pub focused: bool,
    pub minimized: bool,
    
    
    pub content: Vec<u32>,
    pub content_width: u32,
    pub content_height: u32,
    
    
    pub target_x: f32,
    pub target_y: f32,
    pub target_opacity: f32,
    pub target_scale: f32,
    pub animation_progress: f32,
    pub animation_duration: f32,
    pub animation_easing: Easing,
}

impl WindowSurface {
    pub fn new(id: u32, x: f32, y: f32, width: f32, height: f32) -> Self {
        let w = width as u32;
        let h = height as u32;
        Self {
            id,
            x,
            y,
            width,
            height,
            opacity: 1.0,
            scale: 1.0,
            rotation: 0.0,
            z_order: 0,
            visible: true,
            focused: false,
            minimized: false,
            content: alloc::vec![0xFF0A0E0B; (w * h) as usize],
            content_width: w,
            content_height: h,
            target_x: x,
            target_y: y,
            target_opacity: 1.0,
            target_scale: 1.0,
            animation_progress: 1.0,
            animation_duration: 0.0,
            animation_easing: Easing::EaseOut,
        }
    }
    
    
    pub fn update(&mut self, fm: f32) {
        if self.animation_progress < 1.0 {
            self.animation_progress += fm / self.animation_duration.max(0.001);
            self.animation_progress = self.animation_progress.min(1.0);
            
            let t = jwz(self.animation_progress, self.animation_easing);
            
            self.x = lerp(self.x, self.target_x, t);
            self.y = lerp(self.y, self.target_y, t);
            self.opacity = lerp(self.opacity, self.target_opacity, t);
            self.scale = lerp(self.scale, self.target_scale, t);
        }
    }
    
    
    pub fn pye(&mut self, x: f32, y: f32, yq: f32, easing: Easing) {
        self.target_x = x;
        self.target_y = y;
        self.animation_progress = 0.0;
        self.animation_duration = yq;
        self.animation_easing = easing;
    }
    
    
    pub fn qfl(&mut self, yq: f32) {
        self.opacity = 0.0;
        self.target_opacity = 1.0;
        self.scale = 0.95;
        self.target_scale = 1.0;
        self.animation_progress = 0.0;
        self.animation_duration = yq;
        self.animation_easing = Easing::EaseOut;
    }
    
    
    pub fn ats(&mut self, yq: f32) {
        self.target_opacity = 0.0;
        self.target_scale = 0.95;
        self.animation_progress = 0.0;
        self.animation_duration = yq;
        self.animation_easing = Easing::EaseIn;
    }
}






pub struct Compositor {
    pub width: u32,
    pub height: u32,
    pub surfaces: Vec<WindowSurface>,
    pub theme: CompositorTheme,
    pub background_color: u32,
    pub initialized: bool,
    
    
    pub shadow_offset: f32,
    pub shadow_blur: f32,
    pub shadow_opacity: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_glow: f32,
    
    
    pub time: f32,
    pub fps: f32,
    
    
    pub bg_gradient_top: u32,
    pub bg_gradient_bottom: u32,
    pub bg_pattern: BackgroundPattern,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BackgroundPattern {
    Solid,
    Gradient,
    Grid,
    Noise,
    Animated,
}

impl Compositor {
    pub const fn new() -> Self {
        Self {
            width: 1280,
            height: 800,
            surfaces: Vec::new(),
            theme: CompositorTheme::Modern,
            background_color: 0xFF070707,
            initialized: false,
            shadow_offset: 8.0,
            shadow_blur: 16.0,
            shadow_opacity: 0.4,
            corner_radius: 8.0,
            border_width: 1.0,
            border_glow: 0.0,
            time: 0.0,
            fps: 60.0,
            bg_gradient_top: 0xFF070707,
            bg_gradient_bottom: 0xFF020303,
            bg_pattern: BackgroundPattern::Gradient,
        }
    }
    
    
    pub fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        
        ice(width, height);
        fzc(UR_);
        fzc(AEL_);
        
        
        eoi(OE_);
        dqu();
        met(0.0, width as f32, height as f32, 0.0, -100.0, 100.0);
        
        eoi(AEO_);
        dqu();
        
        self.initialized = true;
    }
    
    
    pub fn pxx(&mut self, surface: WindowSurface) -> u32 {
        let id = surface.id;
        self.surfaces.push(surface);
        self.sort_surfaces();
        id
    }
    
    
    pub fn qts(&mut self, id: u32) {
        self.surfaces.retain(|j| j.id != id);
    }
    
    
    pub fn get_surface_mut(&mut self, id: u32) -> Option<&mut WindowSurface> {
        self.surfaces.iter_mut().find(|j| j.id == id)
    }
    
    
    fn sort_surfaces(&mut self) {
        self.surfaces.sort_by(|a, b| a.z_order.cmp(&b.z_order));
    }
    
    
    pub fn update(&mut self, fm: f32) {
        self.time += fm;
        for surface in &mut self.surfaces {
            surface.update(fm);
        }
    }
    
    
    pub fn render(&self) {
        if !self.initialized {
            return;
        }
        
        
        icd(
            ((self.background_color >> 16) & 0xFF) as f32 / 255.0,
            ((self.background_color >> 8) & 0xFF) as f32 / 255.0,
            (self.background_color & 0xFF) as f32 / 255.0,
            1.0,
        );
        icc(AEM_ | AEN_);
        
        
        self.render_background();
        
        
        for surface in &self.surfaces {
            if !surface.visible || surface.opacity <= 0.001 {
                continue;
            }
            self.render_surface(surface);
        }
        
        
        mer();
    }
    
    
    fn render_background(&self) {
        match self.bg_pattern {
            BackgroundPattern::Solid => {
                self.draw_filled_rect(0.0, 0.0, self.width as f32, self.height as f32, 
                                      self.background_color, -99.0);
            }
            BackgroundPattern::Gradient => {
                self.draw_gradient_rect(0.0, 0.0, self.width as f32, self.height as f32,
                                        self.bg_gradient_top, self.bg_gradient_bottom, -99.0);
            }
            BackgroundPattern::Grid => {
                self.draw_gradient_rect(0.0, 0.0, self.width as f32, self.height as f32,
                                        self.bg_gradient_top, self.bg_gradient_bottom, -99.0);
                self.draw_grid(-98.0);
            }
            BackgroundPattern::Animated => {
                self.draw_animated_background(-99.0);
            }
            _ => {
                self.draw_filled_rect(0.0, 0.0, self.width as f32, self.height as f32,
                                      self.background_color, -99.0);
            }
        }
    }
    
    
    fn draw_animated_background(&self, z: f32) {
        
        self.draw_gradient_rect(0.0, 0.0, self.width as f32, self.height as f32,
                                self.bg_gradient_top, self.bg_gradient_bottom, z);
        
        
        let t = self.time * 0.5;
        let mfa = (self.width as f32 / 2.0) + (t.sin() * 200.0);
        let mfb = (self.height as f32 / 3.0) + (t.cos() * 100.0);
        self.draw_glow(mfa, mfb, 300.0, 0x1000FF44, z + 0.1);
    }
    
    
    fn render_surface(&self, surface: &WindowSurface) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        let z = surface.z_order as f32;
        
        
        match self.theme {
            CompositorTheme::Modern => {
                
                if self.shadow_opacity > 0.0 {
                    self.draw_shadow(x, y, w, h, z - 0.5);
                }
                
                self.draw_window_frame(surface, z);
            }
            CompositorTheme::Glass => {
                
                self.draw_blur_rect(x, y, w, h, z - 0.3);
                
                self.draw_glass_frame(surface, z);
            }
            CompositorTheme::Neon => {
                
                self.draw_neon_glow(x, y, w, h, z - 0.5);
                
                self.draw_neon_frame(surface, z);
            }
            CompositorTheme::Minimal => {
                
                self.draw_minimal_frame(surface, z);
            }
            CompositorTheme::Flat => {
                
                self.draw_flat_frame(surface, z);
            }
        }
        
        
        self.draw_surface_content(surface, z + 0.1);
    }
    
    
    fn draw_shadow(&self, x: f32, y: f32, w: f32, h: f32, z: f32) {
        let offset = self.shadow_offset;
        let awi = self.shadow_blur;
        let alpha = (self.shadow_opacity * 255.0) as u32;
        let bjd = alpha << 24;
        
        
        for i in 0..4 {
            let wd = awi * (i as f32 / 4.0);
            let mxe = alpha / (i + 1);
            let color = mxe << 24;
            
            self.draw_filled_rect(
                x + offset - wd,
                y + offset - wd,
                w + wd * 2.0,
                h + wd * 2.0,
                color,
                z - (i as f32 * 0.01),
            );
        }
    }
    
    
    fn draw_window_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        
        let ana = 32.0;
        let bwl = if surface.focused { 0xFF0D120F } else { 0xFF0A0D0B };
        self.draw_filled_rect(x, y, w, ana, bwl, z);
        
        
        if surface.focused {
            self.draw_filled_rect(x, y + ana - 2.0, w, 2.0, 0xFF008844, z + 0.01);
        }
        
        
        let kda = 0xFF0A0E0B;
        self.draw_filled_rect(x, y + ana, w, h - ana, kda, z);
        
        
        let ri = if surface.focused { 0xFF006633 } else { 0xFF004422 };
        self.draw_rect_outline(x, y, w, h, ri, z + 0.02);
        
        
        self.draw_window_controls(x + 8.0, y + 8.0, surface.focused, z + 0.03);
    }
    
    
    fn draw_glass_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        
        let alpha = (surface.opacity * 0.85 * 255.0) as u32;
        let mex = (alpha << 24) | 0x0D1210;
        self.draw_filled_rect(x, y, w, h, mex, z);
        
        
        self.draw_rect_outline(x, y, w, h, 0x4000FF66, z + 0.01);
        
        
        self.draw_filled_rect(x + 1.0, y + 1.0, w - 2.0, 1.0, 0x2000FF66, z + 0.02);
    }
    
    
    fn draw_neon_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        
        self.draw_filled_rect(x, y, w, h, 0xFF050505, z);
        
        
        let aog = if surface.focused { 0xFF00FF66 } else { 0xFF00AA44 };
        
        
        for i in 1..5 {
            let wd = i as f32 * 2.0;
            let alpha = (60 - i * 15) as u32;
            let color = (alpha << 24) | (aog & 0x00FFFFFF);
            self.draw_rect_outline(x - wd, y - wd, w + wd * 2.0, h + wd * 2.0, color, z + 0.01);
        }
        
        
        self.draw_rect_outline(x, y, w, h, aog, z + 0.05);
    }
    
    
    fn draw_minimal_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        
        self.draw_filled_rect(x, y, w, h, 0xFF0A0E0B, z);
        
        
        let ri = if surface.focused { 0xFF00CC55 } else { 0xFF004422 };
        self.draw_rect_outline(x, y, w, h, ri, z + 0.01);
    }
    
    
    fn draw_flat_frame(&self, surface: &WindowSurface, z: f32) {
        let x = surface.x;
        let y = surface.y;
        let w = surface.width * surface.scale;
        let h = surface.height * surface.scale;
        
        
        let ana = 32.0;
        self.draw_filled_rect(x, y, w, ana, 0xFF0D120F, z);
        
        
        self.draw_filled_rect(x, y + ana, w, h - ana, 0xFF0A0E0B, z);
        
        
        self.draw_rect_outline(x, y, w, h, 0xFF006633, z + 0.01);
    }
    
    
    fn draw_window_controls(&self, x: f32, y: f32, focused: bool, z: f32) {
        let spacing = 20.0;
        let radius = 6.0;
        
        
        let klg = if focused { 0xFF4A3535 } else { 0xFF2A2020 };
        self.draw_circle(x, y + 8.0, radius, klg, z);
        
        
        let nfi = if focused { 0xFF3A3A30 } else { 0xFF202010 };
        self.draw_circle(x + spacing, y + 8.0, radius, nfi, z);
        
        
        let ncs = if focused { 0xFF2A3A2F } else { 0xFF10201A };
        self.draw_circle(x + spacing * 2.0, y + 8.0, radius, ncs, z);
    }
    
    
    fn draw_surface_content(&self, surface: &WindowSurface, z: f32) {
        
        
        let x = surface.x;
        let y = surface.y + 32.0; 
        let w = surface.width * surface.scale;
        let h = (surface.height - 32.0) * surface.scale;
        
        
    }
    
    
    
    
    
    
    fn draw_filled_rect(&self, x: f32, y: f32, w: f32, h: f32, color: u32, z: f32) {
        let (r, g, b, a) = byg(color);
        
        aqw(KZ_);
        cae(r, g, b, a);
        dt(x, y, z);
        dt(x + w, y, z);
        dt(x + w, y + h, z);
        dt(x, y + h, z);
        aqx();
    }
    
    
    fn draw_gradient_rect(&self, x: f32, y: f32, w: f32, h: f32, 
                          top_color: u32, bottom_color: u32, z: f32) {
        let (uh, bbu, gf, eb) = byg(top_color);
        let (ju, axe, iq, fy) = byg(bottom_color);
        
        aqw(KZ_);
        cae(uh, bbu, gf, eb);
        dt(x, y, z);
        dt(x + w, y, z);
        cae(ju, axe, iq, fy);
        dt(x + w, y + h, z);
        dt(x, y + h, z);
        aqx();
    }
    
    
    fn draw_rect_outline(&self, x: f32, y: f32, w: f32, h: f32, color: u32, z: f32) {
        let (r, g, b, a) = byg(color);
        
        aqw(OD_);
        cae(r, g, b, a);
        dt(x, y, z);
        dt(x + w, y, z);
        dt(x + w, y + h, z);
        dt(x, y + h, z);
        aqx();
    }
    
    
    fn draw_circle(&self, cx: f32, u: f32, radius: f32, color: u32, z: f32) {
        let (r, g, b, a) = byg(color);
        let segments = 16;
        
        aqw(AVS_);
        cae(r, g, b, a);
        dt(cx, u, z); 
        
        for i in 0..=segments {
            let cc = (i as f32 / segments as f32) * core::f32::consts::PI * 2.0;
            let p = cx + cc.cos() * radius;
            let o = u + cc.sin() * radius;
            dt(p, o, z);
        }
        aqx();
    }
    
    
    fn draw_glow(&self, cx: f32, u: f32, radius: f32, color: u32, z: f32) {
        let (r, g, b, _) = byg(color);
        
        
        for i in 0..8 {
            let t = i as f32 / 8.0;
            let hpu = radius * (0.3 + t * 0.7);
            let alpha = 0.3 * (1.0 - t);
            
            aqw(AVS_);
            cae(r, g, b, alpha);
            dt(cx, u, z);
            
            let segments = 24;
            for ay in 0..=segments {
                let cc = (ay as f32 / segments as f32) * core::f32::consts::PI * 2.0;
                let p = cx + cc.cos() * hpu;
                let o = u + cc.sin() * hpu;
                dt(p, o, z);
            }
            aqx();
        }
    }
    
    
    fn draw_neon_glow(&self, x: f32, y: f32, w: f32, h: f32, z: f32) {
        let aog = 0x00FF66u32;
        let (r, g, b, _) = byg(aog);
        
        for i in 1..6 {
            let wd = i as f32 * 3.0;
            let alpha = 0.4 / (i as f32);
            
            aqw(OD_);
            cae(r, g, b, alpha);
            dt(x - wd, y - wd, z);
            dt(x + w + wd, y - wd, z);
            dt(x + w + wd, y + h + wd, z);
            dt(x - wd, y + h + wd, z);
            aqx();
        }
    }
    
    
    fn draw_blur_rect(&self, x: f32, y: f32, w: f32, h: f32, z: f32) {
        
        self.draw_filled_rect(x, y, w, h, 0x800D1210, z);
    }
    
    
    fn draw_grid(&self, z: f32) {
        let fzl = 0x08004422u32;
        let (r, g, b, a) = byg(fzl);
        let spacing = 40.0;
        
        aqw(UU_);
        cae(r, g, b, a);
        
        
        let mut x = 0.0;
        while x < self.width as f32 {
            dt(x, 0.0, z);
            dt(x, self.height as f32, z);
            x += spacing;
        }
        
        
        let mut y = 0.0;
        while y < self.height as f32 {
            dt(0.0, y, z);
            dt(self.width as f32, y, z);
            y += spacing;
        }
        aqx();
    }
    
    
    pub fn set_theme(&mut self, theme: CompositorTheme) {
        self.theme = theme;
        
        
        match theme {
            CompositorTheme::Modern => {
                self.shadow_opacity = 0.4;
                self.shadow_blur = 16.0;
                self.corner_radius = 8.0;
                self.border_glow = 0.0;
            }
            CompositorTheme::Glass => {
                self.shadow_opacity = 0.2;
                self.shadow_blur = 24.0;
                self.corner_radius = 12.0;
                self.border_glow = 0.3;
            }
            CompositorTheme::Neon => {
                self.shadow_opacity = 0.0;
                self.corner_radius = 4.0;
                self.border_glow = 1.0;
                self.bg_pattern = BackgroundPattern::Grid;
            }
            CompositorTheme::Minimal => {
                self.shadow_opacity = 0.0;
                self.shadow_blur = 0.0;
                self.corner_radius = 0.0;
                self.border_glow = 0.0;
            }
            CompositorTheme::Flat => {
                self.shadow_opacity = 0.0;
                self.shadow_blur = 0.0;
                self.corner_radius = 0.0;
                self.border_glow = 0.0;
            }
        }
    }
}






fn byg(color: u32) -> (f32, f32, f32, f32) {
    let a = ((color >> 24) & 0xFF) as f32 / 255.0;
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    (r, g, b, a)
}

use crate::math::lerp;


fn jwz(t: f32, easing: Easing) -> f32 {
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        Easing::Bounce => {
            let eur = 7.5625;
            let vh = 2.75;
            let mut t = t;
            if t < 1.0 / vh {
                eur * t * t
            } else if t < 2.0 / vh {
                t -= 1.5 / vh;
                eur * t * t + 0.75
            } else if t < 2.5 / vh {
                t -= 2.25 / vh;
                eur * t * t + 0.9375
            } else {
                t -= 2.625 / vh;
                eur * t * t + 0.984375
            }
        }
        Easing::Elastic => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let aa = 0.3;
                let j = aa / 4.0;
                (2.0f32).powf(-10.0 * t) * ((t - j) * (2.0 * core::f32::consts::PI / aa)).sin() + 1.0
            }
        }
    }
}





static Gg: Mutex<Compositor> = Mutex::new(Compositor::new());


pub fn compositor() -> spin::MutexGuard<'static, Compositor> {
    Gg.lock()
}


pub fn mpa(width: u32, height: u32) {
    compositor().init(width, height);
}


pub fn jez(theme: CompositorTheme) {
    compositor().set_theme(theme);
}


pub fn ofh() {
    compositor().render();
}
