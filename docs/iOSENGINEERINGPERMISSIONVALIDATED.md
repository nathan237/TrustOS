# iOS/macOS DFU Engineering Workflow
## Apple Silicon DFU → USB Passthrough → WSL Linux Analysis

**Scope**: Firmware analysis on owned hardware via DFU mode  
**Target**: Apple Silicon Mac (A3240) — proprietary firmware verification  
**Goal**: Empirically verify Apple iBoot/SEP is not TF-A based; map USB protocol surface

---

## Prerequisites

### Windows side
```powershell
# 1. usbipd-win (USB passthrough to WSL)
winget install usbipd

# 2. WSL2 with Ubuntu (if not installed)
wsl --install -d Ubuntu
wsl --set-default-version 2
```

### WSL Linux side
```bash
sudo apt update
sudo apt install -y \
    libimobiledevice-utils \
    idevicerestore \
    usbutils \
    tcpdump \
    wireshark-common \
    build-essential \
    git \
    python3-pip

# irecovery (from source for latest Apple Silicon support)
sudo apt install -y libimobiledevice-dev libusbmuxd-dev
git clone https://github.com/libimobiledevice/libirecovery
cd libirecovery && ./autogen.sh && make && sudo make install
```

---

## Phase 1 — Mettre le Mac en DFU Mode

### Apple Silicon (M1/M2/M3)
1. Mac éteint
2. Brancher USB-C vers le PC Windows
3. Maintenir le bouton power **10 secondes**
4. Relâcher — le Mac reste éteint, aucun affichage

### Vérification sur Windows
```
Device Manager → Universal Serial Bus devices
→ "Apple Mobile Device (DFU Mode)"   ✅
```
Si tu vois `Apple Mobile Device (Recovery Mode)` → c'est recovery, pas DFU. Recommencer.

---

## Phase 2 — USB Passthrough vers WSL

```powershell
# Lister tous les devices USB
usbipd list

# Identifier le Mac DFU:
# BUS-ID  VID:PID    DEVICE
# 2-1     05ac:1227  Apple Inc. Apple Mobile Device (DFU Mode)

# Bind (une seule fois)
usbipd bind --busid 2-1

# Attacher à WSL
usbipd attach --wsl --busid 2-1
```

### Vérification dans WSL
```bash
lsusb
# Bus 001 Device 002: ID 05ac:1227 Apple, Inc. Mobile Device (DFU Mode)

# VID:PID Apple DFU = 05ac:1227
# VID:PID Apple Recovery = 05ac:1281
```

---

## Phase 3 — Analyse iBoot via irecovery

### Shell iBoot interactif
```bash
irecovery -s
```

### Commandes iBoot à exécuter (fingerprinting firmware)
```
# Dans le shell irecovery:

getenv build-version        # Version iBoot — format Apple, pas TF-A
getenv chip-id              # ID du chip Apple Silicon
getenv board-id             # Board identifier
getenv model                # Modèle (ex: J413AP)
getenv debug-enabled        # Debug mode actif?
getenv boot-args            # Arguments de boot actuels
memory-map                  # Carte mémoire iBoot — révèle les régions sécurisées
devicetree                  # Device tree iBoot
```

### Comparaison TF-A vs Apple iBoot
| Indicateur | TF-A | Apple iBoot |
|-----------|------|-------------|
| Build string | `"BL31 vX.Y"` | `"iBoot-XXXX.Y.Z"` |
| PSCI calls | `0x84000001` → version | No response |
| SiP range | `0xC2000000+` | Apple custom range |
| Memory map format | TF-A regions | Apple SRAM/DRAM/SecureROM |
| DTB format | Standard FDT | Apple ADT (Apple Device Tree) |

---

## Phase 4 — Capture USB Traffic Brut

```bash
# Charger usbmon
sudo modprobe usbmon

# Trouver le bus du device Apple
lsusb -t
# Identifier le bus number (ex: Bus 001)

# Capturer tout le trafic USB sur ce bus
sudo tcpdump -i usbmon1 -w ~/dfu_capture.pcap

# Dans un autre terminal: interagir avec irecovery
irecovery -s

# Arrêter tcpdump (Ctrl+C) puis analyser
```

### Analyser avec tshark
```bash
# Voir les transfers USB
tshark -r ~/dfu_capture.pcap -Y "usb" -T fields \
    -e usb.transfer_type \
    -e usb.endpoint_address \
    -e usb.data_len \
    | head -50

# Extraire payloads
tshark -r ~/dfu_capture.pcap -Y "usb.capdata" -T fields -e usb.capdata
```

---

## Phase 5 — SMC Enumeration (quand TrustOS tourne)

Une fois TrustOS bootable sur le device, ajouter à `hwscan`:

```rust
// À intégrer dans kernel/src/hwscan/trustzone.rs
// Probe les SMC function IDs standard TF-A
const TFA_PSCI_IDS: &[(u64, &str)] = &[
    (0x84000001, "PSCI_VERSION"),
    (0x84000008, "PSCI_SYSTEM_OFF"),
    (0x84000009, "PSCI_SYSTEM_RESET"),
    (0xC2000000, "SiP Service (Altera-style)"),
    (0x8400_00FF, "PSCI_FEATURES"),
];
// Si Apple répond → firmware basé TF-A
// Si SMC_UNK (0xFFFFFFFF) ou no response → propriétaire confirmé
```

---

## Phase 6 — Analyse binaire IPSW (offline, sans device)

```bash
# Télécharger IPSW pour Mac A3240
# (URL depuis ipsw.me pour le modèle exact)

# Extraire iBoot
pip3 install pyimg4
pyimg4 img4 extract -i iBoot.img4 -p iBoot.bin

# Strings search — absent si propriétaire
strings iBoot.bin | grep -E "Trusted Firmware|BL31|PSCI|TF-A|OP-TEE"

# Si 0 résultats → propriétaire confirmé
# Chercher strings Apple
strings iBoot.bin | grep -E "iBoot|Apple|SecureROM|SEPOS"
```

---

## Résultats Attendus

```
HYPOTHESIS: Apple firmware is NOT TF-A based

EVIDENCE TO COLLECT:
[ ] iBoot build string format = "iBoot-XXXX" not "BL31 vX.Y"
[ ] PSCI SMC calls return SMC_UNK
[ ] Memory map regions match Apple SecureROM layout (not TF-A BL31)
[ ] DTB format = Apple ADT not standard FDT
[ ] strings iBoot.bin | grep TF-A = 0 results
[ ] USB DFU protocol uses Apple-specific vendor commands

STATUS: [ ] Pending hardware test
```

---

## Notes de Sécurité

- Tout le testing se fait sur hardware personnel (Mac A3240 owned)
- DFU mode ne bypass pas l'Activation Lock — il permet l'analyse firmware uniquement
- Aucune modification du firmware Apple — read-only analysis
- Compatible avec les guidelines Apple Security Research Device Program
