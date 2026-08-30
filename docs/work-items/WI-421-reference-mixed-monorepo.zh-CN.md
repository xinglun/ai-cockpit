---
author: AI Cockpit maintainers
title: "WI-421——mixed-monorepo fixture 边界"
description: "逐文件比对固定 mixed Python/Node fixture，不复制业务代码或工具链资产。"
workItemId: WI-421-reference-mixed-monorepo
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-421-reference-mixed-monorepo
terminalArchive: .ai/work-items/archive/WI-421-reference-mixed-monorepo.contract.json
terminalVerification: .ai/evidence/WI-421-reference-mixed-monorepo.verification.json
terminalFinalization: .ai/decisions/WI-421-reference-mixed-monorepo.finalize.627a79dd6109dc4aae0c50825bdcce80fb2101ff72a7c5db80906aef485f0137.json
terminalDecision: .ai/decisions/WI-421-reference-mixed-monorepo.close.json
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
---

# WI-421——mixed-monorepo fixture 边界

[English](WI-421-reference-mixed-monorepo.md) · [日本語](WI-421-reference-mixed-monorepo.ja.md)

## 意图与边界

逐一阅读 `examples/fixtures/mixed-monorepo/` 下的固定源文件，记录哪些责任可迁移到
attach 的 Rust 仓库。Python/Node 业务示例、包元数据、源命令和安装行为不进入 Runtime；
只保留对象工程可继承的事实、范围、provider 执行责任和证据绑定边界。

## 范围

五个文件为 `fixture.json`、`package.json`、`pyproject.toml`、`services/api/app.py` 和
`services/api/tests/test_app.py`，并同步更新清单、三语 comparison/parity、参考索引和适配页。

## 验收

- 五个固定路径均已阅读，并在清单中各出现一次，分类为 `reference-only`，理由和 Rust/对象工程对应非空。
- 不复制 fixture 源码、Python/Node 依赖、安装器、provider 全局配置或源 JSON wire。
- 中英日路线对源提交、文件清单、继承边界和不声明内容保持一致。
- 清单、文档和对象/采用方工程继承检查通过。

## 验证边界

本批是语义/文档 parity，不是 Python/Node 工具链支持、源命令兼容或第二技术栈 adopter 验收。
验证使用带显式 `--repo` 的已安装共享 Runtime；对象工程自己的解释器、依赖、命令和 provider
证据不属于本 Work Item。
