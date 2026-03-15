




pub fn oam(feature: Feature) -> bool {
    let dr = super::bme();
    
    match feature {
        Feature::Anv => dr.map(|r| r.tsc).unwrap_or(false),
        Feature::Anw => dr.map(|r| r.fan).unwrap_or(false),
        Feature::Uc => dr.map(|r| r.fsd).unwrap_or(false),
        Feature::Amy => dr.map(|r| r.eiw).unwrap_or(false),
        Feature::Qu => dr.map(|r| r.eix).unwrap_or(false),
        Feature::Amz => dr.map(|r| r.fvj).unwrap_or(false),
        Feature::Anc => dr.map(|r| r.fvl).unwrap_or(false),
        Feature::Ana => dr.map(|r| r.fvk).unwrap_or(false),
        Feature::Anb => dr.map(|r| r.eyy).unwrap_or(false),
        Feature::Agk => dr.map(|r| r.dof).unwrap_or(false),
        Feature::Ow => dr.map(|r| r.dog).unwrap_or(false),
        Feature::Agl => dr.map(|r| r.eml).unwrap_or(false),
        Feature::Asm => dr.map(|r| r.hka).unwrap_or(false),
        Feature::Agb => dr.map(|r| r.doa).unwrap_or(false),
        Feature::Alb => dr.map(|r| r.ewm).unwrap_or(false),
        Feature::Amp => dr.map(|r| r.eyl).unwrap_or(false),
        Feature::Alv => dr.map(|r| r.cbg).unwrap_or(false),
        Feature::Alw => dr.map(|r| r.cmc).unwrap_or(false),
        Feature::Aoc => dr.map(|r| r.vmx).unwrap_or(false),
        Feature::Anf => dr.map(|r| r.svm).unwrap_or(false),
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Feature {
    
    Anv,
    Anw,
    Uc,
    
    
    Amy,
    Qu,
    Amz,
    Anc,
    Ana,
    Anb,
    Agk,
    Ow,
    Agl,
    Asm,
    
    
    Agb,
    Alb,
    Amp,
    Alv,
    Alw,
    
    
    Aoc,
    Anf,
}

impl core::fmt::Display for Feature {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Feature::Anv => write!(bb, "TSC"),
            Feature::Anw => write!(bb, "Invariant TSC"),
            Feature::Uc => write!(bb, "RDTSCP"),
            Feature::Amy => write!(bb, "SSE"),
            Feature::Qu => write!(bb, "SSE2"),
            Feature::Amz => write!(bb, "SSE3"),
            Feature::Anc => write!(bb, "SSSE3"),
            Feature::Ana => write!(bb, "SSE4.1"),
            Feature::Anb => write!(bb, "SSE4.2"),
            Feature::Agk => write!(bb, "AVX"),
            Feature::Ow => write!(bb, "AVX2"),
            Feature::Agl => write!(bb, "AVX-512F"),
            Feature::Asm => write!(bb, "FMA"),
            Feature::Agb => write!(bb, "AES-NI"),
            Feature::Alb => write!(bb, "PCLMULQDQ"),
            Feature::Amp => write!(bb, "SHA-NI"),
            Feature::Alv => write!(bb, "RDRAND"),
            Feature::Alw => write!(bb, "RDSEED"),
            Feature::Aoc => write!(bb, "VMX"),
            Feature::Anf => write!(bb, "SVM"),
        }
    }
}


pub fn vlg() {
    let features = [
        Feature::Anv,
        Feature::Anw,
        Feature::Uc,
        Feature::Amy,
        Feature::Qu,
        Feature::Amz,
        Feature::Anc,
        Feature::Ana,
        Feature::Anb,
        Feature::Agk,
        Feature::Ow,
        Feature::Agl,
        Feature::Asm,
        Feature::Agb,
        Feature::Alb,
        Feature::Amp,
        Feature::Alv,
        Feature::Alw,
        Feature::Aoc,
        Feature::Anf,
    ];
    
    crate::println!("CPU Features:");
    for feature in features.iter() {
        let status = if oam(*feature) { "✓" } else { "✗" };
        crate::println!("  {} {}", status, feature);
    }
}
