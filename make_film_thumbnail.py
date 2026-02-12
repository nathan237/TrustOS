"""
TrustOS Film Release Thumbnail — "120,000 lines. ONE person. Zero C."
Cinematic, high-impact thumbnail for the film release video.
"""
from PIL import Image, ImageDraw, ImageFont, ImageFilter, ImageEnhance
import numpy as np
import os

W, H = 1280, 720
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))


def make_film_thumbnail():
    pixels = np.zeros((H, W, 3), dtype=np.uint8)
    rng = np.random.RandomState(77)

    # ── Matrix rain background (subtle green) ──
    for col in range(0, W, 5):
        brightness = rng.randint(3, 16)
        length = rng.randint(100, 600)
        start = rng.randint(0, H)
        for y in range(start, min(start + length, H)):
            fade = 1.0 - (y - start) / length
            v = int(brightness * fade)
            pixels[y, col:min(col + 2, W), 1] = v

    # ── Orange/warm radial glow (left center — behind text) ──
    Y, X = np.ogrid[:H, :W]
    cx1, cy1 = W * 0.28, H * 0.45
    dist1 = np.sqrt(((X - cx1) / (W * 0.45)) ** 2 + ((Y - cy1) / (H * 0.55)) ** 2)
    glow1 = np.clip(1.0 - dist1, 0, 1) ** 1.6
    pixels[:, :, 0] = np.clip(pixels[:, :, 0] + glow1 * 12, 0, 255).astype(np.uint8)
    pixels[:, :, 1] = np.clip(pixels[:, :, 1] + glow1 * 22, 0, 255).astype(np.uint8)

    # ── Cyan/blue glow (right — behind screenshot) ──
    cx2, cy2 = W * 0.78, H * 0.48
    dist2 = np.sqrt(((X - cx2) / (W * 0.30)) ** 2 + ((Y - cy2) / (H * 0.45)) ** 2)
    glow2 = np.clip(1.0 - dist2, 0, 1) ** 1.8
    pixels[:, :, 1] = np.clip(pixels[:, :, 1] + glow2 * 10, 0, 255).astype(np.uint8)
    pixels[:, :, 2] = np.clip(pixels[:, :, 2] + glow2 * 14, 0, 255).astype(np.uint8)

    # ── Center horizontal spotlight ──
    cy3 = H * 0.42
    dist3 = np.abs(Y - cy3) / (H * 0.30)
    glow3 = np.clip(1.0 - dist3, 0, 1) ** 2.5
    pixels[:, :, 0] = np.clip(pixels[:, :, 0] + glow3 * 4, 0, 255).astype(np.uint8)
    pixels[:, :, 1] = np.clip(pixels[:, :, 1] + glow3 * 6, 0, 255).astype(np.uint8)
    pixels[:, :, 2] = np.clip(pixels[:, :, 2] + glow3 * 3, 0, 255).astype(np.uint8)

    img = Image.fromarray(pixels)
    draw = ImageDraw.Draw(img)

    # ── Screenshot on right side ──
    screenshot_path = None
    for name in ['screen_desktop.ppm', 'screen_showcase3.ppm', 'screen_showcase2.ppm',
                 'screen_trustos.ppm', 'screen_trustcode5.ppm', 'screen.ppm']:
        p = os.path.join(SCRIPT_DIR, name)
        if os.path.exists(p):
            screenshot_path = p
            break

    if screenshot_path:
        try:
            ss = Image.open(screenshot_path).convert('RGB')
            ss = ImageEnhance.Contrast(ss).enhance(1.35)
            ss = ImageEnhance.Color(ss).enhance(1.2)
            ss = ImageEnhance.Brightness(ss).enhance(1.1)

            ss_w = 520
            ss_h = int(ss_w * ss.size[1] / ss.size[0])
            if ss_h > 440:
                ss_h = 440
                ss_w = int(ss_h * ss.size[0] / ss.size[1])
            ss = ss.resize((ss_w, ss_h), Image.LANCZOS)

            sx = W - ss_w - 50
            sy = (H - ss_h) // 2 + 5

            # Green glow border
            glow_rect = Image.new('RGBA', (ss_w + 40, ss_h + 40), (0, 0, 0, 0))
            glow_d = ImageDraw.Draw(glow_rect)
            glow_d.rounded_rectangle([0, 0, glow_rect.width - 1, glow_rect.height - 1],
                                     radius=10, fill=(0, 255, 100, 40), outline=(0, 255, 100, 120), width=2)
            glow_rect = glow_rect.filter(ImageFilter.GaussianBlur(12))
            glow_rgb = glow_rect.convert('RGB')
            img_arr = np.array(img).astype(np.float32)
            g_arr = np.array(glow_rgb).astype(np.float32)
            gy, gx = sy - 20, sx - 20
            gh, gw = glow_rgb.size[1], glow_rgb.size[0]
            ey, ex = min(gy + gh, H), min(gx + gw, W)
            if gy >= 0 and gx >= 0:
                img_arr[gy:ey, gx:ex] = np.clip(
                    img_arr[gy:ey, gx:ex] + g_arr[:ey - gy, :ex - gx] * 0.6, 0, 255)
            img = Image.fromarray(img_arr.astype(np.uint8))
            draw = ImageDraw.Draw(img)

            img.paste(ss, (sx, sy))
            draw = ImageDraw.Draw(img)

            # Corner brackets (green accent)
            accent = (0, 255, 120)
            cl = 24
            for cx_, cy_, dx, dy in [
                (sx - 3, sy - 3, 1, 1), (sx + ss_w + 3, sy - 3, -1, 1),
                (sx - 3, sy + ss_h + 3, 1, -1), (sx + ss_w + 3, sy + ss_h + 3, -1, -1)
            ]:
                draw.line([(cx_, cy_), (cx_ + cl * dx, cy_)], fill=accent, width=3)
                draw.line([(cx_, cy_), (cx_, cy_ + cl * dy)], fill=accent, width=3)
        except Exception as e:
            print(f"  [!] Screenshot error: {e}")

    # ── Fonts ──
    try:
        font_massive = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 88)
        font_huge = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 72)
        font_big = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 58)
        font_mid = ImageFont.truetype("C:\\Windows\\Fonts\\arialbd.ttf", 34)
        font_small = ImageFont.truetype("C:\\Windows\\Fonts\\arial.ttf", 24)
        font_tag = ImageFont.truetype("C:\\Windows\\Fonts\\consolab.ttf", 21)
    except:
        font_massive = ImageFont.load_default()
        font_huge = font_massive
        font_big = font_massive
        font_mid = font_massive
        font_small = font_massive
        font_tag = font_massive

    tx = 38

    # ── Line 1: "120,000 LINES" — green neon glow (hero number) ──
    ty = 50
    _draw_glow_text(img, draw, (tx, ty), "120,000", font_massive, (0, 255, 90))
    draw = ImageDraw.Draw(img)
    num_w = draw.textbbox((0, 0), "120,000 ", font=font_massive)[2]
    _draw_text_outline(draw, (tx + num_w, ty + 12), "LINES", font_huge, (255, 255, 255),
                       outline_color=(0, 0, 0), thickness=4)

    # ── Line 2: "ONE PERSON." — white bold ──
    ty += 100
    _draw_text_outline(draw, (tx, ty), "ONE PERSON.", font_huge, (255, 255, 255),
                       outline_color=(0, 0, 0), thickness=4)

    # ── Line 3: "ZERO C." — orange/rust accent ──
    ty += 90
    _draw_glow_text(img, draw, (tx, ty), "ZERO C.", font_huge, (255, 130, 40))
    draw = ImageDraw.Draw(img)

    # ── Subtitle: "A complete OS — from scratch — in pure Rust" ──
    ty += 95
    _draw_text_outline(draw, (tx, ty), "A complete OS.", font_mid, (180, 180, 180),
                       outline_color=(0, 0, 0), thickness=3)
    ty += 40
    _draw_text_outline(draw, (tx, ty), "From scratch. In pure Rust.", font_mid, (140, 140, 140),
                       outline_color=(0, 0, 0), thickness=2)

    # ── Stats accent line ──
    ty += 50
    draw.line([(tx, ty), (420, ty)], fill=(0, 200, 80), width=2)
    lg = Image.new('RGB', (W, H), (0, 0, 0))
    ImageDraw.Draw(lg).line([(tx, ty), (420, ty)], fill=(0, 220, 90), width=6)
    lg = lg.filter(ImageFilter.GaussianBlur(7))
    img_arr = np.array(img).astype(np.float32)
    img = Image.fromarray(np.clip(img_arr + np.array(lg).astype(np.float32) * 0.6, 0, 255).astype(np.uint8))
    draw = ImageDraw.Draw(img)

    # ── Bottom tag bar ──
    bar_y = H - 56
    draw.rectangle([0, bar_y - 2, W, H], fill=(0, 8, 4))
    draw.line([(0, bar_y - 2), (W, bar_y - 2)], fill=(0, 160, 70), width=2)

    tags = ["Rust", "bare-metal", "x86_64", "no_std", "8 days", "10 MB ISO"]
    tag_x = 22
    for tag in tags:
        bbox = draw.textbbox((0, 0), tag, font=font_tag)
        tw = bbox[2] - bbox[0]
        draw.rounded_rectangle([tag_x - 5, bar_y + 10, tag_x + tw + 5, bar_y + 38],
                               radius=4, fill=(0, 35, 18), outline=(0, 130, 55))
        draw.text((tag_x, bar_y + 12), tag, font=font_tag, fill=(0, 210, 85))
        tag_x += tw + 20

    # GitHub bottom-right
    gh_text = "github.com/nathan237/TrustOS"
    gh_bbox = draw.textbbox((0, 0), gh_text, font=font_tag)
    gh_w = gh_bbox[2] - gh_bbox[0]
    draw.text((W - gh_w - 22, bar_y + 13), gh_text, font=font_tag, fill=(80, 80, 80))

    # ── Vignette ──
    v_arr = np.array(img).astype(np.float32)
    Y, X = np.ogrid[:H, :W]
    vd = np.sqrt(((X - W * 0.40) / (W * 0.62)) ** 2 + ((Y - H * 0.46) / (H * 0.62)) ** 2)
    vm = np.clip(1.0 - (vd - 0.35) * 1.1, 0.20, 1.0)
    v_arr *= vm[:, :, np.newaxis]
    img = Image.fromarray(np.clip(v_arr, 0, 255).astype(np.uint8))

    # ── Save ──
    output = os.path.join(SCRIPT_DIR, "thumbnail_film.png")
    img.save(output, quality=95)
    size_kb = os.path.getsize(output) // 1024
    print(f"  Thumbnail saved: {output}")
    print(f"     {W}x{H} — {size_kb} KB")
    return output


def _draw_text_outline(draw, pos, text, font, fill, outline_color=(0, 0, 0), thickness=3):
    x, y = pos
    for dx in range(-thickness, thickness + 1):
        for dy in range(-thickness, thickness + 1):
            if dx * dx + dy * dy <= thickness * thickness:
                draw.text((x + dx, y + dy), text, font=font, fill=outline_color)
    draw.text((x, y), text, font=font, fill=fill)


def _draw_glow_text(img, draw, pos, text, font, color):
    x, y = pos
    for blur_r, alpha in [(20, 0.3), (12, 0.45), (5, 0.65)]:
        glow = Image.new('RGB', (W, H), (0, 0, 0))
        ImageDraw.Draw(glow).text((x, y), text, font=font, fill=color)
        glow = glow.filter(ImageFilter.GaussianBlur(blur_r))
        img_arr = np.array(img).astype(np.float32)
        g_arr = np.array(glow).astype(np.float32)
        img.paste(Image.fromarray(np.clip(img_arr + g_arr * alpha, 0, 255).astype(np.uint8)))
    draw = ImageDraw.Draw(img)
    _draw_text_outline(draw, pos, text, font, color, outline_color=(0, 0, 0), thickness=4)
    return draw


if __name__ == "__main__":
    make_film_thumbnail()
