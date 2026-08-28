---
author: AI Cockpit maintainers
title: "Performance diagnosis"
description: "Evidence-only diagnosis of governance cost for one repository-bound Work Item."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Performance diagnosis

[简体中文](performance-diagnosis.zh-CN.md) · [日本語](performance-diagnosis.ja.md)

Performance diagnosis explains measured governance cost; it does not change
governance. The Runtime's request-scoped `diagnose` output and verification
cost observations can report snapshot work, files read/hashed, verification
runs, executed/reused nodes, worker/process counts, elapsed time, and bounded
bottleneck hints for one repository and optional Work Item.

Reports must keep these distinctions:

- execution and reuse are physical observations, while each Work Item still
  receives its own identity-bound evidence receipt;
- local process time is not proof of provider wait, human wait, token usage,
  release time, or adopter speedup;
- malformed, cross-Work-Item, mismatched, or incomplete observations stay
  unknown/partial and cannot lower a required verification route;
- comparisons are valid only for matching repository, Runtime, profile,
  policy, command, stage, and input identities.

The reference JSONL parser and its report wire shape are not Runtime protocol
requirements. AI Cockpit deliberately does not invent P95, provider wait, or
enterprise performance claims. Use [Governance cost metrics](governance-cost-metrics.md)
and [Governance profiles](governance-profiles.md) for the authority boundary.

The same advisory boundary applies to every adopter with explicit `--repo`:
performance facts are local telemetry, not a global project state or a
permission to skip checks.
