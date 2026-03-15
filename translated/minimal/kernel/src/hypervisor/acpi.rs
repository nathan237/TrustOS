


















const YS_: u64 = 0x50000;


const BEU_: u64 = 0xE0000;


const AHI_: usize = 36;


const CDL_: u32 = 0xFEE0_0000;


const CCD_: u32 = 0xFEC0_0000;


const CCB_: u8 = 1;


const KT_: &[u8; 6] = b"TRUST\0";


const OM_: &[u8; 8] = b"TRUSTVM\0";
















pub fn esu(fe: &mut [u8]) -> u64 {
    let ar = YS_ as usize;
    
    
    let qex = 0x500; 
    for a in 0..qex {
        if ar + a < fe.len() {
            fe[ar + a] = 0;
        }
    }
    
    let dll = ar;           
    let gxb = ar + 0x040;   
    let lkc = ar + 0x080;   
    let kuz = ar + 0x100;   
    let krq = ar + 0x200;   
    let lcr = ar + 0x400;   
    
    
    let bmy = qte(fe, krq);
    
    
    let ujh = qtl(fe, lkc);
    
    
    let sqv = qtf(fe, kuz, krq as u64);
    
    
    let tqi = qti(fe, lcr);
    
    
    let xwr = qud(fe, gxb, &[
        lkc as u64,
        kuz as u64,
        lcr as u64,
    ]);
    
    
    qub(fe, dll, gxb as u64);
    
    
    
    let kdl = BEU_ as usize;
    if kdl + AHI_ <= fe.len() {
        
        let mut pek = [0u8; 36];
        pek.dg(&fe[dll..dll + AHI_]);
        fe[kdl..kdl + AHI_].dg(&pek);
    }
    
    crate::serial_println!("[ACPI] Tables installed at GPA 0x{:X}:", YS_);
    crate::serial_println!("[ACPI]   RSDP: 0x{:X} (also at 0x{:X})", dll, BEU_);
    crate::serial_println!("[ACPI]   XSDT: 0x{:X} ({} bytes, {} entries)", gxb, xwr, 3);
    crate::serial_println!("[ACPI]   MADT: 0x{:X} ({} bytes)", lkc, ujh);
    crate::serial_println!("[ACPI]   FADT: 0x{:X} ({} bytes)", kuz, sqv);
    crate::serial_println!("[ACPI]   HPET: 0x{:X} ({} bytes)", lcr, tqi);
    crate::serial_println!("[ACPI]   DSDT: 0x{:X} ({} bytes)", krq, bmy);
    
    
    YS_
}

















fn qub(mem: &mut [u8], l: usize, xws: u64) {
    
    mem[l..l + 8].dg(b"RSD PTR ");
    
    
    mem[l + 9..l + 15].dg(KT_);
    
    
    mem[l + 15] = 2;
    
    
    sx(mem, l + 16, 0);
    
    
    sx(mem, l + 20, 36);
    
    
    tw(mem, l + 24, xws);
    
    
    mem[l + 8] = 0; 
    let mut sum: u8 = 0;
    for a in 0..20 {
        sum = sum.cn(mem[l + a]);
    }
    mem[l + 8] = 0u8.nj(sum);
    
    
    mem[l + 32] = 0; 
    let mut mid: u8 = 0;
    for a in 0..36 {
        mid = mid.cn(mem[l + a]);
    }
    mem[l + 32] = 0u8.nj(mid);
}







fn qud(mem: &mut [u8], l: usize, icw: &[u64]) -> usize {
    let ame = icw.len();
    let aeb = 36 + ame * 8;
    
    
    mem[l..l + 4].dg(b"XSDT");
    
    
    sx(mem, l + 4, aeb as u32);
    
    
    mem[l + 8] = 1;
    
    
    mem[l + 9] = 0;
    
    
    mem[l + 10..l + 16].dg(KT_);
    
    
    mem[l + 16..l + 24].dg(OM_);
    
    
    sx(mem, l + 24, 1);
    
    
    mem[l + 28..l + 32].dg(b"TROS");
    
    
    sx(mem, l + 32, 1);
    
    
    for (a, &ag) in icw.iter().cf() {
        tw(mem, l + 36 + a * 8, ag);
    }
    
    
    hju(mem, l, aeb);
    
    aeb
}













fn qtl(mem: &mut [u8], l: usize) -> usize {
    
    
    
    let mut u = l;
    
    
    mem[u..u + 4].dg(b"APIC"); 
    
    u += 4;
    let udz = u;
    u += 4; 
    
    mem[l + 8] = 4; 
    mem[l + 9] = 0; 
    mem[l + 10..l + 16].dg(KT_);
    mem[l + 16..l + 24].dg(OM_);
    sx(mem, l + 24, 1); 
    mem[l + 28..l + 32].dg(b"TROS"); 
    sx(mem, l + 32, 1); 
    
    u = l + 36;
    
    
    
    
    sx(mem, u, CDL_);
    u += 4;
    
    
    sx(mem, u, 1); 
    u += 4;
    
    
    mem[u] = 0;     
    mem[u + 1] = 8; 
    mem[u + 2] = 0; 
    mem[u + 3] = 0; 
    sx(mem, u + 4, 1); 
    u += 8;
    
    
    mem[u] = 1;      
    mem[u + 1] = 12; 
    mem[u + 2] = CCB_; 
    mem[u + 3] = 0;  
    sx(mem, u + 4, CCD_); 
    sx(mem, u + 8, 0); 
    u += 12;
    
    
    
    mem[u] = 2;      
    mem[u + 1] = 10; 
    mem[u + 2] = 0;  
    mem[u + 3] = 0;  
    sx(mem, u + 4, 2); 
    aqr(mem, u + 8, 0); 
    u += 10;
    
    
    
    mem[u] = 2;      
    mem[u + 1] = 10; 
    mem[u + 2] = 0;  
    mem[u + 3] = 9;  
    sx(mem, u + 4, 9); 
    aqr(mem, u + 8, 0x000D); 
    u += 10;
    
    
    
    mem[u] = 4;     
    mem[u + 1] = 6; 
    mem[u + 2] = 0xFF; 
    aqr(mem, u + 3, 0x0005); 
    mem[u + 5] = 1; 
    u += 6;
    
    let aeb = u - l;
    
    
    sx(mem, udz, aeb as u32);
    
    
    hju(mem, l, aeb);
    
    aeb
}












fn qtf(mem: &mut [u8], l: usize, nob: u64) -> usize {
    
    let aeb: usize = 276;
    
    
    mem[l..l + 4].dg(b"FACP");
    sx(mem, l + 4, aeb as u32);
    mem[l + 8] = 5; 
    mem[l + 9] = 0; 
    mem[l + 10..l + 16].dg(KT_);
    mem[l + 16..l + 24].dg(OM_);
    sx(mem, l + 24, 1); 
    mem[l + 28..l + 32].dg(b"TROS"); 
    sx(mem, l + 32, 1); 
    
    
    
    
    sx(mem, l + 36, 0);
    
    
    sx(mem, l + 40, nob as u32);
    
    
    mem[l + 45] = 0;
    
    
    
    aqr(mem, l + 46, 9);
    
    
    sx(mem, l + 48, 0);
    
    
    mem[l + 52] = 0;
    
    mem[l + 53] = 0;
    
    
    sx(mem, l + 56, 0xB000);
    
    
    sx(mem, l + 60, 0);
    
    
    sx(mem, l + 64, 0xB004);
    
    
    sx(mem, l + 68, 0);
    
    
    sx(mem, l + 72, 0);
    
    
    sx(mem, l + 76, 0xB008);
    
    
    sx(mem, l + 80, 0);
    
    sx(mem, l + 84, 0);
    
    
    mem[l + 88] = 4;
    
    mem[l + 89] = 2;
    
    mem[l + 90] = 0;
    
    mem[l + 91] = 4;
    
    
    mem[l + 92] = 0;
    
    mem[l + 93] = 0;
    
    
    
    
    
    
    let squ: u32 = (1 << 0)  
                        | (1 << 4)  
                        | (1 << 8); 
    sx(mem, l + 112, squ);
    
    
    
    mem[l + 116] = 1; 
    mem[l + 117] = 8; 
    mem[l + 118] = 0; 
    mem[l + 119] = 1; 
    tw(mem, l + 120, 0xCF9); 
    
    
    mem[l + 128] = 0x06;
    
    
    aqr(mem, l + 129, 0);
    
    
    mem[l + 131] = 1;
    
    
    tw(mem, l + 132, 0);
    
    
    tw(mem, l + 140, nob);
    
    
    mrc(mem, l + 148, 1, 32, 0, 2, 0xB000); 
    
    
    
    mrc(mem, l + 172, 1, 16, 0, 2, 0xB004); 
    
    
    
    
    
    mrc(mem, l + 208, 1, 32, 0, 3, 0xB008); 
    
    
    hju(mem, l, aeb);
    
    aeb
}


















fn qte(mem: &mut [u8], l: usize) -> usize {
    
    let mut mv: [u8; 512] = [0u8; 512];
    let mut u: usize = 0;
    
    
    
    let wde = u;
    mv[u] = 0x10; 
    u += 1;
    let wdd = u;
    u += 2; 
    
    
    mv[u..u + 4].dg(b"_SB_");
    u += 4;
    
    
    let vft = u;
    mv[u] = 0x5B; u += 1; 
    mv[u] = 0x82; u += 1; 
    let vfs = u;
    u += 2; 
    mv[u..u + 4].dg(b"PCI0");
    u += 4;
    
    
    
    mv[u] = 0x08; u += 1; 
    mv[u..u + 4].dg(b"_HID");
    u += 4;
    
    mv[u] = 0x0C; u += 1; 
    let sjs = kth(b"PNP", 0x0A03);
    mv[u..u + 4].dg(&sjs.ho());
    u += 4;
    
    
    mv[u] = 0x08; u += 1; 
    mv[u..u + 4].dg(b"_ADR");
    u += 4;
    mv[u] = 0x00; u += 1; 
    
    
    mv[u] = 0x08; u += 1; 
    mv[u..u + 4].dg(b"_BBN");
    u += 4;
    mv[u] = 0x00; u += 1; 
    
    
    
    mv[u] = 0x08; u += 1; 
    mv[u..u + 4].dg(b"_PRT");
    u += 4;
    
    
    let vno = u;
    mv[u] = 0x12; u += 1; 
    let vnn = u;
    u += 2; 
    mv[u] = 0x04; u += 1; 
    
    
    
    
    u = iso(&mut mv, u, 1, 0, 16);
    
    u = iso(&mut mv, u, 2, 0, 17);
    
    u = iso(&mut mv, u, 3, 0, 18);
    
    u = iso(&mut mv, u, 4, 0, 19);
    
    
    let vnm = u - vno;
    ggd(&mut mv, vnn, vnm - 1); 
    
    
    let tzo = u;
    mv[u] = 0x5B; u += 1; 
    mv[u] = 0x82; u += 1; 
    let tzn = u;
    u += 2; 
    mv[u..u + 4].dg(b"ISA_");
    u += 4;
    
    
    mv[u] = 0x08; u += 1; 
    mv[u..u + 4].dg(b"_ADR");
    u += 4;
    mv[u] = 0x0C; u += 1; 
    mv[u..u + 4].dg(&0x0001_0000u32.ho());
    u += 4;
    
    
    let rmn = u;
    mv[u] = 0x5B; u += 1; 
    mv[u] = 0x82; u += 1; 
    let rmm = u;
    u += 2; 
    mv[u..u + 4].dg(b"COM1");
    u += 4;
    
    
    mv[u] = 0x08; u += 1; 
    mv[u..u + 4].dg(b"_HID");
    u += 4;
    mv[u] = 0x0C; u += 1; 
    let sjr = kth(b"PNP", 0x0501);
    mv[u..u + 4].dg(&sjr.ho());
    u += 4;
    
    
    mv[u] = 0x08; u += 1; 
    mv[u..u + 4].dg(b"_UID");
    u += 4;
    mv[u] = 0x01; u += 1; 
    
    
    let rml = u - rmn;
    ggd(&mut mv, rmm, rml - 2); 
    
    
    let tzm = u - tzo;
    ggd(&mut mv, tzn, tzm - 2);
    
    
    let vfr = u - vft;
    ggd(&mut mv, vfs, vfr - 2);
    
    
    let voq = u;
    mv[u] = 0x5B; u += 1; 
    mv[u] = 0x82; u += 1; 
    let vop = u;
    u += 2; 
    mv[u..u + 4].dg(b"PWRB");
    u += 4;
    
    
    mv[u] = 0x08; u += 1; 
    mv[u..u + 4].dg(b"_HID");
    u += 4;
    mv[u] = 0x0C; u += 1; 
    let sjt = kth(b"PNP", 0x0C0C);
    mv[u..u + 4].dg(&sjt.ho());
    u += 4;
    
    
    let voo = u - voq;
    ggd(&mut mv, vop, voo - 2);
    
    
    let wdc = u - wde;
    ggd(&mut mv, wdd, wdc - 1); 
    
    
    
    mv[u] = 0x08; u += 1; 
    
    mv[u] = 0x5C; u += 1; 
    mv[u..u + 4].dg(b"_S5_");
    u += 4;
    mv[u] = 0x12; u += 1; 
    mv[u] = 0x06; u += 1; 
    mv[u] = 0x04; u += 1; 
    mv[u] = 0x0A; u += 1; 
    mv[u] = 0x05; u += 1; 
    mv[u] = 0x0A; u += 1; 
    mv[u] = 0x05; u += 1; 
    mv[u] = 0x00; u += 1; 
    mv[u] = 0x00; u += 1; 
    
    let kaj = u;
    
    
    let aeb = 36 + kaj;
    
    
    mem[l..l + 4].dg(b"DSDT");
    sx(mem, l + 4, aeb as u32);
    mem[l + 8] = 2; 
    mem[l + 9] = 0; 
    mem[l + 10..l + 16].dg(KT_);
    mem[l + 16..l + 24].dg(OM_);
    sx(mem, l + 24, 1);     
    mem[l + 28..l + 32].dg(b"TROS"); 
    sx(mem, l + 32, 1);     
    
    
    mem[l + 36..l + 36 + kaj].dg(&mv[..kaj]);
    
    hju(mem, l, aeb);
    
    aeb
}



fn kth(llr: &[u8; 3], baj: u16) -> u32 {
    
    let acw = (llr[0] - b'@') & 0x1F;
    let rw = (llr[1] - b'@') & 0x1F;
    let tx = (llr[2] - b'@') & 0x1F;
    
    
    let ond = ((acw as u16) << 10) | ((rw as u16) << 5) | (tx as u16);
    
    
    
    let wu = (ond >> 8) as u8;
    let of = (ond & 0xFF) as u8;
    let tb = (baj >> 8) as u8;
    let ajw = (baj & 0xFF) as u8;
    
    u32::dj([wu, of, tb, ajw])
}



fn iso(mv: &mut [u8], mut u: usize, gk: u32, pin: u8, bup: u32) -> usize {
    let smi = u;
    mv[u] = 0x12; u += 1; 
    let smh = u;
    u += 1; 
    mv[u] = 0x04; u += 1; 
    
    
    let ag = (gk << 16) | 0xFFFF;
    mv[u] = 0x0C; u += 1; 
    mv[u..u + 4].dg(&ag.ho());
    u += 4;
    
    
    mv[u] = 0x0A; u += 1; 
    mv[u] = pin; u += 1;
    
    
    mv[u] = 0x00; u += 1; 
    
    
    mv[u] = 0x0C; u += 1; 
    mv[u..u + 4].dg(&bup.ho());
    u += 4;
    
    
    let bue = u - smi;
    mv[smh] = (bue - 1) as u8; 
    
    u
}



fn ggd(mv: &mut [u8], u: usize, go: usize) {
    if go < 63 {
        
        mv[u] = go as u8;
        mv[u + 1] = 0; 
        
        
    }
    
    
    let hh = (go & 0x0F) as u8;
    let gd = ((go >> 4) & 0xFF) as u8;
    mv[u] = 0x40 | hh; 
    mv[u + 1] = gd;
}






fn hju(mem: &mut [u8], l: usize, len: usize) {
    mem[l + 9] = 0;
    let mut sum: u8 = 0;
    for a in 0..len {
        sum = sum.cn(mem[l + a]);
    }
    mem[l + 9] = 0u8.nj(sum);
}


fn mrc(mem: &mut [u8], l: usize, ijc: u8, gbd: u8, mzf: u8, cct: u8, re: u64) {
    mem[l] = ijc;
    mem[l + 1] = gbd;
    mem[l + 2] = mzf;
    mem[l + 3] = cct;
    tw(mem, l + 4, re);
}

fn aqr(mem: &mut [u8], l: usize, bn: u16) {
    let bf = bn.ho();
    mem[l] = bf[0];
    mem[l + 1] = bf[1];
}

fn sx(mem: &mut [u8], l: usize, bn: u32) {
    let bf = bn.ho();
    mem[l..l + 4].dg(&bf);
}

fn tw(mem: &mut [u8], l: usize, bn: u64) {
    let bf = bn.ho();
    mem[l..l + 8].dg(&bf);
}























fn qti(mem: &mut [u8], l: usize) -> usize {
    let aeb: usize = 56;
    
    
    mem[l..l + 4].dg(b"HPET");
    
    
    sx(mem, l + 4, aeb as u32);
    
    
    mem[l + 8] = 1;
    
    
    mem[l + 10..l + 16].dg(KT_);
    
    
    mem[l + 16..l + 24].dg(OM_);
    
    
    sx(mem, l + 24, 1);
    
    
    mem[l + 28..l + 32].dg(b"TROS");
    
    
    sx(mem, l + 32, 1);
    
    
    
    
    
    
    
    
    let qqj: u32 = (0x8086 << 16)  
                       | (1 << 15)       
                       | (1 << 13)       
                       | (2 << 8)        
                       | 0x01;           
    sx(mem, l + 36, qqj);
    
    
    
    
    
    
    
    mem[l + 40] = 0;   
    mem[l + 41] = 64;  
    mem[l + 42] = 0;   
    mem[l + 43] = 0;   
    tw(mem, l + 44, 0xFED0_0000); 
    
    
    mem[l + 52] = 0;
    
    
    aqr(mem, l + 53, 128);
    
    
    mem[l + 55] = 0;
    
    
    hju(mem, l, aeb);
    
    aeb
}








#[cfg(test)]
mod tests {
    use super::*;

    fn gqm(mem: &[u8], l: usize) -> u32 {
        u32::dj([mem[l], mem[l+1], mem[l+2], mem[l+3]])
    }
    fn paj(mem: &[u8], l: usize) -> u64 {
        u64::dj([
            mem[l], mem[l+1], mem[l+2], mem[l+3],
            mem[l+4], mem[l+5], mem[l+6], mem[l+7],
        ])
    }
    fn gcm(mem: &[u8], l: usize, len: usize) -> bool {
        let mut sum: u8 = 0;
        for a in 0..len { sum = sum.cn(mem[l + a]); }
        sum == 0
    }

    #[test]
    fn zsg() {
        let mut mem = vec![0u8; 0xF0000];
        esu(&mut mem);
        assert_eq!(&mem[0x50000..0x50008], b"RSD PTR ");
        assert!(gcm(&mem, 0x50000, 20), "RSDP v1 checksum");
        assert!(gcm(&mem, 0x50000, 36), "RSDP v2 checksum");
        assert_eq!(mem[0x50000 + 15], 2, "RSDP revision");
    }

    #[test]
    fn zsi() {
        let mut mem = vec![0u8; 0xF0000];
        esu(&mut mem);
        let bya = 0x50040;
        assert_eq!(&mem[bya..bya+4], b"XSDT");
        let len = gqm(&mem, bya + 4) as usize;
        assert!(gcm(&mem, bya, len));
        assert_eq!(paj(&mem, bya + 36), 0x50080); 
        assert_eq!(paj(&mem, bya + 44), 0x50100); 
    }

    #[test]
    fn zrz() {
        let mut mem = vec![0u8; 0xF0000];
        esu(&mut mem);
        let madt = 0x50080;
        assert_eq!(&mem[madt..madt+4], b"APIC");
        let len = gqm(&mem, madt + 4) as usize;
        assert!(gcm(&mem, madt, len));
        assert_eq!(gqm(&mem, madt + 36), 0xFEE0_0000);
    }

    #[test]
    fn zrq() {
        let mut mem = vec![0u8; 0xF0000];
        esu(&mut mem);
        let fadt = 0x50100;
        assert_eq!(&mem[fadt..fadt+4], b"FACP");
        let len = gqm(&mem, fadt + 4) as usize;
        assert_eq!(len, 276);
        assert!(gcm(&mem, fadt, len));
        assert_eq!(gqm(&mem, fadt + 76), 0xB008); 
    }

    #[test]
    fn zrp() {
        let mut mem = vec![0u8; 0xF0000];
        esu(&mut mem);
        let dgt = 0x50200;
        assert_eq!(&mem[dgt..dgt+4], b"DSDT");
        let len = gqm(&mem, dgt + 4) as usize;
        assert_eq!(len, 36);
        assert!(gcm(&mem, dgt, len));
    }

    #[test]
    fn zrf() {
        let mut mem = vec![0u8; 0xF0000];
        esu(&mut mem);
        assert_eq!(&mem[0x50000..0x50024], &mem[0xE0000..0xE0024]);
    }
}
