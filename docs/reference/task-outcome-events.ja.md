---
author: AI Cockpit maintainers
title: "Task Outcome events"
description: "Rust Task Outcome projection の append-only event 規則。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-457
---

# Task Outcome events

Rust Runtime は生成した event を `.ai/work-items/active/<id>.events.jsonl` に保存します。
各行は repository と Work Item identity を持つ strict な `TaskOutcomeEvent` です。
`finish` は completion event を作り、warning、stop、resolution も記録します。

event stream は append-only です。修正は過去の event と関連付けた新しい event を追加し、
履歴行を削除・書き換えません。validator は malformed JSON、unknown field、foreign identity、
unsafe evidence path、secret-like detail、重複 ID、まだ現れていない event への参照を拒否します。

event family は `finding`、`risk`、`warning`、`confirmation`、`stop`、`resume`、
`resolution`、`risk-accepted`、`check-pass-after-fix`、`prevention`、`completed`、
`cancelled` を明示します。historical compatibility のため `blocked` と `recovered` も保持します。
correction/supersession は `event_corrected` または `event_superseded` とし、既出 event ID を
`correctionOf` で bind します。未 bind の correction は拒否されます。

`finding` と `risk` には deterministic な `findingFingerprint` が必要です。Rust は event family、
空白を正規化した detail、ソートした repository-relative evidence reference から計算します。
同じ fingerprint は拒否されますが、明示的に correction/supersession に bind された場合は許可されます。
修正後の再発は元の event を変更せず、新しい監査 event になります。

`archive` は event stream を byte-for-byte で移動し、archive manifest に `eventsDigest` を bind します。
`close` は final report の前に stream を再検証します。event は evidence source であり lifecycle authority
ではなく、scope、merge、release、provider identity、enterprise compliance を承認しません。

blocked lifecycle gate は failed gate と recovery condition を持つ red の active Outcome に投影されます。
後続の `work-item recover` receipt は `retry` または明示的な successor を許可できますが、blocked predecessor
を書き換えず、verification を自動で green にしません。receipt は predecessor の Contract/Summary/Outcome/event
digest と current Runtime に bind され、後続 decision は digest suffix path に append されます。

Rust Runtime は process 内で equivalent な generation/validation を行い、参照 Python script は semantic
source であって Runtime dependency ではありません。event 数は performance score ではありません。

これは semantic parity であり source wire compatibility ではありません。Rust は strict な
`TaskOutcomeEvent` と repository binding を維持し、template の Python schema や Make target をコピーしません。
publication/provider evidence、locale projection、Status/PR summary は独立した evidence と presentation の境界です。

[Task Outcome report](../features/task-outcome-report.ja.md) | [Outcome reference](outcome-report.ja.md) | [English](task-outcome-events.md) | [中文](task-outcome-events.zh-CN.md)
