use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageId(pub String);

impl LineageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageRecord {
    pub id: LineageId,
    pub pattern_name: String,
    pub source_text: String,
    pub matched: bool,
}

impl LineageRecord {
    pub fn new(pattern_name: &str, source_text: &str, matched: bool) -> Self {
        Self {
            id: LineageId::new(),
            pattern_name: pattern_name.to_string(),
            source_text: source_text.to_string(),
            matched,
        }
    }
}
