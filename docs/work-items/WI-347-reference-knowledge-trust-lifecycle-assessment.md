---
author: AI Cockpit maintainers
title: "WI-347 — Knowledge, input trust, installed lifecycle, and Japanese capability assessment"
workItemId: WI-347-reference-knowledge-trust-lifecycle-assessment
description: "Compare the next ten pinned reference paths and publish bounded Rust-native, tri-language mappings."
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
terminalArchive: .ai/work-items/archive/WI-347-reference-knowledge-trust-lifecycle-assessment.contract.json
terminalVerification: .ai/evidence/WI-347-reference-knowledge-trust-lifecycle-assessment.verification.json
terminalFinalization: .ai/decisions/WI-347-reference-knowledge-trust-lifecycle-assessment.finalize.json
terminalDecision: .ai/decisions/WI-347-reference-knowledge-trust-lifecycle-assessment.close.json
capabilityClaims:
  - reference_parity
---

# WI-347 — Knowledge, input trust, installed lifecycle, and Japanese capability assessment

[简体中文](WI-347-reference-knowledge-trust-lifecycle-assessment.zh-CN.md) · [日本語](WI-347-reference-knowledge-trust-lifecycle-assessment.ja.md)

## Intent and boundary

This Work Item compares the next ten paths at pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`. It publishes adopter-readable
Rust-native mappings for implementation Knowledge, input provenance, installed
Runtime lifecycle, instruction traceability, human-report semantics, and the
bounded Japanese capability assessment.

The target keeps one shared external Runtime and explicit `--repo` repository
contexts. Source Python/Make/YAML orchestration, generated assessment bytes,
provider-global configuration, and source JSON wire compatibility are out of
scope. A documented difference is not a claim that the source and target have
identical commands or fields.

## File-by-file decisions

| Pinned reference path | Classification | Bounded target decision |
| --- | --- | --- |
| `docs/reference/human-report-semantic-quality.md` | `implemented-different-by-design` | Map the decision-view order and forbidden-claim boundary to the human-benefit, task-outcome, and Outcome pages. |
| `docs/reference/implementation-knowledge.ja.md` | `implemented-different-by-design` | Provide a Japanese reader route for the typed, read-only Knowledge projection. |
| `docs/reference/implementation-knowledge.md` | `implemented-different-by-design` | Document current deterministic CLI/MCP filters and explicitly bound out date/commit/supersession dimensions that are not implemented. |
| `docs/reference/implementation-knowledge.zh-CN.md` | `implemented-different-by-design` | Provide the Chinese route with the same filter and evidence boundary. |
| `docs/reference/input-trust-dataflow.ja.md` | `implemented-different-by-design` | Map source provenance guidance to typed Rust origins and traceable derivations. |
| `docs/reference/input-trust-dataflow.md` | `implemented-different-by-design` | Document content/tool-output classification, cross-step preservation, and fail-closed injection handling. |
| `docs/reference/input-trust-dataflow.zh-CN.md` | `implemented-different-by-design` | Provide the Chinese route and explicit non-authentication boundary. |
| `docs/reference/installed-lifecycle.md` | `implemented-different-by-design` | Map shared installation, explicit attach, immutable Release acceptance, and separate migration/rollback ownership. |
| `docs/reference/instruction-traceability.md` | `implemented-different-by-design` | Map the inventory and Work Item evidence/closure chain to the source forward/reverse traceability responsibility. |
| `docs/reference/japanese-capability-assessment.json` | `implemented-different-by-design` | Map the source assessment to tri-language docs and executable presentation/adversarial checks without importing source bytes or claiming general fluency. |

All ten rows are registered in the machine inventory and in the tri-language
comparison ledger. The adopter boundary is part of the acceptance: facts,
knowledge, evidence, adapter records, and decisions remain local to each
repository even though the Runtime binary is shared.

## Acceptance and verification

- Every pinned path appears exactly once with the listed classification and a
  non-empty counterpart/reason; no `deferred-next-batch` or `migrate-gap` row
  remains in this batch.
- The five new reference pages have English, Chinese, and Japanese links and
  state the source/target semantic and non-wire boundary.
- Knowledge docs do not advertise unsupported date/commit/supersession filters;
  input-trust docs do not treat content as identity or authorization;
  installation docs do not conflate Runtime installation with repository
  attachment or migration; and Japanese docs make the no-general-fluency
  limitation explicit.
- Inventory, documentation metadata/links, governance integrity, comparison
  and parity checks pass; no source Python/Make/V1 file or global Agent/MCP
  configuration is added.
- Installed Runtime lifecycle is executed with explicit repository context:
  checkpoint → verify → finish → archive → reviewed PR/merge → close, with a
  visible human Outcome and exact branch/worktree cleanup.

Pinned reference commit: `e5acb677da6621004d96f0ef353c58fe8d3acfbf`.
Target base commit: `6ddd41d85b972a663fee85562592fc247749bf49`.
