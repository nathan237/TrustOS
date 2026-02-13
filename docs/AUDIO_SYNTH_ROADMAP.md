# 🎹 TrustOS Audio Synthesizer — Roadmap

## Étude de référence : fonctionnalités standard des logiciels audio

Après recherche sur les DAW et éditeurs audio les plus utilisés (FL Studio, Audacity, LMMS, Ableton Live, GarageBand, Ardour), voici les fonctionnalités standard que tous ces logiciels fournissent à leurs utilisateurs :

### 🔊 Synthétiseur
| Fonctionnalité | FL Studio | LMMS | Audacity | Ableton | Notre cible |
|---|---|---|---|---|---|
| Oscillateurs multiples (sine, square, saw, triangle) | ✅ | ✅ (TripleOsc) | ✅ (génération) | ✅ | ✅ |
| Enveloppe ADSR (Attack/Decay/Sustain/Release) | ✅ | ✅ | ❌ | ✅ | ✅ |
| LFO (Low-Frequency Oscillator) | ✅ | ✅ (Monstro) | ❌ | ✅ | ✅ |
| FM Synthesis | ✅ | ✅ (OpulenZ) | ❌ | ✅ | ⬜ (v2) |
| Wavetable | ✅ (Serum) | ✅ (BitInvader) | ❌ | ✅ | ⬜ (v2) |
| Polyphonie (accords) | ✅ | ✅ | ❌ | ✅ | ✅ |
| Presets / patches | ✅ | ✅ | ❌ | ✅ | ✅ |

### 🎛️ Filtres & Effets
| Fonctionnalité | Standard? | Description | Notre cible |
|---|---|---|---|
| Low-pass filter (LPF) | ✅ Universal | Coupe les fréquences hautes | ✅ Phase 4 |
| High-pass filter (HPF) | ✅ Universal | Coupe les fréquences basses | ✅ Phase 4 |
| Band-pass filter | ✅ Standard | Laisse passer une bande de fréquences | ✅ Phase 4 |
| Résonance (Q factor) | ✅ Standard | Amplifie autour de la fréquence de coupure | ✅ Phase 4 |
| Reverb | ✅ Universal | Simule l'écho d'un espace clos | ⬜ Phase 5 |
| Delay/Echo | ✅ Universal | Répétition temporisée du signal | ✅ Phase 4 |
| Distortion | ✅ Standard | Saturation du signal (clipping, overdrive) | ✅ Phase 4 |
| Volume/Gain | ✅ Universal | Amplification du signal | ✅ Phase 2 |
| Fade in/out | ✅ Universal | Volume progressif | ✅ Phase 3 |
| Tremolo | ✅ Standard | Modulation de volume par LFO | ✅ Phase 4 |
| Vibrato | ✅ Standard | Modulation de pitch par LFO | ✅ Phase 4 |
| Chorus | ✅ Standard | Doublage décalé du signal | ⬜ Phase 5 |
| Equalizer (EQ) | ✅ Universal | Ajustement par bandes de fréquences | ⬜ Phase 5 |
| Compressor | ✅ Standard | Réduit la dynamique | ⬜ Phase 5 |
| Noise gate | ✅ Standard | Coupe en dessous d'un seuil | ⬜ Phase 5 |

### 🔁 Loop & Recording
| Fonctionnalité | Standard? | Description | Notre cible |
|---|---|---|---|
| Pattern editor | ✅ (FL Studio) | Séquencer de notes en grille | ✅ Phase 2 |
| Loop playback | ✅ Universal | Lecture en boucle d'un pattern | ✅ Phase 2 |
| Loop recording | ✅ Standard | Enregistrer pendant la lecture en boucle | ✅ Phase 3 |
| Overdub | ✅ Standard | Ajouter des couches sans effacer | ✅ Phase 3 |
| Undo / Redo | ✅ Universal | Annuler les dernières actions | ⬜ Phase 5 |

### 🎵 Track Manager
| Fonctionnalité | Standard? | Description | Notre cible |
|---|---|---|---|
| Multi-track | ✅ Universal | Plusieurs pistes simultanées | ✅ Phase 3 |
| Solo / Mute par piste | ✅ Universal | Isoler ou couper une piste | ✅ Phase 3 |
| Volume par piste | ✅ Universal | Mixeur avec faders individuels | ✅ Phase 3 |
| Pan (gauche/droite) | ✅ Universal | Panoramique stéréo | ⬜ Phase 5 |
| Timeline / arrangement | ✅ Standard | Vue temporelle des patterns | ⬜ Phase 5 |
| BPM / Tempo | ✅ Universal | Battements par minute | ✅ Phase 2 |

### 📋 Fonctionnalités UI / UX
| Fonctionnalité | Standard? | Notre cible |
|---|---|---|
| Piano roll / clavier visuel | ✅ Universal | ✅ Phase 2 |
| Waveform display | ✅ Universal | ✅ Phase 3 |
| Spectrum analyzer (FFT) | ✅ Standard | ⬜ Phase 5 |
| VU Meter (peak meter) | ✅ Universal | ✅ Phase 3 |
| Transport (Play/Stop/Rec) | ✅ Universal | ✅ Phase 2 |

---

## État actuel de TrustOS Audio

### Ce qu'on a déjà ✅
- **Intel HDA Driver** (~620 lignes) — contrôleur initialisé, CORB/RIRB, codec discovery
- **Triangle wave tone generator** — `fill_tone(freq, duration)` 
- **Commandes shell** — `beep [freq] [ms]`, `audio [init|status|stop|test]`
- **Gamme de test** — C4→C5 scale via `audio test`
- **Format audio** — 48 kHz, 16-bit, stereo, DMA via BDL

### Ce qu'il faut construire 🔧
Un **TrustSynth** — synthétiseur + loop recorder + track manager complet dans le shell

---

## Roadmap détaillée

### Phase 1 — Moteur de synthèse (Oscillateurs) 🎵
**Durée estimée : 3-4 jours | ~600 lignes**

Remplacer le triangle wave naïf par un vrai moteur de synthèse multi-forme d'onde.

#### 1.1 — Oscillateurs de base
```
Waveform enum : Sine, Square, Saw, Triangle, Noise
```
- **Sine** : table de sinus 256 entrées (lookup integer, pas de libm)
- **Square** : seuil sur compteur de phase (+max / -max)
- **Sawtooth** : rampe linéaire
- **Triangle** : rampe aller-retour (on l'a déjà, à nettoyer)
- **White Noise** : LFSR 16-bit (Linear Feedback Shift Register)
- Phase accumulator pattern (increment = freq × table_size / sample_rate)

#### 1.2 — Enveloppe ADSR
```rust
struct Envelope {
    attack_ms: u32,    // Temps de montée (0→max)
    decay_ms: u32,     // Temps de descente (max→sustain)
    sustain_level: u16, // Niveau soutenu (0-32767)
    release_ms: u32,   // Temps de release (sustain→0)
    state: EnvState,   // Idle, Attack, Decay, Sustain, Release
    current: u16,      // Valeur actuelle
}
```

#### 1.3 — Notes & fréquences
- Table MIDI note → fréquence (128 entrées, integer Hz)
- `Note` struct avec pitch, velocity, duration
- Commande shell : `synth note C4 500` / `synth note A#3 1000`

#### Référence mathématique
```
Phase increment = (frequency × TABLE_SIZE) / SAMPLE_RATE
Sample = waveform_table[phase >> FRAC_BITS] × envelope × velocity / 127
```

---

### Phase 2 — Pattern Editor & Loop Playback 🔁
**Durée estimée : 4-5 jours | ~800 lignes**

#### 2.1 — Séquenceur de patterns
```rust
struct Pattern {
    name: [u8; 16],
    steps: Vec<Step>,      // 16, 32 ou 64 steps
    bpm: u16,              // 60-300 BPM
    length: u8,            // Nombre de steps
}

struct Step {
    note: Option<u8>,      // MIDI note (0-127), None = silence
    velocity: u8,          // 0-127
    waveform: Waveform,
}
```

#### 2.2 — Moteur de lecture
- **Tick engine** : calcul du temps par step depuis le BPM
  - `step_duration_samples = (60 × SAMPLE_RATE) / (BPM × steps_per_beat)`
- Double buffering : pendant qu'un buffer joue via DMA, on génère le suivant  
- Loop : quand on arrive au dernier step, retour au premier

#### 2.3 — Interface shell Pattern Editor
```
synth pattern new <name> [16|32|64]    — Créer un pattern
synth pattern edit <name>              — Mode édition interactif
synth pattern play <name>              — Jouer en boucle
synth pattern stop                     — Arrêter
synth pattern list                     — Lister les patterns
synth pattern bpm <60-300>             — Changer le tempo
```

#### 2.4 — Édition interactive (Piano Roll simplifié)
Vue texte dans le terminal :
```
 Step: 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16
 Note: C4 -- E4 -- G4 -- C5 -- C4 -- E4 -- G4 -- C5 --
 Vel:  7F -- 60 -- 7F -- 50 -- 7F -- 60 -- 7F -- 50 --
 Wave: Sq -- Sn -- Sq -- Tr -- Sq -- Sn -- Sq -- Tr --
 ▶ Playing... [Step 05] BPM: 120
```
- Navigation : flèches gauche/droite (ou `h`/`l`)
- Modification : taper la note (C4, D#5, etc.)
- Waveform : `s` = sine, `q` = square, `w` = saw, `t` = triangle, `n` = noise

#### 2.5 — Presets intégrés
Quelques patterns pré-programmés pour la démo :
- **"techno"** — kick pattern 4/4 basique
- **"arp"** — arpège C mineur
- **"bass"** — ligne de basse saw
- **"chiptune"** — mélodie 8-bit style

---

### Phase 3 — Multi-Track Manager & Loop Recording 🎚️
**Durée estimée : 5-6 jours | ~1000 lignes**

#### 3.1 — Architecture multi-piste
```rust
struct TrackManager {
    tracks: Vec<Track>,     // Max 8 pistes
    master_bpm: u16,
    master_volume: u8,      // 0-255
    playing: bool,
    recording_track: Option<usize>,
    mix_buffer: Vec<i16>,   // Buffer de mixage stéréo
}

struct Track {
    name: [u8; 16],
    pattern: Option<usize>, // Index du pattern assigné
    volume: u8,             // 0-255
    muted: bool,
    solo: bool,
    waveform: Waveform,     // Override waveform pour la piste
}
```

#### 3.2 — Mixeur audio
- Mix N pistes dans un buffer master : somme pondérée + clipping
- `master[i] = clamp(Σ track[j].sample[i] × track[j].volume / 255, -32768, 32767)`
- Solo : si une piste est en solo, seules les pistes solo sont mixées
- Mute : la piste muted n'est pas incluse dans le mix

#### 3.3 — Loop Recording
- Mode record : pendant que les pistes existantes jouent en loop, les notes entrées au clavier sont enregistrées sur la piste active
- Overdub : les nouvelles notes s'ajoutent aux existantes
- Quantization : recalage des notes sur le step le plus proche

#### 3.4 — Interface Track Manager
```
track list                    — Afficher toutes les pistes
track add <name>             — Ajouter une piste
track remove <n>             — Supprimer la piste #n
track assign <n> <pattern>   — Assigner un pattern à la piste
track volume <n> <0-255>     — Volume de la piste
track mute <n>               — Toggle mute
track solo <n>               — Toggle solo
track rec <n>                — Enregistrer sur la piste

mix play                     — Jouer toutes les pistes
mix stop                     — Arrêter
mix volume <0-255>           — Volume master
mix status                   — Afficher le mixeur
```

#### 3.5 — Affichage mixeur (VU Meters)
```
 ┌─────────────────────────── TrustMixer ───────────────────────────┐
 │  Track 1: "kick"     [████████░░] Vol:200 ♪ Pattern: techno     │
 │  Track 2: "bass"     [██████░░░░] Vol:180   Pattern: bass       │
 │  Track 3: "lead"     [████░░░░░░] Vol:150 M Pattern: arp        │
 │  Track 4: "drums"    [███████░░░] Vol:220 S Pattern: techno     │
 │─────────────────────────────────────────────────────────────────  │
 │  Master:             [█████████░] Vol:240   BPM: 128            │
 │  ▶ Playing | Loop ON | Step 12/16 | Rec: Track 2                │
 └──────────────────────────────────────────────────────────────────┘
```

---

### Phase 4 — Filtres & Effets audio 🎛️
**Durée estimée : 4-5 jours | ~700 lignes**

#### 4.1 — Filtres numériques (biquad)
Implémentation d'un filtre biquad integer (pas de float) :
```rust
struct BiquadFilter {
    filter_type: FilterType,  // LPF, HPF, BPF, Notch
    cutoff_hz: u32,
    resonance: u16,           // Q factor × 1000
    // Coefficients (integer scaled)
    a0: i32, a1: i32, a2: i32,
    b1: i32, b2: i32,
    // State
    x1: i32, x2: i32,
    y1: i32, y2: i32,
}
```

**Algorithme (adapté pour integer) :**
- Low-pass : $H(z) = \frac{b_0 + b_1 z^{-1} + b_2 z^{-2}}{a_0 + a_1 z^{-1} + a_2 z^{-2}}$
- Utilise fixed-point Q15 (×32768) pour les coefficients
- Pas de libm nécessaire : approximation polynomiale de sin/cos pour calculer les coefficients

#### 4.2 — LFO (Low-Frequency Oscillator)
```rust
struct LFO {
    waveform: Waveform,      // Sine, Triangle, Square
    rate_hz: u32,            // 0.1 Hz - 20 Hz (×10 pour integer)
    depth: u16,              // 0-32767
    target: LFOTarget,       // Pitch, Volume, Filter Cutoff
    phase: u32,
}
```
- Tremolo = LFO → Volume
- Vibrato = LFO → Pitch
- Wah-wah = LFO → Filter cutoff

#### 4.3 — Effets temporels
- **Delay** : buffer circulaire de N samples, feedback configurable
  ```rust
  struct DelayEffect {
      buffer: Vec<i16>,
      write_pos: usize,
      delay_ms: u32,
      feedback: u16,  // 0-32767 (0-100%)
      mix: u16,       // Dry/Wet ratio
  }
  ```
- **Distortion** : soft clipping avec seuil configurable
  ```rust
  fn distort(sample: i16, drive: u16) -> i16 {
      let amplified = (sample as i32 * drive as i32) >> 8;
      clamp(amplified, -32768, 32767) as i16
  }
  ```

#### 4.4 — Chaîne d'effets par piste
```rust
struct EffectChain {
    effects: Vec<Effect>,  // Max 4 effets en série
}

enum Effect {
    Filter(BiquadFilter),
    LFO(LFO),
    Delay(DelayEffect),
    Distortion { drive: u16 },
    Gain { level: u16 },
}
```

#### 4.5 — Commandes shell
```
fx add <track> lpf <cutoff> [q]    — Ajouter un low-pass filter
fx add <track> hpf <cutoff> [q]    — Ajouter un high-pass filter
fx add <track> delay <ms> [feedback]
fx add <track> dist <drive>
fx add <track> tremolo <rate> <depth>
fx add <track> vibrato <rate> <depth>
fx list <track>                     — Lister les effets
fx remove <track> <n>               — Supprimer un effet
fx bypass <track>                   — Bypass tous les effets
```

---

### Phase 5 — Polish & Fonctionnalités avancées ✨
**Durée estimée : 5-7 jours | ~800 lignes**

#### 5.1 — Reverb (Schroeder)
Implémentation simplifiée avec 4 comb filters + 2 allpass filters :
- Peu coûteux en mémoire (~64KB de delay lines)
- Résultat convaincant pour un OS bare-metal

#### 5.2 — Chorus
- 2-3 delay lines modulées par LFO à des rates légèrement différentes
- Mix avec le signal dry

#### 5.3 — Equalizer 3 bandes
- Low shelf, Mid peak, High shelf
- 3 filtres biquad en série

#### 5.4 — Stéréo Pan
- Pan law : `left = sample × cos(pan × π/2)`, `right = sample × sin(pan × π/2)`
- Approximation integer pour cos/sin

#### 5.5 — Undo / Redo
- Stack d'états pour les modifications de pattern
- 16 niveaux d'undo

#### 5.6 — Export / Save (optionnel)
- Sauvegarder les patterns en format binaire sur le disque virtuel
- Export WAV brut (header + PCM data)

#### 5.7 — Spectrum Analyzer (FFT)
- FFT 256 points integer (Cooley-Tukey radix-2)
- Affichage barres verticales dans le terminal

---

## Résumé global

| Phase | Nom | Lignes ~est. | Durée ~est. | Prérequis |
|-------|-----|-------------|-------------|-----------|
| **1** | Moteur de synthèse (Oscillateurs + ADSR) | ~600 | 3-4 jours | Driver HDA ✅ |
| **2** | Pattern Editor & Loop Playback | ~800 | 4-5 jours | Phase 1 |
| **3** | Multi-Track Manager & Loop Recording | ~1000 | 5-6 jours | Phase 2 |
| **4** | Filtres & Effets (LPF/HPF/Delay/Dist/LFO) | ~700 | 4-5 jours | Phase 1 |
| **5** | Polish (Reverb/Chorus/EQ/Pan/FFT) | ~800 | 5-7 jours | Phase 3+4 |
| **Total** | | **~3900 lignes** | **~21-27 jours** | |

### Architecture des fichiers
```
kernel/src/
├── drivers/
│   └── hda.rs                  # Intel HDA controller (existant, ~620 lignes)
│
├── audio/
│   ├── mod.rs                  # Module audio principal
│   ├── synth.rs                # Oscillateurs + ADSR + Note engine (Phase 1)
│   ├── pattern.rs              # Pattern editor + séquenceur (Phase 2)
│   ├── mixer.rs                # Track manager + mix engine (Phase 3)
│   ├── effects.rs              # Filtres + LFO + Delay + Distortion (Phase 4)
│   └── tables.rs               # Tables de sinus, fréquences MIDI, etc.
│
└── shell.rs                    # Commandes: synth, pattern, track, mix, fx
```

### Commande unifiée (TrustSynth)
```
synth                           — Ouvrir le mode synthétiseur interactif
synth note <note> [ms] [wave]  — Jouer une note (ex: synth note C#4 500 saw)
synth wave <type>               — Changer la waveform (sine/square/saw/tri/noise)
synth adsr <a> <d> <s> <r>    — Configurer l'enveloppe

pattern new <name> [steps]     — Nouveau pattern
pattern edit <name>            — Éditeur interactif
pattern play <name>            — Jouer un pattern en boucle
pattern bpm <60-300>           — Tempo

track list | add | remove | volume | mute | solo | rec
mix play | stop | volume | status

fx add <track> <effect> [params...]
fx list | remove | bypass
```

---

## Choix techniques importants

### Pourquoi pas de `f32`/`f64` ?
TrustOS est en `x86_64-unknown-none` — on n'a **pas de runtime FPU** garanti dans notre kernel.  
Tout le DSP est en **integer / fixed-point** :
- Coefficients en Q15 (×32768)
- Phase accumulators en Q16.16
- Lookup tables pour sin/cos (256 entrées)
- Pas de `libm`, pas de `#![feature(core_intrinsics)]` pour les floats

### Pourquoi max 8 pistes ?
- Chaque piste génère des samples en temps réel
- Le mix se fait avant envoi au DMA (pas de hardware mixer)
- 8 pistes × 48000 Hz × 2 ch × 2 bytes = ~1.5 MB/s de processing
- Raisonnable pour du TCG (émulation CPU dans QEMU)

### Références open-source utiles
| Projet | Langage | Intérêt |
|--------|---------|---------|
| LMMS TripleOscillator | C++ | Architecture synth multi-osc |
| ZynAddSubFX | C++ | ADSR, filtres biquad, effets |
| puredata (Pd) | C | DSP primitives, patch-based |
| Csound | C | Scoring language, oscillators |
| Ardour | C++ | Multi-track, mixer, transport |
| bytebeat | C | Algorithmic music minimale |

---

*Document créé le 12 février 2026 — TrustOS Audio Synthesizer Roadmap v1.0*
