---
author: AI Cockpit maintainers
title: WI-642 - Reference rebaseline batch 56
description: Complete the pinned reference file-by-file comparison without copying source implementation.
audience: [maintainer, reviewer, adopter]
workItemId: WI-642-reference-rebaseline-batch-56
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-642-reference-rebaseline-batch-56
terminalArchive: .ai/work-items/archive/WI-642-reference-rebaseline-batch-56.contract.json
terminalVerification: .ai/evidence/WI-642-reference-rebaseline-batch-56.verification.json
terminalFinalization: .ai/decisions/WI-642-reference-rebaseline-batch-56.finalize.json
terminalDecision: .ai/decisions/WI-642-reference-rebaseline-batch-56.close.json
---

# WI-642 - Reference rebaseline batch 56

## Intent

Re-read the final bounded set of paths from the pinned local reference one at a time, record an evidence-backed Rust counterpart or an explicit reference-only boundary, and resolve the last deferred comparison records. The reference remains a specification and behavior corpus, not a directory to copy.

## Scope and boundaries

- Reference commit: `a9224aed77b5c317b53c4551a9eec306d91ee330`.
- Exactly 54 paths below, plus the machine inventory, metadata, tri-language comparison/parity pages, and this Work Item's three language pages.
- No source Python, Shell, Make, provider policy, generated history, or JSON wire bytes are copied. Object/adopter repositories, release publication, and global Agent/MCP configuration are out of scope.
- Every attached repository inherits the shared external Runtime, explicit `--repo`, isolated Contract/evidence/knowledge, dynamic verification, fail-closed lifecycle, and visible human Outcome.

## Ordered path set

- `.ai/knowledge/work-items/fix-lockfile-noindex-20260903.json`
- `.ai/knowledge/work-items/fix-release-python-pin-20260903.json`
- `.ai/knowledge/work-items/release-post-publish-projection-v0-5-71-20260826.json`
- `.ai/knowledge/work-items/repository-release-publish-v0-5-71-20260826.json`
- `.ai/knowledge/work-items/wi-ci-performance-governance-20260826.json`
- `.ai/work-items/archive/2026/dependabot-lockfile-sync-20260826.archive-manifest.json`
- `.ai/work-items/archive/2026/dependabot-lockfile-sync-20260826.contract.json`
- `.ai/work-items/archive/2026/dependabot-lockfile-sync-20260826.outcome.json`
- `.ai/work-items/archive/2026/dependabot-lockfile-sync-20260826.outcome.md`
- `.ai/work-items/archive/2026/dependabot-lockfile-sync-20260826.summary.json`
- `.ai/work-items/archive/2026/dependabot-ruff-0165-sync-20260902.archive-manifest.json`
- `.ai/work-items/archive/2026/dependabot-ruff-0165-sync-20260902.contract.json`
- `.ai/work-items/archive/2026/dependabot-ruff-0165-sync-20260902.outcome.json`
- `.ai/work-items/archive/2026/dependabot-ruff-0165-sync-20260902.outcome.md`
- `.ai/work-items/archive/2026/dependabot-ruff-0165-sync-20260902.summary.json`
- `.ai/work-items/archive/2026/fix-lockfile-noindex-20260903.archive-manifest.json`
- `.ai/work-items/archive/2026/fix-lockfile-noindex-20260903.contract.json`
- `.ai/work-items/archive/2026/fix-lockfile-noindex-20260903.outcome.json`
- `.ai/work-items/archive/2026/fix-lockfile-noindex-20260903.outcome.md`
- `.ai/work-items/archive/2026/fix-lockfile-noindex-20260903.summary.json`
- `.ai/work-items/archive/2026/fix-release-python-pin-20260903.archive-manifest.json`
- `.ai/work-items/archive/2026/fix-release-python-pin-20260903.contract.json`
- `.ai/work-items/archive/2026/fix-release-python-pin-20260903.outcome.json`
- `.ai/work-items/archive/2026/fix-release-python-pin-20260903.outcome.md`
- `.ai/work-items/archive/2026/fix-release-python-pin-20260903.summary.json`
- `.ai/work-items/archive/2026/release-post-publish-projection-v0-5-71-20260826.archive-manifest.json`
- `.ai/work-items/archive/2026/release-post-publish-projection-v0-5-71-20260826.contract.json`
- `.ai/work-items/archive/2026/release-post-publish-projection-v0-5-71-20260826.outcome.json`
- `.ai/work-items/archive/2026/release-post-publish-projection-v0-5-71-20260826.outcome.md`
- `.ai/work-items/archive/2026/release-post-publish-projection-v0-5-71-20260826.summary.json`
- `.ai/work-items/archive/2026/repository-release-publish-v0-5-71-20260826.archive-manifest.json`
- `.ai/work-items/archive/2026/repository-release-publish-v0-5-71-20260826.contract.json`
- `.ai/work-items/archive/2026/repository-release-publish-v0-5-71-20260826.outcome.json`
- `.ai/work-items/archive/2026/repository-release-publish-v0-5-71-20260826.outcome.md`
- `.ai/work-items/archive/2026/repository-release-publish-v0-5-71-20260826.summary.json`
- `.ai/work-items/archive/2026/wi-ci-performance-governance-20260826.archive-manifest.json`
- `.ai/work-items/archive/2026/wi-ci-performance-governance-20260826.contract.json`
- `.ai/work-items/archive/2026/wi-ci-performance-governance-20260826.outcome.json`
- `.ai/work-items/archive/2026/wi-ci-performance-governance-20260826.outcome.md`
- `.ai/work-items/archive/2026/wi-ci-performance-governance-20260826.summary.json`
- `.ai/work-items/external-handoffs/run-release-rehearsal-v0-5-71-20260826.json`
- `.ai/work-items/recovery-receipts/dependabot-lockfile-sync-20260826-2.json`
- `.ai/work-items/recovery-receipts/dependabot-lockfile-sync-20260826.json`
- `.ai/work-items/starts/dependabot-lockfile-sync-20260826.json`
- `.ai/work-items/starts/dependabot-ruff-0165-sync-20260902.json`
- `.ai/work-items/starts/fix-lockfile-noindex-20260903.json`
- `.ai/work-items/starts/fix-release-python-pin-20260903.json`
- `.ai/work-items/starts/release-post-publish-projection-v0-5-71-20260826.json`
- `.ai/work-items/starts/repository-release-publish-v0-5-71-20260826.json`
- `.ai/work-items/starts/wi-ci-performance-governance-20260826.json`
- `docs/reference/ci-performance-baseline.json`
- `docs/superpowers/plans/2026-08-26-wi-ci-performance-governance.md`
- `docs/superpowers/specs/2026-08-26-wi-ci-performance-governance-design.md`
- `tests/test_ci_performance_baseline.py`

## Decision

Five source-generated knowledge records are `reference-only`; the other 49 archive, start, recovery, handoff, and performance paths are `implemented-different-by-design`. No path remains `deferred-next-batch` or `migrate-gap`. The exact counterpart and bounded reason for every row are in [`reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json). Source/provider history is retained as comparison evidence, not imported as current authority.

## Verification

Run the inventory, metadata, documentation, parity, status-consistency, diff, and repository quality checks declared by the Contract. Review the visible human Outcome before finish/archive/close. Release publication is intentionally deferred until this batch and its required documentation promotion/cleanup are closed; the six semantic batches are then released together.

See also: [中文](WI-642-reference-rebaseline-batch-56.zh-CN.md) · [日本語](WI-642-reference-rebaseline-batch-56.ja.md).
