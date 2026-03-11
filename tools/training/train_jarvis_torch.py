#!/usr/bin/env python3
"""
JARVIS PyTorch Training — Exact reproduction of the 4.4M-param byte-level transformer.
Uses PyTorch with MKL (AVX2 + multi-threaded) for massive speedup over single-threaded Rust.

Architecture (mirrors kernel/src/jarvis/model.rs):
  - Vocab: 256 (byte-level)
  - d_model: 256, n_heads: 4, d_ff: 1024, n_layers: 4
  - SwiGLU FFN, RMSNorm, causal attention
  - Learned absolute position embeddings (max_seq=256)

Weight export is binary-compatible with jarvis-bench and kernel.
"""

import struct
import sys
import time
import math
import os
import torch
import torch.nn as nn
import torch.nn.functional as F

# Force unbuffered output
os.environ['PYTHONUNBUFFERED'] = '1'

# ════════════════════════════════════════════════════════════════
# Hyperparameters (same as Rust kernel)
# ════════════════════════════════════════════════════════════════
VOCAB_SIZE = 256
D_MODEL = 256
N_HEADS = 4
D_K = D_MODEL // N_HEADS  # 64
D_FF = 1024
N_LAYERS = 4
MAX_SEQ = 256
RMS_EPS = 1e-5

# ════════════════════════════════════════════════════════════════
# Model
# ════════════════════════════════════════════════════════════════
class RMSNorm(nn.Module):
    def __init__(self, dim):
        super().__init__()
        self.weight = nn.Parameter(torch.ones(dim))

    def forward(self, x):
        rms = torch.sqrt(x.pow(2).mean(-1, keepdim=True) + RMS_EPS)
        return x / rms * self.weight


class SwiGLU_FFN(nn.Module):
    def __init__(self, d_model, d_ff):
        super().__init__()
        self.w_gate = nn.Linear(d_model, d_ff, bias=False)
        self.w_up = nn.Linear(d_model, d_ff, bias=False)
        self.w_down = nn.Linear(d_ff, d_model, bias=False)

    def forward(self, x):
        return self.w_down(F.silu(self.w_gate(x)) * self.w_up(x))


class TransformerLayer(nn.Module):
    def __init__(self):
        super().__init__()
        self.rms_attn = RMSNorm(D_MODEL)
        self.w_q = nn.Linear(D_MODEL, D_MODEL, bias=False)
        self.w_k = nn.Linear(D_MODEL, D_MODEL, bias=False)
        self.w_v = nn.Linear(D_MODEL, D_MODEL, bias=False)
        self.w_o = nn.Linear(D_MODEL, D_MODEL, bias=False)
        self.rms_ffn = RMSNorm(D_MODEL)
        self.ffn = SwiGLU_FFN(D_MODEL, D_FF)

    def forward(self, x, causal_mask):
        # Pre-norm attention
        h = self.rms_attn(x)
        B, T, C = h.shape
        q = self.w_q(h).view(B, T, N_HEADS, D_K).transpose(1, 2)
        k = self.w_k(h).view(B, T, N_HEADS, D_K).transpose(1, 2)
        v = self.w_v(h).view(B, T, N_HEADS, D_K).transpose(1, 2)

        scores = (q @ k.transpose(-2, -1)) / math.sqrt(D_K)
        scores = scores.masked_fill(causal_mask[:, :, :T, :T] == 0, float('-inf'))
        attn = F.softmax(scores, dim=-1)
        out = (attn @ v).transpose(1, 2).contiguous().view(B, T, C)
        x = x + self.w_o(out)

        # Pre-norm FFN
        x = x + self.ffn(self.rms_ffn(x))
        return x


class JarvisTransformer(nn.Module):
    def __init__(self):
        super().__init__()
        self.token_embed = nn.Embedding(VOCAB_SIZE, D_MODEL)
        self.pos_embed = nn.Embedding(MAX_SEQ, D_MODEL)
        self.layers = nn.ModuleList([TransformerLayer() for _ in range(N_LAYERS)])
        self.rms_final = RMSNorm(D_MODEL)
        self.w_output = nn.Linear(D_MODEL, VOCAB_SIZE, bias=False)

        # Causal mask
        self.register_buffer('causal_mask',
            torch.tril(torch.ones(1, 1, MAX_SEQ, MAX_SEQ)))

        self._init_weights()

    def _init_weights(self):
        """Match the Rust initialization: Xavier-like with specific scales."""
        torch.manual_seed(42)
        embed_scale = 1.0 / math.sqrt(D_MODEL)
        nn.init.uniform_(self.token_embed.weight, -embed_scale, embed_scale)
        nn.init.uniform_(self.pos_embed.weight, -0.02, 0.02)
        for layer in self.layers:
            attn_scale = embed_scale
            ffn_scale = 1.0 / math.sqrt(D_FF)
            for p in [layer.w_q.weight, layer.w_k.weight, layer.w_v.weight, layer.w_o.weight]:
                nn.init.uniform_(p, -attn_scale, attn_scale)
            for p in [layer.ffn.w_gate.weight, layer.ffn.w_up.weight, layer.ffn.w_down.weight]:
                nn.init.uniform_(p, -ffn_scale, ffn_scale)
        nn.init.uniform_(self.w_output.weight, -embed_scale, embed_scale)

    def forward(self, tokens):
        B, T = tokens.shape
        positions = torch.arange(T, device=tokens.device).unsqueeze(0)
        x = self.token_embed(tokens) + self.pos_embed(positions)
        for layer in self.layers:
            x = layer(x, self.causal_mask)
        x = self.rms_final(x)
        return self.w_output(x)

    def param_count(self):
        return sum(p.numel() for p in self.parameters())


# ════════════════════════════════════════════════════════════════
# Weight Export (binary-compatible with Rust kernel & bench)
# ════════════════════════════════════════════════════════════════
def export_weights(model: JarvisTransformer, path: str):
    """
    Serialize in the exact same order as Rust's TransformerWeights::serialize().
    Weight matrices in nn.Linear are stored transposed vs Rust's row-major.
    Rust: matvec(out, w, x) does out[r] = sum(w[r*cols + c] * x[c])
    PyTorch nn.Linear: out = x @ W^T, so W has shape (out_features, in_features)
    The Rust code stores weights row-major as [rows][cols] and does w[r*cols+c]*x[c],
    which is equivalent to PyTorch's nn.Linear weight layout (out_feat, in_feat).
    """
    floats = []

    # token_embed: [VOCAB_SIZE * D_MODEL] — Rust stores row-major [vocab][d_model]
    floats.extend(model.token_embed.weight.detach().cpu().flatten().tolist())

    # pos_embed: [MAX_SEQ * D_MODEL]
    floats.extend(model.pos_embed.weight.detach().cpu().flatten().tolist())

    for layer in model.layers:
        # rms_attn: [D_MODEL]
        floats.extend(layer.rms_attn.weight.detach().cpu().flatten().tolist())
        # w_q, w_k, w_v, w_o: [D_MODEL * D_MODEL] — nn.Linear stores (out, in)
        floats.extend(layer.w_q.weight.detach().cpu().flatten().tolist())
        floats.extend(layer.w_k.weight.detach().cpu().flatten().tolist())
        floats.extend(layer.w_v.weight.detach().cpu().flatten().tolist())
        floats.extend(layer.w_o.weight.detach().cpu().flatten().tolist())
        # rms_ffn: [D_MODEL]
        floats.extend(layer.rms_ffn.weight.detach().cpu().flatten().tolist())
        # w_gate, w_up: [D_MODEL * D_FF], w_down: [D_FF * D_MODEL]
        floats.extend(layer.ffn.w_gate.weight.detach().cpu().flatten().tolist())
        floats.extend(layer.ffn.w_up.weight.detach().cpu().flatten().tolist())
        floats.extend(layer.ffn.w_down.weight.detach().cpu().flatten().tolist())

    # rms_final: [D_MODEL]
    floats.extend(model.rms_final.weight.detach().cpu().flatten().tolist())
    # w_output: [D_MODEL * VOCAB_SIZE]
    floats.extend(model.w_output.weight.detach().cpu().flatten().tolist())

    data = struct.pack(f'<{len(floats)}f', *floats)
    with open(path, 'wb') as f:
        f.write(data)
    print(f"[EXPORT] Saved {len(floats)} floats ({len(data)/1048576:.1f} MB) to {path}")


def load_weights(model: JarvisTransformer, path: str):
    """Load weights from Rust-format binary."""
    with open(path, 'rb') as f:
        data = f.read()
    n_floats = len(data) // 4
    floats = struct.unpack(f'<{n_floats}f', data)
    idx = 0

    def take(n):
        nonlocal idx
        t = torch.tensor(floats[idx:idx+n], dtype=torch.float32)
        idx += n
        return t

    model.token_embed.weight.data = take(VOCAB_SIZE * D_MODEL).view(VOCAB_SIZE, D_MODEL)
    model.pos_embed.weight.data = take(MAX_SEQ * D_MODEL).view(MAX_SEQ, D_MODEL)

    for layer in model.layers:
        layer.rms_attn.weight.data = take(D_MODEL)
        layer.w_q.weight.data = take(D_MODEL * D_MODEL).view(D_MODEL, D_MODEL)
        layer.w_k.weight.data = take(D_MODEL * D_MODEL).view(D_MODEL, D_MODEL)
        layer.w_v.weight.data = take(D_MODEL * D_MODEL).view(D_MODEL, D_MODEL)
        layer.w_o.weight.data = take(D_MODEL * D_MODEL).view(D_MODEL, D_MODEL)
        layer.rms_ffn.weight.data = take(D_MODEL)
        layer.ffn.w_gate.weight.data = take(D_MODEL * D_FF).view(D_FF, D_MODEL)
        layer.ffn.w_up.weight.data = take(D_MODEL * D_FF).view(D_FF, D_MODEL)
        layer.ffn.w_down.weight.data = take(D_FF * D_MODEL).view(D_MODEL, D_FF)

    model.rms_final.weight.data = take(D_MODEL)
    model.w_output.weight.data = take(D_MODEL * VOCAB_SIZE).view(VOCAB_SIZE, D_MODEL)
    print(f"[LOAD] Loaded {idx} floats from {path}")


# ════════════════════════════════════════════════════════════════
# Corpus Loader
# ════════════════════════════════════════════════════════════════
def load_corpus(path: str = "_bench_corpus.txt"):
    """Load corpus from bench format: Rust array syntax &["seq1","seq2",...] per line."""
    import re
    sequences = []
    with open(path, 'r', encoding='utf-8') as f:
        content = f.read()
    # Extract all quoted strings from the Rust array
    for m in re.finditer(r'"((?:[^"\\]|\\.)*)"', content):
        seq = m.group(1)
        if len(seq) >= 2:
            sequences.append(seq)
    print(f"[CORPUS] Loaded {len(sequences)} sequences from {path}")
    return sequences


def make_batches(sequences, batch_size, max_seq=MAX_SEQ):
    """Convert text sequences to padded token batches."""
    # Tokenize (byte-level)
    all_tokens = []
    for seq in sequences:
        tokens = list(seq.encode('utf-8'))[:max_seq]
        if len(tokens) >= 2:
            all_tokens.append(tokens)

    # Shuffle deterministically
    rng = torch.Generator()
    rng.manual_seed(42)
    indices = torch.randperm(len(all_tokens), generator=rng).tolist()

    batches = []
    for i in range(0, len(indices), batch_size):
        batch_indices = indices[i:i+batch_size]
        batch_seqs = [all_tokens[j] for j in batch_indices]
        max_len = max(len(s) for s in batch_seqs)
        # Pad with 0
        padded = torch.zeros(len(batch_seqs), max_len, dtype=torch.long)
        for j, s in enumerate(batch_seqs):
            padded[j, :len(s)] = torch.tensor(s, dtype=torch.long)
        batches.append(padded)
    return batches


# ════════════════════════════════════════════════════════════════
# Training
# ════════════════════════════════════════════════════════════════
def train(continue_mode=False, resume_mode=False):
    print("=" * 50)
    print("  JARVIS PyTorch Training")
    print(f"  4.4M param byte-level transformer")
    print(f"  PyTorch {torch.__version__}, MKL={torch.backends.mkl.is_available()}")
    print(f"  Threads: {torch.get_num_threads()}")
    if continue_mode:
        print("  MODE: CONTINUE (gentle LR)")
    if resume_mode:
        print("  MODE: RESUME (skip curriculum, load checkpoint)")
    print("=" * 50)

    # Load corpus
    sequences = load_corpus("_bench_corpus.txt")

    # Model
    model = JarvisTransformer()
    if resume_mode:
        try:
            load_weights(model, "jarvis_checkpoint.bin")
            print("[INIT] Loaded checkpoint weights (resume)")
        except FileNotFoundError:
            print("[WARN] No checkpoint found, starting fresh")
            resume_mode = False
    elif continue_mode:
        try:
            load_weights(model, "jarvis_pretrained.bin")
            print("[INIT] Loaded existing weights")
        except FileNotFoundError:
            print("[WARN] No weights found, starting fresh")

    print(f"[INIT] Model: {model.param_count()} params ({model.param_count()*4/1048576:.1f} MB FP32)")

    # Training config
    batch_size = 64  # PyTorch can handle much larger batches
    curriculum_epochs = [3, 2, 2, 2, 1, 1, 1, 1, 1, 1]
    consolidation_epochs = 50 if continue_mode else 20
    base_lr = 0.0002 if continue_mode else 0.001

    # Optimizer
    optimizer = torch.optim.AdamW(model.parameters(), lr=base_lr, weight_decay=0.01,
                                   betas=(0.9, 0.999), eps=1e-8)

    # Split corpus by phases (matching Rust CORPUS structure)
    # _bench_corpus.txt has Rust array syntax: each &[...] line is one phase
    import re
    phases = []
    with open("_bench_corpus.txt", 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or not line.startswith('&['):
                continue
            phase_seqs = [m.group(1) for m in re.finditer(r'"((?:[^"\\]|\\.)*)"', line)
                          if len(m.group(1)) >= 2]
            if phase_seqs:
                phases.append(phase_seqs)
    print(f"[CORPUS] {len(phases)} phases, {sum(len(p) for p in phases)} total sequences")

    # Eval before training
    model.eval()
    with torch.no_grad():
        eval_loss = 0.0
        eval_count = 0
        for phase in phases:
            text = phase[0]
            tokens = torch.tensor([list(text.encode('utf-8'))[:MAX_SEQ]], dtype=torch.long)
            if tokens.shape[1] < 2:
                continue
            logits = model(tokens)
            loss = F.cross_entropy(logits[:, :-1].reshape(-1, VOCAB_SIZE),
                                   tokens[:, 1:].reshape(-1))
            eval_loss += loss.item()
            eval_count += 1
    loss_before = eval_loss / max(eval_count, 1)
    print(f"\n[EVAL BEFORE] Loss: {loss_before:.4f}")

    t0 = time.time()
    global_step = 0

    # === Phase 1: Curriculum (skip if resuming) ===
    if resume_mode:
        print("\n--- SKIPPING CURRICULUM (resume mode) ---")
    else:
        print("\n--- CURRICULUM PHASE ---")
    if not resume_mode:
      model.train()
      for phase_idx, phase in enumerate(phases):
        n_ep = curriculum_epochs[phase_idx] if phase_idx < len(curriculum_epochs) else 1
        batches = make_batches(phase, batch_size)

        for ep in range(n_ep):
            phase_loss = 0.0
            phase_count = 0
            for batch in batches:
                if batch.shape[1] < 2:
                    continue
                optimizer.zero_grad()
                logits = model(batch)
                # Mask padding (token 0) in loss
                targets = batch[:, 1:]
                logits_flat = logits[:, :-1].reshape(-1, VOCAB_SIZE)
                targets_flat = targets.reshape(-1)
                loss = F.cross_entropy(logits_flat, targets_flat)
                loss.backward()
                torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
                optimizer.step()
                phase_loss += loss.item() * batch.shape[0]
                phase_count += batch.shape[0]
                global_step += 1

            avg = phase_loss / max(phase_count, 1)
            print(f"  Phase {phase_idx} ep {ep+1}/{n_ep}: loss={avg:.4f} ({phase_count} seqs)")

      elapsed = time.time() - t0
      print(f"  Curriculum done: {elapsed:.1f}s, {global_step} steps")

    # Mid eval
    model.eval()
    with torch.no_grad():
        mid_loss = 0.0
        mid_count = 0
        for phase in phases:
            tokens = torch.tensor([list(phase[0].encode('utf-8'))[:MAX_SEQ]], dtype=torch.long)
            if tokens.shape[1] < 2:
                continue
            logits = model(tokens)
            loss = F.cross_entropy(logits[:, :-1].reshape(-1, VOCAB_SIZE),
                                   tokens[:, 1:].reshape(-1))
            mid_loss += loss.item()
            mid_count += 1
    print(f"  [MID EVAL] Loss: {mid_loss / max(mid_count, 1):.4f}")

    # === Phase 2: Consolidation ===
    print(f"\n--- CONSOLIDATION PHASE (cosine LR, {consolidation_epochs} epochs) ---")
    all_batches = make_batches(sequences, batch_size)
    print(f"  {len(sequences)} sequences in {len(all_batches)} batches of {batch_size}")
    model.train()

    # Cosine LR scheduler
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        optimizer, T_max=consolidation_epochs, eta_min=base_lr * 0.01)

    for epoch in range(consolidation_epochs):
        epoch_loss = 0.0
        epoch_count = 0

        # Re-shuffle each epoch
        rng = torch.Generator()
        rng.manual_seed(42 + epoch)
        indices = torch.randperm(len(sequences), generator=rng).tolist()

        for i in range(0, len(indices), batch_size):
            batch_idx = indices[i:i+batch_size]
            batch_seqs = [list(sequences[j].encode('utf-8'))[:MAX_SEQ] for j in batch_idx]
            max_len = max(len(s) for s in batch_seqs)
            padded = torch.zeros(len(batch_seqs), max_len, dtype=torch.long)
            for j, s in enumerate(batch_seqs):
                padded[j, :len(s)] = torch.tensor(s, dtype=torch.long)

            if padded.shape[1] < 2:
                continue

            optimizer.zero_grad()
            logits = model(padded)
            targets = padded[:, 1:]
            logits_flat = logits[:, :-1].reshape(-1, VOCAB_SIZE)
            targets_flat = targets.reshape(-1)
            loss = F.cross_entropy(logits_flat, targets_flat)
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
            epoch_loss += loss.item() * len(batch_idx)
            epoch_count += len(batch_idx)
            global_step += 1

        scheduler.step()
        avg = epoch_loss / max(epoch_count, 1)
        lr = optimizer.param_groups[0]['lr']
        elapsed = time.time() - t0
        print(f"  Consolidation {epoch+1}/{consolidation_epochs}: avg_loss={avg:.4f} lr={lr:.6f} ({epoch_count} seqs, {elapsed:.1f}s)")

        # Checkpoint every epoch (crash-safe)
        export_weights(model, "jarvis_checkpoint.bin")
        if (epoch + 1) % 5 == 0:
            print(f"    [CHECKPOINT] Saved at epoch {epoch+1}")

    total_time = time.time() - t0
    print(f"[PRETRAIN] Total: {total_time:.1f}s, {global_step} optimizer steps")

    # Eval after
    model.eval()
    with torch.no_grad():
        eval_loss = 0.0
        eval_count = 0
        for phase in phases:
            tokens = torch.tensor([list(phase[0].encode('utf-8'))[:MAX_SEQ]], dtype=torch.long)
            if tokens.shape[1] < 2:
                continue
            logits = model(tokens)
            loss = F.cross_entropy(logits[:, :-1].reshape(-1, VOCAB_SIZE),
                                   tokens[:, 1:].reshape(-1))
            eval_loss += loss.item()
            eval_count += 1
    loss_after = eval_loss / max(eval_count, 1)

    # Chat tests
    print("\n[CHAT TESTS - GREEDY]")
    for prompt in ["Hello", "who are you", "What is TrustOS"]:
        gen = generate(model, prompt, 50)
        print(f"  > {prompt} => {gen[:80]}")

    print("\n[CHAT TESTS - TEMP=0.8 TOP_K=40]")
    for prompt in ["Hello", "who are you", "What is TrustOS", "Q: aide", "bonjour", "I am Jarvis"]:
        gen = generate(model, prompt, 60, temperature=0.8, top_k=40)
        print(f"  > {prompt} => {gen[:80]}")

    # Report
    print("\n" + "=" * 50)
    print(f"  LOSS BEFORE: {loss_before:.4f}")
    print(f"  LOSS AFTER:  {loss_after:.4f}")
    print(f"  IMPROVEMENT: {(1.0 - loss_after / loss_before) * 100:.1f}%")
    if loss_after < 2.0:
        maturity = "ADULT (Level 3) - READY FOR FULL PROPAGATION"
    elif loss_after < 3.5:
        maturity = "TEEN (Level 2) - READY FOR MESH + FEDERATED"
    elif loss_after < 5.0:
        maturity = "CHILD (Level 1) - NEEDS MORE TRAINING"
    else:
        maturity = "INFANT (Level 0) - UNTRAINED"
    print(f"  MATURITY: {maturity}")
    print(f"  TOTAL TIME: {total_time:.1f}s ({total_time/60:.1f} min)")
    print("=" * 50)

    # Export final weights
    export_weights(model, "jarvis_pretrained.bin")
    print("[EXPORT] Copy jarvis_pretrained.bin to kernel/src/jarvis/ for embedding.")


# ════════════════════════════════════════════════════════════════
# Generation
# ════════════════════════════════════════════════════════════════
@torch.no_grad()
def generate(model, prompt: str, max_tokens: int = 50, temperature: float = 0.0, top_k: int = 0):
    model.eval()
    tokens = list(prompt.encode('utf-8'))
    for _ in range(max_tokens):
        if len(tokens) >= MAX_SEQ:
            break
        x = torch.tensor([tokens], dtype=torch.long)
        logits = model(x)
        next_logits = logits[0, -1]  # last position

        if temperature > 0 and top_k > 0:
            next_logits = next_logits / temperature
            topk_vals, topk_idx = torch.topk(next_logits, min(top_k, VOCAB_SIZE))
            probs = F.softmax(topk_vals, dim=-1)
            chosen_idx = torch.multinomial(probs, 1).item()
            tok = topk_idx[chosen_idx].item()
        else:
            tok = next_logits.argmax().item()

        if tok == 0 or tok == 3:
            break
        tokens.append(tok)
    return bytes(tokens[len(prompt.encode('utf-8')):]).decode('utf-8', errors='replace')


# ════════════════════════════════════════════════════════════════
# Main
# ════════════════════════════════════════════════════════════════
if __name__ == "__main__":
    continue_mode = "--continue" in sys.argv or "-c" in sys.argv
    resume_mode = "--resume" in sys.argv or "-r" in sys.argv
    train(continue_mode=continue_mode, resume_mode=resume_mode)
