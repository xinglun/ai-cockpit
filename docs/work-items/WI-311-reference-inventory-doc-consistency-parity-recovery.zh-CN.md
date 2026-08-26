---
author: AI Cockpit maintainers
title: "WI-311——参考 inventory 文档 parity 恢复"
workItemId: WI-311-reference-inventory-doc-consistency-parity-recovery
description: "从 manifest 派生台账计数，并在归档前登记三语 parity。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
---

# WI-311——参考 inventory 文档 parity 恢复

## 意图与边界

WI-310 已归档但遗漏了当前周期所需的三语 parity 登记。本 successor 从最新
`origin/main` 重新交付同一有界的 inventory 文档修正；前序字节保持不可变，parity
登记必须先于 verification evidence。这里是语义责任对齐，不是 source wire 兼容。

## 范围

- 将三语 reference-file comparison 台账同步到固定 manifest 计数（总计 5,119；
  generated-history 4,262；implemented-different-by-design 182；implemented-equivalent 1；
  not-applicable 3；reference-only 2；deferred-next-batch 669；migrate-gap 0）。
- 增加从 manifest 派生计数、校验三语机器标记的确定性回归。
- 在 verification evidence 生成前，将本 Work Item 登记到英/中/日 parity 台账。
- 保持三语 Work Item 文档同步。

## 不在范围内

Rust Runtime 行为、参考分类变更、源实现复制、release/adopter/CI workflow、全局
Agent/MCP 配置，以及改写 WI-310 或任何历史 evidence，均不在范围内。

## 验收与验证

文档、parity 行和 Work Item 文档必须通过 repository documentation 与
governance-integrity 检查；过期、错误、缺失或三语不一致的计数必须失败。使用安装版
Runtime，并为每条命令显式指定 `--repo`，完成审阅生命周期，最终 Outcome 以中文可见。
