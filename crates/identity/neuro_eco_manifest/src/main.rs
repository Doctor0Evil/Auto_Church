use neuro_eco_manifest::{NeuroEcoIdentityManifest, DVector};
use nalgebra::DMatrix;

fn main() {
    let mut manifest = NeuroEcoIdentityManifest::default();
    println!("NeuroEcoIdentityManifest initialized for Phoenix, AZ (MST baseline). Inner domain: absolute. Outer: RAF r0=0.5, HB=9.7/10 bee-focus.");

    // Sim: 0.2mi walk+smoke (M_neg: CO2=0.5kg, PM2.5=0.1kg) vs car (M_neg: CO2=2.0kg, PM2.5=0.05kg)
    let m_walk_smoke_neg = DVector::from_vec(vec![0.5, 0.1]);
    let m_car_neg = DVector::from_vec(vec![2.0, 0.05]);
    let m_rest_pos = DVector::from_vec(vec![1.0, 0.0]);  // Cybo-Air restoration

    let delta_walk = manifest.raf_delta(m_rest_pos.clone(), m_walk_smoke_neg).unwrap();  // +0.05 net (eco-grant)
    let delta_car = manifest.raf_delta(DVector::zeros(2), m_car_neg).unwrap();  // -0.2 (greed-unfair, triggers Errority)

    if delta_car < -0.15 {
        let err_event = extensions::ErrorityEvent { description: "High-emission choice; route to restoration".to_string(), delta_r: delta_car };
        manifest.err_log(err_event);  // Logs for polytope tighten, earns WISE
    }

    // Broadcast live delta: Fairness measurable (greed as only unfair object: high-neg without pos)
    println!("RAF delta walk+smoke+restore: {:.2} (fair, earns TECH/NANO)", delta_walk);
    println!("RAF delta car: {:.2} (unfair greed-scale; Errority logged, inner safe)", delta_car);

    // Polytope check: x_proj for low-impact action
    let x_proj = DVector::from_vec(vec![0.1, 0.05, 0.0]);  // Stressors: CO2, PM, VOC
    let a_eco = DMatrix::from_row_slice(3, 3, &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);  // Identity constraints
    let b_eco = DVector::from_vec(vec![1.0, 0.2, 0.1]);  // Bounds: CO2<1kg, PM<0.2, VOC<0.1
    if manifest.eco_admissible(&x_proj) {
        println!("Action admissible: Bee-safe (BEE_WEIGHT=1.5x on PM/VOC), earns POWER.");
    }

    // Hex-stamp demo: Evidence for good-deed (restoration sim)
    let evidence = b"Restorative action: 1kg CO2 neutralized via Cybo-Air.";
    let stamp = manifest.hex_stamp(evidence);
    println!("Hex-stamp for eco-grant: {}", stamp);  // Earns CHURCH via verifiable deed

    // Fairness demo: Exclude FEAR from scaling (inner invariant); greed (unrestored neg) only unfair object.
    // Last-response integrated: Hybrid signaling + anchors = measurable fairness (e.g., delta_car triggers restoration path, not reversal).
    println!("Fairness: Greed (unmitigated neg ΔK) scales outer down; inner/rights unchanged. Errority ensures learning, not punishment.");
}
