#!/usr/bin/env python3
"""Debug: find where tokenizer breaks on URL strings."""
import sys
sys.path.insert(0, '.')

orig = open('kernel/src/shell/desktop.rs', encoding='utf-8').readlines()
trans = open('translated/minimal/kernel/src/shell/desktop.rs', encoding='utf-8').readlines()

for i in range(min(len(orig), len(trans))):
    o, t = orig[i].rstrip(), trans[i].rstrip()
    if 'http' in o and '//' in o and '"' in o:
        ok = 'OK' if '//' in t else 'BROKEN'
        print(f'LINE {i+1} [{ok}]: {o[:120]}')
        if ok == 'BROKEN':
            print(f'  TRANS:  {t[:120]}')
