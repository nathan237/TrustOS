


















const ZX_: u64 = 0x50000;


const BGW_: u64 = 0xE0000;


const AJE_: usize = 36;


const CGU_: u32 = 0xFEE0_0000;


const CFO_: u32 = 0xFEC0_0000;


const CFM_: u8 = 1;


const LM_: &[u8; 6] = b"TRUST\0";


const PK_: &[u8; 8] = b"TRUSTVM\0";
















pub fn cay(guest_memory: &mut [u8]) -> u64 {
    let base = ZX_ as usize;
    
    
    let jtm = 0x500; 
    for i in 0..jtm {
        if base + i < guest_memory.len() {
            guest_memory[base + i] = 0;
        }
    }
    
    let biz = base;           
    let dgv = base + 0x040;   
    let ggl = base + 0x080;   
    let fwd = base + 0x100;   
    let ftg = base + 0x200;   
    let gbe = base + 0x400;   
    
    
    let ahw = kew(guest_memory, ftg);
    
    
    let nbv = kfe(guest_memory, ggl);
    
    
    let luc = kex(guest_memory, fwd, ftg as u64);
    
    
    let mml = kfb(guest_memory, gbe);
    
    
    let pvw = kfw(guest_memory, dgv, &[
        ggl as u64,
        fwd as u64,
        gbe as u64,
    ]);
    
    
    kfu(guest_memory, biz, dgv as u64);
    
    
    
    let fjg = BGW_ as usize;
    if fjg + AJE_ <= guest_memory.len() {
        
        let mut jbr = [0u8; 36];
        jbr.copy_from_slice(&guest_memory[biz..biz + AJE_]);
        guest_memory[fjg..fjg + AJE_].copy_from_slice(&jbr);
    }
    
    crate::serial_println!("[ACPI] Tables installed at GPA 0x{:X}:", ZX_);
    crate::serial_println!("[ACPI]   RSDP: 0x{:X} (also at 0x{:X})", biz, BGW_);
    crate::serial_println!("[ACPI]   XSDT: 0x{:X} ({} bytes, {} entries)", dgv, pvw, 3);
    crate::serial_println!("[ACPI]   MADT: 0x{:X} ({} bytes)", ggl, nbv);
    crate::serial_println!("[ACPI]   FADT: 0x{:X} ({} bytes)", fwd, luc);
    crate::serial_println!("[ACPI]   HPET: 0x{:X} ({} bytes)", gbe, mml);
    crate::serial_println!("[ACPI]   DSDT: 0x{:X} ({} bytes)", ftg, ahw);
    
    
    ZX_
}

















fn kfu(mem: &mut [u8], offset: usize, xsdt_phys: u64) {
    
    mem[offset..offset + 8].copy_from_slice(b"RSD PTR ");
    
    
    mem[offset + 9..offset + 15].copy_from_slice(LM_);
    
    
    mem[offset + 15] = 2;
    
    
    write_u32(mem, offset + 16, 0);
    
    
    write_u32(mem, offset + 20, 36);
    
    
    write_u64(mem, offset + 24, xsdt_phys);
    
    
    mem[offset + 8] = 0; 
    let mut sum: u8 = 0;
    for i in 0..20 {
        sum = sum.wrapping_add(mem[offset + i]);
    }
    mem[offset + 8] = 0u8.wrapping_sub(sum);
    
    
    mem[offset + 32] = 0; 
    let mut gws: u8 = 0;
    for i in 0..36 {
        gws = gws.wrapping_add(mem[offset + i]);
    }
    mem[offset + 32] = 0u8.wrapping_sub(gws);
}







fn kfw(mem: &mut [u8], offset: usize, ebf: &[u64]) -> usize {
    let entry_count = ebf.len();
    let total_len = 36 + entry_count * 8;
    
    
    mem[offset..offset + 4].copy_from_slice(b"XSDT");
    
    
    write_u32(mem, offset + 4, total_len as u32);
    
    
    mem[offset + 8] = 1;
    
    
    mem[offset + 9] = 0;
    
    
    mem[offset + 10..offset + 16].copy_from_slice(LM_);
    
    
    mem[offset + 16..offset + 24].copy_from_slice(PK_);
    
    
    write_u32(mem, offset + 24, 1);
    
    
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS");
    
    
    write_u32(mem, offset + 32, 1);
    
    
    for (i, &addr) in ebf.iter().enumerate() {
        write_u64(mem, offset + 36 + i * 8, addr);
    }
    
    
    dpx(mem, offset, total_len);
    
    total_len
}













fn kfe(mem: &mut [u8], offset: usize) -> usize {
    
    
    
    let mut pos = offset;
    
    
    mem[pos..pos + 4].copy_from_slice(b"APIC"); 
    
    pos += 4;
    let mxx = pos;
    pos += 4; 
    
    mem[offset + 8] = 4; 
    mem[offset + 9] = 0; 
    mem[offset + 10..offset + 16].copy_from_slice(LM_);
    mem[offset + 16..offset + 24].copy_from_slice(PK_);
    write_u32(mem, offset + 24, 1); 
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS"); 
    write_u32(mem, offset + 32, 1); 
    
    pos = offset + 36;
    
    
    
    
    write_u32(mem, pos, CGU_);
    pos += 4;
    
    
    write_u32(mem, pos, 1); 
    pos += 4;
    
    
    mem[pos] = 0;     
    mem[pos + 1] = 8; 
    mem[pos + 2] = 0; 
    mem[pos + 3] = 0; 
    write_u32(mem, pos + 4, 1); 
    pos += 8;
    
    
    mem[pos] = 1;      
    mem[pos + 1] = 12; 
    mem[pos + 2] = CFM_; 
    mem[pos + 3] = 0;  
    write_u32(mem, pos + 4, CFO_); 
    write_u32(mem, pos + 8, 0); 
    pos += 12;
    
    
    
    mem[pos] = 2;      
    mem[pos + 1] = 10; 
    mem[pos + 2] = 0;  
    mem[pos + 3] = 0;  
    write_u32(mem, pos + 4, 2); 
    write_u16(mem, pos + 8, 0); 
    pos += 10;
    
    
    
    mem[pos] = 2;      
    mem[pos + 1] = 10; 
    mem[pos + 2] = 0;  
    mem[pos + 3] = 9;  
    write_u32(mem, pos + 4, 9); 
    write_u16(mem, pos + 8, 0x000D); 
    pos += 10;
    
    
    
    mem[pos] = 4;     
    mem[pos + 1] = 6; 
    mem[pos + 2] = 0xFF; 
    write_u16(mem, pos + 3, 0x0005); 
    mem[pos + 5] = 1; 
    pos += 6;
    
    let total_len = pos - offset;
    
    
    write_u32(mem, mxx, total_len as u32);
    
    
    dpx(mem, offset, total_len);
    
    total_len
}












fn kex(mem: &mut [u8], offset: usize, dsdt_phys: u64) -> usize {
    
    let total_len: usize = 276;
    
    
    mem[offset..offset + 4].copy_from_slice(b"FACP");
    write_u32(mem, offset + 4, total_len as u32);
    mem[offset + 8] = 5; 
    mem[offset + 9] = 0; 
    mem[offset + 10..offset + 16].copy_from_slice(LM_);
    mem[offset + 16..offset + 24].copy_from_slice(PK_);
    write_u32(mem, offset + 24, 1); 
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS"); 
    write_u32(mem, offset + 32, 1); 
    
    
    
    
    write_u32(mem, offset + 36, 0);
    
    
    write_u32(mem, offset + 40, dsdt_phys as u32);
    
    
    mem[offset + 45] = 0;
    
    
    
    write_u16(mem, offset + 46, 9);
    
    
    write_u32(mem, offset + 48, 0);
    
    
    mem[offset + 52] = 0;
    
    mem[offset + 53] = 0;
    
    
    write_u32(mem, offset + 56, 0xB000);
    
    
    write_u32(mem, offset + 60, 0);
    
    
    write_u32(mem, offset + 64, 0xB004);
    
    
    write_u32(mem, offset + 68, 0);
    
    
    write_u32(mem, offset + 72, 0);
    
    
    write_u32(mem, offset + 76, 0xB008);
    
    
    write_u32(mem, offset + 80, 0);
    
    write_u32(mem, offset + 84, 0);
    
    
    mem[offset + 88] = 4;
    
    mem[offset + 89] = 2;
    
    mem[offset + 90] = 0;
    
    mem[offset + 91] = 4;
    
    
    mem[offset + 92] = 0;
    
    mem[offset + 93] = 0;
    
    
    
    
    
    
    let lub: u32 = (1 << 0)  
                        | (1 << 4)  
                        | (1 << 8); 
    write_u32(mem, offset + 112, lub);
    
    
    
    mem[offset + 116] = 1; 
    mem[offset + 117] = 8; 
    mem[offset + 118] = 0; 
    mem[offset + 119] = 1; 
    write_u64(mem, offset + 120, 0xCF9); 
    
    
    mem[offset + 128] = 0x06;
    
    
    write_u16(mem, offset + 129, 0);
    
    
    mem[offset + 131] = 1;
    
    
    write_u64(mem, offset + 132, 0);
    
    
    write_u64(mem, offset + 140, dsdt_phys);
    
    
    hcp(mem, offset + 148, 1, 32, 0, 2, 0xB000); 
    
    
    
    hcp(mem, offset + 172, 1, 16, 0, 2, 0xB004); 
    
    
    
    
    
    hcp(mem, offset + 208, 1, 32, 0, 3, 0xB008); 
    
    
    dpx(mem, offset, total_len);
    
    total_len
}


















fn kew(mem: &mut [u8], offset: usize) -> usize {
    
    let mut fl: [u8; 512] = [0u8; 512];
    let mut pos: usize = 0;
    
    
    
    let okn = pos;
    fl[pos] = 0x10; 
    pos += 1;
    let okm = pos;
    pos += 2; 
    
    
    fl[pos..pos + 4].copy_from_slice(b"_SB_");
    pos += 4;
    
    
    let nss = pos;
    fl[pos] = 0x5B; pos += 1; 
    fl[pos] = 0x82; pos += 1; 
    let nsr = pos;
    pos += 2; 
    fl[pos..pos + 4].copy_from_slice(b"PCI0");
    pos += 4;
    
    
    
    fl[pos] = 0x08; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_HID");
    pos += 4;
    
    fl[pos] = 0x0C; pos += 1; 
    let loq = fup(b"PNP", 0x0A03);
    fl[pos..pos + 4].copy_from_slice(&loq.to_le_bytes());
    pos += 4;
    
    
    fl[pos] = 0x08; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_ADR");
    pos += 4;
    fl[pos] = 0x00; pos += 1; 
    
    
    fl[pos] = 0x08; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_BBN");
    pos += 4;
    fl[pos] = 0x00; pos += 1; 
    
    
    
    fl[pos] = 0x08; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_PRT");
    pos += 4;
    
    
    let nzd = pos;
    fl[pos] = 0x12; pos += 1; 
    let nzc = pos;
    pos += 2; 
    fl[pos] = 0x04; pos += 1; 
    
    
    
    
    pos = elg(&mut fl, pos, 1, 0, 16);
    
    pos = elg(&mut fl, pos, 2, 0, 17);
    
    pos = elg(&mut fl, pos, 3, 0, 18);
    
    pos = elg(&mut fl, pos, 4, 0, 19);
    
    
    let nzb = pos - nzd;
    cxe(&mut fl, nzc, nzb - 1); 
    
    
    let muh = pos;
    fl[pos] = 0x5B; pos += 1; 
    fl[pos] = 0x82; pos += 1; 
    let mug = pos;
    pos += 2; 
    fl[pos..pos + 4].copy_from_slice(b"ISA_");
    pos += 4;
    
    
    fl[pos] = 0x08; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_ADR");
    pos += 4;
    fl[pos] = 0x0C; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(&0x0001_0000u32.to_le_bytes());
    pos += 4;
    
    
    let kvx = pos;
    fl[pos] = 0x5B; pos += 1; 
    fl[pos] = 0x82; pos += 1; 
    let kvw = pos;
    pos += 2; 
    fl[pos..pos + 4].copy_from_slice(b"COM1");
    pos += 4;
    
    
    fl[pos] = 0x08; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_HID");
    pos += 4;
    fl[pos] = 0x0C; pos += 1; 
    let lop = fup(b"PNP", 0x0501);
    fl[pos..pos + 4].copy_from_slice(&lop.to_le_bytes());
    pos += 4;
    
    
    fl[pos] = 0x08; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_UID");
    pos += 4;
    fl[pos] = 0x01; pos += 1; 
    
    
    let kvv = pos - kvx;
    cxe(&mut fl, kvw, kvv - 2); 
    
    
    let muf = pos - muh;
    cxe(&mut fl, mug, muf - 2);
    
    
    let nsq = pos - nss;
    cxe(&mut fl, nsr, nsq - 2);
    
    
    let nzx = pos;
    fl[pos] = 0x5B; pos += 1; 
    fl[pos] = 0x82; pos += 1; 
    let nzw = pos;
    pos += 2; 
    fl[pos..pos + 4].copy_from_slice(b"PWRB");
    pos += 4;
    
    
    fl[pos] = 0x08; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_HID");
    pos += 4;
    fl[pos] = 0x0C; pos += 1; 
    let lor = fup(b"PNP", 0x0C0C);
    fl[pos..pos + 4].copy_from_slice(&lor.to_le_bytes());
    pos += 4;
    
    
    let nzv = pos - nzx;
    cxe(&mut fl, nzw, nzv - 2);
    
    
    let okl = pos - okn;
    cxe(&mut fl, okm, okl - 1); 
    
    
    
    fl[pos] = 0x08; pos += 1; 
    
    fl[pos] = 0x5C; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(b"_S5_");
    pos += 4;
    fl[pos] = 0x12; pos += 1; 
    fl[pos] = 0x06; pos += 1; 
    fl[pos] = 0x04; pos += 1; 
    fl[pos] = 0x0A; pos += 1; 
    fl[pos] = 0x05; pos += 1; 
    fl[pos] = 0x0A; pos += 1; 
    fl[pos] = 0x05; pos += 1; 
    fl[pos] = 0x00; pos += 1; 
    fl[pos] = 0x00; pos += 1; 
    
    let fgy = pos;
    
    
    let total_len = 36 + fgy;
    
    
    mem[offset..offset + 4].copy_from_slice(b"DSDT");
    write_u32(mem, offset + 4, total_len as u32);
    mem[offset + 8] = 2; 
    mem[offset + 9] = 0; 
    mem[offset + 10..offset + 16].copy_from_slice(LM_);
    mem[offset + 16..offset + 24].copy_from_slice(PK_);
    write_u32(mem, offset + 24, 1);     
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS"); 
    write_u32(mem, offset + 32, 1);     
    
    
    mem[offset + 36..offset + 36 + fgy].copy_from_slice(&fl[..fgy]);
    
    dpx(mem, offset, total_len);
    
    total_len
}



fn fup(mfr: &[u8; 3], product: u16) -> u32 {
    
    let og = (mfr[0] - b'@') & 0x1F;
    let hw = (mfr[1] - b'@') & 0x1F;
    let jf = (mfr[2] - b'@') & 0x1F;
    
    
    let inp = ((og as u16) << 10) | ((hw as u16) << 5) | (jf as u16);
    
    
    
    let kl = (inp >> 8) as u8;
    let gf = (inp & 0xFF) as u8;
    let iq = (product >> 8) as u8;
    let sc = (product & 0xFF) as u8;
    
    u32::from_le_bytes([kl, gf, iq, sc])
}



fn elg(fl: &mut [u8], mut pos: usize, slot: u32, pin: u8, gsi: u32) -> usize {
    let lqu = pos;
    fl[pos] = 0x12; pos += 1; 
    let lqt = pos;
    pos += 1; 
    fl[pos] = 0x04; pos += 1; 
    
    
    let addr = (slot << 16) | 0xFFFF;
    fl[pos] = 0x0C; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(&addr.to_le_bytes());
    pos += 4;
    
    
    fl[pos] = 0x0A; pos += 1; 
    fl[pos] = pin; pos += 1;
    
    
    fl[pos] = 0x00; pos += 1; 
    
    
    fl[pos] = 0x0C; pos += 1; 
    fl[pos..pos + 4].copy_from_slice(&gsi.to_le_bytes());
    pos += 4;
    
    
    let alm = pos - lqu;
    fl[lqt] = (alm - 1) as u8; 
    
    pos
}



fn cxe(fl: &mut [u8], pos: usize, length: usize) {
    if length < 63 {
        
        fl[pos] = length as u8;
        fl[pos + 1] = 0; 
        
        
    }
    
    
    let lo = (length & 0x0F) as u8;
    let hi = ((length >> 4) & 0xFF) as u8;
    fl[pos] = 0x40 | lo; 
    fl[pos + 1] = hi;
}






fn dpx(mem: &mut [u8], offset: usize, len: usize) {
    mem[offset + 9] = 0;
    let mut sum: u8 = 0;
    for i in 0..len {
        sum = sum.wrapping_add(mem[offset + i]);
    }
    mem[offset + 9] = 0u8.wrapping_sub(sum);
}


fn hcp(mem: &mut [u8], offset: usize, efb: u8, bit_width: u8, bit_offset: u8, access_size: u8, address: u64) {
    mem[offset] = efb;
    mem[offset + 1] = bit_width;
    mem[offset + 2] = bit_offset;
    mem[offset + 3] = access_size;
    write_u64(mem, offset + 4, address);
}

fn write_u16(mem: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_le_bytes();
    mem[offset] = bytes[0];
    mem[offset + 1] = bytes[1];
}

fn write_u32(mem: &mut [u8], offset: usize, value: u32) {
    let bytes = value.to_le_bytes();
    mem[offset..offset + 4].copy_from_slice(&bytes);
}

fn write_u64(mem: &mut [u8], offset: usize, value: u64) {
    let bytes = value.to_le_bytes();
    mem[offset..offset + 8].copy_from_slice(&bytes);
}























fn kfb(mem: &mut [u8], offset: usize) -> usize {
    let total_len: usize = 56;
    
    
    mem[offset..offset + 4].copy_from_slice(b"HPET");
    
    
    write_u32(mem, offset + 4, total_len as u32);
    
    
    mem[offset + 8] = 1;
    
    
    mem[offset + 10..offset + 16].copy_from_slice(LM_);
    
    
    mem[offset + 16..offset + 24].copy_from_slice(PK_);
    
    
    write_u32(mem, offset + 24, 1);
    
    
    mem[offset + 28..offset + 32].copy_from_slice(b"TROS");
    
    
    write_u32(mem, offset + 32, 1);
    
    
    
    
    
    
    
    
    let kcs: u32 = (0x8086 << 16)  
                       | (1 << 15)       
                       | (1 << 13)       
                       | (2 << 8)        
                       | 0x01;           
    write_u32(mem, offset + 36, kcs);
    
    
    
    
    
    
    
    mem[offset + 40] = 0;   
    mem[offset + 41] = 64;  
    mem[offset + 42] = 0;   
    mem[offset + 43] = 0;   
    write_u64(mem, offset + 44, 0xFED0_0000); 
    
    
    mem[offset + 52] = 0;
    
    
    write_u16(mem, offset + 53, 128);
    
    
    mem[offset + 55] = 0;
    
    
    dpx(mem, offset, total_len);
    
    total_len
}








#[cfg(test)]
mod tests {
    use super::*;

    fn ddf(mem: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([mem[offset], mem[offset+1], mem[offset+2], mem[offset+3]])
    }
    fn iym(mem: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes([
            mem[offset], mem[offset+1], mem[offset+2], mem[offset+3],
            mem[offset+4], mem[offset+5], mem[offset+6], mem[offset+7],
        ])
    }
    fn cva(mem: &[u8], offset: usize, len: usize) -> bool {
        let mut sum: u8 = 0;
        for i in 0..len { sum = sum.wrapping_add(mem[offset + i]); }
        sum == 0
    }

    #[test]
    fn raa() {
        let mut mem = vec![0u8; 0xF0000];
        cay(&mut mem);
        assert_eq!(&mem[0x50000..0x50008], b"RSD PTR ");
        assert!(cva(&mem, 0x50000, 20), "RSDP v1 checksum");
        assert!(cva(&mem, 0x50000, 36), "RSDP v2 checksum");
        assert_eq!(mem[0x50000 + 15], 2, "RSDP revision");
    }

    #[test]
    fn rac() {
        let mut mem = vec![0u8; 0xF0000];
        cay(&mut mem);
        let anj = 0x50040;
        assert_eq!(&mem[anj..anj+4], b"XSDT");
        let len = ddf(&mem, anj + 4) as usize;
        assert!(cva(&mem, anj, len));
        assert_eq!(iym(&mem, anj + 36), 0x50080); 
        assert_eq!(iym(&mem, anj + 44), 0x50100); 
    }

    #[test]
    fn qzt() {
        let mut mem = vec![0u8; 0xF0000];
        cay(&mut mem);
        let madt = 0x50080;
        assert_eq!(&mem[madt..madt+4], b"APIC");
        let len = ddf(&mem, madt + 4) as usize;
        assert!(cva(&mem, madt, len));
        assert_eq!(ddf(&mem, madt + 36), 0xFEE0_0000);
    }

    #[test]
    fn qzk() {
        let mut mem = vec![0u8; 0xF0000];
        cay(&mut mem);
        let fadt = 0x50100;
        assert_eq!(&mem[fadt..fadt+4], b"FACP");
        let len = ddf(&mem, fadt + 4) as usize;
        assert_eq!(len, 276);
        assert!(cva(&mem, fadt, len));
        assert_eq!(ddf(&mem, fadt + 76), 0xB008); 
    }

    #[test]
    fn qzj() {
        let mut mem = vec![0u8; 0xF0000];
        cay(&mut mem);
        let bge = 0x50200;
        assert_eq!(&mem[bge..bge+4], b"DSDT");
        let len = ddf(&mem, bge + 4) as usize;
        assert_eq!(len, 36);
        assert!(cva(&mem, bge, len));
    }

    #[test]
    fn qyz() {
        let mut mem = vec![0u8; 0xF0000];
        cay(&mut mem);
        assert_eq!(&mem[0x50000..0x50024], &mem[0xE0000..0xE0024]);
    }
}
