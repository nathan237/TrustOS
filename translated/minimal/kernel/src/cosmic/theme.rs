




use super::Color;









pub mod matrix {
    use super::Color;
    
    
    
    
    
    
    
    pub const SO_: Color = Color::rgb(0.020, 0.031, 0.020);
    
    pub const CJ_: Color = Color::rgb(0.027, 0.059, 0.027);
    
    pub const DHH_: Color = Color::new(0.0, 1.0, 0.47, 0.04);
    
    pub const BNU_: Color = Color::new(0.0, 1.0, 0.47, 0.035);
    
    
    
    pub const UX_: Color = Color::rgb(0.0, 1.0, 0.533);
    
    pub const AVY_: Color = Color::rgb(0.0, 1.0, 0.4);
    
    pub const AVW_: Color = Color::rgb(0.2, 1.0, 0.6);
    
    pub const Y_: Color = Color::rgb(0.0, 0.6, 0.32);
    
    pub const AVX_: Color = Color::rgb(0.0, 0.35, 0.18);
    
    
    
    pub const AAZ_: Color = Color::new(0.0, 1.0, 0.47, 0.22);
    
    pub const SR_: Color = Color::new(0.0, 1.0, 0.47, 0.35);
    
    pub const DRE_: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    
    pub const DRF_: Color = Color::new(0.0, 1.0, 0.47, 0.28);
    
    
    
    
    
    
    pub const DK_: Color = SO_;
    pub const CY_: Color = CJ_;
    pub const SL_: Color = Color::rgb(0.035, 0.075, 0.035);
    pub const SN_: Color = Color::new(0.0, 1.0, 0.47, 0.15);
    
    
    pub const El: Color = BNU_;
    pub const JT_: Color = Color::new(0.0, 1.0, 0.47, 0.08);
    pub const QR_: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    
    
    pub const Ch: Color = UX_;
    pub const RR_: Color = AVY_;
    pub const RS_: Color = Y_;
    
    
    pub const DWY_: Color = AVY_;
    pub const DWW_: Color = Y_;
    pub const DWV_: Color = AVW_;
    pub const DWX_: Color = AVX_;
    
    
    pub const AB_: Color = UX_;      
    pub const O_: Color = Y_;   
    pub const QW_: Color = AVX_;      
    pub const QV_: Color = AVW_;     
    
    
    pub const MY_: Color = Color::new(0.0, 1.0, 0.47, 0.06);
    pub const SW_: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    pub const SX_: Color = Color::new(0.0, 1.0, 0.47, 0.18);
    pub const MZ_: Color = UX_;
    pub const KE_: Color = Color::rgb(0.6, 0.2, 0.2); 
    
    
    
    pub const HF_: Color = Color::new(0.0, 1.0, 0.47, 0.06);
    
    pub const DSV_: Color = Color::new(0.0, 1.0, 0.47, 0.01);
    pub const AXX_: Color = AAZ_;
    
    
    pub const IH_: Color = Color::rgb(0.45, 0.25, 0.25);      
    pub const BPF_: Color = Color::rgb(0.55, 0.30, 0.30);
    pub const JB_: Color = Color::rgb(0.25, 0.40, 0.30);   
    pub const DWZ_: Color = Color::rgb(0.30, 0.50, 0.35);
    pub const JE_: Color = Color::rgb(0.35, 0.35, 0.25);   
    pub const DXV_: Color = Color::rgb(0.45, 0.45, 0.30);
    
    
    
    pub const LO_: Color = Color::new(0.0, 1.0, 0.47, 0.05);
    pub const XJ_: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    pub const ECG_: Color = Color::new(0.0, 1.0, 0.47, 0.18);
    
    
    pub const Bp: Color = AAZ_;
    
    
    pub const Nh: Color = Color::rgb(0.2, 0.65, 0.4);   
    pub const Nw: Color = Color::rgb(0.7, 0.6, 0.25);   
    pub const Hr: Color = Color::rgb(0.6, 0.3, 0.3);      
    pub const Zw: Color = UX_;
    
    
    
    
    
    
    pub const EOB_: f32 = 14.0;
    
    pub const DHY_: f32 = 8.0;
    
    pub const ECI_: f32 = 28.0;
    
    pub const OP_: f32 = 10.0;
    
    
    pub const DSW_: f32 = 28.0;
    
    pub const DNE_: f32 = 56.0;
    
    pub const BV_: f32 = 44.0;
    
    
    pub const DGI_: u32 = 120;
    pub const DGJ_: u32 = 180;
    
    
    pub const DHL_: f32 = 18.0;
    
    
    pub const DTE_: f32 = 1.04;
    
    pub const DNF_: f32 = 1.10;
}






pub mod dark {
    use super::Color;
    
    
    pub const DK_: Color = super::matrix::SO_;
    pub const CY_: Color = super::matrix::CJ_;
    pub const SL_: Color = super::matrix::SL_;
    pub const SN_: Color = super::matrix::SN_;
    
    
    pub const El: Color = super::matrix::El;
    pub const JT_: Color = super::matrix::JT_;
    pub const QR_: Color = super::matrix::QR_;
    
    
    pub const Ch: Color = super::matrix::Ch;
    pub const RR_: Color = super::matrix::RR_;
    pub const RS_: Color = super::matrix::RS_;
    
    
    pub const AB_: Color = super::matrix::AB_;
    pub const O_: Color = super::matrix::O_;
    pub const QW_: Color = super::matrix::QW_;
    
    
    pub const MY_: Color = super::matrix::MY_;
    pub const SW_: Color = super::matrix::SW_;
    pub const SX_: Color = super::matrix::SX_;
    pub const MZ_: Color = super::matrix::MZ_;
    pub const KE_: Color = super::matrix::KE_;
    
    
    pub const HF_: Color = super::matrix::HF_;
    pub const AXX_: Color = super::matrix::AXX_;
    pub const IH_: Color = super::matrix::IH_;
    pub const JB_: Color = super::matrix::JB_;
    pub const JE_: Color = super::matrix::JE_;
    
    
    pub const LO_: Color = super::matrix::LO_;
    pub const XJ_: Color = super::matrix::XJ_;
    
    
    pub const Bp: Color = super::matrix::Bp;
    pub const SR_: Color = super::matrix::SR_;
    
    
    pub const Nh: Color = super::matrix::Nh;
    pub const Nw: Color = super::matrix::Nw;
    pub const Hr: Color = super::matrix::Hr;
    pub const Zw: Color = super::matrix::Zw;
}


pub mod light {
    use super::Color;
    
    pub const DK_: Color = Color::rgb(0.976, 0.976, 0.976);
    pub const CY_: Color = Color::rgb(0.949, 0.949, 0.949);
    pub const El: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const Ch: Color = Color::rgb(0.0, 0.7, 0.45);
    pub const AB_: Color = Color::rgb(0.1, 0.15, 0.1);
    pub const O_: Color = Color::rgb(0.3, 0.35, 0.3);
}






#[derive(Clone, Copy)]
pub struct CosmicTheme {
    
    pub bg_base: Color,
    pub bg_component: Color,
    pub bg_container: Color,
    pub bg_divider: Color,
    
    
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_pressed: Color,
    
    
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    
    
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    
    
    pub button_bg: Color,
    pub button_hover: Color,
    pub button_pressed: Color,
    pub button_suggested: Color,
    pub button_destructive: Color,
    
    
    pub header_bg: Color,
    pub close_bg: Color,
    pub maximize_bg: Color,
    pub minimize_bg: Color,
    
    
    pub panel_bg: Color,
    pub panel_hover: Color,
    
    
    pub border: Color,
    pub border_focused: Color,
    pub corner_radius: f32,
    pub spacing: f32,
    pub padding: f32,
}

impl CosmicTheme {
    
    pub const fn dark() -> Self {
        Self {
            bg_base: dark::DK_,
            bg_component: dark::CY_,
            bg_container: dark::SL_,
            bg_divider: dark::SN_,
            surface: dark::El,
            surface_hover: dark::JT_,
            surface_pressed: dark::QR_,
            accent: dark::Ch,
            accent_hover: dark::RR_,
            accent_pressed: dark::RS_,
            text_primary: dark::AB_,
            text_secondary: dark::O_,
            text_disabled: dark::QW_,
            button_bg: dark::MY_,
            button_hover: dark::SW_,
            button_pressed: dark::SX_,
            button_suggested: dark::MZ_,
            button_destructive: dark::KE_,
            header_bg: dark::HF_,
            close_bg: dark::IH_,
            maximize_bg: dark::JB_,
            minimize_bg: dark::JE_,
            panel_bg: dark::LO_,
            panel_hover: dark::XJ_,
            border: dark::Bp,
            border_focused: dark::SR_,
            corner_radius: 8.0,
            spacing: 8.0,
            padding: 12.0,
        }
    }
    
    
    pub const fn light() -> Self {
        Self {
            bg_base: light::DK_,
            bg_component: light::CY_,
            bg_container: light::CY_,
            bg_divider: light::CY_,
            surface: light::El,
            surface_hover: light::CY_,
            surface_pressed: light::DK_,
            accent: light::Ch,
            accent_hover: light::Ch,
            accent_pressed: light::Ch,
            text_primary: light::AB_,
            text_secondary: light::O_,
            text_disabled: light::O_,
            button_bg: light::CY_,
            button_hover: light::DK_,
            button_pressed: light::CY_,
            button_suggested: light::Ch,
            button_destructive: dark::KE_,
            header_bg: light::El,
            close_bg: dark::IH_,
            maximize_bg: dark::JB_,
            minimize_bg: dark::JE_,
            panel_bg: light::DK_,
            panel_hover: light::CY_,
            border: light::CY_,
            border_focused: light::Ch,
            corner_radius: 8.0,
            spacing: 8.0,
            padding: 12.0,
        }
    }
    
    
    pub const fn matrix() -> Self {
        Self {
            bg_base: matrix::DK_,
            bg_component: matrix::CY_,
            bg_container: matrix::SL_,
            bg_divider: matrix::SN_,
            surface: matrix::El,
            surface_hover: matrix::JT_,
            surface_pressed: matrix::QR_,
            accent: matrix::Ch,
            accent_hover: matrix::RR_,
            accent_pressed: matrix::RS_,
            text_primary: matrix::AB_,
            text_secondary: matrix::O_,
            text_disabled: matrix::QW_,
            button_bg: matrix::MY_,
            button_hover: matrix::SW_,
            button_pressed: matrix::SX_,
            button_suggested: matrix::MZ_,
            button_destructive: matrix::KE_,
            header_bg: matrix::HF_,
            close_bg: matrix::IH_,
            maximize_bg: matrix::JB_,
            minimize_bg: matrix::JE_,
            panel_bg: matrix::LO_,
            panel_hover: matrix::XJ_,
            border: matrix::Bp,
            border_focused: matrix::SR_,
            corner_radius: 4.0,  
            spacing: 6.0,
            padding: 8.0,
        }
    }
}

impl Default for CosmicTheme {
    fn default() -> Self {
        Self::dark()
    }
}


static mut NP_: CosmicTheme = CosmicTheme::dark();


pub fn theme() -> &'static CosmicTheme {
    unsafe { &NP_ }
}


pub fn set_theme(t: CosmicTheme) {
    unsafe { NP_ = t; }
}
