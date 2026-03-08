# Analyse de Sécurité iOS 18.5 — iPhone 11 Pro (A13 Bionic)
## TrustOS Security Research — Scénarios de Jailbreak Probables

**Date**: Mars 2026  
**Cible**: iPhone 11 Pro — SoC A13 Bionic (T8030) — iOS 18.5  
**Objectif**: Identifier les vecteurs d'attaque les plus probables pour un jailbreak

---

## 1. TAXONOMIE DES EXPLOITS iOS HISTORIQUES

### 1.1 Thèmes récurrents dans TOUS les jailbreaks

| Thème | Fréquence | Exemples |
|-------|-----------|----------|
| **Memory Corruption (UAF/Heap Overflow)** | ~70% des exploits kernel | CVE-2023-32434 (triangulation), CVE-2022-32917, oob_timestamp |
| **Type Confusion** | ~25% des exploits kernel | IOKit driver type confusion, XNU mach msg type confusion |
| **Race Condition** | ~20% des exploits kernel | CVE-2021-30883 (IOMobileFrameBuffer), SockPuppet |
| **Logic Bug / Design Flaw** | ~15% | MacDirtyCow (CVE-2022-46689), multicast_bytecopy |
| **Integer Overflow** | ~10% | tfp0 via mach_voucher, IOKit integer truncation |
| **IOKit Driver Bugs** | **~60% de tous les kernel exploits** | IOMobileFrameBuffer, AppleAVE2, AppleCLCD, AGXAccelerator |
| **WebKit/JavaScriptCore** | ~90% des remote chains | JIT bugs, type confusion, OOB read/write |
| **Sandbox Escape** | Requis dans toute chain | sysdiagnose, mediaserverd, bluetoothd |

### 1.2 Le thème DOMINANT : **IOKit + Memory Corruption**

**Constat critique** : ~60% de tous les kernel exploits iOS passent par un driver IOKit.

Pourquoi :
- Les drivers IOKit tournent en **ring 0** (kernel)
- Ils sont accessibles depuis le **sandbox** via `IOServiceOpen()`
- Apple a des centaines de drivers, chacun est une surface d'attaque
- Beaucoup sont écrits en **C++** (memory unsafe)
- Le fuzzing de ces interfaces produit régulièrement des crashes

**Top drivers les plus exploités historiquement** :
1. `IOMobileFrameBuffer` — 5+ CVEs (supprimé dans iOS 16+)
2. `AppleAVE2` (encodeur vidéo) — 4+ CVEs
3. `AGXAccelerator` (GPU) — 3+ CVEs
4. `AppleCLCD` — 2+ CVEs
5. `AppleJPEGDriver` — 2+ CVEs
6. `AppleMultitouchHID` — exploité dans Pegasus
7. `AppleSPU` (signal processor) — moins couvert, potentiel élevé

---

## 2. MITIGATIONS SPÉCIFIQUES À A13 + iOS 18

### 2.1 Protections Hardware (A13 Bionic)

| Protection | Description | Impact |
|------------|-------------|--------|
| **PAC (Pointer Authentication)** | Signatures cryptographiques sur pointeurs de code/data | Empêche ROP/JOP classique. Nécessite un PAC bypass dédié |
| **MTE absent** | A13 n'a PAS Memory Tagging (introduit A17/M3) | Pas de protection hardware sur heap — **avantage attaquant** |
| **PPL (Page Protection Layer)** | EL1 ne peut pas modifier les page tables signées | Empêche le remapping kernel direct. Bypass requis |
| **KTRR (Kernel Text RR)** | Kernel text en read-only hardware | Empêche le patching du kernel texte |
| **AMCC (memory controller lockdown)** | Regions physiques verrouillées | Protège le SecureROM et SEP |
| **SEP (Secure Enclave)** | Processeur isolé pour crypto/biométrie | Pas besoin de compromettre pour jailbreak |
| **CoreTrust** | Validation des signatures de code | Doit être bypassed pour installer apps non-signées |

### 2.2 Protections Software (iOS 18.x)

| Protection | Détail |
|------------|--------|
| **SMAP/SMEP equiv.** | PAN (Privileged Access Never) — kernel ne peut pas lire userland |
| **KASLR** | Randomisation du kernel — nécessite un info leak préalable |
| **Zone allocator hardening** | kalloc zones séparées, sequestering, type isolation |
| **Lockdown Mode** | Réduit la surface d'attaque (désactive JIT, limite USB, etc.) |
| **kASAN en debug** | Apple utilise kASAN en interne — bugs trouvés plus tôt |
| **Rapid Security Response** | Patches entre les versions majeures |
| **ExclaveOS** | iOS 18 introduit ExclaveOS — microkernel séparé pour certains services critiques |

### 2.3 Ce qui est ABSENT sur A13 (= avantages attaquant)

- ❌ **Pas de MTE** → Le heap est classiquement exploitable (UAF, overflow)
- ❌ **PAC v1 seulement** → PAC keys plus faibles qu'A15+ (PAC v2)
- ❌ **Pas de ExclaveOS complet** → Certains services encore dans le kernel XNU
- ❌ **DMA toujours possible via USB** → L'interface USB reste un vecteur

---

## 3. SURFACE D'ATTAQUE iOS 18.5 — VECTEURS CLASSÉS

### 3.1 🔴 PROBABILITÉ HAUTE — WebKit + Kernel Chain

**Scénario** : Exploit Safari → Sandbox escape → Kernel exploit → PPL/PAC bypass → Root + persistence

**Pourquoi c'est le plus probable** :
- WebKit est la surface d'attaque la plus large (tout navigateur iOS utilise WebKit)
- JavaScriptCore JIT est un générateur de bugs infini
- Historiquement, **80% des jailbreaks publics** utilisent un WebKit entry point ou un équivalent userland
- iOS 18 a un nouveau JIT compiler ("Maglev" port?) = nouveau code = nouveaux bugs

**Bugs typiques recherchés** :
```
1. JIT type confusion dans DFG/FTL → OOB read/write
2. RegExp engine bug → heap buffer overflow
3. DOM UAF → arbitrary read primitive
4. WebAssembly compilation bug → code execution dans JIT region
```

**Chain complète estimée** :
```
WebKit RCE → PAC bypass (PACGA gadget ou signing oracle)
           → IOKit driver bug (kernel r/w primitive)
           → PPL bypass (PTE manipulation via physmap)
           → CoreTrust bypass (trust cache injection)
           → Persistence (boot-time hook)
```

**Effort estimé** : 3-6 mois pour une équipe de 2-3 researchers
**Probabilité qu'un tel bug existe sur 18.5** : **~85%** (basé sur le taux historique de WebKit CVEs)

---

### 3.2 🟠 PROBABILITÉ MOYENNE-HAUTE — Kernel Logic Bug (MacDirtyCow-style)

**Scénario** : Bug logique dans XNU sans corruption mémoire → contournement d'une vérification → escalade de privilèges

**Exemples historiques** :
- **MacDirtyCow** (CVE-2022-46689) : Race condition dans `vm_copy()` permettant d'écrire dans des fichiers read-only
- **multicast_bytecopy** : Logic bug dans le networking stack
- **Smith** (CVE-2023-41991 + 41992 + 41993) : Triple logic bug chain

**Pourquoi c'est probable** :
- XNU fait ~5M lignes de code
- Le networking stack (TCP/IP, IGMP, multicast) est complexe et historiquement buggy
- Le VM subsystem (`mach_vm_*`, `vm_map`, Copy-On-Write) contient une classe entière de race conditions
- Les logic bugs sont **plus difficiles à trouver** mais **impossible à mitiger** par hardware

**Zones à auditer** :
```
1. vm_map.c — COW races, shared memory handling
2. bsd/kern/uipc_* — socket IPC, multicast
3. osfmk/ipc/mach_msg.c — mach message handling, port lifecycle
4. bsd/vfs/ — VFS raceconditions, mount handling
5. IOKit/IOUserClient.cpp — external method dispatch
```

**Effort estimé** : 2-4 mois de fuzzing + audit manuel
**Probabilité** : **~60%** qu'un bug exploitable non-patché existe

---

### 3.3 🟠 PROBABILITÉ MOYENNE — IOKit Driver 0day

**Scénario** : Fuzzing d'une interface IOKit userland-accessible → heap corruption → kernel r/w

**Drivers cibles prioritaires sur iOS 18** :
```
┌─────────────────────────┬──────────────┬─────────────────────────────────┐
│ Driver                  │ Risque       │ Raison                          │
├─────────────────────────┼──────────────┼─────────────────────────────────┤
│ AGXAccelerator (GPU)    │ TRÈS ÉLEVÉ   │ Complex, JIT shader compiler    │
│ AppleAVE2 (vidéo H/W)  │ ÉLEVÉ        │ Historiquement buggy            │
│ AppleJPEGDriver         │ ÉLEVÉ        │ Parsing de données non-trusted  │
│ AppleSPUProfileDriver   │ MOYEN        │ Moins audité = moins patché     │
│ AppleARMIODevice        │ MOYEN        │ Interface générique, large      │
│ IOHIDFamily             │ MOYEN        │ Input handling, historique bugs  │
│ AppleConvergedIPCOLYBTControl │ MOYEN  │ Nouveau dans iOS 18, Bluetooth  │
│ IOSurface               │ ÉLEVÉ        │ Shared memory, historique CVEs  │
│ AppleANE (Neural Engine)│ MOYEN        │ Complexe, moins audité          │
└─────────────────────────┴──────────────┴─────────────────────────────────┘
```

**Méthodologie d'attaque** :
1. Énumérer les `IOUserClient` accessibles depuis le sandbox
2. Fuzzer les `externalMethod()` avec des inputs aléatoires
3. Monitorer les kernel panics via `log show --predicate 'eventMessage contains "panic"'`
4. Analyser les crashes pour primitives read/write

**Outils** :
- PassiveFuzz, SockFuzzer, ipc-fuzzer
- Corellium (virtualisation iPhone, ~$300/mois)
- kext disassembly dans Ghidra/IDA

**Effort estimé** : 1-3 mois de fuzzing intensif
**Probabilité** : **~50%** pour un crash exploitable

---

### 3.4 🟡 PROBABILITÉ MOYENNE — USB/Lightning Physical Attack

**Scénario** : Exploit via l'interface USB pendant le boot ou en mode DFU/Recovery

**Pourquoi on l'explore** :
- L'interface USB est accessible **physiquement** (on a l'appareil)
- Le stack USB iOS a eu des bugs historiques (checkm8 était un USB bug!)
- Le mode Recovery utilise un **stripped-down OS** avec moins de protections
- L'iPhone 11 Pro a encore **Lightning** (pas USB-C), le protocole Lightning a des particularités

**Vecteurs possibles** :
```
1. USB descriptor fuzzing en mode Recovery
   → iBoot USB stack != BootROM USB stack
   → iBoot est SIGNÉ mais pas ROM, donc patchable par Apple
   → Mais iBoot a son propre ensemble de bugs USB

2. USB-C/Lightning protocol fuzzing
   → Le chip Tristar/Hydra gère le protocole Lightning
   → Certains modes debug peuvent être activés avec un câble spécial (DCSD)
   → Apple Internal cables activent "serial over lightning"

3. Accessory protocol attacks
   → MFi (Made for iPhone) protocol
   → CarPlay USB handshake
   → Exploit via faux accessoire USB
```

**Avantage clé** : Les bugs USB dans iBoot/RestoreOS sont **persistants** entre reboots si on peut patcher iBoot avant la signature check.

**Matériel requis** :
- Câble DCSD (Debug Cable, ~$50) ou fabrication maison
- Analyseur USB (Beagle 480, ~$500) ou USBPcap logiciel
- Fuzzer USB hardware (Facedancer, ~$100)

**Effort estimé** : 3-6 mois
**Probabilité** : **~25%** (iBoot passe par beaucoup d'audit interne Apple)

---

### 3.5 🟡 PROBABILITÉ MOYENNE-BASSE — Baseband / Wi-Fi Exploit

**Scénario** : Exploit du baseband Intel (iPhone 11 Pro utilise Intel XMM 7660) → escalade vers AP

**Pourquoi** :
- Le baseband est un processeur **séparé** avec son propre firmware
- Historiquement peu audité par la communauté (firmware propriétaire)
- **Intel a vendu sa division modem à Apple** — le code legacy Intel peut avoir des bugs
- Le baseband a accès DMA à la mémoire partagée avec l'AP

**Mais** :
- L'iPhone 11 Pro utilise Intel XMM 7660 (**pas** le Qualcomm Shannon, mieux documenté)
- Les baseband exploits sont **extrêmement complexes** (reverse engineering du firmware)
- Depuis iOS 14, le baseband est davantage isolé par DART (IOMMU Apple)

**Probabilité** : **~15%** (très spécialisé, mais potentiel élevé si trouvé)

---

### 3.6 🟢 PROBABILITÉ BASSE — SEP / Secure Enclave Attack

Le SEP est un processeur ARM séparé avec son propre OS (SEPOS).  
Historiquement, **1 seul exploit public** (CVE-2020-9839, panic-based).  
Surface d'attaque minimale — nécessite déjà un kernel exploit.

**Probabilité** : **~5%** (pas nécessaire pour un jailbreak classique)

---

## 4. SCÉNARIO D'ATTAQUE RECOMMANDÉ — "PLAN DE RECHERCHE"

### Phase 1 : Reconnaissance (2-4 semaines)
```
┌─────────────────────────────────────────────────────────┐
│  1. Extraire le kernelcache iOS 18.5 (ipsw.me)         │
│  2. Déchiffrer avec img4tool + keys connues             │
│  3. Charger dans Ghidra avec iBootLoader plugin         │
│  4. Lister tous les IOUserClient accessibles sandbox    │
│  5. Diff avec iOS 18.4 kernelcache (trouver nouveau    │
│     code = nouvelles vulnérabilités potentielles)       │
│  6. Extraire la liste des kexts + symboles              │
└─────────────────────────────────────────────────────────┘
```

### Phase 2 : Fuzzing IOKit (4-8 semaines)
```
┌─────────────────────────────────────────────────────────┐
│  1. Setup Corellium ou iPhone physique + profil crash   │
│  2. Fuzzer les top 5 IOKit drivers (AGX, AVE2, JPEG,   │
│     IOSurface, ANE)                                     │
│  3. Collecter panics + analyser les crash logs          │
│  4. Identifier les primitives : OOB read, OOB write,   │
│     UAF, double-free                                    │
│  5. Développer le exploit si crash exploitable trouvé   │
└─────────────────────────────────────────────────────────┘
```

### Phase 3 : Kernel Exploit Development (4-8 semaines)
```
┌─────────────────────────────────────────────────────────┐
│  1. Transformer le crash en primitive kernel r/w        │
│  2. Implémenter le KASLR bypass (info leak)             │
│  3. Implémenter le PAC bypass                           │
│     → Méthode probable : PAC signing oracle via         │
│       IOKit kernel object corruption                    │
│     → OU : PACGA forgery via collision                  │
│  4. Implémenter le PPL bypass                           │
│     → Méthode probable : physmap manipulation           │
│     → Écrire via kcall dans zone non-PPL               │
│  5. Root + tfp0 (task_for_pid(0))                       │
│  6. CoreTrust bypass pour injection trust cache         │
└─────────────────────────────────────────────────────────┘
```

### Phase 4 : Post-Exploitation & Persistence (2-4 semaines)
```
┌─────────────────────────────────────────────────────────┐
│  1. Monter le rootfs en r/w                             │
│  2. Installer un bootstrap (Procursus/Elucubratus)      │
│  3. Installer un gestionnaire de packages (Sileo/Zebra) │
│  4. Optionnel : persistence via boot hook               │
│     → Semi-tethered (exploit à chaque boot)             │
│     → OU untethered si bug dans early boot chain        │
│  5. Déployer ios-recon pour hardware mapping            │
│  6. Booter TrustOS via PongoOS-like loader              │
└─────────────────────────────────────────────────────────┘
```

---

## 5. ANALYSE DE PROBABILITÉ GLOBALE

```
╔══════════════════════════════════════════════════════════════╗
║  ESTIMATION : Jailbreak iOS 18.5 iPhone 11 Pro              ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  Probabilité qu'un 0day exploitable existe : ~90%            ║
║  (iOS a historiquement ~50 kernel CVEs/an)                   ║
║                                                              ║
║  Probabilité de le TROUVER en 6 mois : ~40-60%              ║
║  (dépend des resources : Corellium, temps plein, etc.)       ║
║                                                              ║
║  Probabilité de développer une full chain : ~25-35%          ║
║  (PAC + PPL + CoreTrust = 3 bypasses requis en plus)         ║
║                                                              ║
║  Timeline estimée si succès : 4-12 mois                      ║
║                                                              ║
║  Coût estimé :                                               ║
║  - Corellium : $300/mois × 6 = $1,800                       ║
║  - Ou : iPhone 11 Pro physique = déjà possédé ✓             ║
║  - Câble DCSD : ~$50                                         ║
║  - Facedancer USB : ~$100                                    ║
║  - Ghidra : gratuit                                          ║
║  - Temps : 500-2000 heures de recherche                      ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

---

## 6. CE QU'ON PEUT FAIRE MAINTENANT (SANS JAILBREAK)

### 6.1 Analyse statique du kernelcache
```bash
# Télécharger l'IPSW iOS 18.5 pour iPhone 11 Pro
# https://ipsw.me/iPhone12,3

# Extraire le kernelcache
unzip iPhone12,3_18.5_*.ipsw -d ipsw_extract
# Le kernelcache est dans : ipsw_extract/kernelcache.release.iphone12b

# Déchiffrer (si keys disponibles)
img4 -i kernelcache.release.iphone12b -o kernelcache.raw

# Charger dans Ghidra
# Plugin recommandé : https://github.com/0x36/ghidra_kernelcache
```

### 6.2 Diff de kernelcache entre versions
```python
# Utiliser BinDiff ou Diaphora pour comparer iOS 18.4 vs 18.5
# Les fonctions modifiées indiquent les patches de sécurité
# → Les patches récents indiquent des bugs récemment découverts
# → Ces bugs pourraient exister dans la version N-1
```

### 6.3 Fuzzing WebKit en local
```bash
# Compiler WebKit open-source avec ASAN
git clone https://github.com/nicoh-8/WebKit.git
cd WebKit
Tools/Scripts/build-webkit --asan --jsc-only

# Fuzzer avec Fuzzilli (Google)
git clone https://github.com/nicoh-8/fuzzilli.git
# Fuzzer JavaScriptCore pendant des semaines
# Les crashes JIT sont courants
```

### 6.4 Outils TrustOS déjà prêts
```
✅ tools/ios-recon/     — Prêt pour déploiement post-jailbreak
✅ kernel/drivers/apple/ — AIC + UART drivers prêts
✅ TrustProbe scanner    — Prêt pour audit hardware
✅ Network scanner       — Prêt pour recon réseau depuis bare-metal
```

---

## 7. RÉSUMÉ DES THÈMES DE VULNÉRABILITÉ

### Le pattern universel iOS :
```
    ┌──────────────────┐
    │   ENTRY POINT    │  WebKit (remote) ou IOKit (local)
    └────────┬─────────┘
             │
    ┌────────▼─────────┐
    │  MEMORY CORRUPT  │  UAF, heap overflow, type confusion
    └────────┬─────────┘
             │
    ┌────────▼─────────┐
    │   KERNEL R/W     │  via corrupted kernel object
    └────────┬─────────┘
             │
    ┌────────▼─────────┐
    │  BYPASS PAC/PPL  │  Signing oracle, physmap trick
    └────────┬─────────┘
             │
    ┌────────▼─────────┐
    │   ROOT + TFP0    │  Patch credentials, get task port
    └────────┬─────────┘
             │
    ┌────────▼─────────┐
    │   PERSISTENCE    │  Trust cache injection, boot hook
    └────────┴─────────┘
```

### Les 3 thèmes qui reviennent TOUJOURS :

1. **IOKit est le maillon faible du kernel** — C++ drivers avec interfaces userland accessibles
2. **La complexité est l'ennemi de la sécurité** — XNU + IOKit + kexts = millions de lignes
3. **Les logic bugs survivent aux mitigations hardware** — PAC/PPL n'arrêtent pas les bugs logiques

---

## 8. PROCHAINES ÉTAPES CONCRÈTES

| # | Action | Priorité | Temps |
|---|--------|----------|-------|
| 1 | Télécharger IPSW iOS 18.5 iPhone 11 Pro + extraire kernelcache | 🔴 Haute | 1 jour |
| 2 | Setup Ghidra + plugin kernelcache + analyser IOUserClients | 🔴 Haute | 1 semaine |
| 3 | Diff kernelcache 18.4 vs 18.5 (identifier les patches récents) | 🔴 Haute | 2-3 jours |
| 4 | Compiler et fuzzer WebKit/JSC avec Fuzzilli | 🟠 Moyenne | 2 semaines |
| 5 | Lister les IOKit services accessibles depuis sandbox | 🟠 Moyenne | 3 jours |
| 6 | Commander câble DCSD + setup USB analysis | 🟡 Basse | 2 semaines |
| 7 | Étudier les write-ups récents (Project Zero, ZecOps, Kaspersky) | 🔴 Haute | Continu |

---

*Ce document fait partie du projet TrustOS Security Research.*  
*Voir aussi : [TRUSTPROBE_ROADMAP.md](TRUSTPROBE_ROADMAP.md), [tfa_sip_handler_vulnerability_analysis.md](tfa_sip_handler_vulnerability_analysis.md)*
