---
author: AI Cockpit maintainers
title: "WI-830 — release dispatch syntax repair"
description: "Repair the release close expression exposed before publication."
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
workItemId: WI-830-release-dispatch-syntax
lastVerifiedBy: WI-830-release-dispatch-syntax
---

[简体中文](WI-830-release-dispatch-syntax.zh-CN.md) · [日本語](WI-830-release-dispatch-syntax.ja.md)

# WI-830 — release dispatch syntax repair

## Intent and boundary

WI-830 repairs the extra closing parenthesis in the dispatch-only release
close condition that GitHub rejected before creating a run. It adds a local
regression for the exact malformed expression.

Runtime protocol, product behavior, release acceptance execution, immutable
tag or Release rewriting, historical migration, and user-global configuration
are outside this Work Item.

## Acceptance

- GitHub accepts `workflow_dispatch`.
- The local release policy rejects the malformed close expression and accepts
  the corrected balanced expression.
- No product artifact, tag, or Release is rewritten by this repair.
- Hosted checks pass before retrying publication.

## Verification

- `bash tests/ci/release_gate_policy_test.sh`
- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- `bash tests/release/action_runtime_policy.sh .github/workflows/release.yml`
- `cargo fmt --all --check`
- Hosted PR checks and one explicit publication dispatch.

