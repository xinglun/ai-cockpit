---
author: AI Cockpit maintainers
title: "WI-740 — P1 collaboration invariant coverage mapping (redelivery)"
description: "Redelivers WI-681's invariant coverage mapping after a multi-agent WI number collision, correcting invariant 8's assessment and reflecting invariants 5 and 9 being independently closed."
audience: [maintainer, reviewer, adopter]
workItemId: WI-740-p1-invariant-coverage-mapping
status: implemented
authority: authorized
lastVerifiedBy: WI-740-p1-invariant-coverage-mapping
terminalArchive: .ai/work-items/archive/WI-740-p1-invariant-coverage-mapping.contract.json
terminalVerification: .ai/evidence/WI-740-p1-invariant-coverage-mapping.verification.json
terminalFinalization: .ai/decisions/WI-740-p1-invariant-coverage-mapping.finalize.json
terminalDecision: .ai/decisions/WI-740-p1-invariant-coverage-mapping.close.json
---

[简体中文](WI-740-p1-invariant-coverage-mapping.zh-CN.md) · [日本語](WI-740-p1-invariant-coverage-mapping.ja.md)

# WI-740 — P1 collaboration invariant coverage mapping (redelivery)

## Intent

Redeliver the collaboration invariant coverage mapping originally attempted
as WI-681, which was closed without merging due to a genuine multi-agent WI
number collision (a different concurrent agent independently used the same
short id `WI-681` for an unrelated documentation-promotion Work Item that
merged first, in PR #677). This redelivery also corrects one assessment:
invariant 8 (authorization applicability across session switches) was
originally marked "No automated test found," but closer reading found
`crates/cockpit-repository/tests/preflight_review.rs::bound_human_review_receipt_allows_checkpoint_but_not_stale_reuse`
already proves its core claim. Invariants 5 and 9, which the original draft
correctly named as gaps, were independently closed by WI-682 (PR #679) and
WI-710 (PR #702) before this redelivery, so this page reflects that current
state rather than the stale gap list. Per explicit repository-owner
delegation to continue the AI Cockpit collaboration-language initiative.

## Boundary

This is a documentation-only Work Item. It adds
`docs/reference/collaboration-invariant-coverage.md` (+ zh-CN/ja), one index
entry in `docs/reference/README.md` (+ zh-CN/ja), this record page, and its
own reference-parity registration. It changes no test file, no CLI/MCP
behavior, and no Contract/Outcome schema. Writing the invariant-7 test
remains explicitly out of scope and is named as a bounded follow-on Work
Item in the delivered document.

## Acceptance and lifecycle

- Invariants 1, 2, 3, 4, 5, 6, 8, 9, 10 are stated as automated (fully or
  partially), each citing a test file and function verified by direct
  reading of the current test source at delivery time. Invariant 7 is
  stated as the sole remaining named gap.
- The document explicitly states why it supersedes WI-681 and what changed
  since that draft.
- `start → preflight → checkpoint → verify → finish → archive → close` is the
  governed route; `user_visible_benefit_not_declared` remains explicit.
- `bash tests/docs/documentation_acceptance.sh` passes on the exact reviewed
  head.

## Evidence

- archive: `.ai/work-items/archive/WI-740-p1-invariant-coverage-mapping.contract.json`
- verification: `.ai/evidence/WI-740-p1-invariant-coverage-mapping.verification.json`
- finalization: `.ai/decisions/WI-740-p1-invariant-coverage-mapping.finalize.json` (pending merge)
- close: `.ai/decisions/WI-740-p1-invariant-coverage-mapping.close.json` (pending merge)
