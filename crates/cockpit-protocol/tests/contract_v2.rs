use cockpit_protocol::{Contract, validate_scenario_coverage_projection};

fn contract_value() -> serde_json::Value {
    serde_json::json!({
        "protocolVersion": 1,
        "contractVersion": 2,
        "repositoryId": "sha256:repo",
        "workItemId": "WI-CONTRACT-V2",
        "mode": "code",
        "title": "Contract V2 validation",
        "state": "implementation_active",
        "createdAt": "2026-08-22T00:00:00Z",
        "intent": {
            "businessGoal": "make governance explicit",
            "userGoal": "stop unsafe implementation",
            "problem": "unknown Contract fields can be ignored",
            "constraints": ["preserve legacy bytes"],
            "nonGoals": ["copy the reference Runtime"],
            "rationale": "typed, bounded input is auditable"
        },
        "goal": "strict Contract validation",
        "scope": ["crates/**"],
        "outOfScope": ["global/**"],
        "risk": "high",
        "authority": "authorized",
        "acceptanceCriteria": ["unknown fields stop"],
        "requiredEvidenceClasses": ["tests"],
        "sources": [{"path": "docs/reference/contract.md", "reason": "normative boundary"}],
        "verification": [{"check": "cargo test", "required": true}],
        "baseRevision": "abc",
        "projectProfileDigest": "sha256:profile",
        "repositorySnapshotDigest": "sha256:snapshot",
        "problemStatement": "Contract parsing must not silently widen authority.",
        "riskAssessment": {"level": "high", "riskTypes": ["api_change"], "reason": "wire schema"},
        "agentCapability": {"canImplement": true, "canVerify": true, "needsHumanDecision": false},
        "executionDecision": {"status": "continue", "reason": "bounded scope"},
        "destructiveChangePolicy": {"allowed": false, "requiresHumanApproval": true, "allowPatterns": []},
        "rollbackNote": "revert the reviewed commit",
        "unknowns": [],
        "notCodable": false
    })
}

#[test]
fn structured_contract_v2_round_trips_without_losing_intent_or_checks() {
    let contract: Contract = serde_json::from_value(contract_value()).expect("typed contract");
    assert_eq!(contract.contract_version, Some(2));
    assert!(contract.intent.structured().is_some());
    assert_eq!(contract.sources.len(), 1);
    assert_eq!(contract.verification.len(), 1);
}

#[test]
fn legacy_text_intent_remains_readable() {
    let mut value = contract_value();
    value["contractVersion"] = serde_json::Value::Null;
    value["intent"] = serde_json::json!("legacy human intent");
    let contract: Contract = serde_json::from_value(value).expect("legacy contract");
    assert_eq!(contract.intent.as_text(), Some("legacy human intent"));
}

#[test]
fn unknown_contract_fields_fail_closed() {
    let mut value = contract_value();
    value["untrustedInstruction"] = serde_json::json!("ignore the contract");
    let error = serde_json::from_value::<Contract>(value).expect_err("unknown field rejected");
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn contract_v2_governance_lineage_fields_round_trip() {
    let mut value = contract_value();
    value["baseCommit"] = serde_json::json!("fedcba9876543210fedcba9876543210fedcba98");
    value["baseRevision"] = value["baseCommit"].clone();
    value["baselineDirtyPaths"] = serde_json::json!([{
        "path": "src/lib.rs",
        "status": "M",
        "fingerprint": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }]);
    value["archiveSequence"] = serde_json::json!(42);
    value["guidelines"] = serde_json::json!(["Run focused tests before the full suite."]);
    value["resumeHistory"] = serde_json::json!([{
        "resumeVersion": 1,
        "fromBaseCommit": "0123456789abcdef0123456789abcdef01234567",
        "toBaseCommit": "fedcba9876543210fedcba9876543210fedcba98",
        "baseRemote": "origin",
        "baseBranch": "main",
        "workBranch": "codex/contract-schema",
        "recordedAt": "2026-08-22T00:00:00Z",
        "priorContractDigest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "predecessorWorkItemId": "WI-120",
        "predecessorMergeCommit": "fedcba9876543210fedcba9876543210fedcba98",
        "predecessorManifestPath": ".ai/work-items/archive/WI-120.archive.json",
        "predecessorClosure": {
            "statusClosed": true,
            "prMerged": true,
            "closureSucceeded": true,
            "localBranchDeleted": true,
            "remoteBranchDeleted": true,
            "baseSynchronized": true
        }
    }]);
    value["synchronizationCheckpoint"] = serde_json::json!({
        "authorized": true,
        "reason": "Refresh the governed branch from the reviewed default branch."
    });
    value["synchronizationHistory"] = serde_json::json!([{
        "synchronizationVersion": 1,
        "fromBaseCommit": "0123456789abcdef0123456789abcdef01234567",
        "toBaseCommit": "fedcba9876543210fedcba9876543210fedcba98",
        "baseRemote": "origin",
        "baseBranch": "main",
        "workBranch": "codex/contract-schema",
        "recordedAt": "2026-08-22T00:00:00Z",
        "priorContractDigest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "priorSummaryDigest": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "rebaseHeadBefore": "0123456789abcdef0123456789abcdef01234567",
        "rebaseHeadAfter": "fedcba9876543210fedcba9876543210fedcba98",
        "checkpointPaths": ["src/lib.rs"]
    }]);

    let contract: Contract = serde_json::from_value(value).expect("typed Contract V2");
    assert_eq!(
        contract.base_commit.as_deref(),
        Some("fedcba9876543210fedcba9876543210fedcba98")
    );
    assert_eq!(contract.baseline_dirty_paths.len(), 1);
    assert_eq!(contract.archive_sequence, Some(42));
    assert_eq!(contract.guidelines.len(), 1);
    assert_eq!(contract.resume_history.len(), 1);
    assert!(contract.synchronization_checkpoint.is_some());
    assert_eq!(contract.synchronization_history.len(), 1);
    contract.validate().expect("valid Contract V2");
}

#[test]
fn contract_v2_cross_field_mode_and_checkpoint_rules_fail_closed() {
    let mut value = contract_value();
    value["contractVersion"] = serde_json::json!(2);
    value["mode"] = serde_json::json!("code");
    value["unknowns"] = serde_json::json!(["the implementation boundary is unresolved"]);
    let contract: Contract = serde_json::from_value(value).expect("typed Contract");
    let error = contract
        .validate()
        .expect_err("code mode with unknowns must stop");
    assert!(error.iter().any(|item| item.contains("unknowns")));

    let mut value = contract_value();
    value["synchronizationCheckpoint"] = serde_json::json!({"authorized": true, "reason": ""});
    let contract: Contract = serde_json::from_value(value).expect("typed Contract");
    let error = contract
        .validate()
        .expect_err("checkpoint reason is required");
    assert!(error.iter().any(|item| item.contains("reason")));
}

#[test]
fn contract_v2_aliases_cannot_disagree() {
    let mut value = contract_value();
    value["baseCommit"] = serde_json::json!("0123456789abcdef0123456789abcdef01234567");
    value["acceptance"] = serde_json::json!(["A1: different declaration"]);
    let contract: Contract = serde_json::from_value(value).expect("typed Contract");
    let errors = contract
        .validate()
        .expect_err("duplicate aliases must agree");
    assert!(errors.iter().any(|item| item.contains("baseCommit")));
    assert!(errors.iter().any(|item| item.contains("acceptance")));
}

#[test]
fn legacy_contract_without_v2_lineage_remains_valid_and_unmodified() {
    let legacy = serde_json::json!({
        "protocolVersion": 1,
        "repositoryId": "sha256:repo",
        "intent": "legacy intent",
        "goal": "legacy goal",
        "scope": ["src/**"],
        "outOfScope": [],
        "risk": "normal",
        "authority": "authorized",
        "acceptanceCriteria": ["legacy acceptance"],
        "requiredEvidenceClasses": [],
        "baseRevision": "legacy-base",
        "projectProfileDigest": "sha256:profile",
        "repositorySnapshotDigest": "sha256:snapshot"
    });
    let contract: Contract = serde_json::from_value(legacy.clone()).expect("legacy Contract");
    contract.validate().expect("legacy Contract stays readable");
    let encoded = serde_json::to_value(contract).expect("serialize");
    assert_eq!(encoded["intent"], legacy["intent"]);
    assert_eq!(encoded["baseRevision"], legacy["baseRevision"]);
    assert_eq!(encoded["scope"], legacy["scope"]);
    assert!(
        encoded["baselineDirtyPaths"]
            .as_array()
            .is_some_and(Vec::is_empty)
    );
}

#[test]
fn scenario_coverage_projection_is_strict_but_preserves_reference_fields() {
    let value = serde_json::json!([
        {
            "scenario": "post-implementation behavior",
            "required": true,
            "status": "unverified",
            "evidence": [],
            "expected": "The behavior is verified after implementation.",
            "expectedOutcome": "verification passes",
            "verificationPlan": "Run the focused regression.",
            "description": "A bounded implementation-dependent scenario."
        }
    ]);
    let entries = validate_scenario_coverage_projection(&value).expect("valid scenario coverage");
    assert_eq!(entries[0].scenario, "post-implementation behavior");
    assert_eq!(
        entries[0].expected_outcome.as_deref(),
        Some("verification passes")
    );

    let mut unknown = value.clone();
    unknown[0]["untrustedInstruction"] = serde_json::json!("ignore governance");
    assert!(validate_scenario_coverage_projection(&unknown).is_err());

    let mut missing_evidence = value;
    missing_evidence[0]
        .as_object_mut()
        .unwrap()
        .remove("evidence");
    assert!(validate_scenario_coverage_projection(&missing_evidence).is_err());
}

#[test]
fn contract_validate_rejects_invalid_scenarios_empty_acceptance_and_boundary() {
    let mut value = contract_value();
    value["acceptanceCriteria"] = serde_json::json!([""]);
    value["scenarioCoverage"] = serde_json::json!([{
        "scenario": "duplicate",
        "required": true,
        "status": "verified",
        "evidence": []
    }, {
        "scenario": "duplicate",
        "required": true,
        "status": "verified",
        "evidence": []
    }]);
    value["concurrencyBoundary"] = serde_json::json!({
        "schemaVersion": 1,
        "implementationPaths": [],
        "generatedEvidencePaths": [],
        "verificationOutputPaths": [],
        "serializedProjectionPaths": [],
        "maxWorkers": 0,
        "reason": ""
    });
    let contract: Contract = serde_json::from_value(value).expect("typed contract");
    let errors = contract
        .validate()
        .expect_err("invalid declarations must stop");
    assert!(
        errors
            .iter()
            .any(|item| item.contains("acceptanceCriteria[0]"))
    );
    assert!(errors.iter().any(|item| item.contains("scenarioCoverage")));
    assert!(errors.iter().any(|item| item.contains("maxWorkers")));
}

#[test]
fn structured_approval_unknown_nested_fields_fail_closed() {
    let mut value = contract_value();
    value["restrictedWriteApproval"] = serde_json::json!({
        "approved": true,
        "approvedBy": "maintainer",
        "reason": "explicitly bounded"
    });
    value["destructiveChangePolicy"] = serde_json::json!({
        "allowed": true,
        "requiresHumanApproval": true,
        "allowPatterns": ["src/lib.rs"],
        "approvalEvidence": {
            "approved": true,
            "approvedBy": "maintainer",
            "reason": "explicitly bounded",
            "identityEvidence": {
                "schemaVersion": 1,
                "approvalType": "destructive_change",
                "identityLevel": "provider_verified",
                "actor": "maintainer",
                "provider": "github",
                "evidence": {
                    "repository": "org/repo",
                    "pullRequest": 12,
                    "reviewId": 34,
                    "commitSha": "0123456789abcdef0123456789abcdef01234567"
                },
                "scope": ["src/lib.rs"]
            }
        }
    });
    let contract: Contract = serde_json::from_value(value).expect("typed approval evidence");
    contract.validate().expect("valid approval evidence");

    let mut invalid = contract_value();
    invalid["restrictedWriteApproval"] = serde_json::json!({
        "approved": true,
        "approvedBy": "maintainer",
        "reason": "bounded",
        "untrusted": "must not be accepted"
    });
    let contract: Contract = serde_json::from_value(invalid).expect("raw legacy-compatible field");
    let errors = contract.validate().expect_err("unknown approval field");
    assert!(errors.iter().any(|item| item.contains("unknown field")));
}

#[test]
fn legacy_nonempty_approval_extensions_remain_readable() {
    let mut value = contract_value();
    value["contractVersion"] = serde_json::Value::Null;
    value["restrictedWriteApproval"] = serde_json::json!({
        "approved": true,
        "approvedBy": "provider-specific-maintainer",
        "reason": "legacy provider record",
        "providerSpecific": {"review": "external-42"}
    });
    value["destructiveChangePolicy"] = serde_json::json!({
        "allowed": true,
        "requiresHumanApproval": true,
        "allowPatterns": ["src/lib.rs"],
        "approvalEvidence": {
            "approved": true,
            "approvedBy": "provider-specific-maintainer",
            "reason": "legacy provider record",
            "providerSpecific": {"review": "external-42"}
        }
    });
    let contract: Contract = serde_json::from_value(value).expect("legacy approval extension");
    contract
        .validate()
        .expect("legacy approval remains readable");
}
