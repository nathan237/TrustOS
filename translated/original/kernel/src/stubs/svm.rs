//! SVM stub for non-x86_64 architectures

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

pub fn get_features() -> SvmFeatures {
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
        pub const RIP: usize = 0x178;
        pub const RSP: usize = 0x1D8;
        pub const RFLAGS: usize = 0x170;
        pub const CR0: usize = 0x150;
        pub const CR2: usize = 0x140;
        pub const CR3: usize = 0x158;
        pub const CR4: usize = 0x160;
        pub const EFER: usize = 0x168;
        pub const CS_SELECTOR: usize = 0x100;
        pub const CS_BASE: usize = 0x108;
        pub const CS_LIMIT: usize = 0x110;
        pub const CS_ATTRIB: usize = 0x114;
        pub const DS_SELECTOR: usize = 0x120;
        pub const SS_SELECTOR: usize = 0x130;
        pub const SS_BASE: usize = 0x138;
        pub const SS_LIMIT: usize = 0x13C;
        pub const SS_ATTRIB: usize = 0x13E;
        pub const ES_SELECTOR: usize = 0x140;
        pub const FS_SELECTOR: usize = 0x148;
        pub const GS_SELECTOR: usize = 0x150;
        pub const CPL: usize = 0x14B;
    }

    pub mod control_offsets {
        pub const EXITCODE: usize = 0x070;
        pub const EXITINFO1: usize = 0x078;
        pub const EXITINFO2: usize = 0x080;
    }
}
