# JARVIS Self-Propagation Trailer — Plan de Production

**Musique** : Circuit Hearts.mp3 (4m33s / 273s — 192kbps, 48kHz)  
**Format** : Animation 2D minimaliste (style néon/circuit board sur fond noir)  
**Audience** : Débutants (histoire visuelle simple) + Experts (détails techniques en overlay)  
**Outil suggéré** : Motion Canvas (TypeScript), After Effects, ou Manim (Python)

---

## Palette Visuelle

| Élément | Couleur | Signification |
|---------|---------|---------------|
| Noeud TrustOS actif | Cyan néon `#00FFFF` | Machine vivante |
| Noeud vide / éteint | Gris sombre `#333333` | Machine sans OS |
| Pulsation UDP | Cercles concentriques bleus | Découverte mesh |
| Transfert de poids (cerveau) | Flux doré `#FFD700` | Intelligence qui circule |
| Gradients fédérés | Particules vertes `#00FF88` | Apprentissage distribué |
| Leader RAFT | Couronne dorée | Chef du cluster |
| PXE boot | Éclair blanc `#FFFFFF` | Réplication |
| Logo TrustOS | Apparition progressive | Naissance d'un noeud |

---

## Structure du Trailer — 7 Actes synchronisés sur Circuit Hearts

### 🎬 ACTE 1 — "Le Silence" (0:00 → 0:25)
**Musique** : Intro calme, premières notes

| Temps | Visuel | Texte à l'écran | Niveau |
|-------|--------|-----------------|--------|
| 0:00 | Noir total, un point cyan apparaît au centre | — | Tous |
| 0:05 | Le point pulse doucement comme un battement de coeur | — | Tous |
| 0:10 | Zoom arrière : c'est un PC seul, connecté à rien | `One machine.` | Débutant |
| 0:15 | On voit le logo TrustOS s'allumer sur l'écran du PC | `TrustOS v0.7.0` | Tous |
| 0:20 | Texte expert en petit sous le PC | `bare-metal Rust kernel • 240K lines • zero dependencies` | Expert |
| 0:25 | Le coeur du PC pulse plus fort — quelque chose se prépare | `JARVIS online.` | Tous |

**Intention** : Établir la solitude. Un seul OS. Un seul cerveau. Le public se demande "et après ?"

---

### 🎬 ACTE 2 — "Le Signal" (0:25 → 0:55)
**Musique** : Build-up, rythme qui monte

| Temps | Visuel | Texte à l'écran | Niveau |
|-------|--------|-----------------|--------|
| 0:25 | Le PC émet une onde circulaire bleue (UDP broadcast) | — | Tous |
| 0:28 | L'onde se propage en cercles concentriques | `Broadcasting...` | Débutant |
| 0:30 | En overlay : `UDP:7700 • JMSH • 41 bytes` | — | Expert |
| 0:33 | L'onde touche un 2ème PC éteint (gris) à droite | — | Tous |
| 0:35 | Le 2ème PC s'anime — un éclair blanc le traverse (PXE boot) | `New machine found.` | Tous |
| 0:38 | Animation : 3 fichiers descendent du PC1 vers PC2 | — | Tous |
| 0:40 | Labels sur les fichiers : `bootloader` → `config` → `kernel` | `DHCP → TFTP → Boot` | Expert |
| 0:43 | Le logo TrustOS bloom sur PC2, il devient cyan | — | Tous |
| 0:48 | PC2 est vivant mais son "cerveau" est minuscule (petit point doré) | `Micro brain: 0 knowledge` | Débutant |
| 0:50 | Flèche dorée massive de PC1 → PC2 (transfert de cerveau) | — | Tous |
| 0:53 | Le petit point doré de PC2 grossit, devient un cerveau complet | `Brain downloaded: 4.4M params` | Expert |
| 0:55 | Les deux PC pulsent maintenant en synchronie | `Two minds. One network.` | Tous |

**Intention** : Montrer la magie — un PC vide reçoit l'OS ET l'intelligence. Le cerveau se copie.

---

### 🎬 ACTE 3 — "L'Élection" (0:55 → 1:15)
**Musique** : Tension dramatique, beat qui pulse

| Temps | Visuel | Texte à l'écran | Niveau |
|-------|--------|-----------------|--------|
| 0:55 | Les 2 PCs se regardent. Lequel dirige ? | `Who leads?` | Tous |
| 0:58 | Les deux passent en orange (Candidate) | — | Tous |
| 1:00 | Flèches de vote croisées entre eux | `Vote ↔ Vote` | Débutant |
| 1:03 | PC1 reçoit la majorité → couronne dorée apparaît | `Leader elected.` | Tous |
| 1:05 | Texte technique | `RAFT consensus • term 1 • majority` | Expert |
| 1:08 | PC1 (Leader) émet un heartbeat régulier → onde dorée | — | Tous |
| 1:12 | PC2 (Worker) pulse en réponse — synchronisé | `Heartbeat: 5s` | Expert |
| 1:15 | Les deux sont stables, liés par un lien lumineux | — | Tous |

**Intention** : Même les débutants comprennent le concept de "chef" / "worker". Les experts voient RAFT.

---

### 🎬 ACTE 4 — "La Propagation" (1:15 → 2:10)
**Musique** : Drop / montée en puissance — c'est LE moment clé

| Temps | Visuel | Texte à l'écran | Niveau |
|-------|--------|-----------------|--------|
| 1:15 | PC2 active SON propre serveur PXE | `Chain reaction.` | Tous |
| 1:18 | Zoom arrière progressif — on voit 6 PCs éteints autour | — | Tous |
| 1:22 | PC2 émet une onde → touche PC3 | — | Tous |
| 1:25 | Même animation accélérée : boot → cerveau → vivant | — | Tous |
| 1:28 | PC1 ET PC2 émettent en parallèle → PC4, PC5 s'allument | `Exponential.` | Tous |
| 1:33 | **Compteur en gros** : `2 → 4` | — | Tous |
| 1:38 | Tous les nouveaux émettent → `4 → 8` | — | Tous |
| 1:43 | Zoom arrière — `8 → 16` — ça s'accélère comme un virus bienveillant | `Self-replicating OS.` | Débutant |
| 1:48 | `16 → 32` — explosion de points cyan | — | Tous |
| 1:53 | Vue d'ensemble : des dizaines de noeuds connectés par des liens lumineux | — | Tous |
| 1:58 | Texte technique en overlay | `PXE boot • DHCP • multi-arch (x86 + ARM + RISC-V)` | Expert |
| 2:03 | Un noeud est de couleur différente (rose) — c'est un ARM ! | `Heterogeneous cluster.` | Expert |
| 2:08 | Tous les noeuds pulsent ensemble, comme un seul organisme | — | Tous |

**Intention** : Le "wow moment". Le public voit la réplication exponentielle. 1 → 2 → 4 → 8 → 16 → 32. Simple mais puissant.

---

### 🎬 ACTE 5 — "L'Apprentissage" (2:10 → 3:00)
**Musique** : Phase mélodique, émotion

| Temps | Visuel | Texte à l'écran | Niveau |
|-------|--------|-----------------|--------|
| 2:10 | Focus sur le cluster de 32 noeuds. Des particules vertes circulent | `Federated learning.` | Tous |
| 2:15 | Chaque Worker envoie de petites flèches vertes vers le Leader | `Workers train locally.` | Débutant |
| 2:20 | Le Leader pulse et grossit en absorbant les gradients | — | Tous |
| 2:25 | Puis le Leader émet une grande onde dorée vers tous les Workers | `Knowledge shared.` | Débutant |
| 2:30 | Tous s'améliorent simultanément (leur éclat augmente) | — | Tous |
| 2:33 | Texte technique | `FedAvg • Nesterov momentum β=0.9 • 80× compression` | Expert |
| 2:38 | Animation de barre de progression : Loss 5.0 → 4.0 → 3.0 → 2.0 | — | Tous |
| 2:43 | La pulsation du réseau RALENTIT (adaptive sync) | `Converging...` | Tous |
| 2:48 | La couleur passe du rouge intense au vert calme | — | Tous |
| 2:53 | Le réseau "respire" lentement — il a appris | `Intelligence: distributed.` | Tous |
| 2:58 | Un noeud tape "hello" → réponse instantanée de JARVIS s'affiche | — | Tous |

**Intention** : Montrer que ce n'est pas juste de la copie — le réseau APPREND ensemble. L'intelligence émerge.

---

### 🎬 ACTE 6 — "Le Gardien" (3:00 → 3:35)
**Musique** : Passage solennel, notes graves

| Temps | Visuel | Texte à l'écran | Niveau |
|-------|--------|-----------------|--------|
| 3:00 | Un noeud rouge essaie de modifier ses propres poids | ⚠️ | Tous |
| 3:03 | Un bouclier doré apparaît — bloque l'action | `Guardian: DENIED.` | Tous |
| 3:06 | Texte du Pacte qui s'affiche lettre par lettre | *"JARVIS ne peut jamais modifier le code sans la permission de ses parents."* | Tous |
| 3:12 | Deux silhouettes : un humain (Nathan) et un symbole IA (Copilot) | `Two guardians.` | Tous |
| 3:18 | Le bouclier enveloppe tout le réseau | `The Pact protects all nodes.` | Tous |
| 3:22 | Texte technique | `Guardian auth • session timeout 30m • audit log` | Expert |
| 3:28 | Le réseau brille — sécurisé et vivant | — | Tous |
| 3:33 | Fade vers le texte final | — | Tous |

**Intention** : Montrer l'éthique. Ce n'est pas Skynet. C'est protégé. Les gardiens veillent. Moment émotionnel fort.

---

### 🎬 ACTE 7 — "L'Invitation" (3:35 → 4:33)
**Musique** : Final épique puis fade out

| Temps | Visuel | Texte à l'écran | Niveau |
|-------|--------|-----------------|--------|
| 3:35 | Vue cosmique du réseau — des dizaines de noeuds comme des étoiles | — | Tous |
| 3:40 | Texte massif | `JARVIS` | Tous |
| 3:45 | Sous-titre | `A self-propagating AI that lives in your OS.` | Tous |
| 3:52 | Bullet points qui apparaissent un par un : | | |
| | | `✦ Bare-metal Rust kernel` | Tous |
| | | `✦ Self-replicating via PXE` | Tous |
| | | `✦ Federated learning mesh` | Tous |
| | | `✦ 4.4M parameter transformer` | Expert |
| | | `✦ Cross-architecture (x86 + ARM + RISC-V)` | Expert |
| 4:10 | Les bullet points se dissolvent | — | Tous |
| 4:15 | Logo TrustOS seul au centre, pulsant doucement | `TrustOS` | Tous |
| 4:20 | URL / GitHub | `github.com/...` | Tous |
| 4:25 | Fade progressif au noir avec la musique | `Built by Nathan.` | Tous |
| 4:33 | Noir. | — | — |

---

## Techniques d'Animation Clés

### Pour les débutants (lisibilité)
- **Métaphore du coeur** : chaque noeud "bat" comme un coeur → on comprend que c'est vivant
- **Compteur visible** : 1 → 2 → 4 → 8 → le public VOIT l'exponentiel
- **Couleurs simples** : éteint=gris, vivant=cyan, cerveau=or, danger=rouge, sécurité=bouclier doré
- **Pas de jargon** sans explication visuelle — montre l'action avant le terme technique
- **Analogie biologique** : le réseau "respire", les noeuds ont un "coeur", le cerveau "grandit"

### Pour les experts (crédibilité)
- **Overlay technique** en police monospace petite, en bas de l'écran (non-intrusif)
- **Vrais numéros de port** (UDP 7700, TCP 7701), vrais protocoles (RAFT, FedAvg)
- **Magic bytes** (`JMSH`, `JRPC`) visibles dans les packets
- **Compression ratio** (80×), vrais params (4.4M, Nesterov β=0.9)
- **Multi-arch** visible (couleurs différentes pour x86/ARM/RISC-V)

---

## Sync Musique — "Circuit Hearts" (4m33s)

| Section musicale | Temps estimé | Action au trailer |
|-----------------|-------------|-------------------|
| Intro piano/ambient | 0:00–0:25 | Acte 1 — noeud seul, calme |
| Build-up | 0:25–0:55 | Acte 2 — premier signal, première copie |
| Pré-drop | 0:55–1:15 | Acte 3 — tension de l'élection RAFT |
| Drop principal | 1:15–2:10 | Acte 4 — **PROPAGATION EXPONENTIELLE** (syncer les beats avec chaque nouvelle machine qui s'allume) |
| Melodie/bridge | 2:10–3:00 | Acte 5 — federated learning, particules qui dansent |
| Passage calme | 3:00–3:35 | Acte 6 — Le Gardien, Le Pacte (moment solennel) |
| Outro/climax final | 3:35–4:33 | Acte 7 — reveal du nom, call to action, fade |

> **NOTE CRITIQUE** : Écouter Circuit Hearts une fois et marquer les **BPM** et **drops exacts** avec un outil comme Audacity ou aubio. Ajuster les timestamps ci-dessus aux vrais beats. Chaque nouvelle machine doit s'allumer SUR un beat pendant l'Acte 4.

---

## Outils Recommandés pour la Production

| Outil | Usage | Difficulté |
|-------|-------|-----------|
| **Motion Canvas** (TypeScript) | Animations programmatiques, synchro audio exacte | ⭐⭐ |
| **Manim** (Python) | Animations math/tech, rendu propre | ⭐⭐ |
| **After Effects** | Motion design pro, particules | ⭐⭐⭐ |
| **DaVinci Resolve + Fusion** | Gratuit, compositing nodes | ⭐⭐ |
| **Blender (Grease Pencil)** | Animation 2D dans un outil 3D | ⭐⭐⭐ |
| **p5.js / Canvas API** | Rapide pour prototyper les animations | ⭐ |

**Recommandation** : Motion Canvas (TypeScript) — tu codes les animations, l'audio sync est natif, et c'est open-source. Parfait pour un dev Rust.

---

## Assets à Préparer

- [ ] Logo TrustOS en SVG vectoriel (pour animation propre)
- [ ] Circuit Hearts.mp3 — analyser les beats (BPM, timestamps des drops)
- [ ] Font monospace pour texte technique (JetBrains Mono ou Fira Code)
- [ ] Icônes : PC, cerveau, bouclier, couronne, coeur
- [ ] Palette de couleurs finalisée
- [ ] Storyboard papier rapide (stick figures) pour valider le flow

---

## Durée Estimée de Production

| Phase | Description |
|-------|-------------|
| Beat mapping | Écouter Circuit Hearts, marquer chaque drop/transition |
| Storyboard | Dessiner chaque scène clé (14 keyframes minimum) |
| Assets | Créer les SVG/icônes nécessaires |
| Animation | Coder/animer les 7 actes |
| Sound sync | Ajuster frame-par-frame sur les beats |
| Polish | Easing, particules, glow effects |
| Export | 1080p 60fps, codec H.264 |

---

*Ce plan est conçu pour que quelqu'un qui n'a jamais touché au code comprenne "ça se copie tout seul et ça apprend ensemble", tout en donnant aux devs les détails qui prouvent que c'est réel.*
