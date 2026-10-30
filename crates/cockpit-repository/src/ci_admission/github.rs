//! Fixed-origin bounded metadata collection. No artifact download or trust grant.

use cockpit_core::Digest;
use cockpit_protocol::ci_admission::{
    GithubCollectionReport, GithubEvidenceSelector, GithubReadObservation,
};
use std::io::Read;
use std::time::{Duration, Instant};

const ORIGIN: &str = "https://api.github.com";
const PREFIX: &str = "/repos/xinglun/ai-cockpit";
const BODY_LIMIT: usize = 8 * 1024 * 1024;
const TOTAL_LIMIT: usize = 32 * 1024 * 1024;
const PAGE_LIMIT: u32 = 32;
const COLLECTION_SECONDS: u64 = 120;

pub(super) struct Collected {
    pub collection: GithubCollectionReport,
    pub metadata: Option<serde_json::Value>,
    pub rejected: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Failure {
    NotImplemented,
    Transport,
    Deadline,
    BodyLimit,
    TotalLimit,
    Pagination,
    Metadata,
    Route,
    HttpStatus(u16),
}

impl Failure {
    fn code(&self) -> String {
        match self {
            Self::NotImplemented => "provider_collection_not_implemented".into(),
            Self::Transport => "provider_transport_unavailable".into(),
            Self::Deadline => "provider_collection_deadline".into(),
            Self::BodyLimit => "provider_response_size_limit".into(),
            Self::TotalLimit => "provider_aggregate_size_limit".into(),
            Self::Pagination => "provider_pagination_unverified".into(),
            Self::Metadata => "provider_metadata_invalid".into(),
            Self::Route => "provider_request_route_rejected".into(),
            Self::HttpStatus(status) => format!("provider_http_status_{status}"),
        }
    }
}

pub(super) struct Reply {
    pub status: u16,
    pub link: Option<String>,
    pub body: Vec<u8>,
}

// Private test seam. Production callers cannot provide a transport or construct
// a public provider-authenticated capability from caller JSON.
pub(super) trait MetadataReader {
    fn read(&mut self, path: &str, raw: bool, deadline: Instant) -> Result<Reply, Failure>;
}

struct HttpsReader;

impl MetadataReader for HttpsReader {
    fn read(&mut self, path: &str, raw: bool, deadline: Instant) -> Result<Reply, Failure> {
        validate_path(path)?;
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Failure::Deadline)?;
        let timeout = remaining.min(Duration::from_secs(20));
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            .timeout_resolve(Some(timeout))
            .timeout_connect(Some(timeout))
            .max_redirects(0)
            .proxy(None)
            .http_status_as_error(false)
            .build();
        let agent = ureq::Agent::new_with_config(config);
        let mut response = agent
            .get(&format!("{ORIGIN}{path}"))
            .header(
                "Accept",
                if raw {
                    "application/vnd.github.raw+json"
                } else {
                    "application/vnd.github+json"
                },
            )
            .header("User-Agent", "ai-cockpit-github-verification-plan")
            .call()
            .map_err(|_| Failure::Transport)?;
        let status = response.status().as_u16();
        if status != 200 {
            return Err(Failure::HttpStatus(status));
        }
        let link = response
            .headers()
            .get("link")
            .map(|v| {
                v.to_str()
                    .map(str::to_owned)
                    .map_err(|_| Failure::Pagination)
            })
            .transpose()?;
        if link.as_ref().is_some_and(|s| s.len() > 8192) {
            return Err(Failure::Pagination);
        }
        let mut body = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take((BODY_LIMIT + 1) as u64)
            .read_to_end(&mut body)
            .map_err(|_| Failure::Transport)?;
        if body.len() > BODY_LIMIT {
            return Err(Failure::BodyLimit);
        }
        if Instant::now() >= deadline {
            return Err(Failure::Deadline);
        }
        Ok(Reply { status, link, body })
    }
}

fn validate_path(path: &str) -> Result<(), Failure> {
    if !path.starts_with(&format!("{PREFIX}/"))
        || path.contains("..")
        || path
            .bytes()
            .any(|v| v.is_ascii_control() || v == b'\\' || v == b' ' || v == b'#')
        || path.contains("://")
    {
        return Err(Failure::Route);
    }
    Ok(())
}

fn next_page(link: Option<&str>, base: &str, page: u32) -> Result<Option<u32>, Failure> {
    let Some(link) = link else {
        return Ok(None);
    };
    if link.len() > 8192 {
        return Err(Failure::Pagination);
    }
    let mut next = None;
    for item in link.split(',') {
        let (url, relation) = item
            .trim()
            .split_once("; rel=\"")
            .ok_or(Failure::Pagination)?;
        let url = url
            .strip_prefix('<')
            .and_then(|s| s.strip_suffix('>'))
            .ok_or(Failure::Pagination)?;
        let relation = relation.strip_suffix('"').ok_or(Failure::Pagination)?;
        if !matches!(relation, "next" | "prev" | "first" | "last") {
            return Err(Failure::Pagination);
        }
        let uri: ureq::http::Uri = url.parse().map_err(|_| Failure::Pagination)?;
        if uri.scheme_str() != Some("https")
            || uri.authority().map(|x| x.as_str()) != Some("api.github.com")
            || uri.path() != base
        {
            return Err(Failure::Pagination);
        }
        let mut values = std::collections::BTreeMap::new();
        for pair in uri.query().ok_or(Failure::Pagination)?.split('&') {
            let (key, value) = pair.split_once('=').ok_or(Failure::Pagination)?;
            if !matches!(key, "per_page" | "page") || values.insert(key, value).is_some() {
                return Err(Failure::Pagination);
            }
        }
        if values.get("per_page") != Some(&"100") || values.len() != 2 {
            return Err(Failure::Pagination);
        }
        let target: u32 = values["page"].parse().map_err(|_| Failure::Pagination)?;
        if target == 0 || target > PAGE_LIMIT {
            return Err(Failure::Pagination);
        }
        if relation == "next" {
            if next.is_some() || target != page + 1 {
                return Err(Failure::Pagination);
            }
            next = Some(target);
        }
    }
    Ok(next)
}

struct Session<'a, R> {
    reader: &'a mut R,
    deadline: Instant,
    bytes: usize,
    observations: Vec<GithubReadObservation>,
}

impl<R: MetadataReader> Session<'_, R> {
    fn read(&mut self, path: &str, raw: bool) -> Result<Reply, Failure> {
        validate_path(path)?;
        if Instant::now() >= self.deadline {
            return Err(Failure::Deadline);
        }
        let reply = self.reader.read(path, raw, self.deadline)?;
        if Instant::now() >= self.deadline {
            return Err(Failure::Deadline);
        }
        if reply.status != 200 {
            return Err(Failure::HttpStatus(reply.status));
        }
        if reply.body.len() > BODY_LIMIT {
            return Err(Failure::BodyLimit);
        }
        self.bytes = self
            .bytes
            .checked_add(reply.body.len())
            .ok_or(Failure::TotalLimit)?;
        if self.bytes > TOTAL_LIMIT {
            return Err(Failure::TotalLimit);
        }
        self.observations.push(GithubReadObservation {
            path: path.into(),
            status: reply.status,
            body_digest: Digest::sha256_bytes(&reply.body),
            body_bytes: reply.body.len() as u64,
            link_digest: reply
                .link
                .as_ref()
                .map(|s| Digest::sha256_bytes(s.as_bytes())),
        });
        Ok(reply)
    }

    fn pages(&mut self, base: &str) -> Result<Vec<Vec<u8>>, Failure> {
        let mut pages = Vec::new();
        let mut page = 1;
        loop {
            let reply = self.read(&format!("{base}?per_page=100&page={page}"), false)?;
            let next = next_page(reply.link.as_deref(), base, page)?;
            pages.push(reply.body);
            let Some(value) = next else {
                return Ok(pages);
            };
            if pages.len() >= PAGE_LIMIT as usize {
                return Err(Failure::Pagination);
            }
            page = value;
        }
    }
}

pub(super) fn collect(selector: &GithubEvidenceSelector) -> Collected {
    collect_with(
        selector,
        &mut HttpsReader,
        Duration::from_secs(COLLECTION_SECONDS),
    )
}

pub(super) fn collect_with<R: MetadataReader>(
    selector: &GithubEvidenceSelector,
    reader: &mut R,
    budget: Duration,
) -> Collected {
    let started = Instant::now();
    let mut session = Session {
        reader,
        deadline: started + budget,
        bytes: 0,
        observations: Vec::new(),
    };
    let result = collect_snapshot(selector, &mut session);
    let (metadata, failure, rejected) = match result {
        Ok((value, rejected)) => (Some(value), None, rejected),
        Err(e) => (
            None,
            Some(e.code()),
            matches!(e, Failure::Metadata | Failure::Pagination | Failure::Route),
        ),
    };
    Collected {
        collection: GithubCollectionReport {
            origin: ORIGIN.into(),
            assurance: "https_origin_metadata_only".into(),
            elapsed_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            maximum_seconds: budget.as_secs(),
            requests: session.observations,
            complete: failure.is_none(),
            failure,
        },
        metadata,
        rejected,
    }
}

fn collect_snapshot<R: MetadataReader>(
    _selector: &GithubEvidenceSelector,
    _session: &mut Session<'_, R>,
) -> Result<(serde_json::Value, bool), Failure> {
    // Conservative assertion-RED checkpoint: no metadata is promoted or admitted.
    Err(Failure::NotImplemented)
}

mod snapshot {
    //! Portable, UNAPPLIED WI-1114 main-provider gap-plan proposal.
    //! Product integration path is pending the core owner's exact scope choice.
    //! Pure supplied-snapshot inspection: no network, files, process creation,
    //! metadata command, reusable receipt, acceptance capability or write API.

    use cockpit_core::Digest;
    use serde::{Deserialize, Serialize, de::DeserializeOwned};
    use std::collections::BTreeSet;
    use std::str::FromStr;

    pub const MAX_MAIN_SNAPSHOT_BODY_BYTES: usize = 8 * 1024 * 1024;
    const MAX_TOTAL_SNAPSHOT_BYTES: usize = 32 * 1024 * 1024;
    const MAX_COLLECTION_PAGES: usize = 32;
    const MAX_PAGE_OBJECTS: usize = 100;
    const REPOSITORY_ID: u64 = 1341189248;
    const REPOSITORY_NAME: &str = "xinglun/ai-cockpit";
    const WORKFLOW_ID: u64 = 338986437;
    const WORKFLOW_PATH: &str = ".github/workflows/ci.yml";

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub struct MainFixtureSelector {
        pub run_id: u64,
        pub attempt: u32,
        pub job_id: u64,
        pub artifact_id: u64,
    }

    /// Supplied expectations are NOT an installed trusted catalog or a grant.
    /// Matching them describes bytes only; it cannot authenticate acquisition.
    pub struct MainFixtureExpectations {
        pub workflow_commit: String,
        pub workflow_bytes_digest: Digest,
    }

    /// All input bytes remain supplied snapshots, even if they originally came
    /// from a real provider read. Production authenticated transport is separate.
    pub struct MainSnapshotFixture<'a> {
        pub run_json: &'a [u8],
        pub jobs_pages: &'a [&'a [u8]],
        pub artifact_pages: &'a [&'a [u8]],
        pub supplied_workflow_read_commit: Option<&'a str>,
        pub supplied_workflow_bytes: Option<&'a [u8]>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum AdmissionState {
        NotAttempted,
    }
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum NodeEvaluation {
        NotPerformed,
    }
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SnapshotProvenance {
        SuppliedSnapshotNotAuthenticated,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MainPlanMismatch {
        FixtureNotEvaluated,
        RunId,
        Attempt,
        Repository,
        HeadRepository,
        EventNotMainPush,
        Branch,
        WorkflowId,
        WorkflowPath,
        RunNotSuccessful,
        JobRun,
        JobAttempt,
        JobRole,
        JobNotSuccessful,
        ArtifactRun,
        ArtifactRepository,
        ArtifactHeadRepository,
        ArtifactSource,
        ArtifactName,
        ArtifactExpired,
        ArtifactDigestMalformed,
        WorkflowCommit,
        WorkflowReadCommitClaim,
        WorkflowBytes,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MainPlanGap {
        SnapshotAcquisitionUnauthenticated,
        PaginationTransportUnverified,
        JobsSuppliedCountsIncomplete,
        ArtifactsSuppliedCountsIncomplete,
        JobNotObserved,
        ArtifactNotObserved,
        ArtifactDigestUnavailable,
        ArtifactJobAttemptBindingUnavailable,
        WorkflowBytesUnavailable,
        WorkflowExecutionClosureUnverified,
        ProducerRuntimeUnverified,
        RawArtifactBytesNotInspected,
        NativeReceiptNotInspected,
        NativeEvidenceUploadSkipped,
        UploadObservationUnavailable,
        CurrentConsumerBindingUnavailable,
        AdmissionNotImplemented,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WorkflowFixtureState {
        NotSupplied,
        BytesMatchSuppliedPin,
        Mismatch,
    }
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum UploadFixtureState {
        Unavailable,
        ReportedSkipped,
        ReportedSuccess,
        ReportedOther,
    }
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub struct WorkflowFixtureObservation {
        pub state: WorkflowFixtureState,
        pub main_commit_candidate: String,
        pub supplied_read_commit_claim: Option<String>,
        pub observed_bytes_digest: Option<Digest>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub struct SnapshotBodyReference {
        pub role: String,
        pub page_index: usize,
        pub body_digest: Digest,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub struct MainFixtureGapPlan {
        pub schema_version: u32,
        pub selector: MainFixtureSelector,
        pub provenance: SnapshotProvenance,
        /// Consistency of supplied API fields only; NEVER provider authentication.
        pub metadata_consistent: bool,
        pub observed_job_id: Option<u64>,
        pub observed_artifact_id: Option<u64>,
        pub observed_artifact_digest: Option<String>,
        /// Count equality in supplied pages, NOT authenticated pagination proof.
        pub jobs_supplied_counts_cover_declared_total: bool,
        pub artifacts_supplied_counts_cover_declared_total: bool,
        /// Advisory role observation only. Step names cannot bootstrap trust.
        pub upload_observation: UploadFixtureState,
        pub upload_observed_step_number: Option<u64>,
        pub workflow: WorkflowFixtureObservation,
        pub bodies: Vec<SnapshotBodyReference>,
        pub mismatches: Vec<MainPlanMismatch>,
        pub gaps: Vec<MainPlanGap>,
        pub admission: AdmissionState,
        pub node_evaluation: NodeEvaluation,
        pub processes_spawned: usize,
        pub writes_performed: usize,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum MainFixtureError {
        InvalidSelector,
        InvalidExpectations,
        SizeLimit,
        MalformedSnapshot,
        InconsistentPageTotals,
        CountExceedsDeclaredTotal,
        DuplicateIdentity,
    }

    // These are intentionally non-authoritative projections of provider JSON.
    // GitHub adds fields; unknown fields are ignored, not reinterpreted as proofs.
    #[derive(Deserialize)]
    struct Repo {
        id: u64,
        full_name: String,
    }
    #[derive(Deserialize)]
    struct Run {
        id: u64,
        run_attempt: u32,
        event: String,
        path: String,
        workflow_id: u64,
        head_sha: String,
        head_branch: String,
        status: String,
        conclusion: Option<String>,
        repository: Repo,
        head_repository: Repo,
    }
    #[derive(Deserialize)]
    struct Step {
        number: u64,
        name: String,
        status: String,
        conclusion: Option<String>,
    }
    #[derive(Deserialize)]
    struct Job {
        id: u64,
        run_id: u64,
        run_attempt: u32,
        name: String,
        status: String,
        conclusion: Option<String>,
        steps: Vec<Step>,
    }
    #[derive(Deserialize)]
    struct ArtifactRun {
        id: u64,
        repository_id: u64,
        head_repository_id: u64,
        head_sha: String,
    }
    #[derive(Deserialize)]
    struct Artifact {
        id: u64,
        name: String,
        expired: bool,
        digest: Option<String>,
        workflow_run: ArtifactRun,
    }

    fn valid_oid(value: &str) -> bool {
        matches!(value.len(), 40 | 64)
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }

    fn parse_pages<T: DeserializeOwned>(
        pages: &[&[u8]],
        key: &str,
        identity: fn(&T) -> u64,
    ) -> Result<(Vec<T>, bool), MainFixtureError> {
        let mut rows = Vec::new();
        let mut expected_total = None;
        let mut ids = BTreeSet::new();
        for page in pages {
            let value: serde_json::Value =
                serde_json::from_slice(page).map_err(|_| MainFixtureError::MalformedSnapshot)?;
            let total = value
                .get("total_count")
                .and_then(serde_json::Value::as_u64)
                .ok_or(MainFixtureError::MalformedSnapshot)?;
            if total > (MAX_COLLECTION_PAGES * MAX_PAGE_OBJECTS) as u64 {
                return Err(MainFixtureError::SizeLimit);
            }
            if expected_total.is_some_and(|prior| prior != total) {
                return Err(MainFixtureError::InconsistentPageTotals);
            }
            expected_total = Some(total);
            let items = value
                .get(key)
                .and_then(serde_json::Value::as_array)
                .ok_or(MainFixtureError::MalformedSnapshot)?;
            if items.len() > MAX_PAGE_OBJECTS {
                return Err(MainFixtureError::SizeLimit);
            }
            for item in items {
                let row: T = serde_json::from_value(item.clone())
                    .map_err(|_| MainFixtureError::MalformedSnapshot)?;
                if identity(&row) == 0 || !ids.insert(identity(&row)) {
                    return Err(MainFixtureError::DuplicateIdentity);
                }
                rows.push(row);
            }
        }
        if expected_total.is_some_and(|total| rows.len() as u64 > total) {
            return Err(MainFixtureError::CountExceedsDeclaredTotal);
        }
        // This is only declared-total/count equality, not Link/request provenance.
        let complete = expected_total == Some(rows.len() as u64);
        Ok((rows, complete))
    }

    fn check_bounds(
        selector: &MainFixtureSelector,
        expected: &MainFixtureExpectations,
        input: &MainSnapshotFixture<'_>,
    ) -> Result<(), MainFixtureError> {
        if selector.run_id == 0
            || selector.attempt == 0
            || selector.job_id == 0
            || selector.artifact_id == 0
        {
            return Err(MainFixtureError::InvalidSelector);
        }
        if !valid_oid(&expected.workflow_commit)
            || Digest::from_str(expected.workflow_bytes_digest.as_str()).is_err()
        {
            return Err(MainFixtureError::InvalidExpectations);
        }
        if input
            .supplied_workflow_read_commit
            .is_some_and(|claim| !valid_oid(claim))
        {
            return Err(MainFixtureError::MalformedSnapshot);
        }
        if input.jobs_pages.len() > MAX_COLLECTION_PAGES
            || input.artifact_pages.len() > MAX_COLLECTION_PAGES
        {
            return Err(MainFixtureError::SizeLimit);
        }
        let mut total = 0_usize;
        for body in std::iter::once(input.run_json)
            .chain(input.jobs_pages.iter().copied())
            .chain(input.artifact_pages.iter().copied())
            .chain(input.supplied_workflow_bytes)
        {
            if body.len() > MAX_MAIN_SNAPSHOT_BODY_BYTES {
                return Err(MainFixtureError::SizeLimit);
            }
            total = total
                .checked_add(body.len())
                .ok_or(MainFixtureError::SizeLimit)?;
            if total > MAX_TOTAL_SNAPSHOT_BYTES {
                return Err(MainFixtureError::SizeLimit);
            }
        }
        Ok(())
    }

    fn initial_plan(
        selector: &MainFixtureSelector,
        run: &Run,
        input: &MainSnapshotFixture<'_>,
    ) -> MainFixtureGapPlan {
        let bodies = std::iter::once(("run", 0, input.run_json))
            .chain(
                input
                    .jobs_pages
                    .iter()
                    .enumerate()
                    .map(|(i, b)| ("jobs", i, *b)),
            )
            .chain(
                input
                    .artifact_pages
                    .iter()
                    .enumerate()
                    .map(|(i, b)| ("artifacts", i, *b)),
            )
            .map(|(role, page_index, body)| SnapshotBodyReference {
                role: role.into(),
                page_index,
                body_digest: Digest::sha256_bytes(body),
            })
            .collect();
        MainFixtureGapPlan {
            schema_version: 1,
            selector: selector.clone(),
            provenance: SnapshotProvenance::SuppliedSnapshotNotAuthenticated,
            metadata_consistent: false,
            observed_job_id: None,
            observed_artifact_id: None,
            observed_artifact_digest: None,
            jobs_supplied_counts_cover_declared_total: false,
            artifacts_supplied_counts_cover_declared_total: false,
            upload_observation: UploadFixtureState::Unavailable,
            upload_observed_step_number: None,
            workflow: WorkflowFixtureObservation {
                state: WorkflowFixtureState::NotSupplied,
                main_commit_candidate: run.head_sha.clone(),
                supplied_read_commit_claim: input.supplied_workflow_read_commit.map(str::to_owned),
                observed_bytes_digest: None,
            },
            bodies,
            mismatches: Vec::new(),
            gaps: vec![
                MainPlanGap::SnapshotAcquisitionUnauthenticated,
                MainPlanGap::PaginationTransportUnverified,
                MainPlanGap::ArtifactJobAttemptBindingUnavailable,
                MainPlanGap::WorkflowExecutionClosureUnverified,
                MainPlanGap::ProducerRuntimeUnverified,
                MainPlanGap::RawArtifactBytesNotInspected,
                MainPlanGap::NativeReceiptNotInspected,
                MainPlanGap::CurrentConsumerBindingUnavailable,
                MainPlanGap::AdmissionNotImplemented,
            ],
            admission: AdmissionState::NotAttempted,
            node_evaluation: NodeEvaluation::NotPerformed,
            processes_spawned: 0,
            writes_performed: 0,
        }
    }

    /// Inspect bounded, caller-supplied main snapshots. Even a completely
    /// consistent report stays PlanOnly. Do not feed it into the kernel as trusted
    /// producer facts, a cache writer, an acceptance operation or a skip decision.
    pub fn plan_main_snapshot_fixture(
        selector: &MainFixtureSelector,
        expected: &MainFixtureExpectations,
        input: MainSnapshotFixture<'_>,
    ) -> Result<MainFixtureGapPlan, MainFixtureError> {
        check_bounds(selector, expected, &input)?;
        let run: Run = serde_json::from_slice(input.run_json)
            .map_err(|_| MainFixtureError::MalformedSnapshot)?;
        if !valid_oid(&run.head_sha) {
            return Err(MainFixtureError::MalformedSnapshot);
        }
        let (jobs, jobs_complete) = parse_pages::<Job>(input.jobs_pages, "jobs", |v| v.id)?;
        let (artifacts, artifacts_complete) =
            parse_pages::<Artifact>(input.artifact_pages, "artifacts", |v| v.id)?;
        let mut p = initial_plan(selector, &run, &input);
        macro_rules! require {
            ($condition:expr,$reason:ident) => {
                if !$condition {
                    p.mismatches.push(MainPlanMismatch::$reason);
                }
            };
        }
        require!(run.id == selector.run_id, RunId);
        require!(run.run_attempt == selector.attempt, Attempt);
        require!(
            run.repository.id == REPOSITORY_ID && run.repository.full_name == REPOSITORY_NAME,
            Repository
        );
        require!(
            run.head_repository.id == REPOSITORY_ID
                && run.head_repository.full_name == REPOSITORY_NAME,
            HeadRepository
        );
        require!(run.event == "push", EventNotMainPush);
        require!(run.head_branch == "main", Branch);
        require!(run.workflow_id == WORKFLOW_ID, WorkflowId);
        require!(run.path == WORKFLOW_PATH, WorkflowPath);
        require!(
            run.status == "completed" && run.conclusion.as_deref() == Some("success"),
            RunNotSuccessful
        );
        p.jobs_supplied_counts_cover_declared_total = jobs_complete;
        p.artifacts_supplied_counts_cover_declared_total = artifacts_complete;
        if !jobs_complete {
            p.gaps.push(MainPlanGap::JobsSuppliedCountsIncomplete);
        }
        if !artifacts_complete {
            p.gaps.push(MainPlanGap::ArtifactsSuppliedCountsIncomplete);
        }
        if let Some(job) = jobs.iter().find(|v| v.id == selector.job_id) {
            p.observed_job_id = Some(job.id);
            require!(job.run_id == selector.run_id, JobRun);
            require!(job.run_attempt == selector.attempt, JobAttempt);
            require!(job.name == "quality", JobRole);
            require!(
                job.status == "completed" && job.conclusion.as_deref() == Some("success"),
                JobNotSuccessful
            );
            let upload_roles = job
                .steps
                .iter()
                .filter(|s| s.name == "Upload hosted Work Item verification evidence")
                .collect::<Vec<_>>();
            if let [upload] = upload_roles.as_slice() {
                p.upload_observed_step_number = Some(upload.number);
                p.upload_observation = match (upload.status.as_str(), upload.conclusion.as_deref())
                {
                    ("completed", Some("skipped")) => UploadFixtureState::ReportedSkipped,
                    ("completed", Some("success")) => UploadFixtureState::ReportedSuccess,
                    _ => UploadFixtureState::ReportedOther,
                };
                if p.upload_observation == UploadFixtureState::ReportedSkipped {
                    p.gaps.push(MainPlanGap::NativeEvidenceUploadSkipped);
                }
            } else {
                p.gaps.push(MainPlanGap::UploadObservationUnavailable);
            }
        } else {
            p.gaps.push(MainPlanGap::JobNotObserved);
            p.gaps.push(MainPlanGap::UploadObservationUnavailable);
        }
        if let Some(artifact) = artifacts.iter().find(|v| v.id == selector.artifact_id) {
            p.observed_artifact_id = Some(artifact.id);
            p.observed_artifact_digest = artifact.digest.clone();
            require!(artifact.workflow_run.id == selector.run_id, ArtifactRun);
            require!(
                artifact.workflow_run.repository_id == REPOSITORY_ID,
                ArtifactRepository
            );
            require!(
                artifact.workflow_run.head_repository_id == REPOSITORY_ID,
                ArtifactHeadRepository
            );
            require!(
                artifact.workflow_run.head_sha == run.head_sha,
                ArtifactSource
            );
            require!(artifact.name == "repository-gate-receipts", ArtifactName);
            require!(!artifact.expired, ArtifactExpired);
            if let Some(digest) = &artifact.digest {
                require!(Digest::from_str(digest).is_ok(), ArtifactDigestMalformed);
            } else {
                p.gaps.push(MainPlanGap::ArtifactDigestUnavailable);
            }
        } else {
            p.gaps.push(MainPlanGap::ArtifactNotObserved);
        }
        require!(run.head_sha == expected.workflow_commit, WorkflowCommit);
        if let (Some(read_commit), Some(bytes)) = (
            input.supplied_workflow_read_commit,
            input.supplied_workflow_bytes,
        ) {
            let digest = Digest::sha256_bytes(bytes);
            p.workflow.observed_bytes_digest = Some(digest.clone());
            require!(read_commit == run.head_sha, WorkflowReadCommitClaim);
            require!(digest == expected.workflow_bytes_digest, WorkflowBytes);
            p.workflow.state = if read_commit == run.head_sha
                && run.head_sha == expected.workflow_commit
                && digest == expected.workflow_bytes_digest
            {
                WorkflowFixtureState::BytesMatchSuppliedPin
            } else {
                WorkflowFixtureState::Mismatch
            };
        } else {
            p.gaps.push(MainPlanGap::WorkflowBytesUnavailable);
        }
        p.metadata_consistent = p.mismatches.is_empty()
            && jobs_complete
            && artifacts_complete
            && p.observed_job_id.is_some()
            && p.observed_artifact_id.is_some();
        // Conservative assertion-RED scaffold. Keep bounded parse/field checks,
        // but withhold all positive observation projections until real assertion
        // failures have been observed by core. This does not create acceptance.
        p.metadata_consistent = false;
        p.observed_job_id = None;
        p.observed_artifact_id = None;
        p.observed_artifact_digest = None;
        p.jobs_supplied_counts_cover_declared_total = false;
        p.artifacts_supplied_counts_cover_declared_total = false;
        p.workflow.state = WorkflowFixtureState::NotSupplied;
        p.workflow.observed_bytes_digest = None;
        p.upload_observation = UploadFixtureState::Unavailable;
        p.upload_observed_step_number = None;
        p.mismatches.push(MainPlanMismatch::FixtureNotEvaluated);
        Ok(p)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::{Value, json};

        fn sample() -> (Value, Value, Value) {
            (
                json!({"id":40814284609_u64,"run_attempt":1,"event":"push",
                "path":".github/workflows/ci.yml","workflow_id":338986437,
                "head_sha":"efc9f309f1500e433240a76fe5dc76db64abb53b",
                "head_branch":"main","status":"completed","conclusion":"success",
                "repository":{"id":1341189248,"full_name":"xinglun/ai-cockpit"},
                "head_repository":{"id":1341189248,"full_name":"xinglun/ai-cockpit"}}),
                json!({"total_count":1,"jobs":[{"id":123253162675_u64,
                "run_id":40814284609_u64,"run_attempt":1,"name":"quality",
                "status":"completed","conclusion":"success","steps":[
                    {"number":18,"name":"Upload hosted Work Item verification evidence",
                     "status":"completed","conclusion":"skipped"}]}]}),
                json!({"total_count":1,"artifacts":[{"id":12873430691_u64,
                "name":"repository-gate-receipts","expired":false,
                "digest":"sha256:598fa26542da1a1a88879012701df0829e29ea56a8212f0e41e538643ec3db0f",
                "workflow_run":{"id":40814284609_u64,"repository_id":1341189248,
                "head_repository_id":1341189248,"head_sha":"efc9f309f1500e433240a76fe5dc76db64abb53b"}}]}),
            )
        }

        fn selector() -> MainFixtureSelector {
            MainFixtureSelector {
                run_id: 40814284609,
                attempt: 1,
                job_id: 123253162675,
                artifact_id: 12873430691,
            }
        }

        fn inspect(run: &Value, jobs: &Value, artifacts: &Value) -> MainFixtureGapPlan {
            let run = serde_json::to_vec(run).unwrap();
            let jobs = serde_json::to_vec(jobs).unwrap();
            let artifacts = serde_json::to_vec(artifacts).unwrap();
            let expectations = MainFixtureExpectations {
                workflow_commit: "efc9f309f1500e433240a76fe5dc76db64abb53b".into(),
                workflow_bytes_digest: Digest::sha256_bytes(b"workflow fixture bytes"),
            };
            plan_main_snapshot_fixture(
                &selector(),
                &expectations,
                MainSnapshotFixture {
                    run_json: &run,
                    jobs_pages: &[&jobs],
                    artifact_pages: &[&artifacts],
                    supplied_workflow_read_commit: Some(&expectations.workflow_commit),
                    supplied_workflow_bytes: Some(b"workflow fixture bytes"),
                },
            )
            .unwrap()
        }

        #[test]
        fn consistent_main_snapshot_yields_plan_only_and_real_missing_provenance_gaps() {
            let (r, j, a) = sample();
            let p = inspect(&r, &j, &a);
            assert!(p.metadata_consistent);
            assert_eq!(
                p.workflow.state,
                WorkflowFixtureState::BytesMatchSuppliedPin
            );
            assert_eq!(p.observed_job_id, Some(123253162675));
            assert_eq!(p.observed_artifact_id, Some(12873430691));
            assert_eq!(p.admission, AdmissionState::NotAttempted);
            assert_eq!(p.node_evaluation, NodeEvaluation::NotPerformed);
            assert!(
                p.gaps
                    .contains(&MainPlanGap::SnapshotAcquisitionUnauthenticated)
            );
            assert!(
                p.gaps
                    .contains(&MainPlanGap::ArtifactJobAttemptBindingUnavailable)
            );
            assert!(p.gaps.contains(&MainPlanGap::ProducerRuntimeUnverified));
            assert!(p.gaps.contains(&MainPlanGap::RawArtifactBytesNotInspected));
            assert!(p.gaps.contains(&MainPlanGap::NativeEvidenceUploadSkipped));
            assert_eq!(p.processes_spawned, 0);
            assert_eq!(p.writes_performed, 0);
            let value = serde_json::to_value(&p).unwrap();
            assert_eq!(value["admission"], json!("not_attempted"));
            assert!(value.get("receipt").is_none());
            assert!(value.get("reusableNodes").is_none());
        }

        #[test]
        fn self_reported_provider_origin_job_attempt_and_green_never_remove_gaps() {
            let (mut r, j, mut a) = sample();
            let before = inspect(&r, &j, &a);
            r["providerAuthenticated"] = json!(true);
            r["origin"] = json!("github");
            a["artifacts"][0]["jobId"] = json!(123253162675_u64);
            a["artifacts"][0]["runAttempt"] = json!(1);
            a["artifacts"][0]["producerRuntimeVerified"] = json!(true);
            let p = inspect(&r, &j, &a);
            assert_eq!(p.gaps, before.gaps);
            assert_eq!(p.admission, AdmissionState::NotAttempted);
        }

        #[test]
        fn wrong_attempt_and_artifact_run_are_mismatches_not_authentication() {
            let (r, mut j, mut a) = sample();
            j["jobs"][0]["run_attempt"] = json!(2);
            a["artifacts"][0]["workflow_run"]["id"] = json!(99);
            let p = inspect(&r, &j, &a);
            assert!(!p.metadata_consistent);
            assert!(p.mismatches.contains(&MainPlanMismatch::JobAttempt));
            assert!(p.mismatches.contains(&MainPlanMismatch::ArtifactRun));
            assert!(
                p.gaps
                    .contains(&MainPlanGap::ArtifactJobAttemptBindingUnavailable)
            );
        }

        #[test]
        fn fork_wrong_workflow_and_pr_route_are_not_supported_main_metadata() {
            let (r, j, a) = sample();
            for changed in [
                {
                    let mut v = r.clone();
                    v["event"] = json!("pull_request");
                    v
                },
                {
                    let mut v = r.clone();
                    v["head_repository"]["id"] = json!(7);
                    v
                },
                {
                    let mut v = r.clone();
                    v["workflow_id"] = json!(8);
                    v
                },
                {
                    let mut v = r.clone();
                    v["head_branch"] = json!("other");
                    v
                },
            ] {
                let p = inspect(&changed, &j, &a);
                assert!(!p.metadata_consistent);
                assert_eq!(p.admission, AdmissionState::NotAttempted);
            }
        }

        #[test]
        fn incomplete_pages_are_unknown_and_duplicate_ids_are_malformed() {
            let (r, mut j, a) = sample();
            j["total_count"] = json!(2);
            let p = inspect(&r, &j, &a);
            assert!(!p.metadata_consistent);
            assert!(p.gaps.contains(&MainPlanGap::JobsSuppliedCountsIncomplete));
            j["total_count"] = json!(2);
            let first = j["jobs"][0].clone();
            j["jobs"].as_array_mut().unwrap().push(first);
            let rb = serde_json::to_vec(&r).unwrap();
            let jb = serde_json::to_vec(&j).unwrap();
            let ab = serde_json::to_vec(&a).unwrap();
            let expected = MainFixtureExpectations {
                workflow_commit: "a".repeat(40),
                workflow_bytes_digest: Digest::sha256_bytes(b"x"),
            };
            assert!(matches!(
                plan_main_snapshot_fixture(
                    &selector(),
                    &expected,
                    MainSnapshotFixture {
                        run_json: &rb,
                        jobs_pages: &[&jb],
                        artifact_pages: &[&ab],
                        supplied_workflow_read_commit: None,
                        supplied_workflow_bytes: None
                    }
                ),
                Err(MainFixtureError::DuplicateIdentity)
            ));
        }

        #[test]
        fn absent_artifact_digest_and_workflow_bytes_stay_unknown() {
            let (r, j, mut a) = sample();
            a["artifacts"][0].as_object_mut().unwrap().remove("digest");
            let rb = serde_json::to_vec(&r).unwrap();
            let jb = serde_json::to_vec(&j).unwrap();
            let ab = serde_json::to_vec(&a).unwrap();
            let expected = MainFixtureExpectations {
                workflow_commit: r["head_sha"].as_str().unwrap().into(),
                workflow_bytes_digest: Digest::sha256_bytes(b"x"),
            };
            let p = plan_main_snapshot_fixture(
                &selector(),
                &expected,
                MainSnapshotFixture {
                    run_json: &rb,
                    jobs_pages: &[&jb],
                    artifact_pages: &[&ab],
                    supplied_workflow_read_commit: None,
                    supplied_workflow_bytes: None,
                },
            )
            .unwrap();
            assert_eq!(p.workflow.state, WorkflowFixtureState::NotSupplied);
            assert!(p.gaps.contains(&MainPlanGap::ArtifactDigestUnavailable));
            assert!(p.gaps.contains(&MainPlanGap::WorkflowBytesUnavailable));
        }

        #[test]
        fn upload_role_can_move_but_missing_or_duplicate_role_stays_unavailable() {
            let (r, mut j, a) = sample();
            j["jobs"][0]["steps"][0]["number"] = json!(19);
            let p = inspect(&r, &j, &a);
            assert_eq!(p.upload_observation, UploadFixtureState::ReportedSkipped);
            assert_eq!(p.upload_observed_step_number, Some(19));
            let role = j["jobs"][0]["steps"][0].clone();
            j["jobs"][0]["steps"].as_array_mut().unwrap().push(role);
            let p = inspect(&r, &j, &a);
            assert_eq!(p.upload_observation, UploadFixtureState::Unavailable);
            assert!(p.gaps.contains(&MainPlanGap::UploadObservationUnavailable));
            j["jobs"][0]["steps"] = json!([]);
            let p = inspect(&r, &j, &a);
            assert_eq!(p.upload_observation, UploadFixtureState::Unavailable);
            assert!(p.gaps.contains(&MainPlanGap::UploadObservationUnavailable));
        }

        #[test]
        fn supplied_page_count_coverage_is_not_pagination_authentication() {
            let (_, j, _) = sample();
            let first = j["jobs"][0].clone();
            let mut second = first.clone();
            second["id"] = json!(999);
            let p1 = json!({"total_count":2,"jobs":[first.clone()]});
            let p2 = json!({"total_count":2,"jobs":[second.clone()]});
            let b1 = serde_json::to_vec(&p1).unwrap();
            let b2 = serde_json::to_vec(&p2).unwrap();
            let (rows, covered) = parse_pages::<Job>(&[&b1, &b2], "jobs", |v| v.id).unwrap();
            assert_eq!(rows.len(), 2);
            assert!(covered);
            let (_, covered) = parse_pages::<Job>(&[&b1], "jobs", |v| v.id).unwrap();
            assert!(!covered);
            assert!(matches!(
                parse_pages::<Job>(&[&b1, &b1], "jobs", |v| v.id),
                Err(MainFixtureError::DuplicateIdentity)
            ));
            let drift =
                serde_json::to_vec(&json!({"total_count":3,"jobs":[second.clone()]})).unwrap();
            assert!(matches!(
                parse_pages::<Job>(&[&b1, &drift], "jobs", |v| v.id),
                Err(MainFixtureError::InconsistentPageTotals)
            ));
            let excess =
                serde_json::to_vec(&json!({"total_count":1,"jobs":[first,second]})).unwrap();
            assert!(matches!(
                parse_pages::<Job>(&[&excess], "jobs", |v| v.id),
                Err(MainFixtureError::CountExceedsDeclaredTotal)
            ));
            let (r, j, a) = sample();
            let report = inspect(&r, &j, &a);
            assert!(report.jobs_supplied_counts_cover_declared_total);
            assert!(
                report
                    .gaps
                    .contains(&MainPlanGap::PaginationTransportUnverified)
            );
        }

        #[test]
        fn body_and_selector_limits_fail_before_any_observation_can_be_used() {
            let s = MainFixtureSelector {
                run_id: 0,
                ..selector()
            };
            let expected = MainFixtureExpectations {
                workflow_commit: "a".repeat(40),
                workflow_bytes_digest: Digest::sha256_bytes(b"x"),
            };
            assert!(matches!(
                plan_main_snapshot_fixture(
                    &s,
                    &expected,
                    MainSnapshotFixture {
                        run_json: b"not JSON",
                        jobs_pages: &[],
                        artifact_pages: &[],
                        supplied_workflow_read_commit: None,
                        supplied_workflow_bytes: None
                    }
                ),
                Err(MainFixtureError::InvalidSelector)
            ));
            let huge = vec![b' '; MAX_MAIN_SNAPSHOT_BODY_BYTES + 1];
            assert!(matches!(
                plan_main_snapshot_fixture(
                    &selector(),
                    &expected,
                    MainSnapshotFixture {
                        run_json: &huge,
                        jobs_pages: &[],
                        artifact_pages: &[],
                        supplied_workflow_read_commit: None,
                        supplied_workflow_bytes: None
                    }
                ),
                Err(MainFixtureError::SizeLimit)
            ));
        }
    }
}
