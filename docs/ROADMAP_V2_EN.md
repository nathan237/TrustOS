# TrustOS — Roadmap V2: Optimization, Persistence & Compiler

> **Current state:** 98,112 lines | 202 files | ~1,891 dead lines identified  
> **Goal:** Self-sufficient Rust OS for developers — as compact and performant as possible

---

## Phase 0 — Immediate Cleanup (Net: -3,500 lines)

### 0.1 Remove Dead Code
| File | Lines | Reason |
|------|-------|--------|
| `rasterizer_temp.rs` | 787 | Exact copy of `rasterizer.rs`, not declared in `main.rs` |
| `holovolume_backup.rs` | 641 | Obsolete backup of `holovolume.rs`, not declared |
| `tls.rs` | 192 | Depends on `mbedtls` (disabled), replaced by `tls13/` |
| `c_runtime.rs` | 187 | C ABI runtime, not declared |
| `mbedtls_alloc.rs` | 84 | mbedtls allocator, not declared |
| **Total** | **1,891** | |

### 0.2 Consolidate Math Duplicates
- `formula3d.rs` reinvents `V3`, `V2`, `sin/cos` Taylor → use `graphics::math3d` + `libm`
- Remove `micromath` OR `libm` (keep only one)
- **Estimated savings:** ~150 lines

### 0.3 Clean Up Shell Stubs
- 46 shell commands only print "not implemented"
- Remove or compact into a single `shell_stubs.rs` of ~80 lines max
- **Estimated savings:** ~200 lines

### 0.4 Unify Network Types
- `network.rs` duplicates `MacAddress`, `Ipv4Address` → already in `netstack/`
- Reduce `network.rs` to a driver trait + re-export types
- **Estimated savings:** ~300 lines

### 0.5 Choose One UI Framework
| Framework | Lines | Verdict |
|-----------|-------|---------|
| `cosmic/` | 1,824 | **KEEP** — used by COSMIC2, most complete |
| `gui/` | 1,274 | ARCHIVE — Windows 11 style, no longer used |
| `ui/` | 1,873 | ARCHIVE — Qt-style widgets, virtio-gpu only |
- **Estimated savings:** ~3,100 lines if `gui/` + `ui/` are archived

**Phase 0 total impact: ~5,600 lines saved → ~92,500 lines**

---

## Phase 1 — Reliable Persistence (Priority #1)

### Current Disk Stack State
```
┌─────────────────────────────┐
│  VFS (vfs/mod.rs)          │ ← Mountpoints, auto-mount
├─────────────────────────────┤
│  TrustFS (vfs/trustfs.rs)  │ ← Superblock, inodes, 12 direct blocks
│  FAT32 (vfs/fat32.rs)      │ ← Read + Write (partial)
├─────────────────────────────┤
│  VirtIO-blk / AHCI         │ ← Sector Read/Write — WORKING
├─────────────────────────────┤
│  Partition (MBR/GPT)       │ ← Parser — WORKING
└─────────────────────────────┘
```

### 1.1 Fix TrustFS — Indirect Blocks
- **Problem:** 12 direct × 512B = **6KB max per file**
- **Solution:** Add 1 indirect block (512/4 = 128 pointers → ~70KB max)
- **Estimated:** +80 lines in `trustfs.rs`

### 1.2 Block Cache / Buffer Layer
- LRU sector cache in memory (e.g., 256 entries × 512B = 128KB)
- Avoids repeated disk accesses for metadata
- **Estimated:** +150 lines (new file `vfs/block_cache.rs`)

### 1.3 Minimal Write-Ahead Log (WAL)
- Transaction journal before write (max 64 entries)
- If crash during write → replay journal on mount
- **Estimated:** +200 lines in `trustfs.rs`

### 1.4 Remove `persistence.rs` Raw Sectors
- Currently writes to sectors 2048+ **outside VFS** → corruption risk
- Migrate everything to TrustFS via VFS
- **Savings:** -200 lines

### 1.5 Integrate with TrustCode (Editor)
- Ctrl+S in editor → `ramfs.write()` → `trustfs.write_file()`
- Files persist between reboots
- **Estimated:** +30 lines in `text_editor.rs`

### Phase 1 Result
```
Files up to ~70KB ✓
Crash-safe writes ✓
Performant disk cache ✓
Editor saves to disk ✓
```
**Line impact: net +260 lines**

---

## Phase 2 — Integrated Compiler/Interpreter (The Holy Grail)

### Strategy: TrustLang — Rust Subset Compiled to Bytecode

Compiling real Rust (borrow checker, generics, traits, lifetimes) would require
~50,000+ lines. **Not realistic.**

**Alternative:** A **TrustLang** language = Rust subset → bytecode VM

```rust
// TrustLang — simplified Rust syntax
fn fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    let result = fibonacci(10);
    print(result);   // 55
}
```

### 2.1 Lexer + Tokenizer (~300 lines)
- Reuse the tokenizer from `text_editor.rs` (already done for Rust syntax highlighting!)
- Extend: produce structured `Token`s instead of `ColorSpan`
- Types: `fn`, `let`, `if/else`, `while`, `for`, `return`, `struct`, operators, literals

### 2.2 Parser → AST (~500 lines)
- Recursive descent parser
- AST nodes: `FnDecl`, `LetStmt`, `IfExpr`, `WhileLoop`, `BinOp`, `Call`, `Return`
- Support: functions, local variables, basic types (`i64`, `f64`, `bool`, `&str`)
- No borrow checker, no lifetimes, no generics (V1)

### 2.3 Bytecode VM (~400 lines)
- Stack-based VM (like Lua/Python)
- Opcodes: `PUSH`, `POP`, `ADD`, `SUB`, `MUL`, `DIV`, `CMP`, `JMP`, `CALL`, `RET`, `PRINT`, `LOAD`, `STORE`
- Registers: Stack + 256 locals per frame
- Builtins: `print()`, `read_line()`, `file_read()`, `file_write()`, `sleep()`
- **Estimated:** ~400 lines

### 2.4 Compiler AST → Bytecode (~300 lines)
- Visitor pattern on AST
- Variable resolution, stack frame layout
- Function calls with internal calling convention

### 2.5 Shell-integrated REPL (~100 lines)
- `trustlang` command → interactive REPL
- `trustlang run file.tl` → compile + execute
- `trustlang compile file.tl` → generate bytecode

### 2.6 TrustCode Integration
- "Run" button (F5) in editor → compile and execute open file
- Output in integrated terminal panel
- Clickable error line numbers

### Phase 2 Line Budget
| Component | Estimated Lines |
|-----------|----------------|
| Lexer/Tokenizer | 300 |
| Parser/AST | 500 |
| Bytecode VM | 400 |
| Compiler | 300 |
| REPL + Shell | 100 |
| **Total** | **~1,600 lines** |

### Why This is a Killer Feature
- **No bare-metal OS** has a compiler/interpreter for a Rust-like language
- Users can write code, compile, and execute **without leaving the OS**
- The tokenizer already exists (syntax highlighting) → we extend it
- The transpiler/runtime already exists → we reuse the syscall runtime

---

## Phase 3 — Architectural Optimizations

### 3.1 Refactor `shell.rs` (13,052 lines → ~8,000 lines)
```
shell.rs                    →  shell/
                                ├── mod.rs          (core + dispatch)
                                ├── cosmic.rs       (COSMIC2 desktop, 4,700 lines)
                                ├── commands/
                                │   ├── fs.rs       (ls, cd, cat, cp, mv)
                                │   ├── system.rs   (clear, time, whoami)
                                │   ├── network.rs  (ping, curl, ifconfig)
                                │   ├── vm.rs       (hypervisor, linux)
                                │   └── misc.rs     (browser, transpiler)
                                └── stubs.rs        (unimplemented commands)
```
- **Net:** 0 lines (reorganization), but shell.rs goes from 13K → ~5K (excluding COSMIC)

### 3.2 Merge Compositors
- `compositor.rs` (SSE2) + `graphics/compositor.rs` (TrustGL)
- → Single `graphics/compositor.rs` with backends
- **Savings:** ~400 lines

### 3.3 Framebuffer Optimization
- `fill_rect()` loops pixel-by-pixel in some paths
- Use `rep stosd` / `memset` for solid rectangles
- SIMD `_mm_store_si128` for surface copies
- **Performance gain:** +20-30% rendering

---

## Phase 4 — Developer Features

### 4.1 Pipes & Redirection (~200 lines)
```bash
cat file.rs | grep "fn " | wc -l
echo "data" > output.txt
ls >> log.txt
```
- Pipeline parser in `shell/dispatch.rs`
- Inter-command output buffer

### 4.2 GDB-like Debugger (~500 lines)
- Breakpoints (INT3), single-step (RFLAGS.TF)
- Inspect registers, memory, stack
- Integrated with TrustLang REPL: `debug run file.tl`

### 4.3 Minimal Git Client (~800 lines)
- `git clone` via HTTPS (TLS 1.3 already implemented!)
- `git status`, `git add`, `git commit` (local)
- Pack format parser, object storage on TrustFS

### 4.4 Package Manager (~300 lines)
- `trust install <package>` from an HTTP registry
- Download → extract → place in `/usr/bin/`
- Simple TOML manifest

---

## Summary Table

| Phase | Goal | Lines Δ | Estimated Time |
|-------|------|---------|----------------|
| **Phase 0** | Cleanup & dead code | **-5,600** | 1-2 hours |
| **Phase 1** | Reliable persistence | **+260** | 3-4 hours |
| **Phase 2** | TrustLang compiler | **+1,600** | 6-8 hours |
| **Phase 3** | Architecture & perf | **-1,000** | 3-4 hours |
| **Phase 4** | Developer features | **+1,800** | 8-10 hours |
| **Total** | | **~89,200** | ~22-28 hours |

### Estimated Final Score
```
Before:  98,112 lines — with 1,891 dead code, 3,100 duplicated frameworks
After:   ~89,200 lines — with compiler, persistence, debugger, pipes
```

**A complete OS with an integrated compiler in ~89K lines** — unique in the world.

For comparison:
- xv6: 10K lines (but zero features)
- Redox: 200K+ lines (no integrated compiler)
- SerenityOS: 500K+ lines (no integrated editor or compiler)
- Linux: 35M+ lines

---

## Recommended Execution Order

```
1. Phase 0.1-0.3  →  Remove dead code (quick wins)
2. Phase 1.1-1.3  →  TrustFS indirect blocks + WAL
3. Phase 1.5      →  Ctrl+S → disk in TrustCode
4. Phase 2.1-2.3  →  Lexer + Parser + VM TrustLang
5. Phase 2.4-2.6  →  Compiler + REPL + editor integration
6. Phase 0.4-0.5  →  Network cleanup + UI frameworks
7. Phase 3        →  Architecture refactoring
8. Phase 4        →  Pipes, debugger, git
```

Phases 0.1 and 1 can start **immediately**.

---

*Last updated: February 2026*
