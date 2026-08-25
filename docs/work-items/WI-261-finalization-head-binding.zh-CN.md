---
author: AI Cockpit maintainers
title: "WI-261 — Finalization head 绑定"
workItemId: WI-261-finalization-head-binding
description: "在非治理漂移后拒绝过期的 pre-merge finalization 回执。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-261 — Finalization head 绑定

## Intent

将 pre-merge finalization 证据绑定到实际 reviewed branch 或 pull-request head。
不能因为回执字段彼此一致，就让包含未审查代码的后续 checkout 获得授权。

## Scope

- 从 feature checkout（`HEAD`）或 synthetic pull-request merge checkout（reviewed
  feature parent）解析 reviewed head。
- 只接受精确 head，或同一 Work Item 明确允许的 append-only governance range；后续代码
  和无关漂移必须拒绝。
- 用确定性的 fixture 和 shell regression 覆盖绑定及 finalization 后漂移。
- 同步英文、中文、日文文档。

fixture builder 只用于把 canonical finalization 回执建模为 append-only commit；不修改
Runtime 或 Rust crates。

## Out of scope

Rust crates、provider 配置、全局 Agent/MCP 配置，以及独立的 post-merge
`stale_awaiting_merge_close` recovery lifecycle。

## Acceptance

1. feature finalization 回执绑定旧 checkout 时，后续代码提交必须 fail-closed。
2. synthetic pull-request merge checkout 必须绑定到 reviewed feature parent。
3. canonical/digest-suffixed finalization、同 Work Item close，以及固定的 post-finalize
   evidence append 必须保持明确受限。
4. modified、deleted、renamed、无关、malformed 或非治理 path 必须被拒绝。
5. 三语 reference 文档必须说明同一边界。

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `python3 -m py_compile tests/ci/governance_integrity_gate.py tests/ci/fixtures/governance-integrity/build_fixture.py`
- focused gate 通过后执行 Contract 声明的 workspace verification。
