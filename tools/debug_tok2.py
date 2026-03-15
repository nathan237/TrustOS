#!/usr/bin/env python3
"""Debug: tokenize a specific range of desktop.rs and see tokens around line 3715."""
import sys
sys.path.insert(0, 'tools')
from source_translator import RustTokenizer, TT

source = open('kernel/src/shell/desktop.rs', encoding='utf-8').read()
tok = RustTokenizer(source)
tokens = tok.tokenize()

# Find tokens around line 3714-3716
for i, t in enumerate(tokens):
    if 3713 <= t.line <= 3717:
        val = repr(t.value) if len(t.value) > 80 else t.value
        print(f"  [{i:5d}] L{t.line:5d}:{t.col:3d}  {t.type.name:15s}  {val[:120]}")
