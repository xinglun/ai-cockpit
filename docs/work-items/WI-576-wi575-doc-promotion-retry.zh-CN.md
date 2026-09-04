---
author: AI Cockpit maintainers
title: "WI-576——WI-575 文档晋级重试"
description: "以可证明的生命周期顺序重新交付 WI-574 终态文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-576-wi575-doc-promotion-retry
lastVerifiedBy: WI-576-wi575-doc-promotion-retry
---

[English](WI-576-wi575-doc-promotion-retry.md) · [日本語](WI-576-wi575-doc-promotion-retry.ja.md)

# WI-576——WI-575 文档晋级重试

## 目标

WI-575 的 PR #556 因不可证明的生命周期顺序而作为不可变失败交付关闭。
本 successor 保留该失败作为 provider 审计历史，只修正顺序：先登记三语
parity 行，再归档、评审、合并、关闭，最后晋级。

## 范围与边界

- 晋级 WI-574 的 Work Item 页面及三份 reference-parity 行。
- 维护本 successor 的三语页面和 parity 登记。
- 保留已关闭的 PR #556，不声称其合并，也不改写任何 WI-575 字节。

Runtime 行为、对象工程、全局 Agent/MCP 配置、参考源实现复制、发布以及历史治理字节
不在本 Work Item 范围内。

## 验收

1. WI-576 parity 登记在 archive 之前提交，验证关闭前保持 `进行中`。
2. WI-574 页面仅依据已验证的终态证据标记为已实现。
3. 三份 parity ledger 在关闭后包含准确终态路径。
4. 文档、治理、状态、workspace 和 diff 检查通过。
5. 不改写 WI-575 或其他历史治理记录。

## 验证

- `tests/docs/documentation_acceptance.sh`
- `tests/docs/parity_status_check.sh`
- `tests/docs/pending_parity_registry_test.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `cargo test --locked --workspace`
- `git diff --check`
