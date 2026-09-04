---
author: AI Cockpit maintainers
title: "WI-552 — installation と upgrade reference 比較 batch 40"
description: "Pinned installer/upgrade 17 path を逐次比較し、Runtime capability discovery を厳密化する。"
audience: [maintainer, reviewer]
status: current
authority: canonical
workItemId: WI-552-reference-file-comparison-batch-40-install-upgrade
lastVerifiedBy: WI-552-reference-file-comparison-batch-40-install-upgrade
---

# WI-552 — installation と upgrade reference 比較 batch 40

## Goal

Pinned reference の installer/upgrade path を一つずつ比較し、Python 実装、source JSON wire、provider registry、repository-local installer state を copy せずに、portable な governance responsibility を shared Rust Runtime に保持します。

## Scope と result

`tests/conformance/reference_file_inventory.json` に記録された 17 path（install facts、plan/status/wizard、repository detection/evidence/ownership/transaction、version、upgrade apply/conflict/proposal、Python launcher）を対象にしました。全 path は `implemented-different-by-design` または `reference-only` として明示され、`migrate-gap` はありません。

Runtime は `.ai/agent-interface.json` 用に Protocol-owned capability registry を持ちます。`attach` は discovery のため complete command surface を公開しますが、readiness、authorization、evidence、lifecycle gate は request-scoped/repository-bound のままです。Agent は manifest の後に CLI/MCP schema を確認します。capability の存在は permission ではありません。

## Non-claims

Runtime install は external shared operation です。`attach` は minimum scaffold だけを作ります。source installer catalog、Python launcher、provider policy、global Agent/MCP configuration、source wire JSON は object project に継承しません。

## Verification

- Rust attach regression が complete protocol capability registry と idempotent manifest bytes を確認します。
- Inventory と shell conformance が 17 source path を検査し、この batch の deferred/migrate-gap を拒否します。
- Tri-language capability/configuration/reference/parity docs が capability discovery と `--help`/MCP `tools/list` lookup を説明します。
