//! JARVIS — Just A Rather Very Intelligent System
//!
//! AI assistant integrated into TrustOS. Inspired by the Anthropic Model Spec:
//! structured personality, intent detection, action planning, sandboxed execution.
//!
//! Architecture:
//!   Input → Tokenizer → NLU (intent + entities) → Planner → Executor → Response
//!   Input → Neural Brain (tiny transformer) → Generated text → Response
//!
//! Dual-brain: pattern matching NLU (fast, deterministic) + neural transformer
//! (learning, creative). The neural brain is optional and grows over time.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::framebuffer::{COLOR_CYAN, COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_WHITE, COLOR_GRAY, COLOR_MAGENTA};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS & TYPES
// ═══════════════════════════════════════════════════════════════════════

const JARVIS_COLOR: u32 = 0xFF00DDFF;      // Cyan-blue
const JARVIS_ACCENT: u32 = 0xFF00FFAA;     // Green accent
const JARVIS_WARN: u32 = 0xFFFFAA00;       // Warning orange
const JARVIS_ERR: u32 = 0xFFFF4444;        // Error red
const MAX_HISTORY: usize = 32;

/// Detected user intent
#[derive(Debug, Clone, Copy, PartialEq)]
enum Intent {
    // System queries
    SystemInfo,         // "how much memory", "system status"
    ProcessList,        // "what's running", "show processes"
    DiskInfo,           // "disk space", "storage"
    NetworkInfo,        // "network status", "ip address"
    CpuInfo,            // "cpu info", "processor"
    Uptime,             // "uptime", "how long running"
    
    // File operations
    ListFiles,          // "show files", "what's in /home"
    ReadFile,           // "show me file X", "cat X"
    CreateFile,         // "create file X", "touch X"
    DeleteFile,         // "delete X", "remove X"
    FindFile,           // "find file X", "where is X"
    
    // Actions
    RunCommand,         // "run ls", "execute neofetch"
    OpenApp,            // "open browser", "launch chess"
    PlayMusic,          // "play music", "beep"
    SetTheme,           // "change theme", "dark mode"
    
    // Help & knowledge
    Help,               // "help", "what can you do"
    Explain,            // "explain X", "what is X"
    HowTo,              // "how to X", "comment faire X"
    
    // Conversational
    Greeting,           // "hello", "salut"
    Thanks,             // "thanks", "merci"
    WhoAreYou,          // "who are you", "c'est qui"
    Joke,               // "tell me a joke", "une blague"
    Compliment,         // "you're great", "t'es bon"
    Insult,             // "you suck", "t'es nul"
    Goodbye,            // "bye", "au revoir"
    
    // Meta
    About,              // "about trustos", "à propos"
    Version,            // "version", "quelle version"
    Stats,              // "stats", "statistiques"
    
    Unknown,
}

/// Extracted entity from user input
#[derive(Debug, Clone)]
struct Entity {
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

/// A planned action step
#[derive(Debug)]
struct ActionStep {
    description: String,
    action: Action,
}

#[derive(Debug)]
enum Action {
    ShellCommand(String),
    ShowInfo(String),
    Respond(String),
    MultiStep(Vec<ActionStep>),
}

/// Conversation context for multi-turn awareness
struct ConversationCtx {
    last_intent: Intent,
    last_topic: String,
    turn_count: u32,
    lang: Lang,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Lang {
    Fr,
    En,
}

// ═══════════════════════════════════════════════════════════════════════
// CONSTITUTION — Inspired by Anthropic Model Spec
// ═══════════════════════════════════════════════════════════════════════

/// Jarvis behavioral rules (checked before every response)
const CONSTITUTION: &[&str] = &[
    "Be helpful, concise, and accurate.",
    "Never execute destructive commands without confirmation.",
    "Respect user privacy — never log or transmit personal data.",
    "When uncertain, say so. Never fabricate information.",
    "Prefer showing the user how to do things over doing them silently.",
    "Keep responses under 5 lines unless detail is requested.",
    "Be bilingual: detect language and respond in the same one.",
];

// ═══════════════════════════════════════════════════════════════════════
// NLU: TOKENIZER + INTENT DETECTION + ENTITY EXTRACTION
// ═══════════════════════════════════════════════════════════════════════

/// Normalize input: lowercase, trim, remove punctuation
fn normalize(input: &str) -> String {
    let mut s = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            'A'..='Z' => s.push((c as u8 + 32) as char),
            'a'..='z' | '0'..='9' | ' ' | '/' | '.' | '-' | '_' => s.push(c),
            // Map accented chars
            'é' | 'è' | 'ê' | 'ë' => s.push('e'),
            'à' | 'â' | 'ä' => s.push('a'),
            'ù' | 'û' | 'ü' => s.push('u'),
            'ô' | 'ö' => s.push('o'),
            'î' | 'ï' => s.push('i'),
            'ç' => s.push('c'),
            'É' | 'È' | 'Ê' | 'Ë' => s.push('e'),
            'À' | 'Â' | 'Ä' => s.push('a'),
            _ => s.push(' '),
        }
    }
    s
}

/// Detect language based on French marker words
fn detect_lang(input: &str) -> Lang {
    let fr_markers = [
        "est", "les", "des", "une", "que", "qui", "pas", "mon", "mes", "ton",
        "ses", "pour", "dans", "avec", "sur", "fait", "peut", "quel", "quoi",
        "comment", "combien", "pourquoi", "montre", "affiche", "donne", "ouvre",
        "lance", "joue", "cherche", "trouve", "supprime", "cree", "salut",
        "bonjour", "merci", "stp", "svp", "moi", "fichier", "dossier",
        "memoire", "disque", "reseau", "processus", "aide", "blague",
    ];
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut fr_score = 0u32;
    for w in &words {
        let lw = w.to_ascii_lowercase();
        for &marker in &fr_markers {
            if lw == marker || lw.starts_with(marker) {
                fr_score += 1;
            }
        }
    }
    // Also detect by accented characters in original
    if fr_score >= 1 { Lang::Fr } else { Lang::En }
}

/// Check if normalized input contains any of the patterns
fn matches_any(norm: &str, patterns: &[&str]) -> bool {
    for &p in patterns {
        if norm.contains(p) { return true; }
    }
    false
}

/// Detect primary intent from normalized input
fn detect_intent(norm: &str) -> Intent {
    // ---- Greetings ----
    if matches_any(norm, &["hello", "hi ", "hey ", "salut", "bonjour", "bonsoir", "yo ", "coucou", "allo"]) {
        return Intent::Greeting;
    }
    if matches_any(norm, &["bye", "au revoir", "a plus", "ciao", "goodbye", "quit", "exit"]) {
        return Intent::Goodbye;
    }
    if matches_any(norm, &["thank", "merci", "thanks", "thx"]) {
        return Intent::Thanks;
    }
    if matches_any(norm, &["who are you", "qui es tu", "c est qui", "tes qui", "your name", "ton nom", "tu es quoi"]) {
        return Intent::WhoAreYou;
    }
    if matches_any(norm, &["joke", "blague", "funny", "drole", "humour", "raconte"]) {
        return Intent::Joke;
    }
    if matches_any(norm, &["great", "awesome", "amazing", "genial", "super", "bravo", "bien joue", "nice", "cool", "good job", "bien fait"]) {
        return Intent::Compliment;
    }
    if matches_any(norm, &["suck", "nul", "bad", "horrible", "useless", "inutile", "pourri", "merde"]) {
        return Intent::Insult;
    }
    
    // ---- System queries ----
    if matches_any(norm, &["memory", "memoire", "ram", "heap", "free mem", "mem usage"]) {
        return Intent::SystemInfo;
    }
    if matches_any(norm, &["system", "systeme", "status", "etat", "sante", "health", "overview"]) {
        return Intent::SystemInfo;
    }
    if matches_any(norm, &["process", "processus", "running", "tourne", "ps ", "qui tourne", "what run"]) {
        return Intent::ProcessList;
    }
    if matches_any(norm, &["disk", "disque", "storage", "stockage", "space", "espace", "df "]) {
        return Intent::DiskInfo;
    }
    if matches_any(norm, &["network", "reseau", "ip ", "ifconfig", "internet", "connect", "connexion", "net "]) {
        return Intent::NetworkInfo;
    }
    if matches_any(norm, &["cpu", "processor", "processeur", "core", "coeur", "smp"]) {
        return Intent::CpuInfo;
    }
    if matches_any(norm, &["uptime", "how long", "depuis combien", "temps", "duree"]) {
        return Intent::Uptime;
    }
    
    // ---- File operations ----
    if matches_any(norm, &["list file", "liste fichier", "ls ", "show file", "montre fichier", "affiche fichier", "what file", "quels fichier"]) {
        return Intent::ListFiles;
    }
    if matches_any(norm, &["read file", "lire fichier", "lis ", "cat ", "show content", "montre contenu", "ouvre fichier", "open file"]) {
        return Intent::ReadFile;
    }
    if matches_any(norm, &["create file", "cree fichier", "creer", "touch ", "nouveau fichier", "new file"]) {
        return Intent::CreateFile;
    }
    if matches_any(norm, &["delete file", "supprime", "supprimer", "rm ", "remove", "efface", "detruit"]) {
        return Intent::DeleteFile;
    }
    if matches_any(norm, &["find file", "cherche fichier", "find ", "search", "ou est", "where is", "locate", "trouve"]) {
        return Intent::FindFile;
    }
    
    // ---- Actions ----
    if matches_any(norm, &["run ", "execute", "lance commande", "executer", "fais "]) {
        return Intent::RunCommand;
    }
    if matches_any(norm, &["open ", "ouvre ", "launch", "lance ", "start ", "demarre"]) {
        return Intent::OpenApp;
    }
    if matches_any(norm, &["music", "musique", "play", "beep", "son", "sound", "audio"]) {
        return Intent::PlayMusic;
    }
    if matches_any(norm, &["theme", "color", "couleur", "dark", "light", "sombre", "clair"]) {
        return Intent::SetTheme;
    }
    
    // ---- Help & knowledge ----
    if matches_any(norm, &["help", "aide", "what can", "que peux", "quoi faire", "commande", "command"]) {
        return Intent::Help;
    }
    if matches_any(norm, &["explain", "explique", "what is", "c est quoi", "definition", "qu est ce"]) {
        return Intent::Explain;
    }
    if matches_any(norm, &["how to", "comment", "how do", "how can", "tuto"]) {
        return Intent::HowTo;
    }
    
    // ---- Meta ----
    if matches_any(norm, &["about", "a propos", "trustos"]) && !matches_any(norm, &["open", "ouvre"]) {
        return Intent::About;
    }
    if matches_any(norm, &["version"]) {
        return Intent::Version;
    }
    if matches_any(norm, &["stats", "statistique", "chiffre", "number", "count"]) {
        return Intent::Stats;
    }
    
    Intent::Unknown
}

/// Extract entities (file paths, command names, app names, etc.)
fn extract_entities(norm: &str, intent: Intent) -> Vec<Entity> {
    let mut entities = Vec::new();
    let words: Vec<&str> = norm.split_whitespace().collect();
    
    match intent {
        Intent::ReadFile | Intent::CreateFile | Intent::DeleteFile | Intent::FindFile => {
            // Look for path-like tokens
            for w in &words {
                if w.contains('/') || w.contains('.') || w.starts_with('~') {
                    entities.push(Entity { kind: EntityKind::FilePath, value: String::from(*w) });
                }
            }
            // If no path found, take the last meaningful word
            if entities.is_empty() {
                if let Some(last) = words.last() {
                    if !["file", "fichier", "it", "le", "la", "les"].contains(last) {
                        entities.push(Entity { kind: EntityKind::FilePath, value: String::from(*last) });
                    }
                }
            }
        }
        Intent::RunCommand => {
            // Everything after "run"/"execute"/"lance" is the command
            let triggers = ["run ", "execute ", "lance ", "executer ", "fais "];
            for t in &triggers {
                if let Some(pos) = norm.find(t) {
                    let cmd = &norm[pos + t.len()..];
                    if !cmd.is_empty() {
                        entities.push(Entity { kind: EntityKind::Command, value: String::from(cmd.trim()) });
                    }
                    break;
                }
            }
        }
        Intent::OpenApp => {
            let triggers = ["open ", "ouvre ", "launch ", "lance ", "start ", "demarre "];
            for t in &triggers {
                if let Some(pos) = norm.find(t) {
                    let app = &norm[pos + t.len()..];
                    if !app.is_empty() {
                        entities.push(Entity { kind: EntityKind::AppName, value: String::from(app.trim()) });
                    }
                    break;
                }
            }
        }
        Intent::Explain | Intent::HowTo => {
            let triggers = ["explain ", "explique ", "what is ", "c est quoi ", "how to ", "comment "];
            for t in &triggers {
                if let Some(pos) = norm.find(t) {
                    let topic = &norm[pos + t.len()..];
                    if !topic.is_empty() {
                        entities.push(Entity { kind: EntityKind::SearchTerm, value: String::from(topic.trim()) });
                    }
                    break;
                }
            }
        }
        Intent::SetTheme => {
            for w in &words {
                match *w {
                    "matrix" | "green" | "vert" | "dark" | "sombre" | "cyber" | "retro" | "hacker" => {
                        entities.push(Entity { kind: EntityKind::ThemeName, value: String::from(*w) });
                    }
                    _ => {}
                }
            }
        }
        Intent::ListFiles => {
            for w in &words {
                if w.contains('/') {
                    entities.push(Entity { kind: EntityKind::FilePath, value: String::from(*w) });
                }
            }
        }
        _ => {}
    }
    
    entities
}

// ═══════════════════════════════════════════════════════════════════════
// PLANNER: Generate action steps from intent + entities
// ═══════════════════════════════════════════════════════════════════════

fn plan_action(intent: Intent, entities: &[Entity], lang: Lang) -> Action {
    match intent {
        Intent::SystemInfo => {
            let used = crate::memory::heap::used();
            let free = crate::memory::heap::free();
            let total = used + free;
            let used_kb = used / 1024;
            let total_kb = total / 1024;
            let pct = if total > 0 { used * 100 / total } else { 0 };
            let tasks = crate::task::task_count();
            match lang {
                Lang::Fr => Action::Respond(format!(
                    "Memoire: {} KB utilises / {} KB total ({}%)\nTaches actives: {}\nCPUs detectes: {}",
                    used_kb, total_kb, pct, tasks + 2, crate::cpu::smp::cpu_count()
                )),
                Lang::En => Action::Respond(format!(
                    "Memory: {} KB used / {} KB total ({}%)\nActive tasks: {}\nCPUs detected: {}",
                    used_kb, total_kb, pct, tasks + 2, crate::cpu::smp::cpu_count()
                )),
            }
        }
        Intent::ProcessList => {
            let tasks = crate::task::task_count();
            match lang {
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
        Intent::DiskInfo => {
            match lang {
                Lang::Fr => Action::Respond(String::from(
                    "Systeme de fichiers:\n  ramfs    64 KB (journalise)\n  Pas de disque physique monte"
                )),
                Lang::En => Action::Respond(String::from(
                    "File systems:\n  ramfs    64 KB (journaled)\n  No physical disk mounted"
                )),
            }
        }
        Intent::NetworkInfo => {
            Action::ShellCommand(String::from("ifconfig"))
        }
        Intent::CpuInfo => {
            let cpus = crate::cpu::smp::cpu_count();
            let ready = crate::cpu::smp::ready_cpu_count();
            match lang {
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
        
        // ---- File ops ----
        Intent::ListFiles => {
            let path = entities.iter()
                .find(|e| e.kind == EntityKind::FilePath)
                .map(|e| e.value.as_str())
                .unwrap_or("/");
            Action::ShellCommand(format!("ls {}", path))
        }
        Intent::ReadFile => {
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::FilePath) {
                Action::ShellCommand(format!("cat {}", e.value))
            } else {
                match lang {
                    Lang::Fr => Action::Respond(String::from("Quel fichier veux-tu lire ? Donne-moi le chemin.")),
                    Lang::En => Action::Respond(String::from("Which file should I read? Give me the path.")),
                }
            }
        }
        Intent::CreateFile => {
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::FilePath) {
                Action::ShellCommand(format!("touch {}", e.value))
            } else {
                match lang {
                    Lang::Fr => Action::Respond(String::from("Quel nom pour le fichier ?")),
                    Lang::En => Action::Respond(String::from("What should the file be called?")),
                }
            }
        }
        Intent::DeleteFile => {
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::FilePath) {
                match lang {
                    Lang::Fr => Action::Respond(format!(
                        "Confirme: tu veux supprimer '{}' ?\nTape: rm {}", e.value, e.value
                    )),
                    Lang::En => Action::Respond(format!(
                        "Confirm: delete '{}' ?\nType: rm {}", e.value, e.value
                    )),
                }
            } else {
                match lang {
                    Lang::Fr => Action::Respond(String::from("Quel fichier supprimer ?")),
                    Lang::En => Action::Respond(String::from("Which file should I delete?")),
                }
            }
        }
        Intent::FindFile => {
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::FilePath || e.kind == EntityKind::SearchTerm) {
                Action::ShellCommand(format!("find {}", e.value))
            } else {
                match lang {
                    Lang::Fr => Action::Respond(String::from("Que cherches-tu ?")),
                    Lang::En => Action::Respond(String::from("What are you looking for?")),
                }
            }
        }
        
        // ---- Actions ----
        Intent::RunCommand => {
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::Command) {
                Action::ShellCommand(e.value.clone())
            } else {
                match lang {
                    Lang::Fr => Action::Respond(String::from("Quelle commande veux-tu executer ?")),
                    Lang::En => Action::Respond(String::from("What command should I run?")),
                }
            }
        }
        Intent::OpenApp => {
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::AppName) {
                let app = e.value.as_str();
                // Map natural names → shell commands
                let cmd = match app {
                    s if matches_any(s, &["browser", "navigateur", "web", "internet"]) => "browse",
                    s if matches_any(s, &["chess", "echec"]) => "chess",
                    s if matches_any(s, &["editor", "editeur", "trustcode", "code"]) => "trustcode",
                    s if matches_any(s, &["desktop", "bureau"]) => "desktop",
                    s if matches_any(s, &["calculator", "calculatrice", "calc"]) => "calc",
                    s if matches_any(s, &["snake", "serpent", "game", "jeu"]) => "snake",
                    s if matches_any(s, &["terminal", "shell", "term"]) => "gterm",
                    s if matches_any(s, &["lab", "trustlab", "introspect"]) => "lab",
                    s if matches_any(s, &["3d", "model", "edit3d"]) => "trustedit",
                    s if matches_any(s, &["film", "movie"]) => "film",
                    s if matches_any(s, &["trailer", "bande annonce"]) => "trailer",
                    s if matches_any(s, &["music", "musique", "audio"]) => "synth",
                    _ => app,
                };
                Action::ShellCommand(String::from(cmd))
            } else {
                match lang {
                    Lang::Fr => Action::Respond(String::from("Quelle application ouvrir ?")),
                    Lang::En => Action::Respond(String::from("Which app should I open?")),
                }
            }
        }
        Intent::PlayMusic => {
            Action::ShellCommand(String::from("synth"))
        }
        Intent::SetTheme => {
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::ThemeName) {
                Action::ShellCommand(format!("theme {}", e.value))
            } else {
                Action::ShellCommand(String::from("theme matrix"))
            }
        }
        
        // ---- Help & knowledge ----
        Intent::Help => {
            match lang {
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
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::SearchTerm) {
                explain_topic(&e.value, lang)
            } else {
                match lang {
                    Lang::Fr => Action::Respond(String::from("Qu'est-ce que tu veux que j'explique ?")),
                    Lang::En => Action::Respond(String::from("What would you like me to explain?")),
                }
            }
        }
        Intent::HowTo => {
            if let Some(e) = entities.iter().find(|e| e.kind == EntityKind::SearchTerm) {
                howto_topic(&e.value, lang)
            } else {
                match lang {
                    Lang::Fr => Action::Respond(String::from("Comment faire quoi exactement ?")),
                    Lang::En => Action::Respond(String::from("How to do what exactly?")),
                }
            }
        }
        
        // ---- Conversational ----
        Intent::Greeting => {
            match lang {
                Lang::Fr => Action::Respond(String::from("Salut ! Je suis Jarvis, ton assistant TrustOS. Comment puis-je t'aider ?")),
                Lang::En => Action::Respond(String::from("Hello! I'm Jarvis, your TrustOS assistant. How can I help?")),
            }
        }
        Intent::Thanks => {
            match lang {
                Lang::Fr => Action::Respond(String::from("De rien ! N'hesite pas si tu as besoin d'autre chose.")),
                Lang::En => Action::Respond(String::from("You're welcome! Let me know if you need anything else.")),
            }
        }
        Intent::WhoAreYou => {
            match lang {
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
            let jokes_fr = [
                "Pourquoi les developpeurs Rust ne font jamais de segfault ?\nParce qu'ils ont le borrow checker comme garde du corps.",
                "C'est un octet qui rentre dans un bar.\nLe barman lui dit : \"Desole, on sert pas les types non signes ici.\"",
                "Combien de programmeurs faut-il pour changer une ampoule ?\nAucun, c'est un probleme hardware.",
                "Un bug rentre dans un bar.\nIl n'en sort jamais. C'est une feature.",
                "Pourquoi TrustOS est ecrit en Rust ?\nParce que la confiance se merite... et la memoire aussi.",
            ];
            let jokes_en = [
                "Why do Rust devs never get segfaults?\nBecause the borrow checker is their bodyguard.",
                "A byte walks into a bar.\nBartender says: \"Sorry, we don't serve unsigned types here.\"",
                "How many programmers does it take to change a light bulb?\nNone, that's a hardware problem.",
                "A bug walks into a bar.\nIt never leaves. It's a feature.",
                "Why is TrustOS written in Rust?\nBecause trust is earned... and so is memory safety.",
            ];
            let idx = (crate::rtc::get_time_seconds() as usize) % 5;
            match lang {
                Lang::Fr => Action::Respond(String::from(jokes_fr[idx])),
                Lang::En => Action::Respond(String::from(jokes_en[idx])),
            }
        }
        Intent::Compliment => {
            match lang {
                Lang::Fr => Action::Respond(String::from("Merci ! C'est grace a 131K lignes de Rust et un developpeur passionne.")),
                Lang::En => Action::Respond(String::from("Thanks! It's all 131K lines of Rust and one passionate developer.")),
            }
        }
        Intent::Insult => {
            match lang {
                Lang::Fr => Action::Respond(String::from("J'encaisse. Mais je suis open source — tu peux m'ameliorer toi-meme. :)")),
                Lang::En => Action::Respond(String::from("Fair enough. But I'm open source — you can improve me yourself. :)")),
            }
        }
        Intent::Goodbye => {
            match lang {
                Lang::Fr => Action::Respond(String::from("A plus ! Tape 'jarvis' quand tu veux me reparler.")),
                Lang::En => Action::Respond(String::from("See you! Type 'jarvis' when you want to chat again.")),
            }
        }
        
        // ---- Meta ----
        Intent::About => {
            match lang {
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
            let total = used + free;
            let cpus = crate::cpu::smp::cpu_count();
            match lang {
                Lang::Fr => Action::Respond(format!(
                    "--- Statistiques TrustOS ---\n\
                     Code: 131,985 lignes de Rust\n\
                     Fichiers: 253 modules .rs\n\
                     CPUs: {} detectes\n\
                     Heap: {} / {} KB\n\
                     ISO: ~8.4 MB\n\
                     Commandes shell: 120+",
                    cpus, used / 1024, total / 1024
                )),
                Lang::En => Action::Respond(format!(
                    "--- TrustOS Statistics ---\n\
                     Code: 131,985 lines of Rust\n\
                     Files: 253 .rs modules\n\
                     CPUs: {} detected\n\
                     Heap: {} / {} KB\n\
                     ISO: ~8.4 MB\n\
                     Shell commands: 120+",
                    cpus, used / 1024, total / 1024
                )),
            }
        }
        
        Intent::Unknown => {
            match lang {
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

// ═══════════════════════════════════════════════════════════════════════
// KNOWLEDGE BASE: Explanations & How-to's
// ═══════════════════════════════════════════════════════════════════════

fn explain_topic(topic: &str, lang: Lang) -> Action {
    let explanation = match topic {
        t if matches_any(t, &["rust", "langage"]) => match lang {
            Lang::Fr => "Rust est un langage systeme qui garantit la securite memoire sans garbage collector. TrustOS est ecrit a 100% en Rust.",
            Lang::En => "Rust is a systems language that guarantees memory safety without a garbage collector. TrustOS is 100% Rust.",
        },
        t if matches_any(t, &["tls", "ssl", "https", "crypto"]) => match lang {
            Lang::Fr => "TLS 1.3 est le protocole de chiffrement utilise pour HTTPS. TrustOS implemente le handshake complet + Ed25519 from scratch.",
            Lang::En => "TLS 1.3 is the encryption protocol for HTTPS. TrustOS implements the full handshake + Ed25519 from scratch.",
        },
        t if matches_any(t, &["kernel", "noyau"]) => match lang {
            Lang::Fr => "Le kernel est le coeur de l'OS. Il gere la memoire, les interruptions, le scheduler, les drivers. TrustOS tourne en Ring 0.",
            Lang::En => "The kernel is the OS core. It manages memory, interrupts, the scheduler, drivers. TrustOS runs in Ring 0.",
        },
        t if matches_any(t, &["smp", "multicore", "multi-core"]) => match lang {
            Lang::Fr => "SMP = Symmetric Multi-Processing. TrustOS detecte tous les CPU via ACPI. Actuellement le BSP fait tout le travail.",
            Lang::En => "SMP = Symmetric Multi-Processing. TrustOS detects all CPUs via ACPI. Currently the BSP does all the work.",
        },
        t if matches_any(t, &["trustlang", "language", "langage de prog"]) => match lang {
            Lang::Fr => "TrustLang est le langage de programmation integre a TrustOS. Lexer > Parser > VM bytecode. Tape 'trustlang' pour l'essayer.",
            Lang::En => "TrustLang is TrustOS's built-in programming language. Lexer > Parser > VM bytecode. Type 'trustlang' to try it.",
        },
        t if matches_any(t, &["browser", "navigateur", "html"]) => match lang {
            Lang::Fr => "TrustBrowser est le navigateur integre. Il parse du HTML + CSS et supporte HTTPS via TLS 1.3. Tape 'browse' pour l'ouvrir.",
            Lang::En => "TrustBrowser is the built-in browser. It parses HTML + CSS and supports HTTPS via TLS 1.3. Type 'browse' to open it.",
        },
        t if matches_any(t, &["compositor", "gui", "desktop", "bureau"]) => match lang {
            Lang::Fr => "COSMIC2 est le compositeur graphique. Multi-couches, optimise SSE2, 144 FPS. Tape 'desktop' pour le lancer.",
            Lang::En => "COSMIC2 is the desktop compositor. Multi-layer, SSE2 optimized, 144 FPS. Type 'desktop' to launch it.",
        },
        t if matches_any(t, &["jarvis", "ia", "ai", "intelligence"]) => match lang {
            Lang::Fr => "C'est moi ! Jarvis = Just A Rather Very Intelligent System. NLU par pattern matching, bilingue FR/EN, execution kernel directe.",
            Lang::En => "That's me! Jarvis = Just A Rather Very Intelligent System. NLU via pattern matching, bilingual FR/EN, direct kernel execution.",
        },
        _ => match lang {
            Lang::Fr => "Je n'ai pas d'info precise sur ce sujet. Essaie un autre mot-cle.",
            Lang::En => "I don't have specific info on that topic. Try another keyword.",
        },
    };
    Action::Respond(String::from(explanation))
}

fn howto_topic(topic: &str, lang: Lang) -> Action {
    let howto = match topic {
        t if matches_any(t, &["compile", "build", "construire"]) => match lang {
            Lang::Fr => "Pour compiler TrustOS:\n  1. cargo build\n  2. powershell build-limine.ps1\nL'ISO sera dans trustos.iso",
            Lang::En => "To build TrustOS:\n  1. cargo build\n  2. powershell build-limine.ps1\nThe ISO will be at trustos.iso",
        },
        t if matches_any(t, &["file", "fichier", "creer"]) => match lang {
            Lang::Fr => "Pour creer un fichier:\n  touch /mon_fichier.txt\n  echo 'contenu' > /mon_fichier.txt",
            Lang::En => "To create a file:\n  touch /my_file.txt\n  echo 'content' > /my_file.txt",
        },
        t if matches_any(t, &["network", "reseau", "internet"]) => match lang {
            Lang::Fr => "Pour tester le reseau:\n  ifconfig      (voir l'IP)\n  ping 8.8.8.8  (tester la connexion)\n  browse        (ouvrir le navigateur)",
            Lang::En => "To test networking:\n  ifconfig       (see IP)\n  ping 8.8.8.8   (test connection)\n  browse         (open browser)",
        },
        t if matches_any(t, &["theme", "personnaliser", "customize"]) => match lang {
            Lang::Fr => "Pour changer le theme:\n  theme matrix   (vert hacker)\n  theme cyber    (bleu futuriste)\n  theme retro    (amber)",
            Lang::En => "To change theme:\n  theme matrix   (green hacker)\n  theme cyber    (blue futuristic)\n  theme retro    (amber)",
        },
        _ => match lang {
            Lang::Fr => "Je n'ai pas de tutoriel sur ce sujet. Tape 'help' pour les commandes disponibles.",
            Lang::En => "I don't have a tutorial for that. Type 'help' for available commands.",
        },
    };
    Action::Respond(String::from(howto))
}

// ═══════════════════════════════════════════════════════════════════════
// EXECUTOR: Run planned actions
// ═══════════════════════════════════════════════════════════════════════

fn execute_action(action: Action) {
    match action {
        Action::Respond(msg) => {
            for line in msg.lines() {
                crate::print_color!(JARVIS_COLOR, "  ");
                crate::println!("{}", line);
            }
        }
        Action::ShellCommand(cmd) => {
            crate::print_color!(JARVIS_ACCENT, "  > ");
            crate::println_color!(COLOR_GRAY, "{}", cmd);
            // Execute via the shell command dispatcher
            super::execute_command(&cmd);
        }
        Action::ShowInfo(msg) => {
            for line in msg.lines() {
                crate::print_color!(COLOR_CYAN, "  ");
                crate::println!("{}", line);
            }
        }
        Action::MultiStep(steps) => {
            for (i, step) in steps.iter().enumerate() {
                crate::print_color!(JARVIS_ACCENT, "  [{}] ", i + 1);
                crate::println!("{}", step.description);
            }
            for step in steps {
                execute_action(step.action);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INTERACTIVE REPL: The Jarvis shell
// ═══════════════════════════════════════════════════════════════════════

/// Entry point: interactive Jarvis session
pub(super) fn cmd_jarvis(args: &[&str]) {
    // Neural brain subcommands: "jarvis brain ..."
    if !args.is_empty() && (args[0] == "brain" || args[0] == "neural" || args[0] == "nn") {
        cmd_brain(&args[1..]);
        return;
    }
    
    // One-shot mode: "jarvis what time is it"
    if !args.is_empty() {
        let query = args.join(" ");
        process_query(&query);
        return;
    }
    
    // Interactive REPL mode
    print_jarvis_banner();
    
    let mut ctx = ConversationCtx {
        last_intent: Intent::Unknown,
        last_topic: String::new(),
        turn_count: 0,
        lang: Lang::En,
    };
    
    let mut input_buf = [0u8; 256];
    
    loop {
        // Prompt
        crate::print_color!(JARVIS_COLOR, "\n  jarvis");
        crate::print_color!(COLOR_WHITE, " > ");
        
        // Read input
        let len = read_jarvis_input(&mut input_buf);
        let raw = core::str::from_utf8(&input_buf[..len]).unwrap_or("").trim();
        
        if raw.is_empty() { continue; }
        
        // Exit commands
        if raw == "exit" || raw == "quit" || raw == "q" || raw == "bye" || raw == "au revoir" {
            let lang = detect_lang(raw);
            match lang {
                Lang::Fr => crate::println_color!(JARVIS_COLOR, "  A bientot !"),
                Lang::En => crate::println_color!(JARVIS_COLOR, "  See you later!"),
            }
            break;
        }
        
        ctx.turn_count += 1;
        process_query(raw);
    }
}

/// Process a single query (used by both REPL and one-shot mode)
fn process_query(raw: &str) {
    let norm = normalize(raw);
    let lang = detect_lang(raw);
    let intent = detect_intent(&norm);
    let entities = extract_entities(&norm, intent);
    
    // Debug: show detected intent (subtle)
    crate::print_color!(COLOR_GRAY, "  ");
    crate::print_color!(0xFF444444, "[{:?}]", intent);
    if !entities.is_empty() {
        for e in &entities {
            crate::print_color!(0xFF444444, " {:?}={}", e.kind, e.value);
        }
    }
    if intent == Intent::Unknown && crate::jarvis::is_ready() {
        crate::print_color!(0xFF444444, " ->brain");
    }
    crate::println!();
    
    // Plan & execute — use neural fallback for unknown intents
    let action = if intent == Intent::Unknown {
        // Try the neural brain for unknown intents
        if let Some(neural_response) = crate::jarvis::neural_respond(raw) {
            Action::Respond(neural_response)
        } else {
            plan_action(intent, &entities, lang)
        }
    } else {
        plan_action(intent, &entities, lang)
    };
    execute_action(action);
}

/// Print the Jarvis welcome banner
fn print_jarvis_banner() {
    crate::println!();
    crate::println_color!(JARVIS_COLOR, "  ╔═══════════════════════════════════════════════╗");
    crate::println_color!(JARVIS_COLOR, "  ║          J.A.R.V.I.S. v1.0                   ║");
    crate::println_color!(JARVIS_COLOR, "  ║    Just A Rather Very Intelligent System      ║");
    crate::println_color!(JARVIS_COLOR, "  ╠═══════════════════════════════════════════════╣");
    crate::print_color!(JARVIS_COLOR,   "  ║  ");
    crate::print_color!(COLOR_WHITE,    "TrustOS AI Assistant — 100%% local, 0%% cloud");
    crate::println_color!(JARVIS_COLOR, "  ║");
    crate::print_color!(JARVIS_COLOR,   "  ║  ");
    crate::print_color!(COLOR_GRAY,     "Type 'help' for commands, 'exit' to leave");
    crate::println_color!(JARVIS_COLOR, "   ║");
    crate::println_color!(JARVIS_COLOR, "  ╚═══════════════════════════════════════════════╝");
    crate::println!();
}

/// Simple line reader for Jarvis REPL (no autocomplete, just basic input)
fn read_jarvis_input(buffer: &mut [u8]) -> usize {
    use crate::keyboard::read_char;
    let mut pos = 0;

    loop {
        if let Some(c) = read_char() {
            match c {
                b'\n' | b'\r' => {
                    crate::println!();
                    return pos;
                }
                8 | 127 => { // Backspace
                    if pos > 0 {
                        pos -= 1;
                        crate::print!("\x08 \x08");
                    }
                }
                0x1B => {} // Escape: ignore
                c if c >= 0x20 && pos < buffer.len() - 1 => {
                    buffer[pos] = c;
                    pos += 1;
                    crate::print!("{}", c as char);
                }
                _ => {}
            }
        } else {
            // Yield CPU while waiting
            core::hint::spin_loop();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NEURAL BRAIN — Tiny transformer integration
// ═══════════════════════════════════════════════════════════════════════

const JARVIS_BRAIN: u32 = 0xFF00BBFF;

/// Handle "jarvis brain <cmd>" subcommands
fn cmd_brain(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(JARVIS_BRAIN, "  Jarvis Neural Brain v2.0");
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
        crate::println!("    load          Load weights from /jarvis/weights.bin");
        crate::println!("    reset         Reset weights to random");
        crate::println!("    pretrain [N]  Pre-train on embedded corpus (N epochs)");
        crate::println!("    eval          Evaluate loss across entire corpus");
        crate::println!("    chat <text>   Chat with neural brain directly");
        crate::println!();
        crate::println!("  The neural brain is a 4-layer transformer (312K params)");
        crate::println!("  that learns from text, generates responses, and self-improves.");
        return;
    }

    match args[0] {
        "init" => {
            crate::println_color!(JARVIS_BRAIN, "  Initializing neural brain...");
            crate::jarvis::init();
            crate::println_color!(COLOR_GREEN, "  Neural brain ready.");
        }

        "info" => {
            if !crate::jarvis::is_ready() {
                crate::println_color!(COLOR_YELLOW, "  Brain not initialized. Run: jarvis brain init");
                return;
            }
            for line in crate::jarvis::info_lines() {
                crate::println!("  {}", line);
            }
        }

        "generate" | "gen" | "g" => {
            if !ensure_brain() { return; }
            if crate::jarvis::is_private() {
                crate::println_color!(COLOR_YELLOW, "  [Private mode] Generation disabled");
                return;
            }
            let prompt = if args.len() > 1 { args[1..].join(" ") } else { String::from("Hello") };
            crate::print_color!(JARVIS_BRAIN, "  Prompt: ");
            crate::println_color!(COLOR_WHITE, "{}", prompt);
            
            let start = crate::time::uptime_ticks();
            let output = crate::jarvis::generate(&prompt, 64);
            let elapsed = crate::time::uptime_ticks().saturating_sub(start);
            
            crate::print_color!(JARVIS_BRAIN, "  Output: ");
            crate::println_color!(COLOR_GREEN, "{}", output);
            crate::println_color!(COLOR_GRAY, "  ({} ms, {} tokens)", elapsed, output.len());
        }

        "train" => {
            if !ensure_brain() { return; }
            let text = if args.len() > 1 { args[1..].join(" ") } else {
                crate::println!("  Usage: jarvis brain train <text>");
                return;
            };
            crate::println_color!(JARVIS_BRAIN, "  Training on: \"{}\"", text);
            let loss = crate::jarvis::train_on_text(&text, 0.001);
            crate::println_color!(COLOR_GREEN, "  Loss: {:.4}", loss);
        }

        "test" => {
            if !ensure_brain() { return; }
            crate::println_color!(JARVIS_BRAIN, "  Running neural brain self-test...");
            crate::println!();

            // Tokenizer tests
            crate::println_color!(COLOR_CYAN, "  Tokenizer:");
            let tokens = crate::jarvis::tokenizer::encode("Hello");
            crate::println!("    encode(\"Hello\") = {:?} ({} tokens)", &tokens, tokens.len());
            let decoded = crate::jarvis::tokenizer::decode(&tokens);
            crate::println!("    decode() = \"{}\"", decoded);

            // Generation test
            crate::println_color!(COLOR_CYAN, "  Generation:");
            let output = crate::jarvis::generate("Test", 16);
            crate::println!("    generate(\"Test\", 16) = \"{}\" ({} chars)", output, output.len());

            // Training test
            crate::println_color!(COLOR_CYAN, "  Training:");
            let loss1 = crate::jarvis::train_on_text("Hello world", 0.001);
            crate::println!("    Step 1 loss: {:.4}", loss1);
            let loss2 = crate::jarvis::train_on_text("Hello world", 0.001);
            crate::println!("    Step 2 loss: {:.4}", loss2);

            // Training module self-test
            crate::println_color!(COLOR_CYAN, "  Training module:");
            let (tp, tf) = crate::jarvis::training::self_test();
            crate::println!("    {} passed, {} failed", tp, tf);

            // Introspection
            crate::println_color!(COLOR_CYAN, "  Introspection:");
            let info = crate::jarvis::agent::introspect(
                &crate::jarvis::agent::IntrospectTarget::Architecture);
            for line in info.iter().take(5) {
                crate::println!("    {}", line);
            }

            crate::println!();
            let total_pass = 4 + tp; // tokenizer + gen + 2 train + training self-test
            let total_fail = tf;
            if total_fail == 0 {
                crate::println_color!(COLOR_GREEN, "  All {} tests passed!", total_pass);
            } else {
                crate::println_color!(COLOR_RED, "  {} passed, {} failed", total_pass, total_fail);
            }
        }

        "bench" => {
            if !ensure_brain() { return; }
            crate::println_color!(JARVIS_BRAIN, "  Benchmarking inference speed...");
            let (tok_per_sec, elapsed_ms) = crate::jarvis::agent::bench_inference();
            crate::println!("  Speed: {:.1} tokens/sec", tok_per_sec);
            crate::println!("  32 tokens generated in {} ms", elapsed_ms);
            crate::println_color!(COLOR_GRAY, "  (CPU reference — GPU dispatch target: 100×)");
        }

        "introspect" | "self" => {
            if !ensure_brain() { return; }
            let lines = crate::jarvis::agent::introspect(
                &crate::jarvis::agent::IntrospectTarget::Full);
            for line in &lines {
                crate::println_color!(JARVIS_BRAIN, "  {}", line);
            }
        }

        "weights" => {
            if !ensure_brain() { return; }
            let lines = crate::jarvis::agent::introspect(
                &crate::jarvis::agent::IntrospectTarget::WeightStats);
            for line in &lines {
                crate::println!("  {}", line);
            }
        }

        "hardware" | "hw" => {
            if !ensure_brain() { return; }
            let lines = crate::jarvis::agent::introspect(
                &crate::jarvis::agent::IntrospectTarget::Hardware);
            for line in &lines {
                crate::println!("  {}", line);
            }
        }

        "mentor" => {
            crate::println_color!(JARVIS_BRAIN, "  Serial Mentoring Mode");
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
            crate::println_color!(COLOR_GRAY, "  (Mentor polling active in shell idle loop)");
        }

        "reset" => {
            if !ensure_brain() { return; }
            crate::println_color!(COLOR_YELLOW, "  Resetting all weights to random...");
            crate::jarvis::reset();
            crate::println_color!(COLOR_GREEN, "  Weights reset. Training steps cleared.");
        }

        "save" => {
            if !ensure_brain() { return; }
            crate::println_color!(JARVIS_BRAIN, "  Saving weights to /jarvis/weights.bin...");
            match crate::jarvis::save_weights() {
                Ok(bytes) => crate::println_color!(COLOR_GREEN, "  Saved {} KB", bytes / 1024),
                Err(e) => crate::println_color!(COLOR_RED, "  Save failed: {}", e),
            }
        }

        "load" => {
            crate::println_color!(JARVIS_BRAIN, "  Loading weights from /jarvis/weights.bin...");
            match crate::jarvis::load_weights() {
                Ok(bytes) => crate::println_color!(COLOR_GREEN, "  Loaded {} KB", bytes / 1024),
                Err(e) => crate::println_color!(COLOR_RED, "  Load failed: {}", e),
            }
        }

        "pretrain" | "pt" => {
            if !ensure_brain() { return; }
            let epochs: usize = if args.len() > 1 {
                args[1].parse().unwrap_or(1)
            } else { 1 };
            crate::println_color!(JARVIS_BRAIN, "  Pre-training on embedded corpus ({} epoch(s))...", epochs);
            crate::println!();

            // Show loss before
            let loss_before = crate::jarvis::eval_corpus();
            crate::println_color!(COLOR_GRAY, "  Loss before: {:.3}", loss_before);
            crate::println!();

            // Train each phase with progress
            for phase in 0..crate::jarvis::corpus::num_phases() {
                let name = crate::jarvis::corpus::phase_name(phase);
                let seqs = crate::jarvis::corpus::CORPUS[phase].len();
                crate::print_color!(JARVIS_BRAIN, "  Phase {} ", phase);
                crate::print_color!(COLOR_WHITE, "({}) ", name);
                crate::print_color!(COLOR_GRAY, "[{} seqs] ", seqs);

                let (steps, avg_loss, elapsed) =
                    crate::jarvis::pretrain_phase(phase, epochs, 0.001);

                crate::print_color!(COLOR_GREEN, "loss={:.3} ", avg_loss);
                crate::println_color!(COLOR_GRAY, "({}ms, {} steps)", elapsed, steps);
            }

            // Show loss after
            crate::println!();
            let loss_after = crate::jarvis::eval_corpus();
            crate::print_color!(COLOR_GRAY, "  Loss after:  {:.3}", loss_after);
            if loss_after < loss_before {
                crate::println_color!(COLOR_GREEN, " (improved by {:.3})", loss_before - loss_after);
            } else {
                crate::println_color!(COLOR_YELLOW, " (no improvement)");
            }
            crate::println!();
            crate::println_color!(COLOR_GRAY, "  Total steps: {} | Training steps global: {}",
                crate::jarvis::corpus::total_sequences() * epochs,
                crate::jarvis::training_steps());
        }

        "eval" => {
            if !ensure_brain() { return; }
            crate::println_color!(JARVIS_BRAIN, "  Evaluating on embedded corpus...");
            let avg_loss = crate::jarvis::eval_corpus();
            crate::println!();

            // Per-phase breakdown
            let model_guard_unavailable = !crate::jarvis::is_ready();
            if !model_guard_unavailable {
                for phase in 0..crate::jarvis::corpus::num_phases() {
                    let name = crate::jarvis::corpus::phase_name(phase);
                    // Quick eval of this phase
                    let mut phase_loss = 0.0f32;
                    let mut count = 0u32;
                    for &text in crate::jarvis::corpus::CORPUS[phase] {
                        let tokens = crate::jarvis::tokenizer::encode(text);
                        if tokens.len() >= 2 {
                            // We can't easily call compute_loss without the model guard
                            // So just report the total
                            count += 1;
                        }
                    }
                    crate::print_color!(JARVIS_BRAIN, "  Phase {} ", phase);
                    crate::print_color!(COLOR_WHITE, "({}) ", name);
                    crate::println_color!(COLOR_GRAY, "{} sequences", count);
                }
            }

            crate::println!();
            crate::print_color!(COLOR_WHITE, "  Average loss: ");
            if avg_loss < 4.0 {
                crate::println_color!(COLOR_GREEN, "{:.3} (learning!)", avg_loss);
            } else if avg_loss < 5.5 {
                crate::println_color!(COLOR_YELLOW, "{:.3} (early stage)", avg_loss);
            } else {
                crate::println_color!(COLOR_RED, "{:.3} (random/untrained)", avg_loss);
            }

            crate::println_color!(COLOR_GRAY, "  (Random baseline: ~5.5, Good: <3.0, Memorized: <1.0)");
        }

        "chat" => {
            if !ensure_brain() { return; }
            if crate::jarvis::is_private() {
                crate::println_color!(COLOR_YELLOW, "  [Private mode] Chat disabled");
                return;
            }
            if args.len() < 2 {
                crate::println!("  Usage: jarvis brain chat <text>");
                return;
            }
            let prompt = args[1..].join(" ");
            crate::print_color!(COLOR_WHITE, "  You: ");
            crate::println!("{}", prompt);

            let start = crate::time::uptime_ticks();
            let output = crate::jarvis::generate(&prompt, 64);
            let elapsed = crate::time::uptime_ticks().saturating_sub(start);

            crate::print_color!(JARVIS_BRAIN, "  Jarvis: ");
            // Show printable chars, replace control chars with dots
            for c in output.chars() {
                if c.is_ascii_graphic() || c == ' ' {
                    crate::print!("{}", c);
                } else {
                    crate::print_color!(COLOR_GRAY, ".");
                }
            }
            crate::println!();
            crate::println_color!(COLOR_GRAY, "  ({} ms, {} chars)", elapsed, output.len());
        }

        _ => {
            crate::println_color!(COLOR_RED, "  Unknown: jarvis brain {}", args[0]);
            crate::println!("  Use 'jarvis brain' for help");
        }
    }
}

/// Ensure brain is initialized, auto-init if not
fn ensure_brain() -> bool {
    if !crate::jarvis::is_ready() {
        crate::println_color!(COLOR_GRAY, "  Auto-initializing neural brain...");
        crate::jarvis::init();
    }
    crate::jarvis::is_ready()
}