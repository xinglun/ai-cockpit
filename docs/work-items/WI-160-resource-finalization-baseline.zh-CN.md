---
author: AI Cockpit maintainers
title: "WI-160——资源收尾与 branch/worktree 关闭基线"
description: "定义 reviewed PR 合并后的资源收尾边界，并用静态检查防止遗漏。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-160-resource-finalization-baseline
workItemId: WI-160-resource-finalization-baseline
---

# WI-160——资源收尾与 branch/worktree 关闭基线

## Intent

合并和 Work Item 关闭是不同事实。本 Work Item 防止在准确的 branch 或 worktree
仍然 dirty、无法识别、没有决定却被保留，或仍然存在时，把 reviewed PR 当作已经完整关闭。

## 边界

Policy baseline 如下：

```text
finalize-plan → finalize → finalize-verify → close
```

`finalize-plan` 记录准确的 branch、worktree、provider PR、合并 head、remote、default
branch 和清理意图，但不执行删除。`finalize` 只有在 identity、保护规则和 dirty 状态检查
通过后，才能处理准确的已合并 resource。`finalize-verify` 证明 default branch 已同步、
相关 worktree 干净，并且准确的本地/远程 branch 已删除。

观察失败或 provider error 必须为 `unknown`，保持 Work Item 打开以便恢复。`retain` 只在
具备 owner、理由、范围和过期/复核条件的明确且有界人类决定时允许，不能静默转成清理成功。
在 finalize 成功前 `close` 一律禁止。

Runtime `0.2.17` 尚未把这些名称作为 CLI 命令提供。本 Work Item 只增加 docs/static
policy baseline；Runtime command、receipt 和 provider 集成留给后续的独立 Runtime Work Item。

验证：`.ai/evidence/WI-160-resource-finalization-baseline.verification.json`。
归档：`.ai/work-items/archive/WI-160-resource-finalization-baseline.archive.json`。
决定：`.ai/decisions/WI-160-resource-finalization-baseline.close.json`。

## In scope

- 三语 `agent-workflow` 和 `reference-parity` 契约文本。
- `tests/workflow/resource_finalization_policy.sh` 及其 test wrapper 静态/回归 gate。
- 说明该边界及 Runtime pending 状态的三语 Work Item 文档。

## Out of scope

- Runtime source 或 `crates/**` 修改。
- provider 侧 branch 删除、GitHub workflow 或全局 Agent/MCP 配置。
- 删除或修改现有 branch 与 worktree。

## Acceptance

1. 三语 workflow 页面都要求 `finalize-plan`、`finalize`、`finalize-verify`，保留
   `unknown`/`retain` 语义，禁止静默删除和清理前 close，并标明 Runtime 集成 pending。
2. 三语 parity 页面描述相同的 Partial 边界，不声称 Runtime 已提供这些命令。
3. repository 静态 gate 通过；从任一语言页面移除必要收尾规则时，测试必须失败。
4. 修改限定在 `docs/` 与 `tests/`，不手动编辑 Runtime source 或生成的 governance receipt。

## Verification

运行 `tests/workflow/resource_finalization_policy_test.sh` 和文档 acceptance gate。Runtime
lifecycle evidence 会绑定本 Contract 与 verification receipt；CLI 集成属于后续 Work Item。
