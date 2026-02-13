use chrono::{TimeZone, Utc};

/// Current Unix timestamp in seconds.
pub fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// Build a Utc DateTime from unix seconds.
pub fn from_unix(secs: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(secs, 0).single().unwrap()
}
