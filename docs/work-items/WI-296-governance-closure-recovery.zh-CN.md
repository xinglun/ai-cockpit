---
author: AI Cockpit maintainers
title: "WI-296——治理收尾恢复"
workItemId: WI-296-governance-closure-recovery
description: "补齐 consumed retry 历史投影、parity 和终态 finalization 检查。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-296-governance-closure-recovery
authority: canonical
---

# WI-296——治理收尾恢复

## 意图

Work Item 关闭后，仍将已消费的 retry 保留为历史证据；同时让文档和
finalization gate 与 Runtime 实际生成的终态回执一致。

## 范围

- 将 consumed retry 历史保持为历史事实，而不是当前错误。
- 当 merge 与精确清理被一次性观察到时，接受完整绑定的直接终态
  finalization receipt。
- 对不完整、malformed、foreign 或 forked evidence 继续 fail-closed。
- 根据不可变 closure evidence 同步 WI-294 终态文档和三语 parity ledger。

## 边界

Rust Core 行为、发布/adopter harness 以及历史 archive bytes 不在本次恢复范围。

## 验收

- confirmed close 之后，consumed retry 仍显示为历史事实。
- 直接终态 receipt 只有在 merge、删除资源状态和 merge identity 完整时才可接受；
  transition chain 仍必须有 sequence 1 和 2。
- WI-294 文档由不可变 closure evidence 正确提升。
- 完整 repository gate 与 hosted checks 通过。

## 验证

关闭前必须通过已安装 Runtime lifecycle、repository governance gates、文档验收和
hosted quality checks。

## 未知项

用户可见收益在 Work Item owner 明确声明前保持 unknown。
