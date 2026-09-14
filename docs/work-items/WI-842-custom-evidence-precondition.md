---
author: AI Cockpit maintainers
title: "WI-842 — custom evidence verification preconditions"
description: "Validate repository-bound custom evidence before any project verification process starts."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-842-custom-evidence-precondition
lastVerifiedBy: WI-842-custom-evidence-precondition
---

[简体中文](WI-842-custom-evidence-precondition.zh-CN.md) · [日本語](WI-842-custom-evidence-precondition.ja.md)

# WI-842 — custom evidence verification preconditions

## Intent and boundary

This Work Item makes the verification entry gate validate custom evidence with
the current repository, Contract, file type, and byte digest before spawning a
project process. A complete projection must not be blocked by a report-only
warning. Missing, stale, malformed, foreign, symlinked, and non-regular
evidence remains fail-closed. Runtime-wide reuse policy, historical evidence
rewriting, release publication, and resource cleanup are outside this boundary.

## Recovery boundary

An in-scope failure is amended and revalidated on the current Work Item. A
successor is used only for a genuinely different scope, authority, or base, an
independent change, an unsafe in-scope repair, immutable failed delivery, or
explicit human direction. A successor must retain predecessor bindings.

## Acceptance

- Valid repository-bound custom evidence permits verification preconditions,
  including when required high-risk scenarios are still unverified but planned.
- Invalid custom evidence is rejected before project process spawn with a
  stable diagnostic.
- Execution attempts retain their own exit status, timeout, duration, logs, and
  input identity even when formal completion evidence is rejected.
- Governance-only corrections may reuse unchanged execution inputs; execution
  input changes invalidate reuse.

## Verification

- `cargo test --locked -p cockpit-repository --test lifecycle_entry`
- `cargo test --locked -p cockpit-repository --test archive_integrity --test verification_attempts`
- `cargo fmt --all -- --check`
- `git diff --check`
