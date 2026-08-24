---
author: AI Cockpit maintainers
title: "WI-259——close decision 恢复与文档投影"
workItemId: WI-259-close-decision-recovery
description: "在不改写 immutable lifecycle records 的前提下恢复 predecessor 的文档投影。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-259-close-decision-recovery
authority: canonical
---

# WI-259——close decision 恢复与文档投影

## 意图

精确保留 WI-258，同时恢复其 close decision 无法通过的文档投影。本
successor 不重新解释或替换 predecessor 的实现、证据或人工决定。

## 范围

本变更只涉及三语 WI-258 恢复投影、三语 WI-259 记录、reference-parity 行和
Runtime 生成的 WI-258 recovery decision。不涉及生产 Runtime、发布物或
predecessor 的 `.ai` 字节。

## 验收

- WI-258 archive、evidence、finalization、close 字节保持完全不变。
- recovery decision 绑定精确 predecessor digest 与 successor ID。
- 三语 WI-258 文档和 parity 行标记为 Recovered，并链接本 successor。
- 只有在 WI-259 自身获得 approved structured close 与 terminal evidence 后，
  才能将其文档 promotion 为 Implemented。
- 文档、parity、governance-integrity 和 promotion 检查通过。

## 证据边界

Successor 是审计投影与恢复边界，不把 predecessor 的描述性决定等同于
`approved`；只有 WI-259 新的明确 close 才能授权自身的 terminal 文档 promotion。
