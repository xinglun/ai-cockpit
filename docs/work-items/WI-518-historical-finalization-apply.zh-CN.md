---
author: AI Cockpit maintainers
title: "WI-518 — 历史 finalization 应用"
description: "在没有 canonical predecessor 时，让发布版 Runtime 能诚实记录无 PR 的历史 direct merge，并提供精确的 fail-closed identity 诊断。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
workItemId: WI-518-historical-finalization-apply
lastVerifiedBy: WI-518-historical-finalization-apply
---

[English](WI-518-historical-finalization-apply.md) · [日本語](WI-518-historical-finalization-apply.ja.md)

## 目标

为没有 PR、也没有现存 canonical finalization receipt 的历史 direct merge
（`historicalKind=direct_merge_no_pr`）提供可审计、绑定仓库的应用路径。保留不可变历史，要求真实 Git merge commit 与 parents，并让 resource context 失败原因可操作。

## 范围

- Rust protocol 与 repository 校验/记录路径；
- `finalize-recovery` CLI 帮助；
- protocol、repository 回归测试；
- 英文、简体中文、日文命令文档。

对象仓库保持只读。本 WI 不改写历史 receipt，不放宽当前 Runtime 校验，不虚构 PR 或 human decision，也不负责发布版本。

## 验收

- predecessor 不存在时，完整 direct-merge receipt 可由 `finalize-recovery` 作为第一条 canonical record 写入，并执行与 `finalize` 相同的 archive、Contract、Git parents、repository、current Runtime 校验；
- 只有明确的 historical 低 assurance direct merge 可以解析 provisional legacy context；外部 worktree/base/provider 绑定仍 fail-closed，并指出 binding 类别；
- `finalize-recovery-plan` 输出确定性 identity facts 和人类负责字段，不虚构 branch、authority、PR 或 decision；
- 拒绝输入时不可变 predecessor 与仓库状态不变；
- 三语文档说明 semantic/non-wire 与 historical-low assurance 边界。

## 验证

```text
cargo test --locked -p cockpit-protocol --test resource_finalization -- --test-threads=1
cargo test --locked -p cockpit-repository --test resource_finalization_transition -- --test-threads=1
cargo test --locked -p cockpit-cli --test resource_finalization -- --test-threads=1
cargo test --locked --workspace
```

发布版 adopter acceptance 仍属于发布后的职责，不由这些源码测试替代。
