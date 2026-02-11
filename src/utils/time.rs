pub fn time_discount_factor(age_seconds: u64) -> f64 {
    // Exponential decay: e^(-age / tau), tau = 1 day
    let tau = 86400.0;
    (-(age_seconds as f64) / tau).exp()
}
