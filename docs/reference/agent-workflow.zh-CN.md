---
author: AI Cockpit maintainers
title: "Agent 工作流与评审边界"
description: "未来 AI Cockpit Work Item 继承的仓库本地操作规则。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - agent_workflow_boundaries
---

# Agent 工作流与评审边界

本文是参考源操作规则在本工程中的适用投影，保留治理意图，但使用本
Rust Runtime 与本仓库的 Protocol 词汇。

## 继承的规则

- 从仓库发现的远端 default branch 最新提交开始工作，并在 Work Item
  Contract 中记录 remote、default branch 和 base revision。
- 每个 Work Item 使用一个 Contract、一个专用 branch/worktree 和一个 PR。
  只有 scope、evidence ownership、repository context 与串行投影均隔离且
  Runtime 判定兼容时，独立 Work Item 才能并行。
- 修改前阅读 `.ai/README.md` 与 `.ai/glossary.md`，查询 `inspect`、`status`、
  `doctor`；修改不得超出声明 scope；保留测试和证据；更新 Summary；执行
  Contract 声明的工程检查。
- 如果 `preflight` 返回 `not_ready` 或 `needs_human_confirmation`，必须暂停并
  向人展示 Preflight Review；命令即使以 advisory 模式成功退出，也不代表获得了
  实施授权。
- 如果脚手架的 intent、goal、scope、out-of-scope、acceptance 或 authority 为空，
  Runtime 必须返回 `yellow` 并标记 `reviewState: needs_human_confirmation`，绝不能当作
  ready。`verification_pending` 的 yellow 只允许用于收集 Contract 已声明的证据；
  `needs_human_confirmation` 不得越过 checkpoint。
- 事前 Contract review 会绑定 repository、Work Item、Contract digest 与 snapshot digest。
  任一绑定对象变化后，必须重新 preflight 才能 checkpoint。
- 当 `reviewState` 为 `needs_human_confirmation` 时，preflight 同时返回
  `humanDecisionRequest`（发生了什么、为什么重要、可选决定、推荐项、问题和恢复条件）。
  它是面向人的请求，不是批准；只能由人补充或修订 Contract 后重新 preflight，不能由 Agent
  自行把 request 当作授权。
- 人可以且只能通过 repository-local 的 `decisionEvidence` projection 记录这个有界 review。
  严格 receipt 必须绑定 `decisionId`、Work Item、repository、Contract digest、preflight 决定 digest、
  snapshot digest、actor、时间戳和理由。有效 receipt 只允许跨过 checkpoint；它不能证明测试、scenario、
  verification 或 release 已完成。缺失、过期、foreign、格式错误或符号链接 receipt 都必须保持停止。
- review receipt 采用 append-only 方式。Contract 或 repository snapshot 变化后，新的 receipt 写入带 digest 后缀的 decision path；旧 receipt 保留为历史 evidence，绝不覆盖。`work-item recover` 记录独立且严格的 `retry` 或 `successor` decision，并绑定 predecessor 的 Contract/Summary/Outcome/event digest 与当前 Runtime；它不会让 verification 自动变绿，也不会重写 predecessor。
- 只能在实现后才能执行的高风险必需 scenario，可以在 Contract `scenarioCoverage` 中保持 `unverified`，
  但必须同时提供非空 `expected`（或 `expectedResult`）和具体 `verificationPlan`。这只是实现计划证据，
  不是完成证据；Summary scenario guard 与 `finish` 仍然要求真实执行 evidence。
- 单独交付面向人的 Outcome，并以 `Outcome: 🟢`、`Outcome: 🟡` 或
  `Outcome: 🔴` 开头，包含 unknown、evidence、人工决定和下一步。Outcome
  缺失、仅折叠显示、过期、矛盾或格式错误时必须 fail closed，不得授权继续。
- 发现属于当前 Work Item 的问题时，先修复并 amend/revalidate 当前 Contract。
  只有 scope、authority 或 base 真正不同、变更独立、无法安全在当前范围修复、
  失败交付必须重新交付，或人明确指示时，才创建 successor。
- 安装和升级验收使用不可变的公开 Release tag 与下载 binary。合并后，closure
  必须核验归档证据、decision、合并 PR head、同步后的 default branch、干净
  worktree 和精确 branch 删除。
  归档证据应依据不可变的 archive manifest 校验；不能仅因合并改变当前
  repository snapshot 就把它重新判定为 stale。任一步失败都保持可恢复的未闭合状态。

## 本工程的适配

参考源包含 `make ai-*` 命令和 `contractVersion: 2` 模板 Protocol；它们不是
本工程的命令或 schema 要求。本 Rust 工程使用已安装共享 Runtime 与显式流程：

```text
start → preflight → checkpoint → verify → finish → archive → close
```

每个 repository-bound 命令都带 `--repo`。Runtime 没有全局 current repository、
Work Item 或 project profile。Contract 条件保留其原始语言，只有面向人的表现层
负责本地化。

## Agent provider 表面

Adapter 只是把上述规则投影到仓库本地的薄层，不是第二套 policy engine。
`agent install` 必须显式执行并记录 ownership。新的 Cursor 安装使用 provider
原生的 `.cursor/rules/ai-cockpit.mdc`。如果仓库已经有受管理的
`.cursor/rules/ai-cockpit.md`，升级时保持这个 legacy target，不重命名也不覆盖
用户文件。Runtime 不会自动安装 `AGENTS.md`、`CLAUDE.md`、`GEMINI.md`、Cursor
规则，也不会修改全局 provider/MCP 配置。

生成的 managed section 与上文保持相同的 Contract-first、暂停、Summary、可见
Outcome 和 closure 语义。它只是 advisory discovery guidance；当前治理状态始终
以显式 Runtime 查询为准，provider prompt 不能授予权限。

## 安全边界

规则保持语言中立并属于仓库本地。不得写入 secret 或机器凭据，不得修改用户全局
Agent/MCP 配置，也不得把 managed Agent prompt 当作治理 authority。不得把 V1
Runtime 代码、schema、installer 或模板实现复制进本仓库。
除非人明确要求，不得回退用户变更。默认指令读取集是 `.ai/README.md`、
`.ai/glossary.md`、`AGENTS.md` 与当前机器可读治理记录；`docs/archive/**` 和参考资料
属于历史/说明内容，只有人或 Contract 明确纳入时才具有当前指导作用。status、receipt、
archive 等生成文件必须由 Runtime 生成，不得手工编辑。
参考源的 hosted-verification snapshot 例外在本 Rust 工程没有对应命令；不得把未发布的
本地 snapshot 推送出去替代经过评审的 branch/PR 流程。
