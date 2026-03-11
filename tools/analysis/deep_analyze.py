"""
Deep analysis of 'Untitled (1).mp3' — the Suno reference track
Break down by time segments: what happens when, what frequencies dominate,
energy curve, beat pattern, and tonal content.
"""
import librosa
import numpy as np
import os

dl = os.path.expanduser("~/Downloads")
fname = "Untitled (1).mp3"
p = os.path.join(dl, fname)

print(f"Loading {fname}...")
y, sr = librosa.load(p, sr=None, mono=True)
dur = len(y) / sr
print(f"Duration: {dur:.1f}s  SR: {sr}Hz\n")

# === BPM (more precise) ===
tempo, beats = librosa.beat.beat_track(y=y, sr=sr)
tempo_val = float(np.mean(tempo)) if hasattr(tempo, "__len__") else float(tempo)
print(f"Detected BPM: {tempo_val:.1f}")
# Check if half-time makes sense
if tempo_val > 160:
    print(f"  -> Likely half-time feel: {tempo_val/2:.1f} BPM")
print()

# === Key analysis with Krumhansl-Schmuckler ===
chroma = librosa.feature.chroma_cqt(y=y, sr=sr)
chroma_mean = np.mean(chroma, axis=1)
keys = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"]

# K-S major and minor profiles
major_profile = np.array([6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88])
minor_profile = np.array([6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17])

best_key = ""
best_corr = -2
for i in range(12):
    rolled = np.roll(chroma_mean, -i)
    c_maj = float(np.corrcoef(rolled, major_profile)[0, 1])
    c_min = float(np.corrcoef(rolled, minor_profile)[0, 1])
    if c_maj > best_corr:
        best_corr = c_maj
        best_key = f"{keys[i]} major"
    if c_min > best_corr:
        best_corr = c_min
        best_key = f"{keys[i]} minor"

print(f"Key (Krumhansl-Schmuckler): {best_key} (r={best_corr:.3f})")
print()

# === Spectral centroid over time (brightness) ===
centroid = librosa.feature.spectral_centroid(y=y, sr=sr)[0]
times_c = librosa.times_like(centroid, sr=sr)

# === RMS energy over time ===
rms = librosa.feature.rms(y=y)[0]
times_r = librosa.times_like(rms, sr=sr)

# === Section-by-section deep analysis ===
# Split into ~10s windows
window_sec = 10
n_windows = int(dur / window_sec) + 1
print(f"=== SECTION-BY-SECTION ANALYSIS (every {window_sec}s) ===\n")

for w in range(n_windows):
    t0 = w * window_sec
    t1 = min((w + 1) * window_sec, dur)
    if t1 - t0 < 1:
        break
    
    s0 = int(t0 * sr)
    s1 = int(t1 * sr)
    seg = y[s0:s1]
    
    # RMS
    seg_rms = float(np.sqrt(np.mean(seg ** 2)))
    
    # Spectral centroid
    seg_centroid = float(np.mean(librosa.feature.spectral_centroid(y=seg, sr=sr)))
    
    # Dominant frequency (via FFT)
    fft = np.abs(np.fft.rfft(seg))
    freqs = np.fft.rfftfreq(len(seg), 1.0/sr)
    # Ignore DC and very low noise
    fft[:5] = 0
    dom_idx = int(np.argmax(fft))
    dom_freq = freqs[dom_idx]
    
    # Sub-bass energy (20-100 Hz)
    sub_mask = (freqs >= 20) & (freqs <= 100)
    sub_energy = float(np.mean(fft[sub_mask])) if np.any(sub_mask) else 0
    
    # Bass energy (100-300 Hz)
    bass_mask = (freqs >= 100) & (freqs <= 300)
    bass_energy = float(np.mean(fft[bass_mask])) if np.any(bass_mask) else 0
    
    # Mid energy (300-3000 Hz)
    mid_mask = (freqs >= 300) & (freqs <= 3000)
    mid_energy = float(np.mean(fft[mid_mask])) if np.any(mid_mask) else 0
    
    # High energy (3000-15000 Hz)
    hi_mask = (freqs >= 3000) & (freqs <= 15000)
    hi_energy = float(np.mean(fft[hi_mask])) if np.any(hi_mask) else 0
    
    # Chroma for this segment
    seg_chroma = librosa.feature.chroma_cqt(y=seg, sr=sr)
    seg_cm = np.mean(seg_chroma, axis=1)
    top3 = sorted(range(12), key=lambda i: seg_cm[i], reverse=True)[:3]
    top_notes = ", ".join(f"{keys[i]}" for i in top3)
    
    # Onset count (percussive events)
    onsets = librosa.onset.onset_detect(y=seg, sr=sr)
    
    bar = "#" * int(seg_rms * 150)
    print(f"[{t0:5.1f}s - {t1:5.1f}s]  RMS={seg_rms:.4f} {bar}")
    print(f"  Centroid: {seg_centroid:.0f}Hz  DomFreq: {dom_freq:.1f}Hz  Onsets: {len(onsets)}")
    print(f"  Sub(20-100): {sub_energy:.1f}  Bass(100-300): {bass_energy:.1f}  Mid(300-3k): {mid_energy:.1f}  Hi(3k-15k): {hi_energy:.1f}")
    print(f"  Top notes: {top_notes}")
    print()

# === Onset strength pattern (for rhythm analysis) ===
print("=== BEAT PATTERN ANALYSIS ===\n")
onset_env = librosa.onset.onset_strength(y=y, sr=sr)
beat_frames = beats
beat_times = librosa.frames_to_time(beat_frames, sr=sr)

# Look at inter-beat intervals
if len(beat_times) > 2:
    ibis = np.diff(beat_times)
    print(f"Beat intervals: mean={np.mean(ibis)*1000:.1f}ms  std={np.std(ibis)*1000:.1f}ms")
    print(f"  -> BPM from intervals: {60.0/np.mean(ibis):.1f}")
    print()

# === Structure detection (more precise) ===
print("=== STRUCTURE BOUNDARIES ===\n")
try:
    # Use recurrence-based segmentation
    bounds = librosa.segment.agglomerative(chroma, k=12)
    bound_times = librosa.frames_to_time(bounds, sr=sr)
    for i, bt in enumerate(bound_times):
        # Get energy at this boundary
        frame_idx = int(bt * sr / 512)
        if frame_idx < len(rms):
            e = rms[frame_idx]
        else:
            e = 0
        print(f"  Boundary {i+1}: {bt:.1f}s  (energy={e:.4f})")
except Exception as e:
    print(f"  Error: {e}")

print("\n=== FREQUENCY SPECTRUM SUMMARY ===\n")
# Overall spectral shape
fft_full = np.abs(np.fft.rfft(y))
freqs_full = np.fft.rfftfreq(len(y), 1.0/sr)
# Find peaks
for band_name, lo, hi in [("Sub-bass", 20, 80), ("Bass", 80, 250), ("Low-mid", 250, 500), 
                            ("Mid", 500, 2000), ("Upper-mid", 2000, 5000), ("Treble", 5000, 15000)]:
    mask = (freqs_full >= lo) & (freqs_full <= hi)
    if np.any(mask):
        band_fft = fft_full[mask]
        band_freqs = freqs_full[mask]
        peak_idx = int(np.argmax(band_fft))
        peak_freq = band_freqs[peak_idx]
        peak_amp = band_fft[peak_idx]
        avg_amp = float(np.mean(band_fft))
        bar = "#" * int(avg_amp / 500)
        print(f"  {band_name:12s} ({lo:5d}-{hi:5d}Hz): peak@{peak_freq:.0f}Hz  avg={avg_amp:.0f} {bar}")
