---
author: AI Cockpit maintainers
title: "WI-489 — 有界终态文档晋级"
description: "防止已关闭的文档晋级 Work Item 为自身页面创建无界 successor 链。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-489-doc-promotion-terminal-boundary
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-489-doc-promotion-terminal-boundary
---

# WI-489 — 有界终态文档晋级

本 Work Item 将终态文档投影定义为有界规则：普通、格式错误或混合范围
继续 fail-closed；文档晋级 Work Item 可以在自身页面完成终态投影，而不会
仅为更新自身页面而无限创建 successor。

[English](WI-489-doc-promotion-terminal-boundary.md) · [日本語](WI-489-doc-promotion-terminal-boundary.ja.md)

## 范围

- 为文档晋级 helper 和三语状态一致性检查器加入已验证的 self-terminal 边界。
- 为普通晋级、格式错误范围和有界终态投影增加回归 fixture。
- 在中、英、日三语 workflow 中记录该边界。

## 验收

- 边界由精确的仅文档范围推导，不能隐藏任意漂移或通配符路径。
- 普通 Work Item 仍必须完成有证据支持的终态晋级。
- 检查保持确定性，不改写不可变治理记录或全局 Agent/MCP 配置。

## 验证

- `bash tests/docs/promote_closed_work_item_test.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `python3 tests/conformance/reference_file_inventory.py --check`
