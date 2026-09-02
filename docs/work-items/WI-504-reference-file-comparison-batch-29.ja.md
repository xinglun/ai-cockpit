---
author: AI Cockpit maintainers
title: "WI-504 — reference documentation batch 29"
description: "変更された 5 つの local reference document を再読し、証拠のある Rust reader route の欠落だけを補完します。"
audience:
  - maintainer
  - reviewer
workItemId: WI-504-reference-file-comparison-batch-29
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-504-reference-file-comparison-batch-29
---

# WI-504 — reference documentation batch 29

[English](WI-504-reference-file-comparison-batch-29.md) · [简体中文](WI-504-reference-file-comparison-batch-29.zh-CN.md)

## Goal

Pinned local reference checkout の変更された 5 path を一つずつ比較します。
portable な governance semantic は Rust-native reader route に保持し、具体的な
navigation omission が証拠で確認できる場合だけ補完します。source Python、Make、
provider command、source receipt、object/adopter repository state は copy/変更しません。

## Scope と file decision

Reference commit は `fde3380f81fea5fd2e288f7a8849f737dc074060` です。各 path に明示的な ledger decision を記録します。

| Reference path | Decision | Rust boundary |
| --- | --- | --- |
| `docs/reference/repository-workflow.ja.md` | implemented-different-by-design | Rust の Japanese workflow は削除された `REPORT_LANGUAGE` 引数を使用せず、localized Runtime presentation と explicit repository-scoped lifecycle/cleanup を保持します。 |
| `docs/reference/troubleshooting.md` | implemented-different-by-design | Rust tri-language troubleshooting route は一般的な stop/recovery と evidence preservation を保持し、provider handoff record は external boundary のままです。 |
| `docs/reference/verification-evidence-reuse.md` | implemented-different-by-design | source の no-change decision は Python/Make proposal に固有です。Rust の別途承認済み reuse は identity-bound で fail-closed です。 |
| `docs/reference/work-item-lifecycle-closure.md` | implemented-different-by-design | Rust-native closure、exact cleanup、recovery route が portable boundary を保持し、source hosted-governance/Make recovery detail は Runtime command ではありません。 |
| `docs/upgrade.md` | implemented-different-by-design | 最小の root compatibility pointer で canonical な Rust tri-language upgrade reference への reader route を復元します。 |

## Acceptance

- Pinned local commit で 5 path を再読し、non-deferred かつ evidence-backed な inventory record（counterpart/reason 付き）を作成します。
- root `docs/upgrade.md` が存在し、source implementation や claim を複製せず canonical upgrade reference にリンクします。
- tri-language comparison/parity documentation が同じ 5 decision を記録し、current count が一致して `migrate-gap` が 0 のままです。
- source implementation、provider config、global Agent/MCP setting、object/adopter repository は変更しません。
- conformance、documentation、Runtime verification、reviewed PR、merge、close、exact cleanup が pass します。

## Verification

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

Ledger は semantic/documentation parity であり、source command、JSON wire、provider state、release claim の互換性を意味しません。reference checkout は `AI_COCKPIT_REFERENCE_ROOT` から読み取り、本 Work Item では変更しません。
