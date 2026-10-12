---
author: AI Cockpit maintainers
title: "Provider reconciliation boundary"
description: "Historical provider inventories are evidence context, not current provider truth."
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Provider reconciliation boundary

[简体中文](provider-reconciliation-boundary.zh-CN.md) · [日本語](provider-reconciliation-boundary.ja.md)

Reference files such as `open-pr-issue-reconciliation-662.*` and
`pre-release-documentation-alignment.json` are historical, source-repository
assessment artifacts. They record what a provider or reviewer observed at a
past revision; they do not prove the current state of this repository, a
current GitHub PR, a release, or an enterprise approval.

AI Cockpit keeps provider responsibilities explicit:

- the Runtime can require, bind, display, and archive delegated evidence;
- GitHub/hosted CI, reviewers, branch protection, release publication, and
  enterprise retention remain external systems;
- a stale or missing reconciliation is unknown and cannot authorize merge,
  release, or close;
- a new provider observation must be collected at the current repository and
  Work Item identity and carry its own digest/timestamp/source.

The source JSON/Markdown records are therefore `reference-only` in the
file-by-file ledger. They are not copied into `.ai/`, not merged into current
status, and never override a repository-local Contract or Runtime evidence.
Use the current [Release distribution](../release/distribution.md) and
[Reference parity](reference-parity.md) pages for target-specific boundaries.

## Explicit lifecycle layout evidence

Schema-v3 resource finalization uses `postMergeSync.lifecycleLayout` (`separate` or `combined`) and `postMergeSync.documentation` (`projected` or `absent`). Schema-v1 and schema-v2 retain their original rules. `combined` binds one real parent-to-commit finish/archive transition; it never invents an intermediate commit. The Runtime validates the canonical Summary, Outcome, reports, events, archive manifest and verification bindings together. Frozen reports may retain historical active paths. Existing verification and failed attempts keep their original bytes and Runtime identity; newly added attempts require execution bindings.

The source PR identity remains immutable. The sync PR binds its actual base, head and merge; its base must contain the source merge. Runtime proves the conflict-free Git merge result, rejecting conflicts and additional merge content instead of assuming the merge tree equals the sync head tree. Only this Work Item's lifecycle artifacts and required documentation may change on the sync branch.

`absent` is explicit evidence, not a documentation waiver: the source head, lifecycle parent, archive, sync base/head and merge must have no Work Item page, parity entry or active documentation reference, and the Contract must permit that absence. Historical evidence and report references remain valid. Provider identity, freshness and ambiguity checks, a surviving control checkout in the same Git common directory, Retained → Deleted, finalize-verify and a distinct real human close decision remain mandatory.
