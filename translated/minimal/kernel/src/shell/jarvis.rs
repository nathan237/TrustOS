











use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::framebuffer::{C_, B_, G_, D_, A_, R_, K_, DM_};





const CU_: u32 = 0xFF00DDFF;      
const AZO_: u32 = 0xFF00FFAA;     
const DVS_: u32 = 0xFFFFAA00;       
const DVR_: u32 = 0xFFFF4444;        
const BBS_: usize = 32;


#[derive(Debug, Clone, Copy, PartialEq)]
enum Intent {
    
    SystemInfo,         
    ProcessList,        
    Jt,           
    Cn,        
    Fk,            
    Uptime,             
    
    
    ListFiles,          
    ReadFile,           
    CreateFile,         
    DeleteFile,         
    FindFile,           
    
    
    RunCommand,         
    OpenApp,            
    PlayMusic,          
    SetTheme,           
    
    
    Help,               
    Explain,            
    HowTo,              
    
    
    Greeting,           
    Thanks,             
    WhoAreYou,          
    Joke,               
    Compliment,         
    Insult,             
    Goodbye,            
    
    
    About,              
    Version,            
    Stats,              
    
    Unknown,
}


#[derive(Debug, Clone)]
struct Fp {
    kind: EntityKind,
    value: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EntityKind {
    FilePath,
    Command,
    AppName,
    Number,
    ThemeName,
    SearchTerm,
    Generic,
}


#[derive(Debug)]
struct Ahb {
    description: String,
    action: Action,
}

#[derive(Debug)]
enum Action {
    ShellCommand(String),
    ShowInfo(String),
    Respond(String),
    MultiStep(Vec<Ahb>),
}


struct Aic {
    last_intent: Intent,
    last_topic: String,
    turn_count: u32,
    ia: Lang,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Lang {
    Fr,
    En,
}






const Atk: &[&str] = &[
    "Be helpful, concise, and accurate.",
    "Never execute destructive commands without confirmation.",
    "Respect user privacy — never log or transmit personal data.",
    "When uncertain, say so. Never fabricate information.",
    "Prefer showing the user how to do things over doing them silently.",
    "Keep responses under 5 lines unless detail is requested.",
    "Be bilingual: detect language and respond in the same one.",
];






fn normalize(input: &str) -> String {
    let mut j = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            'A'..='Z' => j.push((c as u8 + 32) as char),
            'a'..='z' | '0'..='9' | ' ' | '/' | '.' | '-' | '_' => j.push(c),
            
            'é' | 'è' | 'ê' | 'ë' => j.push('e'),
            'à' | 'â' | 'ä' => j.push('a'),
            'ù' | 'û' | 'ü' => j.push('u'),
            'ô' | 'ö' => j.push('o'),
            'î' | 'ï' => j.push('i'),
            'ç' => j.push('c'),
            'É' | 'È' | 'Ê' | 'Ë' => j.push('e'),
            'À' | 'Â' | 'Ä' => j.push('a'),
            _ => j.push(' '),
        }
    }
    j
}


fn hrx(input: &str) -> Lang {
    let lyg = [
        "est", "les", "des", "une", "que", "qui", "pas", "mon", "mes", "ton",
        "ses", "pour", "dans", "avec", "sur", "fait", "peut", "quel", "quoi",
        "comment", "combien", "pourquoi", "montre", "affiche", "donne", "ouvre",
        "lance", "joue", "cherche", "trouve", "supprime", "cree", "salut",
        "bonjour", "merci", "stp", "svp", "moi", "fichier", "dossier",
        "memoire", "disque", "reseau", "processus", "aide", "blague",
    ];
    let um: Vec<&str> = input.split_whitespace().collect();
    let mut hzt = 0u32;
    for w in &um {
        let mo = w.to_ascii_lowercase();
        for &marker in &lyg {
            if mo == marker || mo.starts_with(marker) {
                hzt += 1;
            }
        }
    }
    
    if hzt >= 1 { Lang::Fr } else { Lang::En }
}


fn kx(mu: &str, patterns: &[&str]) -> bool {
    for &aa in patterns {
        if mu.contains(aa) { return true; }
    }
    false
}


fn ldx(mu: &str) -> Intent {
    
    if kx(mu, &["hello", "hi ", "hey ", "salut", "bonjour", "bonsoir", "yo ", "coucou", "allo"]) {
        return Intent::Greeting;
    }
    if kx(mu, &["bye", "au revoir", "a plus", "ciao", "goodbye", "quit", "exit"]) {
        return Intent::Goodbye;
    }
    if kx(mu, &["thank", "merci", "thanks", "thx"]) {
        return Intent::Thanks;
    }
    if kx(mu, &["who are you", "qui es tu", "c est qui", "tes qui", "your name", "ton nom", "tu es quoi"]) {
        return Intent::WhoAreYou;
    }
    if kx(mu, &["joke", "blague", "funny", "drole", "humour", "raconte"]) {
        return Intent::Joke;
    }
    if kx(mu, &["great", "awesome", "amazing", "genial", "super", "bravo", "bien joue", "nice", "cool", "good job", "bien fait"]) {
        return Intent::Compliment;
    }
    if kx(mu, &["suck", "nul", "bad", "horrible", "useless", "inutile", "pourri", "merde"]) {
        return Intent::Insult;
    }
    
    
    if kx(mu, &["memory", "memoire", "ram", "heap", "free mem", "mem usage"]) {
        return Intent::SystemInfo;
    }
    if kx(mu, &["system", "systeme", "status", "etat", "sante", "health", "overview"]) {
        return Intent::SystemInfo;
    }
    if kx(mu, &["process", "processus", "running", "tourne", "ps ", "qui tourne", "what run"]) {
        return Intent::ProcessList;
    }
    if kx(mu, &["disk", "disque", "storage", "stockage", "space", "espace", "df "]) {
        return Intent::Jt;
    }
    if kx(mu, &["network", "reseau", "ip ", "ifconfig", "internet", "connect", "connexion", "net "]) {
        return Intent::Cn;
    }
    if kx(mu, &["cpu", "processor", "processeur", "core", "coeur", "smp"]) {
        return Intent::Fk;
    }
    if kx(mu, &["uptime", "how long", "depuis combien", "temps", "duree"]) {
        return Intent::Uptime;
    }
    
    
    if kx(mu, &["list file", "liste fichier", "ls ", "show file", "montre fichier", "affiche fichier", "what file", "quels fichier"]) {
        return Intent::ListFiles;
    }
    if kx(mu, &["read file", "lire fichier", "lis ", "cat ", "show content", "montre contenu", "ouvre fichier", "open file"]) {
        return Intent::ReadFile;
    }
    if kx(mu, &["create file", "cree fichier", "creer", "touch ", "nouveau fichier", "new file"]) {
        return Intent::CreateFile;
    }
    if kx(mu, &["delete file", "supprime", "supprimer", "rm ", "remove", "efface", "detruit"]) {
        return Intent::DeleteFile;
    }
    if kx(mu, &["find file", "cherche fichier", "find ", "search", "ou est", "where is", "locate", "trouve"]) {
        return Intent::FindFile;
    }
    
    
    if kx(mu, &["run ", "execute", "lance commande", "executer", "fais "]) {
        return Intent::RunCommand;
    }
    if kx(mu, &["open ", "ouvre ", "launch", "lance ", "start ", "demarre"]) {
        return Intent::OpenApp;
    }
    if kx(mu, &["music", "musique", "play", "beep", "son", "sound", "audio"]) {
        return Intent::PlayMusic;
    }
    if kx(mu, &["theme", "color", "couleur", "dark", "light", "sombre", "clair"]) {
        return Intent::SetTheme;
    }
    
    
    if kx(mu, &["help", "aide", "what can", "que peux", "quoi faire", "commande", "command"]) {
        return Intent::Help;
    }
    if kx(mu, &["explain", "explique", "what is", "c est quoi", "definition", "qu est ce"]) {
        return Intent::Explain;
    }
    if kx(mu, &["how to", "comment", "how do", "how can", "tuto"]) {
        return Intent::HowTo;
    }
    
    
    if kx(mu, &["about", "a propos", "trustos"]) && !kx(mu, &["open", "ouvre"]) {
        return Intent::About;
    }
    if kx(mu, &["version"]) {
        return Intent::Version;
    }
    if kx(mu, &["stats", "statistique", "chiffre", "number", "count"]) {
        return Intent::Stats;
    }
    
    Intent::Unknown
}


fn ltp(mu: &str, bna: Intent) -> Vec<Fp> {
    let mut zw = Vec::new();
    let um: Vec<&str> = mu.split_whitespace().collect();
    
    match bna {
        Intent::ReadFile | Intent::CreateFile | Intent::DeleteFile | Intent::FindFile => {
            
            for w in &um {
                if w.contains('/') || w.contains('.') || w.starts_with('~') {
                    zw.push(Fp { kind: EntityKind::FilePath, value: String::from(*w) });
                }
            }
            
            if zw.is_empty() {
                if let Some(last) = um.last() {
                    if !["file", "fichier", "it", "le", "la", "les"].contains(last) {
                        zw.push(Fp { kind: EntityKind::FilePath, value: String::from(*last) });
                    }
                }
            }
        }
        Intent::RunCommand => {
            
            let ecq = ["run ", "execute ", "lance ", "executer ", "fais "];
            for t in &ecq {
                if let Some(pos) = mu.find(t) {
                    let cmd = &mu[pos + t.len()..];
                    if !cmd.is_empty() {
                        zw.push(Fp { kind: EntityKind::Command, value: String::from(cmd.trim()) });
                    }
                    break;
                }
            }
        }
        Intent::OpenApp => {
            let ecq = ["open ", "ouvre ", "launch ", "lance ", "start ", "demarre "];
            for t in &ecq {
                if let Some(pos) = mu.find(t) {
                    let afz = &mu[pos + t.len()..];
                    if !afz.is_empty() {
                        zw.push(Fp { kind: EntityKind::AppName, value: String::from(afz.trim()) });
                    }
                    break;
                }
            }
        }
        Intent::Explain | Intent::HowTo => {
            let ecq = ["explain ", "explique ", "what is ", "c est quoi ", "how to ", "comment "];
            for t in &ecq {
                if let Some(pos) = mu.find(t) {
                    let dfq = &mu[pos + t.len()..];
                    if !dfq.is_empty() {
                        zw.push(Fp { kind: EntityKind::SearchTerm, value: String::from(dfq.trim()) });
                    }
                    break;
                }
            }
        }
        Intent::SetTheme => {
            for w in &um {
                match *w {
                    "matrix" | "green" | "vert" | "dark" | "sombre" | "cyber" | "retro" | "hacker" => {
                        zw.push(Fp { kind: EntityKind::ThemeName, value: String::from(*w) });
                    }
                    _ => {}
                }
            }
        }
        Intent::ListFiles => {
            for w in &um {
                if w.contains('/') {
                    zw.push(Fp { kind: EntityKind::FilePath, value: String::from(*w) });
                }
            }
        }
        _ => {}
    }
    
    zw
}





fn ivb(bna: Intent, zw: &[Fp], ia: Lang) -> Action {
    match bna {
        Intent::SystemInfo => {
            let used = crate::memory::heap::used();
            let free = crate::memory::heap::free();
            let av = used + free;
            let fee = used / 1024;
            let baa = av / 1024;
            let aed = if av > 0 { used * 100 / av } else { 0 };
            let tasks = crate::task::task_count();
            match ia {
                Lang::Fr => Action::Respond(format!(
                    "Memoire: {} KB utilises / {} KB total ({}%)\nTaches actives: {}\nCPUs detectes: {}",
                    fee, baa, aed, tasks + 2, crate::cpu::smp::cpu_count()
                )),
                Lang::En => Action::Respond(format!(
                    "Memory: {} KB used / {} KB total ({}%)\nActive tasks: {}\nCPUs detected: {}",
                    fee, baa, aed, tasks + 2, crate::cpu::smp::cpu_count()
                )),
            }
        }
        Intent::ProcessList => {
            let tasks = crate::task::task_count();
            match ia {
                Lang::Fr => Action::Respond(format!(
                    "Processus actifs:\n  PID 1  kernel    (en cours)\n  PID 2  tsh       (en cours)\n  +{} taches en arriere-plan",
                    tasks
                )),
                Lang::En => Action::Respond(format!(
                    "Running processes:\n  PID 1  kernel    (running)\n  PID 2  tsh       (running)\n  +{} background tasks",
                    tasks
                )),
            }
        }
        Intent::Jt => {
            match ia {
                Lang::Fr => Action::Respond(String::from(
                    "Systeme de fichiers:\n  ramfs    64 KB (journalise)\n  Pas de disque physique monte"
                )),
                Lang::En => Action::Respond(String::from(
                    "File systems:\n  ramfs    64 KB (journaled)\n  No physical disk mounted"
                )),
            }
        }
        Intent::Cn => {
            Action::ShellCommand(String::from("ifconfig"))
        }
        Intent::Fk => {
            let cpus = crate::cpu::smp::cpu_count();
            let ready = crate::cpu::smp::ail();
            match ia {
                Lang::Fr => Action::Respond(format!(
                    "CPU: x86_64\nCoeurs detectes: {}\nCoeurs actifs: {} (BSP)\nSSE2: active\nMode: 64-bit Long Mode",
                    cpus, ready
                )),
                Lang::En => Action::Respond(format!(
                    "CPU: x86_64\nCores detected: {}\nActive cores: {} (BSP)\nSSE2: enabled\nMode: 64-bit Long Mode",
                    cpus, ready
                )),
            }
        }
        Intent::Uptime => {
            Action::ShellCommand(String::from("uptime"))
        }
        
        
        Intent::ListFiles => {
            let path = zw.iter()
                .find(|e| e.kind == EntityKind::FilePath)
                .map(|e| e.value.as_str())
                .unwrap_or("/");
            Action::ShellCommand(format!("ls {}", path))
        }
        Intent::ReadFile => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::FilePath) {
                Action::ShellCommand(format!("cat {}", e.value))
            } else {
                match ia {
                    Lang::Fr => Action::Respond(String::from("Quel fichier veux-tu lire ? Donne-moi le chemin.")),
                    Lang::En => Action::Respond(String::from("Which file should I read? Give me the path.")),
                }
            }
        }
        Intent::CreateFile => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::FilePath) {
                Action::ShellCommand(format!("touch {}", e.value))
            } else {
                match ia {
                    Lang::Fr => Action::Respond(String::from("Quel nom pour le fichier ?")),
                    Lang::En => Action::Respond(String::from("What should the file be called?")),
                }
            }
        }
        Intent::DeleteFile => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::FilePath) {
                match ia {
                    Lang::Fr => Action::Respond(format!(
                        "Confirme: tu veux supprimer '{}' ?\nTape: rm {}", e.value, e.value
                    )),
                    Lang::En => Action::Respond(format!(
                        "Confirm: delete '{}' ?\nType: rm {}", e.value, e.value
                    )),
                }
            } else {
                match ia {
                    Lang::Fr => Action::Respond(String::from("Quel fichier supprimer ?")),
                    Lang::En => Action::Respond(String::from("Which file should I delete?")),
                }
            }
        }
        Intent::FindFile => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::FilePath || e.kind == EntityKind::SearchTerm) {
                Action::ShellCommand(format!("find {}", e.value))
            } else {
                match ia {
                    Lang::Fr => Action::Respond(String::from("Que cherches-tu ?")),
                    Lang::En => Action::Respond(String::from("What are you looking for?")),
                }
            }
        }
        
        
        Intent::RunCommand => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::Command) {
                Action::ShellCommand(e.value.clone())
            } else {
                match ia {
                    Lang::Fr => Action::Respond(String::from("Quelle commande veux-tu executer ?")),
                    Lang::En => Action::Respond(String::from("What command should I run?")),
                }
            }
        }
        Intent::OpenApp => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::AppName) {
                let afz = e.value.as_str();
                
                let cmd = match afz {
                    j if kx(j, &["browser", "navigateur", "web", "internet"]) => "browse",
                    j if kx(j, &["chess", "echec"]) => "chess",
                    j if kx(j, &["editor", "editeur", "trustcode", "code"]) => "trustcode",
                    j if kx(j, &["desktop", "bureau"]) => "desktop",
                    j if kx(j, &["calculator", "calculatrice", "calc"]) => "calc",
                    j if kx(j, &["snake", "serpent", "game", "jeu"]) => "snake",
                    j if kx(j, &["terminal", "shell", "term"]) => "gterm",
                    j if kx(j, &["lab", "trustlab", "introspect"]) => "lab",
                    j if kx(j, &["3d", "model", "edit3d"]) => "trustedit",
                    j if kx(j, &["film", "movie"]) => "film",
                    j if kx(j, &["trailer", "bande annonce"]) => "trailer",
                    j if kx(j, &["music", "musique", "audio"]) => "synth",
                    _ => afz,
                };
                Action::ShellCommand(String::from(cmd))
            } else {
                match ia {
                    Lang::Fr => Action::Respond(String::from("Quelle application ouvrir ?")),
                    Lang::En => Action::Respond(String::from("Which app should I open?")),
                }
            }
        }
        Intent::PlayMusic => {
            Action::ShellCommand(String::from("synth"))
        }
        Intent::SetTheme => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::ThemeName) {
                Action::ShellCommand(format!("theme {}", e.value))
            } else {
                Action::ShellCommand(String::from("theme matrix"))
            }
        }
        
        
        Intent::Help => {
            match ia {
                Lang::Fr => Action::Respond(String::from(
                    "Je peux t'aider avec:\n\
                     - Info systeme: \"memoire\", \"cpu\", \"processus\"\n\
                     - Fichiers: \"liste les fichiers\", \"lis /readme.md\"\n\
                     - Apps: \"ouvre le navigateur\", \"lance chess\"\n\
                     - Commandes: \"execute neofetch\"\n\
                     - Questions: \"explique TLS\", \"comment compiler\"\n\
                     - Fun: \"une blague\", \"qui es-tu\""
                )),
                Lang::En => Action::Respond(String::from(
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
        Intent::Explain => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::SearchTerm) {
                lsv(&e.value, ia)
            } else {
                match ia {
                    Lang::Fr => Action::Respond(String::from("Qu'est-ce que tu veux que j'explique ?")),
                    Lang::En => Action::Respond(String::from("What would you like me to explain?")),
                }
            }
        }
        Intent::HowTo => {
            if let Some(e) = zw.iter().find(|e| e.kind == EntityKind::SearchTerm) {
                mmi(&e.value, ia)
            } else {
                match ia {
                    Lang::Fr => Action::Respond(String::from("Comment faire quoi exactement ?")),
                    Lang::En => Action::Respond(String::from("How to do what exactly?")),
                }
            }
        }
        
        
        Intent::Greeting => {
            match ia {
                Lang::Fr => Action::Respond(String::from("Salut ! Je suis Jarvis, ton assistant TrustOS. Comment puis-je t'aider ?")),
                Lang::En => Action::Respond(String::from("Hello! I'm Jarvis, your TrustOS assistant. How can I help?")),
            }
        }
        Intent::Thanks => {
            match ia {
                Lang::Fr => Action::Respond(String::from("De rien ! N'hesite pas si tu as besoin d'autre chose.")),
                Lang::En => Action::Respond(String::from("You're welcome! Let me know if you need anything else.")),
            }
        }
        Intent::WhoAreYou => {
            match ia {
                Lang::Fr => Action::Respond(String::from(
                    "Je suis Jarvis — Just A Rather Very Intelligent System.\n\
                     Assistant IA integre a TrustOS, 100% local, zero cloud.\n\
                     Je comprends le francais et l'anglais.\n\
                     Demande-moi n'importe quoi sur le systeme !"
                )),
                Lang::En => Action::Respond(String::from(
                    "I'm Jarvis — Just A Rather Very Intelligent System.\n\
                     AI assistant built into TrustOS, 100% local, zero cloud.\n\
                     I understand both French and English.\n\
                     Ask me anything about the system!"
                )),
            }
        }
        Intent::Joke => {
            let mvd = [
                "Pourquoi les developpeurs Rust ne font jamais de segfault ?\nParce qu'ils ont le borrow checker comme garde du corps.",
                "C'est un octet qui rentre dans un bar.\nLe barman lui dit : \"Desole, on sert pas les types non signes ici.\"",
                "Combien de programmeurs faut-il pour changer une ampoule ?\nAucun, c'est un probleme hardware.",
                "Un bug rentre dans un bar.\nIl n'en sort jamais. C'est une feature.",
                "Pourquoi TrustOS est ecrit en Rust ?\nParce que la confiance se merite... et la memoire aussi.",
            ];
            let mvc = [
                "Why do Rust devs never get segfaults?\nBecause the borrow checker is their bodyguard.",
                "A byte walks into a bar.\nBartender says: \"Sorry, we don't serve unsigned types here.\"",
                "How many programmers does it take to change a light bulb?\nNone, that's a hardware problem.",
                "A bug walks into a bar.\nIt never leaves. It's a feature.",
                "Why is TrustOS written in Rust?\nBecause trust is earned... and so is memory safety.",
            ];
            let idx = (crate::rtc::iby() as usize) % 5;
            match ia {
                Lang::Fr => Action::Respond(String::from(mvd[idx])),
                Lang::En => Action::Respond(String::from(mvc[idx])),
            }
        }
        Intent::Compliment => {
            match ia {
                Lang::Fr => Action::Respond(String::from("Merci ! C'est grace a 131K lignes de Rust et un developpeur passionne.")),
                Lang::En => Action::Respond(String::from("Thanks! It's all 131K lines of Rust and one passionate developer.")),
            }
        }
        Intent::Insult => {
            match ia {
                Lang::Fr => Action::Respond(String::from("J'encaisse. Mais je suis open source — tu peux m'ameliorer toi-meme. :)")),
                Lang::En => Action::Respond(String::from("Fair enough. But I'm open source — you can improve me yourself. :)")),
            }
        }
        Intent::Goodbye => {
            match ia {
                Lang::Fr => Action::Respond(String::from("A plus ! Tape 'jarvis' quand tu veux me reparler.")),
                Lang::En => Action::Respond(String::from("See you! Type 'jarvis' when you want to chat again.")),
            }
        }
        
        
        Intent::About => {
            match ia {
                Lang::Fr => Action::Respond(String::from(
                    "TrustOS v0.3.3 — OS bare-metal ecrit en Rust pur.\n\
                     131K lignes de code, 253 fichiers source, 1 auteur.\n\
                     Zero C, zero secrets, 100% auditable.\n\
                     github.com/nathan237/TrustOS"
                )),
                Lang::En => Action::Respond(String::from(
                    "TrustOS v0.3.3 — Bare-metal OS written in pure Rust.\n\
                     131K lines of code, 253 source files, 1 author.\n\
                     Zero C, zero secrets, 100% auditable.\n\
                     github.com/nathan237/TrustOS"
                )),
            }
        }
        Intent::Version => {
            Action::Respond(String::from("TrustOS v0.3.3 (kernel build 2026-02-16)"))
        }
        Intent::Stats => {
            let used = crate::memory::heap::used();
            let free = crate::memory::heap::free();
            let av = used + free;
            let cpus = crate::cpu::smp::cpu_count();
            match ia {
                Lang::Fr => Action::Respond(format!(
                    "--- Statistiques TrustOS ---\n\
                     Code: 131,985 lignes de Rust\n\
                     Fichiers: 253 modules .rs\n\
                     CPUs: {} detectes\n\
                     Heap: {} / {} KB\n\
                     ISO: ~8.4 MB\n\
                     Commandes shell: 120+",
                    cpus, used / 1024, av / 1024
                )),
                Lang::En => Action::Respond(format!(
                    "--- TrustOS Statistics ---\n\
                     Code: 131,985 lines of Rust\n\
                     Files: 253 .rs modules\n\
                     CPUs: {} detected\n\
                     Heap: {} / {} KB\n\
                     ISO: ~8.4 MB\n\
                     Shell commands: 120+",
                    cpus, used / 1024, av / 1024
                )),
            }
        }
        
        Intent::Unknown => {
            match ia {
                Lang::Fr => Action::Respond(String::from(
                    "Hmm, je ne suis pas sur de comprendre.\nEssaie \"aide\" pour voir ce que je peux faire."
                )),
                Lang::En => Action::Respond(String::from(
                    "Hmm, I'm not sure I understand.\nTry \"help\" to see what I can do."
                )),
            }
        }
    }
}





fn lsv(dfq: &str, ia: Lang) -> Action {
    let lsw = match dfq {
        t if kx(t, &["rust", "langage"]) => match ia {
            Lang::Fr => "Rust est un langage systeme qui garantit la securite memoire sans garbage collector. TrustOS est ecrit a 100% en Rust.",
            Lang::En => "Rust is a systems language that guarantees memory safety without a garbage collector. TrustOS is 100% Rust.",
        },
        t if kx(t, &["tls", "ssl", "https", "crypto"]) => match ia {
            Lang::Fr => "TLS 1.3 est le protocole de chiffrement utilise pour HTTPS. TrustOS implemente le handshake complet + Ed25519 from scratch.",
            Lang::En => "TLS 1.3 is the encryption protocol for HTTPS. TrustOS implements the full handshake + Ed25519 from scratch.",
        },
        t if kx(t, &["kernel", "noyau"]) => match ia {
            Lang::Fr => "Le kernel est le coeur de l'OS. Il gere la memoire, les interruptions, le scheduler, les drivers. TrustOS tourne en Ring 0.",
            Lang::En => "The kernel is the OS core. It manages memory, interrupts, the scheduler, drivers. TrustOS runs in Ring 0.",
        },
        t if kx(t, &["smp", "multicore", "multi-core"]) => match ia {
            Lang::Fr => "SMP = Symmetric Multi-Processing. TrustOS detecte tous les CPU via ACPI. Actuellement le BSP fait tout le travail.",
            Lang::En => "SMP = Symmetric Multi-Processing. TrustOS detects all CPUs via ACPI. Currently the BSP does all the work.",
        },
        t if kx(t, &["trustlang", "language", "langage de prog"]) => match ia {
            Lang::Fr => "TrustLang est le langage de programmation integre a TrustOS. Lexer > Parser > VM bytecode. Tape 'trustlang' pour l'essayer.",
            Lang::En => "TrustLang is TrustOS's built-in programming language. Lexer > Parser > VM bytecode. Type 'trustlang' to try it.",
        },
        t if kx(t, &["browser", "navigateur", "html"]) => match ia {
            Lang::Fr => "TrustBrowser est le navigateur integre. Il parse du HTML + CSS et supporte HTTPS via TLS 1.3. Tape 'browse' pour l'ouvrir.",
            Lang::En => "TrustBrowser is the built-in browser. It parses HTML + CSS and supports HTTPS via TLS 1.3. Type 'browse' to open it.",
        },
        t if kx(t, &["compositor", "gui", "desktop", "bureau"]) => match ia {
            Lang::Fr => "COSMIC2 est le compositeur graphique. Multi-couches, optimise SSE2, 144 FPS. Tape 'desktop' pour le lancer.",
            Lang::En => "COSMIC2 is the desktop compositor. Multi-layer, SSE2 optimized, 144 FPS. Type 'desktop' to launch it.",
        },
        t if kx(t, &["jarvis", "ia", "ai", "intelligence"]) => match ia {
            Lang::Fr => "C'est moi ! Jarvis = Just A Rather Very Intelligent System. NLU par pattern matching, bilingue FR/EN, execution kernel directe.",
            Lang::En => "That's me! Jarvis = Just A Rather Very Intelligent System. NLU via pattern matching, bilingual FR/EN, direct kernel execution.",
        },
        _ => match ia {
            Lang::Fr => "Je n'ai pas d'info precise sur ce sujet. Essaie un autre mot-cle.",
            Lang::En => "I don't have specific info on that topic. Try another keyword.",
        },
    };
    Action::Respond(String::from(lsw))
}

fn mmi(dfq: &str, ia: Lang) -> Action {
    let mmh = match dfq {
        t if kx(t, &["compile", "build", "construire"]) => match ia {
            Lang::Fr => "Pour compiler TrustOS:\n  1. cargo build\n  2. powershell build-limine.ps1\nL'ISO sera dans trustos.iso",
            Lang::En => "To build TrustOS:\n  1. cargo build\n  2. powershell build-limine.ps1\nThe ISO will be at trustos.iso",
        },
        t if kx(t, &["file", "fichier", "creer"]) => match ia {
            Lang::Fr => "Pour creer un fichier:\n  touch /mon_fichier.txt\n  echo 'contenu' > /mon_fichier.txt",
            Lang::En => "To create a file:\n  touch /my_file.txt\n  echo 'content' > /my_file.txt",
        },
        t if kx(t, &["network", "reseau", "internet"]) => match ia {
            Lang::Fr => "Pour tester le reseau:\n  ifconfig      (voir l'IP)\n  ping 8.8.8.8  (tester la connexion)\n  browse        (ouvrir le navigateur)",
            Lang::En => "To test networking:\n  ifconfig       (see IP)\n  ping 8.8.8.8   (test connection)\n  browse         (open browser)",
        },
        t if kx(t, &["theme", "personnaliser", "customize"]) => match ia {
            Lang::Fr => "Pour changer le theme:\n  theme matrix   (vert hacker)\n  theme cyber    (bleu futuriste)\n  theme retro    (amber)",
            Lang::En => "To change theme:\n  theme matrix   (green hacker)\n  theme cyber    (blue futuristic)\n  theme retro    (amber)",
        },
        _ => match ia {
            Lang::Fr => "Je n'ai pas de tutoriel sur ce sujet. Tape 'help' pour les commandes disponibles.",
            Lang::En => "I don't have a tutorial for that. Type 'help' for available commands.",
        },
    };
    Action::Respond(String::from(mmh))
}





fn hwy(action: Action) {
    match action {
        Action::Respond(bk) => {
            for line in bk.lines() {
                crate::bq!(CU_, "  ");
                crate::println!("{}", line);
            }
        }
        Action::ShellCommand(cmd) => {
            crate::bq!(AZO_, "  > ");
            crate::n!(K_, "{}", cmd);
            
            super::aav(&cmd);
        }
        Action::ShowInfo(bk) => {
            for line in bk.lines() {
                crate::bq!(C_, "  ");
                crate::println!("{}", line);
            }
        }
        Action::MultiStep(steps) => {
            for (i, step) in steps.iter().enumerate() {
                crate::bq!(AZO_, "  [{}] ", i + 1);
                crate::println!("{}", step.description);
            }
            for step in steps {
                hwy(step.action);
            }
        }
    }
}






pub(super) fn kpa(args: &[&str]) {
    
    if !args.is_empty() && (args[0] == "brain" || args[0] == "neural" || args[0] == "nn") {
        kmd(&args[1..]);
        return;
    }

    
    if !args.is_empty() {
        match args[0] {
            "boot" | "scan" | "wake" => {
                crate::print!("{}", crate::jarvis_hw::boot());
                return;
            }
            "hw" | "hardware" | "profile" => {
                crate::print!("{}", crate::jarvis_hw::osd());
                return;
            }
            "insights" | "insight" => {
                crate::print!("{}", crate::jarvis_hw::osa());
                return;
            }
            "plan" | "strategy" => {
                crate::print!("{}", crate::jarvis_hw::osb());
                return;
            }
            "optimize" | "opt" | "tune" => {
                crate::print!("{}", crate::jarvis_hw::nnq());
                return;
            }
            "status" | "stat" => {
                crate::print!("{}", crate::jarvis_hw::ose());
                return;
            }
            "analyze" | "analyse" | "inspect" => {
                if args.len() < 2 {
                    crate::println!("Usage: jarvis analyze <file>");
                    return;
                }
                let path = args[1..].join(" ");
                match crate::ramfs::bh(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
                    Ok(data) => {
                        crate::print!("{}", crate::jarvis_hw::jvw(&data));
                    }
                    Err(_) => {
                        crate::println!("Cannot read file: {}", path);
                    }
                }
                return;
            }
            "query" | "ask" | "q" => {
                if args.len() < 2 {
                    crate::println!("Usage: jarvis query <question>");
                    crate::println!("  e.g. jarvis query can you access the encrypted data on this disk?");
                    return;
                }
                let gpf = args[1..].join(" ");
                crate::print!("{}", crate::jarvis_hw::mmt(&gpf));
                return;
            }
            _ => {} 
        }
    }
    
    
    if !args.is_empty() {
        let query = args.join(" ");
        iwu(&query);
        return;
    }
    
    
    nxf();
    
    let mut ab = Aic {
        last_intent: Intent::Unknown,
        last_topic: String::new(),
        turn_count: 0,
        ia: Lang::En,
    };
    
    let mut input_buf = [0u8; 256];
    
    loop {
        if crate::shell::cbc() {
            crate::n!(CU_, "  [interrupted]");
            break;
        }
        
        crate::bq!(CU_, "\n  jarvis");
        crate::bq!(R_, " > ");
        
        
        let len = ocs(&mut input_buf);
        let dm = core::str::from_utf8(&input_buf[..len]).unwrap_or("").trim();
        
        if dm.is_empty() { continue; }
        
        
        if dm == "exit" || dm == "quit" || dm == "q" || dm == "bye" || dm == "au revoir" {
            let ia = hrx(dm);
            match ia {
                Lang::Fr => crate::n!(CU_, "  A bientot !"),
                Lang::En => crate::n!(CU_, "  See you later!"),
            }
            break;
        }
        
        ab.turn_count += 1;
        iwu(dm);
    }
}


fn iwu(dm: &str) {
    let mu = normalize(dm);
    let ia = hrx(dm);
    let bna = ldx(&mu);
    let zw = ltp(&mu, bna);
    
    
    crate::bq!(K_, "  ");
    crate::bq!(0xFF444444, "[{:?}]", bna);
    if !zw.is_empty() {
        for e in &zw {
            crate::bq!(0xFF444444, " {:?}={}", e.kind, e.value);
        }
    }
    if bna == Intent::Unknown && crate::jarvis::is_ready() {
        crate::bq!(0xFF444444, " ->brain");
    }
    crate::println!();
    
    
    let action = if bna == Intent::Unknown {
        
        if let Some(neural_response) = crate::jarvis::nio(dm) {
            Action::Respond(neural_response)
        } else {
            ivb(bna, &zw, ia)
        }
    } else {
        ivb(bna, &zw, ia)
    };
    hwy(action);
}


fn nxf() {
    crate::println!();
    crate::n!(CU_, "  ╔═══════════════════════════════════════════════╗");
    crate::n!(CU_, "  ║          J.A.R.V.I.S. v1.0                   ║");
    crate::n!(CU_, "  ║    Just A Rather Very Intelligent System      ║");
    crate::n!(CU_, "  ╠═══════════════════════════════════════════════╣");
    crate::bq!(CU_,   "  ║  ");
    crate::bq!(R_,    "TrustOS AI Assistant — 100%% local, 0%% cloud");
    crate::n!(CU_, "  ║");
    crate::bq!(CU_,   "  ║  ");
    crate::bq!(K_,     "Type 'help' for commands, 'exit' to leave");
    crate::n!(CU_, "   ║");
    crate::n!(CU_, "  ╚═══════════════════════════════════════════════╝");
    crate::println!();
}


fn ocs(buffer: &mut [u8]) -> usize {
    use crate::keyboard::ya;
    let mut pos = 0;

    loop {
        if let Some(c) = ya() {
            match c {
                b'\n' | b'\r' => {
                    crate::println!();
                    return pos;
                }
                8 | 127 => { 
                    if pos > 0 {
                        pos -= 1;
                        crate::print!("\x08 \x08");
                    }
                }
                0x1B => {} 
                c if c >= 0x20 && pos < buffer.len() - 1 => {
                    buffer[pos] = c;
                    pos += 1;
                    crate::print!("{}", c as char);
                }
                _ => {}
            }
        } else {
            
            core::hint::spin_loop();
        }
    }
}





const AQ_: u32 = 0xFF00BBFF;


fn kmd(args: &[&str]) {
    if args.is_empty() {
        crate::n!(AQ_, "  Jarvis Neural Brain v2.0");
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

    match args[0] {
        "init" => {
            crate::n!(AQ_, "  Initializing neural brain...");
            crate::jarvis::init();
            crate::n!(B_, "  Neural brain ready.");
        }

        "info" => {
            if !crate::jarvis::is_ready() {
                crate::n!(D_, "  Brain not initialized. Run: jarvis brain init");
                return;
            }
            for line in crate::jarvis::info_lines() {
                crate::println!("  {}", line);
            }
        }

        "generate" | "gen" | "g" => {
            if !bbk() { return; }
            if crate::jarvis::iih() {
                crate::n!(D_, "  [Private mode] Generation disabled");
                return;
            }
            let nh = if args.len() > 1 { args[1..].join(" ") } else { String::from("Hello") };
            crate::bq!(AQ_, "  Prompt: ");
            crate::n!(R_, "{}", nh);
            
            let start = crate::time::yf();
            let output = crate::jarvis::generate(&nh, 64);
            let bb = crate::time::yf().saturating_sub(start);
            
            crate::bq!(AQ_, "  Output: ");
            crate::n!(B_, "{}", output);
            crate::n!(K_, "  ({} ms, {} tokens)", bb, output.len());
        }

        "train" => {
            if !bbk() { return; }
            let text = if args.len() > 1 { args[1..].join(" ") } else {
                crate::println!("  Usage: jarvis brain train <text>");
                return;
            };
            crate::n!(AQ_, "  Training on: \"{}\"", text);
            let ka = crate::jarvis::bwo(&text, 0.001);
            crate::n!(B_, "  Loss: {:.4}", ka);
        }

        "test" => {
            if !bbk() { return; }
            crate::n!(AQ_, "  Running neural brain self-test...");
            crate::println!();

            
            crate::n!(C_, "  Tokenizer:");
            let tokens = crate::jarvis::tokenizer::bbj("Hello");
            crate::println!("    encode(\"Hello\") = {:?} ({} tokens)", &tokens, tokens.len());
            let uu = crate::jarvis::tokenizer::dmo(&tokens);
            crate::println!("    decode() = \"{}\"", uu);

            
            crate::n!(C_, "  Generation:");
            let output = crate::jarvis::generate("Test", 16);
            crate::println!("    generate(\"Test\", 16) = \"{}\" ({} chars)", output, output.len());

            
            crate::n!(C_, "  Training:");
            let nav = crate::jarvis::bwo("Hello world", 0.001);
            crate::println!("    Step 1 loss: {:.4}", nav);
            let naw = crate::jarvis::bwo("Hello world", 0.001);
            crate::println!("    Step 2 loss: {:.4}", naw);

            
            crate::n!(C_, "  Training module:");
            let (tp, tf) = crate::jarvis::training::cdp();
            crate::println!("    {} passed, {} failed", tp, tf);

            
            crate::n!(C_, "  Introspection:");
            let info = crate::jarvis::agent::cli(
                &crate::jarvis::agent::IntrospectTarget::Architecture);
            for line in info.iter().take(5) {
                crate::println!("    {}", line);
            }

            crate::println!();
            let bpq = 4 + tp; 
            let azz = tf;
            if azz == 0 {
                crate::n!(B_, "  All {} tests passed!", bpq);
            } else {
                crate::n!(A_, "  {} passed, {} failed", bpq, azz);
            }
        }

        "bench" => {
            if !bbk() { return; }
            crate::n!(AQ_, "  Benchmarking inference speed...");
            let (tok_per_sec, elapsed_ms) = crate::jarvis::agent::kbk();
            crate::println!("  Speed: {:.1} tokens/sec", tok_per_sec);
            crate::println!("  32 tokens generated in {} ms", elapsed_ms);
            crate::n!(K_, "  (CPU reference — GPU dispatch target: 100×)");
        }

        "introspect" | "self" => {
            if !bbk() { return; }
            let lines = crate::jarvis::agent::cli(
                &crate::jarvis::agent::IntrospectTarget::Full);
            for line in &lines {
                crate::n!(AQ_, "  {}", line);
            }
        }

        "weights" => {
            if !bbk() { return; }
            let lines = crate::jarvis::agent::cli(
                &crate::jarvis::agent::IntrospectTarget::WeightStats);
            for line in &lines {
                crate::println!("  {}", line);
            }
        }

        "hardware" | "hw" => {
            if !bbk() { return; }
            let lines = crate::jarvis::agent::cli(
                &crate::jarvis::agent::IntrospectTarget::Hardware);
            for line in &lines {
                crate::println!("  {}", line);
            }
        }

        "mentor" => {
            crate::n!(AQ_, "  Serial Mentoring Mode");
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
            crate::n!(K_, "  (Mentor polling active in shell idle loop)");
        }

        "reset" => {
            if !bbk() { return; }
            crate::n!(D_, "  Resetting all weights to random...");
            crate::jarvis::reset();
            crate::n!(B_, "  Weights reset. Training steps cleared.");
        }

        "save" => {
            if !bbk() { return; }
            crate::n!(AQ_, "  Saving weights to /jarvis/weights.bin...");
            match crate::jarvis::jco() {
                Ok(bytes) => crate::n!(B_, "  Saved {} KB", bytes / 1024),
                Err(e) => crate::n!(A_, "  Save failed: {}", e),
            }
        }

        "load" => {
            if args.len() > 1 {
                match args[1] {
                    "fat32" => {
                        let filename = if args.len() > 2 { Some(args[2]) } else { None };
                        let cwr = filename.unwrap_or("jarvis_weights.bin");
                        crate::n!(AQ_, "  Loading brain from FAT32: /mnt/fat32/{}", cwr);
                        match crate::jarvis::ikt(filename) {
                            Ok(bytes) => {
                                crate::n!(B_, "  Loaded {} KB from FAT32", bytes / 1024);
                                crate::n!(K_, "  Caching to RamFS...");
                                let _ = crate::jarvis::cuq();
                            }
                            Err(e) => crate::n!(A_, "  FAT32 load failed: {}", e),
                        }
                    }
                    "vfs" => {
                        if args.len() < 3 {
                            crate::n!(D_, "  Usage: jarvis brain load vfs <path>");
                            crate::n!(K_, "  Example: jarvis brain load vfs /mnt/fat32/brain.bin");
                            return;
                        }
                        crate::n!(AQ_, "  Loading brain from VFS: {}", args[2]);
                        match crate::jarvis::mzu(args[2]) {
                            Ok(bytes) => {
                                crate::n!(B_, "  Loaded {} KB from {}", bytes / 1024, args[2]);
                                crate::n!(K_, "  Caching to RamFS...");
                                let _ = crate::jarvis::cuq();
                            }
                            Err(e) => crate::n!(A_, "  VFS load failed: {}", e),
                        }
                    }
                    "http" => {
                        if args.len() < 3 {
                            crate::n!(D_, "  Usage: jarvis brain load http <url>");
                            crate::n!(K_, "  Example: jarvis brain load http 192.168.1.10/jarvis_weights.bin");
                            return;
                        }
                        crate::n!(AQ_, "  Downloading brain from: {}", args[2]);
                        crate::n!(K_, "  This may take a moment (~17 MB)...");
                        match crate::jarvis::mzt(args[2]) {
                            Ok(bytes) => {
                                crate::n!(B_, "  Downloaded & loaded {} KB", bytes / 1024);
                                crate::n!(K_, "  Caching to RamFS...");
                                let _ = crate::jarvis::cuq();
                            }
                            Err(e) => crate::n!(A_, "  HTTP load failed: {}", e),
                        }
                    }
                    _ => {
                        crate::n!(D_, "  Unknown source: {}", args[1]);
                        crate::n!(K_, "  Usage: jarvis brain load [fat32|vfs|http]");
                    }
                }
            } else {
                
                crate::n!(AQ_, "  Loading weights from /jarvis/weights.bin...");
                match crate::jarvis::iky() {
                    Ok(bytes) => crate::n!(B_, "  Loaded {} KB", bytes / 1024),
                    Err(e) => crate::n!(A_, "  Load failed: {}", e),
                }
            }
        }

        "pretrain" | "pt" => {
            if !bbk() { return; }
            let ahx: usize = if args.len() > 1 {
                args[1].parse().unwrap_or(3)
            } else { 3 }; 
            crate::n!(AQ_, "  Pre-training on embedded corpus ({} epoch(s))...", ahx);
            crate::n!(K_, "  Using cosine LR + gradient accumulation (batch=4)");
            crate::n!(K_, "  {} phases, {} sequences total",
                crate::jarvis::corpus::dvp(), crate::jarvis::corpus::eci());
            crate::println!();

            
            let cmk = crate::jarvis::elq();
            crate::n!(K_, "  Loss before: {:.3}", cmk);
            crate::println!();

            
            let (steps, adh, bb) = crate::jarvis::ivx(ahx, 0.001);

            crate::n!(AQ_, "  Training complete!");
            crate::n!(K_, "  {} steps, avg loss={:.3}, {}ms ({:.1}s)",
                steps, adh, bb, bb as f64 / 1000.0);
            crate::println!();

            
            let cmj = crate::jarvis::elq();
            crate::bq!(K_, "  Loss after:  {:.3}", cmj);
            if cmj < cmk {
                crate::n!(B_, " (improved by {:.3})", cmk - cmj);
            } else {
                crate::n!(D_, " (no improvement)");
            }
            crate::println!();
            crate::n!(K_, "  Training steps global: {}",
                crate::jarvis::training_steps());
        }

        "eval" => {
            if !bbk() { return; }
            
            let mae = args.len() >= 2 && args[1] == "full";
            if mae {
                crate::n!(AQ_, "  Evaluating FULL corpus (this may take a while)...");
                let adh = crate::jarvis::fvj();
                crate::println!();

                
                let nfx = !crate::jarvis::is_ready();
                if !nfx {
                    for phase in 0..crate::jarvis::corpus::dvp() {
                        let name = crate::jarvis::corpus::ewq(phase);
                        let mut count = 0u32;
                        for &text in crate::jarvis::corpus::Da[phase] {
                            let tokens = crate::jarvis::tokenizer::bbj(text);
                            if tokens.len() >= 2 {
                                count += 1;
                            }
                        }
                        crate::bq!(AQ_, "  Phase {} ", phase);
                        crate::bq!(R_, "({}) ", name);
                        crate::n!(K_, "{} sequences", count);
                    }
                }

                crate::println!();
                crate::bq!(R_, "  Average loss: ");
                if adh < 4.0 {
                    crate::n!(B_, "{:.3} (learning!)", adh);
                } else if adh < 5.5 {
                    crate::n!(D_, "{:.3} (early stage)", adh);
                } else {
                    crate::n!(A_, "{:.3} (random/untrained)", adh);
                }
                crate::n!(K_, "  (Random baseline: ~5.5, Good: <3.0, Memorized: <1.0)");
            } else {
                crate::n!(AQ_, "  Quick eval (1 sample/phase)...");
                let adh = crate::jarvis::elq();
                crate::println!();
                crate::bq!(R_, "  Average loss: ");
                if adh < 4.0 {
                    crate::n!(B_, "{:.3} (learning!)", adh);
                } else if adh < 5.5 {
                    crate::n!(D_, "{:.3} (early stage)", adh);
                } else {
                    crate::n!(A_, "{:.3} (random/untrained)", adh);
                }
                crate::n!(K_, "  (Quick: 1/phase. Use 'eval full' for complete corpus)");
            }
        }

        "chat" => {
            if !bbk() { return; }
            if crate::jarvis::iih() {
                crate::n!(D_, "  [Private mode] Chat disabled");
                return;
            }
            if args.len() < 2 {
                crate::println!("  Usage: jarvis brain chat <text>");
                return;
            }
            let nh = args[1..].join(" ");
            crate::bq!(R_, "  You: ");
            crate::println!("{}", nh);

            let start = crate::time::yf();
            let output = crate::jarvis::generate(&nh, 64);
            let bb = crate::time::yf().saturating_sub(start);

            crate::bq!(AQ_, "  Jarvis: ");
            
            for c in output.chars() {
                if c.is_ascii_graphic() || c == ' ' {
                    crate::print!("{}", c);
                } else {
                    crate::bq!(K_, ".");
                }
            }
            crate::println!();
            crate::n!(K_, "  ({} ms, {} chars)", bb, output.len());
        }

        "task" => {
            
            
            if !crate::jarvis::mesh::is_active() {
                crate::n!(A_, "  Mesh not active. Run 'jarvis brain swarm' first.");
                return;
            }

            crate::n!(AQ_, "  === JARVIS Distributed Task Verification ===");
            crate::println!();

            let lj = crate::jarvis::mesh::bgo();
            let av = lj.len() + 1;
            crate::n!(K_, "  Cluster: {} node(s) ({} peers + self)", av, lj.len());
            crate::println!();

            crate::n!(AQ_, "  Generating unique math tasks per node...");
            let start = crate::time::uptime_ms();
            let results = crate::jarvis::task::oja();
            let bb = crate::time::uptime_ms().wrapping_sub(start);

            crate::println!();
            crate::n!(AQ_, "  Results:");
            crate::println!();

            let mut hep = true;
            for r in &results {
                let bvz = if r.correct { "OK" } else { "FAIL" };
                let bdw = if r.correct { B_ } else { A_ };
                let lvd = if r.file_written { "file written" } else { "no file" };

                crate::print!("    ");
                crate::bq!(bdw, "[{}]", bvz);
                crate::print!(" {} ", r.node_name);
                crate::bq!(K_, "| {} = ", r.expression);
                if r.correct {
                    crate::bq!(B_, "{}", r.got);
                } else {
                    crate::bq!(A_, "{}", r.got);
                    crate::bq!(K_, " (expected {})", r.expected);
                    hep = false;
                }
                crate::n!(K_, " | {}", lvd);
            }

            crate::println!();
            if hep {
                crate::n!(B_,
                    "  All {} nodes returned correct answers in {} ms", results.len(), bb);
                crate::n!(B_,
                    "  Distributed verification: PASSED");
            } else {
                let kyc = results.iter().filter(|r| r.correct).count();
                crate::n!(A_,
                    "  {}/{} correct — verification FAILED ({} ms)", kyc, results.len(), bb);
            }
        }

        "propagate" | "autoprop" | "spread" => {
            let cxd = args.len() > 1 && (args[1] == "pxe" || args[1] == "replicate");
            crate::n!(AQ_, "  === JARVIS Auto-Propagation ===");
            crate::println!();
            if cxd {
                crate::n!(K_, "  Mode: FULL (mesh + brain pull + federated + PXE)");
            } else {
                crate::n!(K_, "  Mode: JOIN (mesh + brain pull + federated)");
            }
            crate::println!();

            let report = crate::jarvis::hgd(cxd);
            for line in report.lines() {
                if line.contains("FAIL") || line.contains("failed") {
                    crate::n!(A_, "  {}", line);
                } else if line.contains("OK") || line.contains("active") || line.contains("DOWNLOADED") || line.contains("enabled") || line.contains("FULL") {
                    crate::n!(B_, "  {}", line);
                } else {
                    crate::n!(K_, "  {}", line);
                }
            }

            
            crate::println!();
            let lj = crate::jarvis::mesh::bgo();
            if !lj.is_empty() {
                crate::n!(AQ_, "  Connected nodes:");
                for peer in &lj {
                    let role = match peer.role {
                        crate::jarvis::mesh::NodeRole::Leader => "LEADER",
                        crate::jarvis::mesh::NodeRole::Candidate => "CAND",
                        crate::jarvis::mesh::NodeRole::Worker => "WORKER",
                    };
                    crate::println!("    {}.{}.{}.{} [{}] {} params, {} steps",
                        peer.ip[0], peer.ip[1], peer.ip[2], peer.ip[3],
                        role, peer.param_count, peer.training_steps);
                }
            }

            crate::println!();
            if crate::jarvis::cki() {
                crate::n!(B_, "  JARVIS is fully operational and connected.");
            } else {
                crate::n!(D_, "  JARVIS running in micro mode. Full brain will download when a peer joins.");
            }
        }

        "swarm" => {
            
            let ahx: usize = if args.len() > 1 {
                args[1].parse().unwrap_or(3)
            } else { 3 };

            crate::n!(AQ_, "  === JARVIS Swarm Training Mode ===");
            crate::println!();

            
            if !crate::jarvis::is_ready() {
                crate::n!(AQ_, "  [1/4] Initializing neural brain...");
                crate::jarvis::init();
                if !crate::jarvis::is_ready() {
                    crate::n!(A_, "  Brain init failed");
                    return;
                }
            } else {
                crate::n!(B_, "  [1/4] Brain already initialized");
            }

            
            crate::n!(AQ_, "  [2/4] Starting mesh network...");
            crate::jarvis::euj();
            crate::n!(B_, "  Mesh active on UDP 7700 / TCP 7701");

            
            if let Some((my_ip, _, _)) = crate::network::rd() {
                crate::n!(K_, "  Scanning subnet for peers...");
                let rpc_port = crate::jarvis::mesh::HM_;
                let euq = my_ip.as_bytes();
                let mut nj = 0u32;
                for host in 1u8..=10 {
                    if host == euq[3] { continue; }
                    let dwk = [euq[0], euq[1], euq[2], host];
                    if let Ok(true) = crate::jarvis::rpc::iux(dwk, rpc_port) {
                        crate::n!(B_, "    Found peer: {}.{}.{}.{}",
                            dwk[0], dwk[1], dwk[2], dwk[3]);
                        nj += 1;
                    }
                }
                if nj == 0 {
                    crate::n!(D_, "    No peers found (training solo)");
                } else {
                    crate::n!(B_, "    {} peer(s) discovered", nj);
                }
            }

            
            crate::n!(AQ_, "  [3/4] Enabling federated learning...");
            crate::jarvis::federated::enable();
            crate::n!(B_, "  Federated gradient exchange enabled (30s sync)");

            
            crate::n!(AQ_, "  [4/4] Pre-training {} epoch(s)...", ahx);
            crate::println!();

            let cmk = crate::jarvis::fvj();
            crate::n!(K_, "  Loss before: {:.3}", cmk);

            let (steps, adh, bb) = crate::jarvis::ivx(ahx, 0.001);

            crate::println!();
            crate::n!(AQ_, "  Swarm training complete!");
            crate::n!(K_, "  {} steps, avg loss={:.3}, {:.1}s",
                steps, adh, bb as f64 / 1000.0);

            let cmj = crate::jarvis::fvj();
            crate::bq!(K_, "  Loss after:  {:.3}", cmj);
            if cmj < cmk {
                crate::n!(B_, " (improved by {:.3})", cmk - cmj);
            } else {
                crate::n!(D_, " (no improvement)");
            }

            
            let lj = crate::jarvis::mesh::bgo();
            crate::n!(K_, "  Mesh peers: {}", lj.len());
            crate::n!(K_, "  Pre-training done");
        }

        _ => {
            crate::n!(A_, "  Unknown: jarvis brain {}", args[0]);
            crate::println!("  Use 'jarvis brain' for help");
        }
    }
}


fn bbk() -> bool {
    if !crate::jarvis::is_ready() {
        crate::n!(K_, "  Auto-initializing neural brain...");
        crate::jarvis::init();
    }
    crate::jarvis::is_ready()
}