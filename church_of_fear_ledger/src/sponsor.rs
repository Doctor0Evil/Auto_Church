use serde::{Deserialize, Serialize};

/// Eco-grant proposal – attach to context_json of a deed to sponsor real NPO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoGrantProposal {
    pub recipient: String,          // NPO name / wallet
    pub amount_usd_equiv: f64,
    pub proof_hash: String,         // IPFS / Arweave CID of receipt
    pub purpose: String,            // "homelessness_relief", "reforestation", etc.
}

pub struct SponsorDistributor {
    /// In real deployment this would be a multisig + on-chain treasury
    pub available_pwr: u64,
}

impl SponsorDistributor {
    pub fn new() -> Self { Self { available_pwr: 1_000_000 } }

    pub fn propose_grant(&mut self, recipient: String, amount_usd_equiv: f64, proof_hash: String) -> EcoGrantProposal {
        EcoGrantProposal {
            recipient,
            amount_usd_equiv,
            proof_hash,
            purpose: "ecological_sustainability".to_string(),
        }
    }
}
