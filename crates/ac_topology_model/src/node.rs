use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Cpu,
    Gpu,
    ArVr,
    Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub cluster_id: String,
    pub kind: NodeKind,
    pub capacity_score: u32,
    pub eco_cost_score: u32,
}

impl Node {
    pub fn new(cluster_id: &str, kind: NodeKind) -> Self {
        Self {
            id: NodeId::new(),
            cluster_id: cluster_id.to_string(),
            kind,
            capacity_score: 100,
            eco_cost_score: 10,
        }
    }
}
