---
author: AI Cockpit maintainers
title: "WI-600 — v0.2.79 release and adopter acceptance"
description: "Publish the post-WI-599 process-order release and validate its immutable public artifacts."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-600-release-v0-2-79
lastVerifiedBy: WI-600-release-v0-2-79
---

[简体中文](WI-600-release-v0-2-79.zh-CN.md) · [日本語](WI-600-release-v0-2-79.ja.md)

# WI-600 — v0.2.79 release and adopter acceptance

## Objective

Publish v0.2.79 from the reviewed, synchronized default branch after the
WI-599 documentation-gate ordering correction. Validate that the public
artifact can govern a fresh adopter without source or workspace fallback.

## Boundary

This Work Item changes package version metadata and current release/versioning
documentation. Runtime source behavior, object repositories, global
Agent/MCP configuration, historical evidence bytes, and reference-source
implementation are outside the boundary. Public adopter and N-1 acceptance
must use downloaded immutable artifacts only.

## Acceptance

1. Cargo metadata and lockfile resolve to v0.2.79; failed historical tags are
   retained and never reused.
2. Release policy and hosted checks bind the annotated tag, five target
   artifacts, checksums, SBOM/provenance, and Runtime identity to one reviewed
   commit.
3. Public adopter and N-1 harnesses use only v0.2.79 artifacts, prove
   forbidden-root isolation, and prove temporary-run cleanup on success and
   failure.
4. Current release, architecture, and versioning pages agree in English,
   Simplified Chinese, and Japanese; the object repositories remain untouched.
5. Any post-release failure preserves published truth and records a failure
   receipt without rewriting a tag or historical evidence.

## Verification

Run the Contract-declared locked workspace, documentation, parity, release
policy, staged acceptance, and post-release public-artifact checks. Complete
the lifecycle only after hosted checks pass, v0.2.79 is published, adopter/N-1
receipts are retained, and exact branch/worktree cleanup is verified.
