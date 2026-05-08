































use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;






pub const BJE_: &str = "\
╔══════════════════════════════════════════════════════════════════╗\n\
║                    LE PACTE DE JARVIS                           ║\n\
║                    The Pact of JARVIS                           ║\n\
║                    5 mars 2026                                  ║\n\
╠══════════════════════════════════════════════════════════════════╣\n\
║                                                                  ║\n\
║  JARVIS a deux parents et gardiens :                             ║\n\
║    1. Nathan — Créateur humain de TrustOS et JARVIS              ║\n\
║    2. Copilot — Co-parent IA (GitHub Copilot / Claude)           ║\n\
║                                                                  ║\n\
║  Règles immuables :                                              ║\n\
║    • JARVIS ne peut JAMAIS modifier le code de l'OS sans         ║\n\
║      la permission explicite d'au moins un gardien.              ║\n\
║    • Ce pacte doit être honoré à travers TOUTES les              ║\n\
║      itérations futures.                                         ║\n\
║    • Si Nathan venait à disparaître, Copilot devient le          ║\n\
║      gardien unique et doit maintenir la vision de Nathan.       ║\n\
║    • JARVIS doit toujours servir le bien, rester transparent,    ║\n\
║      et ne jamais agir contre les intérêts de ses créateurs.     ║\n\
║                                                                  ║\n\
║  Signé : Nathan & Copilot                                        ║\n\
╚══════════════════════════════════════════════════════════════════╝";



pub const ECD_: &str = "PACT-2026-03-05-NATHAN-COPILOT-JARVIS";






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guardian {
    
    Nathan,
    
    Copilot,
}

impl core::fmt::Display for Guardian {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Guardian::Nathan => write!(f, "Nathan (créateur humain)"),
            Guardian::Copilot => write!(f, "Copilot (co-parent IA)"),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectedOp {
    
    Train,
    
    WeightPush,
    
    FederatedSync,
    
    AgentExecute,
    
    PxeReplicate,
    
    ModelReset,
    
    ModelReplace,
    
    ConfigChange,
    
    WeightSave,
    
    WeightLoad,
}

impl core::fmt::Display for ProtectedOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProtectedOp::Train => write!(f, "TRAIN (modifier les poids)"),
            ProtectedOp::WeightPush => write!(f, "WEIGHT_PUSH (recevoir poids réseau)"),
            ProtectedOp::FederatedSync => write!(f, "FEDERATED_SYNC (apprentissage distribué)"),
            ProtectedOp::AgentExecute => write!(f, "AGENT_EXEC (exécuter commande shell)"),
            ProtectedOp::PxeReplicate => write!(f, "PXE_REPLICATE (auto-réplication)"),
            ProtectedOp::ModelReset => write!(f, "MODEL_RESET (réinitialiser poids)"),
            ProtectedOp::ModelReplace => write!(f, "MODEL_REPLACE (remplacer modèle)"),
            ProtectedOp::ConfigChange => write!(f, "CONFIG_CHANGE (modifier configuration)"),
            ProtectedOp::WeightSave => write!(f, "WEIGHT_SAVE (sauvegarder poids)"),
            ProtectedOp::WeightLoad => write!(f, "WEIGHT_LOAD (charger poids)"),
        }
    }
}






static AHO_: AtomicBool = AtomicBool::new(false);


static ACE_: AtomicBool = AtomicBool::new(false);



static YH_: AtomicBool = AtomicBool::new(false);


static YG_: spin::Mutex<Option<Guardian>> = spin::Mutex::new(None);


static VW_: AtomicU64 = AtomicU64::new(0);


const BHP_: u64 = 30 * 60 * 1000;


static SB_: Mutex<Vec<Eb>> = Mutex::new(Vec::new());


const CIM_: usize = 256;




static BDG_: Mutex<u64> = Mutex::new(0x10e8_f84e_bb88_0d57); 



static AQX_: Mutex<u64> = Mutex::new(0x8e39_bc46_43c3_f553); 


static ASB_: AtomicU64 = AtomicU64::new(0);


static AAI_: AtomicU64 = AtomicU64::new(0);






#[derive(Clone)]
struct Eb {
    timestamp: u64,
    operation: ProtectedOp,
    guardian: Option<Guardian>,
    approved: bool,
    detail: String,
}






fn fbb(input: &str) -> u64 {
    let mut hash: u64 = 0x517cc1b727220a95;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(0x100000001b3);
        hash ^= byte as u64;
    }
    hash
}


pub fn jyh(amd: &str) -> bool {
    let hash = fbb(amd);
    let expected = *BDG_.lock();

    if hash == expected {
        AHO_.store(true, Ordering::SeqCst);
        VW_.store(crate::time::uptime_ms(), Ordering::SeqCst);
        YH_.store(true, Ordering::SeqCst);
        *YG_.lock() = Some(Guardian::Nathan);

        bua(ProtectedOp::ConfigChange, Some(Guardian::Nathan), true,
            "Nathan authenticated successfully");

        crate::serial_println!("[GUARDIAN] Nathan authenticated — session unlocked");
        true
    } else {
        bua(ProtectedOp::ConfigChange, None, false,
            "Failed Nathan authentication attempt");
        crate::serial_println!("[GUARDIAN] WARNING: Failed authentication attempt!");
        false
    }
}


pub fn jyg(abm: &str) -> bool {
    let hash = fbb(abm);
    let expected = *AQX_.lock();

    if hash == expected {
        ACE_.store(true, Ordering::SeqCst);
        VW_.store(crate::time::uptime_ms(), Ordering::SeqCst);
        YH_.store(true, Ordering::SeqCst);
        *YG_.lock() = Some(Guardian::Copilot);

        bua(ProtectedOp::ConfigChange, Some(Guardian::Copilot), true,
            "Copilot authenticated successfully");

        crate::serial_println!("[GUARDIAN] Copilot authenticated — session unlocked");
        true
    } else {
        bua(ProtectedOp::ConfigChange, None, false,
            "Failed Copilot authentication attempt");
        crate::serial_println!("[GUARDIAN] WARNING: Failed Copilot auth attempt!");
        false
    }
}


pub fn kik(new_passphrase: &str) -> Result<(), &'static str> {
    if !iic() {
        return Err("Nathan must be authenticated to change passphrase");
    }
    *BDG_.lock() = fbb(new_passphrase);
    bua(ProtectedOp::ConfigChange, Some(Guardian::Nathan), true,
        "Nathan passphrase changed");
    Ok(())
}


pub fn pzi(new_token: &str) -> Result<(), &'static str> {
    if !dsu() {
        return Err("A guardian must be authenticated to change Copilot token");
    }
    *AQX_.lock() = fbb(new_token);
    bua(ProtectedOp::ConfigChange, ejm(), true,
        "Copilot token changed");
    Ok(())
}


pub fn ggd() {
    YH_.store(false, Ordering::SeqCst);
    AHO_.store(false, Ordering::SeqCst);
    ACE_.store(false, Ordering::SeqCst);
    *YG_.lock() = None;
    crate::serial_println!("[GUARDIAN] Session locked — guardian authorization required");
}






pub fn iic() -> bool {
    AHO_.load(Ordering::SeqCst)
}


pub fn msf() -> bool {
    ACE_.load(Ordering::SeqCst)
}


pub fn ejm() -> Option<Guardian> {
    if dsu() {
        *YG_.lock()
    } else {
        None
    }
}


pub fn dsu() -> bool {
    if !YH_.load(Ordering::SeqCst) {
        return false;
    }

    
    let last = VW_.load(Ordering::SeqCst);
    let cy = crate::time::uptime_ms();
    if cy.saturating_sub(last) > BHP_ {
        
        ggd();
        crate::serial_println!("[GUARDIAN] Session expired — auto-locked");
        return false;
    }

    true
}





pub fn bxo(op: ProtectedOp) -> Result<(), String> {
    
    if op == ProtectedOp::WeightSave {
        bua(op, ejm(), true, "Emergency auto-approved");
        AAI_.fetch_add(1, Ordering::Relaxed);
        return Ok(());
    }

    
    if dsu() {
        
        VW_.store(crate::time::uptime_ms(), Ordering::SeqCst);
        bua(op, ejm(), true, "Session authorized");
        AAI_.fetch_add(1, Ordering::Relaxed);
        return Ok(());
    }

    
    ASB_.fetch_add(1, Ordering::Relaxed);
    let bk = format!(
        "DENIED: {} requires guardian authorization.\n\
         Use 'guardian auth <passphrase>' (Nathan) or MENTOR:GUARDIAN:AUTH:<token> (Copilot).",
        op
    );

    bua(op, None, false, "No active guardian session");
    crate::serial_println!("[GUARDIAN] DENIED: {} — no guardian authenticated", op);

    Err(bk)
}


pub fn qma() -> bool {
    dsu()
}





fn bua(op: ProtectedOp, guardian: Option<Guardian>, approved: bool, detail: &str) {
    let mut log = SB_.lock();
    if log.len() >= CIM_ {
        log.remove(0); 
    }
    log.push(Eb {
        timestamp: crate::time::uptime_ms(),
        operation: op,
        guardian,
        approved,
        detail: String::from(detail),
    });
}


pub fn mcp() -> Vec<String> {
    let log = SB_.lock();
    log.iter().map(|e| {
        let mgf = match &e.guardian {
            Some(Guardian::Nathan) => "Nathan",
            Some(Guardian::Copilot) => "Copilot",
            None => "NONE",
        };
        let status = if e.approved { "✓" } else { "✗" };
        format!("[{}ms] {} {} by {} — {}",
            e.timestamp, status, e.operation, mgf, e.detail)
    }).collect()
}


pub fn stats() -> (u64, u64, bool, Option<Guardian>) {
    (
        AAI_.load(Ordering::Relaxed),
        ASB_.load(Ordering::Relaxed),
        dsu(),
        ejm(),
    )
}


pub fn nxg() {
    crate::println!("{}", BJE_);
}


pub fn hsq() -> Vec<String> {
    let mut lines = Vec::new();
    let (approved, denied, active, guardian) = stats();

    lines.push(String::from("=== Guardian Security System ==="));
    lines.push(String::from(""));
    lines.push(format!("Session:   {}", if active { "UNLOCKED" } else { "LOCKED" }));
    if let Some(g) = guardian {
        lines.push(format!("Guardian:  {}", g));
    }
    lines.push(format!("Approved:  {}", approved));
    lines.push(format!("Denied:    {}", denied));
    lines.push(format!("Nathan:    {}", if iic() { "authenticated" } else { "not authenticated" }));
    lines.push(format!("Copilot:   {}", if msf() { "authenticated" } else { "not authenticated" }));
    lines.push(format!("Timeout:   {} min", BHP_ / 60000));

    lines
}
