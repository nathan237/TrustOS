# 🛠️ TrustOS Hardware Deployment Roadmap

## 📋 PHASE 1 : DRIVERS ESSENTIELS (Base système)

| Catégorie | Driver | Status | Priorité |
|-----------|--------|--------|----------|
| **CPU** | x86_64 CPUID/MSR | ✅ Intégré | - |
| **CPU** | TSC (Time Stamp Counter) | ✅ Intégré | - |
| **CPU** | SIMD (SSE/SSE2) | ✅ Intégré | - |
| **Mémoire** | Paging x86_64 | ✅ Intégré | - |
| **Interrupts** | PIC 8259 | ✅ Intégré | - |
| **Interrupts** | APIC/IOAPIC | ⚠️ Partiel | HAUTE |
| **Timer** | PIT 8254 | ✅ Intégré | - |
| **Timer** | HPET | ⚠️ Détecté | MOYENNE |
| **Timer** | Local APIC Timer | ❌ Manquant | HAUTE |
| **Clavier** | PS/2 Keyboard | ✅ Intégré | - |
| **Souris** | PS/2 Mouse | ✅ Intégré | - |
| **Console** | VGA Text Mode | ✅ Intégré | - |
| **Console** | Framebuffer (UEFI GOP) | ✅ Intégré | - |
| **Serial** | UART 16550 | ✅ Intégré | - |
| **ACPI** | Parser RSDP/XSDT | ✅ Intégré | - |

---

## 📋 PHASE 2 : STOCKAGE (Persistance)

| Driver | Chips/Standards Supportés | Status | Priorité |
|--------|---------------------------|--------|----------|
| **AHCI/SATA** | Intel ICH, AMD | ⚠️ Partiel | CRITIQUE |
| **NVMe** | NVMe 1.4 | ❌ Manquant | CRITIQUE |
| **IDE/PATA** | Legacy ATA | ❌ Manquant | BASSE |
| **USB Mass Storage** | USB 2.0/3.0 | ❌ Manquant | HAUTE |
| **SD/MMC** | SDHCI | ❌ Manquant | MOYENNE |
| **RAID** | Software RAID | ❌ Manquant | BASSE |

### Filesystems nécessaires :
- ✅ FAT32 (lecture)
- ⚠️ FAT32 (écriture partielle)
- ❌ ext4
- ❌ NTFS (lecture)
- ❌ Btrfs

---

## 📋 PHASE 3 : RÉSEAU (Connectivité)

| Driver | Chips Supportés | Status | Priorité |
|--------|-----------------|--------|----------|
| **e1000/e1000e** | Intel 82540-82599 | ✅ Intégré | - |
| **RTL8139** | Realtek | ⚠️ Stub | HAUTE |
| **RTL8169** | Realtek Gigabit | ❌ Manquant | HAUTE |
| **virtio-net** | VirtIO | ⚠️ Stub | MOYENNE |
| **Broadcom BCM** | BCM57xx | ❌ Manquant | MOYENNE |
| **Intel I225/I226** | 2.5G | ❌ Manquant | MOYENNE |
| **WiFi 802.11** | Intel AX200/AX210 | ❌ Manquant | BASSE |
| **Bluetooth** | Intel BT | ❌ Manquant | BASSE |

### Stack réseau :
- ✅ Ethernet (L2)
- ✅ ARP
- ✅ IPv4
- ✅ ICMP (ping)
- ✅ UDP
- ✅ TCP (basic)
- ✅ DHCP Client
- ⚠️ DNS (partiel)
- ❌ IPv6

---

## 📋 PHASE 4 : USB (Périphériques)

| Controller | Standard | Status | Priorité |
|------------|----------|--------|----------|
| **UHCI** | USB 1.0 | ⚠️ Détecté | MOYENNE |
| **OHCI** | USB 1.1 | ❌ Manquant | MOYENNE |
| **EHCI** | USB 2.0 | ⚠️ Détecté | HAUTE |
| **xHCI** | USB 3.0/3.1/3.2 | ❌ Manquant | CRITIQUE |

### Classes USB :
- ❌ HID (Keyboard/Mouse)
- ❌ Mass Storage
- ❌ Hub
- ❌ Audio
- ❌ Video

---

## 📋 PHASE 5 : GRAPHIQUES (GPU)

| Driver | Chips | Status | Priorité |
|--------|-------|--------|----------|
| **VESA/GOP** | Standard UEFI | ✅ Intégré | - |
| **VirtIO GPU** | QEMU/KVM | ❌ Manquant | HAUTE |
| **Intel HD** | Gen 9-12 | ❌ Manquant | MOYENNE |
| **AMD AMDGPU** | GCN/RDNA | ❌ Manquant | BASSE |
| **NVIDIA** | (propriétaire) | ❌ N/A | - |

---

## 📋 PHASE 6 : AUDIO

| Driver | Chips | Status | Priorité |
|--------|-------|--------|----------|
| **PC Speaker** | Beep | ❌ Manquant | BASSE |
| **Intel HDA** | Realtek ALC/Intel | ❌ Manquant | MOYENNE |
| **AC97** | Legacy | ❌ Manquant | BASSE |
| **USB Audio** | Class 1/2 | ❌ Manquant | BASSE |

---

## 📋 PHASE 7 : AVANCÉ (Performance Max)

| Feature | Description | Status | Priorité |
|---------|-------------|--------|----------|
| **SMP** | Multi-core | ⚠️ Détection seule | CRITIQUE |
| **NUMA** | Multi-socket | ❌ Manquant | BASSE |
| **Power Management** | ACPI S-states | ❌ Manquant | MOYENNE |
| **Virtualization** | VT-x/AMD-V passthrough | ⚠️ Hyperviseur | MOYENNE |
| **IOMMU** | VT-d/AMD-Vi | ❌ Manquant | BASSE |
| **TPM 2.0** | Security | ❌ Manquant | MOYENNE |
| **Secure Boot** | UEFI signing | ❌ Manquant | BASSE |

---

## 🎯 ROADMAP PRIORITÉ

### IMMÉDIAT (pour déploiement réel)
```
┌─────────────────────────────────────┐
│ 1. NVMe Driver (90% des PC modernes)│
│ 2. xHCI USB 3.0 (clavier/souris)    │
│ 3. SMP Boot (multi-core)            │
│ 4. APIC Timer (timing précis)       │
│ 5. AHCI complet (SATA)              │
└─────────────────────────────────────┘
```

### COURT TERME (1-3 mois)
```
┌─────────────────────────────────────┐
│ 6. RTL8169 (Realtek Gigabit)        │
│ 7. USB HID (clavier/souris USB)     │
│ 8. USB Mass Storage                 │
│ 9. ext4 filesystem                  │
│ 10. Intel HDA Audio                 │
└─────────────────────────────────────┘
```

### MOYEN TERME (3-6 mois)
```
┌─────────────────────────────────────┐
│ 11. Intel GPU 2D                    │
│ 12. WiFi Intel AX                   │
│ 13. Bluetooth                       │
│ 14. Power Management                │
│ 15. IPv6                            │
└─────────────────────────────────────┘
```

---

## 📊 Hardware Compatibility Matrix

### Tested Platforms
| Platform | CPU | Storage | Network | Status |
|----------|-----|---------|---------|--------|
| QEMU q35 | qemu64 | AHCI | e1000 | ✅ Works |
| VirtualBox | Any | SATA | e1000 | ⚠️ Partial |
| VMware | Any | SATA | e1000 | ❌ Untested |
| Real HW | Intel | NVMe | RTL | ❌ Needs NVMe |
| Real HW | AMD | NVMe | RTL | ❌ Needs NVMe |

### Target Hardware (2024+)
- Intel Core 12th-14th Gen
- AMD Ryzen 5000-7000
- NVMe SSD (Samsung, WD, Crucial)
- Intel/Realtek Ethernet
- Intel WiFi AX200/210

---

*Last updated: February 2026*
