// crates/ac_topology_model/src/branch.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchKind {
    Regex,
    Codex,
    System,
    Language,
    Devops,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub kind: BranchKind,
    pub description: String,
}

impl Branch {
    pub fn new(name: &str, kind: BranchKind, description: &str) -> Self {
        Self {
            name: name.to_string(),
            kind,
            description: description.to_string(),
        }
    }
}
