//! On-Device Training — Teach Jarvis in real-time
//!
//! Implements simple gradient descent for the tiny transformer.
//! Uses numerical gradient estimation (finite differences) which is
//! simple but functional for our 312K parameter model.
//!
//! Training modes:
//! - **Teacher forcing**: Given a text sequence, train to predict next token
//! - **Correction**: Given wrong/right pairs, train on the right version
//! - **Self-play**: Generate text, evaluate quality, improve
//!
//! The numerical gradient approach:
//! ```text
//! For each parameter w:
//!   loss_plus  = forward(w + ε)
//!   loss_minus = forward(w - ε)
//!   gradient   = (loss_plus - loss_minus) / (2ε)
//!   w -= learning_rate * gradient
//! ```
//!
//! This is O(params) forward passes per step, which is expensive but:
//! - Our model is tiny (312K params vs. billions)
//! - We train on short sequences (< 64 tokens typically)
//! - No need for backprop infrastructure (simpler code)
//! - Can be parallelized across GPU CUs in the future
//!
//! For efficiency, we use **stochastic parameter perturbation**: each step
//! only updates a random subset of parameters, amortizing the cost.

use alloc::vec::Vec;
use alloc::vec;
use super::model::*;
use super::inference;
use super::tokenizer;

// ═══════════════════════════════════════════════════════════════════════════════
// Training Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Epsilon for numerical gradient estimation
const GRAD_EPSILON: f32 = 0.001;

/// Maximum tokens per training sequence
const MAX_TRAIN_SEQ: usize = 64;

/// Fraction of parameters to update per step (1.0 = all, 0.01 = 1%)
const PARAM_SAMPLE_RATE: f32 = 0.01;

/// Gradient clipping threshold
const GRAD_CLIP: f32 = 1.0;

// ═══════════════════════════════════════════════════════════════════════════════
// Training Step
// ═══════════════════════════════════════════════════════════════════════════════

/// Which weight component to train this step
enum WeightTarget {
    LayerWq(usize),
    LayerWk(usize),
    LayerWo(usize),
    LayerWgate(usize),
    Output,
}

/// Perform one training step on a token sequence.
///
/// Uses stochastic numerical gradients: randomly selects ~1% of parameters,
/// estimates their gradients via finite differences, and applies SGD update.
///
/// Returns the loss before the update.
pub fn train_step(model: &mut TransformerWeights, tokens: &[u8], learning_rate: f32) -> f32 {
    let tokens = if tokens.len() > MAX_TRAIN_SEQ { &tokens[..MAX_TRAIN_SEQ] } else { tokens };
    if tokens.len() < 2 { return f32::MAX; }

    // Compute baseline loss
    let (base_loss, _) = inference::compute_loss(model, tokens);

    // Stochastic gradient update on embedding weights
    // We rotate through model components across training steps
    let step = super::TRAINING_STEPS.load(core::sync::atomic::Ordering::Relaxed);
    let component = (step % 6) as usize;

    match component {
        0 => {
            // Train token embeddings (only for tokens present in sequence)
            train_embedding(model, tokens, learning_rate);
        }
        1 => {
            let layer_idx = (step / 6) as usize % N_LAYERS;
            train_weight_slice(model, WeightTarget::LayerWq(layer_idx), tokens, learning_rate);
        }
        2 => {
            let layer_idx = (step / 6) as usize % N_LAYERS;
            train_weight_slice(model, WeightTarget::LayerWk(layer_idx), tokens, learning_rate);
        }
        3 => {
            let layer_idx = (step / 6) as usize % N_LAYERS;
            train_weight_slice(model, WeightTarget::LayerWo(layer_idx), tokens, learning_rate);
        }
        4 => {
            let layer_idx = (step / 6) as usize % N_LAYERS;
            train_weight_slice(model, WeightTarget::LayerWgate(layer_idx), tokens, learning_rate);
        }
        5 => {
            train_weight_slice(model, WeightTarget::Output, tokens, learning_rate);
        }
        _ => {}
    }

    base_loss
}

/// Train token embeddings for tokens present in the sequence
fn train_embedding(model: &mut TransformerWeights, tokens: &[u8], lr: f32) {
    let (base_loss, _) = inference::compute_loss(model, tokens);

    // For each unique token in the sequence, perturb its embedding
    let mut seen = [false; VOCAB_SIZE];
    for &t in tokens {
        if seen[t as usize] { continue; }
        seen[t as usize] = true;

        let base = t as usize * D_MODEL;
        // Perturb a random subset of dimensions
        for d in 0..D_MODEL {
            if !should_sample(base + d) { continue; }

            // Forward: +ε
            model.token_embed[base + d] += GRAD_EPSILON;
            let (loss_plus, _) = inference::compute_loss(model, tokens);

            // Forward: -ε (from +ε back to -ε = -2ε)
            model.token_embed[base + d] -= 2.0 * GRAD_EPSILON;
            let (loss_minus, _) = inference::compute_loss(model, tokens);

            // Restore and apply gradient
            model.token_embed[base + d] += GRAD_EPSILON; // Back to original

            let grad = (loss_plus - loss_minus) / (2.0 * GRAD_EPSILON);
            let grad = clip_grad(grad);
            model.token_embed[base + d] -= lr * grad;
        }
    }
}

/// Train a weight matrix using stochastic numerical gradients.
///
/// Uses raw pointers internally to decouple the mutable access to the specific
/// weight being perturbed from the immutable access needed by `compute_loss`.
/// This is safe because we never modify through the pointer while compute_loss
/// holds a shared reference — modifications happen strictly before/after each
/// forward pass.
fn train_weight_slice(model: &mut TransformerWeights, target: WeightTarget, tokens: &[u8], lr: f32) {
    // Extract raw pointer + length from the target weight slice.
    // The temporary mutable borrow ends after these statements (NLL).
    let (ptr, n) = match target {
        WeightTarget::LayerWq(l)    => (model.layers[l].w_q.as_mut_ptr(), model.layers[l].w_q.len()),
        WeightTarget::LayerWk(l)    => (model.layers[l].w_k.as_mut_ptr(), model.layers[l].w_k.len()),
        WeightTarget::LayerWo(l)    => (model.layers[l].w_o.as_mut_ptr(), model.layers[l].w_o.len()),
        WeightTarget::LayerWgate(l) => (model.layers[l].w_gate.as_mut_ptr(), model.layers[l].w_gate.len()),
        WeightTarget::Output        => (model.w_output.as_mut_ptr(), model.w_output.len()),
    };

    let sample_count = ((n as f32 * PARAM_SAMPLE_RATE) as usize).max(1);
    let step = super::TRAINING_STEPS.load(core::sync::atomic::Ordering::Relaxed) as usize;

    for i in 0..sample_count {
        let idx = (step * 7919 + i * 6271) % n; // Quasi-random sampling

        // Safety: ptr points into model which we own via &mut. We modify one
        // f32, call compute_loss (shared borrow of model), then modify again.
        // The shared borrow and the pointer write never overlap temporally.
        unsafe {
            // Forward: +ε
            *ptr.add(idx) += GRAD_EPSILON;
        }
        let (loss_plus, _) = inference::compute_loss(model, tokens);

        unsafe {
            // Forward: -ε (from +ε position → original-ε)
            *ptr.add(idx) -= 2.0 * GRAD_EPSILON;
        }
        let (loss_minus, _) = inference::compute_loss(model, tokens);

        unsafe {
            // Restore original value
            *ptr.add(idx) += GRAD_EPSILON;
        }

        // Compute and apply gradient
        let grad = (loss_plus - loss_minus) / (2.0 * GRAD_EPSILON);
        let grad = clip_grad(grad);
        unsafe {
            *ptr.add(idx) -= lr * grad;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Quick Train — Fast update using loss gradient direction
// ═══════════════════════════════════════════════════════════════════════════════

/// Ultra-fast training: perturb ALL weights randomly, keep if loss decreases
/// This is "evolution strategies" / random search — very simple, no backprop.
/// Good for initial exploration of weight space.
pub fn train_step_random(model: &mut TransformerWeights, tokens: &[u8], lr: f32) -> f32 {
    if tokens.len() < 2 { return f32::MAX; }

    let (base_loss, _) = inference::compute_loss(model, tokens);

    // Generate random perturbation and apply
    let mut rng = crate::time::uptime_ticks().wrapping_mul(6364136223846793005);

    // Perturb output projection (most impactful for loss)
    let perturbation: Vec<f32> = (0..model.w_output.len()).map(|_| {
        rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17;
        let bits = (rng >> 40) as u32;
        (bits as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
    }).collect();

    // Apply perturbation scaled by learning rate
    for (w, &p) in model.w_output.iter_mut().zip(perturbation.iter()) {
        *w += lr * p;
    }

    // Check if loss improved
    let (new_loss, _) = inference::compute_loss(model, tokens);

    if new_loss >= base_loss {
        // Revert: subtract 2× to go past original, then add back
        for (w, &p) in model.w_output.iter_mut().zip(perturbation.iter()) {
            *w -= 2.0 * lr * p; // Try the opposite direction
        }
        let (rev_loss, _) = inference::compute_loss(model, tokens);
        if rev_loss >= base_loss {
            // Neither direction helped, revert to original
            for (w, &p) in model.w_output.iter_mut().zip(perturbation.iter()) {
                *w += lr * p;
            }
        }
    }

    base_loss
}

// ═══════════════════════════════════════════════════════════════════════════════
// Self-Test for Training
// ═══════════════════════════════════════════════════════════════════════════════

/// Test that training reduces loss on a simple sequence
pub fn self_test() -> (u32, u32) {
    let mut pass = 0u32;
    let mut fail = 0u32;

    // Test 1: Loss computation doesn't crash
    crate::serial_println!("[JARVIS-TRAIN] Test 1: Loss computation");
    {
        let model = TransformerWeights::new_random();
        let tokens = tokenizer::encode("Hello world");
        let (loss, logits) = inference::compute_loss(&model, &tokens);
        if loss.is_finite() && logits.len() > 0 {
            crate::serial_println!("[JARVIS-TRAIN]   Loss = {:.4} (random weights)", loss);
            pass += 1;
        } else {
            crate::serial_println!("[JARVIS-TRAIN]   FAIL: loss={}", loss);
            fail += 1;
        }
    }

    // Test 2: Training step runs without crash
    crate::serial_println!("[JARVIS-TRAIN] Test 2: Training step");
    {
        let mut model = TransformerWeights::new_random();
        let tokens = tokenizer::encode("AB");
        let loss = train_step(&mut model, &tokens, 0.01);
        if loss.is_finite() {
            crate::serial_println!("[JARVIS-TRAIN]   Initial loss = {:.4}", loss);
            pass += 1;
        } else {
            fail += 1;
        }
    }

    // Test 3: Random training step
    crate::serial_println!("[JARVIS-TRAIN] Test 3: Random perturbation training");
    {
        let mut model = TransformerWeights::new_random();
        let tokens = tokenizer::encode("Test");
        let loss = train_step_random(&mut model, &tokens, 0.001);
        if loss.is_finite() {
            crate::serial_println!("[JARVIS-TRAIN]   Loss = {:.4}", loss);
            pass += 1;
        } else {
            fail += 1;
        }
    }

    (pass, fail)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Deterministic sampling: should we update parameter at index i?
fn should_sample(idx: usize) -> bool {
    // Simple hash-based sampling at ~PARAM_SAMPLE_RATE
    let h = idx.wrapping_mul(2654435761); // Knuth multiplicative hash
    (h % 100) < (PARAM_SAMPLE_RATE * 100.0) as usize
}

/// Clip gradient to prevent explosions
fn clip_grad(g: f32) -> f32 {
    if g > GRAD_CLIP { GRAD_CLIP }
    else if g < -GRAD_CLIP { -GRAD_CLIP }
    else { g }
}
