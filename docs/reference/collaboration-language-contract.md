---
author: AI Cockpit maintainers
title: "Collaboration language contract"
description: "A cross-cutting map from human-Agent communication moments to existing Runtime facts, states, and decisions; not a second governance state machine."
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-679-p0-collaboration-language-contract
---

# Collaboration language contract

[简体中文](collaboration-language-contract.zh-CN.md) · [日本語](collaboration-language-contract.ja.md)

North Star: Calibrated Human-Agent Trust. A new user who cannot ask a
maintainer for a live explanation should still be able to read what an Agent
says at each collaboration moment, understand the current Runtime state,
make a bounded decision, and hand off to another Agent or session without
losing that understanding.

## What this contract is, and is not

This page is a **map**, not a new protocol. It does not define a new state,
marker, exit code, or lifecycle command, and it does not change the meaning
of any existing one. Every fact named below already has one authoritative
source: [`.ai/glossary.md`](../../.ai/glossary.md), the
[Repository Protocol specification](../protocol/v1/specification.md),
[Human-facing Outcome](outcome-report.md),
[How to read Cockpit status](how-to-read-cockpit-status.md),
[Agent workflow and review boundaries](agent-workflow.md), and
[Troubleshooting and recovery](troubleshooting.md). Where this page
paraphrases one of those sources for a specific communication moment, the
source document remains authoritative; this page must be read as an index
into it, not a replacement.

Nothing here is contradicted by, or contradicts, the pending performance
(`docs/reference/performance-initiative-2026-09.*`, not yet on `main`, tracked
by the still-unmerged WI-651 successor chain) or architecture
(`docs/reference/architecture-responsibility-map-2026-09.*`, not yet on
`main`, tracked by the still-unmerged WI-652 successor chain) initiative
documents. Both are cited here only as future cross-references once merged;
this contract does not depend on their content and does not treat their
draft branches as canonical.

## The five questions every hand-off must answer

Regardless of surface language, CLI/MCP entry point, or which Agent or model
is speaking, a communication moment that hands a decision or a result to a
human must let the reader answer all five of these from what was said, not
from memory of an earlier conversation:

1. **What state is this in right now?** — sourced from `status --repo <repo>
   [--id <work-item>]`, the active Summary `state`, and/or `work-item outcome`.
2. **Which facts are confirmed, and which are still unknown?** — an `unknown`
   field is a fact, not a placeholder for "probably fine"; see
   [Human-facing Outcome § status markers](outcome-report.md).
3. **Is a human decision required, and why?** — sourced from `preflight`'s
   `reviewState` and, when `needs_human_confirmation`, its structured
   `humanDecisionRequest` (what happened, why it matters, options, a
   recommendation, the question, and a resume condition); see
   [Agent workflow](agent-workflow.md).
4. **What does each option actually do, and what is its authorization
   scope?** — an option's effect is whatever the named Runtime command does
   (for example: unlocks one checkpoint transition; does not prove a test,
   scenario, verification, or release result). No option may be described in
   terms broader than the command it triggers.
5. **Where does this go after the decision, and under what condition does it
   stop again?** — the next action and its resume condition, taken verbatim
   from the Outcome's `next action` field or the Preflight Review, never
   invented.

A response may compress or omit some of these when the answer is unchanged
from the last time it was stated (see [Multilingual semantic parity](multilingual-semantic-parity.md)
on why re-stating a fixed heading every time is not required), but it must
never make one of the five unanswerable by omission.

## Communication nodes

Each node below is a moment where an Agent must hand something to a human.
For each: the trigger and its fact source, the state the user needs to hold
in mind, what the Agent must show, whether a human decision is required,
what each option actually authorizes, the next step and where it stops
again, and how to handle missing information, a misunderstanding, or a
cancellation.

### 1. Task start and scope confirmation

- **Trigger / source of truth**: `work-item new --mode <mode>` scaffold, or
  `start --intent --goal [--scope] [--out-of-scope]`; the resulting Contract
  `state` and `preflight`'s `reviewState`.
- **State to understand**: a scaffold with empty `intent`, `goal`, `scope`,
  `out-of-scope`, `acceptanceCriteria`, or `authority` is explicitly
  `not_ready` / `needs_human_confirmation`; this is never "ready to
  implement" regardless of exit code.
- **Must present**: the known facts the Runtime already resolved
  (`repositoryId`, `baseRevision`, digests) versus the human-owned fields
  still empty; the discovered remote default branch and base revision.
- **Human decision needed**: yes, always, before implementation — supplying
  intent/scope/acceptance/authority is itself the decision. There is no
  silent default.
- **Options and their scope**: supplying scope only binds edits inside that
  boundary; it does not grant risk, authority, or acceptance. Each field is
  independent and none can be inferred from the others.
- **Next step / stop boundary**: a successful `work-item new` or `start`
  moves to `preflight`; a `not_ready` or `needs_human_confirmation`
  preflight result stops there until the Contract is amended — an advisory
  zero exit status is not permission to proceed.
- **Missing/misunderstanding/cancellation**: an empty or ambiguous field
  stays `unknown`, never inferred from file names, prose, or a prior
  session; a person may cancel by leaving the Work Item `not_ready` — no
  cleanup obligation exists until `start` succeeds.

### 2. Authorization request

- **Trigger / source of truth**: `preflight` returning
  `needs_human_confirmation` with a `humanDecisionRequest`; the
  repository-local `decisionEvidence` projection that records a bounded
  review.
- **State to understand**: an authorization request is scoped to one
  Contract digest and one repository snapshot digest; it is not a standing
  approval for future changes to either.
- **Must present**: what happened, why it matters, the options, a
  recommendation, the exact question, and the resume condition — all four
  structured fields, not a summary sentence.
- **Human decision needed**: yes. A successful command or a yellow result is
  never itself authorization (see [Pause Rule](../../.ai/glossary.md)).
- **Options and their scope**: a recorded decision receipt binds
  `decisionId`, Work Item, repository, Contract digest, preflight decision
  digest, snapshot digest, actor, timestamp, and reason; it may unlock one
  checkpoint transition and nothing else — never a test result, a
  verification result, or a release decision.
- **Reusing an existing authorization vs. requesting a new one vs. a pure
  technical recovery** — these are three different things and must not be
  conflated:
  - *Reuse* is only valid when a Receipt's identity bindings still match
    (repository, Work Item, Contract digest, snapshot digest, Runtime
    identity); reuse is a rule-and-record match, never a natural-language
    guess that "this looks like the same request."
  - *A new authorization* is required whenever the Contract or repository
    snapshot changes after a prior review; the previous receipt remains
    historical evidence and a fresh receipt is written under a
    digest-suffixed path.
  - *A pure technical recovery* (`work-item recover` with `retry`,
    `successor`, or `supersede`) never grants new authorization and never
    turns a blocked or red result green; it only restores a legal lifecycle
    position so a fresh `verify`/`finish` can run.
- **Next step / stop boundary**: a valid decision unlocks exactly the
  checkpoint transition it names; the Work Item still requires fresh
  `verify` before `finish`.
- **Missing/misunderstanding/cancellation**: a missing, stale, foreign, or
  malformed receipt remains stopped; declining leaves the Work Item at its
  current safe state with no forced retry.

### 3. Verification passed, failed, or evidence insufficient

- **Trigger / source of truth**: `verify --repo <repo> --id <id>`; the
  written `.ai/evidence/` receipt; the Outcome's separate Verification
  status line.
- **State to understand**: **verification passing is not the same fact as
  merge authorization, release authorization, or full acceptance.** The
  Outcome deliberately keeps Verification, Lifecycle, Human decision, and
  Governance signal as four separate lines; a green Verification line says
  only that the declared checks passed on a fresh, identity-bound receipt.
- **Must present**: which declared checks ran, on which snapshot, and the
  resulting decision-state color; whether the evidence is current, legacy
  (`legacy_evidence_historical`), or historical-but-not-revalidated
  (`historical_evidence_not_revalidated`) — see
  [Human-facing Outcome](outcome-report.md).
- **Human decision needed**: not for a routine green result (review the
  named evidence and proceed per your own process); yes for yellow
  (investigate or confirm) and always stop for red.
- **Options and their scope**: proceeding past a green Verification line
  authorizes exactly the declared checks it names — nothing about merge,
  release, or a security claim. A yellow result is not repaired by rerunning
  an unrelated command.
- **Next step / stop boundary**: green moves toward `finish`; yellow
  requires investigation or an explicit decision before `finish`; red stops
  and requires the named recovery condition.
- **Missing/misunderstanding/cancellation**: a cancelled or interrupted
  verification leaves no receipt and the Work Item remains at its prior
  lifecycle state; retrying is always safe and never silently reuses a
  mismatched receipt.

### 4. Block and recovery

- **Trigger / source of truth**: a blocked `finish` (or `checkpoint`)
  persisting a red/yellow Outcome and returning its original nonzero error;
  `work-item recover` for an explicit lifecycle recovery.
- **State to understand**: a blocked Work Item keeps its checkpointed
  lifecycle state; the block is visible as a Runtime-projected Outcome, not
  a silently swallowed failure.
- **Must present**: the failed gate, the deterministic recovery condition,
  and whether the intended next action is a plain retry (same Contract,
  same scope) or a recovery decision (`retry`/`successor`/`supersede`).
- **Human decision needed**: a plain retry after fixing the named cause
  needs no new authorization request; a `successor` or `supersede` decision
  is a new, explicit, recorded decision because it changes scope, authority,
  or base, or declares a predecessor historical.
- **Options and their scope**: a `retry` restores the legal `checkpointed`
  position for one fresh verification attempt; it does not rewrite the
  earlier blocked event. `supersede` preserves the predecessor's original
  bytes and evidence as historical — it is not a current pass and not a
  current failure.
- **Next step / stop boundary**: after a successful retry, a fresh `verify`
  and `finish` must produce the next current Outcome; the blocked event
  remains in the append-only event stream for audit.
- **Missing/misunderstanding/cancellation**: an ambiguous, foreign, or
  tampered recovery candidate is rejected as `recovery_decision_invalid` and
  changes nothing; declining to recover leaves the Work Item blocked, which
  is itself a safe, inspectable state.

### 5. Merge confirmation

- **Trigger / source of truth**: a reviewed pull request whose hosted checks
  have passed; `work-item finalize-plan` → `finalize` → `finalize-verify` →
  `close`.
- **State to understand**: **merge is not Work Item closure.** A merged PR
  only starts the resource-finalization boundary; branch/worktree cleanup,
  a `Deleted` finalization receipt, and a confirmed `close` decision are all
  still required, and are a different boundary from repository verification.
- **Must present**: the merged PR's head SHA, whether the default branch is
  synchronized, and the exact remaining sequence (clean branch/worktree →
  finalization receipt → `finalize-verify` → `close`).
- **Human decision needed**: yes, for `close` — it requires an explicit
  `--human-decision`; a `Retained` finalization receipt is an intermediate
  observation or explicit legacy fact and never authorizes `close` on its
  own.
- **Options and their scope**: approving the PR authorizes the merge itself;
  it does not authorize skipping cleanup, does not authorize `close` by
  itself, and does not retroactively upgrade local verification to hosted
  or enterprise assurance (see [How to read Cockpit status § evidence
  boundaries](how-to-read-cockpit-status.md)).
- **Next step / stop boundary**: after `close`, the lifecycle projection
  becomes `closed` only when the confirmed close decision is valid; an
  invalid or missing decision never promotes an archive to `closed`.
- **Missing/misunderstanding/cancellation**: a provider error, identity
  mismatch, or incomplete finalization observation is `unknown` and keeps
  the Work Item open for recovery — it is never treated as permission to
  continue to the next Work Item.

### 6. Outcome

- **Trigger / source of truth**: `work-item outcome --repo <repo> --id <id>`
  (CLI) or the repository-bound MCP `work_item_outcome` tool with an
  explicit `workItemId`.
- **State to understand**: the Outcome is the terminal, visible handoff; a
  folded log line or a raw `work_item_get` machine record is never a
  substitute for it. The full field order, marker semantics, and evidence
  rules are defined once in [Human-facing Outcome](outcome-report.md) and
  are not restated here.
- **Must present**: the `Outcome: 🟢/🟡/🔴` marker plus the reader-first
  order (task result, completed work, problems found, stops triggered,
  problems resolved, risks, unknowns, human decisions, verification and
  evidence, impact, next action) exactly as generated — an Agent must not
  reorder, fold, or drop a section.
- **Human decision needed**: depends on the marker; the Outcome itself
  states whether one is pending and what it is.
- **Options and their scope**: none are introduced by the Outcome itself; it
  reports facts and the already-recorded decision, if any.
- **Next step / stop boundary**: exactly the `next action` field states.
- **Missing/misunderstanding/cancellation**: an empty section renders as
  `Not recorded` or `Not assessed`, never as a positive fact; if a reader
  mistakes a green Verification line for a completed release, the correction
  is to re-point them at the four separated status lines, not to change the
  Outcome's wording.

### 7. Agent or session handoff

- **Trigger / source of truth**: a new Agent, a new conversation, or a
  different underlying model resuming work on the same repository and Work
  Item.
- **State to understand**: everything the previous node describes above is
  re-derivable from the Runtime alone — `status`, `work-item outcome`, the
  active/archived Contract, Summary, and decision records — without any
  conversation transcript. There is currently no dedicated Runtime command
  that packages "handoff state" into one call; the incoming Agent assembles
  it from the same `status`/`outcome`/Contract/Summary sources named above.
- **Must present**: current goal and scope boundary (Contract), what is
  done versus pending (Summary `state`, `checkpointCount`), which evidence
  is valid and for what scope (`.ai/evidence/`, freshness bindings), which
  authorizations are recorded and which decisions are still open
  (`decisionEvidence`, `humanDecisionRequest`), and the blocking reason and
  legal next step if blocked (persisted Outcome).
- **Human decision needed**: only if one was already open before the
  handoff; a handoff must never manufacture a new decision point, and must
  never silently resolve an old one.
- **Options and their scope**: none are created by a handoff itself.
- **Next step / stop boundary**: identical to whatever it was before the
  handoff; a new Agent inherits the same stop boundary, not a fresh one.
- **Missing/misunderstanding/cancellation**: if a fact cannot be recovered
  from the Runtime, it must be presented as an explicit gap requiring a
  human answer, never inferred from the new Agent's own habits, a specific
  model's phrasing conventions, or an unprovided prior conversation. A
  session switch must not silently reset a recorded authorization to
  "not yet granted," and must not silently broaden a scoped authorization
  to cover new ground it never named. Whether an existing authorization
  still applies is decided by the identity-binding rules in node 2, not by
  which Agent is now speaking.

## Ten semantic invariants

These restate, as checkable rules, guarantees that the sources above already
document. None of them is a new governance rule; each cites where it is
already true today, and — where relevant — the follow-on Work Item expected
to add an automated check for it. Automated coverage does not exist yet
unless explicitly stated; see [Current unknowns](#current-unknowns-and-follow-on-work).

1. **Verification passing does not by itself mean full acceptance or merge
   authorization.** Already documented: [Human-facing Outcome](outcome-report.md)
   ("A green result does not authorize merge, release, publication, or a
   security claim.").
2. **An empty record does not mean no risk.** Already documented: empty
   sections render as `Not recorded` / `Not assessed`, never as a positive
   finding ([Human-facing Outcome](outcome-report.md)).
3. **An unknown fact cannot be completed by the presentation layer.**
   Already documented: `unknown` evidence is never interpreted as a pass
   ([`.ai/glossary.md`](../../.ai/glossary.md), [Decision states](../protocol/v1/specification.md)).
4. **A historical record cannot become a current failure or a current valid
   proof without an explicit basis.** Already documented: superseded and
   historical evidence is projected with a yellow historical marker, never
   silently revalidated as current ([Human-facing Outcome § historical
   markers](outcome-report.md)).
5. **The same fact must not contradict itself across CLI, MCP, a summary, and
   a full report.** Documented as a design intent in
   [Multilingual semantic parity](multilingual-semantic-parity.md) and the
   MCP section of [Human-facing Outcome](outcome-report.md) (`work_item_outcome`
   returns the same localized handoff the CLI prints); a structural,
   automated cross-entry-point check is not yet built — tracked as follow-on
   work.
6. **Changing language must not change facts, authorization scope, or
   operational consequences.** Already documented:
   [Multilingual semantic parity](multilingual-semantic-parity.md) — JSON
   field names and enum values remain stable, and Contract-owned text is
   never machine-translated.
7. **The displayed next step must match current Runtime state and policy.**
   Already documented: the next action is taken from the Outcome or the
   Preflight Review, never invented ("The report never fills a governance
   decision from inference." — [Human-facing Outcome](outcome-report.md)).
8. **Whether an existing authorization applies is decided by rule and
   record, not discarded or broadened by switching sessions.** Already
   documented for Receipts ([`.ai/glossary.md`](../../.ai/glossary.md):
   "may be reused only when all authorized identity bindings still match");
   applied to Agent/session handoff in [node 7](#7-agent-or-session-handoff)
   above. A dedicated handoff-continuity check across a simulated session
   switch is not yet built — tracked as follow-on work.
9. **Every question that requires a human decision must name the decision
   subject, its impact, and its recovery/resume condition.** Already
   documented: the structured `humanDecisionRequest` shape
   ([Agent workflow](agent-workflow.md)).
10. **A summary may omit detail but must never hide a blocker, a key
    unknown, or a required human decision.** Already documented: a blocked
    `finish` still emits its persisted red/yellow Outcome even under
    `--json` suppression of the human-readable form on stderr, and unknowns
    are never dropped by compression ([Human-facing Outcome](outcome-report.md)).

## Current unknowns and follow-on work

This document is a docs-only artifact verified by structural documentation
checks (frontmatter completeness, internal link resolution, tri-language
presence) and by maintainer/reviewer reading — **not** by an automated
semantic check, and not by any user study. It does not claim that every
invariant above has automated, cross-entry-point enforcement today; several
explicitly do not yet (see invariants 5 and 8). The following are scoped as
separate, later Work Items rather than folded into this one:

- A state/transition-derived scenario matrix covering legal and illegal
  transitions, evidence defects, authorization reuse versus new authorization,
  verification outcomes, archive/close/supersede/historical query, and
  Agent/session handoff across languages and entry points.
- Automated checks for the ten invariants above, with positive, negative, and
  boundary cases per invariant, run against a controlled test repository
  rather than fixed-string matching.
- End-to-end verification that a displayed state, a chosen option, and the
  resulting Runtime behavior agree, including interruption and resume, in a
  controlled test repository; any simulated human decision used for this
  purpose must be marked as test data and must never become a real
  authorization record.
- A handoff-completeness check confirming that a new Agent or session can
  reconstruct goal/scope, done/pending, valid evidence and its scope, valid
  authorizations and open decisions, and blocking reasons from the Runtime
  alone, without a conversation transcript.

Within the scenarios this document currently covers, it aims for consistent,
traceable communication semantics grounded in the cited sources. It does not
claim that every reader will understand it correctly; that claim would
require observation this Work Item does not perform.
