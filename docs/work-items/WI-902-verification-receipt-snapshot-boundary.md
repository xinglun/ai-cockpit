---
author: AI Cockpit maintainers
workItemId: WI-902-verification-receipt-snapshot-boundary
title: Verification receipt snapshot boundary
description: Keep successful typed verification current across Runtime governance writes while invalidating it for real source changes.
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:xinglun
lastVerifiedBy: WI-902-verification-receipt-snapshot-boundary
---

# WI-902 — Verification receipt snapshot boundary

This Work Item addresses Issue #902. A successful current-Runtime typed
verification must remain usable at the immediately following preflight, gate,
and finish boundary when only Runtime governance records changed. Source-file
changes must still invalidate the same evidence. The fix preserves typed-first
verification, zero-process red preflight, receipt identity, and tamper safety.

## Acceptance and evidence

- A repository-bound fixture proves that governance-only writes do not make a
  valid verification receipt stale.
- A source mutation makes that receipt stale and blocks progression.
- CLI and MCP consume the same identity and snapshot decision.
- Existing malformed, foreign-runtime, tampered, and symlink receipt failures
  remain fail-closed.
