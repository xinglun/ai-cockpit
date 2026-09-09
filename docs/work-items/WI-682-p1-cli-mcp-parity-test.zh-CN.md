---
author: AI Cockpit maintainers
title: "WI-682——P1 CLI/MCP Outcome 一致性测试"
description: "补齐 WI-681 指出的不变量5缺口:新增一个集成测试,启动真实 CLI 二进制并调用生产环境 MCP handler 针对同一仓库,断言二者的 Outcome 表示完全一致。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-682-p1-cli-mcp-parity-test
status: implemented
authority: authorized
lastVerifiedBy: WI-682-p1-cli-mcp-parity-test
terminalArchive: .ai/work-items/archive/WI-682-p1-cli-mcp-parity-test.contract.json
terminalVerification: .ai/evidence/WI-682-p1-cli-mcp-parity-test.verification.json
terminalFinalization: .ai/decisions/WI-682-p1-cli-mcp-parity-test.finalize.json
terminalDecision: .ai/decisions/WI-682-p1-cli-mcp-parity-test.close.json
---

[English](WI-682-p1-cli-mcp-parity-test.md) · [日本語](WI-682-p1-cli-mcp-parity-test.ja.md)

# WI-682——P1 CLI/MCP Outcome 一致性测试

## 意图

补齐 WI-681 覆盖映射(`docs/reference/collaboration-invariant-coverage.md`,
在本 Work Item 提交时尚未进入默认分支;PR 待创建)中指出的不变量5缺口:
"同一事实在 CLI、MCP、摘要与完整报告中保持一致"此前没有真正的跨进程自动化
检查。本 Work Item 只新增一个集成测试,组合两个已被验证过的既有模式
(来自 `crates/cockpit-cli/tests/outcome_handoff.rs` 的子进程启动方式;来自
`crates/cockpit-mcp/tests/rpc.rs` 的进程内 MCP handler 调用方式),而不是
引入新的测试基础设施,依据仓库所有者的明确授权,继续推进 AI Cockpit 协作
语言专项。

## 边界

这是仅限测试的 Work Item。只新增一个文件:
`crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs`。不修改任何生产源码
文件,也不修改任何既有测试文件。

## 验收与生命周期

- 新测试针对同一个仓库 fixture 与同一个 Work Item,为 CLI 一侧启动真实
  `ai-cockpit` 二进制子进程,为 MCP 一侧在进程内调用
  `cockpit_mcp::handle_request_for_repo()`,并断言两者的 Outcome 表示完全
  相等。
- `cargo test --locked -p cockpit-cli --test cli_mcp_outcome_parity`、
  `cargo fmt --check`、`cargo clippy --tests -- -D warnings` 均通过。
- 遵循 `start → preflight → checkpoint → verify → finish → archive → close`;
  保留 `user_visible_benefit_not_declared`。

## 证据

- archive:`.ai/work-items/archive/WI-682-p1-cli-mcp-parity-test.contract.json`
- verification:`.ai/evidence/WI-682-p1-cli-mcp-parity-test.verification.json`
- finalization:`.ai/decisions/WI-682-p1-cli-mcp-parity-test.finalize.json`(待合并后生成)
- close:`.ai/decisions/WI-682-p1-cli-mcp-parity-test.close.json`(待合并后生成)
