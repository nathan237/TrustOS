"""
TrustOS YouTube Thumbnail — "I Built 3D Chess Inside My Own OS"
Chess-themed thumbnail with dramatic text and screenshot.
"""
from PIL import Image, ImageDraw, ImageFont, ImageFilter, ImageEnhance
import numpy as np
import os

W, H = 1280, 720
BASE = os.path.dirname(os.path.abspath(__file__))

def make_thumbnail():
    pixels = np.zeros((H, W, 3), dtype=np.uint8)
    rng = np.random.RandomState(77)

    # ── Matrix rain background ──
    for col in range(0, W, 5):
        brightness = rng.randint(3, 18)
        length = rng.randint(60, 500)
        start = rng.randint(0, H)
        for y in range(start, min(start + length, H)):
            fade = 1.0 - (y - start) / length
            v = int(brightness * fade)
            pixels[y, col:min(col + 2, W), 1] = v

    # ── Chess pattern overlay (subtle, bottom-right) ──
    sq = 40
    for r in range(H // sq + 1):
        for c in range(W // sq + 1):
            if (r + c) % 2 == 0:
                y0, y1 = r * sq, min((r + 1) * sq, H)
                x0, x1 = c * sq, min((c + 1) * sq, W)
                # Very faint chessboard
                dist_from_center = abs(c * sq - W * 0.7) / W + abs(r * sq - H * 0.5) / H
                alpha = max(0, 0.06 - dist_from_center * 0.08)
                if alpha > 0:
                    pixels[y0:y1, x0:x1, 1] = np.clip(
                        pixels[y0:y1, x0:x1, 1].astype(float) + alpha * 40, 0, 255
                    ).astype(np.uint8)

    # ── Warm glow left (text area) ──
    Y, X = np.ogrid[:H, :W]
    cx1, cy1 = W * 0.28, H * 0.42
    dist1 = np.sqrt(((X - cx1) / (W * 0.48)) ** 2 + ((Y - cy1) / (H * 0.55)) ** 2)
    glow1 = np.clip(1.0 - dist1, 0, 1) ** 1.6
    pixels[:, :, 0] = np.clip(pixels[:, :, 0].astype(float) + glow1 * 6, 0, 255).astype(np.uint8)
    pixels[:, :, 1] = np.clip(pixels[:, :, 1].astype(float) + glow1 * 18, 0, 255).astype(np.uint8)

    # ── Cool glow right (screenshot area) ──
    cx2, cy2 = W * 0.78, H * 0.48
    dist2 = np.sqrt(((X - cx2) / (W * 0.30)) ** 2 + ((Y - cy2) / (H * 0.48)) ** 2)
    glow2 = np.clip(1.0 - dist2, 0, 1) ** 2
    pixels[:, :, 1] = np.clip(pixels[:, :, 1].astype(float) + glow2 * 16, 0, 255).astype(np.uint8)
    pixels[:, :, 2] = np.clip(pixels[:, :, 2].astype(float) + glow2 * 6, 0, 255).astype(np.uint8)

    img = Image.fromarray(pixels)
    draw = ImageDraw.Draw(img)

    # ── Screenshot right side ──
    screenshot_path = None
    for name in ['screen_showcase3.ppm', 'screen_showcase2.ppm', 'screen_showcase.ppm',
                 'screen_desktop.ppm', 'screen_trustos.ppm', 'screen.ppm']:
        p = os.path.join(BASE, name)
        if os.path.exists(p):
            screenshot_path = p
            break

    if screenshot_path:
        try:
            ss = Image.open(screenshot_path).convert('RGB')
            ss = ImageEnhance.Contrast(ss).enhance(1.3)
            ss = ImageEnhance.Color(ss).enhance(1.15)
            ss = ImageEnhance.Brightness(ss).enhance(1.1)

            ss_w = 540
            ss_h = int(ss_w * ss.size[1] / ss.size[0])
            if ss_h > 460:
                ss_h = 460
                ss_w = int(ss_h * ss.size[0] / ss.size[1])
            ss = ss.resize((ss_w, ss_h), Image.LANCZOS)

            sx = W - ss_w - 50
            sy = (H - ss_h) // 2 + 15

            # Green glow border
            glow_rect = Image.new('RGBA', (ss_w + 40, ss_h + 40), (0, 0, 0, 0))
            glow_d = ImageDraw.Draw(glow_rect)
            glow_d.rounded_rectangle([0, 0, glow_rect.width - 1, glow_rect.height - 1],
                                     radius=10, fill=(0, 255, 100, 30), outline=(0, 255, 100, 90), width=2)
            glow_rect = glow_rect.filter(ImageFilter.GaussianBlur(10))
            glow_rgb = glow_rect.convert('RGB')
            img_arr = np.array(img).astype(np.float32)
            g_arr = np.array(glow_rgb).astype(np.float32)
            gy, gx = sy - 20, sx - 20
            gh_, gw_ = glow_rgb.size[1], glow_rgb.size[0]
            ey, ex = min(gy + gh_, H), min(gx + gw_, W)
            if gy >= 0 and gx >= 0:
                img_arr[gy:ey, gx:ex] = np.clip(
                    img_arr[gy:ey, gx:ex] + g_arr[:ey - gy, :ex - gx] * 0.5, 0, 255)
            img = Image.fromarray(img_arr.astype(np.uint8))
            draw = ImageDraw.Draw(img)

            img.paste(ss, (sx, sy))
            draw = ImageDraw.Draw(img)

            # Corner brackets
            accent = (0, 255, 120)
            cl = 24
            for cx_, cy_, ddx, ddy in [
                (sx - 2, sy - 2, 1, 1), (sx + ss_w + 2, sy - 2, -1, 1),
                (sx - 2, sy + ss_h + 2, 1, -1), (sx + ss_w + 2, sy + ss_h + 2, -1, -1)
            ]:
                draw.line([(cx_, cy_), (cx_ + cl * ddx, cy_)], fill=accent, width=3)
                draw.line([(cx_, cy_), (cx_, cy_ + cl * ddy)], fill=accent, width=3)
        except Exception as e:
            print(f"  [!] Screenshot error: {e}")

    # ── Fonts ──
    try:
        font_huge = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 76)
        font_big = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 60)
        font_mid = ImageFont.truetype("C:\\Windows\\Fonts\\arialbd.ttf", 34)
        font_small = ImageFont.truetype("C:\\Windows\\Fonts\\arial.ttf", 24)
        font_tag = ImageFont.truetype("C:\\Windows\\Fonts\\consolab.ttf", 21)
        font_sub = ImageFont.truetype("C:\\Windows\\Fonts\\ariali.ttf", 28)
    except:
        font_huge = font_big = font_mid = font_small = font_tag = font_sub = ImageFont.load_default()

    tx = 40

    # ── Line 1: "I BUILT" in white ──
    ty = 55
    _draw_text_outline(draw, (tx, ty), "I BUILT", font_huge, (255, 255, 255),
                       outline_color=(0, 0, 0), thickness=4)

    # ── Line 2: "3D CHESS" in green neon glow ──
    ty += 85
    _draw_glow_text(img, draw, (tx, ty), "3D CHESS", font_huge, (0, 255, 90))
    draw = ImageDraw.Draw(img)

    # ── Line 3: "INSIDE MY" white + "OWN OS" orange ──
    ty += 85
    _draw_text_outline(draw, (tx, ty), "INSIDE MY", font_big, (255, 255, 255),
                       outline_color=(0, 0, 0), thickness=4)
    w_inside = draw.textbbox((0, 0), "INSIDE MY ", font=font_big)[2]
    _draw_text_outline(draw, (tx + w_inside, ty), "OWN OS", font_big, (255, 140, 40),
                       outline_color=(0, 0, 0), thickness=4)

    # ── Line 4: subtitle ──
    ty += 75
    _draw_text_outline(draw, (tx, ty), "from scratch. no libraries.", font_mid, (180, 180, 180),
                       outline_color=(0, 0, 0), thickness=3)

    # ── Chess piece emoji/symbol area ──
    ty += 55
    try:
        font_chess = ImageFont.truetype("C:\\Windows\\Fonts\\seguisym.ttf", 50)
        pieces = "\u265A \u265B \u265C \u265D \u265E \u265F"
        _draw_text_outline(draw, (tx, ty), pieces, font_chess, (0, 200, 80),
                           outline_color=(0, 0, 0), thickness=2)
    except:
        pass

    # ── Bottom info bar ──
    bar_y = H - 58
    draw.rectangle([0, bar_y - 2, W, H], fill=(0, 8, 4))
    draw.line([(0, bar_y - 2), (W, bar_y - 2)], fill=(0, 160, 70), width=2)

    tags = ["Rust", "bare-metal", "x86_64", "no_std", "3D engine"]
    tag_x = 25
    for tag in tags:
        bbox = draw.textbbox((0, 0), tag, font=font_tag)
        tw = bbox[2] - bbox[0]
        draw.rounded_rectangle([tag_x - 6, bar_y + 10, tag_x + tw + 6, bar_y + 38],
                               radius=4, fill=(0, 35, 18), outline=(0, 130, 55))
        draw.text((tag_x, bar_y + 11), tag, font=font_tag, fill=(0, 210, 85))
        tag_x += tw + 22

    # GitHub bottom-right
    gh_text = "github.com/nathan237/TrustOS"
    gh_bbox = draw.textbbox((0, 0), gh_text, font=font_tag)
    gh_w = gh_bbox[2] - gh_bbox[0]
    draw.text((W - gh_w - 25, bar_y + 13), gh_text, font=font_tag, fill=(90, 90, 90))

    # ── Green separator ──
    sep_y = bar_y - 18
    draw.line([(tx, sep_y), (480, sep_y)], fill=(0, 180, 70), width=2)
    lg = Image.new('RGB', (W, H), (0, 0, 0))
    ImageDraw.Draw(lg).line([(tx, sep_y), (480, sep_y)], fill=(0, 200, 80), width=5)
    lg = lg.filter(ImageFilter.GaussianBlur(6))
    img_arr = np.array(img).astype(np.float32)
    img = Image.fromarray(np.clip(img_arr + np.array(lg).astype(np.float32) * 0.7, 0, 255).astype(np.uint8))

    # ── Vignette ──
    v_arr = np.array(img).astype(np.float32)
    vd = np.sqrt(((X - W * 0.42) / (W * 0.65)) ** 2 + ((Y - H * 0.48) / (H * 0.65)) ** 2)
    vm = np.clip(1.0 - (vd - 0.4) * 1.0, 0.25, 1.0)
    v_arr *= vm[:, :, np.newaxis]
    img = Image.fromarray(np.clip(v_arr, 0, 255).astype(np.uint8))

    # ── Save ──
    output = os.path.join(BASE, "thumbnail_chess3d.png")
    img.save(output, quality=95)
    size_kb = os.path.getsize(output) // 1024
    print(f"  Thumbnail saved: {output}")
    print(f"     {W}x{H} - {size_kb} KB")
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
    for blur_r, alpha in [(18, 0.35), (10, 0.5), (5, 0.7)]:
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
    make_thumbnail()
