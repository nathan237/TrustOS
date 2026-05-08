
















use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::{HypervisorError, Result};






const DY_: u64 = 0x7000;


const II_: u64 = 0x20000;


const BPG_: usize = 2048;


const PQ_: u64 = 0x70000;


const IV_: u64 = 0x60000;


const FY_: u64 = 0x100000;


const VL_: u64 = 0x1000000; 


const CBZ_: u64 = 0x80000; 







#[derive(Debug, Clone)]
pub struct Aet {
    
    pub setup_sects: u8,
    
    pub syssize: u32,
    
    pub header_magic: u32,
    
    pub version: u16,
    
    pub type_of_loader: u8,
    
    pub btz: u8,
    
    pub code32_start: u32,
    
    pub ramdisk_image: u32,
    
    pub ramdisk_size: u32,
    
    pub cmd_line_ptr: u32,
    
    pub initrd_addr_max: u32,
    
    pub kernel_alignment: u32,
    
    pub relocatable_kernel: u8,
    
    pub min_alignment: u8,
    
    pub xloadflags: u16,
    
    pub init_size: u32,
    
    pub pref_address: u64,
}


pub const WC_: u8 = 0x01;      
pub const ABG_: u8 = 0x80;     


pub const BLK_: u16 = 0x01;   
pub const EOF_: u16 = 0x02;
pub const EOG_: u16 = 0x04;
pub const EOH_: u16 = 0x08;


#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum E820Type {
    Ram = 1,
    Reserved = 2,
    Acpi = 3,
    Nvs = 4,
    Unusable = 5,
}


#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ru {
    pub addr: u64,
    pub size: u64,
    pub entry_type: u32,
}






pub struct Pj {
    
    pub header: Aet,
    
    pub kernel_data: Vec<u8>,
    
    pub setup_data: Vec<u8>,
    
    pub supports_64bit: bool,
    
    pub entry_64: u64,
}


pub fn itr(data: &[u8]) -> Result<Pj> {
    if data.len() < 0x300 {
        crate::serial_println!("[Linux] bzImage too small: {} bytes", data.len());
        return Err(HypervisorError::InvalidConfiguration);
    }

    
    let magic = read_u32(data, 0x202);
    if magic != 0x53726448 { 
        crate::serial_println!("[Linux] Invalid bzImage magic: 0x{:08X} (expected 0x53726448)", magic);
        return Err(HypervisorError::InvalidConfiguration);
    }

    let version = read_u16(data, 0x206);
    crate::serial_println!("[Linux] Boot protocol version: {}.{}", 
                          version >> 8, version & 0xFF);

    if version < 0x020F {
        crate::serial_println!("[Linux] Boot protocol version too old (need >= 2.15)");
        return Err(HypervisorError::InvalidConfiguration);
    }

    let setup_sects = data[0x1F1];
    let setup_sects = if setup_sects == 0 { 4 } else { setup_sects }; 

    let header = Aet {
        setup_sects,
        syssize: read_u32(data, 0x1F4),
        header_magic: magic,
        version,
        type_of_loader: data[0x210],
        btz: data[0x211],
        code32_start: read_u32(data, 0x214),
        ramdisk_image: read_u32(data, 0x218),
        ramdisk_size: read_u32(data, 0x21C),
        cmd_line_ptr: read_u32(data, 0x228),
        initrd_addr_max: read_u32(data, 0x22C),
        kernel_alignment: read_u32(data, 0x230),
        relocatable_kernel: data[0x234],
        min_alignment: data[0x235],
        xloadflags: read_u16(data, 0x236),
        init_size: read_u32(data, 0x260),
        pref_address: read_u64(data, 0x258),
    };

    let supports_64bit = (header.xloadflags & BLK_) != 0;

    
    let faj = (1 + setup_sects as usize) * 512;
    let setup_data = if faj <= data.len() {
        data[..faj].to_vec()
    } else {
        crate::serial_println!("[Linux] Warning: setup size {} > image size {}", faj, data.len());
        data.to_vec()
    };

    
    let iiz = faj;
    let kernel_data = if iiz < data.len() {
        data[iiz..].to_vec()
    } else {
        crate::serial_println!("[Linux] No protected-mode kernel data!");
        return Err(HypervisorError::InvalidConfiguration);
    };

    
    let entry_64 = if supports_64bit {
        FY_ + 0x200
    } else {
        FY_
    };

    crate::serial_println!("[Linux] Parsed bzImage:");
    crate::serial_println!("  Setup sectors: {}", setup_sects);
    crate::serial_println!("  Kernel size: {} bytes ({} KB)", 
                          kernel_data.len(), kernel_data.len() / 1024);
    crate::serial_println!("  64-bit: {}", supports_64bit);
    crate::serial_println!("  Entry point: 0x{:X}", entry_64);
    crate::serial_println!("  Preferred load: 0x{:X}", header.pref_address);
    crate::serial_println!("  Init size: {} KB", header.init_size / 1024);

    Ok(Pj {
        header,
        kernel_data,
        setup_data,
        supports_64bit,
        entry_64,
    })
}






pub struct Ml {
    
    pub cmdline: String,
    
    pub memory_size: u64,
    
    pub initrd: Option<Vec<u8>>,
}

impl Default for Ml {
    fn default() -> Self {
        Self {
            cmdline: String::from("console=ttyS0 earlyprintk=serial,ttyS0 nokaslr"),
            memory_size: 256 * 1024 * 1024, 
            initrd: None,
        }
    }
}


pub struct Pi {
    
    pub entry_point: u64,
    
    pub stack_ptr: u64,
    
    pub boot_params_addr: u64,
    
    pub cr3: u64,
    
    pub gdt_base: u64,
}










pub fn iku(
    guest_memory: &mut [u8],
    ny: &Pj,
    config: &Ml,
) -> Result<Pi> {
    let bcr = guest_memory.len() as u64;
    
    crate::serial_println!("[Linux] Loading kernel into {} MB guest memory",
                          bcr / (1024 * 1024));

    
    let int = FY_ + ny.kernel_data.len() as u64 + 0x100000;
    if bcr < int {
        crate::serial_println!("[Linux] Insufficient guest memory: need {} MB, have {} MB",
                              int / (1024 * 1024), bcr / (1024 * 1024));
        return Err(HypervisorError::OutOfMemory);
    }

    
    let bhk = FY_ as usize + ny.kernel_data.len();
    if bhk > guest_memory.len() {
        return Err(HypervisorError::OutOfMemory);
    }
    guest_memory[FY_ as usize..bhk]
        .copy_from_slice(&ny.kernel_data);
    crate::serial_println!("[Linux] Kernel loaded at 0x{:X}-0x{:X}", 
                          FY_, bhk);

    
    let bqz = config.cmdline.as_bytes();
    let chi = bqz.len().min(BPG_ - 1);
    let byc = II_ as usize;
    guest_memory[byc..byc + chi]
        .copy_from_slice(&bqz[..chi]);
    guest_memory[byc + chi] = 0; 
    crate::serial_println!("[Linux] Command line at 0x{:X}: \"{}\"", 
                          II_, config.cmdline);

    
    setup_boot_params(guest_memory, ny, config)?;

    
    oqj(guest_memory, bcr)?;

    
    oqi(guest_memory)?;

    
    let biz = super::acpi::cay(guest_memory);
    
    write_u64(guest_memory, DY_ as usize + 0x070, biz);
    
    
    if let Some(ref bck) = config.initrd {
        let czt = VL_ as usize + bck.len();
        if czt > guest_memory.len() {
            crate::serial_println!("[Linux] Initrd too large for guest memory");
            return Err(HypervisorError::OutOfMemory);
        }
        guest_memory[VL_ as usize..czt]
            .copy_from_slice(bck);
        crate::serial_println!("[Linux] Initrd loaded at 0x{:X}-0x{:X} ({} KB)",
                              VL_, czt, bck.len() / 1024);
    }

    Ok(Pi {
        entry_point: ny.entry_64,
        stack_ptr: CBZ_,
        boot_params_addr: DY_,
        cr3: PQ_,
        gdt_base: IV_,
    })
}









fn setup_boot_params(
    guest_memory: &mut [u8],
    ny: &Pj,
    config: &Ml,
) -> Result<()> {
    let bp = DY_ as usize;
    
    
    for i in 0..4096 {
        if bp + i < guest_memory.len() {
            guest_memory[bp + i] = 0;
        }
    }

    
    
    let iek = 0x1F1;
    let iej = 0x290.min(ny.setup_data.len());
    if iej > iek {
        let hrq = bp + 0x1F1;
        let src = &ny.setup_data[iek..iej];
        let mt = &mut guest_memory[hrq..hrq + src.len()];
        mt.copy_from_slice(src);
    }

    

    
    guest_memory[bp + 0x210] = 0xFF;

    
    guest_memory[bp + 0x211] = WC_ | ABG_;

    
    write_u16(guest_memory, bp + 0x224, 0xFE00);

    
    write_u32(guest_memory, bp + 0x228, II_ as u32);

    
    if config.initrd.is_some() {
        write_u32(guest_memory, bp + 0x218, VL_ as u32);
        write_u32(guest_memory, bp + 0x21C, 
                  config.initrd.as_ref().unwrap().len() as u32);
    }

    
    
    guest_memory[bp + 0x06] = 80;  
    guest_memory[bp + 0x07] = 25;  
    guest_memory[bp + 0x0F] = 0x22; 

    
    
    let bcr = config.memory_size;
    let ftr = setup_e820_map(guest_memory, bp, bcr);
    guest_memory[bp + 0x1E8] = ftr;

    crate::serial_println!("[Linux] boot_params at 0x{:X}, {} e820 entries, cmdline at 0x{:X}",
                          DY_, ftr, II_);

    Ok(())
}



fn setup_e820_map(guest_memory: &mut [u8], bp: usize, bcr: u64) -> u8 {
    
    
    let dog = bp + 0x2D0;
    let mut count = 0u8;

    
    write_e820_entry(guest_memory, dog, count, 
                     0, 0x9FC00, E820Type::Ram);
    count += 1;

    
    write_e820_entry(guest_memory, dog, count,
                     0x9FC00, 0xA0000 - 0x9FC00, E820Type::Reserved);
    count += 1;

    
    write_e820_entry(guest_memory, dog, count,
                     0x50000, 0x1000, E820Type::Acpi);
    count += 1;

    
    write_e820_entry(guest_memory, dog, count,
                     0xA0000, 0x60000, E820Type::Reserved);
    count += 1;

    
    let etq = bcr - 0x100000;
    write_e820_entry(guest_memory, dog, count,
                     0x100000, etq, E820Type::Ram);
    count += 1;

    count
}

fn write_e820_entry(
    mem: &mut [u8], base: usize, index: u8,
    addr: u64, size: u64, entry_type: E820Type,
) {
    let offset = base + (index as usize) * 20;
    write_u64(mem, offset, addr);
    write_u64(mem, offset + 8, size);
    write_u32(mem, offset + 16, entry_type as u32);
}















fn oqj(guest_memory: &mut [u8], _mem_size: u64) -> Result<()> {
    let cok = PQ_ as usize;

    
    for i in 0..(6 * 4096) {
        if cok + i < guest_memory.len() {
            guest_memory[cok + i] = 0;
        }
    }

    
    let ntg = PQ_ + 0x1000;
    write_u64(guest_memory, cok, ntg | 0x3); 

    
    for i in 0..4u64 {
        let nta = PQ_ + 0x2000 + i * 0x1000;
        write_u64(guest_memory, cok + 0x1000 + (i as usize) * 8,
                  nta | 0x3); 
    }

    
    for iw in 0..4u64 {
        let ntc = cok + 0x2000 + (iw as usize) * 0x1000;
        for entry in 0..512u64 {
            let phys_addr = (iw * 512 + entry) * 0x200000; 
            
            let dcm = phys_addr | 0x83; 
            write_u64(guest_memory, ntc + (entry as usize) * 8, dcm);
        }
    }

    crate::serial_println!("[Linux] Guest page tables at 0x{:X} (identity map 0-4GB, 2MB pages)",
                          PQ_);
    Ok(())
}















fn oqi(guest_memory: &mut [u8]) -> Result<()> {
    let gdt_base = IV_ as usize;

    
    for i in 0..512 {
        if gdt_base + i < guest_memory.len() {
            guest_memory[gdt_base + i] = 0;
        }
    }

    

    
    
    
    write_u64(guest_memory, gdt_base + 0x08, 
              enx(0, 0xFFFFF, 0x9A, 0xA)); 

    
    
    write_u64(guest_memory, gdt_base + 0x10,
              enx(0, 0xFFFFF, 0x92, 0xC)); 

    
    write_u64(guest_memory, gdt_base + 0x18,
              enx(0, 0xFFFFF, 0x9A, 0xC)); 

    
    write_u64(guest_memory, gdt_base + 0x20,
              enx(0, 0xFFFFF, 0x92, 0xC)); 

    
    let fyf = 5 * 8 - 1; 
    write_u16(guest_memory, gdt_base + 0x100, fyf as u16);
    write_u64(guest_memory, gdt_base + 0x102, IV_);

    crate::serial_println!("[Linux] Guest GDT at 0x{:X}: null, code64(0x08), data64(0x10), code32(0x18), data32(0x20)",
                          IV_);
    Ok(())
}







fn enx(base: u32, jm: u32, access: u8, flags: u8) -> u64 {
    let mut entry = 0u64;
    
    
    entry |= (jm & 0xFFFF) as u64;
    
    entry |= ((base & 0xFFFF) as u64) << 16;
    
    entry |= (((base >> 16) & 0xFF) as u64) << 32;
    
    entry |= (access as u64) << 40;
    
    entry |= (((jm >> 16) & 0xF) as u64) << 48;
    
    entry |= ((flags & 0xF) as u64) << 52;
    
    entry |= (((base >> 24) & 0xFF) as u64) << 56;
    
    entry
}












pub fn kwy(
    vmcs: &super::vmcs::Vmcs,
    pk: &Pi,
) -> Result<()> {
    use super::vmcs::fields;

    
    
    let cr0 = 0x8005_0033u64; 
    vmcs.write(fields::AWC_, cr0)?;
    
    
    vmcs.write(fields::AWD_, pk.cr3)?;
    
    
    let cr4 = 0x000006A0u64; 
    vmcs.write(fields::AWE_, cr4)?;

    
    
    vmcs.write(fields::AWI_, 0x08)?;
    vmcs.write(fields::AWG_, 0)?;
    vmcs.write(fields::AWH_, 0xFFFFFFFF)?;
    vmcs.write(fields::AWF_, 0xA09B)?; 

    
    vmcs.write(fields::AXN_, 0x10)?;
    vmcs.write(fields::AXL_, 0)?;
    vmcs.write(fields::AXM_, 0xFFFFFFFF)?;
    vmcs.write(fields::AXK_, 0xC093)?; 

    
    for (sel_field, base_field, limit_field, access_field) in [
        (fields::AWM_, fields::AWK_, fields::AWL_, fields::AWJ_),
        (fields::AWQ_, fields::AWO_, fields::AWP_, fields::AWN_),
        (fields::AWU_, fields::AWS_, fields::AWT_, fields::AWR_),
        (fields::AXA_, fields::AWY_, fields::AWZ_, fields::AWX_),
    ] {
        vmcs.write(sel_field, 0x10)?;
        vmcs.write(base_field, 0)?;
        vmcs.write(limit_field, 0xFFFFFFFF)?;
        vmcs.write(access_field, 0xC093)?;
    }

    
    vmcs.write(fields::AXR_, 0)?;
    vmcs.write(fields::AXP_, 0)?;
    vmcs.write(fields::AXQ_, 0xFFFF)?;
    vmcs.write(fields::AXO_, 0x8B)?; 

    
    vmcs.write(fields::AXH_, 0)?;
    vmcs.write(fields::AXF_, 0)?;
    vmcs.write(fields::AXG_, 0xFFFF)?;
    vmcs.write(fields::AXE_, 0x10082)?; 

    
    vmcs.write(fields::AWV_, pk.gdt_base)?;
    vmcs.write(fields::AWW_, 5 * 8 - 1)?;

    
    vmcs.write(fields::AXC_, 0)?;
    vmcs.write(fields::AXD_, 0xFFF)?;

    
    vmcs.write(fields::FV_, pk.entry_point)?;
    vmcs.write(fields::AXJ_, pk.stack_ptr)?;
    vmcs.write(fields::AXI_, 0x2)?; 

    
    
    vmcs.write(fields::AXB_, 0x501)?; 

    crate::serial_println!("[Linux] VMCS configured: RIP=0x{:X} RSP=0x{:X} CR3=0x{:X} RSI=0x{:X}",
                          pk.entry_point, pk.stack_ptr, pk.cr3, pk.boot_params_addr);

    Ok(())
}








pub fn nwu(
    guest_memory: &mut [u8],
    bas: &[u8],
    cmdline: &str,
    initrd: Option<&[u8]>,
) -> Result<Pi> {
    
    let ny = itr(bas)?;

    if !ny.supports_64bit {
        crate::serial_println!("[Linux] Warning: kernel does not advertise 64-bit support");
        
    }

    
    let config = Ml {
        cmdline: String::from(cmdline),
        memory_size: guest_memory.len() as u64,
        initrd: initrd.map(|d| d.to_vec()),
    };

    
    iku(guest_memory, &ny, &config)
}






pub fn fpb() -> Vec<u8> {
    
    
    

    let mut image = vec![0u8; 4096]; 

    
    image[0x1F1] = 1;

    
    image[0x202] = b'H';
    image[0x203] = b'd';
    image[0x204] = b'r';
    image[0x205] = b'S';

    
    image[0x206] = 0x0F;
    image[0x207] = 0x02;

    
    image[0x211] = WC_;

    
    write_u32(&mut image, 0x214, 0x100000);

    
    write_u16(&mut image, 0x236, BLK_);

    
    write_u64(&mut image, 0x258, 0x100000);

    
    write_u32(&mut image, 0x260, 0x100000);

    
    
    
    
    
    while image.len() < 1024 {
        image.push(0);
    }

    
    let esa = image.len();
    
    
    for _ in 0..0x200 {
        image.push(0x90); 
    }

    
    
    
    
    
    
    let message = b"[TrustVM-Linux] Boot OK - 64-bit entry reached!\r\n";
    for &byte in message {
        
        image.extend_from_slice(&[0x66, 0xBA, 0xF8, 0x03]);
        
        image.extend_from_slice(&[0xB0, byte]);
        
        image.push(0xEE);
    }

    
    let lcd = b"[TrustVM-Linux] 64-bit kernel entry OK\n";
    for &byte in lcd {
        image.extend_from_slice(&[0xB0, byte]);
        image.extend_from_slice(&[0xE6, 0xE9]);
    }

    
    
    image.extend_from_slice(&[0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00]); 
    image.extend_from_slice(&[0x0F, 0x01, 0xC1]); 

    
    image.extend_from_slice(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]); 
    image.extend_from_slice(&[0x0F, 0x01, 0xC1]); 

    
    image.push(0xF4);

    
    let kernel_size = image.len() - esa;
    write_u32(&mut image, 0x1F4, (kernel_size / 16) as u32);

    crate::serial_println!("[Linux] Created test kernel: {} bytes ({} setup + {} kernel)",
                          image.len(), esa, kernel_size);

    image
}





fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset], data[offset + 1],
        data[offset + 2], data[offset + 3],
    ])
}

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
    ])
}

fn write_u16(data: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_le_bytes();
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    let bytes = value.to_le_bytes();
    data[offset] = bytes[0];
    data[offset + 1] = bytes[1];
    data[offset + 2] = bytes[2];
    data[offset + 3] = bytes[3];
}

fn write_u64(data: &mut [u8], offset: usize, value: u64) {
    let bytes = value.to_le_bytes();
    for i in 0..8 {
        data[offset + i] = bytes[i];
    }
}
