use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioloadMetrics {
    pub bioload_delta: f64,
    pub roh: f64,
    pub decay: f64,
}

impl BioloadMetrics {
    pub fn new(bioload_delta: f64, roh: f64, decay: f64) -> Self {
        Self {
            bioload_delta,
            roh,
            decay,
        }
    }

    pub fn is_positive(&self) -> bool {
        self.bioload_delta < 0.0 && self.roh <= 0.3 && self.decay <= 1.0
    }
}
