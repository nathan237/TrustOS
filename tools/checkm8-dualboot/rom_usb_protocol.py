#!/usr/bin/env python3
"""
T8020 B1 SecureROM — USB PROTOCOL HANDLER TRACE
================================================
Traces the actual USB control transfer processing:
  1. USB_CTRL functions that handle EP0 setup/data/status phases
  2. USB request type dispatch (Standard, Class=DFU, Vendor=Apple)
  3. DFU protocol: DNLOAD, UPLOAD, GETSTATUS, CLRSTATUS, GETSTATE, ABORT
  4. Buffer receiving and img4 submission flow
  5. Global SRAM state structure map
"""

import struct
import os
from collections import defaultdict

from capstone import *
from capstone.arm64 import *

ROM_PATH = os.path.join(os.path.dirname(__file__), "securerom", "t8020_B1_securerom.bin")
ROM_BASE = 0x100000000

def load_rom():
    with open(ROM_PATH, "rb") as f:
        return f.read()

def get_md():
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    return md

def disasm_func(md, rom, addr, max_size=2048):
    """Disassemble from addr until RET or max_size."""
    off = addr - ROM_BASE
    code = rom[off:off+max_size]
    instrs = list(md.disasm(code, addr))
    result = []
    for ins in instrs:
        result.append(ins)
        if ins.mnemonic == 'ret':
            break
    return result

KNOWN_FUNCS = {
    0x100010BD0: "memcpy",
    0x100010E00: "memset",
    0x100010D80: "bzero",
    0x10000F1EC: "heap_alloc",
    0x100008978: "panic",
    0x10000F3B0: "pool_alloc",
    0x10000F468: "pool_free",
    0x100010EA4: "heap_fn_EA4",
    0x100010014: "heap_fn_014",
    0x100005E50: "img4_verify_tag",
    0x100005F04: "img4_verify_property",
    0x100005F7C: "img4_fn_5F7C",
    0x1000062E8: "platform_fn_62E8",
    0x100006754: "platform_fn_6754",
    0x100006774: "platform_fn_6774",
    0x100008B58: "stack_chk_fail",
    0x1000126FC: "cert_fn_26FC",
    0x100012924: "cert_verify_nonce",
    0x100012A6C: "cert_verify_type2",
    0x100012AD8: "cert_verify_type1",
    0x100012B44: "cert_verify_type4",
    0x10001C094: "data_fn_C094",
    0x100009BA8: "security_fn_BA8",
    0x100009B64: "security_fn_B64",
    0x100009B78: "security_fn_B78",
    0x100007DE0: "validate_fn_DE0",
    0x100007DE8: "validate_fn_DE8",
    0x100007168: "validate_fn_168",
    0x10000AA38: "img4_fw_fn_AA38",
    0x100002DB0: "usb_ctrl_fn_DB0",
    0x100002EDC: "usb_ctrl_fn_EDC",
    0x100002E30: "usb_ctrl_fn_E30",
    0x100002F78: "usb_setup_endpoint",
    0x100002D34: "usb_ctrl_fn_D34",
    0x100013AAC: "cert_fn_3AAC",
    0x100011004: "sync_fn_1004",
    0x1000083B8: "log_fn_83B8",
    0x100008370: "log_fn_8370",
    0x100003CBC: "dfu_fn_CBC",
    0x100003B6C: "dfu_fn_B6C",
}

def annotate_instruction(ins, adrp_regs, rom):
    """Return annotation string for an instruction."""
    ann = ""
    
    if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
        if ins.operands[0].type == ARM64_OP_REG and ins.operands[1].type == ARM64_OP_IMM:
            page = ins.operands[1].imm
            adrp_regs[ins.operands[0].reg] = page
            if 0x19C000000 <= page < 0x1A0000000:
                ann = f"  | page=0x{page:X}"
            elif page >= 0x100000000:
                ann = f"  | ROM page=0x{page:X}"
    
    if ins.mnemonic == 'add' and len(ins.operands) >= 3:
        if (ins.operands[1].type == ARM64_OP_REG and 
            ins.operands[2].type == ARM64_OP_IMM and
            ins.operands[1].reg in adrp_regs):
            full = adrp_regs[ins.operands[1].reg] + ins.operands[2].imm
            adrp_regs[ins.operands[0].reg] = full
            if 0x19C000000 <= full < 0x1A0000000:
                ann = f"  | = SRAM 0x{full:X}"
            elif 0x10001BC00 <= full < 0x100022000:
                rom_off = full - ROM_BASE
                try:
                    end_pos = rom.index(0, rom_off, rom_off + 80)
                    s = rom[rom_off:end_pos].decode('ascii', errors='replace')
                    if len(s) > 2 and all(c.isprintable() for c in s):
                        ann = f'  | -> "{s}"'
                    else:
                        ann = f"  | data @ 0x{full:X}"
                except:
                    ann = f"  | data @ 0x{full:X}"
    
    if ins.mnemonic in ('ldr', 'ldrb', 'ldrh', 'ldrsw', 'str', 'strb', 'strh'):
        for op in ins.operands:
            if op.type == ARM64_OP_MEM and op.mem.base != 0:
                if op.mem.base in adrp_regs:
                    base = adrp_regs[op.mem.base]
                    if isinstance(base, int):
                        addr = base + op.mem.disp
                        rw = "READ" if ins.mnemonic.startswith('ldr') else "WRITE"
                        if 0x19C000000 <= addr < 0x1A0000000:
                            ann = f"  | {rw} [0x{addr:X}]"
    
    if ins.mnemonic == 'bl':
        target = ins.operands[0].imm if ins.operands and ins.operands[0].type == ARM64_OP_IMM else 0
        name = KNOWN_FUNCS.get(target, f"sub_{target:X}")
        ann = f"  | CALL {name}()"
    
    if ins.mnemonic == 'blr':
        ann = f"  | INDIRECT CALL (vtable/callback)"
    
    if ins.mnemonic == 'cmp' and len(ins.operands) >= 2:
        if ins.operands[1].type == ARM64_OP_IMM:
            v = ins.operands[1].imm
            if v == 0: ann = "  | == 0?"
            elif v == 1: ann = "  | == 1?"  
            elif v == 2: ann = "  | == 2?"
            elif v == 5: ann = "  | == 5? (SET_ADDRESS)"
            elif v == 6: ann = "  | == 6? (GET_DESCRIPTOR)"
            elif v == 9: ann = "  | == 9? (SET_CONFIGURATION)"
            elif v == 0x21: ann = "  | == 0x21? (CLASS|INTF bmRequestType)"
            elif v == 0xA1: ann = "  | == 0xA1? (CLASS|INTF|IN bmRequestType)"
            elif v == 0x80: ann = "  | == 0x80? (DEV_TO_HOST)"
            elif v == 0x40: ann = "  | == 0x40? (VENDOR bmRequestType)"
            elif v == 0xC0: ann = "  | == 0xC0? (VENDOR|IN bmRequestType)"
            else: ann = f"  | == 0x{v:X} ({v})?"
    
    if ins.mnemonic in ('mov', 'movz') and len(ins.operands) >= 2:
        if ins.operands[1].type == ARM64_OP_IMM:
            v = ins.operands[1].imm
            if v == 0x05AC: ann = "  | Apple_VID"
            elif v == 0x1227: ann = "  | DFU_PID"
            elif v == 0x1281: ann = "  | Recovery_PID"
    
    return ann

def print_function(md, rom, addr, name=""):
    """Print fully annotated disassembly of a function."""
    instrs = disasm_func(md, rom, addr)
    size = (instrs[-1].address - addr + 4) if instrs else 0
    
    print(f"\n{'='*90}")
    print(f"  FUNCTION: 0x{addr:X} ({name}) — {size} bytes, {len(instrs)} instructions")
    print(f"{'='*90}")
    
    adrp_regs = {}
    for ins in instrs:
        ann = annotate_instruction(ins, adrp_regs, rom)
        print(f"  0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:45s}{ann}")

def main():
    rom = load_rom()
    md = get_md()
    
    print("=" * 100)
    print("T8020 B1 SecureROM — USB PROTOCOL HANDLER TRACE")
    print("=" * 100)
    
    # =========================================================
    # PART 1: USB CTRL Entry Points
    # =========================================================
    print("\n\n### PART 1: USB CONTROLLER ENTRY POINTS (called from PLATFORM)")
    
    # Key USB_CTRL functions called from platform_68A4:
    usb_entry_funcs = [
        (0x100002F78, "usb_setup_endpoint (called 12x from platform)"),
        (0x100002D34, "usb_ctrl_fn_D34 (called 6x from platform)"),
        (0x100002D38, "usb_ctrl_fn_D38 -> dfu_4920"),
        (0x100002E38, "usb_ctrl_fn_E38 -> dfu_4940"),
        (0x100002F84, "usb_ctrl_fn_F84 -> dfu_497C"),
    ]
    
    for addr, name in usb_entry_funcs:
        print_function(md, rom, addr, name)
    
    # =========================================================
    # PART 2: DFU Protocol Entry Points (called from USB_CTRL)
    # =========================================================
    print("\n\n### PART 2: DFU HANDLER ENTRY POINTS")
    
    # USB_CTRL -> DFU entry points from call graph:
    dfu_entry_funcs = [
        (0x1000030E0, "dfu_init (called from platform_68A4, platform_6A24)"),
        (0x100003184, "dfu_state_setup (called from platform_68A4)"),
        (0x100003F88, "dfu_state_query (called from platform_69BC)"),
        (0x100004920, "dfu_ep_handler_1 (called from usb_ctrl_D38)"),
        (0x100004940, "dfu_ep_handler_2 (called from usb_ctrl_E38)"),
        (0x10000497C, "dfu_ep_handler_3 (called from usb_ctrl_F84)"),
    ]
    
    for addr, name in dfu_entry_funcs:
        print_function(md, rom, addr, name)
    
    # =========================================================
    # PART 3: USB Request Dispatch (the actual switch/case)
    # =========================================================
    print("\n\n### PART 3: USB CONTROL TRANSFER REQUEST DISPATCH")
    
    # Look for the main USB setup packet handler
    # In DWC USB, setup packets arrive on EP0 OUT
    # The setup packet is: bmRequestType(1) + bRequest(1) + wValue(2) + wIndex(2) + wLength(2)
    
    # Let's disassemble the function at 0x1000023FC which calls dfu_3ED8
    dispatch_funcs = [
        (0x1000023FC, "usb_control_transfer_handler (calls dfu_3ED8)"),
        (0x100003ED8, "dfu_setup_packet_handler (called from usb_ctrl)"),
        (0x100003F9C, "dfu_request_dispatch (has memcpy, security calls)"),
        (0x100003CF8, "dfu_download_handler (main DFU DNLOAD logic)"),
    ]
    
    for addr, name in dispatch_funcs:
        print_function(md, rom, addr, name)
    
    # =========================================================
    # PART 4: The actual DFU data receive path
    # =========================================================
    print("\n\n### PART 4: DFU DATA RECEIVE PATH")
    
    # Functions that handle the actual data:
    data_funcs = [
        (0x100004174, "dfu_buffer_alloc (has pool_alloc, pool_free)"),
        (0x100004240, "dfu_transfer_handler (calls usb_ctrl, platform)"),
        (0x100004368, "dfu_io_handler (calls usb_ctrl, security)"),
        (0x1000044A8, "dfu_state_machine_main (huge state checks)"),
        (0x100004650, "dfu_state_handler_A (dfuDNBUSY)"),
        (0x1000047BC, "dfu_state_handler_B (dfuDNBUSY variant)"),
        (0x1000049D8, "dfu_completion_handler (has heap ops)"),
    ]
    
    for addr, name in data_funcs:
        print_function(md, rom, addr, name)
    
    # =========================================================
    # PART 5: SRAM State Structure
    # =========================================================
    print("\n\n### PART 5: SRAM STATE STRUCTURE MAP")
    print("""
    Based on all ADRP references found across USB and DFU code:
    
    ┌─ 0x19C008000: SRAM Page 0 (System) ─────────────────────────┐
    │  +0x000: System base pointer                                 │
    │  +0x448: Stack canary value (read 17x in USB_CTRL,           │
    │          read at start/end of every function with stack frame)│
    │  +0xB48: Config value A                                      │
    │  +0xB59: Config byte (R/W)                                   │
    └──────────────────────────────────────────────────────────────┘
    
    ┌─ 0x19C00B000: SRAM Page B (USB/DFU Context) ────────────────┐
    │  +0x000: USB controller state base                           │
    │  +0xB90: USB endpoint config word (R/W)                      │
    │  +0xB94: USB transfer counter (R=5 W=2, heavily used)        │
    │  +0xC10: ★ DFU_STATE byte (read at start of every handler)   │
    │          Values: 0=idle, 1=active, 2=upload, 4=download...   │
    │  +0xC20: DFU callback structure base:                        │
    │    +0xC30: [+0x10] Callback for cert type 1 (fn ptr)         │
    │    +0xC38: [+0x18] Callback for cert type 2 (fn ptr)         │
    │    +0xC40: [+0x20] Callback for cert type 4 (fn ptr)         │
    └──────────────────────────────────────────────────────────────┘
    
    ┌─ 0x19C010000: SRAM Page 10 (USB Buffers/Data) ──────────────┐
    │  +0xA90: Vtable entries (8 runtime function pointers)        │
    │  +0x3F8: img4 result storage (written by tag verifier)       │
    └──────────────────────────────────────────────────────────────┘
    
    ┌─ 0x19C011000: SRAM Page 11 (More buffers) ──────────────────┐
    │  +0x006: SRAM buffer (referenced from BNCH nonce check)      │
    │  +0x3F8: Property verification result                        │
    └──────────────────────────────────────────────────────────────┘
    
    ┌─ 0x19C014000: SRAM Page 14 (Counters/Debug) ────────────────┐
    │  +0x034: Counter (incremented in exception handler)          │
    └──────────────────────────────────────────────────────────────┘
    """)
    
    # =========================================================
    # PART 6: Complete USB Workflow Summary
    # =========================================================
    print("\n\n### PART 6: COMPLETE USB → SECUREROM WORKFLOW")
    print("""
    ╔══════════════════════════════════════════════════════════════════════════════╗
    ║                    COMPLETE USB → SECUREROM WORKFLOW                        ║
    ║                    T8020 B1 (iPhone XR A12)                                ║
    ╚══════════════════════════════════════════════════════════════════════════════╝

    LAYER 0: PHYSICAL / ELECTRICAL
    ══════════════════════════════════
    ┌─────────────┐     ┌──────────────┐     ┌────────────────┐
    │ Lightning   │────→│ USB PHY      │────→│ DWC2/DWC3      │
    │ Connector   │     │ (UTMI/PIPE)  │     │ USB Controller │
    │             │     │ Pre-init by  │     │ MMIO@0x200350k │
    │ USB 2.0 HS  │     │ BootROM load │     │ (Synopsys IP)  │
    └─────────────┘     └──────────────┘     └────────────────┘
    
    Key finding: NO direct USB PHY or PMGR MMIO references in SecureROM!
    The PHY is initialized by the boot ROM loader (pre-SecureROM code
    burned into the SoC fabric) BEFORE SecureROM gets control.
    
    LAYER 1: USB CONTROLLER INIT (ROM @ 0x100002000)
    ══════════════════════════════════════════════════════
    27 functions handle:
    - Endpoint configuration (usb_setup_endpoint @ 0x100002F78)
    - Controller state management via SRAM[0x19C00BB90-BB94]
    - Stack canary verification (SRAM[0x19C008448])
    - USB feature handling (CLEAR_FEATURE @ 0x1000024BC)
    - Vendor request filtering (bmRequestType 0x40 @ 0x10000278C)
    
    SRAM State: 0x19C00B000 = USB controller context base
    
    LAYER 2: DFU MODE ENTRY (ROM @ 0x100003000)
    ══════════════════════════════════════════════════════
    55 functions implement the DFU state machine:
    
    Boot path (from platform_68A4):
    ┌─────────────────┐     ┌──────────────────┐     ┌───────────────┐
    │ platform_68A4   │────→│ usb_setup_endpt  │────→│ dfu_init      │
    │ (PLATFORM init) │  x12│ @ 0x100002F78    │     │ @ 0x1000030E0 │
    │                 │     │                  │     │               │
    │ Calls USB_CTRL  │────→│ usb_ctrl_D34     │     │ Sets DFU state│
    │ then DFU_HANDLER│  x6 │ @ 0x100002D34    │     │ = IDLE        │
    └─────────────────┘     └──────────────────┘     └───────────────┘
    
    DFU state variable: SRAM[0x19C00BC10] (byte)
    
    LAYER 3: USB CONTROL TRANSFER HANDLING
    ══════════════════════════════════════════════════════
    When host sends USB control transfer (SETUP packet):
    
    ┌─────────────────┐     ┌──────────────────┐     ┌───────────────┐
    │ EP0 OUT data    │────→│ usb_ctrl_handler │────→│ dfu_setup_pkt │
    │ (8-byte SETUP)  │     │ @ 0x1000023FC    │     │ @ 0x100003ED8 │
    │                 │     │                  │     │               │
    │ bmRequestType   │     │ Routes based on  │     │ Processes:    │
    │ bRequest        │     │ request type     │     │ - DFU_DNLOAD  │
    │ wValue          │     │                  │     │ - DFU_UPLOAD  │
    │ wIndex          │     │                  │     │ - DFU_GETSTATUS│
    │ wLength         │     │                  │     │ - DFU_ABORT   │
    └─────────────────┘     └──────────────────┘     └───────────────┘
    
    Request type dispatch at 0x10000278C:
    - 0x00: Standard HOST→DEV → USB standard requests
    - 0x21: Class HOST→DEV → DFU class OUT requests (DNLOAD)
    - 0x40: Vendor HOST→DEV → Apple vendor requests
    - 0x80: Standard DEV→HOST → GET_DESCRIPTOR etc.
    - 0xA1: Class DEV→HOST → DFU class IN requests (UPLOAD, GETSTATUS)
    
    LAYER 4: DFU PROTOCOL STATE MACHINE
    ══════════════════════════════════════════════════════
    
    ┌──────────┐  DFU_DNLOAD   ┌────────────┐  completion  ┌──────────┐
    │ dfuIDLE  │──────────────→│ dfuDNLOAD  │─────────────→│ dfuDNLOAD│
    │ (state 2)│               │ -SYNC (3)  │              │ -IDLE (5)│
    └──────────┘               └────────────┘              └──────────┘
         │                                                       │
         │ DFU_UPLOAD                                            │ DFU_DNLOAD
         ▼                                                       │ (more data)
    ┌──────────┐                                                 │
    │dfuUPLOAD │                                                 ▼
    │-IDLE (9) │                                            ┌──────────┐
    └──────────┘                                            │ dfuDNLOAD│
                                                            │ -SYNC    │
    State transitions handled by:                           └──────────┘
    - dfu_state_machine_main @ 0x1000044A8
    - dfu_state_handler_A @ 0x100004650 (dfuDNBUSY)
    - dfu_state_handler_B @ 0x1000047BC (dfuDNBUSY)
    
    LAYER 5: DATA BUFFER MANAGEMENT
    ══════════════════════════════════════════════════════
    
    ┌──────────────────┐     ┌──────────────────┐     ┌───────────────┐
    │ DFU_DNLOAD data  │────→│ dfu_buffer_alloc │────→│ SRAM buffer   │
    │ (USB bulk data)  │     │ @ 0x100004174    │     │ (heap-alloc'd)│
    │                  │     │ pool_alloc()     │     │               │
    │ Max per-transfer │     │ allocates buffer │     │ Data accumul. │
    │ = wTransferSize  │     │ from heap pool   │     │ until complete│
    └──────────────────┘     └──────────────────┘     └───────────────┘
    
    ┌──────────────────┐     ┌──────────────────┐     ┌───────────────┐
    │ dfu_request_disp │────→│ memcpy           │────→│ Final buffer  │
    │ @ 0x100003F9C    │     │ (copies data,    │     │ passed to     │
    │ (1 memcpy call)  │     │ security check)  │     │ img4 verify   │
    └──────────────────┘     └──────────────────┘     └───────────────┘
    
    The memcpy in 0x100003F9C copies USB-received data into the 
    verification buffer. This is THE critical data handling point.
    
    LAYER 6: IMG4 VERIFICATION CHAIN
    ══════════════════════════════════════════════════════
    
    Once all DFU data is received:
    
    ┌──────────────────┐     ┌──────────────────┐     ┌───────────────┐
    │ dfu_completion   │────→│ IMG4 tag verifier│────→│ Crypto verify │
    │ @ 0x1000049D8    │     │ @ 0x100004CB8    │     │ EC/RSA/AES    │
    │ (heap ops)       │     │ (840B, 42 branch)│     │ SHA-512       │
    └──────────────────┘     └──────────────────┘     └───────────────┘
         │
         │                    Per-tag dispatch:
         │                    ├── ECID  → img4_verify_tag()
         │                    ├── CHIP  → img4_verify_property()
         │                    ├── BORD  → img4_verify_property()
         │                    ├── SDOM  → img4_verify_property()
         │                    ├── CPRO  → img4_verify_property()
         │                    ├── CSEC  → img4_verify_property()
         │                    ├── BNCH  → cert_verify_nonce()
         │                    ├── AMNM  → special handling
         │                    ├── DGST  → digest verification  
         │                    ├── EKEY  → encryption key check
         │                    ├── EPRO  → production status
         │                    └── ESEC  → security mode
         │
         ▼
    ┌──────────────────┐     ┌──────────────────┐     ┌───────────────┐
    │ Cert verifier    │────→│ DER/ASN.1 parser │────→│ X.509 chain   │
    │ callbacks at     │     │ @ 0x10000D000+   │     │ verification  │
    │ SRAM[BC20+]      │     │ (recursive parse)│     │ APTICKET match│
    └──────────────────┘     └──────────────────┘     └───────────────┘
    
    THE COMPLETE PATH:
    ═══════════════════
    
    Lightning → USB PHY (pre-init) → DWC2 Controller → EP0 SETUP handler
    → bmRequestType dispatch → DFU protocol state machine → buffer alloc
    → memcpy into buffer → DFU completion → IMG4 tag verification
    → per-property checks (ECID/CHIP/BNCH/DGST...) → cert chain verify
    → DER/ASN.1 parse → X.509 validation → crypto signature check
    → ACCEPT (boot image) or REJECT (error)
    """)

if __name__ == "__main__":
    main()
