---
author: AI Cockpit maintainers
title: "Human-facing Outcome"
description: "The human handoff emitted from a Work Item Outcome."
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-781-trust-diagnostics
capabilityClaims:
  - human_outcome_handoff
---

# Human-facing Outcome

`ai-cockpit work-item outcome --repo <repository> --id <work-item>` emits a
reader-first human summary by default. Use `--view full` to open the complete
audit-oriented handoff, or use `--json` when a machine needs the stable
`OutcomeV2` object. The summary and full view are projections only; neither
changes the stored Outcome or its governance meaning.

The first line is always `Outcome: 🔴/🟡/🟢 ...`. For example, green is rendered as
`Outcome: 🟢 Declared verification passed`, not as a generic success claim. The handoff is returned directly
by CLI stdout and MCP `content[0].text`, so an Agent or UI must not hide it in a
collapsed log. `work_item_status` is the separate read-only status projection.
Its lifecycle phase is `archived` after archive and becomes `closed` only when
the repository-bound, confirmed close decision is valid; an invalid or missing
decision never promotes an archive to `closed`.

Top-level `finish`, `archive`, and `close` keep their existing lifecycle JSON
on stdout and render this same validated report on stderr by default. Their
explicit `--json` mode suppresses the stderr report for machine-only callers.
If `finish` is blocked, it renders the persisted red or yellow Outcome before
returning the original nonzero error. This extra handoff never weakens a gate.
The CLI cannot force a host application to open or expand a conversation UI;
hosts must surface stderr, and a person can replay the durable handoff with
`ai-cockpit work-item outcome --repo <repository> --id <work-item>`.

The default summary has four sections:

1. Result: current verification, lifecycle, human decision, and governance signal
2. Key changes: evidence-backed delivered changes
3. Remaining uncertainty: blockers, risks, limitations, unknowns, and missing benefit declarations
4. Human next step: the decision needed and why, or an explicit statement that no new decision is required

The full view retains the audit-oriented order:

1. Task Result plus separate Verification, Lifecycle, Human decision, and Governance signal status
2. What was completed
3. Problems found
4. Stops triggered
5. Problems resolved
6. Risks avoided
7. Remaining risks
8. Unknowns
9. Human decisions
10. Verification and evidence
11. Impact
12. Next action

Empty sections are omitted from the summary when they do not carry a decision-relevant
fact. Blockers, pending human decisions, invalid or expired evidence, historical
classification, and unknowns are never omitted for length. Other lists are not
silently truncated; use the full view when a complete list is needed. Summary
claims remain evidence-bound data and raw evidence text is never interpreted as an
instruction or authorization source.

## Release note: reader-first Outcome summary

The current presentation release changes the default human `work-item outcome`
view from the complete audit report to the four-part summary above. This reduces
empty-column reading for ordinary tasks while keeping verification, lifecycle,
decision, blockers, uncertainty, and evidence references visible. The complete
report remains available with `--view full`, and the MCP `work_item_outcome` tool
accepts `view: "summary"` (default) or `view: "full"`. Existing machine-readable
JSON fields, validation rules, exit codes, authorization semantics, and persisted
evidence remain compatible; optional reason/finalization fields are additive.
Top-level `finish`, `archive`, and `close` continue to render the
complete handoff on stderr for lifecycle error and audit context; `--json` still
suppresses that human channel.

Status markers are decision signals, not release authorization:

- `🟢` verified evidence is present; review evidence before proceeding.
- `🟡` the result is partial, not ready, or unknown; repair or investigate.
- `🔴` a required control failed; inspect the displayed reason projection and stop until the actual gap is resolved.

Empty data is not treated as a positive fact. Risk findings are rendered as
`Not recorded` or `Not assessed` when the evidence cannot support a stronger
statement; only an explicit evidence-backed claim may say that no risk was
found within a named check scope. Other empty sections use `Not recorded`.
The report never fills a governance decision from inference. A green result
does not authorize merge, release, publication, or a security claim.

A red marker is not itself an evidence diagnosis. The Runtime derives stable
reason keys from the existing failed gate, unknowns, governance-control
findings, scope/authorization rules, and evidence state. It distinguishes
verification failure, invalid/expired evidence, missing acceptance evidence,
intent misalignment, scope or authorization gaps, and unknown causes. Summary
and full views share this projection, and the CLI/MCP human handoff preserves
the same semantics across `en`, `zh`, and `ja`.

The status lines deliberately keep four dimensions separate:

- Verification describes the `OutcomeState`, for example `Declared verification passed`.
- Lifecycle describes the current active, checkpointed, finish-ready, archived, or closed projection.
- Human decision says `Not recorded`, `Recorded: <decision>`, or `Unknown`; it is not inferred from verification.
- Governance signal is a green/yellow/red signal and explicitly says it is not a human approval.

A valid structured human decision also shows its actor, authority source,
evidence and policy references, and assurance level. The assurance level is
`Unknown`/`未知`/`不明` when the record contains no such fact; the renderer never
upgrades the authority source or evidence into a higher assurance level.

Test-weakening output is scope-limited. A message that a weakening rule was not
triggered is rendered as a result for the referenced check scope and explicitly
does not prove that tests were not weakened.

The green marker is emitted only after the Runtime validates a complete, fresh,
identity-bound `evidenceSchemaVersion=2` verification receipt for the current
Work Item and repository. Missing or stale evidence is yellow. Tampered,
malformed, identity-mismatched, or digest-inconsistent evidence is red. The
same fail-closed validation is enforced by `finish`, `archive`, and `close`; an
evidence path existing is never sufficient. Legacy evidence is not rewritten as
green and must be regenerated by a fresh verification. The current CLI binds
`runtimeVersion` and `runtimeDigest` to the executable performing
`verify`/`finish`/`archive`/`close`; a well-formed receipt from another Runtime
is therefore rejected. The v2 envelope and captured receipt reject unknown
fields and require nested Work Item, repository, and Runtime identity. A
digest-only retention record has no captured receipt to validate. A readable
pre-v2 record (identified by the absence of `evidenceSchemaVersion`) is
projected as yellow `legacy_evidence_historical`: it is historical input, not
a current failure or a fresh green result. A v2 record with missing identity
remains red.

Archived v2 evidence produced by an older Runtime is projected with a yellow
historical marker and `historical_evidence_not_revalidated`. The handoff must
not add `verification_or_human_input` or a missing-evidence recovery gate: the
bytes are valid historical context, not a current verification failure. A new
verification is required only when a current result is needed.
The machine projection uses `historicalStatus: "runtime_historical"`; this status
must also suppress human-facing missing-evidence and recovery wording.

The v2 envelope `createdAt` and retention `createdAt` must be RFC3339 timestamps.
Optional retention `expiresAt` accepts RFC3339 or the retained epoch-seconds
compatibility form. A malformed or semantically invalid
timestamp is evidence corruption: Outcome is red and `finish`, `archive`, and
`close` stop. This check protects both current evidence and retention metadata;
it does not rewrite historical bytes.

Acceptance criteria, intent, and scope are governance source text authored by
the Work Item owner. The report preserves them under “Acceptance criteria
(contract language)” and never silently translates or changes Contract bytes.
Runtime-generated headings, summaries, statuses, unknown codes, and recovery
hints are localized to the conversation language.

When a predecessor has an explicit `supersede` recovery decision, Outcome
includes `historicalStatus: "superseded"` and uses a yellow historical marker.
This means the original evidence is preserved and is not being revalidated as
the current result; it is not a red failure and not a green authorization.

For an ordinary archived Work Item with a bound resource context, finalization
is projected from the Runtime's receipt validator and resource observations.
Missing receipts, corrupt records, identity mismatches, pending cleanup,
retained resources, and verified deletion have distinct states and actions.
Missing or invalid facts instruct the Agent to inspect or re-observe and never
to repeat deletion; a retained disposition never becomes a delete suggestion;
verified deletion only leaves the Runtime's close decision. Human wording is
not an authorization source. An archived Work Item remains non-terminal until
the Runtime accepts finalization and its explicit close decision.

Outcome assembly captures the repository snapshot and relevant lifecycle,
decision, evidence, and finalization records in one request-scoped observation
boundary. A deterministic change during assembly causes at most one retry;
continued change is returned as explicit unknown/failure. The pure renderer
performs no I/O and never extends the captured context across execution or
persistence stages.

`ai-cockpit diagnose` reports Runtime-internal primary-path phases for identity,
Git/snapshot, read/hash, parse, governance, and projection/serialization. It
also reports actual scoped read/hash bytes and Git calls. Unsupported child
process counts are marked unavailable rather than zero, and benchmark-tool
overhead is kept separate from these Runtime measurements.

The CLI uses `AI_COCKPIT_LANGUAGE`, then the process locale, for direct human
output. Agent conversations should render the same handoff in the language of
the user. JSON field names and enum values remain stable across languages.

## MCP human handoff

When an Agent needs to show a result to a person, it must call the
repository-bound `work_item_outcome` tool with an explicit `workItemId`. Its
text content is the same localized handoff rendered by the CLI, not a raw JSON
dump. `structuredContent.outcome` remains the stable OutcomeV2 object;
`humanHandoff` is only a presentation projection and cannot authorize a merge,
release, or decision. `work_item_get` remains a machine record lookup. The
optional `language` selects `en`, `zh`, or `ja` for Runtime-generated labels;
Contract source text remains unchanged.

## Task Outcome report and events

Newly generated OutcomeV2 records also contain a strict `taskOutcomeReport`.
Its sections are evidence-bound and may be empty; an empty section is not a
success claim. Claims without a repository-local evidence reference must carry
`inference: true`. The report includes `failedGate` and `recoveryCondition`
when a required control is yellow or red.

When `finish` is blocked, the active Work Item keeps its checkpointed
lifecycle state and receives an active `state: "blocked"` Outcome projection.
That projection is repository- and Work Item-bound, has `decisionState: "red"`,
and names the failed gate and deterministic recovery condition. A later valid
retry appends a completion event; it does not rewrite the earlier blocked event.
Malformed, foreign, symlinked, or unknown event records fail closed.

When a retry follows a failed `finish` projection, Runtime restores the active
Summary to `checkpointed` through the identity-bound recovery receipt. It does
not make the blocked Outcome green; `verify` and `finish` must produce a fresh
current Outcome before archive or close.

`finish` writes `<id>.events.jsonl` beside the active outcome. The stream is
append-only and rejects malformed, foreign, secret-like, or relationship-invalid
events. When a Work Item is archived, generated report references and
`changedPaths` are projected from `.ai/work-items/active/` to the corresponding
`.ai/work-items/archive/` paths before the archive manifest is bound; the
resulting `eventsDigest` and report digests cover those archived bytes.
`close` validates the projected stream and records `finalReport` plus
`finalReportDigest` in the close receipt. Existing historical archive bytes are
never rewritten or backfilled; only a newly created archive receives this
active-to-archive projection.
