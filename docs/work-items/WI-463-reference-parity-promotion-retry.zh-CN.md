---
author: AI Cockpit maintainers
title: "WI-463——reference parity 文档 promotion 重试"
workItemId: WI-463-reference-parity-promotion-retry
description: "在干净基线重试 WI-461 parity 投影，保留之前被 CI 治理证据顺序阻塞的不可变交付记录。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-463-reference-parity-promotion-retry
---

# WI-463——reference parity 文档 promotion 重试

本 bounded successor 从干净基线重新交付已验证并关闭的 WI-461 面向读者的
parity 投影。只修改文档；不修改 Runtime、发布事实、对象工程或不可变 evidence。
失败的 WI-462 交付作为独立审计记录保留。

[English](WI-463-reference-parity-promotion-retry.md) · [日本語](WI-463-reference-parity-promotion-retry.ja.md)

## 范围

- 在三份 parity ledger 中将 WI-461 行提升为终态“已实现”。
- 保留 WI-461 不可变 archive、verification、finalization 和 close 路径。
- 维护本 Work Item 的三语页面及归档前 parity 行。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `cargo test --locked --workspace`

只有在评审合并、finalization 和 close 后，文档 promotion helper 才会提升本 Work Item 的终态字段。
