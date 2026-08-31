---
author: AI Cockpit maintainers
title: "WI-459 — v0.2.53 release and public binary acceptance"
workItemId: WI-459-release-v0-2-53
description: "Publish the next reviewed Rust Runtime patch and verify the public binary with the adopter baseline."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-459-release-v0-2-53
terminalArchive: .ai/work-items/archive/WI-459-release-v0-2-53.contract.json
terminalVerification: .ai/evidence/WI-459-release-v0-2-53.verification.json
terminalFinalization: .ai/decisions/WI-459-release-v0-2-53.finalize.json
terminalDecision: .ai/decisions/WI-459-release-v0-2-53.close.json
---

# WI-459 — v0.2.53 release and public binary acceptance

This Work Item packages the reviewed changes already on the default branch,
publishes them as v0.2.53, and verifies the immutable public binary through the
post-release adopter acceptance flow. It then returns the repository to the
reference-source parity queue.

[简体中文](WI-459-release-v0-2-53.zh-CN.md) · [日本語](WI-459-release-v0-2-53.ja.md)

## Scope

- Align the workspace package and lockfile identity to v0.2.53.
- Update the trilingual installation, release, and versioning projections while
  preserving prior release and failure history.
- Keep the existing annotated-tag, manifest, checksum, SBOM, provenance, and
  staged/public adopter gates as the release authority.
- After merge, validate the downloaded public v0.2.53 binary and retain its
  runtime, repository, isolation, cleanup, and lifecycle receipts.

## Out of scope

The reference inventory and parity ledgers owned by WI-445, the local reference
checkout, object repositories, global Agent/MCP configuration, Homebrew tap
mutation, source-build fallback, and unrelated Runtime behavior.

## Acceptance

- Workspace metadata, lockfile, and all three release-document routes identify
  v0.2.53 without rewriting historical release truth.
- The reviewed PR and hosted release workflow bind the annotated tag, source
  commit, manifest, Cargo.lock digest, archive/SBOM checksums, provenance, and
  public assets.
- Version, workflow, documentation, and workspace quality gates pass without a
  workspace binary or source fallback.
- The published v0.2.53 archive is downloaded and checksum-verified by the
  adopter harness; the receipt proves repository/runtime identity, isolation,
  evidence reuse, cleanup, and the `first-adopter-smoke` `not_ready` contract.
- After the release and acceptance, the default branch is synchronized, the
  Work Item is closed, and the repository is `ready_on_base`.

## Verification

- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/release/version_consistency_test.sh`
- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- strict repository gate manifest and documentation acceptance
- `cargo test --locked --workspace`
- post-release `tests/release/adopter_acceptance.sh` using only public v0.2.53

## Release boundary

The tag is pushed only after reviewed merge and default-branch synchronization.
The provider Release is created by the workflow after all source, artifact, and
staged-adopter gates pass. Public acceptance runs only after publication and
records failure without rewriting an already published Release.
