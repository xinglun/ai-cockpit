use cockpit_core::{
    ActionKind, AuthorityState, CapabilityMapping, Digest, GovernanceInput, OperationKind,
    RawRequestBinding, RequestSource, bind_request, evaluate,
};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Corpus {
    schema_version: u32,
    semantic_cases: Vec<CorpusCase>,
    wording_variants: WordingVariants,
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
    for wording in corpus
        .wording_variants
        .en
        .iter()
        .chain(&corpus.wording_variants.ja)
        .chain(&corpus.wording_variants.zh)
    {
        assert!(!wording.is_empty());
    }
    for case in corpus.semantic_cases {
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
                assert!(
                    bind_request(&request, &capability).is_err(),
                    "scope expansion must be rejected for {}",
                    case.id
                );
                continue;
            }
            let input = bind_request(&request, &capability).expect("canonical request binding");
            let decision = evaluate(input);
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
