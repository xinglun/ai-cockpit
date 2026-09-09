---
author: AI Cockpit maintainers
title: "WI-680 — P1 collaboration scenario matrix"
description: "A state/transition-derived collaboration scenario matrix, plus tri-language registration of WI-679 and WI-680 in the reference-parity ledger."
audience: [maintainer, reviewer, adopter]
workItemId: WI-680-p1-collaboration-scenario-matrix
status: in_progress
authority: authorized
lastVerifiedBy: WI-680-p1-collaboration-scenario-matrix
---

[简体中文](WI-680-p1-collaboration-scenario-matrix.zh-CN.md) · [日本語](WI-680-p1-collaboration-scenario-matrix.ja.md)

# WI-680 — P1 collaboration scenario matrix

## Intent

Continue the AI Cockpit collaboration-language initiative's P1 scope: derive
a scenario matrix from the repository's actual supported lifecycle, evidence
states, authorization states, and operation types, prioritizing key
boundaries and easily-confused combinations over exhaustive enumeration.
Also complete the reference-parity ledger registration that WI-679 could not
finish in the correct pre-archive commit order without rewriting
already-pushed history.

## Boundary

This is a documentation-only Work Item. It adds
`docs/reference/collaboration-scenario-matrix.md` (+ zh-CN/ja),
`docs/reference/collaboration-scenario-matrix.json` (structured source of
truth), one index entry in `docs/reference/README.md` (+ zh-CN/ja), the
`docs/work-items/WI-679-*` and `docs/work-items/WI-680-*` record pages, and
`docs/reference/reference-parity.*` rows registering WI-679 and WI-680. It
changes no CLI, MCP, Contract schema, Outcome schema, or lifecycle behavior.
Turning scenarios into automated, executable checks against a controlled
test repository is explicitly out of scope and named as follow-on work in
the delivered document.

## Authorization record

Human authorization was explicitly granted on 2026-09-08 to continue the
collaboration-language initiative through the Work Item process, including
completing documentation-registration gaps discovered during delivery. The
exact scope and evidence remain recorded in the Contract and PR.

## Acceptance and lifecycle

- The scenario matrix's structured JSON covers all required categories with
  sourceType-labeled entries (observed/documented/designed), most of them
  citing real commands/output produced during this delivery.
- WI-679 and WI-680 both appear in the tri-language reference-parity ledger
  with the pre-archive `In progress → Implemented after verified close`
  status, each row's first appearance preceding this Work Item's own
  verification-evidence commit.
- `start → preflight → checkpoint → verify → finish → archive → close` is the
  governed route; `user_visible_benefit_not_declared` remains explicit.
- `bash tests/docs/documentation_acceptance.sh` passes on the exact reviewed
  head.

## Evidence

- archive: `.ai/work-items/archive/WI-680-p1-collaboration-scenario-matrix.contract.json`
- verification: `.ai/evidence/WI-680-p1-collaboration-scenario-matrix.verification.json`
- finalization: `.ai/decisions/WI-680-p1-collaboration-scenario-matrix.finalize.json` (pending merge)
- close: `.ai/decisions/WI-680-p1-collaboration-scenario-matrix.close.json` (pending merge)
