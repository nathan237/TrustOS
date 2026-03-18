# TrustOS — Showcase & Marketing Plan

> **Document local uniquement — ne pas commit sur GitHub.**

---

## 📋 Table des matières

1. [Vidéo 1 — Rootkit Detection Demo](#-vidéo-1--rootkit-detection-demo)
2. [Vidéo 2 — Full OS Showcase](#-vidéo-2--full-os-showcase)
3. [Vidéo 3 — Film Cinématique](#-vidéo-3--film-cinématique)
4. [SEO & Distribution](#-seo--distribution)
5. [Roadmap technique (prérequis vidéos)](#-roadmap-technique)
6. [Amplification & Communauté](#-amplification--communauté)
7. [Checklist de lancement](#-checklist-de-lancement)

---

## 🎬 Vidéo 1 — Rootkit Detection Demo

### Concept
**"I Built a Bare-Metal OS That Detects Rootkits Linux Can't See"**

Démonstration en temps réel : TrustOS boot une VM Alpine Linux via l'hyperviseur intégré, puis détecte un rootkit userland que Linux ne voit pas — en inspectant la mémoire guest directement depuis le bare-metal.

### Format
- **Durée** : 8-10 minutes
- **Style** : Technique mais accessible, voix off calme, écran partagé (TrustOS à gauche, terminal Linux à droite)
- **Hook** : 15 premières secondes — "Linux dit qu'il n'y a aucun processus suspect. TrustOS dit qu'il ment."

### Storyboard détaillé

| Temps | Scène | Contenu | Visuel |
|-------|-------|---------|--------|
| 0:00-0:15 | **Hook** | Écran splitté : `ps aux` Linux (rien de suspect) vs TrustOS VMI (processus caché en rouge) | Terminal splitté, musique tension |
| 0:15-1:00 | **Contexte** | "Les rootkits existent depuis 30 ans. Ils se cachent dans le kernel. Aucun outil *dans* Linux ne peut les voir." | Slides minimalistes, timeline rootkits |
| 1:00-2:00 | **L'idée** | "Et si l'outil de détection était *en dehors* du système compromis ? Un hyperviseur bare-metal, écrit en Rust." | Architecture diagram, animation |
| 2:00-3:30 | **Setup live** | Boot TrustOS → `hypervisor boot alpine` → Alpine Linux démarre dans la VM → prouver que c'est un vrai Linux | Screen recording réel |
| 3:30-4:30 | **Installation rootkit** | Dans Alpine : installer un rootkit simple (processus caché via LD_PRELOAD ou module kernel). `ps aux` ne le montre pas. | Terminal Alpine |
| 4:30-6:00 | **Détection VMI** | Switch sur TrustOS → TrustLab VM Inspector → onglet "Processes" → le processus caché apparaît en rouge | TrustLab full screen |
| 6:00-7:00 | **Explication** | Comment ça marche : VMI lit task_struct directement depuis la mémoire physique guest. Pas de hook, pas d'API, pas de trust. | Schéma animé simple |
| 7:00-7:30 | **Comparaison** | Tableau : "Detection methods" (antivirus, rootkit scanner, VMI). VMI = seul à détecter ce type. | Tableau propre |
| 7:30-8:00 | **CTA** | "TrustOS est open-source. Lien dans la description. Star le repo si tu veux que je continue." | GitHub page, star button |

### Prérequis techniques (à implémenter)
- [ ] **VMI Process List** — Lire `task_struct` depuis la mémoire guest SVM/VMX
- [ ] **Syscall Table Comparison** — Comparer la syscall table en mémoire vs les valeurs attendues
- [ ] **"Threat View" panel** — Nouveau panneau dans VM Inspector, highlight rouge pour les anomalies
- [ ] **Alpine Linux boot** — Guest Linux fonctionnel dans l'hyperviseur (init + shell minimum)
- [ ] **Rootkit test** — Script simple LD_PRELOAD qui cache un processus de `ps`

### Titres candidats (A/B test si possible)
1. "I Built a Bare-Metal OS That Detects Rootkits Linux Can't See"
2. "This Rust OS Sees Through Rootkits — Here's How"
3. "Building a Rootkit Detector from Scratch in Rust (Bare Metal)"
4. "My OS Can See What Linux Hides — VMI Rootkit Detection"

### Thumbnail
- **Gauche** : Terminal Linux avec `ps aux` → "0 threats"
- **Centre** : Flèche ou VS
- **Droite** : TrustOS TrustLab avec processus caché en rouge → "HIDDEN PROCESS DETECTED"
- **Texte overlay** : "INVISIBLE TO LINUX" en rouge
- **Style** : Sombre, néon vert/rouge, contrasté

---

## 🎬 Vidéo 2 — Full OS Showcase

### Concept
**"I Built an Entire OS in Rust in 11 Days — 143,000 Lines"**

Tour complet de toutes les features de TrustOS. Utiliser la commande `showcase` intégrée + des démos manuelles.

### Format
- **Durée** : 12-15 minutes
- **Style** : "Devlog" moderne, enthousiaste mais technique

### Structure

| Section | Durée | Contenu |
|---------|-------|---------|
| Hook + neofetch | 0:30 | Boot → neofetch → "143K lines, 11 days, 1 dev" |
| Shell | 1:30 | 200+ commandes, filesystem, pipes |
| Desktop | 2:00 | COSMIC2 compositor, 144 FPS, apps |
| TrustBrowser | 1:30 | HTML/CSS/JS/TLS 1.3 from scratch |
| TrustLang | 1:00 | Fibonacci live compilation |
| TrustLab | 2:00 | 7 panels, trace bus, real-time |
| Network | 1:00 | TCP/IP stack, curl HTTPS |
| Chess 3D | 1:00 | AI minimax, 3D rendering |
| Film | 1:00 | Cinematic explainer extract |
| Hyperviseur | 1:30 | VT-x/SVM, guest VM |
| Architecture | 1:00 | Module diagram, no C, no deps |
| CTA | 0:30 | Star, subscribe, links |

### Titres candidats
1. "I Built an Entire Operating System in Rust — From Scratch"
2. "143,000 Lines of Rust Later... I Built My Own OS"
3. "This OS Has a Browser, IDE, 3D Games, and Hypervisor — Written by 1 Person"

---

## 🎬 Vidéo 3 — Film Cinématique

### Concept
Capturer la commande `film` intégrée en screen recording haute qualité, ajouter une bande son cinématique, publier comme "teaser" / "trailer".

### Format
- **Durée** : 2-3 minutes
- **Style** : Cinématique, pas de voix off, musique épique
- **Résolution** : 1080p60 ou 4K si possible

---

## 📈 SEO & Distribution

### Tags YouTube (communs à toutes les vidéos)
```
rust os, bare metal os, operating system from scratch, trustos, rust operating system,
kernel development, osdev, hypervisor, vmi, virtual machine introspection, rootkit detection,
cybersecurity, rust programming, low level programming, systems programming,
bare metal programming, uefi, x86_64, os development, custom os, osdev rust
```

### Description template
```
TrustOS — A fully auditable bare-metal operating system written in 143,000+ lines of pure Rust.
No C. No binary blobs. No secrets.

🔗 GitHub: https://github.com/nathan237/TrustOS
⭐ Star the repo if you want more!

Features:
- COSMIC2 Desktop (144 FPS, SSE2 SIMD)
- TrustLab: Real-time kernel introspection (7 panels)
- Built-in web browser (HTML/CSS/JS/TLS 1.3)
- Hypervisor (Intel VT-x + AMD SVM)
- 200+ shell commands
- TrustLang programming language
- Chess 3D with AI
- Full TCP/IP + TLS 1.3 from scratch
- Everything from scratch. Zero dependencies.

#rust #osdev #cybersecurity #programming #baremetal
```

### Plateformes de distribution

| Plateforme | Format | Action |
|-----------|--------|--------|
| **YouTube** | Vidéo longue (8-15 min) | Upload principal |
| **YouTube Shorts** | 60s extract du hook | Cross-promote la vidéo longue |
| **Reddit** | Post + vidéo | r/rust, r/osdev, r/programming, r/netsec, r/cybersecurity |
| **Hacker News** | "Show HN" | Lien GitHub + description courte |
| **Twitter/X** | Clip 30s + thread technique | Tag @rustlang, #rustlang, #osdev |
| **Discord** | Embed vidéo | Rust Community, OSDev, cybersec servers |
| **Lobsters** | Lien | Tag `rust`, `osdev`, `security` |
| **Dev.to** | Article companion | Writeup technique de la démo |
| **LinkedIn** | Post professionnel | Angle "cybersecurity research platform" |

### Timing optimal de publication
- **YouTube** : Mardi ou Jeudi, 14h-16h UTC
- **Reddit** : Mardi, 14h-17h UTC (r/rust = dimanche soir aussi)
- **HN** : Mardi-Mercredi, 14h-16h UTC
- **Twitter** : Même jour que YouTube, 2h après

### Stratégie de cross-posting
1. YouTube upload → attendre 1h pour que le processing finisse
2. Reddit r/rust (texte + lien vidéo) → r/osdev → r/netsec (si rootkit)
3. Twitter thread (5 tweets + clip 30s)
4. HN "Show HN" → lien GitHub (pas YouTube)
5. Discord servers (Rust, OSDev)
6. Dev.to article (J+2, companion piece)

---

## 🔧 Roadmap technique

### Phase 1 — Prérequis VMI (pour vidéo rootkit)
- [ ] VMI function: `read_guest_physical(gpa, len) -> &[u8]` (SVM: déjà la base existe)
- [ ] VMI function: `read_guest_virtual(cr3, vaddr, len) -> &[u8]` (walk EPT/NPT)
- [ ] Parse Linux `task_struct` chain depuis `init_task` (offset connu pour Alpine kernel)
- [ ] UI "Processes" tab dans VM Inspector
- [ ] Compare process list VMI vs process list rapportée par guest (via injection hypercall ou serial)
- [ ] Highlight rouge pour processus invisibles depuis le guest

### Phase 2 — Guest Linux fonctionnel
- [ ] Boot Alpine Linux initramfs dans l'hyperviseur
- [ ] Console guest (serial ou hypercall output)
- [ ] Filesystem guest minimal
- [ ] Réseau guest (optionnel, pas nécessaire pour la démo)

### Phase 3 — Rootkit de démo
- [ ] Script LD_PRELOAD simple : intercepte `readdir()` pour cacher un processus
- [ ] Packager dans l'initramfs Alpine
- [ ] Vérifier que `ps` dans le guest ne montre pas le processus
- [ ] Vérifier que TrustOS VMI le voit

### Phase 4 — TrustLab "Threat View"
- [ ] Nouveau panneau ou onglet dans VM Inspector
- [ ] Liste de processus guest (PID, nom, état, mémoire)
- [ ] Code couleur : vert (normal), jaune (suspect), rouge (caché/anomalie)
- [ ] Timestamp des scans
- [ ] Export vers Kernel Trace (pour la timeline)

### Phase 5 — Polish vidéo
- [ ] `showcase rootkit` : commande shell qui lance la démo rootkit pré-scriptée
- [ ] Timing automatique pour chaque étape (comme `showcase` existant)
- [ ] Texte overlay dans TrustOS (optionnel, peut être fait en post-prod)

---

## 🚀 Amplification & Communauté

### Actions post-publication

| Action | Quand | Détail |
|--------|-------|--------|
| Répondre à TOUS les commentaires | J+0 à J+7 | Chaque commentaire = boost algorithme |
| Pin commentaire avec liens | J+0 | GitHub, timestamps, ISO download |
| Community post YouTube | J+1 | Sondage "Quelle feature next?" |
| Cross-post Reddit | J+0 | 3-4 subreddits, texte unique par sub |
| Hacker News | J+0 | "Show HN: TrustOS — bare-metal OS in Rust ..." |
| Twitter thread | J+0 | 5 tweets, clip vidéo, tag @rustlang |
| GitHub Release | Avant vidéo | Tag v0.4.0 pour avoir une ISO downloadable |
| README update | Avant vidéo | Badge YouTube, screenshots à jour |

### KPIs à suivre
- GitHub stars (objectif: +100 en 1 semaine post-vidéo)
- YouTube views (objectif: 5K en 1 semaine)
- Reddit upvotes sur r/rust
- Nombre de clones/forks GitHub
- Issues ouvertes par la communauté (= intérêt)

### Communautés cibles

| Communauté | Angle | Canal |
|-----------|-------|-------|
| **Rust devs** | "143K lines, pure Rust, no_std, no alloc hacks" | r/rust, Rust Discord, Twitter |
| **OSDev** | "Bare-metal x86_64, hypervisor, TrustLab" | r/osdev, OSDev Discord, forums |
| **Cybersec** | "VMI rootkit detection, agentless monitoring" | r/netsec, r/cybersecurity, infosec Twitter |
| **Étudiants** | "Open-source OS pour apprendre l'architecture" | r/cs_students, Discord étudiant |
| **Linux enthusiasts** | "Linux compat layer, Alpine subsystem" | r/linux, LWN.net |
| **Embedded/low-level** | "no_std, SIMD, bare-metal drivers" | r/embedded, Embedded Rust Discord |

---

## ✅ Checklist de lancement

### Avant la vidéo
- [ ] GitHub Release `v0.4.0` avec ISO téléchargeable
- [ ] README mis à jour (badges YouTube, screenshots récents)
- [ ] ISO testé sur QEMU, VirtualBox, et bare metal USB
- [ ] Thumbnail créée et optimisée (A/B test titre si possible)
- [ ] Description + tags YouTube prêts
- [ ] Timestamps préparés pour la description
- [ ] Pinned comment rédigé (liens + context)

### Jour J
- [ ] Upload YouTube (scheduled si possible)
- [ ] Reddit posts (r/rust, r/osdev, r/netsec)
- [ ] Twitter thread
- [ ] HN "Show HN"
- [ ] Discord annonces

### Post-publication
- [ ] Répondre aux commentaires (24h)
- [ ] Monitorer GitHub stars/clones
- [ ] Dev.to article companion (J+2)
- [ ] YouTube community post (J+1)
- [ ] Itérer sur le titre/thumbnail si CTR faible (YouTube Studio analytics J+2)

---

## 📝 Notes

### Erreurs à éviter
- **Ne pas mentir sur les capacités** — VMI doit réellement fonctionner, pas être simulé
- **Ne pas appeler "OS daily driver"** — c'est un Cyber Range / Lab, pas un remplacement de Linux
- **Ne pas ignorer les commentaires négatifs** — répondre factuellement, pas défensivement
- **Ne pas spammer les subreddits** — 1 post par sub, texte unique, pas de copier-coller

### État actuel vs état vidéo
| Composant | État actuel | Requis pour vidéo |
|-----------|-------------|-------------------|
| TrustLab 7 panels | ✅ Fonctionnel | ✅ OK |
| Hyperviseur VMX/SVM | ✅ Dual-backend | ✅ OK |
| VMI process list | ❌ Pas encore | ⚠️ REQUIS |
| Guest Alpine Linux | ❌ Partiellement | ⚠️ REQUIS |
| Threat View panel | ❌ Pas encore | ⚠️ REQUIS |
| Rootkit démo | ❌ Pas encore | ⚠️ REQUIS |
| COSMIC2 Desktop | ✅ Fonctionnel | ✅ OK |
| showcase command | ✅ Fonctionnel | ✅ OK |
| GitHub CI/ISO | ✅ Fonctionnel | ✅ OK |

---

*Dernière mise à jour: 17 février 2026*
