---
author: AI Cockpit maintainers
title: "WI-43 — Runtime 兼容性与 Repository Migration Protocol"
description: "Runtime-only 升级与显式 repository migration 的实现边界和用户流程。"
audience:
  - maintainer
  - adopter
status: current
authority: canonical
lastVerifiedBy: implementation-acceptance
capabilityClaims:
  - runtime_upgrade_boundary
  - repository_migration
---

# WI-43 — Runtime 兼容性与 Repository Migration Protocol

## 目标

让一份共享 Runtime 可以升级，同时不静默改变 repository 的治理状态。
Runtime-only 升级保持 `.ai/` 不变；repository schema 变化必须显式、可审查、经批准并绑定 receipt。

## 用户流程

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
ai-cockpit migrate apply --repo /path/to/repository --approved
```

兼容性状态：

- `COMPATIBLE`：可运行正常 lifecycle、Agent、MCP 和 verification；
- `MIGRATION_REQUIRED`：保留 inspect 与只读 plan，但停止会写入状态或产生 evidence 的操作；
- `INCOMPATIBLE`：fail closed，直到安装支持已保存 schema 的 Runtime。

当前 Repository Protocol 是 version 1，repository schema 目标是 version 2。
旧 schema 文件按 legacy state 读取；`status`、`attach` 或普通 Runtime 调用都不会自动升级它们。

## Receipt 与保留规则

应用 migration 会写入 `.ai/migrations/<migration-id>.json`，记录 source/target schema、前后 digest、
Runtime version、Runtime digest、变更文件和 result。只允许修改 versioned protocol files 与 migration
record。Archive Work Item、evidence、decision、knowledge 和其他历史记录保持不变。Runtime 不持有全局
current repository 或全局 Work Item。

## 验收

- 旧 schema 默认识别为 version 1 并报告 `MIGRATION_REQUIRED`；
- `migrate plan` 只读并声明需要人工批准；
- 没有 `--approved` 的 `migrate apply` 失败且不改变 bytes；
- 批准后进入 `COMPATIBLE` 并生成绑定 Runtime 的 receipt；
- 重复应用会被拒绝；
- 历史 evidence 和 archive Work Item bytes 不变；
- 所有 repository 命令继续要求显式 `--repo`。
