//! PCI Configuration Space Emulation
//!
//! Emulates a minimal PCI bus so Linux can enumerate devices without
//! panicking. Uses the standard port I/O mechanism:
//!   - Port 0xCF8: CONFIG_ADDRESS (write bus/device/function/register)
//!   - Port 0xCFC-0xCFF: CONFIG_DATA (read/write config register)
//!
//! CONFIG_ADDRESS format (32-bit):
//!   Bit  31:     Enable
//!   Bits 23:16:  Bus number
//!   Bits 15:11:  Device number
//!   Bits 10:8:   Function number
//!   Bits 7:2:    Register offset (DWORD aligned)
//!   Bits 1:0:    Always 0
//!
//! Emulated devices:
//!   Bus 0, Dev 0, Fn 0: Host Bridge (Intel 440FX compatible)
//!   Bus 0, Dev 1, Fn 0: ISA Bridge  (Intel PIIX3/82371SB compatible)
//!
//! All other BDF combinations return 0xFFFF_FFFF (no device).
//!
//! References:
//!   - PCI Local Bus Specification 3.0
//!   - Intel 440FX PCIset Datasheet
//!   - Intel 82371SB (PIIX3) Datasheet

/// PCI config space state
#[derive(Debug, Clone)]
pub struct PciBus {
    /// Currently latched CONFIG_ADDRESS value
    pub config_addr: u32,
    /// Host Bridge config space (256 bytes)
    pub host_bridge: [u8; 256],
    /// ISA Bridge config space (256 bytes)
    pub isa_bridge: [u8; 256],
}

impl Default for PciBus {
    fn default() -> Self {
        let mut bus = Self {
            config_addr: 0,
            host_bridge: [0u8; 256],
            isa_bridge: [0u8; 256],
        };
        bus.init_host_bridge();
        bus.init_isa_bridge();
        bus
    }
}

/// Write a little-endian u16 into a config space buffer
fn write_config_u16(config: &mut [u8], offset: usize, val: u16) {
    let bytes = val.to_le_bytes();
    config[offset] = bytes[0];
    config[offset + 1] = bytes[1];
}

/// Write a little-endian u32 into a config space buffer
fn write_config_u32(config: &mut [u8], offset: usize, val: u32) {
    let bytes = val.to_le_bytes();
    config[offset] = bytes[0];
    config[offset + 1] = bytes[1];
    config[offset + 2] = bytes[2];
    config[offset + 3] = bytes[3];
}

/// Read a little-endian u32 from a config space buffer
fn read_config_u32(config: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        config[offset],
        config[offset + 1],
        config[offset + 2],
        config[offset + 3],
    ])
}

// PCI config space register offsets
mod regs {
    pub const VENDOR_ID: usize      = 0x00; // 16-bit
    pub const DEVICE_ID: usize      = 0x02; // 16-bit
    pub const COMMAND: usize        = 0x04; // 16-bit
    pub const STATUS: usize         = 0x06; // 16-bit
    pub const REVISION_ID: usize    = 0x08; // 8-bit
    pub const PROG_IF: usize        = 0x09; // 8-bit
    pub const SUBCLASS: usize       = 0x0A; // 8-bit
    pub const CLASS_CODE: usize     = 0x0B; // 8-bit
    pub const CACHE_LINE: usize     = 0x0C; // 8-bit
    pub const LATENCY_TIMER: usize  = 0x0D; // 8-bit
    pub const HEADER_TYPE: usize    = 0x0E; // 8-bit
    pub const BIST: usize           = 0x0F; // 8-bit
    pub const BAR0: usize           = 0x10; // 32-bit
    pub const BAR1: usize           = 0x14; // 32-bit
    pub const SUBSYSTEM_VENDOR: usize = 0x2C; // 16-bit
    pub const SUBSYSTEM_ID: usize   = 0x2E; // 16-bit
    pub const CAPABILITIES: usize   = 0x34; // 8-bit (pointer)
    pub const INTERRUPT_LINE: usize = 0x3C; // 8-bit
    pub const INTERRUPT_PIN: usize  = 0x3D; // 8-bit
}

impl PciBus {
    /// Initialize Host Bridge config space (Bus 0, Device 0, Function 0)
    ///
    /// Intel 440FX-compatible:
    ///   Vendor: 0x8086 (Intel)
    ///   Device: 0x1237 (440FX - Natoma)
    ///   Class:  0x06/0x00 (Host Bridge)
    fn init_host_bridge(&mut self) {
        let c = &mut self.host_bridge;
        
        // Vendor ID: Intel
        write_config_u16(c, regs::VENDOR_ID, 0x8086);
        // Device ID: 440FX (Natoma)
        write_config_u16(c, regs::DEVICE_ID, 0x1237);
        // Command: Memory space + I/O space enabled
        write_config_u16(c, regs::COMMAND, 0x0006);
        // Status: Fast back-to-back capable
        write_config_u16(c, regs::STATUS, 0x0000);
        // Revision
        c[regs::REVISION_ID] = 0x02;
        // Class code: Host Bridge
        c[regs::PROG_IF] = 0x00;
        c[regs::SUBCLASS] = 0x00;
        c[regs::CLASS_CODE] = 0x06;
        // Header type 0 (single function)
        c[regs::HEADER_TYPE] = 0x00;
        // Subsystem
        write_config_u16(c, regs::SUBSYSTEM_VENDOR, 0x8086);
        write_config_u16(c, regs::SUBSYSTEM_ID, 0x1237);
    }
    
    /// Initialize ISA Bridge config space (Bus 0, Device 1, Function 0)
    ///
    /// Intel PIIX3-compatible:
    ///   Vendor: 0x8086 (Intel)
    ///   Device: 0x7000 (82371SB PIIX3 ISA)
    ///   Class:  0x06/0x01 (ISA Bridge)
    fn init_isa_bridge(&mut self) {
        let c = &mut self.isa_bridge;
        
        // Vendor ID: Intel
        write_config_u16(c, regs::VENDOR_ID, 0x8086);
        // Device ID: PIIX3 ISA (82371SB)
        write_config_u16(c, regs::DEVICE_ID, 0x7000);
        // Command: I/O + bus master
        write_config_u16(c, regs::COMMAND, 0x0007);
        // Status
        write_config_u16(c, regs::STATUS, 0x0200); // Medium timing
        // Revision
        c[regs::REVISION_ID] = 0x00;
        // Class: ISA Bridge (0x06 / 0x01)
        c[regs::PROG_IF] = 0x00;
        c[regs::SUBCLASS] = 0x01;
        c[regs::CLASS_CODE] = 0x06;
        // Header type 0, multi-function
        c[regs::HEADER_TYPE] = 0x80;
        // Subsystem
        write_config_u16(c, regs::SUBSYSTEM_VENDOR, 0x8086);
        write_config_u16(c, regs::SUBSYSTEM_ID, 0x7000);
    }
    
    /// Handle write to CONFIG_ADDRESS port (0xCF8)
    pub fn write_config_address(&mut self, value: u32) {
        self.config_addr = value;
    }
    
    /// Handle read from CONFIG_ADDRESS port (0xCF8)
    pub fn read_config_address(&self) -> u32 {
        self.config_addr
    }
    
    /// Parse CONFIG_ADDRESS into (enable, bus, device, function, register)
    fn parse_address(&self) -> (bool, u8, u8, u8, u8) {
        let addr = self.config_addr;
        let enable = (addr >> 31) & 1 != 0;
        let bus = ((addr >> 16) & 0xFF) as u8;
        let device = ((addr >> 11) & 0x1F) as u8;
        let function = ((addr >> 8) & 0x7) as u8;
        let register = (addr & 0xFC) as u8; // DWORD aligned
        (enable, bus, device, function, register)
    }
    
    /// Get config space slice for the given BDF, returns None if no device
    fn get_config_space(&self, bus: u8, device: u8, function: u8) -> Option<&[u8; 256]> {
        match (bus, device, function) {
            (0, 0, 0) => Some(&self.host_bridge),
            (0, 1, 0) => Some(&self.isa_bridge),
            _ => None,
        }
    }
    
    /// Get mutable config space slice for the given BDF
    fn get_config_space_mut(&mut self, bus: u8, device: u8, function: u8) -> Option<&mut [u8; 256]> {
        match (bus, device, function) {
            (0, 0, 0) => Some(&mut self.host_bridge),
            (0, 1, 0) => Some(&mut self.isa_bridge),
            _ => None,
        }
    }
    
    /// Handle read from CONFIG_DATA port (0xCFC-0xCFF)
    /// `offset` is the byte offset within the DWORD (0-3)
    pub fn read_config_data(&self, offset: u8) -> u32 {
        let (enable, bus, device, function, register) = self.parse_address();
        
        if !enable {
            return 0xFFFF_FFFF;
        }
        
        if let Some(config) = self.get_config_space(bus, device, function) {
            let reg_offset = (register as usize) + (offset as usize);
            if reg_offset + 4 <= 256 {
                read_config_u32(config, reg_offset & 0xFC) // Always DWORD-aligned read
            } else {
                0xFFFF_FFFF
            }
        } else {
            // No device at this BDF → return all 1s
            0xFFFF_FFFF
        }
    }
    
    /// Handle write to CONFIG_DATA port (0xCFC-0xCFF)
    pub fn write_config_data(&mut self, offset: u8, value: u32) {
        let (enable, bus, device, function, register) = self.parse_address();
        
        if !enable {
            return;
        }
        
        if let Some(config) = self.get_config_space_mut(bus, device, function) {
            let reg_offset = register as usize;
            if reg_offset + 4 <= 256 {
                // Protect read-only fields
                match reg_offset {
                    // Vendor ID, Device ID are read-only
                    0x00 => {}
                    // Status register upper byte is read-only / write-1-to-clear
                    0x04 => {
                        // Command is writable (lower 16 bits)
                        write_config_u16(config, regs::COMMAND, value as u16);
                        // Status: W1C for bits that are set
                        let old_status = u16::from_le_bytes([config[0x06], config[0x07]]);
                        let w1c_mask = (value >> 16) as u16;
                        write_config_u16(config, regs::STATUS, old_status & !w1c_mask);
                    }
                    // Class code, revision are read-only
                    0x08 => {}
                    // Cache line, latency timer are writable; header type, BIST read-only-ish
                    0x0C => {
                        config[regs::CACHE_LINE] = value as u8;
                        config[regs::LATENCY_TIMER] = (value >> 8) as u8;
                    }
                    // BARs: let writes through (for probing)
                    0x10..=0x27 => {
                        write_config_u32(config, reg_offset, value);
                    }
                    // Interrupt line is writable
                    0x3C => {
                        config[regs::INTERRUPT_LINE] = value as u8;
                    }
                    // Everything else: accept writes silently
                    _ => {
                        if reg_offset + 4 <= 256 {
                            write_config_u32(config, reg_offset, value);
                        }
                    }
                }
            }
        }
    }
    
    /// Get a human-readable summary of what's on the bus
    pub fn describe_bus(&self) -> &'static str {
        "PCI Bus 0: [0:0.0] Host Bridge (440FX), [0:1.0] ISA Bridge (PIIX3)"
    }
    
    /// Check if a device exists at the given BDF
    pub fn device_exists(&self, bus: u8, device: u8, function: u8) -> bool {
        self.get_config_space(bus, device, function).is_some()
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pci_bus_defaults() {
        let bus = PciBus::default();
        // Host bridge should exist
        assert!(bus.device_exists(0, 0, 0));
        // ISA bridge should exist
        assert!(bus.device_exists(0, 1, 0));
        // No other device
        assert!(!bus.device_exists(0, 2, 0));
        assert!(!bus.device_exists(1, 0, 0));
    }
    
    #[test]
    fn test_host_bridge_ids() {
        let bus = PciBus::default();
        let vendor = u16::from_le_bytes([bus.host_bridge[0], bus.host_bridge[1]]);
        let device = u16::from_le_bytes([bus.host_bridge[2], bus.host_bridge[3]]);
        assert_eq!(vendor, 0x8086);
        assert_eq!(device, 0x1237);
    }
    
    #[test]
    fn test_isa_bridge_class() {
        let bus = PciBus::default();
        assert_eq!(bus.isa_bridge[regs::CLASS_CODE], 0x06);
        assert_eq!(bus.isa_bridge[regs::SUBCLASS], 0x01);
    }
    
    #[test]
    fn test_config_read_no_device() {
        let mut bus = PciBus::default();
        // Select bus 0, dev 31, fn 0, reg 0 — no device here
        bus.write_config_address(0x8000_F800);
        let val = bus.read_config_data(0);
        assert_eq!(val, 0xFFFF_FFFF);
    }
    
    #[test]
    fn test_config_read_host_bridge() {
        let mut bus = PciBus::default();
        // Select bus 0, dev 0, fn 0, reg 0 (vendor + device ID)
        bus.write_config_address(0x8000_0000);
        let val = bus.read_config_data(0);
        assert_eq!(val & 0xFFFF, 0x8086);       // Vendor ID
        assert_eq!((val >> 16) & 0xFFFF, 0x1237); // Device ID
    }
    
    #[test]
    fn test_config_read_disabled() {
        let mut bus = PciBus::default();
        // Bit 31 not set = disabled
        bus.write_config_address(0x0000_0000);
        let val = bus.read_config_data(0);
        assert_eq!(val, 0xFFFF_FFFF);
    }
    
    #[test]
    fn test_bar_probing() {
        let mut bus = PciBus::default();
        // Select host bridge BAR0
        bus.write_config_address(0x8000_0010);
        // Write all 1s (standard BAR probe)
        bus.write_config_data(0, 0xFFFF_FFFF);
        // Read back — for our host bridge it's just memory
        let val = bus.read_config_data(0);
        // Value should be 0xFFFF_FFFF (no BAR implemented → all writable)
        assert_eq!(val, 0xFFFF_FFFF);
    }
}
