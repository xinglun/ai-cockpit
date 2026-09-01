---
author: AI Cockpit maintainers
title: "WI-470 — terminal documentation promotion and historical artifact recovery"
description: "Restore archive-manifest-referenced historical artifacts and promote the closed WI-469 projections without rewriting predecessor truth."
audience: [maintainer, reviewer, adopter]
workItemId: WI-470-terminal-doc-promotion-and-artifact-recovery
status: implemented
authority: authorized
lastVerifiedBy: WI-470-terminal-doc-promotion-and-artifact-recovery
terminalArchive: .ai/work-items/archive/WI-470-terminal-doc-promotion-and-artifact-recovery.contract.json
terminalVerification: .ai/evidence/WI-470-terminal-doc-promotion-and-artifact-recovery.verification.json
terminalFinalization: .ai/decisions/WI-470-terminal-doc-promotion-and-artifact-recovery.finalize.json
terminalDecision: .ai/decisions/WI-470-terminal-doc-promotion-and-artifact-recovery.close.json
---

# WI-470 — terminal documentation promotion and historical artifact recovery

## Intent and boundary

WI-470 is a bounded recovery Work Item. It restores exact historical task
reports referenced by immutable WI-467/WI-468 archive manifests and promotes
the closed WI-469 terminal documentation projections in all three languages.
It does not rewrite predecessor archive, evidence, recovery, or close bytes.

## Scope

- Restore the manifest-referenced WI-467 and WI-468 task-report artifacts from
  their recorded source commits, byte-for-byte.
- Promote the WI-469 Work Item document and reference-parity row after its
  verified close.
- Preserve explicit supersede recovery and close receipts for WI-467/WI-468.
- Keep post-close documentation and archive-manifest checks repeatable.

## Out of scope

Reference inventory source files, Runtime/Core implementation, object
repositories, release/adopter scripts, and global Agent/MCP configuration.

## Acceptance

1. Both missing task-report pairs are restored exactly and their archive
   manifests validate.
2. WI-469 terminal projections are consistent in English, Simplified Chinese,
   and Japanese.
3. Documentation, conformance, and workspace gates pass without rewriting
   predecessor archive/evidence/recovery bytes.

## Verification

- `cargo test --locked --workspace`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/parity_status_check.sh`

## Recovery boundary

The predecessor recovery receipts remain historical evidence. WI-469 is the
verified successor; this Work Item only repairs the missing manifest artifacts
and terminal projections needed for an auditable closed history.
