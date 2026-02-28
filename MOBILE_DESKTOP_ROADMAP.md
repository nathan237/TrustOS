# TrustOS Mobile Desktop — Plan de Match

## Vision
Transformer TrustOS en un OS mobile-first capable de tourner sur tablettes ARM (Pine64, Raspberry Pi), téléphones RISC-V (StarFive), et appareils x86 convertibles. Le desktop existant (COSMIC-style avec dock, fenêtres, animations) devient un desktop responsive qui s'adapte automatiquement au form factor.

---

## Phase 1 — Touch Input Pipeline (Semaines 1-2)

### 1.1 Pilote Touchscreen Générique
- **Fichier:** `kernel/src/touch.rs`
- Protocole: USB HID multitouch (les tablettes Pine64/RPi utilisent des contrôleurs USB I2C-HID)
- Structures:
  ```
  TouchPoint { id, x, y, pressure, state: Down|Move|Up }
  TouchState { points: [TouchPoint; 10], count, timestamp }
  ```
- Ring buffer lock-free (même pattern que `mouse.rs`)
- IRQ handler + polling fallback
- API: `get_touch_state() -> TouchState`

### 1.2 Gesture Recognizer
- **Fichier:** `kernel/src/gesture.rs`
- Gestes de base:
  | Geste | Détection | Action |
  |-------|-----------|--------|
  | Tap | contact <200ms, déplacement <10px | Click gauche |
  | Long press | contact >500ms, immobile | Click droit / context menu |
  | Swipe left/right | Δx > 50px, vitesse > 200px/s | Retour / Avancer |
  | Swipe up (depuis bas) | Y part du bord inférieur | Ouvrir App Launcher |
  | Swipe down (depuis haut) | Y part du bord supérieur | Panneau notifications |
  | Pinch (2 doigts) | Distance entre points change | Zoom in/out |
  | Two-finger scroll | 2 points, même direction | Scroll vertical/horizontal |
  | Three-finger swipe | 3 points horizontaux | Switch app (Alt+Tab) |
- State machine avec timeouts (via `arch::timestamp()`)
- Callback system: `on_gesture(Gesture) -> bool`

### 1.3 Intégration Touch → Desktop
- `desktop.rs` : ajouter `handle_touch()` à côté de `handle_click()`
- Mapper les gestes sur les actions existantes (déjà implémentées pour souris/clavier)

---

## Phase 2 — Responsive Layout Engine (Semaines 3-4)

### 2.1 Form Factor Detection
- **Fichier:** `kernel/src/gui/layout.rs`
- Détection automatique au boot:
  ```rust
  enum FormFactor { Phone, Tablet, Desktop, Convertible }
  
  fn detect_form_factor(width: u32, height: u32) -> FormFactor {
      let diagonal_in = ((w*w + h*h) as f64).sqrt() / dpi;
      match diagonal_in {
          d if d < 7.0  => Phone,
          d if d < 13.0 => Tablet,
          _             => Desktop,
      }
  }
  ```
- Supporter la rotation dynamique (portrait ↔ landscape)

### 2.2 Layout Adaptatif
Remplacer les constantes hardcodées par des calculs responsifs:

| Élément | Desktop (>1024px) | Tablette (600-1024px) | Phone (<600px) |
|---------|--------------------|-----------------------|----------------|
| Dock | Gauche, 60px | Bas, 56px, auto-hide | Bas, 48px, auto-hide |
| Taskbar | Bas, 40px | Masqué (geste swipe-up) | Masqué |
| Fenêtres | Libres, redimensionnables | Snappées moitié ou plein | Toujours plein écran |
| Title bar | 28px, boutons min/max/close | 36px, tactile-friendly | 44px, juste back + close |
| Marges tactiles | N/A | 44px min touch target | 48px min touch target |
| Start menu | Pop-up coin bas-gauche | Plein écran launcher | Plein écran launcher |

### 2.3 Système de Contraintes
```rust
struct LayoutConstraints {
    min_touch_target: u32,  // 44px minimum (Apple HIG / Material)
    safe_area: Insets,      // notch/barre status
    orientation: Orientation,
    scale_factor: f32,
}
```
- Intégrer avec le module `graphics/scaling.rs` existant (déjà 1x/2x/3x)
- Ajouter 1.5x pour les tablettes mid-DPI

---

## Phase 3 — Clavier Virtuel (Semaines 5-6)

### 3.1 On-Screen Keyboard (OSK)
- **Fichier:** `kernel/src/gui/virtual_keyboard.rs`
- Layouts:
  - QWERTY (défaut), AZERTY, numérique
  - Shift / symboles / emoji (page de base)
- Rendu: touches arrondies avec feedback visuel (highlight au tap)
- Hauteur: ~40% de l'écran en portrait, ~30% en landscape
- Animation: slide-up/down (réutiliser `WindowAnimation`)

### 3.2 Intégration
- Auto-show quand un champ texte reçoit le focus
- Auto-hide quand le focus change à un non-texte
- Ajuster le viewport: pousser le contenu vers le haut quand OSK visible
- Supporter les prédictions de texte basiques (dictionnaire en mémoire)

### 3.3 Input Method
- Connecter au `keyboard.rs` existant via `push_key()`
- Le reste du système (terminal, éditeur, browser URL bar) fonctionne sans modification

---

## Phase 4 — Mobile App Launcher & Navigation (Semaines 7-8)

### 4.1 App Launcher (remplace Start Menu sur mobile)
- Grille d'icônes plein écran (style iOS/Android launcher)
- 4 colonnes (phone) ou 6 colonnes (tablette)
- Pages avec swipe horizontal + indicateur de dots
- Barre de recherche en haut
- Dossiers d'apps (tap-and-hold pour réorganiser)

### 4.2 Navigation Mobile
- **Barre de navigation tactile** (bas de l'écran, 48px):
  - Bouton retour (triangle)
  - Bouton home (cercle) — revient au launcher
  - Bouton récents (carré) — app switcher en cartes
- **App Switcher**: Vue en cartes empilées (swipe gauche/droite pour naviguer, swipe haut pour fermer)
- **Geste pilule** (optionnel, style iPhone): swipe depuis le bord inférieur

### 4.3 Notifications
- Panneau slide-down depuis le bord supérieur
- Quick settings: Wi-Fi, Bluetooth, luminosité, volume, rotation lock
- Réutiliser le système `Toast` existant dans `gui/engine.rs`

---

## Phase 5 — Optimisations ARM/RISC-V (Semaines 9-10)

### 5.1 Accélération Graphique
- **NEON (aarch64)**: Remplacer les boucles pixel par pixel avec des intrinsics NEON
  - `fill_rect_neon()`: 4 pixels à la fois (128-bit)
  - `alpha_blend_neon()`: SIMD alpha compositing
  - Remplacer le `BLEND_TABLE` (64KB LUT) par du calcul NEON en temps réel
- **RVV (riscv64)**: RISC-V Vector Extension pour les mêmes ops (quand disponible)
- Détection runtime: `cfg(target_feature = "neon")` / feature probing

### 5.2 Framebuffer Optimisé
- DMA-backed framebuffer sur RPi4 (Mailbox interface)
- GPU rendering via V3D (RPi4) ou PanVK (Mali) — long terme
- Pour le moment: optimiser le `swap_buffers()` avec `memcpy` SIMD

### 5.3 Power Management
- CPU frequency scaling (DVFS) via device tree
- Display sleep timeout
- Suspend-to-RAM (PSCI sur ARM)
- Battery monitoring (fuel gauge I2C driver)

---

## Phase 6 — Capteurs & Connectivité Mobile (Semaines 11-12)

### 6.1 Capteurs
| Capteur | Interface | Usage |
|---------|-----------|-------|
| Accéléromètre | I2C/SPI | Rotation automatique |
| Gyroscope | I2C/SPI | UI inertie/momentum |
| GPS | UART/USB | Localisation |
| Luminosité | I2C ADC | Luminosité auto |
| Proximité | I2C | Éteindre écran à l'oreille |

### 6.2 Connectivité
- **Wi-Fi**: driver existant (`network.rs`) — ajouter scan/connect UI
- **Bluetooth**: HCI USB driver (stubs, puis implémentation)
- **Cellular**: QMI/MBIM USB modem (long terme)
- **NFC**: USB NFC reader (paiement, partage)

---

## Targets Matériel Prioritaires

| Appareil | Architecture | Écran | Priorité |
|----------|-------------|-------|----------|
| QEMU virt (test) | aarch64/riscv64 | Virtio GPU | P0 — dev/test |
| Raspberry Pi 4/5 | aarch64 | HDMI touch | P1 — premier vrai hardware |
| Pine64 PinePhone Pro | aarch64 | 720×1440 IPS touch | P1 — vrai mobile |
| StarFive VisionFive 2 | riscv64 | HDMI | P2 — RISC-V validation |
| Surface Go (x86) | x86_64 | 1800×1200 touch | P2 — convertible |
| PineTab2 | aarch64 | 1200×800 touch | P2 — tablette |

---

## Dépendances sur le Travail Existant

| Composant existant | Réutilisation mobile |
|--------------------|---------------------|
| `framebuffer.rs` double-buffering | Direct — fonctionne tel quel |
| `gui/engine.rs` dirty rects | Direct — essentiel pour perf mobile |
| `gui/engine.rs` alpha blend LUT | Remplacer par NEON/RVV sur ARM |
| `graphics/scaling.rs` 1x/2x/3x | Étendre avec 1.5x + DPI auto |
| `desktop.rs` fenêtres/animations | Adapter layout, garder logique |
| `mouse.rs` lock-free pattern | Template pour `touch.rs` |
| `arch/` abstraction layer | Base portable — déjà prêt |
| Stub modules (`stubs/`) | Permettent la compilation mobile |

---

## Métriques de Succès

- [ ] Touch input fonctionnel sur QEMU virt (virtio-input)
- [ ] 5 gestes de base reconnus (tap, long-press, swipe, pinch, 2-finger scroll)
- [ ] Desktop adaptatif sur 3 résolutions (1080p desktop, 800×1280 tablette, 720×1440 phone)
- [ ] Clavier virtuel fonctionnel dans le terminal
- [ ] 60 FPS maintenu sur RPi4 avec dirty rect optimization
- [ ] Boot-to-desktop en <5s sur RPi4
- [ ] App launcher grille fonctionnel avec navigation tactile
- [ ] Rotation portrait ↔ landscape via accéléromètre (ou bouton)

---

## Timeline Résumé

```
Semaines 1-2:  Touch Input Pipeline + Gesture Recognizer
Semaines 3-4:  Responsive Layout Engine + Form Factor Detection  
Semaines 5-6:  Virtual Keyboard + Input Integration
Semaines 7-8:  Mobile App Launcher + Navigation UI
Semaines 9-10: ARM/RISC-V Graphics Optimization + Power Management
Semaines 11-12: Sensors + Connectivity Drivers
```

**Prérequis accompli:** Support multi-architecture (x86_64 + aarch64 + riscv64) — compilant avec 0 erreurs.
