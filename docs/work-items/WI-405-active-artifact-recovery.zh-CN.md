---
author: AI Cockpit maintainers
title: WI-405——活动产物恢复
description: 在不隐藏残留或改写不可变历史的前提下恢复失败 Work Item 产物。
workItemId: WI-405-active-artifact-recovery
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-405-active-artifact-recovery
---

# WI-405——活动产物恢复

## 意图

保留失败或中断 Work Item 产物的可审计性，同时避免 `active/` 中的陈旧文件
被误认为有效的活动治理状态。

## 范围

- 检测并恢复可识别的失败尝试 outcome 与 event 变体。
- 在 archive manifest 中保留其 bytes 与摘要。
- 将孤立活动产物与有效活动 Contract 分开报告。
- 保持 repository 与 Runtime evidence 隔离。

## 证据

- 归档 Contract：`.ai/work-items/archive/WI-405-active-artifact-recovery.contract.json`
- Verification：`.ai/evidence/WI-405-active-artifact-recovery.verification.json`
- 已安装 Runtime：v0.2.40

## 边界

本 Work Item 不改写或删除历史 evidence，不改变发布自动化，也不改变既有
Work Item 决定的含义。
