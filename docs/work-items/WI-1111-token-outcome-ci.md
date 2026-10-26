---
author: AI Cockpit maintainers
title: "WI-1111-token-outcome-ci — Usage intake and Outcome parity"
description: "Implemented and archived; native human close and later delivery remain pending."
audience:
  - maintainer
  - reviewer
workItemId: WI-1111-token-outcome-ci
status: implemented
authority: native-contract
lastVerifiedBy: WI-1111-token-outcome-ci
---

# WI-1111-token-outcome-ci — Usage intake, Outcome parity and bounded CI acceleration

Implementation and local verification are recorded. The native Work Item was archived on 2026-10-26 and remains yellow pending a separate human close. This page does not claim that close, public delivery, hosted CI, measured performance benefit, or this task's actual token usage has happened.

## Delivered and verified locally

- Native usage intake preserves source identity, immutable receipts, nullable counters and caller provenance. Unsupported or unavailable current-session usage remains unknown; cumulative Codex JSONL is not treated as per-invocation usage.
- JSON, Markdown, CLI and MCP Outcome views preserve known/unknown usage consistently across English, Chinese and Japanese. Finish/archive bytes remain frozen; close usage has a separate cutoff.
- The report projection now binds its typed policy, producer and rendered artifacts. Fixed historical reports are replayed through finite evidence-backed rules without rewriting their bytes.
- Cheap prechecks, bounded Windows command partitioning, the stable aggregate gate, package coverage and zero-warning checks remain represented in local tests.
- The native verification receipt records 54/54 checks passed, 54 spawned, 0 reused, one worker, a 900-second per-check cap, no timeouts and 24m 15s elapsed. The archived Summary marks 13/13 scenarios verified. The fixed Runtime is `sha256:134567563cc90a9a20744f075952eae6812b21c7f606ae3722d49ae249b0bd5b`; the archive records its source/Contract/snapshot bindings.

## Evidence limits and closeout notes

- The archived Outcome has 16 acceptance result rows, but the archived Summary reports 14 `acceptanceEvidence` entries. The read-only archive validator reports `acceptanceEvidence: verified`, while the raw entries are plan declarations marked unverified and point to the former active Contract path; `/acceptanceCriteria/13` and `/acceptanceCriteria/14` have no entry. Scenario text also points to attempt `8c1471fc…` and receipt digest `6dd8ccc4…`, while the current verification record identifies attempt `9ecb62e3…` and receipt digest `c32b0b13…`. Reconcile these native references before treating the 54-check receipt or scenario statuses as one-to-one proof of all 16 criteria or before any human close.
- The successful repository test node's stdout was truncated by the Runtime capture limit. The zero-warning gate passed; an earlier failed ENOSPC attempt and its build-warning lines remain preserved.
- Three source-material syntax classifications remain Unknown after a self-declared agent review; they were not converted to Clean. Current-session usage has no receipts, and all usage counters remain null. User-visible benefit is unknown.
- After the documentation-only sync, Runtime status reports `material_review_receipt_stale` and `repository_material_inspection_unavailable` alongside the archived-close blocker. No new material review or formal verification was recorded; the existing review receipt remains tied to the earlier snapshot. The read-only archive validator still reports the frozen archive as verified, which does not clear the current status unknowns.
- Hosted final-HEAD Linux/Windows CI, a real draft-PR run and comparable performance measurements remain unverified. The local route test passes for an archived Contract with an exact branch+PR binding. This archived Contract has no `resourceContext`; the current simulated PR route therefore selected `ordinary_repository_route`, so no WI-bound PR/hosted run is claimed.
- The archive is `archived`, not closed. `close_decision_pending` and `user_visible_benefit_not_declared` remain; no close decision was recorded.

## Native evidence

- Source HEAD: `e9e6526008f73459064026852d79bbaee6400c23`.
- Archived Contract: `.ai/work-items/archive/WI-1111-token-outcome-ci.contract.json` (`sha256:a82110bf724bca97055a265b3792f95942f1b4c98f394dad52c100d350baa2f4`).
- Archive manifest and frozen outputs: `.ai/work-items/archive/WI-1111-token-outcome-ci.archive.json`, `summary.json`, `outcome.json`, `task-report.json` and `task-report.md`.
- Verification receipt: `.ai/evidence/WI-1111-token-outcome-ci.verification.json`.
- Material-review receipt: `.ai/evidence/material-inspection-review/WI-1111-token-outcome-ci/99092156923ce9d1bd906919ab9cf163a5acfa7effea682805c7dc152dd5f1e9.json`.

Public source/native/helper publication and a draft PR require a separate bounded authorization. Any hosted CI must run against the exact delivery HEAD and preserve the existing gate set. No merge, release, production activation, shared Runtime replacement or human close is authorized by this page.
