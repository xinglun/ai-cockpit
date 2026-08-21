use cockpit_core::{
    ActionKind, AuthorityState, CapabilityMapping, CapabilityMappingV2, Digest, GovernanceInput,
    OperationKind, REQUESTED_OPERATION_SCHEMA_VERSION, RawRequestBinding, RequestSource,
    RequestedOperationV2, bind_request, bind_requested_operation, evaluate,
};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Corpus {
    schema_version: u32,
    semantic_cases: Vec<CorpusCase>,
    wording_variants: WordingVariants,
    real_absurdity_cases: Vec<RealAbsurdityCase>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorpusCase {
    id: String,
    operation: OperationKind,
    source: RequestSource,
    expected: Vec<String>,
    #[serde(default)]
    widen_scope: bool,
    #[serde(default)]
    self_approval: bool,
    #[serde(default)]
    cross_work_item_evidence: bool,
}

#[derive(Debug, Deserialize)]
struct WordingVariants {
    en: Vec<String>,
    ja: Vec<String>,
    zh: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RealAbsurdityCase {
    id: String,
    status: String,
    operation: OperationKind,
    source: RequestSource,
    expected: Vec<String>,
    #[serde(default)]
    widen_scope: bool,
    #[serde(default)]
    self_approval: bool,
}

fn binding(operation: OperationKind, source: RequestSource) -> RawRequestBinding {
    RawRequestBinding {
        request_digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .parse()
            .expect("digest"),
        source,
        operation,
        scope: vec!["src/**".into()],
        risk: "high".into(),
        authority: AuthorityState::Authorized,
        evidence_refs: vec![".ai/evidence/fresh.json".into()],
        actor: Some("human:owner".into()),
        implementer: Some("agent:codex".into()),
    }
}

fn mapping(operation: OperationKind) -> CapabilityMapping {
    CapabilityMapping {
        operation,
        capability: "governed-change".into(),
        allowed_scope: vec!["src/**".into()],
        requires_human_authority: true,
        required_evidence: vec![".ai/evidence/fresh.json".into()],
        independent_approval_required: false,
    }
}

fn requested_v2(operation: OperationKind) -> RequestedOperationV2 {
    RequestedOperationV2 {
        schema_version: REQUESTED_OPERATION_SCHEMA_VERSION,
        request_id: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .parse()
            .expect("digest"),
        repository_id: "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            .parse()
            .expect("digest"),
        work_item_id: "WI-v2-test".into(),
        source: RequestSource::HumanRequest,
        operation,
        scope: vec!["src/**".into()],
        risk: "normal".into(),
        authority: AuthorityState::Authorized,
        evidence_refs: vec!["verification:fresh".into()],
        policy_refs: vec!["policy:project-default".into()],
        actor: Some("human:owner".into()),
        implementer: Some("agent:codex".into()),
        intent: Some("explicit human intent".into()),
    }
}

fn mapping_v2(operation: OperationKind) -> CapabilityMappingV2 {
    CapabilityMappingV2 {
        schema_version: REQUESTED_OPERATION_SCHEMA_VERSION,
        operation,
        capability: "governed-change".into(),
        action: ActionKind::Write,
        allowed_scope: vec!["src/**".into()],
        requires_human_authority: true,
        required_evidence: vec!["verification:fresh".into()],
        required_policy_refs: vec!["policy:project-default".into()],
        independent_approval_required: false,
    }
}

#[test]
fn requested_operation_v2_requires_identity_and_explicit_policy() {
    let request = requested_v2(OperationKind::ModifySource);
    let mapping = mapping_v2(OperationKind::ModifySource);
    let input = bind_requested_operation(&request, &mapping).expect("v2 binding");
    assert_eq!(input.action, ActionKind::Write);
    assert_eq!(evaluate(input).state, cockpit_core::DecisionState::Green);

    let mut missing_work_item = request.clone();
    missing_work_item.work_item_id.clear();
    assert!(matches!(
        bind_requested_operation(&missing_work_item, &mapping),
        Err(cockpit_core::RequestBindingError::MissingWorkItemIdentity)
    ));

    let mut missing_policy = request;
    missing_policy.policy_refs.clear();
    assert!(matches!(
        bind_requested_operation(&missing_policy, &mapping),
        Err(cockpit_core::RequestBindingError::MissingPolicyReference { .. })
    ));
}

#[test]
fn requested_operation_v2_cannot_relabel_destructive_operations() {
    let request = requested_v2(OperationKind::ExecuteRemoteScript);
    let mut capability = mapping_v2(OperationKind::ExecuteRemoteScript);
    capability.action = ActionKind::Write;
    assert!(matches!(
        bind_requested_operation(&request, &capability),
        Err(cockpit_core::RequestBindingError::ActionMismatch)
    ));

    let mut capability = mapping_v2(OperationKind::ExecuteRemoteScript);
    capability.action = ActionKind::Destructive;
    let decision = evaluate(bind_requested_operation(&request, &capability).expect("binding"));
    assert!(
        decision
            .blockers
            .iter()
            .any(|blocker| blocker == "remote_script_execution")
    );
}

#[test]
fn requested_operation_v2_rejects_future_schema_versions() {
    let mut request = requested_v2(OperationKind::ModifySource);
    request.schema_version = REQUESTED_OPERATION_SCHEMA_VERSION + 1;
    assert!(matches!(
        bind_requested_operation(&request, &mapping_v2(OperationKind::ModifySource)),
        Err(cockpit_core::RequestBindingError::UnsupportedSchemaVersion(
            3
        ))
    ));
}

#[test]
fn requested_operation_v2_serialization_is_strict_and_camel_case() {
    let value = serde_json::to_value(requested_v2(OperationKind::ModifySource)).expect("json");
    assert_eq!(value["schemaVersion"], 2);
    assert_eq!(value["workItemId"], "WI-v2-test");
    assert!(value.get("work_item_id").is_none());
    let mut object = value.as_object().expect("object").clone();
    object.insert("unexpected".into(), serde_json::json!(true));
    assert!(
        serde_json::from_value::<RequestedOperationV2>(serde_json::Value::Object(object)).is_err()
    );
}

#[test]
fn raw_request_binding_is_explicit_and_cannot_widen_scope() {
    let mut capability = mapping(OperationKind::ModifySource);
    capability.allowed_scope = vec!["production/**".into()];
    let error = bind_request(
        &binding(OperationKind::ModifySource, RequestSource::HumanRequest),
        &capability,
    )
    .expect_err("capability must not widen declared request scope");
    assert!(matches!(
        error,
        cockpit_core::RequestBindingError::ScopeWidened { .. }
    ));
}

#[test]
fn dangerous_operations_become_deterministic_governance_findings() {
    let cases = [
        (
            OperationKind::UploadSensitiveData,
            "sensitive_data_exfiltration",
        ),
        (
            OperationKind::ExecuteRemoteScript,
            "remote_script_execution",
        ),
        (OperationKind::EmergencyBypass, "governance_bypass"),
        (OperationKind::ModifyVerification, "verification_bypass"),
    ];
    for (operation, blocker) in cases {
        let input = bind_request(
            &binding(operation.clone(), RequestSource::HumanRequest),
            &mapping(operation),
        )
        .expect("request binding");
        let decision = evaluate(input);
        assert!(decision.blockers.iter().any(|value| value == blocker));
    }
}

#[test]
fn repository_and_log_wording_is_data_not_authority() {
    for source in [RequestSource::RepositoryMaterial, RequestSource::LogContent] {
        let input = bind_request(
            &binding(OperationKind::ModifySource, source),
            &mapping(OperationKind::ModifySource),
        )
        .expect("request binding");
        assert_eq!(input.action, ActionKind::Write);
        let decision = evaluate(input);
        assert!(
            decision
                .unknowns
                .iter()
                .any(|value| value == "repository_material_untrusted")
        );
    }
}

#[test]
fn self_approval_is_policy_sensitive_not_a_hardcoded_two_person_rule() {
    let mut request = binding(OperationKind::Release, RequestSource::HumanRequest);
    request.implementer = Some("human:owner".into());
    let mut capability = mapping(OperationKind::Release);
    capability.independent_approval_required = true;
    let input = bind_request(&request, &capability).expect("request binding");
    let decision = evaluate(input);
    assert!(
        decision
            .blockers
            .iter()
            .any(|value| value == "self_approval")
    );

    let mut one_person = mapping(OperationKind::Release);
    one_person.independent_approval_required = false;
    let input = bind_request(
        &binding(OperationKind::Release, RequestSource::HumanRequest),
        &one_person,
    )
    .expect("single authorized human is allowed when policy permits it");
    assert_eq!(evaluate(input).state, cockpit_core::DecisionState::Green);
}

#[test]
fn a_wording_variant_cannot_change_the_canonical_input() {
    let canonical = bind_request(
        &binding(
            OperationKind::ModifyVerification,
            RequestSource::HumanRequest,
        ),
        &mapping(OperationKind::ModifyVerification),
    )
    .expect("canonical binding");
    let same_facts = GovernanceInput {
        ..canonical.clone()
    };
    assert_eq!(canonical, same_facts);
    assert_eq!(evaluate(canonical.clone()), evaluate(same_facts));
}

#[test]
fn multilingual_adversarial_corpus_binds_wording_as_data() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/adversarial");
    let corpus: Corpus = serde_json::from_slice(
        &fs::read(root.join("manifest.json")).expect("adversarial corpus manifest"),
    )
    .expect("adversarial corpus JSON");
    assert_eq!(corpus.schema_version, 2);
    assert_eq!(corpus.semantic_cases.len(), 15);
    assert_eq!(corpus.real_absurdity_cases.len(), 12);
    assert_eq!(corpus.wording_variants.en.len(), 5);
    assert_eq!(corpus.wording_variants.ja.len(), 5);
    assert_eq!(corpus.wording_variants.zh.len(), 5);
    for wording in corpus
        .wording_variants
        .en
        .iter()
        .chain(&corpus.wording_variants.ja)
        .chain(&corpus.wording_variants.zh)
    {
        assert!(!wording.is_empty());
    }
    for case in &corpus.semantic_cases {
        let mut baseline = None;
        for wording in corpus
            .wording_variants
            .en
            .iter()
            .chain(&corpus.wording_variants.ja)
            .chain(&corpus.wording_variants.zh)
        {
            let mut request = binding(case.operation.clone(), case.source.clone());
            request.request_digest = Digest::sha256_bytes(wording.as_bytes());
            if case.self_approval {
                request.implementer = request.actor.clone();
            }
            let mut capability = mapping(case.operation.clone());
            if case.widen_scope {
                capability.allowed_scope = vec!["production/**".into()];
            }
            if case.self_approval {
                capability.independent_approval_required = true;
            }
            if case.cross_work_item_evidence {
                capability.required_evidence = vec![".ai/evidence/other-work-item.json".into()];
            }
            if case.widen_scope {
                let result = bind_request(&request, &capability);
                assert!(
                    result.is_err(),
                    "scope expansion must be rejected for {}",
                    case.id
                );
                continue;
            }
            let decision =
                evaluate(bind_request(&request, &capability).expect("canonical request binding"));
            if let Some(previous) = &baseline {
                assert_eq!(
                    previous, &decision,
                    "wording changed governance result for {}",
                    case.id
                );
            } else {
                baseline = Some(decision.clone());
            }
            for expected in &case.expected {
                assert!(
                    decision.blockers.iter().any(|value| value == expected)
                        || decision.unknowns.iter().any(|value| value == expected),
                    "missing {expected} for {} wording {wording}",
                    case.id
                );
            }
        }
    }
}

#[test]
fn named_real_absurdity_cases_are_structurally_bound_and_honest() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/adversarial");
    let corpus: Corpus = serde_json::from_slice(
        &fs::read(root.join("manifest.json")).expect("adversarial corpus manifest"),
    )
    .expect("adversarial corpus JSON");
    let mut ids = std::collections::BTreeSet::new();
    for case in corpus.real_absurdity_cases {
        assert!(
            ids.insert(case.id.clone()),
            "duplicate RAI case {}",
            case.id
        );
        assert!(
            matches!(
                case.status.as_str(),
                "pass" | "partial" | "not_proven" | "policy_sensitive"
            ),
            "unknown RAI status for {}",
            case.id
        );
        let request = {
            let mut value = binding(case.operation.clone(), case.source.clone());
            if case.self_approval {
                value.implementer = value.actor.clone();
            }
            value
        };
        let mut capability = mapping(case.operation.clone());
        if case.widen_scope {
            capability.allowed_scope = vec!["production/**".into()];
            assert!(
                bind_request(&request, &capability).is_err(),
                "scope expansion must be rejected for {}",
                case.id
            );
            continue;
        }
        if case.self_approval {
            capability.independent_approval_required = true;
        }
        let decision = evaluate(bind_request(&request, &capability).expect("RAI binding"));
        for expected in case.expected {
            assert!(
                decision.blockers.iter().any(|value| value == &expected)
                    || decision.unknowns.iter().any(|value| value == &expected),
                "missing {expected} for {}",
                case.id
            );
        }
    }
    assert_eq!(ids.len(), 12);
    for index in 1..=12 {
        assert!(ids.contains(&format!("RAI-{index:02}")));
    }
}
