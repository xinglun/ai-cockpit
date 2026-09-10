---
author: AI Cockpit maintainers
title: "WI-792 — v0.2.90 release"
description: "Publish the next patch release after integrated trust-diagnostics and governance-documentation acceptance."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized-release
workItemId: WI-792-release-v0-2-90
lastVerifiedBy: WI-792-release-v0-2-90
---

[简体中文](WI-792-release-v0-2-90.zh-CN.md) · [日本語](WI-792-release-v0-2-90.ja.md)

# WI-792 — v0.2.90 release

## Intent and boundary

Publish the next strategy-approved patch after the integrated A–E trust review.
The release must be produced from a reviewed PR and a synchronized default
branch, use a new annotated tag, and be accepted only from downloaded public
artifacts. This Work Item does not change Rust behavior or Runtime behavior,
rewrite historical evidence, reuse a tag, or weaken release gates.

## Release-note categories

- Communication fixes: the Outcome reason projection distinguishes the actual
  governance gap; finalization recovery actions retain concrete conditions; and
  CLI/MCP, summary/full, and tri-language collaboration checks validate human
  semantics.
- Validation coverage: the A–E counterexamples, no-history handoff, historical
  compatibility, controlled resource cleanup, and deterministic concurrent-change
  paths are bound to executable checks.
- Architecture consistency: Outcome inputs are assembled from one bounded,
  revalidated ObservationContext boundary while pure rendering remains I/O-free.
- Performance diagnostics: the main path exposes stage timing and controlled
  byte/hash/Git/process counters, including macOS/Linux filesystem paths. These
  are diagnostic measurements; no unverified speedup is claimed, and the
  previously rejected optimization remains rejected.

## Acceptance

- Cargo metadata, lockfile, current tri-language release/version projections,
  and the reference metadata agree on the exact package identity.
- The reviewed PR passes hosted quality, route, behavioral, Windows, and task
  completion checks before merge.
- The annotated tag and public Release manifest bind the merged main commit;
  all published artifacts, checksums, SBOMs, and attestations pass their gates.
- Public installation and N-1 upgrade acceptance use downloaded artifacts in
  isolated roots and prove cleanup; a workspace build is not a release
  substitute.
- The Work Item is archived, finalized, closed, and its exact branch/worktree
  cleanup plus synchronized main are verified.

## Verification

- `bash tests/release/version_consistency.sh --repo <repo>`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `cargo test --locked --workspace`
- `bash tests/release/version_consistency.sh --repo <repo> --post-release --repository xinglun/ai-cockpit --tag v0.2.90`
- `tests/release/adopter_acceptance.sh` and `tests/release/adopter_upgrade_acceptance.sh` against the public Release
