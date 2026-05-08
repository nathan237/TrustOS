

use alloc::string::String;

pub fn is_supported() -> bool { false }

pub struct SvmFeatures {
    pub revision: u8,
    pub num_asids: u32,
    pub npt: bool,
    pub lbr_virt: bool,
    pub svm_lock: bool,
    pub nrip_save: bool,
    pub tsc_rate_msr: bool,
    pub vmcb_clean: bool,
    pub flush_by_asid: bool,
    pub decode_assists: bool,
    pub pause_filter: bool,
    pub pause_filter_thresh: bool,
    pub avic: bool,
    pub vmsave_virt: bool,
    pub vgif: bool,
}

pub fn ckb() -> SvmFeatures {
    SvmFeatures {
        revision: 0,
        num_asids: 0,
        npt: false,
        lbr_virt: false,
        svm_lock: false,
        nrip_save: false,
        tsc_rate_msr: false,
        vmcb_clean: false,
        flush_by_asid: false,
        decode_assists: false,
        pause_filter: false,
        pause_filter_thresh: false,
        avic: false,
        vmsave_virt: false,
        vgif: false,
    }
}

pub mod vmcb {
    pub mod state_offsets {
        pub const Af: usize = 0x178;
        pub const De: usize = 0x1D8;
        pub const Ek: usize = 0x170;
        pub const Jn: usize = 0x150;
        pub const Og: usize = 0x140;
        pub const Jo: usize = 0x158;
        pub const Jp: usize = 0x160;
        pub const Eu: usize = 0x168;
        pub const KO_: usize = 0x100;
        pub const TP_: usize = 0x108;
        pub const ACO_: usize = 0x110;
        pub const ACN_: usize = 0x114;
        pub const NT_: usize = 0x120;
        pub const YO_: usize = 0x130;
        pub const AJZ_: usize = 0x138;
        pub const AKA_: usize = 0x13C;
        pub const AJY_: usize = 0x13E;
        pub const UF_: usize = 0x140;
        pub const AEB_: usize = 0x148;
        pub const AEV_: usize = 0x150;
        pub const Of: usize = 0x14B;
    }

    pub mod control_offsets {
        pub const Lv: usize = 0x070;
        pub const Lx: usize = 0x078;
        pub const Ly: usize = 0x080;
    }
}
