---
author: AI Cockpit maintainers
workItemId: WI-132-agent-adapter-parity
title: Agent adapter 与 provider 表面一致性
description: 在明确 Rust Runtime 边界的同时，把参考源的 Contract-first 与可见 Outcome 规则传递给仓库本地 Agent adapter。
audience:
  - adopter
  - contributor
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-132-agent-adapter-parity
---

# WI-132 — Agent adapter 与 provider 表面一致性

## Intent

让已安装的 Agent 获得与参考源一致的安全操作边界，同时不复制 Python/Make
Runtime。Adapter 是显式的 discovery 投影，当前治理状态仍由共享 Rust Runtime
负责。

## Boundaries

- 新安装的 Cursor 使用 provider 原生 `.cursor/rules/ai-cockpit.mdc`。
- 已有受管理的 `.cursor/rules/ai-cockpit.md` 保持可读、可拥有、可回滚；不重命名或覆盖用户文件。
- managed section 包含 Contract-first、unknowns、preflight 人工暂停、Summary、可见 Outcome 和合并后的 closure 规则。
- 更新英文、日文、简体中文的 glossary 与 reference workflow。
- 不安装 provider/全局配置，不改 Core protocol，不复制 V1 runtime code、schema、installer、Python module 或 Make command。

## Acceptance

- provider detection、install、doctor、repair、detach 保持 repository-bound、确定性、隔离且对损坏 ownership 或 symlink 表面 fail closed。
- 新安装 Cursor 的 canonical target 为 `.mdc`，受管理 legacy `.md` 无需不安全迁移即可继续使用。
- `not_ready` 或 `needs_human_confirmation` 必须暂停并请求人类；不得擅自补全 Contract 决定；archive/closure 前必须展示可见 Outcome。
- glossary 与三语 workflow/parity 文档说明 Rust 适配边界和 provider 表面策略。

## Verification

Focused Agent/CLI 测试、workspace checks、clippy 和 documentation acceptance 的结果
参见 active Contract 与 Runtime evidence。
