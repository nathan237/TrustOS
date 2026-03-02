"""
Deep analysis of 'Untitled (2).mp3' — the cyberpunk Suno reference track
"""
import librosa
import numpy as np
import os
from mutagen.mp3 import MP3
from mutagen import File as MutagenFile

dl = os.path.expanduser("~/Downloads")
fname = "Untitled (2).mp3"
p = os.path.join(dl, fname)

# === Metadata ===
print(f"=== METADATA: {fname} ===\n")
mp3 = MP3(p)
print(f"Duration: {mp3.info.length:.1f}s ({mp3.info.length/60:.1f}min)")
print(f"Bitrate: {mp3.info.bitrate//1000}kbps  SR: {mp3.info.sample_rate}Hz  Channels: {mp3.info.channels}")

mf = MutagenFile(p)
if mf and mf.tags:
    for key in mf.tags:
        val = str(mf.tags[key])
        if len(val) < 300:
            print(f"  {key}: {val}")
print()

# === Load audio ===
print("Loading audio...")
y, sr = librosa.load(p, sr=None, mono=True)
dur = len(y) / sr
print(f"Loaded: {dur:.1f}s at {sr}Hz\n")

# === BPM ===
tempo, beats = librosa.beat.beat_track(y=y, sr=sr)
tempo_val = float(np.mean(tempo)) if hasattr(tempo, "__len__") else float(tempo)
print(f"BPM: {tempo_val:.1f}")
if tempo_val > 160:
    print(f"  Half-time: {tempo_val/2:.1f}")
if tempo_val < 100:
    print(f"  Double-time: {tempo_val*2:.1f}")
print()

# === Key (Krumhansl-Schmuckler) ===
chroma = librosa.feature.chroma_cqt(y=y, sr=sr)
chroma_mean = np.mean(chroma, axis=1)
keys = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"]

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

print(f"Key: {best_key} (r={best_corr:.3f})")
# Top chroma
top_idx = sorted(range(12), key=lambda i: chroma_mean[i], reverse=True)
print(f"Top notes: {', '.join(f'{keys[i]}({chroma_mean[i]:.2f})' for i in top_idx[:6])}")
print()

# === Section-by-section (8s windows) ===
window_sec = 8
n_windows = int(dur / window_sec) + 1
print(f"=== SECTION ANALYSIS (every {window_sec}s) ===\n")

for w in range(n_windows):
    t0 = w * window_sec
    t1 = min((w + 1) * window_sec, dur)
    if t1 - t0 < 1:
        break
    
    s0 = int(t0 * sr)
    s1 = int(t1 * sr)
    seg = y[s0:s1]
    
    seg_rms = float(np.sqrt(np.mean(seg ** 2)))
    seg_centroid = float(np.mean(librosa.feature.spectral_centroid(y=seg, sr=sr)))
    
    fft = np.abs(np.fft.rfft(seg))
    freqs = np.fft.rfftfreq(len(seg), 1.0/sr)
    fft[:5] = 0
    dom_idx = int(np.argmax(fft))
    dom_freq = freqs[dom_idx]
    
    sub_mask = (freqs >= 20) & (freqs <= 100)
    sub_energy = float(np.mean(fft[sub_mask])) if np.any(sub_mask) else 0
    bass_mask = (freqs >= 100) & (freqs <= 300)
    bass_energy = float(np.mean(fft[bass_mask])) if np.any(bass_mask) else 0
    mid_mask = (freqs >= 300) & (freqs <= 3000)
    mid_energy = float(np.mean(fft[mid_mask])) if np.any(mid_mask) else 0
    hi_mask = (freqs >= 3000) & (freqs <= 15000)
    hi_energy = float(np.mean(fft[hi_mask])) if np.any(hi_mask) else 0
    
    seg_chroma = librosa.feature.chroma_cqt(y=seg, sr=sr)
    seg_cm = np.mean(seg_chroma, axis=1)
    top3 = sorted(range(12), key=lambda i: seg_cm[i], reverse=True)[:3]
    top_notes = ", ".join(f"{keys[i]}" for i in top3)
    
    onsets = librosa.onset.onset_detect(y=seg, sr=sr)
    
    bar = "#" * int(seg_rms * 120)
    print(f"[{t0:5.1f}s - {t1:5.1f}s]  RMS={seg_rms:.4f} {bar}")
    print(f"  Centroid: {seg_centroid:.0f}Hz  DomFreq: {dom_freq:.1f}Hz  Onsets: {len(onsets)}")
    print(f"  Sub: {sub_energy:.1f}  Bass: {bass_energy:.1f}  Mid: {mid_energy:.1f}  Hi: {hi_energy:.1f}")
    print(f"  Notes: {top_notes}")
    print()

# === Beat intervals ===
print("=== RHYTHM ===\n")
beat_times = librosa.frames_to_time(beats, sr=sr)
if len(beat_times) > 2:
    ibis = np.diff(beat_times)
    print(f"Beat intervals: mean={np.mean(ibis)*1000:.1f}ms  std={np.std(ibis)*1000:.1f}ms")
    print(f"BPM from intervals: {60.0/np.mean(ibis):.1f}")
print()

# === Structure boundaries ===
print("=== STRUCTURE ===\n")
try:
    bounds = librosa.segment.agglomerative(chroma, k=10)
    bound_times = librosa.frames_to_time(bounds, sr=sr)
    rms = librosa.feature.rms(y=y)[0]
    for i, bt in enumerate(bound_times):
        frame_idx = int(bt * sr / 512)
        e = rms[frame_idx] if frame_idx < len(rms) else 0
        print(f"  Boundary {i+1}: {bt:.1f}s  (energy={e:.4f})")
except Exception as e:
    print(f"  Error: {e}")

# === Overall spectrum ===
print("\n=== FREQUENCY SPECTRUM ===\n")
fft_full = np.abs(np.fft.rfft(y))
freqs_full = np.fft.rfftfreq(len(y), 1.0/sr)
for band_name, lo, hi in [("Sub-bass", 20, 80), ("Bass", 80, 250), ("Low-mid", 250, 500), 
                            ("Mid", 500, 2000), ("Upper-mid", 2000, 5000), ("Treble", 5000, 15000)]:
    mask = (freqs_full >= lo) & (freqs_full <= hi)
    if np.any(mask):
        band_fft = fft_full[mask]
        band_freqs = freqs_full[mask]
        peak_idx = int(np.argmax(band_fft))
        peak_freq = band_freqs[peak_idx]
        avg_amp = float(np.mean(band_fft))
        bar = "#" * int(avg_amp / 300)
        print(f"  {band_name:12s} ({lo:5d}-{hi:5d}Hz): peak@{peak_freq:.0f}Hz  avg={avg_amp:.0f} {bar}")

# === Onset strength for rhythm texture ===
print("\n=== ONSET DENSITY PER SECTION ===\n")
onset_env = librosa.onset.onset_strength(y=y, sr=sr)
for w in range(n_windows):
    t0 = w * window_sec
    t1 = min((w + 1) * window_sec, dur)
    if t1 - t0 < 1:
        break
    f0 = librosa.time_to_frames(t0, sr=sr)
    f1 = librosa.time_to_frames(t1, sr=sr)
    seg_onset = onset_env[f0:f1]
    avg_strength = float(np.mean(seg_onset)) if len(seg_onset) > 0 else 0
    bar = "#" * int(avg_strength * 8)
    print(f"  [{t0:5.1f}s-{t1:5.1f}s] onset_str={avg_strength:.2f} {bar}")
