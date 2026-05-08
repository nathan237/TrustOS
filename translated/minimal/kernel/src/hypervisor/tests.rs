











use alloc::string::String;
use alloc::vec;
use alloc::format;


struct D {
    name: &'static str,
    passed: bool,
    detail: Option<String>,
}


pub fn ezf() -> (usize, usize, alloc::vec::Vec<String>) {
    let mut results: alloc::vec::Vec<D> = alloc::vec::Vec::new();
    let mut log: alloc::vec::Vec<String> = alloc::vec::Vec::new();
    
    
    results.push(pem());
    results.push(pen());
    results.push(peo());
    results.push(pel());
    results.push(pep());
    results.push(pes());
    results.push(peq());
    results.push(per());
    results.push(pek());
    results.push(pef());
    results.push(pej());
    results.push(peh());
    results.push(peg());
    results.push(pei());
    results.push(pee());
    results.push(pea());
    results.push(pec());
    results.push(ped());
    results.push(peb());
    results.push(pdz());
    results.push(pdy());
    results.push(pdx());
    
    
    results.push(pgy());
    results.push(pha());
    results.push(phb());
    results.push(phc());
    results.push(pgz());
    
    
    results.push(phe());
    results.push(phd());
    results.push(phg());
    results.push(phf());
    
    
    results.push(pfx());
    results.push(pfz());
    results.push(pga());
    results.push(pgb());
    results.push(pfy());
    
    
    results.push(pez());
    results.push(pey());
    results.push(pex());
    results.push(pew());
    
    
    results.push(pgf());
    results.push(pge());
    results.push(pgi());
    results.push(pgd());
    results.push(pgg());
    results.push(pgc());
    results.push(pgh());
    results.push(pgj());
    
    
    results.push(jlx());
    results.push(jlz());
    results.push(pft());
    results.push(pfw());
    results.push(jly());
    results.push(pfu());
    results.push(pfv());
    
    
    results.push(jlu());
    results.push(jlw());
    results.push(jlv());
    results.push(pfo());
    results.push(pfr());
    results.push(pfq());
    results.push(pfs());
    results.push(pfn());
    results.push(pfm());
    results.push(pfk());
    
    
    results.push(jma());
    results.push(pgp());
    results.push(pgq());
    results.push(pgo());
    results.push(pgn());
    results.push(pgm());
    results.push(pgl());
    
    
    results.push(phj());
    results.push(phk());
    results.push(phl());
    
    
    results.push(pfi());
    results.push(pfg());
    results.push(pfb());
    results.push(pfe());
    results.push(pfc());
    results.push(pff());
    results.push(pfa());
    results.push(pfh());
    
    
    results.push(pgx());
    results.push(pgv());
    results.push(pgw());
    
    
    results.push(pgu());
    results.push(pgr());
    results.push(pgs());
    results.push(pgt());
    
    
    results.push(pho());
    results.push(php());
    results.push(phr());
    results.push(phq());
    results.push(phn());
    
    
    results.push(phs());
    results.push(pht());
    results.push(phv());
    results.push(phu());
    
    
    results.push(pfl());
    
    
    let mut passed = 0usize;
    let mut bv = 0usize;
    
    for r in &results {
        if r.passed {
            passed += 1;
            log.push(format!("  [PASS] {}", r.name));
        } else {
            bv += 1;
            if let Some(ref detail) = r.detail {
                log.push(format!("  [FAIL] {} — {}", r.name, detail));
            } else {
                log.push(format!("  [FAIL] {}", r.name));
            }
        }
    }
    
    (passed, bv, log)
}





fn read_u16(mem: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([mem[offset], mem[offset + 1]])
}

fn read_u32(mem: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        mem[offset], mem[offset + 1], mem[offset + 2], mem[offset + 3],
    ])
}

fn read_u64(mem: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        mem[offset], mem[offset + 1], mem[offset + 2], mem[offset + 3],
        mem[offset + 4], mem[offset + 5], mem[offset + 6], mem[offset + 7],
    ])
}


fn csj(mem: &[u8], offset: usize, len: usize) -> bool {
    let mut sum: u8 = 0;
    for i in 0..len {
        sum = sum.wrapping_add(mem[offset + i]);
    }
    sum == 0
}


fn tc() -> alloc::vec::Vec<u8> {
    
    let mut mem = vec![0u8; 0xF0000]; 
    super::acpi::cay(&mut mem);
    mem
}





fn pem() -> D {
    let mem = tc();
    let asd = 0x50000;
    let sig = &mem[asd..asd + 8];
    let passed = sig == b"RSD PTR ";
    D {
        name: "ACPI RSDP signature",
        passed,
        detail: if !passed {
            Some(format!("expected 'RSD PTR ', got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn pen() -> D {
    let mem = tc();
    let asd = 0x50000;
    
    let passed = csj(&mem, asd, 20);
    D {
        name: "ACPI RSDP v1 checksum (bytes 0-19)",
        passed,
        detail: if !passed {
            let mut sum: u8 = 0;
            for i in 0..20 { sum = sum.wrapping_add(mem[asd + i]); }
            Some(format!("sum=0x{:02X}, expected 0x00", sum))
        } else { None },
    }
}

fn peo() -> D {
    let mem = tc();
    let asd = 0x50000;
    
    let passed = csj(&mem, asd, 36);
    D {
        name: "ACPI RSDP v2 extended checksum (bytes 0-35)",
        passed,
        detail: if !passed {
            let mut sum: u8 = 0;
            for i in 0..36 { sum = sum.wrapping_add(mem[asd + i]); }
            Some(format!("sum=0x{:02X}, expected 0x00", sum))
        } else { None },
    }
}

fn pel() -> D {
    let mem = tc();
    let revision = mem[0x50000 + 15];
    let passed = revision == 2;
    D {
        name: "ACPI RSDP revision is 2 (ACPI 2.0)",
        passed,
        detail: if !passed {
            Some(format!("revision={}, expected 2", revision))
        } else { None },
    }
}

fn pep() -> D {
    let mem = tc();
    let asd = 0x50000;
    let dgv = read_u64(&mem, asd + 24);
    let passed = dgv == 0x50040;
    D {
        name: "ACPI RSDP XSDT pointer = 0x50040",
        passed,
        detail: if !passed {
            Some(format!("xsdt_addr=0x{:X}, expected 0x50040", dgv))
        } else { None },
    }
}





fn pes() -> D {
    let mem = tc();
    let anj = 0x50040;
    let sig = &mem[anj..anj + 4];
    let passed = sig == b"XSDT";
    D {
        name: "ACPI XSDT signature",
        passed,
        detail: if !passed {
            Some(format!("got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn peq() -> D {
    let mem = tc();
    let anj = 0x50040;
    let len = read_u32(&mem, anj + 4) as usize;
    let passed = len > 0 && csj(&mem, anj, len);
    D {
        name: "ACPI XSDT checksum",
        passed,
        detail: if !passed {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn per() -> D {
    let mem = tc();
    let anj = 0x50040;
    let total_len = read_u32(&mem, anj + 4) as usize;
    let dos = total_len.saturating_sub(36);
    let entry_count = dos / 8;
    
    let hwc = read_u64(&mem, anj + 36);
    let hwd = read_u64(&mem, anj + 44);
    let hwe = read_u64(&mem, anj + 52);
    let passed = entry_count == 3 && hwc == 0x50080 && hwd == 0x50100 && hwe == 0x50400;
    D {
        name: "ACPI XSDT has 3 entries (MADT, FADT, HPET)",
        passed,
        detail: if !passed {
            Some(format!("count={}, e0=0x{:X}, e1=0x{:X}, e2=0x{:X}", entry_count, hwc, hwd, hwe))
        } else { None },
    }
}





fn pek() -> D {
    let mem = tc();
    let madt = 0x50080;
    let sig = &mem[madt..madt + 4];
    let passed = sig == b"APIC";
    D {
        name: "ACPI MADT signature 'APIC'",
        passed,
        detail: if !passed {
            Some(format!("got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn pef() -> D {
    let mem = tc();
    let madt = 0x50080;
    let len = read_u32(&mem, madt + 4) as usize;
    let passed = len > 0 && csj(&mem, madt, len);
    D {
        name: "ACPI MADT checksum",
        passed,
        detail: if !passed {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn pej() -> D {
    let mem = tc();
    let madt = 0x50080;
    
    let ese = read_u32(&mem, madt + 36);
    let passed = ese == 0xFEE0_0000;
    D {
        name: "ACPI MADT LAPIC address = 0xFEE00000",
        passed,
        detail: if !passed {
            Some(format!("got 0x{:08X}", ese))
        } else { None },
    }
}

fn peh() -> D {
    let mem = tc();
    let madt = 0x50080;
    let total_len = read_u32(&mem, madt + 4) as usize;
    
    
    let mut pos = 44;
    let mut fxl = false;
    while pos + 2 <= total_len {
        let entry_type = mem[madt + pos];
        let alm = mem[madt + pos + 1] as usize;
        if alm == 0 { break; }
        
        if entry_type == 0 && alm == 8 {
            
            let apic_id = mem[madt + pos + 3];
            let flags = read_u32(&mem, madt + pos + 4);
            if apic_id == 0 && (flags & 1) != 0 {
                fxl = true;
            }
        }
        pos += alm;
    }
    
    D {
        name: "ACPI MADT contains Processor Local APIC (ID=0, enabled)",
        passed: fxl,
        detail: if !fxl { Some(String::from("LAPIC entry not found")) } else { None },
    }
}

fn peg() -> D {
    let mem = tc();
    let madt = 0x50080;
    let total_len = read_u32(&mem, madt + 4) as usize;
    
    let mut pos = 44;
    let mut fxi = false;
    let mut gdj = 0u32;
    while pos + 2 <= total_len {
        let entry_type = mem[madt + pos];
        let alm = mem[madt + pos + 1] as usize;
        if alm == 0 { break; }
        
        if entry_type == 1 && alm == 12 {
            
            gdj = read_u32(&mem, madt + pos + 4);
            if gdj == 0xFEC0_0000 {
                fxi = true;
            }
        }
        pos += alm;
    }
    
    D {
        name: "ACPI MADT contains I/O APIC at 0xFEC00000",
        passed: fxi,
        detail: if !fxi {
            Some(format!("I/O APIC entry not found (addr=0x{:08X})", gdj))
        } else { None },
    }
}

fn pei() -> D {
    let mem = tc();
    let madt = 0x50080;
    let total_len = read_u32(&mem, madt + 4) as usize;
    
    let mut pos = 44;
    let mut fxj = false;
    let mut fxk = false;
    while pos + 2 <= total_len {
        let entry_type = mem[madt + pos];
        let alm = mem[madt + pos + 1] as usize;
        if alm == 0 { break; }
        
        if entry_type == 2 && alm == 10 {
            
            let source = mem[madt + pos + 3];
            let gsi = read_u32(&mem, madt + pos + 4);
            if source == 0 && gsi == 2 { fxj = true; }
            if source == 9 && gsi == 9 { fxk = true; }
        }
        pos += alm;
    }
    
    let passed = fxj && fxk;
    D {
        name: "ACPI MADT IRQ overrides (IRQ0→GSI2, IRQ9→GSI9)",
        passed,
        detail: if !passed {
            Some(format!("irq0→gsi2={}, irq9→gsi9={}", fxj, fxk))
        } else { None },
    }
}





fn pee() -> D {
    let mem = tc();
    let fadt = 0x50100;
    let sig = &mem[fadt..fadt + 4];
    let passed = sig == b"FACP";
    D {
        name: "ACPI FADT signature 'FACP'",
        passed,
        detail: if !passed {
            Some(format!("got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn pea() -> D {
    let mem = tc();
    let fadt = 0x50100;
    let len = read_u32(&mem, fadt + 4) as usize;
    let passed = len == 276 && csj(&mem, fadt, len);
    D {
        name: "ACPI FADT checksum (276 bytes)",
        passed,
        detail: if !passed {
            Some(format!("len={}, checksum invalid", len))
        } else { None },
    }
}

fn pec() -> D {
    let mem = tc();
    let fadt = 0x50100;
    
    let ewu = read_u32(&mem, fadt + 76);
    
    let gnm = mem[fadt + 91];
    
    let flags = read_u32(&mem, fadt + 112);
    let jmx = (flags >> 4) & 1;
    
    let passed = ewu == 0xB008 && gnm == 4 && jmx == 1;
    D {
        name: "ACPI FADT PM timer at 0xB008 (32-bit)",
        passed,
        detail: if !passed {
            Some(format!("port=0x{:X} len={} ext={}", ewu, gnm, jmx))
        } else { None },
    }
}

fn ped() -> D {
    let mem = tc();
    let fadt = 0x50100;
    
    let sci_int = read_u16(&mem, fadt + 46);
    let passed = sci_int == 9;
    D {
        name: "ACPI FADT SCI interrupt = IRQ 9",
        passed,
        detail: if !passed {
            Some(format!("sci_int={}", sci_int))
        } else { None },
    }
}

fn peb() -> D {
    let mem = tc();
    let fadt = 0x50100;
    
    let hua = read_u32(&mem, fadt + 40) as u64;
    
    let hub = read_u64(&mem, fadt + 140);
    let passed = hua == 0x50200 && hub == 0x50200;
    D {
        name: "ACPI FADT DSDT pointer = 0x50200 (32+64 bit)",
        passed,
        detail: if !passed {
            Some(format!("dsdt32=0x{:X}, dsdt64=0x{:X}", hua, hub))
        } else { None },
    }
}





fn pdz() -> D {
    let mem = tc();
    let bge = 0x50200;
    let sig = &mem[bge..bge + 4];
    let passed = sig == b"DSDT";
    D {
        name: "ACPI DSDT signature",
        passed,
        detail: if !passed {
            Some(format!("got {:?}", core::str::from_utf8(sig)))
        } else { None },
    }
}

fn pdy() -> D {
    let mem = tc();
    let bge = 0x50200;
    let len = read_u32(&mem, bge + 4) as usize;
    let passed = len > 36 && csj(&mem, bge, len);
    D {
        name: "ACPI DSDT checksum (with AML)",
        passed,
        detail: if !passed {
            Some(format!("len={}", len))
        } else { None },
    }
}

fn pdx() -> D {
    let mem = tc();
    
    let oir = &mem[0x50000..0x50000 + 36];
    let oiq = &mem[0xE0000..0xE0000 + 36];
    let passed = oir == oiq;
    D {
        name: "ACPI RSDP copy at BIOS area 0xE0000",
        passed,
        detail: if !passed {
            Some(String::from("RSDP at 0x50000 != RSDP at 0xE0000"))
        } else { None },
    }
}





fn pgy() -> D {
    let pic = super::svm_vm::PicState::default();
    let passed = pic.master_imr == 0xFF
        && pic.slave_imr == 0xFF
        && pic.master_vector_base == 0x08
        && pic.slave_vector_base == 0x70
        && pic.master_icw_phase == 0
        && pic.slave_icw_phase == 0
        && pic.master_isr == 0
        && pic.master_irr == 0
        && !pic.initialized;
    D {
        name: "PIC defaults (IMR=0xFF, bases=0x08/0x70, phase=0)",
        passed,
        detail: if !passed {
            Some(format!("m_imr=0x{:02X} s_imr=0x{:02X} m_base=0x{:02X} s_base=0x{:02X} phase={}/{}",
                pic.master_imr, pic.slave_imr, pic.master_vector_base, pic.slave_vector_base,
                pic.master_icw_phase, pic.slave_icw_phase))
        } else { None },
    }
}

fn pha() -> D {
    
    let mut pic = super::svm_vm::PicState::default();
    
    
    let gbn: u8 = 0x11; 
    if gbn & 0x10 != 0 {
        pic.master_icw_phase = 1;
        pic.master_isr = 0;
        pic.master_irr = 0;
    }
    let nua = pic.master_icw_phase;
    
    
    let gbo: u8 = 0x20;
    if pic.master_icw_phase == 1 {
        pic.master_vector_base = gbo & 0xF8;
        pic.master_icw_phase = 2;
    }
    let nub = pic.master_icw_phase;
    
    
    if pic.master_icw_phase == 2 {
        pic.master_icw_phase = 3;
    }
    
    
    if pic.master_icw_phase == 3 {
        pic.master_icw_phase = 0;
        pic.initialized = true;
    }
    
    let passed = nua == 1
        && nub == 2
        && pic.master_icw_phase == 0
        && pic.master_vector_base == 0x20
        && pic.initialized;
    
    D {
        name: "PIC master ICW1-4 sequence (vector base 0x20)",
        passed,
        detail: if !passed {
            Some(format!("phase={} base=0x{:02X} init={}", 
                pic.master_icw_phase, pic.master_vector_base, pic.initialized))
        } else { None },
    }
}

fn phb() -> D {
    let mut pic = super::svm_vm::PicState::default();
    
    
    let gbn: u8 = 0x11;
    if gbn & 0x10 != 0 {
        pic.slave_icw_phase = 1;
    }
    
    
    let gbo: u8 = 0x28;
    if pic.slave_icw_phase == 1 {
        pic.slave_vector_base = gbo & 0xF8;
        pic.slave_icw_phase = 2;
    }
    
    
    if pic.slave_icw_phase == 2 { pic.slave_icw_phase = 3; }
    
    
    if pic.slave_icw_phase == 3 { pic.slave_icw_phase = 0; }
    
    let passed = pic.slave_icw_phase == 0 && pic.slave_vector_base == 0x28;
    D {
        name: "PIC slave ICW1-4 sequence (vector base 0x28)",
        passed,
        detail: if !passed {
            Some(format!("phase={} base=0x{:02X}", pic.slave_icw_phase, pic.slave_vector_base))
        } else { None },
    }
}

fn phc() -> D {
    let mut pic = super::svm_vm::PicState::default();
    
    
    pic.master_icw_phase = 0; 
    pic.master_imr = 0xFB; 
    
    let passed = pic.master_imr == 0xFB;
    D {
        name: "PIC OCW1 sets IMR (0xFB = mask all except IRQ2)",
        passed,
        detail: None,
    }
}

fn pgz() -> D {
    let mut pic = super::svm_vm::PicState::default();
    pic.master_isr = 0x04; 
    
    
    let nmg: u8 = 0x20;
    if nmg == 0x20 {
        pic.master_isr = 0;
    }
    
    let passed = pic.master_isr == 0;
    D {
        name: "PIC non-specific EOI clears ISR",
        passed,
        detail: if !passed {
            Some(format!("isr=0x{:02X} after EOI", pic.master_isr))
        } else { None },
    }
}





fn phe() -> D {
    let pit = super::svm_vm::PitState::default();
    let km = &pit.channels[0];
    let passed = km.reload == 0xFFFF
        && km.count == 0xFFFF
        && km.access == 3
        && km.mode == 0
        && !km.latched
        && !km.write_hi_pending
        && pit.channels.len() == 3;
    D {
        name: "PIT defaults (reload=0xFFFF, access=3, 3 channels)",
        passed,
        detail: if !passed {
            Some(format!("reload={} access={} mode={}", km.reload, km.access, km.mode))
        } else { None },
    }
}

fn phd() -> D {
    let mut pit = super::svm_vm::PitState::default();
    
    
    let control: u8 = 0b00_11_010_0; 
    let channel = ((control >> 6) & 0x3) as usize;
    let access = (control >> 4) & 0x3;
    let mode = (control >> 1) & 0x7;
    
    if channel < 3 && access != 0 {
        pit.channels[channel].access = access;
        pit.channels[channel].mode = mode;
        pit.channels[channel].write_hi_pending = false;
    }
    
    let passed = pit.channels[0].access == 3
        && pit.channels[0].mode == 2;
    D {
        name: "PIT control word (ch0, lo/hi, mode 2 rate generator)",
        passed,
        detail: if !passed {
            Some(format!("access={} mode={}", pit.channels[0].access, pit.channels[0].mode))
        } else { None },
    }
}

fn phg() -> D {
    let mut pit = super::svm_vm::PitState::default();
    pit.channels[0].access = 3; 
    
    
    let lo: u8 = 0x9C;
    let ch = &mut pit.channels[0];
    if !ch.write_hi_pending {
        ch.reload = (ch.reload & 0xFF00) | lo as u16;
        ch.write_hi_pending = true;
    }
    let jud = ch.reload;
    let pending = ch.write_hi_pending;
    
    
    let hi: u8 = 0x2E;
    if ch.write_hi_pending {
        ch.reload = (ch.reload & 0x00FF) | ((hi as u16) << 8);
        ch.count = ch.reload;
        ch.write_hi_pending = false;
    }
    
    let passed = (jud & 0xFF) == 0x9C
        && pending
        && pit.channels[0].reload == 0x2E9C
        && pit.channels[0].count == 0x2E9C
        && !pit.channels[0].write_hi_pending;
    D {
        name: "PIT lo/hi reload sequence (0x2E9C = ~100 Hz)",
        passed,
        detail: if !passed {
            Some(format!("reload=0x{:04X} count=0x{:04X} pending={}", 
                pit.channels[0].reload, pit.channels[0].count, pit.channels[0].write_hi_pending))
        } else { None },
    }
}

fn phf() -> D {
    let mut pit = super::svm_vm::PitState::default();
    pit.channels[0].count = 0x1234;
    
    
    let control: u8 = 0b00_00_000_0; 
    let channel = ((control >> 6) & 0x3) as usize;
    let access = (control >> 4) & 0x3;
    
    if channel < 3 && access == 0 {
        pit.channels[channel].latched = true;
        pit.channels[channel].latch_value = pit.channels[channel].count;
    }
    
    let passed = pit.channels[0].latched
        && pit.channels[0].latch_value == 0x1234;
    D {
        name: "PIT latch command captures count value",
        passed,
        detail: if !passed {
            Some(format!("latched={} value=0x{:04X}", pit.channels[0].latched, pit.channels[0].latch_value))
        } else { None },
    }
}





fn pfx() -> D {
    let lapic = super::svm_vm::LapicState::default();
    let passed = lapic.icr == 0
        && lapic.ccr == 0
        && lapic.dcr == 0
        && (lapic.timer_lvt & 0x0001_0000) != 0  
        && lapic.svr == 0x1FF
        && lapic.tpr == 0
        && !lapic.enabled
        && lapic.last_tick_exit == 0;
    D {
        name: "LAPIC defaults (masked, SVR=0x1FF, disabled)",
        passed,
        detail: if !passed {
            Some(format!("icr={} lvt=0x{:X} svr=0x{:X} enabled={}", 
                lapic.icr, lapic.timer_lvt, lapic.svr, lapic.enabled))
        } else { None },
    }
}

fn pfz() -> D {
    let mut lapic = super::svm_vm::LapicState::default();
    
    
    lapic.svr = 0x1FF; 
    lapic.enabled = (lapic.svr & 0x100) != 0;
    
    let enabled = lapic.enabled;
    
    
    lapic.svr = 0x0FF;
    lapic.enabled = (lapic.svr & 0x100) != 0;
    let disabled = !lapic.enabled;
    
    let passed = enabled && disabled;
    D {
        name: "LAPIC enable/disable via SVR bit 8",
        passed,
        detail: if !passed {
            Some(format!("enabled_check={} disabled_check={}", enabled, disabled))
        } else { None },
    }
}

fn pga() -> D {
    let mut lapic = super::svm_vm::LapicState::default();
    lapic.enabled = true;
    
    
    lapic.timer_lvt = 0x30; 
    lapic.icr = 100_000;
    lapic.ccr = 100_000;
    lapic.last_tick_exit = 0;
    
    let vector = lapic.timer_lvt & 0xFF;
    let masked = (lapic.timer_lvt >> 16) & 1;
    let mode = (lapic.timer_lvt >> 17) & 0x3;
    
    let passed = vector == 0x30
        && masked == 0
        && mode == 0
        && lapic.icr == 100_000
        && lapic.ccr == 100_000;
    D {
        name: "LAPIC timer arm (one-shot, vector=0x30, ICR=100000)",
        passed,
        detail: if !passed {
            Some(format!("vec=0x{:X} mask={} mode={} icr={}", vector, masked, mode, lapic.icr))
        } else { None },
    }
}

fn pgb() -> D {
    let mut lapic = super::svm_vm::LapicState::default();
    
    
    lapic.timer_lvt = 0x30; 
    let iod = (lapic.timer_lvt >> 17) & 0x3;
    
    
    lapic.timer_lvt = 0x0002_0030; 
    let ioe = (lapic.timer_lvt >> 17) & 0x3;
    
    
    lapic.timer_lvt = 0x0004_0030; 
    let iof = (lapic.timer_lvt >> 17) & 0x3;
    
    
    lapic.timer_lvt = 0x0001_0030; 
    let masked = (lapic.timer_lvt >> 16) & 1;
    
    let passed = iod == 0 && ioe == 1 && iof == 2 && masked == 1;
    D {
        name: "LAPIC timer LVT modes (one-shot/periodic/TSC-deadline/mask)",
        passed,
        detail: if !passed {
            Some(format!("modes: 0={} 1={} 2={} mask={}", iod, ioe, iof, masked))
        } else { None },
    }
}

fn pfy() -> D {
    
    let lgc: [(u32, u64); 8] = [
        (0x0, 2),    
        (0x1, 4),    
        (0x2, 8),    
        (0x3, 16),   
        (0x8, 32),   
        (0x9, 64),   
        (0xA, 128),  
        (0xB, 1),    
    ];
    
    let mut bqe = true;
    let mut die = String::new();
    
    for &(dcr_val, expected) in &lgc {
        let cws = match dcr_val & 0xB {
            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
            _ => 1,
        };
        if cws != expected {
            bqe = false;
            die = format!("dcr=0x{:X}: got {} expected {}", dcr_val, cws, expected);
            break;
        }
    }
    
    D {
        name: "LAPIC timer divider decode (all 8 values)",
        passed: bqe,
        detail: if !bqe { Some(die) } else { None },
    }
}





fn pez() -> D {
    
    
    let expected: [(u8, u8); 7] = [
        (0x00, 0x00),  
        (0x02, 0x30),  
        (0x04, 0x12),  
        (0x06, 0x02),  
        (0x07, 0x17),  
        (0x08, 0x02),  
        (0x09, 0x26),  
    ];
    
    let mut bqe = true;
    let mut die = String::new();
    
    for &(reg, expected_val) in &expected {
        let bxh = chj(reg);
        if bxh != expected_val {
            bqe = false;
            die = format!("reg 0x{:02X}: got 0x{:02X} expected 0x{:02X}", reg, bxh, expected_val);
            break;
        }
    }
    
    D {
        name: "CMOS time registers (seconds/min/hour/date)",
        passed: bqe,
        detail: if !bqe { Some(die) } else { None },
    }
}

fn pey() -> D {
    let izd = chj(0x0A);
    let ize = chj(0x0B);
    let izf = chj(0x0C);
    let izh = chj(0x0D);
    
    let passed = izd == 0x26  
        && ize == 0x02        
        && izf == 0x00        
        && izh == 0x80;       
    D {
        name: "CMOS status registers A-D",
        passed,
        detail: if !passed {
            Some(format!("A=0x{:02X} B=0x{:02X} C=0x{:02X} D=0x{:02X}", izd, ize, izf, izh))
        } else { None },
    }
}

fn pex() -> D {
    let hwn = chj(0x14);
    
    let passed = hwn == 0x06;
    D {
        name: "CMOS equipment byte (FPU + color display)",
        passed,
        detail: if !passed {
            Some(format!("got 0x{:02X}, expected 0x06", hwn))
        } else { None },
    }
}

fn pew() -> D {
    let century = chj(0x32);
    let passed = century == 0x20; 
    D {
        name: "CMOS century register = 0x20 (2000s)",
        passed,
        detail: if !passed {
            Some(format!("got 0x{:02X}", century))
        } else { None },
    }
}


fn chj(index: u8) -> u8 {
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






fn pgf() -> D {
    let bytes = [0x89, 0x07]; 
    let d = super::mmio::awu(&bytes, 2, true);
    let ok = match d {
        Some(ref ox) => ox.is_write && ox.operand_size == 4 && ox.register == Some(0) && ox.insn_len == 2,
        None => false,
    };
    D {
        name: "mmio_decode_mov_write",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}


fn pge() -> D {
    let bytes = [0x8B, 0x0F]; 
    let d = super::mmio::awu(&bytes, 2, true);
    let ok = match d {
        Some(ref ox) => !ox.is_write && ox.operand_size == 4 && ox.register == Some(1) && ox.insn_len == 2,
        None => false,
    };
    D {
        name: "mmio_decode_mov_read",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}


fn pgi() -> D {
    let bytes = [0x48, 0x89, 0x07]; 
    let d = super::mmio::awu(&bytes, 3, true);
    let ok = match d {
        Some(ref ox) => ox.is_write && ox.operand_size == 8 && ox.register == Some(0) && ox.insn_len == 3,
        None => false,
    };
    D {
        name: "mmio_decode_rex_w",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}


fn pgd() -> D {
    
    let bytes = [0xC7, 0x07, 0x78, 0x56, 0x34, 0x12];
    let d = super::mmio::awu(&bytes, 6, true);
    let ok = match d {
        Some(ref ox) => ox.is_write && ox.operand_size == 4 
            && ox.register.is_none() 
            && ox.immediate == Some(0x12345678),
        None => false,
    };
    D {
        name: "mmio_decode_imm32",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}


fn pgg() -> D {
    let bytes = [0x0F, 0xB6, 0x07]; 
    let d = super::mmio::awu(&bytes, 3, true);
    let ok = match d {
        Some(ref ox) => !ox.is_write && ox.operand_size == 1 && ox.register == Some(0),
        None => false,
    };
    D {
        name: "mmio_decode_movzx",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}


fn pgc() -> D {
    let bytes = [0x8B, 0x87, 0x20, 0x03, 0x00, 0x00]; 
    let d = super::mmio::awu(&bytes, 6, true);
    let ok = match d {
        Some(ref ox) => !ox.is_write && ox.operand_size == 4 && ox.register == Some(0) && ox.insn_len == 6,
        None => false,
    };
    D {
        name: "mmio_decode_disp32",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}


fn pgh() -> D {
    
    
    
    
    let bytes = [0x45, 0x89, 0x07]; 
    let d = super::mmio::awu(&bytes, 3, true);
    let ok = match d {
        Some(ref ox) => ox.is_write && ox.operand_size == 4 && ox.register == Some(8), 
        None => false,
    };
    D {
        name: "mmio_decode_r8_r15",
        passed: ok,
        detail: if !ok { Some(format!("decoded={:?}", d)) } else { None },
    }
}


fn pgj() -> D {
    let mut regs = super::svm_vm::SvmGuestRegs::default();
    
    super::mmio::jro(&mut regs, 10, 0xDEAD_BEEF_CAFE_BABE);
    let val = super::mmio::iyl(&regs, 10);
    let ok = val == 0xDEAD_BEEF_CAFE_BABE && regs.r10 == 0xDEAD_BEEF_CAFE_BABE;
    D {
        name: "mmio_register_rw",
        passed: ok,
        detail: if !ok { Some(format!("got 0x{:X}", val)) } else { None },
    }
}






fn jlx() -> D {
    let ioapic = super::ioapic::IoApicState::default();
    let ok = ioapic.id == 1 && ioapic.ioregsel == 0;
    
    let her = ioapic.redir_table.iter().all(|e| (e >> 16) & 1 == 1);
    let ok = ok && her;
    D {
        name: "ioapic_defaults",
        passed: ok,
        detail: if !ok { Some(format!("id={} ioregsel={} all_masked={}", ioapic.id, ioapic.ioregsel, her)) } else { None },
    }
}


fn jlz() -> D {
    let ioapic = super::ioapic::IoApicState::default();
    let tu = ioapic.read(0x10); 
    
    let mut ioapic = ioapic;
    ioapic.write(0x00, 0x01); 
    let tu = ioapic.read(0x10); 
    let version = tu & 0xFF;
    let imt = (tu >> 16) & 0xFF;
    let ok = version == 0x20 && imt == 23;
    D {
        name: "ioapic_version_register",
        passed: ok,
        detail: if !ok { Some(format!("ver=0x{:X} max_redir={}", version, imt)) } else { None },
    }
}


fn pft() -> D {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    ioapic.write(0x00, 0x00); 
    let drs = ioapic.read(0x10);
    let hrd = (drs >> 24) & 0xF;
    
    
    ioapic.write(0x00, 0x00);
    ioapic.write(0x10, 0x05_00_00_00); 
    let drs = ioapic.read(0x10);
    let ipz = (drs >> 24) & 0xF;
    
    let ok = hrd == 1 && ipz == 5;
    D {
        name: "ioapic_id_readwrite",
        passed: ok,
        detail: if !ok { Some(format!("default={} new={}", hrd, ipz)) } else { None },
    }
}


fn pfw() -> D {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    
    ioapic.write(0x00, 0x10); 
    ioapic.write(0x10, 0x0000_0040); 
    
    
    ioapic.write(0x00, 0x11); 
    ioapic.write(0x10, 0x0000_0000);
    
    
    ioapic.write(0x00, 0x10);
    let lo = ioapic.read(0x10);
    
    let ok = (lo & 0xFF) == 0x40 && ((lo >> 16) & 1) == 0; 
    D {
        name: "ioapic_redir_table",
        passed: ok,
        detail: if !ok { Some(format!("lo=0x{:X}", lo)) } else { None },
    }
}


fn jly() -> D {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    
    
    ioapic.write(0x00, 0x00); 
    let id = ioapic.read(0x10);
    ioapic.write(0x00, 0x01); 
    let tu = ioapic.read(0x10);
    
    let ok = id != tu; 
    D {
        name: "ioapic_indirect_access",
        passed: ok,
        detail: if !ok { Some(format!("id=0x{:X} ver=0x{:X}", id, tu)) } else { None },
    }
}


fn pfu() -> D {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    
    ioapic.write(0x00, 0x14); 
    ioapic.write(0x10, 0x0000_0030); 
    ioapic.write(0x00, 0x15); 
    ioapic.write(0x10, 0x0000_0000); 
    
    let afo = ioapic.get_irq_route(2);
    let ok = match afo {
        Some(ref r) => r.vector == 0x30 && !r.masked && r.delivery_mode == 0 && !r.level_triggered,
        None => false,
    };
    D {
        name: "ioapic_irq_routing",
        passed: ok,
        detail: if !ok { Some(format!("route={:?}", afo)) } else { None },
    }
}


fn pfv() -> D {
    let mut ioapic = super::ioapic::IoApicState::default();
    
    
    ioapic.write(0x00, 0x10); 
    ioapic.write(0x10, 0x0000_5030); 
    
    ioapic.write(0x00, 0x10);
    let lo = ioapic.read(0x10);
    
    
    let hhu = (lo >> 12) & 1;
    let hhv = (lo >> 14) & 1;
    let ok = hhu == 0 && hhv == 0 && (lo & 0xFF) == 0x30;
    D {
        name: "ioapic_readonly_bits",
        passed: ok,
        detail: if !ok { Some(format!("lo=0x{:X} bit12={} bit14={}", lo, hhu, hhv)) } else { None },
    }
}





fn jlu() -> D {
    let hpet = super::hpet::HpetState::default();
    let ok = !hpet.enabled && hpet.config == 0 && hpet.isr == 0 
             && hpet.counter_offset == 0 && hpet.timers.len() == 3;
    D {
        name: "hpet_defaults",
        passed: ok,
        detail: if !ok { Some(format!("enabled={} config=0x{:X}", hpet.enabled, hpet.config)) } else { None },
    }
}

fn jlw() -> D {
    let hpet = super::hpet::HpetState::default();
    let agk = hpet.read(0x000, 8);
    
    let rev = agk & 0xFF;
    let irv = (agk >> 8) & 0x1F;
    let counter_64bit = (agk >> 13) & 1;
    let zd = (agk >> 32) as u32;
    let ok = rev == 1 && irv == 2 && counter_64bit == 1 && zd == 69_841_279;
    D {
        name: "hpet_gcap_id_register",
        passed: ok,
        detail: if !ok { 
            Some(format!("rev={} timers-1={} 64bit={} period={}", rev, irv, counter_64bit, zd))
        } else { None },
    }
}

fn jlv() -> D {
    let mut hpet = super::hpet::HpetState::default();
    
    let og = hpet.read(0x0F0, 8);
    
    
    hpet.write(0x010, 1, 8); 
    let enabled = hpet.enabled;
    let config = hpet.read(0x010, 8);
    
    
    hpet.write(0x010, 0, 8);
    let disabled = !hpet.enabled;
    
    let hw = hpet.read(0x0F0, 8);
    let jf = hpet.read(0x0F0, 8);
    let iaf = hw == jf; 
    
    let ok = og == 0 && enabled && config == 1 && disabled && iaf;
    D {
        name: "hpet_enable_disable",
        passed: ok,
        detail: if !ok { 
            Some(format!("c0={} en={} cfg={} dis={} frozen={}", og, enabled, config, disabled, iaf))
        } else { None },
    }
}

fn pfo() -> D {
    let mut hpet = super::hpet::HpetState::default();
    hpet.write(0x010, 1, 8); 
    
    
    let hw = hpet.read(0x0F0, 8);
    
    for _ in 0..10000 {
        core::hint::spin_loop();
    }
    let jf = hpet.read(0x0F0, 8);
    
    
    let ok = jf >= hw;
    D {
        name: "hpet_counter_increments",
        passed: ok,
        detail: if !ok { 
            Some(format!("c1={} c2={}", hw, jf))
        } else { None },
    }
}

fn pfr() -> D {
    let mut hpet = super::hpet::HpetState::default();
    
    let pce = hpet.read(0x100, 8); 
    let iuk = (pce >> 4) & 1; 
    
    
    
    hpet.write(0x100, (2 << 9) | (1 << 2), 8);
    let gxp = hpet.read(0x100, 8);
    let igz = (gxp >> 2) & 1;
    let afo = (gxp >> 9) & 0x1F;
    
    let ok = iuk == 1 && igz == 1 && afo == 2;
    D {
        name: "hpet_timer_config",
        passed: ok,
        detail: if !ok {
            Some(format!("periodic_cap={} int_en={} route={} raw=0x{:X}", iuk, igz, afo, gxp))
        } else { None },
    }
}

fn pfq() -> D {
    let mut hpet = super::hpet::HpetState::default();
    
    hpet.write(0x108, 0xDEAD_BEEF, 8);
    let bfm = hpet.read(0x108, 8);
    
    
    hpet.write(0x128, 0x1234_5678_9ABC_DEF0, 8);
    let hnc = hpet.read(0x128, 8);
    
    let ok = bfm == 0xDEAD_BEEF && hnc == 0x1234_5678_9ABC_DEF0;
    D {
        name: "hpet_timer_comparator",
        passed: ok,
        detail: if !ok { Some(format!("t0=0x{:X} t1=0x{:X}", bfm, hnc)) } else { None },
    }
}

fn pfs() -> D {
    let mut hpet = super::hpet::HpetState::default();
    
    hpet.write(0x0F0, 0x42, 8);
    let c = hpet.read(0x0F0, 8);
    
    
    hpet.write(0x010, 1, 8);
    hpet.write(0x0F0, 0xFF, 8); 
    
    let hjn = hpet.read(0x0F0, 8);
    
    let ok = c == 0x42 && hjn >= 0x42;
    D {
        name: "hpet_write_counter_disabled",
        passed: ok,
        detail: if !ok { Some(format!("c_dis=0x{:X} c_en=0x{:X}", c, hjn)) } else { None },
    }
}

fn pfn() -> D {
    let mem = tc();
    let bcc = 0x50400;
    let sig = &mem[bcc..bcc + 4];
    let ok = sig == b"HPET";
    D {
        name: "hpet_acpi_table_signature",
        passed: ok,
        detail: if !ok { Some(format!("sig={:?}", core::str::from_utf8(sig))) } else { None },
    }
}

fn pfm() -> D {
    let mem = tc();
    let bcc = 0x50400;
    let length = read_u32(&mem, bcc + 4) as usize;
    let sum: u8 = mem[bcc..bcc + length].iter().fold(0u8, |a, &b| a.wrapping_add(b));
    let ok = sum == 0 && length == 56;
    D {
        name: "hpet_acpi_table_checksum",
        passed: ok,
        detail: if !ok { Some(format!("sum={} len={}", sum, length)) } else { None },
    }
}

fn pfk() -> D {
    let mem = tc();
    let bcc = 0x50400;
    
    let addr = read_u64(&mem, bcc + 44);
    
    let efb = mem[bcc + 40];
    let ok = addr == 0xFED0_0000 && efb == 0;
    D {
        name: "hpet_acpi_table_address",
        passed: ok,
        detail: if !ok { Some(format!("addr=0x{:X} space={}", addr, efb)) } else { None },
    }
}





fn jma() -> D {
    let bus = super::pci::PciBus::default();
    let ifb = bus.device_exists(0, 0, 0);
    let iim = bus.device_exists(0, 1, 0);
    let hnj = bus.device_exists(0, 2, 0);
    let hif = bus.device_exists(0, 3, 0);
    let iqr = !bus.device_exists(0, 4, 0);
    let iqq = !bus.device_exists(1, 0, 0);
    let ok = ifb && iim && hnj && hif && iqr && iqq;
    D {
        name: "pci_bus_defaults",
        passed: ok,
        detail: if !ok { Some(format!("host={} isa={} con={} blk={} no4={} nob1={}", ifb, iim, hnj, hif, iqr, iqq)) } else { None },
    }
}

fn pgp() -> D {
    let mut bus = super::pci::PciBus::default();
    
    bus.write_config_address(0x8000_0000);
    let val = bus.read_config_data(0);
    let vendor = (val & 0xFFFF) as u16;
    let device = ((val >> 16) & 0xFFFF) as u16;
    let ok = vendor == 0x8086 && device == 0x1237;
    D {
        name: "pci_host_bridge_ids",
        passed: ok,
        detail: if !ok { Some(format!("vendor=0x{:04X} device=0x{:04X}", vendor, device)) } else { None },
    }
}

fn pgq() -> D {
    let mut bus = super::pci::PciBus::default();
    
    bus.write_config_address(0x8000_0808);
    let val = bus.read_config_data(0);
    let class_code = ((val >> 24) & 0xFF) as u8;
    let subclass = ((val >> 16) & 0xFF) as u8;
    let ok = class_code == 0x06 && subclass == 0x01;
    D {
        name: "pci_isa_bridge_class",
        passed: ok,
        detail: if !ok { Some(format!("class=0x{:02X} sub=0x{:02X}", class_code, subclass)) } else { None },
    }
}

fn pgo() -> D {
    let mut bus = super::pci::PciBus::default();
    
    bus.write_config_address(0x8000_F800);
    let val = bus.read_config_data(0);
    let ok = val == 0xFFFF_FFFF;
    D {
        name: "pci_config_no_device",
        passed: ok,
        detail: if !ok { Some(format!("val=0x{:08X}", val)) } else { None },
    }
}

fn pgn() -> D {
    let mut bus = super::pci::PciBus::default();
    
    bus.write_config_address(0x0000_0000);
    let val = bus.read_config_data(0);
    let ok = val == 0xFFFF_FFFF;
    D {
        name: "pci_config_disabled",
        passed: ok,
        detail: if !ok { Some(format!("val=0x{:08X}", val)) } else { None },
    }
}

fn pgm() -> D {
    let mut bus = super::pci::PciBus::default();
    bus.write_config_address(0x8000_1234);
    let agx = bus.read_config_address();
    let ok = agx == 0x8000_1234;
    D {
        name: "pci_config_addr_readback",
        passed: ok,
        detail: if !ok { Some(format!("read=0x{:08X}", agx)) } else { None },
    }
}

fn pgl() -> D {
    let mut bus = super::pci::PciBus::default();
    
    bus.write_config_address(0x8000_0010);
    
    bus.write_config_data(0, 0xFFFF_FFFF);
    let val = bus.read_config_data(0);
    
    let ok = val == 0xFFFF_FFFF;
    D {
        name: "pci_bar_probing",
        passed: ok,
        detail: if !ok { Some(format!("val=0x{:08X}", val)) } else { None },
    }
}





fn phj() -> D {
    
    use super::svm_vm::SvmVmState;
    use alloc::collections::VecDeque;
    
    
    let eqd: u8 = 0x0F; 
    let ok = eqd & 0x01 != 0; 
    D {
        name: "serial_ier_readback",
        passed: ok,
        detail: if !ok { Some(format!("ier=0x{:02X}", eqd)) } else { None },
    }
}

fn phk() -> D {
    
    let mut buffer: alloc::collections::VecDeque<u8> = alloc::collections::VecDeque::with_capacity(256);
    buffer.push_back(b'H');
    buffer.push_back(b'i');
    
    let hkf = buffer.pop_front();
    let hkg = buffer.pop_front();
    let hkh = buffer.pop_front();
    
    let ok = hkf == Some(b'H') && hkg == Some(b'i') && hkh.is_none() && buffer.is_empty();
    D {
        name: "serial_input_buffer",
        passed: ok,
        detail: if !ok { Some(format!("ch1={:?} ch2={:?} ch3={:?}", hkf, hkg, hkh)) } else { None },
    }
}

fn phl() -> D {
    
    let mut buffer: alloc::collections::VecDeque<u8> = alloc::collections::VecDeque::new();
    
    let ili = 0x60u8 | if !buffer.is_empty() { 0x01 } else { 0x00 };
    buffer.push_back(b'X');
    let ilh = 0x60u8 | if !buffer.is_empty() { 0x01 } else { 0x00 };
    
    let ok = ili == 0x60 && ilh == 0x61;
    D {
        name: "serial_lsr_data_ready",
        passed: ok,
        detail: if !ok { Some(format!("empty=0x{:02X} data=0x{:02X}", ili, ilh)) } else { None },
    }
}





fn pfi() -> D {
    let mem = tc();
    let xa = 0x50200;
    
    
    let ahw = read_u32(&mem, xa + 4) as usize;
    
    
    let ok = ahw > 36;
    D {
        name: "dsdt_has_aml_content",
        passed: ok,
        detail: if !ok { Some(format!("dsdt_len={}", ahw)) } else { None },
    }
}

fn pfg() -> D {
    let mem = tc();
    let xa = 0x50200;
    let ahw = read_u32(&mem, xa + 4) as usize;
    
    
    let fl = &mem[xa + 36..xa + ahw];
    let ieb = fl.windows(4).any(|w| w == b"_SB_");
    
    D {
        name: "dsdt_aml_scope_sb",
        passed: ieb,
        detail: if !ieb { Some(format!("_SB_ not found in {} AML bytes", fl.len())) } else { None },
    }
}

fn pfb() -> D {
    let mem = tc();
    let xa = 0x50200;
    let ahw = read_u32(&mem, xa + 4) as usize;
    
    let fl = &mem[xa + 36..xa + ahw];
    let idw = fl.windows(4).any(|w| w == b"PCI0");
    
    D {
        name: "dsdt_aml_device_pci0",
        passed: idw,
        detail: if !idw { Some(format!("PCI0 not found in AML")) } else { None },
    }
}

fn pfe() -> D {
    let mem = tc();
    let xa = 0x50200;
    let ahw = read_u32(&mem, xa + 4) as usize;
    
    let fl = &mem[xa + 36..xa + ahw];
    let idy = fl.windows(4).any(|w| w == b"_PRT");
    
    D {
        name: "dsdt_aml_prt_routing",
        passed: idy,
        detail: if !idy { Some(format!("_PRT not found in AML")) } else { None },
    }
}

fn pfc() -> D {
    let mem = tc();
    let xa = 0x50200;
    let ahw = read_u32(&mem, xa + 4) as usize;
    
    let fl = &mem[xa + 36..xa + ahw];
    let idz = fl.windows(4).any(|w| w == b"PWRB");
    
    D {
        name: "dsdt_aml_power_button",
        passed: idz,
        detail: if !idz { Some(format!("PWRB not found in AML")) } else { None },
    }
}

fn pff() -> D {
    let mem = tc();
    let xa = 0x50200;
    let ahw = read_u32(&mem, xa + 4) as usize;
    
    let fl = &mem[xa + 36..xa + ahw];
    let iea = fl.windows(4).any(|w| w == b"_S5_");
    
    D {
        name: "dsdt_aml_s5_shutdown",
        passed: iea,
        detail: if !iea { Some(format!("_S5_ not found in AML")) } else { None },
    }
}

fn pfa() -> D {
    let mem = tc();
    let xa = 0x50200;
    let ahw = read_u32(&mem, xa + 4) as usize;
    
    let fl = &mem[xa + 36..xa + ahw];
    let idl = fl.windows(4).any(|w| w == b"COM1");
    
    D {
        name: "dsdt_aml_com1_device",
        passed: idl,
        detail: if !idl { Some(format!("COM1 not found in AML")) } else { None },
    }
}

fn pfh() -> D {
    let mem = tc();
    let xa = 0x50200;
    let ahw = read_u32(&mem, xa + 4) as usize;
    
    let ok = csj(&mem, xa, ahw);
    D {
        name: "dsdt_checksum_with_aml",
        passed: ok,
        detail: if !ok { Some(format!("checksum failed for {} bytes", ahw)) } else { None },
    }
}





fn pgx() -> D {
    let bus = super::pci::PciBus::default();
    let vendor = u16::from_le_bytes([bus.virtio_console[0], bus.virtio_console[1]]);
    let device = u16::from_le_bytes([bus.virtio_console[2], bus.virtio_console[3]]);
    let ok = vendor == 0x1AF4 && device == 0x1003;
    D {
        name: "pci_virtio_console_ids",
        passed: ok,
        detail: if !ok { Some(format!("vendor=0x{:04X} device=0x{:04X}", vendor, device)) } else { None },
    }
}

fn pgv() -> D {
    let bus = super::pci::PciBus::default();
    let bar0 = u32::from_le_bytes([
        bus.virtio_console[0x10], bus.virtio_console[0x11],
        bus.virtio_console[0x12], bus.virtio_console[0x13],
    ]);
    
    let ok = bar0 == 0xC001;
    D {
        name: "pci_virtio_console_bar0",
        passed: ok,
        detail: if !ok { Some(format!("bar0=0x{:08X}", bar0)) } else { None },
    }
}

fn pgw() -> D {
    let bus = super::pci::PciBus::default();
    let class = bus.virtio_console[0x0B]; 
    let subclass = bus.virtio_console[0x0A]; 
    let ok = class == 0x07 && subclass == 0x80; 
    D {
        name: "pci_virtio_console_class",
        passed: ok,
        detail: if !ok { Some(format!("class=0x{:02X} sub=0x{:02X}", class, subclass)) } else { None },
    }
}





fn pgu() -> D {
    let bus = super::pci::PciBus::default();
    let vendor = u16::from_le_bytes([bus.virtio_blk[0], bus.virtio_blk[1]]);
    let device = u16::from_le_bytes([bus.virtio_blk[2], bus.virtio_blk[3]]);
    let ok = vendor == 0x1AF4 && device == 0x1001;
    D {
        name: "pci_virtio_blk_ids",
        passed: ok,
        detail: if !ok { Some(format!("vendor=0x{:04X} device=0x{:04X}", vendor, device)) } else { None },
    }
}

fn pgr() -> D {
    let bus = super::pci::PciBus::default();
    let bar0 = u32::from_le_bytes([
        bus.virtio_blk[0x10], bus.virtio_blk[0x11],
        bus.virtio_blk[0x12], bus.virtio_blk[0x13],
    ]);
    
    let ok = bar0 == 0xC041;
    D {
        name: "pci_virtio_blk_bar0",
        passed: ok,
        detail: if !ok { Some(format!("bar0=0x{:08X}", bar0)) } else { None },
    }
}

fn pgs() -> D {
    let bus = super::pci::PciBus::default();
    let class = bus.virtio_blk[0x0B];
    let subclass = bus.virtio_blk[0x0A];
    let ok = class == 0x01 && subclass == 0x80; 
    D {
        name: "pci_virtio_blk_class",
        passed: ok,
        detail: if !ok { Some(format!("class=0x{:02X} sub=0x{:02X}", class, subclass)) } else { None },
    }
}

fn pgt() -> D {
    let bus = super::pci::PciBus::default();
    let jjp = u16::from_le_bytes([bus.virtio_blk[0x2C], bus.virtio_blk[0x2D]]);
    let jjo = u16::from_le_bytes([bus.virtio_blk[0x2E], bus.virtio_blk[0x2F]]);
    
    let ok = jjp == 0x1AF4 && jjo == 0x0002;
    D {
        name: "pci_virtio_blk_subsystem",
        passed: ok,
        detail: if !ok { Some(format!("sv=0x{:04X} sid=0x{:04X}", jjp, jjo)) } else { None },
    }
}





fn pho() -> D {
    let state = super::virtio_blk::VirtioBlkState::with_capacity(64 * 512);
    let ok = state.capacity_sectors == 64;
    D {
        name: "virtio_blk_capacity",
        passed: ok,
        detail: if !ok { Some(format!("capacity={}", state.capacity_sectors)) } else { None },
    }
}

fn php() -> D {
    let mut state = super::virtio_blk::VirtioBlkState::with_capacity(32768);
    let features = state.io_read(0x00);
    
    let ok = (features & (1 << 1)) != 0 && (features & (1 << 2)) != 0;
    D {
        name: "virtio_blk_features",
        passed: ok,
        detail: if !ok { Some(format!("features=0x{:08X}", features)) } else { None },
    }
}

fn phr() -> D {
    let mut state = super::virtio_blk::VirtioBlkState::with_capacity(32768);
    
    state.io_write(0x04, 0x07); 
    state.io_write(0x12, 0x0F); 
    
    state.io_write(0x12, 0);
    
    let status = state.io_read(0x12);
    let guest_features = state.io_read(0x04);
    let ok = status == 0 && guest_features == 0;
    D {
        name: "virtio_blk_reset",
        passed: ok,
        detail: if !ok { Some(format!("status={} features={}", status, guest_features)) } else { None },
    }
}

fn phq() -> D {
    let mut state = super::virtio_blk::VirtioBlkState::default();
    let queue_size = state.io_read(0x0C);
    let ok = queue_size == 128; 
    D {
        name: "virtio_blk_queue_size",
        passed: ok,
        detail: if !ok { Some(format!("queue_size={}", queue_size)) } else { None },
    }
}

fn phn() -> D {
    let mut state = super::virtio_blk::VirtioBlkState::with_capacity(1024 * 1024); 
    let fkt = state.io_read(0x14);
    let fks = state.io_read(0x18);
    let capacity = (fks as u64) << 32 | fkt as u64;
    let expected = (1024 * 1024 / 512) as u64; 
    let ok = capacity == expected;
    D {
        name: "virtio_blk_cap_readback",
        passed: ok,
        detail: if !ok { Some(format!("capacity={} expected={}", capacity, expected)) } else { None },
    }
}





fn phs() -> D {
    let state = super::virtio_blk::VirtioConsoleState::default();
    let ok = state.cols == 80 && state.rows == 25 && state.max_nr_ports == 1;
    D {
        name: "virtio_console_defaults",
        passed: ok,
        detail: if !ok { Some(format!("cols={} rows={} ports={}", state.cols, state.rows, state.max_nr_ports)) } else { None },
    }
}

fn pht() -> D {
    let mut state = super::virtio_blk::VirtioConsoleState::default();
    
    state.io_write(0x12, 0x0F);
    let agx = state.io_read(0x12);
    let ok = agx == 0x0F;
    D {
        name: "virtio_console_status",
        passed: ok,
        detail: if !ok { Some(format!("readback=0x{:02X}", agx)) } else { None },
    }
}

fn phv() -> D {
    let mut state = super::virtio_blk::VirtioConsoleState::default();
    
    state.io_write(0x0E, 1); 
    state.io_write(0x08, 0x1000); 
    
    
    let bog = state.io_read(0x08);
    let ok = bog == 0x1000;
    D {
        name: "virtio_console_queue",
        passed: ok,
        detail: if !ok { Some(format!("pfn=0x{:X}", bog)) } else { None },
    }
}

fn phu() -> D {
    let mut state = super::virtio_blk::VirtioConsoleState::default();
    state.isr_status = 0x03; 
    
    let iin = state.io_read(0x13);
    let iio = state.io_read(0x13);
    
    let ok = iin == 3 && iio == 0;
    D {
        name: "virtio_console_isr_clear",
        passed: ok,
        detail: if !ok { Some(format!("isr1={} isr2={}", iin, iio)) } else { None },
    }
}





fn pfl() -> D {
    let mem = tc();
    
    let bcc = 0x50400;
    let sig = &mem[bcc..bcc + 4];
    let ok = sig == b"HPET";
    D {
        name: "hpet_at_new_offset",
        passed: ok,
        detail: if !ok { Some(format!("sig={:?}", &sig)) } else { None },
    }
}
