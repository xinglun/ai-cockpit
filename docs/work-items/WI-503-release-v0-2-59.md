---
author: AI Cockpit maintainers
title: "WI-503 — release v0.2.59 and public adopter acceptance"
description: "Publish the next identity-bound Runtime release and prove the public artifact before resuming reference parity work."
audience:
  - adopter
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-503-release-v0-2-59
workItemId: WI-503-release-v0-2-59
---

# WI-503 — release v0.2.59 and public adopter acceptance

[简体中文](WI-503-release-v0-2-59.zh-CN.md) · [日本語](WI-503-release-v0-2-59.ja.md)

## Intent

Publish an immutable `v0.2.59` Runtime release after the WI-502 terminal
documentation gate fix. The public binary must work from zero in an isolated
adopter, pass the N-1 acceptance path, and be installed on this repository
before the next reference-source comparison batch begins.

## Scope

- Align workspace package versions, lockfile entries, and current tri-language
  release/versioning guidance to `v0.2.59` while preserving historical facts.
- Register this release Work Item in the tri-language reference-parity ledgers.
- Pass a reviewed hosted PR before creating the annotated tag from synchronized
  `main`; publish archives, checksums, SBOM, provenance, and manifest.
- Run public adopter and N-1 acceptance only from downloaded immutable artifacts,
  including isolation, evidence binding, `not_ready` scaffold, and cleanup proofs.
- Install the published binary on this repository and verify inspect, status,
  doctor, Agent doctor, and documentation-promotion health.

## Out of scope

The local reference source, the object/adopter repository, global Agent/MCP or
Homebrew configuration, source/workspace binary fallback, unrelated Runtime
redesign, and hand-editing generated governance records.

## Acceptance criteria

1. Workspace packages, `Cargo.lock`, and current tri-language release documents
   identify `v0.2.59` without rewriting prior release history.
2. Hosted checks pass on the reviewed PR before the annotated `v0.2.59` tag is
   created from the synchronized default branch.
3. The public Release exposes identity-bound archives, SHA256 checksums, SBOM,
   provenance, and release manifest.
4. Public adopter and N-1 acceptance use only downloaded immutable artifacts,
   prove isolation and temporary-root cleanup, and preserve
   `first-adopter-smoke=not_ready`.
5. The published binary is installed on this repository; inspect, status,
   doctor, Agent doctor, and post-close documentation checks remain healthy.
6. This Work Item has a visible human Outcome, archive, finalization, close,
   and exact branch/worktree cleanup before publication.

## Verification

```text
cargo test --locked --workspace
```

Release publication and public acceptance are post-release evidence. A failed
publication remains immutable failed history and is never relabeled or reused.

## Boundary

The Runtime binary is shared, while this repository's Protocol, Work Items,
evidence, knowledge, and adapters remain repository-local. Publishing the
Runtime never implicitly attaches or mutates another repository.
