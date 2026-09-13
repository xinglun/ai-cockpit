---
author: AI Cockpit maintainers
title: "WI-829 — explicit release trigger repair"
description: "Make release publication identity-bound and dispatch-only."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-829-release-trigger-repair
lastVerifiedBy: WI-829-release-trigger-repair
---

[简体中文](WI-829-release-trigger-repair.zh-CN.md) · [日本語](WI-829-release-trigger-repair.ja.md)

# WI-829 — explicit release trigger repair

## Intent and boundary

This Work Item makes publication start through an explicit `workflow_dispatch`
that carries the governing Work Item identity. The annotated tag is created
and pushed first as an immutable input; a tag push alone cannot start an
unresolved release route.

Runtime protocol changes, product behavior, historical Work Item migration,
existing Release or tag rewriting, and user-global configuration are outside
this Work Item.

## Acceptance

- Missing, malformed, or ambiguous release identity is rejected before
  compilation or publication.
- The dispatch validates the immutable tag, source commit, Work Item, and
  Contract before expensive release jobs run.
- Candidate and public installation, upgrade, handoff, attestation, and close
  barriers remain required.
- Policy and all three release documentation projections describe the same
  dispatch-only boundary.

## Verification

- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- `bash tests/release/action_runtime_policy.sh .github/workflows/release.yml`
- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh <repo>`
- `cargo fmt --all --check`
- Hosted CI on the reviewed PR and one explicit v0.2.92 publication dispatch.

## Recovery policy

The immutable tag and public Release are never rewritten. A failed dispatch is
retried only from the first invalid phase, reusing evidence whose input,
Contract, helper, and configuration identities still match.

