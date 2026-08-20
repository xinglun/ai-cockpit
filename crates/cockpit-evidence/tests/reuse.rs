use cockpit_evidence::{Binding, EvidenceBinding, ReuseAction, ReuseInput, decide_reuse};

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

#[test]
fn diff_and_environment_bindings_are_first_class() {
    let diff = Binding::diff(
        "base",
        "head",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let environment = Binding::environment(
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert!(matches!(
        diff.evidence_binding,
        EvidenceBinding::Diff { .. }
    ));
    assert!(matches!(
        environment.evidence_binding,
        EvidenceBinding::Environment { .. }
    ));
}
