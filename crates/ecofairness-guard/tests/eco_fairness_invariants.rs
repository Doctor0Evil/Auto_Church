use ecofairness_guard::GraceEquityKernel;
use std::path::PathBuf;

#[test]
fn eco_fairness_kernel_sum_min_share_must_not_exceed_one() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("..");
    path.push("policies");
    path.push("eco-fairness.aln");

    let kernel = GraceEquityKernel::from_path(&path)
        .expect(".eco-fairness.aln must load and satisfy invariants");

    let sum_min: f32 = kernel.classes.values().map(|b| b.min_share).sum();
    assert!(
        sum_min <= 1.0 + 1e-6,
        "sum(min_share) must be ≤ 1.0, got {}",
        sum_min
    );
}
