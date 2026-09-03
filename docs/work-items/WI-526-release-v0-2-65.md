---
author: AI Cockpit maintainers
title: "WI-526 — v0.2.65 release and object-adopter recovery acceptance"
description: "Publish the direct-merge recovery context fix and verify the immutable public artifact without modifying object repositories."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-526-release-v0-2-65
lastVerifiedBy: WI-526-release-v0-2-65
---

[简体中文](WI-526-release-v0-2-65.zh-CN.md) · [日本語](WI-526-release-v0-2-65.ja.md)

## Goal

Publish v0.2.65 from a reviewed, synchronized default branch. The release
contains the direct-merge recovery context compatibility fix and the terminal
documentation projection correction. The object repository remains read-only
and is exercised by its own team after publication.

## Scope

- Workspace package/lockfile version and current release documentation in all
  three languages.
- Release workflow and immutable public artifact evidence.
- This Work Item's tri-language documentation and parity registration.
- Promotion of the closed WI-527 terminal documentation from its immutable
  archive, verification, finalization, and close evidence.
- Downloaded-binary installation, health, isolation, and adopter acceptance.

The object repository `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`
is explicitly read-only. Do not hand-edit its `.ai/` records or fabricate PR
identities. Global Agent/MCP configuration and source-build fallback are out of
scope.

## Acceptance

- Packages and lockfile identify v0.2.65 while preserving historical release facts.
- Hosted checks pass before an annotated v0.2.65 tag is created from synchronized `main`.
- Public archives, SHA256SUMS, SBOM, provenance, and manifest bind the same tag and bytes.
- Downloaded-artifact adopter and N-1 acceptance prove isolation and temporary-root cleanup.
- The published binary is installed and all repository health/documentation checks pass.
- Visible Outcome, archive, finalization, close, and exact branch/worktree cleanup are recorded.

## Verification

```text
cargo metadata --locked --format-version 1
cargo test --locked --workspace
tests/release/version_consistency.sh --repo <repo>
tests/release/action_runtime_policy.sh .github/workflows/ci.yml .github/workflows/release.yml
tests/release/adopter_acceptance_test.sh
tests/release/adopter_upgrade_acceptance_test.sh
tests/docs/promote_closed_work_item.py --repo <repo> --check-all
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo> --report <report>
```

Publication and post-release acceptance are separate facts. A post-release
failure records `releasePublished: true` and `adopterAcceptance: failed`; it
never rewrites release truth.
