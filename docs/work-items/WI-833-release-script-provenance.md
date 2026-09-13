---
author: AI Cockpit maintainers
title: "WI-833 — release script provenance"
description: "Keep release orchestration code separate from immutable source identity."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-833-release-script-provenance
lastVerifiedBy: WI-833-release-script-provenance
---

[简体中文](WI-833-release-script-provenance.zh-CN.md) · [日本語](WI-833-release-script-provenance.ja.md)

# WI-833 — release script provenance

## Intent

Release acceptance must execute the reviewed orchestration from the workflow
commit. The immutable target Release tag is still the only source of artifact
and source identity; it must not also select the acceptance script.

## Decision

The four staged and public adopter jobs checkout `github.sha` at the workspace
root and checkout the requested `to_tag` into `release-source`. They invoke the
acceptance harness from the root and pass `release-source` as `--source-repo`.
This keeps the WI-832 command-reuse repair available during acceptance without
changing the immutable tag or its artifacts.

## Verification

The adopter acceptance regressions assert both checkout identities and reject a
job that uses the target tag as its harness checkout or omits the separate
source checkout. Formal Runtime verification records the targeted regression
result and its exit code.

## Out of scope

Runtime reuse semantics, product builds, immutable tags and Releases, historical
Work Items, and unrelated branch or worktree cleanup.
