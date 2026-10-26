---
author: AI Cockpit maintainers
title: "WI-1112-windows-archive-byte-preservation — Windows archive byte preservation"
description: "Repair Windows-only local Git fixture and immutable evidence checkout failures without weakening byte-integrity checks."
audience:
  - maintainer
  - reviewer
workItemId: WI-1112-windows-archive-byte-preservation
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1112-windows-archive-byte-preservation
---

# WI-1112 — Windows archive byte preservation

This successor addresses four failures in the Windows repository-library job for [PR #1074](https://github.com/xinglun/ai-cockpit/pull/1074), run [40271040290](https://github.com/xinglun/ai-cockpit/actions/runs/40271040290/job/121404907259). Two fixed-history tests failed while cloning a local repository from a Windows path; two more failed strict Markdown byte comparisons after `.ai` evidence was checked out with CRLF line endings.

## Scope and boundaries

- Keep the local Git clone isolated with `--shared --no-checkout`; make its source argument portable without treating a Windows path as a remote transport URL.
- Preserve the exact bytes of `.ai` governance artifacts in current and historical fixture checkouts, including with `core.autocrlf=true`.
- Keep digest checks and byte-equality assertions strict. A one-byte change must remain rejected; do not normalize report contents to hide checkout conversion.
- Add a separately versioned, read-only delivery-continuity section to the active successor handoff. It validates the bound predecessor archive and recovery lineage, separates the local archive from pending external validation, and displays only Runtime-admitted next actions. Invalid lineage stays unknown; `OutcomeDelivery.deliveryState` remains a transport state.
- Let the read-only material plan show exact predecessor-reviewed Unknown members only after validating the archived receipt, request, source revision, repository, Contract lineage, and each member's path/kind/cause/hunk/blob identity. This is reference context only: it never clears a current Unknown, transfers a decision, or satisfies the successor's own request.
- Limit changes to the 27 paths declared by the active Contract. Do not change CI workflows, global Git settings, or WI-1111's archived records.

## Verification plan

Add focused regressions for local repository paths containing spaces, current and fixed historical bytes under an autocrlf-enabled checkout, and rejection after a one-byte evidence mutation. Cover validated and corrupted predecessor lineage, preserved archive bytes, changed/new material identities, and the fact that predecessor context cannot discharge current Unknowns. Run targeted repository, CLI, and MCP tests, then the Contract-declared workspace verification and zero-warning Clippy. Hosted Windows CI remains pending and must not be claimed until a later authorized run executes on the final delivery HEAD.

## Current status

The successor was created from the verified archived WI-1111 after its Windows repository-library job failed. The Windows byte-preservation tests, validated-lineage continuity test, material-reference identity test, and CLI/MCP successor-handoff parity test pass locally; related repository material-review (41/41), CLI material-review (4/4), MCP material-review (2/2), and Outcome report (29/29) tests pass. Full Contract verification and zero-warning Clippy have now passed locally on Linux; hosted Windows CI remains pending, and no Windows-hosted success is claimed. The predecessor archive and failed hosted run remain unchanged.
