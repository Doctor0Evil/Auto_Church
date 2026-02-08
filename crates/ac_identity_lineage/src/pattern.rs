use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    pub name: String,
    pub raw: String,
}

impl CommandPattern {
    pub fn compile(&self) -> Result<Regex, crate::error::LineageError> {
        Regex::new(&self.raw)
            .map_err(|e| crate::error::LineageError::InvalidPattern(e.to_string()))
    }
}
