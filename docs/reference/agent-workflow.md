---
author: AI Cockpit maintainers
title: "Agent workflow and review boundaries"
description: "Repository-local operating rules inherited by future AI Cockpit Work Items."
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - agent_workflow_boundaries
---

# Agent workflow and review boundaries

This page is the repository-specific projection of the applicable operating
rules from the reference source. It keeps the governance intent while using
the installed Rust Runtime and this repository's Protocol vocabulary.

## Rules inherited

- Start from the latest commit on the repository's discovered remote default
  branch. Record the remote, default branch, and base revision in the Work Item
  Contract.
- The canonical delivery order is latest remote default base → dedicated
  branch/worktree → implement → finish/archive → push → reviewed PR → merge →
  close → synchronize and clean. Never pre-merge a feature branch into local
  `main`, delete its branch before merge, or let a provider auto-delete it to
  bypass finalization. A failed remote step preserves the retry checkout and
  identity; `ready_on_base` is true only after reviewed merge, synchronized
  default branch, and exact cleanup, not for a detached worktree.
- Use one Contract, one dedicated branch/worktree, and one PR per Work Item.
  Compatible independent Work Items may run concurrently when scope, evidence
  ownership, repository context, and serialized projections are isolated.
- Before `start` or `work-item new`, read the top-level `status.readiness`
  projection. A normal next Work Item is rejected when an archived item lacks
  a valid close decision, the worktree already contains non-`.ai` changes, the
  HEAD is detached, or a discovered remote default base does not equal HEAD.
  `readyOnBase` is the only positive readiness claim; missing or ambiguous
  remote metadata is `unknown`, never an implicit green. Recovery successors
  remain an explicit continuation of their predecessor.
- Before editing, read `.ai/README.md` and `.ai/glossary.md`; query `inspect`,
  `status`, and `doctor`; keep edits inside the declared scope; preserve tests
  and evidence; update the Summary; and run the Contract's project checks.
- If `preflight` reports `not_ready` or `needs_human_confirmation`, pause and
  show the Preflight Review to the person. An advisory successful exit does not
  authorize implementation.
- A scaffold with empty intent, goal, scope, out-of-scope, acceptance, or
  authority is explicitly `yellow` with `reviewState:
  needs_human_confirmation`; it is never treated as ready. A yellow
  `verification_pending` result may be checkpointed to collect declared
  evidence, but a human-confirmation result may not cross the checkpoint.
- The pre-edit Contract review binds the repository, Work Item, Contract
  digest, and snapshot digest. Changing either after review requires a fresh
  preflight before checkpointing.
- When `reviewState` is `needs_human_confirmation`, preflight also returns a
  structured `humanDecisionRequest` containing what happened, why it matters,
  options, a recommendation, the question, and a resume condition. It is a
  human-facing request, not approval; an Agent must amend the Contract and
  rerun preflight rather than treating the request as authorization.
- A human may record the bounded review only through the repository-local
  `decisionEvidence` projection. The strict receipt binds `decisionId`, Work
  Item, repository, Contract digest, preflight decision digest, snapshot
  digest, actor, timestamp, and reason. A valid receipt may unlock the
  checkpoint transition; it never proves a test, scenario, verification, or
  release result. Missing, stale, foreign, malformed, or symlinked receipts
  remain stopped.
- Review receipts are append-only. When a Contract or repository snapshot
  changes, a fresh receipt is written under a digest-suffixed decision path;
  the previous receipt remains historical evidence and is never overwritten.
  `work-item recover` records a separate, strict `retry`, `successor`, or
  `supersede` decision bound to the predecessor Contract/Summary/Outcome/event
  digests and current Runtime. `supersede` requires an already-bound successor,
  archives the predecessor as an explicit historical terminal state, and keeps
  its original bytes unchanged. It does not make verification green or rewrite
  the predecessor; the superseded item is neither a current pass nor a current
  failure, and follow-up belongs to the successor.
- A required high-risk scenario that can only run after implementation may be
  marked `unverified` in Contract `scenarioCoverage` only when both a non-empty
  `expected` (or `expectedResult`) and a concrete `verificationPlan` are
  present. This is implementation planning evidence, not completion evidence;
  Summary scenario guards and `finish` still require executed evidence.
- Deliver a separate visible human Outcome with `Outcome: 🟢`, `Outcome: 🟡`,
  or `Outcome: 🔴`, unknowns, evidence, human decision, and next action. A
  missing, folded-only, stale, contradictory, or malformed Outcome fails
  closed and cannot authorize progression.
  Top-level `finish`, `archive`, and `close` keep stdout JSON stable and emit
  this handoff on stderr by default; `--json` is the machine-only form. A
  blocked `finish` emits its persisted red/yellow Outcome and still returns the
  original nonzero failure. Because the CLI cannot force a host conversation
  panel to expand, hosts must surface stderr or replay `work-item outcome`.
- A green Rust terminal is the reference equivalent of `status=completed` plus
  `humanStatusColor=green`; it additionally requires `state=Verified`,
  `decisionState=green`, current bindings, and direct human-visible delivery.
  The handoff reports issue count, blockers or stopping reason, resolved issues,
  risks, verification, impact, and next action. Facts require evidence and
  unsupported benefits are explicitly an inference.
- Resolve an in-scope defect in the current Work Item by amending and
  revalidating its Contract. Create a successor only when scope, authority, or
  base genuinely differs, the change is independent, safe in-scope repair is
  impossible, immutable failed delivery requires re-delivery, or a person
  explicitly directs it.
- Installation and upgrade acceptance use an immutable published Release tag
  and downloaded binary. After merge, closure verifies archived evidence,
  decision, merged PR head, synchronized default branch, clean worktrees, and
  exact branch removal; archived evidence is validated against its immutable
  archive manifest rather than reclassified as stale solely because the merge
  changed the current repository snapshot. A failed step remains open for
  recovery.

## Project-specific adaptation

The reference source contains `make ai-*` commands and a `contractVersion: 2`
template protocol. They are not commands or schema requirements here. This
Rust project uses the installed shared Runtime and the explicit lifecycle:

```text
start → preflight → checkpoint → verify → finish → archive → close
```

Every repository-bound command carries `--repo`. The Runtime has no global
current repository, Work Item, or project profile. Contract criteria remain in
their source language; only the human presentation layer is localized.

Before implementation, Contract review also checks declared intent, scenario
shape, acceptance statements, and the parallel boundary. A malformed scenario
list, duplicate scenario, empty acceptance statement, or invalid slot boundary
is a review finding, not a fact the Agent may repair by inference. Risk policy
decides whether scenario coverage is required; the Runtime remains yellow/red
until human-owned declarations and fresh evidence are present.

Agent Risk and checkpoint controls use the same Rust lifecycle validator.
Typed required verification declarations are consumed at preflight, verify,
finish, archive, and close; missing, duplicate, failed, or invalidated gates
never become permission through a presentation field. A typed
`checkpointPolicy` selects only Verification strength (`light`, `standard`,
`strict`, or `release`) and required stages/checks; it does not imply Evidence
Assurance. `work-item revalidate-amendment --repo <repo> --id <id> --reason <text>`
appends Contract-amendment evidence without replacing `before_edit`. After
verification it invalidates prior required checks and requires fresh preflight
and verification. Resume history is checked against checkpoint timestamps, so
stale predecessor evidence cannot authorize a current Work Item.
For repositories created before typed checkpoint evidence, an amendment may
deterministically upgrade the stored legacy checkpoint identity fields into a
typed `before_edit` entry; it does not infer intent, authority, or verification.

Checkpoint snapshots are intentionally temporal. A valid `before_edit` or
amendment entry records the repository state at that authorization boundary and
may be older after an authorized edit and fresh preflight. Only `before_finish`
is terminal evidence: it must match the current Contract, repository identity,
and snapshot, and stale, foreign, malformed, duplicate, or symlink-backed
entries fail closed.

Finalization is snapshot-sensitive too: after the final verification, run
`finish` and `archive` before committing Runtime-generated finish/outcome/archive
records. A commit changes snapshot identity even when it contains only `.ai/`
files; a commit between verification and archive makes the receipt stale and
must be retried from the current Work Item, never bypassed. Commit archived
records only after archive succeeds, then run provider finalization and hosted
checks.

## Resource finalization boundary

New Work Items must complete provider-side branch and worktree cleanup before
`close`: a `retained`, `blocked`, or `unknown` finalization result is not a
terminal success. The Runtime rejects that ordering in both the legacy library
entry point and the Runtime-bound CLI path. For immutable historical records
created by an older Runtime, `work-item finalize` may append one identity-bound
deleted transition after close. This is a bounded reconciliation only: it
must bind the closed root digest, preserve the original close bytes, and prove
the exact merged-PR, deleted-branch, and removed-worktree postconditions. It
does not authorize a new Work Item or weaken the normal cleanup-before-close
rule.

Finalization evidence is append-only. The canonical `<id>.finalize.json` is the immutable chain root; later provider observations use `<id>.finalize.<digest>.json` and bind the predecessor digest and sequence. The archived Contract freezes `baseRevision`: every canonical or transition receipt's `pullRequest.baseRevision` must equal that exact value during both recording and `finalize-verify`. Rebase before archive requires a fresh active Contract binding and review; rebase after archive is prohibited and requires fail-closed recovery instead of rewriting either record. `finalize-verify` and `close` require one unique linear head. Stale predecessors, forks, malformed records, symlinks, base mismatch, and identity drift fail closed. A pre-merge blocked root advances through continuous merge-observation (`retained`) and cleanup (`deleted`) transitions. If committing the canonical governance receipt advances the PR head, only the first unmerged-to-merged observation may declare `governanceAppendRevision`: all PR, branch, and worktree heads must move together, and Git must prove the old head is its ancestor. That append range may add regular same-Work-Item finalization receipts and the complete Runtime-generated post-finalize evidence bundle at exactly `.ai/evidence/<id>/quality-route-post-finalize.json` and `.ai/evidence/<id>/repository-gates-post-finalize.json`. Every accepted path is an `A`-only Git change whose tree entry is a `100644` regular blob. The evidence files must have their fixed schemas and bind the archived Contract, PR base and bounded head, route receipt digest, manifest digest, selected profile, and passing required gates. They are bound observations, not authority by themselves, and the range must still contain a finalization receipt addition. Missing bundle members, another Work Item or filename, malformed or duplicate-key JSON, mismatched bindings, deletion, modification, rename, symlink, unrelated change, non-merge drift, or later head drift is rejected. Archive bytes are never rewritten. Cleanup retains the accepted head.

## Pending parity registration

`docs/reference/pending-parity-registry.json` is a typed, temporary bridge for
an archived code Work Item whose three parity rows cannot safely be added to
the same scoped PR. It is not parity evidence and never means Implemented. An
entry binds the repository, full Work Item ID, GitHub PR, Contract base,
canonical finalization head, exact archive/evidence/finalize paths, three
exact `In progress` rows, and an RFC 3339 creation time. `headRevision` equals
the canonical receipt's PR, branch, and worktree heads.
`registryBaseRevision` separately binds the direct parent of one registry-only
commit, so a reviewed base merge is not confused with finalization identity.

Normal archive, verification, and finalization validation runs first. Only an
exact feature-branch or pull-request entry can replace the three
`missing_parity_entry` findings with `pending_parity_registration`. Unknown or
duplicate fields, foreign identities, unsafe or symlink paths, missing or
mismatched records, another ancestor, non-registry append, partial parity, and
malformed JSON fail closed. On the default branch, after merge, or when any
parity row exists, the entry is `stale_pending_parity_registration`. A
follow-up adds all three exact rows and removes the entry atomically without
rewriting predecessor `.ai` records.

A parity-writing Work Item uses a different, self-contained route. If its
Contract scope or acceptance, or its active Summary changed paths, explicitly
owns `docs/reference/reference-parity*` or parity registration, the light
governance gate requires all three lifecycle-bound rows before verification.
Standard and strict inherit the same static check; an ordinary code Work Item
is classified `active_non_parity` and is not forced into documentation scope.
Each row lists the future archived Contract, verification, canonical finalize,
and close paths and uses the conditional status
`In progress → Implemented after verified close` (localized in the Chinese and
Japanese ledgers). Git must prove the row commit strictly precedes addition of
the verification evidence. Missing, partial, wrong-status, foreign-path, or
post-archive-only rows fail closed. The unchanged row is therefore truthful
while active, awaiting merge/close, and closed without rewriting archived
evidence. This route does not relax the pending registry's default-branch stale
rule.

A pull-request merge ref is a combined tree, not a replay of the feature
snapshot. If the default branch contributes a later authoritative lifecycle
decision, every parity row must name that decision in addition to the retained
pre-merge receipt. Missing the later close path fails closed even when the push
head was green. A Runtime recovery successor preserves predecessor bytes and
tests the exact base-plus-feature topology before the delivery is promoted.

Recovery evidence is also validated when it is read, not only when it is
recorded. Before a current recovery candidate may affect Outcome projection or
superseded archive behavior, the Runtime rechecks its regular-file and filename
boundary, repository and current Runtime identity, predecessor
Contract/Summary/Outcome/Events digests, timestamp, decision shape, and exact
successor Contract binding. A malformed, foreign, stale, tampered, or
ambiguous candidate produces the stable `recovery_decision_invalid` boundary
and cannot move active artifacts. Historical archived records keep their
immutable bytes and historical projection; this current-read rule does not
retroactively rewrite or reclassify them.

An archived Work Item whose manifest is still `archived` may receive a valid,
append-only `supersede` recovery decision after the original archive (for
example when the provider's PR base cannot be reconciled with the frozen
Contract base). The Runtime may then close that predecessor through the
explicit recovery path without rewriting the archive manifest or its artifact
bytes. A normal archived Work Item with a bound resource context but no valid
provider finalization receipt remains yellow/not-ready; an invalid recovery
candidate is never allowed to bypass that finalization gate.

Recovery receipts are an append-only chain. When the canonical
`<id>.recovery.json` is an earlier retry, CI resolves valid digest-suffixed
`<id>.recovery.<digest>.json` successor/supersession records and binds the
selected path in each parity projection. Invalid or ambiguous candidates
remain fail-closed; the gate never treats a retry as a terminal successor.
When a retry's predecessor Contract, Summary, Outcome, or Events digest no
longer matches the archived record after fresh verification, CI treats that
retry as consumed historical evidence and continues to the actual
finalization decision. A retry with matching bindings and an explicit blocked
Summary remains a current recovery boundary.

A `retry` decision is an explicit lifecycle transition, not a green claim. If
a failed gate left the active item in `finish_ready`, Runtime moves only the
current Summary back to the legal `checkpointed` retry point and clears the
transient failure projection. The blocked Outcome and predecessor digests
remain referenced by the append-only recovery receipt; a fresh `verify` and
`finish` must generate the next current Outcome. Other lifecycle states reject
retry. Superseded manifests bind copied report digests directly, so historical
Outcome bytes are never rewritten merely to embed generated task-report
digests.

When a human-authorized retry follows a `finish.lifecycle` failure, Runtime may set an
explicit `recoveryRetryPending` marker. That marker permits one fresh verification attempt
against the recorded recovery receipt; it never synthesizes a green preflight. `finish` clears
the marker only after replacement report and event bindings succeed, so a partial retry remains
visible and recoverable.

Merge is not Work Item closure. After hosted checks pass, the exact branch and
worktree are a separate resource-finalization boundary:

```text
finalize-plan → finalize → finalize-verify → close
```

These are Runtime commands. They require an explicit `--repo` and a typed,
identity-bound context/receipt; they do not delete resources implicitly. A
Work Item may be archived only after verification, and it may be closed only
after `finalize-verify` accepts an identity-bound `Deleted` receipt. A
`Retained` receipt is an intermediate merge observation or an explicit legacy
recovery fact; it never authorizes a new close. Archived verification evidence
remains immutable historical truth: after a Runtime upgrade it is projected as
historical rather than revalidated as a current result, while a new
finalization receipt is bound to the Runtime executing the close request.

Structural close is followed by a controlled documentation projection and the
terminal default-branch check:

```text
close → promote closed docs → terminal CI
```

Run `python3 tests/docs/promote_closed_work_item.py --repo <repo> --work-item
<id>` from the synchronized detached closure context, then run the same helper
with `--check-all`. The helper first validates regular non-symlink archive,
verification, linear finalization, sequence-2 deleted, merge, and structured
close identities. It changes only the exact three Work Item documents'
machine-owned lifecycle frontmatter and the exact Work Item row in each
reference-parity document. It does not rewrite body prose or `.ai` lifecycle
truth. Invalid input fails before writes; a stale projection fails the quality
gate. This is an explicit repository workflow helper, not an automatic Runtime
Core Markdown mutation.

## Release-tag transition ordering

The release tag is created only after the PR has merged and a valid pre-merge
finalization receipt is committed. Source quality treats that immutable tag as
an `awaiting_merge_close` boundary only when the receipt is identity-bound and
its recorded PR head is proven to be an ancestor of the tagged commit. This
does not close the Work Item or waive cleanup. The published binary must then
be used to run `finalize`/`finalize-verify` and the structured human `close`;
ordinary branches, unproven tags, and malformed receipts remain fail-closed.

- `finalize-plan` records the exact Work Item branch and worktree, provider PR,
  merged head, remote, default branch, and intended cleanup. It never deletes a
  branch or worktree.
- `finalize` may act only on the exact merged branch/worktree after the PR,
  head, dirty-state, and protection checks pass. Silent branch deletion is
  forbidden.
- `finalize-verify` proves the synchronized default branch, clean relevant
  worktrees, and exact local/remote branch removal. A provider error, identity
  mismatch, or incomplete observation is `unknown` and keeps the Work Item
  open for recovery; it is not permission to continue.
- `retain` is an explicit human decision, with owner, reason, scope, and an
  expiry/review condition. Retained resources never silently become cleanup
  success and cannot authorize a new `close`; an older closed record may only
  receive the bounded deleted reconciliation described above.
- `close` must not occur before `finalize-verify` succeeds with `Deleted`.
  Every failure preserves the retry identity and a visible yellow/red Outcome.

## Agent provider surfaces

The adapter is a thin repository-local projection of these rules; it is not a
second policy engine. `agent install` is explicit and owned. New Cursor
installations use the provider-native `.cursor/rules/ai-cockpit.mdc` surface.
Repositories that already have a managed `.cursor/rules/ai-cockpit.md` keep that
legacy target so an upgrade does not rename or overwrite user files. The
Runtime never auto-installs `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, Cursor rules,
or global provider/MCP configuration.

The generated managed section carries the same Contract-first, pause,
Summary, visible Outcome, and closure semantics documented above. It is
advisory discovery guidance: current governance state must always come from
the explicit Runtime query, and a provider's prompt cannot grant authority.

## Safety boundary

Rules remain language-neutral and repository-local. Do not include secrets or
machine credentials, edit user-global Agent or MCP configuration, or treat
managed Agent prompts as governance authority. Do not copy V1 runtime code,
schemas, installers, or template implementation into this repository.
Never revert user changes unless explicitly asked. The default instruction read
set is `.ai/README.md`, `.ai/glossary.md`, `AGENTS.md`, and current
machine-readable governance records; `docs/archive/**` and reference material
are historical/informational unless explicitly included by a person or Contract.
Generated status, receipt, and archive files must be produced by the Runtime,
not hand-edited.
The reference template's hosted-verification snapshot exception has no
equivalent command here; never push an unpublished local snapshot as a
substitute for the reviewed branch/PR workflow.
