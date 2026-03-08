# A12 T8020 B1 SecureROM — Recherche Complète & Synthèse

## Date: Session 25 - Recherche Internet Exhaustive

---

## 1. LA DÉCOUVERTE CRITIQUE : Le UAF checkm8 EXISTE sur A12

Source: [theapplewiki.com/wiki/Checkm8_Exploit](https://theapplewiki.com/wiki/Checkm8_Exploit)

> "The main use-after-free is actually unpatched in A12, A12X/A12Z or A13, 
> but cannot be exploited without a memory leak, of which the one used in 
> checkm8 was made unreachable in A12 and above."

**C'est la pièce manquante dans notre analyse.** Le bug principal (UAF dans le DFU USB stack) 
est PRÉSENT et DÉCLENCHABLE sur notre T8020 B1. Ce n'est PAS le bug qui a été corrigé — 
c'est le **mécanisme d'exploitation** (le memory leak ZLP) qui a été rendu inaccessible.

### Comment le ZLP leak fonctionne (A7-A11) :
1. Stall le endpoint device-to-host
2. Envoyer beaucoup de requêtes USB → allocations `io_request` sur le heap
3. Certaines requêtes satisfont les conditions pour envoyer un Zero Length Packet (ZLP)
4. Déclencher un USB reset → les ZLP sont alloués mais jamais envoyés → **leaked**
5. Les objets leakés créent un "trou" dans le heap de taille 0x800
6. Au re-entry de DFU, le nouveau IO buffer est alloué dans ce trou (pas sur l'ancien)
7. L'ancien buffer freed est maintenant accessible via le UAF

### Pourquoi ça ne marche pas sur A12+ :
Source: [alfiecg.uk - Comprehensive checkm8 writeup](https://alfiecg.uk/2023/07/21/A-comprehensive-write-up-of-the-checkm8-BootROM-exploit)

> "In A12+ SoCs, when a USB reset occurs, the abort that is subsequently 
> triggered also aborts EP0_IN for each setup packet - resulting in abort() 
> being called twice. The first abort will queue an additional zero-length 
> packet, but the second will successfully reap it and de-allocate it."

**Le double-abort** est LE mécanisme de protection. Sur A12+:
- Premier `abort()` → queue le ZLP (comme sur A11)
- Deuxième `abort()` → **reap (free) le ZLP** ← C'EST ÇA LE PATCH
- Puis `bzero()` nettoie tout

---

## 2. TOUS LES OUTILS PUBLICS S'ARRÊTENT À A11

| Outil | Support max | A12? |
|-------|-----------|------|
| checkra1n | A11 (jusqu'à iOS 18.7.1) | NON |
| palera1n | A11 (jusqu'à iOS 18.7.1) | NON |
| ipwndfu | T8015 (A11) | NON |
| gaster | A5-A11 (payloads A9/notA9 seulement) | NON |
| checkm8-a5 | A5 (Arduino) | NON |

**Aucun outil public ne supporte l'A12.**

---

## 3. JAILBREAKS A12+ : TOUS AU NIVEAU KERNEL (PAS BOOTROM)

Les jailbreaks qui fonctionnent sur A12+ utilisent TOUS des vulnérabilités dans iOS/XNU, 
pas dans le bootrom :

| Jailbreak | iOS | Exploits utilisés |
|-----------|-----|-------------------|
| Chimera | 12.0-12.5.7 | voucher_swap, SockPuppet (kernel) |
| Unc0ver | 12.0-14.8 | voucher_swap, SockPuppet, oob_timestamp, tachy0n (kernel) |
| Taurine | 14.0-14.8.1 | cicuta_virosa (kernel) |
| XinaA15 | 15.0-15.1.1 | multicast_bytecopy, weightBufs (kernel) |
| Dopamine | 15.0-16.6.1 | kfd, dmaFail, badRecovery (kernel+PPL+PAC) |

**AUCUN de ces jailbreaks ne contourne le SecureROM.** Ils opèrent après le boot, 
au niveau du kernel iOS. Ils ne permettent PAS de dualboot car le bootrom vérifie 
toujours la chain of trust au démarrage.

---

## 4. SEPROM : BLACKBIRD NE TOUCHE PAS A12

Source: [theapplewiki.com/wiki/Blackbird_Exploit](https://theapplewiki.com/wiki/Blackbird_Exploit)

> "A12 and later are believed to no longer drop the bits and are therefore 
> not vulnerable to the bug."

- blackbird exploite un décalage 32-bit→shift dans les registres TZ0/TZ1
- Vulnérable: A8, A8X, A9, A9X, A10, A10X, T2
- A11: bug existe mais le memory integrity tree empêche l'exploitation
- **A12+: le bug n'existe tout simplement plus**

---

## 5. SWD (Serial Wire Debug) : BLOQUÉ PAR CPFM

Source: [theapplewiki.com/wiki/Serial_Wire_Debug](https://theapplewiki.com/wiki/Serial_Wire_Debug)

> "Serial Wire Debug is only available when the SoC has a CPFM which is lower 
> than 0x01, or if the device is demoted."

- Notre device: **CPFM=0x03** (production fused)
- SWD nécessite CPFM < 0x01 ou device demoté
- Les sondes SWD sont du matériel Apple interne (KongSWD, KanziSWD, KobaSWD)
- KIS-SWD (logiciel) disponible uniquement pour A12+... MAIS nécessite quand même CPFM bas
- **"Demotion" d'un device production n'est possible qu'avec un exploit bootrom existant** → cercle vicieux

---

## 6. dmaFail : INTÉRESSANT MAIS PAS BOOTROM

Source: [theapplewiki.com/wiki/DmaFail](https://theapplewiki.com/wiki/DmaFail)

- Vulnérabilité dans les registres debug AGX (GPU)
- Permet d'écrire dans la mémoire physique via le cache L2 du GPU
- **Affecte A12-A14** sur iOS ≤ 16.5
- Utilisé par Dopamine 2.x comme PPL bypass
- MAIS : nécessite d'abord code execution via un kernel exploit
- C'est une vuln iOS, pas bootrom — ne survit pas à un reboot

---

## 7. VECTEURS THÉORIQUES RESTANTS

### 7A. Alternative Memory Leak sur A12 (★★★★☆)

**C'est LA question centrale.** Le UAF existe. Si on trouve un AUTRE moyen de 
leak des objets heap qui survivent au shutdown USB, checkm8 pourrait fonctionner 
sur A12.

Angles à explorer dans notre ROM :
1. **Race condition entre les deux aborts** : Le second `abort()` free le ZLP. 
   Si on pouvait faire en sorte que le second abort rate sa cible...
2. **Autres types de callbacks** : `standard_device_request_cb()` n'est pas la 
   seule callback. Y a-t-il d'autres chemins qui allouent des objets persistants ?
3. **Timing manipulation** : Un glitch EMFI sur la seule instruction du second 
   abort qui free le ZLP
4. **Allocation parasite** : D'autres opérations USB qui allouent des objets 
   entre les deux aborts

### 7B. EMFI/Voltage Glitching (★★★☆☆)

- Riscure (maintenant Keysight) avait de la recherche sur "Glitching the Apple iPhone"
  mais leur page redirige maintenant
- Quarkslab a soudé des fils UART sur un Pixel/Titan M pour debugger
- L'idée : glitcher le `CBZ` après `img4_validate_property_callback` 
  pour forcer le branchement "valid"
- Nécessite : oscilloscope, générateur de glitch, sonde EM, beaucoup de patience
- Alternative ciblée : glitcher UNIQUEMENT le second `abort()` dans le ZLP path 
  pour "débloquer" le leak checkm8

### 7C. Bootrom Revision 3865.0.0.4.6 (★★☆☆☆)

L'A12 a TROIS revisions de bootrom :
- 3865.0.0.1.23 (non-production)
- **3865.0.0.4.6** ← potentiellement plus ancien
- 3865.0.0.4.7 ← notre version

Si la rev .4.6 a un bug corrigé dans .4.7, trouver un device .4.6 serait utile.
Mais : les bootrom sont en hardware (ROM mask), donc probablement tous les XR en 
circulation ont la même revision.

### 7D. Nonce Collision / Downgrade via Kernel (★★☆☆☆)

Avec un kernel exploit (via Dopamine/kfd sur iOS ≤ 16.6.1) :
- On pourrait set le boot-nonce
- Downgrade vers une ancienne version iOS
- Mais on ne contourne PAS le bootrom — le Secure Enclave valide le nonce

### 7E. Attaque Physique: Chip-off / Microprobing (★☆☆☆☆)

- Decap le SoC et lire la ROM directement
- Altérer les fuses CPFM pour activer SWD
- Nécessite : FIB (Focused Ion Beam), labo semi-conducteur
- Coût: >100K€, risque de destruction total du chip

---

## 8. PLAN D'ACTION CONCRET

### Phase 1: Analyser le double-abort dans notre ROM (FAISABLE MAINTENANT)

Dans notre dump T8020 B1, trouver :
- La fonction `usb_core_abort_endpoint()` ou équivalent
- Le chemin précis du double-abort sur EP0_IN
- Le code qui reap/free le ZLP après le premier abort
- **Chercher si le timing ou l'ordre du double-abort a une faille**

### Phase 2: Chercher des allocations alternatives (FAISABLE MAINTENANT)

Dans le code USB/DFU de la ROM :
- Mapper TOUS les chemins d'allocation heap depuis les handlers USB
- Identifier tout objet alloué qui n'est PAS correctement libéré 
  lors du shutdown USB
- Chercher des callbacks qui allouent des objets "oubliés"

### Phase 3: EMFI ciblé (NÉCESSITE MATÉRIEL)

Si on identifie l'instruction exacte qui free le ZLP dans le second abort :
- Un glitch EMFI unique et précis sur cette instruction pourrait "skip" le free
- Le ZLP survivrait → le heap feng shui de checkm8 fonctionnerait 
- C'est une attaque sur UNE seule instruction, pas sur tout le bootrom
- Ça combine le meilleur du software (UAF existant) et du hardware (glitch ciblé)

---

## 9. VERDICT FINAL

| Catégorie | État |
|-----------|------|
| UAF checkm8 dans la ROM | ✅ PRÉSENT (non patché) |
| Memory leak ZLP | ❌ BLOQUÉ (double-abort) |
| Alternative memory leak | ❓ INCONNUE (à chercher) |
| Outils publics A12 | ❌ AUCUN |
| SEPROM blackbird | ❌ NON VULNÉRABLE |
| SWD debug | ❌ CPFM=0x03 (production) |
| Kernel exploits A12+ | ✅ EXISTENT (mais pas bootrom) |
| EMFI/Glitch ciblé | ❓ THÉORIQUEMENT VIABLE |

**Le problème est réduit à UNE seule question :**

> **Existe-t-il un moyen de leak des objets heap dans le SecureROM A12 
> qui survivent au shutdown USB, OU de glitcher le second abort pour 
> restaurer le leak ZLP original ?**

Si OUI → checkm8 fonctionne sur A12.
Si NON → le T8020 est effectivement inexpugnable par voie logicielle pure.

---

## 10. SOURCES

1. theapplewiki.com/wiki/Checkm8_Exploit — Confirmation UAF non-patché sur A12
2. alfiecg.uk/2023/07/21/checkm8 — Mécanisme exact du double-abort A12+
3. theapplewiki.com/wiki/T8020 — "Vulnerabilities: None" pour T8020
4. theapplewiki.com/wiki/Blackbird_Exploit — SEPROM non-vuln sur A12+
5. theapplewiki.com/wiki/Serial_Wire_Debug — SWD nécessite CPFM<0x01
6. theapplewiki.com/wiki/DmaFail — PPL bypass kernel-level (pas bootrom)
7. theapplewiki.com/wiki/Jailbreak_Exploits — Historique complet des exploits
8. theapplewiki.com/wiki/Bootrom_Exploits — Revisions bootrom A12
9. github.com/0x7ff/gaster — A5-A11 only
10. littlelailo gist (apollo.txt) — Description originale du bug checkm8
11. blog.quarkslab.com — Méthodologie attaque Titan M (référence glitching)
