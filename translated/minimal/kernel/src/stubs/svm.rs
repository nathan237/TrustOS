

use alloc::string::String;

pub fn gkj() -> bool { false }

pub struct SvmFeatures {
    pub afe: u8,
    pub fph: u32,
    pub npt: bool,
    pub lid: bool,
    pub mig: bool,
    pub evl: bool,
    pub mnj: bool,
    pub mpr: bool,
    pub hjy: bool,
    pub iqs: bool,
    pub ltc: bool,
    pub ltd: bool,
    pub gzk: bool,
    pub mpt: bool,
    pub mpf: bool,
}

pub fn fjn() -> SvmFeatures {
    SvmFeatures {
        afe: 0,
        fph: 0,
        npt: false,
        lid: false,
        mig: false,
        evl: false,
        mnj: false,
        mpr: false,
        hjy: false,
        iqs: false,
        ltc: false,
        ltd: false,
        gzk: false,
        mpt: false,
        mpf: false,
    }
}

pub mod vmcb {
    pub mod state_offsets {
        pub const Aw: usize = 0x178;
        pub const Hc: usize = 0x1D8;
        pub const Kv: usize = 0x170;
        pub const Vu: usize = 0x150;
        pub const Agy: usize = 0x140;
        pub const Vv: usize = 0x158;
        pub const Vw: usize = 0x160;
        pub const Lh: usize = 0x168;
        pub const JU_: usize = 0x100;
        pub const SJ_: usize = 0x108;
        pub const AAY_: usize = 0x110;
        pub const AAX_: usize = 0x114;
        pub const MV_: usize = 0x120;
        pub const XH_: usize = 0x130;
        pub const AID_: usize = 0x138;
        pub const AIE_: usize = 0x13C;
        pub const AIC_: usize = 0x13E;
        pub const SZ_: usize = 0x140;
        pub const ACL_: usize = 0x148;
        pub const ADF_: usize = 0x150;
        pub const Agx: usize = 0x14B;
    }

    pub mod control_offsets {
        pub const Abm: usize = 0x070;
        pub const Abn: usize = 0x078;
        pub const Abo: usize = 0x080;
    }
}
