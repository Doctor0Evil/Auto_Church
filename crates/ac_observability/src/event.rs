use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventId(pub String);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    JobStarted,
    JobFinished,
    PolicyViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub kind: EventKind,
    pub description: String,
}

impl Event {
    pub fn new(kind: EventKind, description: &str) -> Self {
        Self {
            id: EventId::new(),
            kind,
            description: description.to_string(),
        }
    }
}
