



























































pub mod probe;
pub mod analyzer;
pub mod media;
pub mod optimizer;
pub mod query;

use alloc::string::String;
use alloc::format;



pub fn boot() -> String {
    optimizer::kdk()
}


pub fn osd() -> String {
    if let Some(ai) = probe::cur() {
        ai.format_report()
    } else {
        String::from("\x01RJARVIS hardware scan not yet performed.\x01W\nRun: \x01Cjarvis boot\x01W\n")
    }
}


pub fn osa() -> String {
    if let Some(ai) = probe::cur() {
        let nu = analyzer::hfd(&ai);
        analyzer::lxm(&nu)
    } else {
        String::from("\x01RNo hardware data. Run: jarvis boot\x01W\n")
    }
}


pub fn osb() -> String {
    if let Some(vr) = optimizer::current_plan() {
        analyzer::hzp(&vr)
    } else if let Some(ai) = probe::cur() {
        let vr = analyzer::ibc(&ai);
        analyzer::hzp(&vr)
    } else {
        String::from("\x01RNo hardware data. Run: jarvis boot\x01W\n")
    }
}


pub fn nnq() -> String {
    if let Some(output) = optimizer::oje() {
        output
    } else {
        String::from("\x01ROptimizer not active. Run: jarvis boot\x01W\n")
    }
}


pub fn ose() -> String {
    optimizer::status()
}


pub fn jvw(data: &[u8]) -> String {
    let analysis = media::jvu(data);
    let mut output = analysis.format_report();

    
    let format = media::dmx(data);
    if matches!(format, media::BinaryFormat::Fu | media::BinaryFormat::Gpt) {
        let au = media::nqx(data);
        if !au.is_empty() {
            output.push_str(&media::lxp(&au));
        }
    }

    output
}



pub fn qkf() -> String {
    if let Some(ai) = probe::cur() {
        let mut ab = ai.to_ai_context();

        
        if let Some(vr) = optimizer::current_plan() {
            ab.push_str(&format!("PLAN: simd={} batch={} gpu={} workers={}\n",
                vr.simd_tier.as_str(), vr.optimal_batch_size,
                vr.use_gpu, vr.worker_threads));
        }

        
        if optimizer::is_active() {
            ab.push_str("OPTIMIZER: active, self-tuning enabled\n");
        }

        ab
    } else {
        String::from("HARDWARE: not yet scanned\n")
    }
}



pub fn mmt(gpf: &str) -> String {
    if let Some(ai) = probe::cur() {
        let result = query::jwi(gpf, &ai);
        query::lxq(&result)
    } else {
        String::from("\x01RHardware not scanned. Run: jarvis boot\x01W\n")
    }
}
