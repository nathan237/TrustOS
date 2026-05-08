//! Userland stub for non-x86_64 architectures
//!
//! Ring 3 support requires SYSCALL/SYSRET (x86_64) or equivalent.

use core::sync::atomic::AtomicU64;

pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USER_STACK_TOP: u64 = 0x0000_7FFF_FFFF_0000;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USER_STACK_SIZE: usize = 1024 * 1024;
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USER_CODE_BASE: u64 = 0x0000_0000_0040_0000;

pub static mut USER_RSP_TEMP: u64 = 0;
pub static mut USER_RETURN_RIP: u64 = 0;
pub static mut USER_RETURN_RFLAGS: u64 = 0;
pub static mut SIGNAL_DELIVER_SIGNO: u64 = 0;
pub static mut KERNEL_SYSCALL_STACK_TOP: u64 = 0;
pub // Variable atomique — accès thread-safe sans verrou.
static WAITING_KERNEL_TID: AtomicU64 = AtomicU64::new(0);

// Fonction publique — appelable depuis d'autres modules.
pub fn init() {}

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn jump_to_ring3(_entry_point: u64, _user_stack: u64) -> ! {
    panic!("jump_to_ring3 not implemented for this architecture");
}

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn jump_to_ring3_with_args(
    _entry_point: u64, _user_stack: u64, _arg1: u64, _arg2: u64,
) -> ! {
    panic!("jump_to_ring3_with_args not implemented for this architecture");
}

// Fonction publique — appelable depuis d'autres modules.
pub fn initialize_syscall_stack() {}

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn execute_ring3_process(_entry_point: u64, _user_stack: u64) -> i32 {
    -1 // Not supported
}

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn return_from_ring3(_exit_code: i32) -> ! {
    panic!("return_from_ring3 not implemented for this architecture");
}

// Fonction publique — appelable depuis d'autres modules.
pub fn is_process_active() -> bool { false }

pub // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn launch_user_process(_entry_point: u64, _user_stack: u64) -> i32 {
    -1 // Not supported
}

// Fonction publique — appelable depuis d'autres modules.
pub fn user_thread_exit(_exit_code: i32) {}
