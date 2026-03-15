























#[derive(Debug, Clone, Copy)]
pub struct Bk {
    
    pub scancode: u8,
    
    pub ti: u8,
}



static CEW_: &[Bk] = &[
    
    Bk { scancode: 0x2C, ti: 48 }, 
    Bk { scancode: 0x2D, ti: 50 }, 
    Bk { scancode: 0x2E, ti: 52 }, 
    Bk { scancode: 0x2F, ti: 53 }, 
    Bk { scancode: 0x30, ti: 55 }, 
    Bk { scancode: 0x31, ti: 57 }, 
    Bk { scancode: 0x32, ti: 59 }, 
    Bk { scancode: 0x33, ti: 60 }, 
    Bk { scancode: 0x34, ti: 62 }, 
    Bk { scancode: 0x35, ti: 64 }, 
    
    Bk { scancode: 0x1F, ti: 49 }, 
    Bk { scancode: 0x20, ti: 51 }, 
    
    Bk { scancode: 0x22, ti: 54 }, 
    Bk { scancode: 0x23, ti: 56 }, 
    Bk { scancode: 0x24, ti: 58 }, 
    
    Bk { scancode: 0x26, ti: 61 }, 
    Bk { scancode: 0x27, ti: 63 }, 
];


static CZQ_: &[Bk] = &[
    
    Bk { scancode: 0x10, ti: 60 }, 
    Bk { scancode: 0x11, ti: 62 }, 
    Bk { scancode: 0x12, ti: 64 }, 
    Bk { scancode: 0x13, ti: 65 }, 
    Bk { scancode: 0x14, ti: 67 }, 
    Bk { scancode: 0x15, ti: 69 }, 
    Bk { scancode: 0x16, ti: 71 }, 
    Bk { scancode: 0x17, ti: 72 }, 
    Bk { scancode: 0x18, ti: 74 }, 
    Bk { scancode: 0x19, ti: 76 }, 
    
    Bk { scancode: 0x03, ti: 61 }, 
    Bk { scancode: 0x04, ti: 63 }, 
    
    Bk { scancode: 0x06, ti: 66 }, 
    Bk { scancode: 0x07, ti: 68 }, 
    Bk { scancode: 0x08, ti: 70 }, 
    
    Bk { scancode: 0x0A, ti: 73 }, 
    Bk { scancode: 0x0B, ti: 75 }, 
];


static GW_: core::sync::atomic::AtomicI8 = core::sync::atomic::AtomicI8::new(0);


static APX_: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(100);

use core::sync::atomic::Ordering;



pub fn hyv(scancode: u8) -> Option<u8> {
    let l = GW_.load(Ordering::Relaxed) as i16 * 12;

    
    for hqs in CEW_ {
        if hqs.scancode == scancode {
            let jp = hqs.ti as i16 + l;
            return if jp >= 0 && jp <= 127 { Some(jp as u8) } else { None };
        }
    }

    
    for hqs in CZQ_ {
        if hqs.scancode == scancode {
            let jp = hqs.ti as i16 + l;
            return if jp >= 0 && jp <= 127 { Some(jp as u8) } else { None };
        }
    }

    None
}


pub fn uwy() -> i8 {
    let cv = GW_.load(Ordering::Relaxed);
    if cv < 4 {
        GW_.store(cv + 1, Ordering::Relaxed);
    }
    GW_.load(Ordering::Relaxed)
}


pub fn uwx() -> i8 {
    let cv = GW_.load(Ordering::Relaxed);
    if cv > -4 {
        GW_.store(cv - 1, Ordering::Relaxed);
    }
    GW_.load(Ordering::Relaxed)
}


pub fn ytj() -> i8 {
    GW_.load(Ordering::Relaxed)
}


pub fn pjj(bxr: u8) {
    APX_.store(bxr.v(127), Ordering::Relaxed);
}


pub fn hlm() -> u8 {
    APX_.load(Ordering::Relaxed)
}


pub fn yzv(scancode: u8) -> bool {
    hyv(scancode).is_some()
}


pub fn nlx() -> &'static str {
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
