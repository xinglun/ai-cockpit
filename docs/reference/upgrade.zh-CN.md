---
author: AI Cockpit maintainers
title: 升级
description: 升级共享 Runtime 和仓库绑定，但不把它误认为项目已就绪。
audience:
  - adopter
  - maintainer
status: current
authority: translation
canonical: docs/reference/upgrade.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
capabilityClaims:
  - runtime_upgrade
---

# 升级

[English](upgrade.md) · [简体中文](upgrade.zh-CN.md) · [日本語](upgrade.ja.md)

已安装 Runtime 的升级和仓库 schema migration 是两种不同操作。Runtime-only upgrade 通常只改变机器上的共享 binary，不改变仓库 `.ai/` bytes。Migration 是显式、经过评审的仓库 Work Item，需要计划、备份/回滚证据和人工决定。

## Runtime 升级

安装前使用不可变公开 Release archive，并验证 manifest、SHA-256 和 Runtime identity。保留当前 Runtime 以便回滚，直到新 binary 通过 doctor 和 release acceptance。安装后每个仓库仍需显式 attach 和 request-scoped 命令：

```sh
ai-cockpit inspect --repo /path/to/project
ai-cockpit compatibility --repo /path/to/project
ai-cockpit doctor --repo /path/to/project
```

Runtime 不会 commit、push、创建/合并 PR，也不会编辑全局 Agent/MCP 配置。如果需要改变 managed adapter，应在目标仓库用独立的显式 `agent install` Work Item 完成。

在 repository migration 或 managed-file replacement 前，先确认不会无意中改变 active Work Item。
Migration plan 必须列出受影响路径、schema/version 过渡、备份位置、回滚条件和人工决定；计划
缺失、格式错误、冲突或过期时停止写入。仅升级 Runtime 不会激活新的 project profile，也不会
宣称仓库已就绪。

如果检测到项目拥有或已分叉的治理文件，保留当前 bytes 并生成冲突报告供评审，不能覆盖文件或
手工修改生成的 evidence。Managed Agent adapter（包括 Cursor rule）必须显式在仓库内安装，并
具有 ownership 与 detach 路径；Runtime 升级不会静默注入它们。

## Repository migration

先运行 `ai-cockpit migrate plan --repo <path>`，只使用命令要求的显式 approval 应用经过评审的计划。Migration 必须保留 Contract、evidence、decision、knowledge 和 archive 历史；不能仅因 Runtime 版本变化就重写旧证据。Migration 未完成或不兼容时，仍可使用只读诊断，但有状态的 lifecycle 写入会 fail closed。

参考源的 installer、`Makefile.ai`、Python 模块和 provider marker 文件不会复制到 Rust 仓库。语义边界是共享的外部 Runtime 加隔离的 repository Protocol。

因此，参考源的 installer/Make 命令只是说明性资料。实际使用安装的 binary、不可变 Release
证据，以及带显式 `--repo` 的目标仓库命令。
