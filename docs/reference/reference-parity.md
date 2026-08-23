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
| Resource finalization and exact branch/worktree closure | Implemented | Runtime exposes `finalize-plan`, `finalize`, and `finalize-verify`; strict typed receipts bind repository, Work Item, Contract, PR, branch, worktree, and Runtime identity. Close is fail-closed for missing/unknown cleanup, and archived evidence is explicitly historical after Runtime upgrades. |
| Task Outcome and Human Benefit report | Partial | WI-136 adds a Rust-native strict report projection, append-only generated event stream, archive binding, and close final-report receipt; full reference recovery/event reconstruction remains outside this boundary. Evidence: `.ai/evidence/WI-136-task-outcome-report.verification.json`. |
| Archived Outcome path projection | Implemented | WI-148 projects newly archived generated report references and `changedPaths` from active to archive paths before binding the manifest; historical archive bytes remain immutable. |
| Contract preflight human-review gate | Implemented | Incomplete scaffold Contracts are yellow with an explicit `reviewState`, persist repository/Contract/snapshot bindings, and cannot cross checkpoint without human confirmation. |
| Contract V2 structured intent and strict schema | Implemented | WI-121 provides structured intent, typed sources/verification, strict unknown-field/duplicate-key fail-closed checks, `humanDecisionRequest`, and the preflight/checkpoint gate. |
| Contract cross-field dimensions (intent/scope/evidence/decision) | Implemented | WI-122 validates high-risk scenario coverage, stable acceptance evidence, intent alignment, and the exact twenty-dimension final receipt. The optional `fourPillarProjection` is presentation-only; there is no literal `4D` protocol field. |
| Contract-bound parallel boundary and slots | Implemented | WI-123 provides repository-local boundary validation, conservative overlap handling, and exclusive slot leases; unknown or malformed state fails closed. |
| Bounded verification and fail-closed evidence reuse | Implemented | Runtime identity, snapshot/toolchain/environment bindings, receipts, and fail-closed validation are recorded. |
| MCP repository binding | Implemented | Repository-bound stdio MCP exposes the same governed services with explicit binding. |
| Human-facing MCP projection | Implemented | Runtime validates OutcomeV2 and emits the localized `humanHandoff`; the Agent or conversation layer chooses, displays, and passes it on without treating presentation as governance authority. |
| Public Release and fresh-adopter acceptance | Partial | The complete v0.2.16 post-release adopter baseline runs on `x86_64-unknown-linux-gnu`; the other targets have build/smoke evidence, not the full lifecycle. |
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
| WI-145 — CI Runtime verification shadow | Implemented | [Work Item](../work-items/WI-145-ci-runtime-shadow.md); `.ai/evidence/WI-145-ci-runtime-shadow.verification.json`; `.ai/decisions/WI-145-ci-runtime-shadow.close.json` |
| WI-146 — Verification cost observation | Implemented | [Work Item](../work-items/WI-146-verification-cost-observation.md); [reference](verification-cost.md); `.ai/evidence/WI-146-verification-cost-observation.verification.json`; `.ai/decisions/WI-146-verification-cost-observation.close.json` |
| WI-147 — Verification route convergence | Implemented | [Work Item](../work-items/WI-147-verification-route-convergence.md); [reference](verification-route.md); `.ai/evidence/WI-147-verification-route-convergence.verification.json`; `.ai/decisions/WI-147-verification-route-convergence.close.json` |
| WI-148 — Archived Outcome path projection | Implemented | [Work Item](../work-items/WI-148-outcome-archive-path.md); [reference](outcome-report.md); `.ai/evidence/WI-148-outcome-archive-path.verification.json`; `.ai/decisions/WI-148-outcome-archive-path.close.json` |
| WI-149 — Structured release adopter decisions | Implemented | [Work Item](../work-items/WI-149-release-decision-acceptance.md); [release distribution](../release/distribution.md); `.ai/evidence/WI-149-release-decision-acceptance.verification.json`; `.ai/decisions/WI-149-release-decision-acceptance.close.json` |
| WI-150 — v0.2.16 release baseline | Implemented | [Work Item](../work-items/WI-150-release-v0-2-16.md); [v0.2.16 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.16); `.ai/evidence/WI-150-release-v0-2-16.verification.json` |
| WI-151 — v0.2.16 post-release self-governance acceptance | Implemented | [Work Item](../work-items/WI-151-post-release-v0-2-16-self-governance.md); `.ai/evidence/WI-151-post-release-v0-2-16-self-governance.verification.json`; `.ai/decisions/WI-151-post-release-v0-2-16-self-governance.close.json` |
| WI-152 — v0.2.16 documentation parity correction | Implemented | [Work Item](../work-items/WI-152-documentation-parity-after-v0-2-16.md); `.ai/evidence/WI-152-documentation-parity-after-v0-2-16.verification.json`; `.ai/decisions/WI-152-documentation-parity-after-v0-2-16.close.json` |
| WI-153 — Historical evidence projection | Implemented | [Work Item](../work-items/WI-153-historical-evidence-projection.md); `.ai/evidence/WI-153-historical-evidence-projection.verification.json`; `.ai/decisions/WI-153-historical-evidence-projection.close.json` |
| WI-154 — Policy-bound Runtime verification route | Implemented | [Work Item](../work-items/WI-154-policy-bound-runtime-route.md); [verification route](verification-route.md); `.ai/evidence/WI-154-policy-bound-runtime-route.verification.json`; `.ai/decisions/WI-154-policy-bound-runtime-route.close.json` |
| WI-155 — CI/release gate convergence | Implemented | [Work Item](../work-items/WI-155-ci-release-gate-convergence.md); [release distribution](../release/distribution.md); `.ai/evidence/WI-155-ci-release-gate-convergence.verification.json`; `.ai/decisions/WI-155-ci-release-gate-convergence.close.json` |
| WI-156 — Physical execution and Work Item evidence receipts | Implemented | [Work Item](../work-items/WI-156-physical-execution-receipt.md); `.ai/evidence/WI-156-physical-execution-receipt.verification.json`; `.ai/decisions/WI-156-physical-execution-receipt.close.json` |
| WI-157 — v0.2.17 release and adopter acceptance | Implemented | [Work Item](../work-items/WI-157-release-v0-2-17-adopter-acceptance.md); [public Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.17); `.ai/evidence/external/v0.2.17/adopter/`, `.ai/evidence/external/v0.2.17/upgrade/`, and `.ai/evidence/WI-157-release-v0-2-17-adopter-acceptance.verification.json`. |
| WI-166 — release adopter acceptance finalization | Implemented | [Archived Contract](../../.ai/work-items/archive/WI-166-release-acceptance-finalization.contract.json); [verification evidence](../../.ai/evidence/WI-166-release-acceptance-finalization.verification.json). The public and N-1 harnesses now bind resource finalization before structured close. The original v0.2.18 workflow failure remains immutable release history. |
| WI-167 — v0.2.19 public Release and adopter acceptance | Implemented | [v0.2.19 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.19); the immutable public binary and source baseline are bound by `.ai/evidence/WI-167-release-v0-2-19-recovery.verification.json`; the original v0.2.18 failure remains immutable history. |
| WI-168 — N-1 release acceptance finalization correction | Implemented | [Archived Contract](../../.ai/work-items/archive/WI-168-n-minus-one-finalization.contract.json); [verification evidence](../../.ai/evidence/WI-168-n-minus-one-finalization.verification.json). The old and new N-1 Work Items now both execute `finalize-plan` → `finalize` → `finalize-verify` before structured close. |
| WI-169 — v0.2.20 public Release and adopter acceptance | Implemented | [v0.2.20 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.20); [release workflow](https://github.com/xinglun/ai-cockpit/actions/runs/32617519173); `.ai/evidence/WI-169-release-v0-2-20.verification.json`. The public ARM64 binary, adopter acceptance, and v0.2.19-to-v0.2.20 N-1 acceptance are bound to immutable runtime identities; the original v0.2.19 N-1 failure remains immutable history. |
| WI-170 — v0.2.20 post-release parity and branch reconciliation | Implemented | [PR #125](https://github.com/xinglun/ai-cockpit/pull/125); `.ai/evidence/WI-170-post-release-parity-branch-reconciliation.verification.json`; archived Contract/Outcome and recovery decision preserve the immutable predecessor record. Verified merged branches were cleaned while dirty historical worktrees were retained. |
| WI-171 — finalization reconciliation successor | Implemented | [PR #126](https://github.com/xinglun/ai-cockpit/pull/126); `.ai/evidence/WI-171-finalization-reconciliation.verification.json`; `.ai/decisions/WI-171-finalization-reconciliation.finalize.json`; `.ai/decisions/WI-171-finalization-reconciliation.close.json`. The missing finalize-plan → finalize → finalize-verify → close chain is now recorded without rewriting WI-170 or Release truth. |
| WI-172 — v0.2.20 parity closure | Implemented | [PR #127](https://github.com/xinglun/ai-cockpit/pull/127); `.ai/evidence/WI-172-parity-closure.verification.json`; WI-170 and WI-171 are now represented as implemented in all three parity documents. |
| WI-173 — v0.2.21 release baseline | Implemented | [PR #129](https://github.com/xinglun/ai-cockpit/pull/129), merge commit `176e384efef41d2c25919734b1257170b9a13c00`; public [v0.2.21 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.21), workflow [32620133057](https://github.com/xinglun/ai-cockpit/actions/runs/32620133057), public aarch64 macOS archive SHA256 `9438b975fb25531e3b1a7e349779b917ff41c5c5fa2ab62443c472ff5385cea5`, installed public binary SHA256 `38aa88d7976d27647a9ae4419f57d309df2a08717fedccb4e9a613b370433e88`; public adopter and N-1 acceptance passed; closure [PR #130](https://github.com/xinglun/ai-cockpit/pull/130), `.ai/decisions/WI-173-release-v0-2-21.finalize.json`, `.ai/decisions/WI-173-release-v0-2-21.close.json`. |
| WI-174 — v0.2.21 post-release parity | Implemented | [PR #131](https://github.com/xinglun/ai-cockpit/pull/131), merge commit `b8b2e7a9b8f36e237fcfe507ed946278a75ba0b7`; installed public v0.2.21 documentation acceptance and post-release version consistency passed; closure [PR #132](https://github.com/xinglun/ai-cockpit/pull/132), `.ai/decisions/WI-174-post-release-parity-v0-2-21.finalize.json`, `.ai/decisions/WI-174-post-release-parity-v0-2-21.close.json`. |
| WI-175 — v0.2.22 release baseline | Implemented | [PR #133](https://github.com/xinglun/ai-cockpit/pull/133), merge commit `b75b828be99e5ddd1510d323ca3f72698d5666a7`; public [v0.2.22 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.22), workflow [32622398424](https://github.com/xinglun/ai-cockpit/actions/runs/32622398424), public aarch64 macOS archive SHA256 `b74857298bc32b53a8b7a349b5d719cb670c4d9beb25b2414b562e4a7e13a145`; the public Release workflow adopter and N-1 acceptance jobs passed. Finalization recovery is recorded by WI-176; closure [PR #134](https://github.com/xinglun/ai-cockpit/pull/134). |
| WI-176 — WI-175 finalization reconciliation | Implemented | [PR #133](https://github.com/xinglun/ai-cockpit/pull/133) and closure [PR #134](https://github.com/xinglun/ai-cockpit/pull/134); `.ai/evidence/WI-176-release-finalization-reconciliation.verification.json`; `.ai/decisions/WI-176-release-finalization-reconciliation.finalize.json`; historical WI-175 bytes were retained and the missing finalize-plan → finalize → finalize-verify → close chain was recorded. |
| WI-177 — v0.2.22 public adopter acceptance baseline | Implemented | Installed public v0.2.22 binary SHA256 `fff455d0d88d9ca4fa96b5caba85d8a6a198e131bd6ecc5a33dd9bc5cc180ab2`; public archive SHA256 `b74857298bc32b53a8b7a349b5d719cb670c4d9beb25b2414b562e4a7e13a145`; isolated adopter evidence is retained under `.ai/evidence/WI-177-post-release-adopter-v0-2-22/`, including runtime identity, attach/profile/agent doctor, not_ready scaffold, evidence reuse, lifecycle, isolation, and cleanup. The predecessor is historically superseded and closed after WI-178 recorded finalization. |
| WI-178 — v0.2.22 adopter finalization reconciliation | Implemented | [PR #135](https://github.com/xinglun/ai-cockpit/pull/135), closure [PR #136](https://github.com/xinglun/ai-cockpit/pull/136); `.ai/evidence/WI-178-post-release-adopter-finalization-reconciliation.verification.json`; `.ai/decisions/WI-178-post-release-adopter-finalization-reconciliation.finalize.json`; `.ai/decisions/WI-178-post-release-adopter-finalization-reconciliation.close.json`. The merged feature branch was deleted, the shared main worktree was explicitly retained clean, and finalize-verify passed under installed public v0.2.22. |
| WI-179 — v0.2.22 post-release parity correction | Implemented | [PR #137](https://github.com/xinglun/ai-cockpit/pull/137), closure [PR #138](https://github.com/xinglun/ai-cockpit/pull/138); `.ai/evidence/WI-179-post-release-parity-v0-2-22.verification.json`; `.ai/decisions/WI-179-post-release-parity-v0-2-22.finalize.json`; `.ai/decisions/WI-179-post-release-parity-v0-2-22.close.json`. Installed public v0.2.22 documentation and post-release consistency checks passed. |
| WI-180 — parity status closure correction | Implemented | [PR #139](https://github.com/xinglun/ai-cockpit/pull/139), closure [PR #140](https://github.com/xinglun/ai-cockpit/pull/140); `.ai/evidence/WI-180-parity-status-closure-correction.verification.json`; `.ai/decisions/WI-180-parity-status-closure-correction.finalize.json`; `.ai/decisions/WI-180-parity-status-closure-correction.close.json`. This corrective Work Item records the stale WI-179 status found by final self-check and binds the three-language correction and prevention evidence. |
| WI-181 — parity evidence binding correction | Implemented | [PR #141](https://github.com/xinglun/ai-cockpit/pull/141), closure [PR #144](https://github.com/xinglun/ai-cockpit/pull/144); `.ai/evidence/WI-181-parity-evidence-binding.verification.json`; `.ai/decisions/WI-181-parity-evidence-binding.finalize.json`; `.ai/decisions/WI-181-parity-evidence-binding.close.json`. The parity gate now fails closed when a closed row lacks auditable evidence bindings. |
| WI-182 — parallel lease atomic publication correction | Implemented | [PR #142](https://github.com/xinglun/ai-cockpit/pull/142), closure [PR #143](https://github.com/xinglun/ai-cockpit/pull/143); `.ai/evidence/WI-182-parallel-lease-atomic-install.verification.json`; `.ai/decisions/WI-182-parallel-lease-atomic-install.finalize.json`; `.ai/decisions/WI-182-parallel-lease-atomic-install.close.json`. Parallel lease JSON is now published atomically to prevent first-use EOF races. |
| WI-183 — v0.2.23 release baseline | Implemented | [PR #145](https://github.com/xinglun/ai-cockpit/pull/145), merge `1778e3c`; `.ai/evidence/WI-183-release-v0-2-23.verification.json`; `.ai/work-items/archive/WI-183-release-v0-2-23.archive.json`; `.ai/decisions/WI-183-release-v0-2-23.recovery.json`. The tri-language release baseline now targets v0.2.23 while retaining v0.2.22 as N-1; public Release and adopter evidence are bound by the post-release acceptance work. |
| WI-184 — v0.2.23 release finalization reconciliation | Implemented | [PR #146](https://github.com/xinglun/ai-cockpit/pull/146), merge `aabff99`; `.ai/evidence/WI-184-release-v0-2-23-finalization-reconciliation.verification.json`; `.ai/decisions/WI-184-release-v0-2-23-finalization-reconciliation.finalize.json`; `.ai/decisions/WI-184-release-v0-2-23-finalization-reconciliation.close.json`. This corrective Work Item records the predecessor recovery/finalization binding and exact branch cleanup before publication. |
| WI-185 — v0.2.23 parity closure | Implemented | `.ai/evidence/WI-185-release-v0-2-23-parity-closure.verification.json`; `.ai/work-items/archive/WI-185-release-v0-2-23-parity-closure.archive.json`; `.ai/decisions/WI-185-release-v0-2-23-parity-closure.finalize.json`; `.ai/decisions/WI-185-release-v0-2-23-parity-closure.close.json`. The three-language parity gate was extended through WI-184 before the public release. |
| WI-186 — v0.2.23 post-release public adopter acceptance | Implemented | [Work Item](../work-items/WI-186-release-v0-2-23-post-release-acceptance.md); [v0.2.23 Release](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.23); [release workflow](https://github.com/xinglun/ai-cockpit/actions/runs/32629400996); `.ai/evidence/external/v0.2.23/release-adopter-acceptance/acceptance.json`; `.ai/evidence/external/v0.2.23/adopter/acceptance.json`; `.ai/evidence/external/v0.2.23/upgrade/acceptance.json`; `.ai/evidence/WI-186-release-v0-2-23-post-release-acceptance.verification.json`. The immutable public ARM64 binary, fresh adopter lifecycle, N-1 upgrade, isolation, evidence reuse, and cleanup are bound without rewriting Release truth. |
| WI-159 — Runtime resource finalization integration | Implemented | `.ai/evidence/WI-159-resource-finalization-runtime.verification.json`; `.ai/decisions/WI-159-resource-finalization-runtime.close.json`; finalization receipt history under `.ai/evidence/external/WI-159-finalization/`. |
| WI-160 — Resource finalization and branch/worktree closure baseline | Implemented | [Work Item](../work-items/WI-160-resource-finalization-baseline.md); `.ai/evidence/WI-160-resource-finalization-baseline.verification.json`; `.ai/work-items/archive/WI-160-resource-finalization-baseline.archive.json`; `.ai/decisions/WI-160-resource-finalization-baseline.close.json`. Runtime command/receipt integration is implemented by WI-159; historical-runtime close compatibility is covered by WI-161. |
| WI-161 — Historical Runtime evidence close compatibility | Implemented | [Work Item](../work-items/WI-161-historical-runtime-close.md); archived evidence remains immutable and foreign Runtime bytes are projected as historical; regression evidence is recorded in `.ai/evidence/WI-161-historical-runtime-close.verification.json`. |
| WI-162 — Historical snapshot compatibility after archive | Implemented | `.ai/evidence/WI-162-historical-snapshot-compat.verification.json`; archived plan receipts remain bound to their recorded snapshot without rewriting history. |
| WI-163 — Historical Outcome projection | Implemented | `.ai/evidence/WI-163-historical-outcome-projection.verification.json`; historical evidence is not presented as a current verification failure. |
| WI-164 — Historical Outcome human rendering | Implemented | `.ai/evidence/WI-164-historical-outcome-render.verification.json`; tri-language handoff suppresses missing-evidence recovery wording for historical evidence. |

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

Resource finalization is a separate closure boundary: the exact branch and
worktree must pass `finalize-plan` → `finalize` → `finalize-verify` before
`close`. Unknown provider/resource state remains open, and any retained
resource requires an explicit bounded human decision. WI-160 records this
policy and its static gate; WI-159 implements the Runtime commands and
receipts. Historical verification evidence is not rewritten or treated as a
current failure after Runtime upgrades; only the new finalization receipt is
bound to the closing Runtime.

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
