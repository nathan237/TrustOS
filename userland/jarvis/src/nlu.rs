//! Natural Language Understanding

pub enum Intent {
    Execute,
    Query,
    Configure,
    Unknown,
}

pub struct NluResult {
    pub intent: Intent,
    pub confidence: u8,
}

pub fn parse(_input: &str) -> NluResult {
    NluResult {
        intent: Intent::Unknown,
        confidence: 0,
    }
}
