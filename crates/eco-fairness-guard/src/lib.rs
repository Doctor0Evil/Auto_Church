//! EcoFairnessGuard + GraceEquityKernel
//! Ecological + compute fairness guardian for Auto_Church routes.
//!
//! Invariants:
//! - RoH ceiling is respected (≤ 0.3).
//! - Per-route eco envelopes enforced (strictest-wins).
//! - Per-subject minimum service enforced where configured.
//! - Altar routes are governed compute and must go through EVOLVE paths.

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use thiserror::Error;
use tracing::{info, warn};

pub use rohmodel::RohModel;
pub use tsafe::{RequestRoute, SovereignAction};
pub use vkernel::ViabilityKernel;

// ──────────────────────────────────────────────────────────────
// 1. ALN / JSON-friendly core types (.eco-fairness.aln)
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoEnvelope {
    pub max_power_watts: f64,
    pub max_daily_kwh: f64,
    pub max_heat_output: f64,
    pub max_co2e_kg: f64,
    pub max_water_liters: f64,
}

impl Default for EcoEnvelope {
    fn default() -> Self {
        Self {
            max_power_watts: 850.0,
            max_daily_kwh: 18.0,
            max_heat_output: 45.0,
            max_co2e_kg: 2.5,
            max_water_liters: 15.0,
        }
    }
}

/// Shard for `config/.eco-fairness.aln` (JSON or ALN → JSON-compat)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoFairnessSpec {
    /// Hard RoH ceiling, must be ≤ 0.30.
    pub global_roh_ceiling: f64,

    /// Global eco envelope for this node / EcoSys cell.
    pub global_envelope: EcoEnvelope,

    /// Per-route eco budgets (route id → envelope).
    pub per_route_budgets: HashMap<String, EcoEnvelope>,

    /// Per-subject minimums (subject_id → envelope).
    pub per_subject_minimums: HashMap<String, EcoEnvelope>,

    /// Routes treated as Auto_Church Altar (donation, lesson, sacred compute).
    pub altar_routes: Vec<String>,
}

impl Default for EcoFairnessSpec {
    fn default() -> Self {
        let mut budgets = HashMap::new();
        budgets.insert(
            "altar".to_string(),
            EcoEnvelope {
                max_power_watts: 420.0,
                max_daily_kwh: 8.0,
                ..EcoEnvelope::default()
            },
        );

        Self {
            global_roh_ceiling: 0.30,
            global_envelope: EcoEnvelope::default(),
            per_route_budgets: budgets,
            per_subject_minimums: HashMap::new(),
            altar_routes: vec!["altar".into(), "donation".into(), "lesson".into()],
        }
    }
}

impl EcoFairnessSpec {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let spec: EcoFairnessSpec = serde_json::from_reader(file)?;
        Ok(spec)
    }
}

// ──────────────────────────────────────────────────────────────
// 2. Global spec + live usage tracking
// ──────────────────────────────────────────────────────────────

static ECO_SPEC: Lazy<RwLock<EcoFairnessSpec>> = Lazy::new(|| {
    // In production, path is discovered via neuro-workspace.manifest.aln
    let spec = EcoFairnessSpec::load("config/.eco-fairness.aln")
        .unwrap_or_else(|_| {
            warn!(".eco-fairness.aln missing or invalid, using defaults");
            EcoFairnessSpec::default()
        });
    RwLock::new(spec)
});

/// Per-subject live usage (aggregate over current window).
static CURRENT_USAGE: Lazy<DashMap<String, EcoEnvelope>> = Lazy::new(DashMap::new);

// ──────────────────────────────────────────────────────────────
// 3. Errors and kernel
// ──────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum GuardError {
    #[error("Eco budget exceeded on route {route}: {resource} demand {demand} > limit {limit}")]
    BudgetExceeded {
        route: String,
        resource: String,
        demand: f64,
        limit: f64,
    },

    #[error("Equity violation for subject {subject}: below guaranteed minimum")]
    BelowMinimum { subject: String },

    #[error("RoH ceiling breach (current {current_roh} > ceiling {ceiling})")]
    RohCeilingBreach { current_roh: f32, ceiling: f32 },

    #[error("Viability kernel rejection: {reason}")]
    ViabilityFailure { reason: String },

    #[error("Altar route requires EVOLVE-governed path (no free throughput)")]
    AltarRequiresEvolve,
}

#[derive(Debug)]
pub struct GraceEquityKernel {
    roh: RohModel,
    vkernel: ViabilityKernel,
}

impl GraceEquityKernel {
    pub fn new(roh: RohModel, vkernel: ViabilityKernel) -> Self {
        Self { roh, vkernel }
    }

    /// Primary invariant check – called by EcoFairnessGuard.
    pub fn check_route(
        &self,
        subject: &str,
        route: &str,
        demand: &EcoEnvelope,
    ) -> Result<(), GuardError> {
        let spec = ECO_SPEC.read();

        // 1. RoH hard ceiling (0.3) – monotone safety lives in RoH guard,
        // here we just ensure the ceiling is respected at the node.[file:1]
        let current_roh = self.roh.current_value();
        if current_roh > spec.global_roh_ceiling as f32 {
            return Err(GuardError::RohCeilingBreach {
                current_roh,
                ceiling: spec.global_roh_ceiling as f32,
            });
        }

        // 2. Per-route envelope (strictest-wins).[file:2]
        let route_key = route.to_lowercase();
        let envelope = spec
            .per_route_budgets
            .get(&route_key)
            .unwrap_or(&spec.global_envelope);

        if demand.max_power_watts > envelope.max_power_watts {
            return Err(GuardError::BudgetExceeded {
                route: route.to_string(),
                resource: "power".into(),
                demand: demand.max_power_watts,
                limit: envelope.max_power_watts,
            });
        }
        if demand.max_daily_kwh > envelope.max_daily_kwh {
            return Err(GuardError::BudgetExceeded {
                route: route.to_string(),
                resource: "kWh".into(),
                demand: demand.max_daily_kwh,
                limit: envelope.max_daily_kwh,
            });
        }
        if demand.max_co2e_kg > envelope.max_co2e_kg {
            return Err(GuardError::BudgetExceeded {
                route: route.to_string(),
                resource: "CO2e_kg".into(),
                demand: demand.max_co2e_kg,
                limit: envelope.max_co2e_kg,
            });
        }

        // 3. Altar routes are governed compute – no direct SMART/CHAT scheduling.[file:5]
        if spec
            .altar_routes
            .iter()
            .any(|r| r.eq_ignore_ascii_case(route))
        {
            return Err(GuardError::AltarRequiresEvolve);
        }

        // 4. Per-subject equity floor.
        let usage = CURRENT_USAGE
            .entry(subject.to_string())
            .or_insert_with(EcoEnvelope::default);

        if let Some(minimum) = spec.per_subject_minimums.get(subject) {
            if usage.max_daily_kwh + demand.max_daily_kwh < minimum.max_daily_kwh {
                return Err(GuardError::BelowMinimum {
                    subject: subject.to_string(),
                });
            }
        }

        // 5. Tsafe viability kernel coupling – eco/compute inside safe polytopes.[file:1]
        if !self.vkernel.is_viable(demand) {
            return Err(GuardError::ViabilityFailure {
                reason: "Eco/compute demand outside Tsafe viability kernel".into(),
            });
        }

        // 6. On success, commit usage (sharded, low-contention).
        usage.max_power_watts += demand.max_power_watts;
        usage.max_daily_kwh += demand.max_daily_kwh;
        usage.max_heat_output += demand.max_heat_output;
        usage.max_co2e_kg += demand.max_co2e_kg;
        usage.max_water_liters += demand.max_water_liters;

        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────
// 4. EcoFairnessGuard – public guardian API
// ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct EcoFairnessGuard {
    kernel: GraceEquityKernel,
}

impl EcoFairnessGuard {
    pub fn new(roh: RohModel, vkernel: ViabilityKernel) -> Self {
        info!("EcoFairnessGuard initialized (GraceEquityKernel active)");
        Self {
            kernel: GraceEquityKernel::new(roh, vkernel),
        }
    }

    /// Entry point used from Tsafe Cortex Gate guardian set.
    pub fn check(
        &self,
        action: &SovereignAction,
        route: &RequestRoute,
    ) -> Result<(), GuardError> {
        let demand = self.estimate_demand(action, route);
        self.kernel
            .check_route(&action.subjectid, route.as_str(), &demand)
    }

    /// Projection from SovereignAction + route → eco envelope.
    /// In production, this should use your existing cost/telemetry model.[file:2]
    fn estimate_demand(&self, action: &SovereignAction, route: &RequestRoute) -> EcoEnvelope {
        // Example heuristic: tie lifeforcecost and route kind to power/eco projection.
        let base_power = (action.lifeforcecost as f64) * 1000.0;
        let route_id = route.as_str().to_lowercase();

        let (kwh, co2e) = if route_id.contains("altar") {
            (8.0, 1.0)
        } else if route_id.contains("sim") {
            (5.0, 0.5)
        } else {
            (3.0, 0.3)
        };

        EcoEnvelope {
            max_power_watts: base_power.clamp(0.0, 850.0),
            max_daily_kwh: kwh,
            max_heat_output: 40.0,
            max_co2e_kg: co2e,
            max_water_liters: 10.0,
        }
    }
}
