"""Generate a YouTube thumbnail for TrustOS video (1280x720)"""
from PIL import Image, ImageDraw, ImageFont, ImageFilter
import math, random, os

W, H = 1280, 720
img = Image.new("RGB", (W, H), (8, 8, 20))
draw = ImageDraw.Draw(img)

random.seed(42)

# === BACKGROUND: Dark gradient + matrix rain columns ===
for y in range(H):
    r = int(8 + 15 * (y / H))
    g = int(8 + 25 * (y / H))
    b = int(20 + 40 * (y / H))
    draw.line([(0, y), (W, y)], fill=(r, g, b))

# Matrix rain effect (subtle)
matrix_chars = "01アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン"
for col in range(0, W, 18):
    length = random.randint(5, 25)
    start_y = random.randint(-200, H)
    for i in range(length):
        y = start_y + i * 16
        if 0 <= y < H:
            alpha = max(0, 255 - i * 15)
            green = int(alpha * 0.35)
            c = random.choice(matrix_chars)
            draw.text((col, y), c, fill=(0, green, 0))

# === GLOW EFFECTS ===
glow = Image.new("RGB", (W, H), (0, 0, 0))
glow_draw = ImageDraw.Draw(glow)

# Left side orange/red glow
for r in range(300, 0, -2):
    alpha = int(40 * (1 - r / 300))
    glow_draw.ellipse([100 - r, 360 - r, 100 + r, 360 + r], fill=(alpha, int(alpha * 0.3), 0))

# Right side cyan glow  
for r in range(250, 0, -2):
    alpha = int(35 * (1 - r / 250))
    glow_draw.ellipse([1100 - r, 300 - r, 1100 + r, 300 + r], fill=(0, int(alpha * 0.6), alpha))

# Center blue-white glow
for r in range(400, 0, -3):
    alpha = int(20 * (1 - r / 400))
    glow_draw.ellipse([640 - r, 320 - r, 640 + r, 320 + r], fill=(int(alpha * 0.3), int(alpha * 0.4), alpha))

glow = glow.filter(ImageFilter.GaussianBlur(40))
img = Image.composite(Image.blend(img, glow, 0.7), img, Image.new("L", (W, H), 180))

draw = ImageDraw.Draw(img)

# === TRY TO LOAD GOOD FONTS ===
def find_font(names, size):
    font_dirs = [
        "C:/Windows/Fonts/",
        os.path.expanduser("~/AppData/Local/Microsoft/Windows/Fonts/"),
    ]
    for name in names:
        for d in font_dirs:
            path = os.path.join(d, name)
            if os.path.exists(path):
                try:
                    return ImageFont.truetype(path, size)
                except:
                    pass
    return ImageFont.load_default()

font_huge = find_font(["impact.ttf", "Impact.ttf", "arialbd.ttf"], 130)
font_big = find_font(["impact.ttf", "Impact.ttf", "arialbd.ttf"], 90)
font_med = find_font(["arialbd.ttf", "arial.ttf", "calibrib.ttf"], 42)
font_small = find_font(["arialbd.ttf", "arial.ttf", "calibri.ttf"], 32)
font_stat = find_font(["impact.ttf", "Impact.ttf", "arialbd.ttf"], 56)
font_rust = find_font(["arialbd.ttf", "calibrib.ttf"], 36)

# === MAIN TEXT: "I Built a 6MB OS" ===
# Line 1: "I BUILT A"
text1 = "I BUILT A"
bb1 = draw.textbbox((0, 0), text1, font=font_big)
tw1 = bb1[2] - bb1[0]
x1 = (W - tw1) // 2 + 40
y1 = 60

# Text shadow/outline
for ox in range(-3, 4):
    for oy in range(-3, 4):
        draw.text((x1 + ox, y1 + oy), text1, fill=(0, 0, 0), font=font_big)
draw.text((x1, y1), text1, fill=(255, 255, 255), font=font_big)

# Line 2: "6MB OS" - huge, orange gradient effect
text2 = "6MB OS"
bb2 = draw.textbbox((0, 0), text2, font=font_huge)
tw2 = bb2[2] - bb2[0]
x2 = (W - tw2) // 2 + 40
y2 = 155

# Glow behind big text
for ox in range(-4, 5):
    for oy in range(-4, 5):
        draw.text((x2 + ox, y2 + oy), text2, fill=(0, 0, 0), font=font_huge)

# Orange-yellow gradient text via mask
txt_layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
txt_draw = ImageDraw.Draw(txt_layer)
txt_draw.text((x2, y2), text2, fill=(255, 255, 255, 255), font=font_huge)

grad = Image.new("RGB", (W, H), (0, 0, 0))
grad_draw = ImageDraw.Draw(grad)
th = bb2[3] - bb2[1]
for py in range(y2, y2 + th + 20):
    t = (py - y2) / max(th, 1)
    r = int(255)
    g = int(200 - t * 120)
    b = int(30 - t * 30)
    grad_draw.line([(0, py), (W, py)], fill=(r, max(0, g), max(0, b)))

mask = txt_layer.split()[3]
img.paste(grad, (0, 0), mask)
draw = ImageDraw.Draw(img)

# === "IN PURE RUST" with Rust orange ===
text3 = "IN PURE RUST"
bb3 = draw.textbbox((0, 0), text3, font=font_med)
tw3 = bb3[2] - bb3[0]
x3 = (W - tw3) // 2 + 40
y3 = 300

for ox in range(-2, 3):
    for oy in range(-2, 3):
        draw.text((x3 + ox, y3 + oy), text3, fill=(0, 0, 0), font=font_med)
draw.text((x3, y3), text3, fill=(222, 120, 50), font=font_med)

# === STATS BAR at bottom ===
# Dark semi-transparent bar
bar_y = 520
draw.rectangle([(0, bar_y), (W, bar_y + 130)], fill=(5, 5, 15))
draw.line([(0, bar_y), (W, bar_y)], fill=(0, 200, 255), width=2)
draw.line([(0, bar_y + 130), (W, bar_y + 130)], fill=(0, 200, 255), width=2)

stats = [
    ("99K", "LINES OF RUST", (0, 220, 180)),
    ("6 MB", "ISO SIZE", (255, 170, 50)),
    ("<1s", "BOOT TIME", (100, 200, 255)),
    ("7", "DAYS", (255, 80, 80)),
]

stat_w = W // len(stats)
for i, (val, label, color) in enumerate(stats):
    cx = i * stat_w + stat_w // 2
    
    # Value
    bb = draw.textbbox((0, 0), val, font=font_stat)
    vw = bb[2] - bb[0]
    draw.text((cx - vw // 2, bar_y + 12), val, fill=color, font=font_stat)
    
    # Label
    bb = draw.textbbox((0, 0), label, font=font_small)
    lw = bb[2] - bb[0]
    draw.text((cx - lw // 2, bar_y + 78), label, fill=(180, 180, 200), font=font_small)

# === "TRUST OS" logo text on left ===
logo = "TRUSTOS"
bb_logo = draw.textbbox((0, 0), logo, font=font_med)
lw = bb_logo[2] - bb_logo[0]

# Neon glow effect for logo
for ox in range(-3, 4):
    for oy in range(-3, 4):
        if abs(ox) + abs(oy) > 3:
            draw.text((38 + ox, 660 + oy), logo, fill=(0, 40, 80), font=font_small)
draw.text((38, 660), logo, fill=(0, 200, 255), font=font_small)

# === Small "vs Windows 6GB" comparison text ===
vs_text = "vs Windows = 6 GB"
for ox in range(-2, 3):
    for oy in range(-2, 3):
        draw.text((x3 + tw3 // 2 - 130 + ox, y3 + 55 + oy), vs_text, fill=(0, 0, 0), font=font_small)
draw.text((x3 + tw3 // 2 - 130, y3 + 55), vs_text, fill=(255, 70, 70), font=font_small)

# === "0 LINES OF C" ===
zero_text = "0 LINES OF C."
bb_z = draw.textbbox((0, 0), zero_text, font=font_small)
zw = bb_z[2] - bb_z[0]
zx = (W - zw) // 2 + 40
zy = y3 + 110
for ox in range(-2, 3):
    for oy in range(-2, 3):
        draw.text((zx + ox, zy + oy), zero_text, fill=(0, 0, 0), font=font_small)
draw.text((zx, zy), zero_text, fill=(180, 255, 180), font=font_small)

# === DECORATIVE: Small Rust gear/crab emoji substitute ===
# Draw a simple gear shape
gear_x, gear_y = 85, 390
for angle in range(0, 360, 45):
    rad = math.radians(angle)
    ex = int(gear_x + 18 * math.cos(rad))
    ey = int(gear_y + 18 * math.sin(rad))
    draw.ellipse([ex - 4, ey - 4, ex + 4, ey + 4], fill=(222, 120, 50))
draw.ellipse([gear_x - 10, gear_y - 10, gear_x + 10, gear_y + 10], fill=(222, 120, 50))
draw.ellipse([gear_x - 5, gear_y - 5, gear_x + 5, gear_y + 5], fill=(15, 15, 30))

# === Sparkle particles ===
for _ in range(60):
    sx = random.randint(0, W)
    sy = random.randint(0, H)
    brightness = random.randint(100, 255)
    size = random.choice([1, 1, 1, 2])
    draw.ellipse([sx, sy, sx + size, sy + size], fill=(brightness, brightness, brightness))

# === SAVE ===
out_path = os.path.join(os.path.dirname(__file__), "thumbnail.png")
img.save(out_path, "PNG", quality=95)
print(f"Thumbnail saved to: {out_path}")
print(f"Size: {os.path.getsize(out_path) / 1024:.0f} KB")
print(f"Resolution: {W}x{H}")
