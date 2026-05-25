# PRISM-LUMEN

PRISM-LUMEN est un produit de recherche et de rendu autonome: un affichage
virtuel inspire d'un tube cathodique, ou un signal devient lumiere simulee,
trajectoires optiques, excitation de phosphores et persistance temporelle.

Il ne fait pas partie de T-Wookie. T-Wookie reste une application de
communication; PRISM-LUMEN explore une nouvelle logique d'affichage qui pourra
eventuellement exposer une API ou produire des assets utilisables par d'autres
produits.

## Vision

Un ecran classique remplit directement des pixels. PRISM-LUMEN renderise plutot
un dispositif:

```text
signal d'entree
  -> canon / sources lumineuses virtuelles
  -> rayons, lentilles, prismes ou champs de deviation
  -> surface phosphore ou volume emissif
  -> persistance, bloom physique borne, balayage
  -> image affichee
```

L'interet n'est pas seulement esthetique. Le moteur peut rendre visibles des
structures de signal, de profondeur, de temps et d'interaction que la grille
RGBA ordinaire ecrase.

## Ce que le produit peut devenir

- Un renderer CRT virtuel temps reel pour interfaces et visualisations.
- Un laboratoire de phosphores, refraction, aberration, balayage et memoire
  lumineuse.
- Un canvas interactif ou la logique applicative module des faisceaux plutot
  que de dessiner des sprites directement.
- Un moteur d'affichage pour experiences TrustOS, installations ou jeux.

## Limites de perimetre

- Ce produit n'est pas un protocole de messagerie.
- Ce produit n'est pas une preuve cryptographique.
- Toute future modulation de donnees par la lumiere sera un module optionnel,
  documente separement et teste comme tel.

## Documents

- `PRISM_LUMEN_SPEC.md`: architecture initiale et modeles d'affichage.
- `PLANNING.md`: phases de developpement concret.
- `CONTEXTE_PREPARE.md`: reprise optimisee pour une nouvelle session Codex.
