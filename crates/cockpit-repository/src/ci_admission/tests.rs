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
