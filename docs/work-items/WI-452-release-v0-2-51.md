---
author: AI Cockpit maintainers
title: "WI-452 — Release v0.2.51"
workItemId: WI-452-release-v0-2-51
description: "Publish and verify the v0.2.51 Runtime from immutable public artifacts."
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-452-release-v0-2-51
---

# WI-452 — Release v0.2.51

Publish v0.2.51 from a reviewed, synchronized `main`, then install and
accept the immutable public artifact. Failed historical tags remain immutable
and are never reused.

[简体中文](WI-452-release-v0-2-51.zh-CN.md) · [日本語](WI-452-release-v0-2-51.ja.md)

## Scope

- Bump the workspace package identity to v0.2.51.
- Synchronize current tri-language release, distribution, architecture, and
  versioning documentation.
- Run release policy, source archive, checksum/SBOM, and locked workspace
  gates before publication.
- After merge, publish the immutable tag, install only the downloaded artifact,
  and run isolated adopter acceptance.

## Boundary

No object repository, user-global Agent/MCP configuration, or failed release
tag is modified. Source checkout and workspace binaries are not valid release
acceptance inputs.

## Verification

- `cargo test --locked --workspace`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- `bash tests/release/source_archive_policy_test.sh`
- `bash tests/release/version_consistency_test.sh`
