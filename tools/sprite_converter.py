#!/usr/bin/env python3
"""
sprite_converter.py — Convert a PNG sprite to a Rust const [u32; N] array.

Usage:
    python tools/sprite_converter.py <input.png> [--size 24] [--name MILITANT_IDLE]

Outputs a Rust `pub const NAME: [u32; W*H] = [...]` ready to paste into sprites.rs.
Transparent pixels (alpha < 128) become 0x00000000.
"""

import sys
import argparse
from pathlib import Path

def main():
    parser = argparse.ArgumentParser(description="Convert PNG to Rust sprite const")
    parser.add_argument("input", help="Input PNG file")
    parser.add_argument("--size", type=int, default=24, help="Target size (square, default 24)")
    parser.add_argument("--name", default="MILITANT_IDLE", help="Rust const name")
    parser.add_argument("--output", "-o", help="Output .rs file (default: stdout)")
    args = parser.parse_args()

    try:
        from PIL import Image
    except ImportError:
        print("ERROR: Pillow required. Install: pip install Pillow", file=sys.stderr)
        sys.exit(1)

    img = Image.open(args.input).convert("RGBA")

    # Downscale to target size with nearest-neighbor (crisp pixels)
    target = args.size
    img = img.resize((target, target), Image.NEAREST)

    pixels = list(img.getdata())
    w, h = img.size

    lines = []
    lines.append(f"/// {args.name} sprite — {w}x{h} ARGB pixels")
    lines.append(f"/// Generated from: {Path(args.input).name}")
    lines.append(f"pub const {args.name}: [u32; {w * h}] = [")

    for row in range(h):
        row_vals = []
        for col in range(w):
            r, g, b, a = pixels[row * w + col]
            if a < 128:
                row_vals.append("0x00000000")
            else:
                argb = (a << 24) | (r << 16) | (g << 8) | b
                row_vals.append(f"0x{argb:08X}")
        lines.append("    " + ", ".join(row_vals) + ",")

    lines.append("];")
    lines.append("")

    output = "\n".join(lines)

    if args.output:
        Path(args.output).write_text(output)
        print(f"Written to {args.output} ({w}x{h}, {w*h} pixels)")
    else:
        print(output)


if __name__ == "__main__":
    main()
