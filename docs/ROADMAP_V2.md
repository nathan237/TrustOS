# TrustOS — Roadmap V2 : Optimisation, Persistance & Compilateur

> **État actuel:** 98,112 lignes | 202 fichiers | ~1,891 lignes mortes identifiées
> **Objectif:** OS Rust auto-suffisant pour développeurs — le plus compact et performant possible

---

## Phase 0 — Nettoyage Immédiat (Net: -3,500 lignes)

### 0.1 Supprimer le code mort
| Fichier | Lignes | Raison |
|---------|--------|--------|
| `rasterizer_temp.rs` | 787 | Copie exacte de `rasterizer.rs`, non déclaré dans `main.rs` |
| `holovolume_backup.rs` | 641 | Backup obsolète de `holovolume.rs`, non déclaré |
| `tls.rs` | 192 | Dépend de `mbedtls` (désactivé), remplacé par `tls13/` |
| `c_runtime.rs` | 187 | C ABI runtime, non déclaré |
| `mbedtls_alloc.rs` | 84 | Allocateur mbedtls, non déclaré |
| **Total** | **1,891** | |

### 0.2 Consolider les doublons math
- `formula3d.rs` réinvente `V3`, `V2`, `sin/cos` Taylor → utiliser `graphics::math3d` + `libm`
- Supprimer `micromath` OU `libm` (garder un seul)
- **Gain estimé:** ~150 lignes

### 0.3 Nettoyer les stubs shell
- 46 commandes shell qui affichent juste "not implemented"
- Supprimer ou compacter dans un fichier `shell_stubs.rs` de ~80 lignes max
- **Gain estimé:** ~200 lignes

### 0.4 Unifier les types réseau
- `network.rs` duplique `MacAddress`, `Ipv4Address` → déjà dans `netstack/`
- Réduire `network.rs` à un trait driver + ré-exporter les types
- **Gain estimé:** ~300 lignes

### 0.5 Choisir un seul framework UI
| Framework | Lignes | Verdict |
|-----------|--------|---------|
| `cosmic/` | 1,824 | **GARDER** — utilisé par COSMIC2, le plus complet |
| `gui/` | 1,274 | ARCHIVER — Windows 11 style, plus utilisé |
| `ui/` | 1,873 | ARCHIVER — widgets Qt-style, virtio-gpu seulement |
- **Gain estimé:** ~3,100 lignes si on archive `gui/` + `ui/`

**Impact total Phase 0: ~5,600 lignes économisées → ~92,500 lignes**

---

## Phase 1 — Persistance Fiable (Priorité #1)

### État actuel du stack disque
```
┌─────────────────────────────┐
│  VFS (vfs/mod.rs)          │ ← Mountpoints, auto-mount
├─────────────────────────────┤
│  TrustFS (vfs/trustfs.rs)  │ ← Superblock, inodes, 12 direct blocks
│  FAT32 (vfs/fat32.rs)      │ ← Read + Write (partiellement)
├─────────────────────────────┤
│  VirtIO-blk / AHCI         │ ← Read/Write secteurs — FONCTIONNEL
├─────────────────────────────┤
│  Partition (MBR/GPT)       │ ← Parser — FONCTIONNEL
└─────────────────────────────┘
```

### 1.1 Corriger TrustFS — Blocs indirects
- **Problème:** 12 direct × 512B = **6KB max par fichier**
- **Solution:** Ajouter 1 indirect block (512/4 = 128 pointeurs → ~70KB max)
- **Estimé:** +80 lignes dans `trustfs.rs`

### 1.2 Block cache / Buffer layer
- Cache LRU de secteurs en mémoire (ex: 256 entrées × 512B = 128KB)
- Évite les accès disque répétés pour les métadonnées
- **Estimé:** +150 lignes (nouveau fichier `vfs/block_cache.rs`)

### 1.3 Write-Ahead Log (WAL) minimal
- Journal de transactions avant écriture (max 64 entrées)
- Si crash pendant write → replay journal au mount
- **Estimé:** +200 lignes dans `trustfs.rs`

### 1.4 Supprimer `persistence.rs` raw sectors
- Actuellement écrit en sectors 2048+ **hors VFS** → risque de corruption
- Migrer tout vers TrustFS via VFS
- **Gain:** -200 lignes

### 1.5 Intégrer avec TrustCode (éditeur)
- Ctrl+S dans l'éditeur → `ramfs.write()` → `trustfs.write_file()`
- Fichiers persistent entre reboots
- **Estimé:** +30 lignes dans `text_editor.rs`

### Résultat Phase 1
```
Fichiers jusqu'à ~70KB ✓
Crash-safe writes ✓
Cache disque performant ✓
Éditeur sauvegarde sur disque ✓
```
**Impact ligne: net +260 lignes**

---

## Phase 2 — Compilateur/Interpréteur Intégré (Le Saint Graal)

### Stratégie: TrustLang — Sous-ensemble Rust compilé en bytecode

Compiler le vrai Rust (borrow checker, generics, traits, lifetimes) nécessiterait
~50,000+ lignes. **Pas réaliste.** 

**Alternative:** Un langage **TrustLang** = sous-ensemble de Rust → bytecode VM

```rust
// TrustLang — syntaxe Rust simplifiée
fn fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    let result = fibonacci(10);
    print(result);   // 55
}
```

### 2.1 Lexer + Tokenizer (~300 lignes)
- Réutiliser le tokenizer de `text_editor.rs` (déjà fait pour Rust syntax highlighting!)
- Étendre: produire des `Token` structurés au lieu de `ColorSpan`
- Types: `fn`, `let`, `if/else`, `while`, `for`, `return`, `struct`, opérateurs, littéraux

### 2.2 Parser → AST (~500 lignes)
- Recursive descent parser
- AST nodes: `FnDecl`, `LetStmt`, `IfExpr`, `WhileLoop`, `BinOp`, `Call`, `Return`
- Support: fonctions, variables locales, types basiques (`i64`, `f64`, `bool`, `&str`)
- Pas de borrow checker, pas de lifetimes, pas de generics (V1)

### 2.3 Bytecode VM (~400 lignes)
- Stack-based VM (comme Lua/Python)
- Opcodes: `PUSH`, `POP`, `ADD`, `SUB`, `MUL`, `DIV`, `CMP`, `JMP`, `CALL`, `RET`, `PRINT`, `LOAD`, `STORE`
- Registres: Stack + 256 locals par frame
- Builtins: `print()`, `read_line()`, `file_read()`, `file_write()`, `sleep()`
- **Estimé:** ~400 lignes

### 2.4 Compilateur AST → Bytecode (~300 lignes)
- Visitor pattern sur l'AST
- Résolution de variables, stack frame layout
- Appels de fonctions avec convention d'appel interne

### 2.5 REPL intégré au shell (~100 lignes)
- `trustlang` commande → REPL interactif
- `trustlang run fichier.tl` → compile + exécute
- `trustlang compile fichier.tl` → génère bytecode

### 2.6 Intégration TrustCode
- "Run" button (F5) dans l'éditeur → compile et exécute le fichier ouvert
- Output dans un panneau terminal intégré
- Erreurs avec numéros de ligne cliquables

### Budget lignes Phase 2
| Composant | Lignes estimées |
|-----------|----------------|
| Lexer/Tokenizer | 300 |
| Parser/AST | 500 |
| Bytecode VM | 400 |
| Compiler | 300 |
| REPL + Shell | 100 |
| **Total** | **~1,600 lignes** |

### Pourquoi c'est killer
- **Aucun OS bare-metal** n'a un compilateur/interpréteur pour un langage Rust-like
- L'utilisateur peut écrire du code, compiler, et exécuter **sans quitter l'OS**
- Le tokenizer existe déjà (syntax highlighting) → on l'étend
- Le transpiler/runtime existe déjà → on réutilise le runtime de syscalls

---

## Phase 3 — Optimisations Architecturales

### 3.1 Refactorer `shell.rs` (13,052 lignes → ~8,000 lignes)
```
shell.rs                    →  shell/
                                ├── mod.rs          (core + dispatch)
                                ├── cosmic.rs       (COSMIC2 desktop, 4,700 lignes)
                                ├── commands/
                                │   ├── fs.rs       (ls, cd, cat, cp, mv)
                                │   ├── system.rs   (clear, time, whoami)
                                │   ├── network.rs  (ping, curl, ifconfig)
                                │   ├── vm.rs       (hypervisor, linux)
                                │   └── misc.rs     (browser, transpiler)
                                └── stubs.rs        (commandes non-implémentées)
```
- **Net:** 0 lignes (réorganisation), mais shell.rs passe de 13K → ~5K (hors COSMIC)

### 3.2 Fusionner les compositeurs
- `compositor.rs` (SSE2) + `graphics/compositor.rs` (TrustGL)
- → Un seul `graphics/compositor.rs` avec backends
- **Gain:** ~400 lignes

### 3.3 Optimisation framebuffer
- `fill_rect()` boucle pixel par pixel dans certains chemins
- Utiliser `rep stosd` / `memset` pour les rectangles pleins
- SIMD `_mm_store_si128` pour les copies de surface
- **Gain perf:** +20-30% rendering

---

## Phase 4 — Features Développeur

### 4.1 Pipes & Redirection (~200 lignes)
```bash
cat file.rs | grep "fn " | wc -l
echo "data" > output.txt
ls >> log.txt
```
- Parser de pipeline dans `shell/dispatch.rs`
- Buffer de sortie inter-commandes

### 4.2 GDB-like Debugger (~500 lignes)
- Breakpoints (INT3), single-step (RFLAGS.TF)
- Inspect registres, mémoire, stack
- Intégré au REPL TrustLang : `debug run file.tl`

### 4.3 Git Client Minimal (~800 lignes)
- `git clone` via HTTPS (TLS 1.3 déjà implémenté!)
- `git status`, `git add`, `git commit` (local)
- Pack format parser, object storage sur TrustFS

### 4.4 Package Manager (~300 lignes)
- `trust install <package>` depuis un registry HTTP
- Download → extract → place dans `/usr/bin/`
- Manifest TOML simple

---

## Tableau Récapitulatif

| Phase | Objectif | Lignes Δ | Estimation temps |
|-------|----------|----------|-----------------|
| **Phase 0** | Nettoyage & dead code | **-5,600** | 1-2 heures |
| **Phase 1** | Persistance fiable | **+260** | 3-4 heures |
| **Phase 2** | TrustLang compiler | **+1,600** | 6-8 heures |
| **Phase 3** | Architecture & perf | **-1,000** | 3-4 heures |
| **Phase 4** | Dev features | **+1,800** | 8-10 heures |
| **Total** | | **~89,200** | ~22-28 heures |

### Score final estimé
```
Avant:  98,112 lignes — avec 1,891 de code mort, 3,100 de frameworks dupliqués
Après:  ~89,200 lignes — avec compilateur, persistance, debugger, pipes
```

**Un OS complet avec compilateur intégré en ~89K lignes** — c'est unique au monde.

Pour comparaison:
- xv6: 10K lignes (mais zéro features)
- Redox: 200K+ lignes (pas de compilateur intégré)
- SerenityOS: 500K+ lignes (pas d'éditeur ni compilateur intégré)
- Linux: 35M+ lignes

---

## Ordre d'exécution recommandé

```
1. Phase 0.1-0.3  →  Supprimer dead code (quick wins)
2. Phase 1.1-1.3  →  TrustFS blocs indirects + WAL
3. Phase 1.5      →  Ctrl+S → disque dans TrustCode
4. Phase 2.1-2.3  →  Lexer + Parser + VM TrustLang
5. Phase 2.4-2.6  →  Compiler + REPL + intégration éditeur
6. Phase 0.4-0.5  →  Nettoyage réseau + UI frameworks
7. Phase 3        →  Refactoring architecture
8. Phase 4        →  Pipes, debugger, git
```

Phases 0.1 et 1 peuvent commencer **immédiatement**.
