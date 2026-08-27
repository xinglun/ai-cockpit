---
author: AI Cockpit maintainers
title: "WI-327 — reference file comparison batch 07"
workItemId: WI-327-reference-file-comparison-batch-07
description: "Compare nine pinned adopter, calibration, and long-cycle reference documentation paths with evidence-backed Rust-native boundaries."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-327-reference-file-comparison-batch-07
terminalArchive: .ai/work-items/archive/WI-327-reference-file-comparison-batch-07.contract.json
terminalVerification: .ai/evidence/WI-327-reference-file-comparison-batch-07.verification.json
terminalFinalization: .ai/decisions/WI-327-reference-file-comparison-batch-07.finalize.json
terminalDecision: .ai/decisions/WI-327-reference-file-comparison-batch-07.close.json
---

# WI-327 — reference file comparison batch 07

## Intent and boundary

Compare the nine paths below from pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` one file at a time. Preserve
adopter-facing calibration, evidence, and long-cycle governance semantics
without copying source Python, Make, fixture, scanner, or internal-progress
implementation.

The shared Rust Runtime remains external and every repository request is bound
with an explicit `--repo`. This is a documentation and conformance-ledger
batch; it does not add Runtime behavior or claim source wire compatibility.

## File-level comparison

| Pinned reference path | Classification | Rust/adopter counterpart and boundary |
| --- | --- | --- |
| `docs/reference/adopter-long-cycle-validation.ja.md` | `implemented-different-by-design` | Published-binary adopter/upgrade acceptance and Japanese lifecycle/security routes preserve isolated install, lifecycle, rollback, and cleanup evidence; source multi-stack fixtures and Make/Python orchestration are not copied. |
| `docs/reference/adopter-long-cycle-validation.md` | `implemented-different-by-design` | Published-binary adopter/upgrade acceptance and lifecycle/security routes preserve isolated install, lifecycle, rollback, and cleanup evidence; source multi-stack fixtures and Make/Python orchestration are not copied. |
| `docs/reference/adoption-reality-report.md` | `implemented-different-by-design` | Runtime capability/profile/status projections and immutable adopter receipts separate template capability, adopter execution, provider evidence, and enterprise assurance. |
| `docs/reference/bandit-synchronization-security-audit.md` | `reference-only` | Source-specific historical Bandit findings and digest are not target evidence. The Rust target has no Python/Bandit surface and keeps native quality/threat-model boundaries separate. |
| `docs/reference/calibration-inventory.md` | `implemented-different-by-design` | Repository-bound profile proposal/confirmation, capability/status projections, and explicit unknowns preserve the fact/evidence boundary without copying the source Python inventory. |
| `docs/reference/calibration-profiles.ja.md` | `implemented-different-by-design` | Japanese calibration guidance and strict JSON profile policy preserve cumulative Lite/Standard/Strict controls, human selection, monotonic upgrade, and explicit downgrade evidence. |
| `docs/reference/calibration-profiles.md` | `implemented-different-by-design` | Calibration guidance and strict JSON profile policy preserve cumulative Lite/Standard/Strict controls, human selection, monotonic upgrade, and explicit downgrade evidence. |
| `docs/reference/calibration-profiles.zh-CN.md` | `implemented-different-by-design` | Chinese calibration guidance and strict JSON profile policy preserve cumulative Lite/Standard/Strict controls, human selection, monotonic upgrade, and explicit downgrade evidence. |
| `docs/reference/calibration-session-model.ja.md` | `implemented-different-by-design` | Explicit proposal, confirmation, and repository-bound facts replace the source internal Session model; no generic interactive Session or checklist authority is introduced. |

## Adopter feedback boundary

The Cursor adopter report is external validation input, not a new source wire
contract. Current Runtime v0.2.33 already provides stable lifecycle stdout JSON,
the human `work-item outcome` handoff, close-before-next-entry checks, and
fail-closed start/verification bindings. Cursor must explicitly install its
repository-local adapter and replay the durable handoff because an IDE cannot
be forced to expand stderr in chat. Diagnostic remediation, close-gap
convenience commands, and automatic controls scaffolding remain separate
product decisions and are not silently claimed here.

## Non-goals

This Work Item does not add Runtime commands, copy source Python/Make/YAML,
fixture, or Bandit files, require `Makefile.ai`, modify global Agent/MCP
configuration, or implement optional host-panel, controls-scaffold, or
close-gap conveniences. It does not change the pinned source or target commit.

## Acceptance and evidence

1. All nine pinned paths are read and have exactly one inventory record with a
   non-empty, evidence-backed reason.
2. The generated inventory records eight `implemented-different-by-design`
   entries and one `reference-only` entry for this Work Item, with no deferred
   or migrate-gap entry in the batch.
3. English, Simplified Chinese, and Japanese comparison/parity pages and this
   Work Item record agree on the source pin, classifications, and boundaries.
4. Source-specific fixture/scanner counts, internal progress, and unrun
   provider or enterprise assurance are not presented as current Runtime facts.
5. Installed Runtime inspect/status/doctor/agent doctor, focused documentation
   and conformance checks, lifecycle closure, hosted CI, and exact cleanup
   provide terminal evidence. Historical evidence is not rewritten.

[简体中文](WI-327-reference-file-comparison-batch-07.zh-CN.md) · [日本語](WI-327-reference-file-comparison-batch-07.ja.md)
