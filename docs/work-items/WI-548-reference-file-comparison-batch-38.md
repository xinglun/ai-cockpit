---
author: AI Cockpit maintainers
title: "WI-548 — Governance and boundary script comparison batch 38"
description: "Compare thirteen pinned reference scripts and record Rust-native or external boundaries without copying source implementation."
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-548-reference-file-comparison-batch-38
lastVerifiedBy: WI-548-reference-file-comparison-batch-38
terminalArchive: .ai/work-items/archive/WI-548-reference-file-comparison-batch-38.contract.json
terminalVerification: .ai/evidence/WI-548-reference-file-comparison-batch-38.verification.json
terminalFinalization: .ai/decisions/WI-548-reference-file-comparison-batch-38.finalize.json
terminalDecision: .ai/decisions/WI-548-reference-file-comparison-batch-38.close.json
---

# WI-548 — Governance and boundary script comparison batch 38

## Objective

Read the next thirteen maintained reference scripts one by one at pinned local
commit `fde3380f81fea5fd2e288f7a8849f737dc074060`. Record semantic parity and
non-claims for the shared Rust Runtime and attached adopter repositories. This
batch does not copy Python modules, Make orchestration, provider state, or
source JSON wire formats.

## File-level result

| Reference path | Decision | Rust boundary |
| --- | --- | --- |
| `scripts/ai_derived_artifacts.py` | `implemented-different-by-design` | Typed Contract/evidence/archive/Outcome projections keep derived views non-authorizing; no source registry is copied. |
| `scripts/ai_detached_uninstaller.py` | `reference-only` | Installed-lifecycle docs cover proposal, ownership, bounded removal, and retention; Rust has no detached uninstaller. |
| `scripts/ai_disable_enable.py` | `reference-only` | Explicit repository attachment and request-scoped Runtime replace a global installer toggle. |
| `scripts/ai_doctor.py` | `implemented-different-by-design` | Repository-bound Rust `doctor` covers protocol/runtime/compatibility and fail-closed diagnostics; provider toolchains remain adopter facts. |
| `scripts/ai_documentation_authority.py` | `implemented-different-by-design` | `.ai` read-set, current/reference routes, frontmatter, and documentation gates provide one authority route. |
| `scripts/ai_documentation_journey.py` | `implemented-different-by-design` | Tri-language current/getting-started/reference indexes preserve the reader journey. |
| `scripts/ai_domain_model.py` | `implemented-different-by-design` | Typed Core/Protocol/repository lifecycle services own transitions, evidence, identity, and fail-closed decisions. |
| `scripts/ai_enterprise_control_evidence.py` | `implemented-different-by-design` | Assurance, expiry, retention, and delegated evidence remain explicit; local receipts cannot become enterprise verdicts. |
| `scripts/ai_evidence_dependencies.py` | `implemented-different-by-design` | Verification binds Work Item, repository, snapshot, Contract, profile, policy, command, stage, runner, and Runtime identity. |
| `scripts/ai_external_handoff.py` | `implemented-different-by-design` | Typed release/MCP/Outcome handoffs preserve digest-bound external responsibility without provider execution in Core. |
| `scripts/ai_external_identity.py` | `implemented-different-by-design` | Typed authority and delegated evidence preserve assurance levels without local person authentication. |
| `scripts/ai_final_north_star_acceptance.py` | `implemented-different-by-design` | Final replacement acceptance retains external adopter/provider evidence boundaries and limitations. |
| `scripts/ai_impact_classifier.py` | `implemented-different-by-design` | Impact derives from explicit Contract, scope, profile, and operation-time facts; unknown impact never weakens a route. |

## Findings and adopter inheritance

No portable implementation omission was found. The detached uninstaller and
global disable/enable modules are deliberate source/provider boundaries, not
missing Runtime features. Every attached object/adopter repository inherits
the same shared binary, explicit `--repo` binding, isolated Contract/evidence/
knowledge, and human Outcome rules. It does not inherit source installer
state, Python registries, or adopter-specific policy values.

## Acceptance

- The inventory records exactly these thirteen current paths at the pinned
  source commit, with a non-empty reason and counterpart or explicit boundary.
- No selected path remains `deferred-next-batch` or `migrate-gap`; retired
  history remains append-only.
- English, Simplified Chinese, and Japanese comparison pages and this Work
  Item page state the same decisions and adopter boundary.
- Inventory, documentation, formatting, lint, and workspace verification checks
  pass before the Work Item is finished.
