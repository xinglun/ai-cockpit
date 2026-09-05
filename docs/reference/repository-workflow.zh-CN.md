---
author: AI Cockpit maintainers
title: 仓库工作流
description: 面向仓库的 Work Item、评审、归档和清理流程。
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: translation
canonical: docs/reference/repository-workflow.md
lastVerifiedBy: WI-378-reference-documentation-batch-17
capabilityClaims:
  - repository_workflow
---

# 仓库工作流

[English](repository-workflow.md) · [简体中文](repository-workflow.zh-CN.md) · [日本語](repository-workflow.ja.md)

AI Cockpit 对一个有界变更使用一个 Work Item、一个专用分支/worktree 和一个经过评审的 Pull Request。安装在机器上的 Runtime 可以共享，但 Contract、证据和 repository identity 按请求隔离。

## 从开始到评审

1. 获取远端默认分支的最新提交，并在 Contract 中记录 remote、branch 和 revision。
2. 从该 revision 创建专用 linked worktree 和分支。
3. 使用显式 scope、out-of-scope、authority、acceptance 和 required evidence 运行 `ai-cockpit start --repo <worktree> --id <id> --intent <text> --goal <text>`。
4. 运行 preflight 和 checkpoint。黄色或红色是可见的人工评审条件，不是编辑或完成的许可。
5. 只修改声明范围，用同一个 `--repo` 和显式 argv 记录验证，然后运行 `finish` 与 `archive`。
6. 推送精确分支，创建一个经过评审的 PR，等待必需的托管检查。不能用本地合并到 `main` 替代评审。

### Finalization 上下文必须明确

`start` 写入的 `resourceContext` 在显式运行
`work-item finalize-plan` 绑定已评审的 PR、provider、base、分支和 worktree
之前都属于临时上下文。`pending` 和 `pending:<stable-reference>` 都是临时哨兵，
不能授权 `finish` 或 `archive`。必须先用真实的已评审资源运行 `finalize-plan`，再执行终态步骤。

## 仓库级串行边界

Runtime 在写入新 Contract 前会检查所有 linked worktree。其他非 detached worktree 中的 active Contract/Summary，或不完整的记录对，都会阻止新 Work Item。Replacement 不会暗中终止 predecessor；应使用显式 recovery/supersede decision 并保留 predecessor bytes。

## 合并、关闭和清理

交付顺序为：

```text
最新远端默认基线 → 专用分支/worktree → 实现
→ verify/finish/archive → 评审 PR → merge → finalize-verify → close
→ 同步默认分支 → 删除精确分支/worktree
```

PR 合并前不得删除分支，也不能让 Provider 自动删除绕过 finalization。新的 Work Item 的
`close` 需要结构化人工决定、归档证据、合并 PR identity、已删除的 finalization receipt、
快进同步的默认分支和干净 worktree。已验证的历史 shared-worktree 或 direct-merge receipt
可以在 `historical_low` assurance、明确人工授权和 repository 绑定的 Git 事实下使用窄化
`retained` 例外；它不适用于新的 Work Item，也不会升级历史 evidence。任何失败的后置条件
都会保持可见并 fail closed。

close 后立即同步文档投影：

```sh
python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all
```

如果检查报告文档过期，应创建范围狭窄的文档晋级 Work Item，运行 helper，并在声明 `ready_on_base` 前重新检查。Helper 只更新读者可见的状态/parity，不重写 Contract、证据、archive 或 decision 历史。

如果文档晋级 Work Item 声明了精确的 docs-only scope，并包含它自己的三语页面和三份 parity ledger，它就是一个有界的自投影终态边界。该 Work Item 关闭后，`--check-all` 仍会验证不可变的终态证据，但会接受它自己的预归档 `进行中 → 验证关闭后已实现` 投影；不能仅为了重写自身而继续创建 successor。混合、通配符或格式错误的 scope 不享受此例外，仍然 fail closed。

## 恢复与采用

恢复是 append-only 且绑定 identity。snapshot 变化、receipt 过期或 provider 冲突必须记录为 retry、successor 或 supersede decision；不能编辑旧证据来把后续状态变绿。安装、升级、adapter 设置和历史 finalization recovery 是独立的仓库操作，适用时使用不可变公开 Release。`work-item finalize-recovery --repo <path> --id <id> --input <receipt.json>` 是不可变旧 finalization 的唯一兼容路径：它绑定 predecessor digest、repository/Work Item/Contract base、当前 Runtime、actor、authority、reason 和 timestamp，但不编辑 predecessor。没有命令会选择进程级 current project，也不会修改 provider 全局 Agent 或 MCP 配置。

由新 Runtime 创建的 successor 必须携带准确的 predecessor Work Item、Contract digest、recovery path 和 repository 绑定。对于在这些 Contract 字段存在之前创建的历史 successor，Runtime 只在 recovery receipt 本身同时绑定 predecessor/successor，且 successor 具备已验证 archive、严格 verification evidence 和已确认 close decision 时提供窄化兼容路径。新追加的 recovery receipt 会标记 `successorBindingMode: legacy_terminal_evidence`；缺失、foreign、stale、malformed、symlink 或不完整 evidence 仍落入 `recovery_decision_invalid`，不能授权任何转换。该兼容投影不会把未完成 successor 变成 green，也不会重写 predecessor bytes。

一个 predecessor 只能有一条已选定的 successor lineage。已有有效的
`successor` receipt 后，若再次为同一 predecessor 指向不同 Work Item，Runtime
会以稳定边界 `recovery_decision_invalid:competing_successor` 拒绝；应继续原
lineage，或显式记录 `supersede`，不能把多个 successor 留给人从文件名中猜测。
这样可以让 recovery graph 确定，并在不重写历史 bytes 的前提下保持终态决策可审计。

如果经审查的修复合法地修改了已归档 Contract，应使用 `work-item revalidate-archived` 记录
`contract_amendment_revalidation` successor decision。它绑定当前 archive manifest 与
Contract digest，同时保留历史 Contract 和 verification evidence digest，创建
`not_ready` successor scaffold，并让 predecessor 保持 pending，直到 successor 达到已
验证、已 finalization 且有人类确认 close 的终态。predecessor bytes 永不改写；无效历史
evidence 不能用于创建 successor。

如果已归档 predecessor 中还保留了一个目标从未完成绑定的旧 successor 尝试，
较新的有效 `supersede` receipt 可以解决这类历史残留。Runtime 只有在该较新 receipt
有效且按记录的决定时间胜出时，才把旧记录视为历史；malformed、foreign、被篡改或
更新但无效的记录仍然 fail closed。Runtime 不会重写任何 Contract、Summary、Outcome、
Events、Evidence 或 recovery receipt bytes。

Repository readiness 对入口门禁使用同一边界。已归档 predecessor 只有在 recovery
receipt 有效，且选定的 successor 已归档，并同时具备通过 manifest 校验、绑定本仓库的
Contract/Summary、已验证 Outcome 和已确认 close decision 时，才会从 `pending close`
列表中移除。缺失、stale、foreign、malformed、symlink 或仍未关闭的 successor 都不会
抑制该 blocker。这样既不会让一条已完成的 recovery lineage 永久阻塞整个仓库，也不会
让未经证明的 successor 悄悄隐藏历史债务。

这是 Rust-native 的语义工作流。参考源的 `make` 命令、Python 模块和生成历史只是比对材料，不是本仓库的命令或 Runtime authority。
