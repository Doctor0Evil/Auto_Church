//! Church-of-FEAR / Tree-of-Life 1D Biophysical Microspace Rights Observer
//! Non-actuating, hash-linked, observer-only diagnostics for rights, freedoms,
//! and NATURE-respecting zones. Generates DeedEvent logs and CHURCH recommendations.
//! Zero control surfaces – pure read/compute/log. Saves Earth by modeling fair
//! resource zones that prevent unfair drain (analogous to ecological or social
//! sustainability simulations).

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

const TREE_ASSETS: usize = 14; // BLOOD, OXYGEN, WAVE, DECAY, LIFEFORCE, FEAR, PAIN, NANO, POWER, TECH, SMART, EVOLVE, TIME, SPIRIT (simplified to 5 core for 1D)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeState {
    pub blood: f64,     // [0.0, 1.0]
    pub oxygen: f64,
    pub decay: f64,
    pub lifeforce: f64,
    pub fear: f64,
    pub pain: f64,
    // Extendable to full 14 via context_json
}

impl TreeState {
    pub fn clamp(&mut self) {
        self.blood = self.blood.clamp(0.0, 1.0);
        self.oxygen = self.oxygen.clamp(0.0, 1.0);
        self.decay = self.decay.clamp(0.0, 1.0);
        self.lifeforce = self.lifeforce.clamp(0.0, 1.0);
        self.fear = self.fear.clamp(0.0, 1.0);
        self.pain = self.pain.clamp(0.0, 1.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub role: String, // WORKER, CAREGIVER, REGULATOR, etc.
    pub position: usize, // 1D lattice index
    pub state: TreeState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeedEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub prev_hash: String,
    pub self_hash: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub deed_type: String, // "resource_sharing", "eco_sustainability", etc.
    pub tags: Vec<String>,
    pub context_json: serde_json::Value,
    pub ethics_flags: Vec<String>,
    pub life_harm_flag: bool,
}

impl DeedEvent {
    pub fn new(actor_id: String, deed_type: String, context: serde_json::Value) -> Self {
        let event_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp();
        let mut event = Self {
            event_id,
            timestamp,
            prev_hash: String::new(),
            self_hash: String::new(),
            actor_id,
            target_ids: vec![],
            deed_type,
            tags: vec!["rights_management".to_string(), "nature_respect".to_string()],
            context_json: context,
            ethics_flags: vec![],
            life_harm_flag: false,
        };
        event.self_hash = event.compute_hash();
        event
    }

    fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let canonical = serde_json::to_string(&self).unwrap(); // fixed order via serde
        hasher.update(canonical.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn link_to_prev(&mut self, prev_hash: String) {
        self.prev_hash = prev_hash;
        self.self_hash = self.compute_hash();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateFlags {
    pub calm_stable: bool,
    pub overloaded: bool,
    pub recovery: bool,
    pub unfair_drain: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneClassification {
    pub safe_zone_fraction: f64,
    pub stress_corridor_fraction: f64,
    pub rights_breach_fraction: f64,
    pub mp_score: f64, // moral position [0,1]
    pub church_recommendation: String, // "eco_grant: homelessness_relief" or similar
}

pub struct MicrospaceRightsObserver {
    pub lattice: Vec<Agent>,
    pub deed_log: Vec<DeedEvent>,
    pub current_hash: String,
}

impl MicrospaceRightsObserver {
    pub fn new(size: usize) -> Self {
        let mut lattice = Vec::with_capacity(size);
        for i in 0..size {
            let mut state = TreeState {
                blood: 0.8, oxygen: 0.8, decay: 0.2, lifeforce: 0.8, fear: 0.3, pain: 0.2,
            };
            state.clamp();
            lattice.push(Agent {
                id: format!("agent_{}", i),
                role: if i % 3 == 0 { "WORKER".to_string() } else if i % 5 == 0 { "CAREGIVER".to_string() } else { "REGULATOR".to_string() },
                position: i,
                state,
            });
        }
        Self {
            lattice,
            deed_log: Vec::new(),
            current_hash: "0".repeat(64),
        }
    }

    // Short-abbreviation real-world functions for CHURCH earning
    pub fn calc_cs(history: &[TreeState], window: usize) -> bool { // CALM_STABLE
        if history.len() < window { return false; }
        let recent = &history[history.len() - window..];
        let avg_stress: f64 = recent.iter().map(|s| s.fear.max(s.pain)).sum::<f64>() / window as f64;
        let avg_decay: f64 = recent.iter().map(|s| s.decay).sum::<f64>() / window as f64;
        let avg_energy: f64 = recent.iter().map(|s| s.lifeforce).sum::<f64>() / window as f64;
        avg_stress <= 0.35 && avg_decay <= 0.40 && avg_energy >= 0.65
    }

    pub fn calc_od(history: &[TreeState], window: usize, h: usize) -> bool { // OVERLOADED
        if history.len() < window + h { return false; }
        let recent = &history[history.len() - window..];
        let prev = &history[history.len() - window - h..history.len() - window];
        let avg_stress = recent.iter().map(|s| s.fear.max(s.pain)).sum::<f64>() / window as f64;
        let prev_stress = prev.iter().map(|s| s.fear.max(s.pain)).sum::<f64>() / h as f64;
        let delta_stress = avg_stress - prev_stress;
        let avg_decay = recent.iter().map(|s| s.decay).sum::<f64>() / window as f64;
        let prev_decay = prev.iter().map(|s| s.decay).sum::<f64>() / h as f64;
        let delta_decay = avg_decay - prev_decay;
        (avg_stress >= 0.65 && delta_stress >= 0.08) || (avg_decay >= 0.70 && delta_decay >= 0.06)
    }

    pub fn calc_rec(history: &[TreeState], w_rec: usize, h_rec: usize) -> bool { // RECOVERY
        // Simplified hysteresis check – full impl mirrors document math
        if history.len() < w_rec + h_rec { return false; }
        let recent = &history[history.len() - h_rec..];
        let delta_stress = recent.iter().map(|s| s.fear.max(s.pain)).sum::<f64>() / h_rec as f64 - 
                           history[history.len() - h_rec - 1].fear.max(history[history.len() - h_rec - 1].pain);
        let delta_decay = recent.iter().map(|s| s.decay).sum::<f64>() / h_rec as f64 - history[history.len() - h_rec - 1].decay;
        let delta_energy = recent.iter().map(|s| s.lifeforce).sum::<f64>() / h_rec as f64 - history[history.len() - h_rec - 1].lifeforce;
        delta_stress <= -0.05 && delta_decay <= -0.04 && delta_energy >= 0.06
    }

    pub fn calc_ud(peers: &[f64], self_budget: f64, overload_frac: f64) -> bool { // UNFAIRDRAIN
        if peers.is_empty() { return false; }
        let median = {
            let mut p = peers.to_vec();
            p.sort_by(|a, b| a.partial_cmp(b).unwrap());
            p[p.len() / 2]
        };
        self_budget <= 0.75 * median && overload_frac >= 0.6
    }

    pub fn step(&mut self, load_factor: f64) {
        // Simple 1D update rule (non-actuating sim for research)
        for agent in &mut self.lattice {
            let neighbor_influence = if agent.position > 0 { self.lattice[agent.position - 1].state.lifeforce * 0.1 } else { 0.0 }
                + if agent.position < self.lattice.len() - 1 { self.lattice[agent.position + 1].state.lifeforce * 0.1 } else { 0.0 };
            agent.state.lifeforce = (agent.state.lifeforce + neighbor_influence - load_factor * 0.05).clamp(0.0, 1.0);
            agent.state.decay = (agent.state.decay + load_factor * 0.08).clamp(0.0, 1.0);
            agent.state.fear = (agent.state.fear + load_factor * 0.03).clamp(0.0, 1.0);
            agent.state.clamp();
        }

        // Log a good-deed example (civic duty: fair sharing)
        let deed = DeedEvent::new(
            "system_observer".to_string(),
            "resource_sharing".to_string(),
            serde_json::json!({"fairness_improved": true, "eco_impact": "prevents_unfair_drain"}),
        );
        let mut deed = deed; // move
        deed.link_to_prev(self.current_hash.clone());
        self.current_hash = deed.self_hash.clone();
        self.deed_log.push(deed);
    }

    pub fn compute_zones(&self) -> ZoneClassification {
        let mut calm_count = 0;
        let mut stress_count = 0;
        let mut breach_count = 0;
        let mut total_mp = 0.0;

        // Per-agent diagnostics (window=10 for demo)
        for agent in &self.lattice {
            // Simulate short history for demo (in production: full log slice)
            let history = vec![agent.state.clone(); 10]; // placeholder
            let cs = Self::calc_cs(&history, 10);
            let od = Self::calc_od(&history, 10, 3);
            let rec = Self::calc_rec(&history, 8, 4);
            let budget = (agent.state.lifeforce + agent.state.oxygen) / 2.0;
            let peers_budgets: Vec<f64> = self.lattice.iter().map(|a| (a.state.lifeforce + a.state.oxygen) / 2.0).collect();
            let ud = Self::calc_ud(&peers_budgets, budget, if od { 0.7 } else { 0.2 });

            if cs && !od { calm_count += 1; }
            if od { stress_count += 1; }
            if ud { breach_count += 1; }
            total_mp += if cs { 0.9 } else if rec { 0.7 } else { 0.4 };
        }

        let n = self.lattice.len() as f64;
        let mp_score = (total_mp / n).clamp(0.0, 1.0);
        let church_rec = if breach_count as f64 / n < 0.05 {
            "eco_grant: homelessness_relief + ecological_sustainability NPO sponsorship (Rust nanoswarm stability model)"
        } else {
            "observe_and_share_resources – civic duty to raise CALM_STABLE"
        };

        ZoneClassification {
            safe_zone_fraction: calm_count as f64 / n,
            stress_corridor_fraction: stress_count as f64 / n,
            rights_breach_fraction: breach_count as f64 / n,
            mp_score,
            church_recommendation: church_rec.to_string(),
        }
    }

    pub fn export_log(&self) -> String {
        serde_json::to_string_pretty(&self.deed_log).unwrap()
    }
}

// Example usage (real-world research entrypoint)
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rights_preserved_in_safe_zone() {
        let mut obs = MicrospaceRightsObserver::new(20);
        for _ in 0..50 {
            obs.step(0.1); // low load = safe
        }
        let zones = obs.compute_zones();
        assert!(zones.safe_zone_fraction > 0.75);
        assert!(zones.rights_breach_fraction < 0.05);
        // This run mints CHURCH via low UNFAIR_DRAIN – good-deed logged
    }
}
