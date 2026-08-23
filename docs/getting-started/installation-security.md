---
author: AI Cockpit maintainers
title: "Strict installation security"
description: "The supply-chain boundary for a shared AI Cockpit Runtime installation."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# Strict installation security

Use the public immutable Release route documented in
[Release and distribution](../release/distribution.md). The archive filename,
target, SHA-256 entry, release manifest, tag, and optional provider attestation
must identify the same artifact. A tag or upload by itself is not installation
evidence, and a checksum mismatch is a stop condition.

The boundaries are explicit:

- a private mirror needs its own independently protected metadata, artifacts,
  digests, and owner; the Runtime does not attest that operator;
- a local source build is contributor evidence, not a substitute for the
  installed immutable public Release used by adopter acceptance;
- an SBOM is an inventory, while provenance is a separate source/build claim;
- neither local repository evidence nor an Agent prompt proves enterprise
  identity, isolation, compliance, or provider controls.

Do not silently fall back to a moving branch or an older artifact. Record and
resolve any exception with the responsible release or security owner. Continue
with [Security and release verification](security-release-verification.md).

[Installation](installation.md) | [中文](installation-security.zh-CN.md) | [日本語](installation-security.ja.md)
