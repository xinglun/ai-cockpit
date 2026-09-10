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
lastVerifiedBy: WI-781-trust-diagnostics
capabilityClaims:
  - human_outcome_handoff
---

# 面向人的 Outcome

`ai-cockpit work-item outcome --repo <repository> --id <work-item>` 默认输出
面向人的四段式摘要。使用 `--view full` 查看完整审计交接；机器需要稳定的
`OutcomeV2` 对象时使用 `--json`。摘要和完整视图只是展示投影，不改变已保存
的 Outcome 或其治理含义。

第一行固定为 `Outcome: 🔴/🟡/🟢 ...`。例如绿色会显示为
`Outcome: 🟢 已声明的验证通过`，而不是笼统的成功；CLI stdout 与 MCP 的
`content[0].text` 都直接返回 handoff，Agent 或 UI 不得把它隐藏在折叠日志中。
`work_item_status` 是独立的只读状态投影。
归档后其生命周期阶段为 `archived`；只有 repository 绑定且已确认的 close decision
通过校验后才会变为 `closed`。缺失或无效的 decision 不能把归档提升为 `closed`。

顶层 `finish`、`archive`、`close` 保持现有 stdout 生命周期 JSON，并默认在 stderr
渲染这份相同的已校验报告。其显式 `--json` 模式为机器调用者抑制 stderr 报告。
`finish` 被阻止时，会先渲染已持久化的红色或黄色 Outcome，再返回原有 nonzero 错误；
额外 handoff 绝不会放宽门禁。CLI 无法强制宿主应用打开或展开对话 UI；宿主必须展示
stderr，人工也可以用 `ai-cockpit work-item outcome --repo <repository> --id <work-item>`
确定性重放持久交接。

默认摘要包含四个部分：

1. 结果：当前验证、生命周期、人工决定和治理信号
2. 关键变化：有证据支持的已完成事项
3. 剩余不确定性：阻断项、风险、限制、未知项和未声明的收益
4. 人的下一步：需要作出的决定及原因；无需新决定时明确说明

完整视图保留审计顺序：结果及分别列出的验证、生命周期、人工决定和治理信号、已完成内容、发现的问题、触发的停止、已解决的问题、
避免的风险、剩余风险、未知项、人工决定、验证与证据、影响、下一步。

摘要会省略不含判断相关事实的空章节；阻断项、待人工决定、无效或过期证据、历史分类和未知项不会因长度被省略。
其他列表不会静默截断，需要完整列表时使用完整视图。摘要中的声明仍是受证据约束的数据，原始证据文本不会被解释为指令或授权来源。

## 发布说明：面向阅读的 Outcome 摘要

当前展示版本将默认的人工 `work-item outcome` 视图从完整审计报告改为上述四段式摘要，减少普通任务阅读空栏目，同时保留验证、生命周期、决定、阻断项、不确定性和证据引用。
完整报告通过 `--view full` 保留；MCP `work_item_outcome` 接受 `view: "summary"`（默认）或 `view: "full"`。
机器 JSON 的既有字段、验证规则、退出码、授权语义和持久化证据保持兼容；新增的原因和 finalization 字段是可选的，旧记录可以缺少。顶层 `finish`、`archive`、`close` 为保留生命周期错误和审计上下文，继续在 stderr 输出完整 handoff；`--json` 仍会抑制该人工通道。

状态标记是决策信号，不是发布授权：

- `🟢` 已有验证证据；继续前先审阅证据。
- `🟡` 部分完成、未就绪或未知；需要修复或调查。
- `🔴` 必需控制失败；先检查展示的具体原因投影，解决实际缺口前必须停止。

空数据不会被当作正面事实。风险发现会在证据不足时显示为 `未记录` 或 `未评估`；
只有有明确证据支持的声明，才能说在指定检查范围内未发现风险。其他空章节显示为 `未记录`。
报告不会通过推断补全治理决定；绿色结果也不授权合并、发布、公开或安全性声明。

红色标记本身不是证据诊断。Runtime 从既有的 failed gate、unknowns、治理控制
发现、范围/授权规则和证据状态派生稳定原因键，区分验证失败、证据无效或过期、
验收证据缺失、意图未对齐、范围或授权缺口以及无法确定的原因。摘要和完整报告
共用这套投影；CLI/MCP 的人工交接在 `en`、`zh`、`ja` 中保持相同语义。

报告刻意分开四个维度：

- 验证状态描述 `OutcomeState`，例如“已声明的验证通过”。
- 生命周期状态描述当前的实施中、检查点、可 finish、归档或关闭投影。
- 人工决定显示“未记录”“已记录：<决定>”或“未知”，不会从验证状态推断。
- 治理信号是绿/黄/红信号，并明确不是人工批准。

有效的结构化人工决定还显示执行人、授权来源、证据和策略引用及保证级别。
记录未提供保证级别时显示“未知”；展示层不会把授权来源或证据升级成更高保证级别。

测试弱化输出限定在检查范围内。未触发弱化规则只表示所引用检查范围内未记录触发，
不证明测试未被弱化。

绿色标记只表示 Runtime 已验证一份完整、未过期且绑定当前 Work Item 与 repository 的
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
机器投影使用 `historicalStatus: "runtime_historical"`；该状态在人类 handoff 中也必须隐藏“缺少 evidence”和恢复提示。

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

对于绑定 resource context 的普通 archived Work Item，finalization 会依据
Runtime 的 receipt 校验器和资源观察进行投影。receipt 缺失、记录损坏、身份不匹配、
清理待完成、资源按计划保留和删除已验证都有不同状态与动作。事实缺失或无效时
只要求检查或重新观察，绝不会建议重复删除；保留状态不会变成删除建议；删除已
验证也只留下 Runtime 的 close 决定。人工文案不是授权来源。archived Work Item
仍须等 Runtime 接受 finalization 且显式 close decision 有效后才是终态。

Outcome 组装在一个请求级观察边界内获取 repository snapshot 及相关生命周期、
决定、证据和 finalization 记录。组装期间发现确定性变化时最多重试一次；持续变化
会明确返回 unknown/失败。纯渲染函数不执行 I/O，也不会把旧上下文延长到执行或持久化阶段。

`ai-cockpit diagnose` 报告 Runtime 内部主路径的 identity、Git/snapshot、read/hash、
parse、governance 和 projection/serialization 阶段，并报告实际范围内的读取/哈希
字节数及 Git 调用次数。不支持的子进程计数会标记为 unavailable 而不是零；基准工具
自身的开销与 Runtime 测量分开。

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

当 retry 紧跟失败的 `finish` 投影时，Runtime 通过绑定 identity 的 recovery receipt 将 active
Summary 恢复到 `checkpointed`。它不会把 blocked Outcome 变成绿色；在 archive 或 close 之前，
必须重新执行 `verify` 与 `finish` 生成新的当前 Outcome。

`finish` 会在 active outcome 旁写入 `<id>.events.jsonl`。事件流追加写入，并拒绝
malformed、foreign、疑似 secret 或关系无效的事件。归档时，Runtime 会在绑定 archive
manifest 前，将生成的报告引用和 `changedPaths` 从 `.ai/work-items/active/` 投影到对应的
`.ai/work-items/archive/` 路径；`eventsDigest` 与报告摘要覆盖投影后的归档 bytes。
`close` 校验投影后的事件流，并在 close receipt 中记录 `finalReport` 与
`finalReportDigest`。已有历史 archive bytes 永远不会被重写或回填；只有新创建的归档执行
active 到 archive 的投影。
