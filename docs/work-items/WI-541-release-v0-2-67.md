---
author: AI Cockpit maintainers
title: "WI-541 — v0.2.67 release and public-artifact acceptance"
description: "Publish the reviewed Runtime release and verify its downloaded public artifact boundary."
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
workItemId: WI-541-release-v0-2-67
lastVerifiedBy: WI-541-release-v0-2-67
terminalArchive: .ai/work-items/archive/WI-541-release-v0-2-67.contract.json
terminalVerification: .ai/evidence/WI-541-release-v0-2-67.verification.json
terminalFinalization: .ai/decisions/WI-541-release-v0-2-67.finalize.json
terminalDecision: .ai/decisions/WI-541-release-v0-2-67.close.json
---

[简体中文](WI-541-release-v0-2-67.zh-CN.md) · [日本語](WI-541-release-v0-2-67.ja.md)

## Goal

Publish v0.2.67 from the reviewed, synchronized default branch, then verify
that the immutable public artifact can be installed and used for a fresh
adopter acceptance without source or workspace-binary fallback.

## Scope and boundary

- Workspace version and lockfile, current release/versioning architecture
  pages, distribution instructions, and tri-language parity registration.
- Hosted release checks and post-release artifact, checksum, SBOM, provenance,
  isolation, cleanup, and installed-runtime evidence.
- The object repository
  `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator` remains
  read-only; no object `.ai/` file, PR identity, or global Agent/MCP setting is
  changed.

## Acceptance

- Cargo metadata and lockfile identify v0.2.67 while historical release facts
  remain unchanged.
- The reviewed PR and hosted checks pass before an annotated v0.2.67 tag is
  created from synchronized `main`.
- Public archives, checksums, SBOM, provenance, and release manifest agree on
  the tagged commit and bytes.
- Downloaded public-artifact adopter and N-1 acceptance pass in isolated roots,
  retain `first-adopter-smoke=not_ready`, and prove temporary-root cleanup.
- The installed public v0.2.67 binary passes repository-bound health checks,
  and the human Outcome, archive, finalization, close, and exact branch cleanup
  are recorded.

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

Publication and post-release acceptance are separate facts. A failed
post-release acceptance records `releasePublished: true` and
`adopterAcceptance: failed`; it never rewrites published Release truth.
