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

// Batch2A: offline byte/schema fixtures only; no fake provider capability,
// native action admission, current-WI grant or filesystem extraction.
use super::archive::{self, GithubArchiveError as ArchiveError};
use cockpit_core::Digest as ArchiveDigest;
use cockpit_protocol::ci_admission::{
    UntrustedGithubArchiveManifest, UntrustedGithubNativeEnvelope,
};
use cockpit_verification::VerificationReceipt as ArchiveNativeReceipt;

fn archive2a_native() -> Vec<u8> {
    // Exact captured native schema2 sample. The parser treats every identity
    // and success claim as UNTRUSTED; this is not a fresh test execution.
    r###"{"captureMode":"full_capture","contractDigest":"sha256:40e7ab0b9c454664c3a004ccb362c969eb5897beabfb25b6c8db78bd1c60b6ea","createdAt":"2026-10-31T05:42:18Z","evidenceSchemaVersion":2,"passed":true,"protocolVersion":1,"receipt":{"costObservation":{"advisoryOnly":true,"confidence":"complete","elapsedMs":44080,"executionElapsedMs":43583,"maxConcurrentProcesses":1,"nodesExecuted":1,"nodesPlanned":1,"nodesReused":0,"passed":true,"planningElapsedMs":0,"processesSpawned":1,"protectedNodesExecuted":0,"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","schemaVersion":1,"unknowns":[]},"elapsedMs":44080,"executionElapsedMs":43583,"executionRecords":[{"commandDigest":"sha256:dd3968def0dd92b8c1cabb1d5198ce956a5824b3ee296eb33f492523bd40965f","deadlineMs":1793426194838,"elapsedMs":43578,"exitCode":0,"nodeId":"project-command-0","passed":true,"spawned":true,"stderr":"7461736b2d617574686f72697a656420576f726b204974656d20766572696669636174696f6e20726571756972657320616e206578706c69636974202d2d7461736b2d63616c6c65722066696c650a63616e6469646174652052756e74696d6520707265666c69676874206661696c65642077697468206578697420310a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a6465636c6172656420766572696669636174696f6e2074696d656f7574206d757374206265206120706f7369746976652066696e69746520696e74656765720a63616e6469646174652052756e74696d65207265706f72747320667265736820766572696669636174696f6e2c206275742069747320666f726d616c2072656365697074206973206e6f74207265757361626c6520286578697374696e675f66726573685f726563656970745f6964656e746974795f6d69736d61746368290a776f726b737061636520726563656970742073796e632072656a65637465643a20666f726d616c20766572696669636174696f6e2072656365697074207265706f7369746f7279206964656e7469747920646966666572732066726f6d2052756e74696d65207374617475730a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a576f726b204974656d20494420697320756e7361666520666f72207468652063616e6f6e6963616c2065766964656e636520706174680a776f726b737061636520726563656970742073796e632072656a65637465643a2063616e6f6e6963616c20776f726b7370616365207265636569707420746172676574206973206e6f74206120726567756c6172206e6f6e2d73796d6c696e6b2066696c650a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a63616e6469646174652052756e74696d6520646f6573206e6f74206163636570742074686520666f726d616c207265636569707420617320667265736820666f72207468652063757272656e7420436f6e747261637420616e6420736f7572636520736e617073686f740a63616e6469646174652052756e74696d652061646d697473206e6569746865722072756e5f707265666c69676874206e6f722072756e5f766572696669636174696f6e0a63616e6469646174652052756e74696d65206e6f206c6f6e6765722061646d6974732072756e5f766572696669636174696f6e2061667465722073746174757320726566726573680a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652072657475726e656420616e20696e76616c696420576f726b204974656d2076616c69646174696f6e207265706f72740a63616e6469646174652052756e74696d652061646d697473206e6569746865722072756e5f707265666c69676874206e6f722072756e5f766572696669636174696f6e0a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a20457870656374696e672070726f7065727479206e616d6520656e636c6f73656420696e20646f75626c652071756f7465733a206c696e65203120636f6c756d6e20322028636861722031290a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a206475706c6963617465207374646f75742f65766964656e6365204a534f4e206669656c640a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a2052756e74696d65207374646f757420646f6573206e6f742070726f6a65637420746f2074686520666f726d616c20726563656970740a686f7374656420657865637574696f6e206f75747075742062696e64696e672072656a65637465643a2052756e74696d65207374646f757420646f6573206e6f742070726f6a65637420746f2074686520666f726d616c20726563656970740a","stderrTruncated":false,"stdout":"776f726b7370616365207061636b61676520636f766572616765207061737365643a2031207061636b616765730a74696d656f75742072656772657373696f6e207061737365643a20626f756e6465643d312c2064656661756c743d312c20756e72656c617465643d312c20696e76616c69643d352c2052756e74696d652d7265667573616c3d310a70726f64756365722f636f6e73756d657220436172676f20636f6e746578742070726f6f66207061737365643a2031322063617365730a686f737465642052756e74696d6520766572696669636174696f6e206f726368657374726174696f6e2072656772657373696f6e207061737365640a","stdoutTruncated":false,"timedOut":false,"timeoutSeconds":900}],"filesHashed":5,"filesRead":5,"gitCalls":6,"maxConcurrentProcesses":1,"nodesExecuted":1,"nodesPlanned":1,"nodesReused":0,"passed":true,"planReceipt":{"affectedPaths":[],"assurance":"repository_verified","baseRevision":"efc9f309f1500e433240a76fe5dc76db64abb53b","dependencyConfidence":"unknown","escalations":[],"executedNodes":["project-command-0"],"executionElapsedMs":43583,"finalTier":"T0","initialTier":"T0","planningElapsedMs":0,"policyRefs":["wi-1114-bounded-release-verification-v1"],"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","repositorySnapshotDigest":"sha256:3ff877b2050551986531641a9573e9ecf332ea3da524b710f6e9a3ee38cb1150","requiredAssurance":"self_declared","requiredTier":"T0","reusedNodes":[],"savedExecutions":0,"schemaVersion":1,"selectionReasons":["effective:ai-cockpit-project-v1:wi-1114-bounded-release-verification-v1: Enforce the approved maximum of 900 seconds per actual verification command with single worker, finite supervised cleanup and at least 1 GiB free disk. Stage applicability does not authorize publication or production use; exact version, tag and assets require separate future owner approval. Keep all platform, source, evidence and approval requirements; the whole multi-command run has a separately declared finite aggregate budget."],"sharedNodes":[],"stage":"task","timeoutSeconds":900,"workItemId":"WI-1114-release-runtime-convergence"},"planningElapsedMs":0,"processSpawnFailures":0,"processesSpawned":1,"protectedNodesExecuted":0,"protectedNodesSkipped":0,"receiptCandidates":[],"repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","rerunStale":0,"rerunUnknown":0,"results":[{"action":"execute","nodeId":"project-command-0","outputDigest":"sha256:3ed74a2373b13cd963aa5dc6b029c636f686b556ba635ac4b49d6f7f3d48634e","outputTruncated":false,"passed":true,"protected":false,"reason":"reuse_not_configured","receiptId":null,"reused":false,"satisfiedBy":"execution","state":"not_applicable","timedOut":false}],"runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","timeoutSeconds":900,"workItemId":"WI-1114-release-runtime-convergence"},"receiptDigest":"sha256:c82b48c8f1c733a39619b96d2ab77af6379cb329a03c1ba070546742cb3a6ff2","repositoryId":"sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b","repositorySnapshotDigest":"sha256:3ff877b2050551986531641a9573e9ecf332ea3da524b710f6e9a3ee38cb1150","runtimeDigest":"sha256:6f7897c9b46ec160a1fb193f6767ad3f745c86aea522dc16e81a8ab60003d838","runtimeVersion":"1.0.1","workItemId":"WI-1114-release-runtime-convergence"}"###.as_bytes().to_vec()
}

fn archive2a_crc(raw: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in raw {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320_u32 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn archive2a_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn archive2a_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

// Small classic, single-disk, stored ZIP writer for self-owned byte fixtures.
// No product ZIP crate dependency is needed to compile the conservative RED.
fn archive2a_zip(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, raw) in entries {
        let offset = out.len() as u32;
        let crc = archive2a_crc(raw);
        archive2a_u32(&mut out, 0x04034b50);
        for value in [20, 0, 0, 0, 0] {
            archive2a_u16(&mut out, value);
        }
        for value in [crc, raw.len() as u32, raw.len() as u32] {
            archive2a_u32(&mut out, value);
        }
        archive2a_u16(&mut out, name.len() as u16);
        archive2a_u16(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(raw);
        archive2a_u32(&mut directory, 0x02014b50);
        for value in [0x0314, 20, 0, 0, 0, 0] {
            archive2a_u16(&mut directory, value);
        }
        for value in [crc, raw.len() as u32, raw.len() as u32] {
            archive2a_u32(&mut directory, value);
        }
        for value in [name.len() as u16, 0, 0, 0, 0] {
            archive2a_u16(&mut directory, value);
        }
        archive2a_u32(&mut directory, 0o100600 << 16);
        archive2a_u32(&mut directory, offset);
        directory.extend_from_slice(name.as_bytes());
    }
    let directory_start = out.len() as u32;
    out.extend_from_slice(&directory);
    archive2a_u32(&mut out, 0x06054b50);
    for value in [0, 0, entries.len() as u16, entries.len() as u16] {
        archive2a_u16(&mut out, value);
    }
    archive2a_u32(&mut out, directory.len() as u32);
    archive2a_u32(&mut out, directory_start);
    archive2a_u16(&mut out, 0);
    out
}

fn archive2a_bundle(native: Vec<u8>) -> Vec<u8> {
    let manifest = json!({"schemaVersion":1,"entries":[{
        "path":"receipts/verification.json","role":"native_verification_evidence",
        "bytes":native.len(),"digest":ArchiveDigest::sha256_bytes(&native)
    }]});
    archive2a_zip(&[
        ("manifest.json", serde_json::to_vec(&manifest).unwrap()),
        ("receipts/verification.json", native),
    ])
}

fn archive2a_reject(raw: &[u8], expected: ArchiveError) {
    let actual = archive::inspect_untrusted_github_archive(raw, &ArchiveDigest::sha256_bytes(raw));
    assert_eq!(actual.unwrap_err(), expected);
}

fn archive2a_central(raw: &[u8]) -> usize {
    let eocd = raw.len() - 22;
    u32::from_le_bytes(raw[eocd + 16..eocd + 20].try_into().unwrap()) as usize
}

// Self-owned synthetic archive bytes only. These helpers never execute or
// authenticate the declarations they encode.
fn archive2a_deflate(raw: &[u8]) -> Vec<u8> {
    use std::io::Write as _;
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(raw).unwrap();
    encoder.finish().unwrap()
}

fn archive2a_zip_encoded(
    entries: &[(&str, Vec<u8>)],
    deflate: bool,
    descriptor: Option<bool>,
) -> Vec<u8> {
    let mut out = Vec::new();
    let mut directory = Vec::new();
    for (name, raw) in entries {
        let offset = out.len() as u32;
        let crc = archive2a_crc(raw);
        let data = if deflate {
            archive2a_deflate(raw)
        } else {
            raw.clone()
        };
        let flags = if descriptor.is_some() { 8 } else { 0 };
        let method = if deflate { 8 } else { 0 };
        archive2a_u32(&mut out, 0x04034b50);
        for v in [20, flags, method, 0, 0] {
            archive2a_u16(&mut out, v);
        }
        for v in if descriptor.is_some() {
            [0, 0, 0]
        } else {
            [crc, data.len() as u32, raw.len() as u32]
        } {
            archive2a_u32(&mut out, v);
        }
        archive2a_u16(&mut out, name.len() as u16);
        archive2a_u16(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&data);
        if let Some(signed) = descriptor {
            if signed {
                archive2a_u32(&mut out, 0x08074b50);
            }
            for v in [crc, data.len() as u32, raw.len() as u32] {
                archive2a_u32(&mut out, v);
            }
        }
        archive2a_u32(&mut directory, 0x02014b50);
        for v in [0x0314, 20, flags, method, 0, 0] {
            archive2a_u16(&mut directory, v);
        }
        for v in [crc, data.len() as u32, raw.len() as u32] {
            archive2a_u32(&mut directory, v);
        }
        for v in [name.len() as u16, 0, 0, 0, 0] {
            archive2a_u16(&mut directory, v);
        }
        archive2a_u32(&mut directory, 0o100600 << 16);
        archive2a_u32(&mut directory, offset);
        directory.extend_from_slice(name.as_bytes());
    }
    let start = out.len() as u32;
    out.extend_from_slice(&directory);
    archive2a_u32(&mut out, 0x06054b50);
    for v in [0, 0, entries.len() as u16, entries.len() as u16] {
        archive2a_u16(&mut out, v);
    }
    archive2a_u32(&mut out, directory.len() as u32);
    archive2a_u32(&mut out, start);
    archive2a_u16(&mut out, 0);
    out
}

fn archive2a_inventory(natives: &[(&str, Vec<u8>)]) -> Vec<(String, Vec<u8>)> {
    let entries = natives
        .iter()
        .map(|(path, bytes)| {
            json!({
                "path":path,"role":"native_verification_evidence","bytes":bytes.len(),
                "digest":ArchiveDigest::sha256_bytes(bytes)
            })
        })
        .collect::<Vec<_>>();
    let mut values = vec![(
        "manifest.json".to_owned(),
        serde_json::to_vec(&json!({"schemaVersion":1,"entries":entries})).unwrap(),
    )];
    values.extend(
        natives
            .iter()
            .map(|(path, raw)| ((*path).to_owned(), raw.clone())),
    );
    values
}

fn archive2a_bundle_encoded(
    natives: &[(&str, Vec<u8>)],
    deflate: bool,
    descriptor: Option<bool>,
) -> Vec<u8> {
    let entries = archive2a_inventory(natives);
    let borrowed = entries
        .iter()
        .map(|(path, bytes)| (path.as_str(), bytes.clone()))
        .collect::<Vec<_>>();
    archive2a_zip_encoded(&borrowed, deflate, descriptor)
}

// Insert bytes in the first local member; relocate central directory and later
// local-header offsets without changing any declared payload sizes.
fn archive2a_insert_first(raw: &[u8], at: usize, bytes: &[u8]) -> Vec<u8> {
    let old_central = archive2a_central(raw);
    let mut changed = raw.to_vec();
    changed.splice(at..at, bytes.iter().copied());
    let central = old_central + bytes.len();
    let eocd = changed.len() - 22;
    changed[eocd + 16..eocd + 20].copy_from_slice(&(central as u32).to_le_bytes());
    let count = u16::from_le_bytes(changed[eocd + 10..eocd + 12].try_into().unwrap());
    let mut entry = central;
    for _ in 0..count {
        let offset = u32::from_le_bytes(changed[entry + 42..entry + 46].try_into().unwrap());
        if offset as usize >= at {
            changed[entry + 42..entry + 46]
                .copy_from_slice(&(offset + bytes.len() as u32).to_le_bytes());
        }
        let name = u16::from_le_bytes(changed[entry + 28..entry + 30].try_into().unwrap()) as usize;
        let extra =
            u16::from_le_bytes(changed[entry + 30..entry + 32].try_into().unwrap()) as usize;
        let comment =
            u16::from_le_bytes(changed[entry + 32..entry + 34].try_into().unwrap()) as usize;
        entry += 46 + name + extra + comment;
    }
    changed
}

fn archive2a_claims_native(count: usize, prefix: &str) -> Vec<u8> {
    // Synthetic internally consistent declarations for cardinality boundaries,
    // not new physical execution facts or authenticated producer evidence.
    let mut envelope: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    let mut result = envelope["receipt"]["results"][0].clone();
    let mut record = envelope["receipt"]["executionRecords"][0].clone();
    record["stdout"] = json!("");
    record["stderr"] = json!("");
    record["elapsedMs"] = json!(0);
    result["outputDigest"] = json!(ArchiveDigest::sha256_bytes(br#"{"success":true,"exitCode":0,"terminationSignal":null,"stdout":[],"stderr":[],"stdoutTruncated":false,"stderrTruncated":false,"timedOut":false}"#));
    let mut results = Vec::new();
    let mut records = Vec::new();
    let mut nodes = Vec::new();
    for index in 0..count {
        let node = format!("{prefix}-{index}");
        let mut next_result = result.clone();
        next_result["nodeId"] = json!(node);
        let mut next_record = record.clone();
        next_record["nodeId"] = json!(node);
        results.push(next_result);
        records.push(next_record);
        nodes.push(node);
    }
    envelope["receipt"]["results"] = json!(results);
    envelope["receipt"]["executionRecords"] = json!(records);
    for name in ["nodesPlanned", "nodesExecuted", "processesSpawned"] {
        envelope["receipt"][name] = json!(count);
    }
    envelope["receipt"]["planReceipt"]["executedNodes"] = json!(nodes);
    envelope["receipt"]
        .as_object_mut()
        .unwrap()
        .remove("costObservation");
    envelope["receiptDigest"] = json!(cockpit_protocol::digest_json(&envelope["receipt"]).unwrap());
    serde_json::to_vec(&envelope).unwrap()
}

#[test]
fn ci_admission_archive_raw_limit_boundaries_need_no_large_allocation() {
    assert!(archive::check_raw_archive_size(64 * 1024 * 1024).is_ok());
    assert_eq!(
        archive::check_raw_archive_size(64 * 1024 * 1024 + 1),
        Err(ArchiveError::RawSize)
    );

    let one = 64_u64 * 1024 * 1024;
    let compressed = one.div_ceil(100);
    assert!(archive::check_declared_budgets(&[(compressed, one), (compressed, one)]).is_ok());
    assert_eq!(
        archive::check_declared_budgets(&[(compressed, one), (compressed, one), (1, 1)]),
        Err(ArchiveError::ExpandedSize)
    );
    assert!(archive::check_declared_budgets(&[(1, 100), (0, 0)]).is_ok());
    assert_eq!(
        archive::check_declared_budgets(&[(1, 101)]),
        Err(ArchiveError::ExpansionRatio)
    );
    assert_eq!(
        archive::check_declared_budgets(&[(0, 1)]),
        Err(ArchiveError::ExpansionRatio)
    );
    assert_eq!(
        archive::check_declared_budgets(&[(u64::MAX, 0), (1, 0)]),
        Err(ArchiveError::ExpansionRatio)
    );
}

#[test]
fn ci_admission_archive_wrong_digest_never_enters_decoder() {
    let entered = std::cell::Cell::new(false);
    let actual = archive::inspect_with_decoder(
        b"not a ZIP",
        &ArchiveDigest::sha256_bytes(b"different"),
        |_, _| {
            entered.set(true);
            Err(ArchiveError::NotImplemented)
        },
    );
    assert_eq!(actual.unwrap_err(), ArchiveError::ArchiveDigest);
    assert!(!entered.get());
}

#[test]
fn ci_admission_archive_manifest_dto_is_strict_and_cannot_claim_authentication() {
    let valid = json!({"schemaVersion":1,"entries":[]});
    assert!(serde_json::from_value::<UntrustedGithubArchiveManifest>(valid.clone()).is_ok());
    for key in ["trusted", "providerAuthenticated", "command", "endpoint"] {
        let mut changed = valid.clone();
        changed[key] = true.into();
        assert!(serde_json::from_value::<UntrustedGithubArchiveManifest>(changed).is_err());
    }
    let duplicate = br#"{"schemaVersion":1,"schemaVersion":1,"entries":[]}"#;
    assert!(serde_json::from_slice::<UntrustedGithubArchiveManifest>(duplicate).is_err());
}

#[test]
fn ci_admission_archive_native_dto_preserves_current_wire_without_authentication() {
    let native = archive2a_native();
    let parsed: UntrustedGithubNativeEnvelope<ArchiveNativeReceipt> =
        serde_json::from_slice(&native).unwrap();
    assert_eq!(parsed.evidence_schema_version, 2);
    assert!(parsed.receipt.passed);
    let mut changed: Value = serde_json::from_slice(&native).unwrap();
    changed["providerAuthenticated"] = true.into();
    assert!(
        serde_json::from_value::<UntrustedGithubNativeEnvelope<ArchiveNativeReceipt>>(changed)
            .is_err()
    );
}

#[test]
fn ci_admission_archive_valid_stored_native_is_only_untrusted_structure() {
    let raw = archive2a_bundle(archive2a_native());
    let observed =
        archive::inspect_untrusted_github_archive(&raw, &ArchiveDigest::sha256_bytes(&raw))
            .unwrap();
    assert_eq!(observed.receipts.len(), 1);
    assert_eq!(observed.members.len(), 2);
    for gap in [
        "provider_origin_unverified",
        "workflow_execution_closure_unverified",
        "producer_runtime_unverified",
        "artifact_job_attempt_binding_unavailable",
        "current_consumer_command_binding_unavailable",
        "native_coverage_manifest_missing",
    ] {
        assert!(
            observed.gaps.iter().any(|value| value == gap),
            "missing gap: {gap}"
        );
    }
    assert_eq!(
        observed
            .members
            .iter()
            .find(|member| member.path == "receipts/verification.json")
            .unwrap()
            .raw,
        archive2a_native()
    );

    for signed in [false, true] {
        let streamed = archive2a_bundle_encoded(
            &[("receipts/verification.json", archive2a_native())],
            false,
            Some(signed),
        );
        let parsed = archive::inspect_untrusted_github_archive(
            &streamed,
            &ArchiveDigest::sha256_bytes(&streamed),
        )
        .unwrap();
        assert_eq!(parsed.receipts.len(), 1);
        assert!(
            parsed
                .gaps
                .iter()
                .any(|gap| gap == "producer_runtime_unverified")
        );
        let central = archive2a_central(&streamed);
        let compressed =
            u32::from_le_bytes(streamed[central + 20..central + 24].try_into().unwrap()) as usize;
        let data = 30 + "manifest.json".len();
        let descriptor_crc = data + compressed + if signed { 4 } else { 0 };
        let mut bad = streamed;
        bad[descriptor_crc] ^= 1;
        archive2a_reject(&bad, ArchiveError::ZipStructure);
    }
}

#[test]
fn ci_admission_archive_valid_deflate_has_the_same_untrusted_semantics() {
    let raw = hex::decode("504b0304140000000800000021002f1252219d000000cb0000000d0000006d616e69666573742e6a736f6e4dcc410e82301484e1bbbc35515afa4ae5206e8c21a51da44681b40d8921dcddba733bf9fed929b9096f7b454c6199a9131561ce31205177db69b579a28e221cc29ad379430c637036177c7aa652541497178a99cbb8a1ff173db6e0313b14357cf2efd328cd15f9f040ca254a9395ac3ba58db88cc2b4239482b18d1d84646506a14766162c94765633d77a80af599a4642b7be6d517b739174dc8f2f504b0304140000000800000021000577229929090000112100001a00000072656365697074732f766572696669636174696f6e2e6a736f6eed59596fdcc811fe2bc43c8f0cb2bbd924b54fbb8a9318c1c686e4244002c3e8a35a62c421273c246b0dfff754159ba3198d6ccbc17a0363fd606bd847d55777b1f87ee5cc769c7af8b9f3b03a5d85a969dec6a5d57ae5ba76ec8d1bff505fc230e2fe706544ae4f550a85b1a9ad9cca95d6ca4993a6ca392bb57095aec0e6655558303658915bed4a6f8bd2facce9d46a3044ba073382ff91a88a54e8932c3d91d9eb343f55e2342bff8947e0a6f6d03ab87057b0317f877ea8bb76752ad6abad1906f0abd3b19f009ffa6eec5cd7ec4e64eb550f0eea2d127f8f420ce34b3b407f6346de7ebf32fea61ebafeee65dbdc2d5450d630f34344aedb6c1b184907d0982d32fb79589d2a959629aebc033711a9e77b5b322fe57ab531efcebad64d7d0fedf8aaef1c20d08111b5a8e1e1395f25e8cbcaabc6b4edfec2394c2c5b7a24269dacdbcb3dae290b3f33b9d89adb851069041cf2f9eb21d39414b3ed867a44e15ff87b8302a4c2a0058d50c297324d4bada4b05e7919ca4ca7224f7359644257652e0a91019412b2ac00997b2384453df5533bd61b78e82a3a14e80aaeb24a83434a260b36ab64d0852e0c522f54ee4a6dc0e442783c0065664a63759aa6129194f79477e65d65cfd267196e0c878e81824fed75dbddb6a89a7fbdf9f085b6db6d9d83eb7a4f34c879361bd3fa8752792f2b5d7a08a9f795b0a5cb9cb136f37956950eaa5c9bbc14ca4a005169b0520655895c48d4685ae93c20780fc637750b84202b2aa984ce2a85121fc29679412befeaf18c43349ddd848d8766fe379af924823c49570f7d66589c223e8e1efa1e6f164a6785d45678fc9be353897642eb1646e75a89342ff8d98a5455b89b6b2fd242eb9cce688dff24de53f83768c01d813b4807efd32f29529dd13afe2e8b543b3e8f7430cebdf07bbc910aee3aa28ba7892efe4e0daf8326fa8af9e48848207d609e9e9e0be68a77887aa14ba28e4fd94c8364280ae2a9cb19c7cc5f66489de491782a630aea9392457ea88d9c3978fc2bf99720143a2324b8cbf2e13fcdab440778859e2ac69d234ae4f29dffef9eff53bc9be288e88a42713ce939b2c89b3f8ad611aa88b35aee711c7b8ab3992adea0384de3293a037846459ec42343d91ca110658c1cc93c401779b8479287437ab857b16e590ee45fe18aa7dbc41fd7087d959a22e616a44a79422ef2ee23c33de227e30e6525c93a54d21c49f5c9bc744075d1ea6c2dd26ec5f639408d2b8aa931dd9df603e5c0233b49ce668af4f634cbe24e883932ea9d7520091963a45c9c7fd672d943af98cfefdd9705a117b36498c725e904910ae217f9027bbf64ef16d1121257b382fe524d50a979ac16a84aedfc87e523c9f53dee7d1ccc27503cccf99ef3b15eb41eb9a6acaff257f18e47383e89aae28c40517a141b1ce774be40693973cc7be8659ed178ae43405afab23af6edf9c3ef58aa0cf7fc2eb7022365248c894f63567cac3381887b39f3919cf5441c484d505f453e5ec4eee900dbe7f8f0794dd419d55e168b56d8e9fc0bea9562fdc25c53630748b54f7397577da2af8cfb473d65ac80df317cc7f0e5187e9368a5ec116b4f7ccfc83927ddc719778ec8258f5d63157163e43e5ebfd4fcd634af2de7e864a055ee082bc6984539e72aeab8af997154cc993a9c7ce9e5f8fd2c70c44bae9273bdcae99d886b66d00e4f78ba2905f77e941bb9d2e189eaebc8aac9479637c46847e6a162c71d8e7b056572a982825869a91eabaf83eed16e6f41f6488d9badb450e12a159e56ebbe7dfcab65baf0ba9f5a6778dc144c33cc53876ea2e1c9a35dd8fccbe2ff45ec1797788b2bdc1bcad9b7492a993dbc45bdf7c337b5d8ad7115a37e3f669107b4b4e018013debcc236dc79aa18c4011e38add2aeb52e8b9ff3b3c7f90d57035a7d507daf7b1aece6f588eee22ea59e31805dc1b09f4f7b983911c8b9ca9948c7d698871ca6f9345b967317c2b7b4c4f14c71c5772d6d1a18f1d79c7c77b85c07d53c9bae4deeac13bd627f5bcf806fac0b16fd060cfbf24efd85bc09317e0ba966670559a7e78b35e85ba81e1cf66b8a2bb797c3e07333f5dd6e399691a3caebffe0cf6fc7eb46c42e031eb2b335ef1cc71bdc2e3536fe629f2fdacf5ed0df475a891d07a65cd00e77053c76126045705995621cbd3149494023586c686dcbb427bab95b136979647865b6869447d77b63fac8e434f9a550fce343ce58e6820caca73605c7a6464f8e69343d150b7a6795d030d0d5fd37cb16eebb13e5cfac860ba6b6a77770e81f9ded6275996a913db4d28813fe9a101d4c3c9ac17c7984f6e3242f3f526d43bc2172d22bdea8ebe6bc810caa2c06e0489d12857e7129b88cc54e8dc50810b681d30d29b5c285b6429fa7a6524802c9dcdd07ecce43f53dd83ff71cf0f0668c25b0fae313d7bc072665f893dbbdc622754c3606ec03f5f2c33ebf478e08da4d19c3cb436c36c76f428f2cafa064e4d8d8676d7db7a3c590c7f939d3ed116a7c9f33674bd8364bc82c46c9102224a30bceacdb449ba90606c26c31ca8c916fac4b871324db24f27898e96dcd6e35532a0973490dc76fd35f4eb24903341324c7819e301893b44d24edb846e9831215c6392257faa7f4a420f90f87ab87e965c8ce69211a187195b37f57897f80e86a4edc6c44ce355d7d7bf40b29d6cb3c0e8fa04f1fb897595a0aa7f48e01de225b0a4cc75823467b618efe390441ba17c5bd363ca4ac2441fc5120c349294b5619a67c95f00106ed324180523aa6bb34e866e42adad93e5fbd54c36de58086f30390d3fb06a6faf3a54ca666ac67a89cb041365726586c4ec0034286374a14571e6f2b2874b0267277f09e3330a1f74e5fed091485be866a319ae578f26d8f58a2cf262840d47dc3f5eccfeb1f845fcfe82d85ad4d625679d0f9ff91ec55fa3fe68ea0675f6bf7da83adcbab8aeb7dbdd272c4ec067a8a7daa3f451ceaf98375003e8730d44f6f8f8b79873e785016d377f2f326e8c697d9665f599cf356889ed749c87c017ca082ca03693ce63393606cb81b6a9a81cf700a5b679ae2dbe08e5c629ab2a8f453a48afb06d57b0237c5c708f3f9db29277fb3de7112e5e18256f31a2def297d1cb69c95dac7c12a89d9a66495cf7cd1ec6db4095eea7bb9d164823ec8723f921915c62b781d5510f40e5fefff031f1578b8ba8a187d85d29ac2a5d1932476f60b2d25965b1f512c61685c1beafa89c95a232a9749935d842e54a638f878b46872056df7c65fced6dfae5e6fb2f504b01021403140000000800000021002f1252219d000000cb0000000d00000000000000000000008081000000006d616e69666573742e6a736f6e504b01021403140000000800000021000577229929090000112100001a00000000000000000000008081c800000072656365697074732f766572696669636174696f6e2e6a736f6e504b0506000000000200020083000000290a00000000").unwrap();
    let observed =
        archive::inspect_untrusted_github_archive(&raw, &ArchiveDigest::sha256_bytes(&raw))
            .unwrap();
    assert_eq!(observed.receipts.len(), 1);
    assert_eq!(
        observed
            .members
            .iter()
            .find(|member| member.path == "receipts/verification.json")
            .unwrap()
            .raw,
        archive2a_native()
    );

    for signed in [false, true] {
        let streamed = archive2a_bundle_encoded(
            &[("receipts/verification.json", archive2a_native())],
            true,
            Some(signed),
        );
        let parsed = archive::inspect_untrusted_github_archive(
            &streamed,
            &ArchiveDigest::sha256_bytes(&streamed),
        )
        .unwrap();
        assert_eq!(parsed.receipts.len(), 1);
        assert_eq!(
            parsed
                .members
                .iter()
                .find(|member| member.path == "receipts/verification.json")
                .unwrap()
                .raw,
            archive2a_native()
        );
    }
    let zipped = archive2a_bundle_encoded(
        &[("receipts/verification.json", archive2a_native())],
        true,
        None,
    );
    let central = archive2a_central(&zipped);
    let compressed = u32::from_le_bytes(zipped[central + 20..central + 24].try_into().unwrap());
    let data_end = 30 + "manifest.json".len() + compressed as usize;
    let mut trailing = archive2a_insert_first(&zipped, data_end, &[0]);
    let central = archive2a_central(&trailing);
    trailing[18..22].copy_from_slice(&(compressed + 1).to_le_bytes());
    trailing[central + 20..central + 24].copy_from_slice(&(compressed + 1).to_le_bytes());
    archive2a_reject(&trailing, ArchiveError::Checksum);
}

#[test]
fn ci_admission_archive_count_and_offsets_reject_before_decoder() {
    let valid = archive2a_bundle(archive2a_native());
    let eocd = valid.len() - 22;
    let mut count = valid.clone();
    count[eocd + 8..eocd + 12].copy_from_slice(&[65, 0, 65, 0]);
    let mut offset = valid.clone();
    offset[eocd + 16..eocd + 20].copy_from_slice(&(u32::MAX - 2).to_le_bytes());
    let mut length = valid.clone();
    length[eocd + 12..eocd + 16].copy_from_slice(&(u32::MAX - 1).to_le_bytes());
    for (raw, expected) in [
        (count, ArchiveError::DirectoryLimit),
        (offset, ArchiveError::DirectoryBounds),
        (length, ArchiveError::DirectoryBounds),
    ] {
        let entered = std::cell::Cell::new(false);
        let actual =
            archive::inspect_with_decoder(&raw, &ArchiveDigest::sha256_bytes(&raw), |_, _| {
                entered.set(true);
                Err(ArchiveError::NotImplemented)
            });
        assert_eq!(actual.unwrap_err(), expected);
        assert!(!entered.get());
    }

    let names = (0..64)
        .map(|index| format!("entry-{index:03}.json"))
        .collect::<Vec<_>>();
    let entries = names
        .iter()
        .map(|name| (name.as_str(), b"{}".to_vec()))
        .collect::<Vec<_>>();
    let exact = archive2a_zip(&entries);
    let entered = std::cell::Cell::new(false);
    let result =
        archive::inspect_with_decoder(&exact, &ArchiveDigest::sha256_bytes(&exact), |_, _| {
            entered.set(true);
            Err(ArchiveError::NotImplemented)
        });
    assert_eq!(result.unwrap_err(), ArchiveError::NotImplemented);
    assert!(
        entered.get(),
        "exact64 directory bound must reach the injected decoder"
    );
}

#[test]
fn ci_admission_archive_zip64_multidisk_encryption_and_method_are_unsupported() {
    let valid = archive2a_bundle(archive2a_native());
    let eocd = valid.len() - 22;
    let central = archive2a_central(&valid);
    let mut zip64 = valid.clone();
    zip64[eocd + 8..eocd + 12].fill(0xff);
    let mut disk = valid.clone();
    disk[eocd + 4] = 1;
    let mut encrypted = valid.clone();
    encrypted[6] = 1;
    encrypted[central + 8] = 1;
    let mut method = valid.clone();
    method[8] = 12;
    method[central + 10] = 12;
    for raw in [zip64, disk, encrypted, method] {
        archive2a_reject(&raw, ArchiveError::UnsupportedFormat);
    }

    for id in [0x0001_u16, 0x7075, 0x6375, 0x9901] {
        let mut extra = Vec::new();
        archive2a_u16(&mut extra, id);
        archive2a_u16(&mut extra, 0);
        let mut changed = archive2a_insert_first(&valid, 30 + "manifest.json".len(), &extra);
        changed[28..30].copy_from_slice(&4_u16.to_le_bytes());
        archive2a_reject(&changed, ArchiveError::UnsupportedFormat);
    }
}

#[test]
fn ci_admission_archive_paths_aliases_duplicates_and_special_files_are_rejected() {
    for name in [
        "../receipt.json",
        "/receipt.json",
        "C:/receipt.json",
        "receipts\\receipt.json",
        "receipts//receipt.json",
        "receipts/./receipt.json",
        "receipts/../receipt.json",
        "receipt\0.json",
    ] {
        archive2a_reject(
            &archive2a_zip(&[(name, b"{}".to_vec())]),
            ArchiveError::Path,
        );
    }
    for name in ["manifest.json", "MANIFEST.JSON"] {
        archive2a_reject(
            &archive2a_zip(&[("manifest.json", b"{}".to_vec()), (name, b"{}".to_vec())]),
            ArchiveError::DuplicateEntry,
        );
    }
    let valid = archive2a_bundle(archive2a_native());
    let central = archive2a_central(&valid);
    for mode in [0o120777_u32, 0o010600, 0o020600, 0o140600] {
        let mut changed = valid.clone();
        changed[central + 38..central + 42].copy_from_slice(&(mode << 16).to_le_bytes());
        archive2a_reject(&changed, ArchiveError::SpecialFile);
    }

    for name in [
        "receipts/CON.json",
        "receipts/NUL.json",
        "receipts/COM1.json",
        "receipts/a.json.",
        "receipts/a%2e.json",
        "receipts/a：b.json",
    ] {
        archive2a_reject(
            &archive2a_zip(&[(name, b"{}".to_vec())]),
            ArchiveError::Path,
        );
    }
    for flag in [0x10_u32, 0x40, 0x400] {
        let mut changed = valid.clone();
        changed[central + 38..central + 42]
            .copy_from_slice(&((0o100600 << 16) | flag).to_le_bytes());
        archive2a_reject(&changed, ArchiveError::SpecialFile);
    }
}

#[test]
fn ci_admission_archive_truncation_crc_and_directory_local_name_conflict_are_rejected() {
    let valid = archive2a_bundle(archive2a_native());
    for cut in [1, 10, 22] {
        archive2a_reject(&valid[..valid.len() - cut], ArchiveError::ZipStructure);
    }
    let mut crc = valid.clone();
    crc[30 + "manifest.json".len()] ^= 1;
    archive2a_reject(&crc, ArchiveError::Checksum);
    let mut name = valid.clone();
    name[30] = b'X';
    archive2a_reject(&name, ArchiveError::ZipStructure);

    let central = archive2a_central(&valid);
    let old = u32::from_le_bytes(valid[central + 20..central + 24].try_into().unwrap());
    let mut overlapping = valid.clone();
    for offset in [18, 22, central + 20, central + 24] {
        overlapping[offset..offset + 4].copy_from_slice(&(old + 1).to_le_bytes());
    }
    let entered = std::cell::Cell::new(false);
    let result = archive::inspect_with_decoder(
        &overlapping,
        &ArchiveDigest::sha256_bytes(&overlapping),
        |_, _| {
            entered.set(true);
            Err(ArchiveError::NotImplemented)
        },
    );
    assert_eq!(result.unwrap_err(), ArchiveError::ZipStructure);
    assert!(
        !entered.get(),
        "overlapping checked local/data spans must fail before decoding"
    );
}

#[test]
fn ci_admission_archive_declared_expansion_and_ratio_are_checked_before_decoder() {
    let valid = archive2a_bundle(archive2a_native());
    let central = archive2a_central(&valid);
    for (size, expected) in [
        (64 * 1024 * 1024_u32 + 1, ArchiveError::ExpandedSize),
        (8 * 1024 * 1024, ArchiveError::ExpansionRatio),
    ] {
        let mut changed = valid.clone();
        changed[central + 24..central + 28].copy_from_slice(&size.to_le_bytes());
        let entered = std::cell::Cell::new(false);
        let actual = archive::inspect_with_decoder(
            &changed,
            &ArchiveDigest::sha256_bytes(&changed),
            |_, _| {
                entered.set(true);
                Err(ArchiveError::NotImplemented)
            },
        );
        assert_eq!(actual.unwrap_err(), expected);
        assert!(!entered.get());
    }

    let compressed = archive2a_deflate(b"hello");
    assert_eq!(
        archive::decode_deflate_exact(&compressed, 5, 5).unwrap(),
        b"hello"
    );
    assert_eq!(
        archive::decode_deflate_exact(&compressed, 4, 5),
        Err(ArchiveError::ExpandedSize)
    );
    assert_eq!(
        archive::decode_deflate_exact(&compressed, 5, 4),
        Err(ArchiveError::ExpandedSize)
    );
    assert_eq!(
        archive::decode_deflate_exact(&compressed, 6, 6),
        Err(ArchiveError::Checksum)
    );
    assert_eq!(
        archive::decode_deflate_exact(&compressed[..compressed.len() - 1], 5, 5),
        Err(ArchiveError::Checksum)
    );
    let mut trailing = compressed;
    trailing.push(0);
    assert_eq!(
        archive::decode_deflate_exact(&trailing, 5, 5),
        Err(ArchiveError::Checksum)
    );
}

#[test]
fn ci_admission_archive_exact_manifest_inventory_and_member_digest_are_mandatory() {
    let native = archive2a_native();
    let valid = json!({"schemaVersion":1,"entries":[{"path":"receipts/verification.json","role":"native_verification_evidence","bytes":native.len(),"digest":ArchiveDigest::sha256_bytes(&native)}]});
    for (key, value, expected) in [
        ("schemaVersion", json!(2), ArchiveError::Manifest),
        ("trusted", json!(true), ArchiveError::Manifest),
    ] {
        let mut m = valid.clone();
        m[key] = value;
        archive2a_reject(
            &archive2a_zip(&[
                ("manifest.json", serde_json::to_vec(&m).unwrap()),
                ("receipts/verification.json", native.clone()),
            ]),
            expected,
        );
    }
    // Equal duplicate values are invalid syntax too. A Value-first parser
    // would silently retain one bytes field and falsely accept this member.
    let duplicate_nested = serde_json::to_string(&valid).unwrap().replacen(
        &format!("\"bytes\":{}", native.len()),
        &format!("\"bytes\":{},\"bytes\":{}", native.len(), native.len()),
        1,
    );
    archive2a_reject(
        &archive2a_zip(&[
            ("manifest.json", duplicate_nested.into_bytes()),
            ("receipts/verification.json", native.clone()),
        ]),
        ArchiveError::Manifest,
    );
    let mut bad_digest = valid.clone();
    bad_digest["entries"][0]["digest"] = json!(ArchiveDigest::sha256_bytes(b"different"));
    archive2a_reject(
        &archive2a_zip(&[
            ("manifest.json", serde_json::to_vec(&bad_digest).unwrap()),
            ("receipts/verification.json", native.clone()),
        ]),
        ArchiveError::EntryDigest,
    );
    archive2a_reject(
        &archive2a_zip(&[
            ("manifest.json", serde_json::to_vec(&valid).unwrap()),
            ("receipts/verification.json", native),
            ("extra.json", b"{}".to_vec()),
        ]),
        ArchiveError::Manifest,
    );
    archive2a_reject(
        &archive2a_zip(&[("other-manifest.json", b"{}".to_vec())]),
        ArchiveError::Manifest,
    );

    let native = archive2a_native();
    let entry = json!({"path":"receipts/verification.json","role":"native_verification_evidence","bytes":native.len(),"digest":ArchiveDigest::sha256_bytes(&native)});
    let duplicated = json!({"schemaVersion":1,"entries":[entry.clone(),entry]});
    archive2a_reject(
        &archive2a_zip(&[
            ("manifest.json", serde_json::to_vec(&duplicated).unwrap()),
            ("receipts/verification.json", native),
        ]),
        ArchiveError::Manifest,
    );
}

#[test]
fn ci_admission_archive_native_failure_timeout_truncation_and_accounting_are_not_success() {
    let original: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    for pointer in [
        "/receipt/results/0/passed",
        "/receipt/executionRecords/0/passed",
        "/receipt/executionRecords/0/timedOut",
        "/receipt/results/0/timedOut",
        "/receipt/results/0/outputTruncated",
        "/receipt/executionRecords/0/stderrTruncated",
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = json!(!pointer.ends_with("passed"));
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for pointer in [
        "/receipt/nodesPlanned",
        "/receipt/nodesExecuted",
        "/receipt/processesSpawned",
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = json!(2);
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }

    for (field, invalid) in [
        ("exitCode", json!(1)),
        ("exitCode", Value::Null),
        ("terminationSignal", json!(15)),
        ("spawned", json!(false)),
        ("stdoutTruncated", json!(true)),
        ("timeoutSeconds", json!(0)),
        ("deadlineMs", json!(0)),
        ("elapsedMs", json!(900001)),
        ("stdout", json!("ff")),
        ("stderr", json!("nothex")),
        ("commandDigest", json!("sha256:bad")),
    ] {
        let mut changed = original.clone();
        changed["receipt"]["executionRecords"][0][field] = invalid;
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for (field, invalid) in [
        ("schemaVersion", json!(2)),
        ("advisoryOnly", json!(false)),
        ("confidence", json!("unknown")),
        ("unknowns", json!(["forged_complete_cost"])),
        (
            "repositoryId",
            json!(ArchiveDigest::sha256_bytes(b"foreign-repository")),
        ),
        (
            "runtimeDigest",
            json!(ArchiveDigest::sha256_bytes(b"foreign-runtime")),
        ),
        ("nodesExecuted", json!(2)),
        ("elapsedMs", json!(44081)),
        ("providerAuthenticated", json!(true)),
    ] {
        let mut changed = original.clone();
        changed["receipt"]["costObservation"][field] = invalid;
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for (pointer, invalid) in [
        ("/receipt/results/0/state", json!("fresh")),
        ("/receipt/results/0/reason", json!("fresh_exact_binding")),
        ("/receipt/planReceipt/stage", json!("unknown_stage")),
        (
            "/receipt/planReceipt/reusedNodes",
            json!(["project-command-0"]),
        ),
        (
            "/receipt/planReceipt/sharedNodes",
            json!(["project-command-0"]),
        ),
        ("/receipt/planReceipt/savedExecutions", json!(1)),
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = invalid;
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for pointer in ["", "/receipt", "/receipt/planReceipt"] {
        let mut changed = original.clone();
        changed.pointer_mut(pointer).unwrap()["qualityGateReceipt"] = json!({"passed":true});
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    for invalid_version in ["", "1", "v1.0.1", "01.0.1", "1.0.1-01", "1.0.1+"] {
        let mut changed = original.clone();
        // Keep all identities equal so this exercises version structure itself.
        changed["runtimeVersion"] = json!(invalid_version);
        changed["receipt"]["runtimeVersion"] = json!(invalid_version);
        changed["receipt"]["costObservation"]["runtimeVersion"] = json!(invalid_version);
        changed["receiptDigest"] =
            json!(cockpit_protocol::digest_json(&changed["receipt"]).unwrap());
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
}

#[test]
fn ci_admission_archive_receipt_digest_and_identity_mismatch_are_rejected() {
    let original: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    for key in [
        "workItemId",
        "repositoryId",
        "runtimeVersion",
        "runtimeDigest",
    ] {
        let mut changed = original.clone();
        changed[key] = json!("different");
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    let mut changed = original;
    changed["receiptDigest"] = json!(ArchiveDigest::sha256_bytes(b"different"));
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
        ArchiveError::NativeReceipt,
    );
}

#[test]
fn ci_admission_archive_aggregate_expansion_and_node_claim_limits_are_not_ignored() {
    let mut raw = archive2a_zip(&[
        ("a.json", vec![0]),
        ("b.json", vec![0]),
        ("c.json", vec![0]),
    ]);
    let central = archive2a_central(&raw);
    for index in 0..3 {
        let start = central + index * (46 + "a.json".len());
        raw[start + 24..start + 28].copy_from_slice(&(64 * 1024 * 1024_u32).to_le_bytes());
    }
    // Total metadata must be bounded before allocating/decompressing entries;
    // aggregate bound takes precedence over later ratio/span checks.
    archive2a_reject(&raw, ArchiveError::ExpandedSize);
    let mut envelope: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    let result = envelope["receipt"]["results"][0].clone();
    envelope["receipt"]["results"] = Value::Array(vec![result; 4097]);
    envelope["receiptDigest"] = json!(cockpit_protocol::digest_json(&envelope["receipt"]).unwrap());
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&envelope).unwrap()),
        ArchiveError::NodeLimit,
    );

    let exact = archive2a_bundle(archive2a_claims_native(4096, "exact"));
    let parsed =
        archive::inspect_untrusted_github_archive(&exact, &ArchiveDigest::sha256_bytes(&exact))
            .unwrap();
    assert_eq!(parsed.receipts[0].receipt.results.len(), 4096);
    let aggregate = archive2a_bundle_encoded(
        &[
            ("receipts/a.json", archive2a_claims_native(2048, "a")),
            ("receipts/b.json", archive2a_claims_native(2048, "b")),
        ],
        false,
        None,
    );
    assert_eq!(
        archive::inspect_untrusted_github_archive(
            &aggregate,
            &ArchiveDigest::sha256_bytes(&aggregate)
        )
        .unwrap()
        .receipts
        .len(),
        2
    );
    let overflow = archive2a_bundle_encoded(
        &[
            ("receipts/a.json", archive2a_claims_native(2048, "a")),
            ("receipts/b.json", archive2a_claims_native(2049, "b")),
        ],
        false,
        None,
    );
    archive2a_reject(&overflow, ArchiveError::NodeLimit);
    let mut counter: Value = serde_json::from_slice(&archive2a_native()).unwrap();
    counter["receipt"]["nodesPlanned"] = json!(u64::MAX);
    counter["receiptDigest"] = json!(cockpit_protocol::digest_json(&counter["receipt"]).unwrap());
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&counter).unwrap()),
        ArchiveError::NodeLimit,
    );
}

#[test]
fn ci_admission_archive_duplicate_json_missing_records_and_producer_reuse_remain_rejected() {
    let original = String::from_utf8(archive2a_native()).unwrap();
    let duplicate = original.replacen("\"passed\":true", "\"passed\":true,\"passed\":true", 1);
    assert_ne!(original, duplicate);
    archive2a_reject(
        &archive2a_bundle(duplicate.into_bytes()),
        ArchiveError::NativeReceipt,
    );
    // Use a valid real native retention schema, then duplicate an equal
    // persistence key at depth3; outer deny_unknown_fields cannot catch it.
    let parsed: Value = serde_json::from_str(&original).unwrap();
    let retention = json!({"schemaVersion":1,"repositoryId":parsed["repositoryId"],
        "workItemId":parsed["workItemId"],"createdAt":parsed["createdAt"],
        "retention":{"classification":"internal","persistence":"full_capture",
            "retentionDays":30,"expiresAt":null,"disposalAction":"retain"}});
    let policy: cockpit_protocol::EvidenceRetentionPolicy =
        serde_json::from_value(retention.clone()).unwrap();
    assert!(cockpit_protocol::validate_evidence_retention(&policy.retention).is_ok());
    let mut with_retention = parsed.clone();
    with_retention["retention"] = retention.clone();
    let raw_with_retention = archive2a_bundle(serde_json::to_vec(&with_retention).unwrap());
    let inspected_with_retention = archive::inspect_untrusted_github_archive(
        &raw_with_retention,
        &ArchiveDigest::sha256_bytes(&raw_with_retention),
    )
    .unwrap();
    assert_eq!(inspected_with_retention.receipts.len(), 1);
    assert_eq!(
        inspected_with_retention.receipts[0].retention,
        Some(retention.clone())
    );
    for (pointer, invalid) in [
        ("/retention/schemaVersion", json!(2)),
        ("/retention/workItemId", json!("WI-FOREIGN")),
        (
            "/retention/repositoryId",
            json!(ArchiveDigest::sha256_bytes(b"foreign-retention")),
        ),
        ("/retention/createdAt", json!("not-a-timestamp")),
        ("/retention/retention/retentionDays", json!(0)),
        ("/retention/retention/retentionDays", json!(u64::MAX)),
        ("/retention/retention/persistence", json!("digest_only")),
        (
            "/retention/retention/classification",
            json!("secret_prohibited"),
        ),
        ("/retention/retention/disposalAction", json!("")),
    ] {
        let mut changed = with_retention.clone();
        *changed.pointer_mut(pointer).unwrap() = invalid;
        archive2a_reject(
            &archive2a_bundle(serde_json::to_vec(&changed).unwrap()),
            ArchiveError::NativeReceipt,
        );
    }
    let mut unknown_retention = with_retention;
    unknown_retention["retention"]["retention"]["providerAuthenticated"] = json!(true);
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&unknown_retention).unwrap()),
        ArchiveError::NativeReceipt,
    );
    let duplicate_retention = serde_json::to_string(&retention).unwrap().replacen(
        "\"persistence\":\"full_capture\"",
        "\"persistence\":\"full_capture\",\"persistence\":\"full_capture\"",
        1,
    );
    let deep_duplicate =
        original.replacen('{', &format!("{{\"retention\":{duplicate_retention},"), 1);
    archive2a_reject(
        &archive2a_bundle(deep_duplicate.into_bytes()),
        ArchiveError::NativeReceipt,
    );
    let mut missing: Value = serde_json::from_str(&original).unwrap();
    missing["receipt"]
        .as_object_mut()
        .unwrap()
        .remove("executionRecords");
    missing["receiptDigest"] = json!(cockpit_protocol::digest_json(&missing["receipt"]).unwrap());
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&missing).unwrap()),
        ArchiveError::NativeReceipt,
    );
    let mut reused: Value = serde_json::from_str(&original).unwrap();
    reused["receipt"]["results"][0]["reused"] = json!(true);
    reused["receiptDigest"] = json!(cockpit_protocol::digest_json(&reused["receipt"]).unwrap());
    // No authenticated upstream proof chain is introduced by Batch2A.
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&reused).unwrap()),
        ArchiveError::NativeReceipt,
    );

    let deep = format!("{}0{}", "[".repeat(65), "]".repeat(65));
    let deep = original.replacen('{', &format!("{{\"retention\":{deep},"), 1);
    archive2a_reject(
        &archive2a_bundle(deep.into_bytes()),
        ArchiveError::NodeLimit,
    );
    let probe = json!(vec![vec![0; 64]; 4096]);
    let mut many: Value = serde_json::from_str(&original).unwrap();
    many["untrustedBudgetProbe"] = probe;
    archive2a_reject(
        &archive2a_bundle(serde_json::to_vec(&many).unwrap()),
        ArchiveError::NodeLimit,
    );
}

// Append inside the existing ci_admission/tests.rs module. Only the parent
// heavy worker may compile/run these tests. No test or formatter was run here.
// The actual current native full-environment identity is proven only on Unix;
// these positive-witness cases do not manufacture support on other platforms.

#[cfg(unix)]
fn c16_native_fixture(
    argument: &str,
) -> (
    tempfile::TempDir,
    cockpit_protocol::RuntimeContext,
    cockpit_protocol::TaskAuthorizationCaller,
    crate::RepositoryVerificationRequest,
) {
    use crate::*;
    let root = tempfile::Builder::new()
        .prefix("c16 native witness ")
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
    confirm_profile_update(root.path(), "git", &[argument.to_owned()]).unwrap();
    let runtime = cockpit_protocol::RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: cockpit_core::Digest::sha256_bytes(
            b"c16-native-fixture-runtime-not-provider-proof",
        ),
    };
    let caller = cockpit_protocol::TaskAuthorizationCaller {
        schema_version: 1,
        actor: "agent:c16-native-fixture".into(),
        authority_source: "fixture:synthetic-owner-not-provider-authentication".into(),
        assurance: cockpit_protocol::AssuranceLevel::SelfDeclared,
    };
    let wi = "WI-C16-NATIVE-WITNESS";
    start_work_item_with_options_and_runtime(
        root.path(),
        wi,
        "observe an actual current native child",
        "do not substitute cold evidence",
        &["tracked.txt".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec![
                "only this invocation's actual bounded child can supply its witness".into(),
            ],
            verification_commands: vec![format!("git {argument}")],
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
        .join(format!(".ai/work-items/active/{wi}.contract.json"));
    preflight_work_item_with_runtime_report(root.path(), &contract_path, &runtime).unwrap();
    let contract: Value = crate::read_json(&contract_path).unwrap();
    let summary: Value = crate::read_json(
        &root
            .path()
            .join(format!(".ai/work-items/active/{wi}.summary.json")),
    )
    .unwrap();
    record_work_item_governance_controls_with_runtime(root.path(), wi, &json!({
        "decisionEvidence": {
            "schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review",
            "workItemId":wi,"repositoryId":crate::repository_id(root.path()),
            "contractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
            "preflightDecisionDigest":summary["preflightDecisionDigest"],
            "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
            "recordedAt":chrono::Utc::now().to_rfc3339(),"recordedBy":"agent:test-recorder",
            "reason":"Synthetic local fixture, not external human or provider authentication."
        }
    }), &runtime).unwrap();
    checkpoint_work_item_with_caller(root.path(), wi, &runtime, Some(&caller)).unwrap();
    let request = RepositoryVerificationRequest {
        node_id: "project-command-0".into(),
        program: "git".into(),
        args: vec![argument.into()],
        scope: vec!["tracked.txt".into()],
        stage: "task".into(),
        runner: "local".into(),
        runtime_digest: runtime.runtime_digest.to_string(),
        base_commit: None,
        workers: 1,
        work_item_id: Some(wi.into()),
        timeout_seconds: None,
        policy: RepositoryVerificationPolicy::ProfileAuthorized,
    };
    (root, runtime, caller, request)
}

#[cfg(unix)]
#[test]
fn ci_admission_c16_native_real_execution_can_substitute_only_with_its_live_witness() {
    let (root, runtime, caller, request) = c16_native_fixture("--version");
    let invocation = crate::CurrentNativeInvocation::new();
    let observed = crate::run_repository_verification_with_native_witness(
        root.path(),
        &request,
        &runtime,
        Some(&caller),
        &invocation,
        None,
    )
    .expect("RED: the actual bounded native runner must produce a real result");
    assert!(observed.run.receipt.passed);
    assert_eq!(observed.run.receipt.processes_spawned, 1);
    assert_eq!(observed.run.receipt.nodes_executed, 1);
    assert_eq!(observed.run.receipt.nodes_reused, 0);
    assert_eq!(observed.run.receipt.execution_records.len(), 1);
    let physical = &observed.run.receipt.execution_records[0];
    assert!(physical.spawned && physical.passed);
    assert_eq!(physical.exit_code, Some(0));
    assert_eq!(physical.termination_signal, None);
    assert!(!physical.stdout_hex.is_empty());
    assert!(!physical.stdout_truncated && !physical.stderr_truncated && !physical.timed_out);
    let witness = observed
        .witness
        .expect("an actual stable native child must have its live witness");
    assert_eq!(witness.receipt(), &observed.run.receipt);
    assert_eq!(witness.native_input()["nodeId"], request.node_id);
    assert_eq!(
        witness.native_input()["requestPolicy"],
        "profile_authorized"
    );
    let substituted = crate::consume_native_execution_witness(
        root.path(),
        &request,
        &runtime,
        Some(&caller),
        &invocation,
        witness,
    )
    .expect("the same current invocation must consume its exact unchanged witness");
    assert!(substituted.receipt.passed);
    assert_eq!(substituted.receipt.nodes_planned, 1);
    assert_eq!(substituted.receipt.nodes_executed, 0);
    assert_eq!(substituted.receipt.nodes_reused, 1);
    assert_eq!(substituted.receipt.processes_spawned, 0);
    assert!(substituted.receipt.execution_records.is_empty());
    assert!(substituted.receipt.receipt_candidates.is_empty());
    assert_eq!(substituted.receipt.results.len(), 1);
    assert!(substituted.receipt.results[0].reused);
    match &substituted.receipt.results[0].satisfied_by {
        cockpit_verification::PlannedSatisfaction::PortableNativeInvocation {
            publication_digest,
        } => {
            assert!(publication_digest.parse::<cockpit_core::Digest>().is_ok());
        }
        other => panic!("expected native invocation projection, got {other:?}"),
    }
}

#[cfg(unix)]
#[test]
fn ci_admission_c16_native_identical_wire_and_preseeded_objects_do_not_transfer_invocations() {
    let (root, runtime, caller, request) = c16_native_fixture("--version");
    let invocation = crate::CurrentNativeInvocation::new();
    let observed = crate::run_repository_verification_with_native_witness(
        root.path(),
        &request,
        &runtime,
        Some(&caller),
        &invocation,
        None,
    )
    .expect("RED: obtain the witness through a real native child");
    assert_eq!(observed.run.receipt.processes_spawned, 1);
    let witness = observed.witness.expect("live native witness");
    let wire = json!({"schemaVersion":1,"nativeInput":witness.native_input(),"nativeEvidence":witness.receipt()});
    let bytes = serde_json::to_vec(&wire).unwrap();
    let digest = cockpit_core::Digest::sha256_bytes(&bytes);
    let directory = root.path().join(".ai/evidence/c16-caller-prefilled");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("objects.json"), &bytes).unwrap();
    std::fs::write(
        directory.join("marker.json"),
        serde_json::to_vec(&json!({"committed":true,"objectDigest":digest})).unwrap(),
    )
    .unwrap();
    let foreign = crate::CurrentNativeInvocation::new();
    assert!(
        crate::consume_native_execution_witness(
            root.path(),
            &request,
            &runtime,
            Some(&caller),
            &foreign,
            witness,
        )
        .is_err(),
        "identical bytes and all caller-created hashes cannot transfer the live invocation"
    );
    // The parent's external import plan must additionally exercise the same
    // bytes with no witness at all and retain its source-unverified gap.
}

#[cfg(unix)]
#[test]
fn ci_admission_c16_native_witness_rechecks_source_and_current_caller() {
    for drift in ["source", "caller"] {
        let (root, runtime, caller, request) = c16_native_fixture("--version");
        let invocation = crate::CurrentNativeInvocation::new();
        let observed = crate::run_repository_verification_with_native_witness(
            root.path(),
            &request,
            &runtime,
            Some(&caller),
            &invocation,
            None,
        )
        .expect("RED: obtain the witness through a real native child");
        assert_eq!(observed.run.receipt.processes_spawned, 1);
        let witness = observed.witness.expect("live native witness");
        let mut current_caller = caller.clone();
        if drift == "source" {
            std::fs::write(root.path().join("tracked.txt"), "after\n").unwrap();
        } else {
            current_caller.authority_source.push_str("-foreign");
        }
        assert!(
            crate::consume_native_execution_witness(
                root.path(),
                &request,
                &runtime,
                Some(&current_caller),
                &invocation,
                witness,
            )
            .is_err(),
            "current {drift} drift must refuse zero-spawn substitution"
        );
    }
}

#[cfg(unix)]
#[test]
fn ci_admission_c16_native_witness_cannot_override_never_reuse_or_protected_policy() {
    for policy in [
        crate::RepositoryVerificationPolicy::NeverReuse,
        crate::RepositoryVerificationPolicy::Protected(
            cockpit_verification::ProtectedGateClass::Security,
        ),
    ] {
        let (root, runtime, caller, mut request) = c16_native_fixture("--version");
        let invocation = crate::CurrentNativeInvocation::new();
        let observed = crate::run_repository_verification_with_native_witness(
            root.path(),
            &request,
            &runtime,
            Some(&caller),
            &invocation,
            None,
        )
        .expect("RED: obtain the witness through a real profile-authorized child");
        assert_eq!(observed.run.receipt.processes_spawned, 1);
        let witness = observed.witness.expect("live native witness");
        request.policy = policy;
        assert!(
            crate::consume_native_execution_witness(
                root.path(),
                &request,
                &runtime,
                Some(&caller),
                &invocation,
                witness,
            )
            .is_err(),
            "a witness must not override current restrictive policy"
        );
    }
}

#[cfg(unix)]
#[test]
fn ci_admission_c16_native_actual_failed_child_is_preserved_without_a_witness() {
    let (root, runtime, caller, request) = c16_native_fixture("--definitely-invalid-c16-option");
    let invocation = crate::CurrentNativeInvocation::new();
    let observed = crate::run_repository_verification_with_native_witness(
        root.path(),
        &request,
        &runtime,
        Some(&caller),
        &invocation,
        None,
    )
    .expect("RED: preserve the actual native failure result");
    assert!(!observed.run.receipt.passed);
    assert_eq!(observed.run.receipt.processes_spawned, 1);
    assert_eq!(observed.run.receipt.execution_records.len(), 1);
    assert!(observed.run.receipt.execution_records[0].spawned);
    assert_ne!(observed.run.receipt.execution_records[0].exit_code, Some(0));
    assert!(
        !observed.run.receipt.execution_records[0]
            .stderr_hex
            .is_empty()
    );
    assert!(
        observed.witness.is_none(),
        "failure cannot produce a usable source witness"
    );
}

// Append directly to ci_admission/tests.rs. These tests call the pure I/O store,
// not acceptance or source-authentication. All paths belong to each TempDir.
const PORTABLE_STORE_TEST_WI: &str = "WI-PORTABLE-STORE";

fn portable_store_test_digest(bytes: &[u8]) -> String {
    cockpit_core::Digest::sha256_bytes(bytes).to_string()
}

fn portable_store_test_marker(id: &str, input: &[u8]) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1,
        "recordId": id,
        "inputDigest": portable_store_test_digest(input),
    }))
    .expect("portable marker fixture")
}

fn portable_store_test_directory(root: &std::path::Path) -> std::path::PathBuf {
    root.join(format!(
        ".ai/evidence/{PORTABLE_STORE_TEST_WI}.verification-admissions/portable"
    ))
}

fn portable_store_test_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let owner = tempfile::tempdir().expect("own portable store fixture");
    let root = std::fs::canonicalize(owner.path()).expect("canonical fixture root");
    (owner, root)
}

#[test]
fn portable_store_missing_record_does_not_create_a_store() {
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"missing binding");
    assert!(
        super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id)
            .expect("missing store is a read-only absence")
            .is_none()
    );
    assert!(
        !root.join(".ai").exists(),
        "read-only miss must not create directories"
    );
}

#[test]
fn portable_store_publish_roundtrip_preserves_exact_untrusted_bytes() {
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"roundtrip binding");
    let input = b"untrusted portable execution bytes";
    let marker = portable_store_test_marker(&id, input);
    let published =
        super::store::publish_portable_record(&root, PORTABLE_STORE_TEST_WI, &id, input, &marker)
            .expect("publish complete immutable record");
    assert_eq!(published.input, input);
    assert_eq!(published.marker, marker);
    assert_eq!(
        published.record_ref,
        format!(
            ".ai/evidence/{PORTABLE_STORE_TEST_WI}.verification-admissions/portable/records/{}.json",
            &id[7..]
        )
    );
    let loaded = super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id)
        .expect("read committed record")
        .expect("final marker exists");
    assert_eq!(loaded, published);
    let object = portable_store_test_directory(&root)
        .join("objects")
        .join(format!("{}.bin", &portable_store_test_digest(input)[7..]));
    assert_eq!(std::fs::read(object).expect("immutable object"), input);
}

#[test]
fn portable_store_exact_repeat_is_idempotent_without_replacing_marker() {
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"idempotent binding");
    let input = b"same bytes";
    let marker = portable_store_test_marker(&id, input);
    let first =
        super::store::publish_portable_record(&root, PORTABLE_STORE_TEST_WI, &id, input, &marker)
            .expect("first publication");
    let marker_path = root.join(&first.record_ref);
    let before = std::fs::metadata(&marker_path).expect("first marker metadata");
    let second =
        super::store::publish_portable_record(&root, PORTABLE_STORE_TEST_WI, &id, input, &marker)
            .expect("exact idempotent repeat");
    assert_eq!(second, first);
    assert_eq!(
        std::fs::read(&marker_path).expect("unchanged marker bytes"),
        marker
    );
    let after = std::fs::metadata(&marker_path).expect("repeated marker metadata");
    assert_eq!(
        before.modified().expect("first time"),
        after.modified().expect("second time")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!((before.dev(), before.ino()), (after.dev(), after.ino()));
    }
}

#[test]
fn portable_store_binding_conflict_is_refused_before_new_object_payload() {
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"one stable record id");
    let original = b"original object";
    let first_marker = portable_store_test_marker(&id, original);
    let first = super::store::publish_portable_record(
        &root,
        PORTABLE_STORE_TEST_WI,
        &id,
        original,
        &first_marker,
    )
    .expect("original committed record");
    let changed = b"different input under same record id";
    let changed_marker = portable_store_test_marker(&id, changed);
    assert!(
        super::store::publish_portable_record(
            &root,
            PORTABLE_STORE_TEST_WI,
            &id,
            changed,
            &changed_marker,
        )
        .is_err()
    );
    let changed_object = portable_store_test_directory(&root)
        .join("objects")
        .join(format!("{}.bin", &portable_store_test_digest(changed)[7..]));
    assert!(
        !changed_object.exists(),
        "known conflict must reject before new payload write"
    );
    assert_eq!(
        std::fs::read(root.join(&first.record_ref)).expect("original marker"),
        first_marker
    );
    assert_eq!(
        super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id)
            .expect("old record remains readable")
            .expect("old record remains present"),
        first
    );
}

#[test]
fn portable_store_invalid_marker_or_identifier_is_refused_without_payload() {
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"valid record binding");
    let input = b"valid bytes";
    let wrong_marker = portable_store_test_marker(&id, b"different bytes");
    assert!(
        super::store::publish_portable_record(
            &root,
            PORTABLE_STORE_TEST_WI,
            &id,
            input,
            &wrong_marker,
        )
        .is_err()
    );
    assert!(!root.join(".ai").exists());
    for bad in ["", "sha256:../escape", "SHA256:123", "sha256:ABCDEF"] {
        assert!(
            super::store::publish_portable_record(
                &root,
                PORTABLE_STORE_TEST_WI,
                bad,
                input,
                &portable_store_test_marker(bad, input),
            )
            .is_err()
        );
        assert!(!root.join(".ai").exists());
    }
    let duplicate = format!(
        "{{\"schemaVersion\":1,\"recordId\":\"{id}\",\"inputDigest\":\"{}\",\"inputDigest\":\"{}\"}}",
        portable_store_test_digest(input),
        portable_store_test_digest(input)
    );
    assert!(
        super::store::publish_portable_record(
            &root,
            PORTABLE_STORE_TEST_WI,
            &id,
            input,
            duplicate.as_bytes(),
        )
        .is_err()
    );
    let mut authority_flag: serde_json::Value =
        serde_json::from_slice(&portable_store_test_marker(&id, input)).unwrap();
    authority_flag["sourceVerified"] = true.into();
    assert!(
        super::store::publish_portable_record(
            &root,
            PORTABLE_STORE_TEST_WI,
            &id,
            input,
            &serde_json::to_vec(&authority_flag).unwrap()
        )
        .is_err()
    );
    assert!(
        !root.join(".ai").exists(),
        "invalid marker must not create any store"
    );
}

#[test]
fn portable_store_orphan_object_without_final_marker_is_not_a_record() {
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"orphan binding");
    let input = b"orphan bytes";
    let objects = portable_store_test_directory(&root).join("objects");
    std::fs::create_dir_all(&objects).expect("own orphan fixture");
    let path = objects.join(format!("{}.bin", &portable_store_test_digest(input)[7..]));
    std::fs::write(&path, input).expect("partial object");
    assert!(
        super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id)
            .expect("orphan does not create a marker")
            .is_none()
    );
    assert_eq!(
        std::fs::read(path).expect("partial evidence preserved"),
        input
    );
}

#[test]
fn portable_store_final_marker_with_missing_or_changed_object_fails_closed() {
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"damaged record binding");
    let input = b"original raw input";
    let dir = portable_store_test_directory(&root);
    let records = dir.join("records");
    let objects = dir.join("objects");
    std::fs::create_dir_all(&records).expect("own records fixture");
    std::fs::create_dir_all(&objects).expect("own objects fixture");
    let marker_path = records.join(format!("{}.json", &id[7..]));
    let marker = portable_store_test_marker(&id, input);
    std::fs::write(&marker_path, &marker).expect("own final marker fixture");
    assert!(super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id).is_err());
    let object_path = objects.join(format!("{}.bin", &portable_store_test_digest(input)[7..]));
    std::fs::write(&object_path, b"corrupt raw input").expect("own corrupt object");
    assert!(super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id).is_err());
    assert_eq!(std::fs::read(marker_path).expect("marker retained"), marker);
}

#[test]
fn portable_store_oversized_object_metadata_is_refused_before_read_allocation() {
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"oversized fixture binding");
    let input = b"expected small bytes";
    let dir = portable_store_test_directory(&root);
    std::fs::create_dir_all(dir.join("records")).unwrap();
    std::fs::create_dir_all(dir.join("objects")).unwrap();
    std::fs::write(
        dir.join("records").join(format!("{}.json", &id[7..])),
        portable_store_test_marker(&id, input),
    )
    .unwrap();
    let object_path = dir
        .join("objects")
        .join(format!("{}.bin", &portable_store_test_digest(input)[7..]));
    let object = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&object_path)
        .unwrap();
    object
        .set_len(super::store::MAX_PORTABLE_INPUT_BYTES + 1)
        .unwrap();
    assert!(super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id).is_err());
}

#[cfg(unix)]
#[test]
fn portable_store_symlink_parent_or_payload_is_rejected_without_touching_target() {
    use std::os::unix::fs::symlink;
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"symlink binding");
    let input = b"outside sentinel";
    let outside = root.join("outside");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("sentinel"), input).unwrap();
    symlink(&outside, root.join(".ai")).unwrap();
    assert!(
        super::store::publish_portable_record(
            &root,
            PORTABLE_STORE_TEST_WI,
            &id,
            input,
            &portable_store_test_marker(&id, input)
        )
        .is_err()
    );
    assert!(!outside.join("evidence").exists());
    assert_eq!(std::fs::read(outside.join("sentinel")).unwrap(), input);
    std::fs::remove_file(root.join(".ai")).unwrap();
    let dir = portable_store_test_directory(&root);
    std::fs::create_dir_all(dir.join("records")).unwrap();
    std::fs::create_dir_all(dir.join("objects")).unwrap();
    std::fs::write(
        dir.join("records").join(format!("{}.json", &id[7..])),
        portable_store_test_marker(&id, input),
    )
    .unwrap();
    symlink(
        outside.join("sentinel"),
        dir.join("objects")
            .join(format!("{}.bin", &portable_store_test_digest(input)[7..])),
    )
    .unwrap();
    assert!(super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id).is_err());
    assert_eq!(std::fs::read(outside.join("sentinel")).unwrap(), input);
}

#[cfg(unix)]
#[test]
fn portable_store_fifo_subprocess_helper() {
    if std::env::var_os("AI_COCKPIT_PORTABLE_STORE_HELPER_MODE").as_deref()
        != Some(std::ffi::OsStr::new("fifo_probe"))
    {
        return;
    }
    let root = std::path::PathBuf::from(
        std::env::var_os("AI_COCKPIT_PORTABLE_STORE_HELPER_ROOT").expect("owned helper root"),
    );
    let id = portable_store_test_digest(b"fifo binding");
    assert!(super::store::load_portable_record(&root, PORTABLE_STORE_TEST_WI, &id).is_err());
    use std::io::Write;
    let mut ack = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("fifo-probe.completed"))
        .expect("own FIFO probe acknowledgment");
    ack.write_all(b"rejected_fifo\n")
        .expect("FIFO probe acknowledgment bytes");
}

#[cfg(unix)]
#[test]
fn portable_store_fifo_payload_without_peer_is_rejected_with_bounded_probe() {
    use std::os::unix::ffi::OsStrExt;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let (_owner, root) = portable_store_test_fixture();
    let id = portable_store_test_digest(b"fifo binding");
    let input = b"FIFO never supplies these expected bytes";
    let dir = portable_store_test_directory(&root);
    std::fs::create_dir_all(dir.join("records")).unwrap();
    std::fs::create_dir_all(dir.join("objects")).unwrap();
    std::fs::write(
        dir.join("records").join(format!("{}.json", &id[7..])),
        portable_store_test_marker(&id, input),
    )
    .unwrap();
    let fifo_path = dir
        .join("objects")
        .join(format!("{}.bin", &portable_store_test_digest(input)[7..]));
    let cpath = std::ffi::CString::new(fifo_path.as_os_str().as_bytes()).unwrap();
    // SAFETY: the owned CString is a live NUL-terminated fixture path.
    assert_eq!(unsafe { libc::mkfifo(cpath.as_ptr(), 0o600) }, 0);
    let mut child = Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            "ci_admission::tests::portable_store_fifo_subprocess_helper",
            "--test-threads=1",
        ])
        .env("AI_COCKPIT_PORTABLE_STORE_HELPER_MODE", "fifo_probe")
        .env("AI_COCKPIT_PORTABLE_STORE_HELPER_ROOT", &root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("owned FIFO probe");
    let deadline = Instant::now() + Duration::from_secs(2);
    let outcome = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
            _ => break None,
        }
    };
    if let Some(status) = outcome {
        assert!(
            status.success(),
            "owned FIFO probe rejected incorrectly: {status}"
        );
        assert_eq!(
            std::fs::read(root.join("fifo-probe.completed")).expect("exact helper actually ran"),
            b"rejected_fifo\n"
        );
    } else {
        let _ = child.kill();
        let reap_deadline = Instant::now() + Duration::from_secs(2);
        let reaped = loop {
            match child.try_wait() {
                Ok(Some(_)) => break true,
                Ok(None) if Instant::now() < reap_deadline => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                _ => break false,
            }
        };
        assert!(reaped, "owned FIFO probe did not reap after bounded kill");
        panic!("portable-store FIFO read blocked; owned probe was killed and reaped");
    }
}

// Portable inspection claims remain unverified even when every byte/hash is valid.
fn c16_portable_input_fixture() -> serde_json::Value {
    json!({"schemaVersion":1,"nativeInput":{"schemaVersion":1,"producer":"untrusted_captured_fixture"},"nativeEvidence":serde_json::from_slice::<Value>(&archive2a_native()).unwrap()})
}
fn c16_portable_runtime() -> cockpit_protocol::RuntimeContext {
    cockpit_protocol::RuntimeContext {
        runtime_version: "1.0.1".into(),
        protocol_version: 1,
        runtime_digest: cockpit_core::Digest::sha256_bytes(b"portable-inspection-not-source-proof"),
    }
}
#[test]
fn ci_admission_c16_portable_inspection_needs_no_wi_or_execution_authority() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("evidence.json");
    std::fs::write(
        &input,
        serde_json::to_vec(&c16_portable_input_fixture()).unwrap(),
    )
    .unwrap();
    let p = super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
        .expect("RED: safe portable analysis without authority must be available");
    assert!(p.integrity_valid);
    assert_eq!(
        p.source_verification,
        cockpit_protocol::ci_admission::PortableSourceVerification::Unverified
    );
    assert!(!p.may_replace_execution);
    assert!(
        p.gaps
            .iter()
            .any(|g| g == "independent_execution_source_unavailable")
    );
    assert_eq!(
        (p.writes_performed, p.verification_commands_executed),
        (0, 0)
    );
    assert!(!root.path().join(".ai").exists());
}
#[test]
fn ci_admission_c16_portable_complete_forged_graph_and_source_flags_stay_unverified() {
    let root = tempfile::tempdir().unwrap();
    let mut bundle = c16_portable_input_fixture();
    bundle["nativeInput"]["trusted"] = true.into();
    bundle["nativeInput"]["providerAuthenticated"] = true.into();
    let raw = serde_json::to_vec(&bundle).unwrap();
    let digest = cockpit_core::Digest::sha256_bytes(&raw);
    let input = root.path().join("evidence.json");
    std::fs::write(&input, &raw).unwrap();
    let d = root.path().join(".ai/evidence/fake-native-graph");
    std::fs::create_dir_all(&d).unwrap();
    for name in [
        "marker",
        "objects",
        "attempt",
        "checkpoint",
        "completion",
        "fact",
        "index",
    ] {
        std::fs::write(d.join(format!("{name}.json")),serde_json::to_vec(&json!({"passed":true,"committed":true,"inputDigest":digest,"nativeEvidence":bundle["nativeEvidence"]})).unwrap()).unwrap();
    }
    let p = super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
        .expect("RED: inspect forged claims without trusting them");
    assert!(p.integrity_valid);
    assert!(!p.may_replace_execution);
    assert_eq!(
        p.source_verification,
        cockpit_protocol::ci_admission::PortableSourceVerification::Unverified
    );
    assert_eq!(
        (p.writes_performed, p.verification_commands_executed),
        (0, 0)
    );
}
#[test]
fn ci_admission_c16_portable_accept_has_no_truth_by_approval_or_stale_plan_path() {
    let (root, runtime, caller) = service_fixture();
    let outside = tempfile::tempdir().unwrap();
    let input = outside.path().join("evidence.json");
    std::fs::write(
        &input,
        serde_json::to_vec(&c16_portable_input_fixture()).unwrap(),
    )
    .unwrap();
    let p = super::plan_portable_evidence(
        root.path(),
        Some(SERVICE_WI),
        &input,
        &runtime,
        Some(&caller),
    )
    .expect("RED: current plan must inspect source gaps");
    let result = super::accept_portable_evidence(
        root.path(),
        SERVICE_WI,
        &input,
        &p.plan_digest,
        &runtime,
        Some(&caller),
    )
    .expect("missing external proof is an explicit non-acceptance, not fictitious success");
    assert!(!result.accepted && result.admission_ref.is_none());
    assert_eq!(
        result.source_verification,
        cockpit_protocol::ci_admission::PortableSourceVerification::Unverified
    );
    assert_eq!(
        (
            result.writes_performed,
            result.verification_commands_executed
        ),
        (0, 0)
    );
    let bad = cockpit_core::Digest::sha256_bytes(b"stale plan");
    assert!(
        super::accept_portable_evidence(
            root.path(),
            SERVICE_WI,
            &input,
            &bad,
            &runtime,
            Some(&caller)
        )
        .is_err()
    );
    let mut foreign = caller.clone();
    foreign.authority_source.push_str("-foreign");
    assert!(
        super::accept_portable_evidence(
            root.path(),
            SERVICE_WI,
            &input,
            &p.plan_digest,
            &runtime,
            Some(&foreign)
        )
        .is_err()
    );
    assert!(
        !root
            .path()
            .join(format!(
                ".ai/evidence/{SERVICE_WI}.verification-admissions/portable"
            ))
            .exists()
    );
}
#[test]
fn ci_admission_c16_portable_failed_claim_is_viewable_but_cannot_skip_work() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("failed.json");
    let mut v = c16_portable_input_fixture();
    v["nativeEvidence"]["passed"] = false.into();
    v["nativeEvidence"]["receipt"]["passed"] = false.into();
    v["nativeEvidence"]["receipt"]["results"][0]["passed"] = false.into();
    v["nativeEvidence"]["receipt"]["executionRecords"][0]["passed"] = false.into();
    v["nativeEvidence"]["receipt"]["executionRecords"][0]["exitCode"] = 7.into();
    v["nativeEvidence"]["receipt"]["costObservation"]["passed"] = false.into();
    v["nativeEvidence"]["receiptDigest"] =
        cockpit_protocol::digest_json(&v["nativeEvidence"]["receipt"])
            .unwrap()
            .to_string()
            .into();
    std::fs::write(&input, serde_json::to_vec(&v).unwrap()).unwrap();
    let p = super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
        .expect("RED: failed results remain analyzable");
    assert!(!p.may_replace_execution);
    assert_eq!(p.claims["passed"], false);
    assert!(
        p.gaps
            .iter()
            .any(|g| g == "native_verification_not_complete_success")
    );
}
#[test]
fn ci_admission_c16_portable_duplicate_deep_keys_and_unknown_authority_fields_refuse() {
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("duplicate.json");
    let v = c16_portable_input_fixture();
    let raw = serde_json::to_string(&v).unwrap();
    // A real nested duplicate is parsed before any Value can swallow it.
    let marker = r#""nativeInput":"#;
    let start = raw.find(marker).unwrap() + marker.len();
    let mut raw = raw;
    let close = raw[start..].find('}').unwrap() + start;
    raw.insert_str(close, r#","duplicate":1,"duplicate":2"#);
    std::fs::write(&input, raw).unwrap();
    assert!(
        super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
            .is_err()
    );
    let mut authority = v;
    authority["sourceVerified"] = true.into();
    std::fs::write(&input, serde_json::to_vec(&authority).unwrap()).unwrap();
    assert!(
        super::plan_portable_evidence(root.path(), None, &input, &c16_portable_runtime(), None)
            .is_err()
    );
}
