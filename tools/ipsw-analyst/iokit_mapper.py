#!/usr/bin/env python3
"""
TrustOS IPSW Analyst — Phase 3: IOKit Attack Surface Mapper
Analyzes a decompressed kernelcache to map all IOKit driver classes,
their UserClient interfaces, and sandbox accessibility.

This is the most critical tool for identifying jailbreak entry points.
"""

import os
import re
import sys
import json
import struct
from pathlib import Path
from collections import defaultdict

try:
    import lief
    HAS_LIEF = True
except ImportError:
    HAS_LIEF = False

try:
    import capstone
    HAS_CAPSTONE = True
except ImportError:
    HAS_CAPSTONE = False


# Known IOKit classes with their security properties
IOKIT_CLASS_DB = {
    # GPU — #1 attack surface
    "AGXAccelerator": {
        "risk": "CRITICAL", "sandbox": True,
        "reason": "GPU driver with JIT shader compiler. Massive codebase, complex parsing.",
        "historical_cves": ["CVE-2023-32409", "CVE-2022-32847"],
        "fuzzing_hint": "Fuzz shader compilation, texture uploads, command buffer submission"
    },
    "AGXFirmwareKextRT": {
        "risk": "HIGH", "sandbox": True,
        "reason": "GPU firmware loader. Can trigger DMA operations.",
    },
    "IOGPUDevice": {
        "risk": "HIGH", "sandbox": True,
        "reason": "Generic GPU device interface. Passes data to AGX.",
    },
    
    # Video codec — #2 attack surface
    "AppleAVE2Driver": {
        "risk": "CRITICAL", "sandbox": True,
        "reason": "Hardware video encoder. Processes untrusted media data.",
        "historical_cves": ["CVE-2022-32940", "CVE-2023-32434"],
        "fuzzing_hint": "Fuzz H.264/H.265 encoding parameters"
    },
    "AppleAVD": {
        "risk": "CRITICAL", "sandbox": True,
        "reason": "Hardware video decoder. Parses media containers.",
        "fuzzing_hint": "Fuzz decode commands, buffer management"
    },
    
    # IOSurface — #3 attack surface
    "IOSurfaceRoot": {
        "risk": "CRITICAL", "sandbox": True,
        "reason": "Shared memory surfaces. Complex reference counting.",
        "historical_cves": ["CVE-2023-38611", "CVE-2021-30807"],
        "fuzzing_hint": "Fuzz surface creation, get/set values, plane management"
    },
    
    # Display — historical target
    "IOMobileFrameBuffer": {
        "risk": "CRITICAL", "sandbox": True,
        "reason": "Display driver. REMOVED in iOS 16+ but replacement may have bugs.",
        "historical_cves": ["CVE-2021-30883", "CVE-2021-30807"],
    },
    
    # JPEG/Image processing
    "AppleJPEGDriver": {
        "risk": "HIGH", "sandbox": True,
        "reason": "Hardware JPEG decoder. Processes untrusted images.",
        "fuzzing_hint": "Fuzz JPEG decoding with malformed images"
    },
    
    # Neural Engine
    "AppleH11ANEIn": {
        "risk": "HIGH", "sandbox": True,
        "reason": "Apple Neural Engine interface. ML model processing.",
        "fuzzing_hint": "Fuzz model loading, inference parameters"
    },
    
    # HID (Input)
    "IOHIDEventDriver": {
        "risk": "MEDIUM", "sandbox": True,
        "reason": "Human input device driver. Touch, keyboard events.",
        "historical_cves": ["CVE-2017-7150"],
    },
    
    # Audio
    "AppleT8030DART": {
        "risk": "MEDIUM", "sandbox": False,
        "reason": "IOMMU controller. If bypassed, enables DMA attacks.",
    },
    
    # USB
    "AppleUSBHostDeviceUserClient": {
        "risk": "MEDIUM", "sandbox": False,
        "reason": "USB host controller. Physical attack vector.",
    },
    
    # Bluetooth
    "IOBluetoothHCIController": {
        "risk": "MEDIUM", "sandbox": True,
        "reason": "Bluetooth stack. Proximity attack vector.",
        "historical_cves": ["CVE-2022-26766"],
    },
    
    # Crypto
    "AppleSEPKeyStore": {
        "risk": "LOW", "sandbox": False,
        "reason": "SEP key management. Hard to exploit.",
    },
}


def extract_iokit_classes_from_binary(data: bytes) -> list:
    """Extract IOKit class names from kernelcache binary data"""
    classes = set()
    
    # Pattern 1: C++ vtable symbols — _ZN prefix (Itanium ABI)
    # e.g., _ZN19IOSurfaceRootClient14externalMethodEj...
    vtable_pattern = re.compile(rb'_ZN(\d+)([A-Za-z_]\w+)')
    for match in vtable_pattern.finditer(data):
        name_len = int(match.group(1))
        name = match.group(2)[:name_len].decode('ascii', errors='ignore')
        if len(name) > 3:
            classes.add(name)
    
    # Pattern 2: OSMetaClass registration strings
    # These appear as literal strings: "ClassName"
    metaclass_regions = []
    search_start = 0
    while True:
        pos = data.find(b'OSMetaClass', search_start)
        if pos < 0:
            break
        # Around OSMetaClass references, class names are often nearby
        region = data[max(0, pos-256):pos+256]
        metaclass_regions.append(region)
        search_start = pos + 1
    
    # Pattern 3: Direct string references to known IOKit prefixes
    iokit_prefixes = [
        b'IO', b'Apple', b'AGX', b'AGXG', b'DCP',
    ]
    
    # Scan for null-terminated strings that look like class names
    i = 0
    while i < len(data) - 4:
        # Look for printable ASCII strings
        if 65 <= data[i] <= 90:  # Starts with uppercase letter
            end = i
            while end < len(data) and end - i < 128:
                b = data[end]
                if b == 0:  # null terminator
                    break
                if not (32 <= b <= 126):  # not printable
                    break
                end += 1
            
            if end - i >= 8 and data[end:end+1] == b'\x00':
                name = data[i:end].decode('ascii', errors='ignore')
                # Check if it looks like an IOKit class
                if (name.startswith(('IO', 'Apple', 'AGX', 'DCP')) and 
                    'Client' in name or 'Driver' in name or 'Service' in name or
                    'Device' in name or 'Controller' in name or 'Interface' in name):
                    classes.add(name)
                # Also catch UserClient specifically
                elif 'UserClient' in name and name[0].isupper():
                    classes.add(name)
        i += 1
    
    return sorted(classes)


def extract_external_methods(data: bytes, class_name: str) -> list:
    """Try to find externalMethod dispatch table for a given class"""
    methods = []
    
    # Look for externalMethod selector references
    # In ARM64, these are typically in __DATA_CONST,__const
    pattern = class_name.encode() + b'::externalMethod'
    pos = data.find(pattern)
    if pos >= 0:
        methods.append({"offset": hex(pos), "found": True})
    
    return methods


def classify_iokit_class(class_name: str) -> dict:
    """Classify an IOKit class by attack surface relevance"""
    
    # Check known database first
    for known_name, info in IOKIT_CLASS_DB.items():
        if known_name.lower() in class_name.lower():
            return {
                "class": class_name,
                "risk": info["risk"],
                "sandbox_accessible": info.get("sandbox", False),
                "reason": info.get("reason", ""),
                "historical_cves": info.get("historical_cves", []),
                "fuzzing_hint": info.get("fuzzing_hint", ""),
                "matched_known": known_name,
            }
    
    # Heuristic classification
    risk = "LOW"
    sandbox = False
    reason = ""
    
    if "UserClient" in class_name:
        risk = "MEDIUM"
        sandbox = True
        reason = "UserClient = userland-accessible interface"
    
    if any(k in class_name for k in ["GPU", "AGX", "Graphics"]):
        risk = "CRITICAL"
        sandbox = True
        reason = "GPU driver — complex, JIT compilation"
    elif any(k in class_name for k in ["Video", "AVE", "AVD", "Codec", "JPEG", "Camera"]):
        risk = "HIGH"
        sandbox = True
        reason = "Media processing — parses untrusted data"
    elif any(k in class_name for k in ["Surface", "Framebuffer", "Display"]):
        risk = "HIGH"
        sandbox = True
        reason = "Display/surface — shared memory, ref counting"
    elif any(k in class_name for k in ["USB", "Lightning", "Thunderbolt"]):
        risk = "MEDIUM"
        sandbox = False
        reason = "Physical interface — requires USB access"
    elif any(k in class_name for k in ["Bluetooth", "BT", "WiFi", "80211"]):
        risk = "MEDIUM"
        sandbox = True
        reason = "Wireless — proximity attack vector"
    elif any(k in class_name for k in ["Neural", "ANE", "ML"]):
        risk = "MEDIUM"
        sandbox = True
        reason = "Neural engine — less audited"
    elif any(k in class_name for k in ["Audio", "Sound", "Codec"]):
        risk = "MEDIUM"
        sandbox = True
        reason = "Audio — data processing"
    elif any(k in class_name for k in ["Crypto", "KeyStore", "SEP"]):
        risk = "LOW"
        sandbox = False
        reason = "Crypto/SEP — isolated, hard target"
    elif any(k in class_name for k in ["HID", "Touch", "Multi"]):
        risk = "MEDIUM"
        sandbox = True
        reason = "Input — user-controlled data flow"
    elif any(k in class_name for k in ["DART", "IOMMU", "SMMU"]):
        risk = "MEDIUM"
        sandbox = False
        reason = "IOMMU — if bypassed enables DMA attacks"
    elif any(k in class_name for k in ["NVMe", "Storage", "NAND"]):
        risk = "LOW"
        sandbox = False
        reason = "Storage — less directly exposed"
    
    return {
        "class": class_name,
        "risk": risk,
        "sandbox_accessible": sandbox,
        "reason": reason,
        "historical_cves": [],
        "fuzzing_hint": "",
        "matched_known": None,
    }


def generate_fuzzing_targets(classified_classes: list) -> list:
    """Generate prioritized fuzzing target list"""
    targets = []
    
    # Sort by risk
    risk_order = {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}
    sorted_classes = sorted(classified_classes, 
                          key=lambda x: (risk_order.get(x["risk"], 4), x["class"]))
    
    for cls in sorted_classes:
        if cls["risk"] in ("CRITICAL", "HIGH") and cls["sandbox_accessible"]:
            target = {
                "class": cls["class"],
                "risk": cls["risk"],
                "approach": [],
            }
            
            if "GPU" in cls["class"] or "AGX" in cls["class"]:
                target["approach"] = [
                    "1. IOServiceOpen() to get connection",
                    "2. Enumerate externalMethod selectors (0-200)",
                    "3. Fuzz each method with random struct payloads",
                    "4. Special focus: shader compilation, buffer allocation",
                    "5. Monitor for kernel panics via crash reporter",
                ]
            elif "Surface" in cls["class"]:
                target["approach"] = [
                    "1. Create IOSurface with various pixel formats",
                    "2. Fuzz IOSurfaceGetPropertyMaximum/Values",
                    "3. Race condition: concurrent create/destroy/lookup",
                    "4. Test shared memory mapping edge cases",
                ]
            elif "Video" in cls["class"] or "AVE" in cls["class"] or "AVD" in cls["class"]:
                target["approach"] = [
                    "1. IOServiceOpen via VideoDecoder/Encoder service",
                    "2. Submit malformed encode/decode requests",
                    "3. Test buffer overflow in parameter structures",
                    "4. Race condition: concurrent codec operations",
                ]
            elif "JPEG" in cls["class"]:
                target["approach"] = [
                    "1. Generate malformed JPEG files (truncated, invalid markers)",
                    "2. Submit via hardware JPEG decoder interface",
                    "3. Test overflow in image dimensions, color space",
                ]
            elif "UserClient" in cls["class"]:
                target["approach"] = [
                    "1. IOServiceOpen to connect",
                    "2. Brute-force externalMethod selectors",
                    "3. Send randomized IOExternalMethodArguments",
                    "4. Test type confusion in scalar/struct parameters",
                ]
            else:
                target["approach"] = [
                    "1. IOServiceOpen to connect to service",
                    "2. Enumerate available methods",
                    "3. Fuzz with random inputs",
                ]
            
            targets.append(target)
    
    return targets


def analyze_kernelcache(path: Path) -> dict:
    """Main analysis function for a kernelcache binary"""
    
    print(f"\n[*] Loading kernelcache: {path}")
    data = path.read_bytes()
    print(f"    Size: {len(data) / (1024*1024):.1f} MB")
    
    # Extract IOKit classes
    print("[*] Extracting IOKit classes...")
    classes = extract_iokit_classes_from_binary(data)
    print(f"    Found {len(classes)} IOKit-related classes")
    
    # Classify each class
    print("[*] Classifying attack surface...")
    classified = [classify_iokit_class(cls) for cls in classes]
    
    critical = [c for c in classified if c["risk"] == "CRITICAL"]
    high = [c for c in classified if c["risk"] == "HIGH"]
    medium = [c for c in classified if c["risk"] == "MEDIUM"]
    low = [c for c in classified if c["risk"] == "LOW"]
    sandbox_reachable = [c for c in classified if c["sandbox_accessible"]]
    
    print(f"\n    Attack Surface Summary:")
    print(f"    CRITICAL: {len(critical)} classes")
    print(f"    HIGH:     {len(high)} classes")
    print(f"    MEDIUM:   {len(medium)} classes")
    print(f"    LOW:      {len(low)} classes")
    print(f"    Sandbox-reachable: {len(sandbox_reachable)}")
    
    # Generate fuzzing targets
    print("\n[*] Generating fuzzing target list...")
    targets = generate_fuzzing_targets(classified)
    
    return {
        "total_classes": len(classes),
        "classes": classified,
        "summary": {
            "critical": len(critical),
            "high": len(high),
            "medium": len(medium),
            "low": len(low),
            "sandbox_reachable": len(sandbox_reachable),
        },
        "fuzzing_targets": targets,
        "critical_classes": [c["class"] for c in critical],
        "high_classes": [c["class"] for c in high],
    }


def main():
    import argparse
    parser = argparse.ArgumentParser(description="IOKit Attack Surface Mapper")
    parser.add_argument("kernelcache", nargs="?", default=None, help="Path to raw kernelcache")
    parser.add_argument("--output", default=None, help="Output directory")
    args = parser.parse_args()
    
    output_dir = Path(args.output) if args.output else Path(__file__).parent / "extracted"
    output_dir.mkdir(parents=True, exist_ok=True)
    
    if args.kernelcache:
        kc_path = Path(args.kernelcache)
    else:
        # Look for extracted kernelcache
        extracted = output_dir
        candidates = list(extracted.glob("kernelcache_*.raw"))
        if candidates:
            kc_path = candidates[0]
            print(f"[*] Using: {kc_path}")
        else:
            print("[!] No kernelcache found. Run ipsw_fetch.py first.")
            print("    Or specify path: python iokit_mapper.py <path_to_kernelcache>")
            sys.exit(1)
    
    if not kc_path.exists():
        print(f"[!] File not found: {kc_path}")
        sys.exit(1)
    
    # Analyze
    results = analyze_kernelcache(kc_path)
    
    # Save results
    report_path = output_dir / "iokit_attack_surface.json"
    report_path.write_text(json.dumps(results, indent=2, default=str), encoding='utf-8')
    print(f"\n[+] Full report: {report_path}")
    
    # Print fuzzing targets
    if results["fuzzing_targets"]:
        print(f"\n{'='*60}")
        print(f"  TOP FUZZING TARGETS")
        print(f"{'='*60}")
        for i, target in enumerate(results["fuzzing_targets"][:10], 1):
            print(f"\n  #{i} [{target['risk']}] {target['class']}")
            for step in target["approach"]:
                print(f"     {step}")
    
    # Save human-readable report
    txt_path = output_dir / "iokit_attack_surface.txt"
    with open(txt_path, 'w', encoding='utf-8') as f:
        f.write("TrustOS — IOKit Attack Surface Map\n")
        f.write("=" * 60 + "\n\n")
        
        f.write(f"Total IOKit classes: {results['total_classes']}\n")
        f.write(f"CRITICAL: {results['summary']['critical']}\n")
        f.write(f"HIGH: {results['summary']['high']}\n")
        f.write(f"MEDIUM: {results['summary']['medium']}\n")
        f.write(f"LOW: {results['summary']['low']}\n")
        f.write(f"Sandbox-reachable: {results['summary']['sandbox_reachable']}\n\n")
        
        f.write("CRITICAL CLASSES:\n" + "-" * 40 + "\n")
        for cls in results.get("critical_classes", []):
            f.write(f"  {cls}\n")
        
        f.write(f"\nHIGH-RISK CLASSES:\n" + "-" * 40 + "\n")
        for cls in results.get("high_classes", []):
            f.write(f"  {cls}\n")
        
        f.write(f"\nFUZZING TARGETS:\n" + "=" * 60 + "\n")
        for i, target in enumerate(results["fuzzing_targets"], 1):
            f.write(f"\n#{i} [{target['risk']}] {target['class']}\n")
            for step in target["approach"]:
                f.write(f"  {step}\n")
    
    print(f"[+] Text report: {txt_path}")


if __name__ == "__main__":
    main()
