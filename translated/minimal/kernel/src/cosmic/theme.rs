




use super::Color;









pub mod matrix {
    use super::Color;
    
    
    
    
    
    
    
    pub const RM_: Color = Color::xt(0.020, 0.031, 0.020);
    
    pub const HN_: Color = Color::xt(0.027, 0.059, 0.027);
    
    pub const DDN_: Color = Color::new(0.0, 1.0, 0.47, 0.04);
    
    pub const BLC_: Color = Color::new(0.0, 1.0, 0.47, 0.035);
    
    
    
    pub const TR_: Color = Color::xt(0.0, 1.0, 0.533);
    
    pub const ATU_: Color = Color::xt(0.0, 1.0, 0.4);
    
    pub const ATS_: Color = Color::xt(0.2, 1.0, 0.6);
    
    pub const X_: Color = Color::xt(0.0, 0.6, 0.32);
    
    pub const ATT_: Color = Color::xt(0.0, 0.35, 0.18);
    
    
    
    pub const ZO_: Color = Color::new(0.0, 1.0, 0.47, 0.22);
    
    pub const RP_: Color = Color::new(0.0, 1.0, 0.47, 0.35);
    
    pub const DNK_: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    
    pub const DNL_: Color = Color::new(0.0, 1.0, 0.47, 0.28);
    
    
    
    
    
    
    pub const DD_: Color = RM_;
    pub const CS_: Color = HN_;
    pub const RJ_: Color = Color::xt(0.035, 0.075, 0.035);
    pub const RL_: Color = Color::new(0.0, 1.0, 0.47, 0.15);
    
    
    pub const Kw: Color = BLC_;
    pub const JA_: Color = Color::new(0.0, 1.0, 0.47, 0.08);
    pub const PU_: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    
    
    pub const Ge: Color = TR_;
    pub const QW_: Color = ATU_;
    pub const QX_: Color = X_;
    
    
    pub const DTG_: Color = ATU_;
    pub const DTE_: Color = X_;
    pub const DTD_: Color = ATS_;
    pub const DTF_: Color = ATT_;
    
    
    pub const AC_: Color = TR_;      
    pub const N_: Color = X_;   
    pub const PZ_: Color = ATT_;      
    pub const PY_: Color = ATS_;     
    
    
    pub const MA_: Color = Color::new(0.0, 1.0, 0.47, 0.06);
    pub const RU_: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    pub const RV_: Color = Color::new(0.0, 1.0, 0.47, 0.18);
    pub const MB_: Color = TR_;
    pub const JN_: Color = Color::xt(0.6, 0.2, 0.2); 
    
    
    
    pub const GO_: Color = Color::new(0.0, 1.0, 0.47, 0.06);
    
    pub const DPB_: Color = Color::new(0.0, 1.0, 0.47, 0.01);
    pub const AVT_: Color = ZO_;
    
    
    pub const HP_: Color = Color::xt(0.45, 0.25, 0.25);      
    pub const BMN_: Color = Color::xt(0.55, 0.30, 0.30);
    pub const II_: Color = Color::xt(0.25, 0.40, 0.30);   
    pub const DTH_: Color = Color::xt(0.30, 0.50, 0.35);
    pub const IL_: Color = Color::xt(0.35, 0.35, 0.25);   
    pub const DUE_: Color = Color::xt(0.45, 0.45, 0.30);
    
    
    
    pub const KV_: Color = Color::new(0.0, 1.0, 0.47, 0.05);
    pub const WA_: Color = Color::new(0.0, 1.0, 0.47, 0.12);
    pub const DYP_: Color = Color::new(0.0, 1.0, 0.47, 0.18);
    
    
    pub const Fj: Color = ZO_;
    
    
    pub const Aep: Color = Color::xt(0.2, 0.65, 0.4);   
    pub const Afq: Color = Color::xt(0.7, 0.6, 0.25);   
    pub const Sf: Color = Color::xt(0.6, 0.3, 0.3);      
    pub const Bjd: Color = TR_;
    
    
    
    
    
    
    pub const EKN_: f32 = 14.0;
    
    pub const DEE_: f32 = 8.0;
    
    pub const DYR_: f32 = 28.0;
    
    pub const NO_: f32 = 10.0;
    
    
    pub const DPC_: f32 = 28.0;
    
    pub const DJQ_: f32 = 56.0;
    
    pub const BU_: f32 = 44.0;
    
    
    pub const DCO_: u32 = 120;
    pub const DCP_: u32 = 180;
    
    
    pub const DDR_: f32 = 18.0;
    
    
    pub const DPK_: f32 = 1.04;
    
    pub const DJR_: f32 = 1.10;
}






pub mod dark {
    use super::Color;
    
    
    pub const DD_: Color = super::matrix::RM_;
    pub const CS_: Color = super::matrix::HN_;
    pub const RJ_: Color = super::matrix::RJ_;
    pub const RL_: Color = super::matrix::RL_;
    
    
    pub const Kw: Color = super::matrix::Kw;
    pub const JA_: Color = super::matrix::JA_;
    pub const PU_: Color = super::matrix::PU_;
    
    
    pub const Ge: Color = super::matrix::Ge;
    pub const QW_: Color = super::matrix::QW_;
    pub const QX_: Color = super::matrix::QX_;
    
    
    pub const AC_: Color = super::matrix::AC_;
    pub const N_: Color = super::matrix::N_;
    pub const PZ_: Color = super::matrix::PZ_;
    
    
    pub const MA_: Color = super::matrix::MA_;
    pub const RU_: Color = super::matrix::RU_;
    pub const RV_: Color = super::matrix::RV_;
    pub const MB_: Color = super::matrix::MB_;
    pub const JN_: Color = super::matrix::JN_;
    
    
    pub const GO_: Color = super::matrix::GO_;
    pub const AVT_: Color = super::matrix::AVT_;
    pub const HP_: Color = super::matrix::HP_;
    pub const II_: Color = super::matrix::II_;
    pub const IL_: Color = super::matrix::IL_;
    
    
    pub const KV_: Color = super::matrix::KV_;
    pub const WA_: Color = super::matrix::WA_;
    
    
    pub const Fj: Color = super::matrix::Fj;
    pub const RP_: Color = super::matrix::RP_;
    
    
    pub const Aep: Color = super::matrix::Aep;
    pub const Afq: Color = super::matrix::Afq;
    pub const Sf: Color = super::matrix::Sf;
    pub const Bjd: Color = super::matrix::Bjd;
}


pub mod light {
    use super::Color;
    
    pub const DD_: Color = Color::xt(0.976, 0.976, 0.976);
    pub const CS_: Color = Color::xt(0.949, 0.949, 0.949);
    pub const Kw: Color = Color::xt(1.0, 1.0, 1.0);
    pub const Ge: Color = Color::xt(0.0, 0.7, 0.45);
    pub const AC_: Color = Color::xt(0.1, 0.15, 0.1);
    pub const N_: Color = Color::xt(0.3, 0.35, 0.3);
}






#[derive(Clone, Copy)]
pub struct CosmicTheme {
    
    pub kcu: Color,
    pub kcv: Color,
    pub kcw: Color,
    pub hai: Color,
    
    
    pub surface: Color,
    pub dwl: Color,
    pub fvv: Color,
    
    
    pub mm: Color,
    pub cof: Color,
    pub fzq: Color,
    
    
    pub dcp: Color,
    pub dwr: Color,
    pub fwn: Color,
    
    
    pub dop: Color,
    pub dor: Color,
    pub dzj: Color,
    pub imm: Color,
    pub iml: Color,
    
    
    pub fkk: Color,
    pub enp: Color,
    pub jfn: Color,
    pub jgd: Color,
    
    
    pub fqd: Color,
    pub jin: Color,
    
    
    pub acu: Color,
    pub dzc: Color,
    pub avn: f32,
    pub aoa: f32,
    pub ob: f32,
}

impl CosmicTheme {
    
    pub const fn dark() -> Self {
        Self {
            kcu: dark::DD_,
            kcv: dark::CS_,
            kcw: dark::RJ_,
            hai: dark::RL_,
            surface: dark::Kw,
            dwl: dark::JA_,
            fvv: dark::PU_,
            mm: dark::Ge,
            cof: dark::QW_,
            fzq: dark::QX_,
            dcp: dark::AC_,
            dwr: dark::N_,
            fwn: dark::PZ_,
            dop: dark::MA_,
            dor: dark::RU_,
            dzj: dark::RV_,
            imm: dark::MB_,
            iml: dark::JN_,
            fkk: dark::GO_,
            enp: dark::HP_,
            jfn: dark::II_,
            jgd: dark::IL_,
            fqd: dark::KV_,
            jin: dark::WA_,
            acu: dark::Fj,
            dzc: dark::RP_,
            avn: 8.0,
            aoa: 8.0,
            ob: 12.0,
        }
    }
    
    
    pub const fn light() -> Self {
        Self {
            kcu: light::DD_,
            kcv: light::CS_,
            kcw: light::CS_,
            hai: light::CS_,
            surface: light::Kw,
            dwl: light::CS_,
            fvv: light::DD_,
            mm: light::Ge,
            cof: light::Ge,
            fzq: light::Ge,
            dcp: light::AC_,
            dwr: light::N_,
            fwn: light::N_,
            dop: light::CS_,
            dor: light::DD_,
            dzj: light::CS_,
            imm: light::Ge,
            iml: dark::JN_,
            fkk: light::Kw,
            enp: dark::HP_,
            jfn: dark::II_,
            jgd: dark::IL_,
            fqd: light::DD_,
            jin: light::CS_,
            acu: light::CS_,
            dzc: light::Ge,
            avn: 8.0,
            aoa: 8.0,
            ob: 12.0,
        }
    }
    
    
    pub const fn matrix() -> Self {
        Self {
            kcu: matrix::DD_,
            kcv: matrix::CS_,
            kcw: matrix::RJ_,
            hai: matrix::RL_,
            surface: matrix::Kw,
            dwl: matrix::JA_,
            fvv: matrix::PU_,
            mm: matrix::Ge,
            cof: matrix::QW_,
            fzq: matrix::QX_,
            dcp: matrix::AC_,
            dwr: matrix::N_,
            fwn: matrix::PZ_,
            dop: matrix::MA_,
            dor: matrix::RU_,
            dzj: matrix::RV_,
            imm: matrix::MB_,
            iml: matrix::JN_,
            fkk: matrix::GO_,
            enp: matrix::HP_,
            jfn: matrix::II_,
            jgd: matrix::IL_,
            fqd: matrix::KV_,
            jin: matrix::WA_,
            acu: matrix::Fj,
            dzc: matrix::RP_,
            avn: 4.0,  
            aoa: 6.0,
            ob: 8.0,
        }
    }
}

impl Default for CosmicTheme {
    fn default() -> Self {
        Self::dark()
    }
}


static mut MR_: CosmicTheme = CosmicTheme::dark();


pub fn theme() -> &'static CosmicTheme {
    unsafe { &MR_ }
}


pub fn bxb(ab: CosmicTheme) {
    unsafe { MR_ = ab; }
}
