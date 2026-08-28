---
author: AI Cockpit maintainers
title: "WI-365 — release upgrade Git identity"
workItemId: WI-365-release-upgrade-git-identity
description: "Make public-to-staged N-1 acceptance commits deterministic in an isolated CI environment."
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-365-release-upgrade-git-identity
capabilityClaims: [release_distribution, adopter_acceptance]
---

# WI-365 — release upgrade Git identity

[简体中文](WI-365-release-upgrade-git-identity.zh-CN.md) · [日本語](WI-365-release-upgrade-git-identity.ja.md)

## Intent

Root-fix the release failure in which the N-1 adopter acceptance harness could
not create a commit on a clean CI runner because a cloned control repository
did not have a Git identity.

## Scope and boundary

- Configure a deterministic Git identity in each harness repository that the
  script commits to, using repository-local `.git/config` only.
- Add a regression for an isolated `HOME`/`XDG_CONFIG_HOME` with
  `GIT_CONFIG_GLOBAL=/dev/null`, including a commit after cloning.
- Preserve the existing immutable artifact, isolation, cleanup, and
  fail-closed acceptance boundaries.

Runtime semantics, hosted workflow policy, global Git/Agent configuration, and
unrelated release behavior are outside this Work Item.

## Acceptance

1. Every commit path in the upgrade harness has an explicit repository-local
   identity; no global Git configuration is written.
2. The regression proves both the initial and cloned-control commit paths work
   with global configuration disabled.
3. Success and failure paths retain acceptance truth, emit cleanup evidence,
   and remove only the validated temporary run root.
4. Release shell tests, documentation checks, and workspace quality checks pass.

## Verification boundary

The installed Runtime records the Contract, preflight, checkpoint, verification,
finish, archive, finalization, and close evidence. Public Release and N-1
acceptance remain immutable external release evidence; a failed post-release
acceptance never rewrites the publication fact.
