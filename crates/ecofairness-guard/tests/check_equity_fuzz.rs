use ecofairness_guard::{check_equity_bounds, GraceEquityKernel, ResourceUsageSnapshot};
use rand::Rng;
use std::path::PathBuf;

/// Simple randomized harness: generate many snapshots + lifeforcecost,
/// assert that whenever check_equity_bounds returns Ok, the projected
/// share does not exceed max_share for that class.
#[test]
fn check_equity_bounds_never_allows_exceeding_max_share() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("..");
    path.push("policies");
    path.push("eco-fairness.aln");

    let kernel = GraceEquityKernel::from_path(&path)
        .expect(".eco-fairness.aln must load and satisfy invariants");

    let class_names: Vec<String> = kernel.classes.keys().cloned().collect();
    assert!(
        !class_names.is_empty(),
        "At least one EquityClass must be defined"
    );

    let mut rng = rand::thread_rng();

    for _ in 0..10_000 {
        let class = &class_names[rng.gen_range(0..class_names.len())];
        let bounds = kernel.classes.get(class).unwrap();

        let total_power_budget = rng.gen_range(1.0_f32..10_000.0);
        let current_class_share = rng.gen_range(0.0_f32..bounds.max_share.min(0.99));
        let current_power_draw = rng.gen_range(
            (current_class_share * total_power_budget)..(total_power_budget * 0.99),
        );
        let class_power_draw = current_class_share * total_power_budget;

        let snapshot = ResourceUsageSnapshot {
            current_power_draw,
            total_power_budget,
            class_power_draw,
        };

        let remaining_fraction = (bounds.max_share - current_class_share).max(0.0);
        let max_allowable_lifeforce = remaining_fraction * total_power_budget;
        let lifeforcecost = if max_allowable_lifeforce <= 0.0 {
            0.0
        } else {
            rng.gen_range(0.0_f32..=(max_allowable_lifeforce * 1.5))
        };

        let projected_class_power = class_power_draw + lifeforcecost;
        let projected_share = projected_class_power / total_power_budget;

        let result = check_equity_bounds(&kernel, class, &snapshot, lifeforcecost);

        match result {
            Ok(()) => {
                assert!(
                    projected_share <= bounds.max_share + 1e-5,
                    "check_equity_bounds allowed projected_share {} above max_share {} \
                     for class '{}', snapshot={:?}, lifeforcecost={}",
                    projected_share,
                    bounds.max_share,
                    class,
                    snapshot,
                    lifeforcecost
                );
            }
            Err(e) => {
                if let ecofairness_guard::EcoFairnessError::MaxShareExceeded { .. } = e {
                    assert!(
                        projected_share > bounds.max_share - 1e-5,
                        "MaxShareExceeded error when projected_share {} ≤ max_share {} \
                         for class '{}', snapshot={:?}, lifeforcecost={}",
                        projected_share,
                        bounds.max_share,
                        class,
                        snapshot,
                        lifeforcecost
                    );
                }
            }
        }
    }
}
