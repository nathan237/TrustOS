//! VirtIO GPU Driver — Real Hardware Implementation
//!
//! Implements VirtIO GPU 2D acceleration using the modern PCI transport.
//! Provides DMA-based framebuffer transfers for efficient display updates.
//!
//! Architecture:
//! - CPU renders to a backing buffer (same as before)
//! - VirtIO GPU transfers the buffer to the host GPU via DMA
//! - Host GPU displays the resource on a scanout
//!
//! Reference: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html

use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use crate::pci::{self, PciDevice};
use crate::memory;

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO GPU Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// VirtIO GPU PCI device ID (modern, VirtIO 1.0+)
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VIRTIO_GPU_PCI_DEVICE_ID: u16 = 0x1050;
/// VirtIO vendor ID
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VIRTIO_VENDOR_ID: u16 = 0x1AF4;

/// VirtIO PCI capability types
pub mod virtio_cap {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const COMMON_CONFIGURATION: u8 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NOTIFY_CONFIGURATION: u8 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INTERRUPT_HANDLER_CONFIGURATION: u8 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEVICE_CONFIGURATION: u8 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _PCI_CONFIGURATION: u8 = 5;
}

/// VirtIO device status bits
pub mod dev_status {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ACKNOWLEDGE: u8 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DRIVER: u8 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DRIVER_OK: u8 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FEATURES_OK: u8 = 8;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _DEVICE_NEEDS_RESET: u8 = 64;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FAILED: u8 = 128;
}

/// VirtIO GPU feature bits
pub mod features {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _VIRTIO_GPU_F_VIRGL: u64 = 1 << 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VIRTIO_GPU_F_EDID: u64 = 1 << 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _VIRTIO_GPU_F_RESOURCE_UUID: u64 = 1 << 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _VIRTIO_GPU_F_RESOURCE_BLOB: u64 = 1 << 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VIRTIO_F_VERSION_1: u64 = 1 << 32;
}

/// VirtIO GPU control command types
#[repr(u32)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum GpuCtrlType {
    // 2D commands
    CommandGetDisplayInformation = 0x0100,
    CommandResourceCreate2d = 0x0101,
    CommandResourceUnref = 0x0102,
    CommandSetScanout = 0x0103,
    CommandResourceFlush = 0x0104,
    CommandTransferToHost2d = 0x0105,
    CommandResourceAttachBacking = 0x0106,
    CommandResourceDetachBacking = 0x0107,
    CommandGetCapsetInformation = 0x0108,
    CommandGetCapset = 0x0109,
    CommandGetEdid = 0x010a,

    // 3D commands (VIRGL — Upgrade #5)
    CommandContextCreate = 0x0200,
    CommandContextDestroy = 0x0201,
    CommandContextAttachResource = 0x0202,
    CommandContextDetachResource = 0x0203,
    CommandResourceCreate3d = 0x0204,
    CommandTransferToHost3d = 0x0205,
    CommandTransferFromHost3d = 0x0206,
    CommandSubmit3d = 0x0207,

    // Cursor commands
    CommandUpdateCursor = 0x0300,
    CommandMoveCursor = 0x0301,

    // Success responses
    ResponseOkNodata = 0x1100,
    ResponseOkDisplayInformation = 0x1101,
    ResponseOkCapsetInformation = 0x1102,
    ResponseOkCapset = 0x1103,
    ResponseOkEdid = 0x1104,

    // Error responses
    ResponseErrorUnspec = 0x1200,
    ResponseErrorOutOfMemory = 0x1201,
    ResponseErrorInvalidScanoutId = 0x1202,
    ResponseErrorInvalidResourceId = 0x1203,
    ResponseErrorInvalidContextId = 0x1204,
    ResponseErrorInvalidParameter = 0x1205,
}

/// Pixel formats
#[repr(u32)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum GpuFormat {
    B8G8R8A8Unorm = 1,
    B8G8R8X8Unorm = 2,
    A8R8G8B8Unorm = 3,
    X8R8G8B8Unorm = 4,
    R8G8B8A8Unorm = 67,
    X8B8G8R8Unorm = 68,
    A8B8G8R8Unorm = 121,
    R8G8B8X8Unorm = 134,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO GPU Protocol Structures (all repr(C) for DMA)
// ═══════════════════════════════════════════════════════════════════════════════

/// Control header — prefix for all commands and responses
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuControllerHeader {
    pub controller_type: u32,
    pub flags: u32,
    pub fence_id: u64,
    pub context_id: u32,
    pub ring_index: u8,
    pub padding: [u8; 3],
}

/// Rectangle
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Display info for one scanout
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuDisplayOne {
    pub r: GpuRect,
    pub enabled: u32,
    pub flags: u32,
}

/// Response to CMD_GET_DISPLAY_INFO
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuResponseDisplayInformation {
    pub header: GpuControllerHeader,
    pub pmodes: [GpuDisplayOne; 16],
}

/// CMD_RESOURCE_CREATE_2D
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuResourceCreate2d {
    pub header: GpuControllerHeader,
    pub resource_id: u32,
    pub format: u32,
    pub width: u32,
    pub height: u32,
}

/// CMD_SET_SCANOUT
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuSetScanout {
    pub header: GpuControllerHeader,
    pub r: GpuRect,
    pub scanout_id: u32,
    pub resource_id: u32,
}

/// CMD_RESOURCE_FLUSH
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuResourceFlush {
    pub header: GpuControllerHeader,
    pub r: GpuRect,
    pub resource_id: u32,
    pub padding: u32,
}

/// CMD_TRANSFER_TO_HOST_2D
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuTransferToHost2d {
    pub header: GpuControllerHeader,
    pub r: GpuRect,
    pub offset: u64,
    pub resource_id: u32,
    pub padding: u32,
}

/// Memory entry for resource backing
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuMemoryEntry {
    pub address: u64,
    pub length: u32,
    pub padding: u32,
}

/// CMD_RESOURCE_ATTACH_BACKING
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuResourceAttachBacking {
    pub header: GpuControllerHeader,
    pub resource_id: u32,
    pub number_entries: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO Ring (local implementation for GPU)
// ═══════════════════════════════════════════════════════════════════════════════

#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, Default)]
struct VirtqDesc {
    address: u64,
    len: u32,
    flags: u16,
    next: u16,
}

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VIRTQ_DESCRIPTOR_F_NEXT: u16 = 1;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VIRTQ_DESCRIPTOR_F_WRITE: u16 = 2;

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    index: u16,
}

#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy, Default)]
struct VirtqUsedElement {
    id: u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    index: u16,
}

/// GPU virtqueue
struct GpuVirtqueue {
    size: u16,
    _physical_base: u64,
    _virt_base: *mut u8,
    desc: *mut VirtqDesc,
    avail: *mut VirtqAvail,
    used: *mut VirtqUsed,
    free_head: u16,
    number_free: u16,
    free_list: Vec<u16>,
    last_used_index: u16,
}

// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Send for GpuVirtqueue {}
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Sync for GpuVirtqueue {}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl GpuVirtqueue {
    fn new(size: u16) -> Result<Self, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        
        let descriptor_bytes = core::mem::size_of::<VirtqDesc>() * size as usize;
        let avail_bytes = 6 + 2 * size as usize;
        let used_offset = ((descriptor_bytes + avail_bytes) + 4095) & !4095;
        let used_bytes = 6 + core::mem::size_of::<VirtqUsedElement>() * size as usize;
        let total_size = used_offset + used_bytes + 4096;
        
        let layout = Layout::from_size_align(total_size, 4096)
            .map_error(|_| "Invalid virtqueue layout")?;
        let ptr = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("Failed to allocate virtqueue"); }
        
        let virt_address = ptr as u64;
        let hhdm = memory::hhdm_offset();
        let physical_address = if virt_address >= hhdm { virt_address - hhdm } else { virt_address };
        
        let desc = ptr as *mut VirtqDesc;
        let avail = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { ptr.add(descriptor_bytes) as *mut VirtqAvail };
        let used = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { ptr.add(used_offset) as *mut VirtqUsed };
        
        let mut free_list = vec![0u16; size as usize];
        for i in 0..(size as usize).saturating_sub(1) {
            free_list[i] = (i + 1) as u16;
        }
        if size > 0 { free_list[size as usize - 1] = 0xFFFF; }
        
        Ok(Self {
            size,
            _physical_base: physical_address,
            _virt_base: ptr,
            desc,
            avail,
            used,
            free_head: 0,
            number_free: size,
            free_list,
            last_used_index: 0,
        })
    }
    
    fn allocator_descriptor(&mut self) -> Option<u16> {
        if self.number_free == 0 { return None; }
        let index = self.free_head;
        self.free_head = self.free_list[index as usize];
        self.number_free -= 1;
        Some(index)
    }
    
    fn free_descriptor(&mut self, index: u16) {
        self.free_list[index as usize] = self.free_head;
        self.free_head = index;
        self.number_free += 1;
    }
    
    fn set_descriptor(&mut self, index: u16, address: u64, len: u32, flags: u16, next: u16) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let d = &mut *self.desc.add(index as usize);
            d.address = address;
            d.len = len;
            d.flags = flags;
            d.next = next;
        }
    }
    
    fn submit(&mut self, head: u16) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let avail = &mut *self.avail;
            let ring_pointer = (self.avail as *mut u8).add(4) as *mut u16;
            let index = avail.index;
            *ring_pointer.add((index % self.size) as usize) = head;
            core::sync::atomic::fence(Ordering::Release);
            avail.index = index.wrapping_add(1);
        }
    }
    
    fn poll_used(&mut self) -> Option<(u32, u32)> {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            core::sync::atomic::fence(Ordering::Acquire);
            let used = &*self.used;
            if used.index == self.last_used_index { return None; }
            let ring_pointer = (self.used as *mut u8).add(4) as *mut VirtqUsedElement;
            let element = *ring_pointer.add((self.last_used_index % self.size) as usize);
            self.last_used_index = self.last_used_index.wrapping_add(1);
            Some((element.id, element.len))
        }
    }
    
    fn descriptor_physical(&self) -> u64 { self._physical_base }
    fn avail_physical(&self) -> u64 {
        let descriptor_bytes = core::mem::size_of::<VirtqDesc>() * self.size as usize;
        self._physical_base + descriptor_bytes as u64
    }
    fn used_physical(&self) -> u64 {
        let descriptor_bytes = core::mem::size_of::<VirtqDesc>() * self.size as usize;
        let avail_bytes = 6 + 2 * self.size as usize;
        let used_offset = ((descriptor_bytes + avail_bytes) + 4095) & !4095;
        self._physical_base + used_offset as u64
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MMIO Region Accessor
// ═══════════════════════════════════════════════════════════════════════════════

struct MmioRegion {
    base: *mut u8,
    _length: u32,
}

// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Send for MmioRegion {}
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Sync for MmioRegion {}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl MmioRegion {
    fn read8(&self, offset: u32) -> u8 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::read_volatile(self.base.add(offset as usize)) }
    }
    fn read16(&self, offset: u32) -> u16 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::read_volatile(self.base.add(offset as usize) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u16) }
    }
    fn read32(&self, offset: u32) -> u32 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::read_volatile(self.base.add(offset as usize) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32) }
    }
    fn write8(&self, offset: u32, value: u8) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::write_volatile(self.base.add(offset as usize), value) }
    }
    fn write16(&self, offset: u32, value: u16) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::write_volatile(self.base.add(offset as usize) as *mut u16, value) }
    }
    fn write32(&self, offset: u32, value: u32) {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::write_volatile(self.base.add(offset as usize) as *mut u32, value) }
    }
    fn write64(&self, offset: u32, value: u64) {
        self.write32(offset, value as u32);
        self.write32(offset + 4, (value >> 32) as u32);
    }
}

/// Common config offsets (VirtIO PCI modern)
mod common_cfg {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEVICE_FEATURE_SELECT: u32 = 0x00;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEVICE_FEATURE: u32 = 0x04;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DRIVER_FEATURE_SELECT: u32 = 0x08;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DRIVER_FEATURE: u32 = 0x0C;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _MSIX_CONFIG: u32 = 0x10;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _NUMBER_QUEUES: u32 = 0x12;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEVICE_STATUS: u32 = 0x14;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _CONFIG_GENERATION: u32 = 0x15;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_SELECT: u32 = 0x16;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_SIZE: u32 = 0x18;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_MSIX_VECTOR: u32 = 0x1A;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_ENABLE: u32 = 0x1C;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_NOTIFY_OFF: u32 = 0x1E;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_DESCRIPTOR: u32 = 0x20;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_DRIVER: u32 = 0x28;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const QUEUE_DEVICE: u32 = 0x30;
}

/// GPU device config offsets
mod gpu_cfg {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _EVENTS_READ: u32 = 0x00;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const _EVENTS_CLEAR: u32 = 0x04;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NUMBER_SCANOUTS: u32 = 0x08;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NUMBER_CAPSETS: u32 = 0x0C;
}

// ═══════════════════════════════════════════════════════════════════════════════
// DMA Command Buffer
// ═══════════════════════════════════════════════════════════════════════════════

struct DmaCommandBuffer {
    physical: u64,
    virt: *mut u8,
    _size: usize,
}

// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Send for DmaCommandBuffer {}
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Sync for DmaCommandBuffer {}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl DmaCommandBuffer {
    fn new(size: usize) -> Result<Self, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(size, 4096)
            .map_error(|_| "DMA buffer layout error")?;
        let ptr = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("DMA buffer allocation failed"); }
        let virt = ptr as u64;
        let hhdm = memory::hhdm_offset();
        let physical = if virt >= hhdm { virt - hhdm } else { virt };
        Ok(Self { physical, virt: ptr, _size: size })
    }
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn write_at<T: Copy>(&self, offset: usize, value: &T) {
        core::ptr::write_volatile(self.virt.add(offset) as *mut T, *value);
    }
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn read_at<T: Copy>(&self, offset: usize) -> T {
        core::ptr::read_volatile(self.virt.add(offset) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const T)
    }
    
    fn physical_at(&self, offset: usize) -> u64 { self.physical + offset as u64 }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GPU Surface (compatibility API)
// ═══════════════════════════════════════════════════════════════════════════════

/// GPU Surface for 2D operations
pub struct GpuSurface {
    pub resource_id: u32,
    pub width: u32,
    pub height: u32,
    pub data: Box<[u32]>,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl GpuSurface {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            resource_id: 0,
            width,
            height,
            data: alloc::vec![0u32; size].into_boxed_slice(),
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn clear(&mut self, color: u32) { self.data.fill(color); }
    
    #[inline]
        // Fonction publique — appelable depuis d'autres modules.
pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = color;
        }
    }
    
    #[inline]
        // Fonction publique — appelable depuis d'autres modules.
pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else { 0 }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x1 = x.minimum(self.width);
        let y1 = y.minimum(self.height);
        let x2 = (x + w).minimum(self.width);
        let y2 = (y + h).minimum(self.height);
        for py in y1..y2 {
            let start = (py * self.width + x1) as usize;
            let end = (py * self.width + x2) as usize;
            self.data[start..end].fill(color);
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn blit(&mut self, source: &GpuSurface, destination_x: i32, destination_y: i32) {
        for sy in 0..source.height {
            for sx in 0..source.width {
                let dx = destination_x + sx as i32;
                let dy = destination_y + sy as i32;
                if dx >= 0 && dy >= 0 && dx < self.width as i32 && dy < self.height as i32 {
                    let pixel = source.get_pixel(sx, sy);
                    let alpha = (pixel >> 24) & 0xFF;
                    if alpha >= 128 {
                        self.set_pixel(dx as u32, dy as u32, pixel);
                    }
                }
            }
        }
    }
    
    fn set_pixel_safe(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.set_pixel(x as u32, y as u32, color);
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).absolute();
        let dy = -(y1 - y0).absolute();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        let (mut x, mut y) = (x0, y0);
                // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
            self.set_pixel_safe(x, y, color);
            if x == x1 && y == y1 { break; }
            let e2 = 2 * error;
            if e2 >= dy { error += dy; x += sx; }
            if e2 <= dx { error += dx; y += sy; }
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let (x, y, w, h) = (x as i32, y as i32, w as i32, h as i32);
        self.draw_line(x, y, x+w-1, y, color);
        self.draw_line(x, y+h-1, x+w-1, y+h-1, color);
        self.draw_line(x, y, x, y+h-1, color);
        self.draw_line(x+w-1, y, x+w-1, y+h-1, color);
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        let (mut x, mut y, mut error) = (radius, 0i32, 0i32);
        while x >= y {
            self.set_pixel_safe(cx+x, cy+y, color);
            self.set_pixel_safe(cx+y, cy+x, color);
            self.set_pixel_safe(cx-y, cy+x, color);
            self.set_pixel_safe(cx-x, cy+y, color);
            self.set_pixel_safe(cx-x, cy-y, color);
            self.set_pixel_safe(cx-y, cy-x, color);
            self.set_pixel_safe(cx+y, cy-x, color);
            self.set_pixel_safe(cx+x, cy-y, color);
            y += 1;
            error += 1 + 2*y;
            if 2*(error-x)+1 > 0 { x -= 1; error += 1 - 2*x; }
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx*dx + dy*dy <= radius*radius {
                    self.set_pixel_safe(cx+dx, cy+dy, color);
                }
            }
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn draw_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, _radius: u32, color: u32) {
        self.draw_rect(x, y, w, h, color);
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn fill_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, _radius: u32, color: u32) {
        self.fill_rect(x, y, w, h, color);
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn blit_scaled(&mut self, source: &GpuSurface, destination_x: i32, destination_y: i32, destination_w: u32, destination_h: u32) {
        if destination_w == 0 || destination_h == 0 || source.width == 0 || source.height == 0 { return; }
        for dy in 0..destination_h {
            for dx in 0..destination_w {
                let sx = (dx * source.width) / destination_w;
                let sy = (dy * source.height) / destination_h;
                let pixel = destination_x + dx as i32;
                let py = destination_y + dy as i32;
                if pixel >= 0 && py >= 0 && pixel < self.width as i32 && py < self.height as i32 {
                    self.set_pixel(pixel as u32, py as u32, source.get_pixel(sx, sy));
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO GPU Driver
// ═══════════════════════════════════════════════════════════════════════════════

pub struct VirtioGpu {
    _pci_device: Option<PciDevice>,
    common_cfg: Option<MmioRegion>,
    notify_configuration: Option<MmioRegion>,
    _interrupt_handler_configuration: Option<MmioRegion>,
    device_configuration: Option<MmioRegion>,
    _notify_off_multiplier: u32,
    controlq: Option<GpuVirtqueue>,
    dma_buffer: Option<DmaCommandBuffer>,
    display_width: u32,
    display_height: u32,
    number_scanouts: u32,
    next_resource_id: u32,
    scanout_resource_id: u32,
    backing_buffer: Option<Box<[u32]>>,
    backing_physical: u64,
    initialized: bool,
    has_3d: bool,
    // Upgrade #4: Double-buffer VirtIO GPU resources (eliminates tearing)
    back_resource_id: u32,
    back_buffer: Option<Box<[u32]>>,
    back_physical: u64,
    double_buffer_enabled: bool,
    front_is_a: bool, // true = resource A is displayed, B is back; false = inverse
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl VirtioGpu {
    pub const fn new() -> Self {
        Self {
            _pci_device: None,
            common_cfg: None,
            notify_configuration: None,
            _interrupt_handler_configuration: None,
            device_configuration: None,
            _notify_off_multiplier: 0,
            controlq: None,
            dma_buffer: None,
            display_width: 0,
            display_height: 0,
            number_scanouts: 0,
            next_resource_id: 1,
            scanout_resource_id: 0,
            backing_buffer: None,
            backing_physical: 0,
            initialized: false,
            has_3d: false,
            back_resource_id: 0,
            back_buffer: None,
            back_physical: 0,
            double_buffer_enabled: false,
            front_is_a: true,
        }
    }
    
    fn map_bar_region(device: &PciDevice, bar_index: u8, offset: u32, length: u32) -> Result<MmioRegion, &'static str> {
        let bar_address = device.bar_address(bar_index as usize)
            .ok_or("BAR not configured")?;
        if !device.bar_is_memory(bar_index as usize) {
            return Err("Expected memory BAR, got I/O");
        }
        let physical = bar_address + offset as u64;
        let virt = memory::map_mmio(physical, length.maximum(4096) as usize)?;
        crate::serial_println!("[VIRTIO-GPU] Mapped BAR{}: phys={:#X} virt={:#X} len={}", 
            bar_index, physical, virt, length);
        Ok(MmioRegion { base: virt as *mut u8, _length: length })
    }
    
    /// Initialize from PCI device
    pub fn init(&mut self, device: PciDevice) -> Result<(), &'static str> {
        crate::serial_println!("[VIRTIO-GPU] === Initializing VirtIO GPU ===");
        crate::serial_println!("[VIRTIO-GPU] PCI {:02X}:{:02X}.{} vid={:#06X} did={:#06X}",
            device.bus, device.device, device.function, device.vendor_id, device.device_id);
        
        pci::enable_bus_master(&device);
        pci::enable_memory_space(&device);
        
        // Find VirtIO PCI capabilities (modern transport)
        let caps = pci::find_virtio_capabilities(&device);
        if caps.is_empty() {
            return Err("No VirtIO capabilities found");
        }
        
        crate::serial_println!("[VIRTIO-GPU] Found {} VirtIO capabilities", caps.len());
        
        let mut notify_capability_offset: u8 = 0;
        
        for &(capability_off, configuration_type, bar, offset, length) in &caps {
            let name = // Correspondance de motifs — branchement exhaustif de Rust.
match configuration_type {
                1 => "COMMON", 2 => "NOTIFY", 3 => "ISR", 4 => "DEVICE", 5 => "PCI", _ => "?",
            };
            crate::serial_println!("[VIRTIO-GPU]   cap@{:#X}: {} BAR{} off={:#X} len={}", 
                capability_off, name, bar, offset, length);
            
                        // Correspondance de motifs — branchement exhaustif de Rust.
match configuration_type {
                virtio_cap::COMMON_CONFIGURATION => {
                    self.common_cfg = Some(Self::map_bar_region(&device, bar, offset, length)?);
                }
                virtio_cap::NOTIFY_CONFIGURATION => {
                    self.notify_configuration = Some(Self::map_bar_region(&device, bar, offset, length)?);
                    notify_capability_offset = capability_off;
                }
                virtio_cap::INTERRUPT_HANDLER_CONFIGURATION => {
                    self._interrupt_handler_configuration = Some(Self::map_bar_region(&device, bar, offset, length)?);
                }
                virtio_cap::DEVICE_CONFIGURATION => {
                    self.device_configuration = Some(Self::map_bar_region(&device, bar, offset, length)?);
                }
                _ => {}
            }
        }
        
        // Verify required capabilities are present
        if self.common_cfg.is_none() { return Err("Missing COMMON_CFG"); }
        if self.notify_configuration.is_none() { return Err("Missing NOTIFY_CFG"); }
        if self.device_configuration.is_none() { return Err("Missing DEVICE_CFG"); }
        
        if notify_capability_offset > 0 {
            self._notify_off_multiplier = pci::read_notify_off_multiplier(&device, notify_capability_offset);
        }
        
        // === VirtIO 1.0 initialization handshake ===
        // Use inline closure to access common_cfg without borrowing self
        
        // 1. Reset
        self.common_write8(common_cfg::DEVICE_STATUS, 0);
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        // 2. ACKNOWLEDGE
        self.common_write8(common_cfg::DEVICE_STATUS, dev_status::ACKNOWLEDGE);
        
        // 3. DRIVER
        self.common_write8(common_cfg::DEVICE_STATUS, dev_status::ACKNOWLEDGE | dev_status::DRIVER);
        
        // 4. Feature negotiation
        self.common_write32(common_cfg::DEVICE_FEATURE_SELECT, 0);
        let feat_lo = self.common_read32(common_cfg::DEVICE_FEATURE);
        self.common_write32(common_cfg::DEVICE_FEATURE_SELECT, 1);
        let feat_hi = self.common_read32(common_cfg::DEVICE_FEATURE);
        let device_features = (feat_lo as u64) | ((feat_hi as u64) << 32);
        
        crate::serial_println!("[VIRTIO-GPU] Device features: {:#018X}", device_features);
        self.has_3d = device_features & features::_VIRTIO_GPU_F_VIRGL != 0;
        
        let mut driver_features = features::VIRTIO_F_VERSION_1;
        if device_features & features::VIRTIO_GPU_F_EDID != 0 {
            driver_features |= features::VIRTIO_GPU_F_EDID;
        }
        
        self.common_write32(common_cfg::DRIVER_FEATURE_SELECT, 0);
        self.common_write32(common_cfg::DRIVER_FEATURE, driver_features as u32);
        self.common_write32(common_cfg::DRIVER_FEATURE_SELECT, 1);
        self.common_write32(common_cfg::DRIVER_FEATURE, (driver_features >> 32) as u32);
        
        // 5. FEATURES_OK
        self.common_write8(common_cfg::DEVICE_STATUS,
            dev_status::ACKNOWLEDGE | dev_status::DRIVER | dev_status::FEATURES_OK);
        
        let status = self.common_read8(common_cfg::DEVICE_STATUS);
        if status & dev_status::FEATURES_OK == 0 {
            self.common_write8(common_cfg::DEVICE_STATUS, dev_status::FAILED);
            return Err("Device rejected features");
        }
        crate::serial_println!("[VIRTIO-GPU] Features OK (3D={})", self.has_3d);
        
        // 6. Setup controlq (queue 0)
        self.setup_controlq()?;
        
        // 7. DRIVER_OK
        self.common_write8(common_cfg::DEVICE_STATUS,
            dev_status::ACKNOWLEDGE | dev_status::DRIVER | dev_status::FEATURES_OK | dev_status::DRIVER_OK);
        crate::serial_println!("[VIRTIO-GPU] DRIVER_OK set");
        
        // 8. DMA command buffer
        self.dma_buffer = Some(DmaCommandBuffer::new(8192)?);
        
        // 9. Read GPU config
        self.number_scanouts = self.device_read32(gpu_cfg::NUMBER_SCANOUTS);
        let number_capsets = self.device_read32(gpu_cfg::NUMBER_CAPSETS);
        crate::serial_println!("[VIRTIO-GPU] scanouts={} capsets={}", self.number_scanouts, number_capsets);
        
        // 10. Get display info
        self.get_display_information()?;
        
        self._pci_device = Some(device);
        self.initialized = true;
        
        crate::serial_println!("[VIRTIO-GPU] === Init complete: {}x{} ===", 
            self.display_width, self.display_height);
        Ok(())
    }
    
    // Helper MMIO accessors that borrow only the specific field
    fn common_write8(&self, offset: u32, value: u8) {
        if let Some(c) = &self.common_cfg { c.write8(offset, value); }
    }
    fn common_write16(&self, offset: u32, value: u16) {
        if let Some(c) = &self.common_cfg { c.write16(offset, value); }
    }
    fn common_write32(&self, offset: u32, value: u32) {
        if let Some(c) = &self.common_cfg { c.write32(offset, value); }
    }
    fn common_read8(&self, offset: u32) -> u8 {
        self.common_cfg.as_ref().map(|c| c.read8(offset)).unwrap_or(0)
    }
    fn common_read16(&self, offset: u32) -> u16 {
        self.common_cfg.as_ref().map(|c| c.read16(offset)).unwrap_or(0)
    }
    fn common_read32(&self, offset: u32) -> u32 {
        self.common_cfg.as_ref().map(|c| c.read32(offset)).unwrap_or(0)
    }
    fn device_read32(&self, offset: u32) -> u32 {
        self.device_configuration.as_ref().map(|c| c.read32(offset)).unwrap_or(0)
    }
    
    fn setup_controlq(&mut self) -> Result<(), &'static str> {
        let common = self.common_cfg.as_ref().ok_or("Missing COMMON_CFG")?;
        common.write16(common_cfg::QUEUE_SELECT, 0);
        let maximum_size = common.read16(common_cfg::QUEUE_SIZE);
        crate::serial_println!("[VIRTIO-GPU] controlq max_size={}", maximum_size);
        if maximum_size == 0 { return Err("controlq not available"); }
        
        let queue_size = maximum_size.minimum(64);
        common.write16(common_cfg::QUEUE_SIZE, queue_size);
        
        let vq = GpuVirtqueue::new(queue_size)?;
        
        common.write64(common_cfg::QUEUE_DESCRIPTOR, vq.descriptor_physical());
        common.write64(common_cfg::QUEUE_DRIVER, vq.avail_physical());
        common.write64(common_cfg::QUEUE_DEVICE, vq.used_physical());
        common.write16(common_cfg::QUEUE_MSIX_VECTOR, 0xFFFF);
        common.write16(common_cfg::QUEUE_ENABLE, 1);
        
        let _notify_off = common.read16(common_cfg::QUEUE_NOTIFY_OFF);
        
        self.controlq = Some(vq);
        crate::serial_println!("[VIRTIO-GPU] controlq ready (size={})", queue_size);
        Ok(())
    }
    
    fn notify_controlq(&self) {
        if let Some(notify) = &self.notify_configuration {
            notify.write16(0, 0);
        }
    }
    
    /// Send command + wait for response (synchronous)
    fn send_command(&mut self, command_length: u32, response_offset: usize, response_length: u32) -> Result<u32, &'static str> {
        // Extract DMA phys base before mutable controlq borrow
        let dma_physical_base = self.dma_buffer.as_ref().ok_or("DMA not ready")?.physical;
        
        let controlq = self.controlq.as_mut().ok_or("controlq not ready")?;
        
        let d_command = controlq.allocator_descriptor().ok_or("No free desc (cmd)")?;
        let d_response = controlq.allocator_descriptor().ok_or("No free desc (resp)")?;
        
        controlq.set_descriptor(d_command, dma_physical_base, command_length, VIRTQ_DESCRIPTOR_F_NEXT, d_response);
        controlq.set_descriptor(d_response, dma_physical_base + response_offset as u64, response_length, VIRTQ_DESCRIPTOR_F_WRITE, 0);
        
        controlq.submit(d_command);
        // Notify inline (avoid re-borrowing self)
        if let Some(notify) = &self.notify_configuration {
            notify.write16(0, 0);
        }
        
        let mut timeout = 5_000_000u32;
                // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
            if let Some(_) = controlq.poll_used() { break; }
            timeout -= 1;
            if timeout == 0 {
                controlq.free_descriptor(d_response);
                controlq.free_descriptor(d_command);
                return Err("Command timeout");
            }
            core::hint::spin_loop();
        }
        
        let dma = self.dma_buffer.as_ref().unwrap();
        let response_type = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.read_at::<GpuControllerHeader>(response_offset) }.controller_type;
        controlq.free_descriptor(d_response);
        controlq.free_descriptor(d_command);
        Ok(response_type)
    }
    
    fn get_display_information(&mut self) -> Result<(), &'static str> {
        let dma = self.dma_buffer.as_ref().ok_or("DMA not ready")?;
        
        let cmd = GpuControllerHeader {
            controller_type: GpuCtrlType::CommandGetDisplayInformation as u32,
            ..Default::default()
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
        
        let response_type = self.send_command(
            core::mem::size_of::<GpuControllerHeader>() as u32,
            512, // response offset
            core::mem::size_of::<GpuResponseDisplayInformation>() as u32,
        )?;
        
        if response_type != GpuCtrlType::ResponseOkDisplayInformation as u32 {
            crate::serial_println!("[VIRTIO-GPU] GET_DISPLAY_INFO failed: {:#X}", response_type);
            return Err("GET_DISPLAY_INFO failed");
        }
        
        let dma = self.dma_buffer.as_ref().unwrap();
        let response: GpuResponseDisplayInformation = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.read_at(512) };
        
        for (i, pm) in response.pmodes.iter().enumerate() {
            if pm.enabled != 0 {
                self.display_width = pm.r.width;
                self.display_height = pm.r.height;
                crate::serial_println!("[VIRTIO-GPU] Display {}: {}x{}", i, pm.r.width, pm.r.height);
                break;
            }
        }
        
        if self.display_width == 0 {
            self.display_width = 1280;
            self.display_height = 800;
            crate::serial_println!("[VIRTIO-GPU] Defaulting to {}x{}", self.display_width, self.display_height);
        }
        Ok(())
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn create_resource_2d(&mut self, width: u32, height: u32) -> Result<u32, &'static str> {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        let dma = self.dma_buffer.as_ref().ok_or("DMA not ready")?;
        
        let cmd = GpuResourceCreate2d {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandResourceCreate2d as u32, ..Default::default() },
            resource_id: id,
            format: GpuFormat::B8G8R8X8Unorm as u32,
            width,
            height,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
        
        let response = self.send_command(
            core::mem::size_of::<GpuResourceCreate2d>() as u32,
            512, core::mem::size_of::<GpuControllerHeader>() as u32,
        )?;
        
        if response != GpuCtrlType::ResponseOkNodata as u32 {
            return Err("RESOURCE_CREATE_2D failed");
        }
        crate::serial_println!("[VIRTIO-GPU] Resource {} created ({}x{})", id, width, height);
        Ok(id)
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn attach_backing(&mut self, resource_id: u32, buffer_physical: u64, buffer_length: u32) -> Result<(), &'static str> {
        let dma = self.dma_buffer.as_ref().ok_or("DMA not ready")?;
        
        let cmd = GpuResourceAttachBacking {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandResourceAttachBacking as u32, ..Default::default() },
            resource_id,
            number_entries: 1,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
        
        let entry = GpuMemoryEntry { address: buffer_physical, length: buffer_length, padding: 0 };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(core::mem::size_of::<GpuResourceAttachBacking>(), &entry); }
        
        let command_size = (core::mem::size_of::<GpuResourceAttachBacking>() + core::mem::size_of::<GpuMemoryEntry>()) as u32;
        let response = self.send_command(command_size, 512, core::mem::size_of::<GpuControllerHeader>() as u32)?;
        
        if response != GpuCtrlType::ResponseOkNodata as u32 {
            return Err("ATTACH_BACKING failed");
        }
        crate::serial_println!("[VIRTIO-GPU] Backing attached: phys={:#X} len={}", buffer_physical, buffer_length);
        Ok(())
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn set_scanout(&mut self, scanout_id: u32, resource_id: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buffer.as_ref().ok_or("DMA not ready")?;
        let cmd = GpuSetScanout {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandSetScanout as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            scanout_id,
            resource_id,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
        
        let response = self.send_command(
            core::mem::size_of::<GpuSetScanout>() as u32,
            512, core::mem::size_of::<GpuControllerHeader>() as u32,
        )?;
        
        if response != GpuCtrlType::ResponseOkNodata as u32 { return Err("SET_SCANOUT failed"); }
        self.scanout_resource_id = resource_id;
        crate::serial_println!("[VIRTIO-GPU] Scanout {} -> resource {} ({}x{})", scanout_id, resource_id, w, h);
        Ok(())
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn transfer_to_host(&mut self, resource_id: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buffer.as_ref().ok_or("DMA not ready")?;
        let cmd = GpuTransferToHost2d {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandTransferToHost2d as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            offset: 0,
            resource_id,
            padding: 0,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
        
        let response = self.send_command(
            core::mem::size_of::<GpuTransferToHost2d>() as u32,
            512, core::mem::size_of::<GpuControllerHeader>() as u32,
        )?;
        if response != GpuCtrlType::ResponseOkNodata as u32 { return Err("TRANSFER failed"); }
        Ok(())
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn flush_resource(&mut self, resource_id: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buffer.as_ref().ok_or("DMA not ready")?;
        let cmd = GpuResourceFlush {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandResourceFlush as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            resource_id,
            padding: 0,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
        
        let response = self.send_command(
            core::mem::size_of::<GpuResourceFlush>() as u32,
            512, core::mem::size_of::<GpuControllerHeader>() as u32,
        )?;
        if response != GpuCtrlType::ResponseOkNodata as u32 { return Err("FLUSH failed"); }
        Ok(())
    }
    
    /// Setup complete scanout pipeline
    /// Uses the Limine framebuffer dimensions for consistency with the compositor
    pub fn setup_scanout(&mut self) -> Result<(), &'static str> {
        if !self.initialized { return Err("GPU not initialized"); }
        
        // Use Limine framebuffer dimensions so GPU resource matches compositor
        let (framebuffer_w, framebuffer_h) = crate::framebuffer::get_dimensions();
        if framebuffer_w > 0 && framebuffer_h > 0 {
            crate::serial_println!("[VIRTIO-GPU] Using framebuffer dimensions: {}x{} (display was {}x{})",
                framebuffer_w, framebuffer_h, self.display_width, self.display_height);
            self.display_width = framebuffer_w;
            self.display_height = framebuffer_h;
        }
        
        let w = self.display_width;
        let h = self.display_height;
        crate::serial_println!("[VIRTIO-GPU] Setting up scanout {}x{}", w, h);
        
        let resource_id = self.create_resource_2d(w, h)?;
        
        // Allocate page-aligned backing buffer
        let buffer_size = (w * h) as usize;
        let buffer_bytes = buffer_size * 4;
        
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(buffer_bytes, 4096).map_error(|_| "Layout error")?;
        let ptr = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("Backing buffer allocation failed"); }
        
        let virt = ptr as u64;
        let hhdm = memory::hhdm_offset();
        let physical = if virt >= hhdm { virt - hhdm } else { virt };
        
        let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let slice = core::slice::from_raw_parts_mut(ptr as *mut u32, buffer_size);
            Box::from_raw(slice as *mut [u32])
        };
        
        self.backing_buffer = Some(buffer);
        self.backing_physical = physical;
        
        self.attach_backing(resource_id, physical, buffer_bytes as u32)?;
        self.set_scanout(0, resource_id, w, h)?;
        
        crate::serial_println!("[VIRTIO-GPU] Scanout ready! phys={:#X}", physical);
        Ok(())
    }
    
    /// Upgrade #4: Setup double-buffered VirtIO GPU resources
    /// Creates a second GPU resource (back buffer) for tear-free rendering.
    /// CPU renders to back buffer, then swaps scanout to display it.
    pub fn setup_double_buffer(&mut self) -> Result<(), &'static str> {
        if !self.initialized { return Err("GPU not initialized"); }
        if self.scanout_resource_id == 0 { return Err("No primary scanout"); }
        
        let w = self.display_width;
        let h = self.display_height;
        
        // Create second resource
        let back_id = self.create_resource_2d(w, h)?;
        
        // Allocate page-aligned backing buffer for back resource
        let buffer_size = (w * h) as usize;
        let buffer_bytes = buffer_size * 4;
        
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(buffer_bytes, 4096)
            .map_error(|_| "Layout error")?;
        let ptr = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("Back buffer allocation failed"); }
        
        let virt = ptr as u64;
        let hhdm = memory::hhdm_offset();
        let physical = if virt >= hhdm { virt - hhdm } else { virt };
        
        let buffer = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let slice = core::slice::from_raw_parts_mut(ptr as *mut u32, buffer_size);
            Box::from_raw(slice as *mut [u32])
        };
        
        self.attach_backing(back_id, physical, buffer_bytes as u32)?;
        
        self.back_resource_id = back_id;
        self.back_buffer = Some(buffer);
        self.back_physical = physical;
        self.double_buffer_enabled = true;
        self.front_is_a = true;
        
        crate::serial_println!("[VIRTIO-GPU] Double buffer enabled: resource A={}, B={}", 
            self.scanout_resource_id, back_id);
        Ok(())
    }
    
    /// Swap front/back GPU buffers: set scanout to the back buffer, make it front
    pub fn swap_gpu_buffers(&mut self) -> Result<(), &'static str> {
        if !self.double_buffer_enabled { return Ok(()); }
        
        let (w, h) = (self.display_width, self.display_height);
        
        if self.front_is_a {
            // Back buffer (B) has new content, make it the displayed one
            self.set_scanout(0, self.back_resource_id, w, h)?;
        } else {
            // Back buffer (A) has new content, make it the displayed one
            self.set_scanout(0, self.scanout_resource_id, w, h)?;
        }
        
        self.front_is_a = !self.front_is_a;
        Ok(())
    }
    
    /// Get the current back buffer (the one we render to)
    pub fn get_back_buffer(&mut self) -> Option<&mut [u32]> {
        if !self.double_buffer_enabled {
            return self.backing_buffer.as_deref_mut();
        }
        if self.front_is_a {
            // A is front (displayed), B is back (render target)
            self.back_buffer.as_deref_mut()
        } else {
            // B is front, A is back
            self.backing_buffer.as_deref_mut()
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn get_buffer(&mut self) -> Option<&mut [u32]> {
        self.backing_buffer.as_deref_mut()
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn get_dimensions(&self) -> (u32, u32) {
        (self.display_width, self.display_height)
    }
    
    /// Present: transfer backing buffer to host + flush display
    /// OPTIMIZED: Batches both commands in a single VirtIO submission
    /// - 1 notify instead of 2
    /// - 1 poll cycle instead of 2
    /// - 4 descriptor alloc/free instead of 4 (but 1 round trip)
    pub fn present(&mut self) -> Result<(), &'static str> {
        let rid = self.scanout_resource_id;
        if rid == 0 { return Err("No scanout"); }
        let (w, h) = (self.display_width, self.display_height);
        
        let dma = self.dma_buffer.as_ref().ok_or("DMA not ready")?;
        let dma_physical = dma.physical;
        
        // Write transfer_to_host command at DMA offset 0
        let transfer_command = GpuTransferToHost2d {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandTransferToHost2d as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            offset: 0,
            resource_id: rid,
            padding: 0,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &transfer_command); }
        
        // Write flush command at DMA offset 256
        let flush_command = GpuResourceFlush {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandResourceFlush as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            resource_id: rid,
            padding: 0,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(256, &flush_command); }
        
        let transfer_size = core::mem::size_of::<GpuTransferToHost2d>() as u32;
        let flush_size = core::mem::size_of::<GpuResourceFlush>() as u32;
        let response_size = core::mem::size_of::<GpuControllerHeader>() as u32;
        
        let controlq = self.controlq.as_mut().ok_or("controlq not ready")?;
        
        // Allocate 4 descriptors for both command chains
        let d0 = controlq.allocator_descriptor().ok_or("No free desc")?;
        let d1 = controlq.allocator_descriptor().ok_or("No free desc")?;
        let d2 = controlq.allocator_descriptor().ok_or("No free desc")?;
        let d3 = controlq.allocator_descriptor().ok_or("No free desc")?;
        
        // Chain 1: transfer cmd (offset 0) → response (offset 512)
        controlq.set_descriptor(d0, dma_physical, transfer_size, VIRTQ_DESCRIPTOR_F_NEXT, d1);
        controlq.set_descriptor(d1, dma_physical + 512, response_size, VIRTQ_DESCRIPTOR_F_WRITE, 0);
        
        // Chain 2: flush cmd (offset 256) → response (offset 768)
        controlq.set_descriptor(d2, dma_physical + 256, flush_size, VIRTQ_DESCRIPTOR_F_NEXT, d3);
        controlq.set_descriptor(d3, dma_physical + 768, response_size, VIRTQ_DESCRIPTOR_F_WRITE, 0);
        
        // Submit both chains to the available ring
        controlq.submit(d0);
        controlq.submit(d2);
        
        // Single notification for both commands
        if let Some(notify) = &self.notify_configuration {
            notify.write16(0, 0);
        }
        
        // Poll for both completions
        let mut completed = 0u8;
        let mut timeout = 5_000_000u32;
        while completed < 2 {
            if let Some(_) = controlq.poll_used() {
                completed += 1;
            }
            if completed < 2 {
                timeout -= 1;
                if timeout == 0 {
                    controlq.free_descriptor(d3);
                    controlq.free_descriptor(d2);
                    controlq.free_descriptor(d1);
                    controlq.free_descriptor(d0);
                    return Err("Batched present timeout");
                }
                core::hint::spin_loop();
            }
        }
        
        // Check responses
        let dma = self.dma_buffer.as_ref().unwrap();
        let t_response = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.read_at::<GpuControllerHeader>(512) }.controller_type;
        let f_response = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.read_at::<GpuControllerHeader>(768) }.controller_type;
        
        controlq.free_descriptor(d3);
        controlq.free_descriptor(d2);
        controlq.free_descriptor(d1);
        controlq.free_descriptor(d0);
        
        if t_response != GpuCtrlType::ResponseOkNodata as u32 { return Err("TRANSFER failed"); }
        if f_response != GpuCtrlType::ResponseOkNodata as u32 { return Err("FLUSH failed"); }
        
        Ok(())
    }
    
    /// Present a single dirty rectangle — partial transfer + flush
    /// Only transfers the specified region instead of the full framebuffer.
    /// Upgrade #3: VirtIO GPU supports per-rect transfer_to_host_2d + resource_flush.
    pub fn present_rect(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let rid = self.scanout_resource_id;
        if rid == 0 { return Err("No scanout"); }
        
        // Clamp to display bounds
        let x = x.minimum(self.display_width);
        let y = y.minimum(self.display_height);
        let w = w.minimum(self.display_width.saturating_sub(x));
        let h = h.minimum(self.display_height.saturating_sub(y));
        if w == 0 || h == 0 { return Ok(()); }
        
        let dma = self.dma_buffer.as_ref().ok_or("DMA not ready")?;
        let dma_physical = dma.physical;
        
        // Calculate byte offset into backing buffer for this rect
        let offset = ((y * self.display_width + x) as u64) * 4;
        
        let transfer_command = GpuTransferToHost2d {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandTransferToHost2d as u32, ..Default::default() },
            r: GpuRect { x, y, width: w, height: h },
            offset,
            resource_id: rid,
            padding: 0,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &transfer_command); }
        
        let flush_command = GpuResourceFlush {
            header: GpuControllerHeader { controller_type: GpuCtrlType::CommandResourceFlush as u32, ..Default::default() },
            r: GpuRect { x, y, width: w, height: h },
            resource_id: rid,
            padding: 0,
        };
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(256, &flush_command); }
        
        let transfer_size = core::mem::size_of::<GpuTransferToHost2d>() as u32;
        let flush_size = core::mem::size_of::<GpuResourceFlush>() as u32;
        let response_size = core::mem::size_of::<GpuControllerHeader>() as u32;
        
        let controlq = self.controlq.as_mut().ok_or("controlq not ready")?;
        let d0 = controlq.allocator_descriptor().ok_or("No free desc")?;
        let d1 = controlq.allocator_descriptor().ok_or("No free desc")?;
        let d2 = controlq.allocator_descriptor().ok_or("No free desc")?;
        let d3 = controlq.allocator_descriptor().ok_or("No free desc")?;
        
        controlq.set_descriptor(d0, dma_physical, transfer_size, VIRTQ_DESCRIPTOR_F_NEXT, d1);
        controlq.set_descriptor(d1, dma_physical + 512, response_size, VIRTQ_DESCRIPTOR_F_WRITE, 0);
        controlq.set_descriptor(d2, dma_physical + 256, flush_size, VIRTQ_DESCRIPTOR_F_NEXT, d3);
        controlq.set_descriptor(d3, dma_physical + 768, response_size, VIRTQ_DESCRIPTOR_F_WRITE, 0);
        
        controlq.submit(d0);
        controlq.submit(d2);
        
        if let Some(notify) = &self.notify_configuration {
            notify.write16(0, 0);
        }
        
        let mut completed = 0u8;
        let mut timeout = 5_000_000u32;
        while completed < 2 {
            if let Some(_) = controlq.poll_used() { completed += 1; }
            if completed < 2 {
                timeout -= 1;
                if timeout == 0 {
                    controlq.free_descriptor(d3);
                    controlq.free_descriptor(d2);
                    controlq.free_descriptor(d1);
                    controlq.free_descriptor(d0);
                    return Err("Rect present timeout");
                }
                core::hint::spin_loop();
            }
        }
        
        controlq.free_descriptor(d3);
        controlq.free_descriptor(d2);
        controlq.free_descriptor(d1);
        controlq.free_descriptor(d0);
        Ok(())
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn is_initialized(&self) -> bool { self.initialized }
        // Fonction publique — appelable depuis d'autres modules.
pub fn has_3d_support(&self) -> bool { self.has_3d }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global Instance & Public API
// ═══════════════════════════════════════════════════════════════════════════════

static GPU: Mutex<VirtioGpu> = Mutex::new(VirtioGpu::new());
// Variable atomique — accès thread-safe sans verrou.
static GPU_AVAILABLE: AtomicBool = AtomicBool::new(false);

// Fonction publique — appelable depuis d'autres modules.
pub fn initialize_from_pci() -> Result<(), &'static str> {
    for device in crate::pci::scan() {
        if device.vendor_id == VIRTIO_VENDOR_ID && device.device_id == VIRTIO_GPU_PCI_DEVICE_ID {
            crate::serial_println!("[VIRTIO-GPU] Found device at {:02x}:{:02x}.{}",
                device.bus, device.device, device.function);
            
            let mut gpu = GPU.lock();
                        // Correspondance de motifs — branchement exhaustif de Rust.
match gpu.init(device) {
                Ok(()) => {
                                        // Correspondance de motifs — branchement exhaustif de Rust.
match gpu.setup_scanout() {
                        Ok(()) => {
                            GPU_AVAILABLE.store(true, Ordering::SeqCst);
                            crate::serial_println!("[VIRTIO-GPU] ✓ Ready for rendering!");
                            // Upgrade #4: Setup double-buffered GPU resources
                            match gpu.setup_double_buffer() {
                                Ok(()) => crate::serial_println!("[VIRTIO-GPU] ✓ Double buffer enabled (tear-free)"),
                                Err(e) => crate::serial_println!("[VIRTIO-GPU] Double buffer skipped: {}", e),
                            }
                        }
                        Err(e) => crate::serial_println!("[VIRTIO-GPU] Scanout failed: {}", e),
                    }
                }
                Err(e) => crate::serial_println!("[VIRTIO-GPU] Init failed: {}", e),
            }
            return Ok(());
        }
    }
    crate::serial_println!("[VIRTIO-GPU] No VirtIO GPU found");
    Ok(())
}

// Fonction publique — appelable depuis d'autres modules.
pub fn is_available() -> bool {
    GPU_AVAILABLE.load(Ordering::SeqCst)
}

// Fonction publique — appelable depuis d'autres modules.
pub fn create_surface(width: u32, height: u32) -> GpuSurface {
    GpuSurface::new(width, height)
}

/// Render frame using VirtIO GPU DMA path
pub fn render_frame<F: FnOnce(&mut [u32], u32, u32)>(render_fn: F) -> Result<(), &'static str> {
    let mut gpu = GPU.lock();
    if !gpu.initialized { return Err("GPU not initialized"); }
    let (w, h) = (gpu.display_width, gpu.display_height);
    if let Some(buffer) = gpu.backing_buffer.as_deref_mut() {
        render_fn(buffer, w, h);
    }
    gpu.present()
}

/// Present the current backing buffer (after external rendering)
pub fn present_frame() -> Result<(), &'static str> {
    GPU.lock().present()
}

/// Upgrade #4: Present using double-buffer swap for tear-free display
/// Transfers the back buffer to host, then atomically swaps scanout resources.
pub fn present_frame_double_buffered() -> Result<(), &'static str> {
    let mut gpu = GPU.lock();
    if !gpu.double_buffer_enabled {
        return gpu.present();
    }
    // Present the current back buffer
    gpu.present()?;
    // Swap front ↔ back
    gpu.swap_gpu_buffers()
}

/// Get back buffer pointer for double-buffered rendering
pub fn get_back_buffer() -> Option<(*mut u32, u32, u32)> {
    let mut gpu = GPU.lock();
    if !gpu.initialized { return None; }
    let (w, h) = (gpu.display_width, gpu.display_height);
    gpu.get_back_buffer().map(|buffer| (buffer.as_mut_pointer(), w, h))
}

/// Present only dirty rectangles — Upgrade #3: partial VirtIO GPU flush
/// Copies backbuffer to GPU backing buffer, then transfers + flushes only the
/// specified dirty regions. Avoids transferring the full 8MB framebuffer.
pub fn present_dirty_rects(rects: &[(u32, u32, u32, u32)]) {
    if rects.is_empty() { return; }
    
    // First: copy the entire backbuffer to GPU backing buffer (fast SSE2 RAM-to-RAM)
    let (framebuffer_w, _framebuffer_h) = crate::framebuffer::get_dimensions();
    if let Some((gpu_pointer, gpu_w, gpu_h)) = get_raw_buffer() {
        if let Some(bb) = crate::framebuffer::get_backbuffer_pointer() {
            let copy_w = (framebuffer_w as usize).minimum(gpu_w as usize);
            let copy_h = gpu_h as usize;
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                for y in 0..copy_h {
                    let source = bb.add(y * framebuffer_w as usize);
                    let destination = gpu_pointer.add(y * gpu_w as usize);
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::copy_row_sse2(destination, source, copy_w);
                    #[cfg(not(target_arch = "x86_64"))]
                    core::ptr::copy_nonoverlapping(source, destination, copy_w);
                }
            }
        }
    }
    
    // Then: issue partial transfer + flush for each dirty region
    let mut gpu = GPU.lock();
    if !gpu.initialized { return; }
    let rid = gpu.scanout_resource_id;
    if rid == 0 { return; }
    
    for &(x, y, w, h) in rects {
        let _ = gpu.present_rect(x, y, w, h);
    }
}

/// Get raw buffer pointer for direct rendering
pub fn get_raw_buffer() -> Option<(*mut u32, u32, u32)> {
    let mut gpu = GPU.lock();
    if !gpu.initialized { return None; }
    let (w, h) = (gpu.display_width, gpu.display_height);
    gpu.backing_buffer.as_deref_mut().map(|buffer| (buffer.as_mut_pointer(), w, h))
}

/// Info string for shell
pub fn information_string() -> alloc::string::String {
    let gpu = GPU.lock();
    if gpu.initialized {
        alloc::format!("VirtIO GPU: {}x{} 2D (3D={})", gpu.display_width, gpu.display_height,
            if gpu.has_3d { "virgl" } else { "no" })
    } else {
        alloc::string::String::from("VirtIO GPU: not available")
    }
}

/// Fallback: blit surface to framebuffer (when no VirtIO GPU)
pub fn blit_to_screen(surface: &GpuSurface, x: u32, y: u32) {
    let (framebuffer_w, framebuffer_h) = crate::framebuffer::get_dimensions();
    crate::framebuffer::set_double_buffer_mode(true);
    for sy in 0..surface.height {
        let sy2 = y + sy;
        if sy2 >= framebuffer_h { break; }
        for sx in 0..surface.width {
            let sx2 = x + sx;
            if sx2 >= framebuffer_w { break; }
            crate::framebuffer::put_pixel(sx2, sy2, surface.get_pixel(sx, sy));
        }
    }
    crate::framebuffer::swap_buffers();
}

// Fonction publique — appelable depuis d'autres modules.
pub fn flush_screen() {
    if crate::framebuffer::is_double_buffer_enabled() {
        crate::framebuffer::swap_buffers();
    }
}

// Fonction publique — appelable depuis d'autres modules.
pub fn init() {
    crate::serial_println!("[GPU] Initializing graphics subsystem...");
    if let Err(e) = initialize_from_pci() {
        crate::serial_println!("[GPU] PCI init error: {}", e);
    }
    crate::framebuffer::initialize_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);
    crate::serial_println!("[GPU] Graphics ready (VirtIO: {})", 
        if is_available() { "ACTIVE" } else { "fallback" });
}

// ═══════════════════════════════════════════════════════════════════════════════
// VIRGL 3D Foundation (Upgrade #5)
// ═══════════════════════════════════════════════════════════════════════════════
//
// VIRGL is the Gallium3D-based protocol that allows sending OpenGL commands
// to the host GPU through VirtIO. This provides the foundation structures and
// context management. Actual shader compilation and draw command submission
// can be built on top of this infrastructure.

/// VIRGL capability set info
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct VirglCapsetInformation {
    pub capset_id: u32,
    pub capset_maximum_version: u32,
    pub capset_maximum_size: u32,
    pub padding: u32,
}

/// VIRGL 3D context creation command
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuContextCreate {
    pub header: GpuControllerHeader,
    pub nlen: u32,
    pub context_initialize: u32,
    pub debug_name: [u8; 64],
}

/// VIRGL 3D context destroy command
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuContextDestroy {
    pub header: GpuControllerHeader,
}

/// VIRGL 3D resource create command
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuResourceCreate3d {
    pub header: GpuControllerHeader,
    pub resource_id: u32,
    pub target: u32,     // PIPE_TEXTURE_2D = 2, PIPE_BUFFER = 0
    pub format: u32,     // VIRGL_FORMAT_*
    pub bind: u32,       // VIRGL_BIND_*
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub array_size: u32,
    pub last_level: u32,
    pub number_samples: u32,
    pub flags: u32,
    pub padding: u32,
}

/// VIRGL submit 3D command buffer
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuSubmit3d {
    pub header: GpuControllerHeader,
    pub size: u32,
    pub padding: u32,
    // followed by `size` bytes of Gallium command stream
}

/// VIRGL context attach resource
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct GpuContextAttachResource {
    pub header: GpuControllerHeader,
    pub resource_id: u32,
    pub padding: u32,
}

/// VIRGL bind targets
#[allow(dead_code)]
pub mod virgl_bind {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DEPTH_STENCIL: u32 = 1 << 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RENDER_TARGET: u32 = 1 << 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SAMPLER_VIEW: u32 = 1 << 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VERTEX_BUFFER: u32 = 1 << 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const INDEX_BUFFER: u32 = 1 << 5;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CONSTANT_BUFFER: u32 = 1 << 6;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SHADER_BUFFER: u32 = 1 << 14;
}

/// VIRGL resource targets
#[allow(dead_code)]
pub mod virgl_target {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BUFFER: u32 = 0;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXTURE_1D: u32 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXTURE_2D: u32 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXTURE_3D: u32 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const TEXTURE_CUBE: u32 = 4;
}

/// VIRGL 3D context manager
pub struct Virgl3dContext {
    context_id: u32,
    active: bool,
    capset_version: u32,
}

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static VIRGL_CONTEXT: Mutex<Virgl3dContext> = Mutex::new(Virgl3dContext {
    context_id: 0,
    active: false,
    capset_version: 0,
});

/// Check if VIRGL 3D is available (GPU supports it and context can be created)
pub fn has_virgl() -> bool {
    let gpu = GPU.lock();
    gpu.has_3d
}

/// Query VIRGL capability set info
pub fn query_virgl_capset() -> Option<VirglCapsetInformation> {
    let mut gpu = GPU.lock();
    if !gpu.has_3d || !gpu.initialized { return None; }
    
    let dma = gpu.dma_buffer.as_ref()?;
    
    // GET_CAPSET_INFO for capset 0 (VIRGL)
    let cmd = GpuControllerHeader {
        controller_type: GpuCtrlType::CommandGetCapsetInformation as u32,
        ..Default::default()
    };
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
    // capset_index = 0 at offset after header
    unsafe { (dma.virt.add(core::mem::size_of::<GpuControllerHeader>()) as *mut u32).write_volatile(0); }
    
    let command_size = core::mem::size_of::<GpuControllerHeader>() as u32 + 4;
    let response_size = core::mem::size_of::<GpuControllerHeader>() as u32 + core::mem::size_of::<VirglCapsetInformation>() as u32;
    
    let response_type = gpu.send_command(command_size, 512, response_size).ok()?;
    if response_type != GpuCtrlType::ResponseOkCapsetInformation as u32 { return None; }
    
    let dma = gpu.dma_buffer.as_ref()?;
    let information: VirglCapsetInformation = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.read_at(512 + core::mem::size_of::<GpuControllerHeader>()) };
    crate::serial_println!("[VIRGL] Capset: id={}, max_version={}, max_size={}", 
        information.capset_id, information.capset_maximum_version, information.capset_maximum_size);
    Some(information)
}

/// Create a VIRGL 3D rendering context
pub fn create_virgl_context(name: &str) -> Result<u32, &'static str> {
    let mut gpu = GPU.lock();
    if !gpu.has_3d { return Err("No 3D support"); }
    if !gpu.initialized { return Err("GPU not initialized"); }
    
    let context_id = 1u32; // Context ID 1 for primary rendering context
    
    let mut cmd = GpuContextCreate {
        header: GpuControllerHeader {
            controller_type: GpuCtrlType::CommandContextCreate as u32,
            context_id,
            ..Default::default()
        },
        nlen: name.len().minimum(63) as u32,
        context_initialize: 0,
        debug_name: [0u8; 64],
    };
    // Copy name
    let name_bytes = name.as_bytes();
    let copy_length = name_bytes.len().minimum(63);
    cmd.debug_name[..copy_length].copy_from_slice(&name_bytes[..copy_length]);
    
    let dma = gpu.dma_buffer.as_ref().ok_or("DMA not ready")?;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
    
    let response = gpu.send_command(
        core::mem::size_of::<GpuContextCreate>() as u32,
        512,
        core::mem::size_of::<GpuControllerHeader>() as u32,
    )?;
    
    if response != GpuCtrlType::ResponseOkNodata as u32 {
        return Err("CTX_CREATE failed");
    }
    
    let mut virgl = VIRGL_CONTEXT.lock();
    virgl.context_id = context_id;
    virgl.active = true;
    
    crate::serial_println!("[VIRGL] 3D context created: id={} name={}", context_id, name);
    Ok(context_id)
}

/// Destroy the VIRGL 3D context
pub fn destroy_virgl_context() -> Result<(), &'static str> {
    let mut virgl = VIRGL_CONTEXT.lock();
    if !virgl.active { return Ok(()); }
    
    let mut gpu = GPU.lock();
    let cmd = GpuContextDestroy {
        header: GpuControllerHeader {
            controller_type: GpuCtrlType::CommandContextDestroy as u32,
            context_id: virgl.context_id,
            ..Default::default()
        },
    };
    
    let dma = gpu.dma_buffer.as_ref().ok_or("DMA not ready")?;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
    
    let response = gpu.send_command(
        core::mem::size_of::<GpuContextDestroy>() as u32,
        512,
        core::mem::size_of::<GpuControllerHeader>() as u32,
    )?;
    
    if response != GpuCtrlType::ResponseOkNodata as u32 {
        return Err("CTX_DESTROY failed");
    }
    
    virgl.active = false;
    crate::serial_println!("[VIRGL] 3D context destroyed");
    Ok(())
}

/// Submit a VIRGL 3D command buffer (Gallium3D command stream)
pub fn submit_virgl_commands(commands: &[u8]) -> Result<(), &'static str> {
    let virgl = VIRGL_CONTEXT.lock();
    if !virgl.active { return Err("No active 3D context"); }
    let context_id = virgl.context_id;
    drop(virgl);
    
    let mut gpu = GPU.lock();
    if !gpu.initialized { return Err("GPU not initialized"); }
    // Max command size: DMA buffer is typically 4096 bytes, header takes ~24 bytes
    if commands.len() > 3800 { return Err("Command buffer too large"); }
    
    let dma = gpu.dma_buffer.as_ref().ok_or("DMA not ready")?;
    
    let cmd = GpuSubmit3d {
        header: GpuControllerHeader {
            controller_type: GpuCtrlType::CommandSubmit3d as u32,
            context_id,
            ..Default::default()
        },
        size: commands.len() as u32,
        padding: 0,
    };
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { dma.write_at(0, &cmd); }
    
    // Copy command data after header
    let command_header_size = core::mem::size_of::<GpuSubmit3d>();
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::ptr::copy_nonoverlapping(
            commands.as_pointer(),
            dma.virt.add(command_header_size),
            commands.len(),
        );
    }
    
    let total_command_size = (command_header_size + commands.len()) as u32;
    let response = gpu.send_command(total_command_size, 512, core::mem::size_of::<GpuControllerHeader>() as u32)?;
    
    if response != GpuCtrlType::ResponseOkNodata as u32 {
        return Err("SUBMIT_3D failed");
    }
    Ok(())
}

/// VIRGL status string for diagnostics
pub fn virgl_information() -> alloc::string::String {
    let gpu = GPU.lock();
    let virgl = VIRGL_CONTEXT.lock();
    if gpu.has_3d {
        alloc::format!("VIRGL: {} (ctx={})", 
            if virgl.active { "active" } else { "ready" },
            virgl.context_id)
    } else {
        alloc::string::String::from("VIRGL: not available")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compositor (compatibility)
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Layer {
    pub surface: GpuSurface,
    pub x: i32,
    pub y: i32,
    pub z_order: i32,
    pub visible: bool,
    pub opacity: u8,
}

// Structure publique — visible à l'extérieur de ce module.
pub struct Compositor {
    layers: Vec<Layer>,
    output: GpuSurface,
    background_color: u32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Compositor {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new(width: u32, height: u32) -> Self {
        Self { layers: Vec::new(), output: GpuSurface::new(width, height), background_color: 0xFF1A1A1A }
    }
        // Fonction publique — appelable depuis d'autres modules.
pub fn add_layer(&mut self, surface: GpuSurface, x: i32, y: i32, z_order: i32) -> usize {
        let index = self.layers.len();
        self.layers.push(Layer { surface, x, y, z_order, visible: true, opacity: 255 });
        self.layers.sort_by_key(|l| l.z_order);
        index
    }
        // Fonction publique — appelable depuis d'autres modules.
pub fn remove_layer(&mut self, index: usize) {
        if index < self.layers.len() { self.layers.remove(index); }
    }
        // Fonction publique — appelable depuis d'autres modules.
pub fn compose(&mut self) {
        self.output.clear(self.background_color);
        for layer in &self.layers {
            if layer.visible { self.output.blit(&layer.surface, layer.x, layer.y); }
        }
    }
        // Fonction publique — appelable depuis d'autres modules.
pub fn render(&self) { blit_to_screen(&self.output, 0, 0); }
        // Fonction publique — appelable depuis d'autres modules.
pub fn get_layer(&self, index: usize) -> Option<&Layer> { self.layers.get(index) }
        // Fonction publique — appelable depuis d'autres modules.
pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> { self.layers.get_mut(index) }
        // Fonction publique — appelable depuis d'autres modules.
pub fn set_background(&mut self, color: u32) { self.background_color = color; }
}
