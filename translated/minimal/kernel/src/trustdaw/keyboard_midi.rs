























#[derive(Debug, Clone, Copy)]
pub struct Al {
    
    pub scancode: u8,
    
    pub midi_note: u8,
}



static CIF_: &[Al] = &[
    
    Al { scancode: 0x2C, midi_note: 48 }, 
    Al { scancode: 0x2D, midi_note: 50 }, 
    Al { scancode: 0x2E, midi_note: 52 }, 
    Al { scancode: 0x2F, midi_note: 53 }, 
    Al { scancode: 0x30, midi_note: 55 }, 
    Al { scancode: 0x31, midi_note: 57 }, 
    Al { scancode: 0x32, midi_note: 59 }, 
    Al { scancode: 0x33, midi_note: 60 }, 
    Al { scancode: 0x34, midi_note: 62 }, 
    Al { scancode: 0x35, midi_note: 64 }, 
    
    Al { scancode: 0x1F, midi_note: 49 }, 
    Al { scancode: 0x20, midi_note: 51 }, 
    
    Al { scancode: 0x22, midi_note: 54 }, 
    Al { scancode: 0x23, midi_note: 56 }, 
    Al { scancode: 0x24, midi_note: 58 }, 
    
    Al { scancode: 0x26, midi_note: 61 }, 
    Al { scancode: 0x27, midi_note: 63 }, 
];


static DDI_: &[Al] = &[
    
    Al { scancode: 0x10, midi_note: 60 }, 
    Al { scancode: 0x11, midi_note: 62 }, 
    Al { scancode: 0x12, midi_note: 64 }, 
    Al { scancode: 0x13, midi_note: 65 }, 
    Al { scancode: 0x14, midi_note: 67 }, 
    Al { scancode: 0x15, midi_note: 69 }, 
    Al { scancode: 0x16, midi_note: 71 }, 
    Al { scancode: 0x17, midi_note: 72 }, 
    Al { scancode: 0x18, midi_note: 74 }, 
    Al { scancode: 0x19, midi_note: 76 }, 
    
    Al { scancode: 0x03, midi_note: 61 }, 
    Al { scancode: 0x04, midi_note: 63 }, 
    
    Al { scancode: 0x06, midi_note: 66 }, 
    Al { scancode: 0x07, midi_note: 68 }, 
    Al { scancode: 0x08, midi_note: 70 }, 
    
    Al { scancode: 0x0A, midi_note: 73 }, 
    Al { scancode: 0x0B, midi_note: 75 }, 
];


static HN_: core::sync::atomic::AtomicI8 = core::sync::atomic::AtomicI8::new(0);


static ARZ_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(100);

use core::sync::atomic::Ordering;



pub fn dyl(scancode: u8) -> Option<u8> {
    let offset = HN_.load(Ordering::Relaxed) as i16 * 12;

    
    for mapping in CIF_ {
        if mapping.scancode == scancode {
            let note = mapping.midi_note as i16 + offset;
            return if note >= 0 && note <= 127 { Some(note as u8) } else { None };
        }
    }

    
    for mapping in DDI_ {
        if mapping.scancode == scancode {
            let note = mapping.midi_note as i16 + offset;
            return if note >= 0 && note <= 127 { Some(note as u8) } else { None };
        }
    }

    None
}


pub fn nmf() -> i8 {
    let current = HN_.load(Ordering::Relaxed);
    if current < 4 {
        HN_.store(current + 1, Ordering::Relaxed);
    }
    HN_.load(Ordering::Relaxed)
}


pub fn nme() -> i8 {
    let current = HN_.load(Ordering::Relaxed);
    if current > -4 {
        HN_.store(current - 1, Ordering::Relaxed);
    }
    HN_.load(Ordering::Relaxed)
}


pub fn qhz() -> i8 {
    HN_.load(Ordering::Relaxed)
}


pub fn jfn(anb: u8) {
    ARZ_.store(anb.min(127), Ordering::Relaxed);
}


pub fn dqs() -> u8 {
    ARZ_.load(Ordering::Relaxed)
}


pub fn qmu(scancode: u8) -> bool {
    dyl(scancode).is_some()
}


pub fn hsp() -> &'static str {
    concat!(
        "TrustDAW Virtual Piano — PC Keyboard Layout\n",
        "═══════════════════════════════════════════════\n",
        "\n",
        " Number row (sharps/flats):\n",
        "  [2]  [3]      [5]  [6]  [7]      [9]  [0]\n",
        "  C#4  D#4      F#4  G#4  A#4      C#5  D#5\n",
        "\n",
        " QWERTY row (upper naturals, octave 4):\n",
        "  [Q]  [W]  [E] [R]  [T]  [Y]  [U] [I]  [O]  [P]\n",
        "   C4   D4   E4  F4   G4   A4   B4  C5   D5   E5\n",
        "\n",
        " Home row (sharps/flats):\n",
        "  [S]  [D]      [G]  [H]  [J]      [L]  [;]\n",
        "  C#3  D#3      F#3  G#3  A#3      C#4  D#4\n",
        "\n",
        " Bottom row (lower naturals, octave 3):\n",
        "  [Z]  [X]  [C] [V]  [B]  [N]  [M] [,]  [.]  [/]\n",
        "   C3   D3   E3  F3   G3   A3   B3  C4   D4   E4\n",
        "\n",
        " Controls:\n",
        "  [F1] Octave -  │  [F2] Octave +\n",
        "  [F3] Vel -     │  [F4] Vel +\n",
        "  [Esc] Exit piano mode\n",
    )
}
