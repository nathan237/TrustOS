# PRISM-LUMEN - Contexte prepare pour Codex

Mise a jour: 2026-05-25

## Reprise rapide

PRISM-LUMEN est un nouveau produit autonome dans:

```text
D:\TrustOS_SafeMirror\Documents_Scripts\OSrust\products\prism-lumen
```

Il ne fait pas partie de T-Wookie et ne doit pas etre traite comme un protocole
de communication ou de cryptographie. L'idee produit est un affichage optique
simule: une CRT virtuelle ou le signal pilote des faisceaux, des trajectoires,
des prismes et une surface phosphore persistante.

## Intuition fondatrice de Nathan

Utiliser le ray tracing comme logique d'affichage elle-meme, un peu comme une
CRT virtuelle. Les pixels finaux ne sont pas simplement colories; ils sont le
resultat d'une lumiere simulee et de sa memoire sur une surface.

## Documents a lire en premier

1. `README.md`
2. `PRISM_LUMEN_SPEC.md`
3. `PLANNING.md`

Ne pas commencer par les anciens prototypes `tools/tpix`: ils concernent un
autre axe de recherche et risquent de ramener la session vers le transport de
donnees plutot que vers le renderer.

## Etat au demarrage

- Produit cree et documente; aucun code de renderer n'est encore implemente.
- Direction recommandee: prototype Web local, plein ecran, utilisant Three.js
  pour la scene et un petit modele CPU de reference pour les tests optiques.
- Le premier MVP est `CRT_BASELINE`, puis `PRISM_FIELD`.
- Les exigences de test initiales sont R01-R10 dans la spec.

## Premier ticket concret

Construire le squelette d'une application Web PRISM-LUMEN:

```text
- scene plein ecran sombre;
- surface phosphore rendue comme objet principal;
- spot CRT anime et balayage raster simple;
- controle persistance/intensite/profil/pause;
- aucun texte marketing ni integration T-Wookie;
- validation visuelle Playwright desktop + mobile.
```

Avant d'ajouter un prisme, prouver que le spot et la persistance forment
vraiment l'image et ne sont pas un simple filtre decoratif.

## Prompt de nouvelle session

```text
Nous demarrons PRISM-LUMEN comme produit autonome. Travaille dans:
D:\TrustOS_SafeMirror\Documents_Scripts\OSrust\products\prism-lumen

Lis README.md, PRISM_LUMEN_SPEC.md et PLANNING.md. PRISM-LUMEN n'est pas
T-Wookie et n'est pas TPIX: c'est un moteur d'affichage optique simule, une
CRT virtuelle ou des faisceaux et une surface phosphore produisent l'image.

Implemente le premier MVP Web CRT_BASELINE: scene plein ecran, spot/balayage,
persistance RGB mesurable et controles sobres. Utilise Three.js pour la scene
visible et garde un modele deterministe testable pour l'optique. Verifie le
rendu avec screenshots desktop/mobile avant de conclure.
```

## Regles de continuite

- Conserver ce produit independant de la messagerie.
- Ne pas presenter une metaphore lumineuse comme une preuve de securite.
- Mettre ce fichier a jour lorsque l'architecture, les tests ou l'etat du MVP
  changent.
