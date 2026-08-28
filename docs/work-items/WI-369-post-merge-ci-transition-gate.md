---
author: AI Cockpit maintainers
title: "WI-369 — Post-merge CI transition gate"
description: "Distinguish the reviewed merge-to-close transition from a stale unclosed Work Item without weakening the gate."
workItemId: WI-369-post-merge-ci-transition-gate
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-369-post-merge-ci-transition-gate
terminalArchive: .ai/work-items/archive/WI-369-post-merge-ci-transition-gate.contract.json
terminalVerification: .ai/evidence/WI-369-post-merge-ci-transition-gate.verification.json
terminalFinalization: .ai/decisions/WI-369-post-merge-ci-transition-gate.finalize.d6e6c0bc91cdbdd880b1a8e9599e087d8003643969967bc0eef1156671d7ffa5.json
terminalDecision: .ai/decisions/WI-369-post-merge-ci-transition-gate.close.json
capabilityClaims:
  - governance_integrity
  - reference_parity
---

# WI-369 — Post-merge CI transition gate

[简体中文](WI-369-post-merge-ci-transition-gate.zh-CN.md) · [日本語](WI-369-post-merge-ci-transition-gate.ja.md)

## Intent and boundary

Default-branch CI currently runs immediately after a reviewed merge, while
provider finalization and the authoritative close receipt are written in the
post-merge cleanup step. This Work Item removes the resulting false
`missing_terminal_decision` failure without turning missing close into an
advisory condition.

The only grace state is a real GitHub `push` to the configured default branch
whose `HEAD` is an exact two-parent merge and which adds the archived Contract
for the Work Item. It is reported as `awaiting_merge_close`; the next ordinary
default-branch commit still requires finalization and close.

The change is limited to the repository gate, its CI invocation comments,
regression fixtures, and three-language documentation/parity records. Rust
Runtime lifecycle semantics, release artifacts, provider APIs, global Agent or
MCP configuration, and source Python/Make/V1 runtime are out of scope.

## Acceptance

- A qualifying merge is accepted as an explicit `awaiting_merge_close`
  transition and emits no false `missing_terminal_decision` finding.
- A follow-up direct default-branch commit without close fails closed.
- Direct commits, malformed or unrelated merges, stale archived Work Items,
  missing parity, and missing/contradictory GitHub context remain blocking.
- The decision is deterministic from Git history and standard immutable GitHub
  context; no bypass flag or process-global current repository is introduced.
- Regression tests and all three language documents describe the same bounded
  transition and eventual finalize/close requirement.
- Installed Runtime verification and a visible human Outcome are completed
  before merge, close, and exact branch/worktree cleanup.

## Verification record

The regression suite constructs a reviewed merge that introduces an archived
Work Item, checks the allowed transition, then appends one ordinary commit and
checks that the same unclosed Work Item blocks the gate. Existing negative
fixtures remain part of the gate test corpus.

The GitHub workflow inherits `GITHUB_EVENT_NAME`, `GITHUB_REF`, and
`GITHUB_SHA`; these identify the event only and never replace Contract,
evidence, PR, or close validation.
