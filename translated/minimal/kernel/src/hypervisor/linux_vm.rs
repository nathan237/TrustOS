

















use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result, CpuVendor, cpu_vendor};
use super::linux_subsystem::{Aau, Ahn, Ru, e820_type};


pub mod boot_proto {
    
    pub const DY_: u64 = 0x10000;
    
    
    pub const II_: u64 = 0x20000;
    
    
    pub const BPH_: usize = 2048;
    
    
    pub const FY_: u64 = 0x100000;
    
    
    pub const AZA_: u64 = 0x8000;
    
    
    pub const IV_: u64 = 0x1000;
    
    
    pub const CFB_: u64 = 0x1000000;  
    
    
    pub const EJE_: usize = 0x1F1;
    
    
    pub const CCK_: u32 = 0x53726448;
    
    
    pub const DXW_: u16 = 0x0200;
    
    
    pub const WC_: u8 = 0x01;
    pub const ABG_: u8 = 0x80;
    
    
    pub const CIA_: u8 = 0xFF;  
    
    
    pub const ATA_: u32 = 1;
    pub const ATB_: u32 = 2;
    pub const DOO_: u32 = 3;
}


#[derive(Debug, Clone)]
pub struct LinuxKernelInfo {
    
    pub protocol_version: u16,
    
    pub setup_sects: u8,
    
    pub btz: u8,
    
    pub code32_start: u32,
    
    pub kernel_version_offset: u16,
    
    pub initrd_addr_max: u32,
    
    pub kernel_alignment: u32,
    
    pub relocatable: bool,
    
    pub fnk: u32,
    
    pub pref_address: u64,
    
    pub init_size: u32,
}

impl LinuxKernelInfo {
    
    pub fn lzb(jx: &[u8]) -> Option<Self> {
        if jx.len() < 0x250 {
            return None;
        }
        
        
        let magic = u32::from_le_bytes([
            jx[0x202],
            jx[0x203],
            jx[0x204],
            jx[0x205],
        ]);
        
        if magic != boot_proto::CCK_ {
            return None;
        }
        
        let protocol_version = u16::from_le_bytes([jx[0x206], jx[0x207]]);
        let setup_sects = jx[0x1F1];
        let btz = jx[0x211];
        let code32_start = u32::from_le_bytes([
            jx[0x214],
            jx[0x215],
            jx[0x216],
            jx[0x217],
        ]);
        let kernel_version_offset = u16::from_le_bytes([jx[0x20E], jx[0x20F]]);
        
        
        let initrd_addr_max = if protocol_version >= 0x0200 {
            u32::from_le_bytes([
                jx[0x22C],
                jx[0x22D],
                jx[0x22E],
                jx[0x22F],
            ])
        } else {
            0x37FFFFFF
        };
        
        
        let (kernel_alignment, relocatable) = if protocol_version >= 0x0205 {
            let align = u32::from_le_bytes([
                jx[0x230],
                jx[0x231],
                jx[0x232],
                jx[0x233],
            ]);
            let bdg = jx[0x234] != 0;
            (align, bdg)
        } else {
            (0x100000, false)
        };
        
        
        let fnk = if protocol_version >= 0x0206 {
            u32::from_le_bytes([
                jx[0x238],
                jx[0x239],
                jx[0x23A],
                jx[0x23B],
            ])
        } else {
            255
        };
        
        
        let (pref_address, init_size) = if protocol_version >= 0x020A {
            let amg = u64::from_le_bytes([
                jx[0x258],
                jx[0x259],
                jx[0x25A],
                jx[0x25B],
                jx[0x25C],
                jx[0x25D],
                jx[0x25E],
                jx[0x25F],
            ]);
            let init = u32::from_le_bytes([
                jx[0x260],
                jx[0x261],
                jx[0x262],
                jx[0x263],
            ]);
            (amg, init)
        } else {
            (0x100000, 0)
        };
        
        Some(Self {
            protocol_version,
            setup_sects,
            btz,
            code32_start,
            kernel_version_offset,
            initrd_addr_max,
            kernel_alignment,
            relocatable,
            fnk,
            pref_address,
            init_size,
        })
    }
    
    
    pub fn get_version_string<'a>(&self, jx: &'a [u8]) -> Option<&'a str> {
        if self.kernel_version_offset == 0 {
            return None;
        }
        
        let offset = self.kernel_version_offset as usize + 0x200;
        if offset >= jx.len() {
            return None;
        }
        
        
        let end = jx[offset..].iter()
            .position(|&b| b == 0)
            .unwrap_or(64);
        
        core::str::from_utf8(&jx[offset..offset + end]).ok()
    }
}


#[derive(Debug, Clone)]
pub struct Gq {
    
    pub memory_mb: usize,
    
    pub cmdline: String,
    
    pub vcpus: u32,
    
    pub serial_console: bool,
    
    pub virtio_console: bool,
}

impl Default for Gq {
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


pub struct LinuxVm {
    
    id: u64,
    
    config: Gq,
    
    kernel_info: Option<LinuxKernelInfo>,
    
    running: AtomicBool,
    
    guest_memory: Vec<u8>,
    
    console_buffer: Mutex<Vec<u8>>,
}

impl LinuxVm {
    
    pub fn new(config: Gq) -> Result<Self> {
        static AHR_: AtomicU64 = AtomicU64::new(0x10000);
        let id = AHR_.fetch_add(1, Ordering::SeqCst);
        
        
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
    
    
    pub fn load_kernel(&mut self, jx: &[u8]) -> Result<()> {
        
        let kernel_info = LinuxKernelInfo::lzb(jx)
            .ok_or(HypervisorError::InvalidBinary)?;
        
        crate::serial_println!("[LINUX-VM {}] Kernel: protocol v{}.{}, setup_sects={}", 
            self.id,
            kernel_info.protocol_version >> 8,
            kernel_info.protocol_version & 0xFF,
            kernel_info.setup_sects);
        
        if let Some(version) = kernel_info.get_version_string(jx) {
            crate::serial_println!("[LINUX-VM {}] Kernel version: {}", self.id, version);
        }
        
        
        let setup_sects = if kernel_info.setup_sects == 0 { 4 } else { kernel_info.setup_sects };
        let iyo = (setup_sects as usize + 1) * 512;
        
        
        let ivi = iyo;
        let ewt = jx.len() - ivi;
        
        crate::serial_println!("[LINUX-VM {}] Real mode: {} bytes, Protected mode: {} bytes", 
            self.id, iyo, ewt);
        
        
        let bhp = boot_proto::FY_ as usize;
        if bhp + ewt > self.guest_memory.len() {
            return Err(HypervisorError::OutOfMemory);
        }
        
        self.guest_memory[bhp..bhp + ewt]
            .copy_from_slice(&jx[ivi..]);
        
        crate::serial_println!("[LINUX-VM {}] Loaded kernel at 0x{:X} ({} KB)", 
            self.id, bhp, ewt / 1024);
        
        self.kernel_info = Some(kernel_info);
        
        Ok(())
    }
    
    
    pub fn load_initramfs(&mut self, initramfs: &[u8]) -> Result<u64> {
        let kernel_info = self.kernel_info.as_ref()
            .ok_or(HypervisorError::InvalidState)?;
        
        
        
        let cmt = kernel_info.initrd_addr_max as u64;
        let mut bhp = boot_proto::CFB_;
        
        
        if bhp + initramfs.len() as u64 > cmt {
            
            bhp = cmt - initramfs.len() as u64;
            bhp &= !0xFFF; 
        }
        
        let offset = bhp as usize;
        if offset + initramfs.len() > self.guest_memory.len() {
            return Err(HypervisorError::OutOfMemory);
        }
        
        self.guest_memory[offset..offset + initramfs.len()].copy_from_slice(initramfs);
        
        crate::serial_println!("[LINUX-VM {}] Loaded initramfs at 0x{:X} ({} KB)", 
            self.id, bhp, initramfs.len() / 1024);
        
        Ok(bhp)
    }
    
    
    pub fn setup_boot_params(&mut self, gcs: u64, initramfs_size: u32) -> Result<()> {
        let kernel_info = self.kernel_info.as_ref()
            .ok_or(HypervisorError::InvalidState)?;
        
        
        let fnj = boot_proto::II_ as usize;
        let bqz = self.config.cmdline.as_bytes();
        let chi = bqz.len().min(boot_proto::BPH_ - 1);
        
        self.guest_memory[fnj..fnj + chi]
            .copy_from_slice(&bqz[..chi]);
        self.guest_memory[fnj + chi] = 0; 
        
        crate::serial_println!("[LINUX-VM {}] Command line: {}", self.id, self.config.cmdline);
        
        
        let boot_params_addr = boot_proto::DY_ as usize;
        
        
        for i in 0..4096 {
            self.guest_memory[boot_params_addr + i] = 0;
        }
        
        
        let mkm = boot_params_addr + 0x1F1;
        
        
        self.guest_memory[mkm] = kernel_info.setup_sects;
        
        
        self.guest_memory[boot_params_addr + 0x210] = boot_proto::CIA_;
        
        
        let btz = boot_proto::WC_ | boot_proto::ABG_;
        self.guest_memory[boot_params_addr + 0x211] = btz;
        
        
        let heap_end: u16 = 0xFE00;
        self.guest_memory[boot_params_addr + 0x224] = (heap_end & 0xFF) as u8;
        self.guest_memory[boot_params_addr + 0x225] = (heap_end >> 8) as u8;
        
        
        let kuj = boot_proto::II_ as u32;
        let bqz = kuj.to_le_bytes();
        self.guest_memory[boot_params_addr + 0x228..boot_params_addr + 0x22C]
            .copy_from_slice(&bqz);
        
        
        let mpv = (gcs as u32).to_le_bytes();
        self.guest_memory[boot_params_addr + 0x218..boot_params_addr + 0x21C]
            .copy_from_slice(&mpv);
        
        
        let mpz = initramfs_size.to_le_bytes();
        self.guest_memory[boot_params_addr + 0x21C..boot_params_addr + 0x220]
            .copy_from_slice(&mpz);
        
        
        self.setup_e820_map(boot_params_addr)?;
        
        crate::serial_println!("[LINUX-VM {}] Boot params at 0x{:X}", 
            self.id, boot_proto::DY_);
        
        Ok(())
    }
    
    
    fn setup_e820_map(&mut self, boot_params_addr: usize) -> Result<()> {
        
        let elb = boot_params_addr + 0x2D0;
        let mut entry_count: u8 = 0;
        
        
        self.write_e820_entry(elb, 0, 0, 0x9FC00, boot_proto::ATA_);
        entry_count += 1;
        
        
        self.write_e820_entry(elb, 1, 0x9FC00, 0x400, boot_proto::ATB_);
        entry_count += 1;
        
        
        self.write_e820_entry(elb, 2, 0xA0000, 0x60000, boot_proto::ATB_);
        entry_count += 1;
        
        
        let ilp = 0x100000u64;
        let etq = (self.guest_memory.len() as u64).saturating_sub(ilp);
        self.write_e820_entry(elb, 3, ilp, etq, boot_proto::ATA_);
        entry_count += 1;
        
        
        self.guest_memory[boot_params_addr + 0x1E8] = entry_count;
        
        crate::serial_println!("[LINUX-VM {}] E820 map: {} entries, {} MB usable", 
            self.id, entry_count, etq / (1024 * 1024));
        
        Ok(())
    }
    
    
    fn write_e820_entry(&mut self, base_offset: usize, index: usize, 
                        addr: u64, size: u64, entry_type: u32) {
        let entry_offset = base_offset + index * 20;  
        
        
        self.guest_memory[entry_offset..entry_offset + 8].copy_from_slice(&addr.to_le_bytes());
        
        
        self.guest_memory[entry_offset + 8..entry_offset + 16].copy_from_slice(&size.to_le_bytes());
        
        
        self.guest_memory[entry_offset + 16..entry_offset + 20].copy_from_slice(&entry_type.to_le_bytes());
    }
    
    
    fn setup_gdt(&mut self) -> Result<u64> {
        let cac = boot_proto::IV_ as usize;
        
        
        
        
        
        
        
        self.guest_memory[cac..cac + 8].copy_from_slice(&[0u8; 8]);
        
        
        let kut: u64 = 0x00CF9A000000FFFF;
        self.guest_memory[cac + 8..cac + 16].copy_from_slice(&kut.to_le_bytes());
        
        
        let lbt: u64 = 0x00CF92000000FFFF;
        self.guest_memory[cac + 16..cac + 24].copy_from_slice(&lbt.to_le_bytes());
        
        
        Ok(boot_proto::IV_)
    }
    
    
    pub fn boot(&mut self, jx: &[u8], initramfs: &[u8]) -> Result<()> {
        
        self.load_kernel(jx)?;
        
        
        let gcs = self.load_initramfs(initramfs)?;
        
        
        self.setup_boot_params(gcs, initramfs.len() as u32)?;
        
        
        let cac = self.setup_gdt()?;
        
        
        let kernel_info = self.kernel_info.as_ref()
            .ok_or(HypervisorError::InvalidState)?;
        
        let entry_point = if kernel_info.code32_start != 0 {
            kernel_info.code32_start as u64
        } else {
            boot_proto::FY_
        };
        
        crate::serial_println!("[LINUX-VM {}] Entry point: 0x{:X}", self.id, entry_point);
        crate::serial_println!("[LINUX-VM {}] GDT at: 0x{:X}", self.id, cac);
        crate::serial_println!("[LINUX-VM {}] Boot params: 0x{:X}", self.id, boot_proto::DY_);
        
        
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
    
    
    fn boot_with_vmx(&mut self, entry_point: u64) -> Result<()> {
        use super::vm::VirtualMachine;
        
        crate::serial_println!("[LINUX-VM {}] VMX boot: creating Intel VT-x VM...", self.id);
        
        
        let mut vm = VirtualMachine::new(self.id + 100, "linux-vmx-guest", self.config.memory_mb)?;
        
        
        vm.initialize()?;
        
        crate::serial_println!("[LINUX-VM {}] VMX VM initialized, loading {} MB...", 
            self.id, self.guest_memory.len() / (1024 * 1024));
        
        
        vm.load_binary(&self.guest_memory, 0)?;
        
        crate::serial_println!("[LINUX-VM {}] Starting Linux kernel via VMX...", self.id);
        crate::serial_println!("[LINUX-VM {}] Entry: 0x{:X}, Boot params: 0x{:X}", 
            self.id, entry_point, boot_proto::DY_);
        
        self.running.store(true, Ordering::SeqCst);
        
        
        
        
        
        match vm.start(entry_point, boot_proto::AZA_) {
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
    
    
    fn boot_with_svm(&mut self, entry_point: u64) -> Result<()> {
        use super::svm_vm;
        
        
        let vm_id = svm_vm::blh("linux-guest", self.config.memory_mb)?;
        
        crate::serial_println!("[LINUX-VM {}] SVM VM #{} created, loading {} MB...", 
            self.id, vm_id, self.guest_memory.len() / (1024 * 1024));
        
        
        let kdh = svm_vm::avv(vm_id, |vm| -> Result<()> {
            
            vm.initialize()?;
            
            
            vm.load_binary(&self.guest_memory, 0)?;
            
            
            vm.setup_protected_mode_for_linux(
                entry_point, 
                boot_proto::AZA_,
                boot_proto::DY_
            )?;
            
            crate::serial_println!("[LINUX-VM {}] Starting Linux kernel execution...", self.id);
            crate::serial_println!("[LINUX-VM {}] Entry: 0x{:X}, Boot params: 0x{:X}", 
                self.id, entry_point, boot_proto::DY_);
            
            
            vm.start()
        });
        
        match kdh {
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
    
    
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    
    pub fn eoa(&self) -> Vec<u8> {
        self.console_buffer.lock().clone()
    }
    
    
    pub fn id(&self) -> u64 {
        self.id
    }
}


static AGH_: Mutex<Option<LinuxVm>> = Mutex::new(None);


pub fn ehd(jx: &[u8], initramfs: &[u8], cmdline: &str) -> Result<u64> {
    let config = Gq {
        memory_mb: 128,
        cmdline: String::from(cmdline),
        ..Default::default()
    };
    
    let mut vm = LinuxVm::new(config)?;
    let id = vm.id();
    
    vm.boot(jx, initramfs)?;
    
    *AGH_.lock() = Some(vm);
    
    Ok(id)
}


pub fn is_running() -> bool {
    AGH_.lock().as_ref().map(|vm| vm.is_running()).unwrap_or(false)
}


pub fn qix() -> Option<u64> {
    AGH_.lock().as_ref().map(|vm| vm.id())
}
