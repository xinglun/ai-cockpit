# WI-116: Archived close evidence after merge

## Objective

Allow the explicit `archive → close` lifecycle to finish after a reviewed branch
is merged. Closing an archived Work Item must validate the immutable verification
evidence, archive manifest, outcome binding, repository identity, and Runtime
identity without treating a changed current Git snapshot as stale by itself.

This is a successor to WI-115 because WI-115 is already merged and its archive
bytes are immutable; the repair cannot safely be amended into that Work Item.

## Scope

- Use the archived evidence path for the close governance gate.
- Keep active/finish/archive gates bound to the current snapshot.
- Preserve fail-closed behavior for tampered evidence, archive manifests,
  repository identity, Work Item identity, and foreign Runtime identity.
- Add a regression for a post-archive merge commit followed by structured close.
- Document the immutable archive-manifest boundary in all supported languages.

## Acceptance

- A valid archived Work Item closes after a post-archive commit.
- Tampered or identity-mismatched evidence remains rejected.
- Existing lifecycle and archive-integrity tests remain green.
- The Runtime emits a structured human decision; no archive bytes are rewritten.

## Status

In progress until Runtime verification, archive, and close complete.
