---
author: AI Cockpit 维护者
title: "WI-411——Java 多模块 fixture 边界"
workItemId: WI-411-reference-java-fixture-boundary
description: "逐一比较固定的 Java fixture 文件，记录仅参考边界，不复制源 fixture。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-411-reference-java-fixture-boundary
terminalArchive: .ai/work-items/archive/WI-411-reference-java-fixture-boundary.contract.json
terminalVerification: .ai/evidence/WI-411-reference-java-fixture-boundary.verification.json
terminalFinalization: .ai/decisions/WI-411-reference-java-fixture-boundary.finalize.0f666cb7a60ec506e3e8abefb6bfec0c973bd690abc1be18aa2394cb2cf1e194.json
terminalDecision: .ai/decisions/WI-411-reference-java-fixture-boundary.close.json
canonical: docs/work-items/WI-411-reference-java-fixture-boundary.md
---

# WI-411——Java 多模块 fixture 边界

## 意图与边界

在参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 下，逐一读取
`examples/fixtures/java-multimodule/` 的九个文件。这些文件是参考工程的
可执行 Java/Maven 样例，不是 Rust Runtime 代码、可移植治理策略或企业证据。

| 固定参考路径 | 分类 | 目标边界决定 |
| --- | --- | --- |
| `.gitignore` | `reference-only` | 仅负责 fixture 构建清理；目标 release harness 自己管理隔离临时目录。 |
| `app/src/main/java/fixture/app/Main.java` | `reference-only` | Java 应用样例；通用 argv 执行不代表 Runtime 提供 Java 专项支持。 |
| `app/src/test/java/fixture/app/MainTest.java` | `reference-only` | fixture 断言；adopter verification 记录声明的命令，不复制该测试。 |
| `core/src/main/java/fixture/core/Decision.java` | `reference-only` | 业务域样例策略；目标仓库策略保持显式类型化。 |
| `core/src/test/java/fixture/core/DecisionTest.java` | `reference-only` | 仅验证样例，不是 Runtime 或企业证据。 |
| `evidence.json` | `reference-only` | 源工程本地证据（含不可用能力）；不提升为目标发布证据。 |
| `fixture.json` | `reference-only` | 源 stack/module 元数据；目标不从中推断 adopter capability。 |
| `pom.xml` | `reference-only` | Maven 构建输入；Java/Maven 执行属于 adopter 或 delegated provider。 |
| `scripts/lifecycle.sh` | `reference-only` | 源 fixture 编排；目标生命周期由已安装 Rust Runtime 提供。 |

目标不增加 Java 源码、Maven manifest 或源 shell 编排。第二技术栈 adopter
验收仍需单独且明确授权的 Work Item；本批不宣称该能力。

## 验收

- 九个固定路径均已读取，并在机器台账中恰好出现一次。
- 九个路径全部为 `reference-only`，均有非空原因和目标边界；本批不留
  `deferred-next-batch` 或 `migrate-gap`。
- 英文、简体中文、日文 comparison/parity 路线对 source pin、九个路径和
  不复制边界保持一致。
- inventory 回归与文档门禁通过，不修改 Runtime 治理语义或全局 Agent/MCP 配置。

## 验证与非声明

这是语义/参考边界对等，不是 Java 工具链支持、源命令兼容、JSON wire 兼容或
第二技术栈 adopter 验收。每个文件的事实以机器台账为准。

[English](WI-411-reference-java-fixture-boundary.md) · [日本語](WI-411-reference-java-fixture-boundary.ja.md)
