use anyhow::{Context, Result};
use clap::Subcommand;
use cockpit_protocol::ci_admission::GithubEvidenceSelector;
use cockpit_protocol::{RuntimeContext, TaskAuthorizationCaller};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub(crate) enum GithubEvidenceCommand {
    /// Collect fixed-origin GitHub main-run metadata and explain missing verification proofs; never accepts evidence.
    Plan {
        /// Repository containing the current Work Item and caller policy.
        #[arg(long)]
        repo: PathBuf,
        /// Active Work Item whose current authority and source bindings are checked.
        #[arg(long)]
        work_item: String,
        /// Positive GitHub workflow run identity in the fixed repository.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        run_id: u64,
        /// Exact positive run attempt; the latest attempt is never substituted.
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        attempt: u32,
        /// Positive quality-job identity in that exact attempt.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        job_id: u64,
        /// Positive artifact metadata identity; no archive is downloaded or accepted.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        artifact_id: u64,
    },
}

pub(crate) fn run(
    command: GithubEvidenceCommand,
    runtime: &RuntimeContext,
    caller: Option<&TaskAuthorizationCaller>,
) -> Result<()> {
    let GithubEvidenceCommand::Plan {
        repo,
        work_item,
        run_id,
        attempt,
        job_id,
        artifact_id,
    } = command;
    super::require_compatible(&repo, runtime)?;
    let plan = cockpit_repository::plan_github_verification_admission(
        &repo,
        &work_item,
        &GithubEvidenceSelector {
            run_id,
            attempt,
            job_id,
            artifact_id,
        },
        runtime,
        caller,
    )
    .context("plan GitHub verification evidence without admission")?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}

#[derive(Debug, Subcommand)]
pub(crate) enum PortableEvidenceCommand {
    /// Inspect a local portable bundle without execution or writes; source claims remain unverified.
    Plan {
        /// Repository checkout whose protocol compatibility is checked.
        #[arg(long)]
        repo: PathBuf,
        /// Local portable evidence JSON file; contents never authenticate their author.
        #[arg(long)]
        input: PathBuf,
        /// Optional consumer Work Item; not required merely to inspect evidence.
        #[arg(long)]
        work_item: Option<String>,
    },
    /// Request admission under current authority and independent source proof; a file cannot authenticate itself.
    Accept {
        /// Repository checkout bound to the consumer Work Item.
        #[arg(long)]
        repo: PathBuf,
        /// Consumer Work Item; current caller and native action admission are rechecked.
        #[arg(long)]
        work_item: String,
        /// Local portable evidence JSON file inspected by the shared service.
        #[arg(long)]
        input: PathBuf,
        /// Exact sha256 plan digest returned by evidence bundle plan; not a source credential.
        #[arg(long, value_parser = parse_plan_digest)]
        expected_plan_digest: String,
    },
}

fn parse_plan_digest(value: &str) -> Result<String, String> {
    value
        .parse::<cockpit_core::Digest>()
        .map_err(|error| error.to_string())?;
    Ok(value.to_owned())
}

pub(crate) fn run_portable(
    command: PortableEvidenceCommand,
    runtime: &RuntimeContext,
    caller: Option<&TaskAuthorizationCaller>,
) -> Result<()> {
    match command {
        PortableEvidenceCommand::Plan {
            repo,
            input,
            work_item,
        } => {
            super::require_compatible(&repo, runtime)?;
            let plan = cockpit_repository::plan_portable_evidence(
                &repo,
                work_item.as_deref(),
                &input,
                runtime,
                caller,
            )
            .context("inspect portable evidence without execution or writes")?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        PortableEvidenceCommand::Accept {
            repo,
            work_item,
            input,
            expected_plan_digest,
        } => {
            let expected_plan_digest = expected_plan_digest
                .parse::<cockpit_core::Digest>()
                .context("validate exact portable evidence plan digest")?;
            super::require_compatible(&repo, runtime)?;
            let acceptance = cockpit_repository::accept_portable_evidence(
                &repo,
                &work_item,
                &input,
                &expected_plan_digest,
                runtime,
                caller,
            )
            .context("request portable evidence admission under current authority")?;
            println!("{}", serde_json::to_string_pretty(&acceptance)?);
        }
    }
    Ok(())
}

// Private source-observation bridge. No JSON-deserializable source capability.
// The host must supply reviewed recipe provenance and an independently fetching
// reader; these helpers alone never authenticate GitHub or accept core evidence.
const SOURCE_META_LIMIT: usize = 256 * 1024;
const SOURCE_LOG_LIMIT: usize = 64 * 1024;
const SOURCE_LINE_LIMIT: usize = 8192;
const SOURCE_JOBS_PER_PAGE: usize = 100;
const SOURCE_MAX_JOBS: usize = 600;
const SOURCE_MAX_PAGES: u32 = 6;
const NATIVE_MACHINE_PREFIX: &str = "AI_COCKPIT_NATIVE_SOURCE_V1 ";
const BINDER_MACHINE_PREFIX: &str = "AI_COCKPIT_ARTIFACT_BINDING_V1 ";

#[derive(Clone, Debug, PartialEq, Eq)]
enum SourceBridgeError {
    NotImplemented,
    RecipeUnavailable,
    RecipeInvalid,
    SelectorInvalid,
    ConsumerNonceInvalid,
    Transport,
    MetadataLimitOrTruncated,
    MetadataInvalid,
    RunBinding,
    JobsIncomplete,
    DuplicateJob,
    JobMissingOrBinding,
    StepBinding,
    LogLimitOrTruncated,
    MachineLineInvalid,
    MachineLineCount,
    MachinePayloadInvalid,
    NativeBinderMismatch,
    ArtifactBinding,
    SourceRootEncoding,
    WorkerRequestInvalid,
    Deadline,
    RequestLimit,
    TotalByteLimit,
    RedirectUnsupported,
    HttpStatus(u16),
    RecipeBytesMismatch,
    UnsupportedEvent,
    PullRequestMergeSourceUnverified,
    HostHeadUnavailable,
}

impl std::fmt::Display for SourceBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "github_source_observation:{self:?}")
    }
}
impl std::error::Error for SourceBridgeError {}

// Constructed by the embedding host after real code/recipe review, never from
// the agent's selector or provider logs. Presence is a prerequisite, not proof
// produced by this validator. Missing reviewed closure must stay unavailable.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewedSourceRecipe {
    repository_id: u64,
    repository: String,
    workflow_id: u64,
    workflow_path: String,
    reviewed_revision: String,
    event: String,
    branch: String,
    recipe_digest: cockpit_core::Digest,
    publisher_code_digest: cockpit_core::Digest,
    binder_code_digest: cockpit_core::Digest,
    expected_runtime_digest: cockpit_core::Digest,
    artifact_name: String,
    producer_job_name: String,
    native_step_name: String,
    upload_step_name: String,
    binder_step_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SourceReadRoute {
    WorkflowSource {
        revision: String,
    },
    PublisherSource {
        revision: String,
    },
    RunCurrent {
        run_id: u64,
    },
    RunAttempt {
        run_id: u64,
        attempt: u32,
    },
    AttemptJobs {
        run_id: u64,
        attempt: u32,
        page: u32,
        per_page: usize,
    },
    // Official numeric job/step logs route. The actual provider number comes
    // from this attempt's unique reviewed role, never GITHUB_JOB or a preset.
    JobStepLog {
        job_id: u64,
        step_number: u32,
    },
    ArtifactMetadata {
        artifact_id: u64,
    },
}

struct SourceRawRead {
    bytes: Vec<u8>,
    truncated: bool,
}

trait ControlledSourceReader {
    fn reviewed_recipe(&self) -> Option<ReviewedSourceRecipe>;
    // The real host worker must independently fetch each exact route on every
    // invocation, with its own transport/deadline policy. No replay flag exists.
    fn read(
        &mut self,
        route: &SourceReadRoute,
    ) -> std::result::Result<SourceRawRead, SourceBridgeError>;
}

#[derive(Debug)]
struct PrivateSourceObservation {
    consumer_nonce: String,
    input_digest: cockpit_core::Digest,
    source_root: cockpit_core::Digest,
    native_root: cockpit_core::Digest,
    publisher_nonce: String,
    artifact_id: u64,
    archive_digest: cockpit_core::Digest,
    // Exact raw response hashes are retained separately from the stable subject
    // root: unrelated jobs/collector timing may change between final rechecks.
    audit_reads: Vec<(SourceReadRoute, cockpit_core::Digest)>,
}

impl PrivateSourceObservation {
    fn audit_summary(&self, requests: usize, bytes: usize) -> serde_json::Value {
        serde_json::json!({"schemaVersion":1,"kind":"github_source_observation_audit",
            "nativeRoot":self.native_root,"publisherNonce":self.publisher_nonce,
            "artifactId":self.artifact_id,"archiveDigest":self.archive_digest,
            "reads":self.audit_reads,"readerCalls":requests,"bodyBytes":bytes})
    }
    fn worker_reply(&self) -> serde_json::Value {
        serde_json::json!({"schemaVersion":1,"nonce":self.consumer_nonce,
            "inputDigest":self.input_digest,"sourceRoot":self.source_root})
    }
}

#[derive(serde::Deserialize)]
struct SourceRepository {
    id: u64,
    full_name: String,
}
#[derive(serde::Deserialize)]
struct SourceRun {
    id: u64,
    run_attempt: u32,
    repository: SourceRepository,
    head_repository: SourceRepository,
    workflow_id: u64,
    head_sha: String,
    head_branch: String,
    path: String,
    event: String,
}
#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct SourceStep {
    number: u32,
    name: String,
    status: String,
    conclusion: Option<String>,
}
#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct SourceJob {
    id: u64,
    run_id: u64,
    run_attempt: u32,
    head_sha: String,
    name: String,
    status: String,
    conclusion: Option<String>,
    steps: Vec<SourceStep>,
}
#[derive(serde::Deserialize)]
struct SourceJobsPage {
    total_count: usize,
    jobs: Vec<SourceJob>,
}
#[derive(serde::Deserialize)]
struct SourceArtifactRun {
    id: u64,
    repository_id: u64,
    head_repository_id: u64,
    head_sha: String,
}
#[derive(serde::Deserialize)]
struct SourceArtifact {
    id: u64,
    name: String,
    digest: String,
    expired: bool,
    size_in_bytes: u64,
    workflow_run: SourceArtifactRun,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativeSourceMachine {
    schema_version: u32,
    publisher_nonce: String,
    input_digest: cockpit_core::Digest,
    native_root: cockpit_core::Digest,
    runtime_digest: cockpit_core::Digest,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BinderSourceMachine {
    schema_version: u32,
    publisher_nonce: String,
    input_digest: cockpit_core::Digest,
    native_root: cockpit_core::Digest,
    runtime_digest: cockpit_core::Digest,
    artifact_id: u64,
    archive_digest: cockpit_core::Digest,
}

// A bounded first pass rejects duplicate keys at every depth before typed
// deserialization can discard unknown provider fields or Value can collapse keys.
struct SourceJsonSeed<'a> {
    nodes: &'a std::cell::Cell<usize>,
    depth: u8,
}
impl<'de> serde::de::DeserializeSeed<'de> for SourceJsonSeed<'_> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        if self.depth > 32 || self.nodes.get() >= 32_768 {
            return Err(serde::de::Error::custom("source JSON budget"));
        }
        self.nodes.set(self.nodes.get() + 1);
        d.deserialize_any(self)
    }
}
impl<'de> serde::de::Visitor<'de> for SourceJsonSeed<'_> {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bounded unique-key JSON")
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> std::result::Result<(), E> {
        if v.is_finite() {
            Ok(())
        } else {
            Err(E::custom("finite JSON required"))
        }
    }
    fn visit_str<E: serde::de::Error>(self, _: &str) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_string<E: serde::de::Error>(self, _: String) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_none<E: serde::de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> std::result::Result<(), A::Error> {
        let mut count = 0;
        while seq
            .next_element_seed(SourceJsonSeed {
                nodes: self.nodes,
                depth: self.depth + 1,
            })?
            .is_some()
        {
            count += 1;
            if count > SOURCE_MAX_JOBS {
                return Err(serde::de::Error::custom("source array budget"));
            }
        }
        Ok(())
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<(), A::Error> {
        let mut keys = std::collections::BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if key.len() > 128 || keys.len() >= 128 || !keys.insert(key) {
                return Err(serde::de::Error::custom("source duplicate/key budget"));
            }
            map.next_value_seed(SourceJsonSeed {
                nodes: self.nodes,
                depth: self.depth + 1,
            })?;
        }
        Ok(())
    }
}
fn source_json<T: serde::de::DeserializeOwned>(
    raw: &[u8],
) -> std::result::Result<T, SourceBridgeError> {
    use serde::de::DeserializeSeed as _;
    let nodes = std::cell::Cell::new(0);
    let mut d = serde_json::Deserializer::from_slice(raw);
    SourceJsonSeed {
        nodes: &nodes,
        depth: 0,
    }
    .deserialize(&mut d)
    .and_then(|()| d.end())
    .map_err(|_| SourceBridgeError::MetadataInvalid)?;
    serde_json::from_slice(raw).map_err(|_| SourceBridgeError::MetadataInvalid)
}
fn source_digest_valid(d: &cockpit_core::Digest) -> bool {
    d.as_str().parse::<cockpit_core::Digest>().is_ok()
}
fn source_revision_valid(s: &str) -> bool {
    matches!(s.len(), 40 | 64)
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn source_plain_text(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && s.bytes().all(|b| (0x20..=0x7e).contains(&b))
}
fn checked_source_read(
    reader: &mut dyn ControlledSourceReader,
    route: SourceReadRoute,
    log: bool,
    audit: &mut Vec<(SourceReadRoute, cockpit_core::Digest)>,
) -> std::result::Result<Vec<u8>, SourceBridgeError> {
    let raw = reader.read(&route)?;
    if raw.truncated
        || raw.bytes.len()
            > if log {
                SOURCE_LOG_LIMIT
            } else {
                SOURCE_META_LIMIT
            }
    {
        return Err(if log {
            SourceBridgeError::LogLimitOrTruncated
        } else {
            SourceBridgeError::MetadataLimitOrTruncated
        });
    }
    audit.push((route, cockpit_core::Digest::sha256_bytes(&raw.bytes)));
    Ok(raw.bytes)
}

// GitHub's step log framing may prepend an ISO UTC timestamp. Strip exactly
// that syntax only; arbitrary prefixes/suffixes cannot hide machine records.
fn source_timestamp_body(line: &str) -> &str {
    let Some((stamp, body)) = line.split_once(' ') else {
        return line;
    };
    let b = stamp.as_bytes();
    let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
    if !(20..=30).contains(&b.len())
        || !stamp.ends_with('Z')
        || separators.iter().any(|(i, c)| b[*i] != *c)
        || b[..19]
            .iter()
            .enumerate()
            .any(|(i, c)| ![4, 7, 10, 13, 16].contains(&i) && !c.is_ascii_digit())
        || (b.len() > 20
            && (b[19] != b'.'
                || b.len() < 22
                || !b[20..b.len() - 1].iter().all(u8::is_ascii_digit)))
    {
        return line;
    }
    body
}
fn source_machine<T: serde::de::DeserializeOwned>(
    raw: &[u8],
    prefix: &str,
) -> std::result::Result<T, SourceBridgeError> {
    if raw.is_empty()
        || !raw.ends_with(b"\n")
        || raw.iter().any(|b| (*b < 0x20 && *b != b'\n') || *b == 0x7f)
    {
        return Err(SourceBridgeError::MachineLineInvalid);
    }
    let text = std::str::from_utf8(raw).map_err(|_| SourceBridgeError::MachineLineInvalid)?;
    if text.chars().any(|c| c.is_control() && c != '\n') {
        return Err(SourceBridgeError::MachineLineInvalid);
    }
    let mut selected = None;
    for line in text.split_terminator('\n') {
        if line.len() > SOURCE_LINE_LIMIT {
            return Err(SourceBridgeError::MachineLineInvalid);
        }
        let body = source_timestamp_body(line);
        if let Some(hex) = body.strip_prefix(prefix) {
            if selected.is_some() {
                return Err(SourceBridgeError::MachineLineCount);
            }
            if hex.is_empty()
                || !hex.len().is_multiple_of(2)
                || !hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(SourceBridgeError::MachineLineInvalid);
            }
            selected = Some(hex::decode(hex).map_err(|_| SourceBridgeError::MachineLineInvalid)?);
        } else if line.contains(NATIVE_MACHINE_PREFIX.trim_end())
            || line.contains(BINDER_MACHINE_PREFIX.trim_end())
        {
            return Err(SourceBridgeError::MachineLineInvalid);
        }
    }
    let bytes = selected.ok_or(SourceBridgeError::MachineLineCount)?;
    source_json(&bytes).map_err(|_| SourceBridgeError::MachinePayloadInvalid)
}

fn read_source_observation(
    reader: &mut dyn ControlledSourceReader,
    selector: &GithubEvidenceSelector,
    expected_input: &cockpit_core::Digest,
    consumer_nonce: &str,
) -> std::result::Result<PrivateSourceObservation, SourceBridgeError> {
    let _ = (reader, selector, expected_input, consumer_nonce);
    Err(SourceBridgeError::NotImplemented)
}

#[cfg(test)]
mod source_bridge_tests {
    use super::*;
    use serde_json::{Value, json};
    use std::collections::VecDeque;

    struct FixtureReader {
        recipe: Option<ReviewedSourceRecipe>,
        replies: VecDeque<(SourceReadRoute, SourceRawRead)>,
        calls: Vec<SourceReadRoute>,
    }
    impl ControlledSourceReader for FixtureReader {
        fn reviewed_recipe(&self) -> Option<ReviewedSourceRecipe> {
            self.recipe.clone()
        }
        fn read(
            &mut self,
            route: &SourceReadRoute,
        ) -> std::result::Result<SourceRawRead, SourceBridgeError> {
            self.calls.push(route.clone());
            let (expected, raw) = self
                .replies
                .pop_front()
                .expect("unexpected extra controlled read");
            assert_eq!(
                *route, expected,
                "exact route: no latest-attempt, URL or broad-log fallback"
            );
            Ok(raw)
        }
    }
    fn digest(bytes: &[u8]) -> cockpit_core::Digest {
        cockpit_core::Digest::sha256_bytes(bytes)
    }
    fn native() -> Value {
        json!({"schemaVersion":1,"publisherNonce":"historical-producer-nonce",
        "inputDigest":digest(b"exact portable JSON bytes"),"nativeRoot":digest(b"exact portable JSON bytes"),"runtimeDigest":digest(b"actual fixture Runtime")})
    }
    fn binder() -> Value {
        let mut v = native();
        v["artifactId"] = 41.into();
        v["archiveDigest"] = json!(digest(b"ZIP bytes"));
        v
    }
    fn machine(prefix: &str, v: &Value) -> Vec<u8> {
        format!(
            "2026-11-01T12:00:00.000Z {prefix}{}\n",
            hex::encode(serde_json::to_vec(v).unwrap())
        )
        .into_bytes()
    }
    fn job(id: u64, producer: bool) -> Value {
        json!({"id":id,"run_id":11,"run_attempt":2,
        "head_sha":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","name":if producer {"native producer"} else {"consumer still running"},
        "status":if producer {"completed"} else {"in_progress"},"conclusion":if producer {Some("success")} else {None},
        "steps":[{"number":7,"name":"native publish","status":"completed","conclusion":"success"},
                 {"number":8,"name":"upload native archive","status":"completed","conclusion":"success"},
                 {"number":9,"name":"bind uploaded archive","status":"completed","conclusion":"success"}]})
    }
    fn fixture() -> (FixtureReader, GithubEvidenceSelector, cockpit_core::Digest) {
        let recipe = ReviewedSourceRecipe {
            repository_id: 101,
            repository: "owner/project".into(),
            workflow_id: 8,
            workflow_path: ".github/workflows/ci.yml".into(),
            reviewed_revision: "e".repeat(40),
            event: "push".into(),
            branch: "main".into(),
            recipe_digest: digest(b"synthetic reviewed recipe: no provider claim"),
            publisher_code_digest: digest(b"synthetic publisher"),
            binder_code_digest: digest(b"synthetic binder"),
            expected_runtime_digest: digest(b"actual fixture Runtime"),
            artifact_name: "ai-cockpit-native-evidence".into(),
            producer_job_name: "native producer".into(),
            native_step_name: "native publish".into(),
            upload_step_name: "upload native archive".into(),
            binder_step_name: "bind uploaded archive".into(),
        };
        let selector = GithubEvidenceSelector {
            run_id: 11,
            attempt: 2,
            job_id: 21,
            artifact_id: 41,
        };
        let mut rows=vec![
            (SourceReadRoute::RunAttempt {run_id:11,attempt:2}, serde_json::to_vec(&json!({"id":11,"run_attempt":2,
                "repository":{"id":101,"full_name":"owner/project"},"head_repository":{"id":101,"full_name":"owner/project"},"workflow_id":8,"path":".github/workflows/ci.yml",
                "head_sha":"e".repeat(40),"head_branch":"main","event":"push","status":"in_progress"})).unwrap()),
            (SourceReadRoute::AttemptJobs {run_id:11,attempt:2,page:1,per_page:100},serde_json::to_vec(&json!({"total_count":2,"jobs":[job(21,true),job(22,false)]})).unwrap()),
            (SourceReadRoute::JobStepLog {job_id:21,step_number:7},machine(NATIVE_MACHINE_PREFIX,&native())),
            (SourceReadRoute::JobStepLog {job_id:21,step_number:9},machine(BINDER_MACHINE_PREFIX,&binder())),
            (SourceReadRoute::ArtifactMetadata {artifact_id:41},serde_json::to_vec(&json!({"id":41,"name":"ai-cockpit-native-evidence","digest":digest(b"ZIP bytes"),
                "expired":false,"size_in_bytes":4096,"workflow_run":{"id":11,"repository_id":101,"head_repository_id":101,"head_sha":"e".repeat(40)}})).unwrap()),
        ];
        let current_run_bytes = rows[0].1.clone();
        rows.insert(
            0,
            (
                SourceReadRoute::RunCurrent { run_id: 11 },
                current_run_bytes,
            ),
        );
        (
            FixtureReader {
                recipe: Some(recipe),
                replies: rows
                    .into_iter()
                    .map(|(route, bytes)| {
                        (
                            route,
                            SourceRawRead {
                                bytes,
                                truncated: false,
                            },
                        )
                    })
                    .collect(),
                calls: Vec::new(),
            },
            selector,
            digest(b"exact portable JSON bytes"),
        )
    }
    fn change_json(reader: &mut FixtureReader, index: usize, change: impl FnOnce(&mut Value)) {
        let mut v: Value = serde_json::from_slice(&reader.replies[index].1.bytes).unwrap();
        change(&mut v);
        reader.replies[index].1.bytes = serde_json::to_vec(&v).unwrap();
    }
    fn observe(
        reader: &mut FixtureReader,
        selector: &GithubEvidenceSelector,
        input: &cockpit_core::Digest,
    ) -> std::result::Result<PrivateSourceObservation, SourceBridgeError> {
        read_source_observation(reader, selector, input, "fresh-consumer-session-nonce")
    }

    #[test]
    fn source_bridge_reads_exact_routes_and_separates_all_roots_and_nonces() {
        let (mut reader, selector, input) = fixture();
        let expected = reader
            .replies
            .iter()
            .map(|(route, _)| route.clone())
            .collect::<Vec<_>>();
        let observation = observe(&mut reader, &selector, &input)
            .expect("TARGET: controlled independent raw records must correlate");
        assert_eq!(reader.calls, expected);
        assert!(reader.replies.is_empty());
        assert_eq!(observation.input_digest, input);
        assert_eq!(
            observation.native_root,
            digest(b"exact portable JSON bytes")
        );
        assert_eq!(observation.publisher_nonce, "historical-producer-nonce");
        assert_eq!(observation.artifact_id, 41);
        assert_eq!(observation.archive_digest, digest(b"ZIP bytes"));
        assert_eq!(observation.audit_reads.len(), 6);
        assert_ne!(observation.source_root, observation.native_root);
        let reply = observation.worker_reply();
        assert_eq!(
            reply,
            json!({"schemaVersion":1,"nonce":"fresh-consumer-session-nonce","inputDigest":input,"sourceRoot":observation.source_root})
        );
        assert_eq!(reply.as_object().unwrap().len(), 4);
        // Role placement is provider data. Moving either reviewed role must
        // select the new numeric endpoints without changing host recipe data.
        for (native_number, upload_number, binder_number) in [(3_u32, 4_u32, 5_u32), (17, 23, 29)] {
            let (mut moved, s, input) = fixture();
            change_json(&mut moved, 2, |v| {
                v["jobs"][0]["steps"][0]["number"] = native_number.into();
                v["jobs"][0]["steps"][1]["number"] = upload_number.into();
                v["jobs"][0]["steps"][2]["number"] = binder_number.into();
            });
            moved.replies[3].0 = SourceReadRoute::JobStepLog {
                job_id: 21,
                step_number: native_number,
            };
            moved.replies[4].0 = SourceReadRoute::JobStepLog {
                job_id: 21,
                step_number: binder_number,
            };
            let expected = moved
                .replies
                .iter()
                .map(|(route, _)| route.clone())
                .collect::<Vec<_>>();
            let changed = observe(&mut moved, &s, &input)
                .expect("TARGET: unique reviewed roles survive provider step movement");
            assert_eq!(moved.calls, expected);
            assert!(moved.replies.is_empty());
            assert_eq!(changed.native_root, observation.native_root);
            assert_ne!(changed.source_root, observation.source_root);
        }
    }
    #[test]
    fn source_bridge_root_is_stable_across_unrelated_progress_and_fresh_consumer_challenges() {
        let (mut first, s, input) = fixture();
        let a = observe(&mut first, &s, &input).expect("TARGET stable subject baseline");
        let (mut second, _, _) = fixture();
        change_json(&mut second, 0, |v| {
            v["status"] = "completed".into();
            v["updated_at"] = "current run progress only".into();
        });
        change_json(&mut second, 1, |v| {
            v["status"] = "completed".into();
            v["updated_at"] = "later unrelated observation".into();
        });
        change_json(&mut second, 2, |v| {
            v["jobs"][1]["status"] = "completed".into();
            v["jobs"][1]["conclusion"] = "failure".into();
        });
        let b = read_source_observation(&mut second, &s, &input, "another-fresh-consumer-nonce")
            .expect("TARGET new invocation revalidates all six reads");
        assert_eq!(second.calls.len(), 6);
        assert_eq!(a.source_root, b.source_root);
        assert_ne!(a.audit_reads, b.audit_reads);
        assert_ne!(a.consumer_nonce, b.consumer_nonce);
        // A later full observation must consult current run again. Exact
        // attempt2 remains unchanged while provider current attempt3 exists.
        for status in ["in_progress", "completed"] {
            let (mut advanced, _, _) = fixture();
            change_json(&mut advanced, 0, |v| {
                v["run_attempt"] = 3.into();
                v["status"] = status.into();
                v["conclusion"] = json!(if status == "completed" {
                    Some("failure")
                } else {
                    None
                });
            });
            assert_eq!(
                observe(&mut advanced, &s, &input).unwrap_err(),
                SourceBridgeError::RunBinding
            );
            assert_eq!(
                advanced.calls,
                vec![SourceReadRoute::RunCurrent { run_id: 11 }]
            );
            assert_eq!(
                advanced.replies.front().unwrap().0,
                SourceReadRoute::RunAttempt {
                    run_id: 11,
                    attempt: 2
                }
            );
        }
    }
    #[test]
    fn source_bridge_finishes_attempt_job_pagination_before_any_step_read() {
        let (mut reader, s, input) = fixture();
        reader.replies[2].1.bytes=serde_json::to_vec(&json!({"total_count":101,"jobs":(1000..1100).map(|id|job(id,false)).collect::<Vec<_>>()})).unwrap();
        reader.replies.insert(
            3,
            (
                SourceReadRoute::AttemptJobs {
                    run_id: 11,
                    attempt: 2,
                    page: 2,
                    per_page: 100,
                },
                SourceRawRead {
                    bytes: serde_json::to_vec(&json!({"total_count":101,"jobs":[job(21,true)]}))
                        .unwrap(),
                    truncated: false,
                },
            ),
        );
        let expected = reader
            .replies
            .iter()
            .map(|(r, _)| r.clone())
            .collect::<Vec<_>>();
        observe(&mut reader, &s, &input).expect("TARGET exact complete paginated attempt");
        assert_eq!(reader.calls, expected);
        assert_eq!(reader.calls.len(), 7);
    }
    #[test]
    fn source_bridge_rejects_wrong_attempt_repository_revision_and_job() {
        for (index, path, replacement, error) in [
            (
                0,
                "/repository/id",
                json!(999),
                SourceBridgeError::RunBinding,
            ),
            (
                0,
                "/head_sha",
                json!("f".repeat(40)),
                SourceBridgeError::RunBinding,
            ),
            (0, "/workflow_id", json!(99), SourceBridgeError::RunBinding),
            (1, "/run_attempt", json!(3), SourceBridgeError::RunBinding),
            (
                1,
                "/repository/id",
                json!(999),
                SourceBridgeError::RunBinding,
            ),
            (
                1,
                "/head_repository/id",
                json!(999),
                SourceBridgeError::RunBinding,
            ),
            (
                1,
                "/head_sha",
                json!("f".repeat(40)),
                SourceBridgeError::RunBinding,
            ),
            (
                2,
                "/jobs/0/run_attempt",
                json!(1),
                SourceBridgeError::JobMissingOrBinding,
            ),
            (
                2,
                "/jobs/0/id",
                json!(777),
                SourceBridgeError::JobMissingOrBinding,
            ),
            (
                2,
                "/jobs/0/status",
                json!("in_progress"),
                SourceBridgeError::JobMissingOrBinding,
            ),
        ] {
            let (mut r, s, input) = fixture();
            change_json(&mut r, index, |v| {
                *v.pointer_mut(path).unwrap() = replacement
            });
            assert_eq!(observe(&mut r, &s, &input).unwrap_err(), error, "{path}");
            assert!(r.calls.len() <= 3);
        }
    }
    #[test]
    fn source_bridge_rejects_wrong_missing_duplicate_or_failed_independent_steps() {
        for change in 0..12 {
            let (mut r, s, input) = fixture();
            change_json(&mut r, 2, |v| match change {
                0 => v["jobs"][0]["steps"][0]["number"] = 0.into(),
                1 => v["jobs"][0]["steps"][2]["name"] = "unreviewed binder".into(),
                2 => v["jobs"][0]["steps"][2]["conclusion"] = "failure".into(),
                3 => v["jobs"][0]["steps"][2]["number"] = 7.into(),
                4 => {
                    let mut duplicate = v["jobs"][0]["steps"][0].clone();
                    duplicate["number"] = 10.into();
                    v["jobs"][0]["steps"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }
                5 => {
                    let mut duplicate = v["jobs"][0]["steps"][2].clone();
                    duplicate["number"] = 11.into();
                    v["jobs"][0]["steps"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }
                6 => v["jobs"][0]["steps"][0]["number"] = 10.into(),
                7 => {
                    v["jobs"][0]["steps"].as_array_mut().unwrap().remove(1);
                }
                8 => {
                    let mut duplicate = v["jobs"][0]["steps"][1].clone();
                    duplicate["number"] = 10.into();
                    v["jobs"][0]["steps"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }
                9 => v["jobs"][0]["steps"][1]["conclusion"] = "failure".into(),
                10 => v["jobs"][0]["steps"][1]["number"] = 10.into(),
                _ => v["jobs"][0]["steps"][1]["number"] = 6.into(),
            });
            assert_eq!(
                observe(&mut r, &s, &input).unwrap_err(),
                SourceBridgeError::StepBinding
            );
            assert_eq!(r.calls.len(), 3);
        }
    }
    #[test]
    fn source_bridge_rejects_native_binder_root_nonce_input_and_artifact_disagreement() {
        for channels in [vec![3_usize], vec![4_usize], vec![3_usize, 4_usize]] {
            let (mut r, s, input) = fixture();
            for channel in channels {
                let mut payload = if channel == 3 { native() } else { binder() };
                payload["runtimeDigest"] = json!(digest(b"different code hash"));
                r.replies[channel].1.bytes = machine(
                    if channel == 3 {
                        NATIVE_MACHINE_PREFIX
                    } else {
                        BINDER_MACHINE_PREFIX
                    },
                    &payload,
                );
            }
            assert_eq!(
                observe(&mut r, &s, &input).unwrap_err(),
                SourceBridgeError::NativeBinderMismatch
            );
        }
        let (mut r, s, input) = fixture();
        for (channel, prefix, mut payload) in [
            (3, NATIVE_MACHINE_PREFIX, native()),
            (4, BINDER_MACHINE_PREFIX, binder()),
        ] {
            payload["nativeRoot"] = json!(digest(b"matching false root is not the raw packet"));
            r.replies[channel].1.bytes = machine(prefix, &payload);
        }
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::NativeBinderMismatch
        );
        for (key, value) in [
            ("nativeRoot", json!(digest(b"wrong native root"))),
            ("publisherNonce", json!("wrong publisher nonce")),
            ("inputDigest", json!(digest(b"wrong input bytes"))),
            ("artifactId", json!(42)),
            ("schemaVersion", json!(2)),
        ] {
            let (mut r, s, input) = fixture();
            let mut line = binder();
            line[key] = value;
            r.replies[4].1.bytes = machine(BINDER_MACHINE_PREFIX, &line);
            assert_eq!(
                observe(&mut r, &s, &input).unwrap_err(),
                SourceBridgeError::NativeBinderMismatch
            );
            assert_eq!(r.calls.len(), 5);
        }
        let (mut r, s, _) = fixture();
        assert_eq!(
            observe(&mut r, &s, &digest(b"other local packet")).unwrap_err(),
            SourceBridgeError::NativeBinderMismatch
        );
    }
    #[test]
    fn source_bridge_rejects_artifact_digest_run_identity_expiry_or_size_mismatch() {
        for (path, replacement) in [
            ("/id", json!(42)),
            ("/name", json!("wrong artifact name")),
            ("/digest", json!(digest(b"different ZIP"))),
            ("/workflow_run/id", json!(12)),
            ("/workflow_run/repository_id", json!(102)),
            ("/workflow_run/head_repository_id", json!(102)),
            ("/expired", json!(true)),
            ("/size_in_bytes", json!(0)),
        ] {
            let (mut r, s, input) = fixture();
            change_json(&mut r, 5, |v| *v.pointer_mut(path).unwrap() = replacement);
            assert_eq!(
                observe(&mut r, &s, &input).unwrap_err(),
                SourceBridgeError::ArtifactBinding,
                "{path}"
            );
            assert_eq!(r.calls.len(), 6);
        }
    }
    #[test]
    fn source_bridge_rejects_multiple_missing_control_truncated_or_injected_machine_lines() {
        for kind in 0..11 {
            let (mut r, s, input) = fixture();
            let original = r.replies[3].1.bytes.clone();
            let expected = match kind {
                0 => {
                    r.replies[3].1.bytes.extend(original);
                    SourceBridgeError::MachineLineCount
                }
                1 => {
                    r.replies[3].1.bytes = b"ordinary output only\n".to_vec();
                    SourceBridgeError::MachineLineCount
                }
                2 => {
                    r.replies[3].1.bytes.insert(0, 0x1b);
                    SourceBridgeError::MachineLineInvalid
                }
                3 => {
                    r.replies[3].1.truncated = true;
                    SourceBridgeError::LogLimitOrTruncated
                }
                4 => {
                    r.replies[3].1.bytes.pop();
                    SourceBridgeError::MachineLineInvalid
                }
                5 => {
                    r.replies[3].1.bytes = machine(BINDER_MACHINE_PREFIX, &binder());
                    SourceBridgeError::MachineLineInvalid
                }
                6 => {
                    r.replies[3].1.bytes = [b"echo ".to_vec(), original].concat();
                    SourceBridgeError::MachineLineInvalid
                }
                7 => {
                    r.replies[3].1.bytes = vec![b'x'; SOURCE_LOG_LIMIT + 1];
                    SourceBridgeError::LogLimitOrTruncated
                }
                8 => {
                    r.replies[3].1.bytes =
                        [vec![b'x'; SOURCE_LINE_LIMIT + 1], b"\n".to_vec(), original].concat();
                    SourceBridgeError::MachineLineInvalid
                }
                9 => {
                    r.replies[3].1.bytes =
                        ["hidden\u{85}control\n".as_bytes().to_vec(), original].concat();
                    SourceBridgeError::MachineLineInvalid
                }
                _ => {
                    r.replies[3].1.bytes = format!("{NATIVE_MACHINE_PREFIX}abc\n").into_bytes();
                    SourceBridgeError::MachineLineInvalid
                }
            };
            assert_eq!(observe(&mut r, &s, &input).unwrap_err(), expected);
        }
    }
    #[test]
    fn source_bridge_rejects_machine_unknown_authority_fields_duplicate_keys_and_multiple_json() {
        let duplicate = format!(
            "{{\"schemaVersion\":1,\"schemaVersion\":1,\"publisherNonce\":\"p\",\"inputDigest\":\"{}\",\"nativeRoot\":\"{}\"}}",
            digest(b"exact portable JSON bytes"),
            digest(b"exact portable JSON bytes")
        );
        let mut authority = native();
        authority["sourceVerified"] = true.into();
        for payload in [
            duplicate.into_bytes(),
            serde_json::to_vec(&authority).unwrap(),
            [serde_json::to_vec(&native()).unwrap(), b"{}".to_vec()].concat(),
        ] {
            let (mut r, s, input) = fixture();
            r.replies[3].1.bytes =
                format!("{NATIVE_MACHINE_PREFIX}{}\n", hex::encode(payload)).into_bytes();
            assert_eq!(
                observe(&mut r, &s, &input).unwrap_err(),
                SourceBridgeError::MachinePayloadInvalid
            );
        }
    }
    #[test]
    fn source_bridge_rejects_incomplete_duplicate_or_drifting_attempt_inventory() {
        let (mut r, s, input) = fixture();
        change_json(&mut r, 2, |v| v["total_count"] = 3.into());
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::JobsIncomplete
        );
        let (mut r, s, input) = fixture();
        change_json(&mut r, 2, |v| {
            let duplicate = v["jobs"][0].clone();
            v["jobs"][1] = duplicate;
        });
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::DuplicateJob
        );
        let (mut r, s, input) = fixture();
        change_json(&mut r, 2, |v| v["total_count"] = 1025.into());
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::JobsIncomplete
        );
    }
    #[test]
    fn source_bridge_requires_real_host_recipe_context_before_any_read() {
        let (mut r, s, input) = fixture();
        r.recipe = None;
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::RecipeUnavailable
        );
        assert!(r.calls.is_empty());
        for duplicate_role in ["native publish", "bind uploaded archive"] {
            let (mut r, s, input) = fixture();
            r.recipe.as_mut().unwrap().upload_step_name = duplicate_role.into();
            assert_eq!(
                observe(&mut r, &s, &input).unwrap_err(),
                SourceBridgeError::RecipeInvalid
            );
            assert!(r.calls.is_empty());
        }
        let (mut r, s, input) = fixture();
        r.recipe.as_mut().unwrap().binder_step_name = "native publish".into();
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::RecipeInvalid
        );
        assert!(r.calls.is_empty());
        let (mut r, s, input) = fixture();
        assert_eq!(
            read_source_observation(&mut r, &s, &input, "bad\nnonce").unwrap_err(),
            SourceBridgeError::ConsumerNonceInvalid
        );
        assert!(r.calls.is_empty());
    }
    #[test]
    fn source_bridge_refuses_truncated_metadata_and_deep_duplicate_json() {
        let (mut r, s, input) = fixture();
        r.replies[1].1.truncated = true;
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::MetadataLimitOrTruncated
        );
        let (mut r, s, input) = fixture();
        r.replies[1].1.bytes = vec![b'x'; SOURCE_META_LIMIT + 1];
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::MetadataLimitOrTruncated
        );
        let (mut r, s, input) = fixture();
        let mut v: Value = serde_json::from_slice(&r.replies[1].1.bytes).unwrap();
        v["ignored"] = json!({"placeholder":1});
        let text = serde_json::to_string(&v)
            .unwrap()
            .replace("\"placeholder\":1", "\"x\":1,\"x\":2");
        r.replies[1].1.bytes = text.into_bytes();
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::MetadataInvalid
        );
    }

    #[test]
    fn source_bridge_selector_cannot_supply_endpoint_command_or_trust() {
        for field in [
            "url",
            "command",
            "trusted",
            "sourceRoot",
            "inputDigest",
            "recipe",
        ] {
            let mut v = json!({"runId":11,"attempt":2,"jobId":21,"artifactId":41});
            v[field] = json!("not authority");
            assert!(serde_json::from_value::<GithubEvidenceSelector>(v).is_err());
        }
        let (mut r, mut s, input) = fixture();
        s.attempt = 0;
        assert_eq!(
            observe(&mut r, &s, &input).unwrap_err(),
            SourceBridgeError::SelectorInvalid
        );
        assert!(r.calls.is_empty());
    }
}

const SOURCE_ORIGIN: &str = "https://api.github.com";
const SOURCE_REPOSITORY: &str = "xinglun/ai-cockpit";
const SOURCE_REPOSITORY_ID: u64 = 1_341_189_248;
const SOURCE_WORKFLOW_ID: u64 = 338_986_437;
const SOURCE_WORKFLOW_PATH: &str = ".github/workflows/ci.yml";
const SOURCE_HELPER_PATH: &str = "tests/ci/run_hosted_runtime_verification.sh";
const SOURCE_WORKFLOW_BYTES: &[u8] = include_bytes!("../../../.github/workflows/ci.yml");
const SOURCE_HELPER_BYTES: &[u8] =
    include_bytes!("../../../tests/ci/run_hosted_runtime_verification.sh");
const SOURCE_MAX_REQUESTS: usize = 16;
const SOURCE_TOTAL_BYTES: usize = 32 * 1024 * 1024;
const SOURCE_AUDIT_BYTES: usize = 8192;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SourceWorkerWire {
    schema_version: u32,
    nonce: String,
    input_digest: cockpit_core::Digest,
    parameters: serde_json::Value,
    repository_root: PathBuf,
    work_item_id: String,
    task_caller: Option<TaskAuthorizationCaller>,
    observer_tool_digest: cockpit_core::Digest,
}

fn source_worker_wire(
    request: &str,
    runtime: &RuntimeContext,
) -> std::result::Result<(SourceWorkerWire, GithubEvidenceSelector), SourceBridgeError> {
    if request.len() > SOURCE_META_LIMIT {
        return Err(SourceBridgeError::WorkerRequestInvalid);
    }
    let wire: SourceWorkerWire = source_json(request.as_bytes())?;
    let selector: GithubEvidenceSelector = serde_json::from_value(wire.parameters.clone())
        .map_err(|_| SourceBridgeError::WorkerRequestInvalid)?;
    selector
        .validate()
        .map_err(|_| SourceBridgeError::SelectorInvalid)?;
    if wire.schema_version != 1
        || !source_plain_text(&wire.nonce)
        || !source_digest_valid(&wire.input_digest)
        || !source_digest_valid(&wire.observer_tool_digest)
        || wire.observer_tool_digest != runtime.runtime_digest
        || !wire.repository_root.is_absolute()
        || wire.work_item_id.is_empty()
    {
        return Err(SourceBridgeError::WorkerRequestInvalid);
    }
    Ok((wire, selector))
}

struct SourceHttpBudget {
    deadline: std::time::Instant,
    requests: usize,
    bytes: usize,
}
impl SourceHttpBudget {
    fn remaining(&self) -> std::result::Result<std::time::Duration, SourceBridgeError> {
        self.deadline
            .checked_duration_since(std::time::Instant::now())
            .filter(|v| !v.is_zero())
            .ok_or(SourceBridgeError::Deadline)
    }
    fn begin_read(&mut self) -> std::result::Result<std::time::Duration, SourceBridgeError> {
        let remaining = self.remaining()?;
        if self.requests >= SOURCE_MAX_REQUESTS {
            return Err(SourceBridgeError::RequestLimit);
        }
        self.requests += 1;
        Ok(remaining.min(std::time::Duration::from_secs(20)))
    }
    fn add_bytes(&mut self, count: usize) -> std::result::Result<(), SourceBridgeError> {
        self.bytes = self
            .bytes
            .checked_add(count)
            .filter(|v| *v <= SOURCE_TOTAL_BYTES)
            .ok_or(SourceBridgeError::TotalByteLimit)?;
        self.remaining()?;
        Ok(())
    }
}

fn bounded_source_body<R: std::io::Read>(
    reader: &mut R,
    content_length: Option<usize>,
    limit: usize,
    budget: &mut SourceHttpBudget,
    read_deadline: std::time::Instant,
) -> std::result::Result<SourceRawRead, SourceBridgeError> {
    if content_length.is_some_and(|v| v > limit || v > SOURCE_TOTAL_BYTES - budget.bytes) {
        return Err(SourceBridgeError::MetadataLimitOrTruncated);
    }
    let mut bytes = Vec::with_capacity(content_length.unwrap_or(0));
    let mut chunk = [0_u8; 8192];
    loop {
        budget.remaining()?;
        if std::time::Instant::now() >= read_deadline {
            return Err(SourceBridgeError::Deadline);
        }
        let count =
            std::io::Read::read(reader, &mut chunk).map_err(|_| SourceBridgeError::Transport)?;
        if count == 0 {
            break;
        }
        budget.add_bytes(count)?;
        if bytes.len().checked_add(count).is_none_or(|v| v > limit) {
            return Err(SourceBridgeError::MetadataLimitOrTruncated);
        }
        bytes
            .try_reserve_exact(count)
            .map_err(|_| SourceBridgeError::TotalByteLimit)?;
        bytes.extend_from_slice(&chunk[..count]);
    }
    budget.remaining()?;
    if std::time::Instant::now() >= read_deadline {
        return Err(SourceBridgeError::Deadline);
    }
    if content_length.is_some_and(|v| bytes.len() != v) {
        return Err(SourceBridgeError::Transport);
    }
    Ok(SourceRawRead {
        bytes,
        truncated: false,
    })
}

struct SourceHttpsReader {
    budget: SourceHttpBudget,
    recipe: Option<ReviewedSourceRecipe>,
    bootstrap_audit: Vec<(SourceReadRoute, cockpit_core::Digest)>,
}
impl SourceHttpsReader {
    fn get(
        &mut self,
        path: &str,
        raw: bool,
        limit: usize,
    ) -> std::result::Result<SourceRawRead, SourceBridgeError> {
        // Paths are assembled only by read() from fixed host strings and typed
        // numbers/revision OIDs. No provider URL/header or agent endpoint is used.
        if !path.starts_with("/repos/xinglun/ai-cockpit/")
            || path.contains("..")
            || path
                .bytes()
                .any(|b| b.is_ascii_control() || b == b' ' || b == b'\\' || b == b'#')
        {
            return Err(SourceBridgeError::Transport);
        }
        let timeout = self.budget.begin_read()?;
        let read_deadline = std::time::Instant::now() + timeout;
        let config = ureq::Agent::config_builder()
            .https_only(true)
            .proxy(None)
            .max_redirects(0)
            .http_status_as_error(false)
            .max_response_header_size(16 * 1024)
            .timeout_global(Some(timeout))
            .timeout_per_call(Some(timeout))
            .timeout_resolve(Some(timeout))
            .timeout_connect(Some(timeout))
            .timeout_recv_response(Some(timeout))
            .timeout_recv_body(Some(timeout))
            .timeout_per_read(Some(timeout))
            .build();
        // A fresh agent per exact GET has no shared cookies/auth or stale
        // pooled connection. No application retry or redirect branch exists.
        let agent = ureq::Agent::new_with_config(config);
        let mut response = agent
            .get(&format!("{SOURCE_ORIGIN}{path}"))
            .header(
                "Accept",
                if raw {
                    "application/vnd.github.raw+json"
                } else {
                    "application/vnd.github+json"
                },
            )
            .header("Accept-Encoding", "identity")
            .header("Cache-Control", "no-cache")
            .header("User-Agent", "ai-cockpit-controlled-source-observer")
            .call()
            .map_err(|_| SourceBridgeError::Transport)?;
        let status = response.status().as_u16();
        if (300..400).contains(&status) {
            return Err(SourceBridgeError::RedirectUnsupported);
        }
        if status != 200 {
            return Err(SourceBridgeError::HttpStatus(status));
        }
        let mut lengths = response.headers().get_all("content-length").iter();
        let content_length = match lengths.next() {
            Some(value) => Some(
                value
                    .to_str()
                    .map_err(|_| SourceBridgeError::Transport)?
                    .parse::<usize>()
                    .map_err(|_| SourceBridgeError::Transport)?,
            ),
            None => None,
        };
        if lengths.next().is_some()
            || content_length
                .is_some_and(|v| v > limit || v > SOURCE_TOTAL_BYTES - self.budget.bytes)
        {
            return Err(SourceBridgeError::MetadataLimitOrTruncated);
        }
        if response
            .headers()
            .get("content-encoding")
            .is_some_and(|v| v.as_bytes() != b"identity")
        {
            return Err(SourceBridgeError::Transport);
        }
        let mut reader = response.body_mut().as_reader();
        bounded_source_body(
            &mut reader,
            content_length,
            limit,
            &mut self.budget,
            read_deadline,
        )
    }
    fn bootstrap(
        deadline: std::time::Instant,
        revision: &str,
        selector: &GithubEvidenceSelector,
        runtime: &RuntimeContext,
    ) -> std::result::Result<Self, SourceBridgeError> {
        if !source_revision_valid(revision) {
            return Err(SourceBridgeError::RecipeInvalid);
        }
        let mut reader = Self {
            budget: SourceHttpBudget {
                deadline,
                requests: 0,
                bytes: 0,
            },
            recipe: None,
            bootstrap_audit: Vec::new(),
        };
        for (route, expected) in [
            (
                SourceReadRoute::WorkflowSource {
                    revision: revision.into(),
                },
                SOURCE_WORKFLOW_BYTES,
            ),
            (
                SourceReadRoute::PublisherSource {
                    revision: revision.into(),
                },
                SOURCE_HELPER_BYTES,
            ),
        ] {
            let raw = reader.read(&route)?;
            if raw.truncated || raw.bytes != expected {
                return Err(SourceBridgeError::RecipeBytesMismatch);
            }
            reader
                .bootstrap_audit
                .push((route, cockpit_core::Digest::sha256_bytes(&raw.bytes)));
        }
        let route = SourceReadRoute::RunCurrent {
            run_id: selector.run_id,
        };
        let current = reader.read(&route)?;
        let run: SourceRun = source_json(&current.bytes)?;
        reader
            .bootstrap_audit
            .push((route, cockpit_core::Digest::sha256_bytes(&current.bytes)));
        if matches!(run.event.as_str(), "pull_request" | "pull_request_target") {
            return Err(SourceBridgeError::PullRequestMergeSourceUnverified);
        }
        if !matches!(run.event.as_str(), "push" | "workflow_dispatch") {
            return Err(SourceBridgeError::UnsupportedEvent);
        }
        if run.repository.id != SOURCE_REPOSITORY_ID
            || run.repository.full_name != SOURCE_REPOSITORY
            || run.head_repository.id != SOURCE_REPOSITORY_ID
            || run.head_repository.full_name != SOURCE_REPOSITORY
            || run.id != selector.run_id
            || run.run_attempt != selector.attempt
            || run.head_sha != revision
            || run.head_branch != "main"
            || run.workflow_id != SOURCE_WORKFLOW_ID
            || run.path != SOURCE_WORKFLOW_PATH
        {
            return Err(SourceBridgeError::RunBinding);
        }
        let workflow_digest = cockpit_core::Digest::sha256_bytes(SOURCE_WORKFLOW_BYTES);
        let helper_digest = cockpit_core::Digest::sha256_bytes(SOURCE_HELPER_BYTES);
        let recipe_digest = cockpit_protocol::digest_json(&serde_json::json!({
            "domain":"ai-cockpit:compiled-github-source-recipe:v1","repository":SOURCE_REPOSITORY,
            "workflowDigest":workflow_digest,"helperDigest":helper_digest,
            "uploadAction":"actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
            "artifactName":"ai-cockpit-native-evidence","fileName":"portable-native-evidence.json"
        }))
        .map_err(|_| SourceBridgeError::SourceRootEncoding)?;
        reader.recipe = Some(ReviewedSourceRecipe {
            repository_id: SOURCE_REPOSITORY_ID,
            repository: SOURCE_REPOSITORY.into(),
            workflow_id: SOURCE_WORKFLOW_ID,
            workflow_path: SOURCE_WORKFLOW_PATH.into(),
            reviewed_revision: revision.into(),
            event: run.event,
            branch: "main".into(),
            recipe_digest,
            publisher_code_digest: helper_digest.clone(),
            binder_code_digest: helper_digest,
            expected_runtime_digest: runtime.runtime_digest.clone(),
            artifact_name: "ai-cockpit-native-evidence".into(),
            producer_job_name: "quality".into(),
            native_step_name: "AI Cockpit native evidence".into(),
            upload_step_name: "Upload AI Cockpit native evidence".into(),
            binder_step_name: "Bind AI Cockpit native evidence".into(),
        });
        reader.budget.remaining()?;
        Ok(reader)
    }
}
impl ControlledSourceReader for SourceHttpsReader {
    fn reviewed_recipe(&self) -> Option<ReviewedSourceRecipe> {
        self.recipe.clone()
    }
    fn read(
        &mut self,
        route: &SourceReadRoute,
    ) -> std::result::Result<SourceRawRead, SourceBridgeError> {
        let (path, raw, limit) = match route {
            SourceReadRoute::RunCurrent { run_id } => (
                format!("/repos/{SOURCE_REPOSITORY}/actions/runs/{run_id}"),
                false,
                SOURCE_META_LIMIT,
            ),
            SourceReadRoute::RunAttempt { run_id, attempt } => (
                format!("/repos/{SOURCE_REPOSITORY}/actions/runs/{run_id}/attempts/{attempt}"),
                false,
                SOURCE_META_LIMIT,
            ),
            SourceReadRoute::AttemptJobs {
                run_id,
                attempt,
                page,
                per_page,
            } => (
                format!(
                    "/repos/{SOURCE_REPOSITORY}/actions/runs/{run_id}/attempts/{attempt}/jobs?per_page={per_page}&page={page}"
                ),
                false,
                SOURCE_META_LIMIT,
            ),
            SourceReadRoute::JobStepLog {
                job_id,
                step_number,
            } => (
                format!(
                    "/repos/{SOURCE_REPOSITORY}/actions/jobs/{job_id}/steps/{step_number}/logs"
                ),
                true,
                SOURCE_LOG_LIMIT,
            ),
            SourceReadRoute::ArtifactMetadata { artifact_id } => (
                format!("/repos/{SOURCE_REPOSITORY}/actions/artifacts/{artifact_id}"),
                false,
                SOURCE_META_LIMIT,
            ),
            SourceReadRoute::WorkflowSource { revision } => {
                if !source_revision_valid(revision) {
                    return Err(SourceBridgeError::RecipeInvalid);
                }
                (
                    format!(
                        "/repos/{SOURCE_REPOSITORY}/contents/{SOURCE_WORKFLOW_PATH}?ref={revision}"
                    ),
                    true,
                    SOURCE_META_LIMIT,
                )
            }
            SourceReadRoute::PublisherSource { revision } => {
                if !source_revision_valid(revision) {
                    return Err(SourceBridgeError::RecipeInvalid);
                }
                (
                    format!(
                        "/repos/{SOURCE_REPOSITORY}/contents/{SOURCE_HELPER_PATH}?ref={revision}"
                    ),
                    true,
                    SOURCE_META_LIMIT,
                )
            }
        };
        self.get(&path, raw, limit)
    }
}

/// Private process mode. Native admission and actual executable identity precede
/// any provider I/O; imported JSON cannot choose a tool, policy or endpoint.
pub(crate) fn source_worker(request: &str, runtime: &RuntimeContext) -> Result<serde_json::Value> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    let (wire, selector) = source_worker_wire(request, runtime)?;
    let before = cockpit_repository::require_portable_source_observation_admission(
        &wire.repository_root,
        &wire.work_item_id,
        runtime,
        wire.task_caller.as_ref(),
    )
    .context("admit controlled source observation")?;
    let revision = before
        .head
        .as_deref()
        .filter(|v| source_revision_valid(v))
        .ok_or(SourceBridgeError::HostHeadUnavailable)?
        .to_owned();
    let mut reader = SourceHttpsReader::bootstrap(deadline, &revision, &selector, runtime)?;
    let mut observation =
        read_source_observation(&mut reader, &selector, &wire.input_digest, &wire.nonce)?;
    reader.budget.remaining()?;
    let after = cockpit_repository::require_portable_source_observation_admission(
        &wire.repository_root,
        &wire.work_item_id,
        runtime,
        wire.task_caller.as_ref(),
    )
    .context("recheck controlled source observation")?;
    if before.head != after.head {
        return Err(SourceBridgeError::RunBinding.into());
    }
    reader.budget.remaining()?;
    observation.audit_reads.splice(0..0, reader.bootstrap_audit);
    let audit = serde_json::to_vec(
        &observation.audit_summary(reader.budget.requests, reader.budget.bytes),
    )?;
    if audit.len() > SOURCE_AUDIT_BYTES {
        return Err(SourceBridgeError::TotalByteLimit.into());
    }
    eprintln!("{}", std::str::from_utf8(&audit)?);
    reader.budget.remaining()?;
    Ok(observation.worker_reply())
}

#[cfg(test)]
mod source_worker_boundary_tests {
    use super::*;
    use serde_json::json;
    fn runtime() -> RuntimeContext {
        RuntimeContext {
            runtime_version: "1.0.1".into(),
            protocol_version: 1,
            runtime_digest: cockpit_core::Digest::sha256_bytes(
                b"actual worker test executable identity",
            ),
        }
    }
    fn wire() -> serde_json::Value {
        json!({"schemaVersion":1,"nonce":"fresh-consumer-nonce",
            "inputDigest":cockpit_core::Digest::sha256_bytes(b"portable bytes"),
            "parameters":{"runId":11,"attempt":2,"jobId":21,"artifactId":41},
            "repositoryRoot":std::env::temp_dir(),"workItemId":"WI-WORKER-TEST","taskCaller":null,
            "observerToolDigest":runtime().runtime_digest})
    }
    #[test]
    fn source_worker_wire_has_no_agent_selected_policy_endpoint_or_tool() {
        source_worker_wire(&wire().to_string(), &runtime())
            .expect("strict four-selector wire shape");
        for key in [
            "tool",
            "path",
            "endpoint",
            "verified",
            "hostConfiguration",
            "policy",
            "config",
        ] {
            let mut v = wire();
            v["parameters"][key] = json!("forged");
            assert!(
                source_worker_wire(&v.to_string(), &runtime()).is_err(),
                "{key}"
            );
            let mut v = wire();
            v[key] = json!("forged");
            assert!(
                source_worker_wire(&v.to_string(), &runtime()).is_err(),
                "outer {key}"
            );
        }
        let mut v = wire();
        v["observerToolDigest"] = json!(cockpit_core::Digest::sha256_bytes(b"another code hash"));
        assert!(matches!(
            source_worker_wire(&v.to_string(), &runtime()),
            Err(SourceBridgeError::WorkerRequestInvalid)
        ));
    }
    #[test]
    fn source_http_budget_refuses_extra_reads_expired_deadline_and_byte_overflow() {
        let mut b = SourceHttpBudget {
            deadline: std::time::Instant::now() + std::time::Duration::from_secs(120),
            requests: 0,
            bytes: 0,
        };
        for _ in 0..16 {
            assert!(b.begin_read().unwrap() <= std::time::Duration::from_secs(20));
        }
        assert_eq!(b.begin_read().unwrap_err(), SourceBridgeError::RequestLimit);
        assert_eq!(b.requests, 16);
        b.bytes = SOURCE_TOTAL_BYTES;
        assert_eq!(
            b.add_bytes(1).unwrap_err(),
            SourceBridgeError::TotalByteLimit
        );
        b.deadline = std::time::Instant::now();
        assert_eq!(b.remaining().unwrap_err(), SourceBridgeError::Deadline);
    }
    #[test]
    fn source_actual_body_reader_checks_caps_before_allocation_and_preserves_exact_bytes() {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        let mut budget = SourceHttpBudget {
            deadline,
            requests: 1,
            bytes: 0,
        };
        let mut body = std::io::Cursor::new(b"abc");
        let error = bounded_source_body(&mut body, Some(4), 3, &mut budget, deadline)
            .err()
            .expect("declared cap refusal");
        assert_eq!(error, SourceBridgeError::MetadataLimitOrTruncated);
        assert_eq!(body.position(), 0);
        let got = bounded_source_body(&mut body, Some(3), 3, &mut budget, deadline).unwrap();
        assert_eq!(got.bytes, b"abc");
        assert_eq!(budget.bytes, 3);
        let mut over = std::io::Cursor::new(b"abcd");
        assert!(matches!(
            bounded_source_body(&mut over, None, 3, &mut budget, deadline),
            Err(SourceBridgeError::MetadataLimitOrTruncated)
        ));
        let mut short = std::io::Cursor::new(b"ab");
        assert!(matches!(
            bounded_source_body(&mut short, Some(3), 3, &mut budget, deadline),
            Err(SourceBridgeError::Transport)
        ));
    }
}
