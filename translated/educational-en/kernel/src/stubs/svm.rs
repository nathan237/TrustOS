//! SVM stub for non-x86_64 architectures

use alloc::string::String;

// Public function — callable from other modules.
pub fn is_supported() -> bool { false }

// Public structure — visible outside this module.
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

// Public function — callable from other modules.
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
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const RIP: usize = 0x178;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const RSP: usize = 0x1D8;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const RFLAGS: usize = 0x170;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CR0: usize = 0x150;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CR2: usize = 0x140;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CR3: usize = 0x158;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CR4: usize = 0x160;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const EFER: usize = 0x168;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CS_SELECTOR: usize = 0x100;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CS_BASE: usize = 0x108;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CS_LIMIT: usize = 0x110;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CS_ATTRIB: usize = 0x114;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const DS_SELECTOR: usize = 0x120;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const SS_SELECTOR: usize = 0x130;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const SS_BASE: usize = 0x138;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const SS_LIMIT: usize = 0x13C;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const SS_ATTRIB: usize = 0x13E;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const ES_SELECTOR: usize = 0x140;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const FILESYSTEM_SELECTOR: usize = 0x148;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const GS_SELECTOR: usize = 0x150;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const CPL: usize = 0x14B;
    }

    pub mod control_offsets {
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const EXITCODE: usize = 0x070;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const EXITINFO1: usize = 0x078;
        pub         // Compile-time constant — evaluated at compilation, zero runtime cost.
const EXITINFO2: usize = 0x080;
    }
}
