//! Church-of-FEAR Moral Ledger – Rights-Respecting, Non-Actuating Observer Layer
//! 
//! This crate implements the exact DeedEvent schema from "Charting the Moral Ledger"
//! plus full append-only, hash-chained integrity guarantees.
//! 
//! Good deeds (ecological_sustainability, homelessness_relief, math_science_education, etc.)
//! with life_harm_flag = false and empty ethics_flags earn CHURCH token recommendations.
//! The ledger itself NEVER mints tokens automatically – it only provides verifiable evidence
//! for Auto_Church validators and the Neuromorph-GOD composite to decide.
//! 
//! Use this ledger to sponsor real NPO projects (homelessness relief, reforestation,
//! open-source Rust science libraries) by attaching grant proposals as context_json.

pub mod deed;
pub mod ledger;
pub mod validator;
pub mod sponsor;

pub use deed::DeedEvent;
pub use ledger::MoralLedger;
pub use validator::{ValidationError, LedgerValidator};
pub use sponsor::{EcoGrantProposal, SponsorDistributor};

/// Global constant – CHURCH token recommendation per verified good deed (advisory only)
pub const CHURCH_RECOMMEND_PER_GOOD_DEED: u64 = 1;

/// Short-abbreviation functions for CHURCH earning (real-world usable)
pub mod church {
    use super::*;
    
    /// NANO-1: Log a verified ecological cleanup deed → potential +1 CHURCH
    pub fn log_ecological_cleanup(ledger: &mut MoralLedger, actor_id: String, evidence_url: String) -> Result<uuid::Uuid, ValidationError> {
        let event = DeedEvent::new_ecological_sustainability(actor_id, evidence_url);
        ledger.append(event)
    }
    
    /// TECH-1: Contribute open-source Rust science crate → potential +2 CHURCH
    pub fn log_open_source_contribution(ledger: &mut MoralLedger, actor_id: String, crate_name: String) -> Result<uuid::Uuid, ValidationError> {
        let event = DeedEvent::new_math_science_education(actor_id, crate_name);
        ledger.append(event)
    }
    
    /// PWR-1: Sponsor homelessness-relief NPO with proof → potential +5 CHURCH
    pub fn propose_homelessness_grant(distributor: &mut SponsorDistributor, recipient: String, amount_usd_equiv: f64, proof_hash: String) -> EcoGrantProposal {
        distributor.propose_grant(recipient, amount_usd_equiv, proof_hash)
    }
}
