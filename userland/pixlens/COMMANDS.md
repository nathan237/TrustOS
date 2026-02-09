# Pixlens - Commandes Recommandées

## Chemin de l'exe
```powershell
$pixlens = "C:\Users\nathan\Documents\Scripts\OSrust\userland\pixlens\target\x86_64-pc-windows-msvc\release\pixlens.exe"
```

## Analyse Rapide

### 1. Test de qualité crypto (LE PLUS IMPORTANT)
```powershell
# Digraph = scatter plot byte→byte
# Bon chiffrement = bruit uniforme | Mauvais = patterns visibles
& $pixlens -i fichier.bin -m digraph -o crypto_test.ppm
```

### 2. Carte d'entropie
```powershell
# Rouge=chiffré | Orange=compressé | Jaune=code | Vert=texte | Bleu=padding
& $pixlens -i fichier.bin -m entropy -p 32 -o entropy.ppm
```

### 3. Détection de zones haute entropie
```powershell
& $pixlens -i fichier.bin -m highent -p 32 -o zones.ppm
```

## Analyse Forensique

### 4. Trouver des strings cachées
```powershell
& $pixlens -i fichier.bin -m ascii -w 512 -o strings.ppm
```

### 5. Révéler du XOR simple
```powershell
# Teste différentes clés XOR
& $pixlens -i fichier.bin -m xor -p 255 -o xor_ff.ppm
& $pixlens -i fichier.bin -m xor -p 170 -o xor_aa.ppm  # 0xAA
& $pixlens -i fichier.bin -m xor -p 85 -o xor_55.ppm   # 0x55
```

### 6. Détecter patterns de bloc (ECB, clés répétées)
```powershell
# Modulo 16 = blocs AES | Modulo 8 = DES
& $pixlens -i fichier.bin -m modulo -p 16 -o blocks_16.ppm
& $pixlens -i fichier.bin -m modulo -p 64 -o blocks_64.ppm
```

### 7. Stéganographie LSB
```powershell
# Bit 0 = LSB (souvent utilisé pour cacher des données)
& $pixlens -i image.bin -m bitplane -p 0 -o lsb.ppm
& $pixlens -i image.bin -m bitplane -p 7 -o msb.ppm
```

## Analyse Complète

### 8. Générer TOUS les mappings
```powershell
& $pixlens -i fichier.bin -o output.ppm --all
```

### 9. Comparer deux fichiers
```powershell
& $pixlens -i original.bin -m digraph -o orig.ppm
& $pixlens -i modified.bin -m digraph -o mod.ppm
# Comparer visuellement les deux images
```

## Analyse Malware

### 10. Workflow malware rapide
```powershell
$file = "suspect.exe"
& $pixlens -i $file -m digraph -o mal_digraph.ppm      # Crypto?
& $pixlens -i $file -m highent -p 32 -o mal_packed.ppm # Packé?
& $pixlens -i $file -m ascii -o mal_strings.ppm        # Strings?
& $pixlens -i $file -m null -o mal_sections.ppm        # Structure?
```

## Interprétation Digraph

| Pattern | Signification |
|---------|---------------|
| Bruit uniforme | Bon chiffrement / vraie randomisation |
| Diagonale | XOR avec clé fixe |
| Carré en bas-gauche | Principalement ASCII |
| Lignes horizontales/verticales | Valeurs répétées |
| Clusters | Distribution non-uniforme |
| Damier | Patterns alternés |

## Demo Script
```powershell
cd C:\Users\nathan\Documents\Scripts\OSrust\userland\pixlens
.\demo.ps1
```
