---
author: AI Cockpit maintainers
title: "Task Outcome 事件"
description: "Rust Task Outcome 投影的追加式事件规则。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-457
---

# Task Outcome 事件

Rust Runtime 将生成的事件写入 `.ai/work-items/active/<id>.events.jsonl`。
每行都是带 repository 和 Work Item identity 的严格 `TaskOutcomeEvent`。
`finish` 生成完成事件，也会记录 warning、stop 和 resolution。

事件流是 append-only。修正必须追加与旧事件关联的新事件，不能删除或重写历史行。
验证器会拒绝格式错误、未知字段、foreign identity、不安全 evidence path、secret-like
detail、重复 ID 以及引用尚未出现的事件。

事件族采用显式词汇：`finding`、`risk`、`warning`、`confirmation`、`stop`、`resume`、
`resolution`、`risk-accepted`、`check-pass-after-fix`、`prevention`、`completed` 和
`cancelled`；同时保留历史兼容的 `blocked` 与 `recovered`。修正和取代必须使用
`event_corrected` 或 `event_superseded`，并通过 `correctionOf` 绑定已经出现的事件 ID；
没有绑定的修正会被拒绝。

`finding` 和 `risk` 必须带确定性的 `findingFingerprint`。Rust 依据事件族、空白归一化后的
detail 以及排序后的仓库相对 evidence 引用计算它。重复 fingerprint 会被拒绝，除非是明确
绑定的 correction/supersession；因此修复后的再次出现会成为新的可审计事件，而不是修改原事件。

`archive` 按字节移动事件流，并在 archive manifest 中绑定 `eventsDigest`；`close`
在写入最终报告前重新验证事件流。事件是 evidence source，不是 lifecycle authority，
不能批准 scope、merge、release、provider identity 或 enterprise compliance。

被阻断的 lifecycle gate 会投影为带 failed gate 和 recovery condition 的红色 active
Outcome。后续 `work-item recover` receipt 可以授权 `retry` 或显式绑定 successor，
但不会重写 blocked predecessor，也不会让 verification 自动变绿。receipt 绑定
predecessor Contract/Summary/Outcome/event digest 和当前 Runtime；后续 decision 使用
带 digest 后缀的路径追加保存。

Rust Runtime 在进程内完成等价的生成和校验；参考源 Python 脚本只是语义来源，不是 Runtime 依赖。
事件数量也不是性能分数。

这里是语义 parity，不是 source wire compatibility：Rust 保留 strict 的 `TaskOutcomeEvent`
结构和 repository binding，不复制模板的 Python schema 或 Make target。发布/provider evidence、
locale 投影以及 Status/PR 摘要仍是独立的 evidence 和 presentation 边界。

[Task Outcome 报告](../features/task-outcome-report.zh-CN.md) | [Outcome 参考](outcome-report.zh-CN.md) | [English](task-outcome-events.md) | [日本語](task-outcome-events.ja.md)
