# Storyboards – Séquences visuelles

## Index des storyboards

### Phase 0 – Kernel
- [ ] `sb-001-first-boot.md` – Premier boot QEMU
- [ ] `sb-002-architecture-overview.md` – Animation architecture
- [ ] `sb-003-ipc-flow.md` – Flux IPC async/batched
- [ ] `sb-004-capability-validation.md` – Validation capabilities
- [ ] `sb-005-trace-dump.md` – Panic et trace dump

### Phase 1 – Userland
- [ ] `sb-101-shell-demo.md` – Démonstration shell
- [ ] `sb-102-filesystem-ops.md` – Opérations filesystem
- [ ] `sb-103-network-stack.md` – Stack réseau async

### Phase 2 – Jarvis
- [ ] `sb-201-jarvis-intro.md` – Introduction Jarvis
- [ ] `sb-202-jarvis-pipeline.md` – Pipeline NLP→Executor
- [ ] `sb-203-jarvis-demo.md` – Démonstration commandes

### Phase 3 – GUI
- [ ] `sb-301-compositor.md` – Premier compositor
- [ ] `sb-302-desktop.md` – Desktop environment
- [ ] `sb-303-jarvis-gui.md` – Jarvis intégré GUI

---

## Template storyboard

```markdown
# Storyboard SB-XXX – Titre

## Métadonnées
- Chapitre: X
- Durée estimée: X minutes
- Statut: [ ] À faire / [x] Complété

## Séquence

### Shot 1 – Description
- Durée: X secondes
- Type: [Screen capture / Animation / Diagram]
- Audio: [Voix off / Musique / Silence]
- Description détaillée...

### Shot 2 – Description
...

## Assets requis
- [ ] Capture QEMU
- [ ] Diagramme X
- [ ] Animation Y

## Notes de production
...
```
