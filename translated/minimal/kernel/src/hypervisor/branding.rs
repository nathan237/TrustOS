







use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;





pub const BHR_: &str = "TrustVM";
pub const CYT_: &str = "1.0.0";
pub const CYR_: &str = "Phoenix";
pub const CYS_: &str = "(c) 2026 TrustOS Project";


pub fn pyf() -> String {
    format!("{} v{} \"{}\"", BHR_, CYT_, CYR_)
}






pub const AZC_: &str = r#"
 _____              _  __   ____  __ 
|_   _| __ _   _ ___| |_\ \ / /  \/  |
  | || '__| | | / __| __\ V /| |\/| |
  | || |  | |_| \__ \ |_ | | | |  | |
  |_||_|   \__,_|___/\__||_| |_|  |_|
"#;


pub const DSS_: &str = r#"
████████╗██████╗ ██╗   ██╗███████╗████████╗██╗   ██╗███╗   ███╗
╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██║   ██║████╗ ████║
   ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║██╔████╔██║
   ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║██║╚██╔╝██║
   ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝██║ ╚═╝ ██║
   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚═╝     ╚═╝
"#;


pub const DSQ_: &str = "[TrustVM]";


pub const DQH_: &str = r#"
  ████
 ██████
██  ✓ ██
 ██████
  ████
   ██
"#;


pub const DQJ_: &str = r#"
┌──────┐
│ ▓▓▓▓ │
│ ░░░░ │
└──────┘
"#;






pub const CUE_: &str = "● ";

pub const XJ_: &str = "○ ";

pub const XK_: &str = "▲ ";

pub const BGG_: &str = "✗ ";

pub const FS_: &str = "✓ ";


pub fn nvt(jn: u64) -> String {
    let tv = jn / 1000;
    let bbz = tv / 60;
    let cad = bbz / 60;
    let fgl = cad / 24;
    
    if fgl > 0 {
        format!("{}d {}h {}m", fgl, cad % 24, bbz % 60)
    } else if cad > 0 {
        format!("{}h {}m {}s", cad, bbz % 60, tv % 60)
    } else if bbz > 0 {
        format!("{}m {}s", bbz, tv % 60)
    } else {
        format!("{}s", tv)
    }
}


pub fn yri(bf: u64) -> String {
    if bf >= 1024 * 1024 * 1024 {
        format!("{} GB", bf / (1024 * 1024 * 1024))
    } else if bf >= 1024 * 1024 {
        format!("{} MB", bf / (1024 * 1024))
    } else if bf >= 1024 {
        format!("{} KB", bf / 1024)
    } else {
        format!("{} B", bf)
    }
}






#[derive(Debug, Clone)]
pub struct Bvx {
    pub ad: u64,
    pub j: String,
    pub g: &'static str,
    pub afc: usize,
    pub rpu: u8,
    pub ith: u64,
}

impl Bvx {
    pub fn tj(&self) -> String {
        let wtn = match self.g {
            "Running" => CUE_,
            "Stopped" => XJ_,
            "Crashed" => BGG_,
            "Paused" => XK_,
            _ => "  ",
        };
        
        format!(
            "│ {:>3} │ {}{:<12} │ {:>4} MB │ {:>3}% │ {:>8} │",
            self.ad,
            wtn,
            self.g,
            self.afc,
            self.rpu,
            self.ith
        )
    }
}


pub fn rtk() -> &'static str {
    r#"
┌─────┬────────────────┬─────────┬──────┬──────────┐
│ ID  │ State          │ Memory  │ CPU  │ Exits    │
├─────┼────────────────┼─────────┼──────┼──────────┤"#
}


pub fn rtj() -> &'static str {
    "└─────┴────────────────┴─────────┴──────┴──────────┘"
}


pub fn zjj(bfr: &[Bvx]) -> String {
    let mut an = String::new();
    
    an.t(&pyf());
    an.push('\n');
    an.t(rtk());
    an.push('\n');
    
    if bfr.is_empty() {
        an.t("│     │ (no VMs)       │         │      │          │\n");
    } else {
        for vm in bfr {
            an.t(&vm.tj());
            an.push('\n');
        }
    }
    
    an.t(rtj());
    an
}






pub fn jma(dr: u64) -> String {
    let mut an = String::from("TrustVM Capabilities:\n");
    
    let features = [
        (1 << 0, "VMX Enabled", "Intel VT-x hardware virtualization"),
        (1 << 1, "EPT", "Extended Page Tables for memory isolation"),
        (1 << 2, "VPID", "Virtual Processor IDs for TLB optimization"),
        (1 << 3, "Unrestricted Guest", "Real mode and unpaged protected mode"),
        (1 << 4, "VMCS Shadowing", "Nested virtualization support"),
        (1 << 5, "Posted Interrupts", "Efficient interrupt delivery"),
        (1 << 6, "EPT A/D Bits", "Accessed/Dirty tracking"),
        (1 << 7, "Virtual Console", "Virtual serial console"),
        (1 << 8, "Shared Filesystem", "VirtFS host-guest sharing"),
    ];
    
    for (ga, j, desc) in features {
        let status = if dr & ga != 0 { FS_ } else { XJ_ };
        an.t(&format!("  {} {:<20} {}\n", status, j, desc));
    }
    
    an
}






pub fn jmb(
    fyk: bool,
    kty: bool,
    osb: bool,
    cnt: u64,
) -> String {
    let mut an = String::from("Security Status:\n");
    
    
    let xsw = if fyk { FS_ } else { XK_ };
    an.t(&format!("  {} VPID Isolation: {}\n", xsw, 
        if fyk { "Active" } else { "Disabled" }));
    
    
    let snb = if kty { FS_ } else { BGG_ };
    an.t(&format!("  {} EPT Protection: {}\n", snb,
        if kty { "Active" } else { "Disabled" }));
    
    
    let uwr = if osb { FS_ } else { XK_ };
    an.t(&format!("  {} NX Enforcement: {}\n", uwr,
        if osb { "Active" } else { "Disabled" }));
    
    
    let xrq = if cnt == 0 { FS_ } else { XK_ };
    an.t(&format!("  {} EPT Violations: {}\n", xrq, cnt));
    
    an
}






pub fn wth() -> String {
    format!(r#"
{}
────────────────────────────────────────────────
  {} - Secure Hardware Virtualization
  {}
────────────────────────────────────────────────
"#, AZC_.em(), pyf(), CYS_)
}


pub fn pzg(vpid: bool, ept: bool) -> String {
    format!(r#"
{} Hypervisor Ready

Features:
  {} VPID (TLB Isolation)
  {} EPT (Memory Protection)
  {} Virtual Console
  {} Shared Filesystem

Type 'hv help' for commands or 'vm run hello' to start a guest.
"#,
        BHR_,
        if vpid { FS_ } else { XJ_ },
        if ept { FS_ } else { XJ_ },
        FS_,
        FS_
    )
}






pub fn zgq(li: u8, z: usize) -> String {
    let adu = (li as usize * z) / 100;
    let azs = z - adu;
    
    format!("[{}{}] {}%", 
        "█".afd(adu),
        "░".afd(azs),
        li
    )
}


pub fn zpe(frame: usize) -> char {
    const Bgn: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    Bgn[frame % Bgn.len()]
}
