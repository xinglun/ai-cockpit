---
author: AI Cockpit maintainers
title: "WI-554 — v0.2.71 release and public-artifact acceptance"
description: "Publish the capability-surface documentation repair as an immutable Runtime release."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-554-release-v0-2-71
lastVerifiedBy: WI-554-release-v0-2-71
---

[简体中文](WI-554-release-v0-2-71.zh-CN.md) · [日本語](WI-554-release-v0-2-71.ja.md)

# WI-554 — v0.2.71 release and public-artifact acceptance

## Objective

Publish v0.2.71 from the reviewed default branch as the next immutable Runtime
baseline. This release carries the capability registry, capability-discovery
documentation, and the WI-552 reference comparison. The prior public v0.2.70
release remains historical N-1 evidence.

## Scope and boundary

- Align Cargo metadata/lockfile and current release, distribution, and
  versioning guidance in English, Simplified Chinese, and Japanese.
- Bind the release to the closed WI-552 comparison and WI-553 documentation
  promotion records.
- Produce and validate the five-target public artifacts, manifest, checksums,
  SBOM, provenance, attestation, and post-release adopter receipts.
- Object repositories, global Agent/MCP configuration, source-template copying,
  and Runtime behavior changes are outside this Work Item.

## Acceptance

1. Cargo metadata, lockfile, and current release/versioning docs identify
   v0.2.71; v0.2.70 is the immediately preceding public baseline and failed
   tags remain immutable history.
2. Release CI produces the identity-bound artifact set and all required supply-
   chain receipts.
3. Public adopter and N-1 acceptance use only downloaded v0.2.71 artifacts in
   isolated roots, prove cleanup/forbidden-root isolation, and exercise the
   same binary against this repository.
4. WI-552 and WI-553 remain closed and promoted; the release starts from a
   clean, ready default branch.

## Verification boundary

Contract prose remains authoritative in its original language. Localized pages
change presentation only. Object-repository acceptance is an external read-only
handoff and is not claimed until its team supplies a receipt.
