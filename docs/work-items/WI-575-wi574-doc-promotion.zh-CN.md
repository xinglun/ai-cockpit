---
author: AI Cockpit maintainers
title: "WI-575 — WI-574 终态文档晋级"
description: "在不重写不可变治理记录的前提下，晋级已关闭 WI-574 的发布文档。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-575-wi574-doc-promotion
lastVerifiedBy: WI-575-wi574-doc-promotion
---

[English](WI-575-wi574-doc-promotion.md) · [日本語](WI-575-wi574-doc-promotion.ja.md)

# WI-575 — WI-574 终态文档晋级

## 目标

晋级 WI-574 的 verified-close 发布文档页面，并在三语 parity 矩阵中注册本次
文档投影。不修改不可变治理记录。

## 范围与边界

范围包括三个 WI-574 页面、三个 WI-575 页面和三个 reference-parity 页面。
Runtime 行为、发布工件、对象工程、全局 Agent/MCP 设置以及历史治理字节均在
范围之外。

## 验收

- 三个 WI-574 页面标记为 `implemented`，并链接 archive、verification、
  finalization 和 close 证据。
- 三个 parity 页面将 WI-574 标记为已实现，并注册 WI-575 的有界终态投影。
- 文档、parity、promotion、状态一致性、治理完整性和 diff 检查通过，不重写
  不可变治理记录。
- WI-575 具有可读且相互对应的英语、简体中文和日语页面。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh .`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report /tmp/wi575-governance-report.json`
- `git diff --check`
