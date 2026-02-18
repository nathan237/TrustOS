//! EPT (Extended Page Tables)
//!
//! Second Level Address Translation pour la virtualisation mémoire
//! Permet de mapper la mémoire physique du guest vers la mémoire physique réelle
//!
//! All EPT table entries contain PHYSICAL addresses (hardware walks EPT).
//! We use virt_to_phys_vmx() to convert Box pointers to physical addresses.

use alloc::boxed::Box;
use alloc::vec::Vec;
use super::{HypervisorError, Result};
use super::vmx::virt_to_phys_vmx;

/// EPT Pointer format
/// Bits 2:0  = Memory type (0 = UC, 6 = WB)
/// Bits 5:3  = Page-walk length - 1 (3 = 4 levels)
/// Bits 11:6 = Reserved
/// Bits N:12 = Physical address of PML4 table
#[derive(Clone, Copy)]
pub struct EptPointer(pub u64);

impl EptPointer {
    pub fn new(pml4_phys: u64) -> Self {
        // Memory type = WB (6), Page walk length = 4 levels (3)
        let eptp = (pml4_phys & 0xFFFF_FFFF_FFFF_F000) | (3 << 3) | 6;
        EptPointer(eptp)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// EPT Entry flags
pub mod flags {
    pub const READ: u64 = 1 << 0;
    pub const WRITE: u64 = 1 << 1;
    pub const EXECUTE: u64 = 1 << 2;
    pub const MEMORY_TYPE_MASK: u64 = 0x38; // Bits 5:3
    pub const MEMORY_TYPE_WB: u64 = 6 << 3;
    pub const MEMORY_TYPE_UC: u64 = 0 << 3;
    pub const IGNORE_PAT: u64 = 1 << 6;
    pub const LARGE_PAGE: u64 = 1 << 7;
    pub const ACCESSED: u64 = 1 << 8;
    pub const DIRTY: u64 = 1 << 9;
    pub const EXECUTE_USER: u64 = 1 << 10;
}

/// EPT Entry (PML4, PDPT, PD, PT)
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct EptEntry(u64);

impl EptEntry {
    pub const fn empty() -> Self {
        EptEntry(0)
    }
    
    pub fn new_table(table_phys: u64) -> Self {
        EptEntry(table_phys | flags::READ | flags::WRITE | flags::EXECUTE)
    }
    
    pub fn new_page(page_phys: u64, rwx: bool) -> Self {
        let mut entry = page_phys | flags::MEMORY_TYPE_WB;
        if rwx {
            entry |= flags::READ | flags::WRITE | flags::EXECUTE;
        }
        EptEntry(entry)
    }
    
    pub fn new_large_page(page_phys: u64) -> Self {
        EptEntry(page_phys | flags::READ | flags::WRITE | flags::EXECUTE 
                 | flags::LARGE_PAGE | flags::MEMORY_TYPE_WB)
    }
    
    pub fn is_present(&self) -> bool {
        (self.0 & (flags::READ | flags::WRITE | flags::EXECUTE)) != 0
    }
    
    pub fn is_large_page(&self) -> bool {
        (self.0 & flags::LARGE_PAGE) != 0
    }
    
    pub fn phys_addr(&self) -> u64 {
        self.0 & 0xFFFF_FFFF_FFFF_F000
    }
}

/// Table EPT (512 entrées, 4KB alignée)
#[repr(C, align(4096))]
pub struct EptTable {
    entries: [EptEntry; 512],
}

impl EptTable {
    pub fn new() -> Self {
        EptTable {
            entries: [EptEntry::empty(); 512],
        }
    }
    
    pub fn entry(&self, index: usize) -> &EptEntry {
        &self.entries[index]
    }
    
    pub fn entry_mut(&mut self, index: usize) -> &mut EptEntry {
        &mut self.entries[index]
    }
}

/// Gestionnaire EPT pour une VM
pub struct EptManager {
    /// PML4 table (niveau 4)
    pml4: Box<EptTable>,
    /// PDPT tables (niveau 3)
    pdpts: Vec<Box<EptTable>>,
    /// PD tables (niveau 2)
    pds: Vec<Box<EptTable>>,
    /// PT tables (niveau 1)
    pts: Vec<Box<EptTable>>,
    /// Taille mémoire allouée au guest (en bytes)
    guest_memory_size: usize,
}

impl EptManager {
    /// Créer un nouveau gestionnaire EPT pour une VM
    pub fn new(guest_memory_size: usize) -> Result<Self> {
        let mut manager = EptManager {
            pml4: Box::new(EptTable::new()),
            pdpts: Vec::new(),
            pds: Vec::new(),
            pts: Vec::new(),
            guest_memory_size,
        };
        
        // Créer le mapping identité pour la mémoire du guest
        // (GPA == HPA pour simplifier)
        manager.setup_identity_mapping(guest_memory_size)?;
        
        Ok(manager)
    }
    
    /// Obtenir l'EPT pointer (uses physical address of PML4)
    pub fn ept_pointer(&self) -> EptPointer {
        let pml4_virt = self.pml4.as_ref() as *const EptTable as u64;
        let pml4_phys = virt_to_phys_vmx(pml4_virt);
        crate::serial_println!("[EPT] PML4 virt=0x{:016X} phys=0x{:016X}", pml4_virt, pml4_phys);
        EptPointer::new(pml4_phys)
    }
    
    /// Configurer un mapping identité (GPA == HPA)
    fn setup_identity_mapping(&mut self, size: usize) -> Result<()> {
        crate::serial_println!("[EPT] Setting up identity mapping for {} MB", size / (1024 * 1024));
        
        // For simplicity, use 2MB large pages
        let pages_2mb = (size + 0x1FFFFF) / 0x200000;
        let pdpts_needed = ((pages_2mb + 511) / 512).max(1);
        
        for pdpt_idx in 0..pdpts_needed {
            let mut pd = Box::new(EptTable::new());
            
            // Fill PD with 2MB large pages
            let start_page = pdpt_idx * 512;
            for pd_idx in 0..512 {
                let page_num = start_page + pd_idx;
                if page_num >= pages_2mb {
                    break;
                }
                
                let phys_addr = (page_num * 0x200000) as u64;
                pd.entries[pd_idx] = EptEntry::new_large_page(phys_addr);
            }
            
            let pd_virt = pd.as_ref() as *const EptTable as u64;
            let pd_phys = virt_to_phys_vmx(pd_virt);
            self.pds.push(pd);
            
            // Create PDPT entry pointing to PD (physical address!)
            let mut pdpt = Box::new(EptTable::new());
            pdpt.entries[0] = EptEntry::new_table(pd_phys);
            
            let pdpt_virt = pdpt.as_ref() as *const EptTable as u64;
            let pdpt_phys = virt_to_phys_vmx(pdpt_virt);
            self.pdpts.push(pdpt);
            
            // PML4 entry pointing to PDPT (physical address!)
            self.pml4.entries[pdpt_idx] = EptEntry::new_table(pdpt_phys);
        }
        
        crate::serial_println!("[EPT] Identity mapping configured: {} 2MB pages, {} PDPT(s)", 
                              pages_2mb, pdpts_needed);
        
        Ok(())
    }
    
    /// Mapper une page physique du guest vers une page physique de l'host
    pub fn map_page(&mut self, guest_phys: u64, host_phys: u64, _flags: u64) -> Result<()> {
        let pml4_idx = ((guest_phys >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((guest_phys >> 30) & 0x1FF) as usize;
        let pd_idx = ((guest_phys >> 21) & 0x1FF) as usize;
        
        // Ensure PML4 entry exists → PDPT
        if !self.pml4.entries[pml4_idx].is_present() {
            let pdpt = Box::new(EptTable::new());
            let pdpt_virt = pdpt.as_ref() as *const EptTable as u64;
            let pdpt_phys = virt_to_phys_vmx(pdpt_virt);
            self.pml4.entries[pml4_idx] = EptEntry::new_table(pdpt_phys);
            self.pdpts.push(pdpt);
        }
        
        // Get PDPT table via HHDM (physical → virtual conversion)
        let pdpt_table_phys = self.pml4.entries[pml4_idx].phys_addr();
        let pdpt_table_virt = crate::memory::phys_to_virt(pdpt_table_phys);
        let pdpt_table = unsafe { &mut *(pdpt_table_virt as *mut EptTable) };
        
        if !pdpt_table.entries[pdpt_idx].is_present() {
            let pd = Box::new(EptTable::new());
            let pd_virt = pd.as_ref() as *const EptTable as u64;
            let pd_phys = virt_to_phys_vmx(pd_virt);
            pdpt_table.entries[pdpt_idx] = EptEntry::new_table(pd_phys);
            self.pds.push(pd);
        }
        
        let pd_table_phys = pdpt_table.entries[pdpt_idx].phys_addr();
        let pd_table_virt = crate::memory::phys_to_virt(pd_table_phys);
        let pd_table = unsafe { &mut *(pd_table_virt as *mut EptTable) };
        
        // Map as 2MB large page (aligned)
        let aligned_host = host_phys & !0x1FFFFF;
        pd_table.entries[pd_idx] = EptEntry::new_large_page(aligned_host);
        
        Ok(())
    }

    /// Set up mapping from GPA 0 to the host physical address of a guest memory buffer.
    ///
    /// This correctly maps guest physical addresses [0, size) to the host physical
    /// addresses where `guest_memory` actually resides, instead of identity mapping.
    pub fn setup_guest_memory_mapping(&mut self, guest_memory: &[u8]) -> Result<()> {
        let guest_mem_virt = guest_memory.as_ptr() as u64;
        let guest_mem_phys = virt_to_phys_vmx(guest_mem_virt);
        let size = guest_memory.len();
        
        crate::serial_println!("[EPT] Mapping GPA 0x0 -> HPA 0x{:X} ({} MB)",
                              guest_mem_phys, size / (1024 * 1024));
        
        // Clear old identity mapping
        self.pml4 = Box::new(EptTable::new());
        self.pdpts.clear();
        self.pds.clear();
        self.pts.clear();
        
        // Map using 2MB large pages: GPA [0, size) -> HPA [guest_mem_phys, guest_mem_phys + size)
        let pages_2mb = (size + 0x1FFFFF) / 0x200000;
        let pdpts_needed = ((pages_2mb + 511) / 512).max(1);
        
        for pdpt_idx in 0..pdpts_needed {
            let mut pd = Box::new(EptTable::new());
            
            let start_page = pdpt_idx * 512;
            for pd_idx in 0..512 {
                let page_num = start_page + pd_idx;
                if page_num >= pages_2mb {
                    break;
                }
                // GPA offset → HPA = guest_mem_phys + offset
                let host_phys = guest_mem_phys + (page_num * 0x200000) as u64;
                pd.entries[pd_idx] = EptEntry::new_large_page(host_phys);
            }
            
            let pd_virt = pd.as_ref() as *const EptTable as u64;
            let pd_phys = virt_to_phys_vmx(pd_virt);
            self.pds.push(pd);
            
            let mut pdpt = Box::new(EptTable::new());
            pdpt.entries[0] = EptEntry::new_table(pd_phys);
            
            let pdpt_virt = pdpt.as_ref() as *const EptTable as u64;
            let pdpt_phys = virt_to_phys_vmx(pdpt_virt);
            self.pdpts.push(pdpt);
            
            self.pml4.entries[pdpt_idx] = EptEntry::new_table(pdpt_phys);
        }
        
        crate::serial_println!("[EPT] Guest memory mapping: {} 2MB pages, {} PDPT(s)",
                              pages_2mb, pdpts_needed);
        Ok(())
    }
}

/// Créer un EPT minimal pour un guest "Hello World"
pub fn create_minimal_ept(guest_memory: &[u8]) -> Result<EptManager> {
    let size = guest_memory.len().max(4 * 1024 * 1024); // Minimum 4MB
    EptManager::new(size)
}
