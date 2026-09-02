---
author: AI Cockpit maintainers
title: Test weakening signals
description: Snapshot-derived detection of reduced verification strength in the Rust Runtime.
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-512-reference-docs-batch-33
capabilityClaims:
  - test_weakening_detection
---

# Test weakening signals

[English](test-weakening-guard.md) · [简体中文](test-weakening-guard.zh-CN.md) · [日本語](test-weakening-guard.ja.md)

The Rust Runtime derives test and coverage weakening signals from the declared
base and the current repository snapshot during preflight and the Contract
quality gate. Agent prose is not evidence, and an empty signal list does not
prove complete semantic coverage.

## Signal boundary

The detector observes repository-relative tracked changes such as deleted
tests, added skip/disable markers, removed negative/security regressions,
non-blocking required checks, reduced coverage requirements, and explicit
success bypasses. Invalid revisions, traversal, non-regular files, escaping
symlinks, and unreadable/binary inputs are handled conservatively and remain
unknown or blocked rather than green.

`test_weakening` is a blocking governance signal. Coverage weakening is a
review/unknown signal unless the applicable Contract or policy makes it
blocking. The dynamic quality route chooses the amount of analysis appropriate
to the changed surface; strict/release routes may require the full check.

Every non-continue result carries a stable finding and recovery condition.
Restore verification strength or provide independently reviewable requirement
change evidence, then rerun against the same base. No environment variable,
local receipt, or human prose can bypass a critical signal. Provider-side
required checks and dynamic/generated test semantics remain external or
explicit limitations.

## Decision and compatibility boundary

The Runtime preserves the reference guard's decision meaning without copying
its Python module or Make surface:

- `continue` means that no configured static signal was observed; it is not a
  claim of sufficient tests.
- `warning` records a reviewer-visible, non-blocking signal such as a safe
  rename or small snapshot change.
- `review` requires an explanation and independently reviewable requirement
  evidence for material assertion, coverage, command-scope, negative-test, or
  required-check reductions.
- `block` stops explicit test/security/regression deletion, success bypasses,
  non-blocking required checks, or a deliberate coverage reduction.

An intentional retirement may be represented by a repository-local,
identity-bound review evidence record. Its base, paths, allowed signals,
human authorization, and digest must match the live finding. It can downgrade
only a review finding to a visible warning; it cannot clear a critical signal.
Legacy reports are read as historical input and require renewed analysis.
Unknown future versions, malformed policy, stale identity, or missing Git
evidence remain fail-closed. This is semantic compatibility, not JSON-wire or
Python API compatibility.

The detector is deliberately conservative but not omniscient: it can miss
helper-level or generated/data-driven semantic changes and provider-side
required-check changes. A fixture or local report therefore cannot establish
provider, adopter, production, legal, or enterprise assurance.

This is a Rust-native semantic counterpart to the reference Test Weakening
Guard. It does not ship the source Python module, Make target, or source JSON
wire format.
