# Chapitre 1 – Introduction : Pourquoi TRustOs

## 🎬 Narration

> *"Windows domine le marché desktop depuis des décennies. Mais sous sa surface familière se cache une architecture héritée de choix faits il y a 30 ans. TRustOs n'est pas une alternative – c'est une réimagination complète de ce qu'un OS moderne pourrait être."*

---

## 📖 Script narratif

### Scène 1.1 – Le constat
**Durée estimée**: 2-3 minutes

**Voix off**:
"Chaque jour, des millions de développeurs lancent Windows. Et chaque jour, ils font face aux mêmes frustrations : un système monolithique, des permissions archaïques, des crashes inexplicables, et un kernel qui n'a jamais été conçu pour le monde moderne."

**Visuels suggérés**:
- Écran bleu de la mort (BSOD) classique
- Task Manager surchargé
- Permissions Windows complexes

### Scène 1.2 – La vision
**Durée estimée**: 2 minutes

**Voix off**:
"Et si on repartait de zéro ? Pas pour copier Windows, mais pour créer quelque chose de fondamentalement différent. Un OS où la sécurité n'est pas un ajout – elle est dans l'ADN. Où le debug n'est pas un cauchemar – il est intégré. Où une IA peut vous aider à administrer votre système, en toute sécurité."

**Visuels suggérés**:
- Animation : transition Windows → architecture TRustOs
- Diagramme microkernel vs monolithique

### Scène 1.3 – TRustOs
**Durée estimée**: 1-2 minutes

**Voix off**:
"Voici TRustOs. Un OS écrit en Rust, avec un microkernel de moins de 10 000 lignes de code. Des services userland isolés. Une sécurité basée sur des capabilities. Et un assistant IA intégré nommé Jarvis."

**Visuels suggérés**:
- Logo TRustOs (à créer)
- Architecture schématique animée
- Terminal avec prompt "Jarvis>"

---

## 🎨 Storyboard visuel

```
┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 1.1 - LE CONSTAT                                     │
├─────────────────────────────────────────────────────────────┤
│  [Image: BSOD Windows]                                      │
│  Texte: "30 ans d'héritage technique..."                   │
│                                                             │
│  [Transition: fade to black]                                │
│                                                             │
│  [Image: Task Manager surchargé]                           │
│  Texte: "...et des frustrations quotidiennes"              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 1.2 - LA VISION                                      │
├─────────────────────────────────────────────────────────────┤
│  [Animation: Bloc monolithique → Microkernel + services]   │
│                                                             │
│  ┌──────────┐        ┌────┐ ┌────┐ ┌────┐                 │
│  │ MONOLITH │   →    │ μK │ │ FS │ │ AI │                 │
│  │ KERNEL   │        └────┘ └────┘ └────┘                 │
│  └──────────┘          ↑       ↑       ↑                   │
│                        └───IPC async───┘                   │
│                                                             │
│  Texte: "Une architecture repensée"                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 1.3 - TRUSTOS                                       │
├─────────────────────────────────────────────────────────────┤
│  [Image: Terminal TRustOs]                                 │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ TRustOs v0.1.0                                   │   │
│  │ ─────────────────────────────────────────────────── │   │
│  │ [INFO ] Kernel initialized successfully!            │   │
│  │ [INFO ] Jarvis AI Agent ready.                      │   │
│  │                                                      │   │
│  │ jarvis> _                                           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Texte: "Bienvenue dans TRustOs"                          │
└─────────────────────────────────────────────────────────────┘
```

---

## 📊 Liens techniques

| Concept narratif | Implémentation technique | Fichier source |
|------------------|-------------------------|----------------|
| Microkernel minimal | `#![no_std]`, kernel < 10K LOC | `kernel/src/main.rs` |
| Sécurité intégrée | Capability-based security | `kernel/src/security/` |
| Debug natif | Event tracing, ring buffer | `kernel/src/trace/` |
| IA intégrée | Jarvis (Phase 2) | `userland/jarvis/` (à venir) |

---

## ✅ Statut chapitre

- [x] Script narratif initial
- [x] Storyboard conceptuel
- [ ] Visuels finaux
- [ ] Enregistrement voix off
- [ ] Montage vidéo
