---
author: AI Cockpit 维护者
title: "WI-347——Knowledge、输入信任、已安装生命周期与日语能力评估"
workItemId: WI-347-reference-knowledge-trust-lifecycle-assessment
description: "比较接下来的十个固定参考路径，发布有界的 Rust 原生三语映射。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - reference_parity
---

# WI-347——Knowledge、输入信任、已安装生命周期与日语能力评估

[English](WI-347-reference-knowledge-trust-lifecycle-assessment.md) · [日本語](WI-347-reference-knowledge-trust-lifecycle-assessment.ja.md)

## 意图与边界

本 Work Item 比较固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 的接下来十个路径，发布面向 adopter 的 Rust 原生映射：实现 Knowledge、输入来源、已安装 Runtime 生命周期、指令可追溯性、Human Report 语义和有界的日语能力评估。

目标保持一个共享外部 Runtime 和显式 `--repo` 仓库上下文。源 Python/Make/YAML 编排、生成评估字节、provider-global 配置和源 JSON wire 兼容均不在范围内。有意差异不表示源和目标拥有相同命令或字段。

## 逐文件决定

| 固定参考路径 | 分类 | 有界目标决定 |
| --- | --- | --- |
| `docs/reference/human-report-semantic-quality.md` | `implemented-different-by-design` | 将决策视图顺序和禁止过度声明边界映射到 human-benefit、task-outcome 和 Outcome 页面。 |
| `docs/reference/implementation-knowledge.ja.md` | `implemented-different-by-design` | 提供类型化只读 Knowledge 投影的日语阅读页。 |
| `docs/reference/implementation-knowledge.md` | `implemented-different-by-design` | 记录当前确定性的 CLI/MCP 过滤器，并明确日期/提交/supersession 维度尚未实现。 |
| `docs/reference/implementation-knowledge.zh-CN.md` | `implemented-different-by-design` | 提供相同过滤器和证据边界的中文页。 |
| `docs/reference/input-trust-dataflow.ja.md` | `implemented-different-by-design` | 将来源指引映射到类型化 Rust origin 和可追溯派生。 |
| `docs/reference/input-trust-dataflow.md` | `implemented-different-by-design` | 说明内容/工具输出分类、跨步骤保留和注入 fail-closed 处理。 |
| `docs/reference/input-trust-dataflow.zh-CN.md` | `implemented-different-by-design` | 提供中文页面并明确不负责身份认证。 |
| `docs/reference/installed-lifecycle.md` | `implemented-different-by-design` | 映射共享安装、显式 attach、不可变 Release 验收和独立的迁移/回滚所有权。 |
| `docs/reference/instruction-traceability.md` | `implemented-different-by-design` | 将 inventory、Work Item 证据与关闭链路映射到源的正向/反向追溯责任。 |
| `docs/reference/japanese-capability-assessment.json` | `implemented-different-by-design` | 映射到三语页面和可执行展示/对抗测试，不导入源字节，也不宣称一般日语流畅度。 |

十行均登记在机器清单和三语 comparison ledger 中。adopter 边界是验收的一部分：尽管 Runtime binary 共享，每个仓库的事实、Knowledge、证据、adapter 记录和决定仍保持本地隔离。

## 验收与验证

- 每个固定路径恰好出现一次，分类和原因如上；本批不保留 `deferred-next-batch` 或 `migrate-gap`。
- 五个新增参考页都具有英文、中文、日语链接，并说明语义/非 wire 边界。
- Knowledge 文档不宣传未支持的日期/提交/supersession 过滤器；输入信任文档不把内容当身份或授权；安装文档不混淆 Runtime 安装、repository attach 和迁移；日语文档明确不宣称一般流畅度。
- inventory、文档元数据/链接、治理完整性、comparison 和 parity 检查通过；不添加源 Python/Make/V1 文件或全局 Agent/MCP 配置。
- 使用显式仓库上下文执行已安装 Runtime 生命周期：checkpoint → verify → finish → archive → reviewed PR/merge → close，并输出可见的人类 Outcome，完成精确分支/worktree 清理。

固定参考提交：`e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
目标基线提交：`6ddd41d85b972a663fee85562592fc247749bf49`。
