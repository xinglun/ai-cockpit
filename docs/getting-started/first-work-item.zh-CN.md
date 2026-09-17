---
author: AI Cockpit maintainers
title: "首个 Work Item"
description: "从 authorized Contract 到 reviewed close 的完整 Runtime 原生路线。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_lifecycle
---

# 首个 Work Item

一个 Work Item 使用一个专用 branch/worktree 和一个 pull request。从 repository 探测到的
remote default branch 最新 commit 开始；每个 repository-bound 命令都显式指定 repository。

```bash
repo=/path/to/repository
id=WI-001-example-change
ai-cockpit start --repo "$repo" --id "$id" --intent "完成有边界的示例变更。" --goal "为示例交付受审查证据。" --scope 'docs/**' --out-of-scope 'src/**' --risk normal --authority authorized --acceptance "文档示例与已登记检查通过。" --required-evidence verification
```

审查生成的 human-owned Contract。它必须写明实际 source、scope、out-of-scope、acceptance、
verification、authority、remote、default branch 与 base revision。绝不手改生成的 Summary、
evidence、Outcome、archive 或 decision receipt。

## 普通路径：声明、实现、验证、评审、合并、清理

普通 repository-only 变更使用 `start --prepare`：它记录 Contract，运行廉价前检，并在不需要
人类决定时创建 checkpoint。详细协议由 Runtime 承接，不要手改治理状态。

```bash
ai-cockpit start --prepare --repo "$repo" --id "$id" \
  --intent "完成有边界的示例变更。" --goal "交付经过评审的示例。" \
  --scope 'docs/**' --out-of-scope 'src/**' --risk normal --authority authorized \
  --acceptance "示例与声明的检查通过。" --required-evidence verification
```

随后实现声明的变更并运行工程验证：

```bash
ai-cockpit verify --repo "$repo" --work-item "$id" --command cargo --args test,--workspace --workers 1
ai-cockpit finish --repo "$repo" --id "$id"
```

使用 Contract 的工程命令；Cargo 只是一例。范围明确、有真实改动且基础检查完成时为**可进入评审**；
所需验证、授权和当前证据满足时才是**可合并**；相应合并决定与准确资源清理完成后才是**已收尾**。
Draft PR 只是评审界面，不等于验证或合并授权。Provider-bound 或发布任务请参阅展开的
[Agent workflow 参考](../reference/agent-workflow.zh-CN.md)。

Preflight 若需要人类决定，展示 Runtime review，只请求该具体决定；`verification_pending` 只能继续
收集声明证据，剩余 unknown 由 Runtime 保留。

## 展示可见 Outcome，再 archive

把面向人的交接作为独立可见消息输出：

```bash
AI_COCKPIT_LANGUAGE=zh-CN ai-cockpit work-item outcome --repo "$repo" --id "$id"
```

交接以 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴` 开始，并包含状态、unknown、证据、
human decision 与 next action。只有 current green Outcome 可以继续。JSON lookup 或 folded tool
result 不能替代交接。

```bash
ai-cockpit archive --repo "$repo" --id "$id"
```

## 经过 merge 与 cleanup 完成收尾

先单独提交并推送 archive bundle。该 push 后重新读取 provider PR，并要求 worktree clean。
然后获取 provider-derived receipt；它必须绑定 repository ID、Work Item、Runtime
version/digest、archived Contract digest、准确 PR、branch、worktree 与 resource context。
Merge 前的严格 receipt 是 blocked：reason 为 `awaiting_merge_close`，PR 为 unmerged、branch
present、worktree clean，且 `failureCodes: ["unmerged_pull_request"]`；这不是 retained success。

```bash
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.premerge-finalize-receipt.json
```

Runtime 会写入 canonical finalization receipt。下一次 governance commit 只提交并推送该 receipt；
不得把 source、documentation、archive 或其他 governance 变更混入这次 head advance。要求 hosted
checks 通过，再由受审查 pull request merge。
不得把 branch 直接 merge 到 local `main`，也不得让 provider 在 cleanup evidence 产生前删除 branch。
Merge 后把 `--repo` 指向仍存在且已 fast-forward 同步的 default-branch checkout；已删除的
feature worktree 不能继续作为 command root。用额外 `work-item finalize` 调用追加
provider-derived merge-observation 与准确 cleanup receipt；receipt 构成不可变线性链：

```bash
repo=/path/to/synchronized-default-branch-worktree
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.merge-observation-receipt.json
ai-cockpit work-item finalize --repo "$repo" --id "$id" --input /tmp/WI-001.cleanup-receipt.json
```

然后验证唯一 terminal head：

```bash
ai-cockpit work-item finalize-verify --repo "$repo" --id "$id"
```

只有在 default branch 已同步、merged head 已绑定、worktree clean 且准确 owned local/remote branch
已删除后，authorized person 才能记录 structured close decision：

```bash
decision_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ai-cockpit close --repo "$repo" --id "$id" --human-decision approved --actor human:repository-owner --authority-source repository-review-policy --reason "受审查证据与准确 cleanup 已完成。" --evidence-ref ".ai/evidence/WI-001-example-change.verification.json" --policy-ref "repository-review-policy" --decided-at "$decision_time" --resume-condition none
```

任何 failed 或 unknown transition 都保持 open，并保留证据与 recovery condition。不得删除或
重写记录来制造 green 生命周期。
[Agent workflow 参考](../reference/agent-workflow.zh-CN.md)定义了上述 receipt 文件使用的
provider/resource 证据边界。

## 经过验证的完整案例

真实的 [WI-663 归档 Outcome](../../.ai/work-items/archive/WI-663-wi659-outcome-trust-replacement.outcome.json)
是同一路线的紧凑示例。它的输入是在声明的 base 和边界范围内进行 Outcome 展示层修复。
归档记录报告 `state=finish_ready`、`decisionState=green` 和 `verification.status=verified`。
独立的[关闭决定](../../.ai/decisions/WI-663-wi659-outcome-trust-replacement.close.json)记录了
repository owner 的批准，[收尾 receipt](../../.ai/decisions/WI-663-wi659-outcome-trust-replacement.finalize.json)
记录了 provider 合并和准确 cleanup 事实。这些是不同事实：验证通过不会变成批准，批准也不会变成发布声明。

[任务报告](../../.ai/work-items/archive/WI-663-wi659-outcome-trust-replacement.task-report.md)和
[验证证据](../../.ai/evidence/WI-663-wi659-outcome-trust-replacement.verification.json)说明了证据边界。
这些检查支持已声明的 repository 和 Work Item 结果，但不能证明普遍安全、用户可见收益，或历史证据在每个
后续 Runtime 下都保持新鲜。剩余未知项 `user_visible_benefit_not_declared` 被有意保留。

要从 checkout 查看历史交接，请使用 [AI Cockpit 首页](../../README.zh-CN.md)中展示的只读查询。
当前 Runtime 可能把历史证据标记为需要重新验证；这是新鲜度限制，不是当前测试失败。如果要依据该记录
作出当前决定，应重新验证并由人明确决定；不能从归档的 green 验证通过推导授权。

[标准采用指南](standard-adoption-guide.zh-CN.md) | [English](first-work-item.md) | [日本語](first-work-item.ja.md)
