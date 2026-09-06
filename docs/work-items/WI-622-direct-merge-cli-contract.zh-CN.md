---
author: AI Cockpit maintainers
title: "WI-622——Direct-merge recovery 计划/应用契约"
description: "确保第一条 direct-merge recovery 计划可以通过两个 CLI 应用入口执行。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-622-direct-merge-cli-contract
status: implemented
authority: canonical
lastVerifiedBy: WI-622-direct-merge-cli-contract
terminalArchive: .ai/work-items/archive/WI-622-direct-merge-cli-contract.contract.json
terminalVerification: .ai/evidence/WI-622-direct-merge-cli-contract.verification.json
terminalFinalization: .ai/decisions/WI-622-direct-merge-cli-contract.finalize.json
terminalDecision: .ai/decisions/WI-622-direct-merge-cli-contract.close.json
---

# WI-622——Direct-merge recovery 计划/应用契约

## 意图

让 Runtime 的第一条 direct-merge recovery 计划成为可执行契约。完整的
`suggestedReceipt` 只补充列出的人工字段后，必须能够通过
`work-item finalize-recovery`，并可由普通 `work-item finalize` 幂等重放；不得
虚构 PR 或编辑 Runtime 生成的仓库记录。

## 边界

本 Work Item 覆盖 Rust protocol/repository/CLI 回归契约，以及中英日命令和故障排查文档。
对象工程、release/adopter harness、源模板实现和全局 Agent/MCP 配置不在范围内。现有严格
identity、merge parent、repository 与 fail-closed 校验仍是权威。

## 验收

- 真实双 parent merge 生成带 number `0`、historical provider、merge parents 和 repository identity
  的完整 `direct_merge_no_pr` `suggestedReceipt`。
- 只补充 `humanInputRequired` 字段即可先通过 `finalize-recovery` 写入，再通过 `finalize` 幂等重放。
- 只有身份绑定正确的 receipt 才能 `finalize-verify` 和 `close`；损坏、外部仓库或矛盾输入继续拒绝。
- CLI help 及三语命令/排障文档说明相同的应用契约，以及不手改/不虚构 PR 的边界。

## 验证

```text
cargo test --locked -p cockpit-repository --test resource_finalization_transition
cargo test --locked -p cockpit-cli --test resource_finalization
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
bash tests/docs/documentation_acceptance.sh
```

另见：[English](WI-622-direct-merge-cli-contract.md) ·
[日本語](WI-622-direct-merge-cli-contract.ja.md)。
