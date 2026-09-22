# Runtime Action Explanation, Task Guides, and Outcome Handoff Implementation Plan

> For agentic workers: REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

Goal: reduce the ordinary Work Item default reading cost while making Runtime state, action admission, task guidance, bounded projection, and human Outcome delivery explicit and evidence-bound.

Architecture: keep the existing WorkItemStatusSnapshot, safeActions, blockers, evidence freshness, OutcomeV2, CLI/MCP transport, and documentation promotion helper as the foundations. Add optional structured action explanation and one shared admission evaluator, route readers through four task guides, and constrain post-close projection to identity-bound derived files. Query remains advisory and read-only; every execution entrypoint recomputes admission from a fresh snapshot.

Tech Stack: Rust workspace crates, serde protocol projections, Clap CLI, stdio MCP transport, Python documentation helpers, shell/Python/Rust repository tests, fixed Runtime /Users/sei-rinn/.local/bin/ai-cockpit version 0.2.105.

Spec: docs/superpowers/specs/2026-09-22-runtime-guide-action-outcome-design.md

## Native progress and evidence boundary

This plan does not introduce `task-start`, `task-done`, a manual progress
ledger, or a second task-report format; any earlier instruction to create one
is superseded and must not be followed. The task brief and the active Contract
remain the scope and acceptance inputs. Start, blocked, recovery, and complete
facts come from the existing Runtime Work Item lifecycle and append-only Task
Outcome event stream; verification commands, exit codes, durations, and logs
come from the existing verification receipts; commits are bound by Git.

Task progress is a read-only projection of those facts and never replaces
Work Item verification, archive, Provider cleanup, close, or human Outcome
delivery. Failed and interrupted events remain evidence. A red test is used
only when the changed behavior has a meaningful failing case; documentation or
format-only changes must not manufacture one. No new task-progress API is in
scope for this Work Item; capability discovery records whether the existing
Runtime surface is sufficient and leaves any gap for a separate design.

## Capability discovery and corrected plan

The fixed Runtime identity exposes Work Item lifecycle/status, `work-item
approach`, human `work-item outcome`, verification receipts, and the existing
append-only `TaskOutcomeEvent` stream. It does not expose an independent
child-task API or `task-start`/`task-done` commands. The event fields are
`schemaVersion`, `eventId`, `repositoryId`, `workItemId`, `eventType`,
`timestamp`, `detail`, `evidenceRefs`, `relatedEventIds`, `correctionOf`, and
`findingFingerprint`; they preserve event evidence but do not constitute a
separate subtask-progress ledger or acceptance decision.

Accordingly, the earlier manual ledger requirement is void and removed. The
active Contract/task brief remains the scope input; Runtime lifecycle/events,
verification receipts, and Git commit bindings remain the existing facts. A
commit proves submitted content, a receipt proves execution for its bound
inputs, and neither alone proves acceptance. No new task-progress interface
is added or made a prerequisite here.

The existing interface generator already projects Protocol/Clap facts and
the CLI and MCP status paths serialize the same `WorkItemStatusSnapshot`.
The corrected plan therefore validates that chain and current command
examples instead of adding a second command table or a new `describe`
command. The existing promotion helper already restricts derived files,
preserves close/history bytes, reports pending/current projection state, and
is covered by two-run idempotence fixtures; C adds focused evidence where a
gap is demonstrated rather than creating another promotion protocol.

## Global Constraints

- Use only /Users/sei-rinn/.local/bin/ai-cockpit, version 0.2.105, binary SHA-256 43a8ed731ac3187868e8d0c021be2a1a1b72986885d13bf3373f2ed5a0296b04, Runtime identity sha256:43a8ed731ac3187868e8d0c021be2a1a1b72986885d13bf3373f2ed5a0296b04.
- Base is origin/main commit 74ce5abed00385c92b026d62aa8893d92f78b570; do not switch executors during this Work Item.
- Do not modify archived Contract, receipt, archive, close, recovery, verification, or historical projection bytes.
- Do not push, dispatch CI, create or merge a PR, mutate tags, publish, mutate provider resources, or edit global Agent/MCP configuration.
- Preserve existing safeActions, blockers, evidence freshness, digests, diagnostics, unknowns, OutcomeV2, and adapter host-visibility boundaries; new fields are additive and legacy JSON remains readable.
- Runtime action admission is authoritative; task guides never grant permission, and a recommendation never equals authorization.
- Query paths are read-only and must start zero verification processes; execution paths recompute admission after Contract or repository snapshot changes.
- Unknown, missing, malformed, unsupported, and contradictory inputs remain distinct.
- Mechanical Reference content comes from existing protocol/Clap/schema/capability definitions; human prose remains manually reviewed.
- Ordinary default static read bytes must decrease by at least 40% from the measured 22,081-byte baseline without layout-only compression.
- Ordinary success-path required command count and full verification count must not increase.
- Repeated close-time projection with the same identity must be byte-idempotent, must not alter closed history, and must not create a recursive successor.
- Generated, returned, host-accepted, and host-displayed Outcome states remain distinct; unknown host display cannot be reported as displayed.

## Review Focus

- A missing or localized task guide must not change action admission; test the same Runtime projection with a missing guide file and compare safeActions/admission.
- A query result must not authorize a later state; test a Contract/snapshot change between status and execution and assert fresh rejection.
- A malformed or unsupported historical receipt must remain a compatibility issue rather than becoming a contradiction or fabricated failure; test the typed issue category.
- A close-time projection write failure must leave close and historical evidence unchanged while exposing projection pending; test before/after manifests and the persisted close bytes.
- An Outcome returned by a host without display confirmation must retain the complete body and unknown display state; test ordered segment identity for success, blocked, and recovery fixtures.

## Task 1: A — route readers through task guides

Files:
- Create: agents/skills/README.md
- Create: agents/skills/ordinary-work-item.md
- Create: agents/skills/verification-failure-recovery.md
- Create: agents/skills/provider-resource-finalization.md
- Create: agents/skills/release-upgrade-acceptance.md
- Modify: AGENTS.md
- Modify: .ai/README.md
- Modify: .ai/glossary.md
- Create: tests/docs/task_guide_route_test.py

Interfaces:
- Consumes: fixed Runtime entrypoint, existing lifecycle names, current Outcome delivery boundary, and docs/reference pages.
- Produces: stable guide IDs ordinary-work-item, verification-failure-recovery, provider-resource-finalization, release-upgrade-acceptance; an AGENTS route that names when each guide is loaded; a minimal README/glossary entry set.

- [x] Step 1: Write the failing route test.

    Add tests/docs/task_guide_route_test.py. Read the four guide files and assert each contains headings Applicability, Authoritative inputs, Operations, Success conditions, Failure evidence, and Continue or stop. Assert AGENTS.md links every guide, states that Runtime output is authoritative for admission, and does not contain the complete release/resource lifecycle sequence. Assert .ai/README.md contains the explicit inspect/status/doctor route and links to the ordinary guide. Assert the default route does not require loading release-upgrade-acceptance or provider-resource-finalization.

- [x] Step 2: Run the route test and observe the expected failure.

    Run:

    python3 tests/docs/task_guide_route_test.py

    Expected: FAIL because agents/skills does not exist and the current AGENTS.md still contains the full lifecycle and conditional release/resource prose.

- [x] Step 3: Add the four guides and reduce the public entry points.

    Create the index with a table mapping guide ID to applicability, path, and the Runtime query that selects it. Each guide must name its non-applicability boundary, read only the explicit Contract/Runtime/projection inputs, describe operations without duplicating action allow-lists, preserve exact evidence on failure, and link to the relevant Reference page. Keep AGENTS.md limited to cross-task constraints, explicit Runtime discovery, guide routing, and complete Outcome delivery. Keep .ai/README.md limited to attachment/configuration/record locations and the short route. Keep the glossary to Runtime, Repository Context, Work Item, Contract, Evidence, Preflight Review, Human Decision, Outcome, and the three colors, with links for deeper terms.

- [x] Step 4: Run the route test and documentation link checks.

    Run:

    python3 tests/docs/task_guide_route_test.py
    tests/docs/documentation_acceptance.sh

    Expected: PASS; links resolve and ordinary guidance does not require release or historical recovery text.

- [x] Step 5: Commit the A slice.

    Run:

    git diff --check
    git add AGENTS.md .ai/README.md .ai/glossary.md agents/skills tests/docs/task_guide_route_test.py
    git commit -m "docs: route work by task guide"

## Task 2: B — add an additive structured action explanation

Files:
- Modify: crates/cockpit-protocol/src/lib.rs
- Modify: crates/cockpit-repository/src/status_projection.rs
- Modify: crates/cockpit-repository/src/lib.rs
- Test: crates/cockpit-repository/tests/status_projection.rs
- Test: crates/cockpit-protocol/tests/interface_description.rs

Interfaces:
- Consumes: WorkItemStatusSnapshot safeActions, blockers, unknowns, evidence freshness, lifecycle phase, governance state, and current snapshot/Contract digests.
- Produces: WorkItemActionIssueKind, WorkItemAdmissionState, WorkItemActionIssue, WorkItemActionExplanation, and optional WorkItemStatusSnapshot.actionExplanation. Existing JSON without actionExplanation deserializes unchanged.

- [x] Step 1: Write failing protocol and projection tests.

    Add protocol tests that serialize an allowed explanation and deserialize a legacy WorkItemStatusSnapshot without actionExplanation. Add repository tests that build a no-resource active fixture, a verification-failure fixture, and a malformed/unsupported evidence fixture. Assert the projection returns guideId, recommendedAction, recommendationReason, admissionState, issue kinds, human decision requirement, missing inputs, and an admission digest. Assert safeActions remains present and unchanged.

- [x] Step 2: Run the focused tests and observe the expected failure.

    Run:

    cargo test -p cockpit-protocol interface_description --locked
    cargo test -p cockpit-repository status_projection --locked

    Expected: FAIL because the additive protocol types and actionExplanation field do not exist.

- [x] Step 3: Implement the minimal additive protocol and projection.

    Add serde-compatible protocol types with camelCase JSON and defaults for the optional snapshot field. Keep issue categories explicit: missing, unknown, malformed, unsupported, contradictory. In status_projection.rs, derive the explanation from the already computed lifecycle/governance/blocker/evidence facts and safeActions. Select ordinary-work-item for ordinary active/closed work, verification-failure-recovery for failed/stale verification, provider-resource-finalization for resource-bound cleanup, and release-upgrade-acceptance only for an explicit release/upgrade Contract operation. Compute the admission digest from repository ID, Work Item ID, Contract digest, snapshot digest, Runtime identity, safeActions, blockers, unknowns, and evidence freshness. Do not read guide files and do not start verification.

- [x] Step 4: Re-run the focused tests and the repository status regression suite.

    Run:

    cargo test -p cockpit-protocol interface_description --locked
    cargo test -p cockpit-repository status_projection --locked
    cargo test -p cockpit-repository scenario_matrix_next_action --locked

    Expected: PASS with legacy snapshot compatibility and distinct issue categories.

- [x] Step 5: Commit the B projection slice.

    Run:

    git diff --check
    git add crates/cockpit-protocol/src/lib.rs crates/cockpit-repository/src/lib.rs crates/cockpit-repository/src/status_projection.rs crates/cockpit-repository/tests/status_projection.rs crates/cockpit-protocol/tests/interface_description.rs
    git commit -m "feat: expose structured work item action explanation"

## Task 3: B — share admission evaluation with execution and CLI/MCP

Files:
- Create: crates/cockpit-repository/src/action_admission.rs
- Modify: crates/cockpit-repository/src/lib.rs
- Modify: crates/cockpit-repository/src/status_projection.rs
- Modify: crates/cockpit-repository/src/lifecycle.rs
- Modify: crates/cockpit-cli/src/main.rs
- Modify: crates/cockpit-mcp/src/lib.rs
- Create: crates/cockpit-cli/tests/action_projection_parity.rs
- Modify: crates/cockpit-repository/tests/scenario_matrix_next_action.rs
- Modify: crates/cockpit-mcp/tests/rpc.rs

Interfaces:
- Consumes: Task 2 WorkItemActionExplanation and the existing repository snapshot/runtime context.
- Produces: repository-internal require_current_action_admission(root, work_item_id, requested_action, runtime) and a fresh action evaluation used by status and the normal verification/finish/archive/close/recovery entrypoints.

- [x] Step 1: Write failing stale-query, gate-consistency, and transport-parity tests.

    In crates/cockpit-repository/tests/scenario_matrix_next_action.rs, query a fixture, change its Contract or repository snapshot, then invoke the corresponding execution operation and assert the action is rejected with a changed admission/snapshot digest. Add a test that a missing guide file does not alter the admission fields. In crates/cockpit-cli/tests/action_projection_parity.rs, run CLI status and the MCP work_item_status fixture and compare language-neutral actionExplanation, safeActions, blockers, evidence freshness, and humanDecisionRequired. Add an RPC assertion that status remains read-only and does not spawn verification.

- [x] Step 2: Run the new tests and observe the expected failure.

    Run:

    cargo test -p cockpit-repository stale_query --locked
    cargo test -p cockpit-cli action_projection_parity --locked
    cargo test -p cockpit-mcp repository_bound_work_item_status_is_read_only --locked

    Expected: FAIL because execution paths do not yet call a shared admission function and the CLI/MCP fixtures do not expose the new structured comparison.

- [x] Step 3: Implement shared admission without replacing existing lifecycle validators.

    Create action_admission.rs with the existing safe-action string as the
    request, a fresh fact loader over the current repository/runtime snapshot,
    and require_current_action_admission that rejects when the requested
    action is absent from fresh `safeActions`. The aggregate
    `admissionState` remains explanatory: an archived item can be blocked as a
    whole while still admitting the explicit action that resolves that block,
    such as `close_after_review`. Return the structured explanation in the
    rejection context. Call this helper from verification preconditions and
    the ordinary finish/archive/close/recovery boundaries after their existing
    specific validators; do not delete existing checks. Make
    status_projection call the same fact evaluator, not a guide loader. Keep
    read-only status paths free of verification subprocesses.

- [x] Step 4: Make CLI and MCP expose the same structured result.

    Serialize WorkItemStatusSnapshot.actionExplanation in CLI status and MCP work_item_status without transport-specific renaming. Keep localized human text outside the machine projection. If capability metadata names the new fields, derive it from the protocol/Clap definitions rather than a hand-maintained command table.

- [x] Step 5: Run focused red-green tests and existing parity tests.

    Run:

    cargo test -p cockpit-repository stale_query --locked
    cargo test -p cockpit-repository scenario_matrix_next_action --locked
    cargo test -p cockpit-cli action_projection_parity --locked
    cargo test -p cockpit-cli cli_mcp_outcome_parity --locked
    cargo test -p cockpit-mcp repository_bound_work_item_status_is_read_only --locked

    Expected: PASS; stale admissions are rejected, guide presence is irrelevant to permission, and CLI/MCP core facts agree.

- [x] Step 6: Commit the shared-admission slice.

    Run:

    git diff --check
    git add crates/cockpit-repository/src/action_admission.rs crates/cockpit-repository/src/lib.rs crates/cockpit-repository/src/status_projection.rs crates/cockpit-repository/src/lifecycle.rs crates/cockpit-cli/src/main.rs crates/cockpit-mcp/src/lib.rs crates/cockpit-cli/tests/action_projection_parity.rs crates/cockpit-repository/tests/scenario_matrix_next_action.rs crates/cockpit-mcp/tests/rpc.rs
    git commit -m "refactor: share work item action admission"

## Task 4: B — generate mechanical interface facts and preserve drift provenance

Files:
- Modify: crates/cockpit-interface/src/interface_description.rs
- Modify: crates/cockpit-cli/src/main.rs
- Modify: crates/cockpit-protocol/tests/interface_description.rs
- Modify: scripts/generate_interface_references.py
- Modify: docs/reference/commands.md
- Modify: docs/reference/commands.zh-CN.md
- Modify: docs/reference/commands.ja.md
- Modify: tests/docs/interface_reference_generator_test.sh
- Create: tests/docs/reference_example_command_test.py

Interfaces:
- Consumes: protocol action explanation, existing capability show command, current marked generated regions, and the recorded pre-change drift.
- Produces: deterministic work-item-status interface facts, generated three-language regions, a check that stale command/argument examples fail, and a separate baseline note for the old drift.

- [x] Step 1: Add failing generator and invalid-example tests.

    Keep the existing generator test as the deterministic Protocol/Clap
    source check. Add reference_example_command_test.py that extracts current
    reader-route command/argument examples and compares them with actual CLI
    help facts; an absent command or argument must fail the check. Preserve
    the known pre-change drift output as a baseline fact instead of treating
    it as a new source mutation.

- [x] Step 2: Run the tests and observe the expected failure.

    Run:

    tests/docs/interface_reference_generator_test.sh
    python3 tests/docs/reference_example_command_test.py

    Expected: FAIL only if a current example is absent from CLI help or the
    existing marked generated region drifts; no task-status surface is
    presumed to exist merely for documentation symmetry.

- [x] Step 3: Extend existing capability projection and generator markers.

    Do not add a new `describe` command or a second status command table. Use
    the existing Protocol/Clap capability generator and direct CLI/MCP
    `WorkItemStatusSnapshot` serialization as the authority. Update only the
    current generator/check boundary when a real drift or missing example is
    found; do not rewrite surrounding human explanations. Keep rendering
    stable for the fixed Runtime and keep CLI/MCP wire names aligned.

- [x] Step 4: Generate and validate the mechanical regions.

    Run:

    python3 scripts/generate_interface_references.py --repo /Users/sei-rinn/.codex/worktrees/wi-runtime-guide-action-outcome/ai-cockpit --write
    tests/docs/interface_reference_generator_test.sh
    python3 tests/docs/reference_example_command_test.py

    Expected: PASS; only marked regions change, output is stable, invalid
    examples are detected, and the three languages share protocol field names
    and meanings.

- [x] Step 5: Commit the Reference slice.

    Run:

    git diff --check
    git add crates/cockpit-interface/src/interface_description.rs crates/cockpit-cli/src/main.rs crates/cockpit-protocol/tests/interface_description.rs scripts/generate_interface_references.py docs/reference/commands.md docs/reference/commands.zh-CN.md docs/reference/commands.ja.md tests/docs/interface_reference_generator_test.sh tests/docs/reference_example_command_test.py
    git commit -m "docs: generate status interface facts"

## Task 5: C — bound projection convergence and Outcome delivery

Files:
- Modify: tests/docs/promote_closed_work_item.py
- Modify: tests/docs/promote_closed_work_item_test.sh
- Modify: tests/docs/work_item_projection_policy.py
- Modify: crates/cockpit-agent/tests/outcome_delivery.rs
- Modify: crates/cockpit-cli/tests/outcome_handoff.rs
- Modify: crates/cockpit-repository/src/outcome_render.rs only if a focused failing test proves a missing domain separation

Interfaces:
- Consumes: Task 1 guide IDs, Task 2/3 action explanation, existing terminal evidence validation, existing bounded self-projection policy, and existing OutcomeDelivery/assistantMessageEvents.
- Produces: identity-bound derived-file projection reporting, pending-on-write-failure behavior without altering close, two-run idempotence evidence, and success/blocked/recovery Outcome delivery assertions.

Implementation note: the existing promotion helper and Outcome adapters already
provided the bounded projection, idempotence, pending/terminal separation, and
host-visibility behavior required by this Work Item. The C audit therefore
added no second projection protocol and made no production change where the
focused fixtures already passed; only the evidence and scenario coverage are
retained.

- [x] Step 1: Audit projection and delivery assertions.

    Extend the promotion fixture to capture Contract, archive, close, finalization, verification, and recovery bytes before sync; run the same explicit Work Item projection twice; assert the second result has changedPaths empty and no successor/extra governance record. Add a write-failure fixture that asserts the close and historical bytes remain identical and the output reports projection pending. Extend agent/CLI delivery tests with blocked and recovery Outcome bodies and assert ordered segments, exact body digests, host acceptance/display distinction, and full handoff content.

- [x] Step 2: Run the focused tests and confirm whether a real gap exists.

    Run:

    tests/docs/promote_closed_work_item_test.sh
    cargo test -p cockpit-agent outcome_delivery --locked
    cargo test -p cockpit-cli outcome_handoff --locked

    Result: the existing bounded projection and delivery assertions passed; no
    missing production behavior was demonstrated, so no replacement protocol
    was introduced.

- [x] Step 3: Retain the existing bounded projection implementation.

    Keep planned_changes restricted to the explicit derived Work Item documents and parity rows. Bind the result to the terminal evidence identity and expose a deterministic projection input/output summary. Catch write failures at the command boundary as projection_pending without writing a successor or changing close. Do not change validate_terminal_evidence rules or any historical record. The focused audit found no missing status, risk, evidence, decision, next-action, or host-visibility domain, so outcome_render.rs remains unchanged.

- [x] Step 4: Re-run projection and delivery tests.

    Run:

    tests/docs/promote_closed_work_item_test.sh
    cargo test -p cockpit-agent outcome_delivery --locked
    cargo test -p cockpit-cli outcome_handoff --locked
    cargo test -p cockpit-repository task_outcome_events --locked

    Expected: PASS; second sync is unchanged, write failure is visible and recoverable, and generated/returned/displayed facts remain separate.

- [x] Step 5: Carry the C audit in the final evidence slice.

    Run:

    git diff --check
    git add tests/docs/promote_closed_work_item.py tests/docs/promote_closed_work_item_test.sh tests/docs/work_item_projection_policy.py crates/cockpit-agent/tests/outcome_delivery.rs crates/cockpit-cli/tests/outcome_handoff.rs crates/cockpit-repository/src/outcome_render.rs
    No separate C production commit is created because the existing bounded
    implementation already satisfies the focused tests; the final evidence
    commit records this bounded result.

## Task 6: integrate measurements, finish the Contract, and perform the final audit

Files:
- Modify: docs/superpowers/specs/2026-09-22-runtime-guide-action-outcome-design.md
- Create: tests/docs/governance_cost_baseline_test.py
- Modify: tests/docs/documentation_acceptance.sh only if the focused guide/reference checks need one existing entrypoint

Interfaces:
- Consumes: all A/B/C outputs, fixed Runtime identity, Contract scenario coverage, generated Reference checks, and existing documentation/projection gates.
- Produces: before/after cost comparison, final scenario evidence, remaining limitations, and a reproducible local verification record.

- [x] Step 1: Write the failing measurement contract.

    Add governance_cost_baseline_test.py with explicit byte inputs AGENTS.md, .ai/README.md, .ai/glossary.md, and .ai/agent-interface.json. Assert the post-change total is at most 13,248 bytes, ordinary guide text does not include release/provider conditions, and the four guide routes remain discoverable. The test must report byte totals and must not create a registry or write repository governance state.

- [x] Step 2: Run the measurement test and record the pre-change failure if applicable.

    Run:

    python3 tests/docs/governance_cost_baseline_test.py

    Expected: the test either fails against the pre-change 22,081-byte entry set or prints the measured current state; retain the exact pre-change and post-change numbers in the design document.

- [x] Step 3: Record final metrics and scope-separated limitations.

    Update the design document with post-change bytes, required command count, read-only query count, verification process starts, phase durations, exit statuses, fixed-environment query sample count/median/P95, and the independent pre-existing Reference drift result. State any unproven user benefit as unknown or inference. Separate implementation, resource cleanup, projection synchronization, and host display confirmation.

- [x] Step 4: Run cheap checks before formal verification.

    Run:

    python3 tests/docs/task_guide_route_test.py
    python3 tests/docs/governance_cost_baseline_test.py
    python3 tests/docs/reference_example_command_test.py
    tests/docs/interface_reference_generator_test.sh
    tests/docs/promote_closed_work_item_test.sh
    tests/docs/documentation_acceptance.sh
    git diff --check

    Expected: PASS with no generated-region drift and no link or guide-route failures.

- [ ] Step 5: Refresh Runtime bindings and run Contract verification.

    Using the fixed Runtime executable, run:

    /Users/sei-rinn/.local/bin/ai-cockpit preflight --repo /Users/sei-rinn/.codex/worktrees/wi-runtime-guide-action-outcome/ai-cockpit --contract .ai/work-items/active/WI-1006-runtime-guide-action-outcome.contract.json
    /Users/sei-rinn/.local/bin/ai-cockpit checkpoint --repo /Users/sei-rinn/.codex/worktrees/wi-runtime-guide-action-outcome/ai-cockpit --id WI-1006-runtime-guide-action-outcome
    cargo test -p cockpit-protocol --locked
    cargo test -p cockpit-repository --locked
    cargo test -p cockpit-cli --locked
    cargo test -p cockpit-mcp --locked
    cargo test -p cockpit-agent --locked
    cargo test --workspace --locked

    The fixed Runtime rejects the legacy `<repo>` placeholder declaration when
    it is passed as one runnable plan command. Use the explicit repository
    bound verification invocation with `--command cargo --args
    test,--workspace,--all-targets,--locked` for the formal receipt, and keep
    that capability gap visible rather than editing the generated Contract.

    Expected: preflight is no longer blocked by missing scenario declarations, all focused and workspace Rust tests pass, and all evidence is bound to the fixed Runtime and refreshed snapshot.

- [ ] Step 6: Commit final evidence and stop before provider actions.

    Run:

    git diff --check
    git status --short --branch
    git log --oneline --decorate --max-count=8
    git add docs/superpowers/specs/2026-09-22-runtime-guide-action-outcome-design.md tests/docs/governance_cost_baseline_test.py
    git commit -m "docs: record runtime guide outcome cost evidence"

    Expected: only declared source, tests, docs, and Runtime-owned active Work Item records are present; no push, CI, PR, merge, or release action is attempted.

## Final verification checklist

- [ ] Design baseline and responsibility migration table are committed.
- [ ] Default entry points route to the ordinary guide and do not require release/history guides.
- [ ] Runtime action explanation is additive and shares admission evaluation with execution.
- [ ] Query remains read-only, starts zero verification processes, and cannot authorize a changed state.
- [ ] CLI/MCP structured action facts agree.
- [ ] Mechanical Reference generation is stable and invalid examples are detected; pre-existing drift is recorded separately.
- [ ] Close-time projection is identity-bound, derived-file-only, idempotent, and non-recursive.
- [ ] Success, blocked, and recovery Outcomes retain full content and unknown host visibility.
- [ ] Byte, command, query, verification, and latency metrics are reported with evidence.
- [ ] Remaining limitations are explicit; provider actions remain for the human.
