#!/usr/bin/env python3
"""Quick viewer for ios185_full_analysis.json"""
import json
from pathlib import Path

db = json.loads(Path("extracted/ios185_full_analysis.json").read_text(encoding="utf-8"))

# Show overflow analysis
ov = db["overflow_analysis"]
print("=== OVERFLOW ANALYSIS ===")
print(f"Total MUL: {ov['total_mul']}")
print(f"Protected: {ov['protected']}")
print(f"Unprotected: {ov['unprotected']}")
print(f"Unprotected addrs: {ov['unprotected_addrs']}")
print(f"Protection types: {json.dumps(ov['protection_types'], indent=2)}")

# Show meta
meta = db["meta"]
print(f"\n=== META ===")
for k, v in meta.items():
    print(f"  {k}: {v}")

# Key dispatch
dt = db["dispatch_table"]
print(f"\n=== DISPATCH TABLE ===")
print(f"VA: {dt['va']}")
print(f"Count: {dt['entry_count']}")
for e in dt["entries"][:13]:
    print(f"  [{e['selector']:2d}] {e['name']:30s} -> {e['target']}")

# String xrefs
print(f"\n=== STRING XREFS ===")
for k, v in db.get("string_xrefs", {}).items():
    print(f"  {k}: {v}")

# Kernel functions
print(f"\n=== KERNEL FUNCTIONS ===")
for k, v in db.get("kernel_functions", {}).items():
    print(f"  {k}: {v}")
