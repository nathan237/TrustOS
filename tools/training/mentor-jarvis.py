#!/usr/bin/env python3
"""
Jarvis Mentor — Host-side training script for TrustOS neural brain.

Connects to QEMU serial via TCP and sends MENTOR: commands to teach Jarvis.
Run this while TrustOS is booted in QEMU with serial on TCP port 5555.

Usage:
    python mentor-jarvis.py                  # Interactive mode
    python mentor-jarvis.py --teach          # Auto-teach from built-in corpus
    python mentor-jarvis.py --eval           # Evaluate current model
    python mentor-jarvis.py --chat           # Chat with Jarvis
    python mentor-jarvis.py --batch FILE     # Train from text file
    python mentor-jarvis.py --status         # Check model status

QEMU launch (example):
    qemu-system-x86_64 -serial tcp:127.0.0.1:5555,server,nowait ...
"""

import socket
import sys
import time
import argparse
import random

# ═══════════════════════════════════════════════════════════════════════════════
# Connection
# ═══════════════════════════════════════════════════════════════════════════════

DEFAULT_HOST = "127.0.0.1"
DEFAULT_PORT = 5555


def connect(host=DEFAULT_HOST, port=DEFAULT_PORT, timeout=5):
    """Connect to QEMU serial TCP socket."""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(timeout)
    try:
        sock.connect((host, port))
        print(f"[+] Connected to {host}:{port}")
        return sock
    except Exception as e:
        print(f"[-] Cannot connect to {host}:{port}: {e}")
        print(f"    Make sure QEMU is running with: -serial tcp:{host}:{port},server,nowait")
        sys.exit(1)


def send_command(sock, cmd, wait_response=True, timeout=10):
    """Send a MENTOR: command and optionally wait for JARVIS: response."""
    full_cmd = f"MENTOR:{cmd}\n"
    sock.sendall(full_cmd.encode("utf-8"))

    if not wait_response:
        return None

    # Read response (JARVIS:...)
    sock.settimeout(timeout)
    buf = b""
    start = time.time()
    while time.time() - start < timeout:
        try:
            data = sock.recv(4096)
            if not data:
                break
            buf += data
            # Look for JARVIS: response line
            lines = buf.decode("utf-8", errors="replace").split("\n")
            for line in lines:
                line = line.strip()
                if line.startswith("JARVIS:"):
                    return line
        except socket.timeout:
            break
    return None


def drain_buffer(sock, timeout=0.5):
    """Read and discard any pending data in the socket."""
    sock.settimeout(timeout)
    try:
        while True:
            data = sock.recv(4096)
            if not data:
                break
    except socket.timeout:
        pass


# ═══════════════════════════════════════════════════════════════════════════════
# Training Corpus (matches kernel/src/jarvis/corpus.rs)
# ═══════════════════════════════════════════════════════════════════════════════

CORPUS_PHASES = [
    # Phase 0: Byte patterns
    [
        "the the the the the the the the",
        "hello hello hello hello hello",
        "world world world world world",
        "trust trust trust trust trust",
        "system system system system",
        "kernel kernel kernel kernel",
        "memory memory memory memory",
    ],
    # Phase 1: Simple sentences
    [
        "Hello, I am Jarvis.",
        "TrustOS is an operating system.",
        "Jarvis is an AI assistant.",
        "The kernel is written in Rust.",
        "Memory is managed by the heap.",
        "Type help for commands.",
        "Bonjour, je suis Jarvis.",
        "The shell is called tsh.",
        "I can learn from text.",
        "I am a neural network.",
    ],
    # Phase 2: Shell commands
    [
        "help: show available commands",
        "ls: list files in directory",
        "cat: display file contents",
        "echo: print text to screen",
        "pwd: print working directory",
        "ps: show running processes",
        "free: show memory usage",
        "uptime: show system uptime",
        "ping: test network connection",
        "jarvis: AI assistant",
    ],
    # Phase 3: Q&A pairs
    [
        "Q: who are you? A: I am Jarvis.",
        "Q: what is TrustOS? A: An OS in Rust.",
        "Q: how much memory? A: Type free.",
        "Q: list files? A: Type ls.",
        "Q: hello A: Hello! How can I help?",
        "Q: bonjour A: Salut!",
        "Q: system info A: Try neofetch.",
        "Q: help me A: Type help.",
    ],
    # Phase 4: Identity
    [
        "I am Jarvis, the TrustOS AI.",
        "I have 312K parameters.",
        "I am a 4-layer transformer.",
        "I run on bare metal.",
        "I learn by training.",
        "I am self-hosted. 100% local.",
    ],
]

# Extended corpus for longer training sessions
EXTENDED_CORPUS = [
    # More natural language
    "The quick brown fox jumps over the lazy dog.",
    "TrustOS boots in under 2 seconds on QEMU.",
    "Rust prevents memory bugs at compile time.",
    "The heap allocator uses a linked-list free list.",
    "VirtIO drivers handle disk and network I/O.",
    "The framebuffer renders at 1024x768 resolution.",
    "Processes are scheduled cooperatively.",
    "The TLS implementation supports Ed25519.",
    "IPv4 networking over virtio-net with DHCP.",
    "The chess engine uses alpha-beta pruning.",
    # French
    "TrustOS est un systeme d'exploitation en Rust.",
    "Jarvis apprend grace a un petit transformeur.",
    "La memoire est geree sans ramasse-miettes.",
    "Le noyau tourne en mode 64 bits.",
    "Le reseau utilise VirtIO et DHCP.",
    # Technical
    "RMSNorm normalizes without mean subtraction.",
    "SwiGLU uses a gated activation function.",
    "Attention is computed as softmax(QK^T/sqrt(d))V.",
    "Byte-level tokenization needs no vocabulary.",
    "Xavier initialization scales by 1/sqrt(d).",
]


# ═══════════════════════════════════════════════════════════════════════════════
# Commands
# ═══════════════════════════════════════════════════════════════════════════════

def cmd_status(sock):
    """Check model status."""
    drain_buffer(sock)
    resp = send_command(sock, "STATUS")
    if resp:
        print(f"  {resp}")
    else:
        print("  No response (is TrustOS booted?)")


def cmd_teach(sock, epochs=3, verbose=True):
    """Teach from the built-in corpus."""
    print(f"\n[*] Starting auto-teach ({epochs} epoch(s), {sum(len(p) for p in CORPUS_PHASES)} base + {len(EXTENDED_CORPUS)} extended sequences)")
    print()

    # Start batch mode
    drain_buffer(sock)
    send_command(sock, "BATCH_START", wait_response=True, timeout=2)

    total_loss = 0
    total_count = 0
    start_time = time.time()

    for epoch in range(epochs):
        print(f"  ── Epoch {epoch + 1}/{epochs} ──")

        # Phase-by-phase
        for phase_idx, phase in enumerate(CORPUS_PHASES):
            phase_names = ["Byte Patterns", "Simple Sentences", "Shell Commands", "Q&A Pairs", "Identity"]
            phase_loss = 0
            phase_count = 0

            for text in phase:
                resp = send_command(sock, f"TEACH:{text}", wait_response=True, timeout=5)
                if resp and "LOSS:" in resp:
                    try:
                        loss = float(resp.split("LOSS:")[1])
                        phase_loss += loss
                        total_loss += loss
                        phase_count += 1
                        total_count += 1
                    except ValueError:
                        pass
                time.sleep(0.01)  # Small delay to not overwhelm serial

            avg = phase_loss / max(phase_count, 1)
            name = phase_names[phase_idx] if phase_idx < len(phase_names) else f"Phase {phase_idx}"
            if verbose:
                print(f"    Phase {phase_idx} ({name}): {phase_count} seqs, avg loss = {avg:.3f}")

        # Extended corpus
        ext_loss = 0
        ext_count = 0
        for text in EXTENDED_CORPUS:
            resp = send_command(sock, f"TEACH:{text}", wait_response=True, timeout=5)
            if resp and "LOSS:" in resp:
                try:
                    loss = float(resp.split("LOSS:")[1])
                    ext_loss += loss
                    total_loss += loss
                    ext_count += 1
                    total_count += 1
                except ValueError:
                    pass
            time.sleep(0.01)

        if verbose and ext_count > 0:
            print(f"    Extended: {ext_count} seqs, avg loss = {ext_loss / ext_count:.3f}")

        print()

    # End batch mode
    resp = send_command(sock, "BATCH_END", wait_response=True, timeout=5)
    if resp:
        print(f"  {resp}")

    elapsed = time.time() - start_time
    avg_total = total_loss / max(total_count, 1)
    print(f"\n[+] Training complete: {total_count} sequences in {elapsed:.1f}s")
    print(f"    Average loss: {avg_total:.3f}")
    print(f"    Sequences/sec: {total_count / max(elapsed, 0.001):.1f}")

    # Save weights
    print("\n[*] Saving weights...")
    resp = send_command(sock, "SAVE", wait_response=True, timeout=5)
    if resp:
        print(f"  {resp}")


def cmd_eval(sock):
    """Evaluate model on test prompts."""
    print("\n[*] Evaluating model...")
    drain_buffer(sock)

    test_prompts = [
        "Hello",
        "The kernel",
        "I am Jarvis",
        "TrustOS is",
        "help: show",
        "Q: who are you? A:",
        "Bonjour",
        "ls: list",
        "memory",
    ]

    for prompt in test_prompts:
        # Eval loss
        resp = send_command(sock, f"EVAL:{prompt}", wait_response=True, timeout=5)
        loss_str = ""
        if resp and "LOSS:" in resp:
            loss_str = resp.split("LOSS:")[1].strip()

        # Generate continuation
        resp = send_command(sock, f"GENERATE:{prompt}", wait_response=True, timeout=10)
        gen_str = ""
        if resp and "GEN:" in resp:
            gen_str = resp.split("GEN:")[1].strip()[:60]

        print(f'  "{prompt}"')
        print(f'    Loss: {loss_str}   Gen: "{gen_str}"')
        print()


def cmd_chat(sock):
    """Interactive chat with Jarvis."""
    print("\n[*] Chat with Jarvis (type 'quit' to exit)")
    print("    Jarvis generates byte-level text — expect gibberish until trained!\n")

    drain_buffer(sock)

    while True:
        try:
            user = input("  You: ").strip()
        except (EOFError, KeyboardInterrupt):
            print()
            break

        if user.lower() in ("quit", "exit", "q"):
            break

        if not user:
            continue

        resp = send_command(sock, f"GENERATE:{user}", wait_response=True, timeout=15)
        if resp and "GEN:" in resp:
            gen = resp.split("GEN:")[1].strip()
            # Show printable chars
            clean = "".join(c if c.isprintable() else "." for c in gen)
            print(f"  Jarvis: {clean}")
        else:
            print("  Jarvis: (no response)")
        print()


def cmd_batch_file(sock, filepath):
    """Train from a text file (one sequence per line)."""
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            lines = [l.strip() for l in f if l.strip()]
    except FileNotFoundError:
        print(f"[-] File not found: {filepath}")
        return

    print(f"\n[*] Training from {filepath} ({len(lines)} lines)")
    drain_buffer(sock)

    send_command(sock, "BATCH_START", wait_response=True, timeout=2)

    total_loss = 0
    count = 0
    start = time.time()

    for i, line in enumerate(lines):
        # Truncate to 60 bytes (MAX_TRAIN_SEQ)
        text = line[:60]
        resp = send_command(sock, f"TEACH:{text}", wait_response=True, timeout=5)
        if resp and "LOSS:" in resp:
            try:
                loss = float(resp.split("LOSS:")[1])
                total_loss += loss
                count += 1
            except ValueError:
                pass

        if (i + 1) % 50 == 0:
            avg = total_loss / max(count, 1)
            print(f"    [{i + 1}/{len(lines)}] avg loss = {avg:.3f}")
        time.sleep(0.01)

    resp = send_command(sock, "BATCH_END", wait_response=True, timeout=5)
    if resp:
        print(f"  {resp}")

    elapsed = time.time() - start
    avg = total_loss / max(count, 1)
    print(f"\n[+] Done: {count} sequences, avg loss = {avg:.3f}, {elapsed:.1f}s")

    # Save
    resp = send_command(sock, "SAVE", wait_response=True, timeout=5)
    if resp:
        print(f"  {resp}")


def cmd_interactive(sock):
    """Interactive mentor session — all commands available."""
    print("\n[*] Interactive Mentor Mode")
    print("    Commands:")
    print("      teach <text>      Train on text")
    print("      eval <text>       Evaluate loss on text")
    print("      gen <prompt>      Generate from prompt")
    print("      status            Model status")
    print("      save / load       Persist weights")
    print("      reset             Reset to random")
    print("      config lr=<val>   Set learning rate")
    print("      auto [N]          Auto-teach N epochs (default 3)")
    print("      chat              Enter chat mode")
    print("      quit              Exit")
    print()

    drain_buffer(sock)

    while True:
        try:
            cmd = input("  mentor> ").strip()
        except (EOFError, KeyboardInterrupt):
            print()
            break

        if not cmd:
            continue

        if cmd.lower() in ("quit", "exit", "q"):
            break

        if cmd.startswith("teach "):
            text = cmd[6:]
            resp = send_command(sock, f"TEACH:{text}")
            print(f"    {resp}" if resp else "    (no response)")

        elif cmd.startswith("eval "):
            text = cmd[5:]
            resp = send_command(sock, f"EVAL:{text}")
            print(f"    {resp}" if resp else "    (no response)")

        elif cmd.startswith("gen "):
            prompt = cmd[4:]
            resp = send_command(sock, f"GENERATE:{prompt}", timeout=15)
            if resp and "GEN:" in resp:
                gen = resp.split("GEN:")[1].strip()
                clean = "".join(c if c.isprintable() else "." for c in gen)
                print(f"    {clean}")
            else:
                print(f"    {resp}" if resp else "    (no response)")

        elif cmd == "status":
            cmd_status(sock)

        elif cmd == "save":
            resp = send_command(sock, "SAVE")
            print(f"    {resp}" if resp else "    (no response)")

        elif cmd == "load":
            resp = send_command(sock, "LOAD")
            print(f"    {resp}" if resp else "    (no response)")

        elif cmd == "reset":
            resp = send_command(sock, "RESET")
            print(f"    {resp}" if resp else "    (no response)")

        elif cmd.startswith("config "):
            resp = send_command(sock, f"CONFIG:{cmd[7:]}")
            print(f"    {resp}" if resp else "    (no response)")

        elif cmd.startswith("auto"):
            parts = cmd.split()
            epochs = int(parts[1]) if len(parts) > 1 else 3
            cmd_teach(sock, epochs=epochs)

        elif cmd == "chat":
            cmd_chat(sock)

        else:
            print(f"    Unknown command: {cmd}")
            print("    Type 'teach', 'eval', 'gen', 'status', 'auto', 'chat', or 'quit'")


# ═══════════════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════════════

def main():
    parser = argparse.ArgumentParser(description="Jarvis Mentor — Teach the TrustOS AI via serial")
    parser.add_argument("--host", default=DEFAULT_HOST, help=f"QEMU serial host (default: {DEFAULT_HOST})")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help=f"QEMU serial port (default: {DEFAULT_PORT})")
    parser.add_argument("--teach", action="store_true", help="Auto-teach from built-in corpus")
    parser.add_argument("--epochs", type=int, default=3, help="Number of training epochs (default: 3)")
    parser.add_argument("--eval", action="store_true", help="Evaluate model on test prompts")
    parser.add_argument("--chat", action="store_true", help="Interactive chat with Jarvis")
    parser.add_argument("--batch", type=str, help="Train from text file (one line per sequence)")
    parser.add_argument("--status", action="store_true", help="Check model status")
    args = parser.parse_args()

    print("╔═══════════════════════════════════════════════════╗")
    print("║     Jarvis Mentor — TrustOS Neural Trainer        ║")
    print("║     Host-side training via QEMU serial             ║")
    print("╚═══════════════════════════════════════════════════╝")
    print()

    sock = connect(args.host, args.port)

    try:
        if args.status:
            cmd_status(sock)
        elif args.teach:
            cmd_teach(sock, epochs=args.epochs)
        elif args.eval:
            cmd_eval(sock)
        elif args.chat:
            cmd_chat(sock)
        elif args.batch:
            cmd_batch_file(sock, args.batch)
        else:
            cmd_interactive(sock)
    except KeyboardInterrupt:
        print("\n[*] Interrupted")
    finally:
        sock.close()
        print("[*] Disconnected")


if __name__ == "__main__":
    main()
