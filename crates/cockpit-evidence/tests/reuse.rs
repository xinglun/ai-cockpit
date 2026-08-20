use cockpit_evidence::{Binding, ReuseAction, ReuseInput, decide_reuse};

#[test]
fn exact_fresh_binding_can_reuse_non_protected_evidence() {
    let binding =
        Binding::content("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let decision = decide_reuse(
        &binding,
        &ReuseInput {
            content_digest: Some(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            ),
            protected: false,
            expired: false,
        },
    );
    assert_eq!(decision.action, ReuseAction::Reuse);
}

#[test]
fn protected_evidence_is_executed_even_when_binding_matches() {
    let binding =
        Binding::content("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let decision = decide_reuse(
        &binding,
        &ReuseInput {
            content_digest: Some(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            ),
            protected: true,
            expired: false,
        },
    );
    assert_eq!(decision.action, ReuseAction::Execute);
}
