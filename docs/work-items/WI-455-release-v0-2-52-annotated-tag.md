---
author: AI Cockpit maintainers
title: "WI-455 — annotated-tag release recovery for v0.2.52"
workItemId: WI-455-release-v0-2-52-annotated-tag
description: "Publish the next patch only through a reviewed annotated tag and immutable public artifacts."
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-455-release-v0-2-52-annotated-tag
---

# WI-455 — annotated-tag release recovery for v0.2.52

This Work Item prepares the next patch release after the immutable v0.2.51
lightweight-tag publication failure. It preserves that failed history, adds a
repeatable annotated-tag check, and keeps provider Release creation inside the
reviewed workflow. It does not operate an adopter repository.

[简体中文](WI-455-release-v0-2-52-annotated-tag.zh-CN.md) · [日本語](WI-455-release-v0-2-52-annotated-tag.ja.md)

## Sources

- `docs/release/distribution.*.md`
- `docs/architecture/release-distribution.*.md`
- `.github/workflows/release.yml`
- `tests/release/annotated_tag_identity.sh`
- failed v0.2.51 workflow run `33417057474`

## Acceptance

- Workspace metadata and trilingual release documentation identify v0.2.52 without rewriting v0.2.51 history.
- A lightweight tag is rejected; an annotated tag is peeled and bound to the reviewed commit.
- Maintainers are instructed to push an annotated tag and never pre-create a provider Release.
- Strict release gates, public artifact checksums/SBOM/provenance, and staged/public adopter acceptance pass without source fallback.
- The published binary is checksum-verified before installation and the repository remains healthy under the installed Runtime.

## Verification

- `tests/release/annotated_tag_identity.sh`
- `tests/release/version_consistency_test.sh`
- `tests/release/workflow_policy.sh .github/workflows/release.yml`
- strict `quality_route.py` + `run_repository_gates.py`
- `cargo test --locked --workspace`
