# TrustOS Embedded ROMs

Place your ROM files here to embed them into TrustOS at compile time.

## Supported formats

| File          | Emulator  | Format   |
|---------------|-----------|----------|
| `game.nes`    | NES       | iNES     |
| `game.gb`     | Game Boy  | GB ROM   |
| `game.gbc`    | Game Boy Color | GBC ROM |

## How to use

1. Place exactly one `.nes` and/or one `.gb`/`.gbc` file in this directory
2. Rebuild: `cargo build --release`
3. The ROM(s) will be auto-loaded when you open the emulator window

## Legal notice

Only use ROMs you have the legal right to use:
- Homebrew games (free, open-source)
- ROMs you dumped from cartridges you own
- Legally purchased digital copies

**Do not use pirated ROMs.**

## Recommended free homebrew

### Game Boy
- Tobu Tobu Girl: https://tangramgames.dk/tobutobugirl/
- uCity: https://github.com/AntonioND/ucity
- Adjustris: https://github.com/tbsp/Adjustris

### NES  
- Micro Mages: https://morphcat.de/micromages/
- From Below: https://github.com/mhughson/mbmern
