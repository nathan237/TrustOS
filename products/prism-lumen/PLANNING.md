# PRISM-LUMEN - Plan de developpement

Mise a jour: 2026-05-25

## Phase 0 - Fondation

- [x] Separer PRISM-LUMEN de T-Wookie/TPIX.
- [x] Definir le produit comme renderer optique / CRT virtuelle.
- [ ] Choisir la pile du prototype Web et creer son squelette.

## Phase 1 - CRT virtuelle

- [ ] Creer une scene plein ecran avec surface phosphore.
- [ ] Implementer le modele de depot et persistance RGB.
- [ ] Implementer un balayage raster pilotant une mire test.
- [ ] Ajouter controles utiles: profil, intensite, persistance, pause, capture.
- [ ] Produire tests de reference R01-R03 et captures desktop/mobile.

## Phase 2 - Optique PRISM

- [ ] Implementer un rayon CPU de reference et les formes simples.
- [ ] Ajouter refraction vectorielle et reflexion totale bornee.
- [ ] Rendre un prisme visible et un mode dispersion RGB.
- [ ] Produire tests R04-R07.

## Phase 3 - Experience produit

- [ ] Definir les sources de signal exploitables: texture, formes, flux.
- [ ] Stabiliser le style visuel et l'ergonomie mobile.
- [ ] Mesurer performance et consommation.
- [ ] Explorer une API d'integration independante de T-Wookie.

## Non-objectifs du MVP

- Communication securisee ou messagerie.
- Encodage de payload dans des images.
- Promesse cryptographique.
- Integration mobile native avant validation du renderer Web.
