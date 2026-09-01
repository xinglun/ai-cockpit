---
author: AI Cockpit maintainers
title: "WI-482 — lifecycle、parallel、trust-layer の reference 比較"
description: "変更された 8 つの local reference path を file 単位で再読し、Rust-native な parity decision を記録します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-482-reference-file-comparison-batch-26
status: current
authority: canonical
lastVerifiedBy: WI-482-reference-file-comparison-batch-26
---

# WI-482 — lifecycle、parallel、trust-layer の reference 比較

## Goal

前回の比較以後に local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` で変更された 8 file を一つずつ比較します。意味のある
governance semantic は保持しますが、reference の Python Runtime、Make workflow、provider config、source-only
document layout はコピーしません。

## Boundary

Rust baseline は `1f65a3b8bf09e54d4f9600fc5d64d8bbcb3ed62f`、Runtime は published `ai-cockpit 0.2.57`
（SHA256 `f03a13251a6fe57783528efbeae6ddd23bc2cc31dd2a1501d5421aac169a1d58`）に固定します。object/adopter
repository、Runtime feature work、global Agent/MCP config は対象外です。

## File decisions

8 path はすべて `implemented-different-by-design` です。

- 三つの `docs/operations/work-item-lifecycle.*` は `docs/reference/agent-workflow.*` と `outcome-report.*` が Rust-native lifecycle、human pause、exact cleanup を担います。
- `docs/reference/agent-parallel-work-items.md` は `cross-work-item-dedup.md`、`affected-verification.md`、`agent-workflow.md`、`AGENTS.md`、`.ai/README.md` が parallel boundary を担い、conversation handoff は adapter の責任です。
- `docs/reference/ai-cockpit-work-item-lifecycle.md` は Rust workflow、Outcome、CI gate document と Runtime が担います。template-only pytest shard と `REPORT_LANGUAGE` は target requirement ではありません。
- 三つの `docs/trust-layer.*` は `philosophy.*`、`security/enterprise-governance.*`、`architecture.*`、`capabilities.*` が trust-chain、delegated evidence、human decision、limitation を担います。

source の変更は reader route と source workflow の整理であり、Rust Runtime gap ではありません。Contract fact は authored language を保ち、localization は governance fact を変更したり human decision を作ったりしません。

## Acceptance and verification

- ledger は 8 path を正確に記録し、source-change provenance と prior classification を保持します。
- 三言語の比較 document と parity ledger が各 path と source implementation を copy しない boundary を明示します。
- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `cargo test --locked --workspace`
