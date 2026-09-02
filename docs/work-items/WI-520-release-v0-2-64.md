---
author: AI Cockpit maintainers
title: "WI-520 — v0.2.64 release and object-adopter compatibility acceptance"
description: "Publish the merged historical-finalization compatibility fix and verify the public artifact without modifying object repositories."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-520-release-v0-2-64
lastVerifiedBy: WI-520-release-v0-2-64
terminalArchive: .ai/work-items/archive/WI-520-release-v0-2-64.contract.json
terminalVerification: .ai/evidence/WI-520-release-v0-2-64.verification.json
terminalFinalization: .ai/decisions/WI-520-release-v0-2-64.finalize.json
terminalDecision: .ai/decisions/WI-520-release-v0-2-64.close.json
---

[简体中文](WI-520-release-v0-2-64.zh-CN.md) · [日本語](WI-520-release-v0-2-64.ja.md)

## Goal

Publish v0.2.64 from a reviewed, synchronized default branch. The release
contains the WI-518 historical direct-merge apply path and its truthful
diagnostics. After publication, the downloaded public artifact must be the
only Runtime used for adopter acceptance; the object repository remains
read-only and is exercised by its own team.

## Scope

- Workspace package/lockfile version and current release documentation in all
  three languages.
- Release workflow, public adopter acceptance, N-1 acceptance, and their
  cleanup/isolation wrappers.
- This Work Item's tri-language documentation and parity registration.
- Immutable tag, public Release assets, checksums, SBOM, provenance, and
  downloaded-binary installation/health checks.

The object repository `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`
is explicitly read-only. Do not hand-edit its `.ai/` records or fabricate PR
identities. Global Agent/MCP configuration, source-build fallback, and
unrelated Runtime behavior are out of scope.

## Acceptance

- All workspace packages and Cargo.lock identify v0.2.64 without rewriting
  historical release facts.
- Hosted checks pass before an annotated v0.2.64 tag is created from synced
  `main`; a failed publication never reuses its tag.
- The public Release manifest, five archives, five SBOMs, SHA256SUMS, and
  provenance bind the same tag, bytes, targets, and digests.
- Public adopter and N-1 acceptance use downloaded immutable artifacts only,
  prove HOME/XDG isolation, classify TMPDIR/CARGO_HOME writes, prove cleanup,
  and preserve `first-adopter-smoke=not_ready`.
- The published binary is installed on this repository and `inspect`,
  `status`, `doctor`, Agent doctor, and documentation promotion checks pass.
- Human Outcome, archive, finalization, close, and exact branch/worktree
  cleanup are visible and recorded before release completion is declared.

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
python3 tests/ci/governance_integrity_gate.py --repo <repo>
```

Publication and post-release acceptance are separate facts. A public
acceptance failure records `releasePublished: true` and
`adopterAcceptance: failed`; it never rewrites Release truth. The final
adopter receipt and object-team instructions are supplied after the immutable
Release exists.
