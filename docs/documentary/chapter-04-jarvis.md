# Chapitre 4 – AI Agent (Jarvis)

## 🎬 Narration

> *"Un OS moderne ne devrait pas seulement répondre aux commandes – il devrait les comprendre. Jarvis est l'assistant IA intégré à TRustOs, capable d'exécuter des tâches complexes en langage naturel."*

---

## 📖 Script narratif

### Scène 4.1 – Vision de Jarvis
**Durée estimée**: 2 minutes

**Voix off**:
"Imaginez pouvoir dire à votre OS : 'Installe nginx, configure un serveur avec PHP et SQLite, et assure-toi que tout est sécurisé.' Et que ce soit fait. Pas par magie, mais par un agent IA qui comprend votre intention et l'exécute étape par étape."

**Démonstration concept**:
```
jarvis> salut jarvis, installe nginx et configure un serveur avec php et sqlite

[JARVIS] Analyse de la demande...
[JARVIS] Plan d'exécution:
  1. Installer nginx via package manager
  2. Installer PHP-FPM
  3. Installer SQLite
  4. Configurer nginx pour PHP
  5. Valider la configuration

[JARVIS] Exécution du plan...
[JARVIS] ✓ nginx installé
[JARVIS] ✓ PHP-FPM installé
[JARVIS] ✓ SQLite installé
[JARVIS] ✓ Configuration nginx mise à jour
[JARVIS] ✓ Services démarrés

[JARVIS] Mission accomplie! Votre serveur est prêt.
```

### Scène 4.2 – Architecture de Jarvis
**Durée estimée**: 3-4 minutes

**Voix off**:
"Jarvis n'est pas une boîte noire. C'est un pipeline structuré : le Parser comprend votre demande, le Planner crée un plan d'actions, et l'Executor exécute chaque étape via les API sécurisées de l'OS."

**Diagramme pipeline**:
```
┌─────────────────────────────────────────────────────────────┐
│                      JARVIS PIPELINE                         │
└─────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────┐    "installe nginx et configure..."
│   CLI INPUT   │────────────────────────────────────┐
└───────────────┘                                    │
        │                                            │
        ▼                                            ▼
┌───────────────┐    Intent: INSTALL_AND_CONFIGURE  ┌─────────┐
│  NLP PARSER   │──────────────────────────────────▶│ Context │
└───────────────┘    Entities: [nginx, php, sqlite] └─────────┘
        │
        ▼
┌───────────────┐    Step 1: pkg_install(nginx)
│    PLANNER    │    Step 2: pkg_install(php)
│               │    Step 3: pkg_install(sqlite)
└───────────────┘    Step 4: config_write(nginx.conf)
        │            Step 5: service_start(nginx)
        ▼
┌───────────────┐
│   EXECUTOR    │──── Sandboxed API Calls ────▶ OS Services
└───────────────┘
        │
        ▼
┌───────────────┐
│   MONITOR     │    Logs, rollback si erreur
└───────────────┘
```

### Scène 4.3 – Sécurité et sandbox
**Durée estimée**: 2-3 minutes

**Voix off**:
"Jarvis est puissant, mais jamais dangereux. Chaque action passe par une sandbox. Jarvis n'exécute jamais de code arbitraire – il appelle des API OS pré-définies et validées."

**Principes de sécurité**:
```
┌─────────────────────────────────────────────────────────────┐
│  RÈGLES DE SÉCURITÉ JARVIS                                  │
├─────────────────────────────────────────────────────────────┤
│  ✗ Jamais d'exécution de code arbitraire                   │
│  ✗ Jamais d'accès direct aux fichiers système              │
│  ✗ Jamais d'escalade de privilèges                         │
├─────────────────────────────────────────────────────────────┤
│  ✓ Appels API OS uniquement                                │
│  ✓ Chaque action limitée par capabilities                  │
│  ✓ Logs complets pour audit                                │
│  ✓ Rollback automatique en cas d'erreur                    │
└─────────────────────────────────────────────────────────────┘
```

### Scène 4.4 – Exemples de commandes
**Durée estimée**: 3-4 minutes

**Voix off**:
"Jarvis comprend le français et l'anglais. Voyons quelques exemples de ce qu'il peut faire..."

**Exemple 1 – Installation logiciel**:
```
jarvis> installe visual studio code et configure-le pour rust

[JARVIS] Plan:
  1. Télécharger VSCode
  2. Installer VSCode
  3. Installer extensions Rust (rust-analyzer)
  4. Configurer settings.json
```

**Exemple 2 – Analyse système**:
```
jarvis> analyse l'utilisation mémoire et trouve les fuites potentielles

[JARVIS] Analyse en cours...
[JARVIS] Rapport:
  - Mémoire totale: 16 GB
  - Utilisée: 8.2 GB
  - Processus suspect: chrome.exe (3.1 GB, croissance constante)
  - Recommandation: Redémarrer Chrome ou vérifier extensions
```

**Exemple 3 – Sécurité**:
```
jarvis> vérifie les ports ouverts et ferme ceux non utilisés

[JARVIS] Scan ports...
[JARVIS] Ports ouverts:
  - 22 (SSH) - utilisé
  - 80 (HTTP) - utilisé
  - 3389 (RDP) - non utilisé
  
[JARVIS] Fermeture port 3389...
[JARVIS] ✓ Port 3389 fermé via firewall
```

### Scène 4.5 – Technologies sous-jacentes
**Durée estimée**: 2 minutes

**Voix off**:
"Jarvis utilise un modèle de langage quantifié pour fonctionner localement, sans cloud. Le moteur d'exécution est en WebAssembly pour la portabilité et l'isolation."

**Stack technologique**:
```
┌─────────────────────────────────────────────────────────────┐
│  STACK JARVIS                                                │
├─────────────────────────────────────────────────────────────┤
│  NLP Engine      : LLM quantifié local (llama.cpp style)   │
│  Runtime         : WASM (wasmtime)                          │
│  API Binding     : TRustOs API (IPC)                   │
│  Sandbox         : Capability-restricted                    │
│  Logging         : Event trace integration                  │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎨 Storyboard visuel

```
┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 4.1 - DÉMONSTRATION                                  │
├─────────────────────────────────────────────────────────────┤
│  [Capture: Terminal avec interaction Jarvis en temps réel] │
│                                                             │
│  Utilisateur tape: "installe nginx..."                     │
│  Jarvis répond avec plan détaillé                          │
│  Animation: barres de progression pour chaque étape        │
│  Résultat final: "Mission accomplie!"                      │
│                                                             │
│  Texte: "L'IA au service de l'utilisateur"                 │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 4.2 - PIPELINE                                       │
├─────────────────────────────────────────────────────────────┤
│  [Animation: Flux de données à travers le pipeline]        │
│                                                             │
│  Input ──▶ Parser ──▶ Planner ──▶ Executor ──▶ Result     │
│                                                             │
│  Chaque étape s'illumine quand active                      │
│  Les données transformées sont visibles                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 📊 Fichiers sources (Phase 2)

| Composant | Fichier prévu | Description |
|-----------|---------------|-------------|
| CLI Parser | `userland/jarvis/parser/mod.rs` | NLP et intent detection |
| Planner | `userland/jarvis/planner/mod.rs` | Génération de plans |
| Executor | `userland/jarvis/executor/mod.rs` | Exécution sandboxée |
| Sandbox | `userland/jarvis/sandbox/mod.rs` | Isolation capabilities |
| LLM | `userland/jarvis/llm/mod.rs` | Moteur NLP local |

---

## ✅ Statut chapitre

- [x] Script narratif
- [x] Diagramme architecture
- [x] Exemples de commandes
- [ ] Implémentation Jarvis (Phase 2)
- [ ] Captures démo réelles
- [ ] Montage vidéo

> ⚠️ **Note**: Ce chapitre sera complété lors de la Phase 2 du développement.
