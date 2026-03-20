# WOA — Development Tracker

> Suivi en temps réel du développement. Mis à jour à chaque session.
> Checkboxes : `[x]` = fait, `[ ]` = à faire, `[~]` = en cours

---

## Phase 0 — "Le Pixel Qui Bouge" ✅
> Engine foundations. Objectif : un carré qui bouge à 60fps. **VALIDÉ 20/03/2026**

### Infrastructure
- [x] Créer `kernel/src/woa/mod.rs` + feature gate `woa` dans Cargo.toml
- [x] Module `woa::engine` — game loop (fixed timestep 60fps, TSC-based)
- [x] Module `woa::renderer` — draw primitives sur framebuffer (rect, sprite, tile)
- [x] Module `woa::input` — keyboard state polling (PS/2 scancode bitmap)
- [x] Module `woa::camera` — viewport 1280×800 sur world coords
- [x] Module `woa::sprites` — sprite data + flip/flash/tint utilities
- [x] Module `woa::effects` — particle system (wind, dust, halo)
- [x] Sprite PixelLab 128×128 militant ant (PNG → Rust const array)
- [x] Résolution interne 1280×800 (1:1 framebuffer natif)

### Validation P0
- [x] Carré 16×16 se déplace avec ZQSD/WASD à 60fps
- [x] Pas de flicker (double buffer + internal 1280×800 native)
- [x] FPS counter affiché (barre verte en haut à droite)
- [x] `serial_println!` pour debug sans ralentir le frame
- [x] Sprite PixelLab fidèle, proportionné correctement (128×128 sur 1280×800)
- [x] Effets particules (jump=vent, slide=poussière, double-jump=halo)

### Decision Checkpoint — "Est-ce que ça tourne ?"
> ✅ 60fps stable, mouvement fluide, double buffer OK → **GO Phase 1**

---

## Phase 1 — "La Fourmi Qui Saute" 🟡
> Platformer core. Objectif : une fourmi qui court et saute.

### Platformer Core
- [ ] Module `woa::physics` — gravity, velocity, acceleration
- [ ] Module `woa::collision` — AABB tilemap collision
- [ ] Player sprite 16×16 (placeholder rect → real sprite later)
- [ ] Mouvement horizontal (accélération/décélération, speed cap 2.5px/f)
- [ ] Saut (variable height: min 2.5 tiles, max 4 tiles)
- [ ] Coyote time (6 frames)
- [ ] Jump buffer (6 frames)
- [ ] Wall slide + wall jump
- [ ] Crouch + crouch walk

### Validation P1
- [ ] Mouvement fluide, responsive
- [ ] Collision solide (pas de clip through)
- [ ] Saut variable height fonctionne
- [ ] Wall interactions correctes

---

## Phase 2 — "Le Monde Existe" ⬜
> Zone 1 complète (Garden Floor). Objectif : un niveau jouable.

### Tilemap & Zone 1
- [ ] Module `woa::tilemap` — tile storage, rendering, layers
- [ ] Tileset Zone 1: 16 tiles minimum (sol, mur, plateforme, hazard, déco)
- [ ] Level editor data format (simple binary or const arrays)
- [ ] Zone 1 layout — 3 rooms minimum avec parkour
- [ ] Semi-solid platforms (jump through from below)
- [ ] Hazards (eau = instant kill, épines = damage)
- [ ] Breakable platforms (timer-based)
- [ ] Parallax background (2 layers minimum)
- [ ] Camera follow + screen boundaries

### Validation P2
- [ ] Zone 1 traversable du début à la fin
- [ ] Hazards fonctionnels
- [ ] Visuellement lisible (tiles distinguables)

---

## Phase 3 — "Le Combat Existe" ⬜
> Turn-based combat sur grille. Objectif : un combat basique qui fonctionne.

### Combat System
- [ ] Module `woa::combat` — state machine (explore ↔ combat)
- [ ] Module `woa::grid` — grille 15×10, cell types (sol, obstacle, vide)
- [ ] Transition explore → combat (trigger zones ou mob contact)
- [ ] Tour par tour : AP system (6 AP), mouvement (1 AP/cell), timer 15s
- [ ] Attaque de base (1 caste : Militant)
- [ ] Module `woa::stats` — HP, AP, MP, For, Int, Agi, Vit, Cha
- [ ] 5 mobs Zone 1 (placeholder sprites, stats de 07_MOBS)
- [ ] Formules dégâts de base (voir 04_COMBAT §Formules)
- [ ] Mort du mob → loot drop (placeholder)
- [ ] Mort du joueur → game over screen

### Validation P3
- [ ] Combat 1v1 fonctionnel
- [ ] AP dépensés correctement
- [ ] Dégâts cohérents avec formules
- [ ] Transition combat ↔ explore fluide

---

## Phase 4 — "Le Combat A Du Poids" ⬜
> Juice + premier sort. Objectif : le combat est satisfaisant.

### Juice & Feel
- [ ] Screen shake on hit
- [ ] Flash blanc on damage
- [ ] Knockback (1 cell push)
- [ ] Particules : impact, mort, level up
- [ ] SFX placeholder (beeps via TrustSynth)
- [ ] Damage numbers popup
- [ ] Turn transition animation

### Militant Spells (20 sorts)
- [ ] 3 specs (Berserker, Tank, Gladiator) — active unlock
- [ ] Tier I sorts (level 1-4)
- [ ] Tier II sorts (level 5-8)
- [ ] Spell targeting UI (AoE preview, range highlight)

### Decision Checkpoint #1 — "Est-ce fun ?"
> Jouer 10 combats. Si fun → Phase 5. Sinon → ajuster.

---

## Phase 5 — "J'ai Du Loot" ⬜
> Gear system D2-style. Objectif : trouver du loot excitant.

- [ ] Module `woa::items` — item struct, 10 slots, rarity
- [ ] Module `woa::inventory` — equip/unequip, inventory grid
- [ ] Drop tables Zone 1 (Normal, Magique, Rare)
- [ ] Affixes random (prefix + suffix pools)
- [ ] Inventory UI (13_UI_SCREENS layout)
- [ ] Stats recalc on equip
- [ ] Item tooltip (stats + affixes)
- [ ] Loot beam colors par rareté

---

## Phase 6 — "La Fourmilière" ⬜
> Hub + roguelike loop. Objectif : boucle complète.

- [ ] Module `woa::hub` — hub tilemap, NPC placement
- [ ] Boucle : Hub → Explore Z1 → Combat → Loot → Return Hub
- [ ] 4 NPCs minimum (Queen, Mandibula, Sage, Vendor)
- [ ] Stash (shared entre runs)
- [ ] Sugar + Scrap economy
- [ ] Save/Load system (VFS or fixed memory)
- [ ] Death penalty (lose carried items, keep stash)

### Decision Checkpoint #2 — "La boucle est addictive ?"

---

## Phase 7 — "8 Castes" ⬜
> Toutes les classes. Objectif : 8 castes jouables.

- [ ] 7 castes restantes (Hydrant → Tyrant)
- [ ] Passifs de caste
- [ ] Caste selection screen
- [ ] Différences de stats de base

### Decision Checkpoint #3 — "Chaque caste se sent unique ?"

---

## Phase 8 — "BOSS!" ⬜
> Premier boss + zones 2-3. Objectif : un boss épique.

- [ ] Boss Zone 1 (Aphid Overlord) — 2 phases
- [ ] Zone 2 (Compost Heap) — tileset + 5 mobs + boss
- [ ] Zone 3 (Forest Canopy) — tileset + 5 mobs + boss + liane swing

---

## Phase 9 — "Le Spell Book" ⬜
> 160 sorts. Objectif : chaque sort a un impact unique.

- [ ] 160 sorts implémentés (20 × 8 castes)
- [ ] Spell book UI
- [ ] Tier II/III unlock system
- [ ] Balance pass

### Decision Checkpoint #4 — "Les specs sont distinctes ?"

---

## Phase 10 — "Le Monde Complet" ⬜
> Zones 4-7 + endgame. Objectif : monde complet.

- [ ] Zone 4 (Rotten Log) + boss
- [ ] Zone 5 (Riverbed) + boss
- [ ] Zone 6 (Underground) + boss
- [ ] Zone 7 (Spider Nest) + Spider Queen final boss
- [ ] Corrupted Shard system
- [ ] NG+ difficulty scaling

### Decision Checkpoint #5 — "Le monde est cohérent ?"

---

## Phase 11 — "Le Loot Parfait" ⬜
> Uniques, Sets, Mythiques, Cosmic. Objectif : chaque drop compte.

- [ ] 35 Unique items (5/zone)
- [ ] 3 Set Légendaires
- [ ] 7 Mythiques (boss drops + slot 8)
- [ ] Cosmic items (1/1B chance)
- [ ] Magic Find system
- [ ] Reroll station (Corrupted Shards)

---

## Phase 12 — "Polish & Feel" ⬜
> Art final + audio + UI. Objectif : qualité pro.

- [ ] Sprites finaux (tous les mobs, player, NPCs)
- [ ] Tilesets finaux (7 zones)
- [ ] Musique (Suno AI → TrustSynth integration)
- [ ] SFX complets
- [ ] UI polish (transitions, animations)
- [ ] Accessibility (contrasts, key rebind)

---

## Phase 13 — "Le Hub Vivant" ⬜
> NPC content complet. Objectif : immersion totale.

- [ ] Tous les NPCs (dialogues, quêtes)
- [ ] Achievements (93)
- [ ] Titles (22)
- [ ] Easter eggs
- [ ] Credits

---

## Stats Globales

| Métrique | Valeur |
|----------|--------|
| Phase actuelle | 0 (infra done, validation pending) |
| Dernière session | 2026-03-20 |
| Modules créés | 5 (mod, engine, renderer, input, camera) |
| Lignes de code WOA | ~380 |
| Sprites finaux | 0 / ~350 |
| Sorts implémentés | 0 / 160 |
| Mobs implémentés | 0 / 35 |
| Zones jouables | 0 / 7 |

---

*Mis à jour automatiquement à chaque session de dev.*
