use serde::{Deserialize, Serialize};
use thiserror::Error;

use biophysical_blockchain::{
    lifeforce::applylifeforceguardedadjustment,
    types::{BioTokenState, HostEnvelope, SystemAdjustment},
};
use augdoctor_neuromorph_core::{NeuromorphFeature, NeuromorphRoute, NeuromorphRouter};

/// Church-of-FEAR fairness envelope: no predation, no structural caps, no savagery.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FearDoctrineEnvelope {
    /// Max safe FLOPs per neuromorph evolution turn (host-local, non-financial).
    pub max_flops_per_turn: f64,
    /// Max neuromorphic eco-energy per evolution turn (nJ-equivalent).
    pub max_eco_energy_nj: f64,
    /// Soft equality floor: minimum SCALE-like micro-step allowed when safe.
    pub min_microstep_scale: f64,
}

/// Per-host neuromorphic evolution config under Church-of-FEAR.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FearNeuromorphConfig {
    pub host_id: String,
    pub doctrine: FearDoctrineEnvelope,
    /// Daily turn limit (must not exceed inner-ledger MAXDAILYTURNS).
    pub max_daily_turns: u8,
}

/// Runtime-tracked daily turn state (host-local).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FearDailyTurnState {
    pub date_yyyymmdd: u32,
    pub turns_used: u8,
}

impl FearDailyTurnState {
    pub fn can_consume_turn(&self, max_turns: u8, today: u32) -> bool {
        if self.date_yyyymmdd != today {
            // new day: always can consume (caller must reset turns_used=0)
            return true;
        }
        self.turns_used < max_turns
    }

    pub fn consume_turn(&mut self, max_turns: u8, today: u32) -> Result<(), FearError> {
        if self.date_yyyymmdd != today {
            self.date_yyyymmdd = today;
            self.turns_used = 0;
        }
        if self.turns_used >= max_turns {
            return Err(FearError::DailyTurnLimitReached);
        }
        self.turns_used += 1;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum FearError {
    #[error("neuromorph routing denied by safety router")]
    NeuromorphDenied,
    #[error("eco or FLOPs budget exceeded for this evolution turn")]
    EcoBudgetExceeded,
    #[error("lifeforce guard rejected the proposed adjustment")]
    LifeforceViolation,
    #[error("daily neuromorphic evolution turn limit reached")]
    DailyTurnLimitReached,
}

/// High-level evolution request for neuromorphic adapter upgrades.
/// This never carries identity or consciousness fields; only safe metrics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FearNeuromorphEvolutionFrame {
    pub plane: String,          // e.g., "neuromorph.softwareonly"
    pub adapter_id: String,     // logical neuromorph adapter name
    pub scope: String,          // e.g., "prosthetic-intent", "cursor-control"
    pub estimated_flops: f64,   // FLOPs cost of the proposed update
    pub estimated_energy_nj: f64,
    /// Expected effect band: "low-latency", "low-error", "eco-optimized", etc.
    pub expected_effect_band: String,
    /// Proposed SCALE-like factor (dimensionless micro-step magnitude).
    pub proposed_scale_delta: f64,
}

/// Church-of-FEAR neuromorph evolution engine: wraps routing + lifeforce + eco + turns.
pub struct FearNeuromorphEngine<R: NeuromorphRouter> {
    pub router: R,
    pub cfg: FearNeuromorphConfig,
}

impl<R: NeuromorphRouter> FearNeuromorphEngine<R> {
    /// Attempt a single, non-predatory neuromorphic evolution micro-step.
    /// Fairness rules:
    /// - Only SAFE routes may evolve.
    /// - FLOPs/eco must stay within doctrine envelope.
    /// - Lifeforce guards must accept SystemAdjustment.
    /// - Daily turn limit must not be exceeded.
    /// - Micro-step downscales rather than hard-denies when close to limits.
    pub fn try_evolve_neuromorph(
        &self,
        feature: &NeuromorphFeature,
        frame: &FearNeuromorphEvolutionFrame,
        host_env: &HostEnvelope,
        state: &mut BioTokenState,
        daily_turns: &mut FearDailyTurnState,
        today_yyyymmdd: u32,
    ) -> Result<SystemAdjustment, FearError> {
        // 1. Route neuromorph feature through Safe/Defer/Deny router.
        let route = self.router.route(feature.clone());
        if !matches!(route, NeuromorphRoute::Safe) {
            return Err(FearError::NeuromorphDenied);
        }

        // 2. Enforce FLOPs + eco envelopes for this evolution frame.
        if frame.estimated_flops > self.cfg.doctrine.max_flops_per_turn
            || frame.estimated_energy_nj > self.cfg.doctrine.max_eco_energy_nj
        {
            return Err(FearError::EcoBudgetExceeded);
        }

        // 3. Respect daily neuromorphic evolution turn limit (host-local).
        if !daily_turns.can_consume_turn(self.cfg.max_daily_turns, today_yyyymmdd) {
            return Err(FearError::DailyTurnLimitReached);
        }

        // 4. Compute a fair, equality-preserving micro-step for SCALE/WAVE.
        //    No predation: never reduces BRAIN, never creates cross-host flows.
        let raw_scale = frame.proposed_scale_delta;
        let micro_scale = raw_scale
            .max(self.cfg.doctrine.min_microstep_scale)
            .min(1.0);

        let adj = SystemAdjustment {
            deltabrain: 0.0,
            deltawave: micro_scale,
            deltablood: 0.0,
            deltaoxygen: 0.0,
            deltanano: 0.0,
            deltasmart: 0.0,
            ecocost: frame.estimated_flops,
            reason: format!(
                "church-of-fear-neuromorph-upgrade:{}:{}",
                frame.adapter_id, frame.scope
            ),
        };

        // 5. Lifeforce guards: fairness means we never override biophysical safety.
        let mut trial_state = state.clone();
        if let Err(_) = applylifeforceguardedadjustment(&mut trial_state, host_env.clone(), adj.clone())
        {
            return Err(FearError::LifeforceViolation);
        }

        // 6. If everything is safe, consume a turn and commit.
        daily_turns.consume_turn(self.cfg.max_daily_turns, today_yyyymmdd)?;
        *state = trial_state;

        Ok(adj)
    }
}
