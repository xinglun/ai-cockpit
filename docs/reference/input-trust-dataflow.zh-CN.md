---
author: AI Cockpit 维护者
title: 输入信任数据流
description: 对仓库内容、工具输出和生成解释进行来源感知处理。
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/input-trust-dataflow.md
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - provenance_aware_observation
---

# 输入信任数据流

[English](input-trust-dataflow.md) · [简体中文](input-trust-dataflow.zh-CN.md) · [日本語](input-trust-dataflow.ja.md)

AI Cockpit 将仓库内容和工具输出视为需要分类的输入，而不是权威。Markdown 中像命令的文字、Issue 中的角色声明或 Agent 生成的结论，都不会自动成为权限或独立证据。

## Rust 原生来源

Runtime 使用类型化的 `FactOrigin`、`TraceableFact` 和 `TraceableDerivation` 表示有限来源。常见来源为 `Observed`、`Declared`、`Derived`、`External` 和 `Unknown`。
快照事实、构建检测、测试输出和仓库文档都保持到仓库及操作的可追溯性；派生信号保留输入引用和规则。

这是语义对齐，不是源 JSON wire 兼容。目标不会复制参考 Python trust 模块，也不会伪造 provider 身份认证。

## 安全规则

- 用户直接指令和仓库策略可以在限定操作内作为 authority；仓库文档、Issue、PR、网页、fixture 和日志是内容或不可信观察。
- 工具输出是数据；Agent 对其解释不是新的独立验证结果。
- 跨步骤使用保留原始来源并追加派生关系，后续步骤不能抹去 earlier unknown 或不可信来源。
- 缺失来源、身份矛盾、不安全注入，或高风险边界上的 unknown/生成结论，应停止本地操作并显示安全替代方案或人工审查要求。

信任层不负责认证人员、验证 provider 或授权外部 merge/release；这些属于显式人工决定、provider 或 enterprise evidence 边界。

## 对象工程

对象工程通过 attach 的 Runtime 继承同样的 fail-closed 分类规则，但事实和证据保持仓库隔离。每次调用仍需显式 `--repo`，不存在全局 current project 或共享来源状态。
