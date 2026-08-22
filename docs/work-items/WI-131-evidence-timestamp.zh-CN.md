---
author: AI Cockpit maintainers
workItemId: WI-131-evidence-timestamp
title: 验证证据时间戳 fail-closed 校验
description: 在 Outcome 或生命周期完成前拒绝验证证据和 retention 元数据中的非法 RFC3339 时间戳。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-133-docs-truth
---

# WI-131 — 验证证据时间戳 fail-closed 校验

## Intent

验证证据是可审计记录。不能因为时间戳字段存在，就把格式错误的时间戳当作当前
证据，或让生命周期以绿色继续。

## Boundaries

- 将 v2 envelope 的 `createdAt` 以及 retention 的 `createdAt`/`expiresAt` 校验为 RFC3339。
- Outcome、finish、archive、close 复用现有证据校验。
- 保留历史 bytes；legacy evidence 在重新验证生成 v2 之前保持 historical yellow。
- 不翻译 Contract 原文，也不改变 retention policy 的语义。

## Acceptance

- 在其他 identity/digest 校验都通过时，有效 v2 时间戳仍显示绿色。
- 缺失、格式错误或语义无效的时间戳不会产生绿色 Outcome，并阻止 finish/archive/close。
- repository 与 CLI 回归测试覆盖篡改、archived close 以及 legacy evidence。
- 英文、简体中文、日文 Outcome 文档说明时间戳校验边界。

## Verification

定向 repository/CLI 测试、workspace 检查和文档验收结果见 archived Contract、
verification evidence、close decision 与 Runtime evidence。证据为
`.ai/evidence/WI-131-evidence-timestamp.verification.json` 与
`.ai/decisions/WI-131-evidence-timestamp.close.json`。
