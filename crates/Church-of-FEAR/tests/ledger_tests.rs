use church_of_fear::ledger::deed_event::{DeedEvent, validate_chain};

#[test]
fn chain_integrity_holds() {
    let genesis = DeedEvent::genesis();
    let d1 = DeedEvent::new(
        genesis.self_hash.clone(),
        "a1".into(),
        vec!["t1".into()],
        "ecological_sustainability".into(),
        vec![],
        serde_json::json!({}),
        vec![],
        false,
    );
    let d2 = DeedEvent::new(
        d1.self_hash.clone(),
        "a2".into(),
        vec!["t2".into()],
        "ecological_sustainability".into(),
        vec![],
        serde_json::json!({}),
        vec![],
        false,
    );

    let chain = vec![genesis, d1, d2];
    assert!(validate_chain(&chain));
}
