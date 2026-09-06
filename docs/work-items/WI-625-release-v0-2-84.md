---
title: "WI-625 — v0.2.84 release and adopter acceptance"
description: "Publish the direct-merge recovery fix and validate immutable release artifacts with post-release adopter evidence."
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: recovered
authority: canonical
workItemId: WI-625-release-v0-2-84
lastVerifiedBy: WI-625-release-v0-2-84
---

[简体中文](WI-625-release-v0-2-84.zh-CN.md) · [日本語](WI-625-release-v0-2-84.ja.md)

# WI-625 — v0.2.84 release and adopter acceptance

## Objective

This historical release attempt is retained as a recovered, failed
publication boundary. The immutable `v0.2.84` tag was never a public release;
its source-quality failure was superseded by WI-626 for `v0.2.85`.

Publish the reviewed Runtime as `v0.2.84`, carrying the first-record
`direct_merge_no_pr` CLI round-trip fix, then validate the public artifacts
with the immutable adopter and N-1 acceptance harnesses.

## Boundary

This Work Item covers version metadata, release documentation, release policy
and post-release evidence. It does not modify object repositories, copy the
reference scaffold/Python/Make implementation, or change global Agent/MCP
configuration.

## Acceptance

1. Workspace packages and `Cargo.lock` resolve to `0.2.84`.
2. Release CI publishes an annotated `v0.2.84` tag with target archives,
   checksums, SBOM/provenance, Formula, and matching Runtime identity.
3. Public adopter and N-1 acceptance use only immutable `v0.2.84` and
   `v0.2.83` artifacts and prove repository isolation and run-root cleanup.
4. English, Chinese, and Japanese release/versioning/parity records identify
   the new release and its `v0.2.83` N-1 boundary.
5. The terminal Outcome is human-visible and records status, unknowns, evidence,
   human decision, and next action.

## Scenario coverage

- `v0.2.84 source and artifact identity`: bind package/version, annotated tag,
  manifest, archives, SBOM, checksums, and Runtime identity to one reviewed
  commit.
- `public adopter and N-1 upgrade`: use only immutable v0.2.84/v0.2.83
  artifacts and prove repository isolation and temporary-root cleanup.
- `direct_merge_no_pr recovery availability`: validate the published Runtime's
  plan-to-apply round trip without hand-editing generated evidence.

## Verification

Run workspace tests, documentation and release policy/version checks, and the
published-artifact adopter/N-1 harnesses. Record the downloaded Runtime
version and SHA-256 in release evidence; never use a source or workspace
binary as release evidence.
