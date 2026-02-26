// Church-of-FEAR / Tree-of-Life Observer Ledger - Real-world rights-respecting moral accounting
// ALN-compliant, zero actuation, tamper-evident, earns CHURCH / POWER / TECH / NANO tokens
// Rust 1.85+, no unsafe, full Serde + SHA-256 chain

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use uuid::Uuid;

/// Core DeedEvent - immutable moral ledger row. Exactly matches the schema in the Moral Ledger PDF.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeedEvent {
    pub event_id: String,           // UUID v4
    pub timestamp: i64,             // Unix seconds (UTC)
    pub prev_hash: String,          // SHA-256 of previous self_hash (hex)
    pub self_hash: String,          // SHA-256 of canonical JSON (hex) - computed on creation
    pub actor_id: String,           // e.g., "user-xboxtj-2026"
    pub target_ids: Vec<String>,    // affected agents / beneficiaries
    pub deed_type: String,          // "ecological_sustainability", "homelessness_relief", "math_science_education"
    pub tags: Vec<String>,          // e.g., ["tree-of-life", "npo-funding", "rust-education"]
    pub context_json: Value,        // unstructured evidence, URIs, photos, measurements
    pub ethics_flags: Vec<String>,  // e.g., ["RoH_CEILING_EXCEEDED"] - diagnostics only
    pub life_harm_flag: bool,       // true if any living creature harmed - triggers strict review
}

impl DeedEvent {
    /// Canonical serialization for hashing (sorted keys, no self_hash to prevent circularity)
    fn canonical_json(&self) -> Result<String> {
        let mut map = serde_json::Map::new();
        map.insert("event_id".to_string(), serde_json::to_value(&self.event_id)?);
        map.insert("timestamp".to_string(), serde_json::to_value(&self.timestamp)?);
        map.insert("prev_hash".to_string(), serde_json::to_value(&self.prev_hash)?);
        map.insert("actor_id".to_string(), serde_json::to_value(&self.actor_id)?);
        map.insert("target_ids".to_string(), serde_json::to_value(&self.target_ids)?);
        map.insert("deed_type".to_string(), serde_json::to_value(&self.deed_type)?);
        map.insert("tags".to_string(), serde_json::to_value(&self.tags)?);
        map.insert("context_json".to_string(), self.context_json.clone());
        map.insert("ethics_flags".to_string(), serde_json::to_value(&self.ethics_flags)?);
        map.insert("life_harm_flag".to_string(), serde_json::to_value(&self.life_harm_flag)?);

        // Sorted keys for deterministic canonical form
        let mut sorted: Vec<_> = map.into_iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        let sorted_map: serde_json::Map<_, _> = sorted.into_iter().collect();
        Ok(serde_json::to_string(&sorted_map)?)
    }

    /// Compute self_hash from canonical JSON (SHA-256 hex)
    pub fn compute_self_hash(&self) -> Result<String> {
        let canonical = self.canonical_json()?;
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Create a new DeedEvent with correct self_hash (pure function)
    pub fn new(
        prev_hash: String,
        actor_id: String,
        target_ids: Vec<String>,
        deed_type: String,
        tags: Vec<String>,
        context_json: Value,
        ethics_flags: Vec<String>,
        life_harm_flag: bool,
    ) -> Result<Self> {
        let event_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp();

        let mut event = DeedEvent {
            event_id,
            timestamp,
            prev_hash,
            self_hash: String::new(), // placeholder
            actor_id,
            target_ids,
            deed_type,
            tags,
            context_json,
            ethics_flags,
            life_harm_flag,
        };

        event.self_hash = event.compute_self_hash()?;
        Ok(event)
    }

    /// Advisory moral position score (mp ∈ [0,1]) - real-world usable for NPO grant eligibility
    pub fn moral_position_score(&self) -> f64 {
        let base = if self.life_harm_flag { 0.0 } else { 0.85 };
        let ethics_penalty = self.ethics_flags.len() as f64 * 0.15;
        let good_tags = self.tags.iter().filter(|t| {
            matches!(t.as_str(), "ecological_sustainability" | "homelessness_relief" | "math_science_education")
        }).count() as f64 * 0.08;
        (base - ethics_penalty + good_tags).clamp(0.0, 1.0)
    }

    /// Advisory eco_grant recommendation in CHURCH-equivalent units (real NPO distribution logic)
    pub fn eco_grant_recommendation(&self) -> f64 {
        let mp = self.moral_position_score();
        let base_grant = match self.deed_type.as_str() {
            "ecological_sustainability" => 25.0,
            "homelessness_relief" => 30.0,
            "math_science_education" => 20.0,
            _ => 10.0,
        };
        base_grant * mp * (1.0 + self.tags.len() as f64 * 0.05)
    }
}

/// Append a new event to .church-ledger.jsonl and return the new self_hash
/// Pure observer - never touches capability or consent.
pub fn append_deed_event<P: AsRef<Path>>(
    ledger_path: P,
    actor_id: String,
    target_ids: Vec<String>,
    deed_type: String,
    tags: Vec<String>,
    context_json: Value,
    ethics_flags: Vec<String>,
    life_harm_flag: bool,
) -> Result<String> {
    let path = ledger_path.as_ref();
    let mut prev_hash = "0".repeat(64); // genesis

    if path.exists() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        if let Some(last_line) = reader.lines().last() {
            let last_event: DeedEvent = serde_json::from_str(&last_line?)?;
            prev_hash = last_event.self_hash;
        }
    }

    let event = DeedEvent::new(
        prev_hash,
        actor_id,
        target_ids,
        deed_type,
        tags,
        context_json,
        ethics_flags,
        life_harm_flag,
    )?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    writeln!(file, "{}", serde_json::to_string(&event)?)?;
    Ok(event.self_hash)
}

/// Validate entire ledger chain (real-world audit function)
pub fn validate_ledger<P: AsRef<Path>>(ledger_path: P) -> Result<bool> {
    let file = File::open(ledger_path)?;
    let reader = BufReader::new(file);
    let mut prev_hash = "0".repeat(64);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let event: DeedEvent = serde_json::from_str(&line)?;
        if event.prev_hash != prev_hash {
            return Ok(false);
        }
        let computed = event.compute_self_hash()?;
        if computed != event.self_hash {
            return Ok(false);
        }
        prev_hash = event.self_hash;
    }
    Ok(true)
}

// Short-abbreviation system objects for CHURCH/POWER/TECH earning (real-world reusable)
pub fn mp_score(deed: &DeedEvent) -> f64 { deed.moral_position_score() }
pub fn eco_grant(deed: &DeedEvent) -> f64 { deed.eco_grant_recommendation() }
pub fn nano_stable_hash(event: &DeedEvent) -> String { event.self_hash.clone() } // for bchainproof integration

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_append_and_validate() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let hash1 = append_deed_event(
            tmp.path(),
            "user-xboxtj".to_string(),
            vec!["eco-project-1".to_string()],
            "ecological_sustainability".to_string(),
            vec!["tree-of-life".to_string()],
            serde_json::json!({"project": "arizona-desert-restoration", "volunteers": 12}),
            vec![],
            false,
        ).unwrap();
        assert!(!hash1.is_empty());
        assert!(validate_ledger(tmp.path()).unwrap());
    }
}
