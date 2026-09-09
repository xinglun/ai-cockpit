---
author: AI Cockpit maintainers
title: "WI-716 — WI-715 governance scope recovery"
description: "Complete the scope-aware governance close for the merged WI-715 delivery without rewriting its immutable history."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-716-wi715-governance-scope-recovery
lastVerifiedBy: WI-716-wi715-governance-scope-recovery
---

[简体中文](WI-716-wi715-governance-scope-recovery.zh-CN.md) · [日本語](WI-716-wi715-governance-scope-recovery.ja.md)

# WI-716 — WI-715 governance scope recovery

## Intent

Complete the governance recovery required after WI-715 merged PR #707. The
Runtime correctly found that the parity and gate repair added after the
original Contract was archived touched governance test paths outside that
immutable Contract scope. This Work Item records the bounded successor close;
it does not redo the P0-B product implementation.

## Boundary

This Work Item covers the full Work Item identity handling in governance
parity/status checks and its three-language documentation projection. It does
not change Outcome behavior, validation rules, exit codes, machine JSON,
authorization semantics, or any WI-715 historical archive/evidence bytes.

## Base and recovery lineage

- Current remote/default base: `origin/main` at `d1141480fb7a045979098480c3770d06002e2a87`.
- Predecessor: WI-715, whose merged delivery, finalization receipt, and verification evidence remain immutable.
- Recovery decision: `.ai/decisions/WI-715-wi713-p0b-redelivery.recovery.json`.
- Predecessor finalization: `.ai/decisions/WI-715-wi713-p0b-redelivery.finalize.json`.
- PR #707: `https://github.com/xinglun/ai-cockpit/pull/707`.

## Acceptance

- The governance gate, documentation status consistency, promotion helper,
  and their regressions address complete Work Item IDs without collapsing
  unrelated records that share a numeric prefix.
- The three-language parity projection preserves WI-715 recovery facts and
  records this successor's evidence and terminal decisions without inventing
  a product or user-benefit claim.
- `cargo test --locked --workspace`, documentation acceptance, and the
  repository governance checks pass on the current default base.
- Provider finalization, exact cleanup, human close, and WI-715 historical
  close are each represented by generated Runtime evidence.

## Current state

The successor is active from the merged default base. Verification, reviewed
delivery, finalization, archive, and explicit human close remain pending.
