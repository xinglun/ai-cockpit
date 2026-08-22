---
author: AI Cockpit maintainers
title: "Task Outcome 事件"
description: "Rust Task Outcome 投影的追加式事件规则。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-139C
---

# Task Outcome 事件

Rust Runtime 将生成的事件写入 `.ai/work-items/active/<id>.events.jsonl`。
每行都是带 repository 和 Work Item identity 的严格 `TaskOutcomeEvent`。
`finish` 生成完成事件，也会记录 warning、stop 和 resolution。

事件流是 append-only。修正必须追加与旧事件关联的新事件，不能删除或重写历史行。
验证器会拒绝格式错误、未知字段、foreign identity、不安全 evidence path、secret-like
detail、重复 ID 以及引用尚未出现的事件。

`archive` 按字节移动事件流，并在 archive manifest 中绑定 `eventsDigest`；`close`
在写入最终报告前重新验证事件流。事件是 evidence source，不是 lifecycle authority，
不能批准 scope、merge、release、provider identity 或 enterprise compliance。

被阻断的 lifecycle gate 会投影为带 failed gate 和 recovery condition 的红色 active
Outcome。后续 `work-item recover` receipt 可以授权 `retry` 或显式绑定 successor，
但不会重写 blocked predecessor，也不会让 verification 自动变绿。receipt 绑定
predecessor Contract/Summary/Outcome/event digest 和当前 Runtime；后续 decision 使用
带 digest 后缀的路径追加保存。

[Task Outcome 报告](../features/task-outcome-report.zh-CN.md) | [Outcome 参考](outcome-report.zh-CN.md) | [English](task-outcome-events.md) | [日本語](task-outcome-events.ja.md)
