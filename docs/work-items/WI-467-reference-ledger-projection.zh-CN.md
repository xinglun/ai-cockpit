---
author: AI Cockpit maintainers
title: "WI-467——参考台账投影一致性"
workItemId: WI-467-reference-ledger-projection
description: "让当前参考台账快照与三语文档始终来自同一可校验来源。"
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
lastVerifiedBy: WI-467-reference-ledger-projection
---

# WI-467——参考台账投影一致性

[English](WI-467-reference-ledger-projection.md) · [日本語](WI-467-reference-ledger-projection.ja.md)

## 意图

修复参考源逐文件比较快照正文与机器台账分叉的问题。保留历史快照和 retired
path，并加入回归门禁，拒绝只更新 marker 而不更新可读统计的情况。

## 范围

- 从 `tests/conformance/reference_file_inventory.json` 推导当前三语快照表；当前
  计数排除 retired path，同时单独保留追加式台账总数。
- 让英文、简体中文、日文当前快照使用相同 canonical counts。
- 扩展 `reference_inventory_docs_test.py` 及其 shell wrapper，同时校验可读表格和已有 marker。

## 不在范围内

参考台账 bytes、source lock、历史叙述段落、Runtime 或对象工程、workflow 架构、发布脚本以及
全局 Agent/MCP 配置。

## 验收

1. 当前表格与机器计数一致：4,450 条当前路径、3,681 条 generated-history、252 条
   implemented-different-by-design、1 条 implemented-equivalent、4 条 not-applicable、62 条
   reference-only、450 条 deferred-next-batch、0 条 migrate-gap、669 条 retired path，以及
   5,119 条追加式记录。
2. 即使机器 marker 保持不变，故意修改表格也必须失败。
3. 历史段落和 retired path 记录在声明的当前快照修改之外保持不变。
4. 三语页面继续保留阅读路线以及 semantic/non-wire 边界。

## 验证

- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/conformance/reference_file_inventory_test.sh`
- Contract 声明的 repository quality 和 documentation gates

## 边界

台账是当前计数的权威来源。历史叙述属于不可变审计记录，不会为了匹配后续快照而静默改写。
