use crate::ledger::deed_event::{DeedError, DeedEvent};
use crate::compliance::eco_reg::EcoRegEnvelope;
use crate::compliance::ethics::EthicsContext;

pub fn validate_deed(
    event: &DeedEvent,
    roh: f64,
    decay: f64,
) -> Result<(), DeedError> {
    event.validate_biophysical(roh, decay)?;

    let eco = EcoRegEnvelope::default();
    if !eco.within_bounds(roh, decay) {
        return Err(DeedError::InvariantViolation(
            "EcoReg envelope breach".to_string(),
        ));
    }

    let ctx = EthicsContext {
        flags: event.ethics_flags.clone(),
        life_harm_flag: event.life_harm_flag,
    };

    if !ctx.is_clean() {
        return Err(DeedError::InvariantViolation(
            "Ethics flags present".to_string(),
        ));
    }

    Ok(())
}
