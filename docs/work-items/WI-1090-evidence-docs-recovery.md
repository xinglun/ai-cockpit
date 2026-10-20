---
author: AI Cockpit maintainers
title: "WI-1090 — Evidence location and documentation recovery"
description: "Evidence location and documentation recovery"
audience:
  - maintainer
  - reviewer
authority: explicit-user-authorization
workItemId: WI-1090-evidence-docs-recovery
status: in_progress
lastVerifiedBy: WI-1090-evidence-docs-recovery
---

# WI-1090-evidence-docs-recovery — Evidence location and documentation recovery

This selected native successor repairs the actual PR1050 Windows checkout and documentation failures. Scope is exactly eight byte-preserving Agent-copy relocations, two locator files, six predecessor documentation projections, and these three successor pages. It changes no Rust, test, workflow, global configuration, or predecessor generated history. The other nineteen long-path candidates stay at their original paths and bytes.

The predecessor `WI-1090-authority-source-material-review-replacement` is archived, with seven native checks passed and hosted CI run `39344530319` failed. Its close remains pending. The append-only selection is `.ai/decisions/WI-1090-authority-source-material-review-replacement.recovery.json`. The original M6 manifests retain their logical keys; resolving the eight moved copies requires `.ai/evidence/WI-1090-authority-source-material-review-replacement/m6-short-20261020/path-map.actual.json` and full byte/hash/Git-blob checks.

This page and all three full-ID parity rows are registered in source before the first own verification receipt. Required checks are `bash tests/docs/documentation_acceptance.sh` and `bash tests/docs/parity_status_check.sh`, plus the actual migration integrity audit. Their current results and lifecycle state must be read from native evidence; registration alone claims no pass or completed delivery.

Own normal checkpoint, verification, controls, finish and archive are authorized only on actual Runtime admission. Active Contract `.ai/work-items/active/WI-1090-evidence-docs-recovery.contract.json` and archived Contract `.ai/work-items/archive/WI-1090-evidence-docs-recovery.contract.json` are lifecycle alternatives, not claims that both exist. Own verification is `.ai/evidence/WI-1090-evidence-docs-recovery.verification.json`; future finalize and close records are `.ai/decisions/WI-1090-evidence-docs-recovery.finalize.json` and `.ai/decisions/WI-1090-evidence-docs-recovery.close.json`.

Neither Work Item's close, merge, branch/worktree/provider deletion, release/tag nor Runtime adoption is authorized. PR1050 remains preserved; a separate Draft PR and actual CI results are distinct delivery evidence. Material inspection unknowns and self_declared assurance limits remain explicit. This successor does not compose WI1089 with WI1090, operate Sentinel, or establish real-consumer recovery.
