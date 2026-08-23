---
author: AI Cockpit maintainers
title: "面向人的 Outcome"
description: "Work Item Outcome 的面向人交接结果。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: outcome-dialog-acceptance
capabilityClaims:
  - human_outcome_handoff
---

# 面向人的 Outcome

`ai-cockpit work-item outcome --repo <repository> --id <work-item>` 默认输出
面向人的交接结果。机器需要稳定的 `OutcomeV2` 对象时使用 `--json`。

第一行固定为 `Outcome: 🔴/🟡/🟢 ...`；CLI stdout 与 MCP 的
`content[0].text` 都直接返回 handoff，Agent 或 UI 不得把它隐藏在折叠日志中。
`work_item_status` 是独立的只读状态投影。
归档后其生命周期阶段为 `archived`；只有 repository 绑定且已确认的 close decision
通过校验后才会变为 `closed`。缺失或无效的 decision 不能把归档提升为 `closed`。

输出顺序为：结果和状态、已完成内容、发现的问题、触发的停止、已解决的问题、
避免的风险、剩余风险、未知项、人工决定、验证与证据、影响、下一步。

状态标记是决策信号，不是发布授权：

- `🟢` 已有验证证据；继续前先审阅证据。
- `🟡` 部分完成、未就绪或未知；需要修复或调查。
- `🔴` 必需控制失败，或权限/范围无效；必须停止并恢复。

空章节会明确显示为 `无`。报告不会通过推断补全治理决定；绿色结果也不授权
合并、发布、公开或安全性声明。

绿色只表示 Runtime 已验证一份完整、未过期且绑定当前 Work Item 与 repository 的
`evidenceSchemaVersion=2` 验证证据。证据缺失或快照过期显示为黄色；证据被篡改、
格式错误、身份不匹配或摘要不一致显示为红色。`finish`、`archive`、`close` 在
相同校验失败时会 fail closed，不会因为证据文件存在就宣称成功。旧版证据不会被
自动改写为绿色，必须重新验证生成新版证据。当前 CLI 会把执行
`verify`/`finish`/`archive`/`close` 的 Runtime `runtimeVersion` 和
`runtimeDigest` 绑定到证据；即使另一个 Runtime 生成的证据格式正确，也会被拒绝。
v2 envelope 和被保存的 receipt 会拒绝未知字段，并要求嵌套的 Work Item、repository 和
Runtime identity。`digest_only` 保留模式没有可供校验的 captured receipt。可读取的
pre-v2 记录（以缺少 `evidenceSchemaVersion` 识别）会投影为黄色
`legacy_evidence_historical`：它只是历史输入，不是当前失败，也不是新的绿色结果。
v2 记录若缺少 identity 仍然显示红色。

由旧 Runtime 生成的归档 v2 evidence 会显示黄色历史标记和
`historical_evidence_not_revalidated`。handoff 不得附加
`verification_or_human_input` 或“缺少 evidence”的恢复 gate；这些 bytes 是有效的历史上下文，
不是当前验证失败。只有需要当前结果时才重新执行 verification。

v2 envelope 的 `createdAt` 和 retention 的 `createdAt` 必须是 RFC3339 时间戳；可选的
`expiresAt` 接受 RFC3339 或保留兼容性的 epoch seconds。格式错误或语义无效的时间戳视为证据损坏，Outcome 显示红色，
`finish`、`archive`、`close` 会停止。该检查同时保护当前证据和 retention 元数据，
但不会改写历史 bytes。

验收标准、intent、scope 等字段是 Work Item owner 写入的治理原文，报告保留原文并
标注“验收标准（Contract 原文）”，不会擅自翻译或改变 Contract bytes。只有 Runtime
生成的固定标题、摘要、状态、未知项和恢复提示会按对话语言显示。

当 predecessor 有明确的 `supersede` recovery decision 时，Outcome 会包含
`historicalStatus: "superseded"` 并显示黄色历史标记。这表示原始 evidence
被保留，未作为当前结果重新验证；它不是红色失败，也不是绿色授权。

CLI 直接输出优先使用 `AI_COCKPIT_LANGUAGE`，其次使用进程 locale。Agent 对话应
使用用户当前语言。JSON 字段名和枚举值在不同语言之间保持稳定。

## MCP 面向人的 handoff

Agent 需要向人展示结果时，必须使用明确 `workItemId` 调用 repository-bound
`work_item_outcome`。其文本 content 与 CLI 使用相同的本地化 handoff，而不是原始 JSON dump。
`structuredContent.outcome` 仍是稳定的 OutcomeV2 对象；`humanHandoff` 只是 presentation projection，
不能授权 merge、release 或人工决定。`work_item_get` 仍是面向机器的记录查询。可选 `language` 用于选择
`en`、`zh` 或 `ja` 的 Runtime 标签；Contract 原文保持不变。

## Task Outcome 报告与事件

新生成的 OutcomeV2 还包含严格的 `taskOutcomeReport`。各 section 都绑定 evidence，
可以为空；空 section 不是成功声明。没有 repository-local evidence 引用的 claim 必须
带 `inference: true`。当必需控制为 yellow 或 red 时，报告包含 `failedGate` 和
`recoveryCondition`。

当 `finish` 被阻止时，active Work Item 会保留 checkpointed 生命周期状态，并写入
一个 active 的 `state: "blocked"` Outcome 投影。该投影绑定当前 repository 与 Work Item，
使用 `decisionState: "red"`，并明确失败 gate 与确定性的恢复条件。之后的有效重试只追加
完成事件，不改写之前的 blocked 事件。格式错误、外部身份、符号链接或未知类型的事件都会
fail closed。

`finish` 会在 active outcome 旁写入 `<id>.events.jsonl`。事件流追加写入，并拒绝
malformed、foreign、疑似 secret 或关系无效的事件。归档时，Runtime 会在绑定 archive
manifest 前，将生成的报告引用和 `changedPaths` 从 `.ai/work-items/active/` 投影到对应的
`.ai/work-items/archive/` 路径；`eventsDigest` 与报告摘要覆盖投影后的归档 bytes。
`close` 校验投影后的事件流，并在 close receipt 中记录 `finalReport` 与
`finalReportDigest`。已有历史 archive bytes 永远不会被重写或回填；只有新创建的归档执行
active 到 archive 的投影。
