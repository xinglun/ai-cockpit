---
author: AI Cockpit maintainers
title: "WI-469——reference ledger parity 顺序恢复"
description: "恢复不可变的 WI-468 交付，并在新验证证据之前登记 parity 投影。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-469-reference-ledger-parity-order-recovery
predecessorWorkItemId: WI-468-reference-ledger-parity-promotion
status: in_progress
authority: authorized
lastVerifiedBy: WI-469-reference-ledger-parity-order-recovery
---

# WI-469——reference ledger parity 顺序恢复

## 意图与边界

WI-469 是不可变 WI-468 的明确恢复 successor。目标是在保留所有前置
archive/evidence 字节的前提下，修正托管治理门拒绝的文档投影顺序。

固定本地参考源为 `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`，
仅用于语义比对；本 Work Item 不复制参考 Runtime、Python 模块或参考仓库状态。

## 范围

- 保持三语比较页面的 manifest 派生当前快照一致。
- 在三语页面中将 WI-467 与 WI-468 投影标记为已恢复。
- 在生成 WI-469 verification evidence 之前，在三份 parity 台账登记 WI-469 行，
  并显式预留终态记录路径。
- 保持前置记录不可变并保留恢复 lineage。
- 让文档与 conformance 门对计数、状态、语言和历史顺序漂移 fail-closed。

## 验收

1. 英文、简体中文、日文文档中的 WI-467/WI-468 均为 `recovered`，并绑定恢复 evidence。
2. 三份 parity 台账中的 WI-469 行都在其 verification evidence 出现在 Git 历史之前。
3. manifest 派生快照和三语阅读路线通过回归门；故意改变表格或行时必须 fail-closed。
4. 不重写任何前置 archive、evidence、recovery 或历史源字节。

## 验证

- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `bash tests/docs/parity_status_check.sh`
- Contract 声明的 workspace quality gate

## 恢复边界

WI-468 的 CI 拒绝是确定性的顺序缺陷：其终态 parity 行在 verification evidence 之后才首次出现。
本 successor 先登记 parity 行，再执行新的验证。前置记录保持历史/已恢复状态，不重写也不冒充当前成功。
