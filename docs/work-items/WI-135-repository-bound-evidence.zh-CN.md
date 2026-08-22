---
author: AI Cockpit maintainers
workItemId: WI-135-repository-bound-evidence
title: Repository 绑定的 retention 与关闭证据
description: 在所有生命周期边界把 retention metadata 与 close receipt 绑定到当前 repository 和 Work Item。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-135-repository-bound-evidence
---

# WI-135 — Repository 绑定的 retention 与关闭证据

## Intent

防止复制、损坏或来自其他 repository 的 retention policy 或 close receipt 被当作当前
repository 的真实治理事实接受。

## 边界

- 使用 retention policy 前校验 schema 版本、repository identity、Work Item identity、
  时间戳和 retention 值。
- verification evidence 中的 retention 与 repository-local policy 同时存在时必须一致。
- close receipt 写入并且必须具备 repository identity；缺失或 foreign receipt 不能把
  archived Work Item 提升为 `closed`，也不能显示为有效人工决定。
- 历史 evidence bytes 保持不可变，本 WI 不重写历史记录。

## 验收

- 有效 retention 与 close 记录保持可读取。
- foreign、缺失、格式错误、未知字段、schema 不匹配及跨 repository 记录在 Outcome、
  MCP、finish、archive、close、status、purge 路径全部 fail closed。
- 回归测试覆盖 repository/Work Item 绑定并保留 legacy historical projection。

## 验证

归档 verification evidence：`.ai/evidence/WI-135-repository-bound-evidence.verification.json`。
关闭 decision：`.ai/decisions/WI-135-repository-bound-evidence.close.json`。本 WI 不引入
Task Report 或 Recovery 状态功能。
