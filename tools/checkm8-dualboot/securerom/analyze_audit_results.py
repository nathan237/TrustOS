#!/usr/bin/env python3
"""Analyze img4_audit_results.json — extract actionable findings."""
import json
from collections import Counter

with open("img4_audit_results.json") as f:
    data = json.load(f)

print("=" * 80)
print("  T8020 B1 SecureROM — AUDIT RESULTS TRIAGE")
print("=" * 80)
print(f"\n  Functions: {data['stats']['functions_audited']}")
print(f"  Instructions: {data['stats']['instructions_analyzed']}")
print(f"  Length loads: {data['stats']['length_loads']}")
print(f"  Bounds checks: {data['stats']['bounds_checks']}")
print(f"\n  Severity: {data['severity_summary']}")

# Count types
types = Counter(f["type"] for f in data["findings"])
print(f"\n  Finding types ({len(types)} distinct):")
for t, c in types.most_common():
    print(f"    {t:40s} : {c}")

# Unique CRITICAL by function
print(f"\n{'=' * 80}")
print(f"  CRITICAL FINDINGS — UNIQUE PER FUNCTION")
print(f"{'=' * 80}\n")

seen = set()
crit_by_func = {}
for f in data["findings"]:
    if f["severity"] == "CRITICAL":
        key = (f["function"], f["type"])
        if key not in seen:
            seen.add(key)
            func = f["function"]
            if func not in crit_by_func:
                crit_by_func[func] = []
            crit_by_func[func].append(f)

# Sort by most interesting (multiple finding types = most interesting)
for func, findings in sorted(crit_by_func.items(), key=lambda x: -len(x[1])):
    types_here = set(f["type"] for f in findings)
    print(f"\n  FUNC {func} ({len(findings)} critical findings)")
    for f in findings:
        detail = f["detail"][:150]
        print(f"    [{f['type']}] @ {f['address']}")
        print(f"      {detail}")

# HIGH findings - unique per function
print(f"\n{'=' * 80}")
print(f"  HIGH FINDINGS — UNIQUE PER FUNCTION (top 30)")
print(f"{'=' * 80}\n")

seen_h = set()
high_by_func = {}
for f in data["findings"]:
    if f["severity"] == "HIGH":
        key = (f["function"], f["type"])
        if key not in seen_h:
            seen_h.add(key)
            func = f["function"]
            if func not in high_by_func:
                high_by_func[func] = []
            high_by_func[func].append(f)

for func, findings in sorted(high_by_func.items(), key=lambda x: -len(x[1]))[:30]:
    print(f"\n  FUNC {func} ({len(findings)} high findings)")
    for f in findings:
        detail = f["detail"][:150]
        print(f"    [{f['type']}] @ {f['address']}")
        print(f"      {detail}")
        
# UNBOUNDED_RECURSION specifically
print(f"\n{'=' * 80}")
print(f"  UNBOUNDED RECURSION FINDINGS (stack overflow targets)")
print(f"{'=' * 80}\n")

for f in data["findings"]:
    if f["type"] == "UNBOUNDED_RECURSION":
        print(f"  {f['severity']} @ {f['function']} / {f['address']}")
        print(f"    {f['detail']}")
        print()

# DER_LENGTH findings
print(f"\n{'=' * 80}")
print(f"  DER LENGTH DECODE FINDINGS")
print(f"{'=' * 80}\n")

for f in data["findings"]:
    if "DER_LENGTH" in f["type"]:
        print(f"  {f['severity']} [{f['type']}] @ {f['function']} / {f['address']}")
        print(f"    {f['detail']}")
        print()

# ALLOC_FROM_PARSED_LENGTH
print(f"\n{'=' * 80}")
print(f"  MALLOC WITH PARSED DATA SIZE")
print(f"{'=' * 80}\n")

for f in data["findings"]:
    if "ALLOC" in f["type"]:
        print(f"  {f['severity']} @ {f['function']} / {f['address']}")
        print(f"    {f['detail']}")
        print()
