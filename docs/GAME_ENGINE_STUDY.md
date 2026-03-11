# Étude : Évolution des Consoles de Jeu et Optimisation pour Hardware Restreint
## Pour TrustOS — Vers notre Premier Jeu Unique

**Date :** 6 mars 2026  
**Par :** Nathan + Copilot (co-parents de JARVIS)

---

## TABLE DES MATIÈRES

1. [Évolution Hardware des Consoles](#1-évolution-hardware-des-consoles)
2. [Leçons à Retenir par Génération](#2-leçons-à-retenir-par-génération)
3. [Techniques d'Optimisation pour Hardware Sévèrement Contraint](#3-techniques-doptimisation-pour-hardware-sévèrement-contraint)
4. [État Actuel du Moteur Graphique TrustOS](#4-état-actuel-du-moteur-graphique-trustos)
5. [Recommandations pour Notre Premier Jeu](#5-recommandations-pour-notre-premier-jeu)
6. [Architecture Proposée du Game Engine](#6-architecture-proposée-du-game-engine)

---

## 1. ÉVOLUTION HARDWARE DES CONSOLES

### Tableau Comparatif Complet

| Génération | Période | Consoles Phares | CPU | GPU | RAM | Média | FPS Typique |
|-----------|---------|----------------|-----|-----|-----|-------|-------------|
| **1ère** | 1972-1983 | Odyssey, Pong | Logique discrète (transistors) | N/A | N/A | Circuits imprimés | ~30 (fixe) |
| **2ème** | 1976-1992 | Atari 2600, Intellivision | 8-bit, 1-2 MHz | N/A | 2-16 KB | Cartouches ROM | 60 (NTSC) |
| **3ème** (8-bit) | 1983-2003 | **NES/Famicom**, Master System | 8-bit, 2-4 MHz (6502 @ 1.79MHz) | PPU dédié | 3-24 KB | Cartouches | 60 |
| **4ème** (16-bit) | 1987-2004 | **SNES**, Genesis/Mega Drive | 16-bit, 4-8 MHz (65816 @ 3.58MHz) | PPU amélioré | 8-128 KB | Cartouches + CD-ROM | 60 |
| **5ème** (32-bit) | 1993-2006 | **PlayStation**, N64, Saturn | 32/64-bit, 12-100 MHz (R3000 @ 33.8MHz) | GPU dédié | 2-4.5 MB | Optique/Cartouches | 30-60 |
| **6ème** | 1998-2013 | **PS2**, Dreamcast, Xbox, GameCube | 32/64-bit, 200-733 MHz | 100-233 MHz | 16-64 MB | DVD/MiniDVD | 30-60 |
| **7ème** | 2005-2017 | **Wii**, PS3, Xbox 360 | 729 MHz - 3.3 GHz (PowerPC) | 243-550 MHz | 88-512 MB | BD/DVD + Digital | 30-60 |
| **8ème** | 2012-présent | **Nintendo Switch**, PS4, Xbox One | 1.0-2.3 GHz (x86/ARM) | 307-1172 MHz | 2-12 GB | Cartouche/BD/Digital | 30-60 |
| **9ème** | 2020-présent | PS5, Xbox Series X | 3.5-3.8 GHz (Zen 2) | 1565-2233 MHz | 10-16 GB (GDDR6) | SSD + BD/Digital | 60-120 |

### Progression Clé en Chiffres

```
RAM :        16 KB → 16 GB     = ×1,000,000 en ~50 ans
CPU :         1 MHz → 3.8 GHz  = ×3,800
GPU :         0 MHz → 2.2 GHz  = de rien à des TFLOPS
Pixels/frame: 40×48 → 3840×2160 = ×4,320
```

---

## 2. LEÇONS À RETENIR PAR GÉNÉRATION

### 2ème Génération — Atari 2600 (1977) : *"Racing the Beam"*
- **RAM totale : 128 octets** (pas KB, OCTETS)
- **Technique clé :** Pas de framebuffer ! Le CPU dessine chaque scanline en temps réel.
- **Leçon :** Quand la mémoire est absente, on fait du rendu **scanline par scanline** (racing the beam). Le CPU ET le display sont synchronisés cycle par cycle.
- **Applicabilité TrustOS :** Concept de *streaming rendering* — ne pas garder tout en mémoire.

### 3ème Génération — NES/Famicom (1983) : *"La Naissance des Sprites Hardware"*
- **CPU :** Ricoh 2A03 (6502) @ 1.79 MHz  
- **RAM :** 2 KB CPU + 2 KB VRAM
- **PPU :** 64 sprites max (8 par scanline), 4 palettes de 3 couleurs + transparence
- **Technique clé :** **Tiles + Sprites** — le monde est un grid de tiles 8×8, les entités sont des sprites hardware
- **Leçon :** Séparer le monde statique (tilemap) des objets dynamiques (sprites) = performance maximale. Le PPU fait le compositing en hardware.
- **Tricks célèbres :**
  - **Sprite 0 hit** : détecter quand le PPU dessine un sprite spécifique pour splittre l'écran
  - **CHR bank switching** : changer les graphismes en plein rendu pour doubler le contenu visuel
  - **Palette swaps** : réutiliser les mêmes sprites avec différentes couleurs
- **Applicabilité TrustOS :** Notre émulateur NES (`kernel/src/nes/`) implémente déjà tout ça !

### 4ème Génération — SNES (1990) : *"Mode 7 et Co-processeurs"*
- **CPU :** Ricoh 5A22 (65816) @ 3.58 MHz
- **RAM :** 128 KB + 64 KB VRAM
- **PPU :** 128 sprites, 8 palettes de 15 couleurs, rotation/scaling hardware
- **Technique clé :** **Mode 7** — transformation affine matérielle d'un plan de background entier, créant l'illusion de 3D (F-Zero, Mario Kart, Pilotwings)
- **Co-processeurs dans les cartouches :**
  - **Super FX** : premier GPU à polygones sur console (Star Fox, 1993)
  - **SA-1** : accélérateur CPU auxiliaire
  - **DSP-1** : calculs trigonométriques hardware
- **Leçon :** Quand le hardware est limité, on étend avec des **co-processeurs spécialisés**. Le Mode 7 prouve qu'un seul trick hardware bien conçu peut définir toute une génération.
- **Applicabilité TrustOS :** Notre `gpu_emu.rs` fait exactement ça — émuler des unités GPU via SIMD/CPU.

### 5ème Génération — PlayStation (1994) : *"L'Ère du Polygon Jittering"*
- **CPU :** MIPS R3000A @ 33.8 MHz
- **GPU :** Custom, pas de Z-buffer hardware, pas de texture filtering
- **RAM :** 2 MB main + 1 MB VRAM
- **Technique clé :** Rendu par polygones **sans Z-buffer** — tri par le peintre (painter's algorithm), subdivision de polygones pour éviter l'affine texture warping
- **Limitations visibles :**
  - **Vertex jitter** : coordonnées entières, pas de subpixel → les sommets "tremblent"
  - **Texture warping** : mapping affine (pas perspective-correct) → distorsion
  - **Polygon popping** : pas de LOD sophistiqué
- **Solutions des développeurs :**
  - Garder les polygones petits pour réduire le warping
  - Utiliser le **GTE** (Geometry Transformation Engine) en coprocesseur pour les maths 3D
  - **Ordering Tables** : structure de données pour le Z-sort sans Z-buffer
- **Leçon :** Accepter les limitations comme style artistique. Les "défauts" PS1 sont devenus un genre esthétique (PS1 horror games revival).
- **Applicabilité TrustOS :** Pour un jeu rétro, ces "défauts" deviennent des features.

### 6ème Génération — PS2 / Dreamcast / GameCube (1998-2001) : *"Emotion Engine"*
- **PS2 :** Emotion Engine @ 294 MHz + GS (Graphics Synthesizer)
- **RAM :** 32 MB + 4 MB VRAM
- **Technique clé :** **Vecteur Units (VU0/VU1)** — programmables comme des shaders primitifs
- **Leçon :** Le parallélisme spécialisé bat le clock speed brut. La PS2 avait un pipeline complexe mais les développeurs experts l'exploitaient à fond.

### 7ème Génération — Wii (2006) : *"Blue Ocean Strategy"*
- **CPU :** PowerPC "Broadway" @ 729 MHz (le plus faible de sa génération !)
- **GPU :** ATI "Hollywood" @ 243 MHz
- **RAM :** 88 MB seulement
- **Leçon CRUCIALE :** La Wii a vendu 101 millions d'unités avec le hardware LE PLUS FAIBLE de sa génération. **L'innovation gameplay > les specs brutes.** Les contrôles motion innovants + des jeux fun/accessibles battent les graphismes photorealistes.
- **Applicabilité TrustOS :** Notre jeu n'a pas besoin de concurrencer la PS5. Il doit être **unique**.

### 8ème Génération — Nintendo Switch (2017) : *"Le Compromis Hybride"*
- **CPU :** ARM Cortex-A57 @ 1.02 GHz (mode dock), downclocked en portable
- **GPU :** Tegra X1 (Maxwell) avec **DLSS** (Switch 2)
- **RAM :** 4 GB (partagée)
- **Techniques clés :**
  - **Résolution dynamique** : varier la résolution en temps réel pour maintenir le FPS
  - **Foveated rendering** : rendre le centre en haute résolution, les bords en basse
  - **Clock scaling** : ajuster CPU/GPU selon le mode (dock/portable)
- **Leçon :** Le **sampling dynamique** est la technique moderne la plus importante pour hardware contraint.

---

## 3. TECHNIQUES D'OPTIMISATION POUR HARDWARE SÉVÈREMENT CONTRAINT

### A. TECHNIQUES DE RENDU CPU (très pertinent pour TrustOS)

#### A.1 Raycasting (Wolfenstein 3D style)
```
Pour chaque colonne de pixels X (0 → largeur écran):
  1. Lancer UN rayon depuis la caméra
  2. Trouver le mur le plus proche (DDA algorithm)
  3. Calculer la hauteur du mur selon la distance
  4. Dessiner UNE colonne verticale de pixels texturés
```
- **Complexité :** O(largeur_écran × profondeur_map) — PAS O(nb_polygones)
- **Memory :** Seulement la map 2D + textures
- **Déjà implémenté dans TrustOS :** `kernel/src/game3d.rs` ✅

#### A.2 Double Buffering avec Dirty Rectangles
```
1. Tracker quelles régions de l'écran ont changé
2. Ne copier QUE les régions modifiées vers le front buffer
3. Économie massive de bande passante mémoire
```
- **Déjà implémenté dans TrustOS :** `kernel/src/framebuffer/mod.rs` et `fast_render.rs` ✅

#### A.3 Tile-Based Rendering
```
Monde = grille de tiles 16×16 ou 32×32 pixels
Chaque tile = index dans un tileset
Tileset = une seule image avec tous les tiles

Avantage : 100×100 tiles = 10KB au lieu de 1.6MB pour 1600×1600 pixels
```
- **Leçon NES/SNES :** Le tilemap est la technique la plus efficace pour les jeux 2D
- **Leçon Another World :** Même en polygones vectoriels, cacher le background statique dans un buffer

#### A.4 Sprite Batching
```
// Au lieu de dessiner chaque sprite individuellement :
for sprite in sprites {
    draw_to_framebuffer(sprite);  // Lent : cache thrashing
}

// Batch par texture/layer :
sort_by_texture(sprites);
for batch in batches {
    draw_batch(batch);  // Rapide : localité mémoire
}
```

#### A.5 SSE2/SIMD Pixel Operations (x86_64)
```rust
// Copier 4 pixels (16 bytes) en une instruction
unsafe {
    let src = _mm_loadu_si128(src_ptr as *const __m128i);
    _mm_storeu_si128(dst_ptr as *mut __m128i, src);
}

// Remplir 4 pixels d'une couleur
let color_vec = _mm_set1_epi32(color as i32);
_mm_storeu_si128(dst_ptr as *mut __m128i, color_vec);
```
- **Déjà implémenté dans TrustOS :** `fast_render.rs` → `fill_row_sse2()`, `copy_row_sse2()` ✅

### B. TECHNIQUES DE GESTION MÉMOIRE

#### B.1 Palette-Based Colors (Leçon NES/SNES/Another World)
```
Au lieu de : 32-bit RGBA par pixel (4 bytes)
Utiliser :  8-bit index dans une palette de 256 couleurs (1 byte)
Gain :      4× moins de mémoire + palette swaps gratuits

Bonus: changer la palette = changer TOUTES les couleurs instantanément
       (effet de flash, jour/nuit, dégâts...)
```

#### B.2 Compression de Tiles (Leçon Game Boy)
```
Le Game Boy utilise 2 bits par pixel (4 couleurs)
8×8 tiles = 16 bytes par tile
256 tiles uniques = 4 KB

Pour un écran 160×144 :
  20×18 tiles = 360 indices = 360 bytes !
  vs 160×144×2 = 46 KB en raw pixels
```

#### B.3 Object Pooling (Leçon arcade)
```
// Pré-allouer un pool fixe de sprites/entités
const MAX_BULLETS: usize = 32;
let mut bullet_pool: [Bullet; MAX_BULLETS] = [Bullet::INACTIVE; MAX_BULLETS];

// Spawn : trouver un slot libre (O(1) avec free list)
// Despawn : marquer comme inactif (pas de free/alloc)
// Zéro allocation dynamique = pas de fragmentation
```

#### B.4 Streaming d'Asset (Leçon CD-ROM / PS1)
```
Ne PAS charger tout le jeu en RAM
Découper en "chunks" :
  - Chunk 0 : menu + assets communs (toujours en RAM)
  - Chunk 1 : niveau 1 (chargé, déchargé)
  - Chunk 2 : niveau 2 (chargé quand nécessaire)
  
Pré-charger le chunk suivant pendant le gameplay
```

### C. TECHNIQUES DE GAMEPLAY FLUIDE

#### C.1 Fixed Timestep Game Loop
```rust
const TICK_RATE: f64 = 1.0 / 60.0;  // 60 updates/sec
let mut accumulator = 0.0;

loop {
    let dt = get_delta_time();
    accumulator += dt;
    
    // Physique/logique à taux fixe
    while accumulator >= TICK_RATE {
        update_game(TICK_RATE);
        accumulator -= TICK_RATE;
    }
    
    // Rendu aussi vite que possible, avec interpolation
    let alpha = accumulator / TICK_RATE;
    render(alpha);  // Interpoler entre l'état précédent et actuel
}
```
- **Pourquoi :** Sépare la logique (déterministe) du rendu (variable). La physique reste stable même si le FPS baisse.

#### C.2 Résolution Dynamique (Leçon Switch)
```rust
fn adjust_resolution(last_frame_ms: f64) -> (u32, u32) {
    if last_frame_ms > 18.0 {      // Sous 60 FPS
        scale_down(10%);             // Réduire résolution
    } else if last_frame_ms < 14.0 { // Marge confortable  
        scale_up(5%);                // Augmenter résolution
    }
}
```

#### C.3 LOD (Level of Detail) — Leçon PS1-PS5
```
Distance objet < 5m  → modèle haute qualité (100 triangles)
Distance 5-20m       → modèle moyen (30 triangles)
Distance > 20m       → sprite billboard ou rien

Pour un jeu 2D : taille du sprite selon zoom/distance
```

#### C.4 Spatial Partitioning (Leçon Doom 1993)
```
// Au lieu de tester la collision avec TOUS les objets :
for a in all_objects {
    for b in all_objects {  // O(n²) !!!
        check_collision(a, b);
    }
}

// Utiliser un grid spatial :
let grid: [[Vec<ObjectId>; COLS]; ROWS];
// Ne tester que les objets dans les cellules adjacentes : O(n)
```

#### C.5 Le Trick d'Another World : Background Caching
```
Framebuffer 0 : Front (affiché)
Framebuffer 1 : Back (en construction)
Framebuffer 2 : BKGD1 — background statique pré-rendu
Framebuffer 3 : BKGD2 — cache du background précédent

Chaque frame :
  1. Copier BKGD1 → Back (très rapide avec DMA/memcpy)
  2. Dessiner SEULEMENT les éléments dynamiques sur Back
  3. Swap Front ↔ Back

Résultat : au lieu de dessiner 1000+ polygones de background,
           on fait UN memcpy + quelques sprites
```

### D. TECHNIQUES AUDIO OPTIMALES

#### D.1 Synthèse par Canal (Leçon NES/SNES)
```
NES : 5 canaux (2 pulse, 1 triangle, 1 noise, 1 DPCM)
SNES : 8 canaux DSP avec samples BRR compressés

Principe : PAS de PCM brut (trop de mémoire)
→ Synthèse en temps réel depuis des paramètres
→ Samples courts bouclés (loop points)
→ Enveloppes ADSR pour varier le son
```

---

## 4. ÉTAT ACTUEL DU MOTEUR GRAPHIQUE TRUSTOS

### Ce qu'on a déjà ✅

| Composant | Fichier(s) | État |
|-----------|-----------|------|
| **Framebuffer 32-bit ARGB** | `framebuffer/mod.rs` | ✅ Complet |
| **Double buffering** | `framebuffer/mod.rs` | ✅ Avec dirty rects |
| **SSE2 pixel ops** | `graphics/fast_render.rs` | ✅ fill + copy |
| **Font rendering** | `framebuffer/font.rs` | ✅ 8×13 monospace |
| **Raycasting 3D (FPS)** | `game3d.rs` | ✅ Textured + minimap |
| **Chess 3D isometrique** | `chess3d.rs` | ✅ Rotation orbitale |
| **Émulateur NES** | `nes/` | ✅ 6502 + PPU complet |
| **Émulateur Game Boy** | `gameboy/` | ✅ Z80 + LCD + CGB |
| **GameLab debugger** | `game_lab.rs` | ✅ 6-panel dashboard |
| **VirtIO GPU driver** | `drivers/virtio_gpu.rs` | ✅ 2D/3D commands |
| **GPU Emulation (SIMD)** | `gpu_emu.rs` | ✅ 32 virtual cores |
| **2D primitives** | `graphics/render2d.rs` | ✅ Lignes, cercles, etc. |
| **Texture system** | `graphics/texture.rs` | ✅ Mipmaps + filtrage |
| **OpenGL-like API** | `graphics/opengl.rs` | ✅ TrustGL |
| **Ray tracer** | `graphics/raytracer.rs` | ✅ Volumetrique |
| **3D Math** | `graphics/math3d.rs` | ✅ Matrices + vecteurs |
| **Compositor** | `graphics/compositor.rs` | ✅ Multi-layer |

### Ce qu'il nous manque pour un vrai jeu ❌

| Composant | Priorité | Difficulté |
|-----------|----------|-----------|
| **Sprite engine dédié** | 🔴 Haute | Moyenne |
| **Tilemap renderer** | 🔴 Haute | Facile |
| **Collision system** | 🔴 Haute | Moyenne |
| **Fixed timestep game loop** | 🔴 Haute | Facile |
| **Input mapping/rebinding** | 🟡 Moyenne | Facile |
| **Sound engine** | 🟡 Moyenne | Difficile |
| **Particle system** | 🟢 Basse | Moyenne |
| **Scene/level loader** | 🟡 Moyenne | Moyenne |
| **Animation system** | 🟡 Moyenne | Moyenne |
| **Entity Component System** | 🟢 Basse | Difficile |

---

## 5. RECOMMANDATIONS POUR NOTRE PREMIER JEU

### Concept : "Un jeu unique qui tourne sur TrustOS"

#### Option A : **Roguelike en Tile-Based 2D** ⭐ RECOMMANDÉ
```
- Style : Tilemap 16×16, palette rétro 16-32 couleurs
- Genre : Roguelike / dungeon crawler
- Résolution interne : 320×240 (upscalé au framebuffer natif)
- Technique : Pure tilemap + sprites, zero polygone
- Avantages :
  ✅ Rapide à développer
  ✅ Parfait pour hardware contraint
  ✅ Procédurale = rejouabilité infinie
  ✅ JARVIS pourrait générer les niveaux via AI !
  ✅ Unique : un roguelike QUI TOURNE DANS L'OS
```

#### Option B : **Raycaster Avancé (Doom-like)**
```
- Style : FPS rétro, extension du game3d.rs existant
- Genre : Action/exploration
- Résolution interne : 320×200 (upscalé)
- Technique : Raycasting existant + sprites billboardés
- Avantages :
  ✅ Base existante (game3d.rs)
  ✅ Esthétique rétro populaire
  ❌ Plus complexe (BSP, clipping, etc.)
```

#### Option C : **Platformer 2D Style SNES**
```
- Style : Scrolling horizontal/vertical, tiles + sprites
- Genre : Action/platformer
- Résolution interne : 256×224 (résolution SNES)
- Technique : Tilemap scrolling + sprite system
- Avantages :
  ✅ Simple et fun
  ✅ Bon showcase des capacités graphiques
  ❌ Nécessite beaucoup d'assets (art)
```

#### Option D : **Space Shooter Vectoriel** (style Another World)
```
- Style : Rendu vectoriel/polygonal minimaliste
- Genre : Shoot'em up spatial
- Résolution interne : 320×200
- Technique : Rasterisation de polygones simples + palette tricks
- Avantages :
  ✅ Très peu d'assets nécessaires
  ✅ Style visuel unique
  ✅ Rapide même sans GPU
  ✅ Effets de palette (flash, fade, transparence)
```

### Ma Recommandation Personnelle

**Option A (Roguelike) + éléments d'Option D (vectoriel).**

Un roguelike avec un style visuel unique "TrustOS" :
- Tilemap pour le donjon
- Sprites pour les entités
- Effets vectoriels/polygonaux pour les sorts, explosions, boss
- JARVIS intégré comme "oracle" dans le jeu (le joueur peut parler à JARVIS pour des indices)
- Génération procédurale des niveaux via l'AI de JARVIS
- Le jeu reflète les thèmes de TrustOS : cybersécurité, IA, réseau mesh

---

## 6. ARCHITECTURE PROPOSÉE DU GAME ENGINE

### Structure de Fichiers Proposée
```
kernel/src/game_engine/
├── mod.rs              // Module principal + game loop
├── sprite.rs           // Sprite renderer (batched, palette-based)
├── tilemap.rs          // Tilemap renderer (scrolling, layers)
├── collision.rs        // AABB + grid spatial partitioning
├── entity.rs           // Entity pool (fixed-size, no alloc)
├── input.rs            // Input abstraction + mapping
├── animation.rs        // Sprite animation (frame sequences)
├── camera.rs           // Camera 2D (scroll, zoom, shake)
├── audio.rs            // Channel-based audio mixer
├── particle.rs         // Simple particle system
└── procgen.rs          // Procedural generation (JARVIS-powered?)
```

### Game Loop Architecture
```
┌─────────────────────────────────────────────┐
│               GAME LOOP (60 Hz)             │
│                                             │
│  ┌─────────┐   ┌──────────┐   ┌─────────┐  │
│  │  INPUT   │──▶│  UPDATE  │──▶│ RENDER  │  │
│  │ (poll    │   │ (fixed   │   │ (dirty  │  │
│  │  keys)   │   │  step)   │   │  rects) │  │
│  └─────────┘   └──────────┘   └─────────┘  │
│       │              │              │        │
│       ▼              ▼              ▼        │
│  Keyboard      Entity Pool     Tilemap      │
│  + Mouse       Collision       Sprites      │
│  + Gamepad     Physics         UI/HUD       │
│               AI (JARVIS?)     Particles    │
│                                             │
│  ┌──────────────────────────────────────┐   │
│  │        PRESENT (parallel copy)       │   │
│  │    Back Buffer ──▶ Front Buffer      │   │
│  │       (SSE2 non-temporal stores)     │   │
│  └──────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### Budgets Performance (cibles)

| Opération | Budget | Technique |
|-----------|--------|-----------|
| Input polling | < 0.1 ms | Direct port read |
| Game logic | < 3 ms | Fixed entities, no alloc |
| Collision detection | < 1 ms | Spatial grid |
| Tilemap render | < 2 ms | Dirty tiles only |
| Sprite render | < 2 ms | Batched by layer |
| UI/HUD | < 1 ms | Cached text |
| Buffer present | < 4 ms | SSE2 parallel copy |
| **TOTAL** | **< 13 ms** | **= 75+ FPS** |

### Optimisations à Implémenter en Priorité

1. **Sprite Engine** avec palette indexée et batching par layer
2. **Tilemap Renderer** avec scrolling par offset (pas de redraw complet)
3. **Fixed Timestep** game loop avec interpolation de rendu
4. **Entity Pool** statique (zéro allocation post-init)
5. **Spatial Grid** pour la détection de collision
6. **Resolution Scaling** dynamique si le FPS descend

---

## CONCLUSION

Les 50 ans d'histoire des consoles nous enseignent une vérité fondamentale :

> **Les meilleurs jeux ne viennent pas du meilleur hardware, mais de la meilleure utilisation du hardware disponible.**

- La NES avec 2 KB de RAM a donné Super Mario Bros
- La Game Boy avec un écran 4 nuances de vert a donné Pokémon
- La Wii avec le GPU le plus faible a vendu plus que PS3 et Xbox 360
- Another World tournait sur un Amiga 500 avec 6000 lignes d'assembleur

TrustOS a déjà un moteur graphique remarquablement complet. Ce qu'il nous faut maintenant, c'est:
1. Un **sprite/tilemap engine** dédié au jeu
2. Une **game loop** robuste à timestep fixe
3. Un **concept de jeu unique** qui exploite JARVIS
4. Le courage de garder les specs modestes et le gameplay maximal

**Le prochain pas :** choisir un concept (je recommande le roguelike cyberpunk TrustOS) et commencer le prototype du game engine.

---

*"La créativité naît de la contrainte."* — À graver dans le silicon de TrustOS.
