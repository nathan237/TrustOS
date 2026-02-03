# Chapitre 2 – Exploration technique

## 🎬 Narration

> *"Avant de construire, il faut comprendre. Rust, microkernel, capabilities... chaque choix technique est une fondation sur laquelle tout le reste repose."*

---

## 📖 Script narratif

### Scène 2.1 – Pourquoi Rust ?
**Durée estimée**: 3 minutes

**Voix off**:
"Les OS traditionnels sont écrits en C. Un langage puissant, mais dangereux. Buffer overflows, use-after-free, race conditions... ces bugs sont la cause de 70% des vulnérabilités de sécurité. Rust élimine ces classes entières de bugs à la compilation."

**Visuels suggérés**:
- Comparaison code C vs Rust
- Statistiques CVE liées à la mémoire
- Compilation Rust avec erreurs de borrow checker

**Code démonstration**:
```rust
// Rust empêche les data races à la compilation
fn safe_concurrent_access() {
    let data = Arc::new(Mutex::new(0));
    // Le compilateur garantit la sécurité mémoire
}
```

### Scène 2.2 – Le choix du microkernel
**Durée estimée**: 3-4 minutes

**Voix off**:
"Linux a un kernel de 30 millions de lignes. Windows NT, environ 50 millions. TRustOs ? Moins de 10 000. Comment ? En déplaçant tout ce qui n'est pas absolument essentiel en userland."

**Visuels suggérés**:
- Animation taille du kernel (barres comparatives)
- Diagramme : ce qui reste dans le kernel vs userland

**Tableau comparatif**:
| Composant | Linux | Windows | TRustOs |
|-----------|-------|---------|----------|
| Scheduler | Kernel | Kernel | Kernel |
| Filesystem | Kernel | Kernel | **Userland** |
| Drivers | Kernel | Kernel | **Userland** |
| Network | Kernel | Kernel | **Userland** |
| GUI | Kernel (DRM) | Kernel | **Userland** |

### Scène 2.3 – Inspirations open-source
**Durée estimée**: 2 minutes

**Voix off**:
"TRustOs n'est pas créé dans le vide. Il s'inspire de projets Rust OS existants : Redox pour la philosophie microkernel, blog_os pour les fondations, seL4 pour les capabilities."

**Références techniques**:
- [Redox OS](https://redox-os.org/) – Microkernel Rust complet
- [blog_os](https://os.phil-opp.com/) – Tutoriel OS Rust
- [seL4](https://sel4.systems/) – Microkernel formellement vérifié

### Scène 2.4 – Premier boot
**Durée estimée**: 2-3 minutes

**Voix off**:
"29 janvier 2026. Le kernel TRustOs boot pour la première fois dans QEMU. Les premiers logs s'affichent sur le port série..."

**Capture à réaliser**:
```
qemu-system-x86_64 -drive format=raw,file=bootimage-trustos_kernel.bin -serial stdio
```

**Output attendu**:
```
[         0][CPU0][INFO ] TRustOs Kernel v0.1.0
[         0][CPU0][INFO ] =========================
[         1][CPU0][INFO ] Initializing memory management...
[         2][CPU0][INFO ] Memory management initialized.
[         3][CPU0][INFO ] Initializing interrupts...
...
[        10][CPU0][INFO ] TRustOs kernel initialized successfully!
[        11][CPU0][INFO ] Entering idle loop...
```

---

## 🎨 Storyboard visuel

```
┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 2.1 - POURQUOI RUST                                  │
├─────────────────────────────────────────────────────────────┤
│  [Split screen: C code avec bug vs Rust compile error]     │
│                                                             │
│  // C - compile, crash at runtime                          │
│  char* ptr = malloc(10);                                   │
│  free(ptr);                                                │
│  ptr[0] = 'x'; // Use after free!                         │
│                                                             │
│  // Rust - caught at compile time                          │
│  error[E0382]: borrow of moved value                       │
│                                                             │
│  Texte: "Rust attrape les bugs avant qu'ils n'arrivent"   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 2.4 - PREMIER BOOT                                   │
├─────────────────────────────────────────────────────────────┤
│  [Capture: Terminal QEMU avec logs kernel]                 │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ $ qemu-system-x86_64 ... -serial stdio              │   │
│  │                                                      │   │
│  │ [INFO ] TRustOs Kernel v0.1.0                   │   │
│  │ [INFO ] =========================                    │   │
│  │ [INFO ] Memory management initialized.              │   │
│  │ [INFO ] Interrupts initialized.                     │   │
│  │ [INFO ] Scheduler initialized.                      │   │
│  │ [INFO ] IPC subsystem initialized.                  │   │
│  │ [INFO ] Security system initialized.                │   │
│  │ [INFO ] Event tracing initialized.                  │   │
│  │ [INFO ] TRustOs kernel initialized successfully!   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Texte: "Le premier battement de cœur de TRustOs"         │
└─────────────────────────────────────────────────────────────┘
```

---

## 📊 Liens avec le développement

| Moment documentaire | Commit/Action | Date |
|--------------------|---------------|------|
| Premier boot QEMU | Phase 0 complétée | 2026-01-29 |
| Logs kernel | serial.rs + logger.rs | 2026-01-29 |
| Memory init | memory/mod.rs | 2026-01-29 |

---

## ✅ Statut chapitre

- [x] Script narratif
- [x] Références techniques
- [x] Storyboard conceptuel
- [ ] Capture premier boot QEMU
- [ ] Enregistrement voix off
- [ ] Montage vidéo
