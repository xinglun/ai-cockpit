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
`deleted`; a retained, blocked, or unknown head stops the operation before a
close decision is written. A verified historical `shared_worktree_retained` or
`direct_merge_no_pr` receipt is the narrow compatibility exception: it may keep
the primary worktree and close only with `assurance=historical_low`, explicit
human authority, and repository-bound Git facts. This exception never applies
to new Work Items and never upgrades historical evidence to provider assurance.
For immutable records produced by an older Runtime, `work-item finalize` accepts
one strictly bound deleted transition after close as a legacy reconciliation.
The transition must bind the closed root path and digest and is verified as an
append-only cleanup observation; it never rewrites the close receipt.

`work-item finalize` stores the first receipt at `.ai/decisions/<id>.finalize.json`. For an ordinary PR receipt its base must equal the archived Contract's immutable `baseRevision`; both recording and `finalize-verify` reject a mismatch, including sequence 0. The narrow historical `direct_merge_no_pr` exception binds `pullRequest.baseRevision` to the real merge commit's first parent and records the immutable Contract base separately as `historical.contractBaseRevision`; the receipt's `contractDigest` still binds the exact archived Contract. `finalize-recovery-plan --merge-commit <sha>` derives both values, so a bundled historical merge can be recorded without changing either fact or inventing a PR. Rebase before archive requires a renewed active Contract binding; rebase after archive requires recovery and never permits receipt or archive rewriting. If that immutable root exists, a typed transition envelope must bind the unique head's predecessor digest and next sequence; Runtime appends `.finalize.<digest>.json`. `finalize-verify` reports `headPath`, `headDigest`, and `sequence`, which `close` binds. A sequence-1 merge observation may additionally bind `governanceAppendRevision` when the receipt commit advanced all aligned heads. Runtime requires an ancestor range of additions only. Besides regular same-Work-Item finalization receipts, the only permitted evidence additions are the complete fixed-schema pair `.ai/evidence/<id>/quality-route-post-finalize.json` and `.ai/evidence/<id>/repository-gates-post-finalize.json`; every path must be an `A`-only `100644` regular blob, and their archived Contract, PR revision, route digest, manifest, profile, and passing gate bindings must agree. The pair is evidence, not authority, and does not replace the required finalization receipt addition. This does not permit arbitrary evidence paths or archive mutation.

All repository commands accept an explicit `--repo <path>`. Commands that
produce records or decisions keep JSON on stdout. `finish`, `archive`, and
`close` additionally emit the localized human handoff on stderr by default;
their `--json` option suppresses only that handoff. `work-item outcome` emits
the localized human handoff on stdout by default; add `--json` for the stable
machine-readable `OutcomeV2`. A failed or unknown decision is not a pass.

| Group | Commands | Boundary |
| --- | --- | --- |
| Read-only | `inspect`, `observe`, `status`, `compatibility`, `migrate plan`, `capability show`, `diagnose`, `doctor` | Read repository state or derived evidence; no silent repair. |
| Derived projection | `knowledge query` | Explicitly materializes or reuses a repository-local `.ai/knowledge/` index; reports `projection.writeBoundary=repository-local-derived` and never changes governance authority. |
| Setup | `attach`, `profile confirm`, `profile propose` | Create/update protocol state, confirm a profile, or emit a read-only candidate. |
| Migration | `migrate apply --approved` | Apply only the reviewed repository-schema migration and write a runtime-bound migration receipt. |
| Governance | `preflight` | Read a Contract and return a green/yellow/red decision plus `reviewState`; incomplete or uncertain Contracts are human-review yellow and cannot cross checkpoint. |
| Work Item | `work-item new`, `start`, `status`, `checkpoint`, `finish`, `archive`, `close`, `validate`, `controls`, `recover`, `revalidate-archived`, `finalize-recovery` | Read a request-scoped status projection or write explicit lifecycle records; `close` and recovery require explicit human decisions. |
| Parallel Work Item | `work-item boundary`, `work-item declare`, `work-item slot acquire|release|list` | Bind Contract-owned concurrency paths and reserve repository-local slots; unknown boundaries serialize. |
| Verification | `verify` | Execute bounded commands, record evidence, and optionally bind it to a Work Item. |
| External evidence | `evidence import`, `evidence list`, `evidence policy`, `evidence purge-plan` | Bind exact provider bytes, declare bounded persistence, or produce a deterministic non-destructive disposal plan. |
| Audit | `audit export` | Produce a stable repository-bound event bundle for an external retention owner; never claim local immutability. |
| Adapter | `agent list/install/doctor/repair/detach`, `mcp` | Manage an explicitly selected repository-local Agent adapter or serve JSON-RPC over stdio; every operation binds `--repo`. |

## Important options

## MCP tool usage

Agents should discover the surface in this order: start the repository-bound
stdio server, call `initialize`, call `tools/list`, then call one tool with the
schema-described arguments. Every `tools/call` is repository-bound and rejects
unknown fields, wrong types, missing required fields, and conflicting aliases
before any repository operation runs.

| Tool | Arguments | Typical call |
| --- | --- | --- |
| `status`, `work_item_list`, `repository_observe`, `capability_show` | `{}` | Read repository facts or the capability registry. |
| `work_item_get`, `work_item_outcome`, `work_item_validate` | Exactly one `workItemId` (or legacy `id`); `work_item_outcome` optionally accepts `language` (`en`, `zh`, `ja`). | `{"workItemId":"WI-123"}` |
| `work_item_status` | `{"all":true}` or exactly one Work Item id. | `{"all":true}` |
| `preflight` | Required repository-relative `contract`. | `{"contract":".ai/work-items/active/WI-123.contract.json"}` |
| `blockers`, `safe_actions` | Optional repository-relative `contract`. | `{"contract":".ai/work-items/active/WI-123.contract.json"}` |
| `knowledge_query` | Optional `topic`, `component`, `state`, `workItemId`. | `{"topic":"verification"}` |
| `evidence_get` | Exactly one of `path`, `evidencePath`, or `id`. | `{"id":"WI-123"}` |
| `delegated_evidence_list` | Required `workItemId`. | `{"workItemId":"WI-123"}` |
| `work_item_controls`, `work_item_recover` | Exactly one Work Item id plus exactly one object: `controls`/`input`, or `receipt`/`input`. | `{"workItemId":"WI-123","controls":{...}}` |
| `verify` | Optional `workItemId`, `command`, and string-array `args`; command is allowlisted. | `{"workItemId":"WI-123","command":"cargo","args":["test","--locked","--workspace"]}` |
| `work_item_parallel` | `action`: `inspect`/`acquire`/`release`/`list`; inspect/acquire/release require an id, release also requires `leaseId`. | `{"action":"inspect","workItemId":"WI-123"}` |

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
- `verify` without `--command` detects Cargo or npm and may use a confirmed
  profile for cross-process reuse. Reuse is admitted only when the current
  repository, snapshot, profile, runtime, command, scope, stage, runner, base,
  toolchain, dependency, and policy identities match exactly. Otherwise the
  declared command executes and the result reports the denial/escalation reason.
  Required and protected nodes are never skipped by timing or cache state.
- `verify --workers <n>` requires a positive worker count and caps concurrency.
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
- Before `start` or `work-item new`, the Runtime applies a repository-scoped
  entry gate. A non-`.ai` working-tree change, detached HEAD, a known HEAD
  mismatch with the locally discovered remote default ref, or an archived Work
  Item without a valid close decision fails closed. The gate never rewrites
  archived bytes. A recovery successor is an explicit continuation and may be
  scaffolded through `work-item recover`; it is not an independent next item.
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
  partial `pullRequest.number=0`/`historical://direct-merge/<sha>` receipt
  skeleton. `pullRequest.baseRevision` is the real merge first parent;
  `historical.contractBaseRevision` is the archived Contract base. Both are
  deterministic facts and must be preserved. It also reports the archived
  Contract digest and any provisional context; receipt IDs, branch/worktree
  facts, disposition, authority, reason, and timestamp remain human-owned. It
  never writes `.ai/decisions` and never invents a PR number, authority, or
  human decision.
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
  supplied projection fields, including the identity-bound `decisionEvidence`
  review receipt; it cannot change lifecycle state, Contract facts, or
  verification receipts.
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
- `work-item new` creates a `not_ready` skeleton. Running `preflight` on it is
  intentionally yellow with `reviewState: needs_human_confirmation`; fill the
  human fields and rerun preflight before checkpoint.
- `close --human-decision approved|confirmed|rejected` is a human decision
  record, not verification evidence. `approved` and an explicit `confirmed`
  decision are positive terminal choices; `rejected` never promotes a Work
  Item to Implemented.
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
