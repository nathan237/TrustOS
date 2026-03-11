#!/usr/bin/env python3
"""Convert kernel corpus.rs to bench format"""

with open('kernel/src/jarvis/corpus.rs', 'r', encoding='utf-8') as f:
    c = f.read()

# Find the start of CORPUS definition
idx = c.index('pub static CORPUS:')
idx = c.index('= &[', idx) + 4

# Parse by tracking brackets and string literals
phases = []
current_strings = []
depth = 0
i = idx
in_string = False
current_str = []
QUOTE = '"'

while i < len(c):
    ch = c[i]
    if in_string:
        if ch == '\\' and i + 1 < len(c) and c[i+1] == 'n':
            current_str.append('\\n')
            i += 2
            continue
        elif ch == QUOTE:
            current_strings.append(''.join(current_str))
            current_str = []
            in_string = False
        else:
            current_str.append(ch)
    else:
        if ch == QUOTE:
            in_string = True
            current_str = []
        elif ch == '[':
            depth += 1
        elif ch == ']':
            depth -= 1
            if depth == 0 and current_strings:
                phases.append(current_strings)
                current_strings = []
            if depth < 0:
                break
    i += 1

total = sum(len(p) for p in phases)
print(f'{len(phases)} phases, {total} sequences')
total_bytes = sum(len(s) for p in phases for s in p)
print(f'Total bytes: {total_bytes}')

# Write bench format
lines = []
for phase in phases:
    parts = [QUOTE + s.replace('\\', '\\\\').replace(QUOTE, '\\' + QUOTE) + QUOTE for s in phase]
    lines.append('    &[' + ','.join(parts) + '],')

with open('_bench_corpus.txt', 'w', encoding='utf-8') as f:
    f.write('\n'.join(lines))
print(f'Written to _bench_corpus.txt ({len(lines)} lines)')
