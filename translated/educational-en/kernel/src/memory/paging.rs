//! Paging - 4-Level Page Tables for x86_64
//!
//! Implements memory isolation with separate address spaces per process.
//! Uses 4KB pages with full permission control (R/W/X/User).

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Page size (4 KB)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PAGE_SIZE: usize = 4096;
/// Number of entries per page table
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const ENTRIES_PER_TABLE: usize = 512;

/// Page table entry flags
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
// Public structure — visible outside this module.
pub struct PageFlags(u64);

// Implementation block — defines methods for the type above.
impl PageFlags {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PRESENT: u64 = 1 << 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const WRITABLE: u64 = 1 << 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const USER: u64 = 1 << 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const WRITE_THROUGH: u64 = 1 << 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NO_CACHE: u64 = 1 << 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ACCESSED: u64 = 1 << 5;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DIRTY: u64 = 1 << 6;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUGE_PAGE: u64 = 1 << 7;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GLOBAL: u64 = 1 << 8;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const NO_EXECUTE: u64 = 1 << 63;
    
    /// PAT bit for 4KB pages (bit 7) — selects PAT entry for memory type
    /// Combined with PCD (bit 4) and PWT (bit 3):
    ///   PAT=0, PCD=0, PWT=0 → PAT entry 0 (WB by default)
    ///   PAT=0, PCD=0, PWT=1 → PAT entry 1 (reprogrammed to WC)
    ///   PAT=0, PCD=1, PWT=0 → PAT entry 2 (UC-)
    ///   PAT=0, PCD=1, PWT=1 → PAT entry 3 (UC)
    ///   PAT=1, ... → PAT entries 4-7
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const PAGE_PAT: u64 = 1 << 7;
    
    /// Kernel code: Present + Readable (no write, no user)
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const KERNEL_CODE: Self = Self(Self::PRESENT);
    
    /// Kernel data: Present + Writable (no user) + NX
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const KERNEL_DATA: Self = Self(Self::PRESENT | Self::WRITABLE | Self::NO_EXECUTE);
    
    /// Kernel read-only: Present (no write, no user) + NX
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const KERNEL_RODATA: Self = Self(Self::PRESENT | Self::NO_EXECUTE);
    
    /// User code: Present + User (no write)
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const USER_CODE: Self = Self(Self::PRESENT | Self::USER);
    
    /// User data: Present + Writable + User + NX
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const USER_DATA: Self = Self(Self::PRESENT | Self::WRITABLE | Self::USER | Self::NO_EXECUTE);
    
    /// User read-only: Present + User + NX
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const USER_RODATA: Self = Self(Self::PRESENT | Self::USER | Self::NO_EXECUTE);
    
    pub const fn new(flags: u64) -> Self {
        Self(flags)
    }
    
    pub const fn bits(&self) -> u64 {
        self.0
    }
    
        // Public function — callable from other modules.
pub fn is_present(&self) -> bool {
        self.0 & Self::PRESENT != 0
    }
    
        // Public function — callable from other modules.
pub fn is_writable(&self) -> bool {
        self.0 & Self::WRITABLE != 0
    }
    
        // Public function — callable from other modules.
pub fn is_user(&self) -> bool {
        self.0 & Self::USER != 0
    }
    
        // Public function — callable from other modules.
pub fn is_executable(&self) -> bool {
        self.0 & Self::NO_EXECUTE == 0
    }
}

/// Page table entry
#[derive(Clone, Copy)]
#[repr(transparent)]
// Public structure — visible outside this module.
pub struct PageTableEntry(u64);

// Implementation block — defines methods for the type above.
impl PageTableEntry {
    /// Physical address mask (bits 12-51)
    const ADDRESS_MASK: u64 = 0x000F_FFFF_FFFF_F000;
    
    pub const fn new() -> Self {
        Self(0)
    }
    
        // Public function — callable from other modules.
pub fn set(&mut self, physical_address: u64, flags: PageFlags) {
        self.0 = (physical_address & Self::ADDRESS_MASK) | flags.bits();
    }
    
        // Public function — callable from other modules.
pub fn clear(&mut self) {
        self.0 = 0;
    }
    
        // Public function — callable from other modules.
pub fn physical_address(&self) -> u64 {
        self.0 & Self::ADDRESS_MASK
    }
    
        // Public function — callable from other modules.
pub fn flags(&self) -> PageFlags {
        PageFlags(self.0 & !Self::ADDRESS_MASK)
    }
    
        // Public function — callable from other modules.
pub fn is_present(&self) -> bool {
        self.0 & PageFlags::PRESENT != 0
    }
    
        // Public function — callable from other modules.
pub fn is_unused(&self) -> bool {
        self.0 == 0
    }
}

/// Page table (512 entries, 4KB aligned)
#[repr(align(4096))]
#[repr(C)]
// Public structure — visible outside this module.
pub struct PageTable {
    pub(crate) entries: [PageTableEntry; ENTRIES_PER_TABLE],
}

// Implementation block — defines methods for the type above.
impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); ENTRIES_PER_TABLE],
        }
    }
    
        // Public function — callable from other modules.
pub fn zero(&mut self) {
        for entry in self.entries.iterator_mut() {
            entry.clear();
        }
    }
}

/// Address space for a process
pub struct AddressSpace {
    /// Physical address of PML4 (CR3 value)
    pml4_physical: u64,
    /// Allocated page tables (for cleanup)
    page_tables: Vec<Box<PageTable>>,
    /// HHDM offset for virtual/physical conversion
    hhdm_offset: u64,
}

// Implementation block — defines methods for the type above.
impl AddressSpace {
    /// Create a new empty address space
    pub fn new() -> Option<Self> {
        let hhdm = crate::memory::hhdm_offset();
        
        // Allocate PML4 (top-level page table)
        let mut pml4 = Box::new(PageTable::new());
        pml4.zero();
        
        // Get physical address of PML4
        let pml4_virt = &*pml4 as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable as u64;
        let pml4_physical = pml4_virt.checked_sub(hhdm)?;
        
        let mut page_tables = Vec::new();
        page_tables.push(pml4);
        
        Some(Self {
            pml4_physical,
            page_tables,
            hhdm_offset: hhdm,
        })
    }
    
    /// Create address space that maps kernel memory
    pub fn new_with_kernel() -> Option<Self> {
        // Address space management uses x86_64 page table format (PML4/PDPT/PD/PT).
        // On aarch64, Limine manages page tables and we use kernel space for all
        // processes until proper aarch64 page table support is added.
        #[cfg(not(target_arch = "x86_64"))]
        return None;

        #[cfg(target_arch = "x86_64")]
        {
            let mut space = Self::new()?;
            space.map_kernel_space()?;
            Some(space)
        }
    }
    
    /// Get CR3 value (physical address of PML4)
    pub fn cr3(&self) -> u64 {
        self.pml4_physical
    }
    
    /// Map kernel higher-half space into this address space
    fn map_kernel_space(&mut self) -> Option<()> {
        // Read current CR3
        let current_cr3: u64;
        #[cfg(target_arch = "x86_64")]
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) current_cr3);
        }
        #[cfg(not(target_arch = "x86_64"))]
        { current_cr3 = 0; }
        
        // Get current PML4
        let current_pml4_virt = current_cr3 + self.hhdm_offset;
        let current_pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { 
            &*(current_pml4_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) 
        };
        
        // Get our PML4
        let our_pml4_virt = self.pml4_physical + self.hhdm_offset;
        let our_pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { 
            &mut *(our_pml4_virt as *mut PageTable) 
        };
        
        // Copy the higher half entries (256-511) from kernel
        // This includes HHDM and kernel code
        for i in 256..512 {
            if current_pml4.entries[i].is_present() {
                our_pml4.entries[i] = current_pml4.entries[i];
            }
        }
        
        Some(())
    }
    
    /// Map a single page
    pub fn map_page(&mut self, virt: u64, physical: u64, flags: PageFlags) -> Option<()> {
        // Extract indices for each level
        let pml4_index = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virt >> 30) & 0x1FF) as usize;
        let pd_index = ((virt >> 21) & 0x1FF) as usize;
        let pt_index = ((virt >> 12) & 0x1FF) as usize;
        
        // Walk/create page table hierarchy using raw pointers to avoid borrow conflicts
        let pml4_virt = self.pml4_physical + self.hhdm_offset;
        let pml4 = pml4_virt as *mut PageTable;
        
        // Get or create PDPT
        let pdpt_physical = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { self.ensure_table_at(&mut (*pml4).entries[pml4_index])? };
        let pdpt = (pdpt_physical + self.hhdm_offset) as *mut PageTable;
        
        // Get or create PD
        let pd_physical = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { self.ensure_table_at(&mut (*pdpt).entries[pdpt_index])? };
        let pd = (pd_physical + self.hhdm_offset) as *mut PageTable;
        
        // Get or create PT
        let pt_physical = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { self.ensure_table_at(&mut (*pd).entries[pd_index])? };
        let pt = (pt_physical + self.hhdm_offset) as *mut PageTable;
        
        // Map the page
        unsafe { (*pt).entries[pt_index].set(physical, flags); }
        
        // Invalidate TLB for this virtual address
        #[cfg(target_arch = "x86_64")]
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }
        
        Some(())
    }
    
    /// Ensure a table exists at the given entry, returns physical address
    fn ensure_table_at(&mut self, entry: &mut PageTableEntry) -> Option<u64> {
        if entry.is_present() {
            Some(entry.physical_address())
        } else {
            // Create new table
            let mut new_table = Box::new(PageTable::new());
            new_table.zero();
            
            let table_virt = &*new_table as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable as u64;
            let table_physical = table_virt.checked_sub(self.hhdm_offset)?;
            
            // Set entry with present + writable + user
            entry.set(table_physical, PageFlags::new(
                PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER
            ));
            
            // Keep ownership
            self.page_tables.push(new_table);
            
            Some(table_physical)
        }
    }
    
    
    /// Map a range of pages
    pub fn map_range(&mut self, virt_start: u64, physical_start: u64, size: usize, flags: PageFlags) -> Option<()> {
        let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        
        for i in 0..pages {
            let offset = (i * PAGE_SIZE) as u64;
            self.map_page(virt_start + offset, physical_start + offset, flags)?;
        }
        
        Some(())
    }
    
    /// Unmap a page
    pub fn unmap_page(&mut self, virt: u64) -> Option<()> {
        let pml4_index = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virt >> 30) & 0x1FF) as usize;
        let pd_index = ((virt >> 21) & 0x1FF) as usize;
        let pt_index = ((virt >> 12) & 0x1FF) as usize;
        
        let pml4_virt = self.pml4_physical + self.hhdm_offset;
        let pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(pml4_virt as *mut PageTable) };
        
        if !pml4.entries[pml4_index].is_present() {
            return None;
        }
        
        let pdpt_virt = pml4.entries[pml4_index].physical_address() + self.hhdm_offset;
        let pdpt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(pdpt_virt as *mut PageTable) };
        
        if !pdpt.entries[pdpt_index].is_present() {
            return None;
        }
        
        let pd_virt = pdpt.entries[pdpt_index].physical_address() + self.hhdm_offset;
        let pd = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(pd_virt as *mut PageTable) };
        
        if !pd.entries[pd_index].is_present() {
            return None;
        }
        
        let pt_virt = pd.entries[pd_index].physical_address() + self.hhdm_offset;
        let pt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(pt_virt as *mut PageTable) };
        
        pt.entries[pt_index].clear();
        
        // Invalidate TLB for this page
        #[cfg(target_arch = "x86_64")]
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }
        
        Some(())
    }
    
    /// Translate a virtual address to its physical address by walking the page tables.
    /// Returns `Some(phys)` with the physical address of the start of the 4K page + page offset,
    /// or `None` if the page is not mapped.
    pub fn translate(&self, virt: u64) -> Option<u64> {
        let pml4_index = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virt >> 30) & 0x1FF) as usize;
        let pd_index = ((virt >> 21) & 0x1FF) as usize;
        let pt_index = ((virt >> 12) & 0x1FF) as usize;
        let page_offset = virt & 0xFFF;
        
        let pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((self.pml4_physical + self.hhdm_offset) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        if !pml4.entries[pml4_index].is_present() { return None; }
        
        let pdpt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pml4.entries[pml4_index].physical_address() + self.hhdm_offset) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        if !pdpt.entries[pdpt_index].is_present() { return None; }
        
        let pd = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pdpt.entries[pdpt_index].physical_address() + self.hhdm_offset) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        if !pd.entries[pd_index].is_present() { return None; }
        
        let pt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pd.entries[pd_index].physical_address() + self.hhdm_offset) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        if !pt.entries[pt_index].is_present() { return None; }
        
        Some(pt.entries[pt_index].physical_address() + page_offset)
    }
    
    /// Switch to this address space
    pub     // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn activate(&self) {
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!(
            "mov cr3, {}",
            in(reg) self.pml4_physical,
            options(nostack, preserves_flags)
        );
    }
    
    /// Check if a virtual address is mapped and accessible with given flags
    pub fn is_accessible(&self, virt: u64, required_flags: PageFlags) -> bool {
        let pml4_index = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virt >> 30) & 0x1FF) as usize;
        let pd_index = ((virt >> 21) & 0x1FF) as usize;
        let pt_index = ((virt >> 12) & 0x1FF) as usize;
        
        let pml4_virt = self.pml4_physical + self.hhdm_offset;
        let pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(pml4_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        
        if !pml4.entries[pml4_index].is_present() {
            return false;
        }
        
        let pdpt_virt = pml4.entries[pml4_index].physical_address() + self.hhdm_offset;
        let pdpt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(pdpt_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        
        if !pdpt.entries[pdpt_index].is_present() {
            return false;
        }
        
        let pd_virt = pdpt.entries[pdpt_index].physical_address() + self.hhdm_offset;
        let pd = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(pd_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        
        if !pd.entries[pd_index].is_present() {
            return false;
        }
        
        let pt_virt = pd.entries[pd_index].physical_address() + self.hhdm_offset;
        let pt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(pt_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        
        if !pt.entries[pt_index].is_present() {
            return false;
        }
        
        let entry_flags = pt.entries[pt_index].flags();
        
        // Check required flags
        if required_flags.is_writable() && !entry_flags.is_writable() {
            return false;
        }
        if required_flags.is_user() && !entry_flags.is_user() {
            return false;
        }
        
        true
    }
}

// Implementation block — defines methods for the type above.
impl AddressSpace {
    /// Walk the lower-half page tables (user space, PML4 entries 0–255) and
    /// return every physical frame backing a leaf 4 KB page to the frame
    /// allocator.  Intermediate page-table pages are owned by
    /// `self.page_tables` and will be freed via `Box::drop` as usual.
    pub fn release_user_frames(&self) -> usize {
        let hhdm = self.hhdm_offset;
        let pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((self.pml4_physical + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
        let mut freed = 0usize;

        for pml4_index in 0..256 {
            if !pml4.entries[pml4_index].is_present() { continue; }
            let pdpt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pml4.entries[pml4_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };

            for pdpt_index in 0..ENTRIES_PER_TABLE {
                if !pdpt.entries[pdpt_index].is_present() { continue; }
                // Skip huge 1 GB pages (unlikely, but guard)
                if pdpt.entries[pdpt_index].flags().0 & PageFlags::HUGE_PAGE != 0 { continue; }
                let pd = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pdpt.entries[pdpt_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };

                for pd_index in 0..ENTRIES_PER_TABLE {
                    if !pd.entries[pd_index].is_present() { continue; }
                    // Skip huge 2 MB pages
                    if pd.entries[pd_index].flags().0 & PageFlags::HUGE_PAGE != 0 { continue; }
                    let pt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pd.entries[pd_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };

                    for pt_index in 0..ENTRIES_PER_TABLE {
                        if !pt.entries[pt_index].is_present() { continue; }
                        let physical = pt.entries[pt_index].physical_address();
                        crate::memory::frame::free_frame(physical);
                        freed += 1;
                    }
                }
            }
        }
        freed
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Drop for AddressSpace {
    fn drop(&mut self) {
        // Free all physical frames backing user-space pages
        let freed = self.release_user_frames();
        if freed > 0 {
            crate::log_debug!("[PAGING] Dropped address space: freed {} user frames ({} KB)",
                freed, freed * 4);
        }
        // Page table structures (Vec<Box<PageTable>>) are freed automatically
    }
}

/// Kernel address space (shared across all processes)
static KERNEL_CR3: AtomicU64 = AtomicU64::new(0);

/// Initialize paging subsystem
pub fn init() {
    // Save current page table base for kernel space
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
    }
    #[cfg(target_arch = "aarch64")]
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        // On aarch64, TTBR1_EL1 holds the kernel page table base
        core::arch::asm!("mrs {}, TTBR1_EL1", out(reg) cr3, options(nomem, nostack));
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    { cr3 = 0; }
    KERNEL_CR3.store(cr3, Ordering::SeqCst);
    
    // Enable NX bit so page table NX flags are enforced
    enable_nx();
    
    crate::log_debug!("Paging initialized, kernel CR3: {:#x}, NX enabled", cr3);
}

/// Enable NX (No-Execute) bit via EFER MSR
fn enable_nx() {
    #[cfg(target_arch = "x86_64")]
    {
                // Compile-time constant — evaluated at compilation, zero runtime cost.
const IA32_EFER: u32 = 0xC0000080;
                // Compile-time constant — evaluated at compilation, zero runtime cost.
const NXE_BIT: u64 = 1 << 11;
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            // Read EFER - rdmsr puts result in EDX:EAX
            let eax: u32;
            let edx: u32;
            core::arch::asm!(
                "rdmsr",
                in("ecx") IA32_EFER,
                out("eax") eax,
                out("edx") edx,
            );
            let efer = ((edx as u64) << 32) | (eax as u64);
            
            // Set NXE bit
            let new_efer = efer | NXE_BIT;
            let low = new_efer as u32;
            let high = (new_efer >> 32) as u32;
            
            core::arch::asm!(
                "wrmsr",
                in("ecx") IA32_EFER,
                in("eax") low,
                in("edx") high,
            );
        }
    }
}

/// Get kernel CR3 value
pub fn kernel_cr3() -> u64 {
    KERNEL_CR3.load(Ordering::Relaxed)
}

/// Check if an address is in user space (lower half)
pub fn is_user_address(address: u64) -> bool {
    address < 0x0000_8000_0000_0000
}

/// Check if an address is in kernel space (higher half)
pub fn is_kernel_address(address: u64) -> bool {
    address >= 0xFFFF_8000_0000_0000
}

/// Validate a user pointer (returns true if safe to access)
pub fn validate_user_pointer(ptr: u64, len: usize, write: bool) -> bool {
    // Check it's in user space
    if !is_user_address(ptr) {
        return false;
    }
    
    // Check end doesn't overflow into kernel space
    let end = ptr.saturating_add(len as u64);
    if !is_user_address(end) {
        return false;
    }
    
    // Zero-length pointers are always valid if the address is in user space
    if len == 0 {
        return true;
    }
    
    // Walk the current page tables to verify every page in the range is mapped
    // with the correct permissions (USER, and WRITABLE if write=true).
    let hhdm = crate::memory::hhdm_offset();
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    
    let required_flags = if write {
        PageFlags::new(PageFlags::PRESENT | PageFlags::USER | PageFlags::WRITABLE)
    } else {
        PageFlags::new(PageFlags::PRESENT | PageFlags::USER)
    };
    
    // Check each page in the range
    let start_page = ptr & !0xFFF;
    let end_page = (end.saturating_sub(1)) & !0xFFF;
    let mut page = start_page;
    
        // Infinite loop — runs until an explicit `break`.
loop {
        if !check_page_flags(cr3, hhdm, page, required_flags) {
            return false;
        }
        if page >= end_page {
            break;
        }
        page += 0x1000;
    }
    
    true
}

/// Check that a single virtual address has the required flags set in the page tables
fn check_page_flags(cr3: u64, hhdm: u64, virt: u64, required: PageFlags) -> bool {
    let pml4_index = ((virt >> 39) & 0x1FF) as usize;
    let pdpt_index = ((virt >> 30) & 0x1FF) as usize;
    let pd_index   = ((virt >> 21) & 0x1FF) as usize;
    let pt_index   = ((virt >> 12) & 0x1FF) as usize;
    
    let pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((cr3 + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
    if !pml4.entries[pml4_index].is_present() { return false; }
    
    let pdpt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pml4.entries[pml4_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
    if !pdpt.entries[pdpt_index].is_present() { return false; }
    
    let pd = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pdpt.entries[pdpt_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
    if !pd.entries[pd_index].is_present() { return false; }
    
    let pt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pd.entries[pd_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
    if !pt.entries[pt_index].is_present() { return false; }
    
    let flags = pt.entries[pt_index].flags();
    if required.is_user() && !flags.is_user() { return false; }
    if required.is_writable() && !flags.is_writable() { return false; }
    
    true
}

/// User memory region for allocating userspace memory
pub struct UserMemoryRegion {
    pub start: u64,
    pub end: u64,
    pub next_allocator: u64,
}

// Implementation block — defines methods for the type above.
impl UserMemoryRegion {
    /// Standard user space regions
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CODE_START: u64 = 0x0000_0000_0040_0000;    // 4 MB
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CODE_END: u64 = 0x0000_0000_1000_0000;      // 256 MB
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HEAP_START: u64 = 0x0000_0000_1000_0000;    // 256 MB
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HEAP_END: u64 = 0x0000_0000_8000_0000;      // 2 GB
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const STACK_TOP: u64 = 0x0000_7FFF_FFFF_0000;     // Near top of user space
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const STACK_SIZE: u64 = 0x0000_0000_0010_0000;    // 1 MB stack
}

/// Map a single MMIO page into the kernel's current page tables
/// Uses flags appropriate for MMIO: present, writable, no-cache, no-execute
pub fn map_kernel_mmio_page(virt: u64, physical: u64) -> Result<(), &'static str> {
    use alloc::boxed::Box;
    
    let hhdm = crate::memory::hhdm_offset();
    
    // Extract indices for each level
    let pml4_index = ((virt >> 39) & 0x1FF) as usize;
    let pdpt_index = ((virt >> 30) & 0x1FF) as usize;
    let pd_index = ((virt >> 21) & 0x1FF) as usize;
    let pt_index = ((virt >> 12) & 0x1FF) as usize;
    
    // Read current CR3 to get kernel's PML4
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
    }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    
    // Access PML4 via HHDM
    let pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((cr3 + hhdm) as *mut PageTable) };
    
    // Get or create PDPT
    let pdpt = if pml4.entries[pml4_index].is_present() {
        let pdpt_physical = pml4.entries[pml4_index].physical_address();
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((pdpt_physical + hhdm) as *mut PageTable) }
    } else {
        // PML4 entry missing — PCI MMIO regions aren't covered by Limine HHDM.
        // Allocate a new PDPT to extend the mapping.
        crate::serial_println!("[MMIO] Creating PDPT for PML4[{}] (phys={:#x})", pml4_index, physical);
        let new_pdpt = Box::new(PageTable::new());
        let pdpt_virt = Box::into_raw(new_pdpt) as u64;
        let pdpt_physical = pdpt_virt.checked_sub(hhdm).ok_or("Cannot convert PDPT virt to phys")?;
        
        let flags = PageFlags::new(PageFlags::PRESENT | PageFlags::WRITABLE);
        pml4.entries[pml4_index].set(pdpt_physical, flags);
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(pdpt_virt as *mut PageTable) }
    };
    
    // Get or create PD
    let pd = if pdpt.entries[pdpt_index].is_present() {
        let pd_physical = pdpt.entries[pdpt_index].physical_address();
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((pd_physical + hhdm) as *mut PageTable) }
    } else {
        // Need to create PD - allocate a new page table
        crate::serial_println!("[MMIO] Creating PD for PDPT[{}]", pdpt_index);
        let new_pd = Box::new(PageTable::new());
        let pd_virt = Box::into_raw(new_pd) as u64;
        let pd_physical = pd_virt.checked_sub(hhdm).ok_or("Cannot convert PD virt to phys")?;
        
        // Set PDPT entry
        let flags = PageFlags::new(PageFlags::PRESENT | PageFlags::WRITABLE);
        pdpt.entries[pdpt_index].set(pd_physical, flags);
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(pd_virt as *mut PageTable) }
    };
    
    // Get or create PT
    let pt = if pd.entries[pd_index].is_present() {
        // Check if it's a huge page (2MB)
        if pd.entries[pd_index].flags().0 & PageFlags::HUGE_PAGE != 0 {
            // Already mapped as 2MB page, MMIO access should work
            return Ok(());
        }
        let pt_physical = pd.entries[pd_index].physical_address();
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((pt_physical + hhdm) as *mut PageTable) }
    } else {
        // Need to create PT - allocate a new page table
        crate::serial_println!("[MMIO] Creating PT for PD[{}]", pd_index);
        let new_pt = Box::new(PageTable::new());
        let pt_virt = Box::into_raw(new_pt) as u64;
        let pt_physical = pt_virt.checked_sub(hhdm).ok_or("Cannot convert PT virt to phys")?;
        
        // Set PD entry
        let flags = PageFlags::new(PageFlags::PRESENT | PageFlags::WRITABLE);
        pd.entries[pd_index].set(pt_physical, flags);
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *(pt_virt as *mut PageTable) }
    };
    
    // Check if already mapped
    if pt.entries[pt_index].is_present() {
        // Page already mapped - check if it's the same physical address
        let existing_physical = pt.entries[pt_index].physical_address();
        if existing_physical == (physical & !0xFFF) {
            // Same page, already mapped
            return Ok(());
        }
        // Different mapping exists - update it for MMIO
        crate::serial_println!("[MMIO] Updating existing mapping at PT[{}]", pt_index);
    }
    
    // Set MMIO page entry: Present + Writable + No-Cache + No-Execute
    let mmio_flags = PageFlags::new(
        PageFlags::PRESENT | 
        PageFlags::WRITABLE | 
        PageFlags::NO_CACHE | 
        PageFlags::WRITE_THROUGH |
        PageFlags::NO_EXECUTE
    );
    
    pt.entries[pt_index].set(physical & !0xFFF, mmio_flags);
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// PAT (Page Attribute Table) — Write-Combining support
// ═══════════════════════════════════════════════════════════════════════════════
//
// Every GPU driver (Mesa, NVIDIA, AMD) uses PAT to set framebuffer memory type
// to Write-Combining (WC). WC batches individual writes into 64-byte burst
// transfers, giving 10-20x throughput vs UC (Uncacheable).
//
// Default x86 PAT entries:
//   0: WB (Write-Back)      1: WT (Write-Through)
//   2: UC- (Uncacheable-)    3: UC (Uncacheable)
//   4: WB                    5: WT        
//   6: UC-                   7: UC
//
// We reprogram entry 1 (PWT=1, PCD=0, PAT=0) from WT → WC (0x01)
// Then any page with PWT=1, PCD=0 gets Write-Combining behavior.

const IA32_PAT_MSR: u32 = 0x277;

/// Memory type values for PAT entries
const PAT_WB: u8 = 0x06;  // Write-Back (default)
const PAT_WT: u8 = 0x04;  // Write-Through  
const PAT_UC: u8 = 0x00;  // Uncacheable
const PAT_WC: u8 = 0x01;  // Write-Combining ← what we want for framebuffer

/// Setup PAT with Write-Combining in entry 1
/// Call once during early boot, before any WC mappings
pub fn setup_pat_write_combining() {
    #[cfg(target_arch = "x86_64")]
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        // Read current PAT MSR
        let pat_lo: u32;
        let pat_hi: u32;
        core::arch::asm!(
            "rdmsr",
            in("ecx") IA32_PAT_MSR,
            out("eax") pat_lo,
            out("edx") pat_hi,
        );
        
        let old_pat = ((pat_hi as u64) << 32) | (pat_lo as u64);
        
        // Build new PAT: replace entry 1 (bits 15:8) with WC (0x01)
        // Entry layout: each entry is 8 bits, entries 0-3 in low dword, 4-7 in high
        let new_pat = (old_pat & !0x0000_0000_0000_FF00) | ((PAT_WC as u64) << 8);
        
        let new_lo = new_pat as u32;
        let new_hi = (new_pat >> 32) as u32;
        
        // Write new PAT MSR
        core::arch::asm!(
            "wrmsr",
            in("ecx") IA32_PAT_MSR,
            in("eax") new_lo,
            in("edx") new_hi,
        );
        
        // Flush TLB to apply new PAT
        core::arch::asm!(
            "mov {tmp}, cr3",
            "mov cr3, {tmp}",
            temporary = out(reg) _,
        );
        
        crate::serial_println!(
            "[PAT] Write-Combining enabled: PAT[1]=WC (was {:#04x}, now {:#04x})",
            (old_pat >> 8) & 0xFF,
            PAT_WC
        );
    }
}

/// Remap a region as Write-Combining (WC) for optimal framebuffer/VRAM writes
/// The region must already be mapped. This function updates the page flags
/// to select PAT entry 1 (WC) by setting PWT=1, PCD=0, PAT_bit=0.
///
/// This is how Mesa, NVIDIA, and AMD drivers map GPU BARs and framebuffers.
pub fn remap_region_write_combining(virt_start: u64, size_bytes: usize) -> Result<usize, &'static str> {
    let hhdm = crate::memory::hhdm_offset();
    let number_pages = (size_bytes + PAGE_SIZE - 1) / PAGE_SIZE;
    let mut remapped = 0usize;
    
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3); }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    
    for page_index in 0..number_pages {
        let virt = virt_start + (page_index * PAGE_SIZE) as u64;
        
        let pml4_index = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_index = ((virt >> 30) & 0x1FF) as usize;
        let pd_index = ((virt >> 21) & 0x1FF) as usize;
        let pt_index = ((virt >> 12) & 0x1FF) as usize;
        
        let pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((cr3 + hhdm) as *mut PageTable) };
        if !pml4.entries[pml4_index].is_present() { continue; }
        
        let pdpt_physical = pml4.entries[pml4_index].physical_address();
        let pdpt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((pdpt_physical + hhdm) as *mut PageTable) };
        if !pdpt.entries[pdpt_index].is_present() { continue; }
        
        // Check for 1GB huge page
        if pdpt.entries[pdpt_index].flags().0 & PageFlags::HUGE_PAGE != 0 { continue; }
        
        let pd_physical = pdpt.entries[pdpt_index].physical_address();
        let pd = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((pd_physical + hhdm) as *mut PageTable) };
        if !pd.entries[pd_index].is_present() { continue; }
        
        // Check for 2MB huge page
        if pd.entries[pd_index].flags().0 & PageFlags::HUGE_PAGE != 0 { continue; }
        
        let pt_physical = pd.entries[pd_index].physical_address();
        let pt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((pt_physical + hhdm) as *mut PageTable) };
        if !pt.entries[pt_index].is_present() { continue; }
        
        // Get current entry
        let physical_address = pt.entries[pt_index].physical_address();
        let old_flags = pt.entries[pt_index].flags().0;
        
        // Set WC: PWT=1, clear PCD and PAT bit (selects PAT entry 1 = WC)
        let new_flags = (old_flags & !(PageFlags::NO_CACHE | PageFlags::PAGE_PAT))
            | PageFlags::WRITE_THROUGH;  // PWT=1, PCD=0, PAT=0 → entry 1 (WC)
        
        pt.entries[pt_index].set(physical_address, PageFlags::new(new_flags));
        
        // Invalidate this TLB entry
        #[cfg(target_arch = "x86_64")]
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
        }
        
        remapped += 1;
    }
    
    crate::serial_println!(
        "[PAT] Remapped {} pages as Write-Combining @ {:#x} ({} KB)",
        remapped, virt_start, (remapped * PAGE_SIZE) / 1024
    );
    
    Ok(remapped)
}

