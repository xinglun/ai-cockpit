---
title: "WI-616 — v0.2.83 release and adopter acceptance"
description: "Publish the direct-merge recovery fix and verify its immutable release artifacts with adopter and N-1 acceptance."
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-616-release-v0-2-83
lastVerifiedBy: WI-616-release-v0-2-83
terminalArchive: .ai/work-items/archive/WI-616-release-v0-2-83.contract.json
terminalVerification: .ai/evidence/WI-616-release-v0-2-83.verification.json
terminalFinalization: .ai/decisions/WI-616-release-v0-2-83.finalize.json
terminalDecision: .ai/decisions/WI-616-release-v0-2-83.close.json
---

[简体中文](WI-616-release-v0-2-83.zh-CN.md) · [日本語](WI-616-release-v0-2-83.ja.md)

# WI-616 — v0.2.83 release and adopter acceptance

## Objective

Publish the reviewed Rust Runtime as `v0.2.83`, then verify installation and
upgrade using only immutable public Release artifacts. This release carries
the WI-614 first-record `direct_merge_no_pr` recovery fix for adopter
repositories.

## Boundary

The Work Item covers package version metadata, release/distribution, tri-language
release and parity records, and staged/public release acceptance evidence. It
does not modify object repositories, copy the reference scaffold/Python/Make
implementation, or change global Agent/MCP configuration.

## Acceptance

1. Workspace package versions and `Cargo.lock` resolve to `0.2.83`.
2. Release CI publishes an annotated `v0.2.83` tag with target archives,
   SBOM/provenance, Formula, checksums, and matching Runtime identity.
3. Public adopter and N-1 acceptance use only immutable `v0.2.83` and
   `v0.2.82` artifacts and prove repository isolation and temporary-run cleanup.
4. English, Chinese, and Japanese release/versioning/parity records identify
   `v0.2.83` and its `v0.2.82` N-1 boundary.
5. The terminal Outcome is human-visible and records status, unknowns,
   evidence, human decision, and next action.

## Verification

Run workspace tests, documentation and metadata checks, release policy/version
consistency checks, and immutable staged/public adopter and N-1 harnesses.
Record the downloaded Runtime version and SHA-256 in release evidence.
