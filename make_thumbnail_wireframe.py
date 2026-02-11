"""Generate a YouTube thumbnail for TrustOS Wireframe 3D Showcase (1280x720)
Focus: 3D wireframe visuals, NO GPU hook, dramatic lighting
"""
from PIL import Image, ImageDraw, ImageFont, ImageFilter
import math, random, os

W, H = 1280, 720
img = Image.new("RGB", (W, H), (5, 5, 15))
draw = ImageDraw.Draw(img)
random.seed(77)

# === BACKGROUND: Deep space gradient ===
for y in range(H):
    t = y / H
    r = int(5 + 8 * t)
    g = int(5 + 12 * t)
    b = int(15 + 30 * t)
    draw.line([(0, y), (W, y)], fill=(r, g, b))

# === Matrix rain (subtle, green) ===
matrix_chars = "01アイウエオカキクケコサシスセソ"
for col in range(0, W, 20):
    length = random.randint(3, 20)
    start_y = random.randint(-150, H)
    for i in range(length):
        y = start_y + i * 16
        if 0 <= y < H:
            alpha = max(0, 200 - i * 18)
            green = int(alpha * 0.25)
            c = random.choice(matrix_chars)
            draw.text((col, y), c, fill=(0, green, 0))

# === WIREFRAME CUBE (left side) ===
def project_3d(x, y, z, cx, cy, fov=400):
    """Simple perspective projection"""
    z_off = z + 4
    if z_off < 0.1:
        z_off = 0.1
    sx = int(cx + x * fov / z_off)
    sy = int(cy + y * fov / z_off)
    return sx, sy

def rotate_y(x, y, z, angle):
    c = math.cos(angle)
    s = math.sin(angle)
    return x * c + z * s, y, -x * s + z * c

def rotate_x(x, y, z, angle):
    c = math.cos(angle)
    s = math.sin(angle)
    return x, y * c - z * s, y * s + z * c

def draw_wireframe_line(draw, p1, p2, color, thick=2):
    """Draw a thick glowing line"""
    for ox in range(-thick//2, thick//2 + 1):
        for oy in range(-thick//2, thick//2 + 1):
            draw.line([(p1[0]+ox, p1[1]+oy), (p2[0]+ox, p2[1]+oy)], fill=color)

def draw_glow_vertex(draw, p, color, size=4):
    """Draw glowing vertex point"""
    for r in range(size, 0, -1):
        alpha = 255 * r // size
        c = tuple(min(255, int(v * alpha / 255)) for v in color)
        draw.ellipse([p[0]-r, p[1]-r, p[0]+r, p[1]+r], fill=c)

# Cube vertices
cube_verts = [
    (-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1),
    (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1),
]
cube_edges = [
    (0,1),(1,2),(2,3),(3,0),  # front
    (4,5),(5,6),(6,7),(7,4),  # back
    (0,4),(1,5),(2,6),(3,7),  # connecting
]

# Draw cube on left
angle = 0.6
cx_cube, cy_cube = 220, 320
projected_cube = []
for v in cube_verts:
    rx, ry, rz = rotate_y(*rotate_x(*v, 0.3), angle)
    p = project_3d(rx, ry, rz, cx_cube, cy_cube, 300)
    projected_cube.append(p)

for e in cube_edges:
    p1, p2 = projected_cube[e[0]], projected_cube[e[1]]
    draw_wireframe_line(draw, p1, p2, (0, 200, 255), 2)
for p in projected_cube:
    draw_glow_vertex(draw, p, (150, 230, 255), 5)

# === TORUS (right side) ===
torus_verts = []
torus_edges = []
segs_major, segs_minor = 14, 10
R, r_t = 1.2, 0.45

for i in range(segs_major):
    for j in range(segs_minor):
        theta = 2 * math.pi * i / segs_major
        phi = 2 * math.pi * j / segs_minor
        x = (R + r_t * math.cos(phi)) * math.cos(theta)
        y = (R + r_t * math.cos(phi)) * math.sin(theta)
        z = r_t * math.sin(phi)
        torus_verts.append((x, y, z))

for i in range(segs_major):
    for j in range(segs_minor):
        curr = i * segs_minor + j
        next_j = i * segs_minor + (j + 1) % segs_minor
        next_i = ((i + 1) % segs_major) * segs_minor + j
        torus_edges.append((curr, next_j))
        torus_edges.append((curr, next_i))

cx_torus, cy_torus = 1060, 320
projected_torus = []
for v in torus_verts:
    rx, ry, rz = rotate_y(*rotate_x(*v, 0.5), 0.8)
    p = project_3d(rx, ry, rz, cx_torus, cy_torus, 280)
    projected_torus.append(p)

for e in torus_edges:
    p1, p2 = projected_torus[e[0]], projected_torus[e[1]]
    # Depth color: cyan to purple
    mid_z = (torus_verts[e[0]][2] + torus_verts[e[1]][2]) / 2
    intensity = max(0.2, min(1.0, 1.0 / (mid_z * 0.3 + 2)))
    color = (int(80 * intensity), int(180 * intensity), int(255 * intensity))
    draw_wireframe_line(draw, p1, p2, color, 1)

# Glow on some vertices
for i, p in enumerate(projected_torus):
    if i % 3 == 0:
        draw_glow_vertex(draw, p, (100, 200, 255), 3)

# === ICOSPHERE (center, subtle, behind text) ===
# Golden ratio icosahedron
phi_g = (1 + math.sqrt(5)) / 2
ico_verts_raw = [
    (-1, phi_g, 0), (1, phi_g, 0), (-1, -phi_g, 0), (1, -phi_g, 0),
    (0, -1, phi_g), (0, 1, phi_g), (0, -1, -phi_g), (0, 1, -phi_g),
    (phi_g, 0, -1), (phi_g, 0, 1), (-phi_g, 0, -1), (-phi_g, 0, 1),
]
# Normalize
ico_verts_norm = []
for v in ico_verts_raw:
    l = math.sqrt(v[0]**2 + v[1]**2 + v[2]**2)
    ico_verts_norm.append((v[0]/l * 1.5, v[1]/l * 1.5, v[2]/l * 1.5))

ico_edges = [
    (0,1),(0,5),(0,7),(0,10),(0,11),
    (1,5),(1,7),(1,8),(1,9),
    (2,3),(2,4),(2,6),(2,10),(2,11),
    (3,4),(3,6),(3,8),(3,9),
    (4,5),(4,9),(4,11),
    (5,9),(5,11),
    (6,7),(6,8),(6,10),
    (7,8),(7,10),
    (8,9),(10,11),
]

cx_ico, cy_ico = 640, 340
projected_ico = []
for v in ico_verts_norm:
    rx, ry, rz = rotate_y(*rotate_x(*v, 0.4), 0.3)
    p = project_3d(rx, ry, rz, cx_ico, cy_ico, 200)
    projected_ico.append(p)

for e in ico_edges:
    p1, p2 = projected_ico[e[0]], projected_ico[e[1]]
    draw_wireframe_line(draw, p1, p2, (30, 80, 50), 1)

for p in projected_ico:
    draw_glow_vertex(draw, p, (50, 150, 80), 3)

# === GLOW EFFECTS (atmospheric) ===
glow = Image.new("RGB", (W, H), (0, 0, 0))
glow_draw = ImageDraw.Draw(glow)

# Left cyan glow (behind cube)
for r_g in range(350, 0, -3):
    alpha = int(30 * (1 - r_g / 350))
    glow_draw.ellipse([220 - r_g, 320 - r_g, 220 + r_g, 320 + r_g], fill=(0, int(alpha * 0.5), alpha))

# Right purple glow (behind torus)
for r_g in range(300, 0, -3):
    alpha = int(25 * (1 - r_g / 300))
    glow_draw.ellipse([1060 - r_g, 320 - r_g, 1060 + r_g, 320 + r_g], fill=(int(alpha * 0.4), 0, alpha))

# Center subtle warm glow
for r_g in range(250, 0, -3):
    alpha = int(15 * (1 - r_g / 250))
    glow_draw.ellipse([640 - r_g, 280 - r_g, 640 + r_g, 280 + r_g], fill=(alpha, int(alpha * 0.6), 0))

glow = glow.filter(ImageFilter.GaussianBlur(50))
img = Image.composite(Image.blend(img, glow, 0.8), img, Image.new("L", (W, H), 200))
draw = ImageDraw.Draw(img)

# === FONTS ===
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

font_huge = find_font(["impact.ttf", "Impact.ttf", "arialbd.ttf"], 110)
font_big = find_font(["impact.ttf", "Impact.ttf", "arialbd.ttf"], 72)
font_med = find_font(["arialbd.ttf", "arial.ttf", "calibrib.ttf"], 38)
font_small = find_font(["arialbd.ttf", "arial.ttf", "calibri.ttf"], 28)
font_stat = find_font(["impact.ttf", "Impact.ttf", "arialbd.ttf"], 50)
font_tag = find_font(["impact.ttf", "Impact.ttf", "arialbd.ttf"], 34)

# === MAIN TEXT ===
# Line 1: "3D ENGINE"
text1 = "3D ENGINE"
bb1 = draw.textbbox((0, 0), text1, font=font_huge)
tw1 = bb1[2] - bb1[0]
x1 = (W - tw1) // 2
y1 = 30

# Black outline
for ox in range(-4, 5):
    for oy in range(-4, 5):
        draw.text((x1 + ox, y1 + oy), text1, fill=(0, 0, 0), font=font_huge)

# Cyan gradient text
txt_layer = Image.new("RGBA", (W, H), (0, 0, 0, 0))
txt_draw = ImageDraw.Draw(txt_layer)
txt_draw.text((x1, y1), text1, fill=(255, 255, 255, 255), font=font_huge)

grad = Image.new("RGB", (W, H), (0, 0, 0))
grad_draw = ImageDraw.Draw(grad)
th = bb1[3] - bb1[1]
for py in range(y1, y1 + th + 20):
    t = (py - y1) / max(th, 1)
    r = int(0 + 100 * t)
    g = int(220 - 40 * t)
    b = int(255)
    grad_draw.line([(0, py), (W, py)], fill=(r, g, b))

mask = txt_layer.split()[3]
img.paste(grad, (0, 0), mask)
draw = ImageDraw.Draw(img)

# Line 2: "INSIDE MY OWN OS"
text2 = "INSIDE MY OWN OS"
bb2 = draw.textbbox((0, 0), text2, font=font_big)
tw2 = bb2[2] - bb2[0]
x2 = (W - tw2) // 2
y2 = 150

for ox in range(-3, 4):
    for oy in range(-3, 4):
        draw.text((x2 + ox, y2 + oy), text2, fill=(0, 0, 0), font=font_big)
draw.text((x2, y2), text2, fill=(255, 255, 255), font=font_big)

# === "NO GPU" red crossed badge ===
badge_text = "NO GPU"
badge_font = find_font(["impact.ttf", "Impact.ttf", "arialbd.ttf"], 60)
bb_badge = draw.textbbox((0, 0), badge_text, font=badge_font)
bw = bb_badge[2] - bb_badge[0]
bh = bb_badge[3] - bb_badge[1]

bx = (W - bw) // 2 - 15
by = 235

# Red background box
pad = 12
draw.rectangle(
    [(bx - pad, by - pad), (bx + bw + pad, by + bh + pad + 5)],
    fill=(180, 0, 0),
    outline=(255, 50, 50),
    width=3
)

# Text
for ox in range(-2, 3):
    for oy in range(-2, 3):
        draw.text((bx + ox, by + oy), badge_text, fill=(0, 0, 0), font=badge_font)
draw.text((bx, by), badge_text, fill=(255, 255, 255), font=badge_font)

# Diagonal strike-through
draw.line(
    [(bx - pad - 5, by + bh + pad + 10), (bx + bw + pad + 5, by - pad - 5)],
    fill=(255, 50, 50), width=5
)

# === "PURE RUST  •  PURE CPU" ===
sub_text = "PURE RUST  ·  PURE CPU"
bb_sub = draw.textbbox((0, 0), sub_text, font=font_med)
tw_sub = bb_sub[2] - bb_sub[0]
x_sub = (W - tw_sub) // 2
y_sub = 320

for ox in range(-2, 3):
    for oy in range(-2, 3):
        draw.text((x_sub + ox, y_sub + oy), sub_text, fill=(0, 0, 0), font=font_med)
draw.text((x_sub, y_sub), sub_text, fill=(222, 130, 50), font=font_med)

# === STATS BAR at bottom ===
bar_y = 545
bar_h = 110

# Dark bar
draw.rectangle([(0, bar_y), (W, bar_y + bar_h)], fill=(3, 3, 12))
draw.line([(0, bar_y), (W, bar_y)], fill=(0, 180, 255), width=2)
draw.line([(0, bar_y + bar_h), (W, bar_y + bar_h)], fill=(0, 180, 255), width=2)

# Scanline effect on bar
for sy in range(bar_y, bar_y + bar_h, 3):
    draw.line([(0, sy), (W, sy)], fill=(0, 0, 0))

stats = [
    ("12", "3D SCENES", (0, 220, 255)),
    ("6 MB", "ENTIRE OS", (255, 180, 50)),
    ("99K", "LINES RUST", (255, 100, 100)),
    ("0", "GPU CALLS", (100, 255, 150)),
]

stat_w = W // len(stats)
for i, (val, label, color) in enumerate(stats):
    cx = i * stat_w + stat_w // 2

    # Value
    bb = draw.textbbox((0, 0), val, font=font_stat)
    vw = bb[2] - bb[0]
    for ox in range(-2, 3):
        for oy in range(-2, 3):
            draw.text((cx - vw // 2 + ox, bar_y + 8 + oy), val, fill=(0, 0, 0), font=font_stat)
    draw.text((cx - vw // 2, bar_y + 8), val, fill=color, font=font_stat)

    # Label
    bb = draw.textbbox((0, 0), label, font=font_small)
    lw = bb[2] - bb[0]
    draw.text((cx - lw // 2, bar_y + 68), label, fill=(150, 150, 180), font=font_small)

# === TRUSTOS watermark bottom-left ===
logo = "TRUSTOS"
for ox in range(-2, 3):
    for oy in range(-2, 3):
        draw.text((20 + ox, 670 + oy), logo, fill=(0, 20, 40), font=font_small)
draw.text((20, 670), logo, fill=(0, 160, 220), font=font_small)

# === "WIREFRAME" tag bottom-right ===
tag = "WIREFRAME"
bb_tag = draw.textbbox((0, 0), tag, font=font_tag)
tw_tag = bb_tag[2] - bb_tag[0]
for ox in range(-2, 3):
    for oy in range(-2, 3):
        draw.text((W - tw_tag - 25 + ox, 672 + oy), tag, fill=(0, 0, 0), font=font_tag)
draw.text((W - tw_tag - 25, 672), tag, fill=(0, 255, 200), font=font_tag)

# === Sparkle particles ===
for _ in range(80):
    sx = random.randint(0, W)
    sy = random.randint(0, H - 120)
    brightness = random.randint(80, 255)
    size = random.choice([1, 1, 2])
    draw.ellipse([sx, sy, sx + size, sy + size],
                 fill=(brightness, brightness, brightness))

# === SAVE ===
out_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "thumbnail_wireframe.png")
img.save(out_path, "PNG", quality=95)
print(f"Thumbnail saved: {out_path}")
print(f"Size: {os.path.getsize(out_path) / 1024:.0f} KB")
print(f"Resolution: {W}x{H}")
