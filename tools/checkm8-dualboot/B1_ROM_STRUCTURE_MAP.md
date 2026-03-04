# T8020 B1 SecureROM — CARTE STRUCTURELLE COMPLÈTE

## Vue d'ensemble

| Paramètre | Valeur |
|---|---|
| ROM | T8020 B1 (iPhone XR) |
| Taille | 524,288 bytes (512 KB) |
| Base | `0x100000000` |
| Code actif | `0x100000000 - 0x10001BC00` (~110 KB) |
| Données/Constantes | `0x10001BC00 - 0x100022000` (~25 KB) |
| Padding (zéros) | `0x100022000 - 0x100080000` (~375 KB) |
| Fonctions trouvées | **448** (via prologue STP x29, x30) |
| Fonctions analysées | **23** (= **5%**) |
| **Fonctions INEXPLORÉES** | **425** (= **95%**) |
| Code bytes analysés | 4,192 / 110,904 = **3.8%** |

---

## Carte mémoire par région (4KB pages)

```
0x100000000 - 0x100001FFF  EARLY BOOT + RESET VECTOR + EXCEPTION TABLE
                            21 fonctions — AUCUNE ANALYSÉE ⚠️
                            (exception handlers, EL switch, PAC key init)

0x100002000 - 0x100002FFF  USB CONTROLLER INIT
                            21 fonctions — 4 connues, 17 INCONNUES ⚠️
                            (usb_init, usb_alloc, usb_reset, usb_detect OK)
                            (+ contrôleur USB raw, endpoint config, descriptors)

0x100003000 - 0x100004FFF  ★★★ DFU HANDLER ★★★
                            38 fonctions — AUCUNE ANALYSÉE ⚠️⚠️⚠️
                            C'est le code qui TRAITE les paquets USB en mode DFU!
                            Inclut: transfer handlers, state machine,
                            buffer management, protocol parsing
                            → 0x100004CB8: MONSTRE de 1,512B avec 25 calls, 48 branches!

0x100005000 - 0x100005FFF  IMG4 / SIGNATURE CHAIN
                            5 fonctions — AUCUNE ANALYSÉE directement ⚠️
                            (0x100005480 = img4_verify_internal tracée via rom_verify_trace)
                            0x100005494: 2,500B, 66 calls, 98 branches = MASSIF
                            0x1000052A0: 500B, 7 calls, 14 branches

0x100006000 - 0x100006FFF  PLATFORM CONFIG / HARDWARE SETUP
                            24 fonctions — 3 connues, 21 INCONNUES ⚠️
                            (mmio_config, sram_base, addr_validator OK)
                            → config matériel, GPIO, clock setup, SoC init

0x100007000 - 0x100007FFF  VALIDATION / PROPERTY CHECKS
                            24 fonctions — AUCUNE ANALYSÉE ⚠️⚠️
                            0x1000075B4: 1,492B, 20 branches = gros validateur
                            Contient: vérificateurs de propriétés, checks d'intégrité
                            → MSR VBAR_EL1 à 0x100007A04 (re-route exceptions!)

0x100008000 - 0x100008FFF  PANIC / LOGGING / ERROR HANDLING
                            23 fonctions — 1 connue (panic_handler), 22 INCONNUES ⚠️
                            → Tout le système de logging/debug

0x100009000 - 0x100009FFF  ★★★ SECURITY SERVICES ★★★
                            34 fonctions — 2 connues, 32 INCONNUES ⚠️⚠️⚠️
                            TRÈS appelée =  116 xrefs!
                            0x100009B64: 16× appelée! (plus appelée des inconnues)
                            0x100009438: 14× appelée!
                            → Gestion de nonces, tokens, état sécuritaire,
                            politiques de boot, configuration crypto

0x10000A000 - 0x10000AFFF  IMG4 VERIFY FRAMEWORK
                            41 fonctions — AUCUNE ANALYSÉE directement ⚠️⚠️
                            → Toute la mécanique IMG4 autour de la vérification
                            Parseurs de manifest, extracteurs de payload,
                            validateurs de tags. Seul img4_verify tracé.

0x10000B000 - 0x10000CFFF  ★★★ IO / TRANSPORT LAYER ★★★
                            33 fonctions — AUCUNE ANALYSÉE ⚠️⚠️⚠️
                            Gros fonctions: 0x10000C3BC (716B, 23 calls, 20 branches)
                            0x10000C974 (832B, 26 branches)
                            → Couche transport pour USB, DART, DMA(?)
                            Potentiel: buffer overflows, parsing de données entrantes

0x10000D000 - 0x10000EFFF  ★★★ DER / ASN.1 PARSER ★★★
                            40 fonctions — AUCUNE ANALYSÉE ⚠️⚠️⚠️
                            0x10000D958: 1,572B, 49 branches = PARSEUR RÉCURSIF
                            0x10000D5EC: 876B, 14 branches
                            12× fonctions de 44B identiques = dispatch stubs
                            → HISTORIQUEMENT la source #1 de bugs SecureROM!
                            Apple a eu des CVEs dans leur parser DER/ASN.1

0x10000F000 - 0x100010FFF  HEAP / MEMORY MANAGEMENT
                            37 fonctions — 7 connues, 30 INCONNUES ⚠️
                            0x1000100AC: 2,044B, 51 branches = allocateur principal
                            → Métadonnées de heap, validation de chunks
                            corruption de heap = code execution

0x100011000 - 0x100011FFF  SYNC / SOC / LOCKS
                            31 fonctions — 5 connues, 26 INCONNUES ⚠️
                            → Primitives de synchronisation, accès SoC
                            Potentiel: race conditions si DFU multi-threaded

0x100012000 - 0x100014FFF  ★★★ ASN.1 / CERT / DER (DEEP) ★★★
                            39 fonctions — AUCUNE ANALYSÉE ⚠️⚠️⚠️
                            0x10001489C: 1,316B, 40 branches
                            0x100014DC0: 1,308B, 25 branches
                            Ce sont les vérificateurs de CERTIFICATS X.509
                            → Parsing de certificats = surface d'attaque classique

0x100015000 - 0x100016FFF  CRYPTO: EC/RSA
                            9 fonctions — AUCUNE ANALYSÉE ⚠️⚠️
                            0x100016C78: 4,000B, 99 branches = EC point multiplication?
                            0x100016218: 2,484B, 82 branches = RSA/ECDSA verify?
                            0x100015680: 1,700B, 40 branches = bignum ops?
                            → Side-channels, timing, implementation bugs

0x100017000 - 0x10001BFFF  CRYPTO: AES/HASH + DONNÉES
                            28 fonctions — AUCUNE ANALYSÉE ⚠️⚠️
                            0x10001AC24: 4,000B = SHA-512 unrolled loop
                            0x10001A184: 640B = AES S-Box?
                            0x100018BD8: 1,052B, 21 branches = AES-GCM?
                            → Implémentations crypto, S-Boxes intégrées

0x10001C000 - 0x100021FFF  DATA: STRINGS + CONSTANTES + DISPATCH TABLES
                            Strings: "IMG4", "DFU Mode", "panic:", "USB DART",
                            "Apple Secure Boot Root CA - G21", "bootstrap"
                            CENTAINES de pointeurs dispatch (ASN.1 type handlers!)
                            → Tables de dispatch = surface de confusion de type
```

---

## Les 10 zones les plus dangereuses à explorer

### 🔴 PRIORITÉ 1: DFU HANDLER (0x3000-0x5000)
- **38 fonctions, 0 analysées**
- C'est LE code qui reçoit les données USB de l'attaquant
- 0x100004CB8 = 1,512 bytes, 48 branches — probablement la machine d'état DFU
- checkm8 original exploitait exactement ce type de code sur A0
- **Pourquoi inexploré**: On a cherché des patterns de bugs connus, pas RE le code DFU en entier
- **Potentiel**: State machine bugs, buffer overflows, use-after-free, double-free

### 🔴 PRIORITÉ 2: DER/ASN.1 PARSER (0xD000-0xEFFF)
- **40 fonctions, 0 analysées** 
- Parser récursif de 1,572B à 0x10000D958 avec 49 branches
- Parser de structures imbriquées = terrain fertile pour les bugs
- **Pourquoi inexploré**: On a scanné les patterns (truncation, overflow) mais JAMAIS lu la logique
- **Potentiel**: Integer overflow dans length fields, récursion non-bornée, confusion de types

### 🔴 PRIORITÉ 3: CERTIFICAT X.509 VERIFIER (0x12000-0x14FFF)
- **39 fonctions, 0 analysées**
- DEUX fonctions de ~1,300B chacune avec 25-40 branches
- Tables de dispatch massives à 0x100021xxx contrôlent le comportement
- **Pourquoi inexploré**: Jamais touché
- **Potentiel**: Parsing de certificats forgés, extensions X.509 malformées, OID confusion

### 🟠 PRIORITÉ 4: IO/TRANSPORT (0xB000-0xCFFF)
- **33 fonctions, 0 analysées**
- Couche intermédiaire entre USB et img4_verify
- 0x10000C974 = 832B, 26 branches
- **Potentiel**: Buffer management, read/write primitives mal bornées

### 🟠 PRIORITÉ 5: SECURITY SERVICES (0x9000-0xA000)
- **34 fonctions, seulement 2 connues**
- La région PLUS APPELÉE du ROM (116 xrefs)
- 0x100009B64 appelée 16× — on ne sait même pas ce que c'est!
- **Potentiel**: Gestion de nonces, politiques de boot, état de sécurité

### 🟠 PRIORITÉ 6: Exception Handlers (0x0000-0x1000)
- **21 fonctions, 0 analysées**
- VBAR_EL1 pointe vers 0x100000000
- 4× ESR_EL1 readings à 0x820, 0x9A0, 0xC20, 0xDA0
- 4× FAR_EL1 readings aux mêmes endroits
- **Potentiel**: Exception handler confusion, fault injection response

### 🟡 PRIORITÉ 7: VALIDATION/CHECKS (0x7000-0x8000)
- **24 fonctions, 0 analysées**
- 0x1000075B4 = 1,492B avec 20 branches = gros validateur
- MSR VBAR_EL1 à 0x100007A04 = re-route la table d'exceptions!
- **Potentiel**: Bypass de validation, VBAR manipulation

### 🟡 PRIORITÉ 8: HEAP internals (0xF000-0x11000)
- **37 fonctions, 7 connues, 30 inconnues**
- 0x1000100AC = 2,044B = allocateur principal complet, 51 branches  
- **Potentiel**: Heap metadata corruption, unlink primitives

### 🟡 PRIORITÉ 9: IMG4 Framework complet (0xA000-0xAFFF)
- **41 fonctions autour de verify, AUCUNE analysée individuellement**
- On a tracé la chaîne verify → résultat, mais PAS les parsers intermédiaires
- **Potentiel**: Manifest parsing bugs, tag confusion, payload extraction flaws

### 🟡 PRIORITÉ 10: Crypto EC/RSA/AES (0x15000-0x1B000)
- **37 fonctions, 0 analysées**
- 0x100016C78 = 4,000B = probablement multiplication de points EC
- **Potentiel**: Side-channels, implementation bugs, padding oracle

---

## Résumé par subsystème

| Subsystème | Fonctions | Analysées | % Couvert | Code (bytes) |
|---|---|---|---|---|
| Early Boot/Exception | 21 | 0 | **0%** | ~5,500 |
| USB Controller | 21 | 4 | 19% | ~3,800 |
| **DFU Handler** | **38** | **0** | **0%** | **~8,400** |
| IMG4/Signature | 5 | 1* | 20% | ~4,200 |
| Platform Config | 24 | 3 | 13% | ~4,800 |
| Validation/Checks | 24 | 0 | **0%** | ~5,200 |
| Panic/Logging | 23 | 1 | 4% | ~4,600 |
| **Security Services** | **34** | **2** | **6%** | **~4,400** |
| **IMG4 Framework** | **41** | **0** | **0%** | **~4,300** |
| **IO/Transport** | **33** | **0** | **0%** | **~7,600** |
| **DER/ASN.1** | **40** | **0** | **0%** | **~8,800** |
| Heap/Memory | 37 | 7 | 19% | ~9,200 |
| Sync/SoC | 31 | 5 | 16% | ~4,400 |
| **Cert/X.509/DER** | **39** | **0** | **0%** | **~11,200** |
| **Crypto EC/RSA** | **9** | **0** | **0%** | **~12,200** |
| **Crypto AES/Hash** | **28** | **0** | **0%** | **~17,000** |

*tracée via call chain, pas RE individuellement

---

## Éléments structurels découverts

### Tables de dispatch (CRITIQUES)
- **0x100020E10**: 24 entrées pointant vers 0x10001C3xx = pointeurs de fonctions string/data
- **0x100021038**: 16 entrées pointant vers fonctions IO/Transport (0x10000Axxx - 0x10000Cxxx)
- **0x100021118**: 5 entrées pointant vers deep parsers ASN.1 (0x100012xxx - 0x100013xxx)
- **0x1000211A0 - 0x100021988**: ~200+ entrées = **TABLE DE DISPATCH ASN.1 GÉANTE**
  - 3 handler principaux: 0x100014ACC, 0x100014B50, 0x100014B94
  - Avec variantes: 0x100014D70, 0x100014D90, 0x100014D40
  - + handlers spéciaux: 0x100014B04, 0x100014C10, 0x100014C28, 0x100014BE8, 0x100014BC4
  - **C'est un jump table par type ASN.1!** Chaque type DER a son handler.

### Registres système utilisés
- **8 clés PAC** initialisées à 0x100000614-0x100000668
- **SCTLR_EL1** écrit 3× (MMU/cache config)
- **TCR_EL1** écrit 1× (translation control)
- **TTBR0_EL1** écrit 2× (page table base — intéressant: pourquoi 2×?)
- **VBAR_EL1** écrit 2× (0x100000048 et 0x100007A04 — 2 tables d'exception!)
- **15+ registres Apple-propriétaires** (S3_0_C15_Cx_y)
- **ESR_EL1** lu 4× (dans exception handlers — analyze pourquoi!)
- **FAR_EL1** lu 4× aux mêmes endroits

---

## Conclusion: CE QU'ON A FAIT vs CE QU'ON N'A PAS FAIT

### ✅ Ce qu'on a couvert:
1. Chaîne img4_verify → résultat (return values)
2. Patterns de bugs A0 → absents en B1
3. Truncation, SXTW, overflow → scans automatiques
4. Fonctions spécifiques: boot, heap_alloc, addr_validator, panic
5. Comparaison binaire A0 vs B1

### ❌ Ce qu'on n'a PAS fait:
1. **LIRE le code DFU** — 38 fonctions, la surface d'attaque principale
2. **LIRE le parser DER/ASN.1** — 80+ fonctions, source historique de bugs
3. **LIRE le vérificateur de certificats** — parsing d'input potentiellement contrôlé
4. **LIRE la couche IO/Transport** — intermédiaire USB→verify
5. **Comprendre les dispatch tables** — 200+ pointeurs ASN.1
6. **Analyser les exception handlers** — 4 ESR reads inexpliqués
7. **Comprendre les security services** — 34 fonctions, coeur sécuritaire
8. **RE les crypto primitives** — EC 4,000B, AES 1,052B
9. **Tracer la 2ème écriture VBAR** — pourquoi changer de table d'exception?
10. **Tracer les 2 écritures TTBR0** — 2 configurations de page table = 2 états MMU?

### 🎯 Stratégie recommandée:
**DFU Handler → DER/ASN.1 → Cert Parser → IO/Transport → Security Services**

Le DFU handler est la porte d'entrée. Le parser DER/ASN.1 est historiquement le plus bugué. Les certificats sont forgés par l'attaquant. Ces 3 régions représentent **~28,000 bytes de code** qu'on n'a JAMAIS lu.
