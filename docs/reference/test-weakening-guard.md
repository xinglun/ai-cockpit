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
lastVerifiedBy: WI-378-reference-documentation-batch-17
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

This is a Rust-native semantic counterpart to the reference Test Weakening
Guard. It does not ship the source Python module, Make target, or source JSON
wire format.
