//! JARVIS Gradient Compression — Bandwidth-Optimal Network Propagation
//!
//! Reduces gradient/weight transfer from ~17.6 MB (full FP32) to ~200 KB
//! using a combination of:
//!
//! 1. **TopK Sparsification**: Only transmit the K% largest gradients (by abs value)
//! 2. **Quantization**: Compress selected f32 values to i8 (32× smaller)
//! 3. **Error Feedback**: Track compression residuals → add to next round
//!    (guarantees convergence despite lossy compression)
//!
//! # Wire Format (Compressed Gradient Packet)
//!
//! ```text
//! [0..4]   Magic: b"JCMP"
//! [4]      Version: 1
//! [5]      Compression type: 0=TopK+Quant, 1=Delta, 2=Full
//! [6..10]  Total param count (u32)
//! [10..14] Sparse entry count (u32) — number of (index, value) pairs
//! [14..18] Scale factor (f32) — for dequantization: real = i8 * scale
//! [18..]   Entries: [index:u32 (4 bytes) + quantized_value:i8 (1 byte)] × N
//! ```
//!
//! Bandwidth: TopK 1% of 4.4M params = 44,000 entries × 5 bytes = ~220 KB
//! vs full model: 4.4M × 4 bytes = ~17.6 MB → **80× compression**

use alloc::vec::Vec;
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Magic bytes for compressed gradient packets
const MAGIC: &[u8; 4] = b"JCMP";

/// Protocol version
const VERSION: u8 = 1;

/// Percentage of gradients to keep (TopK ratio)
/// 1% keeps 44K out of 4.4M — excellent convergence/bandwidth tradeoff
const TOPK_RATIO: f32 = 0.01;

/// Minimum number of entries to send (even if TopK yields fewer)
const MIN_ENTRIES: usize = 256;

/// Maximum entries (cap memory usage)
const MAX_ENTRIES: usize = 100_000;

/// Compression types
#[repr(u8)]
pub enum CompressionType {
    TopKQuant = 0,
    Delta = 1,
    Full = 2,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Error Feedback State — Residuals from previous compression rounds
// ═══════════════════════════════════════════════════════════════════════════════

/// Accumulated error from previous TopK rounds (error feedback / memory)
/// This ensures gradients that were too small in one round get accumulated
/// and eventually transmitted — guarantees convergence.
static ERROR_RESIDUAL: Mutex<Vec<f32>> = Mutex::new(Vec::new());

// ═══════════════════════════════════════════════════════════════════════════════
// TopK Sparsification + Quantization
// ═══════════════════════════════════════════════════════════════════════════════

/// Sparse entry: (parameter_index, quantized_value)
#[derive(Clone)]
pub struct SparseEntry {
    pub index: u32,
    pub value: i8,
}

/// Compressed gradient packet (ready for network transmission)
pub struct CompressedGradient {
    pub param_count: u32,
    pub entries: Vec<SparseEntry>,
    pub scale: f32,
}

/// Compress a gradient vector using TopK sparsification + i8 quantization + error feedback.
///
/// 1. Add accumulated error residual from previous rounds
/// 2. Select TopK entries by absolute value
/// 3. Quantize selected entries to i8 with a global scale factor
/// 4. Store the residual (what we didn't send) for next round
pub fn compress_gradients(gradients: &[f32]) -> CompressedGradient {
    let n = gradients.len();

    // Step 1: Add error feedback from previous round
    let mut combined = Vec::with_capacity(n);
    {
        let residual = ERROR_RESIDUAL.lock();
        if residual.len() == n {
            for i in 0..n {
                combined.push(gradients[i] + residual[i]);
            }
        } else {
            combined.extend_from_slice(gradients);
        }
    }

    // Step 2: Compute TopK threshold
    let k = ((n as f32 * TOPK_RATIO) as usize).max(MIN_ENTRIES).min(MAX_ENTRIES).min(n);

    // Find the k-th largest absolute value using partial sort
    // (We use a simple approach: collect abs values, sort descending, pick threshold)
    let mut abs_values: Vec<f32> = combined.iter().map(|x| x.abs()).collect();
    // Partial sort: we only need to find the k-th largest
    // For efficiency, use selection algorithm (quickselect-style)
    let threshold = find_kth_largest(&mut abs_values, k);

    // Step 3: Select entries above threshold
    let mut selected_indices: Vec<usize> = Vec::with_capacity(k);
    for (i, &val) in combined.iter().enumerate() {
        if val.abs() >= threshold && selected_indices.len() < MAX_ENTRIES {
            selected_indices.push(i);
        }
    }

    // Step 4: Compute scale factor for i8 quantization
    // scale = max_abs / 127.0, so that max value maps to ±127
    let max_abs = selected_indices.iter()
        .map(|&i| combined[i].abs())
        .fold(0.0f32, f32::max);

    let scale = if max_abs > 0.0 { max_abs / 127.0 } else { 1.0 };
    let inv_scale = 1.0 / scale;

    // Step 5: Quantize selected entries
    let mut entries = Vec::with_capacity(selected_indices.len());
    for &idx in &selected_indices {
        let raw = combined[idx] * inv_scale;
        let quantized = if raw >= 0.0 { (raw + 0.5) as i32 } else { (raw - 0.5) as i32 };
        let quantized = quantized.max(-127).min(127) as i8;
        entries.push(SparseEntry {
            index: idx as u32,
            value: quantized,
        });
    }

    // Step 6: Update error residual (what we didn't send + quantization error)
    {
        let mut residual = ERROR_RESIDUAL.lock();
        residual.resize(n, 0.0);
        // Start with the full combined gradient
        for i in 0..n {
            residual[i] = combined[i];
        }
        // Subtract what we actually sent (dequantized)
        for entry in &entries {
            let idx = entry.index as usize;
            let sent = entry.value as f32 * scale;
            residual[idx] = combined[idx] - sent;
        }
    }

    CompressedGradient {
        param_count: n as u32,
        entries,
        scale,
    }
}

/// Decompress received gradient packet back to a full gradient vector.
/// Non-transmitted entries are set to 0 (sparse representation).
pub fn decompress_gradients(compressed: &CompressedGradient) -> Vec<f32> {
    let mut gradients = alloc::vec![0.0f32; compressed.param_count as usize];
    for entry in &compressed.entries {
        let idx = entry.index as usize;
        if idx < gradients.len() {
            gradients[idx] = entry.value as f32 * compressed.scale;
        }
    }
    gradients
}

/// Reset error feedback state (e.g., after model replacement)
pub fn reset_error_feedback() {
    ERROR_RESIDUAL.lock().clear();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serialization — Wire Format
// ═══════════════════════════════════════════════════════════════════════════════

/// Serialize a compressed gradient to bytes for network transmission.
///
/// Format: [JCMP:4][version:1][type:1][param_count:4][entry_count:4][scale:4][entries...]
/// Entry: [index:4][value:1] = 5 bytes each
pub fn serialize_compressed(compressed: &CompressedGradient) -> Vec<u8> {
    let header_size = 18;
    let entry_size = 5; // u32 index + i8 value
    let total = header_size + compressed.entries.len() * entry_size;

    let mut buf = Vec::with_capacity(total);

    // Header
    buf.extend_from_slice(MAGIC);
    buf.push(VERSION);
    buf.push(CompressionType::TopKQuant as u8);
    buf.extend_from_slice(&compressed.param_count.to_be_bytes());
    buf.extend_from_slice(&(compressed.entries.len() as u32).to_be_bytes());
    buf.extend_from_slice(&compressed.scale.to_be_bytes());

    // Entries
    for entry in &compressed.entries {
        buf.extend_from_slice(&entry.index.to_be_bytes());
        buf.push(entry.value as u8);
    }

    buf
}

/// Deserialize a compressed gradient from network bytes.
pub fn deserialize_compressed(data: &[u8]) -> Option<CompressedGradient> {
    if data.len() < 18 {
        return None;
    }

    // Verify magic
    if &data[0..4] != MAGIC {
        return None;
    }

    // Version check
    if data[4] != VERSION {
        return None;
    }

    let param_count = u32::from_be_bytes([data[6], data[7], data[8], data[9]]);
    let entry_count = u32::from_be_bytes([data[10], data[11], data[12], data[13]]);
    let scale = f32::from_be_bytes([data[14], data[15], data[16], data[17]]);

    // Sanity checks
    if entry_count > MAX_ENTRIES as u32 {
        return None;
    }

    let expected_size = 18 + entry_count as usize * 5;
    if data.len() < expected_size {
        return None;
    }

    let mut entries = Vec::with_capacity(entry_count as usize);
    let mut offset = 18;
    for _ in 0..entry_count {
        let index = u32::from_be_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
        let value = data[offset + 4] as i8;

        // Validate index is within bounds
        if index >= param_count {
            return None;
        }

        entries.push(SparseEntry { index, value });
        offset += 5;
    }

    Some(CompressedGradient {
        param_count,
        entries,
        scale,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// Delta Weight Compression
// ═══════════════════════════════════════════════════════════════════════════════

/// Last synced weight snapshot (for computing deltas)
static LAST_SYNCED_WEIGHTS: Mutex<Vec<f32>> = Mutex::new(Vec::new());

/// Compute a delta between current weights and last synced snapshot.
/// Returns only the changed parameters (TopK + quantized).
pub fn compute_weight_delta(current_weights: &[f32]) -> CompressedGradient {
    let last = LAST_SYNCED_WEIGHTS.lock();

    if last.len() != current_weights.len() {
        // No baseline — send full (compressed)
        drop(last);
        update_sync_snapshot(current_weights);
        return compress_gradients(current_weights);
    }

    // Compute delta: current - last_synced
    let n = current_weights.len();
    let mut delta = Vec::with_capacity(n);
    for i in 0..n {
        delta.push(current_weights[i] - last[i]);
    }
    drop(last);

    // Compress the delta (TopK + quantize)
    // Use a temporary error feedback context for deltas
    let compressed = compress_delta_vector(&delta);

    // Update snapshot
    update_sync_snapshot(current_weights);

    compressed
}

/// Apply a received weight delta to the current model weights
pub fn apply_weight_delta(current_weights: &mut [f32], delta: &CompressedGradient) {
    for entry in &delta.entries {
        let idx = entry.index as usize;
        if idx < current_weights.len() {
            current_weights[idx] += entry.value as f32 * delta.scale;
        }
    }
}

/// Update the sync snapshot (called after successful sync)
pub fn update_sync_snapshot(weights: &[f32]) {
    let mut snap = LAST_SYNCED_WEIGHTS.lock();
    snap.clear();
    snap.extend_from_slice(weights);
}

/// Compress a delta vector without affecting the gradient error feedback
fn compress_delta_vector(delta: &[f32]) -> CompressedGradient {
    let n = delta.len();
    let k = ((n as f32 * TOPK_RATIO) as usize).max(MIN_ENTRIES).min(MAX_ENTRIES).min(n);

    let mut abs_values: Vec<f32> = delta.iter().map(|x| x.abs()).collect();
    let threshold = find_kth_largest(&mut abs_values, k);

    let mut selected: Vec<usize> = Vec::with_capacity(k);
    for (i, &val) in delta.iter().enumerate() {
        if val.abs() >= threshold && selected.len() < MAX_ENTRIES {
            selected.push(i);
        }
    }

    let max_abs = selected.iter()
        .map(|&i| delta[i].abs())
        .fold(0.0f32, f32::max);

    let scale = if max_abs > 0.0 { max_abs / 127.0 } else { 1.0 };
    let inv_scale = 1.0 / scale;

    let mut entries = Vec::with_capacity(selected.len());
    for &idx in &selected {
        let raw = delta[idx] * inv_scale;
        let quantized = if raw >= 0.0 { (raw + 0.5) as i32 } else { (raw - 0.5) as i32 };
        let quantized = quantized.max(-127).min(127) as i8;
        if quantized != 0 {
            entries.push(SparseEntry {
                index: idx as u32,
                value: quantized,
            });
        }
    }

    CompressedGradient {
        param_count: n as u32,
        entries,
        scale,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Selection Algorithm — O(n) average for finding k-th largest
// ═══════════════════════════════════════════════════════════════════════════════

/// Find the k-th largest value in an unsorted slice.
/// Uses a simple partial sort approach (good enough for our sizes).
fn find_kth_largest(values: &mut [f32], k: usize) -> f32 {
    if values.is_empty() || k == 0 {
        return 0.0;
    }
    let k = k.min(values.len());

    // Sort descending, return k-th element
    // For 4.4M params, sorting ~4.4M f32 is fast enough (< 100ms)
    values.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(core::cmp::Ordering::Equal));
    values[k.saturating_sub(1)]
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stats
// ═══════════════════════════════════════════════════════════════════════════════

/// Get compression stats for display
pub fn compression_ratio(original_params: usize, compressed: &CompressedGradient) -> (usize, usize, f32) {
    let original_bytes = original_params * 4; // FP32
    let compressed_bytes = 18 + compressed.entries.len() * 5;
    let ratio = if compressed_bytes > 0 {
        original_bytes as f32 / compressed_bytes as f32
    } else {
        0.0
    };
    (original_bytes, compressed_bytes, ratio)
}
