---
author: AI Cockpit maintainers
title: "协作语言契约"
description: "将人与 Agent 的交流节点映射到既有 Runtime 事实、状态与决定的横向索引；不是第二套治理状态机。"
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-679-p0-collaboration-language-contract
---

# 协作语言契约

[English](collaboration-language-contract.md) · [日本語](collaboration-language-contract.ja.md)

北极星：Calibrated Human-Agent Trust（可校准的人机信任）。一个无法当面向维护者求助的新用户，也应当能够读懂 Agent 在每个协作节点说了什么、理解当前 Runtime 状态、作出有边界的决定，并在换到另一个 Agent 或会话后不丢失这份理解。

## 本契约是什么，不是什么

本页是一张**地图**，不是新协议。它不定义任何新的状态、标记、退出码或生命周期命令，也不改变任何既有含义。下文提到的每一项事实都已经有唯一权威来源：[`.ai/glossary.md`](../../.ai/glossary.md)、[Repository Protocol 规范](../protocol/v1/specification.md)、[面向人的 Outcome](outcome-report.md)、[如何阅读 Cockpit 状态](how-to-read-cockpit-status.md)、[Agent 工作流与评审边界](agent-workflow.md) 以及[故障排查与恢复](troubleshooting.md)。本页对某个交流节点的转述若与来源文档不一致，以来源文档为准；本页应被当作指向它们的索引，而不是替代品。

本页内容与尚未合并到 `main` 的性能专项文档（`docs/reference/performance-initiative-2026-09.*`，由尚未合并的 WI-651 后继链承载）以及架构专项文档（`docs/reference/architecture-responsibility-map-2026-09.*`，由尚未合并的 WI-652 后继链承载）均不冲突。这里提及它们仅作为合并后的未来交叉引用；本契约不依赖其内容，也不把这些草案分支当作权威。

## 每次交接都必须能回答的五个问题

无论表达语言、CLI/MCP 入口，还是发言的是哪个 Agent 或模型，任何把决定或结果交给人的交流节点，都必须让读者仅凭当次表达（而非对早前对话的记忆）回答以下五个问题：

1. **当前处于什么状态？**——来源于 `status --repo <repo> [--id <work-item>]`、当前 Summary 的 `state`，以及/或 `work-item outcome`。
2. **哪些事实已经确认，哪些仍然未知？**——`unknown` 字段本身就是一项事实，不是"大概没问题"的占位符；参见[面向人的 Outcome § 状态标记](outcome-report.md)。
3. **是否需要人工决定，为什么？**——来源于 `preflight` 的 `reviewState`，以及当其为 `needs_human_confirmation` 时附带的结构化 `humanDecisionRequest`（发生了什么、为何重要、可选项、建议、问题本身、以及恢复条件）；参见 [Agent 工作流](agent-workflow.md)。
4. **每个选项实际会做什么，其授权范围是什么？**——一个选项的效果就是它所触发的那条 Runtime 命令的效果（例如：解锁某一次 checkpoint 迁移；不证明任何测试、场景、验证或发布结果）。任何选项的描述都不能比它触发的命令更宽泛。
5. **决定之后会推进到哪里，在什么条件下会再次停止？**——下一步及其恢复条件必须逐字取自 Outcome 的 `next action` 字段或 Preflight Review，不得凭空编造。

回应可以在事实未变时省略或压缩其中某些内容的重复表述（参见[多语言语义一致性](multilingual-semantic-parity.md)关于不必每次都重复固定标题的说明），但绝不能因为省略而使这五个问题中的任何一个变得无法回答。

## 协作节点

以下每个节点都是 Agent 必须向人交付某项信息的时刻。每个节点说明：触发条件与事实来源、用户需要理解的状态、Agent 必须呈现的信息、是否需要人工决定、各选项的实际授权范围、决定后的下一步与再次停止的边界，以及信息缺失、误解或取消时的处理方式。

### 一、任务开始与范围确认

- **触发/事实来源**：`work-item new --mode <mode>` 生成的骨架，或 `start --intent --goal [--scope] [--out-of-scope]`；由此产生的 Contract `state` 及 `preflight` 的 `reviewState`。
- **需要理解的状态**：`intent`、`goal`、`scope`、`out-of-scope`、`acceptanceCriteria` 或 `authority` 任一为空的骨架，明确处于 `not_ready` / `needs_human_confirmation`；无论退出码如何，这都从未意味着"可以开始实现"。
- **必须呈现**：Runtime 已解析的既有事实（`repositoryId`、`baseRevision`、各类摘要）与仍为空的人工字段的对比；已发现的远端默认分支与基准修订版本。
- **是否需要人工决定**：是，且在实现之前始终需要——填写 intent/scope/acceptance/authority 本身就是这项决定，不存在静默默认值。
- **各选项及其范围**：仅填写 scope 只约束编辑边界，不授予 risk、authority 或 acceptance；各字段相互独立，任何一个都不能从其他字段推断得出。
- **下一步/停止边界**：`work-item new` 或 `start` 成功后进入 `preflight`；`not_ready` 或 `needs_human_confirmation` 的 preflight 结果会在此停住，直到 Contract 被修订——一个成功的建议性退出码不是继续实现的许可。
- **缺失/误解/取消的处理**：字段为空或含糊时保持 `unknown`，绝不从文件名、自然语言描述或此前会话推断；人可以通过让 Work Item 保持 `not_ready` 来取消——在 `start` 成功之前不存在清理义务。

### 二、授权请求

- **触发/事实来源**：`preflight` 返回带 `humanDecisionRequest` 的 `needs_human_confirmation`；记录了一次有边界评审的仓库本地 `decisionEvidence` 投影。
- **需要理解的状态**：一次授权请求只约束一个 Contract 摘要与一个仓库快照摘要的组合，不是对二者未来变化的常设批准。
- **必须呈现**：发生了什么、为何重要、可选项、建议、确切的问题本身，以及恢复条件——这四个结构化字段必须完整给出，不能用一句概述代替。
- **是否需要人工决定**：是。一次成功的命令或一个黄色结果本身从来都不是授权（参见 [`.ai/glossary.md`](../../.ai/glossary.md) 中的 Pause Rule）。
- **各选项及其范围**：一份记录下来的决定回执绑定 `decisionId`、Work Item、仓库、Contract 摘要、preflight 决定摘要、快照摘要、操作者、时间戳与理由；它只能解锁一次 checkpoint 迁移，此外什么都不能证明——不能证明测试结果、验证结果或发布决定。
- **复用既有授权 vs. 请求新授权 vs. 纯技术性恢复**——这是三件不同的事，不能混为一谈：
  - *复用*仅在 Receipt 的身份绑定（仓库、Work Item、Contract 摘要、快照摘要、Runtime 身份）仍然匹配时才成立；复用是规则与记录的匹配，绝不是"看起来像同一个请求"式的自然语言猜测。
  - 当 Contract 或仓库快照在此前评审之后发生变化时，*必须*请求新授权；此前的回执作为历史证据保留，新的回执写入带摘要后缀的独立路径。
  - *纯技术性恢复*（`work-item recover` 的 `retry`、`successor` 或 `supersede`）从不授予新授权，也从不会把阻断或红色结果变成绿色；它只是恢复到一个合法的生命周期位置，以便重新运行 `verify`/`finish`。
- **下一步/停止边界**：一个有效决定只解锁它所指名的那一次 checkpoint 迁移；Work Item 在 `finish` 之前仍需一次全新的 `verify`。
- **缺失/误解/取消的处理**：缺失、过期、来源不明或格式错误的回执保持停止状态；拒绝授权会让 Work Item 停留在当前安全状态，不会被强制重试。

### 三、验证通过、失败或证据不足

- **触发/事实来源**：`verify --repo <repo> --id <id>`；写入 `.ai/evidence/` 的回执；Outcome 中独立的 Verification 状态行。
- **需要理解的状态**：**验证通过与合并授权、发布授权或全面验收是不同的事实。** Outcome 刻意将 Verification、Lifecycle、Human decision 与 Governance signal 分为四条独立的行；一行绿色的 Verification 只说明所声明的检查在一份新鲜且身份绑定的回执上通过了。
- **必须呈现**：哪些声明的检查已经运行、基于哪个快照、以及由此得到的决定状态颜色；证据是当前的、旧版（`legacy_evidence_historical`）还是历史但未重新验证（`historical_evidence_not_revalidated`）——参见[面向人的 Outcome](outcome-report.md)。
- **是否需要人工决定**：常规绿色结果不需要（按你自己的流程审阅所列证据并继续）；黄色需要（调查或确认）；红色始终需要停下。
- **各选项及其范围**：越过一行绿色 Verification 继续，只授权它所列出的那些声明检查——与合并、发布或安全声明无关。黄色结果不会因为重跑一条无关命令而被修复。
- **下一步/停止边界**：绿色可以推进到 `finish`；黄色需要调查或明确决定后才能进入 `finish`；红色停止，需要满足所列的恢复条件。
- **缺失/误解/取消的处理**：被取消或中断的验证不留下回执，Work Item 保持在此前的生命周期状态；重试始终是安全的，绝不会静默复用一份不匹配的回执。

### 四、阻断与恢复

- **触发/事实来源**：一次被阻断的 `finish`（或 `checkpoint`）会持久化一份红色/黄色 Outcome 并返回其原始的非零错误；`work-item recover` 用于显式的生命周期恢复。
- **需要理解的状态**：被阻断的 Work Item 保持其已 checkpoint 的生命周期状态；阻断以 Runtime 投影出的 Outcome 形式可见，不是被静默吞掉的失败。
- **必须呈现**：失败的检查项、确定性的恢复条件，以及下一步意图是普通重试（相同 Contract、相同范围）还是恢复决定（`retry`/`successor`/`supersede`）。
- **是否需要人工决定**：修复原因后的普通重试不需要新的授权请求；`successor` 或 `supersede` 决定则是一次新的、显式的、被记录的决定，因为它改变了范围、权限或基准，或者把某个前序声明为历史状态。
- **各选项及其范围**：`retry` 恢复到合法的 `checkpointed` 位置以进行一次全新的验证尝试，不会改写此前的阻断事件；`supersede` 将前序的原始字节与证据保留为历史——它既不是当前的通过，也不是当前的失败。
- **下一步/停止边界**：重试成功后，必须由全新的 `verify` 与 `finish` 生成下一份当前 Outcome；被阻断的事件仍保留在只追加的事件流中以供审计。
- **缺失/误解/取消的处理**：含糊、来源不明或被篡改的恢复候选会被拒绝为 `recovery_decision_invalid`，不改变任何状态；不选择恢复会让 Work Item 保持阻断状态，这本身就是一种安全且可检查的状态。

### 五、合并确认

- **触发/事实来源**：一个已通过托管检查的、经过评审的 Pull Request；`work-item finalize-plan` → `finalize` → `finalize-verify` → `close`。
- **需要理解的状态**：**合并不等于 Work Item 关闭。** 一次合并只是开启了资源终结这一独立边界；分支/worktree 清理、一份 `Deleted` 终结回执，以及一次已确认的 `close` 决定，仍然都是必需的，且与仓库验证属于不同的边界。
- **必须呈现**：已合并 PR 的 head SHA、默认分支是否已同步，以及确切的剩余步骤顺序（清理分支/worktree → 终结回执 → `finalize-verify` → `close`）。
- **是否需要人工决定**：是，`close` 需要显式的 `--human-decision`；一份 `Retained` 终结回执只是一次中间观察或明确的历史事实，本身从不授权 `close`。
- **各选项及其范围**：批准 PR 只授权合并本身；不授权跳过清理，本身也不授权 `close`，更不会把本地验证追溯性地升级为托管或企业级保证（参见[如何阅读 Cockpit 状态 § 证据边界](how-to-read-cockpit-status.md)）。
- **下一步/停止边界**：`close` 之后，只有在已确认的关闭决定有效时，生命周期投影才会变为 `closed`；无效或缺失的决定绝不会把归档状态提升为 `closed`。
- **缺失/误解/取消的处理**：提供方错误、身份不匹配或不完整的终结观察结果记为 `unknown`，Work Item 保持开放以供恢复——绝不会被当作继续下一个 Work Item 的许可。

### 六、Outcome

- **触发/事实来源**：`work-item outcome --repo <repo> --id <id>`（CLI），或带显式 `workItemId` 的仓库绑定 MCP `work_item_outcome` 工具。
- **需要理解的状态**：Outcome 是终态的、可见的交接；折叠的日志行或原始的 `work_item_get` 机器记录从来都不能替代它。完整的字段顺序、标记含义与证据规则只在[面向人的 Outcome](outcome-report.md)中定义一次，此处不再重复。
- **必须呈现**：`Outcome: 🟢/🟡/🔴` 标记，以及按读者优先顺序生成的全部内容（任务结果、已完成工作、发现的问题、触发的停止、已解决的问题、风险、未知项、人工决定、验证与证据、影响、下一步）——Agent 不得重新排序、折叠或省略任何一节。
- **是否需要人工决定**：取决于标记；Outcome 本身会说明是否有待决事项以及具体是什么。
- **各选项及其范围**：Outcome 本身不引入任何新选项；它只报告事实与此前已记录的决定（如果有）。
- **下一步/停止边界**：完全以 `next action` 字段所述为准。
- **缺失/误解/取消的处理**：空白小节渲染为"未记录"或"未评估"，绝不当作正面事实；如果读者把一行绿色的 Verification 误认为发布已完成，正确的纠正方式是重新引导其查看四条分离的状态行，而不是改动 Outcome 的措辞。

### 七、Agent 或会话交接

- **触发/事实来源**：新的 Agent、新的会话，或不同的底层模型，在同一仓库与同一 Work Item 上接续工作。
- **需要理解的状态**：以上六个节点描述的一切，都应当仅凭 Runtime 本身——`status`、`work-item outcome`、当前/已归档的 Contract、Summary 与决定记录——就能重新得出，而不依赖任何对话记录。目前没有专门的 Runtime 命令能把"交接状态"打包进一次调用；接手的 Agent 需要从上述同一组 `status`/`outcome`/Contract/Summary 来源自行拼出这份状态。
- **必须呈现**：当前目标与范围边界（Contract）；已完成与待完成事项（Summary 的 `state`、`checkpointCount`）；哪些证据仍然有效及其适用范围（`.ai/evidence/` 与新鲜度绑定）；哪些授权已被记录、哪些决定仍待作出（`decisionEvidence`、`humanDecisionRequest`）；若处于阻断状态，其原因与合法的下一步（已持久化的 Outcome）。
- **是否需要人工决定**：只有当交接之前就存在一个未决决定时才需要；交接本身绝不能凭空制造一个新的决定点，也绝不能静默地了结一个旧的决定点。
- **各选项及其范围**：交接本身不产生任何新选项。
- **下一步/停止边界**：与交接之前完全一致；新的 Agent 继承的是同一个停止边界，而不是一个全新的边界。
- **缺失/误解/取消的处理**：如果某项事实无法从 Runtime 中恢复，必须将其呈现为一个需要人工回答的明确缺口，绝不能依据新 Agent 自身的习惯、某个模型特有的表达方式，或一段未提供的历史对话来推断。切换会话不得静默地把一项已记录的授权重置为"尚未授予"，也不得静默地把一项有边界的授权扩大到它从未指名的新范围。既有授权是否仍然适用，由第二节所述的身份绑定规则决定，与当前是哪个 Agent 在说话无关。

## 十条语义不变量

以下将上文各来源已经记载的保证，重新表述为可检查的规则。它们都不是新的治理规则；每一条都注明了它今天已经在何处成立，以及（在相关时）预期由哪个后续 Work Item 为其补上自动化检查。除非明确说明，否则自动化覆盖目前尚不存在；参见[当前未知事项与后续工作](#当前未知事项与后续工作)。

1. **验证通过本身不代表全面验收或获得合并授权。** 已有记载：[面向人的 Outcome](outcome-report.md)（"一个绿色结果不授权合并、发布、公开或安全声明。"）。
2. **空记录不代表不存在风险。** 已有记载：空白小节渲染为"未记录"/"未评估"，绝不作为正面发现（[面向人的 Outcome](outcome-report.md)）。
3. **未知事实不能由表达层补全。** 已有记载：`unknown` 证据从不被解读为通过（[`.ai/glossary.md`](../../.ai/glossary.md)、[Decision states](../protocol/v1/specification.md)）。
4. **历史记录不能在没有明确依据的情况下变成当前失败或当前有效证明。** 已有记载：被取代的与历史性的证据以黄色历史标记投影，绝不会被静默地当作当前结果重新验证（[面向人的 Outcome § 历史标记](outcome-report.md)）。
5. **同一事实通过 CLI、MCP、摘要与完整报告表达时不能自相矛盾。** 设计意图已记载于[多语言语义一致性](multilingual-semantic-parity.md)以及[面向人的 Outcome](outcome-report.md)的 MCP 一节（`work_item_outcome` 返回与 CLI 打印的相同本地化交接内容）；跨进程 CLI/MCP 相等性检查已由 `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs`（WI-682）实现。
6. **更换语言不能改变事实、授权范围或操作后果。** 已有记载：[多语言语义一致性](multilingual-semantic-parity.md)——JSON 字段名与枚举值保持稳定，Contract 拥有的文本从不被机器翻译。
7. **展示的下一步必须符合当前 Runtime 状态与策略。** 已有记载：下一步取自 Outcome 或 Preflight Review，绝不凭空编造（"报告从不通过推断来填补治理决定。"——[面向人的 Outcome](outcome-report.md)）。
8. **既有授权是否适用，由规则与记录决定，不能因为更换会话而被任意丢弃或扩大。** 已就 Receipt 记载（[`.ai/glossary.md`](../../.ai/glossary.md)："只有当所有已授权的身份绑定仍然匹配时才可复用"）；并在[第七节](#七agent-或会话交接)中应用于 Agent/会话交接。针对模拟会话切换的专门交接连续性检查尚未建成——列为后续工作。
9. **每一个需要人工决定的问题都必须指明决定对象、影响及恢复/恢复条件。** 已有记载：结构化的 `humanDecisionRequest` 形状（[Agent 工作流](agent-workflow.md)）。
10. **摘要可以省略细节，但绝不能隐藏阻断、关键未知项或必要的人工决定。** 已有记载：即使在 `--json` 抑制了 stderr 上的人类可读形式时，一次被阻断的 `finish` 仍会输出其已持久化的红色/黄色 Outcome；未知项也不会因压缩而被丢弃（[面向人的 Outcome](outcome-report.md)）。

## 当前未知事项与后续工作

本文档是一份仅由文档结构性检查（frontmatter 完整性、内部链接可解析性、三语言齐备性）与维护者/评审者阅读来验证的纯文档产物——**不是**由自动化语义检查验证，也不是由任何用户研究验证。它并不宣称上述每一条不变量今天都已具备跨入口的自动化强制执行；其中若干条明确尚未具备（参见第 5 与第 8 条）。以下内容被划为独立的、后续的 Work Item，而不是并入本次交付：

- 一份由状态与转换生成的场景矩阵，覆盖合法与非法转换、证据缺陷、既有授权复用与新授权请求的区分、验证结果、归档/关闭/替代/历史查询，以及跨语言、跨入口的 Agent/会话交接。
- 针对上述十条不变量的自动化检查，每条不变量都配有正例、反例与边界用例，在受控测试仓库中运行，而不是依赖固定字符串匹配。
- 一项交接完整性检查，确认新的 Agent 或会话仅凭 Runtime 本身——不依赖对话记录——就能重建目标/范围、已完成/待完成事项、有效证据及其适用范围、有效授权与待决决定，以及阻断原因。

在本文档目前所覆盖的场景范围内，它力求呈现基于上述来源、前后一致且可追溯的交流语义。它并不宣称每一位读者都会正确理解；这样的宣称需要本 Work Item 并未开展的观察性验证来支撑。
