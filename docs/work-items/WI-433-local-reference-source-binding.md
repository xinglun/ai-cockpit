---
author: AI Cockpit maintainers
title: "WI-433 — local reference source binding"
workItemId: WI-433-local-reference-source-binding
description: "Bind current reference comparison to an operator-maintained local checkout and a pinned commit without network fallback or source copying."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-433-local-reference-source-binding
terminalArchive: .ai/work-items/archive/WI-433-local-reference-source-binding.contract.json
terminalVerification: .ai/evidence/WI-433-local-reference-source-binding.verification.json
terminalFinalization: .ai/decisions/WI-433-local-reference-source-binding.finalize.3dba26b4c6ab10af5e7b49d9edbdb1638014c7d8119c97ea9b995ebb7a855e41.json
terminalDecision: .ai/decisions/WI-433-local-reference-source-binding.close.json
---

# WI-433 — local reference source binding

This Work Item makes the maintained local checkout at
`AI_COCKPIT_REFERENCE_ROOT` the only current semantic comparison source.
`tests/conformance/reference-source.lock` records the exact commit. Missing,
dirty, or mismatched checkouts fail closed; hosted CI uses the offline corpus
and never fetches a public reference repository.

The historical inventory remains immutable and is not silently rebaselined.
This is semantic parity and governance-boundary documentation, not a copy of
the reference Runtime, Python modules, Make rules, or toolchains.

[简体中文](WI-433-local-reference-source-binding.zh-CN.md) · [日本語](WI-433-local-reference-source-binding.ja.md)
