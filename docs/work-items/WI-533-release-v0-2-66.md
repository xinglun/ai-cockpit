---
author: AI Cockpit maintainers
title: "WI-533 — v0.2.66 release and direct-merge recovery acceptance"
description: "Publish the Runtime containing the bundled historical direct-merge compatibility fix and verify the public artifact boundary."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
workItemId: WI-533-release-v0-2-66
lastVerifiedBy: WI-533-release-v0-2-66
terminalArchive: .ai/work-items/archive/WI-533-release-v0-2-66.contract.json
terminalVerification: .ai/evidence/WI-533-release-v0-2-66.verification.json
terminalFinalization: .ai/decisions/WI-533-release-v0-2-66.finalize.json
terminalDecision: .ai/decisions/WI-533-release-v0-2-66.close.json
---

[简体中文](WI-533-release-v0-2-66.zh-CN.md) · [日本語](WI-533-release-v0-2-66.ja.md)

## Goal

Publish v0.2.66 from a reviewed, synchronized default branch. The release
contains the historical direct-merge recovery fix that separates the real
merge parent from the archived Contract base, so bundled merges can be
recorded without inventing a Pull Request or rewriting history.

## Scope and boundary

- Workspace version, lockfile, release workflow, release documentation, and
  tri-language parity registration.
- Immutable release archives, manifest, checksums, SBOM, provenance, and
  downloaded-artifact adopter/N-1 acceptance.
- Runtime installation and self-repository health checks after publication.

The object repository `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`
is read-only. Its `.ai/` records must not be edited and no PR identity may be
invented. Global Agent/MCP configuration and source-build fallback are out of
scope.

## Acceptance

- Package and lockfile identify v0.2.66 while historical release facts remain
  unchanged.
- Hosted checks pass before an annotated v0.2.66 tag is created from synced
  `main`.
- Public artifacts bind the same tag, commit, bytes, SHA256SUMS, SBOM, and
  provenance subjects.
- Public and N-1 acceptance use only immutable downloaded artifacts, prove
  repository isolation and temporary-root cleanup, and retain
  `first-adopter-smoke=not_ready`.
- The published binary is installed and inspect/status/doctor/Agent doctor and
  documentation checks pass.
- A visible human Outcome, archive, finalization, close, and exact branch and
  worktree cleanup are recorded before completion.

## Verification

```text
cargo metadata --locked --format-version 1
cargo test --locked --workspace
tests/release/version_consistency.sh --repo <repo>
tests/release/action_runtime_policy.sh .github/workflows/ci.yml .github/workflows/release.yml
tests/release/adopter_acceptance_test.sh
tests/release/adopter_upgrade_acceptance_test.sh
tests/docs/documentation_acceptance.sh
tests/docs/parity_status_check.sh
tests/docs/promote_closed_work_item.py --repo <repo> --check-all
```

Publication and post-release acceptance are independent facts. A failed
post-release acceptance records `releasePublished: true` and
`adopterAcceptance: failed`; it never rewrites an already-published Release.
