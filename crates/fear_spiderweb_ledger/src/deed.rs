use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeedEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub prev_hash: String,
    pub self_hash: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub deed_type: String,
    pub tags: Vec<String>,
    pub context_json: serde_json::Value,
    pub ethics_flags: Vec<String>,
    pub life_harm_flag: bool,
    // Tree-of-Life projections (computed or ingested)
    pub fear_level: f32,      // [0,1]
    pub pain_level: f32,
    pub decay: f32,
    pub lifeforce: f32,
    pub calm_stable: bool,
    pub overloaded: bool,
    pub recovery: bool,
    pub unfair_drain: bool,
}

impl DeedEvent {
    pub fn compute_self_hash(&self) -> String {
        // Canonical JSON + SHA-256 (mirrors .donutloop.aln)
        let canonical = serde_json::to_string(&self).unwrap(); // fixed order in prod
        let mut hasher = sha2::Sha256::new();
        hasher.update(canonical.as_bytes());
        hex::encode(hasher.finalize())
    }
}
