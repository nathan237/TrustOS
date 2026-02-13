#!/usr/bin/env python3
"""
Generate an optimized YouTube thumbnail for TrustOS showcase.
1280x720 — bold, high contrast, minimal text, tech aesthetic.
"""

from PIL import Image, ImageDraw, ImageFont
import os

W, H = 1280, 720
OUT = os.path.join(os.path.dirname(__file__), "thumbnail_yt.png")


def hex_to_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i+2], 16) for i in (0, 2, 4))


def draw_gradient(draw, x0, y0, x1, y1, color_start, color_end, vertical=True):
    """Draw a gradient rectangle."""
    r1, g1, b1 = color_start
    r2, g2, b2 = color_end
    steps = (y1 - y0) if vertical else (x1 - x0)
    for i in range(steps):
        t = i / max(steps - 1, 1)
        r = int(r1 + (r2 - r1) * t)
        g = int(g1 + (g2 - g1) * t)
        b = int(b1 + (b2 - b1) * t)
        if vertical:
            draw.line([(x0, y0 + i), (x1, y0 + i)], fill=(r, g, b))
        else:
            draw.line([(x0 + i, y0), (x0 + i, y1)], fill=(r, g, b))


def draw_text_with_shadow(draw, pos, text, font, fill, shadow_color=(0, 0, 0), offset=3):
    x, y = pos
    # Shadow
    draw.text((x + offset, y + offset), text, font=font, fill=shadow_color)
    # Main text
    draw.text((x, y), text, font=font, fill=fill)


def draw_code_lines(draw, x, y, w, h):
    """Draw fake code lines to simulate a code editor."""
    colors = [
        (255, 123, 114),  # red (keywords)
        (121, 192, 255),  # blue (functions)
        (206, 145, 120),  # orange (strings)
        (181, 206, 168),  # green (numbers)
        (212, 212, 212),  # white (default)
        (156, 220, 254),  # cyan (variables)
    ]
    
    lines_data = [
        [(255,123,114, "fn "), (121,192,255, "main"), (255,215,0, "() {")],
        [(212,212,212, "    "), (255,123,114, "let "), (156,220,254, "screen_w"), (212,212,212, " = "), (121,192,255, "screen_w"), (255,215,0, "()")],
        [(212,212,212, "    "), (255,123,114, "let mut "), (156,220,254, "pos_x"), (212,212,212, " = "), (181,206,168, "0")],
        [(212,212,212, "    "), (255,123,114, "while "), (156,220,254, "frame"), (212,212,212, " < "), (181,206,168, "300"), (212,212,212, " {")],
        [(212,212,212, "        "), (121,192,255, "fill_rect"), (255,215,0, "("), (156,220,254, "pos_x"), (212,212,212, ", "), (156,220,254, "pos_y"), (255,215,0, ")")],
        [(212,212,212, "        "), (121,192,255, "draw_text"), (255,215,0, "("), (206,145,120, '"SUBSCRIBE"'), (255,215,0, ")")],
        [(212,212,212, "        "), (121,192,255, "flush"), (255,215,0, "()"), (121,192,255, " sleep"), (255,215,0, "("), (181,206,168, "33"), (255,215,0, ")")],
        [(212,212,212, "    "), (255,215,0, "}")],
        [(255,215,0, "}")],
    ]
    
    line_h = 22
    try:
        code_font = ImageFont.truetype("consola.ttf", 16)
    except:
        try:
            code_font = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 16)
        except:
            code_font = ImageFont.load_default()
    
    for i, line_parts in enumerate(lines_data):
        cx = x + 4
        cy = y + 4 + i * line_h
        if cy + line_h > y + h:
            break
        for part in line_parts:
            r, g, b, text = part
            draw.text((cx, cy), text, font=code_font, fill=(r, g, b))
            bbox = code_font.getbbox(text)
            cx += bbox[2] - bbox[0]


def make_thumbnail():
    img = Image.new("RGB", (W, H))
    draw = ImageDraw.Draw(img)

    # ── Background: dark gradient (deep navy → black) ──
    draw_gradient(draw, 0, 0, W, H, (8, 12, 28), (2, 2, 8))

    # ── Matrix rain effect (subtle green columns) ──
    import random
    random.seed(42)
    for col in range(0, W, 18):
        length = random.randint(3, 12)
        start_y = random.randint(0, H)
        for j in range(length):
            py = (start_y + j * 20) % H
            alpha = max(0, 255 - j * 30)
            c = random.randint(33, 126)
            try:
                small_font = ImageFont.truetype("consola.ttf", 14)
            except:
                small_font = ImageFont.load_default()
            green = max(20, 100 - j * 12)
            draw.text((col, py), chr(c), font=small_font, fill=(0, green, 0))

    # ── Darken the rain so text pops ──
    from PIL import ImageEnhance
    img = ImageEnhance.Brightness(img).enhance(0.35)
    draw = ImageDraw.Draw(img)

    # Re-draw gradient over rain (semi-transparent effect via blending)
    overlay = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    ov_draw = ImageDraw.Draw(overlay)
    # Left side darker for text, right side lighter for code
    for x in range(W):
        alpha = int(180 - (x / W) * 100)
        ov_draw.line([(x, 0), (x, H)], fill=(5, 8, 18, alpha))
    img = Image.alpha_composite(img.convert("RGBA"), overlay).convert("RGB")
    draw = ImageDraw.Draw(img)

    # ── Code editor panel (right side) ──
    editor_x = 680
    editor_y = 80
    editor_w = 560
    editor_h = 260
    # Panel background
    draw.rounded_rectangle(
        [editor_x, editor_y, editor_x + editor_w, editor_y + editor_h],
        radius=12, fill=(13, 17, 23), outline=(0, 255, 68, 180), width=2
    )
    # Title bar
    draw.rounded_rectangle(
        [editor_x, editor_y, editor_x + editor_w, editor_y + 30],
        radius=12, fill=(30, 30, 30)
    )
    draw.rectangle([editor_x, editor_y + 18, editor_x + editor_w, editor_y + 30], fill=(30, 30, 30))
    # Window dots  
    for i, color in enumerate([(255, 95, 86), (255, 189, 46), (39, 201, 63)]):
        draw.ellipse([editor_x + 12 + i * 20, editor_y + 8, editor_x + 24 + i * 20, editor_y + 20], fill=color)
    # Title text
    try:
        title_font = ImageFont.truetype("consola.ttf", 12)
    except:
        title_font = ImageFont.load_default()
    draw.text((editor_x + 80, editor_y + 9), "TrustCode — youtube_dvd.tl", font=title_font, fill=(0, 255, 100))
    
    # Code content
    draw_code_lines(draw, editor_x, editor_y + 32, editor_w, editor_h - 32)

    # ── "YouTube play button" result (bottom right) — red rectangle ──
    yt_x, yt_y = 780, 400
    yt_w, yt_h = 120, 85
    # Shadow
    draw.rounded_rectangle([yt_x + 4, yt_y + 4, yt_x + yt_w + 4, yt_y + yt_h + 4], radius=14, fill=(40, 0, 0))
    # Red button
    draw.rounded_rectangle([yt_x, yt_y, yt_x + yt_w, yt_y + yt_h], radius=14, fill=(230, 0, 0))
    # White triangle
    tri_cx = yt_x + yt_w // 2 + 5
    tri_cy = yt_y + yt_h // 2
    tri_size = 22
    draw.polygon([
        (tri_cx - tri_size // 2, tri_cy - tri_size),
        (tri_cx - tri_size // 2, tri_cy + tri_size),
        (tri_cx + tri_size, tri_cy),
    ], fill=(255, 255, 255))

    # Arrow from code → YouTube button
    arrow_start = (editor_x + editor_w // 2, editor_y + editor_h + 10)
    arrow_end = (yt_x + yt_w // 2, yt_y - 8)
    draw.line([arrow_start, arrow_end], fill=(0, 255, 100), width=3)
    # Arrowhead
    draw.polygon([
        (arrow_end[0], arrow_end[1]),
        (arrow_end[0] - 10, arrow_end[1] - 16),
        (arrow_end[0] + 10, arrow_end[1] - 16),
    ], fill=(0, 255, 100))

    # ── Small label under YouTube button ──
    try:
        label_font = ImageFont.truetype("segoeui.ttf", 18)
    except:
        try:
            label_font = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 18)
        except:
            label_font = ImageFont.load_default()
    draw.text((yt_x - 30, yt_y + yt_h + 12), "Runs inside the OS!", font=label_font, fill=(0, 255, 100))

    # ── Main title text (left side) ──
    try:
        bold_font = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 72)
        medium_font = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 42)
        small_font = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 28)
        accent_font = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 34)
    except:
        bold_font = ImageFont.load_default()
        medium_font = bold_font
        small_font = bold_font
        accent_font = bold_font

    # "I BUILT" 
    draw_text_with_shadow(draw, (40, 60), "I BUILT", bold_font, (255, 255, 255), offset=4)
    
    # "AN ENTIRE OS"
    draw_text_with_shadow(draw, (40, 140), "AN ENTIRE", bold_font, (255, 255, 255), offset=4)
    draw_text_with_shadow(draw, (40, 220), "OS", bold_font, (0, 255, 100), offset=4)
    
    # "in Rust" (orange Rust color)
    draw_text_with_shadow(draw, (155, 230), "in Rust", medium_font, (247, 76, 0), offset=3)

    # ── Stats badges ──
    badge_y = 340
    badges = [
        ("120K", "LINES", (0, 255, 100)),
        ("8", "DAYS", (255, 165, 0)),
        ("0", "LINES C", (255, 80, 80)),
    ]
    
    for i, (num, label, color) in enumerate(badges):
        bx = 50 + i * 185
        # Badge background
        draw.rounded_rectangle([bx, badge_y, bx + 160, badge_y + 90], radius=10, 
                                fill=(15, 20, 35), outline=color, width=2)
        # Number
        draw_text_with_shadow(draw, (bx + 15, badge_y + 5), num, medium_font, color, offset=2)
        # Label
        draw.text((bx + 15, badge_y + 55), label, font=small_font, fill=(180, 180, 180))

    # ── Bottom feature tags ──
    features = ["DESKTOP", "BROWSER", "3D ENGINE", "TLS 1.3", "COMPILER"]
    tag_y = 470
    cx = 50
    for feat in features:
        bbox = small_font.getbbox(feat)
        tw = bbox[2] - bbox[0]
        # Tag pill
        draw.rounded_rectangle([cx, tag_y, cx + tw + 20, tag_y + 36], radius=8,
                                fill=(0, 40, 20), outline=(0, 180, 80), width=1)
        draw.text((cx + 10, tag_y + 4), feat, font=small_font, fill=(0, 230, 100))
        cx += tw + 32

    # ── Rust crab emoji area (bottom left) — just the Rust logo colors ──
    # Orange gear/crab icon (simplified)
    gear_x, gear_y = 50, 560
    draw.text((gear_x, gear_y), "🦀", font=medium_font, fill=(247, 76, 0))
    draw.text((gear_x + 55, gear_y + 8), "Pure Rust. Zero Secrets.", font=accent_font, fill=(200, 200, 200))
    
    # ── "FULLY AUDITABLE" ribbon (top-left corner) ──
    draw.polygon([(0, 0), (260, 0), (0, 50)], fill=(0, 200, 80))
    try:
        ribbon_font = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 14)
    except:
        ribbon_font = ImageFont.load_default()
    draw.text((8, 4), "FULLY AUDITABLE", font=ribbon_font, fill=(0, 0, 0))

    # ── Bottom bar ──
    draw.rectangle([0, H - 8, W, H], fill=(0, 255, 100))

    # ── Save ──
    img.save(OUT, quality=95)
    print(f"✅ Thumbnail saved: {OUT}")
    print(f"   Size: {os.path.getsize(OUT) / 1024:.0f} KB")
    return OUT


if __name__ == "__main__":
    make_thumbnail()
