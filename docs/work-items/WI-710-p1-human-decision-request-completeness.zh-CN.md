---
author: AI Cockpit maintainers
title: "WI-710——P1 人工决定请求完整性测试"
description: "补齐 WI-681 指出的不变量9缺口:扩展两个既有的 preflight 测试,断言 HumanDecisionRequest 的每个字段均非空。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-710-p1-human-decision-request-completeness
status: implemented
authority: authorized
lastVerifiedBy: WI-710-p1-human-decision-request-completeness
terminalArchive: .ai/work-items/archive/WI-710-p1-human-decision-request-completeness.contract.json
terminalVerification: .ai/evidence/WI-710-p1-human-decision-request-completeness.verification.json
terminalFinalization: .ai/decisions/WI-710-p1-human-decision-request-completeness.finalize.json
terminalDecision: .ai/decisions/WI-710-p1-human-decision-request-completeness.close.json
---

[English](WI-710-p1-human-decision-request-completeness.md) · [日本語](WI-710-p1-human-decision-request-completeness.ja.md)

# WI-710——P1 人工决定请求完整性测试

## 意图

补齐 WI-681 覆盖映射(`docs/reference/collaboration-invariant-coverage.md`,待合并)
中指出的不变量9缺口:"每个需要人工决定的问题都必须指明决定对象、影响及
恢复条件"此前没有专门断言覆盖 `HumanDecisionRequest` 的每一个字段。本
Work Item 扩展 `crates/cockpit-repository/tests/contract_preflight.rs` 中
两个已经会触发 `needs_human_confirmation` 的既有测试(一个空脚手架,一个
高风险场景覆盖门禁),只新增一个共享断言辅助函数,而不是新增测试文件或
生产代码,依据仓库所有者的明确授权,继续推进 AI Cockpit 协作语言专项。

## 边界

这是仅限测试的 Work Item。只修改一个既有文件:
`crates/cockpit-repository/tests/contract_preflight.rs`,新增一个辅助函数
与两处调用。不修改任何生产源码文件。

## 验收与生命周期

- 新辅助函数断言 `what_happened`、`why_it_matters`、`question`、
  `resume_condition`、`options`、`recommended_option`、
  `recommendation_reason` 均非空,`recommended_option` 指向某个已提供的
  选项,且每个选项的 `id`/`label`/`effect` 均非空。
- 该辅助函数被两个既有的 `needs_human_confirmation` 测试调用,覆盖两个
  独立触发的真实场景。
- `cargo test --locked -p cockpit-repository --test contract_preflight`
  (7/7)、`cargo fmt --check`、`cargo clippy --tests -- -D warnings` 均通过。
- 遵循 `start → preflight → checkpoint → verify → finish → archive → close`;
  保留 `user_visible_benefit_not_declared`。

## 证据

- archive:`.ai/work-items/archive/WI-710-p1-human-decision-request-completeness.contract.json`
- verification:`.ai/evidence/WI-710-p1-human-decision-request-completeness.verification.json`
- finalization:`.ai/decisions/WI-710-p1-human-decision-request-completeness.finalize.json`(待合并后生成)
- close:`.ai/decisions/WI-710-p1-human-decision-request-completeness.close.json`(待合并后生成)
