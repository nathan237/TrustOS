# TrustOS v0.10.2 Release Notes

**Release Date:** 2026-03-14

## 🎉 What's New

### ThinkPad Embedded Controller Driver
- **EC communication** via ports 0x62/0x66 with IBF/OBF handshake protocol and timeout
- **8 temperature sensors** — CPU, miniPCI, HDD, GPU, Battery, and 3 auxiliary
- **Fan control** — Manual speed levels 0-7, automatic (EC-controlled), full speed (disengaged), off
- **Fan RPM readout** — Real-time fan speed from EC registers 0x84/0x85
- **Auto-probe** — Detects ThinkPad EC by reading CPU temp register (0x78)

### CPU Frequency & Voltage Control (Intel SpeedStep)
- **EIST detection** — Reads MSR 0x1A0 bit 16 to check if Enhanced SpeedStep is enabled
- **P-state read** — Current frequency (FID) and voltage (VID) from MSR_IA32_PERF_STATUS (0x198)
- **P-state set** — Write FID/VID pairs to MSR_IA32_PERF_CTL (0x199) for frequency/voltage scaling
- **CPU DTS thermal** — Digital Thermal Sensor readout from MSR 0x19C (degrees below TjMax)
- **Predefined T61 profiles** — 2.0 GHz, 1.6 GHz, 1.2 GHz, 800 MHz for Core 2 Duo T7x00/T8x00

### HDA Audio Fixes (3 critical bugs)
- **Speaker path connection selects** — Only HP Out path had `conn_sel` configured. Speaker path NID 18→10→4 never received audio signal. Now all output paths fully wired.
- **GPIO1 polarity** — T61 amplifier uses direct polarity (GPIO1=HIGH=on). Previous code used inverted polarity from HP laptop fixup, silencing the speaker.
- **Amp Param Override** — Per HDA spec 7.3.4.7, when override bit is clear, use AFG amp caps. AD1984 widgets returned non-zero caps with numsteps=0, causing minimum gain (silence).

### Shell Improvements
- **Scrollback backspace tracking** — `push_char` decrements len on backspace
- **Raw pixel suggestion rendering** — Suggestions bypass Writer to prevent buffer corruption
- **Tab autocomplete** — Direct pixel clearing instead of backspace+space through Writer
- **Auto-snap** — `restore_live_view` draws current_line without re-echo duplication

### New Shell Commands
| Command | Description |
|---------|-------------|
| `fan` | Fan status, speed control (auto/max/off/0-7) |
| `fan auto` | Set EC automatic fan control |
| `fan max` | Full speed (disengaged mode) |
| `temp` / `sensors` | Display all temperature sensors with color-coded output |
| `cpufreq` | Show CPU frequency, voltage, EIST status |
| `cpufreq set <fid> <vid>` | Set CPU P-state manually |
| `cpufreq max` | Maximum performance profile |
| `cpufreq min` | Powersave profile (lowest frequency) |

---

## 📦 Contents

This ZIP contains the bootable TrustOS image ready for QEMU:
- `boot/trustos_kernel` - The kernel binary
- `limine.conf` - Bootloader configuration
- `EFI/BOOT/BOOTX64.EFI` - UEFI bootloader

## 🚀 Quick Start

```bash
# Boot it right now in QEMU:
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio
```

## 📋 Test ThinkPad Hardware Control

```
temp              # Show all temperature sensors
fan               # Show fan status and RPM
fan auto          # Set fan to automatic
fan 5             # Set fan to level 5
cpufreq           # Show CPU frequency and voltage
cpufreq min       # Set powersave mode
```

## 📋 Available Commands

Type `help` in the shell to see all 200+ commands including:
- Hardware: `fan`, `temp`, `sensors`, `cpufreq`, `speedstep`
- File system: `ls`, `cd`, `cat`, `mkdir`, `rm`, `cp`, `mv`
- Network: `ping`, `curl`, `wget`, `ifconfig`
- Execution: `exec test`, `exec hello`
- System: `ps`, `top`, `free`, `uname`
- GUI: `desktop` (launches graphical desktop)
- AI: `jarvis` (on-device AI assistant)

---

*TrustOS — A secure, experimental kernel written in Rust*
