---
author: AI Cockpit maintainers
title: "WI-457 — Task Outcome イベント意味論の整合"
workItemId: WI-457-task-outcome-event-parity
description: "Rust native の append-only Task Outcome event projection と finding/risk fingerprint 検証を追加する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-457-task-outcome-event-parity
terminalArchive: .ai/work-items/archive/WI-457-task-outcome-event-parity.contract.json
terminalVerification: .ai/evidence/WI-457-task-outcome-event-parity.verification.json
terminalFinalization: .ai/decisions/WI-457-task-outcome-event-parity.finalize.d2e8f8795a6a88fc3fcd8bf2633813d2e20d0443e4c48397b5bab254b0ba8a70.json
terminalDecision: .ai/decisions/WI-457-task-outcome-event-parity.close.json
---

# WI-457 — Task Outcome イベント意味論の整合

WI-457 は repository-bound な Rust Task Outcome event projection を追加します。
event stream は append-only のまま維持し、identity と evidence reference を検証します。
finding/risk event には deterministic fingerprint を記録し、同じ finding を暗黙に
新しい進捗として数えません。これは local reference source との semantic parity であり、
Python wire compatibility ではありません。

[English](WI-457-task-outcome-event-parity.md) · [简体中文](WI-457-task-outcome-event-parity.zh-CN.md)

## Delivered boundary

- reference event family、correction/supersession の順序、repository/Work Item identity、
  safe evidence path、unknown field を strict に検証します。
- finding/risk event の `findingFingerprint` を deterministic に生成し、明示的な
  correction/supersession 以外の重複を拒否します。
- Outcome report section から append-only event を生成しますが、authority、approval、
  release、provider assurance、user benefit を発明しません。
- archive では event bytes を保持し、close で stream を再検証します。
- 三言語 documentation で semantic、privacy、localization、non-wire の境界を説明します。

## Verification evidence

Terminal verification は `.ai/evidence/WI-457-task-outcome-event-parity.verification.json` に
記録され、archive/close record は同じ repository、Contract、Runtime identity に bind します。
Finalization history は reviewed merge observation と feature branch/worktree の正確な cleanup を
記録し、immutable な Runtime record は書き換えません。

## Related documentation

- [Task Outcome events](../reference/task-outcome-events.ja.md)
- [Task Outcome report](../features/task-outcome-report.ja.md)
- [Reference parity](../reference/reference-parity.ja.md)
