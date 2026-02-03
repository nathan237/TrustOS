# Chapitre 5 – Défis et apprentissages

## 🎬 Narration

> *"Construire un OS, c'est résoudre des problèmes que personne n'a jamais rencontrés. Chaque bug est une énigme, chaque optimisation un art."*

---

## 📖 Script narratif

### Scène 5.1 – Le debug kernel
**Durée estimée**: 3-4 minutes

**Voix off**:
"Débugger un OS n'est pas comme débugger une application. Pas de printf, pas de debugger classique. Le kernel doit être observable depuis l'intérieur, comme un sous-marin qui s'inspecte lui-même."

**Défis rencontrés**:
```
┌─────────────────────────────────────────────────────────────┐
│  DÉFIS DU DEBUG KERNEL                                       │
├─────────────────────────────────────────────────────────────┤
│  ❌ Pas de std::println! disponible                        │
│  ❌ Pas de GDB traditionnel (pas d'OS hôte)                │
│  ❌ Les panics peuvent corrompre la mémoire                │
│  ❌ Les race conditions sont non-reproductibles            │
├─────────────────────────────────────────────────────────────┤
│  SOLUTIONS TRUSTOS                                         │
├─────────────────────────────────────────────────────────────┤
│  ✓ Serial output via UART dès le premier instant          │
│  ✓ Event tracing lock-free                                 │
│  ✓ Mode déterministe pour reproduction                    │
│  ✓ Panic handler qui dump la trace                        │
└─────────────────────────────────────────────────────────────┘
```

**Code solution**:
```rust
// Le panic handler dump les derniers événements
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::_print(format_args!("[KERNEL PANIC] {}\n", info));
    trace::dump_on_panic();  // Dump ring buffer
    loop { x86_64::instructions::hlt(); }
}
```

### Scène 5.2 – Performance IPC
**Durée estimée**: 3 minutes

**Voix off**:
"L'IPC est le cœur d'un microkernel. Chaque message mal optimisé ralentit tout le système. Nous avons implémenté le batching et le zero-copy pour minimiser les context switches."

**Optimisation batching**:
```
AVANT (naïf):
  Message 1 → context switch → traitement
  Message 2 → context switch → traitement
  Message 3 → context switch → traitement
  Total: 3 context switches (~3000 cycles chacun)

APRÈS (batched):
  [Message 1, 2, 3] → 1 context switch → traitement batch
  Total: 1 context switch
  Gain: 66% de réduction des context switches
```

**Code batching**:
```rust
// Avant: 3 appels séparés
channel.send(msg1)?;
channel.send(msg2)?;
channel.send(msg3)?;

// Après: 1 appel batché
channel.send_batch(&[msg1, msg2, msg3])?;
```

### Scène 5.3 – Lock-free programming
**Durée estimée**: 3-4 minutes

**Voix off**:
"Les locks sont simples mais dangereux dans un kernel. Deadlocks, priority inversion, contention... Nous utilisons des structures lock-free là où c'est critique."

**Exemple ring buffer lock-free**:
```rust
// Écriture lock-free dans le ring buffer
pub fn record_event(event_type: EventType, payload: u64) {
    let timestamp = get_timestamp();
    let cpu_id = 0;
    
    let event = TraceEvent { timestamp, cpu_id, event_type, payload };
    
    // Atomic increment pour obtenir un slot
    let index = WRITE_INDEX.fetch_add(1, Ordering::Relaxed) as usize;
    let slot = index % TRACE_BUFFER_SIZE;
    
    TRACE_BUFFER.lock()[slot] = event;
}
```

**Comparaison performances**:
```
┌─────────────────────────────────────────────────────────────┐
│  LOCK vs LOCK-FREE (tracing hot path)                       │
├─────────────────────────────────────────────────────────────┤
│  Avec Mutex classique:     ~500 cycles / event            │
│  Avec atomic lock-free:    ~50 cycles / event             │
│                                                             │
│  Gain: 10x sur le hot path                                 │
└─────────────────────────────────────────────────────────────┘
```

### Scène 5.4 – Sécurité capabilities
**Durée estimée**: 2-3 minutes

**Voix off**:
"Le plus grand défi de sécurité : comment valider chaque accès sans impacter les performances ? Les capabilities sont vérifiées à chaque appel IPC, mais la vérification doit être O(1)."

**Solution: lookup table**:
```rust
// Validation O(1) via BTreeMap
static CAPABILITIES: Mutex<BTreeMap<CapabilityId, Capability>> = ...;

pub fn validate(cap_id: CapabilityId, required: CapabilityRights) 
    -> Result<(), SecurityError> 
{
    let caps = CAPABILITIES.lock();
    let cap = caps.get(&cap_id)
        .ok_or(SecurityError::InvalidCapability)?;
    
    if !cap.has_rights(required) {
        return Err(SecurityError::InsufficientRights);
    }
    Ok(())
}
```

### Scène 5.5 – Leçons apprises
**Durée estimée**: 2 minutes

**Voix off**:
"Chaque bug résolu est une leçon. Voici ce que nous avons appris..."

**Leçons clés**:
```
┌─────────────────────────────────────────────────────────────┐
│  LEÇONS DE DÉVELOPPEMENT OS                                 │
├─────────────────────────────────────────────────────────────┤
│  1. Toujours avoir une sortie de debug (serial)            │
│  2. Tracer tout, filtrer après                             │
│  3. Les invariants sont sacrés – assert partout            │
│  4. Lock-free quand c'est critique, locks quand c'est sûr │
│  5. Tester dans QEMU avant de toucher au vrai hardware     │
│  6. Le mode déterministe sauve des heures de debug         │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎨 Storyboard visuel

```
┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 5.1 - PANIC TRACE                                    │
├─────────────────────────────────────────────────────────────┤
│  [Capture: Terminal QEMU avec panic et trace dump]         │
│                                                             │
│  [KERNEL PANIC] page fault at 0x4444444000                 │
│                                                             │
│  === TRACE DUMP (last 32 events) ===                       │
│  [100] TimerTick                                           │
│  [101] ContextSwitch task=5                                │
│  [102] PageFault addr=0x4444444000  ← ici!                │
│  === END TRACE DUMP ===                                    │
│                                                             │
│  Texte: "Le kernel raconte sa propre mort"                 │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  SCÈNE 5.2 - BATCHING                                       │
├─────────────────────────────────────────────────────────────┤
│  [Animation: Comparaison avant/après batching]             │
│                                                             │
│  Avant: msg→switch→msg→switch→msg→switch                  │
│  Après: [msg,msg,msg]→switch                               │
│                                                             │
│  Graphique: Réduction 66% context switches                 │
└─────────────────────────────────────────────────────────────┘
```

---

## 📊 Bugs mémorables

| Bug | Cause | Solution | Leçon |
|-----|-------|----------|-------|
| Triple fault au boot | IDT non chargée | Charger IDT avant enable_interrupts | Séquence d'init critique |
| Deadlock scheduler | Lock imbriqués | Restructurer en lock-free | Éviter locks dans hot path |
| Heap corruption | Double free | Rust borrow checker | Faire confiance au compilateur |

---

## ✅ Statut chapitre

- [x] Script narratif
- [x] Exemples de code
- [x] Leçons documentées
- [ ] Captures bugs réels
- [ ] Enregistrement voix off
- [ ] Montage vidéo
