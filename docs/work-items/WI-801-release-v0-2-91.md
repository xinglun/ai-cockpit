---
author: AI Cockpit maintainers
title: "WI-801 — governed v0.2.91 release"
description: "Prepare and publish ai-cockpit v0.2.91 after the WI-799 root-cause repair."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization-root-cause-repair-and-release
workItemId: WI-801-release-v0-2-91
lastVerifiedBy: WI-801-release-v0-2-91
---

[简体中文](WI-801-release-v0-2-91.zh-CN.md) · [日本語](WI-801-release-v0-2-91.ja.md)

# WI-801 — governed v0.2.91 release

## Intent and boundary

WI-801 aligns the Cargo workspace and lockfile to v0.2.91, registers the
tri-language governance projections, and publishes the reviewed main revision
as an immutable release. The release workflow must use its candidate artifacts
and downloaded public artifacts for installation and N-1 upgrade acceptance.

This Work Item changes one Runtime lifecycle boundary: after verification, the
governance refresh remains bound to the Runtime identity that performed the
verification. It does not change other Runtime behavior, historical records,
existing releases or tags, or unrelated source features.

## Acceptance

- `Cargo.toml` and `Cargo.lock` consistently identify v0.2.91 with no unrelated
  dependency changes.
- The three WI-801 pages and tri-language parity rows are registered before
  verification and remain semantically aligned.
- Reviewed main is tagged and published as v0.2.91 only after hosted checks
  pass.
- Candidate fresh install and N-1 upgrade, then public fresh install, public
  N-1 upgrade, version consistency, and release close all bind immutable
  published artifacts and prove isolation cleanup.

## Verification

- `cargo test --locked --workspace`
- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/release/version_consistency_test.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test -p cockpit-repository --test lifecycle_order runtime_bound_verification_keeps_governance_bound_to_current_runtime -- --exact`
- Hosted release preflight, source quality, candidate acceptance, public
  artifact acceptance, N-1 upgrade acceptance, and release close.

## Evidence policy

The release tag, Release assets, workflow receipts, and public adopter
receipts are immutable external facts. Runtime-owned Contract, Summary,
Outcome, verification, archive, finalization, and close records are generated
by the installed Runtime. A moving branch, source checkout, or workspace
binary cannot substitute for the published artifact.
