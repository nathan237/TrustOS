"""
TrustLang YouTube Thumbnail — "Write. Compile. Run. Same OS."
Highlights: code, compile, and run all happen inside the OS itself.
1280x720, catchy tech-dev style.
"""
from PIL import Image, ImageDraw, ImageFont, ImageFilter, ImageEnhance
import numpy as np
import os, math

W, H = 1280, 720


def _draw_text_outline(draw, pos, text, font, fill, outline_color=(0, 0, 0), thickness=3):
    x, y = pos
    for dx in range(-thickness, thickness + 1):
        for dy in range(-thickness, thickness + 1):
            if dx * dx + dy * dy <= thickness * thickness:
                draw.text((x + dx, y + dy), text, font=font, fill=outline_color)
    draw.text((x, y), text, font=font, fill=fill)


def _draw_glow_text(img, draw, pos, text, font, color, intensity=0.6):
    x, y = pos
    for blur_r, alpha in [(20, 0.25 * intensity), (12, 0.4 * intensity), (5, 0.7 * intensity)]:
        glow = Image.new('RGB', (W, H), (0, 0, 0))
        ImageDraw.Draw(glow).text((x, y), text, font=font, fill=color)
        glow = glow.filter(ImageFilter.GaussianBlur(blur_r))
        img_arr = np.array(img).astype(np.float32)
        g_arr = np.array(glow).astype(np.float32)
        img.paste(Image.fromarray(np.clip(img_arr + g_arr * alpha, 0, 255).astype(np.uint8)))
    draw = ImageDraw.Draw(img)
    _draw_text_outline(draw, pos, text, font, color, outline_color=(0, 0, 0), thickness=4)
    return draw


def make_thumbnail():
    rng = np.random.RandomState(42)

    # ── Background: dark gradient ──
    pixels = np.zeros((H, W, 3), dtype=np.uint8)
    for y in range(H):
        t = y / H
        pixels[y, :, 0] = int(6 + 10 * t)
        pixels[y, :, 1] = int(4 + 8 * t)
        pixels[y, :, 2] = int(18 + 22 * t)

    # ── Code rain columns (subtle, purple/cyan tinted) ──
    code_chars = "fn let mut if else return while for loop struct impl pub mod use match => {} () ; : -> // #"
    for col in range(0, W, 8):
        brightness = rng.randint(3, 16)
        length = rng.randint(60, 450)
        start = rng.randint(0, H)
        hue = rng.choice(['cyan', 'purple', 'green'])
        for y in range(start, min(start + length, H)):
            fade = 1.0 - (y - start) / length
            v = int(brightness * fade)
            if hue == 'cyan':
                pixels[y, col:min(col + 2, W), 1] = np.clip(pixels[y, col:min(col + 2, W), 1].astype(int) + v, 0, 255).astype(np.uint8)
                pixels[y, col:min(col + 2, W), 2] = np.clip(pixels[y, col:min(col + 2, W), 2].astype(int) + v, 0, 255).astype(np.uint8)
            elif hue == 'purple':
                pixels[y, col:min(col + 2, W), 0] = np.clip(pixels[y, col:min(col + 2, W), 0].astype(int) + v // 2, 0, 255).astype(np.uint8)
                pixels[y, col:min(col + 2, W), 2] = np.clip(pixels[y, col:min(col + 2, W), 2].astype(int) + v, 0, 255).astype(np.uint8)
            else:
                pixels[y, col:min(col + 2, W), 1] = np.clip(pixels[y, col:min(col + 2, W), 1].astype(int) + v, 0, 255).astype(np.uint8)

    # ── Radial glows ──
    Y, X = np.ogrid[:H, :W]

    # Left glow: warm orange-gold (behind title)
    cx1, cy1 = W * 0.28, H * 0.40
    dist1 = np.sqrt(((X - cx1) / (W * 0.45)) ** 2 + ((Y - cy1) / (H * 0.55)) ** 2)
    glow1 = np.clip(1.0 - dist1, 0, 1) ** 2.0
    pixels[:, :, 0] = np.clip(pixels[:, :, 0].astype(np.float32) + glow1 * 18, 0, 255).astype(np.uint8)
    pixels[:, :, 1] = np.clip(pixels[:, :, 1].astype(np.float32) + glow1 * 10, 0, 255).astype(np.uint8)

    # Right glow: cyan/teal (behind screenshot)
    cx2, cy2 = W * 0.78, H * 0.50
    dist2 = np.sqrt(((X - cx2) / (W * 0.30)) ** 2 + ((Y - cy2) / (H * 0.50)) ** 2)
    glow2 = np.clip(1.0 - dist2, 0, 1) ** 2.0
    pixels[:, :, 1] = np.clip(pixels[:, :, 1].astype(np.float32) + glow2 * 14, 0, 255).astype(np.uint8)
    pixels[:, :, 2] = np.clip(pixels[:, :, 2].astype(np.float32) + glow2 * 10, 0, 255).astype(np.uint8)

    # Center accent glow (purple)
    cx3, cy3 = W * 0.50, H * 0.55
    dist3 = np.sqrt(((X - cx3) / (W * 0.60)) ** 2 + ((Y - cy3) / (H * 0.70)) ** 2)
    glow3 = np.clip(1.0 - dist3, 0, 1) ** 2.5
    pixels[:, :, 0] = np.clip(pixels[:, :, 0].astype(np.float32) + glow3 * 6, 0, 255).astype(np.uint8)
    pixels[:, :, 2] = np.clip(pixels[:, :, 2].astype(np.float32) + glow3 * 10, 0, 255).astype(np.uint8)

    img = Image.fromarray(pixels)
    draw = ImageDraw.Draw(img)

    # ── Screenshot on the right ──
    screenshot_path = None
    for name in ['screen_trustlang.ppm', 'screen_trustlang.png', 'screen_trustcode5.ppm',
                 'screen_trustcode4.ppm', 'screen_showcase3.ppm']:
        p = os.path.join(os.path.dirname(__file__), name)
        if os.path.exists(p):
            screenshot_path = p
            break

    if screenshot_path:
        try:
            ss = Image.open(screenshot_path).convert('RGB')
            ss = ImageEnhance.Contrast(ss).enhance(1.3)
            ss = ImageEnhance.Color(ss).enhance(1.15)
            ss = ImageEnhance.Brightness(ss).enhance(1.1)

            ss_w = 520
            ss_h = int(ss_w * ss.size[1] / ss.size[0])
            if ss_h > 440:
                ss_h = 440
                ss_w = int(ss_h * ss.size[0] / ss.size[1])
            ss = ss.resize((ss_w, ss_h), Image.LANCZOS)

            sx = W - ss_w - 40
            sy = (H - ss_h) // 2 + 15

            # Cyan glow border
            glow_rect = Image.new('RGBA', (ss_w + 44, ss_h + 44), (0, 0, 0, 0))
            glow_d = ImageDraw.Draw(glow_rect)
            glow_d.rounded_rectangle([0, 0, glow_rect.width - 1, glow_rect.height - 1],
                                     radius=12, fill=(0, 200, 255, 30), outline=(0, 200, 255, 120), width=2)
            glow_rect = glow_rect.filter(ImageFilter.GaussianBlur(12))
            glow_rgb = glow_rect.convert('RGB')
            img_arr = np.array(img).astype(np.float32)
            g_arr = np.array(glow_rgb).astype(np.float32)
            gy, gx = sy - 22, sx - 22
            gh, gw = glow_rgb.size[1], glow_rgb.size[0]
            ey, ex = min(gy + gh, H), min(gx + gw, W)
            if gy >= 0 and gx >= 0:
                img_arr[gy:ey, gx:ex] = np.clip(
                    img_arr[gy:ey, gx:ex] + g_arr[:ey - gy, :ex - gx] * 0.5, 0, 255)
            img = Image.fromarray(img_arr.astype(np.uint8))
            draw = ImageDraw.Draw(img)

            img.paste(ss, (sx, sy))
            draw = ImageDraw.Draw(img)

            # Corner brackets (cyan accent)
            accent = (0, 220, 255)
            cl = 24
            for cx_, cy_, ddx, ddy in [
                (sx - 3, sy - 3, 1, 1), (sx + ss_w + 3, sy - 3, -1, 1),
                (sx - 3, sy + ss_h + 3, 1, -1), (sx + ss_w + 3, sy + ss_h + 3, -1, -1)
            ]:
                draw.line([(cx_, cy_), (cx_ + cl * ddx, cy_)], fill=accent, width=3)
                draw.line([(cx_, cy_), (cx_, cy_ + cl * ddy)], fill=accent, width=3)
        except Exception as e:
            print(f"  [!] Screenshot error: {e}")

    # ── Fonts ──
    try:
        font_huge = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 82)
        font_big = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 66)
        font_mid = ImageFont.truetype("C:\\Windows\\Fonts\\arialbd.ttf", 38)
        font_sub = ImageFont.truetype("C:\\Windows\\Fonts\\arialbd.ttf", 30)
        font_small = ImageFont.truetype("C:\\Windows\\Fonts\\arial.ttf", 24)
        font_tag = ImageFont.truetype("C:\\Windows\\Fonts\\consolab.ttf", 21)
        font_arrow = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 50)
    except:
        font_huge = ImageFont.load_default()
        font_big = font_huge
        font_mid = font_huge
        font_sub = font_huge
        font_small = font_huge
        font_tag = font_huge
        font_arrow = font_huge

    tx = 40

    # ── Line 1: "WRITE" — white ──
    ty = 30
    _draw_text_outline(draw, (tx, ty), "WRITE.", font_huge, (255, 255, 255),
                       outline_color=(0, 0, 0), thickness=4)

    # ── Line 2: "COMPILE." — orange/gold glow ──
    ty += 90
    _draw_glow_text(img, draw, (tx, ty), "COMPILE.", font_huge, (255, 180, 40), intensity=0.8)
    draw = ImageDraw.Draw(img)

    # ── Line 3: "RUN." — cyan neon glow ──
    ty += 90
    _draw_glow_text(img, draw, (tx, ty), "RUN.", font_huge, (0, 230, 255), intensity=0.9)
    draw = ImageDraw.Draw(img)

    # ── Arrows between the 3 words ──
    # Small " >> " connectors on the left side for visual flow
    arrow_color = (120, 120, 120)
    # Between WRITE and COMPILE
    draw.text((tx + 240, 55), ">>", font=font_tag, fill=(80, 80, 80))
    # Between COMPILE and RUN
    draw.text((tx + 310, 145), ">>", font=font_tag, fill=(80, 80, 80))

    # ── Line 4: "SAME OS." — big, green neon ──
    ty += 105
    _draw_glow_text(img, draw, (tx, ty), "SAME OS.", font_big, (0, 255, 100), intensity=1.0)
    draw = ImageDraw.Draw(img)

    # ── Subtitle: "No host. No VM. No emulator." ──
    ty += 80
    _draw_text_outline(draw, (tx, ty), "No host. No VM. No emulator.", font_sub, (190, 190, 210),
                       outline_color=(0, 0, 0), thickness=3)

    # ── Pipeline visualization: small boxes with arrows ──
    # CODE → COMPILE → BYTECODE → RUN  (all inside the OS)
    pipe_y = ty + 55
    pipe_x = tx
    stages = [
        ("CODE", (180, 120, 255)),     # purple
        ("COMPILE", (255, 180, 40)),   # orange
        ("BYTECODE", (0, 200, 255)),   # cyan
        ("RUN", (0, 255, 100)),        # green
    ]
    for i, (label, color) in enumerate(stages):
        bbox = draw.textbbox((0, 0), label, font=font_tag)
        bw = bbox[2] - bbox[0]
        bh = bbox[3] - bbox[1]
        rx = pipe_x
        ry = pipe_y
        # Box background
        draw.rounded_rectangle([rx - 4, ry - 2, rx + bw + 8, ry + bh + 8],
                               radius=4, fill=(color[0] // 10, color[1] // 10, color[2] // 10),
                               outline=color, width=2)
        draw.text((rx + 2, ry + 1), label, font=font_tag, fill=color)
        pipe_x += bw + 20
        # Arrow between boxes
        if i < len(stages) - 1:
            arrow_cx = pipe_x - 12
            arrow_cy = ry + bh // 2 + 3
            draw.text((arrow_cx - 4, arrow_cy - 8), "→", font=font_small, fill=(100, 100, 120))

    # ── "All inside the kernel" label under pipeline ──
    pipe_label_y = pipe_y + 35
    # Bracket around the pipeline
    bracket_left = tx - 6
    bracket_right = pipe_x - 12
    bracket_mid = pipe_label_y + 2
    draw.line([(bracket_left, pipe_y + 28), (bracket_left, bracket_mid + 4)], fill=(60, 60, 70), width=1)
    draw.line([(bracket_right, pipe_y + 28), (bracket_right, bracket_mid + 4)], fill=(60, 60, 70), width=1)
    draw.line([(bracket_left, bracket_mid + 4), (bracket_left + (bracket_right - bracket_left) // 2 - 60, bracket_mid + 4)], fill=(60, 60, 70), width=1)
    draw.line([(bracket_left + (bracket_right - bracket_left) // 2 + 95, bracket_mid + 4), (bracket_right, bracket_mid + 4)], fill=(60, 60, 70), width=1)

    all_label = "all inside the kernel"
    all_bbox = draw.textbbox((0, 0), all_label, font=font_small)
    all_w = all_bbox[2] - all_bbox[0]
    all_x = bracket_left + (bracket_right - bracket_left) // 2 - all_w // 2
    draw.text((all_x, bracket_mid - 3), all_label, font=font_small, fill=(100, 255, 140))

    # ── Bottom info bar ──
    bar_y = H - 55
    draw.rectangle([0, bar_y - 2, W, H], fill=(4, 4, 12))
    draw.line([(0, bar_y - 2), (W, bar_y - 2)], fill=(0, 180, 255), width=2)

    tags = ["TrustLang", "Rust", "bare-metal", "bytecode VM", "0 dependencies"]
    tag_x = 22
    for tag in tags:
        bbox = draw.textbbox((0, 0), tag, font=font_tag)
        tw = bbox[2] - bbox[0]
        fill_color = (0, 30, 50) if tag != "TrustLang" else (40, 20, 0)
        outline_color = (0, 160, 220) if tag != "TrustLang" else (255, 160, 40)
        text_color = (0, 200, 255) if tag != "TrustLang" else (255, 180, 50)
        draw.rounded_rectangle([tag_x - 5, bar_y + 10, tag_x + tw + 7, bar_y + 38],
                               radius=4, fill=fill_color, outline=outline_color, width=1)
        draw.text((tag_x + 1, bar_y + 12), tag, font=font_tag, fill=text_color)
        tag_x += tw + 20

    # GitHub bottom-right
    gh_text = "github.com/nathan237/TrustOS"
    gh_bbox = draw.textbbox((0, 0), gh_text, font=font_tag)
    gh_w = gh_bbox[2] - gh_bbox[0]
    draw.text((W - gh_w - 22, bar_y + 14), gh_text, font=font_tag, fill=(80, 80, 90))

    # ── TrustOS logo top-right ──
    logo_text = "TRUSTOS"
    logo_bbox = draw.textbbox((0, 0), logo_text, font=font_mid)
    logo_w = logo_bbox[2] - logo_bbox[0]
    draw.text((W - logo_w - 45, 12), logo_text, font=font_mid, fill=(0, 180, 255))

    # ── Sparkle particles ──
    for _ in range(80):
        sx = rng.randint(0, W)
        sy = rng.randint(0, H)
        brightness = rng.randint(80, 220)
        size = rng.choice([1, 1, 1, 2])
        palette = [(brightness, brightness, brightness),
                   (0, brightness, brightness),
                   (brightness // 2, 0, brightness)]
        c = palette[rng.randint(0, len(palette))]
        draw.ellipse([sx, sy, sx + size, sy + size], fill=c)

    # ── Vignette ──
    v_arr = np.array(img).astype(np.float32)
    Y, X = np.ogrid[:H, :W]
    vd = np.sqrt(((X - W * 0.42) / (W * 0.65)) ** 2 + ((Y - H * 0.48) / (H * 0.65)) ** 2)
    vm = np.clip(1.0 - (vd - 0.35) * 0.9, 0.2, 1.0)
    v_arr *= vm[:, :, np.newaxis]
    img = Image.fromarray(np.clip(v_arr, 0, 255).astype(np.uint8))

    # ── Save ──
    output = os.path.join(os.path.dirname(__file__), "thumbnail_trustlang.png")
    img.save(output, quality=95)
    size_kb = os.path.getsize(output) // 1024
    print(f"  Thumbnail saved: {output}")
    print(f"     {W}x{H} - {size_kb} KB")
    return output


if __name__ == "__main__":
    make_thumbnail()
