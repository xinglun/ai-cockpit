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

## 仓库级串行边界

Runtime 在写入新 Contract 前会检查所有 linked worktree。其他非 detached worktree 中的 active Contract/Summary，或不完整的记录对，都会阻止新 Work Item。Replacement 不会暗中终止 predecessor；应使用显式 recovery/supersede decision 并保留 predecessor bytes。

## 合并、关闭和清理

交付顺序为：

```text
最新远端默认基线 → 专用分支/worktree → 实现
→ verify/finish/archive → 评审 PR → merge → finalize-verify → close
→ 同步默认分支 → 删除精确分支/worktree
```

PR 合并前不得删除分支，也不能让 Provider 自动删除绕过 finalization。`close` 需要结构化人工决定、归档证据、合并 PR identity、已删除的 finalization receipt、快进同步的默认分支和干净 worktree。任何失败的后置条件都会保持可见并 fail closed。

close 后立即同步文档投影：

```sh
python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all
```

如果检查报告文档过期，应创建范围狭窄的文档晋级 Work Item，运行 helper，并在声明 `ready_on_base` 前重新检查。Helper 只更新读者可见的状态/parity，不重写 Contract、证据、archive 或 decision 历史。

## 恢复与采用

恢复是 append-only 且绑定 identity。snapshot 变化、receipt 过期或 provider 冲突必须记录为 retry、successor 或 supersede decision；不能编辑旧证据来把后续状态变绿。安装、升级和 adapter 设置是独立的仓库 Work Item，并使用不可变公开 Release。没有命令会选择进程级 current project，也不会修改 provider 全局 Agent 或 MCP 配置。

由新 Runtime 创建的 successor 必须携带准确的 predecessor Work Item、Contract digest、recovery path 和 repository 绑定。对于在这些 Contract 字段存在之前创建的历史 successor，Runtime 只在 recovery receipt 本身同时绑定 predecessor/successor，且 successor 具备已验证 archive、严格 verification evidence 和已确认 close decision 时提供窄化兼容路径。新追加的 recovery receipt 会标记 `successorBindingMode: legacy_terminal_evidence`；缺失、foreign、stale、malformed、symlink 或不完整 evidence 仍落入 `recovery_decision_invalid`，不能授权任何转换。该兼容投影不会把未完成 successor 变成 green，也不会重写 predecessor bytes。

这是 Rust-native 的语义工作流。参考源的 `make` 命令、Python 模块和生成历史只是比对材料，不是本仓库的命令或 Runtime authority。
