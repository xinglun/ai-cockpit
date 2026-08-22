---
author: AI Cockpit maintainers
title: "Reference Source Parity"
description: "A maintainer and reviewer record of evidence-backed product-boundary comparison."
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# Reference source parity

This is an audit comparison, not an adopter instruction. It records where the
Rust Runtime matches the reference product boundary, where it is partial or
deferred, and which responsibilities remain external. The ordinary user route
starts at [Current reader route](../current/README.md). For field-level
mapping, see [Contract and Summary fields](contract-fields.md).

## Truth states

The matrix uses exactly four states:

- **Implemented** — the stated boundary is implemented and covered by current evidence.
- **Partial** — the core boundary exists, but the reference surface or assurance is broader.
- **Deferred** — intentionally not part of the current Runtime boundary.
- **External boundary** — owned by an Agent host, provider, organization, or external system.

## Parity matrix

| Reference concern | Rust Runtime status | Evidence and boundary |
| --- | --- | --- |
| Reader-first entry and language switching | Implemented | Root and route README files link to one another in English, Simplified Chinese, and Japanese. |
| Purpose, problem, architecture, and capability overview | Implemented | The philosophy, architecture, and capability routes describe the current Runtime and its owners. |
| Shared Runtime with request-scoped repository contexts | Implemented | Explicit `--repo` binding and repository isolation tests keep context and evidence separate. |
| Repository attachment and minimum scaffold | Implemented | `attach` creates the repository-owned Protocol scaffold without installing a Runtime copy. |
| Explicit Agent Discovery / Adapter layer | Implemented | Agent installation is explicit, owned, reversible, and repository-local; generated guidance carries Contract-first/pause/Summary/Outcome/closure semantics, Cursor uses canonical `.cursor/rules/ai-cockpit.mdc`, and managed legacy `.md` remains readable. |
| Work Item lifecycle and governance decisions | Partial | The core lifecycle and human decision records exist; the reference's broader status, cost, and recovery projections are not all exposed as one adopter interface. |
| Task Outcome and Human Benefit report | Partial | WI-136 adds a Rust-native strict report projection, append-only generated event stream, archive binding, and close final-report receipt; full reference recovery/event reconstruction remains outside this boundary. Evidence: `.ai/evidence/WI-136-task-outcome-report.verification.json`. |
| Contract preflight human-review gate | Implemented | Incomplete scaffold Contracts are yellow with an explicit `reviewState`, persist repository/Contract/snapshot bindings, and cannot cross checkpoint without human confirmation. |
| Contract V2 structured intent and strict schema | Implemented | WI-121 provides structured intent, typed sources/verification, strict unknown-field/duplicate-key fail-closed checks, `humanDecisionRequest`, and the preflight/checkpoint gate. |
| Contract cross-field dimensions (intent/scope/evidence/decision) | Implemented | WI-122 validates high-risk scenario coverage, stable acceptance evidence, intent alignment, and the exact twenty-dimension final receipt. The optional `fourPillarProjection` is presentation-only; there is no literal `4D` protocol field. |
| Contract-bound parallel boundary and slots | Implemented | WI-123 provides repository-local boundary validation, conservative overlap handling, and exclusive slot leases; unknown or malformed state fails closed. |
| Bounded verification and fail-closed evidence reuse | Implemented | Runtime identity, snapshot/toolchain/environment bindings, receipts, and fail-closed validation are recorded. |
| MCP repository binding | Implemented | Repository-bound stdio MCP exposes the same governed services with explicit binding. |
| Human-facing MCP projection | Implemented | Runtime validates OutcomeV2 and emits the localized `humanHandoff`; the Agent or conversation layer chooses, displays, and passes it on without treating presentation as governance authority. |
| Public Release and fresh-adopter acceptance | Partial | The complete v0.2.15 post-release adopter baseline runs on `x86_64-unknown-linux-gnu`; the other targets have build/smoke evidence, not the full lifecycle. |
| Second-technology-stack adopter acceptance | Deferred | The current harness uses a Cargo adopter; a separate technology stack remains future work. |
| Runtime-only upgrade versus repository migration | Implemented | Compatibility checks and explicit migration preserve historical records and bind Runtime identity. |
| N-1 old-adopter upgrade acceptance | Implemented | The public-artifact harness covers old-schema detection, approval, history preservation, and continued operation. |
| Adopter capability manifest and status projection | Deferred | Current `capability show` and `status` are truthful Runtime/repository views, not the reference's full adopter manifest/status projection. |
| Recovery state machine and rich recovery projections | Partial | Blocked Outcome, append-only recovery receipts, predecessor-bound retry/successor decisions, and a human/MCP projection now exist; the broader paused/stale/cancelled/rollback surface remains narrower than the reference. |
| Multilingual semantic parity gate | Partial | CLI human output is localized; full field-by-field semantic parity across all reports is not yet a CI gate. |
| Legacy evidence boundary | Implemented | Legacy evidence remains historical input and is never promoted to fresh green verification. |
| Contract source language | Implemented | Contract intent, scope, acceptance, and authority remain source text; translations do not rewrite Contract bytes. |
| Installation and provider configuration | External boundary | Binary delivery and provider/global configuration are separate from repository governance state. |

The matrix deliberately distinguishes a working core from complete reference
surface parity. A green row proves only the named boundary; it does not grant
external identity, provider authorization, branch protection, production
readiness, or organizational approval.

## Current implementation baseline

The current `main` branch contains the following reviewed Contract and
governance boundaries. The Work Item documents describe the user-visible
scope; the repository evidence paths are the machine-readable verification
record for each boundary.

| Work Item | Current Runtime status | Evidence and documentation |
| --- | --- | --- |
| WI-121 — Contract V2 | Implemented | [Work Item](../work-items/WI-121-contract-v2.md); `.ai/evidence/WI-121-contract-v2.verification.json` |
| WI-122 — Scenario, acceptance, and final dimensions | Implemented | [Work Item](../work-items/WI-122-scenarios-acceptance-final-dimensions.md); `.ai/evidence/WI-122-scenarios-acceptance-final-dimensions.verification.json` |
| WI-123 — Parallel Contract boundary and slots | Implemented | [Work Item](../work-items/WI-123-parallel-contract-boundary.md); `.ai/evidence/WI-123-parallel-contract-boundary.verification.json` |
| WI-125 — Contract V2 schema boundary | Implemented | [Work Item](../work-items/WI-125-contract-schema.md); `.ai/evidence/WI-125-contract-schema.verification.json` |
| WI-126 — Read-only status and human handoff | Implemented | [Work Item](../work-items/WI-126-status-outcome.md); `.ai/evidence/WI-126-status-outcome.verification.json` |
| WI-128 — Release acceptance cleanup | Implemented | [Work Item](../work-items/WI-128-release-acceptance-cleanup.md); `.ai/evidence/WI-128-release-acceptance-cleanup.verification.json` |
| WI-129 — Reference parity completeness | Implemented | [Work Item](../work-items/WI-129-parity-gate.md); `.ai/evidence/WI-129-parity-gate.verification.json` |
| WI-130 — Closed Work Item status projection | Implemented | [Work Item](../work-items/WI-130-status-closed-projection.md); `.ai/evidence/WI-130-status-closed-projection.verification.json`; `.ai/decisions/WI-130-status-closed-projection.close.json` |
| WI-131 — Fail-closed verification evidence timestamps | Implemented | [Work Item](../work-items/WI-131-evidence-timestamp.md); `.ai/evidence/WI-131-evidence-timestamp.verification.json`; `.ai/decisions/WI-131-evidence-timestamp.close.json` |
| WI-132 — Agent adapter and provider-surface parity | Implemented | [Work Item](../work-items/WI-132-agent-adapter-parity.md); `.ai/evidence/WI-132-agent-adapter-parity.verification.json`; `.ai/decisions/WI-132-agent-adapter-parity.close.json` |
| WI-133 — Documentation truth reconciliation | Implemented | [Work Item](../work-items/WI-133-docs-truth.md); `.ai/evidence/WI-133-docs-truth.verification.json`; `.ai/decisions/WI-133-docs-truth.close.json` |
| WI-135 — Repository-bound retention and close evidence | Implemented | [Work Item](../work-items/WI-135-repository-bound-evidence.md); `.ai/evidence/WI-135-repository-bound-evidence.verification.json`; `.ai/decisions/WI-135-repository-bound-evidence.close.json` |
| WI-136 — Task Outcome and Human Benefit report | Implemented | [Work Item](../work-items/WI-136-task-outcome-report.md); `.ai/evidence/WI-136-task-outcome-report.verification.json`; `.ai/decisions/WI-136-task-outcome-report.close.json` |
| WI-140 — Verification semantics and artifact archive integrity | Implemented | [Work Item](../work-items/WI-140-verification-semantics.md); `.ai/evidence/WI-140-verification-semantics.verification.json`; `.ai/decisions/WI-140-verification-semantics.close.json` |
| WI-141 — Policy-driven verification planner | Implemented | [Work Item](../work-items/WI-141-policy-planner.md); `.ai/evidence/WI-141-policy-planner.verification.json`; `.ai/decisions/WI-141-policy-planner.close.json` |
| WI-142 — Affected verification and dependency confidence | Implemented | [Work Item](../work-items/WI-142-affected-verification.md); `.ai/evidence/WI-142-affected-verification.verification.json`; `.ai/decisions/WI-142-affected-verification.close.json` |
| WI-143 — Intent scenario and stage binding | Implemented | [Work Item](../work-items/WI-143-intent-scenario-binding.md); `.ai/evidence/WI-143-intent-scenario-binding.verification.json`; `.ai/decisions/WI-143-intent-scenario-binding.close.json` |
| WI-144 — Cross-Work-Item physical execution reuse | Implemented | [Work Item](../work-items/WI-144-cross-work-item-dedup.md); `.ai/evidence/WI-144-cross-work-item-dedup.verification.json`; `.ai/decisions/WI-144-cross-work-item-dedup.close.json` |

## Current boundary

One installed Runtime can govern many independently attached repositories. Each
repository owns its Protocol, Work Items, evidence, knowledge, and adapter
records. Future changes must preserve explicit repository binding, evidence
isolation, human-owned decisions, and the separation between Runtime delivery
and repository state.

After a Work Item is closed, the same release-audit cycle finalizes its
tri-lingual documentation: status `implemented`, links to archived
verification/close evidence, and a matching parity-baseline row. This
documentation-truth rule does not rewrite historical evidence.

## Scenario, acceptance, and final-dimension projections

The Runtime validates (but never invents) three optional governance
projections. A high-risk Contract must declare `scenarioCoverage`, and its
Summary must provide entries with `required`, `status`, and `evidence`, plus a
`reason` when `status` is `not_applicable`. A required unverified scenario is
fail-closed for high-risk work.

Numbered acceptance criteria such as `A1: ...` opt into stable IDs and a
Summary `acceptanceEvidence` mapping. Legacy unnumbered criteria remain
readable and are not assigned IDs by the Runtime. `intentAlignment` is an
optional projection: missing alignment remains `unknown`, and resolved or
unresolved claims require explicit evidence or a reason.

Final acceptance uses the reference's exact twenty dimension names and a
receipt decision of `GO`, `CONDITIONAL_GO`, or `NO_GO`. `GO` requires verified
`real_adopter` and `provider_evidence`; missing, extra, malformed, or
identity-mismatched dimensions fail closed. An optional
`fourPillarProjection` is presentation-only. There is deliberately no
ambiguous `4D` protocol field, and the Runtime does not synthesize evidence or
turn a local projection into provider/enterprise assurance.
