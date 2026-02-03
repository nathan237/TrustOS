# Hardware Test Guide

## TRustOs - Test sur matériel réel

### Prérequis
- Clé USB (4 GB minimum)
- PC x86_64 compatible UEFI/BIOS

### Étapes

#### 1. Préparer l'image
```bash
cd kernel
cargo bootimage --release
```

Image: `target/x86_64-unknown-none/release/bootimage-trustos_kernel.bin`

#### 2. Créer clé bootable (Windows)
```powershell
# Identifier la clé USB
Get-Disk

# Écrire l'image (remplacer X par numéro disk)
$img = "target\x86_64-unknown-none\release\bootimage-trustos_kernel.bin"
dd if=$img of=\\.\PhysicalDriveX bs=4M
```

Ou utiliser Rufus en mode DD.

#### 3. Boot
- Insérer clé USB
- Redémarrer PC
- F12/F2 pour boot menu
- Sélectionner clé USB

### Attendu
```
TRustOs v0.1.0
Initializing...
IDT loaded
Scheduler ready
IPC ready
Security initialized
Syscall interface initialized
Trace ready
GUI drivers initialized
Kernel ready. Starting init...
```

### Troubleshooting
- Écran noir: Vérifier BIOS legacy/UEFI
- Pas de sortie série: Normal (pas d'écran VGA impl)
- Reboot loop: Bootloader incompatible

### Alternative: VirtualBox
VirtualBox 7+ supporte mieux bootloader 0.9:
```bash
VBoxManage convertfromraw bootimage-trustos_kernel.bin trustos.vdi
VBoxManage createvm --name TRustOs --register
VBoxManage storagectl TRustOs --name SATA --add sata
VBoxManage storageattach TRustOs --storagectl SATA --port 0 --type hdd --medium trustos.vdi
VBoxManage startvm TRustOs
```
