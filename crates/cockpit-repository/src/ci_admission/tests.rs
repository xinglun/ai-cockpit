use super::github::{self, Failure, MetadataReader, Reply};
use cockpit_protocol::ci_admission::GithubEvidenceSelector;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

struct FixtureReader {
    replies: VecDeque<(String, Reply)>,
    seen: Vec<String>,
}
impl MetadataReader for FixtureReader {
    fn read(&mut self, path: &str, _: bool, _: Instant) -> Result<Reply, Failure> {
        self.seen.push(path.into());
        let (expected, reply) = self
            .replies
            .pop_front()
            .expect("unexpected provider request");
        assert_eq!(path, expected);
        Ok(reply)
    }
}

fn selector() -> GithubEvidenceSelector {
    GithubEvidenceSelector {
        run_id: 1,
        attempt: 2,
        job_id: 3,
        artifact_id: 4,
    }
}

#[test]
fn ci_admission_duplicate_link_fields_cannot_hide_a_later_next_relation() {
    use ureq::http::{HeaderMap, HeaderValue};
    let mut headers = HeaderMap::new();
    assert_eq!(github::single_link_header(&headers), Ok(None));
    headers.append("link", HeaderValue::from_static("<https://api.github.com/repos/xinglun/ai-cockpit/actions/runs/1/attempts/2/jobs?per_page=100&page=1>; rel=\"prev\""));
    assert!(github::single_link_header(&headers).unwrap().is_some());
    headers.append("link", HeaderValue::from_static("<https://api.github.com/repos/xinglun/ai-cockpit/actions/runs/1/attempts/2/jobs?per_page=100&page=3>; rel=\"next\""));
    assert_eq!(
        github::single_link_header(&headers),
        Err(Failure::Pagination)
    );
}

fn fixture() -> FixtureReader {
    let repo = json!({"id":1341189248,"full_name":"xinglun/ai-cockpit"});
    let run = json!({"id":1,"run_attempt":2,"event":"push","path":".github/workflows/ci.yml","workflow_id":338986437,"head_sha":"a".repeat(40),"head_branch":"main","status":"completed","conclusion":"success","repository":repo,"head_repository":repo});
    let jobs = json!({"total_count":1,"jobs":[{"id":3,"run_id":1,"run_attempt":2,"name":"quality","status":"completed","conclusion":"success","steps":[]}]});
    let artifact = json!({"id":4,"name":"repository-gate-receipts","expired":false,"digest":format!("sha256:{}","b".repeat(64)),"workflow_run":{"id":1,"repository_id":1341189248,"head_repository_id":1341189248,"head_sha":"a".repeat(40)}});
    let prefix = "/repos/xinglun/ai-cockpit";
    let paths = [
        format!("{prefix}/actions/runs/1/attempts/2"),
        format!("{prefix}/actions/runs/1/attempts/2/jobs?per_page=100&page=1"),
        format!("{prefix}/actions/runs/1/artifacts?per_page=100&page=1"),
        format!("{prefix}/actions/artifacts/4"),
        format!(
            "{prefix}/contents/.github/workflows/ci.yml?ref={}",
            "a".repeat(40)
        ),
    ];
    let bodies = [
        serde_json::to_vec(&run).unwrap(),
        serde_json::to_vec(&jobs).unwrap(),
        serde_json::to_vec(&json!({"total_count":1,"artifacts":[artifact.clone()]})).unwrap(),
        serde_json::to_vec(&artifact).unwrap(),
        b"workflow bytes from fixed origin".to_vec(),
    ];
    FixtureReader {
        replies: paths
            .into_iter()
            .zip(bodies)
            .map(|(p, b)| {
                (
                    p,
                    Reply {
                        status: 200,
                        link: None,
                        body: b,
                    },
                )
            })
            .collect(),
        seen: Vec::new(),
    }
}

#[test]
fn ci_admission_complete_metadata_collection_stays_non_admitting_and_has_real_missing_proofs() {
    let mut reader = fixture();
    let p = github::collect_with(&selector(), &mut reader, Duration::from_secs(2));
    assert!(p.collection.complete, "{:?}", p.collection.failure);
    assert_eq!(reader.seen.len(), 5);
    let metadata = p.metadata.unwrap();
    assert_eq!(metadata["admission"], "not_attempted");
    assert_eq!(metadata["nodeEvaluation"], "not_performed");
    assert!(
        metadata["gaps"]
            .as_array()
            .unwrap()
            .contains(&json!("artifact_job_attempt_binding_unavailable"))
    );
    assert!(
        metadata["gaps"]
            .as_array()
            .unwrap()
            .contains(&json!("producer_runtime_unverified"))
    );
    assert_eq!(p.collection.requests.len(), 5);
    assert!(p.diagnostic_gaps.is_empty());
    assert!(!p.rejected);
}

#[test]
fn ci_admission_wrong_attempt_and_artifact_endpoint_conflict_are_rejected() {
    let mut reader = fixture();
    let mut value: Value = serde_json::from_slice(&reader.replies[1].1.body).unwrap();
    value["jobs"][0]["run_attempt"] = 9.into();
    reader.replies[1].1.body = serde_json::to_vec(&value).unwrap();
    let p = github::collect_with(&selector(), &mut reader, Duration::from_secs(2));
    assert!(p.rejected);
    assert_eq!(p.metadata.unwrap()["metadataConsistent"], false);
    let mut reader = fixture();
    let mut value: Value = serde_json::from_slice(&reader.replies[3].1.body).unwrap();
    value["workflow_run"]["id"] = 99.into();
    reader.replies[3].1.body = serde_json::to_vec(&value).unwrap();
    let p = github::collect_with(&selector(), &mut reader, Duration::from_secs(2));
    assert!(!p.collection.complete);
    assert_eq!(
        p.collection.failure.as_deref(),
        Some("provider_metadata_invalid")
    );
}

#[test]
fn ci_admission_foreign_link_loop_duplicate_next_and_incomplete_counts_fail_closed() {
    for link in [
        "<https://evil.invalid/jobs?page=2&per_page=100>; rel=\"next\"",
        "<https://api.github.com/repos/xinglun/ai-cockpit/actions/runs/1/attempts/2/jobs?per_page=100&page=1>; rel=\"next\"",
        "<https://api.github.com/repos/xinglun/ai-cockpit/actions/runs/1/attempts/2/jobs?per_page=100&page=2>; rel=\"next\", <https://api.github.com/repos/xinglun/ai-cockpit/actions/runs/1/attempts/2/jobs?per_page=100&page=2>; rel=\"next\"",
    ] {
        let mut r = fixture();
        r.replies[1].1.link = Some(link.into());
        let p = github::collect_with(&selector(), &mut r, Duration::from_secs(2));
        assert!(!p.collection.complete);
        assert_eq!(
            p.collection.failure.as_deref(),
            Some("provider_pagination_unverified")
        );
        assert_eq!(r.seen.len(), 2);
    }
    let mut r = fixture();
    let mut value: Value = serde_json::from_slice(&r.replies[1].1.body).unwrap();
    value["total_count"] = 2.into();
    r.replies[1].1.body = serde_json::to_vec(&value).unwrap();
    let p = github::collect_with(&selector(), &mut r, Duration::from_secs(2));
    assert!(!p.collection.complete);
    assert_eq!(
        p.collection.failure.as_deref(),
        Some("provider_pagination_unverified")
    );
}

#[test]
fn ci_admission_source_booleans_cannot_promote_the_plan_or_remove_missing_proofs() {
    let mut r = fixture();
    let mut v: Value = serde_json::from_slice(&r.replies[0].1.body).unwrap();
    v["providerAuthenticated"] = true.into();
    v["producerRuntimeVerified"] = true.into();
    r.replies[0].1.body = serde_json::to_vec(&v).unwrap();
    let p = github::collect_with(&selector(), &mut r, Duration::from_secs(2));
    assert!(p.collection.complete);
    let metadata = p.metadata.unwrap();
    assert_eq!(metadata["admission"], "not_attempted");
    assert!(
        metadata["gaps"]
            .as_array()
            .unwrap()
            .contains(&json!("producer_runtime_unverified"))
    );
}

#[test]
fn ci_admission_non_success_and_oversized_responses_never_complete() {
    let mut r = fixture();
    r.replies[0].1.status = 403;
    let p = github::collect_with(&selector(), &mut r, Duration::from_secs(2));
    assert_eq!(
        p.collection.failure.as_deref(),
        Some("provider_http_status_403")
    );
    assert!(has_tag(
        &p.diagnostic_gaps,
        "provider_diagnostic_source_collector_http_status"
    ));
    assert!(has_tag(
        &p.diagnostic_gaps,
        "provider_diagnostic_http_status_403_observed"
    ));
    assert!(p.collection.requests.is_empty());
    assert!(p.metadata.is_none());
    assert!(!p.collection.complete);
    let mut r = fixture();
    r.replies[0].1.body = vec![b' '; 8 * 1024 * 1024 + 1];
    let p = github::collect_with(&selector(), &mut r, Duration::from_secs(2));
    assert_eq!(
        p.collection.failure.as_deref(),
        Some("provider_response_size_limit")
    );
    assert!(!p.collection.complete);
}

#[test]
fn ci_admission_final_deadline_rejects_success_and_prior_error_after_projection() {
    let expired = Instant::now();
    assert_eq!(
        github::collection_result_before_deadline(Ok(()), expired),
        Err(Failure::Deadline)
    );
    assert_eq!(
        github::collection_result_before_deadline::<()>(Err(Failure::Metadata), expired),
        Err(Failure::Deadline)
    );
    assert_eq!(
        github::collection_result_before_deadline(Ok(()), Instant::now() + Duration::from_secs(60)),
        Ok(())
    );
    let mut reader = fixture();
    let plan = github::collect_with(&selector(), &mut reader, Duration::ZERO);
    assert_eq!(
        plan.collection.failure.as_deref(),
        Some("provider_collection_deadline")
    );
    assert!(reader.seen.is_empty());
    assert!(!plan.collection.complete);
}

const SERVICE_WI: &str = "WI-CI-ADMISSION-AUTHORITY";

fn service_fixture() -> (
    tempfile::TempDir,
    cockpit_protocol::RuntimeContext,
    cockpit_protocol::TaskAuthorizationCaller,
) {
    use crate::*;
    let root = tempfile::Builder::new()
        .prefix("cockpit ci admission ")
        .tempdir()
        .unwrap();
    std::fs::write(root.path().join("tracked.txt"), "before\n").unwrap();
    for args in [
        vec!["init", "-q", "--initial-branch=main"],
        vec!["config", "core.autocrlf", "false"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    ] {
        assert!(
            std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .status()
                .unwrap()
                .success()
        );
    }
    attach(root.path()).unwrap();
    confirm_profile_update(root.path(), "cargo", &["test".into()]).unwrap();
    let runtime = cockpit_protocol::RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"ci-admission-authority-fixture"),
    };
    let caller = cockpit_protocol::TaskAuthorizationCaller {
        schema_version: 1,
        actor: "agent:ci-admission-fixture".into(),
        authority_source: "fixture:synthetic-owner-not-provider-or-human-authentication".into(),
        assurance: cockpit_protocol::AssuranceLevel::SelfDeclared,
    };
    start_work_item_with_options_and_runtime(
        root.path(),
        SERVICE_WI,
        "observe metadata",
        "keep current caller authority",
        &["tracked.txt".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["collection cannot outlive current caller authority".into()],
            verification_commands: vec!["cargo test".into()],
            task_authorization: Some(cockpit_protocol::TaskAuthorizationPolicy {
                schema_version: 1,
                agent_actor: caller.actor.clone(),
                authority_source: caller.authority_source.clone(),
                allowed_actions: vec![
                    "run_preflight".into(),
                    "checkpoint".into(),
                    "run_verification".into(),
                ],
                closeout_actions: vec![],
                expires_at: Some("2099-01-01T00:00:00Z".into()),
                public_target: None,
            }),
            ..Default::default()
        },
        &runtime,
    )
    .unwrap();
    let contract_path = root
        .path()
        .join(format!(".ai/work-items/active/{SERVICE_WI}.contract.json"));
    preflight_work_item_with_runtime_report(root.path(), &contract_path, &runtime).unwrap();
    let contract: Value = crate::read_json(&contract_path).unwrap();
    let summary: Value = crate::read_json(
        &root
            .path()
            .join(format!(".ai/work-items/active/{SERVICE_WI}.summary.json")),
    )
    .unwrap();
    record_work_item_governance_controls_with_runtime(root.path(), SERVICE_WI, &json!({
        "decisionEvidence": {
            "schemaVersion": 1, "decisionId": "contract-preflight-review", "decision": "confirm_review",
            "workItemId": SERVICE_WI, "repositoryId": crate::repository_id(root.path()),
            "contractDigest": cockpit_protocol::digest_json(&contract).unwrap(),
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": chrono::Utc::now().to_rfc3339(), "recordedBy": "agent:test-recorder",
            "reason": "Synthetic local fixture; not external human or provider authentication."
        }
    }), &runtime).unwrap();
    checkpoint_work_item_with_caller(root.path(), SERVICE_WI, &runtime, Some(&caller)).unwrap();
    (root, runtime, caller)
}

#[test]
fn ci_admission_wrong_caller_is_rejected_before_collection() {
    let (root, runtime, mut caller) = service_fixture();
    caller.authority_source.push_str("-foreign");
    let calls = std::cell::Cell::new(0);
    let error = super::plan_with_collection(
        root.path(),
        SERVICE_WI,
        &selector(),
        &runtime,
        Some(&caller),
        |_| {
            calls.set(calls.get() + 1);
            github::collect_with(&selector(), &mut fixture(), Duration::from_secs(2))
        },
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("task_authorization_caller_mismatch"),
        "{error}"
    );
    assert_eq!(calls.get(), 0);
}

#[test]
fn ci_admission_withdrawal_during_collection_cannot_publish_a_plan() {
    let (root, runtime, caller) = service_fixture();
    let grant = crate::work_item_status_snapshot_with_runtime(root.path(), SERVICE_WI, &runtime)
        .unwrap()
        .task_authorization
        .unwrap()
        .grant_id;
    let calls = std::cell::Cell::new(0);
    let error = super::plan_with_collection(root.path(), SERVICE_WI, &selector(), &runtime, Some(&caller), |_| {
        calls.set(calls.get() + 1);
        let collected = github::collect_with(&selector(), &mut fixture(), Duration::from_secs(2));
        assert!(collected.collection.complete);
        crate::record_work_item_governance_controls_with_runtime(root.path(), SERVICE_WI, &json!({
            "taskAuthorizationTermination": {"schemaVersion": 1, "grantId": grant,
                "decision": "withdrawn", "caller": caller, "reason": "fixture withdrawal during metadata collection"}
        }), &runtime).unwrap();
        collected
    }).unwrap_err();
    assert_eq!(calls.get(), 1);
    assert!(error.to_string().contains("withdrawn"), "{error}");
    assert!(
        !root
            .path()
            .join(format!(".ai/evidence/{SERVICE_WI}.verification.json"))
            .exists()
    );
}

#[test]
fn ci_admission_source_drift_is_an_explicit_gap_and_never_acceptance() {
    let (root, runtime, caller) = service_fixture();
    let plan = super::plan_with_collection(
        root.path(),
        SERVICE_WI,
        &selector(),
        &runtime,
        Some(&caller),
        |_| {
            let collected =
                github::collect_with(&selector(), &mut fixture(), Duration::from_secs(2));
            std::fs::write(
                root.path().join("tracked.txt"),
                "changed during collection\n",
            )
            .unwrap();
            collected
        },
    )
    .unwrap();
    assert!(
        plan.gaps
            .iter()
            .any(|g| g == "consumer_bindings_changed_during_collection")
    );
    assert_eq!(
        plan.admission,
        cockpit_protocol::ci_admission::GithubPlanAdmission::NotAttempted
    );
    assert_eq!(plan.verification_commands_executed, 0);
    assert_eq!(plan.writes_performed, 0);
    assert!(
        !root
            .path()
            .join(format!(".ai/evidence/{SERVICE_WI}.verification.json"))
            .exists()
    );
}

// Proposed diagnostic regression cases. No network is used by these cases.
fn diagnostic_tags(failure: &Failure) -> Vec<String> {
    match failure {
        Failure::Transport(tags) => tags.clone(),
        other => panic!("expected transport failure, got {other:?}"),
    }
}

fn has_tag(tags: &[String], expected: &str) -> bool {
    tags.iter().any(|tag| tag == expected)
}

#[test]
fn ci_admission_transport_phase_requires_typed_ureq_evidence_not_elapsed_time() {
    for (error, source, phase) in [
        (
            ureq::Error::HostNotFound,
            "source_ureq_host_not_found",
            "phase_resolve",
        ),
        (
            ureq::Error::Tls("DO_NOT_ECHO_TLS_DETAIL"),
            "source_ureq_tls",
            "phase_tls",
        ),
        (
            ureq::Error::ConnectionFailed,
            "source_ureq_connection_failed",
            "phase_unknown",
        ),
        (
            ureq::Error::TlsRequired,
            "source_ureq_tls_required",
            "phase_unknown",
        ),
    ] {
        let failure = github::request_failure(&error);
        let tags = diagnostic_tags(&failure);
        assert!(has_tag(&tags, "provider_diagnostic_stage_request_call"));
        assert!(has_tag(&tags, &format!("provider_diagnostic_{source}")));
        assert!(has_tag(&tags, &format!("provider_diagnostic_{phase}")));
        assert!(!tags.iter().any(|tag| tag.contains("DO_NOT_ECHO")
            || tag.contains("phase_tcp")
            || tag.contains("handshake")));
    }
}

#[test]
fn ci_admission_timeout_scope_preserves_global_unknown_and_connect_setup_ambiguity() {
    for (timeout, scope, phase) in [
        (ureq::Timeout::Global, "global", "unknown"),
        (ureq::Timeout::PerCall, "per_call", "unknown"),
        (ureq::Timeout::Connect, "connect", "connect_setup"),
        (ureq::Timeout::Resolve, "resolve", "resolve"),
        (
            ureq::Timeout::RecvResponse,
            "receive_headers",
            "receive_headers",
        ),
        (ureq::Timeout::RecvBody, "receive_body", "receive_body"),
        (ureq::Timeout::PerRead, "per_read", "transport_read"),
    ] {
        let tags = diagnostic_tags(&github::request_failure(&ureq::Error::Timeout(timeout)));
        assert!(has_tag(&tags, "provider_diagnostic_kind_timeout"));
        assert!(has_tag(
            &tags,
            &format!("provider_diagnostic_timeout_scope_{scope}")
        ));
        assert!(has_tag(
            &tags,
            &format!("provider_diagnostic_phase_{phase}")
        ));
        assert!(
            !tags
                .iter()
                .any(|tag| tag.contains("phase_tcp") || tag.contains("handshake"))
        );
    }
}

#[test]
fn ci_admission_io_kind_is_preserved_without_inventing_a_network_phase() {
    for (kind, want) in [
        (std::io::ErrorKind::TimedOut, "timed_out"),
        (std::io::ErrorKind::ConnectionRefused, "connection_refused"),
        (std::io::ErrorKind::ConnectionReset, "connection_reset"),
        (std::io::ErrorKind::UnexpectedEof, "unexpected_eof"),
        (std::io::ErrorKind::Other, "other"),
    ] {
        let error = ureq::Error::Io(std::io::Error::new(
            kind,
            "DNS TLS timeout HTTP 403 https://secret.invalid/?token=SECRET",
        ));
        let tags = diagnostic_tags(&github::request_failure(&error));
        assert!(has_tag(&tags, "provider_diagnostic_source_ureq_io"));
        assert!(has_tag(
            &tags,
            &format!("provider_diagnostic_io_kind_{want}")
        ));
        assert!(has_tag(&tags, "provider_diagnostic_phase_unknown"));
        assert!(!tags.iter().any(|tag| tag.contains("SECRET")
            || tag.contains("secret.invalid")
            || tag.contains("phase_resolve")
            || tag.contains("phase_tls")
            || tag.contains("http_status_403")));
    }
}

#[test]
fn ci_admission_raw_os_error_is_numeric_evidence_not_a_phase_guess() {
    let tags = diagnostic_tags(&github::body_failure(&std::io::Error::from_raw_os_error(
        12345,
    )));
    assert!(has_tag(&tags, "provider_diagnostic_io_os_error_12345"));
    assert!(has_tag(&tags, "provider_diagnostic_phase_unknown"));
}

#[test]
fn ci_admission_body_read_unwraps_only_real_typed_ureq_io_payload() {
    let error = ureq::Error::Timeout(ureq::Timeout::RecvBody).into_io();
    let tags = diagnostic_tags(&github::body_failure(&error));
    for expected in [
        "provider_diagnostic_stage_response_body_read",
        "provider_diagnostic_http_status_200_observed",
        "provider_diagnostic_source_std_io",
        "provider_diagnostic_source_std_io_wrapped_ureq",
        "provider_diagnostic_source_ureq_timeout",
        "provider_diagnostic_phase_receive_body",
        "provider_diagnostic_kind_timeout",
    ] {
        assert!(has_tag(&tags, expected), "missing {expected}: {tags:?}");
    }
}

#[derive(Debug)]
struct DoNotFormat;
impl std::fmt::Display for DoNotFormat {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("raw diagnostic formatter must not be called")
    }
}
impl std::error::Error for DoNotFormat {}

#[test]
fn ci_admission_unknown_errors_are_redacted_without_display_or_debug_fallback() {
    let error = ureq::Error::Other(Box::new(DoNotFormat));
    let tags = diagnostic_tags(&github::request_failure(&error));
    assert!(has_tag(&tags, "provider_diagnostic_source_ureq_other"));
    assert!(has_tag(&tags, "provider_diagnostic_kind_unknown"));
    assert!(has_tag(&tags, "provider_diagnostic_phase_unknown"));
    let text = serde_json::to_string(&tags).unwrap();
    assert!(!text.contains("DoNotFormat"));
    let error = std::io::Error::other(
        "Authorization: Bearer SECRET\nCookie: secret\nhttps://secret.invalid/path?token=SECRET\nBODY",
    );
    let text = serde_json::to_string(&diagnostic_tags(&github::body_failure(&error))).unwrap();
    for secret in [
        "Authorization",
        "Bearer",
        "SECRET",
        "Cookie",
        "secret.invalid",
        "token=",
        "BODY",
    ] {
        assert!(!text.contains(secret), "leaked {secret}");
    }
}

#[test]
fn ci_admission_wrapped_error_inspection_is_bounded_and_stays_unknown_at_limit() {
    let mut error = ureq::Error::HostNotFound;
    for _ in 0..16 {
        error = ureq::Error::Io(std::io::Error::other(error));
    }
    let tags = diagnostic_tags(&github::request_failure(&error));
    assert!(has_tag(&tags, "provider_diagnostic_source_wrapper_limit"));
    assert!(has_tag(&tags, "provider_diagnostic_kind_unknown"));
    assert!(has_tag(&tags, "provider_diagnostic_phase_unknown"));
    assert!(!has_tag(&tags, "provider_diagnostic_phase_resolve"));
    assert!(tags.len() <= 24);
}

#[test]
fn ci_admission_deadline_priority_keeps_prior_typed_diagnostic_separately() {
    let (result, tags) = github::collection_result_with_diagnostics::<()>(
        Err(github::request_failure(&ureq::Error::HostNotFound)),
        Instant::now(),
    );
    assert_eq!(result, Err(Failure::Deadline));
    assert!(has_tag(
        &tags,
        "provider_diagnostic_source_ureq_host_not_found"
    ));
    assert!(has_tag(&tags, "provider_diagnostic_phase_resolve"));
    let (result, tags) = github::collection_result_with_diagnostics::<()>(
        Err(Failure::HttpStatus(403)),
        Instant::now(),
    );
    assert_eq!(result, Err(Failure::Deadline));
    assert!(has_tag(
        &tags,
        "provider_diagnostic_http_status_403_observed"
    ));
}

struct FailureReader {
    failure: Option<Failure>,
    calls: usize,
}
impl MetadataReader for FailureReader {
    fn read(&mut self, _: &str, _: bool, _: Instant) -> Result<Reply, Failure> {
        self.calls += 1;
        Err(self.failure.take().expect("collector must not retry"))
    }
}

#[test]
fn ci_admission_transport_diagnostics_keep_stable_failure_and_no_complete_observation() {
    let mut reader = FailureReader {
        failure: Some(github::body_failure(&std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "SECRET",
        ))),
        calls: 0,
    };
    let collected = github::collect_with(&selector(), &mut reader, Duration::from_secs(60));
    assert_eq!(reader.calls, 1);
    assert_eq!(
        collected.collection.failure.as_deref(),
        Some("provider_transport_unavailable")
    );
    assert!(!collected.collection.complete);
    assert_eq!(collected.collection.assurance, "not_observed");
    assert!(collected.collection.requests.is_empty());
    assert!(collected.metadata.is_none());
    assert!(!collected.rejected);
    assert!(has_tag(
        &collected.diagnostic_gaps,
        "provider_diagnostic_http_status_200_observed"
    ));
}

#[test]
fn ci_admission_diagnostic_tags_use_existing_plan_gaps_without_acceptance_or_writes() {
    let (root, runtime, caller) = service_fixture();
    let plan = super::plan_with_collection(
        root.path(),
        SERVICE_WI,
        &selector(),
        &runtime,
        Some(&caller),
        |_| {
            let mut reader = FailureReader {
                failure: Some(github::request_failure(&ureq::Error::HostNotFound)),
                calls: 0,
            };
            github::collect_with(&selector(), &mut reader, Duration::from_secs(60))
        },
    )
    .unwrap();
    assert_eq!(
        plan.state,
        cockpit_protocol::ci_admission::GithubPlanState::ProviderUnavailable
    );
    assert_eq!(
        plan.collection.failure.as_deref(),
        Some("provider_transport_unavailable")
    );
    assert!(!plan.collection.complete);
    assert!(plan.collection.requests.is_empty());
    assert!(plan.metadata.is_none());
    assert!(has_tag(&plan.gaps, "provider_transport_unavailable"));
    assert!(has_tag(
        &plan.gaps,
        "provider_diagnostic_source_ureq_host_not_found"
    ));
    for gap in [
        "admission_not_implemented",
        "producer_runtime_unverified",
        "native_receipt_not_inspected",
    ] {
        assert!(has_tag(&plan.gaps, gap));
    }
    assert_eq!(
        plan.admission,
        cockpit_protocol::ci_admission::GithubPlanAdmission::NotAttempted
    );
    assert_eq!(plan.verification_commands_executed, 0);
    assert_eq!(plan.writes_performed, 0);
    assert!(
        !root
            .path()
            .join(format!(".ai/evidence/{SERVICE_WI}.verification.json"))
            .exists()
    );
    let value = serde_json::to_value(&plan).unwrap();
    assert_eq!(value["schemaVersion"], 1);
    assert!(value.get("diagnostics").is_none());
    assert!(value["collection"].get("diagnostics").is_none());
    assert!(!value.to_string().contains("SECRET"));
}

#[test]
fn ci_admission_collector_deadline_wins_after_reader_returns_typed_transport_error() {
    struct OverdueTransportReader {
        calls: usize,
        returned_after_deadline: bool,
    }
    impl MetadataReader for OverdueTransportReader {
        fn read(&mut self, _: &str, _: bool, deadline: Instant) -> Result<Reply, Failure> {
            self.calls += 1;
            // Condition-driven: return the typed error only after this actual
            // collection deadline, not after an unrelated guessed sleep.
            while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
                std::thread::sleep(remaining);
            }
            self.returned_after_deadline = true;
            Err(github::request_failure(&ureq::Error::HostNotFound))
        }
    }
    let mut reader = OverdueTransportReader {
        calls: 0,
        returned_after_deadline: false,
    };
    let collected = github::collect_with(&selector(), &mut reader, Duration::from_secs(1));
    assert_eq!(reader.calls, 1);
    assert!(reader.returned_after_deadline);
    assert_eq!(
        collected.collection.failure.as_deref(),
        Some("provider_collection_deadline")
    );
    assert!(!collected.collection.complete);
    assert_eq!(collected.collection.assurance, "not_observed");
    assert!(collected.collection.requests.is_empty());
    assert!(collected.metadata.is_none());
    assert!(!collected.rejected);
    for expected in [
        "provider_diagnostic_stage_request_call",
        "provider_diagnostic_source_ureq_host_not_found",
        "provider_diagnostic_kind_host_not_found",
        "provider_diagnostic_phase_resolve",
    ] {
        assert!(
            has_tag(&collected.diagnostic_gaps, expected),
            "missing {expected}: {:?}",
            collected.diagnostic_gaps
        );
    }
}
