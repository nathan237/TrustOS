# TrustLab — OS Introspection Laboratory
## The First Real-Time Educational OS Environment Ever Built

---

## Vision

**TrustLab** transforms TrustOS from an operating system into a **living computer science classroom**. Instead of reading textbooks about what happens when you type `ls`, you *watch it happen* — syscalls firing, the VFS traversing inodes, the scheduler switching contexts, the framebuffer rendering pixels — all in real-time, annotated, from inside the OS itself.

No OS has ever done this. Not xv6, not MINIX, not Redox, not Linux. They teach through source code reading. TrustLab teaches through **live observation**.

---

## Architecture

```
┌─────────────────────┬──────────────────────────┬───────────────────────────┐
│   Hardware Status    │  TrustView (Live Binary) │   Command Guide (Docs)   │
│                      │                          │                          │
│  CPU · Memory · IRQs │  Hex + Disasm + Edit     │  Categories · Search     │
│  PCI · Disk · Net    │  Live reload on modify   │  Syntax · Examples       │
│  Allocator stats     │  Linked to File Tree     │  Click → Execute         │
├─────────────────────┼──────────────────────────┼───────────────────────────┤
│   File System Tree   │  TrustLang Editor        │   Live Kernel Trace      │
│                      │                          │                          │
│  Expandable tree     │  Edit · Compile · Run    │  Syscalls · IRQs · VFS   │
│  Icons per type      │  Output → Trace panel    │  Scheduler · Allocator   │
│  Click → View/Edit   │  Bytecode → TrustView    │  Filterable · Annotated  │
├─────────────────────┴──────────────────────────┴───────────────────────────┤
│  [LAB] Shell@trutos: _                                                     │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Competitive Analysis

| Tool | What it does | What TrustLab does differently |
|------|-------------|-------------------------------|
| **xv6 (MIT)** | Minimal teaching OS | Learn by reading C source — no introspection |
| **MINIX 3** | Microkernel teaching OS | Same: read code to understand |
| **DTrace / SystemTap** | Kernel tracing | External tool, not educational, requires expertise |
| **Process Monitor (Windows)** | Syscall logging | Black-box observation from outside |
| **KernelShark** | Trace visualization | Post-mortem analysis, not live |
| **htop / btop** | System monitoring | Metrics only, no educational context |
| **Redox OS** | Rust OS | No introspection mode |
| **TrustLab** | **Live OS classroom** | **Watch kernel internals AS YOU USE IT, annotated, from inside** |

**TrustLab is the only system where typing a command shows you the entire execution pipeline in real-time with educational annotations.**

---

## Implementation Roadmap

### Phase 1 — Foundation: Layout + Event Bus (~400 lines)
**Priority: CRITICAL — Everything depends on this**

- [ ] `kernel/src/lab_mode.rs` — Main module
  - `LabState` struct: panel states, event bus, active panel tracking
  - `LabEvent` enum: FileSelected, CommandExecuted, AddressChanged, TraceEvent
  - `LabEventBus`: ring buffer of events, subscribers per panel
- [ ] `WindowType::LabMode` in desktop.rs
  - Single fullscreen-ish window (1200×700+)
  - 6-panel grid layout with resizable splitters
  - Panel focus system (F1-F6 hotkeys, click, Tab cycle)
- [ ] Shell command: `lab` → opens Lab Mode
- [ ] `[LAB]` indicator in shell prompt when active
- [ ] Shared state: `current_file`, `current_address`, `current_command`

**Deliverable:** Empty 6-panel window opens, panels focusable, shell connected.

---

### Phase 2 — Live Kernel Trace (~600 lines)
**Priority: HIGH — This is the killer feature**

- [ ] `kernel/src/lab_mode/trace.rs` — Trace engine
  - `TraceEvent` struct: timestamp_us, category, message, detail
  - `TraceCategory` enum: Syscall, IRQ, VFS, Scheduler, Allocator, Network, Framebuffer, Shell
  - Global ring buffer: `static TRACE_RING: Mutex<TraceRing>` (capacity 8192 events)
  - `trace_emit!(category, "format", args)` macro — zero-cost when Lab not active
- [ ] Trace hooks (minimal overhead, behind `LAB_ACTIVE` flag):
  - `syscall.rs`: log every syscall entry/exit with args + return value
  - `interrupts.rs`: log IRQ number + handler name
  - `vfs.rs`: log open/read/write/close with path + size
  - `scheduler.rs`: log context switches with process names
  - `memory.rs`: log alloc/dealloc with size + heap stats
  - `shell.rs`: log command parse + dispatch
- [ ] Trace panel renderer:
  - Auto-scroll (follow mode) with manual scroll override
  - Color per category (blue=syscall, red=IRQ, green=VFS, yellow=sched, cyan=alloc)
  - Timestamp column + category badge + message
  - Filter toggles per category (click category header to show/hide)
- [ ] Educational annotations:
  - `[?]` button per event type → expands explanation
  - ~30 pre-written annotations for common events
  - Example: IRQ#1 → "Keyboard interrupt. The PS/2 controller signals the CPU via APIC when a key is pressed. The handler reads the scancode from port 0x60."

**Deliverable:** Type `ls` → watch 15+ trace events scroll in real-time showing the full execution pipeline.

---

### Phase 3 — Hardware Status Panel (~350 lines)
**Priority: MEDIUM**

- [ ] `kernel/src/lab_mode/hardware.rs`
- [ ] Live metrics (refreshed every frame when visible):
  - CPU: vendor, model, features (SSE, AVX, AES-NI), TSC frequency, ring level
  - Memory: heap used/free/total, allocation rate, largest free block
  - IRQ counters: per-IRQ fire count since boot, rate/sec
  - PCI devices: list with vendor:device IDs, driver status
  - Disk: sectors read/written, I/O operations count
  - Network: packets TX/RX, bytes TX/RX, driver status
  - Uptime, tick count, context switch count
- [ ] Visual bars for memory/CPU utilization
- [ ] Flash animation when IRQ fires (educational: "see the hardware reacting")

**Deliverable:** Real-time hardware dashboard that reacts visibly to user actions.

---

### Phase 4 — Command Guide Panel (~500 lines)
**Priority: MEDIUM**

- [ ] `kernel/src/lab_mode/guide.rs`
- [ ] Command database (all ~150+ shell commands):
  - `CommandDoc` struct: name, category, syntax, description, examples[], related_commands[]
  - Categories: Filesystem, Network, System, Programming, Hardware, Debug, Fun
- [ ] UI:
  - Category tabs/dropdown at top
  - Scrollable command list (left side of panel)
  - Detail view (right side): syntax highlight, examples with expected output
  - Search/filter bar: type to filter commands live
- [ ] Interactivity:
  - Click example → paste into shell bar
  - Click command name → show in trace what it does (educational link)
  - "Try it" button → execute and observe trace
- [ ] Context-aware: when user types in shell, auto-highlight matching command in guide

**Deliverable:** Interactive man pages with live examples, fully searchable.

---

### Phase 5 — File System Tree Panel (~350 lines)
**Priority: MEDIUM**

- [ ] `kernel/src/lab_mode/filetree.rs`
- [ ] True expandable/collapsible tree:
  - Indent levels, [+]/[-] toggle icons
  - Lazy loading (expand on click, don't load all at once)
  - Icons per file type: 📁 dir, 🦀 .rs, 📝 .txt, ⚡ binary/ELF, 🔧 .tl
- [ ] Navigation:
  - Arrow keys: up/down navigate, left collapse, right expand
  - Enter: select file (propagate to other panels)
  - Mouse click: same
- [ ] Panel interconnection:
  - Select file → TrustView loads binary analysis (if ELF)
  - Select .tl file → TrustLang editor opens it
  - Select any file → Hardware panel shows file size, inode info
  - New files from editor appear in tree automatically

**Deliverable:** Visual file browser that drives all other panels.

---

### Phase 6 — TrustView Integration (~250 lines)
**Priority: MEDIUM-LOW**

- [ ] Embed existing `binary_viewer` rendering into Lab panel
- [ ] New features for Lab context:
  - Inline hex editing: click byte → type new value → modify in memory
  - "Save" button: write modified binary back to VFS
  - Auto re-analyze: after save, re-run disassembly + xrefs
  - Address sync: when trace shows a syscall address, TrustView jumps to it
- [ ] Linked to File Tree: file selection auto-loads in TrustView

**Deliverable:** Edit binaries live and see the analysis update.

---

### Phase 7 — TrustLang Editor Integration (~200 lines)
**Priority: LOW**

- [ ] Embed TrustCode editor in Lab panel (subset: single file editing)
- [ ] TrustLang-focused workflow:
  - Compile button → show bytecode in TrustView panel
  - Run button → output appears in trace panel as LabEvent
  - Errors highlighted in editor + shown in trace
- [ ] File sync: saved files appear in File Tree immediately

**Deliverable:** Write code → compile → see bytecode → run → observe execution, all in one view.

---

### Phase 8 — Interconnection & Polish (~200 lines)
**Priority: LOW**

- [ ] Bidirectional linking:
  - Address in TrustView ↔ matching trace events highlighted
  - Command in Guide ↔ trace filter auto-applied
  - File in Tree ↔ loaded in View + Editor
- [ ] Keyboard shortcuts:
  - F1-F6: focus panel 1-6
  - Ctrl+F: search in active panel
  - Ctrl+L: toggle Lab Mode
  - Escape: back to desktop
- [ ] Visual polish:
  - Panel border highlights for active panel
  - Smooth scroll animations
  - Panel resize by dragging splitters
- [ ] "Follow mode" toggle: trace auto-scrolls and all panels sync to latest action
- [ ] Intro tutorial: first launch shows guided walkthrough

**Deliverable:** Polished, interconnected, professional-grade lab environment.

---

## Estimated Totals

| Phase | Lines | Priority | Dependencies |
|-------|-------|----------|-------------|
| 1. Foundation | ~400 | Critical | None |
| 2. Kernel Trace | ~600 | High | Phase 1 |
| 3. Hardware Status | ~350 | Medium | Phase 1 |
| 4. Command Guide | ~500 | Medium | Phase 1 |
| 5. File System Tree | ~350 | Medium | Phase 1 |
| 6. TrustView | ~250 | Medium-Low | Phase 1, 5 |
| 7. TrustLang Editor | ~200 | Low | Phase 1, 5 |
| 8. Polish | ~200 | Low | All |
| **Total** | **~2,850** | | |

**Recommended build order:** 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

---

## Video Demo Sequence (see TRUSTLAB_VIDEO.md)

The killer demo: type `ls` and watch the entire OS pipeline execute in real-time — from keypress IRQ to framebuffer render — annotated and explained. No other OS on Earth can do this.

---

*TrustLab — Because the best way to learn an OS is to watch it think.*
