use cockpit_evidence::{
    DiffIdentity, EvidenceContext, REUSABLE_RECEIPT_SCHEMA_VERSION, ReusableReceipt, ReuseAction,
    ReuseReason, ReuseState, decide_reuse,
};

const NOW: i64 = 1_800_000_000;

fn digest(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
}

fn context() -> EvidenceContext {
    EvidenceContext {
        content_digest: digest('a'),
        diff: DiffIdentity {
            base_commit: "1111111111111111111111111111111111111111".into(),
            head_commit: "2222222222222222222222222222222222222222".into(),
            changed_paths_digest: digest('b'),
        },
        environment_digest: digest('c'),
        command_digest: digest('d'),
        scope_digest: digest('e'),
        governance_digest: digest('f'),
        toolchain_digest: digest('1'),
        policy_digest: digest('2'),
        profile_digest: digest('6'),
        stage: "task".into(),
        runner: "local".into(),
    }
}

fn receipt(node_id: &str, passed: bool, context: &EvidenceContext) -> ReusableReceipt {
    ReusableReceipt::new(
        node_id,
        passed,
        context.clone(),
        &digest('3'),
        NOW - 60,
        NOW + 60,
    )
    .expect("valid reusable receipt")
}

#[test]
fn exact_composite_receipt_is_fresh_and_reusable() {
    let current = context();
    let candidate = receipt("tests", true, &current);

    let decision = decide_reuse(Some(&candidate), &current, "tests", NOW, false);

    assert_eq!(decision.state, ReuseState::Fresh);
    assert_eq!(decision.action, ReuseAction::Reuse);
    assert_eq!(decision.reason, ReuseReason::FreshExactBinding);
    assert_eq!(candidate.schema_version, 2);
    assert_eq!(REUSABLE_RECEIPT_SCHEMA_VERSION, 2);
}

#[test]
fn every_composite_identity_mismatch_is_stale_and_executes() {
    let bound = context();
    let candidate = receipt("tests", true, &bound);
    let mut mutations = Vec::new();

    let mut value = bound.clone();
    value.content_digest = digest('4');
    mutations.push(("content", value));
    let mut value = bound.clone();
    value.diff.base_commit = "3333333333333333333333333333333333333333".into();
    mutations.push(("base commit", value));
    let mut value = bound.clone();
    value.diff.head_commit = "4444444444444444444444444444444444444444".into();
    mutations.push(("head commit", value));
    let mut value = bound.clone();
    value.diff.changed_paths_digest = digest('5');
    mutations.push(("changed paths", value));
    let mut value = bound.clone();
    value.environment_digest = digest('6');
    mutations.push(("environment", value));
    let mut value = bound.clone();
    value.command_digest = digest('7');
    mutations.push(("command", value));
    let mut value = bound.clone();
    value.scope_digest = digest('8');
    mutations.push(("scope", value));
    let mut value = bound.clone();
    value.governance_digest = digest('9');
    mutations.push(("governance", value));
    let mut value = bound.clone();
    value.toolchain_digest = digest('0');
    mutations.push(("toolchain", value));
    let mut value = bound.clone();
    value.policy_digest = digest('4');
    mutations.push(("policy", value));
    let mut value = bound.clone();
    value.profile_digest = digest('5');
    mutations.push(("profile", value));
    let mut value = bound.clone();
    value.stage = "pr".into();
    mutations.push(("stage", value));
    let mut value = bound.clone();
    value.runner = "hosted".into();
    mutations.push(("runner", value));

    for (name, current) in mutations {
        let decision = decide_reuse(Some(&candidate), &current, "tests", NOW, false);
        assert_eq!(decision.state, ReuseState::Stale, "{name}");
        assert_eq!(decision.action, ReuseAction::Execute, "{name}");
        assert_eq!(decision.reason, ReuseReason::BindingMismatch, "{name}");
    }
}

#[test]
fn missing_failed_future_and_tampered_receipts_are_unknown_and_execute() {
    let current = context();

    let missing = decide_reuse(None, &current, "tests", NOW, false);
    assert_eq!(missing.state, ReuseState::Unknown);
    assert_eq!(missing.action, ReuseAction::Execute);
    assert_eq!(missing.reason, ReuseReason::EvidenceMissing);

    let failed = receipt("tests", false, &current);
    let decision = decide_reuse(Some(&failed), &current, "tests", NOW, false);
    assert_eq!(decision.state, ReuseState::Unknown);
    assert_eq!(decision.reason, ReuseReason::ReceiptFailed);

    let future = ReusableReceipt::new(
        "tests",
        true,
        current.clone(),
        &digest('3'),
        NOW + 1,
        NOW + 120,
    )
    .expect("future receipt is structurally valid");
    let decision = decide_reuse(Some(&future), &current, "tests", NOW, false);
    assert_eq!(decision.state, ReuseState::Unknown);
    assert_eq!(decision.reason, ReuseReason::ReceiptFromFuture);

    let mut tampered = receipt("tests", true, &current);
    tampered.receipt_id = digest('9');
    let decision = decide_reuse(Some(&tampered), &current, "tests", NOW, false);
    assert_eq!(decision.state, ReuseState::Unknown);
    assert_eq!(decision.reason, ReuseReason::ReceiptInvalid);

    let mut malformed = receipt("tests", true, &current);
    malformed.output_digest = "not-a-digest".into();
    let decision = decide_reuse(Some(&malformed), &current, "tests", NOW, false);
    assert_eq!(decision.state, ReuseState::Unknown);
    assert_eq!(decision.reason, ReuseReason::ReceiptInvalid);
}

#[test]
fn expired_receipt_is_stale_but_wrong_node_receipt_is_unknown() {
    let current = context();
    let expired = ReusableReceipt::new(
        "tests",
        true,
        current.clone(),
        &digest('3'),
        NOW - 120,
        NOW - 1,
    )
    .expect("expired receipt is structurally valid");
    let decision = decide_reuse(Some(&expired), &current, "tests", NOW, false);
    assert_eq!(decision.state, ReuseState::Stale);
    assert_eq!(decision.reason, ReuseReason::EvidenceExpired);

    let other_node = receipt("lint", true, &current);
    let decision = decide_reuse(Some(&other_node), &current, "tests", NOW, false);
    assert_eq!(decision.state, ReuseState::Unknown);
    assert_eq!(decision.reason, ReuseReason::ReceiptInvalid);
}

#[test]
fn protected_node_executes_even_with_fresh_receipt() {
    let current = context();
    let candidate = receipt("scope", true, &current);

    let decision = decide_reuse(Some(&candidate), &current, "scope", NOW, true);

    assert_eq!(decision.action, ReuseAction::Execute);
    assert_eq!(decision.reason, ReuseReason::ProtectedNode);
}

#[test]
fn serialized_receipt_rejects_unknown_schema_fields() {
    let current = context();
    let candidate = receipt("tests", true, &current);
    let mut value = serde_json::to_value(candidate).expect("serialize receipt");
    value
        .as_object_mut()
        .expect("receipt object")
        .insert("untrustedExtension".into(), serde_json::json!(true));

    let parsed = serde_json::from_value::<ReusableReceipt>(value);

    assert!(parsed.is_err(), "unknown receipt fields must fail closed");
}

#[test]
fn pre_persistence_schema_receipt_is_unknown_and_executes() {
    let current = context();
    let mut old = receipt("tests", true, &current);
    old.schema_version = 1;
    old.receipt_id = old.recompute_id().expect("recompute old receipt identity");

    let decision = decide_reuse(Some(&old), &current, "tests", NOW, false);

    assert_eq!(decision.state, ReuseState::Unknown);
    assert_eq!(decision.action, ReuseAction::Execute);
    assert_eq!(decision.reason, ReuseReason::ReceiptInvalid);
}
