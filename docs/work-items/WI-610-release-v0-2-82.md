---
title: "WI-610 — v0.2.82 release and adopter acceptance"
description: "Publish the next Rust Runtime patch and verify its immutable artifact and adopter boundaries."
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-610-release-v0-2-82
lastVerifiedBy: WI-610-release-v0-2-82
terminalArchive: .ai/work-items/archive/WI-610-release-v0-2-82.contract.json
terminalVerification: .ai/evidence/WI-610-release-v0-2-82.verification.json
terminalFinalization: .ai/decisions/WI-610-release-v0-2-82.finalize.json
terminalDecision: .ai/decisions/WI-610-release-v0-2-82.close.json
---

[简体中文](WI-610-release-v0-2-82.zh-CN.md) · [日本語](WI-610-release-v0-2-82.ja.md)

# WI-610 — v0.2.82 release and adopter acceptance

## Objective

Publish the reviewed Rust Runtime as `v0.2.82`, then verify installation and
upgrade using only immutable public Release artifacts. This release preserves
the shared Runtime/repository isolation model and the race-safe adopter cleanup
already verified in the preceding Work Item.

## Boundary

The Work Item covers package version metadata, release/distribution, tri-language
release and parity records, and staged/public release acceptance evidence. It
does not modify object repositories, copy the reference scaffold/Python/Make
implementation, or change global Agent/MCP configuration.

## Acceptance

1. Workspace package versions and `Cargo.lock` resolve to `0.2.82`.
2. Release CI publishes an annotated `v0.2.82` tag with target archives,
   SBOM/provenance, Formula, checksums, and matching Runtime identity.
3. Post-release adopter and N-1 acceptance use only immutable public artifacts
   and prove repository isolation and temporary-run cleanup.
4. English, Chinese, and Japanese release/parity records identify `v0.2.82`
   and the `v0.2.81` N-1 boundary.
5. The terminal Outcome is human-visible and records status, unknowns, evidence,
   human decision, and next action.

## Verification

Run `cargo test --locked --workspace`, documentation and metadata checks, release
policy/version consistency checks, and the immutable staged/public adopter and
N-1 acceptance harnesses. Record the downloaded Runtime version and SHA-256 in
the release evidence. The terminal Outcome remains a separate, human-visible
handoff.
