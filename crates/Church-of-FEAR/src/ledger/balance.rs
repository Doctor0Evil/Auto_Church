use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub account_id: String,
    pub church: u64,
    pub pwr: u64,
    pub timestamp: i64,
}
