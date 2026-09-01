---
author: AI Cockpit maintainers
title: "WI-470——终态文档提升与历史产物恢复"
description: "恢复 archive manifest 引用的历史产物，并在不改写 predecessor truth 的前提下提升已关闭 WI-469 投影。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-470-terminal-doc-promotion-and-artifact-recovery
status: in_progress
authority: authorized
lastVerifiedBy: WI-470-terminal-doc-promotion-and-artifact-recovery
---

# WI-470——终态文档提升与历史产物恢复

## 意图与边界

WI-470 是有界恢复 Work Item：恢复不可变 WI-467/WI-468 archive manifest
引用的精确历史 task report，并在三种语言中提升已验证关闭的 WI-469 终态文档投影。
不改写 predecessor 的 archive、evidence、recovery 或 close bytes。

## 范围

- 从记录的源提交逐字节恢复 WI-467 与 WI-468 缺失的 task-report 产物。
- 在 WI-469 验证关闭后提升其 Work Item 文档和 reference-parity 行。
- 保留 WI-467/WI-468 的 supersede recovery 与 close receipt。
- 保持 post-close 文档与 archive manifest 检查可重复执行。

## 范围外

reference inventory 源文件、Runtime/Core 实现、对象工程、发布/Adopter 脚本及全局 Agent/MCP 配置。

## 验收

1. 两组缺失 task-report 均精确恢复，且 archive manifest 校验通过。
2. 英文、简体中文、日文的 WI-469 终态投影一致。
3. 文档、conformance 与 workspace 门禁通过，且不重写 predecessor archive/evidence/recovery bytes。

## 验证

- `cargo test --locked --workspace`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/parity_status_check.sh`

## 恢复边界

前置 recovery receipt 仍是历史证据；WI-469 是已验证 successor。本 Work Item 只修复可审计关闭历史所需的缺失 manifest 产物与终态投影。
