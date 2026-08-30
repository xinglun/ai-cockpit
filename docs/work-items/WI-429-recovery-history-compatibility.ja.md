---
author: AI Cockpit maintainers
title: "WI-429 — Historical recovery projection"
description: fail-closed 検証を弱めずに archived recovery 残留を解決します。
audience: [contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
workItemId: WI-429-recovery-history-compatibility
lastVerifiedBy: WI-429-recovery-history-compatibility
terminalArchive: .ai/work-items/archive/WI-429-recovery-history-compatibility.contract.json
terminalVerification: .ai/evidence/WI-429-recovery-history-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-429-recovery-history-compatibility.finalize.json
terminalDecision: .ai/decisions/WI-429-recovery-history-compatibility.close.json
---

# WI-429 — Historical recovery projection

## Intent と boundary

archived recovery chain に、対象の binding が未完了だった古い successor 試行と、
その後の有効な supersede receipt が共存する場合があります。Runtime は immutable
history を書き換えず、有効な終端 decision を投影します。

対象は、狭く分類した historical successor-binding 残留、記録時刻で勝つ新しい有効な
`supersede`、malformed/foreign/改ざん/新しいが無効な記録の fail-closed 維持、Rust 回帰
テストと三言語 workflow/parity 文書です。歴史 bytes、広範な graph 再設計、release/CI、
global Agent/MCP 設定は対象外です。

## Acceptance と evidence

最新の信頼できる recovery decision が有効な supersede なら predecessor は表示・close
可能でなければなりません。そうでなければ同じ残留は失敗として可視化します。Contract、
Summary、Outcome、Events、Evidence、recovery receipt の bytes は完全に保持します。

reviewed PR merge 後に `.ai/evidence/` と `.ai/decisions/` へ verification と終端 receipts を記録します。

[English](WI-429-recovery-history-compatibility.md) · [中文](WI-429-recovery-history-compatibility.zh-CN.md)
