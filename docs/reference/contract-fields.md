---
author: AI Cockpit maintainers
title: "Contract and Summary fields"
description: "A Rust Runtime field mapping for Work Item Contracts and Summaries."
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - contract_field_mapping
---

# Contract and Summary fields

This page explains how the current Rust Runtime maps the reference source's
Contract and Summary concepts. It is a field mapping, not a second schema and
not a promise that every reference field is implemented. The Runtime keeps
repository protocol state under `.ai/`; its executable is installed once
outside the governed repository.

Status meanings:

- **Implemented** — the field is read, written, or validated by the current
  Runtime boundary and has a defined repository-local meaning.
- **Partial** — the field is readable or represented, but the reference's
  broader semantics are not a current Runtime guarantee.
- **External** — the fact belongs to an Agent host, provider, organization, or
  another system; the Runtime may bind or display evidence but does not invent it.

## Work Item Contract (`*.contract.json`)

| Field | Rust Runtime mapping | Status |
| --- | --- | --- |
| `protocolVersion` | Repository Protocol version; currently `1`. | Implemented |
| `contractVersion` | Optional typed Contract V2 opt-in; legacy protocol records remain readable. | Implemented |
| `repositoryId` | Repository identity derived from the attached repository and required for isolation. | Implemented |
| `workItemId`, `mode`, `state`, `createdAt` | Work Item identity and lifecycle metadata. | Implemented |
| `intent`, `goal` | Human-owned purpose; `intent` supports legacy text or structured `businessGoal`, `userGoal`, `problem`, `constraints`, `nonGoals`, and `rationale`. | Implemented |
| `scope`, `outOfScope` | Repository-relative implementation boundary; unsafe or ambiguous paths fail closed. | Implemented |
| `risk`, `authority` | Declared risk and authority used by preflight; the repository record does not authenticate a person. | Implemented / External identity boundary |
| `acceptanceCriteria` | Human-owned acceptance statements; numbered `A1:` criteria can bind Summary evidence. | Implemented |
| `requiredEvidenceClasses` | Required evidence categories for lifecycle completion. | Implemented |
| `sources` | Legacy source strings or typed `{path, reason}` references. | Implemented |
| `verification` | Legacy verification strings or typed `{check, required}` declarations; declarations never replace fresh execution. | Implemented |
| `baseRevision` | Snapshot-derived starting revision for the Work Item. | Implemented |
| `projectProfileDigest`, `repositorySnapshotDigest` | Content bindings for the attached project profile and repository snapshot. | Implemented |
| `problemStatement`, `riskAssessment`, `agentCapability`, `executionDecision` | Strictly typed optional V2 safety and review inputs. Non-continue decisions stop preflight. | Implemented |
| `destructiveChangePolicy`, `rollbackNote`, `unknowns`, `notCodable` | Explicit safety, recovery, and unresolved-state declarations. | Implemented |
| `scenarioCoverage` | Optional high-risk scenario projection; required/unverified scenarios fail closed before checkpoint. | Implemented |
| `concurrencyBoundary` | Optional Contract-owned path boundary and slot authorization for parallel Work Items. | Implemented |
| `checkpointPolicy`, `humanDecisionPoints`, `documentationImpact`, `performanceImpact`, `governanceProfile`, and similar extensions | Additive protocol values are preserved only where the current typed validator defines behavior; no generic field is an implicit approval. | Partial |

`authority: authorized` is a repository-local declaration. Enterprise identity,
provider verification, organization policy, and approval authenticity remain
external evidence and must not be inferred from Contract bytes.

## Change Summary (`*.summary.json`)

| Field | Rust Runtime mapping | Status |
| --- | --- | --- |
| `workItemId`, `repositoryId`, `mode`, `state` | Contract and repository binding plus serial lifecycle state. | Implemented |
| `changedPaths` | Snapshot-observed changed paths used for scope and archive checks. | Implemented |
| `checkpointCount` | Exactly-one checkpoint gate for the current lifecycle. | Implemented |
| `preflightState`, `preflightAt`, `preflightContractDigest`, `preflightDecisionDigest`, `preflightRepositorySnapshotDigest` | Repository-bound preflight decision and freshness bindings. | Implemented |
| `scenarioCoverage` | Summary-side scenario statuses, evidence, and reasons validated against the Contract. | Implemented |
| `acceptanceEvidence` | Stable acceptance IDs mapped to explicit evidence and intent alignment. | Implemented |
| `intentAlignment` | Optional resolved/unresolved projection; missing alignment remains unknown. | Implemented |
| `finalDimensions` | Exact twenty-dimension receipt with `GO`, `CONDITIONAL_GO`, or `NO_GO`; optional `fourPillarProjection` is presentation-only. | Implemented |
| `verification` | Runtime execution receipt is written to `.ai/evidence/`; it is never satisfied by a path-existence check. | Implemented |
| `outcome`, archive manifest, and human decision | Generated terminal projections under `.ai/work-items/archive/` and `.ai/decisions/`. | Implemented |
| `reviewReadiness`, `residualRisks`, `knownGaps`, `followUps`, `documentationAlignment` | Useful reference concepts, but not a universal typed Summary contract in this Runtime. | Partial |
| Provider, enterprise, hosted-CI, attestation, SBOM, and organizational approval claims | Imported or linked as delegated evidence when available; the Runtime does not generate provider authority. | External |

## Boundaries

The Runtime preserves source-language Contract text and does not machine
translate governance facts. Human-facing Outcome localization changes labels
and presentation only. Missing, stale, contradictory, malformed, or
identity-mismatched fields remain yellow or red according to the applicable
gate; they never become green through a documentation projection.

Use [Reference source parity](reference-parity.md) for the feature-level
comparison and [Commands](commands.md) for the repository-bound CLI route.
