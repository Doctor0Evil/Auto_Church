use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

mod kernel;

pub use kernel::{EquityBounds, GraceEquityKernel, RouteEnvelope};

/// High-level error type for guard violations or configuration problems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardError {
    pub code: String,
    pub message: String,
}

/// Projection of the RoH model relevant for eco / compute fairness.
/// This is assumed to be parsed from `.rohmodel.aln` (JSON-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RohModel {
    pub ceiling: f32,
    /// Per-axis weights, e.g. { "eco_impact": 0.4, "compute_concentration": 0.3, ... }.
    pub weights: HashMap<String, f32>,
}

/// Per-route Tsafe envelope slice for power, heat, and compute.
/// Conceptually binds to `.tsafe.aln` & `.vkernel.aln` where energy and compute
/// are just additional axes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsafeEcoEnvelope {
    /// Logical route, e.g. "XR", "DRONE", "AUTO_CHURCH_SIM", "AUTO_CHURCH_LIVE".
    pub route: String,
    /// Max allowable instantaneous power draw (Watts equivalent).
    pub max_power: f32,
    /// Max allowable cumulative heat/energy over a time window (Joules equivalent).
    pub max_cumulative_energy: f32,
    /// Max fraction of local compute capacity this route may occupy (0.0–1.0).
    pub max_compute_fraction: f32,
}

/// Equity class: groups of subjects / communities that must receive fair treatment.
/// For Auto_Church you might use classes like "host", "local_congregation",
/// "remote_congregation", "research_only".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EquityClass {
    pub name: String,
}

/// Snapshot of current resource usage, e.g. computed by the scheduler and passed
/// into the guard on every high-risk action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageSnapshot {
    /// Total available power (Watts) for this node / cell / service window.
    pub total_power_budget: f32,
    /// Total available compute capacity (0.0–1.0 normalized).
    pub total_compute_capacity: f32,
    /// Current instantaneous power draw (Watts).
    pub current_power_draw: f32,
    /// Current cumulative energy usage in the time window (Joules).
    pub current_cumulative_energy: f32,
    /// Current compute utilization (0.0–1.0).
    pub current_compute_fraction: f32,
    /// Per-equity-class current share (0.0–1.0, typically relative to
    /// total_compute_capacity or total_power_budget).
    pub class_shares: HashMap<String, f32>,
}

/// Minimal projection of the Tsafe Cortex Gate XRAction; this should match
/// the struct in `tsafecortexgate` so the guard can be imported and used
/// without duplicating logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XRActionKind {
    ReadNeuralShard,
    WriteNeuralShard,
    ProposeEvolve,
    ApplyOta,
    XRRouteStep,
    ScheduleJob,
    ReadKeys,
    SignTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRAction {
    pub kind: XRActionKind,
    pub subjectid: String,
    pub route: String,
    /// Abstract estimate of lifeforce / energy cost for the action.
    pub lifeforcecost: f32,
    /// RoH before the action.
    pub rohbefore: f32,
    /// Estimated RoH after the action.
    pub rohafterestimate: f32,
    /// Optional equity class for the subject (e.g. "host", "local_congregation").
    pub equity_class: Option<String>,
}

/// Configuration shard for EcoFairnessGuard.
/// In practice you would load RohModel from `.rohmodel.aln`,
/// TsafeEcoEnvelope from `.tsafe-eco-envelopes.json` / `.vkernel.aln`,
/// and GraceEquityKernel from `.eco-fairness.aln`.
#[derive(Debug, Clone)]
pub struct EcoFairnessConfig {
    pub roh_model: RohModel,
    pub tsafe_envelopes: HashMap<String, TsafeEcoEnvelope>, // keyed by route
    pub grace_equity: GraceEquityKernel,
}

/// The **EcoFairnessGuard** enforces:
/// 1. Eco envelopes per route (power / energy / compute).
/// 2. GraceEquityKernel fairness bounds per class.
/// 3. RoH ceiling and eco-related RoH contributions.
#[derive(Debug, Clone)]
pub struct EcoFairnessGuard {
    cfg: EcoFairnessConfig,
}

impl EcoFairnessGuard {
    /// Load configuration from three JSON-compatible files:
    /// - `.rohmodel.aln`
    /// - `.tsafe-eco-envelopes.json` (route → envelope)
    /// - `.eco-fairness.aln`
    ///
    /// Adapt paths to your manifest layout (`neuro-workspace.manifest.aln`).
    pub fn from_paths<P: AsRef<Path>>(
        roh_path: P,
        tsafe_eco_path: P,
        eco_fairness_path: P,
    ) -> anyhow::Result<Self> {
        let roh_text = fs::read_to_string(roh_path.as_ref())?;
        let roh_model: RohModel = serde_json::from_str(&roh_text)?;

        let tsafe_text = fs::read_to_string(tsafe_eco_path.as_ref())?;
        let tsafe_envelopes: HashMap<String, TsafeEcoEnvelope> =
            serde_json::from_str(&tsafe_text)?;

        let eco_text = fs::read_to_string(eco_fairness_path.as_ref())?;
        let grace_equity: GraceEquityKernel = serde_json::from_str(&eco_text)?;

        Ok(Self {
            cfg: EcoFairnessConfig {
                roh_model,
                tsafe_envelopes,
                grace_equity,
            },
        })
    }

    /// Main check function to be called from Tsafe Cortex Gate.
    ///
    /// Inputs:
    /// - `action`: candidate XRAction being evaluated.
    /// - `snapshot`: current resource usage snapshot for this node / cell.
    ///
    /// Returns:
    /// - `Ok(())` if within envelopes and fairness constraints,
    /// - `Err(GuardError)` if the action must be denied.
    pub fn check(
        &self,
        action: &XRAction,
        snapshot: &ResourceUsageSnapshot,
    ) -> Result<(), GuardError> {
        // 1. Per-route eco envelope.
        self.check_route_envelope(action, snapshot)?;

        // 2. GraceEquityKernel fairness.
        self.check_equity_bounds(action, snapshot)?;

        // 3. RoH ceiling + eco-related RoH contribution.
        self.check_roh_ecofairness(action)?;

        Ok(())
    }

    fn check_route_envelope(
        &self,
        action: &XRAction,
        snapshot: &ResourceUsageSnapshot,
    ) -> Result<(), GuardError> {
        let env = self
            .cfg
            .tsafe_envelopes
            .get(&action.route)
            .ok_or_else(|| GuardError {
                code: "ECO_NO_ROUTE_ENV".into(),
                message: format!(
                    "No TsafeEcoEnvelope configured for route '{}' – deny by default",
                    action.route
                ),
            })?;

        let projected_power = snapshot.current_power_draw + action.lifeforcecost;
        if projected_power > env.max_power {
            return Err(GuardError {
                code: "ECO_POWER_EXCEEDED".into(),
                message: format!(
                    "Projected power {}W exceeds max {}W for route '{}'",
                    projected_power, env.max_power, action.route
                ),
            });
        }

        // For simplicity, treat lifeforcecost as additional energy to the window.
        let projected_energy = snapshot.current_cumulative_energy + action.lifeforcecost;
        if projected_energy > env.max_cumulative_energy {
            return Err(GuardError {
                code: "ECO_ENERGY_EXCEEDED".into(),
                message: format!(
                    "Projected cumulative energy {}J exceeds max {}J for route '{}'",
                    projected_energy, env.max_cumulative_energy, action.route
                ),
            });
        }

        // Simple normalized compute projection; in a real system this should be
        // bound to concrete CPU/GPU metrics.
        let denom = snapshot.total_compute_capacity.max(1.0);
        let projected_compute = snapshot.current_compute_fraction + (action.lifeforcecost / denom);
        if projected_compute > env.max_compute_fraction {
            return Err(GuardError {
                code: "ECO_COMPUTE_EXCEEDED".into(),
                message: format!(
                    "Projected compute fraction {:.3} exceeds max {:.3} for route '{}'",
                    projected_compute, env.max_compute_fraction, action.route
                ),
            });
        }

        Ok(())
    }

    fn check_equity_bounds(
        &self,
        action: &XRAction,
        snapshot: &ResourceUsageSnapshot,
    ) -> Result<(), GuardError> {
        let class_name = match &action.equity_class {
            Some(c) => c,
            None => {
                // If no class is provided, treat as a configuration error for Auto_Church fairness.
                return Err(GuardError {
                    code: "ECO_NO_EQUITY_CLASS".into(),
                    message: "XRAction missing equity_class; Auto_Church fairness requires it"
                        .into(),
                });
            }
        };

        let bounds = self
            .cfg
            .grace_equity
            .bounds_for_class(class_name)
            .ok_or_else(|| GuardError {
                code: "ECO_UNKNOWN_EQUITY_CLASS".into(),
                message: format!(
                    "Equity class '{}' not present in GraceEquityKernel",
                    class_name
                ),
            })?;

        let current_share = snapshot.class_shares.get(class_name).cloned().unwrap_or(0.0);

        // Compute a naive projected share: add normalized cost to this class's share.
        let denom = snapshot.total_power_budget.max(1.0);
        let projected_share = current_share + (action.lifeforcecost / denom);

        // Upper bound: no class may exceed its max_share.
        if projected_share > bounds.max_share {
            return Err(GuardError {
                code: "ECO_EQUITY_MAX_EXCEEDED".into(),
                message: format!(
                    "Equity class '{}' would exceed max_share {:.3} (projected {:.3})",
                    class_name, bounds.max_share, projected_share
                ),
            });
        }

        // Lower bound: pro-equity bias (do not deny under-served classes here).
        // A more advanced scheduler can use `current_share < min_share` as a
        // "priority uplift" signal.

        Ok(())
    }

    fn check_roh_ecofairness(&self, action: &XRAction) -> Result<(), GuardError> {
        // Standard RoH ceiling & monotone safety: RoH must not increase
        // and must remain ≤ ceiling (typically 0.3).
        if action.rohafterestimate > self.cfg.roh_model.ceiling {
            return Err(GuardError {
                code: "ROH_CEILING".into(),
                message: format!(
                    "RoH estimate {:.3} exceeds ceiling {:.3}",
                    action.rohafterestimate, self.cfg.roh_model.ceiling
                ),
            });
        }

        if action.rohafterestimate > action.rohbefore {
            return Err(GuardError {
                code: "ROH_MONOTONE".into(),
                message: format!(
                    "RoH monotone safety violated: before {:.3}, after {:.3}",
                    action.rohbefore, action.rohafterestimate
                ),
            });
        }

        // Optional: check eco-related RoH axes if present.
        if !self.cfg.roh_model.weights.contains_key("eco_impact")
            || !self
                .cfg
                .roh_model
                .weights
                .contains_key("compute_concentration")
        {
            // Not a hard error for now; CI can tighten this to a failure if required.
        }

        Ok(())
    }

    /// Convenience layer for Tsafe Cortex Gate, so you can call:
    ///
    /// `eco_guard.check_for_gate(&req.action, &snapshot)`
    ///
    /// inside the main `authorize_request` function.
    pub fn check_for_gate(
        &self,
        action: &XRAction,
        snapshot: &ResourceUsageSnapshot,
    ) -> EcoFairnessResult {
        self.check(action, snapshot)
    }
}

// --- Shared error type used by callers integrating multiple guardians ---

/// Result type to mirror other guardian crates (neurorights, RoH, eco, etc.).
pub type EcoFairnessResult = Result<(), GuardError>;
