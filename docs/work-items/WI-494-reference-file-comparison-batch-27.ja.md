---
author: AI Cockpit maintainers
title: "WI-494 — capability、comprehension、deprecated-assets の reference rebaseline"
description: "変更された 7 つの local reference record を再読し、Rust-native boundary を明示します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-494-reference-file-comparison-batch-27
status: implemented
authority: canonical
lastVerifiedBy: WI-494-reference-file-comparison-batch-27
terminalArchive: .ai/work-items/archive/WI-494-reference-file-comparison-batch-27.contract.json
terminalVerification: .ai/evidence/WI-494-reference-file-comparison-batch-27.verification.json
terminalFinalization: .ai/decisions/WI-494-reference-file-comparison-batch-27.finalize.json
terminalDecision: .ai/decisions/WI-494-reference-file-comparison-batch-27.close.json
---

# WI-494 — capability、comprehension、deprecated-assets の reference rebaseline

## Goal

以前 `reference-only` と判断した後に source bytes が変更された 7 つの local reference path を一つずつ再読します。source study data、Python/Make implementation、source cleanup tooling は Rust repository に copy せず、path ごとの bounded decision を記録します。

## Scope と boundary

対象の 7 path は次のとおりです。

- `docs/reference/capability-truth-matrix.json`
- `docs/reference/comprehension-validation-responses/peter_01.en.json`
- `docs/reference/comprehension-validation-responses/tanaka_01.ja.json`
- `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json`
- `docs/reference/comprehension-validation-results.json`
- `docs/reference/comprehension-validation-results.md`
- `docs/reference/deprecated-assets-registry.json`

7 path はすべて `reference-only` のままです。capability matrix は source-owned claim/freshness projection、participant response と comprehension report は revision-bound study evidence、deprecated-assets registry は source-specific cleanup aid です。Rust は typed request-scoped capability view、reader-facing Outcome documentation、immutable Work Item history、reviewed resource finalization で適用可能な境界を保持します。source bytes を Runtime authority や adopter evidence にしません。

inventory application と regression test は prior classification と `sourceChangedSincePrevious` provenance を保持します。tri-language comparison/parity route にも同じ no-copy decision を記録します。

## Acceptance

- 7 path を一つずつ再読し、inventory に non-empty counterpart/reason とともに `reference-only` で記録します。
- participant、comprehension、source capability claim、source cleanup registry の bytes を Runtime/adopter state に copy しません。
- inventory validation、conformance regression、documentation acceptance、parity status check、repository の declared Runtime verification が pass します。
- reviewed PR lifecycle と exact cleanup で delivery し、global Agent/MCP config と object/adopter repository は変更しません。

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

reference checkout は `tests/conformance/reference-source.lock` で local/pinned です。source implementation や JSON wire compatibility は要求しません。
