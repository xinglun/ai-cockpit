---
author: AI Cockpit maintainers
title: Architecture responsibility and dependency map (2026-09)
description: Current ownership of observation, governance, lifecycle, evidence, execution, and projection facts, with cited follow-up boundaries for P1-B through P3.
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human:repository-owner
workItemId: WI-691-p0-responsibility-map
lastVerifiedBy: WI-691-p0-responsibility-map
terminalArchive: .ai/work-items/archive/WI-691-p0-responsibility-map.contract.json
terminalVerification: .ai/evidence/WI-691-p0-responsibility-map.verification.json
terminalFinalization: .ai/decisions/WI-691-p0-responsibility-map.finalize.json
terminalDecision: .ai/decisions/WI-691-p0-responsibility-map.close.json
---

# Architecture responsibility and dependency map (2026-09)

This is the P0 factual map for the 2026-09 architecture-optimization
initiative. It records current ownership and call boundaries, not a target
implementation. The North Star remains **Calibrated Human-Agent Trust**.
This Work Item changes documentation only; it does not change Rust behavior,
wire formats, governance rules, or the `.ai/` record protocol.

## Entry points and current call chain

The CLI parses commands and selects the repository/runtime adapter in
`crates/cockpit-cli/src/main.rs:619-831`; Work Item subcommands and human
Outcome display are dispatched again at `1208-1380`. The MCP adapter exposes
the stable tool list at `crates/cockpit-mcp/src/lib.rs:7-21`, validates and
dispatches requests at `574-674`, and uses the same repository Outcome input
and renderer at `1005-1055`. Neither adapter is a source of governance truth.

The common path is:

```text
CLI / MCP
  -> cockpit-repository operation
     -> cockpit-git RepositorySnapshot + repository-local records
     -> typed cockpit-protocol facts and governance validators
     -> cockpit-verification planning/execution where a command is required
     -> lifecycle/evidence receipt and status/Outcome projection
  -> CLI / MCP JSON or human rendering
```

`RuntimeContext` and `RepositoryContext` are typed identity inputs in
`crates/cockpit-protocol/src/lib.rs:120-131`. The repository crate re-exports
the already-separated `execution_context`, `evidence_store`, `lifecycle`,
`outcome_render`, `project_governance`, and `status_projection` modules at
`crates/cockpit-repository/src/lib.rs:48-92`. This is a module boundary inside
one crate, not a claim that all responsibilities are already pure.

## Responsibility table

| Responsibility | Authoritative facts | Allowed I/O | Validation / decision / display owner |
|---|---|---|---|
| CLI and MCP request entry | Parsed command/tool arguments and Runtime identity; `main.rs:619-831`, `mcp/lib.rs:574-674` | Adapter input/output only; no independent repository authority | Repository operation decides; adapters serialize or select language |
| Repository observation | `RepositorySnapshot` from `cockpit-git/lib.rs:212-231,268-300`; repository identity and source tree digests from `repository/lib.rs:536-552,2443-2663` | Git subprocesses and repository file reads at observation boundaries | `observe` / `observe_cached` at `repository/lib.rs:12084-12231`; consumers validate freshness |
| Runtime and repository identity | `RuntimeContext`, `RepositoryContext`, `.ai/cockpit.toml`, `.ai/project.json` | Repository-bound reads in `status_projection.rs:13-39`, `attach` and identity helpers | Protocol types plus repository identity checks; no human authorization is inferred |
| Contract, policy, and project declarations | Typed `Contract`, `GovernancePolicyDocument`, and `ProjectGovernanceProjection` in `protocol/lib.rs:2603-2645,609-627,322-335` | `project_governance.rs:53-127,241-317` reads declaration files; policy resolution is in `repository/lib.rs:2742-2990` | Strict parsing, identity/snapshot binding, and unknowns are returned by `project_governance`; governance helpers consume them |
| Governance validation and decision | Contract/Summary evidence, policy, snapshot, and Runtime identity; `repository/lib.rs:3359-3555,4395-4450` | Decision helpers read repository records; `governance_controls.rs:1038-1184` validates projections | `required_verification_checks` and checkpoint binding validation are pure (`governance_controls.rs:33-72`); preflight/governance entry points record receipts |
| Lifecycle coordination | Work Item Contract, Summary, checkpoint evidence, verification evidence, and finalization records | `lifecycle.rs:324-467,469-560,883-905,1087-1165`; archive/close paths in `repository/lib.rs:4504-4752,7353-7817` read and write `.ai/` records | Lifecycle ordering and gates belong to `lifecycle`/repository operations; storage helpers must not grant authorization |
| Evidence storage and history | Reusable receipts, repository/profile/node binding, delegated evidence and validity | Capability-scoped nofollow read/write in `evidence_store.rs:36-39,225-280`; protocol evidence types at `protocol/lib.rs:927-960` | Receipt validation belongs to evidence/protocol; a receipt is evidence, not a governance decision |
| Physical execution and scheduling | Verification graph/plan, `PhysicalExecution`, `ExecutionResult`, and Work Item evidence receipt | Process execution, bounded workers, resource budget, and in-process single flight in `cockpit-verification/lib.rs:1206-1441,1468-1525,1595-1833` | Execution reports success/failure; repository governance separately binds applicability and authorization |
| Status and Outcome projection | `OutcomeState`, `TaskOutcomeReport`, `WorkItemStatusSnapshot`, historical/freshness fields at `protocol/lib.rs:3236-3505` | Status projection reads config/profile, one Git snapshot, and records (`status_projection.rs:3-90`) | `status_projection` assembles machine status; `outcome_v2` assembles Outcome facts; no projection grants authority |
| Human Outcome rendering | Validated `OutcomeRenderInput` and language | `render_human_outcome` at `outcome_render.rs:70-76` has no repository parameter and formats only its input | `render_human_outcome` is the display boundary; input assembly currently remains in the same module and is a follow-up concern |
| Persistence and recovery | Atomic JSON records, lifecycle lock, archive manifest, finalization and close decision | `atomic_write` and lifecycle lock at `repository/lib.rs:12033-12064,12071-12081`; finalization operations at `5246-7140`; recovery/readiness at `status_projection.rs:464-585` | The authoritative record and recovery validators must be explicit; projections are rebuildable views only |

## Current mixed responsibilities, duplication, and dependency direction

The following are concrete observations from latest `main`, not proposed
interfaces:

1. `project_governance_projection`, `project_governance_unknowns`, and
   `project_success_criteria` each compute a current snapshot digest and
   repository ID before loading declarations (`project_governance.rs:288-317,
   320-383,385-401`). This is a request-scoped observation-context candidate,
   not proof that the reads are atomic.
2. `create_work_item_scaffold` discovers Git, captures a snapshot, reads the
   attached profile, derives scaffold facts, and writes Contract/Summary
   records (`lifecycle.rs:324-467`). `preflight_work_item_internal` similarly
   reads the Contract and snapshot while evaluating and persisting a decision
   (`lifecycle.rs:883-905` and its continuation). These are lifecycle
   coordinators with real I/O, not pure governance functions.
3. `governance_controls` is mostly a validator, but
   `record_work_item_governance_controls` writes the Summary by design
   (`governance_controls.rs:1186-1250`). The write boundary is explicit but
   should not be mistaken for read-only validation.
4. `render_human_outcome` is pure, but `outcome_render_input_from_outcome`
   passes a root into `build_outcome_render_input`, which reads archive and
   close-decision state (`outcome_render.rs:14-76`). The same module also reads
   lifecycle Summary state and human decisions (`666-705,816-875`). P1-A is
   therefore partly complete: rendering is pure, while projection assembly is
   not yet filesystem-free.
5. The repository submodules use `super::*` and call shared root helpers such
   as `repository_id`, `snapshot_digest`, and `ObserverError`. The current
   dependency is one-way within the crate (root exports modules and modules
   reuse root primitives); no new crate or circular Cargo dependency is
   justified by this map. Further extraction should first narrow these shared
   helper dependencies.

The status path already captures one Git snapshot for the complete status
projection, explicitly avoiding a second snapshot in readiness calculation
(`status_projection.rs:50-90`). This is an existing reusable mechanism, not a
reason to extend snapshot validity across execution or persistence boundaries.

## Existing mechanisms to reuse

- `RepositorySnapshot` and `GitRepository::snapshot` are the observation
  boundary (`cockpit-git/lib.rs:212-231,268-300`). They expose Git identity,
  changed paths, digests, read counts, and dependency fingerprint.
- `RuntimeContext` is the minimal executing-Runtime binding
  (`cockpit-protocol/src/lib.rs:120-124`); `_with_runtime` operations already
  thread it to runtime-bound evidence validation.
- `required_verification_checks` and
  `validate_checkpoint_evidence_bindings` consume typed inputs without command
  execution (`governance_controls.rs:33-72`).
- The capability-scoped nofollow receipt store already separates reusable
  evidence persistence from governance (`evidence_store.rs:36-39,225-280`).
- Lifecycle serialization and atomic single-file replacement already exist
  (`repository/lib.rs:12041-12081`); `evidence_store` also detects pending or
  invalid index states instead of guessing (`evidence_store.rs:71-100`).
- `OutcomeRenderInput` plus the shared CLI/MCP renderer is the right direction
  for one assembled fact input and multiple displays, even though its current
  assembly still needs to move outward (`outcome_render.rs:14-76`).
- Physical execution carries its own identity and result digests, and binds a
  separate Work Item receipt (`cockpit-verification/lib.rs:1261-1441`).

## Bounded candidate follow-ups

### P1-B — Explicit observation context

**Problem.** Several entry points capture their own Git snapshot and also
recompute identity/digests in downstream helpers. A single request can
therefore mix facts from different observation instants; the current code does
not establish an atomic snapshot merely by passing a `RepositorySnapshot`.

**Target boundary.** Capture one context for each real phase—before edit,
after execution, and before persistence—and pass it to pure validators and
projection builders. Keep a fresh boundary where concurrent modification is
being checked; do not use a global current-repository cache.

**Compatibility risk.** Sharing a snapshot across an execution or write
boundary could hide a real change. Existing rechecks may be intentional
concurrency detection, so public signatures, digest semantics, and unknown
results must remain compatible.

**Verification.** Add call-count assertions for request-local reuse and tests
for file/config/repository identity changes during observation and execution.
Confirm that an unstable phase stops, retries, or returns unknown rather than
silently reusing stale facts.

### P2-A — Lifecycle, evidence, execution, and projection ownership

**Problem.** Modules now exist for lifecycle, evidence storage, execution
context, status, and Outcome rendering, but root operations still combine
repository reads, governance checks, and writes in complete use cases such as
scaffold/preflight/archive/close.

**Target boundary.** Tighten one complete use case at a time: Observation
obtains facts, Governance validates and decides, Lifecycle orders transitions,
Evidence persists and relates records, Execution runs commands, and Projection
assembles status. Introduce a Port only at a real substitution or fault
injection boundary.

**Compatibility risk.** Preserve public APIs, JSON/file layout, errors, and
historical reads. Do not duplicate policy or evidence validation in a second
layer.

**Verification.** Use existing integration tests plus focused dependency and
ownership tests. Assert that pure governance functions perform no filesystem,
Git, or process I/O, while each complete lifecycle operation preserves its
current record order and recovery behavior.

### P2-B — State types and legal transitions

**Problem.** The protocol already has typed `OutcomeState`, verification stage,
evidence validity, finalization state, and `HumanDecision` types
(`cockpit-protocol/src/lib.rs:368-415,559-568,927-960,962-1082,3236-3505`),
but lifecycle, evidence freshness/applicability, governance, human assurance,
and historical status still meet as separate strings and optional fields.

**Target boundary.** Reuse these enums and add only narrow types where
missing, invalid, expired, revoked, and not-applicable are currently
collapsed. Derive colors and text from facts; never use display text as a
governance input.

**Compatibility risk.** Keep existing JSON spellings and unknown-version
readability. Never rewrite historical evidence to fit a new internal type.
Preserve the distinction between verified evidence and authorization for a
specific operation, and between closed historical evidence and failure.

**Verification.** Table-test legal combinations and illegal transitions;
replay archived records, including unknown/legacy schema cases, as read-only
history.

### P2-C — Multi-file consistency, concurrency, and recovery

**Current finding.** Existing lifecycle locks, atomic single-file replacement,
pending-index detection, archive manifests, finalization receipts, and close
decision validation are present. This map does not infer a defect from their
presence, and does not yet claim that the multi-file protocol is a transaction.

**Target boundary.** For finish, archive, close, and recovery, identify the
record that represents committed state, the write order, operation identity,
conflict boundary, and recovery entry point. Rebuild projections only from
that authoritative record.

**Compatibility risk.** Changes to commit semantics can affect every historical
record and must remain isolated from unrelated refactors. A single atomic file
rename cannot be used as proof of a multi-file transaction.

**Verification.** Controlled fault injection must interrupt after each write,
simulate write/space failures, repeat the same operation, race two processes,
and remove or corrupt projections. Verify that incomplete operations never
render as completed and that retry does not create a second conflicting
decision.

### P3 — Physical execution and governance binding

**Current finding.** `PhysicalSingleFlightCoordinator`, physical execution
identity, execution result, and Work Item evidence receipt live in
`cockpit-verification` (`lib.rs:1206-1441,1468-1525`), while governance
decision and policy gates live in `cockpit-repository` (`repository/lib.rs:
2934-3131,3359-3555`). The reuse-eligibility assessment is in the repository
execution-context module (`execution_context.rs:14-230`), so its boundary
owner is still split from physical execution.

**Target boundary.** Keep physical execution, Work Item evidence binding, and
governance permission as three separately validated facts. A cache hit or
successful command must never directly grant a Work Item passing state.

**Compatibility risk.** Connecting or widening single-flight sharing changes
concurrency and repository isolation. It is a new architectural commitment;
do not expand it as a side effect of a pure refactor.

**Verification.** Before expanding sharing, test same-key and different-key
concurrency, failure propagation, cancellation, resource peaks, execution
counts, receipt binding, and the final governance decision independently.

## P0 conclusion and remaining unknowns

The current architecture already has useful boundaries—typed protocol facts,
Git snapshots, capability-scoped evidence storage, lifecycle locking, bounded
execution, and a filesystem-free final renderer. The principal remaining
unknowns are end-to-end observation-context ownership, the exact commit record
for each multi-file lifecycle operation, and the production ownership of
verification reuse versus physical execution. Those are investigation inputs
for P1-B, P2-A/P2-C, and P3; this P0 Work Item does not implement them.
