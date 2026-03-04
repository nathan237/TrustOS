#!/usr/bin/env python3
"""Decode IMG4 manifest tags from the SecureROM handler."""

tags_raw = {
    0x45434944: "ECID",
    0x454B4558: "EKEX?",
    0x454B4559: "EKEY",
    0x45504F52: "EPRO",
    0x45534543: "ESEC",
    0x44475354: "DGST",
    0x44504F52: "DPRO",
    0x4348494F: "CHIO?",
    0x43484950: "CHIP",
    0x43534542: "CSEB?",
    0x43534543: "CSEC",
    0x4350524F: "CPRO",
    0x53444F4D: "SDOM",
    0x424F5243: "BORC?",
    0x424F5244: "BORD",
    0x414D4E4D: "AMNM",
    0x424E4348: "BNCH",
}

print("== IMG4 Manifest Property Tags in T8020 B1 SecureROM ==\n")
for tag_val in sorted(tags_raw.keys()):
    name = tags_raw[tag_val]
    ascii_tag = ''.join(chr((tag_val >> (24-i*8)) & 0xFF) for i in range(4))
    hi = (tag_val >> 16) & 0xFFFF
    lo = tag_val & 0xFFFF
    print(f'  0x{tag_val:08X} = "{ascii_tag}"  (movz #0x{hi:04X}, LSL#16 + movk #0x{lo:04X})  -> {name}')

print("\n== Workflow Summary ==")
print("""
The function at 0x100004CB8 is the IMG4 MANIFEST PROPERTY VERIFIER.

It is called for each property in the IMG4 manifest during secure boot.
Arguments:
  x0 = property tag (4CC code like 'ECID', 'CHIP', etc.)
  x1 = manifest context
  x2 = verification mode (0=appIDLE/install, 1=appDETACH/personalized)
  x3 = property data pointer chain

Flow:
  1. Read DFU state from SRAM[0x19C00BC10]
  2. If state == 1 (appDETACH), dispatch to cert callback based on sub-type:
     - subtype 1 -> cert_2AD8() via callback at SRAM[0x19C00BC30]
     - subtype 2 -> cert_2A6C() via callback at SRAM[0x19C00BC38]
     - subtype 4 -> cert_2B44() via callback at SRAM[0x19C00BC40]
  3. Build error code 0x40040011
  4. Load property chain from x3
  5. Call data_C094() (compute something?)
  6. Store result to SRAM[0x19C0113F8]
  7. Branch based on verification mode (w21):
     - mode 0: Check tags: ECID, CHIO, CHIP, CSEB, CSEC, SDOM, CPRO, BORD, AMNM, BNCH
     - mode 1: Check tags: EKEY, ESEC, EPRO, DGST, DPRO, EKEX
  8. For each tag, call img4_fn_5E50() or img4_fn_5F04()
  9. Return success/error

This is the CRITICAL attestation function that ensures the firmware
is signed for THIS specific device (ECID match) with the correct
crypto params (BNCH anti-replay) and hasn't been downgraded (CSEC).
""")
