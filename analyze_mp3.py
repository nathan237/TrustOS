import librosa
import numpy as np
import os

dl = os.path.expanduser("~/Downloads")

for fname in ["Untitled (1).mp3", "Real Life Riches.mp3"]:
    print(f"=== {fname} ===")
    p = os.path.join(dl, fname)
    y, sr = librosa.load(p, sr=None, mono=True)
    dur = len(y) / sr
    print(f"  Duration: {dur:.1f}s  SampleRate: {sr}Hz")

    # BPM
    tempo, beats = librosa.beat.beat_track(y=y, sr=sr)
    tempo_val = float(np.mean(tempo)) if hasattr(tempo, "__len__") else float(tempo)
    print(f"  BPM: {tempo_val:.1f}")

    # Beat times for structure analysis
    beat_times = librosa.frames_to_time(beats, sr=sr)
    print(f"  Beats detected: {len(beat_times)}")
    if len(beat_times) > 4:
        print(f"  First beat: {beat_times[0]:.2f}s  Last: {beat_times[-1]:.2f}s")

    # Key detection
    chroma = librosa.feature.chroma_cqt(y=y, sr=sr)
    chroma_mean = np.mean(chroma, axis=1)
    keys = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"]
    root = int(np.argmax(chroma_mean))
    major_third = chroma_mean[(root + 4) % 12]
    minor_third = chroma_mean[(root + 3) % 12]
    mode = "major" if major_third > minor_third else "minor"
    print(f"  Key: {keys[root]} {mode}")

    # Print full chroma profile
    ranked = sorted(range(12), key=lambda i: chroma_mean[i], reverse=True)
    print(f"  Top notes: {', '.join(f'{keys[i]}={chroma_mean[i]:.3f}' for i in ranked[:6])}")

    # Onset strength for section boundary detection
    onset_env = librosa.onset.onset_strength(y=y, sr=sr)
    # Segment using structural analysis
    try:
        bounds = librosa.segment.agglomerative(chroma, k=8)
        bound_times = librosa.frames_to_time(bounds, sr=sr)
        print(f"  Section boundaries (~8 segments): {[f'{t:.1f}s' for t in bound_times]}")
    except Exception as e:
        print(f"  Section detection error: {e}")

    # RMS energy profile (8 segments)
    rms = librosa.feature.rms(y=y)[0]
    seg_len = len(rms) // 8
    for seg in range(8):
        start_t = seg * dur / 8
        end_t = (seg + 1) * dur / 8
        seg_rms = np.mean(rms[seg * seg_len:(seg + 1) * seg_len])
        bar = "#" * int(seg_rms * 200)
        print(f"  [{start_t:5.1f}s-{end_t:5.1f}s] energy={seg_rms:.4f} {bar}")

    print()
