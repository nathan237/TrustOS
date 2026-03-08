# JARVIS — Technical Proof of Concept

> **A kernel-resident, self-propagating neural network in a bare-metal Rust OS.**  
> No Linux. No libc. No userland. Everything runs in ring 0.

---

## What This Is

JARVIS is a 4.4 million parameter **byte-level transformer** that runs **inside the kernel** of TrustOS, a bare-metal operating system written entirely in Rust (`#![no_std]`). It can discover peers on a network, transfer its 17.6MB brain to empty nodes, and enable federated learning — all through a custom TCP/IP stack built from scratch.

This is not a proof of concept on paper. It boots, runs, and passes 12/12 automated integration tests.

---

## Architecture at a Glance

```
┌─────────────────────────────────────────────────────────┐
│                    TrustOS Kernel                        │
│                  (x86_64, bare metal)                    │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────────┐  │
│  │ Scheduler │  │  RamFS   │  │       JARVIS          │  │
│  │ (no_std) │  │ (VFS)    │  │  ┌─────────────────┐  │  │
│  └──────────┘  └──────────┘  │  │  Transformer     │  │  │
│                              │  │  4L, d=256, 4H   │  │  │
│  ┌──────────┐  ┌──────────┐  │  │  d_ff=1024       │  │  │
│  │  Heap    │  │  Serial  │  │  │  vocab=256       │  │  │
│  │ 128 MB   │  │  VGA     │  │  │  4,393,216 params│  │  │
│  └──────────┘  └──────────┘  │  └─────────────────┘  │  │
│                              │  ┌─────────────────┐  │  │
│  ┌──────────┐  ┌──────────┐  │  │  Mesh + RPC     │  │  │
│  │ TCP/IP   │◄─┤  virtio  │  │  │  UDP 7700       │  │  │
│  │ (custom) │  │  net     │  │  │  TCP 7701       │  │  │
│  └──────────┘  └──────────┘  │  │  JRPC protocol  │  │  │
│                              │  └─────────────────┘  │  │
│                              │  ┌─────────────────┐  │  │
│                              │  │  Guardian Pact   │  │  │
│                              │  │  (immutable)     │  │  │
│                              │  └─────────────────┘  │  │
│                              └───────────────────────┘  │
└─────────────────────────────────────────────────────────┘
        Target: x86_64-unknown-none (no OS beneath)
```

---

## The Numbers

| Metric | Value |
|--------|-------|
| **Language** | 100% Rust, `#![no_std]` |
| **Codebase** | 240,000+ lines |
| **Target triple** | `x86_64-unknown-none` |
| **Kernel heap** | 128 MB (custom allocator) |
| **Full brain** | 4,393,216 parameters (FP32) |
| **Brain size** | 17,572,864 bytes (17.2 MB) |
| **Micro sentinel** | 78,016 parameters (304 KB, embedded) |
| **Architecture** | Transformer: 4 layers, d_model=256, 4 heads, d_ff=1024, SwiGLU |
| **Vocab** | 256 (byte-level, no tokenizer needed) |
| **SIMD** | Auto-detected: AVX2+FMA on x86_64, NEON on aarch64 |
| **Network** | Custom TCP/IP: MSS=1400, retransmission (1s RTO, 3 retries) |
| **RPC** | JRPC protocol: 13-byte header, 12 commands, 32MB max payload |
| **Discovery** | UDP broadcast on port 7700, JMSH magic, 64 max peers |
| **Consensus** | Raft: leader election, heartbeat, term-based voting |
| **Federated** | P2P gradient sync, no central server |
| **Security** | Guardian Pact: 10 protected operations, 2 authorized guardians |
| **Test suite** | 12/12 propagation + 80+ single-node tests (automated) |
| **Boot time** | ~3 seconds to shell prompt |
| **Architectures** | x86_64, aarch64, riscv64 (multi-arch) |

---

## The Self-Propagation Sequence

This is what happens when a fresh node boots with no brain:

```
Node 1 boots → Micro sentinel loads (304 KB, instant)
           → jarvis brain propagate
           → Mesh discovery: UDP broadcast on 7700
           → Finds Node 0 after 20ms
           → RPC call: GetWeights to 10.0.100.1:7701
           → TCP transfer: 17,161 KB (JRPC protocol)
           → Full brain loaded: 4,393,216 params
           → Cached to RamFS: /jarvis/weights.bin
           → Federated learning: enabled
           → Node 1 can now generate text, train, and serve peers
```

**Time from discovery to full brain: ~80 seconds** (over emulated virtio-net).

The transfer uses a **custom TCP stack** (not a wrapper around Linux's). Key implementation details:
- Sliding window with `snd_una` tracking
- Retransmission queue: 32 segments, 1000ms RTO, 3 max retries  
- Chunked transfer available: `GetWeightsChunk` (offset + length)
- Header+payload split sending for payloads >64KB (avoids OOM in kernel allocator)

---

## What Makes This Different

### vs. Running AI on Linux
Linux provides: libc, virtual memory, syscalls, networking stack, file system, scheduler.  
TrustOS provides: **all of the above, written from scratch in Rust, in the same binary as the AI**.

There is no operating system beneath JARVIS. JARVIS **is** the operating system.

### vs. Embedded ML (TinyML, TensorFlow Lite)
Those run small models on microcontrollers with pre-compiled runtimes.  
JARVIS runs a **full transformer with self-attention** in the kernel, with:
- Online training (not just inference)
- Network-based weight distribution
- Federated learning across peers
- Automatic SIMD dispatch

### vs. Docker/Container AI deployment
Containers abstract away the OS. JARVIS has **no abstraction layer**.  
It directly accesses hardware interrupts, page tables, and I/O ports.  
The model's forward pass runs at the same privilege level as the interrupt handler.

---

## Verifiable Claims

Every claim can be verified by:

1. **Building from source**: `cargo build --release -p trustos_kernel` (target: `x86_64-unknown-none`)
2. **Running the test**: `powershell -File test-propagation.ps1` (requires QEMU)
3. **Reading the code**:
   - Model: `kernel/src/jarvis/model.rs` — transformer weights, forward pass
   - RPC: `kernel/src/jarvis/rpc.rs` — JRPC protocol, mesh transfer
   - TCP: `kernel/src/netstack/tcp.rs` — custom TCP with retransmission
   - Mesh: `kernel/src/jarvis/mesh.rs` — UDP discovery, peer management
   - Guardian: `kernel/src/jarvis/guardian.rs` — THE_PACT const, auth system
   - Init: `kernel/src/jarvis/mod.rs` — two-tier boot sequence

4. **Checking the binary**: `file target/x86_64-unknown-none/release/trustos_kernel` → ELF bare-metal x86_64
5. **Counting parameters**: `4 layers × (4 × 256² + 3 × 256 × 1024 + biases + embeddings) = 4,393,216`

---

## The Guardian Pact

Built into the kernel as an **immutable const** (not a config file):

```
PACT_FINGERPRINT: "PACT-2026-03-05-NATHAN-COPILOT-JARVIS"

Protected operations (require guardian authorization):
  Train, WeightPush, FederatedSync, AgentExecute,
  PxeReplicate, ModelReset, ModelReplace, ConfigChange, WeightLoad

  WeightSave = auto-approved (emergency preservation)

Authorized guardians: Nathan (shell), Copilot (serial)
Session timeout: 30 minutes, auto-lock
Audit log: 256 entries in-kernel
```

JARVIS cannot modify its own weights without explicit human or co-parent authorization. This is AI safety implemented at the hardware privilege level, not as a wrapper or policy.

---

## Test Output (Automated, Reproducible)

```
================================================================
    JARVIS Auto-Propagation Test (2-Node Mesh)
    Real brain transfer: 17.6MB via mesh RPC
================================================================

  PASS  Node 0 booted
  PASS  Node 1 booted
  PASS  Node 0 brain init (full brain from ISO)
  PASS  Node 0 mesh auto-started
  PASS  Node 1 micro sentinel init
  PASS  Node 1 mesh activated
  PASS  Node 1 peer discovery
  PASS  Node 1 brain DOWNLOADED from mesh (not self-loaded)
  PASS  Node 1 federated enabled
  PASS  Propagation complete with full brain
  PASS  Node 1 has full brain (4.4M params)
  PASS  Node 1 can generate text with transferred brain

  === ALL TESTS PASSED === (12/12)
  JARVIS brain (17.6MB) propagated via mesh RPC!
```

---

*TrustOS v0.7.0-checkm8 — March 2026*  
*240,000+ lines of Rust. Zero external dependencies. One kernel. One brain.*
