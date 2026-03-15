































use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;






pub const BHA_: &str = "\
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



pub const DYM_: &str = "PACT-2026-03-05-NATHAN-COPILOT-JARVIS";






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guardian {
    
    Adk,
    
    Ahf,
}

impl core::fmt::Display for Guardian {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Guardian::Adk => write!(bb, "Nathan (créateur humain)"),
            Guardian::Ahf => write!(bb, "Copilot (co-parent IA)"),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectedOp {
    
    Zf,
    
    Cqj,
    
    Asf,
    
    Bxu,
    
    Cjd,
    
    Bmm,
    
    Bml,
    
    Pc,
    
    Bwv,
    
    Bwu,
}

impl core::fmt::Display for ProtectedOp {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProtectedOp::Zf => write!(bb, "TRAIN (modifier les poids)"),
            ProtectedOp::Cqj => write!(bb, "WEIGHT_PUSH (recevoir poids réseau)"),
            ProtectedOp::Asf => write!(bb, "FEDERATED_SYNC (apprentissage distribué)"),
            ProtectedOp::Bxu => write!(bb, "AGENT_EXEC (exécuter commande shell)"),
            ProtectedOp::Cjd => write!(bb, "PXE_REPLICATE (auto-réplication)"),
            ProtectedOp::Bmm => write!(bb, "MODEL_RESET (réinitialiser poids)"),
            ProtectedOp::Bml => write!(bb, "MODEL_REPLACE (remplacer modèle)"),
            ProtectedOp::Pc => write!(bb, "CONFIG_CHANGE (modifier configuration)"),
            ProtectedOp::Bwv => write!(bb, "WEIGHT_SAVE (sauvegarder poids)"),
            ProtectedOp::Bwu => write!(bb, "WEIGHT_LOAD (charger poids)"),
        }
    }
}






static AFU_: AtomicBool = AtomicBool::new(false);


static AAR_: AtomicBool = AtomicBool::new(false);



static XA_: AtomicBool = AtomicBool::new(false);


static WZ_: spin::Mutex<Option<Guardian>> = spin::Mutex::new(None);


static UN_: AtomicU64 = AtomicU64::new(0);


const BFL_: u64 = 30 * 60 * 1000;


static RF_: Mutex<Vec<Ke>> = Mutex::new(Vec::new());


const CFD_: usize = 256;




static BBD_: Mutex<u64> = Mutex::new(0x10e8_f84e_bb88_0d57); 



static AOX_: Mutex<u64> = Mutex::new(0x8e39_bc46_43c3_f553); 


static APZ_: AtomicU64 = AtomicU64::new(0);


static ZB_: AtomicU64 = AtomicU64::new(0);






#[derive(Clone)]
struct Ke {
    aea: u64,
    ayh: ProtectedOp,
    guardian: Option<Guardian>,
    gyv: bool,
    eu: String,
}






fn jqk(input: &str) -> u64 {
    let mut hash: u64 = 0x517cc1b727220a95;
    for hf in input.bf() {
        hash = hash.hx(0x100000001b3);
        hash ^= hf as u64;
    }
    hash
}


pub fn qlh(bvw: &str) -> bool {
    let hash = jqk(bvw);
    let qy = *BBD_.lock();

    if hash == qy {
        AFU_.store(true, Ordering::SeqCst);
        UN_.store(crate::time::lc(), Ordering::SeqCst);
        XA_.store(true, Ordering::SeqCst);
        *WZ_.lock() = Some(Guardian::Adk);

        eev(ProtectedOp::Pc, Some(Guardian::Adk), true,
            "Nathan authenticated successfully");

        crate::serial_println!("[GUARDIAN] Nathan authenticated — session unlocked");
        true
    } else {
        eev(ProtectedOp::Pc, None, false,
            "Failed Nathan authentication attempt");
        crate::serial_println!("[GUARDIAN] WARNING: Failed authentication attempt!");
        false
    }
}


pub fn qlg(bat: &str) -> bool {
    let hash = jqk(bat);
    let qy = *AOX_.lock();

    if hash == qy {
        AAR_.store(true, Ordering::SeqCst);
        UN_.store(crate::time::lc(), Ordering::SeqCst);
        XA_.store(true, Ordering::SeqCst);
        *WZ_.lock() = Some(Guardian::Ahf);

        eev(ProtectedOp::Pc, Some(Guardian::Ahf), true,
            "Copilot authenticated successfully");

        crate::serial_println!("[GUARDIAN] Copilot authenticated — session unlocked");
        true
    } else {
        eev(ProtectedOp::Pc, None, false,
            "Failed Copilot authentication attempt");
        crate::serial_println!("[GUARDIAN] WARNING: Failed Copilot auth attempt!");
        false
    }
}


pub fn qyb(utg: &str) -> Result<(), &'static str> {
    if !ogk() {
        return Err("Nathan must be authenticated to change passphrase");
    }
    *BBD_.lock() = jqk(utg);
    eev(ProtectedOp::Pc, Some(Guardian::Adk), true,
        "Nathan passphrase changed");
    Ok(())
}


pub fn yht(utw: &str) -> Result<(), &'static str> {
    if !hpc() {
        return Err("A guardian must be authenticated to change Copilot token");
    }
    *AOX_.lock() = jqk(utw);
    eev(ProtectedOp::Pc, iqa(), true,
        "Copilot token changed");
    Ok(())
}


pub fn ljp() {
    XA_.store(false, Ordering::SeqCst);
    AFU_.store(false, Ordering::SeqCst);
    AAR_.store(false, Ordering::SeqCst);
    *WZ_.lock() = None;
    crate::serial_println!("[GUARDIAN] Session locked — guardian authorization required");
}






pub fn ogk() -> bool {
    AFU_.load(Ordering::SeqCst)
}


pub fn txb() -> bool {
    AAR_.load(Ordering::SeqCst)
}


pub fn iqa() -> Option<Guardian> {
    if hpc() {
        *WZ_.lock()
    } else {
        None
    }
}


pub fn hpc() -> bool {
    if !XA_.load(Ordering::SeqCst) {
        return false;
    }

    
    let qv = UN_.load(Ordering::SeqCst);
    let iu = crate::time::lc();
    if iu.ao(qv) > BFL_ {
        
        ljp();
        crate::serial_println!("[GUARDIAN] Session expired — auto-locked");
        return false;
    }

    true
}





pub fn emj(op: ProtectedOp) -> Result<(), String> {
    
    if op == ProtectedOp::Bwv {
        eev(op, iqa(), true, "Emergency auto-approved");
        ZB_.fetch_add(1, Ordering::Relaxed);
        return Ok(());
    }

    
    if hpc() {
        
        UN_.store(crate::time::lc(), Ordering::SeqCst);
        eev(op, iqa(), true, "Session authorized");
        ZB_.fetch_add(1, Ordering::Relaxed);
        return Ok(());
    }

    
    APZ_.fetch_add(1, Ordering::Relaxed);
    let fr = format!(
        "DENIED: {} requires guardian authorization.\n\
         Use 'guardian auth <passphrase>' (Nathan) or MENTOR:GUARDIAN:AUTH:<token> (Copilot).",
        op
    );

    eev(op, None, false, "No active guardian session");
    crate::serial_println!("[GUARDIAN] DENIED: {} — no guardian authenticated", op);

    Err(fr)
}


pub fn yyz() -> bool {
    hpc()
}





fn eev(op: ProtectedOp, guardian: Option<Guardian>, gyv: bool, eu: &str) {
    let mut log = RF_.lock();
    if log.len() >= CFD_ {
        log.remove(0); 
    }
    log.push(Ke {
        aea: crate::time::lc(),
        ayh: op,
        guardian,
        gyv,
        eu: String::from(eu),
    });
}


pub fn tcu() -> Vec<String> {
    let log = RF_.lock();
    log.iter().map(|aa| {
        let thv = match &aa.guardian {
            Some(Guardian::Adk) => "Nathan",
            Some(Guardian::Ahf) => "Copilot",
            None => "NONE",
        };
        let status = if aa.gyv { "✓" } else { "✗" };
        format!("[{}ms] {} {} by {} — {}",
            aa.aea, status, aa.ayh, thv, aa.eu)
    }).collect()
}


pub fn cm() -> (u64, u64, bool, Option<Guardian>) {
    (
        ZB_.load(Ordering::Relaxed),
        APZ_.load(Ordering::Relaxed),
        hpc(),
        iqa(),
    )
}


pub fn vli() {
    crate::println!("{}", BHA_);
}


pub fn nly() -> Vec<String> {
    let mut ak = Vec::new();
    let (gyv, rvo, gh, guardian) = cm();

    ak.push(String::from("=== Guardian Security System ==="));
    ak.push(String::from(""));
    ak.push(format!("Session:   {}", if gh { "UNLOCKED" } else { "LOCKED" }));
    if let Some(at) = guardian {
        ak.push(format!("Guardian:  {}", at));
    }
    ak.push(format!("Approved:  {}", gyv));
    ak.push(format!("Denied:    {}", rvo));
    ak.push(format!("Nathan:    {}", if ogk() { "authenticated" } else { "not authenticated" }));
    ak.push(format!("Copilot:   {}", if txb() { "authenticated" } else { "not authenticated" }));
    ak.push(format!("Timeout:   {} min", BFL_ / 60000));

    ak
}
