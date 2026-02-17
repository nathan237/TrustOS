//! I/O APIC (Input/Output Advanced Programmable Interrupt Controller) Emulation
//!
//! The I/O APIC routes external interrupts (keyboard, disk, network, timer)
//! to Local APICs. Linux configures it via MMIO at 0xFEC00000.
//!
//! Registers (accessed via indirect IOREGSEL/IOWIN window):
//!   0x00: IOAPICID  — I/O APIC ID
//!   0x01: IOAPICVER — Version + max redirection entries
//!   0x02: IOAPICARB — Arbitration ID
//!   0x10-0x3F: Redirection Table (24 entries × 2 DWORDs each)
//!
//! Each Redirection Table Entry (RTE) is 64 bits:
//!   Bits [7:0]   — Interrupt vector
//!   Bits [10:8]  — Delivery mode (000=Fixed, 010=SMI, 100=NMI, 101=INIT, 111=ExtINT)
//!   Bit  [11]    — Destination mode (0=Physical, 1=Logical)
//!   Bit  [12]    — Delivery status (RO, 0=idle)
//!   Bit  [13]    — Pin polarity (0=active high, 1=active low)
//!   Bit  [14]    — Remote IRR (RO)
//!   Bit  [15]    — Trigger mode (0=edge, 1=level)
//!   Bit  [16]    — Mask (0=enabled, 1=masked)
//!   Bits [63:56] — Destination (APIC ID or logical mask)
//!
//! References:
//!   - 82093AA I/O APIC Datasheet (Intel)
//!   - ACPI Spec 6.4 §5.2.12 (MADT I/O APIC Structure)

/// Maximum number of redirection entries (standard for 82093AA)
const MAX_REDIR_ENTRIES: usize = 24;

/// I/O APIC MMIO base address
pub const IOAPIC_BASE: u64 = 0xFEC0_0000;

/// Register offsets within the MMIO page
const IOREGSEL: u64 = 0x00;  // I/O Register Select (write index)
const IOWIN: u64 = 0x10;     // I/O Window (read/write data)

/// I/O APIC emulation state
#[derive(Debug, Clone)]
pub struct IoApicState {
    /// I/O APIC ID (bits [27:24] of register 0x00)
    pub id: u8,
    /// Currently selected register index (written via IOREGSEL)
    pub ioregsel: u32,
    /// 24 Redirection Table Entries (64-bit each)
    pub redir_table: [u64; MAX_REDIR_ENTRIES],
}

impl Default for IoApicState {
    fn default() -> Self {
        // All entries masked by default (bit 16 = 1)
        let mut redir_table = [0u64; MAX_REDIR_ENTRIES];
        for entry in redir_table.iter_mut() {
            *entry = 1 << 16; // Masked
        }
        
        Self {
            id: 1, // Must match MADT I/O APIC ID
            ioregsel: 0,
            redir_table,
        }
    }
}

impl IoApicState {
    /// Handle MMIO read at the given offset within the I/O APIC page.
    /// Returns the 32-bit value to give to the guest.
    pub fn read(&self, offset: u64) -> u32 {
        match offset {
            IOREGSEL => self.ioregsel,
            IOWIN => self.read_register(self.ioregsel),
            _ => 0,
        }
    }
    
    /// Handle MMIO write at the given offset within the I/O APIC page.
    pub fn write(&mut self, offset: u64, value: u32) {
        match offset {
            IOREGSEL => {
                self.ioregsel = value;
            }
            IOWIN => {
                self.write_register(self.ioregsel, value);
            }
            _ => {}
        }
    }
    
    /// Read an I/O APIC register by index
    fn read_register(&self, index: u32) -> u32 {
        match index {
            // IOAPICID: bits [27:24] = ID
            0x00 => (self.id as u32) << 24,
            
            // IOAPICVER: bits [23:16] = max redir entry, [7:0] = version
            0x01 => {
                let max_entry = (MAX_REDIR_ENTRIES - 1) as u32;
                (max_entry << 16) | 0x20 // Version 0x20 (82093AA compatible)
            }
            
            // IOAPICARB: bits [27:24] = arbitration ID
            0x02 => (self.id as u32) << 24,
            
            // Redirection table entries (0x10 - 0x3F)
            // Each entry is 2 DWORDs: low at 0x10+2*n, high at 0x11+2*n
            0x10..=0x3F => {
                let entry_idx = ((index - 0x10) / 2) as usize;
                let is_high = (index & 1) != 0;
                
                if entry_idx < MAX_REDIR_ENTRIES {
                    if is_high {
                        (self.redir_table[entry_idx] >> 32) as u32
                    } else {
                        self.redir_table[entry_idx] as u32
                    }
                } else {
                    0
                }
            }
            
            _ => 0,
        }
    }
    
    /// Write an I/O APIC register by index
    fn write_register(&mut self, index: u32, value: u32) {
        match index {
            // IOAPICID: only bits [27:24] writable
            0x00 => {
                self.id = ((value >> 24) & 0xF) as u8;
            }
            
            // IOAPICVER and IOAPICARB are read-only
            0x01 | 0x02 => {}
            
            // Redirection table entries
            0x10..=0x3F => {
                let entry_idx = ((index - 0x10) / 2) as usize;
                let is_high = (index & 1) != 0;
                
                if entry_idx < MAX_REDIR_ENTRIES {
                    if is_high {
                        // Write upper 32 bits (destination)
                        self.redir_table[entry_idx] = 
                            (self.redir_table[entry_idx] & 0x0000_0000_FFFF_FFFF)
                            | ((value as u64) << 32);
                    } else {
                        // Write lower 32 bits (vector, mode, mask, etc.)
                        // Bits 12 and 14 are read-only (delivery status, remote IRR)
                        let ro_mask: u32 = (1 << 12) | (1 << 14);
                        let old_lo = self.redir_table[entry_idx] as u32;
                        let new_lo = (value & !ro_mask) | (old_lo & ro_mask);
                        self.redir_table[entry_idx] = 
                            (self.redir_table[entry_idx] & 0xFFFF_FFFF_0000_0000)
                            | (new_lo as u64);
                    }
                }
            }
            
            _ => {}
        }
    }
    
    /// Get the routing information for a given IRQ (GSI).
    /// Returns (vector, masked, delivery_mode, destination) if the entry exists.
    pub fn get_irq_route(&self, gsi: u8) -> Option<IrqRoute> {
        let idx = gsi as usize;
        if idx >= MAX_REDIR_ENTRIES {
            return None;
        }
        
        let entry = self.redir_table[idx];
        let vector = (entry & 0xFF) as u8;
        let delivery_mode = ((entry >> 8) & 0x7) as u8;
        let dest_mode = ((entry >> 11) & 1) != 0; // true = logical
        let polarity = ((entry >> 13) & 1) != 0;   // true = active low
        let trigger = ((entry >> 15) & 1) != 0;     // true = level
        let masked = ((entry >> 16) & 1) != 0;
        let destination = ((entry >> 56) & 0xFF) as u8;
        
        Some(IrqRoute {
            vector,
            delivery_mode,
            dest_logical: dest_mode,
            active_low: polarity,
            level_triggered: trigger,
            masked,
            destination,
        })
    }
}

/// Decoded I/O APIC routing information for a single IRQ
#[derive(Debug, Clone)]
pub struct IrqRoute {
    /// Interrupt vector number
    pub vector: u8,
    /// Delivery mode (0=Fixed, 1=LowestPri, 2=SMI, 4=NMI, 5=INIT, 7=ExtINT)
    pub delivery_mode: u8,
    /// Destination mode: true=logical, false=physical
    pub dest_logical: bool,
    /// Polarity: true=active low, false=active high
    pub active_low: bool,
    /// Trigger: true=level, false=edge
    pub level_triggered: bool,
    /// Masked: true=masked (disabled)
    pub masked: bool,
    /// Destination APIC ID (physical) or logical mask
    pub destination: u8,
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ioapic_defaults() {
        let ioapic = IoApicState::default();
        assert_eq!(ioapic.id, 1);
        assert_eq!(ioapic.ioregsel, 0);
        // All entries should be masked
        for entry in &ioapic.redir_table {
            assert_ne!(*entry & (1 << 16), 0, "Entry should be masked");
        }
    }
    
    #[test]
    fn test_ioapic_id_register() {
        let ioapic = IoApicState::default();
        let id_reg = ioapic.read_register(0x00);
        assert_eq!((id_reg >> 24) & 0xF, 1);
    }
    
    #[test]
    fn test_ioapic_version_register() {
        let ioapic = IoApicState::default();
        let ver = ioapic.read_register(0x01);
        assert_eq!(ver & 0xFF, 0x20); // version
        assert_eq!((ver >> 16) & 0xFF, 23); // max redir = 23
    }
    
    #[test]
    fn test_ioapic_redir_write_read() {
        let mut ioapic = IoApicState::default();
        // Write low DWORD of entry 0: vector=0x30, fixed, physical, edge, unmasked
        ioapic.write_register(0x10, 0x0000_0030);
        // Write high DWORD: destination APIC ID = 0
        ioapic.write_register(0x11, 0x0000_0000);
        
        let lo = ioapic.read_register(0x10);
        assert_eq!(lo & 0xFF, 0x30); // vector
        assert_eq!((lo >> 16) & 1, 0); // not masked
        
        let route = ioapic.get_irq_route(0).unwrap();
        assert_eq!(route.vector, 0x30);
        assert!(!route.masked);
        assert_eq!(route.delivery_mode, 0); // Fixed
    }
    
    #[test]
    fn test_ioapic_indirect_access() {
        let mut ioapic = IoApicState::default();
        // Write via IOREGSEL/IOWIN (as the guest would)
        ioapic.write(IOREGSEL, 0x01); // select version register
        let ver = ioapic.read(IOWIN);
        assert_eq!(ver & 0xFF, 0x20);
    }
}
