use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterId(pub String);

impl ClusterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterRole {
    Master,
    Worker,
    Validator,
    Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: ClusterId,
    pub name: String,
    pub role: ClusterRole,
    pub eco_profile: String,
    pub ethical_profile: String,
}

impl Cluster {
    pub fn new(name: &str, role: ClusterRole) -> Self {
        Self {
            id: ClusterId::new(),
            name: name.to_string(),
            role,
            eco_profile: "eco_friendly".to_string(),
            ethical_profile: "harm_aware_safe".to_string(),
        }
    }
}
