//! SVM stub for non-x86_64 architectures

use alloc::string::String;

// Fonction publique — appelable depuis d'autres modules.
pub fn is_supported() -> bool { false }

// Structure publique — visible à l'extérieur de ce module.
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

// Fonction publique — appelable depuis d'autres modules.
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
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RIP: usize = 0x178;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RSP: usize = 0x1D8;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RFLAGS: usize = 0x170;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CR0: usize = 0x150;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CR2: usize = 0x140;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CR3: usize = 0x158;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CR4: usize = 0x160;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EFER: usize = 0x168;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CS_SELECTOR: usize = 0x100;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CS_BASE: usize = 0x108;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CS_LIMIT: usize = 0x110;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CS_ATTRIB: usize = 0x114;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DS_SELECTOR: usize = 0x120;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SS_SELECTOR: usize = 0x130;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SS_BASE: usize = 0x138;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SS_LIMIT: usize = 0x13C;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SS_ATTRIB: usize = 0x13E;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ES_SELECTOR: usize = 0x140;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FILESYSTEM_SELECTOR: usize = 0x148;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const GS_SELECTOR: usize = 0x150;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CPL: usize = 0x14B;
    }

    pub mod control_offsets {
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXITCODE: usize = 0x070;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXITINFO1: usize = 0x078;
        pub         // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EXITINFO2: usize = 0x080;
    }
}
