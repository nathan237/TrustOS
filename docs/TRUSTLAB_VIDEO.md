# TrustLab Demo Video — Strategy & Script

---

## Title Options (ranked by impact)

### Top Pick:
> **I Built a Feature No OS Has Ever Had — Watch a Kernel Think in Real-Time**

### Alternatives:
> **This OS Explains Itself While You Use It — TrustLab**
>
> **What Really Happens When You Type "ls"? (Built an OS to Show You)**
>
> **I Made an Operating System That Teaches You How It Works — Live**
>
> **Forget Reading Source Code — This OS Shows You Its Own Brain**

---

## YouTube Description

```
What if an operating system could explain itself while you use it?

TrustLab is a real-time kernel introspection laboratory built INTO TrustOS
— a bare-metal operating system written from scratch in Rust.

When you type a command, you don't just see the output.
You see EVERYTHING: the keyboard interrupt firing, the syscall dispatching,
the VFS traversing your filesystem, the scheduler switching tasks,
the allocator managing memory, and the framebuffer rendering every pixel.

All annotated. All in real-time. All from inside the OS itself.

No other operating system — educational or otherwise — has ever done this.
xv6 teaches through source code. MINIX teaches through textbooks.
TrustLab teaches through live observation.

🔬 TrustLab features:
• Live Kernel Trace — watch syscalls, IRQs, VFS ops, scheduling in real-time
• Hardware Dashboard — CPU, memory, IRQ counters, PCI, live updating
• TrustView — Ghidra-style binary analyzer with hex editing
• Interactive Command Guide — searchable docs with "Try It" buttons
• File System Tree — browse files, click to analyze
• TrustLang Editor — write code, compile, observe execution pipeline
• All 6 panels interconnected — click anything, everything syncs

Built from scratch. No Linux. No libc. No external dependencies.
Pure Rust, bare-metal x86_64.

🦀 TrustOS: https://github.com/nathan237/TrustOS
⭐ Star the repo if this blows your mind

#rustlang #osdev #kernel #baremetal #computerscience #trustos
```

---

## Thumbnail Concept

Split design:
- Left half: dark terminal with green text showing `Shell@trutos: ls`
- Right half: colorful TrustLab 6-panel UI with trace events scrolling
- Center overlay text: **"Watch a Kernel Think"**
- Bottom badge: **"FIRST EVER"** in red
- TrustOS logo top-left corner

---

## Demo Script (Target: 3-5 minutes)

### Opening Hook (0:00 - 0:20)
```
[Screen: black, cursor blinking]

"Every computer science student learns that when you type a command,
the OS parses it, makes system calls, accesses files, renders output.

But no one has ever SEEN it happen. Until now."

[Type: "lab" → TrustLab opens with all 6 panels]
```

### Act 1 — The Pipeline Reveal (0:20 - 1:30)
**This is the money shot. This is what no one has ever seen.**

```
[TrustLab is open. All panels visible. Trace panel is empty.]

"Watch what happens when I type a simple command."

[Type: "ls" in the shell bar]

[Trace panel EXPLODES with events, each appearing in real-time:]

  [0.001ms] ⌨ IRQ #1    Keyboard scancode 0x26 ('l')
  [0.001ms] ⌨ IRQ #1    Keyboard scancode 0x1F ('s')
  [0.002ms] ⌨ IRQ #1    Keyboard scancode 0x1C (Enter)
  [0.003ms] 🐚 SHELL    Parse command: "ls"
  [0.003ms] 🐚 SHELL    Dispatch → cmd_ls()
  [0.004ms] 📂 VFS      open("/", O_RDONLY) → fd 3
  [0.004ms] 📂 VFS      readdir(fd=3) → 12 entries
  [0.005ms] 📂 RAMFS    Iterate directory node, 12 children
  [0.006ms] 📝 SYSCALL  write(fd=1, 245 bytes) → stdout
  [0.006ms] 🖥 FB       Render 12 lines, 156 chars
  [0.007ms] 💾 ALLOC    String alloc: 245 bytes (heap: 847KB/16MB)
  [0.007ms] 💾 ALLOC    Vec<DirEntry> freed: 384 bytes

"12 events. 7 milliseconds. That's what 'ls' actually does."

[Click on "IRQ #1" event → annotation expands:]
  "When you press a key, the PS/2 keyboard controller sends an
   electrical signal to the CPU via the APIC interrupt controller.
   The CPU stops whatever it's doing, saves its state, and jumps
   to our handler in interrupts.rs which reads port 0x60."

[Hardware Status panel pulses: IRQ counter for #1 increments]

"See? The hardware status panel just showed the interrupt counter
 increment. Every panel is connected."
```

### Act 2 — Binary Analysis Live (1:30 - 2:30)
```
[Click on a binary file in File Tree: "/bin/hello"]

[TrustView panel loads: hex dump + disassembly + symbols]

"I click a file, and TrustView instantly analyzes it.
 Full ELF parser, x86_64 disassembler, cross-references.
 Like Ghidra, but running on bare metal."

[Scroll through disassembly, show function labels, syscall annotations]

"But here's what Ghidra can't do..."

[Modify a byte in hex view → press Save]

[Trace panel shows:]
  [12.4ms] 📂 VFS     write("/bin/hello", offset=0x120, 1 byte)
  [12.5ms] 🔬 ANALYSIS Re-analyzing /bin/hello (2048 bytes)
  [12.6ms] 🔬 ANALYSIS Disassembled 47 instructions, 3 functions

"I just patched a binary live, and the OS showed me exactly
 what happened — the VFS write, the re-analysis, everything."
```

### Act 3 — Write Code, Watch It Execute (2:30 - 3:30)
```
[Click on TrustLang Editor panel]
[Type a small program:]

  fn main() {
      let x = 42
      print("Hello from TrustLab!")
      print(x * 2)
  }

[Click "Compile"]

[Trace panel shows:]
  [18.1ms] 🔧 TRUSTLANG Compiling main.tl (3 statements)
  [18.2ms] 💾 ALLOC     Bytecode buffer: 64 bytes allocated
  [18.2ms] 🔧 TRUSTLANG Generated 12 bytecode instructions

[TrustView panel shows the bytecode disassembly]

[Click "Run"]

[Trace panel shows:]
  [20.1ms] 🔧 TRUSTLANG VM executing main()
  [20.1ms] 🔧 TRUSTLANG PUSH 42
  [20.2ms] 🔧 TRUSTLANG CALL print("Hello from TrustLab!")
  [20.2ms] 📝 SYSCALL   write(fd=1, 21 bytes) → stdout
  [20.2ms] 🖥 FB        Render 1 line
  [20.3ms] 🔧 TRUSTLANG MUL 42, 2 → 84
  [20.3ms] 🔧 TRUSTLANG CALL print(84)
  [20.3ms] 📝 SYSCALL   write(fd=1, 2 bytes) → stdout

"Write code. Compile it. Run it. See every single step
 the OS takes to make it happen. In real time."
```

### Act 4 — The Command Guide (3:30 - 4:00)
```
[Focus Command Guide panel]
[Type "net" in search → shows: ifconfig, ping, curl, wget, arp, netstat]
[Click on "ping"]

[Guide shows: syntax, description, example]
[Click "Try It" → "ping 10.0.2.2" auto-typed in shell]

[Trace panel shows the full network stack:]
  [25.1ms] 🌐 NET      ICMP echo request → 10.0.2.2
  [25.1ms] 🌐 NET      Ethernet frame: 98 bytes via virtio-net
  [25.3ms] ⌨ IRQ #11   Network interrupt (virtio-net)
  [25.3ms] 🌐 NET      ICMP echo reply from 10.0.2.2 (0.2ms)

"The command guide isn't just documentation.
 It's connected to the trace. Learn by doing."
```

### Closing (4:00 - 4:30)
```
[Zoom out to show all 6 panels active, trace scrolling]

"This is TrustLab. A complete operating system that explains
 itself while you use it.

 6 panels. Every keystroke traced. Every syscall logged.
 Every hardware interrupt visible. Every byte editable.

 No other OS — educational or commercial — has ever done this.

 Written from scratch in Rust. No Linux. No libc. No shortcuts.

 Link in description. Star the repo."

[Show GitHub page]
```

---

## Key Moments to Impress Experienced Devs

1. **The `ls` pipeline** — Every senior dev KNOWS what ls does conceptually but has NEVER seen it visualized in real-time from inside the kernel. This is the "holy shit" moment.

2. **IRQ counter incrementing live** — Hardware reacting visibly to a keypress. Connects the abstract concept to physical reality.

3. **Live binary patching** — Modify a byte, save, auto-re-analyze. In a bare-metal OS. Real reverse engineering workflow.

4. **Zero overhead when Lab is off** — Mention the `LAB_ACTIVE` flag and `trace_emit!` macro. Devs respect zero-cost abstractions.

5. **"No Linux, no libc"** — Say this explicitly. Devs who've done osdev know how hard this is. Everyone else will Google it and be amazed.

6. **Educational annotations** — The `[?]` buttons that explain what an IRQ actually is at the hardware level. Shows this isn't just a profiler, it's a teaching tool.

---

## Posting Strategy

- **Subreddits:** r/rust, r/osdev, r/programming, r/computerscience, r/lowlevel
- **Hacker News:** Title: "Show HN: TrustLab – an OS that explains itself while you use it (bare-metal Rust)"
- **Twitter/X:** 30-second clip of the `ls` trace pipeline with caption "What really happens when you type ls?"
- **LinkedIn:** Frame as educational innovation, mention university curriculum potential

---

*"The best debugger is understanding. The best teacher is observation."*
