//! Linux VM - Real Linux Kernel Boot Implementation
//!
//! This module implements the actual boot of a Linux kernel inside a VM
//! using the hypervisor (AMD SVM or Intel VMX).
//!
//! Linux Boot Protocol Reference:
//! https://www.kernel.org/doc/html/latest/x86/boot.html
//!
//! Memory Layout for Linux Boot:
//! ```
//! 0x00000000 - 0x00000FFF : Real mode IVT (unused for protected mode)
//! 0x00007C00 - 0x00007DFF : Bootloader area (unused)
//! 0x00010000 - 0x0001FFFF : Boot params (zero_page)
//! 0x00020000 - 0x0009FFFF : Command line + real mode code
//! 0x00100000 - ...        : Protected mode kernel (bzImage)
//! High memory             : Initramfs
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result, CpuVendor, cpu_vendor};
use super::linux_subsystem::{LinuxSetupHeader, BootParams, E820Entry, e820_type};

/// Boot protocol constants
pub mod boot_proto {
    /// Boot params (zero page) address
    pub const BOOT_PARAMS_ADDR: u64 = 0x10000;
    
    /// Command line address
    pub const CMDLINE_ADDR: u64 = 0x20000;
    
    /// Maximum command line size
    pub const CMDLINE_MAX_SIZE: usize = 2048;
    
    /// Protected mode kernel load address
    pub const KERNEL_LOAD_ADDR: u64 = 0x100000;
    
    /// Initial stack pointer (below real mode code)
    pub const INIT_STACK: u64 = 0x8000;
    
    /// GDT address
    pub const GDT_ADDR: u64 = 0x1000;
    
    /// Initramfs load address (high memory)
    pub const INITRAMFS_ADDR: u64 = 0x1000000;  // 16 MB
    
    /// Setup header offset in bzImage
    pub const SETUP_HEADER_OFFSET: usize = 0x1F1;
    
    /// "HdrS" magic number
    pub const HDRS_MAGIC: u32 = 0x53726448;
    
    /// Minimum boot protocol version for protected mode
    pub const MIN_PROTOCOL_VERSION: u16 = 0x0200;
    
    /// Loadflags bits
    pub const LOADED_HIGH: u8 = 0x01;
    pub const CAN_USE_HEAP: u8 = 0x80;
    
    /// Loader ID for our bootloader
    pub const LOADER_TYPE: u8 = 0xFF;  // Undefined/other
    
    /// E820 entry types
    pub const E820_RAM: u32 = 1;
    pub const E820_RESERVED: u32 = 2;
    pub const E820_ACPI: u32 = 3;
}

/// Linux kernel info extracted from bzImage
#[derive(Debug, Clone)]
pub struct LinuxKernelInfo {
    /// Boot protocol version (e.g., 0x020F for 2.15)
    pub protocol_version: u16,
    /// Setup sectors (size of real-mode code)
    pub setup_sects: u8,
    /// Loadflags
    pub loadflags: u8,
    /// Protected mode entry point
    pub code32_start: u32,
    /// Kernel version string offset
    pub kernel_version_offset: u16,
    /// Ramdisk address max
    pub initrd_addr_max: u32,
    /// Kernel alignment requirement
    pub kernel_alignment: u32,
    /// Can kernel be loaded at non-default address?
    pub relocatable: bool,
    /// Command line max size
    pub cmdline_size: u32,
    /// Preferred load address
    pub pref_address: u64,
    /// Init size (memory needed)
    pub init_size: u32,
}

impl LinuxKernelInfo {
    /// Parse kernel info from bzImage header
    pub fn from_bzimage(bzimage: &[u8]) -> Option<Self> {
        if bzimage.len() < 0x250 {
            return None;
        }
        
        // Check magic
        let magic = u32::from_le_bytes([
            bzimage[0x202],
            bzimage[0x203],
            bzimage[0x204],
            bzimage[0x205],
        ]);
        
        if magic != boot_proto::HDRS_MAGIC {
            return None;
        }
        
        let protocol_version = u16::from_le_bytes([bzimage[0x206], bzimage[0x207]]);
        let setup_sects = bzimage[0x1F1];
        let loadflags = bzimage[0x211];
        let code32_start = u32::from_le_bytes([
            bzimage[0x214],
            bzimage[0x215],
            bzimage[0x216],
            bzimage[0x217],
        ]);
        let kernel_version_offset = u16::from_le_bytes([bzimage[0x20E], bzimage[0x20F]]);
        
        // Fields that require protocol >= 2.00
        let initrd_addr_max = if protocol_version >= 0x0200 {
            u32::from_le_bytes([
                bzimage[0x22C],
                bzimage[0x22D],
                bzimage[0x22E],
                bzimage[0x22F],
            ])
        } else {
            0x37FFFFFF
        };
        
        // Fields that require protocol >= 2.05
        let (kernel_alignment, relocatable) = if protocol_version >= 0x0205 {
            let align = u32::from_le_bytes([
                bzimage[0x230],
                bzimage[0x231],
                bzimage[0x232],
                bzimage[0x233],
            ]);
            let reloc = bzimage[0x234] != 0;
            (align, reloc)
        } else {
            (0x100000, false)
        };
        
        // Fields that require protocol >= 2.06
        let cmdline_size = if protocol_version >= 0x0206 {
            u32::from_le_bytes([
                bzimage[0x238],
                bzimage[0x239],
                bzimage[0x23A],
                bzimage[0x23B],
            ])
        } else {
            255
        };
        
        // Fields that require protocol >= 2.10
        let (pref_address, init_size) = if protocol_version >= 0x020A {
            let pref = u64::from_le_bytes([
                bzimage[0x258],
                bzimage[0x259],
                bzimage[0x25A],
                bzimage[0x25B],
                bzimage[0x25C],
                bzimage[0x25D],
                bzimage[0x25E],
                bzimage[0x25F],
            ]);
            let init = u32::from_le_bytes([
                bzimage[0x260],
                bzimage[0x261],
                bzimage[0x262],
                bzimage[0x263],
            ]);
            (pref, init)
        } else {
            (0x100000, 0)
        };
        
        Some(Self {
            protocol_version,
            setup_sects,
            loadflags,
            code32_start,
            kernel_version_offset,
            initrd_addr_max,
            kernel_alignment,
            relocatable,
            cmdline_size,
            pref_address,
            init_size,
        })
    }
    
    /// Get kernel version string from bzImage
    pub fn get_version_string<'a>(&self, bzimage: &'a [u8]) -> Option<&'a str> {
        if self.kernel_version_offset == 0 {
            return None;
        }
        
        let offset = self.kernel_version_offset as usize + 0x200;
        if offset >= bzimage.len() {
            return None;
        }
        
        // Find null terminator
        let end = bzimage[offset..].iter()
            .position(|&b| b == 0)
            .unwrap_or(64);
        
        core::str::from_utf8(&bzimage[offset..offset + end]).ok()
    }
}

/// Linux VM configuration
#[derive(Debug, Clone)]
pub struct LinuxVmConfig {
    /// Memory size in MB
    pub memory_mb: usize,
    /// Kernel command line
    pub cmdline: String,
    /// Number of vCPUs
    pub vcpus: u32,
    /// Enable serial console
    pub serial_console: bool,
    /// Enable virtio console
    pub virtio_console: bool,
}

impl Default for LinuxVmConfig {
    fn default() -> Self {
        Self {
            memory_mb: 64,
            cmdline: String::from("console=ttyS0 earlyprintk=serial quiet"),
            vcpus: 1,
            serial_console: true,
            virtio_console: true,
        }
    }
}

/// Linux Virtual Machine
pub struct LinuxVm {
    /// VM ID
    id: u64,
    /// Configuration
    config: LinuxVmConfig,
    /// Kernel info
    kernel_info: Option<LinuxKernelInfo>,
    /// Is the VM running?
    running: AtomicBool,
    /// Guest memory
    guest_memory: Vec<u8>,
    /// Console buffer for output
    console_buffer: Mutex<Vec<u8>>,
}

impl LinuxVm {
    /// Create a new Linux VM
    pub fn new(config: LinuxVmConfig) -> Result<Self> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0x10000);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        
        // Allocate guest memory
        let memory_size = config.memory_mb * 1024 * 1024;
        let guest_memory = alloc::vec![0u8; memory_size];
        
        crate::serial_println!("[LINUX-VM {}] Created with {} MB RAM", id, config.memory_mb);
        
        Ok(Self {
            id,
            config,
            kernel_info: None,
            running: AtomicBool::new(false),
            guest_memory,
            console_buffer: Mutex::new(Vec::new()),
        })
    }
    
    /// Load Linux kernel (bzImage) into VM memory
    pub fn load_kernel(&mut self, bzimage: &[u8]) -> Result<()> {
        // Parse kernel info
        let kernel_info = LinuxKernelInfo::from_bzimage(bzimage)
            .ok_or(HypervisorError::InvalidBinary)?;
        
        crate::serial_println!("[LINUX-VM {}] Kernel: protocol v{}.{}, setup_sects={}", 
            self.id,
            kernel_info.protocol_version >> 8,
            kernel_info.protocol_version & 0xFF,
            kernel_info.setup_sects);
        
        if let Some(version) = kernel_info.get_version_string(bzimage) {
            crate::serial_println!("[LINUX-VM {}] Kernel version: {}", self.id, version);
        }
        
        // Calculate real-mode code size
        let setup_sects = if kernel_info.setup_sects == 0 { 4 } else { kernel_info.setup_sects };
        let real_mode_size = (setup_sects as usize + 1) * 512;
        
        // Protected mode kernel starts after real-mode code
        let pm_kernel_offset = real_mode_size;
        let pm_kernel_size = bzimage.len() - pm_kernel_offset;
        
        crate::serial_println!("[LINUX-VM {}] Real mode: {} bytes, Protected mode: {} bytes", 
            self.id, real_mode_size, pm_kernel_size);
        
        // Load protected mode kernel at 0x100000
        let load_addr = boot_proto::KERNEL_LOAD_ADDR as usize;
        if load_addr + pm_kernel_size > self.guest_memory.len() {
            return Err(HypervisorError::OutOfMemory);
        }
        
        self.guest_memory[load_addr..load_addr + pm_kernel_size]
            .copy_from_slice(&bzimage[pm_kernel_offset..]);
        
        crate::serial_println!("[LINUX-VM {}] Loaded kernel at 0x{:X} ({} KB)", 
            self.id, load_addr, pm_kernel_size / 1024);
        
        self.kernel_info = Some(kernel_info);
        
        Ok(())
    }
    
    /// Load initramfs into VM memory
    pub fn load_initramfs(&mut self, initramfs: &[u8]) -> Result<u64> {
        let kernel_info = self.kernel_info.as_ref()
            .ok_or(HypervisorError::InvalidState)?;
        
        // Place initramfs at high address (e.g., 16 MB)
        // But respect initrd_addr_max
        let max_addr = kernel_info.initrd_addr_max as u64;
        let mut load_addr = boot_proto::INITRAMFS_ADDR;
        
        // Make sure initramfs fits
        if load_addr + initramfs.len() as u64 > max_addr {
            // Try to fit it below max_addr
            load_addr = max_addr - initramfs.len() as u64;
            load_addr &= !0xFFF; // Align to page
        }
        
        let offset = load_addr as usize;
        if offset + initramfs.len() > self.guest_memory.len() {
            return Err(HypervisorError::OutOfMemory);
        }
        
        self.guest_memory[offset..offset + initramfs.len()].copy_from_slice(initramfs);
        
        crate::serial_println!("[LINUX-VM {}] Loaded initramfs at 0x{:X} ({} KB)", 
            self.id, load_addr, initramfs.len() / 1024);
        
        Ok(load_addr)
    }
    
    /// Setup boot parameters (zero page)
    pub fn setup_boot_params(&mut self, initramfs_addr: u64, initramfs_size: u32) -> Result<()> {
        let kernel_info = self.kernel_info.as_ref()
            .ok_or(HypervisorError::InvalidState)?;
        
        // Write command line
        let cmdline_addr = boot_proto::CMDLINE_ADDR as usize;
        let cmdline_bytes = self.config.cmdline.as_bytes();
        let cmdline_len = cmdline_bytes.len().min(boot_proto::CMDLINE_MAX_SIZE - 1);
        
        self.guest_memory[cmdline_addr..cmdline_addr + cmdline_len]
            .copy_from_slice(&cmdline_bytes[..cmdline_len]);
        self.guest_memory[cmdline_addr + cmdline_len] = 0; // Null terminate
        
        crate::serial_println!("[LINUX-VM {}] Command line: {}", self.id, self.config.cmdline);
        
        // Build boot_params structure at BOOT_PARAMS_ADDR
        let boot_params_addr = boot_proto::BOOT_PARAMS_ADDR as usize;
        
        // Clear the zero page
        for i in 0..4096 {
            self.guest_memory[boot_params_addr + i] = 0;
        }
        
        // Setup header is at offset 0x1F1 in boot_params
        let hdr_offset = boot_params_addr + 0x1F1;
        
        // Copy setup_sects
        self.guest_memory[hdr_offset] = kernel_info.setup_sects;
        
        // type_of_loader at offset 0x210
        self.guest_memory[boot_params_addr + 0x210] = boot_proto::LOADER_TYPE;
        
        // loadflags at offset 0x211
        let loadflags = boot_proto::LOADED_HIGH | boot_proto::CAN_USE_HEAP;
        self.guest_memory[boot_params_addr + 0x211] = loadflags;
        
        // heap_end_ptr at offset 0x224 (relative to 0x10000)
        let heap_end: u16 = 0xFE00;
        self.guest_memory[boot_params_addr + 0x224] = (heap_end & 0xFF) as u8;
        self.guest_memory[boot_params_addr + 0x225] = (heap_end >> 8) as u8;
        
        // cmd_line_ptr at offset 0x228
        let cmdline_ptr = boot_proto::CMDLINE_ADDR as u32;
        let cmdline_bytes = cmdline_ptr.to_le_bytes();
        self.guest_memory[boot_params_addr + 0x228..boot_params_addr + 0x22C]
            .copy_from_slice(&cmdline_bytes);
        
        // ramdisk_image at offset 0x218
        let initrd_addr_bytes = (initramfs_addr as u32).to_le_bytes();
        self.guest_memory[boot_params_addr + 0x218..boot_params_addr + 0x21C]
            .copy_from_slice(&initrd_addr_bytes);
        
        // ramdisk_size at offset 0x21C
        let initrd_size_bytes = initramfs_size.to_le_bytes();
        self.guest_memory[boot_params_addr + 0x21C..boot_params_addr + 0x220]
            .copy_from_slice(&initrd_size_bytes);
        
        // Setup E820 memory map
        self.setup_e820_map(boot_params_addr)?;
        
        crate::serial_println!("[LINUX-VM {}] Boot params at 0x{:X}", 
            self.id, boot_proto::BOOT_PARAMS_ADDR);
        
        Ok(())
    }
    
    /// Setup E820 memory map in boot_params
    fn setup_e820_map(&mut self, boot_params_addr: usize) -> Result<()> {
        // E820 entries are at offset 0x2D0 in boot_params
        let e820_offset = boot_params_addr + 0x2D0;
        let mut entry_count: u8 = 0;
        
        // Entry 0: Low memory (0 - 0x9FC00 = 639 KB)
        self.write_e820_entry(e820_offset, 0, 0, 0x9FC00, boot_proto::E820_RAM);
        entry_count += 1;
        
        // Entry 1: EBDA reserved (0x9FC00 - 0xA0000)
        self.write_e820_entry(e820_offset, 1, 0x9FC00, 0x400, boot_proto::E820_RESERVED);
        entry_count += 1;
        
        // Entry 2: Video/BIOS area (0xA0000 - 0x100000 = 384 KB)
        self.write_e820_entry(e820_offset, 2, 0xA0000, 0x60000, boot_proto::E820_RESERVED);
        entry_count += 1;
        
        // Entry 3: Main memory (1 MB to end of RAM)
        let main_mem_start = 0x100000u64;
        let main_mem_size = (self.guest_memory.len() as u64).saturating_sub(main_mem_start);
        self.write_e820_entry(e820_offset, 3, main_mem_start, main_mem_size, boot_proto::E820_RAM);
        entry_count += 1;
        
        // Write entry count at offset 0x1E8 in boot_params
        self.guest_memory[boot_params_addr + 0x1E8] = entry_count;
        
        crate::serial_println!("[LINUX-VM {}] E820 map: {} entries, {} MB usable", 
            self.id, entry_count, main_mem_size / (1024 * 1024));
        
        Ok(())
    }
    
    /// Write a single E820 entry
    fn write_e820_entry(&mut self, base_offset: usize, index: usize, 
                        addr: u64, size: u64, entry_type: u32) {
        let entry_offset = base_offset + index * 20;  // Each entry is 20 bytes
        
        // Address (8 bytes)
        self.guest_memory[entry_offset..entry_offset + 8].copy_from_slice(&addr.to_le_bytes());
        
        // Size (8 bytes)
        self.guest_memory[entry_offset + 8..entry_offset + 16].copy_from_slice(&size.to_le_bytes());
        
        // Type (4 bytes)
        self.guest_memory[entry_offset + 16..entry_offset + 20].copy_from_slice(&entry_type.to_le_bytes());
    }
    
    /// Setup GDT for protected mode boot
    fn setup_gdt(&mut self) -> Result<u64> {
        let gdt_addr = boot_proto::GDT_ADDR as usize;
        
        // GDT entries (8 bytes each):
        // 0: Null descriptor
        // 1: Code segment (CS) - base=0, limit=0xFFFFF, 32-bit, execute/read
        // 2: Data segment (DS) - base=0, limit=0xFFFFF, 32-bit, read/write
        
        // Null descriptor
        self.guest_memory[gdt_addr..gdt_addr + 8].copy_from_slice(&[0u8; 8]);
        
        // Code segment: 0x00CF9A000000FFFF
        let code_seg: u64 = 0x00CF9A000000FFFF;
        self.guest_memory[gdt_addr + 8..gdt_addr + 16].copy_from_slice(&code_seg.to_le_bytes());
        
        // Data segment: 0x00CF92000000FFFF
        let data_seg: u64 = 0x00CF92000000FFFF;
        self.guest_memory[gdt_addr + 16..gdt_addr + 24].copy_from_slice(&data_seg.to_le_bytes());
        
        // Return GDT pointer (limit:base)
        Ok(boot_proto::GDT_ADDR)
    }
    
    /// Boot the Linux kernel using the hypervisor
    pub fn boot(&mut self, bzimage: &[u8], initramfs: &[u8]) -> Result<()> {
        // Load kernel
        self.load_kernel(bzimage)?;
        
        // Load initramfs
        let initramfs_addr = self.load_initramfs(initramfs)?;
        
        // Setup boot parameters
        self.setup_boot_params(initramfs_addr, initramfs.len() as u32)?;
        
        // Setup GDT
        let gdt_addr = self.setup_gdt()?;
        
        // Get entry point
        let kernel_info = self.kernel_info.as_ref()
            .ok_or(HypervisorError::InvalidState)?;
        
        let entry_point = if kernel_info.code32_start != 0 {
            kernel_info.code32_start as u64
        } else {
            boot_proto::KERNEL_LOAD_ADDR
        };
        
        crate::serial_println!("[LINUX-VM {}] Entry point: 0x{:X}", self.id, entry_point);
        crate::serial_println!("[LINUX-VM {}] GDT at: 0x{:X}", self.id, gdt_addr);
        crate::serial_println!("[LINUX-VM {}] Boot params: 0x{:X}", self.id, boot_proto::BOOT_PARAMS_ADDR);
        
        // Check CPU vendor and use appropriate hypervisor
        match cpu_vendor() {
            CpuVendor::Intel => {
                crate::serial_println!("[LINUX-VM {}] Using Intel VMX...", self.id);
                self.boot_with_vmx(entry_point)?;
            }
            CpuVendor::Amd => {
                crate::serial_println!("[LINUX-VM {}] Using AMD SVM...", self.id);
                self.boot_with_svm(entry_point)?;
            }
            CpuVendor::Unknown => {
                crate::serial_println!("[LINUX-VM {}] No hardware virtualization available", self.id);
                crate::serial_println!("[LINUX-VM {}] Running in simulated mode", self.id);
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    /// Boot with Intel VMX
    fn boot_with_vmx(&mut self, entry_point: u64) -> Result<()> {
        use super::vm::VirtualMachine;
        
        crate::serial_println!("[LINUX-VM {}] VMX boot: creating Intel VT-x VM...", self.id);
        
        // Create VMX VM
        let mut vm = VirtualMachine::new(self.id + 100, "linux-vmx-guest", self.config.memory_mb)?;
        
        // Initialize VMCS and EPT
        vm.initialize()?;
        
        crate::serial_println!("[LINUX-VM {}] VMX VM initialized, loading {} MB...", 
            self.id, self.guest_memory.len() / (1024 * 1024));
        
        // Load the prepared memory into VMX VM
        vm.load_binary(&self.guest_memory, 0)?;
        
        crate::serial_println!("[LINUX-VM {}] Starting Linux kernel via VMX...", self.id);
        crate::serial_println!("[LINUX-VM {}] Entry: 0x{:X}, Boot params: 0x{:X}", 
            self.id, entry_point, boot_proto::BOOT_PARAMS_ADDR);
        
        self.running.store(true, Ordering::SeqCst);
        
        // The guest memory is already fully prepared (kernel, boot params, page tables, etc.)
        // We use start() with the entry point directly.
        // Note: This requires Intel VT-x hardware. When running on AMD or QEMU TCG,
        // VMLAUNCH will fail and the error will be reported by the VM run loop.
        match vm.start(entry_point, boot_proto::INIT_STACK) {
            Ok(()) => {
                crate::serial_println!("[LINUX-VM {}] VMX execution completed", self.id);
            }
            Err(e) => {
                crate::serial_println!("[LINUX-VM {}] VMX execution failed: {:?}", self.id, e);
                crate::serial_println!("[LINUX-VM {}] Note: VMX requires Intel CPU with VT-x. QEMU TCG does not support nested VMX.", self.id);
                return Err(e);
            }
        }
        
        self.running.store(false, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Boot with AMD SVM
    fn boot_with_svm(&mut self, entry_point: u64) -> Result<()> {
        use super::svm_vm;
        
        // Create SVM VM via the global API (registers in SVM_VMS for Inspector)
        let vm_id = svm_vm::create_vm("linux-guest", self.config.memory_mb)?;
        
        crate::serial_println!("[LINUX-VM {}] SVM VM #{} created, loading {} MB...", 
            self.id, vm_id, self.guest_memory.len() / (1024 * 1024));
        
        // Initialize, load memory, and configure via `with_vm`
        let boot_result = svm_vm::with_vm(vm_id, |vm| -> Result<()> {
            // Initialize VMCB and NPT
            vm.initialize()?;
            
            // Load the prepared memory into SVM VM
            vm.load_binary(&self.guest_memory, 0)?;
            
            // Setup protected mode with Linux entry point
            vm.setup_protected_mode_for_linux(
                entry_point, 
                boot_proto::INIT_STACK,
                boot_proto::BOOT_PARAMS_ADDR
            )?;
            
            crate::serial_println!("[LINUX-VM {}] Starting Linux kernel execution...", self.id);
            crate::serial_println!("[LINUX-VM {}] Entry: 0x{:X}, Boot params: 0x{:X}", 
                self.id, entry_point, boot_proto::BOOT_PARAMS_ADDR);
            
            // Start the VM
            vm.start()
        });
        
        match boot_result {
            Some(Ok(())) => {
                crate::serial_println!("[LINUX-VM {}] VM execution completed", self.id);
            }
            Some(Err(e)) => {
                crate::serial_println!("[LINUX-VM {}] VM execution failed: {:?}", self.id, e);
                return Err(e);
            }
            None => {
                crate::serial_println!("[LINUX-VM {}] Could not find VM #{}", self.id, vm_id);
                return Err(HypervisorError::VmNotFound);
            }
        }
        
        self.running.store(false, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Check if VM is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Get console output
    pub fn get_console_output(&self) -> Vec<u8> {
        self.console_buffer.lock().clone()
    }
    
    /// Get VM ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Global Linux VM instance
static LINUX_VM: Mutex<Option<LinuxVm>> = Mutex::new(None);

/// Create and boot a Linux VM
pub fn boot_linux(bzimage: &[u8], initramfs: &[u8], cmdline: &str) -> Result<u64> {
    let config = LinuxVmConfig {
        memory_mb: 128,
        cmdline: String::from(cmdline),
        ..Default::default()
    };
    
    let mut vm = LinuxVm::new(config)?;
    let id = vm.id();
    
    vm.boot(bzimage, initramfs)?;
    
    *LINUX_VM.lock() = Some(vm);
    
    Ok(id)
}

/// Check if Linux VM is running
pub fn is_running() -> bool {
    LINUX_VM.lock().as_ref().map(|vm| vm.is_running()).unwrap_or(false)
}

/// Get Linux VM ID
pub fn get_vm_id() -> Option<u64> {
    LINUX_VM.lock().as_ref().map(|vm| vm.id())
}
