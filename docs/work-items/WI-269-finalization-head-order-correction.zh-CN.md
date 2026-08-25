---
author: AI Cockpit maintainers
title: "WI-269——Finalization head-order 修正"
workItemId: WI-269-finalization-head-order-correction
description: "在 reviewed archive/evidence commit 稳定后才完成 finalization。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-269-finalization-head-order-correction
terminalArchive: .ai/work-items/archive/WI-269-finalization-head-order-correction.contract.json
terminalVerification: .ai/evidence/WI-269-finalization-head-order-correction.verification.json
terminalFinalization: .ai/decisions/WI-269-finalization-head-order-correction.finalize.b64cf4237f6474b2dcc9d4be732a67fce482bea85d799eb0c438e95e6d43a24f.json
terminalDecision: .ai/decisions/WI-269-finalization-head-order-correction.close.json
authority: canonical
---

# WI-269——Finalization head-order 修正

## 意图

WI-268 暴露了顺序缺陷：evidence/archive commit 之前就记录了 pre-merge finalization receipt，导致 reviewed head 变旧。本 successor 先登记 parity、提交 archive/evidence，再针对稳定 head 记录 finalization。

## 范围与证据边界

- 保留 WI-268 与 WI-267 的不可变 recovery bytes。
- 在 evidence 出现在 Git history 之前登记 successor parity 行。
- 先提交 archive/evidence，再记录 pre-merge finalization receipt。
- finalization commit 只包含 canonical receipt，并完成 hosted review、精确 cleanup 与结构化 close。

## 验证

- `cargo test --locked --workspace`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/work_item_status_consistency_test.sh`
- 使用显式 `--repo` 的已安装 Runtime lifecycle 与可见人工 Outcome

最终交付必须是可见的 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴`，并包含状态、未知项、证据、人工决定和下一步。
