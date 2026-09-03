---
author: AI Cockpit maintainers
title: "WI-535 — MCP test fixture cleanup"
description: "Make the delegated-evidence integration fixture failure-safe and register its governed delivery."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-535-mcp-fixture-cleanup
lastVerifiedBy: WI-535-mcp-fixture-cleanup
terminalArchive: .ai/work-items/archive/WI-535-mcp-fixture-cleanup.contract.json
terminalVerification: .ai/evidence/WI-535-mcp-fixture-cleanup.verification.json
terminalFinalization: .ai/decisions/WI-535-mcp-fixture-cleanup.finalize.json
terminalDecision: .ai/decisions/WI-535-mcp-fixture-cleanup.close.json
---

[简体中文](WI-535-mcp-fixture-cleanup.zh-CN.md) · [日本語](WI-535-mcp-fixture-cleanup.ja.md)

## Goal

Ensure the delegated-evidence MCP integration test removes its temporary
repository on success, assertion failure, and panic, so later runs cannot
inherit stale Work Item state.

## Scope and boundary

- `crates/cockpit-mcp/tests/rpc.rs` and this Work Item's tri-language
  documentation/parity projections.
- Runtime lifecycle semantics, provider state, and object repositories are
  outside this Work Item.

## Acceptance

- The fixture uses an RAII temporary-directory owner and remains rerunnable.
- Cleanup failure cannot leave a duplicate Work Item for a later test.
- The Work Item is registered in all three parity ledgers before archive.

## Verification

```text
cargo test --locked -p cockpit-mcp --test rpc delegated_evidence_list_exposes_only_repository_bound_receipts
tests/docs/parity_status_check.sh
tests/ci/governance_integrity_gate.py --repo <repo>
```
