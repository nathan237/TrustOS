//! HPET (High Precision Event Timer) Emulation
//!
//! The HPET provides a set of timers with nanosecond-level precision.
//! Linux queries it early in boot as a clocksource and for calibration.
//! MMIO at 0xFED00000 (1 page).
//!
//! Register layout (all 64-bit aligned):
//!   0x000: General Capabilities and ID
//!   0x010: General Configuration
//!   0x020: General Interrupt Status
//!   0x0F0: Main Counter Value
//!   0x100+0x20*N: Timer N Configuration and Capability
//!   0x108+0x20*N: Timer N Comparator Value
//!   0x110+0x20*N: Timer N FSB Interrupt Route
//!
//! We emulate:
//!   - 1 block, 3 timers (standard for a basic HPET)
//!   - 14.318 MHz tick rate (period = ~69.841 ns = 69841279 femtoseconds)
//!   - Read-only main counter that increments based on TSC
//!   - Timer comparator and configuration registers
//!
//! References:
//!   - IA-PC HPET Specification 1.0a (Intel)
//!   - Linux: arch/x86/kernel/hpet.c

/// HPET MMIO base address
pub const HPET_BASE: u64 = 0xFED0_0000;

/// HPET clock period in femtoseconds (69.841279 ns = ~14.318 MHz)
/// This matches the standard HPET frequency
const HPET_PERIOD_FS: u32 = 69_841_279; // ~14.318180 MHz

/// Number of timers we emulate
const NUM_TIMERS: usize = 3;

/// Register offsets
mod regs {
    pub const GCAP_ID: u64      = 0x000;  // General Capabilities and ID
    pub const GCONF: u64        = 0x010;  // General Configuration
    pub const GISR: u64         = 0x020;  // General Interrupt Status
    pub const MAIN_COUNTER: u64 = 0x0F0;  // Main Counter Value
    
    // Timer N registers at 0x100 + 0x20*N
    pub const TIMER_BASE: u64   = 0x100;
    pub const TIMER_STRIDE: u64 = 0x20;
    pub const TN_CONF: u64      = 0x00;   // Timer N Config + Capabilities
    pub const TN_COMP: u64      = 0x08;   // Timer N Comparator
    pub const TN_FSB: u64       = 0x10;   // Timer N FSB Route
}

/// HPET timer state
#[derive(Debug, Clone)]
pub struct HpetTimer {
    /// Configuration and capability register
    pub config: u64,
    /// Comparator value
    pub comparator: u64,
    /// FSB interrupt route
    pub fsb_route: u64,
}

impl Default for HpetTimer {
    fn default() -> Self {
        Self {
            // Capabilities: supports 32/64-bit mode, periodic capable (timer 0 only)
            // Bits [4:3] = interrupt type capability (level/edge)
            // Bit 5 = periodic capable
            // Bits [31:9] = interrupt routing capability (IRQs 0-23)
            config: 0x0000_0000_00F0_0030, // 64-bit capable, periodic capable, IRQs 20-23
            comparator: 0,
            fsb_route: 0,
        }
    }
}

/// HPET emulation state
#[derive(Debug, Clone)]
pub struct HpetState {
    /// General configuration register
    pub config: u64,
    /// General interrupt status register
    pub isr: u64,
    /// Main counter value at the time of last HPET enable
    pub counter_offset: u64,
    /// TSC value when HPET was last enabled (for counter derivation)
    pub tsc_at_enable: u64,
    /// Whether HPET is currently enabled (GCONF bit 0)
    pub enabled: bool,
    /// The 3 timers
    pub timers: [HpetTimer; NUM_TIMERS],
}

impl Default for HpetState {
    fn default() -> Self {
        let mut timers: [HpetTimer; NUM_TIMERS] = core::array::from_fn(|_| HpetTimer::default());
        // Timer 0: periodic capable
        timers[0].config = 0x0000_0000_00F0_0070; // 64-bit, periodic capable, size capable
        // Timer 1,2: not periodic capable  
        timers[1].config = 0x0000_0000_00F0_0030;
        timers[2].config = 0x0000_0000_00F0_0030;
        
        Self {
            config: 0,
            isr: 0,
            counter_offset: 0,
            tsc_at_enable: 0,
            enabled: false,
            timers,
        }
    }
}

impl HpetState {
    /// Read current main counter value.
    /// We derive it from TSC elapsed since enable, scaled to HPET frequency.
    fn main_counter(&self) -> u64 {
        if !self.enabled {
            return self.counter_offset;
        }
        let tsc_now = read_tsc();
        let tsc_elapsed = tsc_now.wrapping_sub(self.tsc_at_enable);
        // Convert TSC ticks to HPET ticks.
        // Assume ~2 GHz TSC; HPET is ~14.318 MHz → ratio ≈ 1:140
        // More accurately: hpet_ticks = tsc_elapsed * HPET_FREQ / TSC_FREQ
        // We approximate: HPET ≈ TSC / 140  (good enough for emulation)
        let hpet_ticks = tsc_elapsed / 140;
        self.counter_offset.wrapping_add(hpet_ticks)
    }
    
    /// Handle MMIO read from the HPET register space.
    /// Returns the value to give to the guest.
    pub fn read(&self, offset: u64, size: u8) -> u64 {
        let val64 = match offset {
            regs::GCAP_ID => {
                // General Capabilities and ID Register:
                // Bits [63:32] = period in femtoseconds
                // Bits [15:8]  = number of timers - 1
                // Bit [13]     = counter is 64-bit capable
                // Bits [7:0]   = revision ID
                let num_timers_minus_1 = (NUM_TIMERS as u64 - 1) << 8;
                let counter_64bit = 1u64 << 13;
                let rev_id = 0x01u64; // Revision 1
                ((HPET_PERIOD_FS as u64) << 32) | num_timers_minus_1 | counter_64bit | rev_id
            }
            regs::GCONF => self.config,
            regs::GISR => self.isr,
            regs::MAIN_COUNTER => self.main_counter(),
            
            // Timer registers
            off if off >= regs::TIMER_BASE && off < regs::TIMER_BASE + (NUM_TIMERS as u64) * regs::TIMER_STRIDE + 0x18 => {
                let timer_off = off - regs::TIMER_BASE;
                let timer_idx = (timer_off / regs::TIMER_STRIDE) as usize;
                let reg_off = timer_off % regs::TIMER_STRIDE;
                
                if timer_idx < NUM_TIMERS {
                    match reg_off {
                        regs::TN_CONF => self.timers[timer_idx].config,
                        regs::TN_COMP => self.timers[timer_idx].comparator,
                        regs::TN_FSB  => self.timers[timer_idx].fsb_route,
                        _ => 0,
                    }
                } else {
                    0
                }
            }
            _ => 0,
        };
        
        // Handle 32-bit reads (low/high dword)
        if size == 4 {
            if offset & 0x4 != 0 {
                // High 32 bits (e.g., reading at GCAP_ID+4 gives period)
                (val64 >> 32) & 0xFFFF_FFFF
            } else {
                val64 & 0xFFFF_FFFF
            }
        } else {
            val64
        }
    }
    
    /// Handle MMIO write to the HPET register space.
    pub fn write(&mut self, offset: u64, value: u64, size: u8) {
        match offset {
            regs::GCAP_ID => {} // Read-only
            
            regs::GCONF => {
                let old_enable = self.enabled;
                self.config = value & 0x3; // Only bits [1:0] are writable (ENABLE_CNF, LEG_RT_CNF)
                self.enabled = (value & 1) != 0;
                
                if self.enabled && !old_enable {
                    // Just enabled: snapshot TSC
                    self.tsc_at_enable = read_tsc();
                } else if !self.enabled && old_enable {
                    // Just disabled: freeze counter
                    self.counter_offset = self.main_counter();
                }
            }
            
            regs::GISR => {
                // Write-1-to-clear semantics
                self.isr &= !value;
            }
            
            regs::MAIN_COUNTER => {
                // Writable only when HPET is disabled
                if !self.enabled {
                    if size == 4 {
                        if offset & 0x4 != 0 {
                            self.counter_offset = (self.counter_offset & 0xFFFF_FFFF) | (value << 32);
                        } else {
                            self.counter_offset = (self.counter_offset & !0xFFFF_FFFF) | (value & 0xFFFF_FFFF);
                        }
                    } else {
                        self.counter_offset = value;
                    }
                }
            }
            
            // Timer registers
            off if off >= regs::TIMER_BASE && off < regs::TIMER_BASE + (NUM_TIMERS as u64) * regs::TIMER_STRIDE + 0x18 => {
                let timer_off = off - regs::TIMER_BASE;
                let timer_idx = (timer_off / regs::TIMER_STRIDE) as usize;
                let reg_off = timer_off % regs::TIMER_STRIDE;
                
                if timer_idx < NUM_TIMERS {
                    match reg_off {
                        regs::TN_CONF => {
                            // Only certain bits are writable:
                            // Bit 1: interrupt type (edge/level)
                            // Bit 2: interrupt enable
                            // Bit 3: periodic mode  
                            // Bit 6: value set (for periodic)
                            // Bits [13:9]: interrupt route
                            // Bit 14: FSB enable
                            let ro_mask: u64 = self.timers[timer_idx].config & 0xFFFF_FFFF_FFFF_8181;
                            let wr_mask: u64 = 0x0000_0000_0000_7E7E;
                            self.timers[timer_idx].config = ro_mask | (value & wr_mask);
                        }
                        regs::TN_COMP => {
                            self.timers[timer_idx].comparator = value;
                        }
                        regs::TN_FSB => {
                            self.timers[timer_idx].fsb_route = value;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Check if any timer has fired (comparator <= main counter).
    /// Returns a list of (timer_index, irq_route) for timers that fired.
    pub fn check_timers(&self) -> [(bool, u8); NUM_TIMERS] {
        let counter = self.main_counter();
        let mut result = [(false, 0u8); NUM_TIMERS];
        
        for i in 0..NUM_TIMERS {
            let config = self.timers[i].config;
            let enabled = (config >> 2) & 1 != 0;
            if !self.enabled || !enabled {
                continue;
            }
            let comparator = self.timers[i].comparator;
            if counter >= comparator && comparator > 0 {
                let irq_route = ((config >> 9) & 0x1F) as u8;
                result[i] = (true, irq_route);
            }
        }
        result
    }
}

/// Read the processor TSC (Time Stamp Counter)
#[inline(always)]
fn read_tsc() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    { 0 }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hpet_defaults() {
        let hpet = HpetState::default();
        assert!(!hpet.enabled);
        assert_eq!(hpet.config, 0);
        assert_eq!(hpet.timers.len(), 3);
    }
    
    #[test]
    fn test_hpet_gcap_id() {
        let hpet = HpetState::default();
        let gcap = hpet.read(0x000, 8);
        let period = (gcap >> 32) as u32;
        assert_eq!(period, HPET_PERIOD_FS);
        let num_timers = ((gcap >> 8) & 0x1F) as usize;
        assert_eq!(num_timers, 2); // NUM_TIMERS - 1
        assert_ne!(gcap & (1 << 13), 0); // 64-bit counter
    }
    
    #[test]
    fn test_hpet_enable_disable() {
        let mut hpet = HpetState::default();
        assert!(!hpet.enabled);
        hpet.write(0x010, 1, 8); // Enable
        assert!(hpet.enabled);
        hpet.write(0x010, 0, 8); // Disable
        assert!(!hpet.enabled);
    }
    
    #[test]
    fn test_hpet_counter_frozen_when_disabled() {
        let mut hpet = HpetState::default();
        hpet.write(0x0F0, 0x12345, 8); // Write main counter
        let c = hpet.read(0x0F0, 8);
        assert_eq!(c, 0x12345);
    }
}
