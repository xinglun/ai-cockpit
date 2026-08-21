# Agent Discovery / Adapter Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement WI-39 as an explicit, repository-bound, verifiable, and reversible Agent Discovery / Adapter layer while keeping Runtime and Repository Protocol state isolated.

**Architecture:** Add a small `cockpit-agent` crate that owns provider-neutral discovery, managed-section ownership, safe mutation, doctor state derivation, and exit-code mapping. Keep `cockpit-repository` responsible for attachment and repository facts, keep `cockpit-cli` as the direct Core caller, and leave MCP/global Agent configuration outside the normal install path.

**Tech Stack:** Rust 1.94, Cargo workspace, `serde`/`serde_json`, `sha2`, `cap-std`/`cap-fs-ext` for handle-relative filesystem operations, Clap CLI, existing `cockpit-protocol` and `cockpit-git` crates, and Rust integration tests.

**Spec:** `docs/superpowers/specs/2026-08-21-agent-discovery-adapter-layer-design.md`

## Global Constraints

- Every Agent command requires explicit `--repo`; no process-global current repository is introduced.
- `attach` remains Layer 2 and does not modify `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursor/**`, `.codex/**`, or home/global MCP configuration.
- `.ai/agent-interface.json` contains discovery facts only; unknown fields and unsupported interface versions fail closed.
- `agent install` is the only normal write entry point for adapter content.
- Managed sections are digest-owned; unrelated user bytes are preserved exactly.
- Duplicate markers, malformed markers, repository mismatch, symlink/reparse targets, and changed managed content fail closed.
- `--provider auto` detects safe candidates and never means “edit every discovered Agent file.”
- CLI calls the Core directly; MCP is optional and never a prerequisite for CLI operation.
- No `--force` detach or destructive recovery path is added in WI-39.
- No public Release, Homebrew/tap change, or provider-global configuration change is in scope.
- Existing workspace format, Clippy, package tests, Windows runtime, and locked V1 Oracle gates must remain green.

---

### Task 1: Version the discovery and ownership protocol types

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs` near `AgentInterfaceManifest`.
- Modify: `crates/cockpit-protocol/tests/strict.rs` or the existing protocol test module.
- Modify: `Cargo.toml` only if the protocol types require a workspace dependency already absent.

**Interfaces:**
- Consumes: current strict `AgentInterfaceManifest`, `RepositoryConfig`, and `Digest`.
- Produces: strict serializable types used by `cockpit-agent`:
  `AgentProvider`, `AgentInterfaceManifest`, `AgentInterface`,
  `AgentAdapterCompatibility`, `ManagedAdapterRecord`, and `AgentDoctorReport`.

- [ ] **Step 1: Write failing protocol tests.** Add tests named
  `agent_manifest_rejects_unknown_fields`, `agent_manifest_rejects_unsupported_interface_version`,
  `managed_adapter_record_round_trips_canonical_json`, and
  `doctor_report_rejects_unknown_fields`. The tests must parse an unknown top-level field,
  a wrong `schemaVersion`, and an ownership record with an unknown provider, and assert an error.

- [ ] **Step 2: Run the protocol tests and verify the expected failures.**

  Run:
  ```bash
  cargo test -p cockpit-protocol --all-targets -- strict
  ```

  Expected: the new types or enum variants are missing, so the tests fail for the intended protocol reason rather than because of a fixture typo.

- [ ] **Step 3: Implement the strict wire types.** Keep the repository protocol major at `1` and add an explicit `interfaceVersion`/compatibility marker inside the discovery manifest. Use `#[serde(rename_all = "camelCase", deny_unknown_fields)]` on every wire type. Represent providers as a closed enum with the serialized values `generic-agents-md`, `codex`, `claude`, `gemini`, and `cursor`. Represent an ownership record with `provider`, `adapterVersion`, `target`, `mode`, `repositoryId`, and `installedDigest`.

- [ ] **Step 4: Run the focused protocol tests and all protocol tests.**

  Run:
  ```bash
  cargo test -p cockpit-protocol --all-targets -- --test-threads=1
  cargo clippy -p cockpit-protocol --all-targets --all-features -- -D warnings
  ```

  Expected: all protocol tests pass, including rejection of unknown fields and unsupported interface versions.

- [ ] **Step 5: Commit the protocol boundary.**

  ```bash
  git add crates/cockpit-protocol/src/lib.rs crates/cockpit-protocol/tests
  git commit -m "feat: define strict agent discovery protocol types"
  ```

### Task 2: Create the provider-neutral `cockpit-agent` crate

**Files:**
- Modify: `Cargo.toml` workspace members and dependencies.
- Create: `crates/cockpit-agent/Cargo.toml`.
- Create: `crates/cockpit-agent/src/lib.rs`.
- Create: `crates/cockpit-agent/tests/context.rs`.

**Interfaces:**
- Consumes: `cockpit-protocol` types from Task 1 and `cockpit-git::GitRepository`.
- Produces: `AgentRepositoryContext`, `AgentProvider`, `AgentError`, `load_agent_context`,
  `canonical_manifest_path`, `repository_id_from_manifest`, and `sha256_file`.

- [ ] **Step 1: Write failing context tests.** Add `manifest_parent_resolves_repository_context`,
  `missing_manifest_is_unattached`, `manifest_repository_mismatch_fails_closed`, and
  `manifest_symlink_is_rejected`. Each fixture must create a temporary Git repository, attach it
  with the existing CLI/repository API, then exercise the new loader. The symlink test must be
  `#[cfg(unix)]` and the Windows equivalent must reject a reparse target when compiled there.

- [ ] **Step 2: Run the new crate test and verify it fails because the crate/API does not exist.**

  ```bash
  cargo test -p cockpit-agent --all-targets -- --test-threads=1
  ```

- [ ] **Step 3: Add the crate and implement `load_agent_context`.** Discover the Git root from the explicit `root` argument, locate `.ai/agent-interface.json` relative to that root, open it with no-follow semantics, validate `schemaVersion`, `interfaceVersion`, protocol version, `rootBinding`, and the expected repository ID, and return a context containing the canonical root, manifest path, repository ID, and parsed manifest. Do not fall back to `cwd`, an arbitrary `AGENTS.md`, or a path-derived identity.

- [ ] **Step 4: Implement bounded streaming SHA-256.** Hash target files through a bounded reader, preserve the canonical `sha256:<hex>` form, reject oversized manifest/ownership files before allocating unbounded memory, and expose the helper to later ownership tasks.

- [ ] **Step 5: Run focused context tests, cross-target check, and Clippy.**

  ```bash
  cargo test -p cockpit-agent --test context -- --test-threads=1
  cargo check -p cockpit-agent --tests --target x86_64-pc-windows-msvc
  cargo clippy -p cockpit-agent --all-targets --all-features -- -D warnings
  ```

- [ ] **Step 6: Commit the crate boundary.**

  ```bash
  git add Cargo.toml crates/cockpit-agent
  git commit -m "feat: add repository-bound agent adapter crate"
  ```

### Task 3: Implement read-only provider detection and plans

**Files:**
- Modify: `crates/cockpit-agent/src/lib.rs` or split into `src/detect.rs` and `src/plan.rs` if the module exceeds one responsibility.
- Create: `crates/cockpit-agent/tests/detection.rs`.

**Interfaces:**
- Consumes: `AgentRepositoryContext` and `AgentProvider`.
- Produces: `DetectionResult`, `AdapterStatus`, `AdapterOperation`, `AdapterPlan`,
  `detect_providers`, and `plan_install`.

- [ ] **Step 1: Write failing detection tests.** Add `detection_is_read_only`,
  `auto_lists_only_safe_surfaces`, `provider_target_is_repository_relative`,
  `duplicate_marker_is_a_conflict`, and `unsupported_host_surface_is_not_selected`.
  Assert that detection changes no bytes and that an existing `AGENTS.md`, `CLAUDE.md`,
  `GEMINI.md`, `.cursor/rules`, or `.codex` is reported as a fact/plan candidate rather than
  modified.

- [ ] **Step 2: Run the detection tests and verify they fail before implementation.**

  ```bash
  cargo test -p cockpit-agent --test detection -- --test-threads=1
  ```

- [ ] **Step 3: Implement the closed provider registry.** Map providers to repository-local target surfaces and a shared managed-section format. `generic-agents-md`, `codex`, `claude`, and `gemini` may share `AGENTS.md` only when the target is unambiguous; `cursor` uses a repository-local `.cursor/rules/ai-cockpit.md` target. Detection must report `not_installed`, `installed`, `modified`, `conflict`, or `unsupported` without writing.

- [ ] **Step 4: Implement `--provider auto` planning.** Sort candidates deterministically by provider name, include target path, current digest, expected operation, repository ID, and every conflict, and make a plan containing any unresolved conflict non-executable. Never select a home-directory or global MCP target.

- [ ] **Step 5: Run focused tests and verify byte stability.**

  ```bash
  cargo test -p cockpit-agent --test detection -- --test-threads=1
  cargo test -p cockpit-repository --test repository_context -- --test-threads=1
  ```

- [ ] **Step 6: Commit the read-only layer.**

  ```bash
  git add crates/cockpit-agent
  git commit -m "feat: add read-only agent provider detection"
  ```

### Task 4: Implement owned managed-section installation

**Files:**
- Modify: `crates/cockpit-agent/src/lib.rs` or `src/install.rs` and `src/ownership.rs`.
- Create: `crates/cockpit-agent/tests/install.rs`.

**Interfaces:**
- Consumes: `AgentPlan`, `ManagedAdapterRecord`, bounded digest helpers, and a loaded repository context.
- Produces: `install_adapter`, `AdapterReceipt`, `managed_block`, and ownership persistence under `.ai/adapters/<provider>.json`.

- [ ] **Step 1: Write failing install tests.** Add `install_creates_only_owned_managed_section`,
  `repeated_install_is_byte_stable`, `install_preserves_unrelated_bytes`,
  `repository_a_and_b_have_independent_ownership_records`,
  `install_rejects_duplicate_or_malformed_markers`, and `install_rejects_symlink_target`.
  Capture a complete tree digest before/after and assert that only the selected target and the
  corresponding `.ai/adapters` record change.

- [ ] **Step 2: Run the tests and verify the expected missing-operation failures.**

  ```bash
  cargo test -p cockpit-agent --test install -- --test-threads=1
  ```

- [ ] **Step 3: Implement managed markers and content.** Emit a provider/version/repository-bound block that points to `.ai/agent-interface.json` and says “Do not infer AI Cockpit state from this file.” Do not include governance rules, current Work Item state, commands containing `.` as identity, or provider-global instructions.

- [ ] **Step 4: Implement safe atomic writes.** Use no-follow, handle-relative operations where available, reject leaf symlinks/reparse points, write the ownership record and managed target atomically, and fail closed if either publication cannot be completed. Preserve all non-owned bytes exactly and make repeated installation a no-op.

- [ ] **Step 5: Run focused install tests on Unix and cross-check Windows compilation.**

  ```bash
  cargo test -p cockpit-agent --test install -- --test-threads=1
  cargo check -p cockpit-agent --tests --target x86_64-pc-windows-msvc
  ```

- [ ] **Step 6: Commit the installation layer.**

  ```bash
  git add crates/cockpit-agent
  git commit -m "feat: install repository-owned agent adapter sections"
  ```

### Task 5: Implement verify, detach, repair, state machine, and exit codes

**Files:**
- Modify: `crates/cockpit-agent/src/lib.rs` or split into `src/doctor.rs` and `src/lifecycle.rs`.
- Create: `crates/cockpit-agent/tests/doctor.rs`.

**Interfaces:**
- Consumes: `AgentRepositoryContext`, ownership records, detection results, and installed-section digests.
- Produces: `doctor`, `detach_adapter`, `repair_adapter`, `AgentDoctorReport`, `AgentState`, and `AgentExitCode`.

- [ ] **Step 1: Write failing lifecycle tests.** Add `doctor_reports_unattached_and_attached`,
  `doctor_requires_matching_repository_probe_for_verified`, `doctor_reports_degraded_without_mcp`,
  `modified_section_blocks_detach`, `unchanged_section_detaches_only_owned_bytes`,
  `repair_refuses_conflict_without_force`, and `exit_codes_match_state`.

- [ ] **Step 2: Run the lifecycle tests and verify they fail before implementation.**

  ```bash
  cargo test -p cockpit-agent --test doctor -- --test-threads=1
  ```

- [ ] **Step 3: Implement derived state.** Derive `UNATTACHED`, `ATTACHED`, `DISCOVERY_AVAILABLE`,
  `ADAPTER_INSTALLED`, `CONNECTED`, `VERIFIED`, `DEGRADED`, and `CONFLICT` from current facts;
  never trust a cached state string. A repository-bound probe must compare the expected ID before
  returning `VERIFIED`. Map states to exit codes 0, 1, 2, 3, and 4 exactly as specified.

- [ ] **Step 4: Implement fail-closed detach and repair.** Compare current managed-section digest with the ownership record. Remove only an unchanged owned block and its record; refuse modified, duplicated, missing, malformed, or mismatched content with a conflict report. Do not add `--force`.

- [ ] **Step 5: Run focused lifecycle tests, all agent tests, and Clippy.**

  ```bash
  cargo test -p cockpit-agent --all-targets -- --test-threads=1
  cargo clippy -p cockpit-agent --all-targets --all-features -- -D warnings
  ```

- [ ] **Step 6: Commit the lifecycle layer.**

  ```bash
  git add crates/cockpit-agent
  git commit -m "feat: add fail-closed agent doctor lifecycle"
  ```

### Task 6: Expose explicit CLI commands without global state

**Files:**
- Modify: `crates/cockpit-cli/src/main.rs` command enums, dispatch, and process exit handling.
- Create: `crates/cockpit-cli/tests/agent.rs`.
- Modify: `crates/cockpit-cli/Cargo.toml` to depend on `cockpit-agent`.

**Interfaces:**
- Consumes: `cockpit_agent::{detect_providers, install_adapter, doctor, repair_adapter, detach_adapter}`.
- Produces: `agent list`, `agent install`, `agent doctor`, `agent repair`, and `agent detach`, each with required `--repo`; `--json` is supported by `agent doctor`.

- [ ] **Step 1: Write failing CLI tests.** Add `agent_commands_require_repo`,
  `agent_install_and_doctor_are_repository_bound`, `agent_install_does_not_touch_global_files`,
  `agent_detach_refuses_modified_content`, `agent_doctor_json_reports_state_and_safe_actions`,
  and `cli_agent_operation_does_not_require_mcp`.

- [ ] **Step 2: Run the CLI tests and verify Clap/API failures.**

  ```bash
  cargo test -p cockpit-cli --test agent -- --test-threads=1
  ```

- [ ] **Step 3: Add the nested command model.** Define `AgentCommand::{List,Install,Doctor,Repair,Detach}` with required `PathBuf repo`; define `AgentProviderArg` conversion to the protocol enum. Do not add a default repository, environment-based current project, or implicit `cwd` fallback.

- [ ] **Step 4: Dispatch directly to `cockpit-agent`.** Print human-readable facts and conflicts by default; print strict JSON for `agent doctor --json`. Return the mapped `AgentExitCode` instead of turning every state into a generic success. Ensure the existing top-level `doctor` behavior remains compatible or is explicitly renamed before adding the nested command.

- [ ] **Step 5: Run focused CLI tests and inspect help.**

  ```bash
  cargo test -p cockpit-cli --test agent -- --test-threads=1
  cargo run -p cockpit-cli -- agent --help
  cargo run -p cockpit-cli -- agent doctor --help
  ```

- [ ] **Step 6: Commit the CLI surface.**

  ```bash
  git add crates/cockpit-cli/Cargo.toml crates/cockpit-cli/src/main.rs crates/cockpit-cli/tests/agent.rs
  git commit -m "feat: expose explicit agent adapter commands"
  ```

### Task 7: Keep MCP optional and verify repository isolation

**Files:**
- Modify: `crates/cockpit-mcp/src/lib.rs` only if an agent doctor/list tool is explicitly exposed; otherwise record the no-change decision in tests/docs.
- Create or modify: `crates/cockpit-cli/tests/mcp.rs` for CLI/MCP boundary coverage.
- Create: `crates/cockpit-agent/tests/isolation.rs`.

**Interfaces:**
- Consumes: direct CLI/Core agent APIs and explicit repository manifests.
- Produces: proof that MCP absence does not block CLI and that no MCP/global settings are modified.

- [ ] **Step 1: Write failing boundary tests.** Add `cli_agent_commands_work_without_mcp_process`,
  `mcp_server_does_not_write_home_configuration`, and
  `parallel_agent_operations_keep_repository_ids_and_records_separate`.

- [ ] **Step 2: Run the boundary tests and verify the missing isolation behavior.**

  ```bash
  cargo test -p cockpit-cli --test mcp -- --test-threads=1
  cargo test -p cockpit-agent --test isolation -- --test-threads=1
  ```

- [ ] **Step 3: Implement only the required no-change or repository-local MCP behavior.** Do not add global configuration writes. If a local MCP export is exposed, require a provider argument, emit a plan first, and record any repository-local ownership; otherwise keep MCP unchanged and make the tests assert that fact.

- [ ] **Step 4: Run all MCP, agent, repository-context, and CLI focused tests.**

  ```bash
  cargo test -p cockpit-mcp --all-targets -- --test-threads=1
  cargo test -p cockpit-agent --all-targets -- --test-threads=1
  cargo test -p cockpit-repository --test repository_context -- --test-threads=1
  cargo test -p cockpit-cli --all-targets -- --test-threads=1
  ```

- [ ] **Step 5: Commit the boundary verification.**

  ```bash
  git add crates/cockpit-agent crates/cockpit-cli/tests crates/cockpit-mcp
  git commit -m "test: prove agent and MCP repository isolation"
  ```

### Task 8: Synchronize documentation, CI, and final acceptance

**Files:**
- Modify: `README.md`, `README.zh-CN.md`, `README.ja.md`.
- Modify: `docs/architecture.md`, `docs/architecture.zh-CN.md`, `docs/architecture.ja.md`.
- Modify: `docs/capabilities.md`, `docs/capabilities.zh-CN.md`, `docs/capabilities.ja.md`.
- Modify: `docs/reference/commands.md`, `docs/reference/commands.zh-CN.md`, `docs/reference/commands.ja.md`.
- Modify: `docs/reference/configuration.md`, `docs/reference/configuration.zh-CN.md`, `docs/reference/configuration.ja.md`.
- Modify: `.github/workflows/ci.yml` to include `cockpit-agent` in the serial package list.
- Modify: `docs/work-items/WI-39.md`, `docs/work-items/WI-39.zh-CN.md`, and `docs/work-items/WI-39.ja.md` after implementation evidence exists.

**Interfaces:**
- Consumes: final CLI output, state/exit-code schema, and the WI-39 acceptance criteria.
- Produces: reader-first three-language instructions for install/detect/doctor/repair/detach, explicit separation of Runtime/attach/adapter/skill/connection/compliance, and hosted evidence bound to the final commit.

- [ ] **Step 1: Write documentation parity tests/checks.** Extend the existing CI counterpart check to require WI-39 files and add a shell check for all five Agent commands, the explicit `--repo` flag, no `--force`, and the three-layer distinctions in all languages.

- [ ] **Step 2: Update user-facing docs.** Add a short first-use route:
  `ai-cockpit attach --repo .` → `ai-cockpit agent install --repo . --provider auto` →
  `ai-cockpit agent doctor --repo .`. Explain that discovery, installation, connection,
  verification, and compliance are distinct. Keep `cockpit.toml` as TOML and state that
  `agent-interface.json` is facts, not prompt or policy.

- [ ] **Step 3: Add `cockpit-agent` to the hosted package list and run all local gates.**

  ```bash
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  for package in cockpit-core cockpit-evidence cockpit-verification cockpit-git cockpit-knowledge cockpit-protocol cockpit-agent cockpit-repository cockpit-mcp cockpit-cli cockpit-release; do
    cargo test -p "$package" --all-targets -- --test-threads=1
  done
  cargo check -p cockpit-agent --tests --target x86_64-pc-windows-msvc
  git diff --check
  ```

- [ ] **Step 4: Run the locked V1 Oracle and existing hosted workflow.** Bind the run to the exact WI-39 head and require quality, Windows runtime, V1 Oracle, trilingual counterparts, and release policy checks to pass. Do not treat a local run as hosted evidence.

- [ ] **Step 5: Update the three WI-39 Outcome sections only after the bound hosted receipt passes.** Record the exact run URL/ID, final commit, test scope, and any explicitly deferred external/provider-global work.

- [ ] **Step 6: Commit documentation and acceptance evidence.**

  ```bash
  git add README* docs .github/workflows/ci.yml
  git commit -m "docs: complete WI-39 agent discovery adapter acceptance"
  ```

## Final Review Checklist

- [ ] Every WI-39 acceptance criterion maps to a test, CLI receipt, or hosted job.
- [ ] `rg -n "TODO|TBD|--force|current project|global MCP"` has no unintended implementation claim or undocumented escape hatch.
- [ ] Unknown manifest/ownership fields fail closed.
- [ ] A/B repository operations are parallel-safe and leave separate records.
- [ ] `attach` and `--provider auto` produce no Agent/global writes.
- [ ] Modified managed content cannot be detached or repaired automatically.
- [ ] `agent doctor` derives state from current facts and returns the documented exit code.
- [ ] CLI remains usable without MCP.
- [ ] All three language docs describe the same commands and boundaries.
- [ ] Local and hosted quality, Windows runtime, and V1 Oracle receipts are bound to the final implementation commit.
