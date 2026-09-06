---
title: "WI-603 — v0.2.80 release and adopter acceptance"
description: "Publish the next Rust Runtime patch and verify its immutable artifact and adopter boundaries."
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-603-release-v0-2-80
lastVerifiedBy: WI-603-release-v0-2-80
terminalArchive: .ai/work-items/archive/WI-603-release-v0-2-80.contract.json
terminalVerification: .ai/evidence/WI-603-release-v0-2-80.verification.json
terminalFinalization: .ai/decisions/WI-603-release-v0-2-80.finalize.json
terminalDecision: .ai/decisions/WI-603-release-v0-2-80.close.json
---

[简体中文](WI-603-release-v0-2-80.zh-CN.md) · [日本語](WI-603-release-v0-2-80.ja.md)

# WI-603 — v0.2.80 release and adopter acceptance

## Objective

Publish the reviewed Rust Runtime as `v0.2.80`, then verify installation and
upgrade using only immutable public Release artifacts. This is a release
boundary after the reference-parity and documentation batch; it is not a
source-template migration.

## Boundary

The Work Item covers package version metadata, release/distribution,
architecture and versioning documentation, parity registration, and the
staged/public release acceptance evidence. It does not copy the reference
project's scaffold, Python/Make implementation, or JSON wire format. Object
and adopter repositories remain read-only external acceptance targets; global
Agent/MCP configuration and historical governance bytes are out of scope.

## Acceptance

1. Workspace package versions and `Cargo.lock` resolve to `0.2.80`.
2. The annotated tag, five target archives, SBOM/provenance, Formula,
   checksums, manifest, and Runtime identity are mutually bound by Release CI.
3. Staged and post-release adopter acceptance use downloaded artifacts only,
   prove repository isolation and temporary-run cleanup, and preserve
   published Release truth on failure.
4. English, Chinese, and Japanese release, architecture, versioning, and
   parity pages agree on the `v0.2.80` baseline and its N-1 `v0.2.79` boundary.
5. The public Release is verified after hosted checks; no reference checkout
   or object repository is modified.

## Verification

Run `cargo test --locked --workspace`, documentation and metadata checks, the
release policy/version consistency checks, and the immutable staged/public
adopter acceptance harnesses. Record the downloaded Runtime version and
SHA-256 in the release evidence. The terminal Outcome must remain a separate,
human-visible handoff with status, unknowns, evidence, decision, and next
action.

