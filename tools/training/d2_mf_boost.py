"""
Diablo 2 Magic Find Memory Editor — Proof of Concept
=====================================================
Single-player ONLY. Educational reverse engineering exercise.
Targets: Diablo 2 Classic 1.13c (most common modding version)

Known D2 1.13c memory layout (from d2mods.info / PhrozenKeep):
  - D2Client.dll + 0x11BBFC → Player Unit pointer
  - Unit + 0x5C → pStatList
  - StatList + 0x24 → pMyStats  
  - StatList + 0x28 → wStatCount
  - pMyStats → array of {wStatID(2), wSubIndex(2), dwValue(4)}
  - Stat ID 80 (0x50) = Magic Find %
"""

import ctypes
import ctypes.wintypes as wt
import struct
import sys
import time

# Windows API constants
PROCESS_ALL_ACCESS = 0x1F0FFF
TH32CS_SNAPPROCESS = 0x2
TH32CS_SNAPMODULE = 0x8
TH32CS_SNAPMODULE32 = 0x10
MAX_PATH = 260

# Target config
TARGET_MF = 4200
MF_STAT_ID = 80  # 0x50
DELAY_SECONDS = 120  # 2 minutes wait

# D2 1.13c offsets
PLAYER_UNIT_OFFSET = 0x11BBFC  # D2Client.dll + this = &PlayerUnit
UNIT_STATLIST_OFFSET = 0x5C
STATLIST_MYSTATS_OFFSET = 0x24
STATLIST_STATCOUNT_OFFSET = 0x28
STAT_ENTRY_SIZE = 8  # 2 + 2 + 4 bytes per stat


class PROCESSENTRY32(ctypes.Structure):
    _fields_ = [
        ("dwSize", wt.DWORD),
        ("cntUsage", wt.DWORD),
        ("th32ProcessID", wt.DWORD),
        ("th32DefaultHeapID", ctypes.POINTER(ctypes.c_ulong)),
        ("th32ModuleID", wt.DWORD),
        ("cntThreads", wt.DWORD),
        ("th32ParentProcessID", wt.DWORD),
        ("pcPriClassBase", ctypes.c_long),
        ("dwFlags", wt.DWORD),
        ("szExeFile", ctypes.c_char * MAX_PATH),
    ]


class MODULEENTRY32(ctypes.Structure):
    _fields_ = [
        ("dwSize", wt.DWORD),
        ("th32ModuleID", wt.DWORD),
        ("th32ProcessID", wt.DWORD),
        ("GlblcntUsage", wt.DWORD),
        ("ProccntUsage", wt.DWORD),
        ("modBaseAddr", ctypes.POINTER(ctypes.c_byte)),
        ("modBaseSize", wt.DWORD),
        ("hModule", wt.HMODULE),
        ("szModule", ctypes.c_char * (MAX_PATH + 1)),
        ("szExePath", ctypes.c_char * MAX_PATH),
    ]


kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)


def find_process(name: str) -> int | None:
    """Find a process by executable name, return PID or None."""
    snap = kernel32.CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
    if snap == -1:
        return None
    entry = PROCESSENTRY32()
    entry.dwSize = ctypes.sizeof(PROCESSENTRY32)
    try:
        if not kernel32.Process32First(snap, ctypes.byref(entry)):
            return None
        while True:
            exe = entry.szExeFile.decode("utf-8", errors="ignore").lower()
            if name.lower() in exe:
                return entry.th32ProcessID
            if not kernel32.Process32Next(snap, ctypes.byref(entry)):
                return None
    finally:
        kernel32.CloseHandle(snap)


def find_module_base(pid: int, module_name: str) -> int | None:
    """Find base address of a DLL in a process."""
    snap = kernel32.CreateToolhelp32Snapshot(
        TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid
    )
    if snap == -1:
        return None
    entry = MODULEENTRY32()
    entry.dwSize = ctypes.sizeof(MODULEENTRY32)
    try:
        if not kernel32.Module32First(snap, ctypes.byref(entry)):
            return None
        while True:
            mod = entry.szModule.decode("utf-8", errors="ignore").lower()
            if module_name.lower() in mod:
                return ctypes.addressof(entry.modBaseAddr.contents)
            if not kernel32.Module32Next(snap, ctypes.byref(entry)):
                return None
    finally:
        kernel32.CloseHandle(snap)


def read_u32(handle, addr: int) -> int | None:
    """Read a 4-byte unsigned int from process memory."""
    buf = ctypes.c_uint32()
    n_read = ctypes.c_size_t()
    ok = kernel32.ReadProcessMemory(
        handle, ctypes.c_void_p(addr), ctypes.byref(buf), 4, ctypes.byref(n_read)
    )
    if ok and n_read.value == 4:
        return buf.value
    return None


def read_u16(handle, addr: int) -> int | None:
    """Read a 2-byte unsigned int from process memory."""
    buf = ctypes.c_uint16()
    n_read = ctypes.c_size_t()
    ok = kernel32.ReadProcessMemory(
        handle, ctypes.c_void_p(addr), ctypes.byref(buf), 2, ctypes.byref(n_read)
    )
    if ok and n_read.value == 2:
        return buf.value
    return None


def write_i32(handle, addr: int, value: int) -> bool:
    """Write a 4-byte signed int to process memory."""
    buf = ctypes.c_int32(value)
    n_written = ctypes.c_size_t()
    ok = kernel32.WriteProcessMemory(
        handle, ctypes.c_void_p(addr), ctypes.byref(buf), 4, ctypes.byref(n_written)
    )
    return bool(ok and n_written.value == 4)


def read_bytes(handle, addr: int, size: int) -> bytes | None:
    """Read raw bytes from process memory."""
    buf = (ctypes.c_byte * size)()
    n_read = ctypes.c_size_t()
    ok = kernel32.ReadProcessMemory(
        handle, ctypes.c_void_p(addr), buf, size, ctypes.byref(n_read)
    )
    if ok and n_read.value == size:
        return bytes(buf)
    return None


def find_mf_in_stats(handle, stats_ptr: int, count: int) -> int | None:
    """
    Walk the stat array looking for MF (stat ID 80).
    Each entry: [wStatID:u16][wSubIndex:u16][dwValue:i32] = 8 bytes
    Returns the address of dwValue if found.
    """
    # Read entire stat array at once for efficiency
    total_size = count * STAT_ENTRY_SIZE
    if total_size > 0x10000:  # sanity check
        print(f"  [!] Stat count suspiciously large: {count}")
        return None

    raw = read_bytes(handle, stats_ptr, total_size)
    if raw is None:
        print("  [!] Failed to read stat array")
        return None

    for i in range(count):
        offset = i * STAT_ENTRY_SIZE
        stat_id = struct.unpack_from("<H", raw, offset)[0]
        sub_idx = struct.unpack_from("<H", raw, offset + 2)[0]
        value = struct.unpack_from("<i", raw, offset + 4)[0]

        # Show interesting stats for debug
        if stat_id == MF_STAT_ID:
            print(f"  [*] Found MF stat at index {i}: ID={stat_id}, value={value}%")
            return stats_ptr + offset + 4  # address of dwValue

    return None


def scan_for_mf_pattern(handle, base_addr: int, region_size: int = 0x200000) -> int | None:
    """
    Fallback: scan a memory region for the MF stat pattern.
    Look for stat ID 80 (0x50, 0x00) followed by subindex 0 and a reasonable MF value.
    """
    print("  [~] Scanning memory for MF stat pattern (fallback)...")
    chunk_size = 0x10000
    for offset in range(0, region_size, chunk_size):
        raw = read_bytes(handle, base_addr + offset, chunk_size)
        if raw is None:
            continue
        # Search for pattern: 50 00 00 00 XX XX XX XX where XX is a sane MF value
        for i in range(0, len(raw) - 8, 4):
            stat_id = struct.unpack_from("<H", raw, i)[0]
            sub_idx = struct.unpack_from("<H", raw, i + 2)[0]
            value = struct.unpack_from("<i", raw, i + 4)[0]
            if stat_id == MF_STAT_ID and sub_idx == 0 and 0 <= value <= 2000:
                addr = base_addr + offset + i + 4
                print(f"  [*] Potential MF found at 0x{addr:08X}, current value={value}%")
                return addr
    return None


def main():
    print("=" * 60)
    print("  Diablo 2 MF Boost — PoC (Single-Player Only)")
    print(f"  Target MF: {TARGET_MF}%")
    print(f"  Delay: {DELAY_SECONDS}s after game detected")
    print("=" * 60)

    # Step 1: Find Game.exe
    print("\n[1] Searching for Diablo II process...")
    pid = find_process("game.exe")
    if pid is None:
        pid = find_process("d2r.exe")
        if pid is None:
            print("  [!] Diablo II not running (looking for Game.exe or D2R.exe)")
            print("  [!] Start the game first, then re-run this script.")
            sys.exit(1)
        print(f"  [!] Found D2R (pid={pid}) — offsets may differ, attempting anyway")
    else:
        print(f"  [+] Found Game.exe (PID: {pid})")

    # Step 2: Wait
    print(f"\n[2] Waiting {DELAY_SECONDS}s for game to load...")
    for remaining in range(DELAY_SECONDS, 0, -10):
        print(f"     {remaining}s remaining...")
        time.sleep(min(10, remaining))
    print("  [+] Wait complete")

    # Step 3: Open process
    print("\n[3] Opening process...")
    handle = kernel32.OpenProcess(PROCESS_ALL_ACCESS, False, pid)
    if not handle:
        err = ctypes.get_last_error()
        print(f"  [!] OpenProcess failed (error {err})")
        print("  [!] Try running as Administrator")
        sys.exit(1)
    print(f"  [+] Process handle: 0x{handle:X}")

    try:
        # Step 4: Find D2Client.dll base
        print("\n[4] Finding D2Client.dll base address...")
        d2client_base = find_module_base(pid, "d2client.dll")

        if d2client_base:
            print(f"  [+] D2Client.dll base: 0x{d2client_base:08X}")

            # Step 5: Read player unit pointer
            print("\n[5] Reading player unit pointer...")
            player_ptr_addr = d2client_base + PLAYER_UNIT_OFFSET
            player_ptr = read_u32(handle, player_ptr_addr)

            if player_ptr and player_ptr > 0x10000:
                print(f"  [+] Player unit at: 0x{player_ptr:08X}")

                # Step 6: Read stat list
                print("\n[6] Reading stat list...")
                statlist_ptr = read_u32(handle, player_ptr + UNIT_STATLIST_OFFSET)

                if statlist_ptr and statlist_ptr > 0x10000:
                    print(f"  [+] StatList at: 0x{statlist_ptr:08X}")

                    mystats_ptr = read_u32(handle, statlist_ptr + STATLIST_MYSTATS_OFFSET)
                    stat_count = read_u32(handle, statlist_ptr + STATLIST_STATCOUNT_OFFSET)

                    if mystats_ptr and stat_count and stat_count < 512:
                        print(f"  [+] MyStats array at: 0x{mystats_ptr:08X} ({stat_count} stats)")

                        # Step 7: Find and modify MF
                        print("\n[7] Searching for MF stat (ID=80)...")
                        mf_addr = find_mf_in_stats(handle, mystats_ptr, stat_count)

                        if mf_addr:
                            print(f"\n[8] Writing MF = {TARGET_MF}%...")
                            if write_i32(handle, mf_addr, TARGET_MF):
                                # Verify
                                verify = read_u32(handle, mf_addr)
                                print(f"  [+] SUCCESS! MF set to {verify}%")
                                print(f"  [+] Open character sheet (C) to confirm")
                            else:
                                print("  [!] Write failed")
                        else:
                            print("  [!] MF stat not found in stat list, trying scan...")
                            mf_addr = scan_for_mf_pattern(handle, player_ptr - 0x1000)
                            if mf_addr:
                                old_val = read_u32(handle, mf_addr)
                                print(f"  [*] Current MF: {old_val}%")
                                print(f"  [8] Writing MF = {TARGET_MF}%...")
                                if write_i32(handle, mf_addr, TARGET_MF):
                                    verify = read_u32(handle, mf_addr)
                                    print(f"  [+] SUCCESS! MF set to {verify}%")
                                else:
                                    print("  [!] Write failed")
                            else:
                                print("  [!] Could not locate MF in memory")
                    else:
                        print(f"  [!] Bad stat data (ptr=0x{mystats_ptr or 0:08X}, count={stat_count})")
                else:
                    print(f"  [!] Bad statlist pointer: 0x{statlist_ptr or 0:08X}")
            else:
                print(f"  [!] No player unit found (ptr=0x{player_ptr or 0:08X})")
                print("  [!] Make sure you're in-game (not in menu)")
        else:
            print("  [!] D2Client.dll not found — might be D2R or different version")
            print("  [!] Attempting direct memory scan on Game.exe...")

            game_base = find_module_base(pid, "game.exe")
            if game_base:
                print(f"  [+] Game.exe base: 0x{game_base:08X}")
                mf_addr = scan_for_mf_pattern(handle, game_base, 0x800000)
                if mf_addr:
                    print(f"\n[8] Writing MF = {TARGET_MF}%...")
                    if write_i32(handle, mf_addr, TARGET_MF):
                        verify = read_u32(handle, mf_addr)
                        print(f"  [+] SUCCESS! MF set to {verify}%")
                    else:
                        print("  [!] Write failed")
                else:
                    print("  [!] MF pattern not found in scan")
            else:
                print("  [!] Could not find any D2 module base")

    finally:
        kernel32.CloseHandle(handle)

    print("\n" + "=" * 60)
    print("  Done. This only affects your local single-player game.")
    print("  MF resets on game restart.")
    print("=" * 60)


if __name__ == "__main__":
    main()
