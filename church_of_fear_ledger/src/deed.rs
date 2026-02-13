use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use zeroize::Zeroize;

/// Exact DeedEvent schema from the Church-of-FEAR moral ledger specification
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct DeedEvent {
    pub event_id: Uuid,
    pub timestamp: i64,                     // Unix epoch seconds
    pub prev_hash: String,                  // hex-encoded SHA-256
    pub self_hash: String,                  // hex-encoded SHA-256 (commitment)
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub deed_type: String,                  // e.g. "ecological_sustainability"
    pub tags: Vec<String>,
    pub context_json: serde_json::Value,    // evidence, URLs, grant proposals
    pub ethics_flags: Vec<String>,          // RoH breaches, ALN violations
    pub life_harm_flag: bool,
}

impl DeedEvent {
    pub fn new(
        actor_id: String,
        target_ids: Vec<String>,
        deed_type: String,
        tags: Vec<String>,
        context_json: serde_json::Value,
    ) -> Self {
        let event_id = Uuid::new_v4();
        let timestamp = Utc::now().timestamp();
        Self {
            event_id,
            timestamp,
            prev_hash: "".to_string(),
            self_hash: "".to_string(),
            actor_id,
            target_ids,
            deed_type,
            tags,
            context_json,
            ethics_flags: vec![],
            life_harm_flag: false,
        }
    }

    /// Convenience constructors – these are the deeds that earn CHURCH recommendations
    pub fn new_ecological_sustainability(actor_id: String, evidence_url: String) -> Self {
        let mut ctx = serde_json::json!({ "evidence_url": evidence_url });
        Self::new(
            actor_id,
            vec![],
            "ecological_sustainability".to_string(),
            vec!["reforestation".to_string(), "carbon_negative".to_string()],
            ctx,
        )
    }

    pub fn new_math_science_education(actor_id: String, crate_name: String) -> Self {
        let ctx = serde_json::json!({ "crate": crate_name, "license": "MIT/Apache-2.0" });
        Self::new(
            actor_id,
            vec![],
            "math_science_education".to_string(),
            vec!["open_source".to_string(), "rust".to_string()],
            ctx,
        )
    }

    /// Finalize hash chain – called by ledger after prev_hash is known
    pub fn finalize_hash_chain(mut self, prev_hash: String) -> Self {
        self.prev_hash = prev_hash;
        self.self_hash = self.compute_self_hash();
        self
    }

    pub fn compute_self_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let serialized = serde_json::to_string(self).expect("serialization infallible for owned data");
        hasher.update(serialized.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// CHURCH recommendation – advisory only, never automatic mint
    pub fn church_recommendation(&self) -> u64 {
        if self.life_harm_flag {
            return 0;
        }
        if !self.ethics_flags.is_empty() {
            return 0;
        }
        match self.deed_type.as_str() {
            "ecological_sustainability" | "homelessness_relief" | "math_science_education" => CHURCH_RECOMMEND_PER_GOOD_DEED,
            _ => 0,
        }
    }
}
