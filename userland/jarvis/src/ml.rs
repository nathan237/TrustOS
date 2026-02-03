//! ML inference engine

pub struct Model {
    weights: u64,
}

impl Model {
    pub fn new() -> Self {
        Self { weights: 0 }
    }
    
    pub fn infer(&self, _input: &[u8]) -> u64 {
        0
    }
}
