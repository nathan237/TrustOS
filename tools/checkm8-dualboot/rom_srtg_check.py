#!/usr/bin/env python3
"""Quick serial string verification"""
rom = open('securerom/t8020_B1_securerom.bin', 'rb').read()

# Search for SRTG in ROM
idx = 0
while True:
    pos = rom.find(b'SRTG', idx)
    if pos == -1: break
    ctx = rom[max(0,pos-32):min(len(rom),pos+64)]
    safe = ''.join(chr(b) if 32 <= b < 127 else '.' for b in ctx)
    print(f'SRTG at 0x{pos:05X}: {safe}')
    idx = pos + 4

# Check the format strings around serial area
off = 0x1C279
end = off
while end < len(rom) and rom[end] != 0:
    end += 1
fmt1 = rom[off:end].decode('ascii', errors='replace')
print(f'\nFormat at 0x{off:05X} ({end-off} bytes): "{fmt1}"')

# What strings follow?
scan = end + 1
for _ in range(5):
    while scan < len(rom) and rom[scan] == 0:
        scan += 1
    send = scan
    while send < len(rom) and rom[send] != 0:
        send += 1
    s = rom[scan:send].decode('ascii', errors='replace')
    print(f'  Next at 0x{scan:05X}: "{s}"')
    scan = send + 1

# Compute both variants
serial_no_srtg = "CPID:8020 CPRV:11 CPFM:03 SCEP:01 BDID:04 ECID:001C15D43C20802E IBFL:01"
blen = 2 + len(serial_no_srtg) * 2
print(f'\nSerial WITHOUT SRTG: {len(serial_no_srtg)} chars, bLength={blen}, mod64={blen%64}')

serial_with = serial_no_srtg + " SRTG:[iBoot-3865.0.0.4.7]"
blen2 = 2 + len(serial_with) * 2
print(f'Serial WITH SRTG: {len(serial_with)} chars, bLength={blen2}, mod64={blen2%64}')
