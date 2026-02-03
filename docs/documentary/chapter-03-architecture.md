# Chapitre 3 – Architecture TRustOs

## 🎬 Narration

> *"L'architecture d'un OS définit ses possibilités et ses limites. TRustOs est conçu pour être minimal, sécurisé, et observable."*

---

## 📖 Script narratif

### Scène 3.1 – Vue d'ensemble
**Durée estimée**: 2 minutes

**Voix off**:
"TRustOs est organisé en couches strictement séparées. Au cœur, un microkernel qui ne fait que l'essentiel. Au-dessus, des services userland qui communiquent par IPC asynchrone. Et au sommet, les applications et l'agent IA Jarvis."

**Diagramme architecture**:
```
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATIONS                              │
│                 (GUI / CLI / Scripts)                        │
├─────────────────────────────────────────────────────────────┤
│                    AI AGENT (JARVIS)                         │
│         ┌─────────┬──────────┬──────────┐                  │
│         │ Parser  │ Planner  │ Executor │                  │
│         └─────────┴──────────┴──────────┘                  │
├─────────────────────────────────────────────────────────────┤
│                  USERLAND SERVICES                           │
│   ┌────────┬───────────┬─────────┬────────────┐            │
│   │  Init  │ Filesystem│ Network │ Device Mgr │            │
│   └────────┴───────────┴─────────┴────────────┘            │
├─────────────────────────────────────────────────────────────┤
│                    MICROKERNEL                               │
│   ┌──────────┬─────┬────────┬───────────┬───────┐          │
│   │Scheduler │ IPC │ Memory │ Interrupts│ Caps  │          │
│   └──────────┴─────┴────────┴───────────┴───────┘          │
├─────────────────────────────────────────────────────────────┤
│                      HARDWARE                                │
└─────────────────────────────────────────────────────────────┘
```

### Scène 3.2 – Le microkernel
**Durée estimée**: 3-4 minutes

**Voix off**:
"Le kernel TRustOs ne fait que cinq choses : scheduler les tâches, gérer la mémoire, router les interruptions, orchestrer l'IPC, et valider les capabilities. Rien d'autre. Pas de filesystem, pas de drivers, pas de réseau."

**Composants kernel**:

| Module | Fichier | Responsabilité |
|--------|---------|----------------|
| Scheduler | `scheduler/mod.rs` | Ordonnancement NUMA-aware |
| Memory | `memory/mod.rs` | Heap + frame allocator |
| Interrupts | `interrupts/mod.rs` | IDT + handlers |
| IPC | `ipc/mod.rs` | Channels async/batched |
| Security | `security/mod.rs` | Capabilities |
| Trace | `trace/mod.rs` | Event ring buffer |

### Scène 3.3 – IPC asynchrone
**Durée estimée**: 3 minutes

**Voix off**:
"Dans un OS monolithique, un appel système est un saut direct dans le kernel. Dans TRustOs, c'est un message envoyé à un service. Et ces messages peuvent être groupés en batch pour réduire le coût des context switches."

**Animation IPC**:
```
┌─────────┐                              ┌─────────────┐
│  App A  │──── Message 1 ────┐         │  Service B  │
│         │──── Message 2 ────┼──batch──│             │
│         │──── Message 3 ────┘         │             │
└─────────┘                              └─────────────┘
          ↓                                    ↓
    1 context switch                    1 context switch
         au lieu de 3                       au lieu de 3
```

**Code IPC**:
```rust
// Envoi batché de messages
let messages = [msg1, msg2, msg3];
channel.send_batch(&messages)?;  // Un seul context switch

// Réception batché
let received = channel.receive_batch(10)?;
```

### Scène 3.4 – Sécurité par capabilities
**Durée estimée**: 3 minutes

**Voix off**:
"Oubliez les permissions Unix. Oubliez les ACL Windows. TRustOs utilise des capabilities : des tokens uniques et infalsifiables qui représentent un droit d'accès spécifique."

**Comparaison**:
```
┌────────────────────────────────────────────────────────────┐
│  PERMISSIONS TRADITIONNELLES                               │
│  ──────────────────────────────────────────────────────── │
│  "L'utilisateur admin peut lire /etc/passwd"              │
│  Problème: Confused deputy, privilege escalation          │
├────────────────────────────────────────────────────────────┤
│  CAPABILITIES                                              │
│  ──────────────────────────────────────────────────────── │
│  "Ce token donne accès READ à cette ressource"            │
│  Le token est infalsifiable et révocable                  │
└────────────────────────────────────────────────────────────┘
```

**Code capability**:
```rust
// Créer une capability
let cap = create_capability(
    CapabilityType::Filesystem,
    CapabilityRights::READ,
    owner_task_id
);

// Valider avant chaque accès
validate(cap_id, CapabilityRights::READ)?;
```

### Scène 3.5 – Event tracing
**Durée estimée**: 2 minutes

**Voix off**:
"Chaque événement kernel est enregistré dans un ring buffer lock-free. En cas de crash, les 32 derniers événements sont dumpés. En mode déterministe, on peut rejouer exactement la même séquence."

**Structure trace**:
```rust
TraceEvent {
    timestamp: 12345,      // Tick kernel
    cpu_id: 0,             // CPU source
    event_type: TimerTick, // Type d'événement
    payload: 0x1234,       // Données spécifiques
}
```

**Dump on panic**:
```
=== TRACE DUMP (last 32 events) ===
[       100][CPU0] TimerTick payload=0x0
[       101][CPU0] ContextSwitch payload=0x5
[       102][CPU0] KeyboardInput payload=0x1e
[       103][CPU0] PageFault payload=0x4444444000
=== END TRACE DUMP ===
```

---

## 🎨 Storyboard visuel

```
┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 3.1 - VUE D'ENSEMBLE                                 │
├─────────────────────────────────────────────────────────────┤
│  [Animation: Construction de l'architecture couche par     │
│   couche, du hardware jusqu'aux applications]              │
│                                                             │
│  1. Hardware apparaît                                       │
│  2. Microkernel se pose dessus                             │
│  3. Services userland apparaissent                         │
│  4. Jarvis et apps au sommet                               │
│                                                             │
│  Texte: "Architecture en couches strictement séparées"     │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 3.3 - IPC ASYNC                                      │
├─────────────────────────────────────────────────────────────┤
│  [Animation: Messages qui s'accumulent puis partent        │
│   en batch vers le service destinataire]                   │
│                                                             │
│  App ──[msg1]──┐                                           │
│      ──[msg2]──┼──batch──→ Service                        │
│      ──[msg3]──┘                                           │
│                                                             │
│  Compteur: "Context switches: 1 au lieu de 3"              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 3.4 - CAPABILITIES                                   │
├─────────────────────────────────────────────────────────────┤
│  [Animation: Token capability comme une clé numérique]     │
│                                                             │
│  ┌──────────────────┐                                      │
│  │ CAPABILITY TOKEN │                                      │
│  │ ──────────────── │                                      │
│  │ Type: Filesystem │                                      │
│  │ Rights: READ     │                                      │
│  │ Owner: Task 42   │                                      │
│  │ ID: 0x1234abcd   │                                      │
│  └──────────────────┘                                      │
│                                                             │
│  Texte: "Chaque accès validé par token"                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 📊 Fichiers sources associés

| Concept | Fichier | Lignes clés |
|---------|---------|-------------|
| Architecture | `kernel/src/main.rs` | Entry point, init sequence |
| Scheduler | `kernel/src/scheduler/mod.rs` | Task queues, priorities |
| IPC | `kernel/src/ipc/channel.rs` | send_batch, receive_batch |
| Capabilities | `kernel/src/security/capability.rs` | CapabilityRights, validation |
| Tracing | `kernel/src/trace/mod.rs` | TraceEvent, ring buffer |

---

## ✅ Statut chapitre

- [x] Script narratif détaillé
- [x] Diagrammes architecture
- [x] Storyboards conceptuels
- [ ] Animations finales
- [ ] Enregistrement voix off
- [ ] Montage vidéo
