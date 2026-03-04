# TrustOS iOS Recon Tool

Hardware reconnaissance tool for jailbroken iOS devices.
Runs via SSH on a jailbroken iPhone (Dopamine/unc0ver/Taurine) and dumps
all hardware topology needed to build bare-metal TrustOS drivers.

## What it does

1. **Device Tree dump** — Extracts the full IODeviceTree from IOKit (MMIO bases, IRQ numbers, clock domains)
2. **Physical memory map** — Reads `/proc/iomem` equivalent via IOKit memory descriptors
3. **iBoot memory dump** — Scans physical memory for iBoot signatures (reverse engineering)
4. **MMIO logger** — Monitors IOKit registry changes and logs MMIO transactions
5. **USB serial bridge** — Opens a TCP→serial relay for remote kernel debug

## Build (on macOS or cross-compile)

```bash
# Native build on jailbroken device (with Theos SDK)
make ARCH=arm64 TARGET=iphone

# Cross-compile from macOS
make CROSS=1
```

## Usage

```bash
# SSH into jailbroken iPhone
ssh root@<iphone-ip>

# Run full recon
./trustos-recon --all --output /var/root/recon_dump.json

# Individual modules
./trustos-recon --devtree          # Dump device tree
./trustos-recon --memmap           # Physical memory map
./trustos-recon --iboot-scan       # Scan for iBoot in memory
./trustos-recon --mmio-log         # Live MMIO monitor
./trustos-recon --serial-bridge 9999  # TCP serial bridge on port 9999
```

## Output

JSON file with complete hardware map:
```json
{
  "device": "iPhone11,8",
  "soc": "T8020 (A12 Bionic)",
  "device_tree": { ... },
  "memory_map": [ ... ],
  "mmio_regions": [ ... ],
  "interrupt_controller": { ... },
  "uart_bases": [ ... ]
}
```

## Requirements

- Jailbroken iOS 14-16 (Dopamine, unc0ver, or Taurine)
- Root access via SSH
- ~2MB disk space
