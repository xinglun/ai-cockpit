---
author: AI Cockpit 维护者
title: "WI-414——Python fixture 边界"
workItemId: WI-414-reference-python-fixture-boundary
description: "逐一比较固定的 Python fixture 文件，记录仅参考边界，不复制源 fixture。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-414-reference-python-fixture-boundary
terminalArchive: .ai/work-items/archive/WI-414-reference-python-fixture-boundary.contract.json
terminalVerification: .ai/evidence/WI-414-reference-python-fixture-boundary.verification.json
terminalFinalization: .ai/decisions/WI-414-reference-python-fixture-boundary.finalize.json
terminalDecision: .ai/decisions/WI-414-reference-python-fixture-boundary.close.json
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
canonical: docs/work-items/WI-414-reference-python-fixture-boundary.md
---

# WI-414——Python fixture 边界

## 意图与边界

在参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 下，逐一读取
`examples/fixtures/python/` 的四个文件。这些文件是参考工程的可执行
Python/pytest 样例，不是 Rust Runtime 代码、Python toolchain 支持、可移植治理策略或企业证据。

| 固定参考路径 | 分类 | 目标边界决定 |
| --- | --- | --- |
| `fixture.json` | `reference-only` | 样例 stack、平台与路径元数据；目标事实保持仓库本地，不从本文件推断。 |
| `pyproject.toml` | `reference-only` | 样例打包和 pytest 配置；Python 安装与测试命令属于对象工程/Provider。 |
| `src/service.py` | `reference-only` | 返回 `ok` 的应用样例；不是治理逻辑，不复制。 |
| `tests/test_service.py` | `reference-only` | fixture 专用 pytest 断言；不是 Runtime 或企业证据，对象工程必须声明自己的 verification 命令。 |

不向 Rust 工程复制 Python 源码、依赖清单、安装器或测试运行器。共享已安装 Runtime
仍向 Python 对象工程提供相同的 Contract、evidence、lifecycle 与面向人的 Outcome 控制；
但这只是语义/文档对齐，不是 Python toolchain 或源命令兼容。第二技术栈 adopter 验收需单独授权，
本 WI 不作该声明。

## 验收

- 四个固定路径均已读取，并在机器台账中恰好出现一次。
- 四个路径全部为 `reference-only`，均有非空原因和目标边界；本批不留
  `deferred-next-batch` 或 `migrate-gap`。
- 英文、简体中文、日文 comparison/parity 路线对 source pin、文件列表和不复制边界保持一致。
- inventory 回归和文档门禁通过，不修改 Runtime 治理语义、Python 工具链或全局 Agent/MCP 配置。

## 验证与非声明

这是语义/参考边界对等，不是 Python 工具链支持、源命令兼容、JSON wire 兼容或第二技术栈
adopter 验收。每个文件的事实以机器台账为准。

[English](WI-414-reference-python-fixture-boundary.md) · [日本語](WI-414-reference-python-fixture-boundary.ja.md)
