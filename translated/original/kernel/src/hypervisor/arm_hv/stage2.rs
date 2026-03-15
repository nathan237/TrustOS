//! Stage-2 Page Tables for ARM EL2 Hypervisor
//!
//! Stage-2 translation converts the guest's **Intermediate Physical Address (IPA)**
//! to the real **Physical Address (PA)**. This is how we trap MMIO:
//!
//! ```text
//! Guest (EL1):  MOV x1, [0x09000000]    (thinks it's accessing UART)
//!                       ↓
//! Stage-2:      IPA 0x09000000 → FAULT!  (we marked it as trapped)
//!                       ↓  
//! EL2 Handler:  Log "READ 0x09000000", do real read, return value to guest
//!                       ↓
//! Guest:        gets the UART value, never knew we intercepted it
//! ```
//!
//! Page table format: ARMv8 Stage-2 (VTTBR_EL2)
//!   - 4KB granule, 3-level walk (40-bit IPA space = 1TB, enough for any phone)
//!   - Level 1: 1GB blocks or table
//!   - Level 2: 2MB blocks or table
//!   - Level 3: 4KB pages
//!
//! For RAM regions: identity map (IPA == PA), full RWX
//! For MMIO regions we want to spy on: mark as FAULT (no mapping)
//! For MMIO regions we don't care about: identity map as Device memory

use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Stage-2 page table entry access permissions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum S2Perm {
    /// No access — will fault to EL2 (this is how we trap MMIO)
    None,
    /// Read-only
    ReadOnly,
    /// Read-Write
    ReadWrite,
    /// Read-Execute (code)
    ReadExec,
    /// Full access (RWX)
    Full,
}

/// Stage-2 memory type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum S2MemType {
    /// Normal memory (RAM) — cacheable write-back
    Normal,
    /// Device memory (MMIO) — non-cacheable, ordered
    Device,
}

/// A mapping in the Stage-2 tables
#[derive(Debug, Clone)]
pub struct S2Mapping {
    /// IPA base (guest physical address)
    pub ipa_base: u64,
    /// PA base (real physical address)
    pub pa_base: u64,
    /// Size in bytes
    pub size: u64,
    /// Permissions
    pub perm: S2Perm,
    /// Memory type
    pub mem_type: S2MemType,
    /// Human label (for debug)
    pub label: &'static str,
}

/// Stage-2 descriptor bits
mod desc {
    /// Valid bit
    pub const VALID: u64 = 1 << 0;
    /// Table descriptor (vs block) — bit 1
    pub const TABLE: u64 = 1 << 1;
    /// Block descriptor type (level 1 or 2): valid + NOT table
    pub const BLOCK: u64 = VALID;
    /// Page descriptor (level 3): valid + table bit = 0b11
    pub const PAGE: u64  = VALID | TABLE;

    /// Access Flag (AF) — must be set or we get access flag faults
    pub const AF: u64 = 1 << 10;

    // S2AP (Stage-2 Access Permission) bits [7:6]
    /// No access
    pub const S2AP_NONE: u64  = 0b00 << 6;
    /// Read-only
    pub const S2AP_RO: u64    = 0b01 << 6;
    /// Write-only
    pub const S2AP_WO: u64    = 0b10 << 6;
    /// Read-Write
    pub const S2AP_RW: u64    = 0b11 << 6;

    // XN (Execute-Never) bits [54:53] for Stage-2
    /// Execute allowed
    pub const XN_NONE: u64    = 0b00 << 53;
    /// Execute-never from EL0
    pub const XN_EL0: u64     = 0b01 << 53;
    /// Execute-never
    pub const XN_ALL: u64     = 0b10 << 53;

    // MemAttr bits [5:2] for Stage-2
    /// Device-nGnRnE (strongly-ordered MMIO)
    pub const MEMATTR_DEVICE: u64 = 0b0000 << 2;
    /// Normal, Outer Write-Back, Inner Write-Back (RAM)
    pub const MEMATTR_NORMAL_WB: u64 = 0b1111 << 2;
    /// Normal, Non-Cacheable  
    pub const MEMATTR_NORMAL_NC: u64 = 0b0101 << 2;

    /// SH (Shareability) bits [9:8]
    pub const SH_INNER: u64 = 0b11 << 8;
    pub const SH_OUTER: u64 = 0b10 << 8;
    pub const SH_NONE: u64  = 0b00 << 8;
}

/// Size constants
const PAGE_SIZE: u64 = 4096;
const L2_BLOCK_SIZE: u64 = 2 * 1024 * 1024;     // 2MB
const L1_BLOCK_SIZE: u64 = 1024 * 1024 * 1024;   // 1GB

/// Number of entries per table (4KB / 8 bytes)
const ENTRIES_PER_TABLE: usize = 512;

/// Stage-2 page table manager
pub struct Stage2Tables {
    /// L1 table (root) — physical address
    l1_table: *mut u64,
    /// All allocated page table pages (for cleanup)
    allocated_pages: Vec<*mut u64>,
    /// Recorded mappings for introspection
    mappings: Vec<S2Mapping>,
    /// VTTBR_EL2 value (physical address of L1 + VMID)
    vttbr: u64,
}

unsafe impl Send for Stage2Tables {}
unsafe impl Sync for Stage2Tables {}

impl Stage2Tables {
    /// Allocate a new Stage-2 page table set
    pub fn new(vmid: u16) -> Self {
        let l1 = Self::alloc_table();
        let pa = l1 as u64;

        // VTTBR_EL2: [63:48] = VMID, [47:1] = BADDR
        let vttbr = ((vmid as u64) << 48) | (pa & 0x0000_FFFF_FFFF_F000);

        Stage2Tables {
            l1_table: l1,
            allocated_pages: vec![l1],
            mappings: Vec::new(),
            vttbr,
        }
    }

    /// Get VTTBR_EL2 value for this page table
    pub fn vttbr(&self) -> u64 {
        self.vttbr
    }

    /// Identity-map RAM region: IPA == PA, Normal memory, full access
    pub fn map_ram(&mut self, base: u64, size: u64) {
        self.map_region(base, base, size, S2Perm::Full, S2MemType::Normal, "RAM");
    }

    /// Identity-map MMIO pass-through: IPA == PA, Device memory, RW
    /// Guest can access these MMIO regions without trapping
    pub fn map_mmio_passthrough(&mut self, base: u64, size: u64, label: &'static str) {
        self.map_region(base, base, size, S2Perm::ReadWrite, S2MemType::Device, label);
    }

    /// Mark an MMIO region as TRAPPED — any access faults to EL2
    /// This is how the MMIO spy works!
    pub fn trap_mmio(&mut self, base: u64, size: u64, label: &'static str) {
        // Don't map it at all — any access generates a Stage-2 translation fault
        self.mappings.push(S2Mapping {
            ipa_base: base,
            pa_base: base,
            size,
            perm: S2Perm::None,
            mem_type: S2MemType::Device,
            label,
        });
        // No page table entries created = access will fault!
    }

    /// Map a region with specific permissions
    pub fn map_region(
        &mut self,
        ipa_base: u64,
        pa_base: u64,
        size: u64,
        perm: S2Perm,
        mem_type: S2MemType,
        label: &'static str,
    ) {
        self.mappings.push(S2Mapping {
            ipa_base, pa_base, size, perm, mem_type, label,
        });

        if perm == S2Perm::None {
            return; // Trapped — don't create entries
        }

        // Build descriptor attributes
        let attr = self.build_descriptor_attrs(perm, mem_type);

        // Map using 2MB blocks where aligned, 4KB pages otherwise
        let mut ipa = ipa_base & !0xFFF;
        let mut pa = pa_base & !0xFFF;
        let end = ipa_base + size;

        while ipa < end {
            // Try 2MB block if aligned
            if ipa & (L2_BLOCK_SIZE - 1) == 0
                && pa & (L2_BLOCK_SIZE - 1) == 0
                && ipa + L2_BLOCK_SIZE <= end
            {
                self.map_2mb_block(ipa, pa, attr);
                ipa += L2_BLOCK_SIZE;
                pa += L2_BLOCK_SIZE;
            } else {
                self.map_4kb_page(ipa, pa, attr);
                ipa += PAGE_SIZE;
                pa += PAGE_SIZE;
            }
        }
    }

    /// Build Stage-2 descriptor attributes from permissions and memory type
    fn build_descriptor_attrs(&self, perm: S2Perm, mem_type: S2MemType) -> u64 {
        let mut attr: u64 = desc::AF;

        // Access permissions
        attr |= match perm {
            S2Perm::None => desc::S2AP_NONE,
            S2Perm::ReadOnly => desc::S2AP_RO,
            S2Perm::ReadWrite => desc::S2AP_RW,
            S2Perm::ReadExec => desc::S2AP_RO,
            S2Perm::Full => desc::S2AP_RW,
        };

        // Execute permissions
        attr |= match perm {
            S2Perm::ReadExec | S2Perm::Full => desc::XN_NONE,
            _ => desc::XN_ALL,
        };

        // Memory type
        attr |= match mem_type {
            S2MemType::Normal => desc::MEMATTR_NORMAL_WB | desc::SH_INNER,
            S2MemType::Device => desc::MEMATTR_DEVICE | desc::SH_NONE,
        };

        attr
    }

    /// Map a 2MB block (Level 2 descriptor)
    fn map_2mb_block(&mut self, ipa: u64, pa: u64, attr: u64) {
        let l1_idx = ((ipa >> 30) & 0x1FF) as usize;
        let l2_idx = ((ipa >> 21) & 0x1FF) as usize;

        // Get or create L2 table
        let l2 = self.get_or_create_l2(l1_idx);

        // Write 2MB block descriptor
        unsafe {
            let entry = pa & 0x0000_FFFF_FFE0_0000 | attr | desc::BLOCK;
            l2.add(l2_idx).write_volatile(entry);
        }
    }

    /// Map a 4KB page (Level 3 descriptor)
    fn map_4kb_page(&mut self, ipa: u64, pa: u64, attr: u64) {
        let l1_idx = ((ipa >> 30) & 0x1FF) as usize;
        let l2_idx = ((ipa >> 21) & 0x1FF) as usize;
        let l3_idx = ((ipa >> 12) & 0x1FF) as usize;

        let l2 = self.get_or_create_l2(l1_idx);
        let l3 = self.get_or_create_l3(l2, l2_idx);

        // Write 4KB page descriptor
        unsafe {
            let entry = pa & 0x0000_FFFF_FFFF_F000 | attr | desc::PAGE;
            l3.add(l3_idx).write_volatile(entry);
        }
    }

    /// Get or create L2 table pointed to by L1[idx]
    fn get_or_create_l2(&mut self, l1_idx: usize) -> *mut u64 {
        unsafe {
            let entry = self.l1_table.add(l1_idx).read_volatile();
            if entry & desc::VALID != 0 && entry & desc::TABLE != 0 {
                // Already a table pointer
                (entry & 0x0000_FFFF_FFFF_F000) as *mut u64
            } else {
                // Allocate new L2 table
                let l2 = Self::alloc_table();
                self.allocated_pages.push(l2);
                let table_entry = (l2 as u64 & 0x0000_FFFF_FFFF_F000) | desc::VALID | desc::TABLE;
                self.l1_table.add(l1_idx).write_volatile(table_entry);
                l2
            }
        }
    }

    /// Get or create L3 table pointed to by L2[idx]
    fn get_or_create_l3(&mut self, l2: *mut u64, l2_idx: usize) -> *mut u64 {
        unsafe {
            let entry = l2.add(l2_idx).read_volatile();
            if entry & desc::VALID != 0 && entry & desc::TABLE != 0 {
                (entry & 0x0000_FFFF_FFFF_F000) as *mut u64
            } else {
                let l3 = Self::alloc_table();
                self.allocated_pages.push(l3);
                let table_entry = (l3 as u64 & 0x0000_FFFF_FFFF_F000) | desc::VALID | desc::TABLE;
                l2.add(l2_idx).write_volatile(table_entry);
                l3
            }
        }
    }

    /// Allocate a zeroed, 4KB-aligned page for a page table
    fn alloc_table() -> *mut u64 {
        // Use the kernel heap with proper alignment
        let layout = core::alloc::Layout::from_size_align(4096, 4096).unwrap();
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) as *mut u64 };
        if ptr.is_null() {
            panic!("Stage-2 page table allocation failed");
        }
        ptr
    }

    /// Invalidate all Stage-2 TLB entries
    pub fn flush_tlb(&self) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!(
                "dsb ishst",
                "tlbi vmalls12e1is",  // Invalidate all Stage-1 and Stage-2 TLB
                "dsb ish",
                "isb",
                options(nomem, nostack)
            );
        }
    }

    /// Get the list of recorded mappings
    pub fn mappings(&self) -> &[S2Mapping] {
        &self.mappings
    }

    /// Check if an IPA falls in a trapped (spy) region
    pub fn is_trapped_ipa(&self, ipa: u64) -> Option<&S2Mapping> {
        self.mappings.iter().find(|m| {
            m.perm == S2Perm::None
                && ipa >= m.ipa_base
                && ipa < m.ipa_base + m.size
        })
    }
}
