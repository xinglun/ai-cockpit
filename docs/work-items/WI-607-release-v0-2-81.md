---
title: "WI-607 — v0.2.81 release and adopter acceptance"
description: "Publish the next Rust Runtime patch and verify its immutable artifact and adopter boundaries."
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-607-release-v0-2-81
lastVerifiedBy: WI-607-release-v0-2-81
---

[简体中文](WI-607-release-v0-2-81.zh-CN.md) · [日本語](WI-607-release-v0-2-81.ja.md)

# WI-607 — v0.2.81 release and adopter acceptance

## Objective

Publish the reviewed Rust Runtime as `v0.2.81`, then verify installation and
upgrade using only immutable public Release artifacts. This release contains
the GitHub Release API authentication fix and is not a source-template
migration.

## Boundary

The Work Item covers package version metadata, release/distribution,
tri-language release and parity records, and staged/public release acceptance
evidence. It does not copy the reference project's scaffold, Python/Make
implementation, or JSON wire format. Object and adopter repositories remain
read-only external acceptance targets; global Agent/MCP configuration and
historical governance bytes are out of scope.

## Acceptance

1. Workspace package versions and `Cargo.lock` resolve to `0.2.81`.
2. Release CI publishes the annotated tag, target archives, SBOM/provenance,
   Formula, checksums, and Runtime identity with matching digests.
3. Post-release adopter and N-1 acceptance use only immutable public artifacts
   and prove isolation and temporary-run cleanup.
4. English, Chinese, and Japanese release/parity records identify the `v0.2.81`
   baseline and `v0.2.80` N-1 boundary.
5. The terminal Outcome is human-visible and records status, unknowns, evidence,
   decision, and next action.

## Verification

Run `cargo test --locked --workspace`, documentation and metadata checks, the
release policy/version consistency checks, and the immutable staged/public
adopter acceptance harnesses. Record the downloaded Runtime version and
SHA-256 in the release evidence. The terminal Outcome must remain a separate,
human-visible handoff.
