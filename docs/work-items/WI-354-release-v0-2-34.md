---
author: AI Cockpit maintainers
title: "WI-354 — v0.2.34 release preparation"
workItemId: WI-354-release-v0-2-34
description: "Prepare the v0.2.34 release route after the lifecycle cleanup guard and hand off public-artifact acceptance."
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-354-release-v0-2-34
capabilityClaims: [release_distribution]
---

# WI-354 — v0.2.34 release preparation

[简体中文](WI-354-release-v0-2-34.zh-CN.md) · [日本語](WI-354-release-v0-2-34.ja.md)

## Intent and boundary

Prepare v0.2.34 from the reviewed default branch after WI-352 closed the
lifecycle cleanup guard. Align the workspace version and current installation
documentation, then publish only through the reviewed hosted release route.

This Work Item does not rewrite historical Release truth or claim public
artifact installation before the tag is published. A post-release successor
must download the immutable public archive, install it, and verify the current
repository and adopter boundary.

## Scope

- Align `Cargo.toml`, `Cargo.lock`, and the current tri-language release,
  distribution-architecture, and versioning routes to v0.2.34.
- Preserve failed v0.2.30 and v0.2.32 publication history as history.
- Run documentation, version-consistency, governance-integrity, release-policy,
  and complete workspace gates before tagging.
- Publish the exact reviewed tag through `.github/workflows/release.yml`,
  binding manifest, checksums, SBOM, provenance, archive smoke, and staged
  adopter evidence.
- Hand off public binary installation and current-repository acceptance to a
  post-release successor.

## Out of scope

WI-351/WI-353 recovery work, new Runtime governance behavior, external
Homebrew-tap publication, global Agent/MCP configuration, second-technology
adopter coverage, and post-release receipt contents are outside this boundary.

## Acceptance and verification

- All workspace packages and `Cargo.lock` resolve to 0.2.34 and the tag is
  `v0.2.34`.
- Current release, distribution-architecture, and versioning documents in all
  three languages name v0.2.34 while retaining prior failed-release facts.
- The reviewed source route and hosted release gates pass before publication.
- The tagged workflow binds the release manifest, `SHA256SUMS`, five target
  archives, target SBOMs, provenance, and staged adopter gates to one commit.
- This Work Item records no post-release installation success; that claim
  belongs to the immutable public-artifact successor.

Declared checks include `cargo test --locked --workspace`, documentation and
release consistency scripts, release-policy tests, and the hosted quality,
Windows, behavioral-oracle, archive, SBOM, and staged-adopter jobs.
