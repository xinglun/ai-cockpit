---
author: AI Cockpit maintainers
title: "Reference"
description: "User-facing command, configuration, and recovery references."
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_index
---

# Reference

Use these pages after the [current reader route](../current/README.md) and first
capability walkthrough. The route indexes keep the ordinary user journey
separate from exact machine-facing details:

- [Getting started](../getting-started/README.md) — install and first attachment.
- [Features](../features/README.md) — capability goals and boundaries.
- [Operations](../operations/README.md) — lifecycle, recovery, upgrades, and acceptance.

- [Command reference](commands.md) — command groups, required bindings, and output behavior.
- [Configuration reference](configuration.md) — `.ai/cockpit.toml`, profiles, and generated records.
- [Troubleshooting and recovery](troubleshooting.md) — stop states and the next safe action.
- [Human-facing Outcome](outcome-report.md) — the readable result, risks, evidence, and next action.
- [Governance profiles](governance-profiles.md) — proportional Light/Standard/Strict routing and its assurance boundary.
- [How to read Cockpit status](how-to-read-cockpit-status.md) — a person-facing reading order for colors, evidence, and next actions.
- [Agent workflow and review boundaries](agent-workflow.md) — inherited Work Item, Outcome, release, and safety rules.
- [Work Item style guide](work-item-style-guide.md) — human-owned intent, scope, acceptance, and executable verification guidance.
- [C# stack adaptation](csharp-adaptation.md) — Rust-native C#/.NET adopter mapping with an explicit installation boundary.
- [Android fixture adaptation](android-fixture-adaptation.md) — file-by-file Rust-native Android fixture mapping with an explicit installation boundary.
- [Flutter fixture adaptation](flutter-fixture-adaptation.md) — file-by-file Rust-native Flutter fixture mapping with an explicit installation boundary.
- [iOS Swift Package fixture adaptation](ios-swift-fixture-adaptation.md) — file-by-file Rust-native Swift Package mapping with an explicit installation boundary.
- [Python fixture adaptation](python-fixture-adaptation.md) — file-by-file Rust-native Python fixture mapping with an explicit installation boundary.
- [TypeScript web fixture adaptation](typescript-fixture-adaptation.md) — file-by-file Rust-native TypeScript/web fixture mapping with an explicit installation boundary.
- [Mixed-monorepo fixture adaptation](mixed-monorepo-fixture-adaptation.md) — file-by-file Rust-native boundary for a mixed Python/Node sample without copying its toolchains.
- [Verification route](verification-route.md) — typed stages, orthogonal tier/assurance, planning, receipts, and CI boundary.
- [Implementation knowledge](implementation-knowledge.md) — deterministic, evidence-bound records and query limits.
- [Input trust data flow](input-trust-dataflow.md) — provenance classification and fail-closed input handling.
- [Installed Runtime lifecycle](installed-lifecycle.md) — shared Runtime installation, attachment, upgrade, and rollback boundaries.
- [Instruction traceability](instruction-traceability.md) — source-path, Work Item, evidence, and closure links.
- [Verification evidence reuse runtime](verification-evidence-reuse-runtime.md) — bounded planning, protected nodes, and identity-bound receipts.
- [Verification evidence reuse decision](verification-evidence-reuse.md) — freshness bindings, invalidation, and measurable call-count reduction.
- [Verification fixture boundary](verification-fixture-boundary.md) — isolated local fixtures and their evidence limits.
- [Work Item Intelligence integration boundary](wiii-v2-integration-audit.md) — read-only Rust projection and non-wire-compatibility boundary.
- [Work Item Intelligence performance baseline](work-item-intelligence-performance-baseline.md) — reproducible local observations without governance authority.
- [Work Item lifecycle closure](work-item-lifecycle-closure.md) — reviewed merge, archive, exact cleanup, and recovery.
- [Japanese capability assessment boundary](japanese-capability-assessment.md) — evidence-bound multilingual coverage without a general fluency claim.
- [Final replacement acceptance](final-replacement-acceptance.md) — the reproducible conformance and no-copy boundary.
- [Repository Protocol v1](../protocol/v1/specification.md) — normative storage and receipt contract.

The [reference parity record](reference-parity.md) is a maintainer/reviewer
comparison. It uses explicit truth states and is not a replacement for the
adopter route or a license to copy implementation history.
