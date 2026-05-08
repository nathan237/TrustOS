//! VirtIO Common Definitions
//!
//! VirtIO is a standardized interface for virtual devices.
//! This module provides the common structures and functions
//! used by all VirtIO device drivers.

use core::sync::atomic::Ordering;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::arch::Port;

/// VirtIO device status bits
pub mod status {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ACKNOWLEDGE: u8 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DRIVER: u8 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DRIVER_OK: u8 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FEATURES_OK: u8 = 8;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEVICE_NEEDS_RESET: u8 = 64;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FAILED: u8 = 128;
}

/// VirtIO PCI capability types
pub mod cap_type {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COMMON_CONFIGURATION: u8 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NOTIFY_CONFIGURATION: u8 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INTERRUPT_HANDLER_CONFIGURATION: u8 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEVICE_CONFIGURATION: u8 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const PCI_CONFIGURATION: u8 = 5;
}

/// Legacy VirtIO PCI registers (offset from BAR0)
pub mod legacy_reg {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEVICE_FEATURES: u16 = 0x00;      // 4 bytes, R
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DRIVER_FEATURES: u16 = 0x04;      // 4 bytes, R/W
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_ADDRESS: u16 = 0x08;        // 4 bytes, R/W (page frame number)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_SIZE: u16 = 0x0C;           // 2 bytes, R
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_SELECT: u16 = 0x0E;         // 2 bytes, R/W
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_NOTIFY: u16 = 0x10;         // 2 bytes, R/W
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEVICE_STATUS: u16 = 0x12;        // 1 byte, R/W
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INTERRUPT_HANDLER_STATUS: u16 = 0x13;           // 1 byte, R
    // Network-specific config starts at 0x14
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NET_MAC: u16 = 0x14;              // 6 bytes, R
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NET_STATUS: u16 = 0x1A;           // 2 bytes, R
}

/// VirtIO ring descriptor flags
pub mod desc_flags {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NEXT: u16 = 1;       // Buffer continues in next descriptor
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const WRITE: u16 = 2;      // Buffer is write-only (for device)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INDIRECT: u16 = 4;   // Buffer contains indirect descriptors
}

/// VirtIO ring descriptor
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct VirtqDesc {
    pub addr: u64,    // Physical address of buffer
    pub len: u32,     // Length of buffer
    pub flags: u16,   // Descriptor flags
    pub next: u16,    // Next descriptor index (if NEXT flag set)
}

/// Available ring header
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug)]
// Structure publique — visible à l'extérieur de ce module.
pub struct VirtqAvail {
    pub flags: u16,
    pub idx: u16,
    // ring: [u16; queue_size] follows
}

/// Used ring element
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct VirtqUsedElement {
    pub id: u32,    // Index of descriptor chain head
    pub len: u32,   // Total bytes written
}

/// Used ring header
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug)]
// Structure publique — visible à l'extérieur de ce module.
pub struct VirtqUsed {
    pub flags: u16,
    pub idx: u16,
    // ring: [VirtqUsedElem; queue_size] follows
}

/// VirtQueue - the heart of VirtIO communication
pub struct Virtqueue {
    /// Queue size (number of descriptors)
    pub size: u16,
    /// Physical base address of the queue
    pub phys_addr: u64,
    /// Virtual address of descriptor table
    pub desc: *mut VirtqDesc,
    /// Virtual address of available ring
    pub avail: *mut VirtqAvail,
    /// Virtual address of used ring
    pub used: *mut VirtqUsed,
    /// Last seen used index
    pub last_used_idx: u16,
    /// Next free descriptor
    pub free_head: u16,
    /// Number of free descriptors
    pub num_free: u16,
    /// Free descriptor list (next pointers)
    pub free_list: Vec<u16>,
}

// SAFETY: The raw pointers in Virtqueue point to DMA memory that is managed
// carefully by the driver. The kernel is single-threaded in practice.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Send for Virtqueue {}
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Sync for Virtqueue {}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Virtqueue {
    /// Calculate queue size in bytes (for allocation)
    pub fn calc_size(queue_size: u16) -> usize {
        let descriptor_size = core::mem::size_of::<VirtqDesc>() * queue_size as usize;
        let avail_size = 6 + 2 * queue_size as usize; // flags + idx + ring + used_event
        let used_size = 6 + 8 * queue_size as usize;  // flags + idx + ring + avail_event
        
        // Align sections properly
        let avail_offset = descriptor_size;
        let used_offset = ((avail_offset + avail_size) + 4095) & !4095; // Page align
        
        used_offset + used_size
    }
    
    /// Allocate a descriptor from the free list
    pub fn alloc_desc(&mut self) -> Option<u16> {
        if self.num_free == 0 {
            return None;
        }
        
        let idx = self.free_head;
        self.free_head = self.free_list[idx as usize];
        self.num_free -= 1;
        Some(idx)
    }
    
    /// Free a descriptor back to the free list
    pub fn free_desc(&mut self, idx: u16) {
        self.free_list[idx as usize] = self.free_head;
        self.free_head = idx;
        self.num_free += 1;
    }
    
    /// Add a buffer to the available ring
    pub     // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn add_available(&mut self, head: u16) {
        let avail = &mut *self.avail;
        let ring_pointer = (self.avail as *mut u8).add(4) as *mut u16;
        let idx = avail.idx;
        *ring_pointer.add((idx % self.size) as usize) = head;
        
        // Memory barrier
        core::sync::atomic::fence(Ordering::Release);
        
        avail.idx = idx.wrapping_add(1);
    }
    

    
    /// Allocate a new virtqueue  
    pub fn new(size: u16) -> Result<Box<Self>, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        
        let total_size = Self::calc_size(size);
        let layout = Layout::from_size_align(total_size, 4096)
            .map_err(|_| "Invalid layout")?;
        
        let ptr = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            return Err("Failed to allocate virtqueue");
        }
        
        let phys_addr = ptr as u64; // Identity mapped in our setup
        
        // Calculate offsets
        let descriptor_size = core::mem::size_of::<VirtqDesc>() * size as usize;
        let avail_offset = descriptor_size;
        let used_offset = ((avail_offset + 6 + 2 * size as usize) + 4095) & !4095;
        
        let desc = ptr as *mut VirtqDesc;
        let avail = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { ptr.add(avail_offset) as *mut VirtqAvail };
        let used = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { ptr.add(used_offset) as *mut VirtqUsed };
        
        // Initialize free list
        let mut free_list = alloc::vec::Vec::with_capacity(size as usize);
        for i in 0..size {
            free_list.push(i + 1);
        }
        if size > 0 {
            free_list[size as usize - 1] = 0;
        }
        
        Ok(Box::new(Self {
            size,
            phys_addr,
            desc,
            avail,
            used,
            last_used_idx: 0,
            free_head: 0,
            num_free: size,
            free_list,
        }))
    }
    
    /// Set a descriptor's fields
    pub fn set_desc(&mut self, idx: u16, addr: u64, len: u32, flags: u16, next: u16) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let desc = &mut *self.desc.add(idx as usize);
            desc.addr = addr;
            desc.len = len;
            desc.flags = flags;
            desc.next = next;
        }
    }
    
    /// Submit a descriptor chain to the available ring
    pub fn submit(&mut self, head: u16) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { self.add_available(head) }
    }
    
    /// Check if there are used entries (safe wrapper)
    pub fn has_used(&self) -> bool {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { 
            let used = &*self.used;
            core::sync::atomic::fence(Ordering::Acquire);
            used.idx != self.last_used_idx
        }
    }
    
    /// Pop a used entry (safe wrapper returning tuple)
    pub fn pop_used(&mut self) -> Option<(u32, u32)> {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let used = &*self.used;
            core::sync::atomic::fence(Ordering::Acquire);
            
            if used.idx == self.last_used_idx {
                return None;
            }
            
            let ring_pointer = (self.used as *mut u8).add(4) as *mut VirtqUsedElement;
            let elem = *ring_pointer.add((self.last_used_idx % self.size) as usize);
            self.last_used_idx = self.last_used_idx.wrapping_add(1);
            
            Some((elem.id, elem.len))
        }
    }
}

/// Legacy VirtIO device (PCI)
pub struct VirtioDevice {
    /// I/O port base
    pub iobase: u16,
    /// Device features
    pub device_features: u32,
    /// Driver features (negotiated)
    pub driver_features: u32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl VirtioDevice {
    /// Create a new legacy VirtIO device
    pub fn new(iobase: u16) -> Self {
        Self {
            iobase,
            device_features: 0,
            driver_features: 0,
        }
    }
    
    /// Read device status
    pub fn read_status(&self) -> u8 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u8>::new(self.iobase + legacy_reg::DEVICE_STATUS);
            port.read()
        }
    }
    
    /// Write device status
    pub fn write_status(&mut self, status: u8) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u8>::new(self.iobase + legacy_reg::DEVICE_STATUS);
            port.write(status);
        }
    }
    
    /// Add status bits
    pub fn add_status(&mut self, bits: u8) {
        let current = self.read_status();
        self.write_status(current | bits);
    }
    
    /// Reset device
    pub fn reset(&mut self) {
        self.write_status(0);
    }
    
    /// Read device features
    pub fn read_device_features(&mut self) -> u32 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u32>::new(self.iobase + legacy_reg::DEVICE_FEATURES);
            self.device_features = port.read();
            self.device_features
        }
    }
    
    /// Write driver features
    pub fn write_driver_features(&mut self, features: u32) {
        self.driver_features = features;
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u32>::new(self.iobase + legacy_reg::DRIVER_FEATURES);
            port.write(features);
        }
    }
    
    /// Select a queue
    pub fn select_queue(&mut self, queue: u16) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u16>::new(self.iobase + legacy_reg::QUEUE_SELECT);
            port.write(queue);
        }
    }
    
    /// Get queue size
    pub fn get_queue_size(&self) -> u16 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u16>::new(self.iobase + legacy_reg::QUEUE_SIZE);
            port.read()
        }
    }
    
    /// Set queue address (physical page frame number)
    pub fn set_queue_address(&mut self, pfn: u32) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u32>::new(self.iobase + legacy_reg::QUEUE_ADDRESS);
            port.write(pfn);
        }
    }
    
    /// Notify the device about a queue
    pub fn notify_queue(&self, queue: u16) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u16>::new(self.iobase + legacy_reg::QUEUE_NOTIFY);
            port.write(queue);
        }
    }
    
    /// Read ISR status
    pub fn read_interrupt_handler(&self) -> u8 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u8>::new(self.iobase + legacy_reg::INTERRUPT_HANDLER_STATUS);
            port.read()
        }
    }
    
    /// Read a byte from device config
    pub fn read_config8(&self, offset: u16) -> u8 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u8>::new(self.iobase + legacy_reg::NET_MAC + offset);
            port.read()
        }
    }
    
    /// Read a u16 from device config
    pub fn read_config16(&self, offset: u16) -> u16 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u16>::new(self.iobase + legacy_reg::NET_MAC + offset);
            port.read()
        }
    }
    
    /// Read a u32 from device config
    pub fn read_config32(&self, offset: u16) -> u32 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let mut port = Port::<u32>::new(self.iobase + legacy_reg::NET_MAC + offset);
            port.read()
        }
    }
}
