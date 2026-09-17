---
author: AI Cockpit maintainers
workItemId: WI-872-outcome-host-delivery
title: "アーカイブ済み Outcome のホスト配信"
description: "アーカイブ済み Work Item の完全な Outcome 引き渡しとホスト配信境界。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-872-outcome-host-delivery
terminalArchive: .ai/work-items/archive/WI-872-outcome-host-delivery.contract.json
terminalVerification: .ai/evidence/WI-872-outcome-host-delivery.verification.json
terminalDecision: .ai/decisions/WI-872-outcome-host-delivery.close.json
---

# WI-872 — アーカイブ済み Outcome のホスト配信

Runtime が準備した完全な Outcome と、ホストの assistant メッセージイベント
との最後の境界を扱う Work Item です。ホスト送信 API がない場合、CLI/MCP は
完全な引き渡しだけを正直に提供します。

## 受入れ境界

- 通常と JSON のアーカイブ出力は同じ検証済み本文を使う。
- ホストアダプターは実際のメッセージ単位 receipt を返し、能力宣言だけで受理・表示を推定しない。
- 中断は ID に束縛された受理済み進捗からだけ再開する。
- return-only アダプターは `full_handoff_only` と未知のホスト状態を明示する。
- 制御可能なアダプターイベントで順序、分割、連続 WI、中断再送、ゼロ送信拒否を検証する。

公開はこの Work Item の範囲外で、全 Work Item と証拠が完了した最後に行います。
