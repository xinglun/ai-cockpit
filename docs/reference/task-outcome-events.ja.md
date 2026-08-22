---
author: AI Cockpit maintainers
title: "Task Outcome events"
description: "Rust Task Outcome projection の append-only event 規則。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-139C
---

# Task Outcome events

Rust Runtime は生成した event を `.ai/work-items/active/<id>.events.jsonl` に保存します。
各行は repository と Work Item identity を持つ strict な `TaskOutcomeEvent` です。
`finish` は completion event を作り、warning、stop、resolution も記録します。

event stream は append-only です。修正は過去の event と関連付けた新しい event を追加し、
履歴行を削除・書き換えません。validator は malformed JSON、unknown field、foreign identity、
unsafe evidence path、secret-like detail、重複 ID、まだ現れていない event への参照を拒否します。

`archive` は event stream を byte-for-byte で移動し、archive manifest に `eventsDigest` を bind します。
`close` は final report の前に stream を再検証します。event は evidence source であり lifecycle authority
ではなく、scope、merge、release、provider identity、enterprise compliance を承認しません。

blocked lifecycle gate は failed gate と recovery condition を持つ red の active Outcome に投影されます。
後続の `work-item recover` receipt は `retry` または明示的な successor を許可できますが、blocked predecessor
を書き換えず、verification を自動で green にしません。receipt は predecessor の Contract/Summary/Outcome/event
digest と current Runtime に bind され、後続 decision は digest suffix path に append されます。

[Task Outcome report](../features/task-outcome-report.ja.md) | [Outcome reference](outcome-report.ja.md) | [English](task-outcome-events.md) | [中文](task-outcome-events.zh-CN.md)
