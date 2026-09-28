---
author: AI Cockpit maintainers
workItemId: WI-1038-task8-parity-projection-recovery
title: Task 8 parity 投影恢复
description: 通过精确绑定的 Runtime successor 修复 WI-1037 不可变归档后的迟到登记，同时保留历史顺序警告，并对未被 successor 覆盖的恢复继续 fail closed。
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1038-task8-parity-projection-recovery
---

[English](WI-1038-task8-parity-projection-recovery.md) · [日本語](WI-1038-task8-parity-projection-recovery.ja.md)

# WI-1038 — Task 8 parity 投影恢复

此有界 successor 修复已归档 WI-1037 在三份 parity 账本中的迟到登记。WI-1037 的 archive 和验证证据不可变；投影必须明确保留为历史事实，不能改写成仿佛登记早于验证。

## 边界

- 只有有效 Runtime successor 决定将 WI-1038 绑定到 WI-1037，且 WI-1038 Contract scope 与 Summary changed paths 均覆盖三份 parity 文档时，才接受该历史投影。
- 保留前驱原始登记顺序作为 historical warning。恢复缺失、畸形、跨仓库、身份不匹配或仅覆盖部分文档时仍须阻塞。
- 保留既有精确 PR 生命周期门禁，以及 WI-1037 不可变的 archive、Summary、Outcome、events 和验证证据。
- 不提供通用豁免、不做宽泛治理重构、不迁移 Task 9、不发布、不改版本、不打 tag。

## 验收

1. 真实 Git 正例接受身份有效且覆盖全部 parity 账本的 successor recovery，并报告三条 `recovered_postarchive_parity_registration` 历史警告。
2. 负例只要有一份 parity 文档不在 successor Contract scope 中，就仍以 `stale_prearchive_parity_registration` 阻塞。
3. 英文、中文、日文账本准确投影 WI-1037 的不可变归档状态和 WI-1038 已归档但等待合并及关闭的生命周期，不宣称任一已关闭。
4. 聚焦生命周期 fixture 证明 WI-1038 的已登记 parity 行在 archive 转换后仍有效。
5. PR #997 合并前，声明的聚焦检查与精确 head 的 hosted CI 均须通过。合并、Task 8 清理及 Task 9 启动仍须 Runtime 准入；发布前停止并交由人工 review。

声明的本地聚焦验证已通过，Runtime 也已归档 WI-1038。归档后的 parity 检查发现当前 parity 行在 Git 中晚于其验证证据引入；保留证据，通过正确提交顺序让登记先于证据路径。精确 head 的 hosted CI、PR 合并、正式关闭和清理仍待完成。
