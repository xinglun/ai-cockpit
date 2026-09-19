---
author: AI Cockpit maintainers
workItemId: WI-925-current-performance
title: Current-version object-repository performance measurement
description: Reproduce current Runtime and development-cycle measurements without changing object repositories.
audience: [adopter, contributor, maintainer]
status: in_progress
authority: human:xinglun
lastVerifiedBy: WI-925-current-performance
---

[简体中文](WI-925-current-performance.zh-CN.md) · [日本語](WI-925-current-performance.ja.md)

# WI-925 — Current-version object-repository performance measurement

This Work Item produces current-version evidence, not an optimization claim. It
uses the installed Runtime and the same Rust/Cargo toolchain on read-only views
of ORG-X, sentinel, goods-garden, and ai-investigation-orchestrator. Their main
or default branches and pre-existing working-tree bytes must remain unchanged.

## Acceptance boundary

- Every release-grade operation has at least 100 valid warm samples. Fewer than
  100 remains diagnostic-only and must fail the release-grade comparator.
- Raw samples, p50/p95/p99, Runtime/binary/repository/toolchain/environment
  identities, execution/reuse counters, and explicit unavailable or invalidation
  reasons are retained.
- Runtime latency is reported separately from Contract→reviewable PR,
  verification→finish, and post-merge cleanup costs. Missing lifecycle data is
  `unknown`, never zero.
- A speedup is reported only when the paired comparison clears the frozen noise
  budget; otherwise the result remains `within_noise` or `unknown`.

## Verification boundary

Use `CARGO_INCREMENTAL=0` and the shared verification target directory. Run the
cheap format, scope, and projection checks before collection. Preserve raw
captures in the Work Item evidence and verify each object repository's branch,
HEAD, and working tree before and after collection. This Work Item does not
modify object repositories, optimize Runtime code, or publish a release.

The development-cycle report must distinguish measured intervals from metrics
that the Runtime cannot observe, including agent operation counts or provider
cleanup timestamps. The final human Outcome will state any remaining unknown
benefit explicitly.
