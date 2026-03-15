
















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::jarvis_hw::probe::T;



pub fn cks(cc: &T) -> Vec<String> {
    let mut bwy = Vec::fc(120);

    rpx(cc, &mut bwy);
    una(cc, &mut bwy);
    vfu(cc, &mut bwy);
    wuu(cc, &mut bwy);
    slk(cc, &mut bwy);
    ush(cc, &mut bwy);
    thf(cc, &mut bwy);
    xph(cc, &mut bwy);
    xhk(cc, &mut bwy);
    wgi(cc, &mut bwy);
    qew(cc, &mut bwy);
    wek(cc, &mut bwy);
    vsx(cc, &mut bwy);

    bwy
}





fn rpx(ai: &T, e: &mut Vec<String>) {
    
    e.push(format!("Q: What CPU is installed? A: {}", ai.dpf));
    e.push(format!("Q: Quel processeur? A: {}", ai.dpf));
    e.push(format!("Q: CPU vendor? A: {}", ai.avo));
    e.push(format!("Q: How many CPU cores? A: {} logical, {} physical",
        ai.cau, ai.djk));
    e.push(format!("Q: Combien de coeurs? A: {} logiques, {} physiques",
        ai.cau, ai.djk));
    e.push(format!("Q: CPU family/model? A: family {} model {} stepping {}",
        ai.heb, ai.hec, ai.hee));

    
    if ai.fam > 0 {
        let one = ai.fam / 1_000_000;
        e.push(format!("Q: CPU frequency? A: ~{} MHz (TSC)", one));
        e.push(format!("Q: Quelle frequence CPU? A: ~{} MHz", one));
    }

    
    let simd = wom(ai);
    e.push(format!("Q: What SIMD is available? A: {}", simd));
    e.push(format!("Q: SIMD disponible? A: {}", simd));

    
    if ai.bzx {
        e.push(format!("My SIMD kernels use AVX2+FMA: 8-wide float ops, 2-3x faster than SSE2."));
    } else {
        e.push(format!("My SIMD kernels use SSE2: 4-wide float ops. No AVX2 available."));
    }

    
    let crypto = rqz(ai);
    if !crypto.is_empty() {
        e.push(format!("Q: Hardware crypto? A: {}", crypto));
    }

    
    if ai.giw {
        e.push(format!("Q: Virtualization? A: Intel VT-x supported"));
    }
    if ai.giu {
        e.push(format!("Q: Virtualization? A: AMD-V supported"));
    }
}

fn wom(ai: &T) -> String {
    let mut ek = Vec::new();
    if ai.ixu { ek.push("SSE"); }
    if ai.dro { ek.push("SSE2"); }
    if ai.ixv { ek.push("SSE3"); }
    if ai.hmq { ek.push("SSE4.2"); }
    if ai.fke { ek.push("AVX"); }
    if ai.bzx { ek.push("AVX2"); }
    if ai.drm { ek.push("AVX-512"); }
    if ek.is_empty() { return String::from("none"); }
    let mut e = String::new();
    for (a, vu) in ek.iter().cf() {
        if a > 0 { e.t(", "); }
        e.t(vu);
    }
    e
}

fn rqz(ai: &T) -> String {
    let mut ek = Vec::new();
    if ai.cfe { ek.push("AES-NI"); }
    if ai.ecm { ek.push("SHA-NI"); }
    if ai.crd { ek.push("RDRAND"); }
    if ai.fkh { ek.push("RDSEED"); }
    let mut e = String::new();
    for (a, vu) in ek.iter().cf() {
        if a > 0 { e.t(", "); }
        e.t(vu);
    }
    e
}





fn una(ai: &T, e: &mut Vec<String>) {
    let amo = ai.ccf / (1024 * 1024);
    e.push(format!("Q: How much RAM? A: {} MB", amo));
    e.push(format!("Q: Combien de RAM? A: {} MB", amo));

    let toc = ai.drr / 1024;
    let jvb = ai.ecw / 1024;
    let kxc = ai.erx / 1024;
    e.push(format!("Q: Heap status? A: {} KB total, {} KB used, {} KB free",
        toc, jvb, kxc));

    e.push(format!("Q: Physical frames? A: {} used, {} free",
        ai.ceu, ai.dhj));

    
    let agc = ai.ceu + ai.dhj;
    if agc > 0 {
        let iga = (ai.ceu * 100) / agc;
        if iga > 80 {
            e.push(format!("Memory pressure is high: {}% of frames in use. Be conservative with allocations.", iga));
        } else if iga < 20 {
            e.push(format!("Memory is abundant: only {}% in use. Safe to allocate large buffers.", iga));
        } else {
            e.push(format!("Memory usage is moderate: {}% of frames in use.", iga));
        }
    }

    if amo <= 64 {
        e.push(String::from("This is a low-memory system. I should minimize allocations."));
    } else if amo >= 1024 {
        e.push(String::from("Plenty of RAM available. Can use larger model buffers."));
    }
}





fn vfu(ai: &T, e: &mut Vec<String>) {
    e.push(format!("Q: How many PCI devices? A: {}", ai.dal));

    if ai.ewl > 0 {
        e.push(format!("Q: Storage controllers? A: {} PCI storage controller(s)",
            ai.ewl));
    }
    if ai.egg > 0 {
        e.push(format!("Q: Network adapters? A: {} PCI network controller(s)",
            ai.egg));
    }
    if ai.ewk > 0 {
        e.push(format!("Q: Display adapters? A: {} PCI display controller(s)",
            ai.ewk));
    }
    if ai.egh > 0 {
        e.push(format!("Q: USB controllers? A: {} PCI USB controller(s)",
            ai.egh));
    }
    if ai.egf > 0 {
        e.push(format!("Q: Audio devices? A: {} PCI audio controller(s)",
            ai.egf));
    }

    
    for (a, ba) in ai.hus.iter().take(5).cf() {
        e.push(format!("PCI device {}: {} {} at {:02x}:{:02x}.{} (vendor {:04x}, device {:04x})",
            a, ba.bpz, ba.bor,
            ba.aq, ba.de, ba.gw,
            ba.ml, ba.mx));
    }
}





fn wuu(ai: &T, e: &mut Vec<String>) {
    if ai.aqm.is_empty() {
        e.push(String::from("Q: Any storage? A: No block storage devices detected."));
        return;
    }

    let xke = ai.dmp / (1024 * 1024 * 1024);
    e.push(format!("Q: Total storage? A: {} GB across {} device(s)",
        xke, ai.aqm.len()));

    for ba in &ai.aqm {
        let qwb = ba.fei / (1024 * 1024);
        let kk = match ba.kk {
            crate::jarvis_hw::probe::StorageKind::Xv => "NVMe",
            crate::jarvis_hw::probe::StorageKind::Qr => "SATA",
            crate::jarvis_hw::probe::StorageKind::Bjk => "IDE",
            crate::jarvis_hw::probe::StorageKind::F => "unknown",
        };
        e.push(format!("Storage: {} ({}) — {} MB", ba.j, kk, qwb));
    }

    
    for vu in &ai.aqd {
        let aga = vu.afz / (1024 * 1024);
        e.push(format!("Partition: {} #{} — {} MB, type: {}{}",
            vu.app, vu.aqb, aga, vu.ddc,
            if vu.cji { " (bootable)" } else { "" }));
    }
}





fn slk(ai: &T, e: &mut Vec<String>) {
    if ai.avs.is_empty() {
        e.push(String::from("Q: Encrypted disks? A: No disk encryption detected."));
        return;
    }

    e.push(format!("Q: Encrypted disks? A: {} encrypted volume(s) detected.",
        ai.avs.len()));

    for bdy in &ai.avs {
        let iss = match bdy.ckf {
            crate::jarvis_hw::probe::EncryptionType::Ajy => "LUKS1",
            crate::jarvis_hw::probe::EncryptionType::Ajz => "LUKS2",
            crate::jarvis_hw::probe::EncryptionType::Aaa => "BitLocker",
            crate::jarvis_hw::probe::EncryptionType::Afn => "VeraCrypt",
            crate::jarvis_hw::probe::EncryptionType::Bgz => "FileVault2",
            crate::jarvis_hw::probe::EncryptionType::Bes => "dm-crypt",
            crate::jarvis_hw::probe::EncryptionType::Bnu => "Opal SED",
            crate::jarvis_hw::probe::EncryptionType::F => "unknown",
        };
        e.push(format!("Encryption: {} on {} — {}", iss, bdy.app, bdy.eu));
    }

    
    e.push(String::from("To access encrypted data I need the user to provide a passphrase or key."));
}





fn ush(ai: &T, e: &mut Vec<String>) {
    if !ai.bzz {
        e.push(String::from("Q: Network available? A: No network interface detected."));
        return;
    }

    let ojj = if ai.aik { "up" } else { "down" };
    e.push(format!("Q: Network status? A: Interface detected, link {}", ojj));
    e.push(format!("Q: Reseau disponible? A: Interface detectee, lien {}", ojj));

    if let Some(ed) = ai.csg {
        e.push(format!("Q: MAC address? A: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]));
    }
}





fn thf(ai: &T, e: &mut Vec<String>) {
    if !ai.bqz {
        e.push(String::from("Q: GPU available? A: No dedicated GPU. Using CPU SIMD for compute."));
        e.push(String::from("Without a GPU, my neural compute runs on CPU SIMD (SSE2 or AVX2)."));
        return;
    }

    e.push(format!("Q: What GPU? A: {} ({} MB VRAM, {} CUs)",
        ai.beh, ai.dhr, ai.erk));
    e.push(format!("Q: Quel GPU? A: {} ({} MB VRAM)", ai.beh, ai.dhr));

    if ai.dhr >= 256 {
        e.push(format!("GPU has enough VRAM for neural weights. Can offload matmul."));
    }
}





fn xph(ai: &T, e: &mut Vec<String>) {
    if !ai.fav || ai.cvc.is_empty() {
        e.push(String::from("Q: USB devices? A: No USB devices connected."));
        return;
    }

    e.push(format!("Q: USB devices? A: {} device(s) on {} controller(s)",
        ai.cvc.len(), ai.fxv));

    for ba in ai.cvc.iter().take(4) {
        e.push(format!("USB: {} ({}) at address {}",
            ba.baj, ba.bpz, ba.re));
    }
}





fn xhk(ai: &T, e: &mut Vec<String>) {
    let mut axe = Vec::new();
    if ai.juh { axe.push("TSC"); }
    if ai.esa { axe.push("HPET"); }

    if !axe.is_empty() {
        let mut mky = String::new();
        for (a, ab) in axe.iter().cf() {
            if a > 0 { mky.t(", "); }
            mky.t(ab);
        }
        e.push(format!("Q: Available timers? A: {}", mky));
    }

    if ai.esa && ai.gjf > 0 {
        e.push(format!("HPET: {} timers, {}bit, vendor 0x{:04x}",
            ai.gjf,
            if ai.iys { "64" } else { "32" },
            ai.iyt));
    }
}





fn wgi(ai: &T, e: &mut Vec<String>) {
    let mut features = Vec::new();
    if ai.erv { features.push("SMEP"); }
    if ai.eru { features.push("SMAP"); }
    if ai.ert { features.push("NX/XD"); }
    if ai.giv { features.push("UMIP"); }

    if !features.is_empty() {
        let mut iue = String::new();
        for (a, bb) in features.iter().cf() {
            if a > 0 { iue.t(", "); }
            iue.t(bb);
        }
        e.push(format!("Q: Security features? A: {}", iue));
        e.push(format!("Q: Protection hardware? A: {}", iue));
    }

    if ai.erv && ai.eru && ai.ert {
        e.push(String::from("Full hardware exploit mitigations active: SMEP+SMAP+NX."));
    }
}





fn qew(ai: &T, e: &mut Vec<String>) {
    e.push(format!("Q: ACPI revision? A: {} (OEM: {})",
        ai.gxt, ai.gxs));

    if ai.hiu {
        e.push(String::from("ACPI hardware-reduced mode is active."));
    }

    if ai.edq > 0 {
        e.push(format!("Q: Interrupt controllers? A: {} I/O APIC(s), {} CPU LAPIC(s)",
            ai.edq, ai.gae.len()));
    }

    if ai.fqo && !ai.gpa.is_empty() {
        e.push(format!("Q: PCIe segments? A: {}", ai.gpa.len()));
    }
}





fn wek(ai: &T, e: &mut Vec<String>) {
    e.push(format!("Q: System overall score? A: {:.0}%", ai.dkj * 100.0));
    e.push(format!("Compute score: {:.0}%, Memory: {:.0}%, Storage: {:.0}%, Network: {:.0}%, Security: {:.0}%",
        ai.cwl * 100.0, ai.dte * 100.0,
        ai.ezb * 100.0, ai.evg * 100.0,
        ai.eyh * 100.0));

    
    if ai.cwl >= 0.7 {
        e.push(String::from("This system has strong compute capability. Complex inference is feasible."));
    } else if ai.cwl < 0.3 {
        e.push(String::from("Compute resources are limited. Keep inference sequences short."));
    }

    if ai.dte >= 0.7 {
        e.push(String::from("Memory is plentiful. Can cache activations and KV values."));
    } else if ai.dte < 0.3 {
        e.push(String::from("Memory is tight. Must be careful with sequence lengths and caching."));
    }
}





fn vsx(ai: &T, e: &mut Vec<String>) {
    let amo = ai.ccf / (1024 * 1024);

    
    if amo >= 512 {
        e.push(format!(
            "I have {} MB RAM. My weights are ~18 MB. I have room for larger batch accumulation.",
            amo));
    } else if amo >= 64 {
        e.push(format!(
            "I have {} MB RAM. My weights are ~18 MB plus optimizer state ~36 MB. Memory is sufficient but not abundant.",
            amo));
    } else {
        e.push(format!(
            "I have only {} MB RAM. Must minimize memory usage. Consider smaller model.",
            amo));
    }

    
    if ai.bzx {
        e.push(String::from(
            "AVX2 is available. My matmul processes 32 floats per iteration with FMA. Training is 2-3x faster than SSE2."));
    } else if ai.dro {
        e.push(String::from(
            "Only SSE2 is available. My matmul processes 16 floats per iteration. Training works but is slower."));
    }

    
    if ai.bqz && ai.dhr >= 64 {
        e.push(format!(
            "GPU detected with {} MB VRAM. Could offload weight matrices for 10-50x matmul speedup.",
            ai.dhr));
    } else {
        e.push(String::from(
            "No GPU available. All neural compute runs on CPU SIMD. Focus on efficient kernels."));
    }

    
    if !ai.avs.is_empty() {
        e.push(String::from(
            "Encrypted volumes detected. I cannot access that data without the user providing credentials. This protects privacy."));
    }

    
    if ai.cau > 1 {
        e.push(format!(
            "System has {} logical CPUs. Could parallelize multi-head attention across cores in the future.",
            ai.cau));
    } else {
        e.push(String::from(
            "Single CPU core. All computation is sequential. Kernel optimization matters most."));
    }

    
    e.push(format!("I am running on {} in {} mode.", ai.arch, ai.jjz));

    
    e.push(format!(
        "System summary: {} cores, {} MB RAM, {} PCI devices, {} storage, {}GPU, {}net.",
        ai.cau, amo, ai.dal,
        ai.aqm.len(),
        if ai.bqz { "" } else { "no " },
        if ai.bzz { "" } else { "no " }));
}
