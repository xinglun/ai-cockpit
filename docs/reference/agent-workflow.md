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
- Use one Contract, one dedicated branch/worktree, and one PR per Work Item.
  Compatible independent Work Items may run concurrently when scope, evidence
  ownership, repository context, and serialized projections are isolated.
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

## Resource finalization boundary

Finalization evidence is append-only. The canonical `<id>.finalize.json` is the immutable chain root; later provider observations use `<id>.finalize.<digest>.json` and bind the predecessor digest and sequence. `finalize-verify` and `close` require one unique linear head. Stale predecessors, forks, malformed records, symlinks, and identity drift fail closed. A pre-merge blocked root advances through continuous merge-observation (`retained`) and cleanup (`deleted`) transitions. If committing the canonical governance receipt advances the PR head, only the first unmerged-to-merged observation may declare `governanceAppendRevision`: all PR, branch, and worktree heads must move together, Git must prove the old head is its ancestor, and the range may add only regular same-Work-Item finalization receipt JSON. Cleanup retains that head; unrelated, non-append, non-merge, or later head drift is rejected.

Merge is not Work Item closure. After hosted checks pass, the exact branch and
worktree are a separate resource-finalization boundary:

```text
finalize-plan → finalize → finalize-verify → close
```

These are Runtime commands. They require an explicit `--repo` and a typed,
identity-bound context/receipt; they do not delete resources implicitly. A
Work Item may be archived only after verification, and it may be closed only
after `finalize-verify` accepts `Deleted` or an explicitly authorized
`Retained` receipt. Archived verification evidence remains immutable historical
truth: after a Runtime upgrade it is projected as historical rather than
revalidated as a current result, while the new finalization receipt is always
bound to the Runtime executing the close request.

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
  success; unless an organization policy explicitly permits a bounded retain
  path, `close` remains blocked.
- `close` must not occur before `finalize-verify` succeeds (or a separately
  authorized, auditable retain path is accepted). Every failure preserves the
  retry identity and a visible yellow/red Outcome.

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
