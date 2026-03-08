#!/usr/bin/env python3
"""
Targeted disassembly of dwc3_usb_reset (C448) and surrounding 
USB RESET path to find where the double-abort happens.
Also disassemble the ep0_handler SETUP abort at D984.
"""
import sys, struct, os
from capstone import *

ROM_BASE = 0x100000000
ROM_PATH = os.path.join("securerom", "t8020_B1_securerom.bin")

KNOWN = {
    0x10000C084: "dwc3_ep_abort",
    0x10000C234: "dwc3_ep_cleanup",
    0x10000C448: "dwc3_usb_reset",
    0x10000C518: "dwc3_post_reset",
    0x10000C810: "dwc3_drain_completed",
    0x10000C8C8: "dwc3_ep0_program",
    0x10000D2D0: "hal_abort_call",
    0x10000D2FC: "hal_cleanup_call",
    0x10000D334: "zlp_send",
    0x10000D368: "ep0_transfer_setup",
    0x10000D94C: "ep0_handler",
    0x10000DF78: "usb_state_dispatch",
    0x10000E018: "usb_reset_handler",
    0x10000E0A8: "dwc3_callback_and_free",
    0x10000E158: "ep_abort_wrapper",
    0x10000E17C: "usb_quiesce",
    0x10000E1DC: "usb_ep_teardown",
    0x10000E708: "dfu_reset_cb",
    0x10000BB88: "dwc3_core_stop",
    0x10000BA30: "dwc3_core_start",
    0x10000BE50: "dwc3_start_transfer",
    0x10000F1EC: "malloc",
    0x10000F3B0: "calloc",
    0x10000F468: "free",
    0x10000F680: "memalign",
    0x100010BD0: "memcpy",
    0x100010D80: "bzero",
    0x100010E00: "memset",
    0x100008978: "panic",
    0x100009974: "signal_event",
    0x1000099C0: "event_wait",
    0x10000D1F4: "start_transfer",
    0x10000D0A8: "usb_core_init",
    0x10000D170: "sub_D170",
    0x10000A5B8: "sub_A5B8",
    0x10000A5D0: "sub_A5D0",
    0x10000D118: "sub_D118_dwc3_stop_call",
    0x100011C70: "lock_acquire",
    0x100011CBC: "lock_release",
}

def load_rom():
    with open(ROM_PATH, "rb") as f:
        return f.read()

def disasm(rom, md, name, addr, size):
    off = addr - ROM_BASE
    code = rom[off:off+size]
    print(f"\n{'='*80}")
    print(f"  {name} @ 0x{addr:X}  (ROM offset 0x{off:X})")
    print(f"{'='*80}")
    for i in md.disasm(code, addr):
        ann = ""
        if i.mnemonic in ("bl", "b"):
            try:
                t = int(i.op_str.lstrip('#'), 0)
                if t in KNOWN:
                    ann = f"  ; → {KNOWN[t]}"
            except ValueError:
                pass
        if i.mnemonic.startswith("b.") or i.mnemonic in ("cbz","cbnz","tbz","tbnz"):
            try:
                t = int(i.op_str.split(',')[-1].strip().lstrip('#'), 0)
                if t < addr:
                    ann = f"  ; ◀◀ LOOP BACK"
                elif t > addr + size:
                    ann = f"  ; → outside function"
            except ValueError:
                pass
        h = ' '.join(f'{b:02x}' for b in i.bytes)
        print(f"  0x{i.address:X}: {h:<12s} {i.mnemonic:<8s} {i.op_str}{ann}")

def main():
    rom = load_rom()
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    
    print("="*80)
    print("  KEY QUESTION: Does dwc3_usb_reset (C448) have an extra EP0_IN abort")
    print("  like dwc3_core_stop (BB88) does at BBFC-BC00?")
    print("="*80)
    
    # 1. dwc3_usb_reset — THE critical function
    # C448 to C518 (dwc3_post_reset starts at C518)
    disasm(rom, md, "dwc3_usb_reset (C448) — FULL", 0x10000C448, 0xD0)
    
    # 2. dwc3_core_stop — confirmed double-abort
    disasm(rom, md, "dwc3_core_stop (BB88) — FULL (confirmed double-abort at BBFC-BC00)", 0x10000BB88, 0xC0)
    
    # 3. ep0_handler SETUP path — abort at D984
    disasm(rom, md, "ep0_handler SETUP entry (D94C→D98x) — abort per SETUP", 0x10000D94C, 0x50)
    
    # 4. The callback function D3D0 — what exactly is standard_device_request_cb?
    # After ep0_transfer_setup returns, D3D0 is the callback check code
    disasm(rom, md, "D3D0 (standard_device_request_cb or post-transfer check)", 0x10000D3D0, 0x30)
    
    # 5. Check usb_quiesce path — does it call ep_abort before or after core_stop?
    disasm(rom, md, "usb_quiesce (E17C) - full", 0x10000E17C, 0x60)
    
    # 6. sub_D118 — what does it call? (from usb_ep_teardown E1EC)
    disasm(rom, md, "sub_D118 (DWC3 stop call?)", 0x10000D118, 0x60)
    
    # 7. sub_D170 — called from usb_quiesce at E1C4
    disasm(rom, md, "sub_D170 (called from usb_quiesce)", 0x10000D170, 0x90)
    
    # 8. sub_A5B8 — called from usb_quiesce at E1C0
    disasm(rom, md, "sub_A5B8 (called from usb_quiesce)", 0x10000A5B8, 0x20)
    
    # 9. Scan ALL calls to ep_abort_wrapper in entire ROM
    print("\n" + "="*80)
    print("  FULL ROM SCAN: All BL/B to ep_abort_wrapper (E158)")
    print("="*80)
    active_end = min(0x25000, len(rom))
    code = rom[:active_end]
    count = 0
    for insn in md.disasm(code, ROM_BASE):
        if insn.mnemonic in ("bl", "b"):
            try:
                t = int(insn.op_str.lstrip('#'), 0)
                if t == 0x10000E158:
                    count += 1
                    print(f"  #{count} 0x{insn.address:X}: {insn.mnemonic} E158 (ep_abort_wrapper)")
            except ValueError:
                pass
    print(f"  Total: {count} calls to ep_abort_wrapper")
    
    # 10. Scan ALL calls to hal_abort_call (D2D0) — which calls dwc3_ep_abort via HAL
    print("\n" + "="*80)
    print("  FULL ROM SCAN: All BL/B to hal_abort_call (D2D0)")
    print("="*80)
    count = 0
    for insn in md.disasm(code, ROM_BASE):
        if insn.mnemonic in ("bl", "b"):
            try:
                t = int(insn.op_str.lstrip('#'), 0)
                if t == 0x10000D2D0:
                    count += 1
                    print(f"  #{count} 0x{insn.address:X}: {insn.mnemonic} D2D0 (hal_abort_call)")
            except ValueError:
                pass
    print(f"  Total: {count} calls to hal_abort_call")
    
    # 11. Scan ALL calls to dwc3_ep_abort directly (C084)
    print("\n" + "="*80)
    print("  FULL ROM SCAN: All BL/B to dwc3_ep_abort (C084)")
    print("="*80)
    count = 0
    for insn in md.disasm(code, ROM_BASE):
        if insn.mnemonic in ("bl", "b"):
            try:
                t = int(insn.op_str.lstrip('#'), 0)
                if t == 0x10000C084:
                    count += 1
                    print(f"  #{count} 0x{insn.address:X}: {insn.mnemonic} C084 (dwc3_ep_abort)")
            except ValueError:
                pass
    print(f"  Total: {count} calls to dwc3_ep_abort")
    
    print("\n" + "="*80)
    print("  ANALYSIS SUMMARY")
    print("="*80)
    print("""
CONFIRMED in dwc3_core_stop (BB88):
  Loop EP5→0: E158(ep|0x80) + E158(ep)  [each EP: IN then OUT]
  EXTRA after loop: E158(0x80)  ← EP0_IN ABORTED AGAIN!

QUESTION: Does dwc3_usb_reset (C448) have the same extra EP0_IN abort?
If NOT → the USB RESET path does NOT have the double-abort!
→ ZLPs from the USB RESET abort callbacks would NOT be freed!
→ This could mean the checkm8 ZLP leak WORKS on the USB RESET path!

BUT: checkm8 uses USB bus reset → dwc3_usb_reset, not dwc3_core_stop.
If only dwc3_core_stop has the double-abort, then the USB RESET path
might still be vulnerable to ZLP leaking!
""")

if __name__ == "__main__":
    main()
