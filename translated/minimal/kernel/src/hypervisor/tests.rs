











use alloc::string::String;
use alloc::vec;
use alloc::format;


struct A {
    j: &'static str,
    cg: bool,
    eu: Option<String>,
}


pub fn jne() -> (usize, usize, alloc::vec::Vec<String>) {
    let mut hd: alloc::vec::Vec<A> = alloc::vec::Vec::new();
    let mut log: alloc::vec::Vec<String> = alloc::vec::Vec::new();
    
    
    hd.push(xch());
    hd.push(xci());
    hd.push(xcj());
    hd.push(xcg());
    hd.push(xck());
    hd.push(xcn());
    hd.push(xcl());
    hd.push(xcm());
    hd.push(xcf());
    hd.push(xca());
    hd.push(xce());
    hd.push(xcc());
    hd.push(xcb());
    hd.push(xcd());
    hd.push(xbz());
    hd.push(xbv());
    hd.push(xbx());
    hd.push(xby());
    hd.push(xbw());
    hd.push(xbu());
    hd.push(xbt());
    hd.push(xbs());
    
    
    hd.push(xeo());
    hd.push(xeq());
    hd.push(xer());
    hd.push(xes());
    hd.push(xep());
    
    
    hd.push(xeu());
    hd.push(xet());
    hd.push(xew());
    hd.push(xev());
    
    
    hd.push(xdn());
    hd.push(xdp());
    hd.push(xdq());
    hd.push(xdr());
    hd.push(xdo());
    
    
    hd.push(xcr());
    hd.push(xcq());
    hd.push(xcp());
    hd.push(xco());
    
    
    hd.push(xdv());
    hd.push(xdu());
    hd.push(xdy());
    hd.push(xdt());
    hd.push(xdw());
    hd.push(xds());
    hd.push(xdx());
    hd.push(xdz());
    
    
    hd.push(psj());
    hd.push(psl());
    hd.push(xdj());
    hd.push(xdm());
    hd.push(psk());
    hd.push(xdk());
    hd.push(xdl());
    
    
    hd.push(psg());
    hd.push(psi());
    hd.push(psh());
    hd.push(xdf());
    hd.push(xdh());
    hd.push(xdg());
    hd.push(xdi());
    hd.push(xde());
    hd.push(xdd());
    hd.push(xdb());
    
    
    hd.push(psm());
    hd.push(xef());
    hd.push(xeg());
    hd.push(xee());
    hd.push(xed());
    hd.push(xec());
    hd.push(xeb());
    
    
    hd.push(xey());
    hd.push(xez());
    hd.push(xfa());
    
    
    hd.push(xcz());
    hd.push(xcx());
    hd.push(xct());
    hd.push(xcv());
    hd.push(xcu());
    hd.push(xcw());
    hd.push(xcs());
    hd.push(xcy());
    
    
    hd.push(xen());
    hd.push(xel());
    hd.push(xem());
    
    
    hd.push(xek());
    hd.push(xeh());
    hd.push(xei());
    hd.push(xej());
    
    
    hd.push(xfd());
    hd.push(xfe());
    hd.push(xfg());
    hd.push(xff());
    hd.push(xfc());
    
    
    hd.push(xfh());
    hd.push(xfi());
    hd.push(xfk());
    hd.push(xfj());
    
    
    hd.push(xdc());
    
    
    let mut cg = 0usize;
    let mut gv = 0usize;
    
    for m in &hd {
        if m.cg {
            cg += 1;
            log.push(format!("  [PASS] {}", m.j));
        } else {
            gv += 1;
            if let Some(ref eu) = m.eu {
                log.push(format!("  [FAIL] {} — {}", m.j, eu));
            } else {
                log.push(format!("  [FAIL] {}", m.j));
            }
        }
    }
    
    (cg, gv, log)
}





fn alp(mem: &[u8], l: usize) -> u16 {
    u16::dj([mem[l], mem[l + 1]])
}

fn za(mem: &[u8], l: usize) -> u32 {
    u32::dj([
        mem[l], mem[l + 1], mem[l + 2], mem[l + 3],
    ])
}

fn aqi(mem: &[u8], l: usize) -> u64 {
    u64::dj([
        mem[l], mem[l + 1], mem[l + 2], mem[l + 3],
        mem[l + 4], mem[l + 5], mem[l + 6], mem[l + 7],
    ])
}


fn fyf(mem: &[u8], l: usize, len: usize) -> bool {
    let mut sum: u8 = 0;
    for a in 0..len {
        sum = sum.cn(mem[l + a]);
    }
    sum == 0
}


fn alr() -> alloc::vec::Vec<u8> {
    
    let mut mem = vec![0u8; 0xF0000]; 
    super::acpi::esu(&mut mem);
    mem
}





fn xch() -> A {
    let mem = alr();
    let chl = 0x50000;
    let sig = &mem[chl..chl + 8];
    let cg = sig == b"RSD PTR ";
    A {
        j: "ACPI RSDP signature",
        cg,
        eu: if !cg {
            Some(format!("expected 'RSD PTR ', got {:?}", core::str::jg(sig)))
        } else { None },
    }
}

fn xci() -> A {
    let mem = alr();
    let chl = 0x50000;
    
    let cg = fyf(&mem, chl, 20);
    A {
        j: "ACPI RSDP v1 checksum (bytes 0-19)",
        cg,
        eu: if !cg {
            let mut sum: u8 = 0;
            for a in 0..20 { sum = sum.cn(mem[chl + a]); }
            Some(format!("sum=0x{:02X}, expected 0x00", sum))
        } else { None },
    }
}

fn xcj() -> A {
    let mem = alr();
    let chl = 0x50000;
    
    let cg = fyf(&mem, chl, 36);
    A {
        j: "ACPI RSDP v2 extended checksum (bytes 0-35)",
        cg,
        eu: if !cg {
            let mut sum: u8 = 0;
            for a in 0..36 { sum = sum.cn(mem[chl + a]); }
            Some(format!("sum=0x{:02X}, expected 0x00", sum))
        } else { None },
    }
}

fn xcg() -> A {
    let mem = alr();
    let afe = mem[0x50000 + 15];
    let cg = afe == 2;
    A {
        j: "ACPI RSDP revision is 2 (ACPI 2.0)",
        cg,
        eu: if !cg {
            Some(format!("revision={}, expected 2", afe))
        } else { None },
    }
}

fn xck() -> A {
    let mem = alr();
    let chl = 0x50000;
    let gxb = aqi(&mem, chl + 24);
    let cg = gxb == 0x50040;
    A {
        j: "ACPI RSDP XSDT pointer = 0x50040",
        cg,
        eu: if !cg {
            Some(format!("xsdt_addr=0x{:X}, expected 0x50040", gxb))
        } else { None },
    }
}





fn xcn() -> A {
    let mem = alr();
    let bya = 0x50040;
    let sig = &mem[bya..bya + 4];
    let cg = sig == b"XSDT";
    A {
        j: "ACPI XSDT signature",
        cg,
        eu: if !cg {
            Some(format!("got {:?}", core::str::jg(sig)))
        } else { None },
    }
}

fn xcl() -> A {
    let mem = alr();
    let bya = 0x50040;
    let len = za(&mem, bya + 4) as usize;
    let cg = len > 0 && fyf(&mem, bya, len);
    A {
        j: "ACPI XSDT checksum",
        cg,
        eu: if !cg {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn xcm() -> A {
    let mem = alr();
    let bya = 0x50040;
    let aeb = za(&mem, bya + 4) as usize;
    let hif = aeb.ao(36);
    let ame = hif / 8;
    
    let nqk = aqi(&mem, bya + 36);
    let nql = aqi(&mem, bya + 44);
    let nqm = aqi(&mem, bya + 52);
    let cg = ame == 3 && nqk == 0x50080 && nql == 0x50100 && nqm == 0x50400;
    A {
        j: "ACPI XSDT has 3 entries (MADT, FADT, HPET)",
        cg,
        eu: if !cg {
            Some(format!("count={}, e0=0x{:X}, e1=0x{:X}, e2=0x{:X}", ame, nqk, nql, nqm))
        } else { None },
    }
}





fn xcf() -> A {
    let mem = alr();
    let madt = 0x50080;
    let sig = &mem[madt..madt + 4];
    let cg = sig == b"APIC";
    A {
        j: "ACPI MADT signature 'APIC'",
        cg,
        eu: if !cg {
            Some(format!("got {:?}", core::str::jg(sig)))
        } else { None },
    }
}

fn xca() -> A {
    let mem = alr();
    let madt = 0x50080;
    let len = za(&mem, madt + 4) as usize;
    let cg = len > 0 && fyf(&mem, madt, len);
    A {
        j: "ACPI MADT checksum",
        cg,
        eu: if !cg {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn xce() -> A {
    let mem = alr();
    let madt = 0x50080;
    
    let jch = za(&mem, madt + 36);
    let cg = jch == 0xFEE0_0000;
    A {
        j: "ACPI MADT LAPIC address = 0xFEE00000",
        cg,
        eu: if !cg {
            Some(format!("got 0x{:08X}", jch))
        } else { None },
    }
}

fn xcc() -> A {
    let mem = alr();
    let madt = 0x50080;
    let aeb = za(&mem, madt + 4) as usize;
    
    
    let mut u = 44;
    let mut kwu = false;
    while u + 2 <= aeb {
        let avt = mem[madt + u];
        let bue = mem[madt + u + 1] as usize;
        if bue == 0 { break; }
        
        if avt == 0 && bue == 8 {
            
            let aed = mem[madt + u + 3];
            let flags = za(&mem, madt + u + 4);
            if aed == 0 && (flags & 1) != 0 {
                kwu = true;
            }
        }
        u += bue;
    }
    
    A {
        j: "ACPI MADT contains Processor Local APIC (ID=0, enabled)",
        cg: kwu,
        eu: if !kwu { Some(String::from("LAPIC entry not found")) } else { None },
    }
}

fn xcb() -> A {
    let mem = alr();
    let madt = 0x50080;
    let aeb = za(&mem, madt + 4) as usize;
    
    let mut u = 44;
    let mut kwr = false;
    let mut lfo = 0u32;
    while u + 2 <= aeb {
        let avt = mem[madt + u];
        let bue = mem[madt + u + 1] as usize;
        if bue == 0 { break; }
        
        if avt == 1 && bue == 12 {
            
            lfo = za(&mem, madt + u + 4);
            if lfo == 0xFEC0_0000 {
                kwr = true;
            }
        }
        u += bue;
    }
    
    A {
        j: "ACPI MADT contains I/O APIC at 0xFEC00000",
        cg: kwr,
        eu: if !kwr {
            Some(format!("I/O APIC entry not found (addr=0x{:08X})", lfo))
        } else { None },
    }
}

fn xcd() -> A {
    let mem = alr();
    let madt = 0x50080;
    let aeb = za(&mem, madt + 4) as usize;
    
    let mut u = 44;
    let mut kws = false;
    let mut kwt = false;
    while u + 2 <= aeb {
        let avt = mem[madt + u];
        let bue = mem[madt + u + 1] as usize;
        if bue == 0 { break; }
        
        if avt == 2 && bue == 10 {
            
            let iy = mem[madt + u + 3];
            let bup = za(&mem, madt + u + 4);
            if iy == 0 && bup == 2 { kws = true; }
            if iy == 9 && bup == 9 { kwt = true; }
        }
        u += bue;
    }
    
    let cg = kws && kwt;
    A {
        j: "ACPI MADT IRQ overrides (IRQ0→GSI2, IRQ9→GSI9)",
        cg,
        eu: if !cg {
            Some(format!("irq0→gsi2={}, irq9→gsi9={}", kws, kwt))
        } else { None },
    }
}





fn xbz() -> A {
    let mem = alr();
    let fadt = 0x50100;
    let sig = &mem[fadt..fadt + 4];
    let cg = sig == b"FACP";
    A {
        j: "ACPI FADT signature 'FACP'",
        cg,
        eu: if !cg {
            Some(format!("got {:?}", core::str::jg(sig)))
        } else { None },
    }
}

fn xbv() -> A {
    let mem = alr();
    let fadt = 0x50100;
    let len = za(&mem, fadt + 4) as usize;
    let cg = len == 276 && fyf(&mem, fadt, len);
    A {
        j: "ACPI FADT checksum (276 bytes)",
        cg,
        eu: if !cg {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn xbx() -> A {
    let mem = alr();
    let fadt = 0x50100;
    
    let jjm = za(&mem, fadt + 76);
    
    let lum = mem[fadt + 91];
    
    let flags = za(&mem, fadt + 112);
    let ptq = (flags >> 4) & 1;
    
    let cg = jjm == 0xB008 && lum == 4 && ptq == 1;
    A {
        j: "ACPI FADT PM timer at 0xB008 (32-bit)",
        cg,
        eu: if !cg {
            Some(format!("port=0x{:X} len={} ext={}", jjm, lum, ptq))
        } else { None },
    }
}

fn xby() -> A {
    let mem = alr();
    let fadt = 0x50100;
    
    let grm = alp(&mem, fadt + 46);
    let cg = grm == 9;
    A {
        j: "ACPI FADT SCI interrupt = IRQ 9",
        cg,
        eu: if !cg {
            Some(format!("sci_int={}", grm))
        } else { None },
    }
}

fn xbw() -> A {
    let mem = alr();
    let fadt = 0x50100;
    
    let nnz = za(&mem, fadt + 40) as u64;
    
    let noa = aqi(&mem, fadt + 140);
    let cg = nnz == 0x50200 && noa == 0x50200;
    A {
        j: "ACPI FADT DSDT pointer = 0x50200 (32+64 bit)",
        cg,
        eu: if !cg {
            Some(format!("dsdt32=0x{:X}, dsdt64=0x{:X}", nnz, noa))
        } else { None },
    }
}





fn xbu() -> A {
    let mem = alr();
    let dgt = 0x50200;
    let sig = &mem[dgt..dgt + 4];
    let cg = sig == b"DSDT";
    A {
        j: "ACPI DSDT signature",
        cg,
        eu: if !cg {
            Some(format!("got {:?}", core::str::jg(sig)))
        } else { None },
    }
}

fn xbt() -> A {
    let mem = alr();
    let dgt = 0x50200;
    let len = za(&mem, dgt + 4) as usize;
    let cg = len > 36 && fyf(&mem, dgt, len);
    A {
        j: "ACPI DSDT checksum (with AML)",
        cg,
        eu: if !cg {
            Some(format!("len={}", len))
        } else { None },
    }
}

fn xbs() -> A {
    let mem = alr();
    
    let wav = &mem[0x50000..0x50000 + 36];
    let wau = &mem[0xE0000..0xE0000 + 36];
    let cg = wav == wau;
    A {
        j: "ACPI RSDP copy at BIOS area 0xE0000",
        cg,
        eu: if !cg {
            Some(String::from("RSDP at 0x50000 != RSDP at 0xE0000"))
        } else { None },
    }
}





fn xeo() -> A {
    let pic = super::svm_vm::PicState::default();
    let cg = pic.eun == 0xFF
        && pic.gsp == 0xFF
        && pic.cgc == 0x08
        && pic.eyo == 0x70
        && pic.ayd == 0
        && pic.bxd == 0
        && pic.dji == 0
        && pic.jfb == 0
        && !pic.jr;
    A {
        j: "PIC defaults (IMR=0xFF, bases=0x08/0x70, phase=0)",
        cg,
        eu: if !cg {
            Some(format!("m_imr=0x{:02X} s_imr=0x{:02X} m_base=0x{:02X} s_base=0x{:02X} phase={}/{}",
                pic.eun, pic.gsp, pic.cgc, pic.eyo,
                pic.ayd, pic.bxd))
        } else { None },
    }
}

fn xeq() -> A {
    
    let mut pic = super::svm_vm::PicState::default();
    
    
    let ldb: u8 = 0x11; 
    if ldb & 0x10 != 0 {
        pic.ayd = 1;
        pic.dji = 0;
        pic.jfb = 0;
    }
    let vhe = pic.ayd;
    
    
    let ldc: u8 = 0x20;
    if pic.ayd == 1 {
        pic.cgc = ldc & 0xF8;
        pic.ayd = 2;
    }
    let vhf = pic.ayd;
    
    
    if pic.ayd == 2 {
        pic.ayd = 3;
    }
    
    
    if pic.ayd == 3 {
        pic.ayd = 0;
        pic.jr = true;
    }
    
    let cg = vhe == 1
        && vhf == 2
        && pic.ayd == 0
        && pic.cgc == 0x20
        && pic.jr;
    
    A {
        j: "PIC master ICW1-4 sequence (vector base 0x20)",
        cg,
        eu: if !cg {
            Some(format!("phase={} base=0x{:02X} init={}", 
                pic.ayd, pic.cgc, pic.jr))
        } else { None },
    }
}

fn xer() -> A {
    let mut pic = super::svm_vm::PicState::default();
    
    
    let ldb: u8 = 0x11;
    if ldb & 0x10 != 0 {
        pic.bxd = 1;
    }
    
    
    let ldc: u8 = 0x28;
    if pic.bxd == 1 {
        pic.eyo = ldc & 0xF8;
        pic.bxd = 2;
    }
    
    
    if pic.bxd == 2 { pic.bxd = 3; }
    
    
    if pic.bxd == 3 { pic.bxd = 0; }
    
    let cg = pic.bxd == 0 && pic.eyo == 0x28;
    A {
        j: "PIC slave ICW1-4 sequence (vector base 0x28)",
        cg,
        eu: if !cg {
            Some(format!("phase={} base=0x{:02X}", pic.bxd, pic.eyo))
        } else { None },
    }
}

fn xes() -> A {
    let mut pic = super::svm_vm::PicState::default();
    
    
    pic.ayd = 0; 
    pic.eun = 0xFB; 
    
    let cg = pic.eun == 0xFB;
    A {
        j: "PIC OCW1 sets IMR (0xFB = mask all except IRQ2)",
        cg,
        eu: None,
    }
}

fn xep() -> A {
    let mut pic = super::svm_vm::PicState::default();
    pic.dji = 0x04; 
    
    
    let uxb: u8 = 0x20;
    if uxb == 0x20 {
        pic.dji = 0;
    }
    
    let cg = pic.dji == 0;
    A {
        j: "PIC non-specific EOI clears ISR",
        cg,
        eu: if !cg {
            Some(format!("isr=0x{:02X} after EOI", pic.dji))
        } else { None },
    }
}





fn xeu() -> A {
    let abu = super::svm_vm::PitState::default();
    let ww = &abu.lq[0];
    let cg = ww.ahs == 0xFFFF
        && ww.az == 0xFFFF
        && ww.vz == 3
        && ww.ev == 0
        && !ww.czf
        && !ww.ccp
        && abu.lq.len() == 3;
    A {
        j: "PIT defaults (reload=0xFFFF, access=3, 3 channels)",
        cg,
        eu: if !cg {
            Some(format!("reload={} access={} mode={}", ww.ahs, ww.vz, ww.ev))
        } else { None },
    }
}

fn xet() -> A {
    let mut abu = super::svm_vm::PitState::default();
    
    
    let control: u8 = 0b00_11_010_0; 
    let channel = ((control >> 6) & 0x3) as usize;
    let vz = (control >> 4) & 0x3;
    let ev = (control >> 1) & 0x7;
    
    if channel < 3 && vz != 0 {
        abu.lq[channel].vz = vz;
        abu.lq[channel].ev = ev;
        abu.lq[channel].ccp = false;
    }
    
    let cg = abu.lq[0].vz == 3
        && abu.lq[0].ev == 2;
    A {
        j: "PIT control word (ch0, lo/hi, mode 2 rate generator)",
        cg,
        eu: if !cg {
            Some(format!("access={} mode={}", abu.lq[0].vz, abu.lq[0].ev))
        } else { None },
    }
}

fn xew() -> A {
    let mut abu = super::svm_vm::PitState::default();
    abu.lq[0].vz = 3; 
    
    
    let hh: u8 = 0x9C;
    let bm = &mut abu.lq[0];
    if !bm.ccp {
        bm.ahs = (bm.ahs & 0xFF00) | hh as u16;
        bm.ccp = true;
    }
    let qfx = bm.ahs;
    let aln = bm.ccp;
    
    
    let gd: u8 = 0x2E;
    if bm.ccp {
        bm.ahs = (bm.ahs & 0x00FF) | ((gd as u16) << 8);
        bm.az = bm.ahs;
        bm.ccp = false;
    }
    
    let cg = (qfx & 0xFF) == 0x9C
        && aln
        && abu.lq[0].ahs == 0x2E9C
        && abu.lq[0].az == 0x2E9C
        && !abu.lq[0].ccp;
    A {
        j: "PIT lo/hi reload sequence (0x2E9C = ~100 Hz)",
        cg,
        eu: if !cg {
            Some(format!("reload=0x{:04X} count=0x{:04X} pending={}", 
                abu.lq[0].ahs, abu.lq[0].az, abu.lq[0].ccp))
        } else { None },
    }
}

fn xev() -> A {
    let mut abu = super::svm_vm::PitState::default();
    abu.lq[0].az = 0x1234;
    
    
    let control: u8 = 0b00_00_000_0; 
    let channel = ((control >> 6) & 0x3) as usize;
    let vz = (control >> 4) & 0x3;
    
    if channel < 3 && vz == 0 {
        abu.lq[channel].czf = true;
        abu.lq[channel].gkx = abu.lq[channel].az;
    }
    
    let cg = abu.lq[0].czf
        && abu.lq[0].gkx == 0x1234;
    A {
        j: "PIT latch command captures count value",
        cg,
        eu: if !cg {
            Some(format!("latched={} value=0x{:04X}", abu.lq[0].czf, abu.lq[0].gkx))
        } else { None },
    }
}





fn xdn() -> A {
    let ku = super::svm_vm::LapicState::default();
    let cg = ku.bnh == 0
        && ku.fel == 0
        && ku.dgc == 0
        && (ku.atq & 0x0001_0000) != 0  
        && ku.bim == 0x1FF
        && ku.guv == 0
        && !ku.iq
        && ku.fmr == 0;
    A {
        j: "LAPIC defaults (masked, SVR=0x1FF, disabled)",
        cg,
        eu: if !cg {
            Some(format!("icr={} lvt=0x{:X} svr=0x{:X} enabled={}", 
                ku.bnh, ku.atq, ku.bim, ku.iq))
        } else { None },
    }
}

fn xdp() -> A {
    let mut ku = super::svm_vm::LapicState::default();
    
    
    ku.bim = 0x1FF; 
    ku.iq = (ku.bim & 0x100) != 0;
    
    let iq = ku.iq;
    
    
    ku.bim = 0x0FF;
    ku.iq = (ku.bim & 0x100) != 0;
    let dqa = !ku.iq;
    
    let cg = iq && dqa;
    A {
        j: "LAPIC enable/disable via SVR bit 8",
        cg,
        eu: if !cg {
            Some(format!("enabled_check={} disabled_check={}", iq, dqa))
        } else { None },
    }
}

fn xdq() -> A {
    let mut ku = super::svm_vm::LapicState::default();
    ku.iq = true;
    
    
    ku.atq = 0x30; 
    ku.bnh = 100_000;
    ku.fel = 100_000;
    ku.fmr = 0;
    
    let wj = ku.atq & 0xFF;
    let bnm = (ku.atq >> 16) & 1;
    let ev = (ku.atq >> 17) & 0x3;
    
    let cg = wj == 0x30
        && bnm == 0
        && ev == 0
        && ku.bnh == 100_000
        && ku.fel == 100_000;
    A {
        j: "LAPIC timer arm (one-shot, vector=0x30, ICR=100000)",
        cg,
        eu: if !cg {
            Some(format!("vec=0x{:X} mask={} mode={} icr={}", wj, bnm, ev, ku.bnh))
        } else { None },
    }
}

fn xdr() -> A {
    let mut ku = super::svm_vm::LapicState::default();
    
    
    ku.atq = 0x30; 
    let ont = (ku.atq >> 17) & 0x3;
    
    
    ku.atq = 0x0002_0030; 
    let onu = (ku.atq >> 17) & 0x3;
    
    
    ku.atq = 0x0004_0030; 
    let onv = (ku.atq >> 17) & 0x3;
    
    
    ku.atq = 0x0001_0030; 
    let bnm = (ku.atq >> 16) & 1;
    
    let cg = ont == 0 && onu == 1 && onv == 2 && bnm == 1;
    A {
        j: "LAPIC timer LVT modes (one-shot/periodic/TSC-deadline/mask)",
        cg,
        eu: if !cg {
            Some(format!("modes: 0={} 1={} 2={} mask={}", ont, onu, onv, bnm))
        } else { None },
    }
}

fn xdo() -> A {
    
    let rzf: [(u32, u64); 8] = [
        (0x0, 2),    
        (0x1, 4),    
        (0x2, 8),    
        (0x3, 16),   
        (0x8, 32),   
        (0x9, 64),   
        (0xA, 128),  
        (0xB, 1),    
    ];
    
    let mut dyf = true;
    let mut gzm = String::new();
    
    for &(njs, qy) in &rzf {
        let gfa = match njs & 0xB {
            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
            _ => 1,
        };
        if gfa != qy {
            dyf = false;
            gzm = format!("dcr=0x{:X}: got {} expected {}", njs, gfa, qy);
            break;
        }
    }
    
    A {
        j: "LAPIC timer divider decode (all 8 values)",
        cg: dyf,
        eu: if !dyf { Some(gzm) } else { None },
    }
}





fn xcr() -> A {
    
    
    let qy: [(u8, u8); 7] = [
        (0x00, 0x00),  
        (0x02, 0x30),  
        (0x04, 0x12),  
        (0x06, 0x02),  
        (0x07, 0x17),  
        (0x08, 0x02),  
        (0x09, 0x26),  
    ];
    
    let mut dyf = true;
    let mut gzm = String::new();
    
    for &(reg, nrz) in &qy {
        let elw = ffe(reg);
        if elw != nrz {
            dyf = false;
            gzm = format!("reg 0x{:02X}: got 0x{:02X} expected 0x{:02X}", reg, elw, nrz);
            break;
        }
    }
    
    A {
        j: "CMOS time registers (seconds/min/hour/date)",
        cg: dyf,
        eu: if !dyf { Some(gzm) } else { None },
    }
}

fn xcq() -> A {
    let pbd = ffe(0x0A);
    let pbe = ffe(0x0B);
    let pbf = ffe(0x0C);
    let pbh = ffe(0x0D);
    
    let cg = pbd == 0x26  
        && pbe == 0x02        
        && pbf == 0x00        
        && pbh == 0x80;       
    A {
        j: "CMOS status registers A-D",
        cg,
        eu: if !cg {
            Some(format!("A=0x{:02X} B=0x{:02X} C=0x{:02X} D=0x{:02X}", pbd, pbe, pbf, pbh))
        } else { None },
    }
}

fn xcp() -> A {
    let nqx = ffe(0x14);
    
    let cg = nqx == 0x06;
    A {
        j: "CMOS equipment byte (FPU + color display)",
        cg,
        eu: if !cg {
            Some(format!("got 0x{:02X}, expected 0x06", nqx))
        } else { None },
    }
}

fn xco() -> A {
    let hcn = ffe(0x32);
    let cg = hcn == 0x20; 
    A {
        j: "CMOS century register = 0x20 (2000s)",
        cg,
        eu: if !cg {
            Some(format!("got 0x{:02X}", hcn))
        } else { None },
    }
}


fn ffe(index: u8) -> u8 {
    match index {
        0x00 => 0x00,  
        0x02 => 0x30,  
        0x04 => 0x12,  
        0x06 => 0x02,  
        0x07 => 0x17,  
        0x08 => 0x02,  
        0x09 => 0x26,  
        0x0A => 0x26,  
        0x0B => 0x02,  
        0x0C => 0x00,  
        0x0D => 0x80,  
        0x0E => 0x00,  
        0x0F => 0x00,  
        0x10 => 0x00,  
        0x12 => 0x00,  
        0x14 => 0x06,  
        0x15 => 0x80,  
        0x16 => 0x02,  
        0x17 => 0x00,  
        0x18 => 0x00,  
        0x32 => 0x20,  
        _ => 0x00,
    }
}






fn xdv() -> A {
    let bf = [0x89, 0x07]; 
    let bc = super::mmio::cpw(&bf, 2, true);
    let bq = match bc {
        Some(ref adr) => adr.rm && adr.aqc == 4 && adr.nw == Some(0) && adr.ake == 2,
        None => false,
    };
    A {
        j: "mmio_decode_mov_write",
        cg: bq,
        eu: if !bq { Some(format!("decoded={:?}", bc)) } else { None },
    }
}


fn xdu() -> A {
    let bf = [0x8B, 0x0F]; 
    let bc = super::mmio::cpw(&bf, 2, true);
    let bq = match bc {
        Some(ref adr) => !adr.rm && adr.aqc == 4 && adr.nw == Some(1) && adr.ake == 2,
        None => false,
    };
    A {
        j: "mmio_decode_mov_read",
        cg: bq,
        eu: if !bq { Some(format!("decoded={:?}", bc)) } else { None },
    }
}


fn xdy() -> A {
    let bf = [0x48, 0x89, 0x07]; 
    let bc = super::mmio::cpw(&bf, 3, true);
    let bq = match bc {
        Some(ref adr) => adr.rm && adr.aqc == 8 && adr.nw == Some(0) && adr.ake == 3,
        None => false,
    };
    A {
        j: "mmio_decode_rex_w",
        cg: bq,
        eu: if !bq { Some(format!("decoded={:?}", bc)) } else { None },
    }
}


fn xdt() -> A {
    
    let bf = [0xC7, 0x07, 0x78, 0x56, 0x34, 0x12];
    let bc = super::mmio::cpw(&bf, 6, true);
    let bq = match bc {
        Some(ref adr) => adr.rm && adr.aqc == 4 
            && adr.nw.is_none() 
            && adr.cag == Some(0x12345678),
        None => false,
    };
    A {
        j: "mmio_decode_imm32",
        cg: bq,
        eu: if !bq { Some(format!("decoded={:?}", bc)) } else { None },
    }
}


fn xdw() -> A {
    let bf = [0x0F, 0xB6, 0x07]; 
    let bc = super::mmio::cpw(&bf, 3, true);
    let bq = match bc {
        Some(ref adr) => !adr.rm && adr.aqc == 1 && adr.nw == Some(0),
        None => false,
    };
    A {
        j: "mmio_decode_movzx",
        cg: bq,
        eu: if !bq { Some(format!("decoded={:?}", bc)) } else { None },
    }
}


fn xds() -> A {
    let bf = [0x8B, 0x87, 0x20, 0x03, 0x00, 0x00]; 
    let bc = super::mmio::cpw(&bf, 6, true);
    let bq = match bc {
        Some(ref adr) => !adr.rm && adr.aqc == 4 && adr.nw == Some(0) && adr.ake == 6,
        None => false,
    };
    A {
        j: "mmio_decode_disp32",
        cg: bq,
        eu: if !bq { Some(format!("decoded={:?}", bc)) } else { None },
    }
}


fn xdx() -> A {
    
    
    
    
    let bf = [0x45, 0x89, 0x07]; 
    let bc = super::mmio::cpw(&bf, 3, true);
    let bq = match bc {
        Some(ref adr) => adr.rm && adr.aqc == 4 && adr.nw == Some(8), 
        None => false,
    };
    A {
        j: "mmio_decode_r8_r15",
        cg: bq,
        eu: if !bq { Some(format!("decoded={:?}", bc)) } else { None },
    }
}


fn xdz() -> A {
    let mut regs = super::svm_vm::SvmGuestRegs::default();
    
    super::mmio::pzy(&mut regs, 10, 0xDEAD_BEEF_CAFE_BABE);
    let ap = super::mmio::paf(&regs, 10);
    let bq = ap == 0xDEAD_BEEF_CAFE_BABE && regs.r10 == 0xDEAD_BEEF_CAFE_BABE;
    A {
        j: "mmio_register_rw",
        cg: bq,
        eu: if !bq { Some(format!("got 0x{:X}", ap)) } else { None },
    }
}






fn psj() -> A {
    let ioapic = super::ioapic::IoApicState::default();
    let bq = ioapic.ad == 1 && ioapic.esz == 0;
    
    let muu = ioapic.ctu.iter().xx(|aa| (aa >> 16) & 1 == 1);
    let bq = bq && muu;
    A {
        j: "ioapic_defaults",
        cg: bq,
        eu: if !bq { Some(format!("id={} ioregsel={} all_masked={}", ioapic.ad, ioapic.esz, muu)) } else { None },
    }
}


fn psl() -> A {
    let ioapic = super::ioapic::IoApicState::default();
    let axh = ioapic.read(0x10); 
    
    let mut ioapic = ioapic;
    ioapic.write(0x00, 0x01); 
    let axh = ioapic.read(0x10); 
    let dk = axh & 0xFF;
    let omd = (axh >> 16) & 0xFF;
    let bq = dk == 0x20 && omd == 23;
    A {
        j: "ioapic_version_register",
        cg: bq,
        eu: if !bq { Some(format!("ver=0x{:X} max_redir={}", dk, omd)) } else { None },
    }
}


fn xdj() -> A {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    ioapic.write(0x00, 0x00); 
    let hnq = ioapic.read(0x10);
    let nke = (hnq >> 24) & 0xF;
    
    
    ioapic.write(0x00, 0x00);
    ioapic.write(0x10, 0x05_00_00_00); 
    let hnq = ioapic.read(0x10);
    let opv = (hnq >> 24) & 0xF;
    
    let bq = nke == 1 && opv == 5;
    A {
        j: "ioapic_id_readwrite",
        cg: bq,
        eu: if !bq { Some(format!("default={} new={}", nke, opv)) } else { None },
    }
}


fn xdm() -> A {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    
    ioapic.write(0x00, 0x10); 
    ioapic.write(0x10, 0x0000_0040); 
    
    
    ioapic.write(0x00, 0x11); 
    ioapic.write(0x10, 0x0000_0000);
    
    
    ioapic.write(0x00, 0x10);
    let hh = ioapic.read(0x10);
    
    let bq = (hh & 0xFF) == 0x40 && ((hh >> 16) & 1) == 0; 
    A {
        j: "ioapic_redir_table",
        cg: bq,
        eu: if !bq { Some(format!("lo=0x{:X}", hh)) } else { None },
    }
}


fn psk() -> A {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    
    
    ioapic.write(0x00, 0x00); 
    let ad = ioapic.read(0x10);
    ioapic.write(0x00, 0x01); 
    let axh = ioapic.read(0x10);
    
    let bq = ad != axh; 
    A {
        j: "ioapic_indirect_access",
        cg: bq,
        eu: if !bq { Some(format!("id=0x{:X} ver=0x{:X}", ad, axh)) } else { None },
    }
}


fn xdk() -> A {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    
    ioapic.write(0x00, 0x14); 
    ioapic.write(0x10, 0x0000_0030); 
    ioapic.write(0x00, 0x15); 
    ioapic.write(0x10, 0x0000_0000); 
    
    let bia = ioapic.hli(2);
    let bq = match bia {
        Some(ref m) => m.wj == 0x30 && !m.bnm && m.iqu == 0 && !m.oiy,
        None => false,
    };
    A {
        j: "ioapic_irq_routing",
        cg: bq,
        eu: if !bq { Some(format!("route={:?}", bia)) } else { None },
    }
}


fn xdl() -> A {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    
    ioapic.write(0x00, 0x10); 
    ioapic.write(0x10, 0x0000_5030); 
    
    ioapic.write(0x00, 0x10);
    let hh = ioapic.read(0x10);
    
    
    let mzd = (hh >> 12) & 1;
    let mze = (hh >> 14) & 1;
    let bq = mzd == 0 && mze == 0 && (hh & 0xFF) == 0x30;
    A {
        j: "ioapic_readonly_bits",
        cg: bq,
        eu: if !bq { Some(format!("lo=0x{:X} bit12={} bit14={}", hh, mzd, mze)) } else { None },
    }
}





fn psg() -> A {
    let hpet = super::hpet::HpetState::default();
    let bq = !hpet.iq && hpet.config == 0 && hpet.cru == 0 
             && hpet.dpe == 0 && hpet.axe.len() == 3;
    A {
        j: "hpet_defaults",
        cg: bq,
        eu: if !bq { Some(format!("enabled={} config=0x{:X}", hpet.iq, hpet.config)) } else { None },
    }
}

fn psi() -> A {
    let hpet = super::hpet::HpetState::default();
    let cew = hpet.read(0x000, 8);
    
    let vv = cew & 0xFF;
    let ory = (cew >> 8) & 0x1F;
    let eoc = (cew >> 13) & 1;
    let awn = (cew >> 32) as u32;
    let bq = vv == 1 && ory == 2 && eoc == 1 && awn == 69_841_279;
    A {
        j: "hpet_gcap_id_register",
        cg: bq,
        eu: if !bq { 
            Some(format!("rev={} timers-1={} 64bit={} period={}", vv, ory, eoc, awn))
        } else { None },
    }
}

fn psh() -> A {
    let mut hpet = super::hpet::HpetState::default();
    
    let acw = hpet.read(0x0F0, 8);
    
    
    hpet.write(0x010, 1, 8); 
    let iq = hpet.iq;
    let config = hpet.read(0x010, 8);
    
    
    hpet.write(0x010, 0, 8);
    let dqa = !hpet.iq;
    
    let rw = hpet.read(0x0F0, 8);
    let tx = hpet.read(0x0F0, 8);
    let nwl = rw == tx; 
    
    let bq = acw == 0 && iq && config == 1 && dqa && nwl;
    A {
        j: "hpet_enable_disable",
        cg: bq,
        eu: if !bq { 
            Some(format!("c0={} en={} cfg={} dis={} frozen={}", acw, iq, config, dqa, nwl))
        } else { None },
    }
}

fn xdf() -> A {
    let mut hpet = super::hpet::HpetState::default();
    hpet.write(0x010, 1, 8); 
    
    
    let rw = hpet.read(0x0F0, 8);
    
    for _ in 0..10000 {
        core::hint::hc();
    }
    let tx = hpet.read(0x0F0, 8);
    
    
    let bq = tx >= rw;
    A {
        j: "hpet_counter_increments",
        cg: bq,
        eu: if !bq { 
            Some(format!("c1={} c2={}", rw, tx))
        } else { None },
    }
}

fn xdh() -> A {
    let mut hpet = super::hpet::HpetState::default();
    
    let wzr = hpet.read(0x100, 8); 
    let ovd = (wzr >> 4) & 1; 
    
    
    
    hpet.write(0x100, (2 << 9) | (1 << 2), 8);
    let mji = hpet.read(0x100, 8);
    let oev = (mji >> 2) & 1;
    let bia = (mji >> 9) & 0x1F;
    
    let bq = ovd == 1 && oev == 1 && bia == 2;
    A {
        j: "hpet_timer_config",
        cg: bq,
        eu: if !bq {
            Some(format!("periodic_cap={} int_en={} route={} raw=0x{:X}", ovd, oev, bia, mji))
        } else { None },
    }
}

fn xdg() -> A {
    let mut hpet = super::hpet::HpetState::default();
    
    hpet.write(0x108, 0xDEAD_BEEF, 8);
    let dfd = hpet.read(0x108, 8);
    
    
    hpet.write(0x128, 0x1234_5678_9ABC_DEF0, 8);
    let nfd = hpet.read(0x128, 8);
    
    let bq = dfd == 0xDEAD_BEEF && nfd == 0x1234_5678_9ABC_DEF0;
    A {
        j: "hpet_timer_comparator",
        cg: bq,
        eu: if !bq { Some(format!("t0=0x{:X} t1=0x{:X}", dfd, nfd)) } else { None },
    }
}

fn xdi() -> A {
    let mut hpet = super::hpet::HpetState::default();
    
    hpet.write(0x0F0, 0x42, 8);
    let r = hpet.read(0x0F0, 8);
    
    
    hpet.write(0x010, 1, 8);
    hpet.write(0x0F0, 0xFF, 8); 
    
    let nbd = hpet.read(0x0F0, 8);
    
    let bq = r == 0x42 && nbd >= 0x42;
    A {
        j: "hpet_write_counter_disabled",
        cg: bq,
        eu: if !bq { Some(format!("c_dis=0x{:X} c_en=0x{:X}", r, nbd)) } else { None },
    }
}

fn xde() -> A {
    let mem = alr();
    let cyo = 0x50400;
    let sig = &mem[cyo..cyo + 4];
    let bq = sig == b"HPET";
    A {
        j: "hpet_acpi_table_signature",
        cg: bq,
        eu: if !bq { Some(format!("sig={:?}", core::str::jg(sig))) } else { None },
    }
}

fn xdd() -> A {
    let mem = alr();
    let cyo = 0x50400;
    let go = za(&mem, cyo + 4) as usize;
    let sum: u8 = mem[cyo..cyo + go].iter().cqs(0u8, |q, &o| q.cn(o));
    let bq = sum == 0 && go == 56;
    A {
        j: "hpet_acpi_table_checksum",
        cg: bq,
        eu: if !bq { Some(format!("sum={} len={}", sum, go)) } else { None },
    }
}

fn xdb() -> A {
    let mem = alr();
    let cyo = 0x50400;
    
    let ag = aqi(&mem, cyo + 44);
    
    let ijc = mem[cyo + 40];
    let bq = ag == 0xFED0_0000 && ijc == 0;
    A {
        j: "hpet_acpi_table_address",
        cg: bq,
        eu: if !bq { Some(format!("addr=0x{:X} space={}", ag, ijc)) } else { None },
    }
}





fn psm() -> A {
    let aq = super::pci::PciBus::default();
    let occ = aq.dpx(0, 0, 0);
    let ogz = aq.dpx(0, 1, 0);
    let nfo = aq.dpx(0, 2, 0);
    let mzp = aq.dpx(0, 3, 0);
    let oqs = !aq.dpx(0, 4, 0);
    let oqr = !aq.dpx(1, 0, 0);
    let bq = occ && ogz && nfo && mzp && oqs && oqr;
    A {
        j: "pci_bus_defaults",
        cg: bq,
        eu: if !bq { Some(format!("host={} isa={} con={} blk={} no4={} nob1={}", occ, ogz, nfo, mzp, oqs, oqr)) } else { None },
    }
}

fn xef() -> A {
    let mut aq = super::pci::PciBus::default();
    
    aq.dni(0x8000_0000);
    let ap = aq.duw(0);
    let acs = (ap & 0xFFFF) as u16;
    let de = ((ap >> 16) & 0xFFFF) as u16;
    let bq = acs == 0x8086 && de == 0x1237;
    A {
        j: "pci_host_bridge_ids",
        cg: bq,
        eu: if !bq { Some(format!("vendor=0x{:04X} device=0x{:04X}", acs, de)) } else { None },
    }
}

fn xeg() -> A {
    let mut aq = super::pci::PciBus::default();
    
    aq.dni(0x8000_0808);
    let ap = aq.duw(0);
    let ajz = ((ap >> 24) & 0xFF) as u8;
    let adl = ((ap >> 16) & 0xFF) as u8;
    let bq = ajz == 0x06 && adl == 0x01;
    A {
        j: "pci_isa_bridge_class",
        cg: bq,
        eu: if !bq { Some(format!("class=0x{:02X} sub=0x{:02X}", ajz, adl)) } else { None },
    }
}

fn xee() -> A {
    let mut aq = super::pci::PciBus::default();
    
    aq.dni(0x8000_F800);
    let ap = aq.duw(0);
    let bq = ap == 0xFFFF_FFFF;
    A {
        j: "pci_config_no_device",
        cg: bq,
        eu: if !bq { Some(format!("val=0x{:08X}", ap)) } else { None },
    }
}

fn xed() -> A {
    let mut aq = super::pci::PciBus::default();
    
    aq.dni(0x0000_0000);
    let ap = aq.duw(0);
    let bq = ap == 0xFFFF_FFFF;
    A {
        j: "pci_config_disabled",
        cg: bq,
        eu: if !bq { Some(format!("val=0x{:08X}", ap)) } else { None },
    }
}

fn xec() -> A {
    let mut aq = super::pci::PciBus::default();
    aq.dni(0x8000_1234);
    let bky = aq.ozx();
    let bq = bky == 0x8000_1234;
    A {
        j: "pci_config_addr_readback",
        cg: bq,
        eu: if !bq { Some(format!("read=0x{:08X}", bky)) } else { None },
    }
}

fn xeb() -> A {
    let mut aq = super::pci::PciBus::default();
    
    aq.dni(0x8000_0010);
    
    aq.mra(0, 0xFFFF_FFFF);
    let ap = aq.duw(0);
    
    let bq = ap == 0xFFFF_FFFF;
    A {
        j: "pci_bar_probing",
        cg: bq,
        eu: if !bq { Some(format!("val=0x{:08X}", ap)) } else { None },
    }
}





fn xey() -> A {
    
    use super::svm_vm::SvmVmState;
    use alloc::collections::VecDeque;
    
    
    let izk: u8 = 0x0F; 
    let bq = izk & 0x01 != 0; 
    A {
        j: "serial_ier_readback",
        cg: bq,
        eu: if !bq { Some(format!("ier=0x{:02X}", izk)) } else { None },
    }
}

fn xez() -> A {
    
    let mut bi: alloc::collections::VecDeque<u8> = alloc::collections::VecDeque::fc(256);
    bi.agt(b'H');
    bi.agt(b'i');
    
    let nce = bi.awp();
    let ncf = bi.awp();
    let ncg = bi.awp();
    
    let bq = nce == Some(b'H') && ncf == Some(b'i') && ncg.is_none() && bi.is_empty();
    A {
        j: "serial_input_buffer",
        cg: bq,
        eu: if !bq { Some(format!("ch1={:?} ch2={:?} ch3={:?}", nce, ncf, ncg)) } else { None },
    }
}

fn xfa() -> A {
    
    let mut bi: alloc::collections::VecDeque<u8> = alloc::collections::VecDeque::new();
    
    let okn = 0x60u8 | if !bi.is_empty() { 0x01 } else { 0x00 };
    bi.agt(b'X');
    let okm = 0x60u8 | if !bi.is_empty() { 0x01 } else { 0x00 };
    
    let bq = okn == 0x60 && okm == 0x61;
    A {
        j: "serial_lsr_data_ready",
        cg: bq,
        eu: if !bq { Some(format!("empty=0x{:02X} data=0x{:02X}", okn, okm)) } else { None },
    }
}





fn xcz() -> A {
    let mem = alr();
    let ast = 0x50200;
    
    
    let bmy = za(&mem, ast + 4) as usize;
    
    
    let bq = bmy > 36;
    A {
        j: "dsdt_has_aml_content",
        cg: bq,
        eu: if !bq { Some(format!("dsdt_len={}", bmy)) } else { None },
    }
}

fn xcx() -> A {
    let mem = alr();
    let ast = 0x50200;
    let bmy = za(&mem, ast + 4) as usize;
    
    
    let mv = &mem[ast + 36..ast + bmy];
    let oay = mv.ee(4).any(|d| d == b"_SB_");
    
    A {
        j: "dsdt_aml_scope_sb",
        cg: oay,
        eu: if !oay { Some(format!("_SB_ not found in {} AML bytes", mv.len())) } else { None },
    }
}

fn xct() -> A {
    let mem = alr();
    let ast = 0x50200;
    let bmy = za(&mem, ast + 4) as usize;
    
    let mv = &mem[ast + 36..ast + bmy];
    let oau = mv.ee(4).any(|d| d == b"PCI0");
    
    A {
        j: "dsdt_aml_device_pci0",
        cg: oau,
        eu: if !oau { Some(format!("PCI0 not found in AML")) } else { None },
    }
}

fn xcv() -> A {
    let mem = alr();
    let ast = 0x50200;
    let bmy = za(&mem, ast + 4) as usize;
    
    let mv = &mem[ast + 36..ast + bmy];
    let oav = mv.ee(4).any(|d| d == b"_PRT");
    
    A {
        j: "dsdt_aml_prt_routing",
        cg: oav,
        eu: if !oav { Some(format!("_PRT not found in AML")) } else { None },
    }
}

fn xcu() -> A {
    let mem = alr();
    let ast = 0x50200;
    let bmy = za(&mem, ast + 4) as usize;
    
    let mv = &mem[ast + 36..ast + bmy];
    let oaw = mv.ee(4).any(|d| d == b"PWRB");
    
    A {
        j: "dsdt_aml_power_button",
        cg: oaw,
        eu: if !oaw { Some(format!("PWRB not found in AML")) } else { None },
    }
}

fn xcw() -> A {
    let mem = alr();
    let ast = 0x50200;
    let bmy = za(&mem, ast + 4) as usize;
    
    let mv = &mem[ast + 36..ast + bmy];
    let oax = mv.ee(4).any(|d| d == b"_S5_");
    
    A {
        j: "dsdt_aml_s5_shutdown",
        cg: oax,
        eu: if !oax { Some(format!("_S5_ not found in AML")) } else { None },
    }
}

fn xcs() -> A {
    let mem = alr();
    let ast = 0x50200;
    let bmy = za(&mem, ast + 4) as usize;
    
    let mv = &mem[ast + 36..ast + bmy];
    let oaj = mv.ee(4).any(|d| d == b"COM1");
    
    A {
        j: "dsdt_aml_com1_device",
        cg: oaj,
        eu: if !oaj { Some(format!("COM1 not found in AML")) } else { None },
    }
}

fn xcy() -> A {
    let mem = alr();
    let ast = 0x50200;
    let bmy = za(&mem, ast + 4) as usize;
    
    let bq = fyf(&mem, ast, bmy);
    A {
        j: "dsdt_checksum_with_aml",
        cg: bq,
        eu: if !bq { Some(format!("checksum failed for {} bytes", bmy)) } else { None },
    }
}





fn xen() -> A {
    let aq = super::pci::PciBus::default();
    let acs = u16::dj([aq.virtio_console[0], aq.virtio_console[1]]);
    let de = u16::dj([aq.virtio_console[2], aq.virtio_console[3]]);
    let bq = acs == 0x1AF4 && de == 0x1003;
    A {
        j: "pci_virtio_console_ids",
        cg: bq,
        eu: if !bq { Some(format!("vendor=0x{:04X} device=0x{:04X}", acs, de)) } else { None },
    }
}

fn xel() -> A {
    let aq = super::pci::PciBus::default();
    let aew = u32::dj([
        aq.virtio_console[0x10], aq.virtio_console[0x11],
        aq.virtio_console[0x12], aq.virtio_console[0x13],
    ]);
    
    let bq = aew == 0xC001;
    A {
        j: "pci_virtio_console_bar0",
        cg: bq,
        eu: if !bq { Some(format!("bar0=0x{:08X}", aew)) } else { None },
    }
}

fn xem() -> A {
    let aq = super::pci::PciBus::default();
    let class = aq.virtio_console[0x0B]; 
    let adl = aq.virtio_console[0x0A]; 
    let bq = class == 0x07 && adl == 0x80; 
    A {
        j: "pci_virtio_console_class",
        cg: bq,
        eu: if !bq { Some(format!("class=0x{:02X} sub=0x{:02X}", class, adl)) } else { None },
    }
}





fn xek() -> A {
    let aq = super::pci::PciBus::default();
    let acs = u16::dj([aq.virtio_blk[0], aq.virtio_blk[1]]);
    let de = u16::dj([aq.virtio_blk[2], aq.virtio_blk[3]]);
    let bq = acs == 0x1AF4 && de == 0x1001;
    A {
        j: "pci_virtio_blk_ids",
        cg: bq,
        eu: if !bq { Some(format!("vendor=0x{:04X} device=0x{:04X}", acs, de)) } else { None },
    }
}

fn xeh() -> A {
    let aq = super::pci::PciBus::default();
    let aew = u32::dj([
        aq.virtio_blk[0x10], aq.virtio_blk[0x11],
        aq.virtio_blk[0x12], aq.virtio_blk[0x13],
    ]);
    
    let bq = aew == 0xC041;
    A {
        j: "pci_virtio_blk_bar0",
        cg: bq,
        eu: if !bq { Some(format!("bar0=0x{:08X}", aew)) } else { None },
    }
}

fn xei() -> A {
    let aq = super::pci::PciBus::default();
    let class = aq.virtio_blk[0x0B];
    let adl = aq.virtio_blk[0x0A];
    let bq = class == 0x01 && adl == 0x80; 
    A {
        j: "pci_virtio_blk_class",
        cg: bq,
        eu: if !bq { Some(format!("class=0x{:02X} sub=0x{:02X}", class, adl)) } else { None },
    }
}

fn xej() -> A {
    let aq = super::pci::PciBus::default();
    let ppo = u16::dj([aq.virtio_blk[0x2C], aq.virtio_blk[0x2D]]);
    let ppn = u16::dj([aq.virtio_blk[0x2E], aq.virtio_blk[0x2F]]);
    
    let bq = ppo == 0x1AF4 && ppn == 0x0002;
    A {
        j: "pci_virtio_blk_subsystem",
        cg: bq,
        eu: if !bq { Some(format!("sv=0x{:04X} sid=0x{:04X}", ppo, ppn)) } else { None },
    }
}





fn xfd() -> A {
    let g = super::virtio_blk::VirtioBlkState::fc(64 * 512);
    let bq = g.gce == 64;
    A {
        j: "virtio_blk_capacity",
        cg: bq,
        eu: if !bq { Some(format!("capacity={}", g.gce)) } else { None },
    }
}

fn xfe() -> A {
    let mut g = super::virtio_blk::VirtioBlkState::fc(32768);
    let features = g.crq(0x00);
    
    let bq = (features & (1 << 1)) != 0 && (features & (1 << 2)) != 0;
    A {
        j: "virtio_blk_features",
        cg: bq,
        eu: if !bq { Some(format!("features=0x{:08X}", features)) } else { None },
    }
}

fn xfg() -> A {
    let mut g = super::virtio_blk::VirtioBlkState::fc(32768);
    
    g.edp(0x04, 0x07); 
    g.edp(0x12, 0x0F); 
    
    g.edp(0x12, 0);
    
    let status = g.crq(0x12);
    let cyj = g.crq(0x04);
    let bq = status == 0 && cyj == 0;
    A {
        j: "virtio_blk_reset",
        cg: bq,
        eu: if !bq { Some(format!("status={} features={}", status, cyj)) } else { None },
    }
}

fn xff() -> A {
    let mut g = super::virtio_blk::VirtioBlkState::default();
    let art = g.crq(0x0C);
    let bq = art == 128; 
    A {
        j: "virtio_blk_queue_size",
        cg: bq,
        eu: if !bq { Some(format!("queue_size={}", art)) } else { None },
    }
}

fn xfc() -> A {
    let mut g = super::virtio_blk::VirtioBlkState::fc(1024 * 1024); 
    let kgk = g.crq(0x14);
    let kgj = g.crq(0x18);
    let aty = (kgj as u64) << 32 | kgk as u64;
    let qy = (1024 * 1024 / 512) as u64; 
    let bq = aty == qy;
    A {
        j: "virtio_blk_cap_readback",
        cg: bq,
        eu: if !bq { Some(format!("capacity={} expected={}", aty, qy)) } else { None },
    }
}





fn xfh() -> A {
    let g = super::virtio_blk::VirtioConsoleState::default();
    let bq = g.ec == 80 && g.lk == 25 && g.jfk == 1;
    A {
        j: "virtio_console_defaults",
        cg: bq,
        eu: if !bq { Some(format!("cols={} rows={} ports={}", g.ec, g.lk, g.jfk)) } else { None },
    }
}

fn xfi() -> A {
    let mut g = super::virtio_blk::VirtioConsoleState::default();
    
    g.edp(0x12, 0x0F);
    let bky = g.crq(0x12);
    let bq = bky == 0x0F;
    A {
        j: "virtio_console_status",
        cg: bq,
        eu: if !bq { Some(format!("readback=0x{:02X}", bky)) } else { None },
    }
}

fn xfk() -> A {
    let mut g = super::virtio_blk::VirtioConsoleState::default();
    
    g.edp(0x0E, 1); 
    g.edp(0x08, 0x1000); 
    
    
    let duh = g.crq(0x08);
    let bq = duh == 0x1000;
    A {
        j: "virtio_console_queue",
        cg: bq,
        eu: if !bq { Some(format!("pfn=0x{:X}", duh)) } else { None },
    }
}

fn xfj() -> A {
    let mut g = super::virtio_blk::VirtioConsoleState::default();
    g.czc = 0x03; 
    
    let oha = g.crq(0x13);
    let ohb = g.crq(0x13);
    
    let bq = oha == 3 && ohb == 0;
    A {
        j: "virtio_console_isr_clear",
        cg: bq,
        eu: if !bq { Some(format!("isr1={} isr2={}", oha, ohb)) } else { None },
    }
}





fn xdc() -> A {
    let mem = alr();
    
    let cyo = 0x50400;
    let sig = &mem[cyo..cyo + 4];
    let bq = sig == b"HPET";
    A {
        j: "hpet_at_new_offset",
        cg: bq,
        eu: if !bq { Some(format!("sig={:?}", &sig)) } else { None },
    }
}
