---
author: AI Cockpit maintainers
title: "WI-523——WI-521 文档晋级重试"
description: "在前驱合并前 finalization 过期后，从最新已评审基线重新交付受限的 WI-521 文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-523-wi521-doc-promotion-retry
lastVerifiedBy: WI-523-wi521-doc-promotion-retry
---

[English](WI-523-wi521-doc-promotion-retry.md) · [日本語](WI-523-wi521-doc-promotion-retry.ja.md)

## 目标

从最新已评审的默认分支重新交付 WI-521 终态文档投影，同时保留 WI-522 的不可变 archive 与 recovery 记录。

## 范围

- 将不可变的 WI-522 前驱标记为已恢复，并链接其 recovery 与 successor。
- 晋级 WI-521、WI-523 读者页面及三种语言的 parity 投影。
- 保持 Runtime 生成的证据、前驱字节、对象工程和全局配置不变。

## 验收

- WI-522 明确保持已恢复；过期 finalization 不得被呈现为成功。
- 三个 WI-523 页面与 parity 行在终态后绑定准确的 archive、verification、finalization 和 close 证据。
- 文档、parity、状态一致性和治理完整性检查在最终 archive 提交上通过。
- 仅在 archive 完成后创建合并前 finalization，且其 head 等于已评审 PR head。
- 不修改前驱证据或对象工程文件。

## 验证

```text
python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all
python3 tests/docs/work_item_status_consistency.py --repo <repo>
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/ci/governance_integrity_gate.py --repo <repo>
git diff --check
```
