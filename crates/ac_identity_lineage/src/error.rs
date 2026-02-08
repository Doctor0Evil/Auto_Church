use thiserror::Error;

#[derive(Debug, Error)]
pub enum LineageError {
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    #[error("No match for target")]
    NoMatch,
}
