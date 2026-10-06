---
author: AI Cockpit maintainers
title: "Command Reference"
description: "The current CLI command surface and its mutation or evidence boundary."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_commands
---

# Command reference

Historical compatibility is explicit and low assurance. Legacy shared-primary
worktree records may use `historical.kind=shared_worktree_retained` with
`assurance=historical_low`; they identify the primary worktree and still need a
human close decision. A pre-PR local merge may use
`historical.kind=direct_merge_no_pr`, `pullRequest.number=0`, and a
`historical://direct-merge/<mergeCommit>` URL, but must bind the real merge
commit, both parents, base revision, repository identity, and authority.
Runtime verifies those facts against Git and never invents a PR. Readiness
reports `historicalDebt` and recovery actions while keeping pending-close
fail-closed.

For a legacy receipt that predates the `historical` field, the Runtime may
derive the same `shared_worktree_retained` projection without rewriting the
receipt, but only when `provider=local`, the Contract and receipt agree, and
both worktree paths canonicalize to the Git primary checkout with the branch
and clean worktree still retained. This is reported as
`historical_low`; an external provider, linked worktree, missing context, or
ambiguous topology remains fail-closed. Use `finalize-recovery-plan` and the
explicit recovery receipt when those facts cannot be derived.

New Work Items still require the current finalization head to have disposition
`deleted` for a successful merged delivery. An explicitly closed, unmerged
failed delivery may instead use `abandoned`, but only when the receipt proves
the branch and worktree were removed, carries exactly
`unmerged_pull_request`, and has no merge commit. `abandoned` is a terminal
failure record, not a successful or merged governance projection. A retained,
blocked, or unknown head stops the operation before a close decision is
written. A verified historical `shared_worktree_retained` or
`direct_merge_no_pr` receipt is the narrow compatibility exception: it may keep
the primary worktree and close only with `assurance=historical_low`, explicit
human authority, and repository-bound Git facts. This exception never applies
to new Work Items and never upgrades historical evidence to provider assurance.
For immutable records produced by an older Runtime, `work-item finalize` accepts
one strictly bound deleted transition after close as a legacy reconciliation.
The transition must bind the closed root path and digest and is verified as an
append-only cleanup observation; it never rewrites the close receipt.

`work-item finalize` stores the first receipt at `.ai/decisions/<id>.finalize.json`. For an ordinary PR receipt, `pullRequest.baseRevision` remains the provider's actual comparison base. A legacy receipt without `contractBaseRevision` must still equal the archived Contract's immutable `baseRevision`; a new receipt may carry `contractBaseRevision`, which must equal that Contract base and allows the provider comparison base to differ. Recording and `finalize-verify` reject a missing, wrong, or contradictory binding. The narrow historical `direct_merge_no_pr` exception instead binds `pullRequest.baseRevision` to the real merge commit's first parent and records the immutable Contract base as `historical.contractBaseRevision`; it never uses provider `contractBaseRevision` or upgrades historical assurance. The receipt's `contractDigest` still binds the exact archived Contract. `finalize-recovery-plan --merge-commit <sha>` derives both historical values, so a bundled historical merge can be recorded without changing either fact or inventing a PR. Rebase before archive requires a renewed active Contract binding; rebase after archive requires recovery and never permits receipt or archive rewriting. If that immutable root exists, a typed transition envelope must bind the unique head's predecessor digest and next sequence; Runtime appends `.finalize.<digest>.json`. `finalize-verify` reports `headPath`, `headDigest`, and `sequence`, which `close` binds. A sequence-1 merge observation may additionally bind `governanceAppendRevision` when the receipt commit advanced all aligned heads. Runtime requires an ancestor range of additions only. Besides regular same-Work-Item finalization receipts, the only permitted evidence additions are the complete fixed-schema pair `.ai/evidence/<id>/quality-route-post-finalize.json` and `.ai/evidence/<id>/repository-gates-post-finalize.json`; every path must be an `A`-only `100644` regular blob, and their archived Contract, PR revision, route digest, manifest, profile, and passing gate bindings must agree. The pair is evidence, not authority, and does not replace the required finalization receipt addition. This does not permit arbitrary evidence paths or archive mutation.

All repository commands accept an explicit `--repo <path>`. Commands that
produce records or decisions keep JSON on stdout. `finish`, `archive`, and
`close` additionally emit the localized human handoff on stderr by default;
their `--json` option suppresses only that handoff. `work-item outcome` emits
the localized human handoff on stdout by default. Adapters should pass
`--language en|zh|zh-CN|ja` for the active conversation language; omitted
language uses the locale fallback. Add `--json` for the stable machine-readable
`OutcomeV2`. A failed or unknown decision is not a pass.

| Group | Commands | Boundary |
| --- | --- | --- |
| Read-only | `inspect`, `observe`, `status`, `compatibility`, `migrate plan`, `capability show`, `diagnose`, `doctor` | Read repository state or derived evidence; no silent repair. |
| Derived projection | `knowledge query` | Explicitly materializes or reuses a repository-local `.ai/knowledge/` index; reports `projection.writeBoundary=repository-local-derived` and never changes governance authority. |
| Setup | `attach`, `profile confirm`, `profile propose` | Create/update protocol state, confirm a profile, or emit a read-only candidate. |
| Migration | `migrate apply --approved` | Apply only the reviewed repository-schema migration and write a runtime-bound migration receipt. |
| Governance write entry | `preflight` | Evaluate a Contract and persist its explicit preflight projection. Repeating an unchanged preflight is idempotent and returns `changedPaths`; incomplete or uncertain Contracts are human-review yellow and cannot cross checkpoint. |
| Work Item | `work-item new`, `start`, `status`, `checkpoint`, `finish`, `archive`, `close`, `validate`, `controls`, `amend`, `amendments`, `revalidate-amendment`, `recover`, `revalidate-archived`, `finalize-plan`, `finalize`, `finalize-verify`, `finalize-recovery`, `finalize-recovery-plan`, `closeout-recovery-plan`, `closeout-recover` | `amend --request` applies reasoned schema-aware changes against the current digest; `--input --reason` remains the legacy additive adapter. History is read-only. Accepted amendments invalidate affected evidence; query `work-item status` and follow Runtime's currently admitted next action (`run_preflight` is common, not unconditional). `revalidate-amendment` is only for direct Contract edits. Cross-checkout plan is read-only; recovery is a separate explicit write. |
| Parallel Work Item | `work-item boundary`, `work-item declare`, `work-item slot acquire|release|list` | Bind Contract-owned concurrency paths and reserve repository-local slots; unknown boundaries serialize. |
| Cross-Work-Item coordination | `work-item coordination inspect|register|report-impact|publish-outcome|request-pause|acknowledge|resume|recover|check-environment-drift|record-environment-drift` | Inspection and drift check are read-only; recording drift is an explicit persistent write before affected actions. |
| Verification | `verify` | Execute bounded commands, record evidence, and optionally bind it to a Work Item. |
| External evidence | `evidence import`, `evidence list`, `evidence policy`, `evidence purge-plan` | Bind exact provider bytes, declare bounded persistence, or produce a deterministic non-destructive disposal plan. |
| Audit | `audit export`, `audit cognitive-benefit` | Export repository-bound events or run the fixed Rust cognitive-benefit evaluation; neither result grants release authority. |
| Adapter | `agent first-start/list/install/doctor/repair/detach`, `mcp` | Print the mandatory first-start gate, manage an explicitly selected repository-local Agent adapter, or serve JSON-RPC over stdio; every repository operation binds `--repo`. |

## Contract amendments and environment drift

In Runtime `1.0.1-rc.1`, a policy-sensitive CLI amendment request may include a
request-bound `authorization` object: `schemaVersion`, `decisionId`,
`decision: "authorize_change"`, `authorizedBy`, `authoritySource`,
`assurance: "self_declared"`, `executedBy`, `repositoryId`, `workItemId`,
`contractDigest`, `repositorySnapshotDigest`, `requestDigest`, and
`changedPaths`. Keep the authorizer and executor distinct. Use the read-only
`work-item amend-check` result to bind the current repository, Work Item,
Contract, snapshot, request digest, and exact changed paths; `amend` rechecks
the request before writing. This declared authority is not host-authenticated
proof and is not a `confirm_review` decision. The current `tools/list` schema
for `work_item_amend` does not expose this optional object, so do not infer MCP
support or send an undeclared field through MCP; use the supported CLI route.

When a sensitive amendment makes the previous preflight stale, run a fresh
`preflight` only when Runtime admits it. That preflight may create a pending
review request bound to the current repository, Work Item, Contract digest,
and repository snapshot. This request-generating route also applies when a
valid recovery retry remains pending and the amendment has reset preflight to
`not_run`; the retry does not substitute for amendment review. Generating the
request records no human decision and is not `confirm_review`; verification
and execution remain blocked until a matching identity-bound human decision
is recorded.

`work-item amend --repo <repo> --id <id> --request <request.json>` accepts a
`ContractAmendmentRequest` with `schemaVersion`, unique `changeId`, current
`expectedContractDigest`, non-empty `reason`, and ordered `changes`. Operations
are `add`, `set`, `clear`, `remove`, `replace`, and `reorder`; unknown or
protected paths fail closed, the prospective Contract is fully validated, and
a digest mismatch conflicts rather than rebasing. Replaying the same request
and `changeId` returns its original receipt. `work-item amendments` is a
read-only append-only history query. An amended Contract carries the protected
`requiredRuntimeCapabilities` marker. The exact old Runtime `0.2.113` rejects
that unknown field during preflight before action admission; its lifecycle
compatibility does not imply amendment/drift support.

For coordination, `check-environment-drift` only observes and reports pending
Runtime-derived drift. `record-environment-drift` explicitly persists the
observed event to shared repository state; its input names the Work Item and
expected generation, not a caller-supplied digest. Refresh admission before
the dependent action. Unrelated actions can continue only with their own fresh
admission. Recovery appends a resolution; it never deletes the event. The
request-scoped observation ledger is not cross-process event storage.

MCP exposes equivalent `work_item_amend`, `work_item_amendments`, and
`work_item_environment_drift` tools. The drift tool separates read-only
`action=check` from explicit `action=record`. Discover exact schemas through
`tools/list`; an older Runtime must not silently ignore unsupported constraints.

## Cross-checkout Work Item closeout recovery

`work-item closeout-recovery-plan --repo <destination> --source-repo <source>
--id <work-item>` is read-only. It validates that the source and destination
are distinct checkouts with the same Runtime repository identity, then checks
the archived Contract, verification evidence, close decision, and any
provider-finalization binding before listing the exact files and digests.
Provider context may be bound by the archived Runtime resource-context record
even when the archived Contract's `resourceContext` is null; that binding must
still be validated and reported.

`work-item closeout-recover` accepts the same arguments and is the explicit
write operation. It revalidates source and destination state, imports only the
validated Work Item files, preserves source bytes, rejects conflicting
destination files, and installs the close decision last. A detected race or
write failure rolls back only files newly installed by this attempt whose
inode and bytes still match; changed or unidentifiable paths are preserved and
reported as incomplete. A process interruption before the close decision may
leave exact immutable files for recovery, but the item remains unclosed and a
retry is idempotent. This is not a multi-file filesystem transaction.
Neither `allowed: true` nor a successful plan grants write authority; require
fresh Runtime admission and authorization for the exact destination.

MCP provides the matching `work_item_closeout_recovery_plan` (read-only) and
`work_item_closeout_recover` (explicit write) tools. Both require
`workItemId` and an absolute `sourceRepo`; the repository-bound MCP server's
repository is the destination. Discover exact schemas through `tools/list`.

Before an Agent performs any repository operation, run
`ai-cockpit agent first-start --repo <path>`. The command is read-only and
prints the canonical English procedure also projected into installed adapter
managed sections. Use `ai-cockpit --help`, the relevant group help, and
`capability show --repo <path>` to discover exact command schemas; a listed
capability is not authorization or readiness.

For the read-only interface facts used by the Outcome trial, add
`--surface work-item-outcome` to `capability show`. The generated facts below
come from the Runtime protocol projection; they describe parsing and wire
shape, not permission or lifecycle readiness.

Refresh only the marked regions with
`python3 scripts/generate_interface_references.py --repo <path> --write`;
the documentation gate uses the side-effect-free `--check` mode.

<!-- AI_COCKPIT_INTERFACE_FACTS:BEGIN work-item-outcome -->
### Interface facts: `work-item-outcome`

- Schema: `v1`
- Runtime: `1.0.1`
- Names, types, requiredness, defaults, and enum values are structured facts; this description grants no authority.

#### `cli` · Transport: `argv`

| Parameters | Type | Required | Default | Enum | Aliases |
| --- | --- | --- | --- | --- | --- |
| `id` | `string` | `yes` | `—` | `—` | `—` |
| `delivery` | `boolean` | `no` | `false` | `—` | `—` |
| `json` | `boolean` | `no` | `false` | `—` | `—` |
| `view` | `enum` | `no` | `summary` | `summary | full` | `—` |
| `language` | `enum` | `no` | `—` | `en | zh | zh-CN | ja` | `—` |

#### `mcp` · Transport: `json-rpc`

| Parameters | Type | Required | Default | Enum | Aliases |
| --- | --- | --- | --- | --- | --- |
| `workItemId` | `string` | `yes` | `—` | `—` | `id` |
| `language` | `enum` | `no` | `—` | `en | zh | zh-CN | ja` | `—` |
| `view` | `enum` | `no` | `summary` | `summary | full` | `—` |
| `delivery` | `boolean` | `no` | `false` | `—` | `—` |
| `deliveryProgress` | `object` | `no` | `—` | `—` | `—` |

<!-- AI_COCKPIT_INTERFACE_FACTS:END work-item-outcome -->

## Release recovery identity binding

The release workflow accepts a governance identity (`work_item_id` or
`contract_path`) and, when recovery changes were delivered by a different Work
Item, an independent source-verification identity (`source_work_item_id` or
`source_contract_path`). The shared resolver reads and validates both from one
request-scoped active-Contract index. Omitting the source selector means a
same-Work-Item technical retry and reuses the governance Contract for source
quality. Supplying both source selectors is allowed only when they identify the
same Contract; missing, partial, stale, foreign, or mismatched bindings fail in
the cheap input-preflight stage before Rust compilation. The release evidence
retains the governance binding separately from the source-quality route, so a
later recovery Contract does not widen an immutable Release acceptance
Contract.

## Important options

## MCP tool usage

Agents should discover the surface in this order: start the repository-bound
stdio server, call `initialize`, call `tools/list`, then call one tool with the
schema-described arguments. Every `tools/call` is repository-bound and rejects
unknown fields, wrong types, missing required fields, and conflicting aliases
before any repository operation runs.

| Tool | Arguments | Typical call |
| --- | --- | --- |
| `status`, `work_item_list`, `repository_observe`, `capability_show` | `{}`; `capability_show` also accepts optional `surface`, `format`, and `language` for a read-only interface description. | Read repository facts or the capability registry. |
| `work_item_get`, `work_item_outcome`, `work_item_validate` | Exactly one `workItemId` (or legacy `id`); `work_item_outcome` optionally accepts the active conversation `language` (`en`, `zh`, `zh-CN`, `ja`). | `{"workItemId":"WI-123"}` |
| `work_item_start` | Required `workItemId`, human-supplied `intent` and `goal`, and non-empty `scope`; optional `outOfScope`, `risk`, `authority`, `acceptanceCriteria`, `requiredEvidenceClasses`, and `sources`. It persists preflight and creates exactly one before-edit checkpoint only when no blocker or human-confirmation boundary is present. | `{"workItemId":"WI-123","intent":"reduce repeated setup","goal":"prepare before implementation","scope":["src/**"],"authority":"authorized","sources":["issue:123"]}` |
| `work_item_status` | `{"all":true}` or exactly one Work Item id. | `{"all":true}` |
| `preflight` | Required repository-relative `contract`. | `{"contract":".ai/work-items/active/WI-123.contract.json"}` |
| `blockers`, `safe_actions` | Optional repository-relative `contract`. | `{"contract":".ai/work-items/active/WI-123.contract.json"}` |
| `knowledge_query` | Optional `topic`, `component`, `state`, `workItemId`. | `{"topic":"verification"}` |
| `evidence_get` | Exactly one of `path`, `evidencePath`, or `id`. | `{"id":"WI-123"}` |
| `delegated_evidence_list` | Required `workItemId`. | `{"workItemId":"WI-123"}` |
| `work_item_controls`, `work_item_recover` | Exactly one Work Item id plus exactly one object: `controls`/`input`, or `receipt`/`input`. | `{"workItemId":"WI-123","controls":{...}}` |
| `work_item_closeout_recovery_plan`, `work_item_closeout_recover` | Both require `workItemId` and absolute `sourceRepo`; the first is read-only and the second explicitly writes validated evidence to the MCP-bound destination checkout. | `{"workItemId":"WI-123","sourceRepo":"/absolute/path/to/source"}` |
| `verify` | Optional `workItemId`, `command`, string-array `args`, finite `timeoutSeconds`, and boolean `planOnly`; command is allowlisted. | `{"workItemId":"WI-123","command":"cargo","args":["test","--locked","--workspace"],"timeoutSeconds":600,"planOnly":true}` |
| `work_item_parallel` | `action`: `inspect`/`acquire`/`release`/`list`; inspect/acquire/release require an id, release also requires `leaseId`. | `{"action":"inspect","workItemId":"WI-123"}` |
| `work_item_amend`, `work_item_amendments` | `work_item_amend` requires `workItemId` and a strict typed `request`; `work_item_amendments` requires only `workItemId` and is read-only. | `{"workItemId":"WI-123","request":{"schemaVersion":1,"changeId":"change-1","expectedContractDigest":"sha256:<current-digest>","reason":"Correct the plan","changes":[{"path":"/goal","operation":"replace","value":"updated goal"}]}}` |
| `work_item_environment_drift` | `action=check|record`, `workItemId`, and `generation`; check is read-only, record persists observed drift. | `{"action":"check","workItemId":"WI-123","generation":2}` |

For a person-facing result, call `work_item_outcome` and surface its text
content without folding it away. Use `--json` only for automation; raw
`work_item_get` data is not a human handoff. A tool result with `isError: true`
is a stop, not a successful empty result. MCP does not configure a host Agent,
post into a chat window, or invent missing intent, authority, acceptance, or a
human decision; the Agent/host owns that presentation and must stop for human
review when the returned state is yellow, red, unknown, or not ready.

- `verify --command <program> --args <comma-separated>` runs an explicit command
  and is always fresh. `--work-item <id>` records the receipt for that Work Item;
  its detected Cargo/npm command uses the dynamic profile-authorized path, while
  an explicit custom command remains fresh.
- `verify --timeout-seconds <n>` uses a finite timeout in the inclusive range
  `1..=900` seconds. With no override, a `cargo test --package` node uses a
  finite 600-second default; other commands retain the 300-second default.
  Every explicit value is an override and requires a finite Contract or
  repository policy ceiling. Values above the Runtime cap are rejected before
  spawn. The effective timeout is included in the plan, receipt, command
  digest, and reuse identity; a deadline fails closed and terminates the
  process tree. Without an explicit `CARGO_TARGET_DIR`, Cargo verification uses
  a target directory derived from the canonical checkout path, so linked
  worktrees do not share test binaries. An explicit target remains authoritative.
- `verify --plan-only` resolves the route and emits the deterministic plan without
  spawning project verification commands. For a Cargo workspace, the plan runs
  one metadata query and partitions `cargo test --locked --workspace` into
  identity-bound package nodes; the formal receipt retains the source command,
  workspace members, metadata digest, exit status, bounded logs, and elapsed time.
- Executed CLI and MCP `verify` results include a presentation-only
  `diagnosticSummary` grouped by severity, lint code, and root message, with the
  occurrence count and affected node IDs. It is derived from bounded stderr
  bytes using lossy UTF-8 for display; per-node results and raw output bytes
  remain unchanged in `executionRecords`, and the summary is excluded from the
  persisted typed verification receipt.
- MCP `verify` with `planOnly: true` uses the same per-node identity preparation
  and reports the planned action, state, reason, and binding mismatches with
  `processesSpawned: 0`; a subsequent execution must resolve the same actions
  unless a bound input changes.
- `verify --archived-recovery --work-item <id> --stage pull_request` is the
  append-only recovery path for an archived Work Item whose source evidence
  projection became stale after a reviewed integration change. It runs one
  fresh, typed, coverage-bound Runtime verification and records which exact
  `evidence_class_projection` judgment it replaces. The archived Contract,
  Summary, Outcome, Events, and historical verification bytes are never
  overwritten; preconditions fail before the project command when recovery is
  already valid or contradictory. This is not a Contract-amendment successor;
  use `work-item revalidate-archived` when the archived Contract itself changed.
- `work-item recover-selected-successor-lineage --repo <path> --id <root>
  --input <receipt.json>` is the append-only close-recovery path for an already
  selected multi-hop successor lineage. The receipt enumerates every adjacent
  predecessor/successor edge and binds each node's archived Contract, Summary,
  Outcome, Events, verification, archive manifest, close, finalization head,
  provider PR identity, and explicit human decision to exact bytes and the
  current Runtime. It never creates a successor or rewrites historical
  records. Missing, malformed, foreign, stale, digest-mismatched, forked, or
  ambiguous lineage fails closed; a valid receipt may satisfy only the covered
  archived nodes' pending-close projection.
- `verify` without `--command` detects Cargo or npm and may use a confirmed
  profile for cross-process reuse. Reuse is admitted only when the current
  repository, snapshot, profile, runtime, command, scope, stage, runner, base,
  toolchain, dependency, and policy identities match exactly. Otherwise the
  declared command executes and the result reports the denial/escalation reason.
  Required and protected nodes are never skipped by timing or cache state.
- `verify --workers <n>` requires a positive worker count and defaults to one.
  Work Item-bound verification is serial by default (`--workers 1`); explicit `--workers >1`
  fails closed until Runtime can prove per-node dependency readiness and
  isolated outputs. Independent CI jobs may still run in parallel when they
  share one immutable route/source identity, have no dependency edge, use
  isolated outputs, and stay within resource limits. Receipt producers and
  consumers remain ordered.
- `work-item boundary --repo <path> --id <id> --file <boundary.json>` binds an
  additive Contract `concurrencyBoundary`. Its four path classes and
  `maxWorkers` are validated; `maxWorkers` is a slot capacity and is distinct
  from `verify --workers`.
- `work-item slot acquire|release|list` manages exclusive leases under
  `.ai/parallel/leases/`. Leases carry repository and Work Item identity. A
  missing, malformed, ambiguous, unsafe, or stale lease/boundary fails closed;
  there is no implicit expiry or global current Work Item.
- `start` requires `--id`, `--intent`, and `--goal`; `--authority authorized`
  is needed for a green governed flow.
- `start --prepare --source <reference>` appends each supplied source before
  preflight, persists that preflight, and creates one before-edit checkpoint
  only when no blocker or human-confirmation boundary is present. A yellow
  verification-pending result is not a passed verification; review-required or
  blocked results do not create a checkpoint. MCP `work_item_start` applies
  the same prepared-start behavior and defaults omitted authority to `missing`.
- `start --required-evidence <class>[,<class>...]` accepts the built-in forms
  `verification`, `verification_receipt`, `verification-receipt`,
  `delegated:<provider>`, `delegated_evidence`, `external_evidence`, and
  non-empty custom labels whose evidence is later digest-bound.
  Stage labels such as `public-install` are rejected before verification;
  inspect the command help for the same vocabulary. Existing historical
  Contracts remain readable, but an active legacy declaration must be repaired
  through an append-only Contract amendment or an explicitly bounded successor
  before it can cross preflight.
- Before `start` or `work-item new`, the Runtime applies a repository-scoped
  entry gate. A non-`.ai` working-tree change, detached HEAD, a known HEAD
  mismatch with the locally discovered remote default ref, or an archived Work
  Item without a valid close decision fails closed. The gate never rewrites
  archived bytes. A recovery successor is an explicit continuation and may be
  scaffolded through `work-item recover`; it is not an independent next item.
  Its preflight still evaluates archived-scope relations: a valid, provably
  disjoint historical scope is not a global blocker, an overlap is allowed
  only for the exact repository-bound recovery decision, and malformed,
  foreign, stale, symlinked, or digest-mismatched history remains fail-closed.
- The same entry gate rejects the repository primary worktree and the known
  default branch for ordinary Work Items. Use a dedicated linked worktree on a
  feature branch. A linked worktree without an unambiguous discovered remote
  default base is rejected rather than treated as ready; local calibration
  repositories with no linked worktree remain `status: unknown` until a base is
  configured.
- `work-item new --repo <path> --id <id> --mode <mode>` creates a `not_ready`
  skeleton. It fills only snapshot-derived facts and leaves human-owned fields
  empty or `unknown`; `start` remains a compatibility path over the same writer.
  A repository-local exclusive reservation makes duplicate races fail closed:
  one same-ID request succeeds, the other fails, while different repositories
  remain independent.
- When `start` activates a scaffold, the default Cargo verification command is
  selected from repository facts: repositories with `Cargo.lock` use
  `cargo test --locked --workspace`; lockfile-less Cargo repositories use
  `cargo test --workspace`; non-Cargo repositories receive no invented command
  and require an owner-approved check. This selection is deterministic and is
  not a substitute for human-owned intent or acceptance.
- `work-item outcome --repo <path> --id <id>` presents the result in the order
  completed work, problems, stops, risks, unknowns, decisions, verification,
  impact, and next action. Use `--json` for automation. See [Human-facing
  Outcome](outcome-report.md) for status-marker and localization rules. A
  completed Work Item also binds a typed `*.task-report.json`, a human-readable
  `*.task-report.md`, and an append-only `*.events.jsonl` stream; these are
  evidence-bound projections, not extra authority or a replacement for the
  Contract and verification receipt.
- `work-item finalize-recovery --repo <path> --id <id> --input <receipt.json>`
  records one append-only, Runtime-bound classification for an immutable
  legacy finalization receipt. The input must bind the exact predecessor
  digest, repository/Work Item/Contract base, current Runtime, actor,
  authority, reason, and timestamp. Use `historicalKind=shared_worktree_retained`
  for an older primary-worktree receipt. For a legacy direct merge with no PR,
  use a complete `historicalKind=direct_merge_no_pr` finalization receipt. If
  no canonical predecessor exists, this command accepts that direct-merge
  receipt as the first canonical record and applies the same archive, Contract,
  Git-parent, repository, and current-Runtime checks as `finalize`; it does not
  create a recovery classification or rewrite any historical bytes. A
  provisional legacy Contract context may be resolved only when the receipt
  remains bound to the same primary worktree and repository/base facts. Any
  other mismatch fails closed and identifies the binding category (for
  example `resourceContext.worktree` or `resourceContext.baseRevision`).
  A direct-merge receipt may preserve the archived Contract's original local
  `resourceContext` verbatim; the Runtime treats that as the historical
  declaration and still requires the receipt's branch, worktree, base, real
  merge parents, repository identity, and `historical_low` assurance to bind.
  The plan also emits an identity-consistent `resourceContext` using the
  explicit historical provider/URL, so Agents do not need to guess which form
  to submit.
  The predecessor is never rewritten, and the recovery record cannot by itself
  make a Work Item green.
- `work-item finalize-recovery-plan --repo <path> --id <id>` is the read-only
  discovery boundary for that recovery. It reports the immutable predecessor
  path/digest, producer Runtime identity, inferred shared-primary disposition,
  and the exact human fields still required. When a Work Item has no canonical
  predecessor, pass the real `--merge-commit <sha>`; the plan verifies its
  parents and emits the deterministic identity facts (`repositoryId`, current
  Runtime, merge commit, parents, base revision, and the zero-PR URL) plus a
  complete typed `pullRequest.number=0`/`historical://direct-merge/<sha>` receipt
  skeleton. `pullRequest.baseRevision` is the real merge first parent;
  `historical.contractBaseRevision` is the archived Contract base. Both are
  deterministic facts and must be preserved. When the archived Contract has a
  non-provisional resource context, the plan also fills receipt/operation IDs,
  branch/worktree identity, base branch/remote, merged before/after PR states,
  and a conservative `retained` result with
  `historical_resource_state_unknown`; it never asks an Agent to reconstruct
  those protocol fields. Only `actor`, `authoritySource`, `reason`, and
  `timestamp` remain human-owned (an incomplete context is listed explicitly
  in `humanInputRequired`). It never writes `.ai/decisions` and never invents
  a PR number, authority, or human decision.
- For a first-record direct merge, the emitted `suggestedReceipt` is complete
  protocol input. After adding only the fields listed in `humanInputRequired`,
  pass it unchanged to either `work-item finalize-recovery` (the explicit
  historical entry point) or `work-item finalize` (the ordinary recorder).
  Both entry points must accept the same identity-bound bytes. If a Runtime
  rejects its own plan output, preserve the failed input outside `.ai/` and
  report the Runtime defect; do not reconstruct the receipt or invent a PR.
- `migrate plan --repo <path>` remains schema-compatible when the repository is
  current, but now also reports `historicalFinalization`. A stale receipt with
  a valid bound close is `historical_verified`/`historical_low`; a pending or
  unreadable receipt is `recovery_required` or `invalid` with safe actions.
  Historical discovery is separate from schema migration and does not rewrite
  predecessor bytes.
- `finish`, `archive`, and `close` preserve their lifecycle JSON on stdout and,
  by default, render the same validated human Outcome on stderr. Add `--json`
  for machine-only output. When `finish` is blocked, the CLI first renders the
  persisted red or yellow Outcome and then returns the original nonzero error;
  it never turns a failed gate into success. The CLI cannot force an embedding
  Agent or UI to open or expand a conversation panel. A host must surface the
  stderr handoff, or replay it deterministically with `work-item outcome`.
- `archive` and `archive-historical` always prepare the complete full Outcome.
  If `AI_COCKPIT_OUTCOME_HOST_PROGRAM` names an executable host adapter, each
  segment is sent as a JSON `assistant_message` request and the adapter must
  return the actual per-message receipt. Without that environment setting the
  result is explicitly `full_handoff_only` with unknown host acceptance and
  display. After an interrupted send, `work-item outcome --delivery` retries
  the same archived delivery from its identity-bound progress record; it never
  archives or verifies again.
- `work-item status --repo <path> --id <id>` is read-only and reports lifecycle,
  governance, activity health, fact counts, blockers, unknowns, evidence, and
  source digests. It never schedules work or invents a percentage.
- `work-item inspect --repo <path> --id <id>` is a read-only projection of
  compatibility, implementation approach, and parallel slots. It computes the
  approach without creating or refreshing `.ai/work-items/active/<id>.approach.json`.
  The explicit `work-item approach` command remains the write boundary when a
  repository-local approach artifact is required.
- An archived Work Item without a valid close decision is a lifecycle blocker,
  not a completed item. Its `safeActions` explicitly identify the remaining
  handoff: resource-bound items require `finalize_resources` or
  `cleanup_resources`, `record_finalization`, `finalize_verify`, and then
  `close_after_cleanup` (or `close` after a verified Deleted receipt); items
  without external resources require `close_after_review`. Agents must follow
  these actions and must not start another Work Item until the predecessor is
  closed or explicitly recovered.
- Top-level `status` includes a deterministic `readiness` object. Its
  `readyOnBase: true` claim is limited to a clean named branch at the single
  locally discovered remote default revision, with no active Work Item and no
  archived item awaiting close. Missing or ambiguous remote metadata is
  `state: unknown`, never green; `blocked` lists the exact entry blockers and
  `unclosedArchivedWorkItems` lists the records that must be closed or
  explicitly recovered.
- `work-item status --repo <path> --all --json` aggregates active and archived
  items in stable ID order. It reports fixed green/yellow/red/unknown counts,
  member diagnostics and digests, the current repository snapshot digest, and
  a deterministic index digest. A malformed or foreign member becomes an
  explicit unknown entry while the other members remain visible. This dynamic
  counterpart does not write `.ai/cockpit/work-items/index.json` or per-item
  status files. MCP clients use `work_item_status` with `{"all": true}`.
- `capability show --repo <path>` emits a Runtime- and repository-bound registry.
  Observed technical capability, profile confirmation, repository binding,
  adopter acceptance, and external ownership are distinct states. File
  presence alone never proves `adopter_accepted`; missing, malformed, stale, or
  foreign input remains unknown. MCP clients use `capability_show`.
- Repeated `observe`, `capability show`, top-level `status`, and single/all Work
  Item status calls do not write tracked repository bytes or observer caches.
- `work-item validate --repo <path> --id <id> [--json]` is a read-only unified
  Contract/Summary check for scenario coverage, stable acceptance evidence,
  intent alignment, and an optional final-dimensions receipt. `work-item
  controls --repo <path> --id <id> --input <json>` records only the explicitly
  supplied projection fields, including `evidenceClasses` and the
  identity-bound `decisionEvidence` review receipt; it cannot change lifecycle
  state, Contract facts, or verification receipts. `evidenceClasses` is the
  explicit mapping for each non-built-in Contract evidence class. Every class
  must name a regular, non-symlink repository file, its locator, verification
  state, and the file SHA-256, all bound to the current Contract digest. Missing,
  changed, malformed, or foreign evidence is rejected or reported stale; a
  scenario marked verified alone never satisfies a custom evidence class.
  `record_governance_controls` is a Runtime-admitted write action for an active
  Work Item. If verified work still lacks required controls, status withholds
  `finish` and recommends recording the missing evidence; refresh status after
  the write to obtain the next action.
- `work-item recover --repo <path> --id <id> --input <receipt.json>` records an
  identity-bound `retry`, `successor`, or `supersede` decision. `supersede`
  requires an already-bound successor Work Item and archives the predecessor
  as an explicit historical `superseded` state without rewriting its original
  bytes. The receipt must bind the
  predecessor Contract, Summary, Outcome, and event digests when those records
  exist, plus the current Runtime identity. Existing receipts are preserved;
  later decisions use digest-suffixed files. A recovery receipt does not make
  verification green or silently rewrite the predecessor. A superseded
  predecessor is neither a current pass nor a current failure; its successor
  owns follow-up. Outcome and archive consumers revalidate every current
  candidate's regular-file/filename boundary, repository and current Runtime
  identity, predecessor digests, timestamp, decision shape, and successor
  Contract binding. Invalid or ambiguous candidates fail closed as
  `recovery_decision_invalid`; historical archive bytes and projections remain
  immutable.
  A retry whose predecessor digest no longer matches the fresh archived
  Contract/Summary/Outcome/Events is consumed history, so the static gate
  projects the real finalization path instead of inventing a recovered
  terminal state; matching blocked retries remain fail-closed recovery.
- `work-item revalidate-archived --repo <path> --id <predecessor> --successor <id>
  --reason <text> --actor <id> --authority-source <source>
  --resume-condition <text> [--evidence-ref <ref>] [--policy-ref <ref>]` is the
  first-class append-only path for a reviewed Contract amendment after archive.
  It verifies the current archive manifest, preserves and digest-binds the
  historical verification evidence, and creates a `not_ready` successor while
  the predecessor remains pending close. The successor must complete its own
  start, verification, archive, finalization, and explicit close before the
  predecessor can close. It never rewrites predecessor Contract, archive,
  Outcome, Events, or verification evidence; malformed, foreign, stale, or
  contradictory history fails closed.
- When the predecessor already has a valid provider PR finalization receipt
  from an older Runtime, a terminal Contract-amendment successor is also the
  supported cross-version close path. After the successor's current
  verification, finalization, and human close are bound, predecessor `close`
  records `historicalRevalidation` with `assurance=historical_low` and binds
  the old receipt's exact path, digest, and sequence. The old receipt bytes
  remain immutable and keep their provider/PR identity; they are never
  reclassified as `direct_merge_no_pr`. Without a resolved successor, or when
  any archive, Contract, evidence, receipt, or lineage binding is malformed,
  foreign, stale, or contradictory, the current Runtime identity check still
  fails closed.
- `profile propose --repo <path>` is read-only and reports a `candidate`/
  `proposed` amendment. It never applies a profile baseline change.
- `agent list --repo <path>` is read-only. `agent install` is the only normal
  adapter write and requires `--provider`; `auto` is accepted only when one
  unambiguous safe surface exists (an `AGENTS.md` surface selects Codex by
  default). `agent doctor --repo <path> --json` returns the strict
  state report and uses exit codes 0 (verified), 1 (degraded), 2 (configuration
  error), and 3 (human intervention). `repair` and `detach` fail closed when
  the managed section or ownership record changed. No command writes global
  Agent or MCP configuration.
- `preflight --contract` normally points to
  `.ai/work-items/active/<id>.contract.json` generated by `start`.
- `inspect`, `status`, `doctor`, and `work-item outcome` are read-only queries.
  `preflight` is an explicit governance write entry: it persists the evaluated
  decision/projection, reports paths it changed as `changedPaths`, and reports
  an empty list when identical inputs produce no write. It does not run
  verification or repair work.
- `work-item new` creates a `not_ready` skeleton. Running `preflight` on it is
  intentionally yellow with `reviewState: needs_human_confirmation`; fill the
  human fields and rerun preflight before checkpoint.
- `close --human-decision approved|confirmed|rejected` is a human decision
  record, not verification evidence. The Runtime accepts only the canonical
  tokens `approved`, `confirmed`, `rejected`, `superseded`, and
  `superseded_failed_delivery`; free-form explanation belongs in `--reason`.
  `approved` and an explicit `confirmed` decision are positive terminal
  choices; `rejected` never promotes a Work Item to Implemented, while the two
  superseded tokens are reserved for explicit historical or abandoned-delivery
  close paths.
- `evidence import --repo <path> --work-item <id> --metadata <metadata.json>
  --raw <provider-output>` verifies the strict `DelegatedEvidence` metadata
  against the exact raw-byte digest and writes a repository/Work Item-bound
  receipt under `.ai/evidence/external/`. `evidence list` revalidates those
  receipts; it does not turn expired or revoked provider claims into authority.
- `evidence policy --repo <path> --work-item <id> --classification <value>
  --persistence <value> --retention-days <n>|--expires-at <timestamp>
  --disposal-action <action>` binds a strict retention policy. `secret_prohibited`
  rejects `full_capture` and `redacted_capture`; `digest_only` omits raw command
  output; `no_persistence` fails closed when completion evidence would otherwise
  be written. `evidence purge-plan --repo <path>` emits a stable plan and never
  deletes evidence by itself.
- `audit export --repo <path> [--output <file>]` emits stable `AuditEvent` records
  with event IDs, subject digests, repository/Work Item identity, and Runtime
  identity. The manifest sets `externalRetentionRequired: true`; an output file
  is idempotent and is only a handoff to SIEM, WORM, S3 Object Lock, or another
  external retention owner.
- `audit cognitive-benefit --repo <path> [--binary <path>] [--check]` runs the
  fixed seven-case evaluation in Rust. `--check` is read-only with respect to
  evaluation artifacts; without it, the command writes the JSON report under
  `.ai/evidence/` and Markdown under `.ai/evidence/external/` in the selected
  repository. An explicit `--binary` must name an existing executable file;
  an invalid selection fails without falling back to another Runtime, building
  with Cargo, or writing evaluation artifacts. Missing files and Unix files
  without execute permission are rejected before evaluation; other launch
  failures are reported when the selected file is invoked. This is an
  intentional safety difference from the former Python evaluator's fallback;
  valid explicit binaries retain output parity. Python remains an oracle for
  parity tests, not the repository gate. No participant study or measured
  cognitive benefit is implied.
- Task Outcome reports are strict typed JSON projections. Every claim carries
  evidence references when available; an explicitly marked inference is not a
  verified fact. The event stream is append-only for a Work Item finish and is
  validated for repository/Work Item identity, ordering, safe detail content,
  and evidence-reference boundaries. Archive manifests bind the event stream
  and report JSON/Markdown digests; close receipts include the final report and
  its digest.
- For an auditable decision, add `--actor`, `--authority-source`, `--reason`,
  `--decided-at`, and optional repeated `--evidence-ref`, `--policy-ref`, and
  `--resume-condition`. The resulting `structuredDecision` is stored under
  `.ai/decisions/<id>.close.json`; the legacy flag remains explicit and is
  recorded with visible `legacy-cli` provenance.
- `compatibility --repo <path>` reports `COMPATIBLE`, `MIGRATION_REQUIRED`, or
  `INCOMPATIBLE` for the installed Runtime and attached repository schema.
  `migrate plan` is read-only. `migrate apply` refuses to write unless
  `--approved` is present and never rewrites Work Items, evidence, decisions,
  knowledge, or archive history.
- Once all attached protocol files exist, stateful governance commands require
  `COMPATIBLE`; `MIGRATION_REQUIRED` and `INCOMPATIBLE` fail closed before a
  new Work Item, lifecycle record, verification evidence, profile/adapter
  write, or governed MCP operation is created. Read-only diagnostics remain
  available for migration review.

## Pre-verification projection and failure recovery

When a repository declares the tri-language reference-parity convention,
create the new Work Item's three regular, non-symlink pages under
`docs/work-items/` and exactly one row in each `docs/reference/reference-parity*`
ledger before `preflight` and before `checkpoint`. Each prearchive page must
declare `status: in_progress`, the matching `workItemId`, and
`lastVerifiedBy`; terminal fields belong only to post-close promotion. Runtime
checks this condition at `preflight`, immediately before a project verification
process, and before writing a close decision. A missing or malformed page or
row reports its exact path and reason and starts no expensive project process;
repositories without this convention retain the generic lifecycle route.

A failure does not by itself authorize a successor. Preserve the attempt and
repair the current Work Item when the same root cause remains inside its
Contract scope, authority, and base: amend/revalidate the Contract, then use a
bound `retry`. Create a successor only for a genuinely different scope,
authority, or base, an independent change, an unsafe in-scope repair,
immutable failed delivery, or explicit human direction. If classification is
uncertain, stop for human direction; do not open a new Work Item merely to
avoid the current lifecycle cost. A successor must be created through
`work-item recover` with predecessor bindings, never through a bare `start`.

## Post-close documentation promotion

After structured `close`, run:

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item <id>
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```

The first command validates the exact regular archive Contract and raw digest,
passing verification receipt, unique linear finalization chain, sequence-2
`deleted` head, merged provider identity, and approved close bindings before it
updates controlled documentation fields. Only `status`, `lastVerifiedBy`, the
four `terminal*` frontmatter fields, and the exact tri-language parity rows are
write targets. The second command is the mandatory quality/terminal-CI form;
it never writes. Missing, foreign, ambiguous, malformed, symlinked, mismatched,
or stale input fails closed. These repository helper commands do not imply
that Runtime Core automatically edits documentation.

## Contract/Summary control validation

The repository library exposes `validate_work_item_governance_controls` for
Agent/MCP adapters that need one stable report covering scenario coverage,
acceptance evidence, intent alignment, and an optional final-dimensions
receipt. The validator is read-only. It reports `blocked` or `unknown` rather
than filling missing fields. The final receipt uses the exact twenty
reference dimensions; `fourPillarProjection` is an explicitly named optional
view and `4D` is not a protocol field.
When an adapter supplies the current Runtime context, the validator also
requires matching `runtimeVersion` and `runtimeDigest`; the standalone value
helper only guarantees non-empty/versioned digest shape.

## Runtime identity

`inspect`, `doctor`, MCP `initialize`, and verification evidence expose runtime
version, runtime digest, and protocol version. `ai-cockpit --version` is only the
short executable version string.

## Release acceptance boundary

`tests/release/adopter_acceptance.sh` is a maintainer-side post-release harness,
not a Runtime command. It downloads and pins a public Release binary, runs the
adopter lifecycle in isolated directories, and emits `acceptance.json` and
`SHA256SUMS`. It must not be replaced with a workspace build or local target
binary, and a failed acceptance never changes the published Release truth.

The lifecycle portion is intentionally complete: `finalize-plan` precedes
verification, and ordinary archived Work Items must pass `finalize` and
`finalize-verify` with a deleted head before structured `close`. Historical
shared-worktree or direct-merge receipts may use the documented low-assurance
retained exception after their Git facts are verified; retained never
authorizes a new Work Item close.

The Runtime rejects `finalize-plan` once the Work Item has reached
`finish_ready`; bind the resource context before verification so a late plan
cannot invalidate an already recorded verification cycle. A checkpointed item
may still bind a provisional context before verification, as an explicit
recovery/setup step. The sentinel form `pending:<stable-reference>` is provisional just like
`unknown`; it is not a provider-bound resource and must be replaced before `finish` or `archive`.

The Runtime-aware entrypoint also supports one bounded repair when a valid Work
Item was archived before its reviewed resource was bound. It appends a
digest-bound `.ai/decisions/<work-item>.resource-context.json` record after
validating the immutable Contract and archive manifest. The archive bytes are
not rewritten, replay with the same context is idempotent, and the record does
not authorize `finalize`, cleanup, or `close` by itself.

The acceptance receipt also records typed before/after manifests for every
isolated root. `HOME` and `XDG_CONFIG_HOME` have empty `allowedPrefixes` and
must remain unchanged; `TMPDIR` and `CARGO_HOME` are the only Runtime-write
roots, and their allowlists are explicitly limited to `<TMPDIR>/**` and
`<CARGO_HOME>/**`. Cleanup status is recorded in `cleanup.json` and the
`cleanupState`/`cleanupError` fields; cleanup failure is a failed acceptance,
not an unpublish or rewrite of Release truth.

`tests/conformance/final_replacement_acceptance.sh` is the source-repository
replacement boundary. It records installed Runtime identity, the locked
reference oracle, conformance/adversarial/performance gates, and the no-copy
check, then emits `acceptance.json` and `SHA256SUMS`.
