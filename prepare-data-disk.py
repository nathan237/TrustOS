#!/usr/bin/env python3
"""
Prepare TrustOS data disk image with embedded audio.

Layout (raw sector image, 512 bytes/sector):
  Sector 0       : Header — magic + file table
  Sector 1..N    : WAV file data (raw bytes, padded to 512)

Header format (512 bytes):
  [0..4]    : Magic "TWAV"
  [4..8]    : Version (u32 LE) = 1
  [8..16]   : WAV data size in bytes (u64 LE)
  [16..20]  : WAV start LBA (u32 LE) = 1
  [20..24]  : WAV sector count (u32 LE)
  [24..28]  : CRC32 of WAV data (u32 LE)
  [28..512] : Reserved (zeros)
"""

import struct
import sys
import os
import zlib

SECTOR_SIZE = 512
DISK_SIZE_MB = 64
DISK_SECTORS = DISK_SIZE_MB * 1024 * 1024 // SECTOR_SIZE

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    wav_path = os.path.join(script_dir, "kernel", "src", "trustdaw", "untitled2.wav")
    out_path = os.path.join(script_dir, "trustos_data.img")

    if not os.path.exists(wav_path):
        print(f"ERROR: WAV file not found: {wav_path}")
        sys.exit(1)

    with open(wav_path, "rb") as f:
        wav_data = f.read()

    wav_size = len(wav_data)
    wav_sectors = (wav_size + SECTOR_SIZE - 1) // SECTOR_SIZE
    total_sectors = 1 + wav_sectors  # header + data

    if total_sectors > DISK_SECTORS:
        print(f"ERROR: WAV ({wav_size} bytes, {wav_sectors} sectors) exceeds disk capacity ({DISK_SECTORS} sectors)")
        sys.exit(1)

    crc = zlib.crc32(wav_data) & 0xFFFFFFFF

    print(f"WAV file : {wav_path}")
    print(f"WAV size : {wav_size:,} bytes ({wav_size / 1024 / 1024:.2f} MB)")
    print(f"Sectors  : {wav_sectors} data + 1 header = {total_sectors} total")
    print(f"CRC32    : {crc:#010x}")
    print(f"Disk     : {DISK_SIZE_MB} MB ({DISK_SECTORS} sectors)")

    # Build header sector
    header = bytearray(SECTOR_SIZE)
    header[0:4] = b"TWAV"
    struct.pack_into("<I", header, 4, 1)           # version
    struct.pack_into("<Q", header, 8, wav_size)     # wav data size
    struct.pack_into("<I", header, 16, 1)           # wav start LBA
    struct.pack_into("<I", header, 20, wav_sectors)  # wav sector count
    struct.pack_into("<I", header, 24, crc)          # CRC32

    # Pad WAV data to sector boundary
    padded_wav = wav_data + b"\x00" * (wav_sectors * SECTOR_SIZE - wav_size)

    # Write raw disk image (pad to full disk size)
    with open(out_path, "wb") as f:
        f.write(bytes(header))
        f.write(padded_wav)
        # Pad remaining space to full disk size
        remaining = DISK_SECTORS - total_sectors
        if remaining > 0:
            # Write in 1MB chunks to avoid large memory allocation
            chunk_sectors = 2048  # 1 MB
            while remaining > 0:
                n = min(remaining, chunk_sectors)
                f.write(b"\x00" * (n * SECTOR_SIZE))
                remaining -= n

    print(f"Output   : {out_path} ({os.path.getsize(out_path) / 1024 / 1024:.1f} MB)")
    print("Done!")


if __name__ == "__main__":
    main()
