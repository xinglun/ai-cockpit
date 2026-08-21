---
author: AI Cockpit maintainers
title: "Reference Source Parity"
description: "Evidence-backed comparison between the Rust runtime and the reference AI Cockpit template."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - reference_parity
---

# Reference source parity

This page records the capability comparison between the Rust runtime and the
reference AI Cockpit product. It is a product-boundary reference for adopters
and reviewers; implementation history is kept outside the reader route.

## Parity matrix

| Reference concern | Rust runtime status | Evidence and boundary |
| --- | --- | --- |
| Reader-first entry and language switching | Implemented | Root README files link to one another and the reader route separates adopter and maintainer material. |
| Purpose, problem, architecture, and capability overview | Implemented | `docs/philosophy*`, `docs/architecture*`, and `docs/capabilities*` describe the Rust runtime and its external responsibilities. |
| Shared Runtime with request-scoped repository contexts | Implemented | `docs/architecture/runtime-topology*`, explicit `--repo` CLI options, and repository isolation tests. |
| Repository attachment and minimum scaffold | Implemented | `attach`, `.ai/cockpit.toml`, `.ai/project.json`, `.ai/agent-interface.json`, and attach tests. |
| Explicit Agent Discovery / Adapter layer | Implemented | `agent list/install/doctor/repair/detach`, owned managed sections, and `.ai/adapters/<provider>.json`. `attach` does not modify Agent files. |
| Work Item lifecycle and governance decisions | Implemented | Contract, preflight, verification evidence, archive, close, and human decision records. |
| Bounded verification and fail-closed evidence reuse | Implemented | Runtime identity, snapshot/toolchain/environment bindings, receipt store, and workspace verification suite. |
| MCP repository binding | Implemented | Repository-bound stdio MCP service and CLI/MCP parity tests. |
| Public Release and fresh-adopter acceptance | Implemented | The public-binary harness, Release evidence, and post-publication CI job are available. |
| Runtime-only upgrade versus repository migration | Implemented | `compatibility`, `migrate plan`, and approved `migrate apply` preserve historical evidence and bind Runtime identity. |
| N-1 old-adopter upgrade acceptance | Available as a public-artifact harness | The harness covers old-schema detection, an approval gate, history preservation, and continued operation; each Release workflow must explicitly enable this gate. |
| Reference installer, Makefile, and V1 helper scripts | Intentionally not copied | The Rust project distributes the Rust binary and keeps installation/provider configuration separate from repository state. |

## What is complete

The Rust implementation covers the reference product's essential user-visible
boundary: one installed Runtime can govern many independently attached
repositories; repository state is isolated; Agent discovery is explicit and
owned; decisions are evidence-bound; and public-release acceptance is repeatable.

The current project intentionally keeps `cockpit.toml` as TOML. The reference
template's JSON project/profile records are represented by the Rust Protocol
files where appropriate; changing `cockpit.toml` to JSON is not part of parity.

## Current boundary

The reader route, Runtime migration boundary, and public-artifact acceptance
harnesses are implemented and documented. Future changes must preserve the separation between
shared Runtime upgrades, explicit repository migration, and repository-local
evidence.
