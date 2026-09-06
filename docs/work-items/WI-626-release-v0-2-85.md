---
title: "WI-626 — v0.2.85 release and adopter acceptance"
description: "Publish the release after the failed v0.2.84 boundary and preserve an auditable replacement lineage."
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-626-release-v0-2-85
lastVerifiedBy: WI-626-release-v0-2-85
terminalArchive: .ai/work-items/archive/WI-626-release-v0-2-85.contract.json
terminalVerification: .ai/evidence/WI-626-release-v0-2-85.verification.json
terminalFinalization: .ai/decisions/WI-626-release-v0-2-85.finalize.json
terminalDecision: .ai/decisions/WI-626-release-v0-2-85.close.json
---

[简体中文](WI-626-release-v0-2-85.zh-CN.md) · [日本語](WI-626-release-v0-2-85.ja.md)

# WI-626 — v0.2.85 release and adopter acceptance

## Objective

Publish the reviewed Runtime as `v0.2.85` after the failed, immutable
`v0.2.84` publication boundary, then verify public installation and upgrade
using only immutable Release artifacts.

## Boundary

This Work Item covers version metadata, release/distribution documentation,
the failed-release recovery projection, and release acceptance interfaces. It
does not modify object repositories, copy the reference scaffold/Python/Make
implementation, or change global Agent/MCP configuration.

## Acceptance

1. Workspace packages and `Cargo.lock` resolve to `0.2.85`.
2. Release CI publishes an annotated `v0.2.85` tag with archives, checksums,
   SBOM/provenance, Formula, and matching Runtime identity.
3. The post-release adopter and N-1 harnesses use only immutable `v0.2.85`
   and `v0.2.83` artifacts, and prove isolation and run-root cleanup.
4. English, Chinese, and Japanese release/versioning/parity records identify
   the `v0.2.85` and `v0.2.83` boundary.
5. The terminal Outcome is human-visible and records status, unknowns,
   evidence, human decision, and next action.

## Verification

Run the workspace test and release/documentation policy checks before merge.
After publication, run the immutable adopter and N-1 acceptance harnesses and
record the downloaded Runtime identity; no source or workspace binary is a
release substitute.
