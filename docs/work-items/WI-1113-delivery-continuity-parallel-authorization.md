---
author: AI Cockpit maintainers
title: "WI-1113-delivery-continuity-parallel-authorization — Scoped delivery continuity and parallel authorization"
description: "Carry forward the bounded Windows archive repair and address receipt migration, explicit one-time scope authorization, parallel Work Item coordination, bounded retry, and interruption recovery."
audience:
  - maintainer
  - reviewer
workItemId: WI-1113-delivery-continuity-parallel-authorization
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1113-delivery-continuity-parallel-authorization
---

# WI-1113 — Delivery continuity and parallel authorization

This Work Item is the native successor to WI-1112, which the fixed Runtime has retained as `replaced/not_verified` with its original evidence preserved. The fixed Runtime recorded WI-1113 staged activation at 2026-10-26T21:14:40Z. Its current effective Contract is `sha256:98fa4e72be9f32e399e1b7e5c05df89e8092e4461b3a5df0d96b8e36b5e9736d`, with `baseRevision` `b4adab2e55119d50bb72ec29f3fdfd2693d14504`; predecessor recovery HEAD `5a942f93acaf0a514a291e39674465feddfa69e7` is a separate identity. Exact native owner confirmation for the current Contract is pending. No implementation or formal verification has been recorded; all 25 required scenarios remain unverified.

## Scope and boundaries

- Preserve WI-1112's Windows local-Git path and exact `.ai` artifact byte protections, strict digest checks, and one-byte tamper rejection.
- Add a read-only delivery-continuity projection derived from validated recovery lineage. Distinguish predecessor local archive, current successor, external checks, and the Runtime-admitted next action. Archive or PR creation alone never means delivery completion.
- Add an append-only material-profile transition: verify an old receipt against its historic Contract, profile, request, manifest, and legal same-Work-Item amendment chain; retain the old receipt and pointer history; open a fresh request. The transition grants no approval and never turns Unknown into Clean. Unsupported, foreign, malformed, or newly changed materials remain blocked.
- Add pre-amendment profile-transition lint so an un-migratable receipt is found before writing the amendment.
- Add one-time, exact-scope authorization manifests for explicitly listed same-repository Work Items, independent child grants and status, atomic per-member start quotas, bounded unchanged-check retries, interruption recovery, and accurate partial results. No cross-Work-Item inheritance, implicit members, shared counters, arbitrary command retries, or deletion of failure history.
- Keep CLI/MCP behavior aligned and document the workflow in English, Japanese, and Simplified Chinese. Do not change CI workflows, weaken assertions, edit global Git settings, merge into `main`, release, enable production, replace the fixed Runtime, or close without a separate exact decision.

## Verification plan

The Contract declares 25 required scenarios and 19 verification commands: one serial all-targets test command for each of the 14 workspace packages, plus strict Clippy, route/resolver, format, and diff checks. They cover Windows path/byte integrity; valid and invalid successor lineage; archive-versus-delivery distinctions; exact material references, deltas, and append-only transition negatives; pre-amendment lint; explicit batch membership and independent grants; atomic quotas; bounded retries and uncertain outcomes; partial results; CLI/MCP parity; and scope/target/expiry enforcement. All scenarios remain `unverified` until the Runtime records real evidence. Hosted PR runs are limited to five total, including automatic triggers, entry failures, and reruns; local checks are limited to 900 seconds each, one worker, and two additional attempts for the same unchanged check.


## PR comparison and baselines

The WI-1113 PR continues from #1075's unchanged head to the WI-1111 branch, so its comparison includes inherited WI-1112 changes and new WI-1113 changes. It does not require #1075 to merge first; preserve #1074 and #1075 open and unchanged. Review the full cumulative diff and trace provenance for every inherited and new change; do not describe it as a WI-1113-only delta. Record predecessor recovery identity, the Contract `baseRevision` written by normal fixed-Runtime staged activation, and the provider comparison base separately.

## Current state

The typed plan records the full cumulative PR comparison and per-package verification plan. WI-1113 is now `implementation_active`, but its exact native owner confirmation is pending; no PR/resource context is bound yet. This page records the current native status and does not claim implementation, verification, hosted CI, or user-visible benefit.
