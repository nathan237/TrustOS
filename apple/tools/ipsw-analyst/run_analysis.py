#!/usr/bin/env python3
"""
TrustOS IPSW Analyst — Master Runner
Orchestrates the full iOS security analysis pipeline:
  1. Fetch kernelcache from IPSW  
  2. Analyze CVE patterns
  3. Map IOKit attack surface
  4. Generate comprehensive report

Usage:
    python run_analysis.py                              # Full analysis, iPhone 11 Pro latest
    python run_analysis.py --device iPhone12,3 --version 18.5
    python run_analysis.py --local path/to/kernelcache  # Analyze local file
"""

import os
import sys
import json
import argparse
from pathlib import Path
from datetime import datetime

# Add tools dir to path
sys.path.insert(0, str(Path(__file__).parent))

from ipsw_fetch import (
    get_firmware_info, list_ipsw_contents, find_kernelcache, 
    download_file_from_ipsw, try_decompress_kernelcache,
    analyze_kernelcache_macho, analyze_kernelcache_manual,
    extract_kext_list, generate_attack_surface_report, parse_img4,
    find_build_manifest, DEVICES_OF_INTEREST, SOC_INFO, OUTPUT_DIR
)

from cve_diff import (
    fetch_apple_security_content, analyze_exploit_patterns,
    predict_attack_vectors, generate_research_plan
)

from iokit_mapper import (
    analyze_kernelcache as analyze_iokit_surface
)


def banner():
    print(r"""
  ╔═══════════════════════════════════════════════════════╗
  ║  TrustOS IPSW Analyst — iOS Security Research Suite  ║
  ║  Target: iPhone 11 Pro (A13/T8030) — iOS 18.5       ║
  ╠═══════════════════════════════════════════════════════╣
  ║  Phase 1: Kernelcache Extraction                     ║
  ║  Phase 2: CVE Pattern Analysis                       ║
  ║  Phase 3: IOKit Attack Surface Mapping               ║
  ║  Phase 4: Comprehensive Report                       ║
  ╚═══════════════════════════════════════════════════════╝
    """)


def run_phase1_fetch(device_id: str, version: str, output_dir: Path, local_file: str = None):
    """Phase 1: Fetch and decompress kernelcache"""
    print("\n" + "=" * 60)
    print("  PHASE 1: KERNELCACHE EXTRACTION")
    print("=" * 60)
    
    if local_file:
        kc_path = Path(local_file)
        print(f"[*] Using local file: {kc_path}")
        data = kc_path.read_bytes()
        ios_version = version or "local"
    else:
        # Fetch from Apple
        fw = get_firmware_info(device_id, version)
        if not fw:
            print("[!] Could not find firmware. Trying offline mode...")
            return None, version or "unknown"
        
        ipsw_url = fw["url"]
        ios_version = fw["version"]
        build_id = fw["buildid"]
        signed = fw.get("signed", False)
        
        print(f"\n[+] Firmware: iOS {ios_version} (build {build_id})")
        print(f"    Signed: {'YES' if signed else 'NO'}")
        print(f"    IPSW size: {fw.get('filesize', 0) / (1024**3):.1f} GB")
        
        # List contents
        files = list_ipsw_contents(ipsw_url)
        if not files:
            print("[!] Could not read IPSW. Network issue?")
            return None, ios_version
        
        # Find and download kernelcache
        kernels = find_kernelcache(files)
        if not kernels:
            print("[!] No kernelcache in IPSW!")
            return None, ios_version
        
        kc_path = output_dir / f"kernelcache_{device_id}_{ios_version.replace('.','_')}.im4p"
        
        if kc_path.exists():
            print(f"[*] Already downloaded: {kc_path}")
        else:
            if not download_file_from_ipsw(ipsw_url, kernels[0], kc_path):
                return None, ios_version
        
        # Also grab BuildManifest
        manifest = find_build_manifest(files)
        if manifest:
            manifest_path = output_dir / "BuildManifest.plist"
            if not manifest_path.exists():
                download_file_from_ipsw(ipsw_url, manifest, manifest_path)
        
        data = kc_path.read_bytes()
    
    # Decompress
    raw_path = output_dir / f"kernelcache_{device_id}_{ios_version.replace('.','_')}.raw"
    
    if raw_path.exists():
        print(f"[*] Already decompressed: {raw_path}")
    else:
        img4_info = parse_img4(data)
        print(f"    Container: {img4_info['format']}")
        
        if img4_info['format'] in ('IM4P', 'IMG4'):
            raw_path = try_decompress_kernelcache(data, raw_path)
        elif img4_info['format'] in ('MachO', 'FatMachO'):
            raw_path.write_bytes(data)
        else:
            raw_path = try_decompress_kernelcache(data, raw_path)
    
    if raw_path and raw_path.exists():
        print(f"[+] Phase 1 complete: {raw_path} ({raw_path.stat().st_size / (1024*1024):.1f} MB)")
        return raw_path, ios_version
    else:
        print("[!] Phase 1 failed — could not extract kernelcache")
        return None, ios_version


def run_phase2_cve(device_id: str, ios_version: str, output_dir: Path):
    """Phase 2: CVE pattern analysis"""
    print("\n" + "=" * 60)
    print("  PHASE 2: CVE PATTERN ANALYSIS")
    print("=" * 60)
    
    cves = fetch_apple_security_content("17")
    patterns = analyze_exploit_patterns(cves)
    predictions = predict_attack_vectors(patterns)
    plan = generate_research_plan(predictions, device_id)
    
    # Save
    results = {
        "cves": cves,
        "patterns": {
            "by_component": {k: v for k, v in patterns["by_component"].items()},
            "by_type": patterns["by_type"],
            "actively_exploited": len(patterns["actively_exploited"]),
            "memory_corruption": len(patterns["memory_corruption"]),
            "logic_bugs": len(patterns["logic_bugs"]),
        },
        "predictions": predictions,
    }
    
    results_path = output_dir / f"cve_analysis_{device_id}.json"
    results_path.write_text(json.dumps(results, indent=2, default=str), encoding='utf-8')
    
    plan_path = output_dir / f"research_plan_{device_id}.txt"
    plan_path.write_text(plan, encoding='utf-8')
    
    print(f"\n[+] Top 5 predicted attack vectors:")
    for i, pred in enumerate(predictions[:5], 1):
        print(f"    {i}. {pred['component']:25s} (score: {pred['score']}, "
              f"active exploits: {pred['actively_exploited']})")
    
    print(f"[+] Phase 2 complete: {results_path}")
    return predictions


def run_phase3_iokit(raw_path: Path, output_dir: Path):
    """Phase 3: IOKit attack surface mapping"""
    print("\n" + "=" * 60)
    print("  PHASE 3: IOKIT ATTACK SURFACE MAPPING")
    print("=" * 60)
    
    if raw_path and raw_path.exists():
        results = analyze_iokit_surface(raw_path)
        
        report_path = output_dir / "iokit_attack_surface.json"
        report_path.write_text(json.dumps(results, indent=2, default=str))
        
        if results.get("fuzzing_targets"):
            print(f"\n[+] Top fuzzing targets:")
            for i, t in enumerate(results["fuzzing_targets"][:5], 1):
                print(f"    {i}. [{t['risk']}] {t['class']}")
        
        print(f"[+] Phase 3 complete: {report_path}")
        return results
    else:
        print("[*] Phase 3 skipped — no kernelcache available")
        print("[*] Running with known IOKit class database instead...")
        
        # Generate report from our database
        from iokit_mapper import IOKIT_CLASS_DB, classify_iokit_class, generate_fuzzing_targets
        classified = [classify_iokit_class(name) for name in IOKIT_CLASS_DB.keys()]
        targets = generate_fuzzing_targets(classified)
        
        results = {
            "source": "database_only",
            "total_classes": len(classified),
            "fuzzing_targets": targets,
        }
        
        report_path = output_dir / "iokit_attack_surface_db.json"
        report_path.write_text(json.dumps(results, indent=2, default=str))
        print(f"[+] Phase 3 (database): {report_path}")
        return results


def run_phase4_report(device_id: str, ios_version: str, output_dir: Path,
                      kc_results: dict, cve_predictions: list, iokit_results: dict):
    """Phase 4: Generate comprehensive report"""
    print("\n" + "=" * 60)
    print("  PHASE 4: COMPREHENSIVE REPORT")
    print("=" * 60)
    
    soc = SOC_INFO.get(device_id, {})
    device_name = DEVICES_OF_INTEREST.get(device_id, device_id)
    
    report = []
    report.append("=" * 70)
    report.append("  TrustOS SECURITY RESEARCH — COMPREHENSIVE ANALYSIS")
    report.append(f"  Generated: {datetime.now().strftime('%Y-%m-%d %H:%M')}")
    report.append("=" * 70)
    report.append("")
    report.append(f"  TARGET DEVICE")
    report.append(f"  {'─' * 40}")
    report.append(f"  Device:    {device_name} ({device_id})")
    report.append(f"  iOS:       {ios_version}")
    report.append(f"  SoC:       {soc.get('soc', '?')} ({soc.get('codename', '?')})")
    report.append(f"  PAC:       {soc.get('pac', 'None')}")
    report.append(f"  MTE:       {'Yes' if soc.get('mte') else 'No'}")
    report.append(f"  checkm8:   {'VULNERABLE' if soc.get('checkm8') else 'NOT vulnerable'}")
    report.append("")
    
    # Security Assessment
    report.append("  SECURITY ASSESSMENT")
    report.append(f"  {'─' * 40}")
    
    if soc.get('pac') == 'v1':
        report.append("  [!] PAC v1: Known bypasses exist (signing oracles, PACGA collisions)")
    elif soc.get('pac') == 'v2':
        report.append("  [+] PAC v2: Stronger, fewer known bypasses")
    else:
        report.append("  [x] No PAC: ROP/JOP attacks are straightforward")
    
    if not soc.get('mte'):
        report.append("  [!] No MTE: Heap exploitation uses classical techniques (UAF, overflow)")
    
    report.append("  [+] PPL: Active -- requires bypass for page table modification")
    report.append("  [+] KTRR: Active -- kernel text is read-only")
    report.append("  [+] CoreTrust: Active -- requires bypass for code signing")
    report.append("")
    
    # Top Attack Vectors
    if cve_predictions:
        report.append("  TOP PREDICTED ATTACK VECTORS")
        report.append(f"  {'─' * 40}")
        for i, pred in enumerate(cve_predictions[:7], 1):
            vuln = "[!!!]" if pred["score"] > 50 else "[!!]" if pred["score"] > 25 else "[!]"
            report.append(f"  {vuln} #{i} {pred['component']:25s} Score:{pred['score']:4d}  "
                        f"CVEs:{pred['historical_cves']:2d}  Active:{pred['actively_exploited']}")
        report.append("")
    
    # IOKit Surface
    if iokit_results:
        total = iokit_results.get("total_classes", 0)
        targets = iokit_results.get("fuzzing_targets", [])
        report.append(f"  IOKIT ATTACK SURFACE: {total} classes analyzed")
        report.append(f"  {'─' * 40}")
        for i, t in enumerate(targets[:8], 1):
            risk_icon = "[!!!]" if t["risk"] == "CRITICAL" else "[!!]" if t["risk"] == "HIGH" else "[!]"
            report.append(f"  {risk_icon} {t['class']}")
        report.append("")
    
    # Recommended chain
    report.append("  RECOMMENDED EXPLOIT CHAIN")
    report.append(f"  {'─' * 40}")
    report.append("  Stage 1 — ENTRY: WebKit type confusion or IOKit from app")
    report.append("  Stage 2 — ESCALATE: Kernel memory corruption (UAF/OOB)")
    report.append("  Stage 3 — PRIMITIVE: kernel read/write via corrupted object")
    report.append("  Stage 4 — PAC BYPASS: Signing oracle from IOKit vtable")
    report.append("  Stage 5 — PPL BYPASS: physmap manipulation for PTE writes")
    report.append("  Stage 6 — ROOT: Patch credentials, get task_for_pid(0)")
    report.append("  Stage 7 — PERSIST: CoreTrust bypass + trust cache injection")
    report.append("")
    
    # Immediate actions
    report.append("  IMMEDIATE ACTIONS (START NOW)")
    report.append(f"  {'─' * 40}")
    report.append("  1. ✅ IPSW analysis toolkit — DONE")
    report.append("  2. ✅ CVE pattern analysis — DONE")
    report.append("  3. ✅ IOKit surface mapping — DONE")
    report.append("  4. ⬜ Download IPSW kernelcache (run: python ipsw_fetch.py)")
    report.append("  5. ⬜ Load kernelcache in Ghidra")
    report.append("  6. ⬜ Diff iOS 18.4 vs 18.5 kernelcache")
    report.append("  7. ⬜ Start WebKit/JSC fuzzing (Fuzzilli)")
    report.append("  8. ⬜ Read Operation Triangulation write-up in detail")
    report.append("")
    
    report_text = "\n".join(report)
    print(report_text)
    
    # Save
    report_path = output_dir / f"FULL_REPORT_{device_id}_{ios_version.replace('.','_')}.txt"
    report_path.write_text(report_text, encoding='utf-8')
    print(f"\n[+] Full report: {report_path}")
    
    # Also save JSON
    json_report = {
        "meta": {
            "device": device_id,
            "device_name": device_name,
            "ios_version": ios_version,
            "soc": soc,
            "analysis_date": datetime.now().isoformat(),
        },
        "cve_analysis": {
            "top_predictions": cve_predictions[:10] if cve_predictions else [],
        },
        "iokit_surface": iokit_results or {},
        "recommended_chain": [
            "WebKit/IOKit entry",
            "Kernel memory corruption",
            "Kernel R/W primitive",
            "PAC bypass",
            "PPL bypass",
            "Root + tfp0",
            "CoreTrust bypass + persistence",
        ],
    }
    
    json_path = output_dir / f"FULL_REPORT_{device_id}_{ios_version.replace('.','_')}.json"
    json_path.write_text(json.dumps(json_report, indent=2, default=str), encoding='utf-8')
    
    return report_text


def main():
    parser = argparse.ArgumentParser(description="TrustOS IPSW Analyst — Full Pipeline")
    parser.add_argument("--device", default="iPhone12,3",
                       help="Device ID (default: iPhone12,3 = iPhone 11 Pro)")
    parser.add_argument("--version", default=None,
                       help="Target iOS version (default: latest)")
    parser.add_argument("--local", default=None,
                       help="Path to local kernelcache (skip download)")
    parser.add_argument("--output", default=None,
                       help="Output directory")
    parser.add_argument("--skip-fetch", action="store_true",
                       help="Skip IPSW download (use existing files)")
    parser.add_argument("--cve-only", action="store_true",
                       help="Only run CVE analysis (no network needed)")
    args = parser.parse_args()
    
    banner()
    
    output_dir = Path(args.output) if args.output else OUTPUT_DIR
    output_dir.mkdir(parents=True, exist_ok=True)
    
    device_id = args.device
    
    # Phase 1: Fetch
    raw_path = None
    ios_version = args.version or "18.5"
    
    if args.cve_only:
        print("[*] CVE-only mode — skipping kernelcache fetch")
    elif args.skip_fetch:
        # Look for existing file
        candidates = list(output_dir.glob(f"kernelcache_{device_id}_*.raw"))
        if candidates:
            raw_path = candidates[0]
            ios_version = raw_path.stem.split("_")[-1].replace("_", ".")
            print(f"[*] Using existing: {raw_path}")
    else:
        raw_path, ios_version = run_phase1_fetch(device_id, args.version, output_dir, args.local)
    
    # Phase 2: CVE analysis (always runs — no network needed for database)
    predictions = run_phase2_cve(device_id, ios_version, output_dir)
    
    # Phase 3: IOKit mapping
    if not args.cve_only:
        iokit_results = run_phase3_iokit(raw_path, output_dir)
    else:
        iokit_results = None
    
    # Phase 4: Comprehensive report
    run_phase4_report(device_id, ios_version, output_dir, 
                     None, predictions, iokit_results)
    
    print(f"\n{'='*60}")
    print(f"  ALL PHASES COMPLETE — Results in: {output_dir}")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
