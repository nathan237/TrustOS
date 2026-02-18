//! Game Boy Timer — DIV, TIMA, TMA, TAC with overflow interrupt
#![allow(dead_code)]

pub struct Timer {
    pub div: u16,       // Internal 16-bit divider (upper byte readable at $FF04)
    pub tima: u8,       // Timer counter ($FF05)
    pub tma: u8,        // Timer modulo ($FF06)
    pub tac: u8,        // Timer control ($FF07)
    pub interrupt: bool, // Timer overflow interrupt request
    overflow_cycles: u8, // Delay for interrupt after overflow
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0xABCC,
            tima: 0,
            tma: 0,
            tac: 0,
            interrupt: false,
            overflow_cycles: 0,
        }
    }

    /// Step timer by given number of CPU cycles
    pub fn step(&mut self, cycles: u32) {
        for _ in 0..cycles {
            let old_div = self.div;
            self.div = self.div.wrapping_add(4); // DIV increments every T-cycle (4 per M-cycle)

            // Timer enabled?
            if self.tac & 0x04 != 0 {
                let bit = match self.tac & 0x03 {
                    0 => 9,  // 4096 Hz  (every 1024 cycles)
                    1 => 3,  // 262144 Hz (every 16 cycles)
                    2 => 5,  // 65536 Hz (every 64 cycles)
                    3 => 7,  // 16384 Hz (every 256 cycles)
                    _ => 9,
                };

                // Falling edge detector on the selected bit
                let old_bit = (old_div >> bit) & 1;
                let new_bit = (self.div >> bit) & 1;

                if old_bit == 1 && new_bit == 0 {
                    let (new_tima, overflow) = self.tima.overflowing_add(1);
                    if overflow {
                        self.tima = self.tma;
                        self.interrupt = true;
                    } else {
                        self.tima = new_tima;
                    }
                }
            }
        }
    }

    pub fn read_div(&self) -> u8 {
        (self.div >> 8) as u8
    }

    pub fn write_div(&mut self) {
        self.div = 0;
    }
}
