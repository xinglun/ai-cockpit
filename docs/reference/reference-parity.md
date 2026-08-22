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
starts at [Current reader route](../current/README.md).

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
| Explicit Agent Discovery / Adapter layer | Implemented | Agent installation is explicit, owned, reversible, and repository-local. |
| Work Item lifecycle and governance decisions | Partial | The core lifecycle and human decision records exist; the reference's broader status, cost, and recovery projections are not all exposed as one adopter interface. |
| Bounded verification and fail-closed evidence reuse | Implemented | Runtime identity, snapshot/toolchain/environment bindings, receipts, and fail-closed validation are recorded. |
| MCP repository binding | Implemented | Repository-bound stdio MCP exposes the same governed services with explicit binding. |
| Human-facing MCP projection | Implemented | Runtime validates OutcomeV2 and emits the localized `humanHandoff`; the Agent or conversation layer chooses, displays, and passes it on without treating presentation as governance authority. |
| Public Release and fresh-adopter acceptance | Partial | The complete v0.2.8 post-release adopter baseline runs on `x86_64-unknown-linux-gnu`; the other targets have build/smoke evidence, not the full lifecycle. |
| Second-technology-stack adopter acceptance | Deferred | The current harness uses a Cargo adopter; a separate technology stack remains future work. |
| Runtime-only upgrade versus repository migration | Implemented | Compatibility checks and explicit migration preserve historical records and bind Runtime identity. |
| N-1 old-adopter upgrade acceptance | Implemented | The public-artifact harness covers old-schema detection, approval, history preservation, and continued operation. |
| Adopter capability manifest and status projection | Deferred | Current `capability show` and `status` are truthful Runtime/repository views, not the reference's full adopter manifest/status projection. |
| Recovery state machine and rich recovery projections | Partial | Stop and recovery guidance exists; the broader paused/blocked/stale/cancelled/rollback surface remains narrower than the reference. |
| Multilingual semantic parity gate | Partial | CLI human output is localized; full field-by-field semantic parity across all reports is not yet a CI gate. |
| Legacy evidence boundary | Implemented | Legacy evidence remains historical input and is never promoted to fresh green verification. |
| Contract source language | Implemented | Contract intent, scope, acceptance, and authority remain source text; translations do not rewrite Contract bytes. |
| Installation and provider configuration | External boundary | Binary delivery and provider/global configuration are separate from repository governance state. |

The matrix deliberately distinguishes a working core from complete reference
surface parity. A green row proves only the named boundary; it does not grant
external identity, provider authorization, branch protection, production
readiness, or organizational approval.

## Current boundary

One installed Runtime can govern many independently attached repositories. Each
repository owns its Protocol, Work Items, evidence, knowledge, and adapter
records. Future changes must preserve explicit repository binding, evidence
isolation, human-owned decisions, and the separation between Runtime delivery
and repository state.
