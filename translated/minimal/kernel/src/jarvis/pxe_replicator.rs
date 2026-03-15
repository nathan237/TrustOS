


















use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use super::mesh::CpuArch;


static Yk: AtomicBool = AtomicBool::new(false);


static OL_: AtomicU8 = AtomicU8::new(0);



static AYZ_: &[u8] = include_bytes!("../../../limine/limine-bios-pxe.bin");



static AEM_: &[u8] = b"timeout: 0\n\n/TrustOS\n    protocol: limine\n    kernel_path: tftp():/trustos_kernel\n";



struct Ajl {
    ptr: usize,  
    aw: usize,
}

unsafe impl Send for Ajl {}
unsafe impl Sync for Ajl {}


static AXQ_: spin::Mutex<Option<Ajl>> = spin::Mutex::new(None);






pub unsafe fn vue(ptr: *const u8, aw: usize) {
    let mut lhi = AXQ_.lock();
    *lhi = Some(Ajl { ptr: ptr as usize, aw });
    crate::serial_println!("[PXE-REPL] Kernel file registered: {:p}, {} bytes ({} KB)",
        ptr, aw, aw / 1024);
}


fn tdv() -> Option<&'static [u8]> {
    let lhi = AXQ_.lock();
    lhi.as_ref().map(|m| unsafe { core::slice::anh(m.ptr as *const u8, m.aw) })
}


pub fn rl() -> bool {
    Yk.load(Ordering::Relaxed)
}


pub fn uux() -> u8 {
    OL_.load(Ordering::Relaxed)
}





pub fn ay() -> Result<(), &'static str> {
    if Yk.load(Ordering::Relaxed) {
        return Err("Already replicating");
    }

    
    let abr = tdv().ok_or("Kernel file not registered (KernelFileRequest missing)")?;

    
    let (aro, up, qcb) = crate::network::aou()
        .ok_or("No IP configuration — run DHCP or set static IP first")?;

    let ip = *aro.as_bytes();
    let hs = *up.as_bytes();

    
    let dkt = [ip[0], ip[1], ip[2], ip[3].cn(10)];

    crate::serial_println!("[PXE-REPL] === TrustOS Self-Replication via PXE ===");
    crate::serial_println!("[PXE-REPL] Server IP: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    crate::serial_println!("[PXE-REPL] Kernel size: {} bytes ({} KB)", abr.len(), abr.len() / 1024);
    crate::serial_println!("[PXE-REPL] Limine PXE: {} bytes", AYZ_.len());
    crate::serial_println!("[PXE-REPL] Limine conf: {} bytes", AEM_.len());

    
    
    crate::netstack::tftpd::exo("limine-bios-pxe.bin", AYZ_);
    crate::netstack::tftpd::exo("limine.conf", AEM_);

    
    
    crate::netstack::tftpd::exo("trustos_kernel", abr);

    
    let kav = CpuArch::cv().j();
    let mwe: &'static str = alloc::string::String::from(format!("trustos_{}", kav)).fmu();
    crate::netstack::tftpd::exo(mwe, abr);
    crate::serial_println!("[PXE-REPL] Serving kernel as: trustos_kernel + {}", mwe);

    
    
    for arch in &[CpuArch::BT_, CpuArch::Fg, CpuArch::Jy] {
        let rno = format!(
            "timeout: 0\n\n/TrustOS ({})\n    protocol: limine\n    kernel_path: tftp():/trustos_{}\n",
            arch.j(), arch.j()
        );
        let rnq: &'static str = alloc::string::String::from(format!("limine_{}.conf", arch.j())).fmu();
        
        let rnp: &'static [u8] = rno.cfq().fmu();
        crate::netstack::tftpd::exo(rnq, rnp);
    }

    
    crate::netstack::tftpd::exo("boot/trustos_kernel", abr);
    crate::netstack::tftpd::exo("boot/limine/limine.conf", AEM_);

    
    crate::netstack::tftpd::ay();

    
    
    crate::netstack::dhcpd::ay(ip, hs, dkt, 16, "limine-bios-pxe.bin");

    Yk.store(true, Ordering::Relaxed);
    OL_.store(0, Ordering::Relaxed);

    crate::serial_println!("[PXE-REPL] Self-replication ACTIVE — waiting for PXE boot requests...");
    crate::serial_println!("[PXE-REPL] Boot sequence: PXE ROM → DHCP → TFTP(limine-bios-pxe.bin) → TFTP(limine.conf) → TFTP(trustos_kernel)");

    Ok(())
}


pub fn qg() {
    if !Yk.load(Ordering::Relaxed) {
        return;
    }

    crate::netstack::tftpd::qg();
    crate::netstack::dhcpd::qg();
    Yk.store(false, Ordering::Relaxed);

    crate::serial_println!("[PXE-REPL] Self-replication stopped. {} nodes booted via PXE.",
        OL_.load(Ordering::Relaxed));
}


pub fn poll() {
    if !Yk.load(Ordering::Relaxed) {
        return;
    }

    
    crate::netstack::tftpd::poll();

    
    
    let bkf = crate::netstack::dhcpd::qez();
    let ubp = OL_.load(Ordering::Relaxed);
    if bkf > ubp {
        OL_.store(bkf, Ordering::Relaxed);
        crate::serial_println!("[PXE-REPL] New node detected! Total PXE clients: {}", bkf);
    }
}


pub fn status() -> (bool, u8, u64, usize) {
    (
        rl(),
        uux(),
        crate::netstack::tftpd::kvt(),
        crate::netstack::tftpd::jzc(),
    )
}





pub fn zjb(arch: CpuArch, f: &'static [u8]) {
    let j: &'static str = alloc::string::String::from(format!("trustos_{}", arch.j())).fmu();
    crate::netstack::tftpd::exo(j, f);
    crate::serial_println!("[PXE-REPL] Remote kernel registered: {} ({} KB)",
        j, f.len() / 1024);
}


pub fn zml() -> CpuArch {
    CpuArch::cv()
}
