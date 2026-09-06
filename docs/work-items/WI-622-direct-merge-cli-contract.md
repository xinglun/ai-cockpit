---
author: AI Cockpit maintainers
title: "WI-622 — Direct-merge recovery plan/apply contract"
description: "Keep the first-record direct-merge recovery plan executable through both CLI apply entry points."
audience: [maintainer, reviewer, adopter]
workItemId: WI-622-direct-merge-cli-contract
status: implemented
authority: canonical
lastVerifiedBy: WI-622-direct-merge-cli-contract
terminalArchive: .ai/work-items/archive/WI-622-direct-merge-cli-contract.contract.json
terminalVerification: .ai/evidence/WI-622-direct-merge-cli-contract.verification.json
terminalFinalization: .ai/decisions/WI-622-direct-merge-cli-contract.finalize.json
terminalDecision: .ai/decisions/WI-622-direct-merge-cli-contract.close.json
---

# WI-622 — Direct-merge recovery plan/apply contract

## Intent

Make the Runtime's first-record direct-merge recovery plan an executable
contract. A complete `suggestedReceipt` plus only the listed human fields must
round-trip through `work-item finalize-recovery` and the ordinary
`work-item finalize` replay path without inventing a pull request or editing
generated repository records.

## Boundary

This Work Item covers the Rust protocol/repository/CLI regression contract and
the English, Chinese, and Japanese command/troubleshooting guidance. The
object repository, release/adopter harnesses, source-template implementation,
and global Agent/MCP configuration are out of scope. Existing strict
identity, merge-parent, repository, and fail-closed validation remains the
authority.

## Acceptance

- A real two-parent merge produces a complete `direct_merge_no_pr`
  `suggestedReceipt` with number `0`, historical provider, merge parents, and
  repository identity.
- Adding only `humanInputRequired` fields allows first write through
  `finalize-recovery` and an idempotent replay through `finalize`.
- `finalize-verify` and `close` remain successful only for the identity-bound
  receipt; malformed, foreign, or contradictory input remains rejected.
- CLI help and all three command/troubleshooting documents explain the same
  apply contract and the no-hand-edit/no-invented-PR boundary.

## Verification

```text
cargo test --locked -p cockpit-repository --test resource_finalization_transition
cargo test --locked -p cockpit-cli --test resource_finalization
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
bash tests/docs/documentation_acceptance.sh
```

See also: [中文](WI-622-direct-merge-cli-contract.zh-CN.md) ·
[日本語](WI-622-direct-merge-cli-contract.ja.md).
