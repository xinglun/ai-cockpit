---
author: AI Cockpit maintainers
title: "WI-489 — 有界な終端ドキュメント昇格"
description: "クローズ済みドキュメント昇格 Work Item が自身のページだけを理由に無限の successor を作らないようにする。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-489-doc-promotion-terminal-boundary
status: implemented
authority: human-authorized
lastVerifiedBy: WI-489-doc-promotion-terminal-boundary
terminalArchive: .ai/work-items/archive/WI-489-doc-promotion-terminal-boundary.contract.json
terminalVerification: .ai/evidence/WI-489-doc-promotion-terminal-boundary.verification.json
terminalFinalization: .ai/decisions/WI-489-doc-promotion-terminal-boundary.finalize.json
terminalDecision: .ai/decisions/WI-489-doc-promotion-terminal-boundary.close.json
---

# WI-489 — 有界な終端ドキュメント昇格

この Work Item は終端ドキュメント投影を明示的かつ有界にします。通常の、
不正な、または混在した scope は fail-closed のままにし、ドキュメント昇格
Work Item 自身のページだけを理由に無限の successor を作成しません。

[English](WI-489-doc-promotion-terminal-boundary.md) · [简体中文](WI-489-doc-promotion-terminal-boundary.zh-CN.md)

## スコープ

- ドキュメント昇格 helper と三言語 status consistency checker に検証済みの
  self-terminal 境界を追加する。
- 通常の昇格、不正な scope、有界終端投影の回帰 fixture を追加する。
- 英語、中国語、日本語の workflow に境界を記載する。

## 受け入れ条件

- 境界は正確なドキュメント専用 scope から導出され、任意の drift や wildcard
  path を隠せない。
- 通常の Work Item は引き続き証拠に基づく終端昇格を要求する。
- 不変の governance 記録やグローバル Agent/MCP 設定を書き換えず決定的に動作する。

## 検証

- `bash tests/docs/promote_closed_work_item_test.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `python3 tests/conformance/reference_file_inventory.py --check`
