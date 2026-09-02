---
author: AI Cockpit maintainers
title: "WI-515 — release v0.2.63 and historical-adopter recovery acceptance"
description: "Publish the Runtime fix for legacy shared-worktree and direct-merge recovery, then provide immutable adopter evidence."
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-515-release-v0-2-63
lastVerifiedBy: WI-515-release-v0-2-63
---

[简体中文](WI-515-release-v0-2-63.zh-CN.md) · [日本語](WI-515-release-v0-2-63.ja.md)

# WI-515 — release v0.2.63 and historical-adopter recovery acceptance

## Intent

Publish the Runtime fix for truthful historical shared-primary `retained`
finalization and direct-merge-without-PR recovery. The object/adopter
repository remains read-only and performs its own acceptance from the public
artifact.

## Scope

- Align workspace versions and the current tri-language release/versioning
  guidance to v0.2.63 without rewriting historical release facts.
- Register this release Work Item in the tri-language parity ledgers.
- Pass hosted checks on a reviewed PR before creating the annotated tag from
  synchronized main.
- Publish archives, SHA256SUMS, SBOM, provenance, and release manifest, then
  run public adopter and N-1 acceptance using downloaded immutable artifacts.
- Install the published binary on this repository and verify inspect, status,
  doctor, Agent doctor, and documentation promotion health.

## Out of scope

The local reference source, object/adopter repositories, global Agent/MCP or
Homebrew configuration, source/workspace fallback, unrelated Runtime changes,
and hand-editing generated governance records.

## Acceptance criteria

1. Workspace packages and lockfile identify v0.2.63 while prior release facts
   remain unchanged.
2. The reviewed PR passes all required hosted checks before the annotated tag
   is created from synchronized main.
3. Public release archives, SHA256SUMS, SBOM, provenance, and manifest agree on
   tag, bytes, and digests.
4. Public adopter and N-1 acceptance use only immutable downloaded artifacts,
   prove isolation and temporary-root cleanup, and retain
   `first-adopter-smoke=not_ready`.
5. The published binary is installed here; health and post-close documentation
   checks remain green.
6. The visible human Outcome, archive, finalization, close, and exact cleanup
   are recorded before publication is declared complete.

## Verification

```text
cargo test --locked --workspace
```

Release publication and object-repository acceptance are separate evidence
boundaries. A failed publication remains immutable history and is never
relabeled or reused.

## Boundary

The Runtime binary is shared; each repository retains isolated Protocol,
Contract, evidence, knowledge, and adapter state. Publishing v0.2.63 never
implicitly attaches or mutates another repository.
