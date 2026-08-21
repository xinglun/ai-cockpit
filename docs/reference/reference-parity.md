---
author: AI Cockpit maintainers
title: "Reference Source Parity"
description: "Evidence-backed comparison between the Rust runtime and the reference AI Cockpit template."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: wi-41-reference-parity
capabilityClaims:
  - reference_parity
---

# Reference source parity

This page records the comparison between `xinglun/ai-cockpit` and the reference
source `spirex-ds-dev/ai-cockpit-template`. The reference snapshot used for this
review was commit `e5acb67`; the Rust runtime baseline was `031f67d`.

The comparison is a boundary audit, not a request to copy the reference
implementation. The Rust project is a separate V2 runtime and must not install
V1 Python modules, Makefile helpers, or V1 repository state.

## Parity matrix

| Reference concern | Rust runtime status | Evidence and boundary |
| --- | --- | --- |
| Reader-first entry and language switching | Partial, corrected by WI-41 | Root README files now link to one another. The full reader route and removal of internal history remain WI-42. |
| Purpose, problem, architecture, and capability overview | Implemented | `docs/philosophy*`, `docs/architecture*`, and `docs/capabilities*` describe the Rust runtime and its external responsibilities. |
| Shared Runtime with request-scoped repository contexts | Implemented | `docs/architecture/runtime-topology*`, explicit `--repo` CLI options, and repository isolation tests. |
| Repository attachment and minimum scaffold | Implemented | `attach`, `.ai/cockpit.toml`, `.ai/project.json`, `.ai/agent-interface.json`, and attach tests. |
| Explicit Agent Discovery / Adapter layer | Implemented | `agent list/install/doctor/repair/detach`, owned managed sections, and `.ai/adapters/<provider>.json`. `attach` does not modify Agent files. |
| Work Item lifecycle and governance decisions | Implemented | Contract, preflight, verification evidence, archive, close, and human decision records. |
| Bounded verification and fail-closed evidence reuse | Implemented | Runtime identity, snapshot/toolchain/environment bindings, receipt store, and workspace verification suite. |
| MCP repository binding | Implemented | Repository-bound stdio MCP service and CLI/MCP parity tests. |
| Public Release and fresh-adopter acceptance | Implemented | WI-40 harness, public `v0.1.1` evidence, and post-publication CI job. |
| Runtime-only upgrade versus repository migration | Not yet implemented | Current versioning docs state the boundary, but there is no `compatibility` or `migrate` command. WI-43 owns the implementation. |
| N-1 old-adopter upgrade acceptance | Not yet implemented | Fresh-adopter acceptance exists; an old-adopter upgrade matrix is WI-44. |
| Reference installer, Makefile, and V1 helper scripts | Intentionally not copied | The Rust project distributes the Rust binary and keeps installation/provider configuration separate from repository state. |
| Reference source historical Work Items and internal progress plans | Not a product capability | Internal history is being removed from reader routes by WI-42; archived evidence remains auditable in Git. |

## What is complete

The Rust implementation covers the reference product's essential user-visible
boundary: one installed Runtime can govern many independently attached
repositories; repository state is isolated; Agent discovery is explicit and
owned; decisions are evidence-bound; and public-release acceptance is repeatable.

The current project intentionally keeps `cockpit.toml` as TOML. The reference
template's JSON project/profile records are represented by the Rust Protocol
files where appropriate; changing `cockpit.toml` to JSON is not part of parity.

## Follow-up work

- WI-42 makes the documentation home reader-first, removes progress-history
  claims from adopter-facing pages, and normalizes the three language routes.
- WI-43 adds the explicit Runtime compatibility gate and versioned repository
  migration protocol without rewriting historical evidence.
- WI-44 adds N-1 old-adopter acceptance to the release evidence matrix.

These are separate Work Items because they change different authority surfaces:
documentation navigation, repository/runtime behavior, and release acceptance.
