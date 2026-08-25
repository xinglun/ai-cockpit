---
author: AI Cockpit maintainers
title: "WI-286——Rust Agent Risk 与 checkpoint evidence 边界"
workItemId: WI-286-agent-risk-checkpoint-evidence
description: "把参考源 Agent Risk 与 checkpoint 控制迁入一个 typed、request-scoped Rust lifecycle 边界。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-286-agent-risk-checkpoint-evidence
terminalArchive: .ai/work-items/archive/WI-286-agent-risk-checkpoint-evidence.contract.json
terminalVerification: .ai/evidence/WI-286-agent-risk-checkpoint-evidence.verification.json
terminalFinalization: .ai/decisions/WI-286-agent-risk-checkpoint-evidence.finalize.bd7963be356babe9075d0f5451851b1cb12d4361b64918feb8bd1072ef85db94.json
terminalDecision: .ai/decisions/WI-286-agent-risk-checkpoint-evidence.close.json
authority: canonical
---

# WI-286——Rust Agent Risk 与 checkpoint evidence 边界

本有界 parity 批次把参考源的 Agent Risk 与 checkpoint 语义迁入 Rust
Runtime，不复制 Python 脚本、Make target 或 provider-global 配置。范围包括
typed strict `checkpointPolicy`/`checkpointEvidence`、intent/scenario 路由约束、
required verification 声明、合法 unknown 路径，以及 append-only Contract amendment
revalidation。

`before_edit` 保持不可变。验证开始后的 Contract amendment 必须记录前后 hash、
理由和被失效的 checks；resume history 会使旧 checkpoint evidence 失效。终态
转换前必须重新 preflight 和 verification。`light`、`standard`、`strict`、`release`
只是 Verification 强度 profile，不代表 Evidence Assurance。

Runtime 保留 Contract acceptance 的原语言。人类 Outcome 只本地化固定展示标签，
不会翻译治理事实。CI 集成、planner/performance、release harness 和大规模模块
拆分属于后续有界批次。

## 参考对应

| 参考责任 | Rust 边界 |
| --- | --- |
| `ai_check_agent_risk.py` | `validate_agent_risk_controls` 与 lifecycle gate 复用 |
| `ai_checkpoint.py` | typed `CheckpointPolicy`、`CheckpointEvidence` 与 `revalidate_contract_amendment` |
| intent/scenario 路由绑定 | command 执行前的 `resolve_verification_route` |
| Agent 规则静态 parity | Rust `agent_rule_parity` 回归测试 |

## 验收边界

- malformed、unknown-field、duplicate、foreign、stale、contradictory 和 symlink
  checkpoint 输入必须 fail closed；
- 缺少或失败的 required verification gate 不得进入 finish/archive；
- amendment 与 resume history 不得复用旧 evidence；
- adopter repository 继续使用显式 repository context 并保持隔离；
- 英语、简体中文、日语文档说明 semantic（而非 wire-byte）parity 边界。
