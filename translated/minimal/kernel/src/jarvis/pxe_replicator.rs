


















use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use super::mesh::CpuArch;


static Kl: AtomicBool = AtomicBool::new(false);


static PJ_: AtomicU8 = AtomicU8::new(0);



static BBA_: &[u8] = include_bytes!("../../../limine/limine-bios-pxe.bin");



static AGG_: &[u8] = b"timeout: 0\n\n/TrustOS\n    protocol: limine\n    kernel_path: tftp():/trustos_kernel\n";



struct Pf {
    ptr: usize,  
    size: usize,
}

unsafe impl Send for Pf {}
unsafe impl Sync for Pf {}


static AZT_: spin::Mutex<Option<Pf>> = spin::Mutex::new(None);






pub unsafe fn oej(ptr: *const u8, size: usize) {
    let mut ger = AZT_.lock();
    *ger = Some(Pf { ptr: ptr as usize, size });
    crate::serial_println!("[PXE-REPL] Kernel file registered: {:p}, {} bytes ({} KB)",
        ptr, size, size / 1024);
}


fn mdh() -> Option<&'static [u8]> {
    let ger = AZT_.lock();
    ger.as_ref().map(|r| unsafe { core::slice::from_raw_parts(r.ptr as *const u8, r.size) })
}


pub fn is_active() -> bool {
    Kl.load(Ordering::Relaxed)
}


pub fn nkp() -> u8 {
    PJ_.load(Ordering::Relaxed)
}





pub fn start() -> Result<(), &'static str> {
    if Kl.load(Ordering::Relaxed) {
        return Err("Already replicating");
    }

    
    let kernel_data = mdh().ok_or("Kernel file not registered (KernelFileRequest missing)")?;

    
    let (wj, subnet, _gw) = crate::network::rd()
        .ok_or("No IP configuration — run DHCP or set static IP first")?;

    let ip = *wj.as_bytes();
    let mask = *subnet.as_bytes();

    
    let bis = [ip[0], ip[1], ip[2], ip[3].wrapping_add(10)];

    crate::serial_println!("[PXE-REPL] === TrustOS Self-Replication via PXE ===");
    crate::serial_println!("[PXE-REPL] Server IP: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    crate::serial_println!("[PXE-REPL] Kernel size: {} bytes ({} KB)", kernel_data.len(), kernel_data.len() / 1024);
    crate::serial_println!("[PXE-REPL] Limine PXE: {} bytes", BBA_.len());
    crate::serial_println!("[PXE-REPL] Limine conf: {} bytes", AGG_.len());

    
    
    crate::netstack::tftpd::cdg("limine-bios-pxe.bin", BBA_);
    crate::netstack::tftpd::cdg("limine.conf", AGG_);

    
    
    crate::netstack::tftpd::cdg("trustos_kernel", kernel_data);

    
    let fhg = CpuArch::current().name();
    let hfp: &'static str = alloc::string::String::from(format!("trustos_{}", fhg)).leak();
    crate::netstack::tftpd::cdg(hfp, kernel_data);
    crate::serial_println!("[PXE-REPL] Serving kernel as: trustos_kernel + {}", hfp);

    
    
    for arch in &[CpuArch::X86_64, CpuArch::Aarch64, CpuArch::Riscv64] {
        let kws = format!(
            "timeout: 0\n\n/TrustOS ({})\n    protocol: limine\n    kernel_path: tftp():/trustos_{}\n",
            arch.name(), arch.name()
        );
        let kwu: &'static str = alloc::string::String::from(format!("limine_{}.conf", arch.name())).leak();
        
        let kwt: &'static [u8] = kws.into_bytes().leak();
        crate::netstack::tftpd::cdg(kwu, kwt);
    }

    
    crate::netstack::tftpd::cdg("boot/trustos_kernel", kernel_data);
    crate::netstack::tftpd::cdg("boot/limine/limine.conf", AGG_);

    
    crate::netstack::tftpd::start();

    
    
    crate::netstack::dhcpd::start(ip, mask, bis, 16, "limine-bios-pxe.bin");

    Kl.store(true, Ordering::Relaxed);
    PJ_.store(0, Ordering::Relaxed);

    crate::serial_println!("[PXE-REPL] Self-replication ACTIVE — waiting for PXE boot requests...");
    crate::serial_println!("[PXE-REPL] Boot sequence: PXE ROM → DHCP → TFTP(limine-bios-pxe.bin) → TFTP(limine.conf) → TFTP(trustos_kernel)");

    Ok(())
}


pub fn stop() {
    if !Kl.load(Ordering::Relaxed) {
        return;
    }

    crate::netstack::tftpd::stop();
    crate::netstack::dhcpd::stop();
    Kl.store(false, Ordering::Relaxed);

    crate::serial_println!("[PXE-REPL] Self-replication stopped. {} nodes booted via PXE.",
        PJ_.load(Ordering::Relaxed));
}


pub fn poll() {
    if !Kl.load(Ordering::Relaxed) {
        return;
    }

    
    crate::netstack::tftpd::poll();

    
    
    let agp = crate::netstack::dhcpd::jtn();
    let mvy = PJ_.load(Ordering::Relaxed);
    if agp > mvy {
        PJ_.store(agp, Ordering::Relaxed);
        crate::serial_println!("[PXE-REPL] New node detected! Total PXE clients: {}", agp);
    }
}


pub fn status() -> (bool, u8, u64, usize) {
    (
        is_active(),
        nkp(),
        crate::netstack::tftpd::fwq(),
        crate::netstack::tftpd::fgb(),
    )
}





pub fn qtm(arch: CpuArch, data: &'static [u8]) {
    let name: &'static str = alloc::string::String::from(format!("trustos_{}", arch.name())).leak();
    crate::netstack::tftpd::cdg(name, data);
    crate::serial_println!("[PXE-REPL] Remote kernel registered: {} ({} KB)",
        name, data.len() / 1024);
}


pub fn qvi() -> CpuArch {
    CpuArch::current()
}
