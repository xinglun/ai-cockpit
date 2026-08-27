---
author: AI Cockpit maintainers
title: "WI-331 — checks catalog and CI/release evidence"
workItemId: WI-331-checks-release-evidence
description: "Compare the pinned checks and CI/release evidence documents and record the Rust-native responsibility boundary."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-331-checks-release-evidence
terminalArchive: .ai/work-items/archive/WI-331-checks-release-evidence.contract.json
terminalVerification: .ai/evidence/WI-331-checks-release-evidence.verification.json
terminalFinalization: .ai/decisions/WI-331-checks-release-evidence.finalize.36c6617937511f6d1d30511c3e83a25ba9717d7713f8d51a9e153b1cd7cb0281.json
terminalDecision: .ai/decisions/WI-331-checks-release-evidence.close.json
capabilityClaims:
  - reference_parity
---

# WI-331 — checks catalog and CI/release evidence

## Intent and boundary

This Work Item compares the following two pinned reference paths at commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`:

| Pinned source path | Target responsibility |
| --- | --- |
| `docs/reference/checks-catalog.md` | `docs/reference/checks-catalog.*` catalog Runtime, workspace, conformance, and release checks without copying source Make/Python execution. |
| `docs/reference/ci-release-evidence.md` | `docs/reference/ci-release-evidence.*`, the versioned gate manifest, CI workflow, Release workflow, and adopter harnesses describe provider-derived evidence and its ownership. |

The target remains a shared external Rust Runtime with repository-local `.ai/`
state and explicit `--repo` context. This is semantic responsibility parity,
not source command, wire, or byte parity. Local checks, hosted provider
evidence, public Release evidence, and enterprise assurance are separate.

## Acceptance

1. Both pinned paths have an explicit inventory classification, target
   counterpart, and evidence-backed reason.
2. English, Simplified Chinese, and Japanese target pages describe the same
   check layers, profile selection, CI evidence, Release evidence, and failure
   boundaries.
3. The documents distinguish verification coverage from Evidence Assurance and
   do not promote local or staged results to provider or enterprise proof.
4. No source Makefile, Python/V1 executor, provider-global configuration, or
   generated lifecycle truth is copied or hand-edited.
5. Inventory and documentation regression gates pass with no `migrate-gap`;
   Runtime verification, reviewed PR, merge, finalization, close, and exact
   branch/worktree cleanup provide the terminal evidence.

[简体中文](WI-331-checks-release-evidence.zh-CN.md) · [日本語](WI-331-checks-release-evidence.ja.md)
