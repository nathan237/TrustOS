//! ACPI Table Structures
//!
//! Common structures used by all ACPI tables.

/// RSDP (Root System Description Pointer) - ACPI 1.0
#[repr(C, packed)]
// Public structure — visible outside this module.
pub struct Rsdp {
    /// "RSD PTR " signature
    pub signature: [u8; 8],
    /// Checksum (sum of first 20 bytes must be 0)
    pub checksum: u8,
    /// OEM ID
    pub oem_id: [u8; 6],
    /// ACPI revision (0 = ACPI 1.0, 2 = ACPI 2.0+)
    pub revision: u8,
    /// Physical address of RSDT
    pub rsdt_address: u32,
}

/// XSDP (Extended System Description Pointer) - ACPI 2.0+
#[repr(C, packed)]
// Public structure — visible outside this module.
pub struct Xsdp {
    /// Base RSDP fields
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    
    // Extended fields (ACPI 2.0+)
    /// Length of entire structure
    pub length: u32,
    /// Physical address of XSDT (64-bit)
    pub xsdt_address: u64,
    /// Extended checksum
    pub extended_checksum: u8,
    /// Reserved
    pub reserved: [u8; 3],
}

/// SDT Header (common to all ACPI tables)
#[repr(C, packed)]
// Public structure — visible outside this module.
pub struct SdtHeader {
    /// 4-byte signature (e.g., "APIC", "FACP", "MCFG")
    pub signature: [u8; 4],
    /// Total table length including header
    pub length: u32,
    /// Table revision
    pub revision: u8,
    /// Checksum (sum of all bytes must be 0)
    pub checksum: u8,
    /// OEM ID
    pub oem_id: [u8; 6],
    /// OEM Table ID
    pub oem_table_id: [u8; 8],
    /// OEM Revision
    pub oem_revision: u32,
    /// Creator ID
    pub creator_id: u32,
    /// Creator Revision
    pub creator_revision: u32,
}

// Implementation block — defines methods for the type above.
impl SdtHeader {
    /// Validate table checksum
    pub fn validate(&self) -> bool {
        let ptr = self as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const _ as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8;
        let len = self.length as usize;
        
        if len < core::mem::size_of::<SdtHeader>() {
            return false;
        }
        
        let sum: u8 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            (0..len).map(|i| *ptr.add(i)).fold(0u8, |a, b| a.wrapping_add(b))
        };
        
        sum == 0
    }
    
    /// Get signature as string
    pub fn signature_str(&self) -> &str {
        core::str::from_utf8(&self.signature).unwrap_or("????")
    }
}

/// Generic Address Structure (GAS)
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Debug)]
// Public structure — visible outside this module.
pub struct GenericAddress {
    /// Address space ID
    pub address_space: u8,
    /// Register bit width
    pub bit_width: u8,
    /// Register bit offset
    pub bit_offset: u8,
    /// Access size
    pub access_size: u8,
    /// Address
    pub address: u64,
}

// Implementation block — defines methods for the type above.
impl GenericAddress {
    /// System Memory
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPACE_SYSTEM_MEMORY: u8 = 0;
    /// System I/O
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPACE_SYSTEM_IO: u8 = 1;
    /// PCI Configuration Space
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPACE_PCI_CONFIG: u8 = 2;
    /// Embedded Controller
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPACE_EMBEDDED_CONTROLLER: u8 = 3;
    /// SMBus
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPACE_SMBUS: u8 = 4;
    /// Functional Fixed Hardware
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SPACE_FFH: u8 = 0x7F;
    
    /// Check if this is a valid address
    pub fn is_valid(&self) -> bool {
        self.address != 0
    }
    
    /// Read value from address (I/O or memory)
    pub     // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn read(&self) -> u64 {
                // Pattern matching — Rust's exhaustive branching construct.
match self.address_space {
            Self::SPACE_SYSTEM_IO => {
                let port = self.address as u16;
                                // Pattern matching — Rust's exhaustive branching construct.
match self.bit_width {
                    8 => x86_64::instructions::port::Port::<u8>::new(port).read() as u64,
                    16 => x86_64::instructions::port::Port::<u16>::new(port).read() as u64,
                    32 => x86_64::instructions::port::Port::<u32>::new(port).read() as u64,
                    _ => 0,
                }
            }
            Self::SPACE_SYSTEM_MEMORY => {
                let address = self.address + crate::memory::hhdm_offset();
                                // Pattern matching — Rust's exhaustive branching construct.
match self.bit_width {
                    8 => core::ptr::read_volatile(address as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8) as u64,
                    16 => core::ptr::read_volatile(address as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u16) as u64,
                    32 => core::ptr::read_volatile(address as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32) as u64,
                    64 => core::ptr::read_volatile(address as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u64),
                    _ => 0,
                }
            }
            _ => 0,
        }
    }
    
    /// Write value to address (I/O or memory)
    pub     // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn write(&self, value: u64) {
                // Pattern matching — Rust's exhaustive branching construct.
match self.address_space {
            Self::SPACE_SYSTEM_IO => {
                let port = self.address as u16;
                                // Pattern matching — Rust's exhaustive branching construct.
match self.bit_width {
                    8 => x86_64::instructions::port::Port::<u8>::new(port).write(value as u8),
                    16 => x86_64::instructions::port::Port::<u16>::new(port).write(value as u16),
                    32 => x86_64::instructions::port::Port::<u32>::new(port).write(value as u32),
                    _ => {}
                }
            }
            Self::SPACE_SYSTEM_MEMORY => {
                let address = self.address + crate::memory::hhdm_offset();
                                // Pattern matching — Rust's exhaustive branching construct.
match self.bit_width {
                    8 => core::ptr::write_volatile(address as *mut u8, value as u8),
                    16 => core::ptr::write_volatile(address as *mut u16, value as u16),
                    32 => core::ptr::write_volatile(address as *mut u32, value as u32),
                    64 => core::ptr::write_volatile(address as *mut u64, value),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
