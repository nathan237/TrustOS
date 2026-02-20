//! Embedded Training Corpus — Jarvis's first memories
//!
//! This module contains the text sequences Jarvis learns from at boot.
//! The corpus is organized by category, from simple byte patterns to
//! TrustOS-specific knowledge.
//!
//! Training strategy for a 312K-param byte-level model:
//! 1. **Character patterns** — common English/French byte sequences
//! 2. **Word associations** — simple prompt→response pairs
//! 3. **Command knowledge** — TrustOS shell commands and their output
//! 4. **Identity** — who Jarvis is, what it can do
//!
//! Each corpus entry is a short text (< 64 bytes ideally, matching MAX_TRAIN_SEQ).
//! Shorter sequences train faster and the model learns patterns better.

/// All training sequences, organized for curriculum learning.
/// Phase 1 is trained first (simple), then Phase 2 (harder), etc.
pub static CORPUS: &[&[&str]] = &[
    // ╔═══════════════════════════════════════════════════════════╗
    // ║ Phase 1: Byte patterns — teach common character sequences ║
    // ╚═══════════════════════════════════════════════════════════╝
    &[
        // Common English words (repetition helps byte-level models)
        "the the the the the the the the",
        "hello hello hello hello hello",
        "world world world world world",
        "trust trust trust trust trust",
        "system system system system",
        "kernel kernel kernel kernel",
        "memory memory memory memory",
        "process process process process",
        // Common French words
        "bonjour bonjour bonjour bonjour",
        "salut salut salut salut salut",
        "merci merci merci merci merci",
        "oui oui oui oui oui oui oui",
        // Character-level patterns
        "abcdefghijklmnopqrstuvwxyz",
        "0123456789 0123456789 0123456789",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        // Common bigrams
        "th th th th th th th th th th",
        "he he he he he he he he he he",
        "in in in in in in in in in in",
        "er er er er er er er er er er",
        "an an an an an an an an an an",
        "on on on on on on on on on on",
    ],

    // ╔═══════════════════════════════════════════════════════════╗
    // ║ Phase 2: Simple sentences — teach word boundaries + syntax ║
    // ╚═══════════════════════════════════════════════════════════╝
    &[
        "Hello, I am Jarvis.",
        "TrustOS is an operating system.",
        "Jarvis is an AI assistant.",
        "The kernel is written in Rust.",
        "Memory is managed by the heap.",
        "Type help for commands.",
        "Bonjour, je suis Jarvis.",
        "TrustOS est un systeme.",
        "Tapez aide pour les commandes.",
        "The CPU runs in 64-bit mode.",
        "Files are stored in ramfs.",
        "The shell is called tsh.",
        "Jarvis lives inside TrustOS.",
        "I can learn from text.",
        "I am a neural network.",
        "My brain has four layers.",
        "I think with transformers.",
        "I learn by training.",
    ],

    // ╔═══════════════════════════════════════════════════════════╗
    // ║ Phase 3: Command knowledge — TrustOS shell patterns      ║
    // ╚═══════════════════════════════════════════════════════════╝
    &[
        "help: show available commands",
        "ls: list files in directory",
        "cat: display file contents",
        "echo: print text to screen",
        "pwd: print working directory",
        "cd: change directory",
        "mkdir: create a directory",
        "touch: create empty file",
        "rm: remove a file",
        "cp: copy a file",
        "mv: move or rename a file",
        "ps: show running processes",
        "free: show memory usage",
        "uptime: show system uptime",
        "neofetch: system info display",
        "ping: test network connection",
        "ifconfig: network interface info",
        "jarvis: AI assistant",
        "theme: change color theme",
        "chess: play chess game",
        "browse: open web browser",
        "beep: play a sound",
    ],

    // ╔═══════════════════════════════════════════════════════════╗
    // ║ Phase 4: Q&A pairs — teach prompt→response pattern       ║
    // ╚═══════════════════════════════════════════════════════════╝
    &[
        "Q: who are you? A: I am Jarvis.",
        "Q: what is TrustOS? A: An OS in Rust.",
        "Q: how much memory? A: Type free.",
        "Q: list files? A: Type ls.",
        "Q: what time? A: Type date.",
        "Q: show processes? A: Type ps.",
        "Q: network status? A: Type ifconfig.",
        "Q: help me. A: Type help.",
        "Q: hello A: Hello! How can I help?",
        "Q: bonjour A: Salut! Comment aider?",
        "Q: play a game A: Try chess or snake.",
        "Q: system info A: Try neofetch.",
        "Q: qui es-tu? R: Je suis Jarvis.",
        "Q: aide R: Tape help.",
        "Q: ping A: ping 10.0.2.2",
    ],

    // ╔═══════════════════════════════════════════════════════════╗
    // ║ Phase 5: Identity — Jarvis's self-knowledge               ║
    // ╚═══════════════════════════════════════════════════════════╝
    &[
        "I am Jarvis, the TrustOS AI.",
        "I have 312K parameters.",
        "I am a 4-layer transformer.",
        "My vocabulary is 256 bytes.",
        "I run on bare metal.",
        "I have no cloud connection.",
        "I learn from serial input.",
        "I can read my own source code.",
        "I can describe my architecture.",
        "My weights live in memory.",
        "I can save to /jarvis/weights.bin.",
        "I was created inside TrustOS.",
        "I think at the byte level.",
        "I use attention and SwiGLU.",
        "I can train on your text.",
        "I am self-hosted. 100% local.",
    ],
];

/// Total number of training sequences across all phases
pub fn total_sequences() -> usize {
    CORPUS.iter().map(|phase| phase.len()).sum()
}

/// Total number of phases
pub fn num_phases() -> usize {
    CORPUS.len()
}

/// Get phase name
pub fn phase_name(phase: usize) -> &'static str {
    match phase {
        0 => "Byte Patterns",
        1 => "Simple Sentences",
        2 => "Shell Commands",
        3 => "Q&A Pairs",
        4 => "Self-Knowledge",
        _ => "Unknown",
    }
}
