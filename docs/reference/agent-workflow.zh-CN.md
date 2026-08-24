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
- review receipt 采用 append-only 方式。Contract 或 repository snapshot 变化后，新的 receipt 写入带 digest 后缀的 decision path；旧 receipt 保留为历史 evidence，绝不覆盖。`work-item recover` 记录独立且严格的 `retry`、`successor` 或 `supersede` decision，并绑定 predecessor 的 Contract/Summary/Outcome/event digest 与当前 Runtime。`supersede` 要求已绑定 successor，并将 predecessor 归档为明确的历史终态，保持原始 bytes 不变；它不会让 verification 自动变绿，也不会重写 predecessor。被替代项既不是当前成功也不是当前失败，后续由 successor 负责。
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

## 资源收尾边界

资源收尾 evidence 使用 append-only 链。canonical `<id>.finalize.json` 是不可变链根；后续 provider 观察写入 `<id>.finalize.<digest>.json`，并绑定 predecessor digest 与 sequence。`finalize-verify` 和 `close` 要求唯一线性 head；stale predecessor、fork、malformed record、symlink 或 identity drift 都会 fail closed。pre-merge blocked 链根通过连续的 merge observation（`retained`）与 cleanup（`deleted`）transition 推进。如果提交 canonical 治理 receipt 导致 PR head 前移，只有第一次 unmerged-to-merged observation 可以声明 `governanceAppendRevision`：PR、branch 与 worktree 的 head 必须同步变化，Git 必须证明旧 head 是新 head 的祖先。该追加区间可以新增同一 Work Item 的普通 finalization receipt，以及完整的 Runtime 生成 post-finalize evidence bundle；后者仅允许精确路径 `.ai/evidence/<id>/quality-route-post-finalize.json` 与 `.ai/evidence/<id>/repository-gates-post-finalize.json`。每个被接受的路径都必须是 Git `A`-only 变更，tree entry 必须是 `100644` regular blob。两个 evidence 文件必须符合固定 schema，并绑定归档 Contract、PR base、有界 head、route receipt digest、manifest digest、selected profile 和全部通过的 required gates。它们只是绑定后的观察结果，本身不授予 authority；该区间仍必须包含 finalization receipt 新增。缺少任一 bundle 文件、其他 Work Item 或文件名、malformed/duplicate-key JSON、绑定不一致、删除、修改、重命名、symlink、无关变更、非 merge 或后续 head 漂移都会被拒绝。归档 bytes 绝不重写；cleanup 必须保持已接受的 head。

合并不等于 Work Item 关闭。Hosted checks 通过后，准确的 branch 和 worktree 还必须经过
独立的资源收尾边界：

```text
finalize-plan → finalize → finalize-verify → close
```

这些就是 Runtime 提供的命令。每次调用都必须显式带 `--repo`，并提交带身份绑定的
context/receipt；Runtime 不会隐式删除资源。Work Item 只有在 verification 后才能 archive，
只有 `finalize-verify` 接受 `Deleted` 或经明确授权的 `Retained` receipt 后才能 close。
Archived verification evidence 保持为不可变的历史事实；Runtime 升级后不把它重新标记为当前
结果，而是显示为历史 evidence。新的 finalization receipt 始终绑定执行 close 的 Runtime。

## Release tag 的 transition 顺序

只有 PR 已合并且有效的 pre-merge finalization receipt 已提交后，才能创建 Release tag。
Source quality 只在该不可变 tag 的 receipt 已绑定身份，并且 Git 证明 receipt 中记录的
PR head 是 tag commit 的祖先时，才将其识别为 `awaiting_merge_close` 边界。这不会关闭
Work Item，也不会豁免清理。发布后的 binary 必须继续执行 `finalize`、`finalize-verify`
和结构化 human `close`；普通分支、无法证明的 tag 或格式错误 receipt 仍然 fail-closed。

- `finalize-plan` 记录准确的 Work Item branch/worktree、provider PR、合并 head、remote、
  default branch 和清理计划；绝不删除 branch 或 worktree。
- `finalize` 只有在 PR、head、dirty 状态和保护检查通过后，才能处理准确的已合并
  branch/worktree。禁止静默删除 branch。
- `finalize-verify` 证明 default branch 已同步、相关 worktree 干净，并且准确的本地/远程
  branch 已删除。provider 错误、identity 不匹配或观察不完整都必须是 `unknown`，保持
  Work Item 打开以便恢复，不能作为继续执行的许可。
- `retain` 必须是明确的人类决定，包含 owner、理由、范围和过期/复核条件。保留资源不能
  静默变成清理成功；除非组织 policy 明确允许有界 retain 路径，否则 `close` 必须保持阻断。
- 在 `finalize-verify` 成功（或单独授权且可审计的 retain 路径被接受）之前不得 `close`。
  任意失败都保留 retry identity，并交付可见的 yellow/red Outcome。

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
