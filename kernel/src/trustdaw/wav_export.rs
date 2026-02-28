//! WAV File Export for TrustDAW
//!
//! Exports rendered audio as standard RIFF/WAV files (PCM 16-bit stereo).
//! Writes to the TrustOS VFS for saving to disk or RAM filesystem.

use alloc::vec::Vec;
use alloc::vec;

/// WAV file header size (44 bytes for standard PCM)
const WAV_HEADER_SIZE: usize = 44;

/// Generate a complete WAV file as a byte buffer
/// - `samples`: stereo interleaved i16 PCM data
/// - `sample_rate`: e.g., 48000
/// - `channels`: 1 (mono) or 2 (stereo)
pub fn generate_wav(samples: &[i16], sample_rate: u32, channels: u16) -> Vec<u8> {
    let data_size = samples.len() * 2; // 2 bytes per i16
    let file_size = WAV_HEADER_SIZE + data_size;

    let mut buffer = vec![0u8; file_size];

    // ─── RIFF Header ───────────────────────────────────────────────────
    // "RIFF"
    buffer[0] = b'R';
    buffer[1] = b'I';
    buffer[2] = b'F';
    buffer[3] = b'F';
    // File size minus 8 bytes (RIFF + size field)
    let chunk_size = (file_size - 8) as u32;
    buffer[4..8].copy_from_slice(&chunk_size.to_le_bytes());
    // "WAVE"
    buffer[8] = b'W';
    buffer[9] = b'A';
    buffer[10] = b'V';
    buffer[11] = b'E';

    // ─── fmt sub-chunk ─────────────────────────────────────────────────
    // "fmt "
    buffer[12] = b'f';
    buffer[13] = b'm';
    buffer[14] = b't';
    buffer[15] = b' ';
    // Sub-chunk size = 16 (for PCM)
    buffer[16..20].copy_from_slice(&16u32.to_le_bytes());
    // Audio format = 1 (PCM)
    buffer[20..22].copy_from_slice(&1u16.to_le_bytes());
    // Number of channels
    buffer[22..24].copy_from_slice(&channels.to_le_bytes());
    // Sample rate
    buffer[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    // Byte rate = sample_rate * channels * bits_per_sample / 8
    let byte_rate = sample_rate * channels as u32 * 2; // 2 bytes per sample
    buffer[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    // Block align = channels * bits_per_sample / 8
    let block_align = channels * 2;
    buffer[32..34].copy_from_slice(&block_align.to_le_bytes());
    // Bits per sample = 16
    buffer[34..36].copy_from_slice(&16u16.to_le_bytes());

    // ─── data sub-chunk ────────────────────────────────────────────────
    // "data"
    buffer[36] = b'd';
    buffer[37] = b'a';
    buffer[38] = b't';
    buffer[39] = b'a';
    // Data size
    buffer[40..44].copy_from_slice(&(data_size as u32).to_le_bytes());

    // ─── PCM data ──────────────────────────────────────────────────────
    for (i, &sample) in samples.iter().enumerate() {
        let offset = WAV_HEADER_SIZE + i * 2;
        buffer[offset..offset + 2].copy_from_slice(&sample.to_le_bytes());
    }

    buffer
}

/// Export rendered project audio to a WAV file in the VFS
pub fn export_wav(path: &str, samples: &[i16], sample_rate: u32, channels: u16) -> Result<usize, &'static str> {
    if samples.is_empty() {
        return Err("No audio data to export");
    }

    let wav_data = generate_wav(samples, sample_rate, channels);
    let size = wav_data.len();

    // Write to VFS
    crate::vfs::write_file(path, &wav_data)
        .map_err(|_| "Failed to write WAV file to VFS")?;

    crate::serial_println!("[TRUSTDAW] Exported WAV: {} ({} bytes, {} samples, {}Hz {}ch)",
        path, size, samples.len(), sample_rate, channels);

    Ok(size)
}

/// Calculate WAV file size for a given number of stereo samples
pub fn estimated_file_size(num_stereo_samples: usize) -> usize {
    WAV_HEADER_SIZE + num_stereo_samples * 2 // 2 bytes per i16
}

/// Get human-readable duration info
pub fn duration_info(num_samples: usize, sample_rate: u32, channels: u16) -> (u32, u32) {
    // Returns (seconds, milliseconds_remainder)
    let frames = num_samples / channels as usize;
    let total_ms = (frames as u64 * 1000) / sample_rate as u64;
    let seconds = (total_ms / 1000) as u32;
    let ms = (total_ms % 1000) as u32;
    (seconds, ms)
}
