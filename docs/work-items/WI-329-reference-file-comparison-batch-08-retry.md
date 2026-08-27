---
author: AI Cockpit maintainers
title: "WI-329 — reference file comparison batch 08 CI regression repair"
workItemId: WI-329-reference-file-comparison-batch-08-retry
description: "Redeliver the nine-file batch from a clean default-branch base after the immutable WI-328 hosted inventory gate failure."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-329-reference-file-comparison-batch-08-retry
terminalArchive: .ai/work-items/archive/WI-329-reference-file-comparison-batch-08-retry.contract.json
terminalVerification: .ai/evidence/WI-329-reference-file-comparison-batch-08-retry.verification.json
terminalFinalization: .ai/decisions/WI-329-reference-file-comparison-batch-08-retry.finalize.json
terminalDecision: .ai/decisions/WI-329-reference-file-comparison-batch-08-retry.close.json
---

# WI-329 — reference file comparison batch 08 CI regression repair

## Intent and boundary

WI-328's source comparison and inventory were archived before hosted quality
identified a brittle assertion: the conformance wrapper required a future WI
number to appear in a reference-only reason. WI-328's closed PR and immutable
records remain historical evidence. This narrow successor replays the same
batch from the synchronized default branch and makes the gate assert the
semantic boundary instead of an arbitrary Work Item number.

The Runtime remains external and every repository operation is explicitly
bound with `--repo`. No source Python/Make implementation, generic Session,
global Agent/MCP configuration, or Runtime behavior is added.

## Repair and file-level scope

| Path | Decision |
| --- | --- |
| `tests/conformance/reference_file_inventory_test.sh` | Replace the brittle future-WI-number assertion with a stable semantic phrase assertion. |
| `tests/conformance/reference_file_inventory.py` and `.json` | Keep the four capability matrix/claim paths `reference-only` and describe a future dedicated Work Item without assigning this repair as their implementation. |
| `docs/reference/reference-file-comparison.*` | Preserve the nine WI-328 classifications and explain the immutable predecessor/successor boundary. |
| `docs/reference/reference-parity.*` | Mark WI-328 as recovered and register this successor before its verification evidence. |
| `docs/work-items/WI-328-reference-file-comparison-batch-08.*` | Correct the future capability-claim follow-up reference to WI-330 so WI-329 remains the CI repair. |
| `docs/work-items/WI-329-reference-file-comparison-batch-08-retry.*` | Record the bounded repair and terminal evidence in three languages. |

The nine pinned source paths remain the same as WI-328: five are
`implemented-different-by-design` and four are explicit `reference-only`.
The source public capability matrix/checker is not copied or advertised as a
target gate; a later capability-claim/evidence Work Item is still required.

## Adopter feedback boundary

The Cursor report is external validation. Stable lifecycle JSON, durable
human Outcome replay, close-before-next checks, and fail-closed start remain
documented as existing Runtime capabilities. Automatic IDE chat posting,
diagnostic remediation, controls scaffolding, close-gap convenience, and a
Makefile requirement remain out of scope.

## Acceptance and evidence

1. `tests/conformance/reference_file_inventory_test.sh` passes with the pinned
   source commit and target baseline, including the corrected semantic
   assertion.
2. The nine WI-328 inventory records remain five
   `implemented-different-by-design` and four `reference-only`, with no
   deferred-next-batch or migrate-gap record.
3. English, Simplified Chinese, and Japanese comparison/parity/Work Item pages
   agree on the predecessor recovery, semantic gate assertion, and future
   capability-claim boundary.
4. WI-328 historical bytes are not rewritten or silently removed; no source
   Python/Make execution or global Agent/MCP configuration is introduced.
5. Installed Runtime inspect/status/doctor/agent doctor, focused gates,
   complete workspace checks, hosted CI, reviewed merge, finalization, close,
   and exact branch/worktree cleanup pass.

[简体中文](WI-329-reference-file-comparison-batch-08-retry.zh-CN.md) · [日本語](WI-329-reference-file-comparison-batch-08-retry.ja.md)
