---
author: AI Cockpit maintainers
title: "WI-403 — v0.2.41 release publication and adopter acceptance"
description: "Publish the performance-batch Runtime and verify the immutable public artifact in this repository and a fresh adopter."
workItemId: WI-403-release-v0-2-41
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-403-release-v0-2-41
capabilityClaims: [release_distribution, adopter_acceptance, runtime_installation]
---

# WI-403 — v0.2.41 release publication and adopter acceptance

[简体中文](WI-403-release-v0-2-41.zh-CN.md) · [日本語](WI-403-release-v0-2-41.ja.md)

## Intent

Publish v0.2.41 from the reviewed, synchronized `main` after the Rust
performance batch, then prove that the immutable public artifact can govern
this repository and a fresh adopter.

## Boundary

This Work Item covers the exact patch-version release, release/distribution
documentation, immutable artifact installation, and post-release adopter
acceptance. It does not change governance semantics, reference-parity
implementation, business-project code, or global Agent/MCP configuration.
Acceptance must reject source-build, workspace-binary, and moving-branch
fallbacks.

## Acceptance

1. Cargo metadata and lockfile advance exactly from v0.2.40 to v0.2.41.
2. The reviewed main commit produces a public Release whose archive, SBOM,
   provenance, manifest, and SHA-256 identities agree.
3. The downloaded public binary is installed and verified here with explicit
   repository context; `inspect`, `status`, and `doctor` are healthy.
4. A fresh isolated adopter completes attach, scaffold, lifecycle, evidence
   reuse, and cleanup checks; `first-adopter-smoke` remains `not_ready`.
5. Runtime identity, repository identity, artifact digests, isolation manifests,
   acceptance output, and cleanup truth are retained as evidence.
6. English, Simplified Chinese, and Japanese release/parity documentation agree,
   and the synchronized main reports `ready_on_base` after exact cleanup.

## Verification boundary

Release publication and adopter acceptance are separate truths. A failed
post-release acceptance records `releasePublished: true` and
`adopterAcceptance: failed`; it never rewrites a published Release or historical
evidence. The acceptance harness is the only authority for binding a downloaded
binary's runtime identity to release evidence.
