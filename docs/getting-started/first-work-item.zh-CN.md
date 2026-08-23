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

## 实现前绑定真实审查资源

提交初始治理 bytes、推送专用 branch，并创建 draft pull request，但不要 merge。读取真实
provider 与 Git facts；不得发明 PR URL、branch、worktree、remote 或 base branch。把这些事实
写入临时 `ResourceFinalizationContext`：

```json
{
  "branch": "feature/example-change",
  "worktree": "/absolute/path/to/worktree",
  "baseBranch": "main",
  "baseRemote": "origin",
  "provider": "github",
  "pullRequest": "https://github.com/owner/repository/pull/123"
}
```

Preflight 前绑定经过审查的 context：

```bash
ai-cockpit work-item finalize-plan --repo "$repo" --id "$id" --input /tmp/WI-001.finalize-context.json
ai-cockpit preflight --repo "$repo" --contract .ai/work-items/active/WI-001-example-change.contract.json
```

Preflight 若返回 `not_ready` 或 `needs_human_confirmation`，必须停止并把 review 展示给人。
`verification_pending` 只能为了收集声明的证据而继续。记录唯一 serial checkpoint，随后只实现
Contract scope：

```bash
ai-cockpit checkpoint --repo "$repo" --id "$id"
ai-cockpit verify --repo "$repo" --work-item "$id" --command cargo --args test,--workspace --workers 1
ai-cockpit finish --repo "$repo" --id "$id"
```

应运行 Contract 的工程命令；Cargo 只是一例。最后一次编辑后，verification 必须对同一 Work
Item 与 snapshot 保持 fresh。

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

[标准采用指南](standard-adoption-guide.zh-CN.md) | [English](first-work-item.md) | [日本語](first-work-item.ja.md)
