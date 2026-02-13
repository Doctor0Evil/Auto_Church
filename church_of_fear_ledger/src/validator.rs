use crate::deed::DeedEvent;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("hash chain broken: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("life harm prohibited")]
    LifeHarm,
    #[error("ethics violation: {0:?}")]
    EthicsViolation(Vec<String>),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct LedgerValidator;

impl LedgerValidator {
    pub fn validate_new_event(event: &DeedEvent, expected_prev_hash: &str) -> Result<(), ValidationError> {
        if event.life_harm_flag {
            return Err(ValidationError::LifeHarm);
        }
        if !event.ethics_flags.is_empty() {
            return Err(ValidationError::EthicsViolation(event.ethics_flags.clone()));
        }
        if expected_prev_hash != "genesis" && event.prev_hash != expected_prev_hash {
            return Err(ValidationError::HashMismatch {
                expected: expected_prev_hash.to_string(),
                actual: event.prev_hash.clone(),
            });
        }
        Ok(())
    }
}
