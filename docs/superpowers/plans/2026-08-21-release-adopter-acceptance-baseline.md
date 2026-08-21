# Release Adopter Acceptance Baseline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a reproducible post-release adopter acceptance baseline using only immutable public Release artifacts.

**Architecture:** A Bash harness downloads and pins one public Release binary, creates an isolated temporary Cargo adopter, runs the repository and Work Item lifecycle, and emits a versioned JSON evidence directory. A tag-only GitHub Actions job runs the same harness after `publish` and `publish_handoff`; it is explicitly post-publication and never rewrites Release truth.

**Tech Stack:** Bash, curl, jq, Git, Cargo test inside the temporary adopter, SHA-256, GitHub Actions artifacts, and the released `ai-cockpit` binary.

**Spec:** `docs/superpowers/specs/2026-08-21-release-adopter-acceptance-baseline-design.md`

## Global Constraints

- The harness must never use `cargo build`, `cargo run`, a workspace binary, or a local `target` binary to obtain AI Cockpit.
- Every Runtime invocation must use the absolute extracted binary whose digest is recorded in `runtime.json`.
- The public Release must be verified as non-draft, non-prerelease, and tag-matching before download.
- `acceptance.json` must preserve `releasePublished: true` when a post-release step fails.
- `SHA256SUMS` must cover every emitted evidence file except itself.
- `HOME`, `XDG_CONFIG_HOME`, `TMPDIR`, `CARGO_HOME`, and the adopter repository must be unique per run.
- `first-adopter-smoke` must remain `not_ready` with human-owned fields empty or `unknown`.
- A stable repeated verify must show at least one reused node and zero spawned processes on the second run.
- No Runtime command, Repository Protocol type, global Agent/MCP setting, or second technology stack is added.

---

### Task 1: Add the acceptance harness contract and fixture assertions

**Files:**
- Create: `tests/release/adopter_acceptance.sh`
- Create: `tests/release/adopter_acceptance_test.sh`
- Modify: `tests/release/workflow_policy.sh`

**Interfaces:**
- `adopter_acceptance.sh --repository OWNER/REPOSITORY --tag vX.Y.Z --target TARGET --output DIRECTORY`
- The script writes raw evidence named by the design and exits nonzero on any failed assertion.
- The test wrapper runs the script against a local fixture HTTP server or a public tag and asserts the evidence shape without using repository source binaries.

- [ ] **Step 1: Define argument parsing and command prerequisites**
  Require `bash`, `curl`, `jq`, `git`, `cargo`, `tar`, and a SHA-256 implementation. Reject missing repository, tag, target, or output values.
- [ ] **Step 2: Implement public Release and archive verification**
  Query `https://api.github.com/repos/${repository}/releases/tags/${tag}`, require `draft == false`, `prerelease == false`, and matching `tag_name`, download only the exact target archive, `release-manifest.json`, and `SHA256SUMS`, and compare the archive digest to the manifest/checksum.
- [ ] **Step 3: Pin the extracted binary**
  Extract into the run work directory, assert the binary is executable and outside the source checkout, compute `sha256:<64 lowercase hex>`, and route all later calls through that absolute path.
- [ ] **Step 4: Add the failure finalizer**
  Register an EXIT trap that writes `acceptance.json` for both success and failure, preserving the release API truth, completed step states, failure reason, and timestamps.
- [ ] **Step 5: Run the focused shell assertions**
  Run `bash -n tests/release/adopter_acceptance.sh` and the fixture wrapper; assert a missing archive, mismatched digest, and source fallback each fail closed.
- [ ] **Step 6: Commit the harness contract**
  Commit the script and focused shell tests with `test: add post-release adopter acceptance harness`.

### Task 2: Implement isolated adopter lifecycle and evidence capture

**Files:**
- Modify: `tests/release/adopter_acceptance.sh`
- Test: `tests/release/adopter_acceptance_test.sh`

**Interfaces:**
- `run_runtime <args...>` invokes only the pinned binary with sanitized stable environment variables.
- `capture_step <name> <command...>` stores stdout as the named evidence file and records state/reason.
- `finalize_evidence` writes the acceptance summary and directory `SHA256SUMS`.

- [ ] **Step 1: Create isolated directories and baseline snapshots**
  Create unique HOME, XDG_CONFIG_HOME, TMPDIR, CARGO_HOME, runtime, adopter, status, and output directories; record empty HOME/XDG manifests before Runtime use.
- [ ] **Step 2: Create and commit the fresh Cargo adopter**
  Run `cargo new --lib`, make one initial Git commit, and write `repository.json` with the eventual repositoryId and initial head.
- [ ] **Step 3: Run attach/profile/Agent evidence**
  Execute `attach`, `profile confirm`, create one repository-local `AGENTS.md`, then capture `agent-list.json`, `agent-install.json`, and `agent-doctor.json`. Assert doctor is `VERIFIED` and its runtime identity matches `runtime.json`.
- [ ] **Step 4: Preserve `first-adopter-smoke` as not ready**
  Run `work-item new --id first-adopter-smoke --mode code`, copy its contract, and assert `state == not_ready`, `intent == ""`, `scope == []`, `acceptanceCriteria == []`, and `authority == "unknown"`.
- [ ] **Step 5: Run a complete lifecycle Work Item**
  Start `release-adopter-lifecycle` with explicit intent, goal, scope, authority, acceptance, and evidence; make one deterministic in-scope test change; capture start, checkpoint, preflight, verify, finish, archive, close, contract, outcome, and verification evidence.
- [ ] **Step 6: Bind and verify evidence reuse**
  Commit adopter governance state, run the pinned Runtime twice with the same sanitized environment, require first execution and second `nodesReused > 0`, `nodesExecuted == 0`, and `processesSpawned == 0`, then copy both JSON outputs and the referenced reuse files.
- [ ] **Step 7: Prove isolation and finalize hashes**
  Compare HOME/XDG before and after, assert the source checkout has no `.ai/`, build `acceptance.json`, and generate `SHA256SUMS` over all evidence except itself.
- [ ] **Step 8: Run the adopter lifecycle test**
  Run the script against `v0.1.1` on the local host and assert all required states and identity bindings.

### Task 3: Add the post-publication GitHub Actions job

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `tests/release/workflow_policy.sh`

**Interfaces:**
- Job `adopter_acceptance` runs only on tag pushes and needs `publish` and `publish_handoff`.
- The job invokes `tests/release/adopter_acceptance.sh --repository "$GITHUB_REPOSITORY" --tag "$GITHUB_REF_NAME" --target x86_64-unknown-linux-gnu --output "$RUNNER_TEMP/release-adopter-acceptance"`.
- The job uploads the output directory with `if-no-files-found: error` on success and `if: always()` for failure evidence.

- [ ] **Step 1: Add the tag-only job and permissions**
  Use `contents: read`, checkout the script, install the pinned Rust toolchain for the adopter's `cargo test`, and do not download candidate artifacts.
- [ ] **Step 2: Bind the job to public Release truth**
  Pass the exact tag and repository from GitHub context; the script itself must query and download the public Release.
- [ ] **Step 3: Upload success and failure evidence**
  Upload the acceptance directory with a stable artifact name containing tag and run identity, preserving `acceptance.json` and `SHA256SUMS` even when the script fails.
- [ ] **Step 4: Extend workflow policy tests**
  Assert the job is post-publication, tag-only, depends on `publish` and `publish_handoff`, invokes the harness, and does not invoke `cargo build`/`cargo run` to obtain the Runtime.
- [ ] **Step 5: Run workflow policy and YAML checks**
  Run `bash tests/release/workflow_policy.sh .github/workflows/release.yml` and the repository's source-quality checks.

### Task 4: Document the baseline and close WI-40

**Files:**
- Create: `docs/work-items/WI-40.md`
- Create: `docs/work-items/WI-40.zh-CN.md`
- Create: `docs/work-items/WI-40.ja.md`
- Modify: `docs/release/distribution.md`
- Modify: `docs/release/distribution.zh-CN.md`
- Modify: `docs/release/distribution.ja.md`
- Modify: `docs/reference/commands.md`
- Modify: `docs/reference/commands.zh-CN.md`
- Modify: `docs/reference/commands.ja.md`

**Interfaces:**
- Docs describe the script as post-release acceptance, not a Runtime command or pre-publication gate.
- Docs link the evidence layout and state that a failed adopter acceptance never changes `releasePublished`.

- [ ] **Step 1: Record WI-40 scope and acceptance criteria in three languages**
  Include the immutable Release, runtime identity, isolation, not-ready skeleton, reuse, lifecycle, and artifact checksum requirements.
- [ ] **Step 2: Add adopter-facing invocation documentation**
  Show the exact script invocation and explain that second-tech-stack coverage is a separate Work Item.
- [ ] **Step 3: Review trilingual parity and links**
  Compare headings, commands, evidence names, and failure semantics across English, Chinese, and Japanese.
- [ ] **Step 4: Run complete verification**
  Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-targets --all-features`, the workflow policy test, the adopter script on `v0.1.1`, and `git diff --check`.
- [ ] **Step 5: Commit, push, hosted-check, merge, and clean**
  Commit the closed WI-40 evidence bundle, open one PR, wait for required checks, merge, prove the exact remote branch is absent, delete only the local WI-40 branch, and verify a clean `main`.
