//! PXE Self-Replication Module
//!
//! Orchestrates the self-replication of TrustOS across the network via PXE boot.
//! When activated, this module:
//! 1. Accesses the kernel binary from Limine's kernel_file data in memory
//! 2. Starts a DHCP server with PXE boot options
//! 3. Starts a TFTP server to serve boot files
//! 4. Monitors for new nodes joining the mesh
//!
//! Multi-architecture support:
//! The running kernel is served under an arch-specific name (trustos_x86_64, etc.)
//! and also as the generic `trustos_kernel` for same-arch clients.
//! Remote kernels can be registered by peers via RPC for cross-arch PXE boot.
//!
//! Boot files served:
//! - limine-bios-pxe.bin (embedded at compile time from limine/)
//! - limine.conf (generated dynamically)
//! - trustos_kernel / trustos_<arch> (from Limine's kernel_file or remote peers)

use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use super::mesh::CpuArch;

/// Whether self-replication is active
static REPLICATING: AtomicBool = AtomicBool::new(false);

/// Number of nodes that have successfully booted via PXE
static NODES_BOOTED: AtomicU8 = AtomicU8::new(0);

/// Embedded limine-bios-pxe.bin bootloader for PXE boot
/// This is the Limine PXE binary that clients download first via TFTP
static LIMINE_PXE_BIN: &[u8] = include_bytes!("../../../limine/limine-bios-pxe.bin");

/// Embedded limine.conf for PXE-booted clients
/// Minimal config that tells Limine to load the kernel from TFTP
static LIMINE_CONF: &[u8] = b"timeout: 0\n\n/TrustOS\n    protocol: limine\n    kernel_path: tftp():/trustos_kernel\n";

/// Wrapper to make kernel file pointer Send+Sync safe
/// The pointer comes from Limine's boot data which is static for kernel lifetime
struct KernelFileRef {
    ptr: usize,  // Store as usize to avoid Send issues
    size: usize,
}

unsafe impl Send for KernelFileRef {}
unsafe impl Sync for KernelFileRef {}

/// Pointer to the kernel binary in memory (set by main.rs from KernelFileRequest)
static KERNEL_FILE_REF: spin::Mutex<Option<KernelFileRef>> = spin::Mutex::new(None);

/// Register the kernel file location from Limine's KernelFileRequest response
///
/// # Safety
/// The pointer must be valid for the lifetime of the kernel and point to
/// the kernel ELF binary as loaded by Limine.
pub unsafe fn register_kernel_file(ptr: *const u8, size: usize) {
    let mut kf = KERNEL_FILE_REF.lock();
    *kf = Some(KernelFileRef { ptr: ptr as usize, size });
    crate::serial_println!("[PXE-REPL] Kernel file registered: {:p}, {} bytes ({} KB)",
        ptr, size, size / 1024);
}

/// Get the kernel binary as a static slice (if registered)
fn get_kernel_data() -> Option<&'static [u8]> {
    let kf = KERNEL_FILE_REF.lock();
    kf.as_ref().map(|r| unsafe { core::slice::from_raw_parts(r.ptr as *const u8, r.size) })
}

/// Check if replication is active
pub fn is_active() -> bool {
    REPLICATING.load(Ordering::Relaxed)
}

/// Get number of PXE-booted nodes
pub fn nodes_booted() -> u8 {
    NODES_BOOTED.load(Ordering::Relaxed)
}

/// Start PXE self-replication
///
/// This starts DHCP and TFTP servers configured for PXE boot,
/// serving the currently running kernel to new machines on the network.
pub fn start() -> Result<(), &'static str> {
    if REPLICATING.load(Ordering::Relaxed) {
        return Err("Already replicating");
    }

    // Verify kernel binary is available
    let kernel_data = get_kernel_data().ok_or("Kernel file not registered (KernelFileRequest missing)")?;

    // Get our IP configuration
    let (our_ip, subnet, _gw) = crate::network::get_ipv4_config()
        .ok_or("No IP configuration — run DHCP or set static IP first")?;

    let ip = *our_ip.as_bytes();
    let mask = *subnet.as_bytes();

    // Calculate pool: start from our_ip + 10
    let pool_start = [ip[0], ip[1], ip[2], ip[3].wrapping_add(10)];

    crate::serial_println!("[PXE-REPL] === TrustOS Self-Replication via PXE ===");
    crate::serial_println!("[PXE-REPL] Server IP: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    crate::serial_println!("[PXE-REPL] Kernel size: {} bytes ({} KB)", kernel_data.len(), kernel_data.len() / 1024);
    crate::serial_println!("[PXE-REPL] Limine PXE: {} bytes", LIMINE_PXE_BIN.len());
    crate::serial_println!("[PXE-REPL] Limine conf: {} bytes", LIMINE_CONF.len());

    // Register boot files with TFTP server
    // The PXE boot filename in DHCP points to limine-bios-pxe.bin
    crate::netstack::tftpd::register_file("limine-bios-pxe.bin", LIMINE_PXE_BIN);
    crate::netstack::tftpd::register_file("limine.conf", LIMINE_CONF);

    // Register the kernel binary — this is the running kernel serving itself
    // Safety: kernel_data is a static reference to memory mapped by Limine
    crate::netstack::tftpd::register_file("trustos_kernel", kernel_data);

    // Register under arch-specific name for multi-arch PXE
    let arch_name = CpuArch::current().name();
    let arch_kernel_name: &'static str = alloc::string::String::from(format!("trustos_{}", arch_name)).leak();
    crate::netstack::tftpd::register_file(arch_kernel_name, kernel_data);
    crate::serial_println!("[PXE-REPL] Serving kernel as: trustos_kernel + {}", arch_kernel_name);

    // Generate arch-specific Limine configs for each known architecture
    // Clients requesting a specific arch config get directed to the right kernel
    for arch in &[CpuArch::X86_64, CpuArch::Aarch64, CpuArch::Riscv64] {
        let conf = format!(
            "timeout: 0\n\n/TrustOS ({})\n    protocol: limine\n    kernel_path: tftp():/trustos_{}\n",
            arch.name(), arch.name()
        );
        let conf_name: &'static str = alloc::string::String::from(format!("limine_{}.conf", arch.name())).leak();
        // Leak the config string so the TFTP server can hold a static reference
        let conf_bytes: &'static [u8] = conf.into_bytes().leak();
        crate::netstack::tftpd::register_file(conf_name, conf_bytes);
    }

    // Also register with common path variants that Limine PXE might request
    crate::netstack::tftpd::register_file("boot/trustos_kernel", kernel_data);
    crate::netstack::tftpd::register_file("boot/limine/limine.conf", LIMINE_CONF);

    // Start TFTP server
    crate::netstack::tftpd::start();

    // Start DHCP server with PXE boot options
    // Boot filename = limine-bios-pxe.bin (the first thing PXE clients download)
    crate::netstack::dhcpd::start(ip, mask, pool_start, 16, "limine-bios-pxe.bin");

    REPLICATING.store(true, Ordering::Relaxed);
    NODES_BOOTED.store(0, Ordering::Relaxed);

    crate::serial_println!("[PXE-REPL] Self-replication ACTIVE — waiting for PXE boot requests...");
    crate::serial_println!("[PXE-REPL] Boot sequence: PXE ROM → DHCP → TFTP(limine-bios-pxe.bin) → TFTP(limine.conf) → TFTP(trustos_kernel)");

    Ok(())
}

/// Stop PXE self-replication
pub fn stop() {
    if !REPLICATING.load(Ordering::Relaxed) {
        return;
    }

    crate::netstack::tftpd::stop();
    crate::netstack::dhcpd::stop();
    REPLICATING.store(false, Ordering::Relaxed);

    crate::serial_println!("[PXE-REPL] Self-replication stopped. {} nodes booted via PXE.",
        NODES_BOOTED.load(Ordering::Relaxed));
}

/// Poll for activity (call from main poll loop or timer)
pub fn poll() {
    if !REPLICATING.load(Ordering::Relaxed) {
        return;
    }

    // Poll TFTP for retransmissions
    crate::netstack::tftpd::poll();

    // Check if new nodes have completed booting
    // (We detect this by checking DHCP lease count changes)
    let leases = crate::netstack::dhcpd::active_leases();
    let known = NODES_BOOTED.load(Ordering::Relaxed);
    if leases > known {
        NODES_BOOTED.store(leases, Ordering::Relaxed);
        crate::serial_println!("[PXE-REPL] New node detected! Total PXE clients: {}", leases);
    }
}

/// Get status information
pub fn status() -> (bool, u8, u64, usize) {
    (
        is_active(),
        nodes_booted(),
        crate::netstack::tftpd::files_served(),
        crate::netstack::tftpd::active_transfers(),
    )
}

/// Register a foreign architecture kernel received from a mesh peer.
/// This enables cross-arch PXE: an x86_64 server can serve aarch64 kernels.
///
/// The data must be a leaked &'static [u8] (caller must ensure lifetime).
pub fn register_remote_kernel(arch: CpuArch, data: &'static [u8]) {
    let name: &'static str = alloc::string::String::from(format!("trustos_{}", arch.name())).leak();
    crate::netstack::tftpd::register_file(name, data);
    crate::serial_println!("[PXE-REPL] Remote kernel registered: {} ({} KB)",
        name, data.len() / 1024);
}

/// Get the architecture of the running PXE server kernel
pub fn server_arch() -> CpuArch {
    CpuArch::current()
}
