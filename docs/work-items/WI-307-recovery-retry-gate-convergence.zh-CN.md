---
author: AI Cockpit maintainers
title: "Recovery retry 投影门禁收敛"
description: "让静态 governance-integrity 门禁与 Runtime 对过期 retry 证据的消费语义一致。"
audience:
  - maintainer
  - reviewer
workItemId: WI-307-recovery-retry-gate-convergence
status: implemented
lastVerifiedBy: WI-307-recovery-retry-gate-convergence
terminalArchive: .ai/work-items/archive/WI-307-recovery-retry-gate-convergence.contract.json
terminalVerification: .ai/evidence/WI-307-recovery-retry-gate-convergence.verification.json
terminalFinalization: .ai/decisions/WI-307-recovery-retry-gate-convergence.finalize.45784a2d6fa2092944e6e238cb7b05755f4f7a30aab55c317032d6b81207da36.json
terminalDecision: .ai/decisions/WI-307-recovery-retry-gate-convergence.close.json
authority: canonical
---

# Recovery retry 投影门禁收敛

## 意图与目标

WI-306 暴露了一个只存在于 CI 的语义不一致：Rust Runtime 在新鲜验证推进
predecessor Contract/Summary/Outcome/Events 绑定后，不再投影旧 retry；而静态
governance-integrity 门禁要求临时的 blocked Summary 标记，否则会把 retry 错误地
当作当前 recovered 终态。本 Work Item 使门禁与 Runtime 的 identity 规则一致，
同时保持 fail-closed recovery 处理，不改写历史 bytes。

## 范围与来源

- `tests/ci/governance_integrity_gate.py`
- `tests/ci/governance_integrity_gate_test.sh`
- `tests/ci/fixtures/governance-integrity`
- 三语 Agent workflow 与 command reference

依据是已安装 Rust Runtime 的 recovery 读侧校验（`load_recovery_decision`），以及
WI-306 hosted run `32978852886`：新鲜验证已经推进归档绑定时，仍报告
`docs_governance_integrity` / `missing_parity_decision`。

## 决定

只有在 predecessor digest 格式有效且已不再匹配新鲜归档记录、同时归档 Outcome
为 green 时，门禁才消费 `retry`。没有 predecessor digest 的旧 fixture 继续使用
明确的 blocked-Summary 兼容路径。invalid、foreign、malformed、ambiguous、
successor、supersede 记录仍然 fail closed。随后门禁投影真实 finalization 路径，
不虚构 recovered 终态。

不改写 Rust Runtime protocol、repository archive、Outcome、verification 或
recovery bytes。这是语义对齐，不是复制源代码或 wire format。

## 验收与验证

- 新鲜 green archive 的 stale retry 投影 `finalize` 与 `awaiting_merge_close`；
- 仍为 blocked 的 retry 保持 recovery 边界；
- successor/supersede 以及 malformed/foreign candidate 保持原有 fail-closed；
- 三语 workflow 与 command 文档说明相同规则；
- 执行 `bash tests/ci/governance_integrity_gate_test.sh`；
- 执行 `bash tests/ci/recovery_gate_acceptance.sh`；
- 执行 `bash tests/docs/documentation_acceptance.sh`；
- 执行 `cargo test --locked --workspace`。

## 边界

外部 Runtime 仍然共享，repository 状态仍然隔离。本 Work Item 只修改 repository
静态 CI 投影与文档，不增加 provider 调用、release 行为或全局 Agent/MCP 配置。
