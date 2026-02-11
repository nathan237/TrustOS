"""
TrustOS YouTube Thumbnail — "AI wrote it, I designed it" angle.
Response to haters: the vision is mine, AI is just the tool.
"""
from PIL import Image, ImageDraw, ImageFont, ImageFilter, ImageEnhance
import numpy as np
import os

W, H = 1280, 720

def make_thumbnail():
    pixels = np.zeros((H, W, 3), dtype=np.uint8)
    rng = np.random.RandomState(42)
    
    # Matrix rain background (subtle)
    for col in range(0, W, 6):
        brightness = rng.randint(4, 20)
        length = rng.randint(80, 500)
        start = rng.randint(0, H)
        for y in range(start, min(start + length, H)):
            fade = 1.0 - (y - start) / length
            v = int(brightness * fade)
            pixels[y, col:min(col+2, W), 1] = v
    
    # Warm radial glow left (where text is)
    Y, X = np.ogrid[:H, :W]
    cx1, cy1 = W * 0.30, H * 0.45
    dist1 = np.sqrt(((X - cx1) / (W * 0.5)) ** 2 + ((Y - cy1) / (H * 0.6)) ** 2)
    glow1 = np.clip(1.0 - dist1, 0, 1) ** 1.8
    pixels[:, :, 0] = np.clip(pixels[:, :, 0].astype(np.float32) + glow1 * 8, 0, 255).astype(np.uint8)
    pixels[:, :, 1] = np.clip(pixels[:, :, 1].astype(np.float32) + glow1 * 20, 0, 255).astype(np.uint8)
    
    # Cool glow right (behind screenshot)
    cx2, cy2 = W * 0.78, H * 0.5
    dist2 = np.sqrt(((X - cx2) / (W * 0.32)) ** 2 + ((Y - cy2) / (H * 0.5)) ** 2)
    glow2 = np.clip(1.0 - dist2, 0, 1) ** 2
    pixels[:, :, 1] = np.clip(pixels[:, :, 1].astype(np.float32) + glow2 * 12, 0, 255).astype(np.uint8)
    pixels[:, :, 2] = np.clip(pixels[:, :, 2].astype(np.float32) + glow2 * 8, 0, 255).astype(np.uint8)
    
    img = Image.fromarray(pixels)
    draw = ImageDraw.Draw(img)
    
    # ── Screenshot right side ──
    screenshot_path = None
    for name in ['screen_desktop.ppm', 'screen_showcase3.ppm', 'screen_showcase2.ppm',
                 'screen_trustos.ppm', 'screen_trustcode5.ppm', 'screen.ppm']:
        p = os.path.join(os.path.dirname(__file__), name)
        if os.path.exists(p):
            screenshot_path = p
            break
    
    if screenshot_path:
        try:
            ss = Image.open(screenshot_path).convert('RGB')
            ss = ImageEnhance.Contrast(ss).enhance(1.35)
            ss = ImageEnhance.Color(ss).enhance(1.2)
            ss = ImageEnhance.Brightness(ss).enhance(1.1)
            
            ss_w = 560
            ss_h = int(ss_w * ss.size[1] / ss.size[0])
            if ss_h > 480:
                ss_h = 480
                ss_w = int(ss_h * ss.size[0] / ss.size[1])
            ss = ss.resize((ss_w, ss_h), Image.LANCZOS)
            
            sx = W - ss_w - 45
            sy = (H - ss_h) // 2 + 10
            
            # Green glow border
            glow_rect = Image.new('RGBA', (ss_w + 40, ss_h + 40), (0, 0, 0, 0))
            glow_d = ImageDraw.Draw(glow_rect)
            glow_d.rounded_rectangle([0, 0, glow_rect.width-1, glow_rect.height-1],
                                     radius=10, fill=(0, 255, 100, 35), outline=(0, 255, 100, 100), width=2)
            glow_rect = glow_rect.filter(ImageFilter.GaussianBlur(10))
            glow_rgb = glow_rect.convert('RGB')
            img_arr = np.array(img).astype(np.float32)
            g_arr = np.array(glow_rgb).astype(np.float32)
            gy, gx = sy - 20, sx - 20
            gh, gw = glow_rgb.size[1], glow_rgb.size[0]
            ey, ex = min(gy + gh, H), min(gx + gw, W)
            if gy >= 0 and gx >= 0:
                img_arr[gy:ey, gx:ex] = np.clip(
                    img_arr[gy:ey, gx:ex] + g_arr[:ey-gy, :ex-gx] * 0.5, 0, 255)
            img = Image.fromarray(img_arr.astype(np.uint8))
            draw = ImageDraw.Draw(img)
            
            img.paste(ss, (sx, sy))
            draw = ImageDraw.Draw(img)
            
            # Corner brackets
            accent = (0, 255, 120)
            cl = 22
            for cx_, cy_, dx, dy in [
                (sx-2, sy-2, 1, 1), (sx+ss_w+2, sy-2, -1, 1),
                (sx-2, sy+ss_h+2, 1, -1), (sx+ss_w+2, sy+ss_h+2, -1, -1)
            ]:
                draw.line([(cx_, cy_), (cx_ + cl*dx, cy_)], fill=accent, width=3)
                draw.line([(cx_, cy_), (cx_, cy_ + cl*dy)], fill=accent, width=3)
        except Exception as e:
            print(f"  [!] Screenshot error: {e}")
    
    # ── Fonts ──
    try:
        font_huge = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 78)
        font_big = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 62)
        font_quote = ImageFont.truetype("C:\\Windows\\Fonts\\ariali.ttf", 32)
        font_mid = ImageFont.truetype("C:\\Windows\\Fonts\\arialbd.ttf", 36)
        font_small = ImageFont.truetype("C:\\Windows\\Fonts\\arial.ttf", 24)
        font_tag = ImageFont.truetype("C:\\Windows\\Fonts\\consolab.ttf", 22)
    except:
        font_huge = ImageFont.load_default()
        font_big = font_huge
        font_quote = font_huge
        font_mid = font_huge
        font_small = font_huge
        font_tag = font_huge
    
    tx = 42
    
    # ── Top: hater quote in italic, dimmed ──
    ty = 30
    quote = '"He just used AI, it doesn\'t count"'
    _draw_text_outline(draw, (tx, ty), quote, font_quote, (140, 140, 140),
                       outline_color=(0, 0, 0), thickness=3)
    
    # Strikethrough over the quote
    q_bbox = draw.textbbox((tx, ty), quote, font=font_quote)
    strike_y = (q_bbox[1] + q_bbox[3]) // 2
    draw.line([(tx, strike_y), (q_bbox[2] + 5, strike_y)], fill=(220, 40, 40), width=3)
    
    # ── Main: "I DESIGNED" — white ──
    ty = 100
    _draw_text_outline(draw, (tx, ty), "I DESIGNED", font_huge, (255, 255, 255),
                       outline_color=(0, 0, 0), thickness=4)
    
    # ── "A FULL OS" — green neon glow ──
    ty += 85
    _draw_glow_text(img, draw, (tx, ty), "A FULL OS", font_huge, (0, 255, 90))
    draw = ImageDraw.Draw(img)
    
    # ── "AI WROTE THE CODE." — white, slightly smaller ──
    ty += 85
    _draw_text_outline(draw, (tx, ty), "AI WROTE", font_big, (255, 255, 255),
                       outline_color=(0, 0, 0), thickness=4)
    # "THE CODE." in orange (Rust color)
    ai_w = draw.textbbox((0, 0), "AI WROTE ", font=font_big)[2]
    _draw_text_outline(draw, (tx + ai_w, ty), "THE CODE.", font_big, (255, 130, 45),
                       outline_color=(0, 0, 0), thickness=4)
    
    # ── "deal with it." — smaller, confident ──
    ty += 80
    _draw_text_outline(draw, (tx, ty), "deal with it.", font_mid, (200, 200, 200),
                       outline_color=(0, 0, 0), thickness=3)
    
    # ── Bottom info bar ──
    bar_y = H - 60
    draw.rectangle([0, bar_y - 2, W, H], fill=(0, 8, 4))
    draw.line([(0, bar_y - 2), (W, bar_y - 2)], fill=(0, 160, 70), width=2)
    
    tags = ["Rust", "bare-metal", "x86_64", "no_std", "0 libraries"]
    tag_x = 25
    for tag in tags:
        bbox = draw.textbbox((0, 0), tag, font=font_tag)
        tw = bbox[2] - bbox[0]
        draw.rounded_rectangle([tag_x - 6, bar_y + 10, tag_x + tw + 6, bar_y + 40],
                               radius=4, fill=(0, 35, 18), outline=(0, 130, 55))
        draw.text((tag_x, bar_y + 12), tag, font=font_tag, fill=(0, 210, 85))
        tag_x += tw + 22
    
    # GitHub bottom-right
    gh_text = "github.com/nathan237/TrustOS"
    gh_bbox = draw.textbbox((0, 0), gh_text, font=font_tag)
    gh_w = gh_bbox[2] - gh_bbox[0]
    draw.text((W - gh_w - 25, bar_y + 14), gh_text, font=font_tag, fill=(90, 90, 90))
    
    # ── Green separator line with glow ──
    sep_y = bar_y - 20
    draw.line([(tx, sep_y), (460, sep_y)], fill=(0, 180, 70), width=2)
    lg = Image.new('RGB', (W, H), (0, 0, 0))
    ImageDraw.Draw(lg).line([(tx, sep_y), (460, sep_y)], fill=(0, 200, 80), width=5)
    lg = lg.filter(ImageFilter.GaussianBlur(6))
    img_arr = np.array(img).astype(np.float32)
    img = Image.fromarray(np.clip(img_arr + np.array(lg).astype(np.float32) * 0.7, 0, 255).astype(np.uint8))
    draw = ImageDraw.Draw(img)
    
    # ── Vignette ──
    v_arr = np.array(img).astype(np.float32)
    Y, X = np.ogrid[:H, :W]
    vd = np.sqrt(((X - W*0.42) / (W*0.65))**2 + ((Y - H*0.48) / (H*0.65))**2)
    vm = np.clip(1.0 - (vd - 0.4) * 1.0, 0.25, 1.0)
    v_arr *= vm[:, :, np.newaxis]
    img = Image.fromarray(np.clip(v_arr, 0, 255).astype(np.uint8))
    
    # Save
    output = os.path.join(os.path.dirname(__file__), "thumbnail_showcase.png")
    img.save(output, quality=95)
    size_kb = os.path.getsize(output) // 1024
    print(f"  Thumbnail saved: {output}")
    print(f"     {W}x{H} - {size_kb} KB")
    return output


def _draw_text_outline(draw, pos, text, font, fill, outline_color=(0,0,0), thickness=3):
    x, y = pos
    for dx in range(-thickness, thickness+1):
        for dy in range(-thickness, thickness+1):
            if dx*dx + dy*dy <= thickness*thickness:
                draw.text((x+dx, y+dy), text, font=font, fill=outline_color)
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
