---
author: AI Cockpit maintainers
title: "WI-793 — v0.2.90 release recovery"
description: "Redeliver v0.2.90 from the latest default branch after immutable WI-792 retry evidence became non-consumable."
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorized-release-recovery
workItemId: WI-793-release-v0-2-90-recovery
lastVerifiedBy: WI-793-release-v0-2-90-recovery
---

[简体中文](WI-793-release-v0-2-90-recovery.zh-CN.md) · [日本語](WI-793-release-v0-2-90-recovery.ja.md)

# WI-793 — v0.2.90 release recovery

## Historical status

WI-793 is a preserved historical predecessor. Runtime recorded a `supersede`
decision in favor of WI-794 after the v0.2.90 Release and adopter evidence were
available. Its archive, verification, retry, and supersede records remain
immutable; current release closure is owned by WI-794.

## Recovery boundary

WI-793 is the strictly bound successor of WI-792. WI-792's retry receipts are
immutable evidence and remain on the predecessor branch; Runtime rejected their
consumption because their equal future-dated timestamps leave an older stale
candidate ordered at or after the current retry. This successor starts from the
latest synchronized `origin/main` and does not rewrite predecessor bytes.

## Intent and release boundary

Publish v0.2.90 after the integrated A–E trust review. The release must come
from a reviewed PR and synchronized default branch, use a new annotated tag,
and accept only downloaded public artifacts. This Work Item does not change
Rust or Runtime behavior, reuse a tag, rewrite governance history, or weaken
release gates.

## Release-note categories

- Communication fixes: Outcome reasons distinguish actual governance gaps;
  finalization recovery actions retain concrete conditions; CLI/MCP, summary/full,
  and tri-language checks validate human semantics.
- Validation coverage: A–E counterexamples, no-history handoff, historical
  compatibility, controlled cleanup, and deterministic concurrent-change paths
  remain executable evidence.
- Architecture consistency: Outcome input is assembled inside one bounded,
  revalidated ObservationContext boundary while pure rendering remains I/O-free.
- Performance diagnostics: main-path stage timing and controlled byte/hash/Git/
  process counters cover macOS/Linux filesystem paths. These are diagnostics;
  no unverified speedup is claimed and the rejected optimization remains rejected.

## Acceptance

- Cargo metadata, lockfile, current tri-language projections, and reference
  metadata agree on one exact v0.2.90 identity.
- The reviewed PR passes hosted route, quality, behavioral, Windows, and task
  completion checks before merge.
- The annotated tag and public Release manifest bind the merged main commit;
  artifacts, checksums, SBOMs, and attestations pass their gates.
- Public installation and N-1 upgrade use downloaded artifacts in isolated roots
  and prove cleanup; a workspace build is not release evidence.
- The successor is archived, finalized, closed, and its exact cleanup plus
  synchronized main are verified.

## Verification

- `bash tests/release/version_consistency.sh --repo <repo>`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `cargo test --locked --workspace`
- `bash tests/release/version_consistency.sh --repo <repo> --post-release --repository xinglun/ai-cockpit --tag v0.2.90`
- `tests/release/adopter_acceptance.sh` and `tests/release/adopter_upgrade_acceptance.sh` against the public Release
