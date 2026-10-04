# Hosted package coverage consumer correction implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Accept a genuine, complete Cargo workspace test receipt from a multi-command Work Item without rerunning its packages, while rejecting ambiguous or forged coverage.

**Architecture:** The CLI continues to produce the existing typed `VerificationCoverageManifest`, but chooses the unique complete workspace `cargo test` command when present rather than the first workspace command. The CI consumer validates that formal manifest against the current Contract's command index, Cargo metadata, execution records, and reuse accounting. The process-observer test fixture clears only inherited hosted-evidence variables on its own fake-package child calls.

**Tech Stack:** Rust Runtime/CLI, existing strict JSON receipt, Bash/Python CI scripts, Cargo, GitHub Actions.

**Spec:** `.ai/work-items/active/WI-1060-hosted-coverage-consumer.contract.json` (current Contract digest `sha256:f9acb50e5c553696d4a2aa26ca04f10c649d6c9bc6fc15690f95f67c196e5080`; initial reviewed planning content was `sha256:5606e3aa58bf8fb1b3bb2bb0db9d7a997a6e89bb977abd048338ee1547d6e099` before Runtime activation).

## Global constraints

- Preserve the failed PR #1011 run 37136421952 and its passing candidate verification/Contract-gate receipts; do not relabel the overall run as green.
- Do not change the strict receipt schema, invent a signing credential, or widen production process-observer locking behavior.
- `VerificationCoverageManifest` remains generic. Only the workspace-package-test consumer requires a unique complete unfiltered Cargo test command.
- Use the repository's Runtime CLI, Cargo, and canonical CI gate; there is no Makefile. Work on `codex/wi-1060-hosted-coverage-consumer` from `c026e59c`.
- Keep WI-1059/PR #1011 untouched until this correction is independently reviewed, merged, and incorporated through a fresh supported verification path.
- Amendment `wi1060-stage-local-before-formal-verify-20261004` changes only the fifth scenario's expected result, verification plan, and description: it records pre-verification local gates first. Formal Runtime verification and hosted exact-head CI remain unchanged mandatory acceptance, never inferred from that local scenario.

## Stage-boundary amendment: before/after field audit

The supported `work-item amend --request` already recorded this one change as amendment sequence 3, from Contract `sha256:fa25c2c122aef0d75e5cd5c7f741b75ae76b3b3bb71c30c02ca3bcde086a833c` to `sha256:f9acb50e5c553696d4a2aa26ca04f10c649d6c9bc6fc15690f95f67c196e5080`. Its append-only receipt is `.ai/evidence/WI-1060-hosted-coverage-consumer.contract-amendments/00000003.committed.json`; do not repeat or erase it.

| Field | Before | After | Effect |
| --- | --- | --- | --- |
| `/scenarioCoverage/4/expected` | Local fmt, clippy, workspace tests and scripts plus hosted candidate verify, coverage gate, and exact-head CI pass before merge | Only the five declared local checks on the frozen candidate are the scenario's pre-verify expected result; formal verify and hosted exact-head CI remain mandatory acceptance | Separates locally observable prerequisite from later gates; does not assert those later gates passed |
| `/scenarioCoverage/4/verificationPlan` | Formal Runtime verify, receipt/CI artifact inspection, independent review, exact-head merge | Run and inspect the five local commands first; then refresh preflight and perform formal Runtime verify and hosted exact-head PR CI under unchanged acceptance | Changes ordering, not the set of checks |
| `/scenarioCoverage/4/description` | Absent | States the fifth scenario covers the local prerequisite only; its historical title still mentions later gates | Prevents the local result being read as hosted success |
| `/scenarioCoverage/4/status` and evidence | `unverified`, no Contract evidence | `unverified`, no Contract evidence; Summary fifth scenario also remains `unverified` | No premature success claim |
| `/acceptanceCriteria` | Six requirements, including full declared verification, independent review, and exact-head PR CI before merge | Byte-for-byte unchanged, six requirements | Product and hosted acceptance retained |
| `/verification` | Five commands: fmt, clippy, workspace Cargo tests, workspace coverage fixture, observer fixture | Byte-for-byte unchanged, five commands | No test removed or made optional |
| Scope, out-of-scope, risk, authority, source declarations | Existing approved boundaries | Unchanged by sequence 3 | No product-scope expansion |

The earlier zero-process `verify --plan-only` rejection only established a stale preflight for its then-current snapshot; it did **not** establish a scenario prerequisite cycle. Runtime source permits planned high-risk scenarios to remain unverified at the initial verification boundary. After the stage amendment, one refreshed preflight returned `needs_human_confirmation` with `contract_amendment_policy_review_required` and `preflight_decision_evidence_invalid` for the new Contract. This is the current exact admission boundary, not an invitation to infer or record a human choice from Raydot's technical agreement. Preserve that rejected attempt and all previous passing evidence.

## Review focus

- A clippy manifest relabeled as test while retaining its original command index/digest must be rejected by the consumer (Task 2).
- Mixed or duplicate package node indices, a missing package, and a wrong per-package digest must be rejected (Task 2).
- A filtered test or two ambiguous workspace test commands must not silently claim full coverage (Tasks 1 and 2).
- Reused test nodes must cause exactly the missing package tests to run, while executed nodes must not rerun (Task 2).
- External hosted receipt/orchestration variables must not contaminate process-observer fixture packages, but must remain active in the production hosted consumer (Task 3).

---

### Task 1: Select the real Cargo test manifest

**Files:** Modify `crates/cockpit-cli/src/main.rs`; test `crates/cockpit-cli/tests/verify.rs`.

**Interface:** Keep the existing `VerificationCoverageManifest` and `VerificationReceipt` schema. The CLI's `coverage_manifest` selection chooses the unique complete unfiltered `cargo test --workspace` expansion; if none exists, retain the generic first-manifest behavior for ordinary verification. If multiple test candidates or filtering make full coverage ambiguous, do not mark a test manifest eligible for package-test reuse.

- [ ] Add a CLI regression with declared `cargo fmt`, `cargo clippy --workspace`, and `cargo test --workspace`; assert the formal receipt's manifest has `sourceArgs[0] == "test"`, all packages, stage-2 node IDs, and corresponding execution-record command digests.
- [ ] Run the focused CLI regression and capture RED against the current first-manifest behavior.
- [ ] Implement the smallest manifest-selection rule in `main.rs`, preserving the generic route when no eligible test exists.
- [ ] Add and pass legacy stage-0 test, clippy-only generic, filtered test, and two-test ambiguity cases.
- [ ] Run `cargo test --locked -p cockpit-cli --test verify` and check the precise outputs.

### Task 2: Consume the formal test receipt without stage-0 assumptions

**Files:** Modify `tests/ci/run_workspace_package_tests.sh`; test `tests/ci/workspace_package_coverage_test.sh`.

**Interface:** Existing `AI_COCKPIT_VERIFICATION_RECEIPT`, `AI_COCKPIT_VERIFICATION_ORCHESTRATION`, and `AI_COCKPIT_RUNTIME_BIN` remain unchanged. The consumer accepts only the current formal receipt and one unfiltered Cargo workspace test manifest whose one command index matches the same index in the Runtime-bound current Contract `verification` declaration. It verifies each manifest `(workspaceMember,nodeId,commandDigest)` tuple against Cargo metadata, result, execution record or valid reuse, and the existing formal/orchestration digest and snapshot/runtime bindings. Unsupported shell wrappers, explicit override, missing plan, or ambiguous declarations fail closed.

- [ ] Extend the fixture to a realistic strict receipt with `sourceProgram`, `sourceArgs`, node IDs, command digests, execution records, Contract, and Runtime status binding; add a stage-2 test positive and demonstrate RED.
- [ ] Replace the `project-command-0-package-*` assumptions with manifest-derived index and exact complete package mapping; compare the index to the current Contract's corresponding supported argv declaration.
- [ ] Add RED-to-GREEN negatives for clippy relabeling with unchanged digest/index, forged/mixed index, duplicate/missing package, wrong command digest, wrong Contract/snapshot/runtime/orchestration digest, only-clippy, filtered/multiple test, and missing plan.
- [ ] Preserve and pass old stage-0 and valid reuse coverage; assert no duplicate Cargo process for already-executed packages and exact missing-package execution for reuse.
- [ ] Run `bash tests/ci/workspace_package_coverage_test.sh` and inspect the emitted coverage report, not only the exit code.

### Task 3: Isolate observer fixture inputs

**Files:** Test-only edit `tests/ci/process_observer_test_runner_test.sh`.

**Interface:** Only the fixture calls to `run_workspace_package_tests.sh` run without inherited `AI_COCKPIT_VERIFICATION_RECEIPT` and `AI_COCKPIT_VERIFICATION_ORCHESTRATION`; production hosted receipt propagation and process-observer lock behavior stay unchanged.

- [ ] Run the existing observer regression with injected nonexistent hosted paths and capture its current RED (`phase=hosted_verification_receipt`).
- [ ] Add a regression assertion that fixture package calls remain clean under external injection, then minimally isolate those child calls.
- [ ] Pass injected and ordinary `bash tests/ci/process_observer_test_runner_test.sh`; retain existing exclusive/shared lock tests.

### Task 4: Integration and handoff

**Files:** No new implementation files beyond the Contract's exact scope.

- [ ] Run focused tests, `cargo fmt --all -- --check`, workspace clippy, workspace Cargo tests, and the Runtime-admitted canonical verification once on the frozen source snapshot.
- [ ] Query Runtime freshness and deliver a visible Outcome; preserve all prior failed and passing receipts.
- [ ] Commit scoped changes, open a canonical-branch PR, and require exact-head route, candidate verification, Rust Contract gate, workspace coverage, observer gate, and Windows/quality CI to reach terminal success.
- [ ] Request independent Raydot review; merge and perform the Runtime-admitted lifecycle/cleanup only after review and CI, then return to WI-1059 for supported sync and revalidation.
