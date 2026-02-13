"""
TrustOS Film Thumbnail — "3MB" THE MOVIE
Cinematic movie poster style. "3MB" is the hero title.
Dark, dramatic, like a blockbuster poster.
"""
from PIL import Image, ImageDraw, ImageFont, ImageFilter, ImageEnhance
import numpy as np
import os

W, H = 1280, 720
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))


def make_film_thumbnail():
    pixels = np.zeros((H, W, 3), dtype=np.uint8)
    rng = np.random.RandomState(42)

    # ── Deep black base with subtle matrix rain (very faint, cinematic) ──
    for col in range(0, W, 6):
        brightness = rng.randint(2, 10)
        length = rng.randint(80, 500)
        start = rng.randint(0, H)
        for y in range(start, min(start + length, H)):
            fade = 1.0 - (y - start) / length
            v = int(brightness * fade)
            pixels[y, col:min(col + 2, W), 1] = v

    # ── Dramatic center spotlight (warm, cinema lighting) ──
    Y, X = np.ogrid[:H, :W]
    cx, cy = W * 0.50, H * 0.38
    dist = np.sqrt(((X - cx) / (W * 0.40)) ** 2 + ((Y - cy) / (H * 0.50)) ** 2)
    spotlight = np.clip(1.0 - dist, 0, 1) ** 2.2
    pixels[:, :, 0] = np.clip(pixels[:, :, 0] + spotlight * 18, 0, 255).astype(np.uint8)
    pixels[:, :, 1] = np.clip(pixels[:, :, 1] + spotlight * 14, 0, 255).astype(np.uint8)
    pixels[:, :, 2] = np.clip(pixels[:, :, 2] + spotlight * 4, 0, 255).astype(np.uint8)

    # ── Subtle green underglow at bottom (TrustOS signature) ──
    bottom_glow = np.clip((Y - H * 0.75) / (H * 0.25), 0, 1) ** 1.5
    pixels[:, :, 1] = np.clip(pixels[:, :, 1] + bottom_glow * 12, 0, 255).astype(np.uint8)

    img = Image.fromarray(pixels)
    draw = ImageDraw.Draw(img)

    # ── Fonts ──
    try:
        font_title = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 240)
        font_subtitle = ImageFont.truetype("C:\\Windows\\Fonts\\impact.ttf", 44)
        font_tagline = ImageFont.truetype("C:\\Windows\\Fonts\\ariali.ttf", 30)
        font_credits = ImageFont.truetype("C:\\Windows\\Fonts\\arial.ttf", 20)
        font_tag = ImageFont.truetype("C:\\Windows\\Fonts\\consolab.ttf", 18)
        font_small = ImageFont.truetype("C:\\Windows\\Fonts\\arial.ttf", 16)
    except:
        font_title = ImageFont.load_default()
        font_subtitle = font_title
        font_tagline = font_title
        font_credits = font_title
        font_tag = font_title
        font_small = font_title

    # ── Top: "A   T R U S T O S   F I L M" — small, centered, spaced ──
    top_text = "A   T R U S T O S   F I L M"
    _draw_text_outline(draw, (_center_x(draw, top_text, font_credits), 28),
                       top_text, font_credits, (120, 120, 120), thickness=2)

    # ── Thin line under it ──
    line_y = 60
    line_w = 280
    lx = (W - line_w) // 2
    draw.line([(lx, line_y), (lx + line_w, line_y)], fill=(60, 60, 60), width=1)

    # ── HERO TITLE: "3MB" — massive, centered, green neon glow ──
    title = "3MB"
    tx = _center_x(draw, title, font_title)
    ty = 68

    # Multi-layer glow (green, outward)
    for blur_r, alpha in [(35, 0.15), (22, 0.25), (12, 0.40), (6, 0.55)]:
        glow = Image.new('RGB', (W, H), (0, 0, 0))
        gd = ImageDraw.Draw(glow)
        gd.text((tx, ty), title, font=font_title, fill=(0, 255, 100))
        glow = glow.filter(ImageFilter.GaussianBlur(blur_r))
        img_arr = np.array(img).astype(np.float32)
        g_arr = np.array(glow).astype(np.float32)
        img = Image.fromarray(np.clip(img_arr + g_arr * alpha, 0, 255).astype(np.uint8))
    draw = ImageDraw.Draw(img)

    # Solid title on top with outline
    _draw_text_outline(draw, (tx, ty), title, font_title, (0, 255, 100),
                       outline_color=(0, 40, 15), thickness=5)

    # ── Subtitle: "THE MOVIE" — white, right under title ──
    sub = "THE MOVIE"
    sub_x = _center_x(draw, sub, font_subtitle)
    sub_y = ty + 235
    _draw_text_outline(draw, (sub_x, sub_y), sub, font_subtitle, (255, 255, 255),
                       outline_color=(0, 0, 0), thickness=3)

    # ── Thin green glow line separator ──
    sep_y = sub_y + 60
    sep_w = 500
    sep_x = (W - sep_w) // 2
    for offset, alpha in [(0, 1.0), (-1, 0.4), (1, 0.4)]:
        draw.line([(sep_x, sep_y + offset), (sep_x + sep_w, sep_y + offset)],
                  fill=(0, int(180 * alpha), int(70 * alpha)), width=1)

    # ── Tagline ──
    tag1 = "An entire operating system."
    tag2 = "Built from scratch. In 3 megabytes of RAM."
    _draw_text_outline(draw, (_center_x(draw, tag1, font_tagline), sep_y + 18),
                       tag1, font_tagline, (180, 180, 180), thickness=2)
    _draw_text_outline(draw, (_center_x(draw, tag2, font_tagline), sep_y + 55),
                       tag2, font_tagline, (140, 140, 140), thickness=2)

    # ── Movie credits stats line ──
    stats_y = sep_y + 110
    stats = "120,000 lines  |  1 author  |  0 lines of C  |  8 days  |  pure Rust"
    _draw_text_outline(draw, (_center_x(draw, stats, font_small), stats_y),
                       stats, font_small, (90, 90, 90), thickness=1)

    # ── Bottom tag bar ──
    bar_y = H - 50
    draw.rectangle([0, bar_y - 2, W, H], fill=(0, 6, 3))
    draw.line([(0, bar_y - 2), (W, bar_y - 2)], fill=(0, 100, 45), width=1)

    tags = ["Rust", "bare-metal", "x86_64", "no_std", "10 MB ISO", "< 1s boot"]
    tag_x = 22
    for tag in tags:
        bbox = draw.textbbox((0, 0), tag, font=font_tag)
        tw = bbox[2] - bbox[0]
        draw.rounded_rectangle([tag_x - 4, bar_y + 8, tag_x + tw + 4, bar_y + 34],
                               radius=3, fill=(0, 25, 12), outline=(0, 100, 45))
        draw.text((tag_x, bar_y + 10), tag, font=font_tag, fill=(0, 190, 80))
        tag_x += tw + 18

    # GitHub bottom-right
    gh = "github.com/nathan237/TrustOS"
    gh_bbox = draw.textbbox((0, 0), gh, font=font_tag)
    gh_w = gh_bbox[2] - gh_bbox[0]
    draw.text((W - gh_w - 20, bar_y + 12), gh, font=font_tag, fill=(60, 60, 60))

    # ── Heavy vignette (cinematic dark edges) ──
    v_arr = np.array(img).astype(np.float32)
    Y, X = np.ogrid[:H, :W]
    vd = np.sqrt(((X - W * 0.50) / (W * 0.55)) ** 2 + ((Y - H * 0.42) / (H * 0.58)) ** 2)
    vm = np.clip(1.0 - (vd - 0.30) * 1.3, 0.12, 1.0)
    v_arr *= vm[:, :, np.newaxis]
    img = Image.fromarray(np.clip(v_arr, 0, 255).astype(np.uint8))

    # ── Save ──
    output = os.path.join(SCRIPT_DIR, "thumbnail_film.png")
    img.save(output, quality=95)
    size_kb = os.path.getsize(output) // 1024
    print(f"  Thumbnail saved: {output}")
    print(f"     {W}x{H} - {size_kb} KB")
    return output


def _center_x(draw, text, font):
    bbox = draw.textbbox((0, 0), text, font=font)
    tw = bbox[2] - bbox[0]
    return (W - tw) // 2


def _draw_text_outline(draw, pos, text, font, fill, outline_color=(0, 0, 0), thickness=3):
    x, y = pos
    for dx in range(-thickness, thickness + 1):
        for dy in range(-thickness, thickness + 1):
            if dx * dx + dy * dy <= thickness * thickness:
                draw.text((x + dx, y + dy), text, font=font, fill=outline_color)
    draw.text((x, y), text, font=font, fill=fill)


if __name__ == "__main__":
    make_film_thumbnail()
