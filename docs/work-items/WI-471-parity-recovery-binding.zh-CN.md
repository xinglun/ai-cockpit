---
author: AI Cockpit maintainers
title: "WI-471——parity recovery 绑定"
description: "在每份 reference-parity 台账中绑定 WI-469 的权威 recovery receipt。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-471-parity-recovery-binding
status: in_progress
authority: authorized
lastVerifiedBy: WI-471-parity-recovery-binding
---

# WI-471——parity recovery 绑定

## 意图与边界

关闭后的 governance-integrity gate 发现，三份 parity 台账中的 WI-469 行
只列出了普通 close 路径，没有列出 Runtime 选定的、带摘要后缀的权威 recovery
receipt。本 Work Item 只补充这一明确绑定，不重写历史字节，也不改变 Runtime 行为。

## 范围

- 在英文、简体中文、日文三份 reference-parity 台账的 WI-469 行中加入准确的
  权威 recovery receipt 路径。
- 保留现有 archive、verification、finalization、close 引用。
- 在本 Work Item 的三语页面记录相同边界。

## 验收

1. 三份 WI-469 行都包含已验证的 recovery receipt 及全部终态 lifecycle 引用。
2. `tests/ci/governance_integrity_gate.py` 报告零 finding。
3. 不重写历史 archive、evidence、recovery、close 或源字节。
4. 关闭后运行三语 Work Item 状态一致性检查仍通过。

## 验证

- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `bash tests/docs/parity_status_check.sh .`
- `bash tests/docs/documentation_acceptance.sh`

## 恢复边界

该 recovery receipt 是 Runtime 因 WI-469 存在已关闭 successor 而选定的不可变终态投影。
在 parity 行中列出它不会重新分类或改写 predecessor，只是让现有决策路径可审计，避免
关闭后的门禁误判为缺失。
