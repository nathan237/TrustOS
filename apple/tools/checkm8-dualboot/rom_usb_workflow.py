#!/usr/bin/env python3
"""
T8020 B1 SecureROM — COMPLETE USB WORKFLOW ANALYSIS
=====================================================
Traces every component from the physical USB connector to SecureROM processing:
  USB PHY → USB Controller (Synopsys DWC3) → MMIO Registers → Interrupt/Polling →
  DFU State Machine → Buffer Management → IMG4/DER Processing

Analyzes:
  1. USB MMIO register map (all accesses)
  2. USB controller init sequence
  3. DFU handler state machine  
  4. USB descriptor handling
  5. Buffer allocation/management
  6. Data flow from EP0 to verify chain
  7. Interrupt/event handling
  8. Complete call graph
"""

import struct
import sys
import os
from collections import defaultdict, OrderedDict

# capstone for disassembly
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

def disasm_range(md, rom, start_off, end_off):
    """Disassemble a range of ROM offsets, return list of instructions."""
    code = rom[start_off:end_off]
    return list(md.disasm(code, ROM_BASE + start_off))

def find_functions_in_range(rom, start_off, end_off):
    """Find all function prologues (STP x29, x30, ...) in range."""
    funcs = []
    for off in range(start_off, end_off, 4):
        insn_bytes = struct.unpack_from("<I", rom, off)[0]
        # STP x29, x30, [sp, #xxx]!  (pre-index)
        if (insn_bytes & 0xFFE003E0) == 0xA9A003E0 or \
           (insn_bytes & 0xFFE003E0) == 0xA98003E0:
            funcs.append(ROM_BASE + off)
        # STP x29, x30, [sp, #xxx] (signed offset)
        elif (insn_bytes & 0xFFC003E0) == 0xA90003E0:
            # Check if x29=fp, x30=lr
            rt = insn_bytes & 0x1F
            rt2 = (insn_bytes >> 10) & 0x1F
            if rt == 29 and rt2 == 30:
                funcs.append(ROM_BASE + off)
    return funcs

def get_function_end(md, rom, func_addr):
    """Find the end of a function by looking for RET or next prologue."""
    off = func_addr - ROM_BASE
    max_search = min(off + 8192, len(rom))
    found_first = False
    for i in range(off, max_search, 4):
        insn_bytes = struct.unpack_from("<I", rom, i)[0]
        if i > off and not found_first:
            found_first = True
        # RET
        if insn_bytes == 0xD65F03C0:
            return ROM_BASE + i + 4
        # Another prologue (STP x29, x30 with pre-index writeback)
        if found_first and i > off + 4:
            if (insn_bytes & 0xFFE003E0) == 0xA9A003E0:
                return ROM_BASE + i
            rt = insn_bytes & 0x1F
            rt2 = (insn_bytes >> 10) & 0x1F
            if (insn_bytes & 0xFFC003E0) == 0xA90003E0 and rt == 29 and rt2 == 30:
                return ROM_BASE + i
    return ROM_BASE + off + 256  # default

# ============================================================
# SECTION 1: USB MMIO Register Map
# ============================================================
def analyze_mmio_registers(md, rom):
    """Find ALL MMIO register accesses in the ROM, classify by subsystem."""
    print("=" * 100)
    print("SECTION 1: USB & PERIPHERAL MMIO REGISTER MAP")
    print("=" * 100)
    
    # Known T8020 MMIO bases (from previous analysis)
    KNOWN_MMIO = {
        0x200350000: "USB_OTG (Synopsys DWC2/DWC3)",
        0x200000000: "AIC (Apple Interrupt Controller)",
        0x2000A0000: "USB_PHY (USBPHY/UTMI)",
        0x2102BC000: "PMGR (Power Manager)",
        0x200060000: "TIMER",
        0x23D2B8000: "AOP (Always-On Processor)",
        0x200100000: "GPIO",
        0x200200000: "SPI",
        0x200300000: "I2C",
        0x2003B0000: "USB_EHCI/xHCI?",
        0x2003B1000: "USB_EHCI_2?",
        0x235004000: "DART (IOMMU)",
        0x23B100000: "SEP (Secure Enclave)",
        0x200340000: "USB_DEVICE_CORE",
        0x200360000: "USB_HOST",
        0x19C000000: "SRAM / IO Region",
    }
    
    # Scan entire ROM for MMIO address construction patterns
    # Pattern: MOVZ Xn, #imm16, LSL #48 ; MOVK Xn, #imm16, LSL #32 ; MOVK Xn, #imm16, LSL #16 ; MOVK Xn, #imm
    # Or: ADRP + ADD pointing to high addresses
    
    mmio_accesses = []  # (addr_in_rom, mmio_addr, access_type, register_offset)
    
    # Disassemble entire active code region
    code_end = 0x25000  # ~148KB active
    instrs = disasm_range(md, rom, 0, code_end)
    
    # Build instruction index by address
    instr_map = {}
    for ins in instrs:
        instr_map[ins.address] = ins
    
    # Track register states for MOVZ/MOVK sequences
    reg_state = {}  # reg -> partial value being built
    
    # Collect all MMIO addresses built via MOVZ+MOVK
    mmio_refs = defaultdict(list)  # mmio_base -> [(rom_addr, full_addr, operation)]
    
    for ins in instrs:
        mnem = ins.mnemonic
        
        if mnem == 'movz' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            imm = ins.operands[1]
            if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                reg_state[dst.reg] = imm.imm << shift
                
        elif mnem == 'movk' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            imm = ins.operands[1]
            if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                if dst.reg in reg_state:
                    # Clear the bits at this shift position and set new
                    mask = ~(0xFFFF << shift)
                    reg_state[dst.reg] = (reg_state[dst.reg] & mask) | (imm.imm << shift)
                    
                    # Check if this looks like an MMIO address
                    val = reg_state[dst.reg]
                    if val >= 0x200000000 and val < 0x300000000:
                        mmio_refs[val & ~0xFFF].append((ins.address, val, "MOVK_BUILD"))
                    elif val >= 0x19C000000 and val < 0x1A0000000:
                        mmio_refs[val & ~0xFFF].append((ins.address, val, "SRAM_REF"))
        
        # Also track LDR/STR to MMIO regions (after address is built)
        elif mnem in ('ldr', 'str', 'ldp', 'stp', 'ldrb', 'strb', 'ldrh', 'strh'):
            for op in ins.operands:
                if op.type == ARM64_OP_MEM and op.mem.base != 0:
                    base_reg = op.mem.base
                    if base_reg in reg_state:
                        addr = reg_state[base_reg] + op.mem.disp
                        if addr >= 0x200000000 and addr < 0x300000000:
                            mmio_refs[addr & ~0xFFF].append((ins.address, addr, mnem.upper()))
                        elif addr >= 0x19C000000 and addr < 0x1A0000000:
                            mmio_refs[addr & ~0xFFF].append((ins.address, addr, mnem.upper()))
        
        # Clear state on branches/calls
        elif mnem in ('bl', 'blr', 'b', 'br', 'ret', 'cbz', 'cbnz', 'tbz', 'tbnz'):
            # Don't clear all, just note the boundary
            pass
    
    # Print MMIO map grouped by peripheral
    print("\n### MMIO Peripheral Map (all pages referenced in ROM)")
    print(f"{'MMIO Page':>18s}  {'Refs':>5s}  {'Subsystem':<40s}")
    print("-" * 70)
    
    usb_pages = {}
    for page in sorted(mmio_refs.keys()):
        refs = mmio_refs[page]
        subsystem = "UNKNOWN"
        for base, name in sorted(KNOWN_MMIO.items()):
            if abs(page - base) < 0x10000:
                subsystem = name
                break
        # More specific matching
        if 0x200340000 <= page < 0x200380000:
            subsystem = "USB_DEVICE_CORE"
            usb_pages[page] = refs
        elif 0x200350000 <= page < 0x200360000:
            subsystem = "USB_OTG (Synopsys DWC)"
            usb_pages[page] = refs
        elif 0x2000A0000 <= page < 0x2000B0000:
            subsystem = "USB_PHY (UTMI/PIPE)"
            usb_pages[page] = refs
        elif 0x19C000000 <= page < 0x19D000000:
            subsystem = "SRAM / IO Buffer"
            
        print(f"  0x{page:012X}  {len(refs):5d}  {subsystem}")
    
    return mmio_refs, usb_pages

# ============================================================
# SECTION 2: USB Controller Init Sequence
# ============================================================
def analyze_usb_controller_init(md, rom):
    """Disassemble and annotate the USB controller initialization region."""
    print("\n" + "=" * 100)
    print("SECTION 2: USB CONTROLLER INITIALIZATION (0x100002000 - 0x100003000)")
    print("=" * 100)
    
    # Find all functions in USB controller init region
    funcs = find_functions_in_range(rom, 0x2000, 0x3000)
    print(f"\nFound {len(funcs)} functions in USB controller init region:")
    
    # Known USB-related MMIO register names (Synopsys DWC2 standard)
    DWC2_REGS = {
        0x000: "GOTGCTL (OTG Control)",
        0x004: "GOTGINT (OTG Interrupt)",
        0x008: "GAHBCFG (AHB Config)",
        0x00C: "GUSBCFG (USB Config)",
        0x010: "GRSTCTL (Reset Control)",
        0x014: "GINTSTS (Interrupt Status)",
        0x018: "GINTMSK (Interrupt Mask)",
        0x01C: "GRXSTSR (Receive Status Read)",
        0x020: "GRXSTSP (Receive Status Pop)",
        0x024: "GRXFSIZ (Receive FIFO Size)",
        0x028: "GNPTXFSIZ (Non-periodic TX FIFO Size)",
        0x02C: "GNPTXSTS (Non-periodic TX Status)",
        0x040: "GHWCFG1 (HW Config 1)",
        0x044: "GHWCFG2 (HW Config 2)",
        0x048: "GHWCFG3 (HW Config 3)",
        0x04C: "GHWCFG4 (HW Config 4)",
        0x100: "HPTXFSIZ (Host Periodic TX FIFO Size)",
        0x104: "DIEPTXF1 (Device IN EP1 TX FIFO)",
        0x108: "DIEPTXF2 (Device IN EP2 TX FIFO)",
        0x800: "DCFG (Device Config)",
        0x804: "DCTL (Device Control)",
        0x808: "DSTS (Device Status)",
        0x810: "DIEPMSK (Device IN EP Common Mask)",
        0x814: "DOEPMSK (Device OUT EP Common Mask)",
        0x818: "DAINT (Device All EP Interrupt)",
        0x81C: "DAINTMSK (Device All EP Interrupt Mask)",
        0x828: "DTHRCTL (Device Threshold Control)",
        0x830: "DIEPEMPMSK (Device IN EP FIFO Empty Mask)",
        0x900: "DIEPCTL0 (Device IN EP0 Control)",
        0x908: "DIEPINT0 (Device IN EP0 Interrupt)",
        0x910: "DIEPTSIZ0 (Device IN EP0 Transfer Size)",
        0x914: "DIEPDMA0 (Device IN EP0 DMA Address)",
        0x918: "DTXFSTS0 (Device IN EP0 TX FIFO Status)",
        0xB00: "DOEPCTL0 (Device OUT EP0 Control)",
        0xB08: "DOEPINT0 (Device OUT EP0 Interrupt)",
        0xB10: "DOEPTSIZ0 (Device OUT EP0 Transfer Size)",
        0xB14: "DOEPDMA0 (Device OUT EP0 DMA Address)",
        0xE00: "PCGCCTL (Power/Clock Gating Control)",
    }
    
    # For each endpoint N (1-15), registers are at base + 0x20*N from EP0
    for ep in range(1, 8):
        DWC2_REGS[0x900 + 0x20 * ep] = f"DIEPCTL{ep} (Device IN EP{ep} Control)"
        DWC2_REGS[0x908 + 0x20 * ep] = f"DIEPINT{ep} (Device IN EP{ep} Interrupt)"
        DWC2_REGS[0x910 + 0x20 * ep] = f"DIEPTSIZ{ep} (Device IN EP{ep} Transfer Size)"
        DWC2_REGS[0x914 + 0x20 * ep] = f"DIEPDMA{ep} (Device IN EP{ep} DMA Address)"
        DWC2_REGS[0xB00 + 0x20 * ep] = f"DOEPCTL{ep} (Device OUT EP{ep} Control)"
        DWC2_REGS[0xB08 + 0x20 * ep] = f"DOEPINT{ep} (Device OUT EP{ep} Interrupt)"
        DWC2_REGS[0xB10 + 0x20 * ep] = f"DOEPTSIZ{ep} (Device OUT EP{ep} Transfer Size)"
        DWC2_REGS[0xB14 + 0x20 * ep] = f"DOEPDMA{ep} (Device OUT EP{ep} DMA Address)"
    
    # Disassemble entire USB init region with annotations
    for i, func_addr in enumerate(funcs):
        func_end = funcs[i + 1] if i + 1 < len(funcs) else ROM_BASE + 0x3000
        func_size = func_end - func_addr
        off = func_addr - ROM_BASE
        
        print(f"\n{'─' * 80}")
        print(f"FUNCTION @ 0x{func_addr:X} (size: {func_size} bytes)")
        print(f"{'─' * 80}")
        
        instrs = disasm_range(md, rom, off, off + func_size)
        
        reg_vals = {}
        calls = []
        mmio_ops = []
        
        for ins in instrs:
            # Track MOVZ/MOVK
            if ins.mnemonic == 'movz' and len(ins.operands) >= 2:
                dst = ins.operands[0]
                imm = ins.operands[1]
                if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                    shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                    reg_vals[dst.reg] = imm.imm << shift
                    
            elif ins.mnemonic == 'movk' and len(ins.operands) >= 2:
                dst = ins.operands[0]
                imm = ins.operands[1]
                if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                    shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                    if dst.reg in reg_vals:
                        mask = ~(0xFFFF << shift)
                        reg_vals[dst.reg] = (reg_vals[dst.reg] & mask) | (imm.imm << shift)
            
            annotation = ""
            
            # Annotate calls
            if ins.mnemonic == 'bl':
                target = ins.operands[0].imm if ins.operands else 0
                calls.append(target)
                # Known functions
                known = {
                    0x100010BD0: "memcpy",
                    0x100010E00: "memset",
                    0x100010D80: "bzero",
                    0x10000F1EC: "heap_alloc",
                    0x100008978: "panic",
                    0x10000F3B0: "pool_alloc",
                }
                if target in known:
                    annotation = f"  ← {known[target]}()"
                else:
                    annotation = f"  ← sub_{target:X}()"
            
            # Annotate MMIO accesses
            if ins.mnemonic in ('ldr', 'str', 'ldrb', 'strb', 'ldrh', 'strh'):
                for op in ins.operands:
                    if op.type == ARM64_OP_MEM and op.mem.base != 0:
                        if op.mem.base in reg_vals:
                            addr = reg_vals[op.mem.base] + op.mem.disp
                            if addr >= 0x200000000:
                                # Check if it's a USB register
                                usb_base = 0x200350000
                                offset = addr - usb_base
                                if 0 <= offset < 0x10000:
                                    reg_name = DWC2_REGS.get(offset, f"REG_0x{offset:03X}")
                                    if ins.mnemonic.startswith('ldr'):
                                        annotation = f"  ← READ USB {reg_name} @ 0x{addr:X}"
                                    else:
                                        annotation = f"  ← WRITE USB {reg_name} @ 0x{addr:X}"
                                    mmio_ops.append((ins.address, addr, ins.mnemonic, reg_name))
                                else:
                                    if ins.mnemonic.startswith('ldr'):
                                        annotation = f"  ← READ MMIO 0x{addr:X}"
                                    else:
                                        annotation = f"  ← WRITE MMIO 0x{addr:X}"
            
            # Annotate MOV immediate for important constants
            if ins.mnemonic == 'mov' and len(ins.operands) >= 2:
                if ins.operands[1].type == ARM64_OP_IMM:
                    val = ins.operands[1].imm
                    if val in (0x05AC, 0x1227, 0x1281):
                        names = {0x05AC: "Apple VID", 0x1227: "DFU PID", 0x1281: "Recovery PID"}
                        annotation = f"  ← USB {names.get(val, '')} = 0x{val:04X}"
            
            print(f"  0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:40s}{annotation}")
        
        if calls:
            print(f"\n  Calls: {', '.join(f'0x{c:X}' for c in calls)}")
        if mmio_ops:
            print(f"  MMIO ops: {len(mmio_ops)} USB register accesses")

# ============================================================  
# SECTION 3: DFU State Machine
# ============================================================
def analyze_dfu_handler(md, rom):
    """Deep analysis of the DFU handler region — the core attack surface."""
    print("\n" + "=" * 100)
    print("SECTION 3: DFU HANDLER STATE MACHINE (0x100003000 - 0x100005000)")
    print("=" * 100)
    
    # Find all functions in DFU handler region
    funcs = find_functions_in_range(rom, 0x3000, 0x5000)
    print(f"\nFound {len(funcs)} functions in DFU handler region")
    
    # Known functions for reference
    KNOWN = {
        0x100010BD0: "memcpy",
        0x100010E00: "memset",
        0x100010D80: "bzero",
        0x10000F1EC: "heap_alloc",
        0x100008978: "panic",
        0x10000F3B0: "pool_alloc",
    }
    
    # DFU protocol states (USB DFU spec)
    DFU_STATES = {
        0: "appIDLE",
        1: "appDETACH",
        2: "dfuIDLE",
        3: "dfuDNLOAD-SYNC",
        4: "dfuDNBUSY",
        5: "dfuDNLOAD-IDLE",
        6: "dfuMANIFEST-SYNC",
        7: "dfuMANIFEST",
        8: "dfuMANIFEST-WAIT-RESET",
        9: "dfuUPLOAD-IDLE",
        10: "dfuERROR",
    }
    
    # DFU requests
    DFU_REQUESTS = {
        0: "DFU_DETACH",
        1: "DFU_DNLOAD",
        2: "DFU_UPLOAD",
        3: "DFU_GETSTATUS",
        4: "DFU_CLRSTATUS",
        5: "DFU_GETSTATE",
        6: "DFU_ABORT",
    }
    
    # Analyze each function with detailed annotation
    func_analysis = []
    
    for i, func_addr in enumerate(funcs):
        func_end = funcs[i + 1] if i + 1 < len(funcs) else ROM_BASE + 0x5000
        func_size = func_end - func_addr
        off = func_addr - ROM_BASE
        
        instrs = disasm_range(md, rom, off, off + func_size)
        
        # Collect statistics
        calls = []
        branches = 0
        mmio_ops = 0
        memcpy_calls = 0
        heap_ops = 0
        comparisons = []
        state_checks = []
        reg_vals = {}
        has_panic = False
        has_usb_vid_pid = False
        buffer_ops = []
        
        for ins in instrs:
            # Track MOVZ/MOVK for register values
            if ins.mnemonic == 'movz' and len(ins.operands) >= 2:
                dst = ins.operands[0]
                imm = ins.operands[1]
                if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                    shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                    reg_vals[dst.reg] = imm.imm << shift
            elif ins.mnemonic == 'movk' and len(ins.operands) >= 2:
                dst = ins.operands[0]
                imm = ins.operands[1]
                if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                    shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                    if dst.reg in reg_vals:
                        mask = ~(0xFFFF << shift)
                        reg_vals[dst.reg] = (reg_vals[dst.reg] & mask) | (imm.imm << shift)
            
            if ins.mnemonic == 'bl':
                target = ins.operands[0].imm if ins.operands else 0
                calls.append(target)
                if target == 0x100010BD0:
                    memcpy_calls += 1
                    buffer_ops.append(("memcpy", ins.address))
                elif target in (0x10000F1EC, 0x10000F3B0):
                    heap_ops += 1
                    buffer_ops.append(("heap_alloc", ins.address))
                elif target == 0x100008978:
                    has_panic = True
                    
            elif ins.mnemonic in ('b.eq', 'b.ne', 'b.gt', 'b.lt', 'b.ge', 'b.le', 
                                   'b.hi', 'b.lo', 'b.hs', 'b.ls', 'b', 'cbz', 'cbnz',
                                   'tbz', 'tbnz'):
                branches += 1
                
            elif ins.mnemonic == 'cmp':
                if len(ins.operands) >= 2 and ins.operands[1].type == ARM64_OP_IMM:
                    cmp_val = ins.operands[1].imm
                    comparisons.append((ins.address, cmp_val))
                    # Check for DFU state values
                    if cmp_val in DFU_STATES:
                        state_checks.append((ins.address, cmp_val, DFU_STATES[cmp_val]))
                    # USB descriptor type checks
                    if cmp_val in (1, 2, 3, 4, 5, 6, 9, 0x21, 0x24):
                        desc_types = {1: "DEVICE", 2: "CONFIG", 3: "STRING", 4: "INTERFACE",
                                     5: "ENDPOINT", 6: "DEVICE_QUALIFIER", 9: "CONFIG",
                                     0x21: "DFU_FUNCTIONAL", 0x24: "CS_INTERFACE"}
                        state_checks.append((ins.address, cmp_val, f"USB_DESC_{desc_types.get(cmp_val, '?')}"))
                    # DFU request checks
                    if cmp_val in DFU_REQUESTS:
                        state_checks.append((ins.address, cmp_val, f"DFU_{DFU_REQUESTS[cmp_val]}"))
            
            # Check for VID/PID
            if ins.mnemonic in ('movz', 'mov', 'movk'):
                for op in ins.operands:
                    if op.type == ARM64_OP_IMM:
                        if op.imm in (0x05AC, 0x1227, 0x1281):
                            has_usb_vid_pid = True
                            
            # MMIO access counting
            if ins.mnemonic in ('ldr', 'str'):
                for op in ins.operands:
                    if op.type == ARM64_OP_MEM and op.mem.base != 0:
                        if op.mem.base in reg_vals:
                            addr = reg_vals[op.mem.base] + op.mem.disp
                            if addr >= 0x200000000:
                                mmio_ops += 1
        
        analysis = {
            'addr': func_addr,
            'size': func_size,
            'calls': calls,
            'branches': branches,
            'mmio_ops': mmio_ops,
            'memcpy': memcpy_calls,
            'heap_ops': heap_ops,
            'comparisons': comparisons,
            'state_checks': state_checks,
            'has_panic': has_panic,
            'has_usb_vid_pid': has_usb_vid_pid,
            'buffer_ops': buffer_ops,
            'num_instrs': len(instrs),
        }
        func_analysis.append(analysis)
    
    # Print summary table
    print(f"\n{'Addr':>14s} {'Size':>6s} {'Instr':>5s} {'Calls':>5s} {'Br':>4s} {'MMIO':>4s} {'MCpy':>4s} {'Heap':>4s} {'Panic':>5s} {'VID':>4s} {'StateChks'}")
    print("-" * 110)
    
    for a in func_analysis:
        sc = "; ".join(f"{n}" for _,_,n in a['state_checks']) if a['state_checks'] else ""
        print(f"  0x{a['addr']:X}  {a['size']:5d}  {a['num_instrs']:5d}  {len(a['calls']):5d}  {a['branches']:4d}  "
              f"{a['mmio_ops']:4d}  {a['memcpy']:4d}  {a['heap_ops']:4d}  {'YES' if a['has_panic'] else '':>5s}  "
              f"{'YES' if a['has_usb_vid_pid'] else '':>4s}  {sc}")
    
    # Now do DETAILED disassembly of the largest / most interesting functions
    # Sort by size descending to find the main DFU handler
    by_size = sorted(func_analysis, key=lambda x: x['size'], reverse=True)
    
    print(f"\n\n### TOP 10 LARGEST DFU FUNCTIONS (detailed disassembly)")
    
    for rank, a in enumerate(by_size[:10]):
        func_addr = a['addr']
        func_size = a['size']
        off = func_addr - ROM_BASE
        
        role = "UNKNOWN"
        if a['has_usb_vid_pid']:
            role = "USB DESCRIPTOR SETUP"
        elif a['state_checks']:
            state_names = [n for _, _, n in a['state_checks']]
            if any('DFU_' in s for s in state_names):
                role = "DFU REQUEST HANDLER"
            elif any('dfuDNLOAD' in s for s in state_names):
                role = "DFU DOWNLOAD HANDLER"
        if a['memcpy'] > 0 and a['heap_ops'] > 0:
            role += " + BUFFER MANAGEMENT"
        elif a['memcpy'] > 0:
            role += " + DATA COPY"
        if a['mmio_ops'] > 3:
            role += " + HW ACCESS"
            
        print(f"\n{'━' * 90}")
        print(f"  RANK #{rank+1}: 0x{func_addr:X} — {func_size} bytes, {a['branches']} branches")
        print(f"  ROLE: {role}")
        print(f"  Calls {len(a['calls'])} functions, {a['memcpy']} memcpy, {a['heap_ops']} heap ops")
        if a['state_checks']:
            print(f"  State checks: {', '.join(n for _,_,n in a['state_checks'])}")
        if a['comparisons']:
            print(f"  CMP values: {', '.join(f'0x{v:X}({v})' for _,v in a['comparisons'][:20])}")
        print(f"{'━' * 90}")
        
        # Full disassembly with annotations
        instrs = disasm_range(md, rom, off, off + func_size)
        reg_vals = {}
        
        for ins in instrs:
            # Track registers
            if ins.mnemonic == 'movz' and len(ins.operands) >= 2:
                dst = ins.operands[0]
                imm = ins.operands[1]
                if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                    shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                    reg_vals[dst.reg] = imm.imm << shift
            elif ins.mnemonic == 'movk' and len(ins.operands) >= 2:
                dst = ins.operands[0]
                imm = ins.operands[1]
                if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                    shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                    if dst.reg in reg_vals:
                        mask = ~(0xFFFF << shift)
                        reg_vals[dst.reg] = (reg_vals[dst.reg] & mask) | (imm.imm << shift)
            
            ann = ""
            
            # Annotate calls
            if ins.mnemonic == 'bl':
                target = ins.operands[0].imm if ins.operands else 0
                if target in KNOWN:
                    ann = f"  ◄━━ {KNOWN[target]}()"
                elif 0x100003000 <= target < 0x100005000:
                    ann = f"  ◄━━ dfu_sub_{target:X}()"
                elif 0x100002000 <= target < 0x100003000:
                    ann = f"  ◄━━ usb_sub_{target:X}()"
                else:
                    ann = f"  ◄━━ sub_{target:X}()"
            
            # Annotate comparisons
            elif ins.mnemonic == 'cmp':
                if len(ins.operands) >= 2 and ins.operands[1].type == ARM64_OP_IMM:
                    v = ins.operands[1].imm
                    if v in DFU_STATES:
                        ann = f"  ◄━━ CHECK: {DFU_STATES[v]}"
                    elif v in DFU_REQUESTS:
                        ann = f"  ◄━━ CHECK: DFU {DFU_REQUESTS[v]}"
                    elif v == 0x05AC:
                        ann = "  ◄━━ Apple VID"
                    elif v == 0x1227:
                        ann = "  ◄━━ DFU PID"
                    elif v in (1, 2, 3, 4, 5, 6, 9):
                        desc = {1: "DEVICE", 2: "CONFIG", 3: "STRING", 4: "INTERFACE",
                               5: "ENDPOINT", 6: "DEVICE_QUALIFIER", 9: "CONFIG"}
                        ann = f"  ◄━━ USB DESC TYPE: {desc.get(v, '?')}"
                    elif v == 0x21:
                        ann = "  ◄━━ DFU FUNCTIONAL DESC"
                    elif v == 0x80:
                        ann = "  ◄━━ USB DIR: DEVICE→HOST"
                    elif v == 0x00:
                        ann = "  ◄━━ USB DIR: HOST→DEVICE"
                    elif v == 0x21:
                        ann = "  ◄━━ USB bmRequestType: CLASS|INTERFACE"
                    elif v == 0xA1:
                        ann = "  ◄━━ USB bmRequestType: CLASS|INTERFACE|DEV→HOST"
                    elif v == 0x40:
                        ann = "  ◄━━ USB bmRequestType: VENDOR"
                    else:
                        ann = f"  ◄━━ cmp #{v} (0x{v:X})"
            
            # Annotate important immediates
            elif ins.mnemonic in ('mov', 'movz'):
                for op in ins.operands:
                    if op.type == ARM64_OP_IMM:
                        v = op.imm
                        if v == 0x05AC: ann = "  ◄━━ Apple VID"
                        elif v == 0x1227: ann = "  ◄━━ DFU PID"
                        elif v == 0x1281: ann = "  ◄━━ Recovery PID"
                        elif v == 0x200: ann = "  ◄━━ USB_EP0_MAX_PACKET_SIZE? (512)"
                        elif v == 0x40: ann = "  ◄━━ USB_EP0_FS_MAX (64)"
                        elif v == 0x800: ann = "  ◄━━ DFU_BUFFER_SIZE? (2048)"
                        elif v == 0x80000: ann = "  ◄━━ 512KB (IMG4 max?)"
                        elif v == 0x10000: ann = "  ◄━━ 64KB"
            
            # MMIO annotations
            if ins.mnemonic in ('ldr', 'str', 'ldrb', 'strb', 'ldrh', 'strh', 'ldp', 'stp'):
                for op in ins.operands:
                    if op.type == ARM64_OP_MEM and op.mem.base != 0:
                        if op.mem.base in reg_vals:
                            addr = reg_vals[op.mem.base] + op.mem.disp
                            if 0x200340000 <= addr < 0x200360000:
                                usb_off = addr - 0x200350000
                                DWC2_REGS_local = {
                                    0x000: "GOTGCTL", 0x008: "GAHBCFG", 0x00C: "GUSBCFG",
                                    0x010: "GRSTCTL", 0x014: "GINTSTS", 0x018: "GINTMSK",
                                    0x800: "DCFG", 0x804: "DCTL", 0x808: "DSTS",
                                    0x900: "DIEPCTL0", 0xB00: "DOEPCTL0",
                                    0x910: "DIEPTSIZ0", 0xB10: "DOEPTSIZ0",
                                }
                                rname = DWC2_REGS_local.get(usb_off, f"USB_0x{usb_off:03X}")
                                rw = "READ" if ins.mnemonic.startswith('ldr') or ins.mnemonic == 'ldp' else "WRITE"
                                ann = f"  ◄━━ {rw} {rname} @ 0x{addr:X}"
                            elif addr >= 0x19C000000 and addr < 0x1A0000000:
                                ann = f"  ◄━━ SRAM/IO @ 0x{addr:X}"
            
            print(f"  0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:45s}{ann}")
    
    return func_analysis

# ============================================================
# SECTION 4: USB Descriptor Structures
# ============================================================
def analyze_usb_descriptors(md, rom):
    """Find and decode USB descriptor structures in the ROM data section."""
    print("\n" + "=" * 100)
    print("SECTION 4: USB DESCRIPTOR STRUCTURES")
    print("=" * 100)
    
    # USB descriptors are typically in the data section
    # Search for Apple VID (0x05AC) and DFU PID (0x1227) in data
    data_start = 0x1BC00
    data_end = 0x22000
    
    # Search for 0xAC05 (little-endian 0x05AC) followed by 0x2712 (little-endian 0x1227)
    print("\n### Scanning for USB Device Descriptor patterns...")
    
    for off in range(data_start, data_end - 20):
        # USB Device Descriptor: bLength=18, bDescriptorType=1, ...
        val = rom[off:off+2]
        if len(val) >= 2:
            # Check for VID at various offsets
            vid = struct.unpack_from("<H", rom, off)[0]
            if vid == 0x05AC:
                # Try to parse as device descriptor (VID is at offset 8)
                if off >= 8:
                    desc_start = off - 8
                    bLength = rom[desc_start]
                    bDescType = rom[desc_start + 1]
                    if bLength == 18 and bDescType == 1:
                        bcdUSB = struct.unpack_from("<H", rom, desc_start + 2)[0]
                        bDevClass = rom[desc_start + 4]
                        bDevSubClass = rom[desc_start + 5]
                        bDevProtocol = rom[desc_start + 6]
                        bMaxPacketSize = rom[desc_start + 7]
                        idVendor = struct.unpack_from("<H", rom, desc_start + 8)[0]
                        idProduct = struct.unpack_from("<H", rom, desc_start + 10)[0]
                        bcdDevice = struct.unpack_from("<H", rom, desc_start + 12)[0]
                        iManufacturer = rom[desc_start + 14]
                        iProduct = rom[desc_start + 15]
                        iSerialNumber = rom[desc_start + 16]
                        bNumConfigs = rom[desc_start + 17]
                        
                        print(f"\n  ★ USB Device Descriptor @ ROM offset 0x{desc_start:X} (0x{ROM_BASE + desc_start:X}):")
                        print(f"    bLength         = {bLength}")
                        print(f"    bDescriptorType = {bDescType} (DEVICE)")
                        print(f"    bcdUSB          = 0x{bcdUSB:04X} (USB {bcdUSB >> 8}.{(bcdUSB >> 4) & 0xF}{bcdUSB & 0xF})")
                        print(f"    bDeviceClass    = 0x{bDevClass:02X}")
                        print(f"    bDeviceSubClass = 0x{bDevSubClass:02X}")
                        print(f"    bDeviceProtocol = 0x{bDevProtocol:02X}")
                        print(f"    bMaxPacketSize0 = {bMaxPacketSize}")
                        print(f"    idVendor        = 0x{idVendor:04X} (Apple Inc.)")
                        print(f"    idProduct       = 0x{idProduct:04X}")
                        print(f"    bcdDevice       = 0x{bcdDevice:04X}")
                        print(f"    iManufacturer   = {iManufacturer}")
                        print(f"    iProduct        = {iProduct}")
                        print(f"    iSerialNumber   = {iSerialNumber}")
                        print(f"    bNumConfigs     = {bNumConfigs}")
    
    # Search for DFU Functional Descriptor (bDescriptorType = 0x21)
    print("\n### Scanning for DFU Functional Descriptor...")
    for off in range(data_start, data_end - 10):
        if rom[off + 1] == 0x21:  # DFU functional descriptor type
            bLength = rom[off]
            if bLength == 9:  # Standard DFU functional descriptor is 9 bytes
                bmAttributes = rom[off + 2]
                wDetachTimeout = struct.unpack_from("<H", rom, off + 3)[0]
                wTransferSize = struct.unpack_from("<H", rom, off + 5)[0]
                bcdDFUVersion = struct.unpack_from("<H", rom, off + 7)[0]
                
                can_download = "YES" if bmAttributes & 1 else "NO"
                can_upload = "YES" if bmAttributes & 2 else "NO"
                manifestation_tolerant = "YES" if bmAttributes & 4 else "NO"
                will_detach = "YES" if bmAttributes & 8 else "NO"
                
                print(f"\n  ★ DFU Functional Descriptor @ ROM offset 0x{off:X} (0x{ROM_BASE + off:X}):")
                print(f"    bLength            = {bLength}")
                print(f"    bDescriptorType    = 0x21 (DFU_FUNCTIONAL)")
                print(f"    bmAttributes       = 0x{bmAttributes:02X}")
                print(f"      Can Download     = {can_download}")
                print(f"      Can Upload       = {can_upload}")
                print(f"      Manifestation    = {manifestation_tolerant}")
                print(f"      Will Detach      = {will_detach}")
                print(f"    wDetachTimeout     = {wDetachTimeout} ms")
                print(f"    wTransferSize      = {wTransferSize} bytes ★★★ MAX DFU TRANSFER SIZE")
                print(f"    bcdDFUVersion      = 0x{bcdDFUVersion:04X}")
    
    # Search for configuration descriptor
    print("\n### Scanning for USB Configuration Descriptors...")
    for off in range(data_start, data_end - 10):
        bLen = rom[off]
        bType = rom[off + 1]
        if bLen == 9 and bType == 2:  # Configuration descriptor
            wTotalLength = struct.unpack_from("<H", rom, off + 2)[0]
            bNumInterfaces = rom[off + 4]
            bConfigValue = rom[off + 5]
            iConfig = rom[off + 6]
            bmAttrib = rom[off + 7]
            bMaxPower = rom[off + 8]
            
            if wTotalLength > 9 and wTotalLength < 256 and bNumInterfaces > 0 and bNumInterfaces < 8:
                print(f"\n  ★ USB Configuration Descriptor @ ROM offset 0x{off:X} (0x{ROM_BASE + off:X}):")
                print(f"    bLength            = {bLen}")
                print(f"    bDescriptorType    = {bType} (CONFIGURATION)")
                print(f"    wTotalLength       = {wTotalLength}")
                print(f"    bNumInterfaces     = {bNumInterfaces}")
                print(f"    bConfigurationVal  = {bConfigValue}")
                print(f"    iConfiguration     = {iConfig}")
                print(f"    bmAttributes       = 0x{bmAttrib:02X}")
                print(f"    bMaxPower          = {bMaxPower} (= {bMaxPower * 2} mA)")
                
                # Dump remaining bytes as sub-descriptors
                pos = off + 9
                end = off + wTotalLength
                while pos < end and pos < data_end:
                    sub_len = rom[pos]
                    sub_type = rom[pos + 1] if pos + 1 < data_end else 0
                    if sub_len == 0:
                        break
                    desc_names = {1: "DEVICE", 2: "CONFIG", 3: "STRING", 4: "INTERFACE",
                                 5: "ENDPOINT", 0x21: "DFU_FUNCTIONAL", 0x24: "CS_INTERFACE"}
                    name = desc_names.get(sub_type, f"UNKNOWN(0x{sub_type:02X})")
                    hexdata = " ".join(f"{rom[pos+j]:02X}" for j in range(min(sub_len, 16)))
                    print(f"      Sub-desc @ +{pos-off}: type={name}, len={sub_len}, data=[{hexdata}]")
                    pos += sub_len
    
    # Also search for string descriptors (type 3)
    print("\n### Scanning for USB String Descriptors in data region...")
    # String descriptors: bLength, bDescriptorType=3, wString (UTF-16LE)
    for off in range(data_start, data_end - 4):
        bLen = rom[off]
        bType = rom[off + 1]
        if bType == 3 and bLen >= 4 and bLen <= 128 and bLen % 2 == 0:
            # Try to decode as UTF-16LE
            string_data = rom[off + 2:off + bLen]
            try:
                text = string_data.decode('utf-16-le')
                if all(c.isprintable() or c == '\x00' for c in text) and len(text) > 1:
                    print(f"  String Descriptor @ 0x{off:X}: \"{text}\"")
            except:
                pass
    
    # Search in code region too — descriptors might be built inline
    print("\n### Scanning for inline VID/PID references in code...")
    code_end = 0x1BC00
    instrs = disasm_range(md, rom, 0, code_end)
    
    vid_pid_refs = []
    for ins in instrs:
        for op in ins.operands:
            if op.type == ARM64_OP_IMM:
                if op.imm in (0x05AC, 0x1227, 0x1281, 0x1226):
                    names = {0x05AC: "Apple VID", 0x1227: "DFU Product ID", 
                             0x1281: "Recovery Product ID", 0x1226: "iBoot Product ID"}
                    vid_pid_refs.append((ins.address, op.imm, names.get(op.imm, "?")))
                    print(f"  0x{ins.address:X}: {ins.mnemonic} {ins.op_str}  ← {names.get(op.imm, '?')}")

# ============================================================
# SECTION 5: Interrupt/Event Handling
# ============================================================
def analyze_interrupt_handling(md, rom):
    """Analyze exception vector table and USB interrupt handling."""
    print("\n" + "=" * 100)
    print("SECTION 5: INTERRUPT & EVENT HANDLING")
    print("=" * 100)
    
    # Exception vector table at 0x100000000 (VBAR_EL1)
    # Each entry is 0x80 (128) bytes, 16 entries total
    print("\n### Exception Vector Table @ 0x100000000 (VBAR_EL1)")
    
    VECTOR_NAMES = [
        "Current EL SP0 - Synchronous",
        "Current EL SP0 - IRQ",
        "Current EL SP0 - FIQ",
        "Current EL SP0 - SError",
        "Current EL SPx - Synchronous",
        "Current EL SPx - IRQ",
        "Current EL SPx - FIQ",
        "Current EL SPx - SError",
        "Lower EL AArch64 - Synchronous",
        "Lower EL AArch64 - IRQ",
        "Lower EL AArch64 - FIQ",
        "Lower EL AArch64 - SError",
        "Lower EL AArch32 - Synchronous",
        "Lower EL AArch32 - IRQ",
        "Lower EL AArch32 - FIQ",
        "Lower EL AArch32 - SError",
    ]
    
    for idx in range(16):
        vec_off = idx * 0x80
        vec_addr = ROM_BASE + vec_off
        
        # Disassemble first few instructions
        instrs = disasm_range(md, rom, vec_off, vec_off + 0x80)
        
        # Determine if vector is used (first insn is not just branch to self or eret)
        first_mnem = instrs[0].mnemonic if instrs else "?"
        branches_to = None
        
        # Find the target if it branches
        for ins in instrs[:8]:
            if ins.mnemonic == 'b' and len(ins.operands) > 0 and ins.operands[0].type == ARM64_OP_IMM:
                branches_to = ins.operands[0].imm
                break
        
        active = "ACTIVE" if first_mnem not in ('eret', 'wfi') else "UNUSED"
        target_str = f" → 0x{branches_to:X}" if branches_to else ""
        
        print(f"\n  Vector #{idx:2d} @ 0x{vec_addr:X} [{VECTOR_NAMES[idx]:40s}] {active}{target_str}")
        
        for ins in instrs[:6]:
            ann = ""
            if ins.mnemonic == 'mrs':
                ops = ins.op_str
                if 'esr_el1' in ops.lower() or 's3_0_c5_c2_0' in ops.lower():
                    ann = "  ← Exception Syndrome Register"
                elif 'far_el1' in ops.lower() or 's3_0_c6_c0_0' in ops.lower():
                    ann = "  ← Fault Address Register"
                elif 'elr_el1' in ops.lower():
                    ann = "  ← Exception Link Register (return addr)"
                elif 'spsr_el1' in ops.lower():
                    ann = "  ← Saved Program Status Register"
            elif ins.mnemonic == 'msr':
                ops = ins.op_str
                if 'vbar_el1' in ops.lower():
                    ann = "  ← Setting new Vector Base!"
            print(f"    0x{ins.address:X}: {ins.mnemonic:8s} {ins.op_str:40s}{ann}")
    
    # Analyze Apple Interrupt Controller (AIC) references
    print("\n\n### Apple Interrupt Controller (AIC) References")
    code_end = 0x25000
    instrs = disasm_range(md, rom, 0, code_end)
    
    reg_vals = {}
    aic_refs = []
    
    for ins in instrs:
        if ins.mnemonic == 'movz' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            imm = ins.operands[1]
            if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                reg_vals[dst.reg] = imm.imm << shift
        elif ins.mnemonic == 'movk' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            imm = ins.operands[1]
            if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                if dst.reg in reg_vals:
                    mask = ~(0xFFFF << shift)
                    reg_vals[dst.reg] = (reg_vals[dst.reg] & mask) | (imm.imm << shift)
                    val = reg_vals[dst.reg]
                    # Check for AIC base (0x200000000 range)
                    if 0x200000000 <= val < 0x200010000:
                        aic_refs.append((ins.address, val))
    
    if aic_refs:
        for addr, val in aic_refs:
            print(f"  0x{addr:X}: AIC register reference → 0x{val:X}")
    else:
        print("  No direct AIC MMIO references found — SecureROM likely POLLS, not interrupts")
    
    # Search for WFI/WFE (wait for interrupt/event) instructions
    print("\n### WFI/WFE Instructions (interrupt wait points)")
    for ins in instrs:
        if ins.mnemonic in ('wfi', 'wfe'):
            print(f"  0x{ins.address:X}: {ins.mnemonic}  ← CPU halts until interrupt/event")

# ============================================================
# SECTION 6: Buffer Management & Data Flow
# ============================================================
def analyze_buffer_management(md, rom):
    """Trace USB data buffer allocation, filling, and consumption."""
    print("\n" + "=" * 100)
    print("SECTION 6: BUFFER MANAGEMENT & DATA FLOW")
    print("=" * 100)
    
    # Key questions:
    # 1. Where is the DFU upload buffer allocated?
    # 2. What is its size?
    # 3. How is data copied from USB to buffer?
    # 4. What checks are done before the copy?
    # 5. Where is the buffer passed to img4_verify?
    
    # Scan for heap_alloc calls in DFU region (0x3000-0x5000)
    print("\n### Heap Allocations in DFU Region")
    
    code_start = 0x2000
    code_end = 0x5000
    instrs = disasm_range(md, rom, code_start, code_end)
    
    KNOWN = {
        0x100010BD0: "memcpy",
        0x100010E00: "memset", 
        0x100010D80: "bzero",
        0x10000F1EC: "heap_alloc",
        0x100008978: "panic",
        0x10000F3B0: "pool_alloc",
    }
    
    # Track the context around each alloc/memcpy call
    window_size = 15  # instructions before/after
    instr_list = list(instrs)
    
    for idx, ins in enumerate(instr_list):
        if ins.mnemonic == 'bl':
            target = ins.operands[0].imm if ins.operands and ins.operands[0].type == ARM64_OP_IMM else 0
            if target in (0x10000F1EC, 0x10000F3B0, 0x100010BD0, 0x100010E00, 0x100010D80):
                fname = KNOWN.get(target, "?")
                print(f"\n  ★ {fname}() called at 0x{ins.address:X}")
                print(f"  Context (±{window_size} instructions):")
                
                start_idx = max(0, idx - window_size)
                end_idx = min(len(instr_list), idx + window_size + 1)
                
                for j in range(start_idx, end_idx):
                    ci = instr_list[j]
                    marker = " >>>>" if j == idx else "     "
                    
                    ann = ""
                    if ci.mnemonic == 'bl':
                        t = ci.operands[0].imm if ci.operands and ci.operands[0].type == ARM64_OP_IMM else 0
                        if t in KNOWN:
                            ann = f"  ← {KNOWN[t]}()"
                    elif ci.mnemonic == 'mov' and len(ci.operands) >= 2:
                        if ci.operands[1].type == ARM64_OP_IMM:
                            v = ci.operands[1].imm
                            if v > 0x100:
                                ann = f"  ← SIZE=0x{v:X} ({v})"
                    elif ci.mnemonic == 'movz' and len(ci.operands) >= 2:
                        if ci.operands[1].type == ARM64_OP_IMM:
                            v = ci.operands[1].imm
                            shift = ci.operands[1].shift.value if ci.operands[1].shift.type != ARM64_SFT_INVALID else 0
                            actual = v << shift
                            if actual >= 0x40:
                                ann = f"  ← VALUE=0x{actual:X} ({actual})"
                    elif ci.mnemonic == 'cmp' and len(ci.operands) >= 2:
                        if ci.operands[1].type == ARM64_OP_IMM:
                            ann = f"  ← BOUNDS CHECK #{ci.operands[1].imm} (0x{ci.operands[1].imm:X})"
                    
                    print(f"  {marker} 0x{ci.address:X}: {ci.mnemonic:8s} {ci.op_str:40s}{ann}")
    
    # Look for SRAM buffer references (0x19C000000 range)
    print("\n\n### SRAM Buffer References (static buffers)")
    instrs = disasm_range(md, rom, 0, 0x25000)
    
    sram_refs = defaultdict(list)
    reg_vals = {}
    
    for ins in instrs:
        if ins.mnemonic == 'movz' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            imm = ins.operands[1]
            if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                reg_vals[dst.reg] = imm.imm << shift
        elif ins.mnemonic == 'movk' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            imm = ins.operands[1]
            if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                if dst.reg in reg_vals:
                    mask = ~(0xFFFF << shift)
                    reg_vals[dst.reg] = (reg_vals[dst.reg] & mask) | (imm.imm << shift)
                    val = reg_vals[dst.reg]
                    if 0x19C000000 <= val < 0x1A0000000:
                        sram_refs[val & ~0xFFF].append((ins.address, val))
    
    print(f"\n  {'SRAM Page':>18s}  {'Refs':>5s}  {'Specific Addresses'}")
    print("  " + "-" * 70)
    for page in sorted(sram_refs.keys()):
        refs = sram_refs[page]
        addrs = sorted(set(v for _, v in refs))
        addrs_str = ", ".join(f"0x{a:X}" for a in addrs[:10])
        print(f"    0x{page:012X}  {len(refs):5d}  {addrs_str}")

# ============================================================
# SECTION 7: Complete Call Graph (USB → Verify)
# ============================================================
def analyze_call_graph(md, rom):
    """Build the complete call graph from USB entry to img4_verify."""
    print("\n" + "=" * 100)
    print("SECTION 7: CALL GRAPH — USB ENTRY → IMG4 VERIFY")
    print("=" * 100)
    
    # Find ALL functions and their calls
    all_funcs = find_functions_in_range(rom, 0, 0x25000)
    
    KNOWN = {
        0x100010BD0: "memcpy",
        0x100010E00: "memset",
        0x100010D80: "bzero",
        0x10000F1EC: "heap_alloc",
        0x100008978: "panic",
        0x10000F3B0: "pool_alloc",
        0x100005480: "img4_verify_internal",
    }
    
    # Region labels
    def get_region(addr):
        off = addr - ROM_BASE
        if 0x0000 <= off < 0x2000: return "BOOT/EXCEPTION"
        if 0x2000 <= off < 0x3000: return "USB_CTRL"
        if 0x3000 <= off < 0x5000: return "DFU_HANDLER"
        if 0x5000 <= off < 0x6000: return "IMG4_VERIFY"
        if 0x6000 <= off < 0x7000: return "PLATFORM"
        if 0x7000 <= off < 0x8000: return "VALIDATION"
        if 0x8000 <= off < 0x9000: return "PANIC/LOG"
        if 0x9000 <= off < 0xA000: return "SECURITY"
        if 0xA000 <= off < 0xB000: return "IMG4_FW"
        if 0xB000 <= off < 0xD000: return "IO_TRANSPORT"
        if 0xD000 <= off < 0xF000: return "DER_ASN1"
        if 0xF000 <= off < 0x11000: return "HEAP_MEM"
        if 0x11000 <= off < 0x12000: return "SYNC_SOC"
        if 0x12000 <= off < 0x15000: return "CERT_X509"
        if 0x15000 <= off < 0x1C000: return "CRYPTO"
        return "DATA"
    
    # Build call graph
    call_graph = defaultdict(set)  # caller_func -> set of callee addrs
    func_set = set(all_funcs)
    
    for i, func_addr in enumerate(all_funcs):
        func_end = all_funcs[i + 1] if i + 1 < len(all_funcs) else ROM_BASE + 0x25000
        off = func_addr - ROM_BASE
        end_off = func_end - ROM_BASE
        
        instrs = disasm_range(md, rom, off, min(end_off, off + 4096))
        
        for ins in instrs:
            if ins.mnemonic == 'bl' and ins.operands and ins.operands[0].type == ARM64_OP_IMM:
                target = ins.operands[0].imm
                call_graph[func_addr].add(target)
    
    # Find USB entry path: trace from USB_CTRL/DFU to img4_verify
    print("\n### Functions that call into DFU handler region (0x3000-0x5000)")
    for caller in sorted(call_graph.keys()):
        for callee in call_graph[caller]:
            callee_off = callee - ROM_BASE
            caller_off = caller - ROM_BASE
            if 0x3000 <= callee_off < 0x5000 and not (0x3000 <= caller_off < 0x5000):
                callee_name = KNOWN.get(callee, f"dfu_{callee:X}")
                caller_name = KNOWN.get(caller, f"sub_{caller:X}")
                print(f"  [{get_region(caller):12s}] 0x{caller:X} ({caller_name}) → 0x{callee:X} ({callee_name})")
    
    # Find path from DFU to img4_verify
    print("\n### DFU functions → IMG4/Verify path")
    dfu_funcs = [f for f in all_funcs if 0x3000 <= (f - ROM_BASE) < 0x5000]
    
    # BFS from each DFU function to find shortest path to img4_verify region
    from collections import deque
    
    img4_region = set(f for f in all_funcs if 0x5000 <= (f - ROM_BASE) < 0x6000)
    
    for start in dfu_funcs:
        visited = set()
        queue = deque([(start, [start])])
        visited.add(start)
        
        while queue:
            current, path = queue.popleft()
            
            if current in img4_region or current == 0x100005480:
                # Found path!
                path_str = " → ".join(f"0x{p:X}[{get_region(p)}]" for p in path)
                print(f"  PATH: {path_str}")
                break
            
            for callee in call_graph.get(current, []):
                if callee not in visited and callee in func_set:
                    visited.add(callee)
                    queue.append((callee, path + [callee]))
    
    # Cross-region call summary
    print("\n### Cross-Region Call Summary")
    region_calls = defaultdict(lambda: defaultdict(int))
    for caller, callees in call_graph.items():
        src = get_region(caller)
        for callee in callees:
            dst = get_region(callee)
            if src != dst:
                region_calls[src][dst] += 1
    
    for src in sorted(region_calls.keys()):
        for dst in sorted(region_calls[src].keys()):
            count = region_calls[src][dst]
            if count > 0:
                print(f"  {src:15s} → {dst:15s}: {count:3d} calls")
    
    # DFU internal call graph
    print("\n### DFU Internal Call Graph (0x3000-0x5000)")
    for func in sorted(dfu_funcs):
        callees = call_graph.get(func, set())
        internal = [c for c in callees if 0x3000 <= (c - ROM_BASE) < 0x5000]
        external = [c for c in callees if not (0x3000 <= (c - ROM_BASE) < 0x5000)]
        
        ext_names = []
        for c in sorted(external):
            if c in KNOWN:
                ext_names.append(KNOWN[c])
            else:
                ext_names.append(f"{get_region(c)}:0x{c:X}")
        
        int_names = [f"dfu_0x{c:X}" for c in sorted(internal)]
        
        all_calls = int_names + ext_names
        if all_calls:
            print(f"  0x{func:X} calls: {', '.join(all_calls[:15])}")
    
    return call_graph

# ============================================================
# SECTION 8: USB PHY and Power Sequencing
# ============================================================
def analyze_usb_phy(md, rom):
    """Analyze USB PHY initialization and power management."""
    print("\n" + "=" * 100)
    print("SECTION 8: USB PHY & POWER SEQUENCING")
    print("=" * 100)
    
    # The USB PHY (UTMI/PIPE) needs specific initialization:
    # 1. Power on USB block in PMGR
    # 2. Configure PHY registers (UTMI)
    # 3. Wait for PLL lock
    # 4. Enable USB controller
    # 5. Perform soft reset
    # 6. Configure device mode
    
    # Look for PMGR (Power Manager) references
    print("\n### Power Manager (PMGR) References")
    code_end = 0x25000
    instrs = disasm_range(md, rom, 0, code_end)
    
    reg_vals = {}
    pmgr_refs = []
    phy_refs = []
    
    for ins in instrs:
        if ins.mnemonic == 'movz' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            imm = ins.operands[1]
            if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                reg_vals[dst.reg] = imm.imm << shift
        elif ins.mnemonic == 'movk' and len(ins.operands) >= 2:
            dst = ins.operands[0]
            imm = ins.operands[1]
            if dst.type == ARM64_OP_REG and imm.type == ARM64_OP_IMM:
                shift = imm.shift.value if imm.shift.type != ARM64_SFT_INVALID else 0
                if dst.reg in reg_vals:
                    mask = ~(0xFFFF << shift)
                    reg_vals[dst.reg] = (reg_vals[dst.reg] & mask) | (imm.imm << shift)
                    val = reg_vals[dst.reg]
                    
                    # PMGR range (varies by SoC, typically 0x2102xxxxx)
                    if 0x210200000 <= val < 0x210400000:
                        pmgr_refs.append((ins.address, val))
                    # USB PHY range
                    elif 0x2000A0000 <= val < 0x2000C0000:
                        phy_refs.append((ins.address, val))
    
    if pmgr_refs:
        print(f"  Found {len(pmgr_refs)} PMGR references:")
        for addr, val in pmgr_refs:
            print(f"    0x{addr:X}: PMGR register 0x{val:X}")
    else:
        print("  No direct PMGR MMIO references found")
        print("  (Power management may be handled by pre-ROM bootcode or miniPMGR)")
    
    if phy_refs:
        print(f"\n  Found {len(phy_refs)} USB PHY references:")
        for addr, val in phy_refs:
            print(f"    0x{addr:X}: USB PHY register 0x{val:X}")
    else:
        print("\n  No direct USB PHY MMIO references found")
        print("  (PHY may be configured via indirect register access or pre-initialized)")
    
    # Look for delay/wait loops (common in PHY init)
    print("\n### Delay/Wait Loops (PHY stabilization)")
    for ins in instrs:
        if ins.mnemonic == 'wfi':
            print(f"  0x{ins.address:X}: WFI (Wait For Interrupt)")
        elif ins.mnemonic == 'wfe':
            print(f"  0x{ins.address:X}: WFE (Wait For Event)")
        elif ins.mnemonic == 'isb':
            # ISB is used after PHY configuration changes
            # Only report in USB init region
            if 0x100002000 <= ins.address < 0x100003000:
                print(f"  0x{ins.address:X}: ISB (Instruction Sync Barrier) — in USB init region")
        elif ins.mnemonic == 'dsb':
            if 0x100002000 <= ins.address < 0x100005000:
                print(f"  0x{ins.address:X}: DSB (Data Sync Barrier) — in USB/DFU region")

# ============================================================
# SECTION 9: USB Reset & Enumeration
# ============================================================
def analyze_usb_reset_enum(md, rom):
    """Analyze USB reset handling and device enumeration."""
    print("\n" + "=" * 100)
    print("SECTION 9: USB RESET & ENUMERATION SEQUENCE")
    print("=" * 100)
    
    # Search for USB reset-related patterns
    # DWC2 reset: write GRSTCTL.CSftRst = 1, poll until cleared
    # DCTL.SftDiscon: bit 1
    
    print("\n### USB Reset Patterns in Code")
    
    # Look for known USB protocol constants
    # SET_ADDRESS = bRequest 5, GET_DESCRIPTOR = 6, SET_CONFIGURATION = 9
    usb_std_requests = {
        0: "GET_STATUS",
        1: "CLEAR_FEATURE",
        3: "SET_FEATURE",
        5: "SET_ADDRESS",
        6: "GET_DESCRIPTOR",
        7: "SET_DESCRIPTOR",
        8: "GET_CONFIGURATION",
        9: "SET_CONFIGURATION",
        10: "GET_INTERFACE",
        11: "SET_INTERFACE",
    }
    
    # Scan DFU region for these patterns
    instrs = disasm_range(md, rom, 0x2000, 0x5000)
    
    # Track CMP + B.EQ patterns (switch/case on request type)
    prev_cmp = None
    switch_cases = []
    
    for ins in instrs:
        if ins.mnemonic == 'cmp' and len(ins.operands) >= 2:
            if ins.operands[1].type == ARM64_OP_IMM:
                prev_cmp = (ins.address, ins.operands[1].imm)
        elif ins.mnemonic in ('b.eq', 'b.ne') and prev_cmp:
            val = prev_cmp[1]
            target = ins.operands[0].imm if ins.operands and ins.operands[0].type == ARM64_OP_IMM else 0
            
            name = ""
            if val in usb_std_requests:
                name = f"USB Std Request: {usb_std_requests[val]}"
            elif val <= 6:
                from_dfu = {0: "DFU_DETACH", 1: "DFU_DNLOAD", 2: "DFU_UPLOAD", 
                           3: "DFU_GETSTATUS", 4: "DFU_CLRSTATUS", 5: "DFU_GETSTATE", 6: "DFU_ABORT"}
                name = f"DFU Request: {from_dfu.get(val, '?')}"
            
            if name:
                switch_cases.append((prev_cmp[0], val, name, target))
                print(f"  0x{prev_cmp[0]:X}: CMP #{val} → {ins.mnemonic} 0x{target:X}  — {name}")
            elif val in (0x80, 0x00, 0x21, 0xA1, 0x40, 0xC0):
                bm_names = {
                    0x80: "bmRequestType: DEV→HOST, Standard, Device",
                    0x00: "bmRequestType: HOST→DEV, Standard, Device",
                    0x21: "bmRequestType: HOST→DEV, Class, Interface",
                    0xA1: "bmRequestType: DEV→HOST, Class, Interface",
                    0x40: "bmRequestType: HOST→DEV, Vendor, Device",
                    0xC0: "bmRequestType: DEV→HOST, Vendor, Device",
                }
                name = bm_names.get(val, f"bmRequestType: 0x{val:02X}")
                print(f"  0x{prev_cmp[0]:X}: CMP #0x{val:X} → {ins.mnemonic} 0x{target:X}  — {name}")
        elif ins.mnemonic not in ('cmp',):
            if ins.mnemonic not in ('b.eq', 'b.ne', 'b.hi', 'b.lo', 'b.hs', 'b.ls',
                                     'b.gt', 'b.lt', 'b.ge', 'b.le'):
                prev_cmp = None  # Reset if not a conditional branch

# ============================================================
# SECTION 10: Strings and Constants referenced by USB/DFU
# ============================================================
def analyze_strings_and_constants(md, rom):
    """Find all strings and magic constants used by USB/DFU code."""
    print("\n" + "=" * 100)
    print("SECTION 10: STRINGS & CONSTANTS IN USB/DFU PATH")
    print("=" * 100)
    
    # Find ADRP + ADD patterns that reference the data section
    instrs = disasm_range(md, rom, 0x2000, 0x5000)
    
    adrp_vals = {}
    string_refs = []
    
    for ins in instrs:
        if ins.mnemonic == 'adrp' and len(ins.operands) >= 2:
            if ins.operands[1].type == ARM64_OP_IMM:
                adrp_vals[ins.operands[0].reg] = ins.operands[1].imm
        elif ins.mnemonic == 'add' and len(ins.operands) >= 3:
            if (ins.operands[1].type == ARM64_OP_REG and 
                ins.operands[2].type == ARM64_OP_IMM and
                ins.operands[1].reg in adrp_vals):
                full_addr = adrp_vals[ins.operands[1].reg] + ins.operands[2].imm
                rom_off = full_addr - ROM_BASE
                if 0x1BC00 <= rom_off < 0x22000:
                    # Try to read as string
                    try:
                        end = rom.index(0, rom_off, rom_off + 200)
                        s = rom[rom_off:end].decode('ascii', errors='replace')
                        if len(s) > 2 and all(c.isprintable() or c in '\t\n\r' for c in s):
                            string_refs.append((ins.address, full_addr, s))
                    except (ValueError, UnicodeDecodeError):
                        pass
    
    print("\n### Strings referenced from USB/DFU code:")
    for code_addr, str_addr, s in sorted(string_refs, key=lambda x: x[0]):
        print(f"  0x{code_addr:X} → 0x{str_addr:X}: \"{s}\"")
    
    # Find numeric constants
    print("\n### Important numeric constants in USB/DFU:")
    instrs = disasm_range(md, rom, 0x2000, 0x5000)
    
    constants = defaultdict(list)
    for ins in instrs:
        if ins.mnemonic in ('movz', 'mov') and len(ins.operands) >= 2:
            if ins.operands[1].type == ARM64_OP_IMM:
                v = ins.operands[1].imm
                shift = 0
                if ins.mnemonic == 'movz' and ins.operands[1].shift.type != ARM64_SFT_INVALID:
                    shift = ins.operands[1].shift.value
                actual = v << shift if ins.mnemonic == 'movz' else v
                if actual >= 0x40 and actual not in (0xFFFFFFFF, 0xFFFFFFFFFFFFFFFF):
                    constants[actual].append(ins.address)
    
    important_vals = {
        0x40: "64 (USB FS max packet)",
        0x200: "512 (USB HS max packet)",
        0x800: "2048 (DFU buffer?)",
        0x1000: "4096 (page size)",
        0x10000: "64KB",
        0x80000: "512KB (ROM size)",
        0x05AC: "Apple VID",
        0x1227: "DFU PID",
        0x1281: "Recovery PID",
        0x0200: "USB 2.0",
        0x0300: "USB 3.0",
        0x0210: "USB 2.1",
        0xFE01: "DFU class-subclass",
    }
    
    for val in sorted(constants.keys()):
        refs = constants[val]
        label = important_vals.get(val, "")
        if label or len(refs) >= 2 or val > 0x1000:
            locs = ", ".join(f"0x{a:X}" for a in refs[:5])
            print(f"  0x{val:08X} ({val:>10d}): used {len(refs):2d}× at {locs}  {label}")


# ============================================================
# MAIN
# ============================================================
def main():
    print("╔══════════════════════════════════════════════════════════════════════════════════════╗")
    print("║  T8020 B1 SecureROM — COMPLETE USB WORKFLOW ANALYSIS                                ║")
    print("║  From USB Connector → USB PHY → DWC Controller → DFU State Machine → IMG4 Verify   ║")
    print("╚══════════════════════════════════════════════════════════════════════════════════════╝")
    
    rom = load_rom()
    md = get_md()
    
    print(f"\nROM loaded: {len(rom)} bytes")
    print(f"ROM base:  0x{ROM_BASE:X}")
    print(f"Active code: ~{0x1BC00} bytes ({0x1BC00 / 1024:.1f} KB)")
    
    # Run all analysis sections
    mmio_refs, usb_pages = analyze_mmio_registers(md, rom)
    analyze_usb_phy(md, rom)
    analyze_usb_controller_init(md, rom)
    analyze_dfu_handler(md, rom)
    analyze_usb_descriptors(md, rom)
    analyze_interrupt_handling(md, rom)
    analyze_buffer_management(md, rom)
    analyze_call_graph(md, rom)
    analyze_usb_reset_enum(md, rom)
    analyze_strings_and_constants(md, rom)
    
    print("\n" + "=" * 100)
    print("ANALYSIS COMPLETE")
    print("=" * 100)

if __name__ == "__main__":
    main()
