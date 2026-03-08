#!/usr/bin/env python3
"""
A12 T8020 B1 SecureROM — Double-Abort ZLP Analysis
====================================================
Disassembles and traces the EXACT code path that blocks the ZLP heap leak
on A12+, which prevents checkm8 exploitation.

KEY QUESTION: During USB RESET, abort() is allegedly called TWICE on EP0_IN.
- First abort → drains queue → callbacks fire → ZLPs allocated
- Second abort → catches the newly allocated ZLPs → frees them
WHERE in the ROM is this second abort, and is there a gap we can exploit?

Critical addresses:
  0x10000C448 = dwc3_usb_reset (USB RESET handler)
  0x10000C084 = dwc3_ep_abort (HAL[0x60]) — THE CORE ABORT FUNCTION  
  0x10000C234 = dwc3_ep_cleanup (HAL[0x68])
  0x10000E158 = ep_abort_wrapper (calls HAL[0x60] then HAL[0x68])
  0x10000C810 = dwc3_drain_completed  
  0x10000E0A8 = dwc3_callback_and_free
  0x10000D334 = zlp_send
  0x10000D368 = ep0_transfer_setup (allocates 0x30-byte io_request)
  0x10000BE50 = dwc3_start_transfer (ep_busy → queue to pending)
  0x10000E17C = usb_quiesce
  0x10000E1DC = usb_ep_teardown
  0x10000BB88 = dwc3_core_stop
  0x10000DF78 = usb_state_dispatch
"""

import sys, struct, os
from capstone import *

ROM_BASE   = 0x100000000
ROM_PATH   = os.path.join("securerom", "t8020_B1_securerom.bin")

# Key function addresses to disassemble
FUNCTIONS = {
    # Core double-abort path
    "dwc3_usb_reset":         (0x10000C448, 0x200),  # The USB RESET handler - WHERE double-abort happens
    "dwc3_ep_abort":          (0x10000C084, 0x1B0),  # HAL[0x60] - core abort function (C084 to ~C234)
    "dwc3_ep_cleanup":        (0x10000C234, 0x100),  # HAL[0x68] - post-abort cleanup
    "ep_abort_wrapper":       (0x10000E158, 0x30),    # Calls HAL[0x60] then HAL[0x68]
    
    # ZLP allocation path (what the abort is supposed to catch)
    "zlp_send":               (0x10000D334, 0x40),    # Allocates io_request for ZLP
    "ep0_transfer_setup":     (0x10000D368, 0xA0),    # Allocates 0x30-byte io_request from heap
    "dwc3_start_transfer":    (0x10000BE50, 0xA8),    # ep_busy check → queue or immediate
    "dwc3_submit_transfer":   (0x10000BCF8, 0x160),   # Submit io_request to EP
    
    # Callback+free path (what processes queue entries)
    "dwc3_callback_and_free": (0x10000E0A8, 0x40),    # Call callback+free — key function
    "dwc3_drain_completed":   (0x10000C810, 0xC0),    # Drain completed queue (loop)
    "dwc3_io_req_to_completed": (0x10000CDB0, 0x30),  # Append to completed list
    
    # USB state dispatch (called BEFORE abort loop in USB RESET)
    "usb_state_dispatch":     (0x10000DF78, 0xA0),    # Dispatch USB state events
    "usb_reset_handler":      (0x10000E018, 0x80),    # USB RESET state handler
    
    # Quiesce path (separate from USB RESET - for DFU exit)
    "usb_quiesce":            (0x10000E17C, 0x60),    # Interface quiesce
    "usb_ep_teardown":        (0x10000E1DC, 0xA0),    # EP teardown (frees buffers)
    "dwc3_core_stop":         (0x10000BB88, 0xA8),    # DWC3 stop/shutdown
    
    # DWC3 IRQ handler and event loop (to understand event processing during abort)
    "dwc3_irq_handler":       (0x10000C3A4, 0xA8),    # Main IRQ handler
    "dwc3_event_loop":        (0x10000C95C, 0xB0),    # Event processing loop
    "dwc3_xfer_complete":     (0x10000CA0C, 0x2A0),   # XferComplete handler
    
    # DFU reset callback (called from usb_state_dispatch during USB RESET) 
    "dfu_reset_cb_E708":      (0x10000E708, 0x30),    # DFU RESET callback — what does it do?
    
    # Helper: D2D0 and D2FC (called from ep_abort_wrapper)
    "hal_abort_call":         (0x10000D2D0, 0x30),    # Calls HAL[0x60]
    "hal_cleanup_call":       (0x10000D2FC, 0x30),    # Calls HAL[0x68]
    
    # start_transfer chain (how ZLPs get queued)
    "start_transfer":         (0x10000D1F4, 0x50),    # Calls DWC3 via HAL
    "usb_core_do_transfer":   (0x10000E0D8, 0x80),    # EP0_IN dispatch
}

# Heap functions to track
HEAP_FUNCS = {
    0x10000F1EC: "malloc",
    0x10000F3B0: "calloc",
    0x10000F468: "free",
    0x10000F680: "memalign",
    0x100010BD0: "memcpy",
    0x100010D80: "bzero",
}

# Known function addresses for annotation
KNOWN_FUNCS = {
    0x10000C084: "dwc3_ep_abort",
    0x10000C234: "dwc3_ep_cleanup",
    0x10000C334: "dwc3_init_core",
    0x10000C3A4: "dwc3_irq_handler",
    0x10000C448: "dwc3_usb_reset",
    0x10000C518: "dwc3_post_reset",
    0x10000C5B4: "dwc3_setup_dispatch",
    0x10000C5E0: "dwc3_data_dispatch",
    0x10000C810: "dwc3_drain_completed",
    0x10000C8C8: "dwc3_ep0_program",
    0x10000C95C: "dwc3_event_loop",
    0x10000CA0C: "dwc3_xfer_complete",
    0x10000CCA8: "dwc3_ep0_in_trb",
    0x10000CDB0: "dwc3_io_req_to_completed",
    0x10000CDD0: "dwc3_cancel_trb",
    0x10000CFB8: "dwc3_ep0_out_trb",
    0x10000D0A8: "usb_core_init",
    0x10000D1F4: "start_transfer",
    0x10000D2D0: "hal_abort_call",
    0x10000D2FC: "hal_cleanup_call",
    0x10000D334: "zlp_send",
    0x10000D368: "ep0_transfer_setup",
    0x10000D3FC: "usb_desc_init",
    0x10000D5D4: "getDFUImage",
    0x10000D94C: "ep0_handler",
    0x10000DA08: "data_handler",
    0x10000DF78: "usb_state_dispatch",
    0x10000E018: "usb_reset_handler",
    0x10000E0A8: "dwc3_callback_and_free",
    0x10000E0D8: "usb_core_do_transfer",
    0x10000E158: "ep_abort_wrapper",
    0x10000E17C: "usb_quiesce",
    0x10000E1DC: "usb_ep_teardown",
    0x10000E274: "dfu_run",
    0x10000E2D0: "dfu_init",
    0x10000E3EC: "dfu_request_handler",
    0x10000E410: "abort_handler",
    0x10000E5EC: "dnload_handler",
    0x10000E708: "dfu_reset_cb",
    0x10000E72C: "dfu_cleanup",
    0x10000A578: "usb_quiesce_and_reinit",
    0x10000BA30: "dwc3_core_start",
    0x10000BB88: "dwc3_core_stop",
    0x10000BC84: "dwc3_set_speed",
    0x10000BCBC: "dwc3_poll_status",
    0x10000BCF8: "dwc3_submit_transfer",
    0x10000BE50: "dwc3_start_transfer",
    0x10000BEF4: "dwc3_ep_stall",
    0x10000BF88: "dwc3_set_stall_flag",
    0x10000BFE4: "dwc3_read_status",
    0x10000C04C: "dwc3_set_config",
    0x100009960: "event_init",
    0x100009974: "signal_event",
    0x1000099C0: "event_wait",
    0x100008978: "panic",
    **HEAP_FUNCS,
}


def load_rom():
    paths = [ROM_PATH]
    for p in paths:
        if os.path.exists(p):
            with open(p, "rb") as f:
                return f.read()
    print(f"[!] ROM not found. Tried: {paths}")
    sys.exit(1)


def disasm_function(rom, md, name, addr, size):
    """Disassemble a function with rich annotations."""
    off = addr - ROM_BASE
    if off < 0 or off + size > len(rom):
        print(f"  [!] {name}: offset 0x{off:X} out of range")
        return []
    
    code = rom[off:off+size]
    insns = list(md.disasm(code, addr))
    
    print(f"\n{'='*80}")
    print(f"  {name} @ 0x{addr:X}  (ROM offset 0x{off:X}, {size} bytes)")
    print(f"{'='*80}")
    
    heap_calls = []
    branch_targets = []
    
    for i in insns:
        ann = ""
        
        # Annotate BL targets
        if i.mnemonic == "bl" or (i.mnemonic == "b" and i.id != 0):
            try:
                target = int(i.op_str.lstrip('#'), 0)
                if target in KNOWN_FUNCS:
                    ann = f"  ; → {KNOWN_FUNCS[target]}"
                    if target in HEAP_FUNCS:
                        heap_calls.append((i.address, HEAP_FUNCS[target]))
                        ann += " ★★★"
                elif target >= ROM_BASE and target < ROM_BASE + len(rom):
                    ann = f"  ; → sub_{target:X}"
            except ValueError:
                pass
        
        # Annotate ADRP/ADD pairs that reference known addresses
        if i.mnemonic == "adrp":
            try:
                page = int(i.op_str.split(',')[1].strip().lstrip('#'), 0)
                if 0x19C000000 <= page <= 0x1A0000000:
                    ann = f"  ; SRAM page"
                elif page >= ROM_BASE:
                    ann = f"  ; ROM page"
            except (ValueError, IndexError):
                pass
        
        # Annotate LDR/STR with offsets (look for key structure accesses)
        op = i.op_str
        if "+0x88" in op or "+0x90" in op:
            ann = f"  ; EP pending queue head/tail"
        elif "+0x98" in op or "+0xa0" in op:
            ann = f"  ; EP completed queue head/tail" 
        elif "+0x7c" in op:
            ann = f"  ; EP0_IN ep_busy"
        elif "+0xcc" in op:
            ann = f"  ; EP0_OUT ep_busy"
        elif "+0x28" in op and "x22" not in op:
            ann = f"  ; +0x28 (io_buffer / callback?)"
        elif "+0x20" in op:
            ann = f"  ; +0x20 (callback)"
        elif "+0x40" in op:
            ann = f"  ; +0x40 (user data start / next ptr)"
        elif "+0x48" in op:
            ann = f"  ; +0x48 (prev ptr)"
        
        # Annotate conditional branches
        if i.mnemonic.startswith("b.") or i.mnemonic in ("cbz", "cbnz", "tbz", "tbnz"):
            try:
                target = int(i.op_str.split(',')[-1].strip().lstrip('#'), 0)
                if target in KNOWN_FUNCS:
                    ann = f"  ; → {KNOWN_FUNCS[target]}"
                elif target > addr and target < addr + size:
                    ann = f"  ; skip forward"
                elif target < addr:
                    ann = f"  ; LOOP BACK!"
                branch_targets.append(target)
            except ValueError:
                pass
        
        # Print
        hex_bytes = ' '.join(f'{b:02x}' for b in i.bytes)
        print(f"  0x{i.address:X}:  {hex_bytes:<12s}  {i.mnemonic:<8s} {i.op_str}{ann}")
    
    return heap_calls


def analyze_double_abort(rom, md):
    """
    The core analysis: trace the EXACT double-abort path.
    
    On A11 (DWC2): USB RESET → abort EP0_IN ONCE → ZLPs allocated by callbacks → leaked
    On A12 (DWC3): USB RESET → abort EP0_IN → ZLPs allocated → SECOND abort → ZLPs freed
    
    We need to find:
    1. WHERE the second abort call is in the USB RESET code path
    2. Whether dwc3_ep_abort (C084) has an internal loop that re-drains after callbacks
    3. Whether usb_state_dispatch (DF78) triggers an abort before the main loop
    """
    
    print("\n" + "█"*80)
    print("  DOUBLE-ABORT ZLP ANALYSIS — A12 T8020 B1 SecureROM")
    print("█"*80)
    
    print("\n" + "▓"*80)
    print("  SECTION 1: USB RESET HANDLER (dwc3_usb_reset @ C448)")
    print("  This is THE function called when USB BUS RESET occurs.")
    print("  We need to trace every call to EP abort within it.")
    print("▓"*80)
    
    # Disassemble the complete USB RESET handler with larger range
    disasm_function(rom, md, "dwc3_usb_reset [FULL]", 0x10000C448, 0x0D0)
    
    print("\n" + "▓"*80)
    print("  SECTION 2: USB STATE DISPATCH (DF78) — Called FIRST in reset")
    print("  Does this trigger any EP abort before the main loop?")
    print("▓"*80)
    
    disasm_function(rom, md, "usb_state_dispatch", 0x10000DF78, 0xA0)
    disasm_function(rom, md, "usb_reset_handler (E018)", 0x10000E018, 0x60)
    
    print("\n" + "▓"*80)
    print("  SECTION 3: DFU RESET CALLBACK (E708)")
    print("  Called from usb_state_dispatch for interface RESET callbacks.")
    print("  Does THIS abort EP0_IN?")
    print("▓"*80)
    
    # Disassemble DFU reset callback with more range 
    disasm_function(rom, md, "dfu_reset_cb_E708", 0x10000E708, 0x30)
    
    print("\n" + "▓"*80)
    print("  SECTION 4: ep_abort_wrapper (E158) — Called from C480 loop")
    print("  Calls HAL[0x60]=C084 then HAL[0x68]=C234")
    print("▓"*80)
    
    disasm_function(rom, md, "ep_abort_wrapper (E158)", 0x10000E158, 0x28)
    disasm_function(rom, md, "hal_abort_call (D2D0)", 0x10000D2D0, 0x30)
    disasm_function(rom, md, "hal_cleanup_call (D2FC)", 0x10000D2FC, 0x30)
    
    print("\n" + "▓"*80)
    print("  SECTION 5: dwc3_ep_abort (C084) — THE CORE ABORT ★★★")
    print("  This is where pending/active queues are drained.")
    print("  KEY QUESTION: Does it have an internal loop that re-checks?")
    print("  If callbacks (e.g. ZLP send) re-populate the queue,")
    print("  does C084 loop back and drain AGAIN?")
    print("▓"*80)
    
    disasm_function(rom, md, "dwc3_ep_abort [FULL]", 0x10000C084, 0x1B0)
    
    print("\n" + "▓"*80)
    print("  SECTION 6: dwc3_callback_and_free (E0A8) — Called per io_req")
    print("  Calls the callback function pointer, then free(io_req).")
    print("  The callback MAY allocate a new ZLP (via send_zlp).")
    print("▓"*80)
    
    disasm_function(rom, md, "dwc3_callback_and_free (E0A8)", 0x10000E0A8, 0x40)
    
    print("\n" + "▓"*80)
    print("  SECTION 7: ZLP SEND path — What callbacks do to allocate ZLPs")
    print("  zlp_send (D334) → ep0_transfer_setup (D368) → malloc")
    print("  → start_transfer → BE50 (queue to pending)")  
    print("▓"*80)
    
    disasm_function(rom, md, "zlp_send (D334)", 0x10000D334, 0x38)
    disasm_function(rom, md, "ep0_transfer_setup (D368)", 0x10000D368, 0xA0)
    disasm_function(rom, md, "start_transfer (D1F4)", 0x10000D1F4, 0x50)
    disasm_function(rom, md, "dwc3_start_transfer (BE50)", 0x10000BE50, 0xA8)
    disasm_function(rom, md, "dwc3_submit_transfer (BCF8)", 0x10000BCF8, 0x160)
    
    print("\n" + "▓"*80)
    print("  SECTION 8: dwc3_ep_cleanup (C234) — Post-abort cleanup")
    print("  Called AFTER dwc3_ep_abort. Zeroes DWC3 regs + ep_state.")
    print("  Does it also drain any queues?")
    print("▓"*80)
    
    disasm_function(rom, md, "dwc3_ep_cleanup (C234)", 0x10000C234, 0x100)
    
    print("\n" + "▓"*80)
    print("  SECTION 9: dwc3_drain_completed (C810) — Completed queue drain")
    print("  Called during normal EP0 event processing.")
    print("  Is it ALSO called during USB RESET?")
    print("▓"*80)
    
    disasm_function(rom, md, "dwc3_drain_completed (C810)", 0x10000C810, 0xC0)
    
    print("\n" + "▓"*80)
    print("  SECTION 10: usb_quiesce (E17C) — DFU EXIT quiesce path")
    print("  This is the separate quiesce for DFU exit (not USB RESET).")
    print("  Does IT also call EP abort?")
    print("▓"*80)
    
    disasm_function(rom, md, "usb_quiesce (E17C)", 0x10000E17C, 0x60)
    disasm_function(rom, md, "dwc3_core_stop (BB88)", 0x10000BB88, 0xA8)
    disasm_function(rom, md, "usb_ep_teardown (E1DC)", 0x10000E1DC, 0xA0)
    
    print("\n" + "▓"*80)
    print("  SECTION 11: DWC3 post-reset reinit (C518)")
    print("  After abort loop, this reinits DWC3 + programs EP0.")
    print("  Does it process any remaining queued items?")
    print("▓"*80)
    
    disasm_function(rom, md, "dwc3_post_reset (C518)", 0x10000C518, 0xA0)
    disasm_function(rom, md, "dwc3_ep0_program (C8C8)", 0x10000C8C8, 0xA0)


def find_all_abort_callers(rom, md):
    """
    Scan the ENTIRE ROM for all calls to dwc3_ep_abort (C084) 
    and ep_abort_wrapper (E158). This reveals if there are hidden
    abort call sites beyond the USB RESET handler.
    """
    
    print("\n" + "█"*80)
    print("  GLOBAL SCAN: All callers of EP abort functions")
    print("█"*80)
    
    # Targets to search for
    targets = {
        0x10000C084: "dwc3_ep_abort",
        0x10000E158: "ep_abort_wrapper",
        0x10000D2D0: "hal_abort_call",
        0x10000C810: "dwc3_drain_completed",
        0x10000E0A8: "dwc3_callback_and_free",
    }
    
    # Scan all BL/B instructions in the active ROM region
    active_end = min(0x25000, len(rom))
    code = rom[:active_end]
    
    for target_addr, target_name in targets.items():
        callers = []
        for insn in md.disasm(code, ROM_BASE):
            if insn.mnemonic in ("bl", "b"):
                try:
                    dest = int(insn.op_str.lstrip('#'), 0)
                    if dest == target_addr:
                        # Find which function this is in
                        caller_func = "unknown"
                        for fname, (faddr, fsize) in FUNCTIONS.items():
                            if faddr <= insn.address < faddr + fsize:
                                caller_func = fname
                                break
                        callers.append((insn.address, insn.mnemonic, caller_func))
                except ValueError:
                    pass
        
        print(f"\n  {target_name} (0x{target_addr:X}) — {len(callers)} callers:")
        for caddr, cmnem, cfunc in callers:
            print(f"    0x{caddr:X}: {cmnem} → in {cfunc}")


def find_all_zlp_alloc_paths(rom, md):
    """
    Scan for ALL calls to zlp_send (D334) and ep0_transfer_setup (D368).
    These are the allocation paths that create ZLP io_requests on the heap.
    We need to know EVERY code path that can allocate a ZLP.
    """
    
    print("\n" + "█"*80)
    print("  ZLP ALLOCATION PATH SCAN")
    print("█"*80)
    
    targets = {
        0x10000D334: "zlp_send",
        0x10000D368: "ep0_transfer_setup",
    }
    
    active_end = min(0x25000, len(rom))
    code = rom[:active_end]
    
    for target_addr, target_name in targets.items():
        callers = []
        for insn in md.disasm(code, ROM_BASE):
            if insn.mnemonic in ("bl", "b"):
                try:
                    dest = int(insn.op_str.lstrip('#'), 0)
                    if dest == target_addr:
                        caller_func = "unknown"
                        for fname, (faddr, fsize) in FUNCTIONS.items():
                            if faddr <= insn.address < faddr + fsize:
                                caller_func = fname
                                break
                        callers.append((insn.address, insn.mnemonic, caller_func))
                except ValueError:
                    pass
        
        print(f"\n  {target_name} (0x{target_addr:X}) — {len(callers)} callers:")
        for caddr, cmnem, cfunc in callers:
            print(f"    0x{caddr:X}: {cmnem} → in {cfunc}")


def analyze_callback_chain(rom, md):
    """
    The ZLP is allocated by a CALLBACK function when io_requests are aborted.
    Trace the standard_device_request_cb equivalent to see:
    1. What conditions trigger ZLP send
    2. Whether the callback can be manipulated to NOT allocate a ZLP
       (or to allocate something ELSE that survives the second abort)
    """
    
    print("\n" + "█"*80)
    print("  CALLBACK CHAIN ANALYSIS")
    print("  When dwc3_ep_abort drains queues, it calls E0A8 which calls")
    print("  the callback at io_req+0x20. What callbacks are registered?")
    print("█"*80)
    
    # The key callback for EP0 requests that triggers ZLP:
    # standard_device_request_cb — we need to find it
    # It's set in ep0_transfer_setup (D368) or usb_core_do_transfer (E0D8)
    
    # Look for STR instructions that write to +0x20 offset (callback field)
    # in ep0_transfer_setup and surrounding code
    print("\n  Searching for callback registration (STR to +0x20 in io_request)...")
    
    # Disassemble the ep0 handler area for callback setup
    disasm_function(rom, md, "ep0_handler (D94C) - callback setup context", 0x10000D94C, 0x100)
    
    # Also check usb_core_do_transfer
    disasm_function(rom, md, "usb_core_do_transfer (E0D8)", 0x10000E0D8, 0x80)
    
    # Check around D334 where ZLP is actually sent - what callback is set on ZLP?
    # The ZLP IO request also has a callback - what happens when THAT is aborted?
    print("\n  KEY QUESTION: When a ZLP io_request is aborted, what callback runs?")
    print("  If the ZLP's callback ALSO allocates something, we get a chain.")


def heap_allocation_during_abort(rom, md):
    """
    Map ALL heap allocations that can occur DURING the abort sequence.
    During the EP abort loop (C480-C4A0), callbacks fire. Each callback
    might allocate heap memory. Any allocation NOT freed by the cleanup
    would be a leak candidate.
    """
    
    print("\n" + "█"*80)
    print("  HEAP ALLOCATIONS DURING ABORT — Finding leaks")
    print("█"*80)
    
    # The abort sequence at C084 calls E0A8 per io_req
    # E0A8 calls callback+0x20, then free(io_req)
    # The callback function could be:
    # 1. standard_device_request_cb → may call zlp_send → malloc
    # 2. Some other callback
    
    # We need to find what values go into io_req+0x20 (callback field)
    # Scan for STR to +0x20 in the relevant code regions
    
    print("\n  Scanning for callback writes (STR to [reg, #0x20])...")
    
    active_end = min(0x25000, len(rom))
    code = rom[:active_end]
    
    callback_writes = []
    for insn in md.disasm(code, ROM_BASE):
        if insn.mnemonic.startswith("str") and "#0x20]" in insn.op_str:
            # Get the source register and check context
            callback_writes.append((insn.address, f"{insn.mnemonic} {insn.op_str}"))
    
    print(f"  Found {len(callback_writes)} STR to +0x20:")
    for addr, text in callback_writes:
        func_ctx = "unknown"
        for fname, (faddr, fsize) in FUNCTIONS.items():
            if faddr <= addr < faddr + fsize:
                func_ctx = fname
                break
        print(f"    0x{addr:X}: {text}  (in {func_ctx})")
    
    # Also look for the actual callback functions (addresses stored at +0x20)
    # These are typically set via ADR/ADRP+ADD pairs followed by STR
    print("\n  Looking for ADR/ADRP patterns that set callback addresses...")
    
    # Check what functions are referenced as callbacks in ep0_transfer_setup
    # and usb_core_do_transfer
    regions = [
        ("ep0_transfer_setup", 0x10000D368, 0xA0),
        ("usb_core_do_transfer", 0x10000E0D8, 0x80),
        ("ep0_handler context", 0x10000D94C, 0x200),
        ("zlp_send", 0x10000D334, 0x38),
    ]
    
    for name, addr, size in regions:
        off = addr - ROM_BASE
        if off < 0 or off + size > len(rom):
            continue
        code_region = rom[off:off+size]
        adrp_page = None
        adrp_reg = None
        for insn in md.disasm(code_region, addr):
            if insn.mnemonic == "adrp":
                parts = insn.op_str.split(',')
                if len(parts) >= 2:
                    adrp_reg = parts[0].strip()
                    try:
                        adrp_page = int(parts[1].strip().lstrip('#'), 0)
                    except ValueError:
                        adrp_page = None
            elif insn.mnemonic == "add" and adrp_page is not None:
                parts = insn.op_str.split(',')
                if len(parts) >= 3 and parts[1].strip() == adrp_reg:
                    try:
                        imm = int(parts[2].strip().lstrip('#'), 0)
                        full_addr = adrp_page + imm
                        if full_addr in KNOWN_FUNCS:
                            print(f"    0x{insn.address:X} in {name}: ADR → 0x{full_addr:X} ({KNOWN_FUNCS[full_addr]})")
                        elif full_addr >= ROM_BASE and full_addr < ROM_BASE + len(rom):
                            print(f"    0x{insn.address:X} in {name}: ADR → 0x{full_addr:X} (sub_{full_addr:X})")
                        elif 0x19C000000 <= full_addr <= 0x1A0000000:
                            print(f"    0x{insn.address:X} in {name}: ADR → 0x{full_addr:X} (SRAM global)")
                    except ValueError:
                        pass
                    adrp_page = None
                    adrp_reg = None


def main():
    rom = load_rom()
    print(f"[+] ROM loaded: {len(rom)} bytes")
    
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    
    # === PHASE 1: Disassemble all critical functions ===
    analyze_double_abort(rom, md)
    
    # === PHASE 2: Find ALL callers of abort functions ===
    find_all_abort_callers(rom, md)
    
    # === PHASE 3: Find ALL ZLP allocation paths ===
    find_all_zlp_alloc_paths(rom, md)
    
    # === PHASE 4: Analyze callback chain ===
    analyze_callback_chain(rom, md)
    
    # === PHASE 5: Map heap allocations during abort ===
    heap_allocation_during_abort(rom, md)
    
    print("\n" + "█"*80)
    print("  ANALYSIS COMPLETE")
    print("█"*80)
    print("""
KEY QUESTIONS TO ANSWER FROM THE DISASSEMBLY:

1. DOUBLE-ABORT MECHANISM:
   - In dwc3_usb_reset (C448), is EP0_IN aborted more than once?
   - Does usb_state_dispatch (DF78) trigger an abort before the C480 loop?
   - Does dwc3_ep_abort (C084) have an internal loop that re-drains?

2. ZLP FATE:
   - When C084 processes pending queue → callbacks → ZLP allocated...
   - Does the ZLP go into the pending queue of an EP that's already been aborted?
   - Or does it go into an EP whose abort hasn't happened yet?
   
3. TIMING WINDOW:
   - Between the first abort (C084 drains queue) and the second abort:
   - Is there a window where a ZLP is allocated but not yet reachable?
   - Could we make the ZLP go to a different EP (not EP0_IN)?

4. ALTERNATIVE LEAKS:
   - Are there ANY heap allocations during the abort sequence that 
     are NOT freed by the subsequent cleanup?
   - What about the io_request allocated by zlp_send — is the ZLP
     io_request's own callback recursive? (callback → alloc ZLP → 
     ZLP aborted → callback → alloc ZLP → ...)
""")


if __name__ == "__main__":
    main()
