"""
TrustOS Showcase Video Editor v2
────────────────────────────────
  ● Crops ONLY the OS framebuffer region (detects the distinct rectangle)
  ● Keeps music/audio CONTINUOUS (never cuts audio)
  ● Zoom + freeze-frame on loud/energetic music moments
  ● Glitch + flash impact effects synced to peaks
  ● Cinematic color grading + vignette + scanlines
  ● Intro title card + outro
  
Usage:
  python make_showcase_video.py                    # auto-detect latest video
  python make_showcase_video.py -v video.mp4       # specify video
  python make_showcase_video.py -v video.mp4 -d 40 # custom duration
"""

import argparse
import os
import sys
import math
import numpy as np
from pathlib import Path

# ═══════════════════════════════════════════════════════════════
# Configuration
# ═══════════════════════════════════════════════════════════════

DEFAULT_OUTPUT = "trustos_showcase.mp4"
FINAL_RES = (1920, 1080)
TARGET_DURATION = 38
FPS = 30

GREEN = (0, 255, 100)
DARK_GREEN = (0, 180, 60)


# ═══════════════════════════════════════════════════════════════
# Smart OS Region Detection
# ═══════════════════════════════════════════════════════════════

def detect_os_region(video_path, num_samples=8):
    """
    Detect the OS framebuffer rectangle inside a VirtualBox recording.
    Looks for a distinct bright rectangular region surrounded by dark chrome.
    Uses edge detection to find the sharp boundary of the OS display.
    """
    from moviepy import VideoFileClip
    
    clip = VideoFileClip(video_path)
    duration = clip.duration
    fw, fh = clip.size  # full frame size
    
    # Sample frames from middle of video (where desktop is likely visible)
    times = [duration * (i + 2) / (num_samples + 3) for i in range(num_samples)]
    
    # Accumulate brightness across samples
    accumulated = np.zeros((fh, fw), dtype=np.float64)
    
    for t in times:
        frame = clip.get_frame(min(t, duration - 0.1))
        gray = np.mean(frame, axis=2)
        accumulated += gray
    
    accumulated /= num_samples
    
    clip.close()
    
    # Row and column brightness profiles
    row_profile = np.mean(accumulated, axis=1)
    col_profile = np.mean(accumulated, axis=0)
    
    # Use gradient (derivative) to find sharp edges of OS framebuffer
    row_grad = np.abs(np.gradient(row_profile))
    col_grad = np.abs(np.gradient(col_profile))
    
    # Top edge: first big gradient spike from top
    row_grad_threshold = np.percentile(row_grad, 90)
    col_grad_threshold = np.percentile(col_grad, 90)
    
    y_min = 0
    y_max = fh - 1
    x_min = 0
    x_max = fw - 1
    
    # Find top edge (first sharp brightness increase from top)
    for y in range(fh // 4):
        if row_grad[y] > row_grad_threshold:
            y_min = y
            break
    
    # Find bottom edge (last sharp brightness drop)
    for y in range(fh - 1, fh * 3 // 4, -1):
        if row_grad[y] > row_grad_threshold:
            y_max = y
            break
    
    # Find left edge
    for x in range(fw // 4):
        if col_grad[x] > col_grad_threshold:
            x_min = x
            break
    
    # Find right edge
    for x in range(fw - 1, fw * 3 // 4, -1):
        if col_grad[x] > col_grad_threshold:
            x_max = x
            break
    
    w = x_max - x_min + 1
    h = y_max - y_min + 1
    
    # If detection gives nearly full frame, try alternative approach:
    # look for the black/dark border bands
    if w > fw * 0.93 and h > fh * 0.93:
        print(f"  [~] Gradient detection got {w}x{h}, trying dark-border method...")
        
        # Find dark edges more aggressively
        # Top: find rows where brightness is very low
        dark_thresh = np.percentile(row_profile, 20)
        bright_thresh = np.percentile(row_profile, 50)
        
        y_min = 0
        for y in range(fh // 3):
            if row_profile[y] > bright_thresh:
                y_min = y
                break
        
        y_max = fh - 1
        for y in range(fh - 1, fh * 2 // 3, -1):
            if row_profile[y] > bright_thresh:
                y_max = y
                break
        
        x_min = 0
        for x in range(fw // 3):
            if col_profile[x] > bright_thresh:
                x_min = x
                break
        
        x_max = fw - 1
        for x in range(fw - 1, fw * 2 // 3, -1):
            if col_profile[x] > bright_thresh:
                x_max = x
                break
        
        w = x_max - x_min + 1
        h = y_max - y_min + 1
    
    # Inward padding to remove border artifacts
    pad = 4
    x_min += pad
    y_min += pad
    w -= pad * 2
    h -= pad * 2
    
    # Ensure minimum reasonable size
    w = max(320, w)
    h = max(240, h)
    
    ratio = w * h / (fw * fh) * 100
    print(f"  [✓] Detected OS region: {w}x{h} at ({x_min},{y_min})")
    print(f"      Full frame: {fw}x{fh} → crop ratio: {ratio:.0f}%")
    
    return (int(x_min), int(y_min), int(w), int(h))


# ═══════════════════════════════════════════════════════════════
# Audio Energy Analysis (for zoom/freeze triggers)
# ═══════════════════════════════════════════════════════════════

def analyze_audio_energy(video_path, duration=None):
    """
    Analyze audio energy to find loud/intense moments.
    Returns (energy_func, peak_times):
      - energy_func(t) → energy at time t (0..1 normalized)
      - peak_times → list of timestamps where energy spikes
    """
    try:
        import librosa
        print("  [♪] Analyzing audio energy...")
        
        y, sr = librosa.load(video_path, duration=duration, mono=True)
        
        hop_length = 512
        rms = librosa.feature.rms(y=y, hop_length=hop_length)[0]
        times = librosa.frames_to_time(np.arange(len(rms)), sr=sr, hop_length=hop_length)
        
        rms_max = np.max(rms) if np.max(rms) > 0 else 1
        rms_norm = rms / rms_max
        
        # Detect onsets (transients/impacts)
        onset_env = librosa.onset.onset_strength(y=y, sr=sr, hop_length=hop_length)
        onset_times = librosa.onset.onset_detect(
            y=y, sr=sr, hop_length=hop_length,
            onset_envelope=onset_env,
            units='time'
        )
        
        # Keep only strong onsets (top 15% energy)
        peak_thresh = np.percentile(rms_norm, 85)
        strong = []
        for ot in onset_times:
            idx = np.argmin(np.abs(times - ot))
            if idx < len(rms_norm) and rms_norm[idx] > peak_thresh:
                strong.append(ot)
        
        # Minimum 2s spacing
        filtered = []
        for t in strong:
            if not filtered or t - filtered[-1] > 2.0:
                filtered.append(t)
        
        def energy_at(t):
            idx = np.argmin(np.abs(times - t))
            return float(rms_norm[idx]) if idx < len(rms_norm) else 0.0
        
        print(f"  [✓] Found {len(filtered)} impact points")
        return energy_at, filtered
        
    except ImportError:
        print("  [!] librosa not available, using simple energy analysis")
        return lambda t: 0.5, []
    except Exception as e:
        print(f"  [!] Audio analysis error: {e}")
        return lambda t: 0.5, []


# ═══════════════════════════════════════════════════════════════
# Visual Effects
# ═══════════════════════════════════════════════════════════════

def apply_vignette(frame, intensity=0.4):
    h, w = frame.shape[:2]
    Y, X = np.ogrid[:h, :w]
    cx, cy = w / 2, h / 2
    dist = np.sqrt(((X - cx) / cx) ** 2 + ((Y - cy) / cy) ** 2)
    vignette = np.clip(1.0 - (dist - 0.6) * intensity * 2.5, 0, 1)
    return (frame * vignette[:, :, np.newaxis]).astype(np.uint8)


def apply_scanlines(frame, opacity=0.06):
    result = frame.copy().astype(np.float32)
    result[::2, :, :] *= (1.0 - opacity)
    return result.astype(np.uint8)


def apply_color_grade(frame, green_boost=1.12, warmth=0.93):
    graded = frame.astype(np.float32)
    graded[:, :, 0] *= warmth
    graded[:, :, 1] *= green_boost
    graded[:, :, 2] *= warmth
    return np.clip(graded, 0, 255).astype(np.uint8)


def apply_glitch(frame, intensity=0.5):
    h, w = frame.shape[:2]
    shift = max(1, int(w * 0.015 * intensity))
    result = frame.copy()
    result[:, shift:, 0] = frame[:, :-shift, 0]
    result[:, :-shift, 2] = frame[:, shift:, 2]
    num_rows = max(1, int(h * 0.05 * intensity))
    for _ in range(num_rows):
        row = np.random.randint(0, h)
        block_h = np.random.randint(1, max(2, int(4 * intensity)))
        offset = np.random.randint(-shift * 4, shift * 4)
        if offset == 0:
            continue
        for r in range(row, min(row + block_h, h)):
            if 0 < offset < w:
                result[r, offset:, :] = frame[r, :-offset, :]
            elif -w < offset < 0:
                result[r, :offset, :] = frame[r, -offset:, :]
    return result


def apply_impact_flash(frame, intensity=1.0):
    flash = frame.astype(np.float32)
    flash[:, :, 0] += 60 * intensity
    flash[:, :, 1] += 180 * intensity
    flash[:, :, 2] += 80 * intensity
    return np.clip(flash, 0, 255).astype(np.uint8)


def apply_zoom(frame, factor=1.0, target_size=None):
    if factor <= 1.001:
        return frame
    from PIL import Image
    h, w = frame.shape[:2]
    crop_w = int(w / factor)
    crop_h = int(h / factor)
    x1 = (w - crop_w) // 2
    y1 = (h - crop_h) // 2
    cropped = frame[y1:y1+crop_h, x1:x1+crop_w]
    tw, th = target_size if target_size else (w, h)
    img = Image.fromarray(cropped).resize((tw, th), Image.LANCZOS)
    return np.array(img)


def make_glow_text(txt, fontsize=72, color=GREEN):
    from PIL import Image, ImageDraw, ImageFont, ImageFilter
    
    char_w = int(fontsize * 0.62)
    text_w = len(txt) * char_w + 140
    text_h = fontsize * 2 + 100
    
    img = Image.new('RGBA', (text_w, text_h), (0, 0, 0, 0))
    
    try:
        font = ImageFont.truetype("C:\\Windows\\Fonts\\consola.ttf", fontsize)
    except:
        try:
            font = ImageFont.truetype("C:\\Windows\\Fonts\\arial.ttf", fontsize)
        except:
            font = ImageFont.load_default()
    
    draw = ImageDraw.Draw(img)
    bbox = draw.textbbox((0, 0), txt, font=font)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    tx = (text_w - tw) // 2
    ty = (text_h - th) // 2
    
    for radius, alpha in [(25, 40), (15, 70), (8, 100)]:
        glow = Image.new('RGBA', (text_w, text_h), (0, 0, 0, 0))
        ImageDraw.Draw(glow).text((tx, ty), txt, font=font,
                                  fill=(color[0], color[1], color[2], alpha))
        glow = glow.filter(ImageFilter.GaussianBlur(radius=radius))
        img = Image.alpha_composite(img, glow)
    
    ImageDraw.Draw(img).text((tx, ty), txt, font=font,
                             fill=(color[0], color[1], color[2], 255))
    return np.array(img)


def _overlay_rgba(frame, rgba_img, x, y, alpha=1.0):
    fh, fw = frame.shape[:2]
    ih, iw = rgba_img.shape[:2]
    x1, y1 = max(0, x), max(0, y)
    x2, y2 = min(fw, x + iw), min(fh, y + ih)
    if x2 <= x1 or y2 <= y1:
        return
    sx1, sy1 = x1 - x, y1 - y
    sx2, sy2 = sx1 + (x2 - x1), sy1 + (y2 - y1)
    if rgba_img.shape[2] == 4:
        for c in range(3):
            bg = frame[y1:y2, x1:x2, c].astype(np.float32)
            fg = rgba_img[sy1:sy2, sx1:sx2, c].astype(np.float32)
            a = rgba_img[sy1:sy2, sx1:sx2, 3].astype(np.float32) / 255.0 * alpha
            frame[y1:y2, x1:x2, c] = np.clip(bg * (1 - a) + fg * a, 0, 255).astype(np.uint8)


# ═══════════════════════════════════════════════════════════════
# Main Builder
# ═══════════════════════════════════════════════════════════════

def build_showcase(video_path, output_path=DEFAULT_OUTPUT, target_duration=TARGET_DURATION):
    from moviepy import (VideoFileClip, VideoClip, ImageClip,
                         CompositeVideoClip, concatenate_videoclips, vfx, afx)
    
    print("\n╔══════════════════════════════════════════╗")
    print("║  TrustOS Cinematic Showcase Builder v2  ║")
    print("╚══════════════════════════════════════════╝\n")
    
    # ── Step 1: Load video ──
    print("[1/5] Loading video...")
    source = VideoFileClip(video_path)
    src_w, src_h = source.size
    src_dur = source.duration
    print(f"  Source: {src_w}x{src_h}, {src_dur:.1f}s, {source.fps:.0f}fps")
    
    # ── Step 2: Detect & crop OS region ──
    print("[2/5] Detecting OS region...")
    crop_region = detect_os_region(video_path)
    
    if crop_region:
        cx, cy, cw, ch = crop_region
        cropped = source.cropped(x1=cx, y1=cy, x2=cx+cw, y2=cy+ch)
    else:
        cropped = source
    
    # ── Step 3: Analyze audio ──
    print("[3/5] Analyzing audio energy...")
    energy_fn, impact_times = analyze_audio_energy(
        video_path, duration=min(src_dur, target_duration + 5)
    )
    
    # ── Step 4: Build continuous cinematic edit ──
    print("[4/5] Building cinematic edit...")
    
    intro_dur = 3.0
    outro_dur = 2.5
    content_dur = target_duration - intro_dur - outro_dur
    
    # Find best continuous segment of the video (keeps audio unbroken)
    best_start = 0
    best_score = -1
    for t_start in np.arange(0, max(0.1, src_dur - content_dur - 1), 1.0):
        n_impacts = sum(1 for it in impact_times if t_start <= it <= t_start + content_dur)
        start_energy = energy_fn(t_start)
        score = n_impacts * 2 - start_energy * 3
        if score > best_score:
            best_score = score
            best_start = t_start
    
    if best_start + content_dur > src_dur:
        best_start = max(0, src_dur - content_dur - 0.5)
    
    content_end = min(best_start + content_dur, src_dur - 0.1)
    actual_dur = content_end - best_start
    
    # Impacts relative to content start
    rel_impacts = [it - best_start for it in impact_times
                   if best_start <= it <= content_end]
    
    print(f"  Segment: {best_start:.1f}s → {content_end:.1f}s ({actual_dur:.1f}s)")
    print(f"  Impacts in range: {len(rel_impacts)}")
    
    # Extract continuous segment (VIDEO + AUDIO together)
    content = cropped.subclipped(best_start, content_end).resized(FINAL_RES)
    
    # Build the time-remap function for freeze frames
    # This slows video to ~15% speed for 0.25s at each impact, creating a freeze feel
    # We need to adjust total duration to account for the added freeze time
    freeze_dur = 0.25
    total_freeze_time = len(rel_impacts) * freeze_dur * 0.85  # 85% of time is "frozen"
    
    # Time remap: maps output time → source time
    # At impact points, output time advances but source time nearly stops
    def time_remap(t):
        src_t = t
        # Account for freeze frames before current time
        for imp_t in rel_impacts:
            if t >= imp_t and t < imp_t + freeze_dur:
                # During freeze: advance source time very slowly
                progress = (t - imp_t) / freeze_dur
                src_t = imp_t + progress * freeze_dur * 0.15  # 15% speed
                return max(0, min(src_t, actual_dur - 0.05))
        return max(0, min(src_t, actual_dur - 0.05))
    
    # Apply cinematic effects per frame
    def cinematic_transform(get_frame, t):
        frame = get_frame(t)
        abs_t = t + best_start
        energy = energy_fn(abs_t)
        
        # ── ZOOM on loud/impact moments ──
        zoom = 1.0
        for imp_t in rel_impacts:
            dt = abs(t - imp_t)
            if dt < 1.5:
                if t < imp_t:
                    zoom_add = (1.0 - dt / 1.5) * 0.18  # Build up
                else:
                    zoom_add = max(0, (1.0 - dt / 0.8)) * 0.22  # Quick release
                zoom = max(zoom, 1.0 + zoom_add)
        zoom += energy * 0.04
        
        if zoom > 1.005:
            frame = apply_zoom(frame, zoom, FINAL_RES)
        
        # ── GLITCH + FLASH on impacts ──
        for imp_t in rel_impacts:
            dt = abs(t - imp_t)
            if dt < 0.12:
                g_intensity = (0.12 - dt) / 0.12
                frame = apply_glitch(frame, intensity=g_intensity * 0.7)
                if dt < 0.04:
                    frame = apply_impact_flash(frame, intensity=(0.04 - dt) / 0.04 * 0.5)
        
        # ── Persistent effects ──
        frame = apply_color_grade(frame, green_boost=1.08, warmth=0.94)
        frame = apply_scanlines(frame, opacity=0.04)
        vig = max(0.15, 0.45 - energy * 0.2)
        frame = apply_vignette(frame, intensity=vig)
        
        return frame
    
    content_fx = content.transform(cinematic_transform)
    
    # Apply time remap for freeze frames (video only, audio stays continuous!)
    if rel_impacts:
        content_fx = content_fx.time_transform(time_remap, apply_to=['video'])
    
    content_fx = content_fx.with_effects([vfx.FadeIn(0.5), vfx.FadeOut(0.5)])
    
    # ── Intro title card ──
    title_img = make_glow_text("TrustOS", fontsize=80, color=GREEN)
    subtitle_img = make_glow_text("A Modern Operating System in Rust", fontsize=32, color=DARK_GREEN)
    
    def make_intro(t):
        frame = np.zeros((FINAL_RES[1], FINAL_RES[0], 3), dtype=np.uint8)
        phase = t * 30
        for gy in range(0, FINAL_RES[1], 50):
            val = int(6 + 3 * math.sin((gy + phase) * 0.04))
            frame[gy:min(gy+1, FINAL_RES[1]), :, 1] = val
        for gx in range(0, FINAL_RES[0], 50):
            val = int(5 + 2 * math.sin((gx + phase) * 0.03))
            frame[:, gx:min(gx+1, FINAL_RES[0]), 1] = val
        
        if t < 1.0:
            a = t
        elif t > intro_dur - 0.5:
            a = (intro_dur - t) / 0.5
        else:
            a = 1.0
        a = max(0, min(1, a))
        
        _overlay_rgba(frame, title_img,
                     (FINAL_RES[0] - title_img.shape[1]) // 2,
                     (FINAL_RES[1] - title_img.shape[0]) // 2 - 30, a)
        if t > 0.6:
            sa = min(1, (t - 0.6) / 0.6) * a
            _overlay_rgba(frame, subtitle_img,
                         (FINAL_RES[0] - subtitle_img.shape[1]) // 2,
                         (FINAL_RES[1] + title_img.shape[0]) // 2, sa)
        return apply_vignette(frame, 0.5)
    
    intro_clip = VideoClip(make_intro, duration=intro_dur).with_fps(FPS)
    
    # ── Outro card ──
    end_img = make_glow_text("github.com/nathan237/TrustOS", fontsize=36, color=DARK_GREEN)
    
    def make_outro(t):
        frame = np.zeros((FINAL_RES[1], FINAL_RES[0], 3), dtype=np.uint8)
        if t < 0.5:
            a = t / 0.5
        elif t > outro_dur - 0.8:
            a = (outro_dur - t) / 0.8
        else:
            a = 1.0
        _overlay_rgba(frame, end_img,
                     (FINAL_RES[0] - end_img.shape[1]) // 2,
                     (FINAL_RES[1] - end_img.shape[0]) // 2, max(0, min(1, a)))
        return apply_vignette(frame, 0.5)
    
    outro_clip = VideoClip(make_outro, duration=outro_dur).with_fps(FPS)
    
    # ── Assemble ──
    print("[5/5] Rendering final video...")
    
    final = concatenate_videoclips([intro_clip, content_fx, outro_clip], method="compose")
    
    # Audio: continuous from the source segment, offset after intro
    src_audio = source.subclipped(best_start, content_end).audio
    if src_audio is not None:
        audio_track = src_audio.with_start(intro_dur)
        audio_track = audio_track.with_effects([
            afx.AudioFadeIn(0.3),
            afx.AudioFadeOut(1.5),
        ])
        final = final.with_audio(audio_track)
    
    final.write_videofile(
        output_path,
        fps=FPS,
        codec='libx264',
        audio_codec='aac',
        bitrate='10000k',
        preset='medium',
        threads=4,
        logger='bar'
    )
    
    source.close()
    
    sz = os.path.getsize(output_path) / (1024 * 1024)
    print(f"\n  ✅ Showcase saved: {output_path}")
    print(f"     Duration: {final.duration:.1f}s | {FINAL_RES[0]}x{FINAL_RES[1]} | {sz:.1f} MB")


# ═══════════════════════════════════════════════════════════════
# CLI
# ═══════════════════════════════════════════════════════════════

def find_latest_video():
    d = Path(r"C:\Users\nathan\Videos\Radeon ReLive\unknown")
    if d.exists():
        mp4s = sorted(d.glob("*.mp4"), key=lambda f: f.stat().st_mtime, reverse=True)
        if mp4s:
            return str(mp4s[0])
    return None


def main():
    parser = argparse.ArgumentParser(description="TrustOS Cinematic Showcase Builder v2")
    parser.add_argument("--video", "-v", help="Input video file")
    parser.add_argument("--out", "-o", default=DEFAULT_OUTPUT, help="Output path")
    parser.add_argument("--duration", "-d", type=int, default=TARGET_DURATION, help="Target duration (s)")
    args = parser.parse_args()
    
    video_path = args.video or find_latest_video()
    if not video_path:
        print("ERROR: No video found. Use --video path.mp4")
        sys.exit(1)
    if not os.path.exists(video_path):
        print(f"ERROR: Not found: {video_path}")
        sys.exit(1)
    
    print(f"  Input: {Path(video_path).name}")
    build_showcase(video_path, args.out, args.duration)


if __name__ == "__main__":
    main()
