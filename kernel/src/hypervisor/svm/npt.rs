//! AMD Nested Page Tables (NPT)
//!
//! NPT is AMD's hardware-assisted paging for VMs,
//! equivalent to Intel's Extended Page Tables (EPT).
//!
//! NPT uses the same page table format as regular x86-64 paging,
//! but with guest physical addresses as input.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// NPT page size (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// NPT 2MB page size
pub const LARGE_PAGE_SIZE: u64 = 2 * 1024 * 1024;

/// NPT 1GB page size
pub const HUGE_PAGE_SIZE: u64 = 1024 * 1024 * 1024;

/// NPT entry flags (same as normal page table flags)
pub mod flags {
    pub const PRESENT: u64 = 1 << 0;
    pub const WRITABLE: u64 = 1 << 1;
    pub const USER: u64 = 1 << 2;
    pub const WRITE_THROUGH: u64 = 1 << 3;
    pub const CACHE_DISABLE: u64 = 1 << 4;
    pub const ACCESSED: u64 = 1 << 5;
    pub const DIRTY: u64 = 1 << 6;
    pub const HUGE_PAGE: u64 = 1 << 7;  // PS bit - 2MB or 1GB page
    pub const GLOBAL: u64 = 1 << 8;
    pub const NO_EXECUTE: u64 = 1 << 63;
    
    /// Default flags for RWX mapping
    pub const RWX: u64 = PRESENT | WRITABLE | USER;
    
    /// Read-only executable
    pub const RX: u64 = PRESENT | USER;
    
    /// Read-write no-execute
    pub const RW: u64 = PRESENT | WRITABLE | USER | NO_EXECUTE;
    
    /// Read-only
    pub const RO: u64 = PRESENT | USER | NO_EXECUTE;
}

/// NPT page table entry
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NptEntry(u64);

impl NptEntry {
    pub const fn empty() -> Self {
        Self(0)
    }
    
    pub const fn new(phys_addr: u64, flags: u64) -> Self {
        Self((phys_addr & 0x000F_FFFF_FFFF_F000) | flags)
    }
    
    #[inline]
    pub fn is_present(&self) -> bool {
        (self.0 & flags::PRESENT) != 0
    }
    
    #[inline]
    pub fn is_huge(&self) -> bool {
        (self.0 & flags::HUGE_PAGE) != 0
    }
    
    #[inline]
    pub fn is_writable(&self) -> bool {
        (self.0 & flags::WRITABLE) != 0
    }
    
    #[inline]
    pub fn is_executable(&self) -> bool {
        (self.0 & flags::NO_EXECUTE) == 0
    }
    
    #[inline]
    pub fn phys_addr(&self) -> u64 {
        self.0 & 0x000F_FFFF_FFFF_F000
    }
    
    #[inline]
    pub fn flags(&self) -> u64 {
        self.0 & 0xFFF0_0000_0000_0FFF
    }
    
    pub fn set(&mut self, phys_addr: u64, flags: u64) {
        self.0 = (phys_addr & 0x000F_FFFF_FFFF_F000) | flags;
    }
    
    pub fn clear(&mut self) {
        self.0 = 0;
    }
    
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// NPT Page Table (512 entries, 4KB aligned)
#[repr(C, align(4096))]
pub struct NptTable {
    entries: [NptEntry; 512],
}

impl NptTable {
    pub const fn new() -> Self {
        Self {
            entries: [NptEntry::empty(); 512],
        }
    }
    
    #[inline]
    pub fn entry(&self, index: usize) -> &NptEntry {
        &self.entries[index]
    }
    
    #[inline]
    pub fn entry_mut(&mut self, index: usize) -> &mut NptEntry {
        &mut self.entries[index]
    }
    
    pub fn phys_addr(&self) -> u64 {
        let virt = self as *const _ as u64;
        virt.wrapping_sub(crate::memory::hhdm_offset())
    }
}

/// NPT Root structure for a VM
pub struct Npt {
    /// PML4 table (root)
    pml4: Box<NptTable>,
    
    /// Allocated page tables (to free on drop)
    tables: Vec<Box<NptTable>>,
    
    /// Guest memory size in bytes
    guest_memory_size: u64,
    
    /// Base host physical address of guest memory
    host_phys_base: u64,
    
    /// ASID for TLB tagging
    asid: u32,
}

impl Npt {
    /// Create a new NPT structure
    pub fn new(asid: u32) -> Self {
        Self {
            pml4: Box::new(NptTable::new()),
            tables: Vec::new(),
            guest_memory_size: 0,
            host_phys_base: 0,
            asid,
        }
    }
    
    /// Get the nCR3 value (physical address of PML4)
    pub fn ncr3(&self) -> u64 {
        self.pml4.phys_addr()
    }
    
    /// Alias for ncr3 (nCR3 = NPT CR3)
    pub fn cr3(&self) -> u64 {
        self.ncr3()
    }
    
    /// Get ASID
    pub fn asid(&self) -> u32 {
        self.asid
    }
    
    /// Map guest physical address range to host physical address range
    /// 
    /// Maps [guest_phys, guest_phys + size) to [host_phys, host_phys + size)
    pub fn map_range(
        &mut self,
        guest_phys: u64,
        host_phys: u64,
        size: u64,
        perms: u64,
    ) -> Result<(), &'static str> {
        let mut offset = 0u64;
        
        while offset < size {
            let gpa = guest_phys + offset;
            let hpa = host_phys + offset;
            let remaining = size - offset;
            
            // Try to use largest possible page size
            if remaining >= HUGE_PAGE_SIZE 
                && (gpa & (HUGE_PAGE_SIZE - 1)) == 0 
                && (hpa & (HUGE_PAGE_SIZE - 1)) == 0 
            {
                self.map_1gb_page(gpa, hpa, perms)?;
                offset += HUGE_PAGE_SIZE;
            } else if remaining >= LARGE_PAGE_SIZE 
                && (gpa & (LARGE_PAGE_SIZE - 1)) == 0 
                && (hpa & (LARGE_PAGE_SIZE - 1)) == 0 
            {
                self.map_2mb_page(gpa, hpa, perms)?;
                offset += LARGE_PAGE_SIZE;
            } else {
                self.map_4kb_page(gpa, hpa, perms)?;
                offset += PAGE_SIZE;
            }
        }
        
        self.guest_memory_size = self.guest_memory_size.max(guest_phys + size);
        if self.host_phys_base == 0 {
            self.host_phys_base = host_phys;
        }
        
        Ok(())
    }
    
    /// Map a 4KB page
    fn map_4kb_page(&mut self, gpa: u64, hpa: u64, perms: u64) -> Result<(), &'static str> {
        let pml4_idx = ((gpa >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((gpa >> 30) & 0x1FF) as usize;
        let pd_idx = ((gpa >> 21) & 0x1FF) as usize;
        let pt_idx = ((gpa >> 12) & 0x1FF) as usize;
        
        // Get or create PDPT
        let pml4_ptr: *mut NptTable = &mut *self.pml4;
        let pdpt_phys = self.ensure_table_at(pml4_ptr, pml4_idx)?;
        let pdpt = unsafe { &mut *((pdpt_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        // Get or create PD
        let pd_phys = self.ensure_table_at(pdpt as *mut _, pdpt_idx)?;
        let pd = unsafe { &mut *((pd_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        // Get or create PT
        let pt_phys = self.ensure_table_at(pd as *mut _, pd_idx)?;
        let pt = unsafe { &mut *((pt_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        // Map the page
        pt.entry_mut(pt_idx).set(hpa, perms);
        
        Ok(())
    }
    
    /// Map a 2MB huge page
    fn map_2mb_page(&mut self, gpa: u64, hpa: u64, perms: u64) -> Result<(), &'static str> {
        let pml4_idx = ((gpa >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((gpa >> 30) & 0x1FF) as usize;
        let pd_idx = ((gpa >> 21) & 0x1FF) as usize;
        
        // Get or create PDPT
        let pml4_ptr: *mut NptTable = &mut *self.pml4;
        let pdpt_phys = self.ensure_table_at(pml4_ptr, pml4_idx)?;
        let pdpt = unsafe { &mut *((pdpt_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        // Get or create PD
        let pd_phys = self.ensure_table_at(pdpt as *mut _, pdpt_idx)?;
        let pd = unsafe { &mut *((pd_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        // Map 2MB page directly in PD
        pd.entry_mut(pd_idx).set(hpa, perms | flags::HUGE_PAGE);
        
        Ok(())
    }
    
    /// Map a 1GB huge page
    fn map_1gb_page(&mut self, gpa: u64, hpa: u64, perms: u64) -> Result<(), &'static str> {
        let pml4_idx = ((gpa >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((gpa >> 30) & 0x1FF) as usize;
        
        // Get or create PDPT
        let pml4_ptr: *mut NptTable = &mut *self.pml4;
        let pdpt_phys = self.ensure_table_at(pml4_ptr, pml4_idx)?;
        let pdpt = unsafe { &mut *((pdpt_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        // Map 1GB page directly in PDPT
        pdpt.entry_mut(pdpt_idx).set(hpa, perms | flags::HUGE_PAGE);
        
        Ok(())
    }
    
    /// Ensure a table exists at the given index in parent, return its physical address
    fn ensure_table_at(&mut self, parent_ptr: *mut NptTable, index: usize) -> Result<u64, &'static str> {
        let parent = unsafe { &mut *parent_ptr };
        
        if !parent.entry(index).is_present() {
            // Create new table
            let new_table = Box::new(NptTable::new());
            let table_phys = new_table.phys_addr();
            self.tables.push(new_table);
            
            // Set parent entry
            parent.entry_mut(index).set(table_phys, flags::PRESENT | flags::WRITABLE | flags::USER);
            Ok(table_phys)
        } else {
            Ok(parent.entry(index).phys_addr())
        }
    }
    
    /// Unmap a guest physical address
    pub fn unmap(&mut self, gpa: u64) -> Result<(), &'static str> {
        let pml4_idx = ((gpa >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((gpa >> 30) & 0x1FF) as usize;
        let pd_idx = ((gpa >> 21) & 0x1FF) as usize;
        let pt_idx = ((gpa >> 12) & 0x1FF) as usize;
        
        // Walk page tables
        if !self.pml4.entry(pml4_idx).is_present() {
            return Ok(());  // Not mapped
        }
        
        let pdpt_phys = self.pml4.entry(pml4_idx).phys_addr();
        let pdpt = unsafe { &mut *((pdpt_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        if !pdpt.entry(pdpt_idx).is_present() {
            return Ok(());
        }
        
        if pdpt.entry(pdpt_idx).is_huge() {
            // 1GB page - unmap it
            pdpt.entry_mut(pdpt_idx).clear();
            return Ok(());
        }
        
        let pd_phys = pdpt.entry(pdpt_idx).phys_addr();
        let pd = unsafe { &mut *((pd_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        if !pd.entry(pd_idx).is_present() {
            return Ok(());
        }
        
        if pd.entry(pd_idx).is_huge() {
            // 2MB page - unmap it
            pd.entry_mut(pd_idx).clear();
            return Ok(());
        }
        
        let pt_phys = pd.entry(pd_idx).phys_addr();
        let pt = unsafe { &mut *((pt_phys + crate::memory::hhdm_offset()) as *mut NptTable) };
        
        // 4KB page - unmap it
        pt.entry_mut(pt_idx).clear();
        
        Ok(())
    }
    
    /// Translate guest physical to host physical address
    pub fn translate(&self, gpa: u64) -> Option<u64> {
        let pml4_idx = ((gpa >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((gpa >> 30) & 0x1FF) as usize;
        let pd_idx = ((gpa >> 21) & 0x1FF) as usize;
        let pt_idx = ((gpa >> 12) & 0x1FF) as usize;
        
        if !self.pml4.entry(pml4_idx).is_present() {
            return None;
        }
        
        let pdpt_phys = self.pml4.entry(pml4_idx).phys_addr();
        let pdpt = unsafe { &*((pdpt_phys + crate::memory::hhdm_offset()) as *const NptTable) };
        
        if !pdpt.entry(pdpt_idx).is_present() {
            return None;
        }
        
        if pdpt.entry(pdpt_idx).is_huge() {
            // 1GB page
            let base = pdpt.entry(pdpt_idx).phys_addr();
            return Some(base | (gpa & (HUGE_PAGE_SIZE - 1)));
        }
        
        let pd_phys = pdpt.entry(pdpt_idx).phys_addr();
        let pd = unsafe { &*((pd_phys + crate::memory::hhdm_offset()) as *const NptTable) };
        
        if !pd.entry(pd_idx).is_present() {
            return None;
        }
        
        if pd.entry(pd_idx).is_huge() {
            // 2MB page
            let base = pd.entry(pd_idx).phys_addr();
            return Some(base | (gpa & (LARGE_PAGE_SIZE - 1)));
        }
        
        let pt_phys = pd.entry(pd_idx).phys_addr();
        let pt = unsafe { &*((pt_phys + crate::memory::hhdm_offset()) as *const NptTable) };
        
        if !pt.entry(pt_idx).is_present() {
            return None;
        }
        
        // 4KB page
        let base = pt.entry(pt_idx).phys_addr();
        Some(base | (gpa & (PAGE_SIZE - 1)))
    }
    
    /// Set up identity mapping for guest (GPA == HPA)
    /// Used for simple guests that expect physical addresses
    pub fn setup_identity_mapping(&mut self, size: u64) -> Result<(), &'static str> {
        self.map_range(0, 0, size, flags::RWX)
    }
    
    /// Set up mapping for guest memory at a specific host physical address
    pub fn setup_guest_memory(
        &mut self,
        host_phys: u64,
        size: u64,
    ) -> Result<(), &'static str> {
        // Map low memory (0 - size) to host physical memory
        self.map_range(0, host_phys, size, flags::RWX)?;
        
        self.host_phys_base = host_phys;
        self.guest_memory_size = size;
        
        Ok(())
    }
    
    /// Invalidate TLB for this NPT (flush by ASID)
    pub fn invalidate_tlb(&self) {
        unsafe {
            super::invlpga(0, self.asid);
        }
    }
    
    /// Get statistics about the NPT
    pub fn stats(&self) -> NptStats {
        NptStats {
            table_count: 1 + self.tables.len(),  // +1 for PML4
            guest_memory_size: self.guest_memory_size,
            host_phys_base: self.host_phys_base,
            asid: self.asid,
        }
    }
}

impl Drop for Npt {
    fn drop(&mut self) {
        // Tables in self.tables will be automatically freed
        // when the Vec is dropped
    }
}

/// NPT statistics
#[derive(Debug)]
pub struct NptStats {
    pub table_count: usize,
    pub guest_memory_size: u64,
    pub host_phys_base: u64,
    pub asid: u32,
}

/// ASID allocator for NPT
pub struct AsidAllocator {
    next_asid: AtomicU64,
    max_asid: u32,
}

impl AsidAllocator {
    pub const fn new(max_asid: u32) -> Self {
        Self {
            next_asid: AtomicU64::new(1),  // ASID 0 is reserved for host
            max_asid,
        }
    }
    
    /// Allocate a new ASID
    pub fn allocate(&self) -> Option<u32> {
        let asid = self.next_asid.fetch_add(1, Ordering::SeqCst);
        if asid as u32 >= self.max_asid {
            // Wrap around (should trigger TLB flush)
            self.next_asid.store(1, Ordering::SeqCst);
            Some(1)
        } else {
            Some(asid as u32)
        }
    }
    
    /// Free an ASID (invalidate its TLB entries)
    pub fn free(&self, asid: u32) {
        // Flush TLB entries for this ASID
        unsafe {
            super::invlpga(0, asid);
        }
    }
}

/// Global ASID allocator
static ASID_ALLOCATOR: AsidAllocator = AsidAllocator::new(65536);

/// Allocate a new ASID
pub fn allocate_asid() -> Option<u32> {
    ASID_ALLOCATOR.allocate()
}

/// Free an ASID
pub fn free_asid(asid: u32) {
    ASID_ALLOCATOR.free(asid);
}

/// Create NPT for a guest with the given memory size
pub fn create_npt_for_guest(guest_memory: &[u8]) -> Result<Npt, &'static str> {
    let asid = allocate_asid().ok_or("Failed to allocate ASID")?;
    let mut npt = Npt::new(asid);
    
    // Get physical address of guest memory
    let guest_virt = guest_memory.as_ptr() as u64;
    let host_phys = guest_virt.wrapping_sub(crate::memory::hhdm_offset());
    
    // Map guest memory starting at GPA 0
    npt.setup_guest_memory(host_phys, guest_memory.len() as u64)?;
    
    Ok(npt)
}
