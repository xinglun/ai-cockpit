---
author: AI Cockpit 维护者
title: "WI-839——Runtime 文档 projection 前置检查"
description: "在昂贵 verification 和终态 close 前拒绝缺失的 Work Item 文档 projection。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-839-runtime-doc-projection-preflight
lastVerifiedBy: WI-839-runtime-doc-projection-preflight
terminalArchive: .ai/work-items/archive/WI-839-runtime-doc-projection-preflight.contract.json
terminalVerification: .ai/evidence/WI-839-runtime-doc-projection-preflight.verification.json
terminalDecision: .ai/decisions/WI-839-runtime-doc-projection-preflight.close.json
---

[English](WI-839-runtime-doc-projection-preflight.md) · [日本語](WI-839-runtime-doc-projection-preflight.ja.md)

# WI-839——Runtime 文档 projection 前置检查

## 意图与边界

本 Work Item 将仓库的三语 Work Item projection 要求前置到安全的生命周期边界，
覆盖 Runtime preflight、verification 入口、close 保护、回归测试和操作命令参考。
产品行为、发布制品、历史记录改写和无关 Work Item 不在范围内。

## 验收

- 使用三语 parity convention 的 repository 会在 verification 前拒绝缺失或格式错误的自身页面或 parity 行。
- 拒绝结果指出准确路径和原因，且不写入终态 lifecycle 状态。
- 没有该 convention 的 repository 保持通用 Work Item 行为。
- 命令参考说明同一根因使用 amend/revalidate 与 retry；只有边界真正不同且有 predecessor 绑定时才使用 successor。

## 验证

- `cargo test -p cockpit-repository --test lifecycle_entry`
- `cargo test -p cockpit-repository --test agent_rule_parity`
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `git diff --check`
