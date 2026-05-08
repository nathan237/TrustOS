







use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;





pub const BJV_: &str = "TrustVM";
pub const DCL_: &str = "1.0.0";
pub const DCJ_: &str = "Phoenix";
pub const DCK_: &str = "(c) 2026 TrustOS Project";


pub fn jqb() -> String {
    format!("{} v{} \"{}\"", BJV_, DCL_, DCJ_)
}






pub const BBD_: &str = r#"
 _____              _  __   ____  __ 
|_   _| __ _   _ ___| |_\ \ / /  \/  |
  | || '__| | | / __| __\ V /| |\/| |
  | || |  | |_| \__ \ |_ | | | |  | |
  |_||_|   \__,_|___/\__||_| |_|  |_|
"#;


pub const DWK_: &str = r#"
████████╗██████╗ ██╗   ██╗███████╗████████╗██╗   ██╗███╗   ███╗
╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██║   ██║████╗ ████║
   ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║██╔████╔██║
   ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║██║╚██╔╝██║
   ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝██║ ╚═╝ ██║
   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚═╝     ╚═╝
"#;


pub const DWI_: &str = "[TrustVM]";


pub const DUB_: &str = r#"
  ████
 ██████
██  ✓ ██
 ██████
  ████
   ██
"#;


pub const DUD_: &str = r#"
┌──────┐
│ ▓▓▓▓ │
│ ░░░░ │
└──────┘
"#;






pub const CXW_: &str = "● ";

pub const YQ_: &str = "○ ";

pub const YR_: &str = "▲ ";

pub const BIK_: &str = "✗ ";

pub const GH_: &str = "✓ ";


pub fn hzr(dh: u64) -> String {
    let im = dh / 1000;
    let acf = im / 60;
    let aoi = acf / 60;
    let cic = aoi / 24;
    
    if cic > 0 {
        format!("{}d {}h {}m", cic, aoi % 24, acf % 60)
    } else if aoi > 0 {
        format!("{}h {}m {}s", aoi, acf % 60, im % 60)
    } else if acf > 0 {
        format!("{}m {}s", acf, im % 60)
    } else {
        format!("{}s", im)
    }
}


pub fn qge(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{} GB", bytes / (1024 * 1024 * 1024))
    } else if bytes >= 1024 * 1024 {
        format!("{} MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{} B", bytes)
    }
}






#[derive(Debug, Clone)]
pub struct Age {
    pub id: u64,
    pub name: String,
    pub state: &'static str,
    pub memory_mb: usize,
    pub cpu_percent: u8,
    pub exits: u64,
}

impl Age {
    pub fn render(&self) -> String {
        let owr = match self.state {
            "Running" => CXW_,
            "Stopped" => YQ_,
            "Crashed" => BIK_,
            "Paused" => YR_,
            _ => "  ",
        };
        
        format!(
            "│ {:>3} │ {}{:<12} │ {:>4} MB │ {:>3}% │ {:>8} │",
            self.id,
            owr,
            self.state,
            self.memory_mb,
            self.cpu_percent,
            self.exits
        )
    }
}


pub fn lbm() -> &'static str {
    r#"
┌─────┬────────────────┬─────────┬──────┬──────────┐
│ ID  │ State          │ Memory  │ CPU  │ Exits    │
├─────┼────────────────┼─────────┼──────┼──────────┤"#
}


pub fn lbl() -> &'static str {
    "└─────┴────────────────┴─────────┴──────┴──────────┘"
}


pub fn qtt(aen: &[Age]) -> String {
    let mut output = String::new();
    
    output.push_str(&jqb());
    output.push('\n');
    output.push_str(lbm());
    output.push('\n');
    
    if aen.is_empty() {
        output.push_str("│     │ (no VMs)       │         │      │          │\n");
    } else {
        for vm in aen {
            output.push_str(&vm.render());
            output.push('\n');
        }
    }
    
    output.push_str(lbl());
    output
}






pub fn eyj(caps: u64) -> String {
    let mut output = String::from("TrustVM Capabilities:\n");
    
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
    
    for (bf, name, desc) in features {
        let status = if caps & bf != 0 { GH_ } else { YQ_ };
        output.push_str(&format!("  {} {:<20} {}\n", status, name, desc));
    }
    
    output
}






pub fn eyk(
    csm: bool,
    ept_enabled: bool,
    nx_enabled: bool,
    violations: u64,
) -> String {
    let mut output = String::from("Security Status:\n");
    
    
    let ptd = if csm { GH_ } else { YR_ };
    output.push_str(&format!("  {} VPID Isolation: {}\n", ptd, 
        if csm { "Active" } else { "Disabled" }));
    
    
    let lrj = if ept_enabled { GH_ } else { BIK_ };
    output.push_str(&format!("  {} EPT Protection: {}\n", lrj,
        if ept_enabled { "Active" } else { "Disabled" }));
    
    
    let nma = if nx_enabled { GH_ } else { YR_ };
    output.push_str(&format!("  {} NX Enforcement: {}\n", nma,
        if nx_enabled { "Active" } else { "Disabled" }));
    
    
    let psb = if violations == 0 { GH_ } else { YR_ };
    output.push_str(&format!("  {} EPT Violations: {}\n", psb, violations));
    
    output
}






pub fn owm() -> String {
    format!(r#"
{}
────────────────────────────────────────────────
  {} - Secure Hardware Virtualization
  {}
────────────────────────────────────────────────
"#, BBD_.trim(), jqb(), DCK_)
}


pub fn jqz(vpid: bool, ept: bool) -> String {
    format!(r#"
{} Hypervisor Ready

Features:
  {} VPID (TLB Isolation)
  {} EPT (Memory Protection)
  {} Virtual Console
  {} Shared Filesystem

Type 'hv help' for commands or 'vm run hello' to start a guest.
"#,
        BJV_,
        if vpid { GH_ } else { YQ_ },
        if ept { GH_ } else { YQ_ },
        GH_,
        GH_
    )
}






pub fn qrf(progress: u8, width: usize) -> String {
    let oz = (progress as usize * width) / 100;
    let empty = width - oz;
    
    format!("[{}{}] {}%", 
        "█".repeat(oz),
        "░".repeat(empty),
        progress
    )
}


pub fn qxl(frame: usize) -> char {
    const Yp: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    Yp[frame % Yp.len()]
}
