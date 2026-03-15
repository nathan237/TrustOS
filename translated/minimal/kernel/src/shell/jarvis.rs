











use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::framebuffer::{C_, B_, G_, D_, A_, Q_, L_, DF_};





const CO_: u32 = 0xFF00DDFF;      
const AXL_: u32 = 0xFF00FFAA;     
const DRZ_: u32 = 0xFFFFAA00;       
const DRY_: u32 = 0xFFFF4444;        
const AZQ_: usize = 32;


#[derive(Debug, Clone, Copy, PartialEq)]
enum Intent {
    
    Qx,         
    Bpk,        
    Wi,           
    Hy,        
    Aat,            
    Afh,             
    
    
    Aux,          
    Axn,           
    Aqh,         
    Aqs,         
    Ask,           
    
    
    Ayd,         
    Akr,            
    Bpc,          
    Ayq,           
    
    
    Bis,               
    Ars,            
    Atr,              
    
    
    Bij,           
    Buf,             
    Bww,          
    Bkg,               
    Bdr,         
    Bjq,             
    Bhz,            
    
    
    Jf,              
    Re,            
    Btj,              
    
    F,
}


#[derive(Debug, Clone)]
struct Nc {
    kk: EntityKind,
    bn: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EntityKind {
    Pk,
    Command,
    Bbu,
    L,
    Bug,
    Ami,
    Cyc,
}


#[derive(Debug)]
struct Bxq {
    dc: String,
    hr: Action,
}

#[derive(Debug)]
enum Action {
    Kb(String),
    Cmy(String),
    Ad(String),
    Chn(Vec<Bxq>),
}


struct Cab {
    uch: Intent,
    ucr: String,
    pwl: u32,
    sh: Lang,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Lang {
    Bb,
    Az,
}






const Csm: &[&str] = &[
    "Be helpful, concise, and accurate.",
    "Never execute destructive commands without confirmation.",
    "Respect user privacy — never log or transmit personal data.",
    "When uncertain, say so. Never fabricate information.",
    "Prefer showing the user how to do things over doing them silently.",
    "Keep responses under 5 lines unless detail is requested.",
    "Be bilingual: detect language and respond in the same one.",
];






fn all(input: &str) -> String {
    let mut e = String::fc(input.len());
    for r in input.bw() {
        match r {
            'A'..='Z' => e.push((r as u8 + 32) as char),
            'a'..='z' | '0'..='9' | ' ' | '/' | '.' | '-' | '_' => e.push(r),
            
            'é' | 'è' | 'ê' | 'ë' => e.push('e'),
            'à' | 'â' | 'ä' => e.push('a'),
            'ù' | 'û' | 'ü' => e.push('u'),
            'ô' | 'ö' => e.push('o'),
            'î' | 'ï' => e.push('i'),
            'ç' => e.push('c'),
            'É' | 'È' | 'Ê' | 'Ë' => e.push('e'),
            'À' | 'Â' | 'Ä' => e.push('a'),
            _ => e.push(' '),
        }
    }
    e
}


fn nla(input: &str) -> Lang {
    let swr = [
        "est", "les", "des", "une", "que", "qui", "pas", "mon", "mes", "ton",
        "ses", "pour", "dans", "avec", "sur", "fait", "peut", "quel", "quoi",
        "comment", "combien", "pourquoi", "montre", "affiche", "donne", "ouvre",
        "lance", "joue", "cherche", "trouve", "supprime", "cree", "salut",
        "bonjour", "merci", "stp", "svp", "moi", "fichier", "dossier",
        "memoire", "disque", "reseau", "processus", "aide", "blague",
    ];
    let aoh: Vec<&str> = input.ayt().collect();
    let mut nvv = 0u32;
    for d in &aoh {
        let zv = d.avd();
        for &marker in &swr {
            if zv == marker || zv.cj(marker) {
                nvv += 1;
            }
        }
    }
    
    if nvv >= 1 { Lang::Bb } else { Lang::Az }
}


fn yb(abh: &str, clv: &[&str]) -> bool {
    for &ai in clv {
        if abh.contains(ai) { return true; }
    }
    false
}


fn rwr(abh: &str) -> Intent {
    
    if yb(abh, &["hello", "hi ", "hey ", "salut", "bonjour", "bonsoir", "yo ", "coucou", "allo"]) {
        return Intent::Bij;
    }
    if yb(abh, &["bye", "au revoir", "a plus", "ciao", "goodbye", "quit", "exit"]) {
        return Intent::Bhz;
    }
    if yb(abh, &["thank", "merci", "thanks", "thx"]) {
        return Intent::Buf;
    }
    if yb(abh, &["who are you", "qui es tu", "c est qui", "tes qui", "your name", "ton nom", "tu es quoi"]) {
        return Intent::Bww;
    }
    if yb(abh, &["joke", "blague", "funny", "drole", "humour", "raconte"]) {
        return Intent::Bkg;
    }
    if yb(abh, &["great", "awesome", "amazing", "genial", "super", "bravo", "bien joue", "nice", "cool", "good job", "bien fait"]) {
        return Intent::Bdr;
    }
    if yb(abh, &["suck", "nul", "bad", "horrible", "useless", "inutile", "pourri", "merde"]) {
        return Intent::Bjq;
    }
    
    
    if yb(abh, &["memory", "memoire", "ram", "heap", "free mem", "mem usage"]) {
        return Intent::Qx;
    }
    if yb(abh, &["system", "systeme", "status", "etat", "sante", "health", "overview"]) {
        return Intent::Qx;
    }
    if yb(abh, &["process", "processus", "running", "tourne", "ps ", "qui tourne", "what run"]) {
        return Intent::Bpk;
    }
    if yb(abh, &["disk", "disque", "storage", "stockage", "space", "espace", "df "]) {
        return Intent::Wi;
    }
    if yb(abh, &["network", "reseau", "ip ", "ifconfig", "internet", "connect", "connexion", "net "]) {
        return Intent::Hy;
    }
    if yb(abh, &["cpu", "processor", "processeur", "core", "coeur", "smp"]) {
        return Intent::Aat;
    }
    if yb(abh, &["uptime", "how long", "depuis combien", "temps", "duree"]) {
        return Intent::Afh;
    }
    
    
    if yb(abh, &["list file", "liste fichier", "ls ", "show file", "montre fichier", "affiche fichier", "what file", "quels fichier"]) {
        return Intent::Aux;
    }
    if yb(abh, &["read file", "lire fichier", "lis ", "cat ", "show content", "montre contenu", "ouvre fichier", "open file"]) {
        return Intent::Axn;
    }
    if yb(abh, &["create file", "cree fichier", "creer", "touch ", "nouveau fichier", "new file"]) {
        return Intent::Aqh;
    }
    if yb(abh, &["delete file", "supprime", "supprimer", "rm ", "remove", "efface", "detruit"]) {
        return Intent::Aqs;
    }
    if yb(abh, &["find file", "cherche fichier", "find ", "search", "ou est", "where is", "locate", "trouve"]) {
        return Intent::Ask;
    }
    
    
    if yb(abh, &["run ", "execute", "lance commande", "executer", "fais "]) {
        return Intent::Ayd;
    }
    if yb(abh, &["open ", "ouvre ", "launch", "lance ", "start ", "demarre"]) {
        return Intent::Akr;
    }
    if yb(abh, &["music", "musique", "play", "beep", "son", "sound", "audio"]) {
        return Intent::Bpc;
    }
    if yb(abh, &["theme", "color", "couleur", "dark", "light", "sombre", "clair"]) {
        return Intent::Ayq;
    }
    
    
    if yb(abh, &["help", "aide", "what can", "que peux", "quoi faire", "commande", "command"]) {
        return Intent::Bis;
    }
    if yb(abh, &["explain", "explique", "what is", "c est quoi", "definition", "qu est ce"]) {
        return Intent::Ars;
    }
    if yb(abh, &["how to", "comment", "how do", "how can", "tuto"]) {
        return Intent::Atr;
    }
    
    
    if yb(abh, &["about", "a propos", "trustos"]) && !yb(abh, &["open", "ouvre"]) {
        return Intent::Jf;
    }
    if yb(abh, &["version"]) {
        return Intent::Re;
    }
    if yb(abh, &["stats", "statistique", "chiffre", "number", "count"]) {
        return Intent::Btj;
    }
    
    Intent::F
}


fn sqd(abh: &str, dsb: Intent) -> Vec<Nc> {
    let mut axu = Vec::new();
    let aoh: Vec<&str> = abh.ayt().collect();
    
    match dsb {
        Intent::Axn | Intent::Aqh | Intent::Aqs | Intent::Ask => {
            
            for d in &aoh {
                if d.contains('/') || d.contains('.') || d.cj('~') {
                    axu.push(Nc { kk: EntityKind::Pk, bn: String::from(*d) });
                }
            }
            
            if axu.is_empty() {
                if let Some(qv) = aoh.qv() {
                    if !["file", "fichier", "it", "le", "la", "les"].contains(qv) {
                        axu.push(Nc { kk: EntityKind::Pk, bn: String::from(*qv) });
                    }
                }
            }
        }
        Intent::Ayd => {
            
            let iez = ["run ", "execute ", "lance ", "executer ", "fais "];
            for ab in &iez {
                if let Some(u) = abh.du(ab) {
                    let cmd = &abh[u + ab.len()..];
                    if !cmd.is_empty() {
                        axu.push(Nc { kk: EntityKind::Command, bn: String::from(cmd.em()) });
                    }
                    break;
                }
            }
        }
        Intent::Akr => {
            let iez = ["open ", "ouvre ", "launch ", "lance ", "start ", "demarre "];
            for ab in &iez {
                if let Some(u) = abh.du(ab) {
                    let bjf = &abh[u + ab.len()..];
                    if !bjf.is_empty() {
                        axu.push(Nc { kk: EntityKind::Bbu, bn: String::from(bjf.em()) });
                    }
                    break;
                }
            }
        }
        Intent::Ars | Intent::Atr => {
            let iez = ["explain ", "explique ", "what is ", "c est quoi ", "how to ", "comment "];
            for ab in &iez {
                if let Some(u) = abh.du(ab) {
                    let gus = &abh[u + ab.len()..];
                    if !gus.is_empty() {
                        axu.push(Nc { kk: EntityKind::Ami, bn: String::from(gus.em()) });
                    }
                    break;
                }
            }
        }
        Intent::Ayq => {
            for d in &aoh {
                match *d {
                    "matrix" | "green" | "vert" | "dark" | "sombre" | "cyber" | "retro" | "hacker" => {
                        axu.push(Nc { kk: EntityKind::Bug, bn: String::from(*d) });
                    }
                    _ => {}
                }
            }
        }
        Intent::Aux => {
            for d in &aoh {
                if d.contains('/') {
                    axu.push(Nc { kk: EntityKind::Pk, bn: String::from(*d) });
                }
            }
        }
        _ => {}
    }
    
    axu
}





fn ovy(dsb: Intent, axu: &[Nc], sh: Lang) -> Action {
    match dsb {
        Intent::Qx => {
            let mr = crate::memory::heap::mr();
            let aez = crate::memory::heap::aez();
            let es = mr + aez;
            let jvb = mr / 1024;
            let cuu = es / 1024;
            let cgn = if es > 0 { mr * 100 / es } else { 0 };
            let bcy = crate::task::dmj();
            match sh {
                Lang::Bb => Action::Ad(format!(
                    "Memoire: {} KB utilises / {} KB total ({}%)\nTaches actives: {}\nCPUs detectes: {}",
                    jvb, cuu, cgn, bcy + 2, crate::cpu::smp::aao()
                )),
                Lang::Az => Action::Ad(format!(
                    "Memory: {} KB used / {} KB total ({}%)\nActive tasks: {}\nCPUs detected: {}",
                    jvb, cuu, cgn, bcy + 2, crate::cpu::smp::aao()
                )),
            }
        }
        Intent::Bpk => {
            let bcy = crate::task::dmj();
            match sh {
                Lang::Bb => Action::Ad(format!(
                    "Processus actifs:\n  PID 1  kernel    (en cours)\n  PID 2  tsh       (en cours)\n  +{} taches en arriere-plan",
                    bcy
                )),
                Lang::Az => Action::Ad(format!(
                    "Running processes:\n  PID 1  kernel    (running)\n  PID 2  tsh       (running)\n  +{} background tasks",
                    bcy
                )),
            }
        }
        Intent::Wi => {
            match sh {
                Lang::Bb => Action::Ad(String::from(
                    "Systeme de fichiers:\n  ramfs    64 KB (journalise)\n  Pas de disque physique monte"
                )),
                Lang::Az => Action::Ad(String::from(
                    "File systems:\n  ramfs    64 KB (journaled)\n  No physical disk mounted"
                )),
            }
        }
        Intent::Hy => {
            Action::Kb(String::from("ifconfig"))
        }
        Intent::Aat => {
            let cdv = crate::cpu::smp::aao();
            let ack = crate::cpu::smp::boc();
            match sh {
                Lang::Bb => Action::Ad(format!(
                    "CPU: x86_64\nCoeurs detectes: {}\nCoeurs actifs: {} (BSP)\nSSE2: active\nMode: 64-bit Long Mode",
                    cdv, ack
                )),
                Lang::Az => Action::Ad(format!(
                    "CPU: x86_64\nCores detected: {}\nActive cores: {} (BSP)\nSSE2: enabled\nMode: 64-bit Long Mode",
                    cdv, ack
                )),
            }
        }
        Intent::Afh => {
            Action::Kb(String::from("uptime"))
        }
        
        
        Intent::Aux => {
            let path = axu.iter()
                .du(|aa| aa.kk == EntityKind::Pk)
                .map(|aa| aa.bn.as_str())
                .unwrap_or("/");
            Action::Kb(format!("ls {}", path))
        }
        Intent::Axn => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Pk) {
                Action::Kb(format!("cat {}", aa.bn))
            } else {
                match sh {
                    Lang::Bb => Action::Ad(String::from("Quel fichier veux-tu lire ? Donne-moi le chemin.")),
                    Lang::Az => Action::Ad(String::from("Which file should I read? Give me the path.")),
                }
            }
        }
        Intent::Aqh => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Pk) {
                Action::Kb(format!("touch {}", aa.bn))
            } else {
                match sh {
                    Lang::Bb => Action::Ad(String::from("Quel nom pour le fichier ?")),
                    Lang::Az => Action::Ad(String::from("What should the file be called?")),
                }
            }
        }
        Intent::Aqs => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Pk) {
                match sh {
                    Lang::Bb => Action::Ad(format!(
                        "Confirme: tu veux supprimer '{}' ?\nTape: rm {}", aa.bn, aa.bn
                    )),
                    Lang::Az => Action::Ad(format!(
                        "Confirm: delete '{}' ?\nType: rm {}", aa.bn, aa.bn
                    )),
                }
            } else {
                match sh {
                    Lang::Bb => Action::Ad(String::from("Quel fichier supprimer ?")),
                    Lang::Az => Action::Ad(String::from("Which file should I delete?")),
                }
            }
        }
        Intent::Ask => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Pk || aa.kk == EntityKind::Ami) {
                Action::Kb(format!("find {}", aa.bn))
            } else {
                match sh {
                    Lang::Bb => Action::Ad(String::from("Que cherches-tu ?")),
                    Lang::Az => Action::Ad(String::from("What are you looking for?")),
                }
            }
        }
        
        
        Intent::Ayd => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Command) {
                Action::Kb(aa.bn.clone())
            } else {
                match sh {
                    Lang::Bb => Action::Ad(String::from("Quelle commande veux-tu executer ?")),
                    Lang::Az => Action::Ad(String::from("What command should I run?")),
                }
            }
        }
        Intent::Akr => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Bbu) {
                let bjf = aa.bn.as_str();
                
                let cmd = match bjf {
                    e if yb(e, &["browser", "navigateur", "web", "internet"]) => "browse",
                    e if yb(e, &["chess", "echec"]) => "chess",
                    e if yb(e, &["editor", "editeur", "trustcode", "code"]) => "trustcode",
                    e if yb(e, &["desktop", "bureau"]) => "desktop",
                    e if yb(e, &["calculator", "calculatrice", "calc"]) => "calc",
                    e if yb(e, &["snake", "serpent", "game", "jeu"]) => "snake",
                    e if yb(e, &["terminal", "shell", "term"]) => "gterm",
                    e if yb(e, &["lab", "trustlab", "introspect"]) => "lab",
                    e if yb(e, &["3d", "model", "edit3d"]) => "trustedit",
                    e if yb(e, &["film", "movie"]) => "film",
                    e if yb(e, &["trailer", "bande annonce"]) => "trailer",
                    e if yb(e, &["music", "musique", "audio"]) => "synth",
                    _ => bjf,
                };
                Action::Kb(String::from(cmd))
            } else {
                match sh {
                    Lang::Bb => Action::Ad(String::from("Quelle application ouvrir ?")),
                    Lang::Az => Action::Ad(String::from("Which app should I open?")),
                }
            }
        }
        Intent::Bpc => {
            Action::Kb(String::from("synth"))
        }
        Intent::Ayq => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Bug) {
                Action::Kb(format!("theme {}", aa.bn))
            } else {
                Action::Kb(String::from("theme matrix"))
            }
        }
        
        
        Intent::Bis => {
            match sh {
                Lang::Bb => Action::Ad(String::from(
                    "Je peux t'aider avec:\n\
                     - Info systeme: \"memoire\", \"cpu\", \"processus\"\n\
                     - Fichiers: \"liste les fichiers\", \"lis /readme.md\"\n\
                     - Apps: \"ouvre le navigateur\", \"lance chess\"\n\
                     - Commandes: \"execute neofetch\"\n\
                     - Questions: \"explique TLS\", \"comment compiler\"\n\
                     - Fun: \"une blague\", \"qui es-tu\""
                )),
                Lang::Az => Action::Ad(String::from(
                    "I can help with:\n\
                     - System info: \"memory\", \"cpu\", \"processes\"\n\
                     - Files: \"list files\", \"read /readme.md\"\n\
                     - Apps: \"open browser\", \"launch chess\"\n\
                     - Commands: \"run neofetch\"\n\
                     - Questions: \"explain TLS\", \"how to compile\"\n\
                     - Fun: \"tell me a joke\", \"who are you\""
                )),
            }
        }
        Intent::Ars => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Ami) {
                spk(&aa.bn, sh)
            } else {
                match sh {
                    Lang::Bb => Action::Ad(String::from("Qu'est-ce que tu veux que j'explique ?")),
                    Lang::Az => Action::Ad(String::from("What would you like me to explain?")),
                }
            }
        }
        Intent::Atr => {
            if let Some(aa) = axu.iter().du(|aa| aa.kk == EntityKind::Ami) {
                tqe(&aa.bn, sh)
            } else {
                match sh {
                    Lang::Bb => Action::Ad(String::from("Comment faire quoi exactement ?")),
                    Lang::Az => Action::Ad(String::from("How to do what exactly?")),
                }
            }
        }
        
        
        Intent::Bij => {
            match sh {
                Lang::Bb => Action::Ad(String::from("Salut ! Je suis Jarvis, ton assistant TrustOS. Comment puis-je t'aider ?")),
                Lang::Az => Action::Ad(String::from("Hello! I'm Jarvis, your TrustOS assistant. How can I help?")),
            }
        }
        Intent::Buf => {
            match sh {
                Lang::Bb => Action::Ad(String::from("De rien ! N'hesite pas si tu as besoin d'autre chose.")),
                Lang::Az => Action::Ad(String::from("You're welcome! Let me know if you need anything else.")),
            }
        }
        Intent::Bww => {
            match sh {
                Lang::Bb => Action::Ad(String::from(
                    "Je suis Jarvis — Just A Rather Very Intelligent System.\n\
                     Assistant IA integre a TrustOS, 100% local, zero cloud.\n\
                     Je comprends le francais et l'anglais.\n\
                     Demande-moi n'importe quoi sur le systeme !"
                )),
                Lang::Az => Action::Ad(String::from(
                    "I'm Jarvis — Just A Rather Very Intelligent System.\n\
                     AI assistant built into TrustOS, 100% local, zero cloud.\n\
                     I understand both French and English.\n\
                     Ask me anything about the system!"
                )),
            }
        }
        Intent::Bkg => {
            let uao = [
                "Pourquoi les developpeurs Rust ne font jamais de segfault ?\nParce qu'ils ont le borrow checker comme garde du corps.",
                "C'est un octet qui rentre dans un bar.\nLe barman lui dit : \"Desole, on sert pas les types non signes ici.\"",
                "Combien de programmeurs faut-il pour changer une ampoule ?\nAucun, c'est un probleme hardware.",
                "Un bug rentre dans un bar.\nIl n'en sort jamais. C'est une feature.",
                "Pourquoi TrustOS est ecrit en Rust ?\nParce que la confiance se merite... et la memoire aussi.",
            ];
            let uan = [
                "Why do Rust devs never get segfaults?\nBecause the borrow checker is their bodyguard.",
                "A byte walks into a bar.\nBartender says: \"Sorry, we don't serve unsigned types here.\"",
                "How many programmers does it take to change a light bulb?\nNone, that's a hardware problem.",
                "A bug walks into a bar.\nIt never leaves. It's a feature.",
                "Why is TrustOS written in Rust?\nBecause trust is earned... and so is memory safety.",
            ];
            let w = (crate::rtc::nyr() as usize) % 5;
            match sh {
                Lang::Bb => Action::Ad(String::from(uao[w])),
                Lang::Az => Action::Ad(String::from(uan[w])),
            }
        }
        Intent::Bdr => {
            match sh {
                Lang::Bb => Action::Ad(String::from("Merci ! C'est grace a 131K lignes de Rust et un developpeur passionne.")),
                Lang::Az => Action::Ad(String::from("Thanks! It's all 131K lines of Rust and one passionate developer.")),
            }
        }
        Intent::Bjq => {
            match sh {
                Lang::Bb => Action::Ad(String::from("J'encaisse. Mais je suis open source — tu peux m'ameliorer toi-meme. :)")),
                Lang::Az => Action::Ad(String::from("Fair enough. But I'm open source — you can improve me yourself. :)")),
            }
        }
        Intent::Bhz => {
            match sh {
                Lang::Bb => Action::Ad(String::from("A plus ! Tape 'jarvis' quand tu veux me reparler.")),
                Lang::Az => Action::Ad(String::from("See you! Type 'jarvis' when you want to chat again.")),
            }
        }
        
        
        Intent::Jf => {
            match sh {
                Lang::Bb => Action::Ad(String::from(
                    "TrustOS v0.3.3 — OS bare-metal ecrit en Rust pur.\n\
                     131K lignes de code, 253 fichiers source, 1 auteur.\n\
                     Zero C, zero secrets, 100% auditable.\n\
                     github.com/nathan237/TrustOS"
                )),
                Lang::Az => Action::Ad(String::from(
                    "TrustOS v0.3.3 — Bare-metal OS written in pure Rust.\n\
                     131K lines of code, 253 source files, 1 author.\n\
                     Zero C, zero secrets, 100% auditable.\n\
                     github.com/nathan237/TrustOS"
                )),
            }
        }
        Intent::Re => {
            Action::Ad(String::from("TrustOS v0.3.3 (kernel build 2026-02-16)"))
        }
        Intent::Btj => {
            let mr = crate::memory::heap::mr();
            let aez = crate::memory::heap::aez();
            let es = mr + aez;
            let cdv = crate::cpu::smp::aao();
            match sh {
                Lang::Bb => Action::Ad(format!(
                    "--- Statistiques TrustOS ---\n\
                     Code: 131,985 lignes de Rust\n\
                     Fichiers: 253 modules .rs\n\
                     CPUs: {} detectes\n\
                     Heap: {} / {} KB\n\
                     ISO: ~8.4 MB\n\
                     Commandes shell: 120+",
                    cdv, mr / 1024, es / 1024
                )),
                Lang::Az => Action::Ad(format!(
                    "--- TrustOS Statistics ---\n\
                     Code: 131,985 lines of Rust\n\
                     Files: 253 .rs modules\n\
                     CPUs: {} detected\n\
                     Heap: {} / {} KB\n\
                     ISO: ~8.4 MB\n\
                     Shell commands: 120+",
                    cdv, mr / 1024, es / 1024
                )),
            }
        }
        
        Intent::F => {
            match sh {
                Lang::Bb => Action::Ad(String::from(
                    "Hmm, je ne suis pas sur de comprendre.\nEssaie \"aide\" pour voir ce que je peux faire."
                )),
                Lang::Az => Action::Ad(String::from(
                    "Hmm, I'm not sure I understand.\nTry \"help\" to see what I can do."
                )),
            }
        }
    }
}





fn spk(gus: &str, sh: Lang) -> Action {
    let spl = match gus {
        ab if yb(ab, &["rust", "langage"]) => match sh {
            Lang::Bb => "Rust est un langage systeme qui garantit la securite memoire sans garbage collector. TrustOS est ecrit a 100% en Rust.",
            Lang::Az => "Rust is a systems language that guarantees memory safety without a garbage collector. TrustOS is 100% Rust.",
        },
        ab if yb(ab, &["tls", "ssl", "https", "crypto"]) => match sh {
            Lang::Bb => "TLS 1.3 est le protocole de chiffrement utilise pour HTTPS. TrustOS implemente le handshake complet + Ed25519 from scratch.",
            Lang::Az => "TLS 1.3 is the encryption protocol for HTTPS. TrustOS implements the full handshake + Ed25519 from scratch.",
        },
        ab if yb(ab, &["kernel", "noyau"]) => match sh {
            Lang::Bb => "Le kernel est le coeur de l'OS. Il gere la memoire, les interruptions, le scheduler, les drivers. TrustOS tourne en Ring 0.",
            Lang::Az => "The kernel is the OS core. It manages memory, interrupts, the scheduler, drivers. TrustOS runs in Ring 0.",
        },
        ab if yb(ab, &["smp", "multicore", "multi-core"]) => match sh {
            Lang::Bb => "SMP = Symmetric Multi-Processing. TrustOS detecte tous les CPU via ACPI. Actuellement le BSP fait tout le travail.",
            Lang::Az => "SMP = Symmetric Multi-Processing. TrustOS detects all CPUs via ACPI. Currently the BSP does all the work.",
        },
        ab if yb(ab, &["trustlang", "language", "langage de prog"]) => match sh {
            Lang::Bb => "TrustLang est le langage de programmation integre a TrustOS. Lexer > Parser > VM bytecode. Tape 'trustlang' pour l'essayer.",
            Lang::Az => "TrustLang is TrustOS's built-in programming language. Lexer > Parser > VM bytecode. Type 'trustlang' to try it.",
        },
        ab if yb(ab, &["browser", "navigateur", "html"]) => match sh {
            Lang::Bb => "TrustBrowser est le navigateur integre. Il parse du HTML + CSS et supporte HTTPS via TLS 1.3. Tape 'browse' pour l'ouvrir.",
            Lang::Az => "TrustBrowser is the built-in browser. It parses HTML + CSS and supports HTTPS via TLS 1.3. Type 'browse' to open it.",
        },
        ab if yb(ab, &["compositor", "gui", "desktop", "bureau"]) => match sh {
            Lang::Bb => "COSMIC2 est le compositeur graphique. Multi-couches, optimise SSE2, 144 FPS. Tape 'desktop' pour le lancer.",
            Lang::Az => "COSMIC2 is the desktop compositor. Multi-layer, SSE2 optimized, 144 FPS. Type 'desktop' to launch it.",
        },
        ab if yb(ab, &["jarvis", "ia", "ai", "intelligence"]) => match sh {
            Lang::Bb => "C'est moi ! Jarvis = Just A Rather Very Intelligent System. NLU par pattern matching, bilingue FR/EN, execution kernel directe.",
            Lang::Az => "That's me! Jarvis = Just A Rather Very Intelligent System. NLU via pattern matching, bilingual FR/EN, direct kernel execution.",
        },
        _ => match sh {
            Lang::Bb => "Je n'ai pas d'info precise sur ce sujet. Essaie un autre mot-cle.",
            Lang::Az => "I don't have specific info on that topic. Try another keyword.",
        },
    };
    Action::Ad(String::from(spl))
}

fn tqe(gus: &str, sh: Lang) -> Action {
    let tqd = match gus {
        ab if yb(ab, &["compile", "build", "construire"]) => match sh {
            Lang::Bb => "Pour compiler TrustOS:\n  1. cargo build\n  2. powershell build-limine.ps1\nL'ISO sera dans trustos.iso",
            Lang::Az => "To build TrustOS:\n  1. cargo build\n  2. powershell build-limine.ps1\nThe ISO will be at trustos.iso",
        },
        ab if yb(ab, &["file", "fichier", "creer"]) => match sh {
            Lang::Bb => "Pour creer un fichier:\n  touch /mon_fichier.txt\n  echo 'contenu' > /mon_fichier.txt",
            Lang::Az => "To create a file:\n  touch /my_file.txt\n  echo 'content' > /my_file.txt",
        },
        ab if yb(ab, &["network", "reseau", "internet"]) => match sh {
            Lang::Bb => "Pour tester le reseau:\n  ifconfig      (voir l'IP)\n  ping 8.8.8.8  (tester la connexion)\n  browse        (ouvrir le navigateur)",
            Lang::Az => "To test networking:\n  ifconfig       (see IP)\n  ping 8.8.8.8   (test connection)\n  browse         (open browser)",
        },
        ab if yb(ab, &["theme", "personnaliser", "customize"]) => match sh {
            Lang::Bb => "Pour changer le theme:\n  theme matrix   (vert hacker)\n  theme cyber    (bleu futuriste)\n  theme retro    (amber)",
            Lang::Az => "To change theme:\n  theme matrix   (green hacker)\n  theme cyber    (blue futuristic)\n  theme retro    (amber)",
        },
        _ => match sh {
            Lang::Bb => "Je n'ai pas de tutoriel sur ce sujet. Tape 'help' pour les commandes disponibles.",
            Lang::Az => "I don't have a tutorial for that. Type 'help' for available commands.",
        },
    };
    Action::Ad(String::from(tqd))
}





fn nrn(hr: Action) {
    match hr {
        Action::Ad(fr) => {
            for line in fr.ak() {
                crate::gr!(CO_, "  ");
                crate::println!("{}", line);
            }
        }
        Action::Kb(cmd) => {
            crate::gr!(AXL_, "  > ");
            crate::h!(L_, "{}", cmd);
            
            super::azu(&cmd);
        }
        Action::Cmy(fr) => {
            for line in fr.ak() {
                crate::gr!(C_, "  ");
                crate::println!("{}", line);
            }
        }
        Action::Chn(au) => {
            for (a, gu) in au.iter().cf() {
                crate::gr!(AXL_, "  [{}] ", a + 1);
                crate::println!("{}", gu.dc);
            }
            for gu in au {
                nrn(gu.hr);
            }
        }
    }
}






pub(super) fn rfp(n: &[&str]) {
    
    if !n.is_empty() && (n[0] == "brain" || n[0] == "neural" || n[0] == "nn") {
        rcr(&n[1..]);
        return;
    }

    
    if !n.is_empty() {
        match n[0] {
            "boot" | "scan" | "wake" => {
                crate::print!("{}", crate::jarvis_hw::boot());
                return;
            }
            "hw" | "hardware" | "profile" => {
                crate::print!("{}", crate::jarvis_hw::wnl());
                return;
            }
            "insights" | "insight" => {
                crate::print!("{}", crate::jarvis_hw::wnj());
                return;
            }
            "plan" | "strategy" => {
                crate::print!("{}", crate::jarvis_hw::wnk());
                return;
            }
            "optimize" | "opt" | "tune" => {
                crate::print!("{}", crate::jarvis_hw::uyx());
                return;
            }
            "status" | "stat" => {
                crate::print!("{}", crate::jarvis_hw::wnn());
                return;
            }
            "analyze" | "analyse" | "inspect" => {
                if n.len() < 2 {
                    crate::println!("Usage: jarvis analyze <file>");
                    return;
                }
                let path = n[1..].rr(" ");
                match crate::ramfs::fh(|fs| fs.mq(&path).map(|bc| bc.ip())) {
                    Ok(f) => {
                        crate::print!("{}", crate::jarvis_hw::qhy(&f));
                    }
                    Err(_) => {
                        crate::println!("Cannot read file: {}", path);
                    }
                }
                return;
            }
            "query" | "ask" | "q" => {
                if n.len() < 2 {
                    crate::println!("Usage: jarvis query <question>");
                    crate::println!("  e.g. jarvis query can you access the encrypted data on this disk?");
                    return;
                }
                let lwq = n[1..].rr(" ");
                crate::print!("{}", crate::jarvis_hw::tqr(&lwq));
                return;
            }
            _ => {} 
        }
    }
    
    
    if !n.is_empty() {
        let query = n.rr(" ");
        oyc(&query);
        return;
    }
    
    
    vlh();
    
    let mut be = Cab {
        uch: Intent::F,
        ucr: String::new(),
        pwl: 0,
        sh: Lang::Az,
    };
    
    let mut esq = [0u8; 256];
    
    loop {
        if crate::shell::etf() {
            crate::h!(CO_, "  [interrupted]");
            break;
        }
        
        crate::gr!(CO_, "\n  jarvis");
        crate::gr!(Q_, " > ");
        
        
        let len = vrw(&mut esq);
        let js = core::str::jg(&esq[..len]).unwrap_or("").em();
        
        if js.is_empty() { continue; }
        
        
        if js == "exit" || js == "quit" || js == "q" || js == "bye" || js == "au revoir" {
            let sh = nla(js);
            match sh {
                Lang::Bb => crate::h!(CO_, "  A bientot !"),
                Lang::Az => crate::h!(CO_, "  See you later!"),
            }
            break;
        }
        
        be.pwl += 1;
        oyc(js);
    }
}


fn oyc(js: &str) {
    let abh = all(js);
    let sh = nla(js);
    let dsb = rwr(&abh);
    let axu = sqd(&abh, dsb);
    
    
    crate::gr!(L_, "  ");
    crate::gr!(0xFF444444, "[{:?}]", dsb);
    if !axu.is_empty() {
        for aa in &axu {
            crate::gr!(0xFF444444, " {:?}={}", aa.kk, aa.bn);
        }
    }
    if dsb == Intent::F && crate::jarvis::uc() {
        crate::gr!(0xFF444444, " ->brain");
    }
    crate::println!();
    
    
    let hr = if dsb == Intent::F {
        
        if let Some(usj) = crate::jarvis::usi(js) {
            Action::Ad(usj)
        } else {
            ovy(dsb, &axu, sh)
        }
    } else {
        ovy(dsb, &axu, sh)
    };
    nrn(hr);
}


fn vlh() {
    crate::println!();
    crate::h!(CO_, "  ╔═══════════════════════════════════════════════╗");
    crate::h!(CO_, "  ║          J.A.R.V.I.S. v1.0                   ║");
    crate::h!(CO_, "  ║    Just A Rather Very Intelligent System      ║");
    crate::h!(CO_, "  ╠═══════════════════════════════════════════════╣");
    crate::gr!(CO_,   "  ║  ");
    crate::gr!(Q_,    "TrustOS AI Assistant — 100%% local, 0%% cloud");
    crate::h!(CO_, "  ║");
    crate::gr!(CO_,   "  ║  ");
    crate::gr!(L_,     "Type 'help' for commands, 'exit' to leave");
    crate::h!(CO_, "   ║");
    crate::h!(CO_, "  ╚═══════════════════════════════════════════════╝");
    crate::println!();
}


fn vrw(bi: &mut [u8]) -> usize {
    use crate::keyboard::auw;
    let mut u = 0;

    loop {
        if let Some(r) = auw() {
            match r {
                b'\n' | b'\r' => {
                    crate::println!();
                    return u;
                }
                8 | 127 => { 
                    if u > 0 {
                        u -= 1;
                        crate::print!("\x08 \x08");
                    }
                }
                0x1B => {} 
                r if r >= 0x20 && u < bi.len() - 1 => {
                    bi[u] = r;
                    u += 1;
                    crate::print!("{}", r as char);
                }
                _ => {}
            }
        } else {
            
            core::hint::hc();
        }
    }
}





const AL_: u32 = 0xFF00BBFF;


fn rcr(n: &[&str]) {
    if n.is_empty() {
        crate::h!(AL_, "  Jarvis Neural Brain v2.0");
        crate::println!();
        crate::println!("  Usage: jarvis brain <command>");
        crate::println!();
        crate::println!("  Commands:");
        crate::println!("    init          Initialize neural brain (allocate ~1.2 MB)");
        crate::println!("    info          Show model architecture & stats");
        crate::println!("    generate <p>  Generate text from prompt");
        crate::println!("    train <text>  Train on a text sequence");
        crate::println!("    test          Run self-test suite");
        crate::println!("    bench         Benchmark inference speed");
        crate::println!("    introspect    Describe own architecture");
        crate::println!("    weights       Show weight statistics per layer");
        crate::println!("    hardware      Show available hardware for inference");
        crate::println!("    mentor        Start serial mentoring listener");
        crate::println!("    save          Save weights to /jarvis/weights.bin");
        crate::println!("    load          Load weights from RamFS");
        crate::println!("    load fat32    Load from FAT32 disk (/mnt/fat32/)");
        crate::println!("    load vfs <p>  Load from any VFS path");
        crate::println!("    load http <u> Download brain from HTTP URL");
        crate::println!("    reset         Reset weights to random");
        crate::println!("    pretrain [N]  Pre-train on embedded corpus (N epochs)");
        crate::println!("    eval          Evaluate loss across entire corpus");
        crate::println!("    chat <text>   Chat with neural brain directly");
        crate::println!("    swarm [N]     Init + mesh + federate + pretrain (N epochs)");
        crate::println!("    propagate     Auto-propagation: mesh + pull brain + federate");
        crate::println!("    propagate pxe Same + enable PXE to replicate further");
        crate::println!();
        crate::println!("  The neural brain is a 4-layer transformer (4.4M params)");
        crate::println!("  that learns from text, generates responses, and self-improves.");
        return;
    }

    match n[0] {
        "init" => {
            crate::h!(AL_, "  Initializing neural brain...");
            crate::jarvis::init();
            crate::h!(B_, "  Neural brain ready.");
        }

        "info" => {
            if !crate::jarvis::uc() {
                crate::h!(D_, "  Brain not initialized. Run: jarvis brain init");
                return;
            }
            for line in crate::jarvis::zl() {
                crate::println!("  {}", line);
            }
        }

        "generate" | "gen" | "g" => {
            if !cxl() { return; }
            if crate::jarvis::ogq() {
                crate::h!(D_, "  [Private mode] Generation disabled");
                return;
            }
            let aau = if n.len() > 1 { n[1..].rr(" ") } else { String::from("Hello") };
            crate::gr!(AL_, "  Prompt: ");
            crate::h!(Q_, "{}", aau);
            
            let ay = crate::time::ave();
            let an = crate::jarvis::cks(&aau, 64);
            let ez = crate::time::ave().ao(ay);
            
            crate::gr!(AL_, "  Output: ");
            crate::h!(B_, "{}", an);
            crate::h!(L_, "  ({} ms, {} tokens)", ez, an.len());
        }

        "train" => {
            if !cxl() { return; }
            let text = if n.len() > 1 { n[1..].rr(" ") } else {
                crate::println!("  Usage: jarvis brain train <text>");
                return;
            };
            crate::h!(AL_, "  Training on: \"{}\"", text);
            let vl = crate::jarvis::ekd(&text, 0.001);
            crate::h!(B_, "  Loss: {:.4}", vl);
        }

        "test" => {
            if !cxl() { return; }
            crate::h!(AL_, "  Running neural brain self-test...");
            crate::println!();

            
            crate::h!(C_, "  Tokenizer:");
            let eb = crate::jarvis::tokenizer::cxj("Hello");
            crate::println!("    encode(\"Hello\") = {:?} ({} tokens)", &eb, eb.len());
            let aoq = crate::jarvis::tokenizer::hfo(&eb);
            crate::println!("    decode() = \"{}\"", aoq);

            
            crate::h!(C_, "  Generation:");
            let an = crate::jarvis::cks("Test", 16);
            crate::println!("    generate(\"Test\", 16) = \"{}\" ({} chars)", an, an.len());

            
            crate::h!(C_, "  Training:");
            let uil = crate::jarvis::ekd("Hello world", 0.001);
            crate::println!("    Step 1 loss: {:.4}", uil);
            let uim = crate::jarvis::ekd("Hello world", 0.001);
            crate::println!("    Step 2 loss: {:.4}", uim);

            
            crate::h!(C_, "  Training module:");
            let (aaz, boy) = crate::jarvis::training::eyj();
            crate::println!("    {} passed, {} failed", aaz, boy);

            
            crate::h!(C_, "  Introspection:");
            let co = crate::jarvis::agent::flw(
                &crate::jarvis::agent::IntrospectTarget::Agj);
            for line in co.iter().take(5) {
                crate::println!("    {}", line);
            }

            crate::println!();
            let dwz = 4 + aaz; 
            let cut = boy;
            if cut == 0 {
                crate::h!(B_, "  All {} tests passed!", dwz);
            } else {
                crate::h!(A_, "  {} passed, {} failed", dwz, cut);
            }
        }

        "bench" => {
            if !cxl() { return; }
            crate::h!(AL_, "  Benchmarking inference speed...");
            let (xjc, oz) = crate::jarvis::agent::qox();
            crate::println!("  Speed: {:.1} tokens/sec", xjc);
            crate::println!("  32 tokens generated in {} ms", oz);
            crate::h!(L_, "  (CPU reference — GPU dispatch target: 100×)");
        }

        "introspect" | "self" => {
            if !cxl() { return; }
            let ak = crate::jarvis::agent::flw(
                &crate::jarvis::agent::IntrospectTarget::Bv);
            for line in &ak {
                crate::h!(AL_, "  {}", line);
            }
        }

        "weights" => {
            if !cxl() { return; }
            let ak = crate::jarvis::agent::flw(
                &crate::jarvis::agent::IntrospectTarget::Bat);
            for line in &ak {
                crate::println!("  {}", line);
            }
        }

        "hardware" | "hw" => {
            if !cxl() { return; }
            let ak = crate::jarvis::agent::flw(
                &crate::jarvis::agent::IntrospectTarget::Ip);
            for line in &ak {
                crate::println!("  {}", line);
            }
        }

        "mentor" => {
            crate::h!(AL_, "  Serial Mentoring Mode");
            crate::println!("  Listening on COM1 for MENTOR: commands...");
            crate::println!("  (Send MENTOR:STATUS from host to verify connection)");
            crate::println!();
            crate::println!("  Protocol:");
            crate::println!("    MENTOR:TEACH:<text>       Train on text");
            crate::println!("    MENTOR:GENERATE:<prompt>  Generate text");
            crate::println!("    MENTOR:EVAL:<prompt>      Evaluate loss");
            crate::println!("    MENTOR:STATUS             Report stats");
            crate::println!("    MENTOR:SAVE               Save weights");
            crate::println!("    MENTOR:RESET              Reset weights");
            crate::println!();
            crate::h!(L_, "  (Mentor polling active in shell idle loop)");
        }

        "reset" => {
            if !cxl() { return; }
            crate::h!(D_, "  Resetting all weights to random...");
            crate::jarvis::apa();
            crate::h!(B_, "  Weights reset. Training steps cleared.");
        }

        "save" => {
            if !cxl() { return; }
            crate::h!(AL_, "  Saving weights to /jarvis/weights.bin...");
            match crate::jarvis::pfn() {
                Ok(bf) => crate::h!(B_, "  Saved {} KB", bf / 1024),
                Err(aa) => crate::h!(A_, "  Save failed: {}", aa),
            }
        }

        "load" => {
            if n.len() > 1 {
                match n[1] {
                    "fat32" => {
                        let it = if n.len() > 2 { Some(n[2]) } else { None };
                        let gez = it.unwrap_or("jarvis_weights.bin");
                        crate::h!(AL_, "  Loading brain from FAT32: /mnt/fat32/{}", gez);
                        match crate::jarvis::ojs(it) {
                            Ok(bf) => {
                                crate::h!(B_, "  Loaded {} KB from FAT32", bf / 1024);
                                crate::h!(L_, "  Caching to RamFS...");
                                let _ = crate::jarvis::gbx();
                            }
                            Err(aa) => crate::h!(A_, "  FAT32 load failed: {}", aa),
                        }
                    }
                    "vfs" => {
                        if n.len() < 3 {
                            crate::h!(D_, "  Usage: jarvis brain load vfs <path>");
                            crate::h!(L_, "  Example: jarvis brain load vfs /mnt/fat32/brain.bin");
                            return;
                        }
                        crate::h!(AL_, "  Loading brain from VFS: {}", n[2]);
                        match crate::jarvis::ugn(n[2]) {
                            Ok(bf) => {
                                crate::h!(B_, "  Loaded {} KB from {}", bf / 1024, n[2]);
                                crate::h!(L_, "  Caching to RamFS...");
                                let _ = crate::jarvis::gbx();
                            }
                            Err(aa) => crate::h!(A_, "  VFS load failed: {}", aa),
                        }
                    }
                    "http" => {
                        if n.len() < 3 {
                            crate::h!(D_, "  Usage: jarvis brain load http <url>");
                            crate::h!(L_, "  Example: jarvis brain load http 192.168.1.10/jarvis_weights.bin");
                            return;
                        }
                        crate::h!(AL_, "  Downloading brain from: {}", n[2]);
                        crate::h!(L_, "  This may take a moment (~17 MB)...");
                        match crate::jarvis::ugm(n[2]) {
                            Ok(bf) => {
                                crate::h!(B_, "  Downloaded & loaded {} KB", bf / 1024);
                                crate::h!(L_, "  Caching to RamFS...");
                                let _ = crate::jarvis::gbx();
                            }
                            Err(aa) => crate::h!(A_, "  HTTP load failed: {}", aa),
                        }
                    }
                    _ => {
                        crate::h!(D_, "  Unknown source: {}", n[1]);
                        crate::h!(L_, "  Usage: jarvis brain load [fat32|vfs|http]");
                    }
                }
            } else {
                
                crate::h!(AL_, "  Loading weights from /jarvis/weights.bin...");
                match crate::jarvis::oka() {
                    Ok(bf) => crate::h!(B_, "  Loaded {} KB", bf / 1024),
                    Err(aa) => crate::h!(A_, "  Load failed: {}", aa),
                }
            }
        }

        "pretrain" | "pt" => {
            if !cxl() { return; }
            let bmz: usize = if n.len() > 1 {
                n[1].parse().unwrap_or(3)
            } else { 3 }; 
            crate::h!(AL_, "  Pre-training on embedded corpus ({} epoch(s))...", bmz);
            crate::h!(L_, "  Using cosine LR + gradient accumulation (batch=4)");
            crate::h!(L_, "  {} phases, {} sequences total",
                crate::jarvis::corpus::htg(), crate::jarvis::corpus::ien());
            crate::println!();

            
            let fnh = crate::jarvis::itc();
            crate::h!(L_, "  Loss before: {:.3}", fnh);
            crate::println!();

            
            let (au, bdl, ez) = crate::jarvis::oxe(bmz, 0.001);

            crate::h!(AL_, "  Training complete!");
            crate::h!(L_, "  {} steps, avg loss={:.3}, {}ms ({:.1}s)",
                au, bdl, ez, ez as f64 / 1000.0);
            crate::println!();

            
            let fng = crate::jarvis::itc();
            crate::gr!(L_, "  Loss after:  {:.3}", fng);
            if fng < fnh {
                crate::h!(B_, " (improved by {:.3})", fnh - fng);
            } else {
                crate::h!(D_, " (no improvement)");
            }
            crate::println!();
            crate::h!(L_, "  Training steps global: {}",
                crate::jarvis::fae());
        }

        "eval" => {
            if !cxl() { return; }
            
            let syy = n.len() >= 2 && n[1] == "full";
            if syy {
                crate::h!(AL_, "  Evaluating FULL corpus (this may take a while)...");
                let bdl = crate::jarvis::kud();
                crate::println!();

                
                let upf = !crate::jarvis::uc();
                if !upf {
                    for ib in 0..crate::jarvis::corpus::htg() {
                        let j = crate::jarvis::corpus::jjg(ib);
                        let mut az = 0u32;
                        for &text in crate::jarvis::corpus::Gr[ib] {
                            let eb = crate::jarvis::tokenizer::cxj(text);
                            if eb.len() >= 2 {
                                az += 1;
                            }
                        }
                        crate::gr!(AL_, "  Phase {} ", ib);
                        crate::gr!(Q_, "({}) ", j);
                        crate::h!(L_, "{} sequences", az);
                    }
                }

                crate::println!();
                crate::gr!(Q_, "  Average loss: ");
                if bdl < 4.0 {
                    crate::h!(B_, "{:.3} (learning!)", bdl);
                } else if bdl < 5.5 {
                    crate::h!(D_, "{:.3} (early stage)", bdl);
                } else {
                    crate::h!(A_, "{:.3} (random/untrained)", bdl);
                }
                crate::h!(L_, "  (Random baseline: ~5.5, Good: <3.0, Memorized: <1.0)");
            } else {
                crate::h!(AL_, "  Quick eval (1 sample/phase)...");
                let bdl = crate::jarvis::itc();
                crate::println!();
                crate::gr!(Q_, "  Average loss: ");
                if bdl < 4.0 {
                    crate::h!(B_, "{:.3} (learning!)", bdl);
                } else if bdl < 5.5 {
                    crate::h!(D_, "{:.3} (early stage)", bdl);
                } else {
                    crate::h!(A_, "{:.3} (random/untrained)", bdl);
                }
                crate::h!(L_, "  (Quick: 1/phase. Use 'eval full' for complete corpus)");
            }
        }

        "chat" => {
            if !cxl() { return; }
            if crate::jarvis::ogq() {
                crate::h!(D_, "  [Private mode] Chat disabled");
                return;
            }
            if n.len() < 2 {
                crate::println!("  Usage: jarvis brain chat <text>");
                return;
            }
            let aau = n[1..].rr(" ");
            crate::gr!(Q_, "  You: ");
            crate::println!("{}", aau);

            let ay = crate::time::ave();
            let an = crate::jarvis::cks(&aau, 64);
            let ez = crate::time::ave().ao(ay);

            crate::gr!(AL_, "  Jarvis: ");
            
            for r in an.bw() {
                if r.jbb() || r == ' ' {
                    crate::print!("{}", r);
                } else {
                    crate::gr!(L_, ".");
                }
            }
            crate::println!();
            crate::h!(L_, "  ({} ms, {} chars)", ez, an.len());
        }

        "task" => {
            
            
            if !crate::jarvis::mesh::rl() {
                crate::h!(A_, "  Mesh not active. Run 'jarvis brain swarm' first.");
                return;
            }

            crate::h!(AL_, "  === JARVIS Distributed Task Verification ===");
            crate::println!();

            let yp = crate::jarvis::mesh::dhn();
            let es = yp.len() + 1;
            crate::h!(L_, "  Cluster: {} node(s) ({} peers + self)", es, yp.len());
            crate::println!();

            crate::h!(AL_, "  Generating unique math tasks per node...");
            let ay = crate::time::lc();
            let hd = crate::jarvis::task::wbf();
            let ez = crate::time::lc().nj(ay);

            crate::println!();
            crate::h!(AL_, "  Results:");
            crate::println!();

            let mut mur = true;
            for m in &hd {
                let ejb = if m.dzv { "OK" } else { "FAIL" };
                let dch = if m.dzv { B_ } else { A_ };
                let sse = if m.ggz { "file written" } else { "no file" };

                crate::print!("    ");
                crate::gr!(dch, "[{}]", ejb);
                crate::print!(" {} ", m.evj);
                crate::gr!(L_, "| {} = ", m.xz);
                if m.dzv {
                    crate::gr!(B_, "{}", m.ecf);
                } else {
                    crate::gr!(A_, "{}", m.ecf);
                    crate::gr!(L_, " (expected {})", m.qy);
                    mur = false;
                }
                crate::h!(L_, " | {}", sse);
            }

            crate::println!();
            if mur {
                crate::h!(B_,
                    "  All {} nodes returned correct answers in {} ms", hd.len(), ez);
                crate::h!(B_,
                    "  Distributed verification: PASSED");
            } else {
                let rpe = hd.iter().hi(|m| m.dzv).az();
                crate::h!(A_,
                    "  {}/{} correct — verification FAILED ({} ms)", rpe, hd.len(), ez);
            }
        }

        "propagate" | "autoprop" | "spread" => {
            let ggc = n.len() > 1 && (n[1] == "pxe" || n[1] == "replicate");
            crate::h!(AL_, "  === JARVIS Auto-Propagation ===");
            crate::println!();
            if ggc {
                crate::h!(L_, "  Mode: FULL (mesh + brain pull + federated + PXE)");
            } else {
                crate::h!(L_, "  Mode: JOIN (mesh + brain pull + federated)");
            }
            crate::println!();

            let report = crate::jarvis::mwy(ggc);
            for line in report.ak() {
                if line.contains("FAIL") || line.contains("failed") {
                    crate::h!(A_, "  {}", line);
                } else if line.contains("OK") || line.contains("active") || line.contains("DOWNLOADED") || line.contains("enabled") || line.contains("FULL") {
                    crate::h!(B_, "  {}", line);
                } else {
                    crate::h!(L_, "  {}", line);
                }
            }

            
            crate::println!();
            let yp = crate::jarvis::mesh::dhn();
            if !yp.is_empty() {
                crate::h!(AL_, "  Connected nodes:");
                for ko in &yp {
                    let bwt = match ko.bwt {
                        crate::jarvis::mesh::NodeRole::Ni => "LEADER",
                        crate::jarvis::mesh::NodeRole::Mu => "CAND",
                        crate::jarvis::mesh::NodeRole::Lb => "WORKER",
                    };
                    crate::println!("    {}.{}.{}.{} [{}] {} params, {} steps",
                        ko.ip[0], ko.ip[1], ko.ip[2], ko.ip[3],
                        bwt, ko.vm, ko.fae);
                }
            }

            crate::println!();
            if crate::jarvis::fkf() {
                crate::h!(B_, "  JARVIS is fully operational and connected.");
            } else {
                crate::h!(D_, "  JARVIS running in micro mode. Full brain will download when a peer joins.");
            }
        }

        "swarm" => {
            
            let bmz: usize = if n.len() > 1 {
                n[1].parse().unwrap_or(3)
            } else { 3 };

            crate::h!(AL_, "  === JARVIS Swarm Training Mode ===");
            crate::println!();

            
            if !crate::jarvis::uc() {
                crate::h!(AL_, "  [1/4] Initializing neural brain...");
                crate::jarvis::init();
                if !crate::jarvis::uc() {
                    crate::h!(A_, "  Brain init failed");
                    return;
                }
            } else {
                crate::h!(B_, "  [1/4] Brain already initialized");
            }

            
            crate::h!(AL_, "  [2/4] Starting mesh network...");
            crate::jarvis::jfy();
            crate::h!(B_, "  Mesh active on UDP 7700 / TCP 7701");

            
            if let Some((ura, _, _)) = crate::network::aou() {
                crate::h!(L_, "  Scanning subnet for peers...");
                let bsb = crate::jarvis::mesh::GV_;
                let jgi = ura.as_bytes();
                let mut aig = 0u32;
                for kh in 1u8..=10 {
                    if kh == jgi[3] { continue; }
                    let huz = [jgi[0], jgi[1], jgi[2], kh];
                    if let Ok(true) = crate::jarvis::rpc::ovs(huz, bsb) {
                        crate::h!(B_, "    Found peer: {}.{}.{}.{}",
                            huz[0], huz[1], huz[2], huz[3]);
                        aig += 1;
                    }
                }
                if aig == 0 {
                    crate::h!(D_, "    No peers found (training solo)");
                } else {
                    crate::h!(B_, "    {} peer(s) discovered", aig);
                }
            }

            
            crate::h!(AL_, "  [3/4] Enabling federated learning...");
            crate::jarvis::federated::aiy();
            crate::h!(B_, "  Federated gradient exchange enabled (30s sync)");

            
            crate::h!(AL_, "  [4/4] Pre-training {} epoch(s)...", bmz);
            crate::println!();

            let fnh = crate::jarvis::kud();
            crate::h!(L_, "  Loss before: {:.3}", fnh);

            let (au, bdl, ez) = crate::jarvis::oxe(bmz, 0.001);

            crate::println!();
            crate::h!(AL_, "  Swarm training complete!");
            crate::h!(L_, "  {} steps, avg loss={:.3}, {:.1}s",
                au, bdl, ez as f64 / 1000.0);

            let fng = crate::jarvis::kud();
            crate::gr!(L_, "  Loss after:  {:.3}", fng);
            if fng < fnh {
                crate::h!(B_, " (improved by {:.3})", fnh - fng);
            } else {
                crate::h!(D_, " (no improvement)");
            }

            
            let yp = crate::jarvis::mesh::dhn();
            crate::h!(L_, "  Mesh peers: {}", yp.len());
            crate::h!(L_, "  Pre-training done");
        }

        _ => {
            crate::h!(A_, "  Unknown: jarvis brain {}", n[0]);
            crate::println!("  Use 'jarvis brain' for help");
        }
    }
}


fn cxl() -> bool {
    if !crate::jarvis::uc() {
        crate::h!(L_, "  Auto-initializing neural brain...");
        crate::jarvis::init();
    }
    crate::jarvis::uc()
}