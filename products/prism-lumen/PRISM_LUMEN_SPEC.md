# PRISM-LUMEN - Spec initiale

Date: 2026-05-25
Statut: concept produit / pre-implementation

## 1. Definition

PRISM-LUMEN est un moteur d'affichage optique simule. Contrairement a un
renderer 2D qui assigne une couleur finale a chaque pixel, il calcule
l'apparition de la lumiere sur une surface receptrice apres emission,
deviation, refraction, depot d'energie et decroissance temporelle.

Le modele de depart est celui d'une CRT virtuelle enrichie:

```text
input signal -> beam emitter -> optical field -> phosphor surface -> camera
```

Le mot `PRISM` designe les transformations optiques. Le mot `LUMEN` designe
la lumiere comme primitive de rendu.

## 2. Difference avec TPIX et T-Wookie

TPIX est un prototype de transport par pixels. T-Wookie est une messagerie.
PRISM-LUMEN est un renderer. Il ne doit pas etre developpe comme une fonction
de securite ou comme un sous-module de chat.

Une future integration est possible seulement par interface claire:

```text
application -> texture / signal / evenements -> PRISM-LUMEN renderer
```

## 3. Pipeline d'affichage

### 3.1 Signal

Le signal d'entree peut etre:

- une image ou une texture;
- une liste de primitives lumineuses;
- une trame de balayage RGB;
- une carte de profondeur ou de normales;
- un flux temporel anime.

Le signal ne devient pas directement le framebuffer final. Il pilote le
dispositif virtuel.

### 3.2 Emetteur

Profil `CRT_BASELINE`:

```text
beam_origin: position normalisee derriere la surface
scan_mode: raster horizontal
beam_radius: largeur gaussienne du spot
beam_energy: intensite par sous-pixel RGB
scan_time: progression dans la frame
```

Profil `PRISM_FIELD`:

```text
emitters: une ou plusieurs sources
rays_per_sample: budget de simulation
dispersion: indice par longueur d'onde / canal RGB
geometry: prismes, lentilles ou surfaces refractives
```

### 3.3 Transport optique

Pour un rayon incident normalise `I`, une normale de surface `N` et le ratio
d'indices optiques `eta`, la refraction minimale utilise:

```text
cos_i = -dot(N, I)
k = 1 - eta^2 * (1 - cos_i^2)
T = eta * I + (eta * cos_i - sqrt(k)) * N     si k >= 0
```

Lorsque `k < 0`, le profil choisit entre reflexion totale et absorption.
La premiere implementation doit limiter les rebonds et utiliser des formes
simples: plan, triangle, sphere et prisme triangulaire.

### 3.4 Surface phosphore

La surface stocke de l'energie, pas une couleur instantanee:

```text
energy_rgb(x, y, t + dt) =
    energy_rgb(x, y, t) * exp(-dt / persistence_rgb)
    + deposits_rgb(x, y, t)
```

Parametres utiles:

- masque RGB: grille, aperture grille ou shadow mask;
- persistance differente par canal;
- courbe energie vers luminance;
- diffusion locale bornee;
- bruit et imperfection desactives par defaut.

### 3.5 Camera et sortie

La camera convertit l'energie simulee en image ecran:

- tone mapping;
- exposition;
- gamma/sRGB;
- echantillonnage final;
- eventuel bloom, strictement derive de l'energie lumineuse.

## 4. Profils initiaux

| Profil | Objet | Usage |
|--------|-------|-------|
| `CRT_BASELINE` | Balayage et phosphore sans optique complexe | Reference technique et performance |
| `CRT_COLOR_MASK` | Sous-pixels, masque et persistance RGB | Signature visuelle |
| `PRISM_FIELD` | Sources et refraction sur geometrie simple | Exploration vectorielle |
| `LIGHT_MEMORY` | Persistance longue et interactions temporelles | Visualisation / instrument |

## 5. Architecture logicielle proposee

Premiere preuve recommandee: application Web locale avec TypeScript et
Three.js/WebGL pour la scene visible, accompagnee d'un calcul CPU de reference
pour les tests deterministes.

```text
products/prism-lumen/
  app/
    scene/
    renderer/
    ui/
  reference/
    phosphor_model.*
    optics_model.*
  tests/
    reference_vectors.*
    render_smoke.*
  captures/
```

La scene 3D principale devra etre plein ecran. L'interface se limite aux
outils reels: choix de profil, intensite, persistance, dispersion, pause et
capture.

## 6. Premier prototype visible

Le prototype initial doit montrer:

1. Une surface noire phosphorescente plein ecran.
2. Un balayage CRT qui dessine une mire ou une texture simple.
3. Une persistance reglable qui laisse la trace du spot.
4. Un toggle `Prism` qui fait passer certains rayons par un prisme 3D.
5. Une comparaison directe entre `CRT_BASELINE` et `PRISM_FIELD`.

La geometrie lumineuse doit produire l'image; elle ne doit pas etre un simple
filtre CSS ajoute sur une texture deja dessinee.

## 7. Tests requis

```text
R01  Une impulsion excite un emplacement attendu sur la surface.
R02  L'energie decroit selon la constante de persistance.
R03  Le balayage raster couvre la grille dans l'ordre defini.
R04  Un rayon sans obstacle touche la cellule attendue.
R05  Une refraction sur prisme respecte un vecteur de reference.
R06  Une dispersion RGB separe correctement les trois composantes.
R07  Deux executions avec memes parametres ont un rendu de reference stable.
R08  Une capture desktop et mobile montre une scene non vide et cadree.
R09  Les controles ne chevauchent pas la scene utile sur mobile.
R10  Le budget de frame est mesure, sans promesse de temps reel non testee.
```

## 8. Questions de recherche

- Une grille phosphore 2D suffit-elle, ou faut-il une couche volumetrique?
- Le balayage est-il pilote par texture, par primitives ou par signal audio?
- Quels parametres deviennent le vocabulaire visuel propre au produit?
- Peut-on rendre un texte/UI lisible avec ce moteur sans retomber dans un
  overlay DOM conventionnel?

## 9. Decision actuelle

Construire d'abord un affichage CRT virtuel interactif. Les concepts de code,
carrier, canal ou chiffrement ne font pas partie du MVP PRISM-LUMEN.
