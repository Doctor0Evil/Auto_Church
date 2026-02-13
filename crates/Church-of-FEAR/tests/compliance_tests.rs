use church_of_fear::ledger::deed_event::DeedEvent;
use church_of_fear::compliance::validator::validate_deed;

#[test]
fn compliant_deed_passes() {
    let genesis = DeedEvent::genesis();
    let deed = DeedEvent::new(
        genesis.self_hash,
        "actor".into(),
        vec![],
        "ecological_sustainability".into(),
        vec![],
        serde_json::json!({}),
        vec![],
        false,
    );
    assert!(validate_deed(&deed, 0.1, 0.2).is_ok());
}

#[test]
fn biophysical_violation_fails() {
    let genesis = DeedEvent::genesis();
    let deed = DeedEvent::new(
        genesis.self_hash,
        "actor".into(),
        vec![],
        "ecological_sustainability".into(),
        vec![],
        serde_json::json!({}),
        vec![],
        false,
    );
    assert!(validate_deed(&deed, 0.9, 1.5).is_err());
}
