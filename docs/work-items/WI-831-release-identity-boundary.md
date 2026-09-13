---
author: AI Cockpit maintainers
title: "WI-831 — release identity boundary"
description: "Separate immutable artifact identity from workflow dispatch identity."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-831-release-identity-boundary
lastVerifiedBy: WI-831-release-identity-boundary
---

[简体中文](WI-831-release-identity-boundary.zh-CN.md) · [日本語](WI-831-release-identity-boundary.ja.md)

# WI-831 — release identity boundary

## Intent

After the dispatch syntax repair in WI-830 was merged, the release workflow
still treated the immutable artifact source commit and the current workflow
dispatch commit as if they had to be equal. This Work Item preserves the
fail-closed boundary while allowing a reviewed orchestration commit to publish
an already-created immutable tag.

## Scope and decision

The dispatch preflight validates that the local annotated tag peels to the same
commit as the remote tag. Publication validates the manifest against that tag
commit. The dispatch workflow revision remains a separate execution identity.
Neither identity can rewrite the other, and v0.2.92 is not retagged.

## Verification

- Policy tests retain dispatch-only publication and balanced close-expression
  requirements.
- The pre-dispatch boundary rejects mutable or moved tags before compilation.
- The hosted workflow is reviewed from the current main branch before the
  existing immutable v0.2.92 tag is dispatched.

## Out of scope

Historical Work Items and receipts, full-workspace reruns, unrelated branch or
worktree cleanup, and rewriting or publishing a tag before this Contract is
verified.
