# WI-1035 Task 8：治理摩擦收敛规格

## 目的与执行边界

在一个专用、串行执行的 Work Item 中，修复或以可重复验收证明此前跨 WI 协作交付暴露的 Runtime 身份、快照、准入、恢复、能力发现及关闭投影摩擦。目标不是再增加一层治理门禁，而是让同一份事实产生一致的动作解释，让显式用户授权在其原定边界内持续有效，同时继续要求当前快照的验证证据。

Task 8 的当前实现 WI 是 WI-1035。WI-1033 → WI-1034 → WI-1035 保持单一串行 successor 链：WI-1034 因初始验收标签不符合 Runtime 稳定 ID 规则，已按 Runtime 回执保留为 `replaced / not_verified`；WI-1035 承接相同 Task 8 范围。更早 WI-1031 → WI-1032 → WI-1033 的历史链保持不变。任何历史失败、归档或人类决策均不得改写为成功。实现不并行拆 WI、不并行改写共享 Runtime/Contract/投影表面；仅验收阶段启动真实并发进程和多个 linked worktree。

已授权的交付边界：实现、Runtime/Cargo/CI 验证、独立 PR 审查、合并、精确清理及 Runtime 允许的历史 lineage 收尾；最终停在发布确认之前。不得发布、打 tag、升级公开版本、修改全局 Agent/MCP 配置、弱化证据/安全检查，或把成功合并等同于验证通过。

## 事实基线

- 仓库：`/Users/sei-rinn/dev/workspace_rust/ai-cockpit`；WI-1035 继续使用专用分支/worktree，从同步主线 `488d5606cb67811722b2e23410dd6f66421f475b` 建立。
- 当前安装 Runtime：`0.2.113`，digest `sha256:c85632062eb5ef8f8b39f6154b765c9e2543fe6b7ca3f54e107a59c174843686`。版本相同不代表安装二进制与 PR/CI 候选二进制相同；必须逐处比较 digest。
- PR #996 已合并；合并 SHA 为 `488d5606cb67811722b2e23410dd6f66421f475b`。CI run `36242432171` 的四个工作中只有 quality 工作失败；质量路线报告 `evidence_stale` 与 `preflight_decision_evidence_invalid`，必要项 `evidence_freshness`、`human_review` 未通过。不可称为 CI 全绿。
- 此 Rust 仓库没有 root Makefile，也没有受支持的 Make 入口。治理入口是显式 `ai-cockpit ... --repo <path>`、Cargo 与 `.github/workflows/ci.yml` 调用的 repository gate manifest；不得为了满足通用模板添加 Makefile。仅在两者都不存在时才 fail closed。
- `snapshot_digest()` 已清除 HEAD/绝对路径并采用 source-tree digest；已有测试覆盖相同 source 修改提交前后稳定。仍需验证无内容变化 commit、Contract/Runtime/preflight 绑定及正式状态投影的交互，不能仅凭函数注释宣称问题已解决。
- `preflight` CLI 明确执行“evaluate and record”，当前会更新 active Summary 的决策、快照、Contract 和时间字段；它是显式写入口，不得再被描述成纯查询。`inspect/status/doctor/outcome` 则必须保持只读。相同输入反复 preflight 应幂等，并报告是否及哪些路径被写入。
- `.ai/agent-interface.json` 的当前结构对应 `AgentInterfaceManifest`，其中接口对象拒绝未知字段。能力扩展不得加入旧 Runtime 不识别的新结构字段；优先使用已存在的 `capabilities: string[]` 与 MCP `tools/list` 的真实工具 schema，并验证旧安装仍可读。

## 治理摩擦清单（完整保留 WI-1033 handoff 的 17 项）

以下 OBS-01 至 OBS-17 对应 `docs/work-items/WI-1033-cross-wi-review-fixes-recovery/implementation-plan.md` 的完整 Task 8 handoff 表；已存在的修复只在回归和端到端行为得到证明时标为保留，不重复造能力。

| ID | 已观察到的摩擦与证据 | 必须满足的结果与保护条件 |
| --- | --- | --- |
| OBS-01 | 同一 source 内容在 commit 前后是否导致快照过期曾被混为一谈。现有 `snapshot_digest_is_stable_when_the_same_source_change_is_committed` 覆盖 dirty→commit，但没有覆盖空 commit/同 tree commit 的 Runtime lifecycle 端到端表现。 | 证明同 tree commit 不改变 source identity；真实 source 变化仍使相应证据失效。分别显示 source tree、Contract、Runtime binary、preflight 与验证绑定，不以 HEAD 身份替代内容身份。 |
| OBS-02 | 显式用户授权在快照变化后被重新要求，且过期 decision binding 可令 hosted CI 报 `preflight_decision_evidence_invalid`。run `36177143896` 有对应记录。 | 区分持久、边界明确的授权与 snapshot-bound preflight/verification evidence。仅当 WI、Contract scope/authority、base 或 stop boundary 改变时才要求新的决定；快照变化必须重新评估和验证，但不得自动重问“是否开始”。 |
| OBS-03 | 2026-09-26 preflight 更新 Summary `preflightAt` 和 observer snapshot 的 `filesRead`，造成无业务输入变化的落盘差异。 | 纯查询逐字节只读。显式 preflight 写入口对不变输入幂等、列出 changed paths；不把一次请求内 observation ledger 宣称为跨进程事件传输，也不隐式运行验证/修复。 |
| OBS-04 | preflight、status、执行命令对 blocker/下一动作曾给出不一致说明。 | 使用同一版本化 admission 结果/同一 blocker 集合；status、preflight 输出和命令拒绝均提供一致的当前动作与可执行恢复入口。 |
| OBS-05 | checkpointed WI 遇到 preflight 缺失/过期时，旧 status 仅显示 `refresh_status`，可能无法回到验证。WI-1033 已加窄 `run_preflight` 提示。 | 对任意适用 WI 提供明确的 `run_preflight` 修复；刷新前拒绝验证，刷新后只有快照仍匹配才开放验证，不新增重复的 start confirmation。 |
| OBS-06 | 必需 scenario 与 test/evidence 的映射曾在长时间 canonical run 结束、finish 阶段才发现缺失。 | 昂贵子进程启动前验证 scenario→check/evidence 映射完整；缺项输出精确名称且进程数为 0；不得自动推断映射。 |
| OBS-07 | `amend` 已写 amendment revalidation checkpoint 后再运行 `revalidate-amendment` 会因未再次改 Contract 而失败，命令语义易错。 | help、成功/失败输出和下一动作明确 `amend` 是否已包含 revalidation；同一 Contract 状态不要求重复运行无效命令。 |
| OBS-08 | 成功的 aggregate execution 曾因 formal receipt 被拒绝而仅得到含糊 `formal_receipt_rejected`，难以知道哪个 Outcome predicate 失败。 | 保留原执行记录；拒绝记录有有界、可读的 projection/decision/unknown/snapshot/evidence 诊断；端到端证明 verify→Outcome→status 对拒绝和成功均一致。 |
| OBS-09 | Agent 能找到部分 CLI/MCP 协作命令，但 `.ai/agent-interface.json` 未列清协作工具，普通 WI 指南未说明何时并行及如何退回串行。 | canonical interface、实际 MCP schema 与本地任务指南可发现兼容性、linked worktree、lease、影响/暂停/恢复流程及串行回退；单 WI 串行不得依赖可选 parallel 声明。 |
| OBS-10 | 长验证被 SIGINT 后曾无本次运行的 durable attempt；仅有前一次绑定旧 HEAD 的记录。 | spawn 前写 identity-bound attempt envelope，逐节点持久化结果，进程终止时保留独立 interrupted 状态、signal 与清理结果；不完整 attempt 永不投影为 passed。 |
| OBS-11 | 已批准行为所需的共享实现文件可在实施中才触发 `scope_exceeded`，造成晚期 Contract amendment。 | spec/计划先做支持文件影响图；preflight/执行前报告缺失 scope；可证明同一 intent 的 additive path amendment 与新增 scope/authority 决定区别开。任何 `scope_exceeded` 未解决不得继续。 |
| OBS-12 | hosted gate 发现工作区未提交的 recovery/retirement/archive receipts、三语 parity 缺行和旧 WI 尚未 terminal close；本地工作区证据与 staged candidate 不一致。 | expensive hosted gates 前对精确候选树校验 lineage、三语行与 receipt 完整；未提交 Runtime 输出不能当 hosted evidence；不得合成 close 或验证成功。 |
| OBS-13 | hosted Windows 曾发现缺失 tempfile、平台整数类型、Unix-only import 和 POSIX 字符串路径断言；只在别的平台编译才暴露。 | 改动涉及的平台测试/依赖进入对应 CI job 与 gate manifest；路径比较按 Path 语义；Windows CI 执行规定的库测试，保持为合并证据。 |
| OBS-14 | 同为 `0.2.113`，安装 Runtime digest `c856…` 与 CI 候选 digest `a4eba…` 不同，导致本地验证接受、候选投影矛盾。 | 所有 preflight/recovery/verification/status/hosted gate 明确绑定执行二进制 digest；版本字符串相同不得作为兼容证明。候选身份变化时只有显式、append-only 重绑定和新鲜验证可恢复。 |
| OBS-15 | 同一个 Clippy 根因在多个依赖 crate 节点重复呈现，raw stderr 还需解码，诊断时间增加。 | 人类输出给出可读、去重的根因摘要，同时保留每节点结果、raw bytes 和不同根因；不能丢掉原始失败证据。 |
| OBS-16 | 单 WI 没有声明可选 parallel 兼容性时曾被投影为 `compatible=false`，Agent 可能误以为串行不允许。 | 分开表示 serial readiness 与 parallel compatibility；单 WI 串行正常通过；明确请求并行但缺兼容声明仍 fail closed。 |
| OBS-17 | test-weakening 词法启发式曾因诊断文本中的 “remove” 触发告警；只改错误字符串才消除。当前已有诊断文本与真实 `xit` 绕过回归。 | 保留当前负/正回归；输出匹配文本和来源；诊断字符串不能单独证明测试行为被削弱，真实绕过仍 fail closed。 |

额外在本轮直接复现/观察到的治理摩擦：

| ID | 观察 | 要求 |
| --- | --- | --- |
| OBS-18 | WI-1031 Runtime status 的 safe action 是 `close_after_review`，但 close 因三语文档 frontmatter 已是 `status: close_pending` 而要求等于 `in_progress` 被拒绝；global inspect 又同时显示 `readyOnBase=true` 和 `unclosedArchivedWorkItems=[WI-1031…]`。 | 正确建模 prearchive、close_pending、terminal projection 阶段；保留归档字节，不要求倒退成 in_progress；`readyOnBase` 与未关闭列表必须一致或解释可执行 lineage 修复。加英语/简中/日语真实 close projection 回归。 |
| OBS-19 | 用户多次追问计划共有几个 task、耗时、是否收敛、何时能合并及百分比；已明确授权仍出现重复确认门槛。 | 把“一个主 WI、串行提交/阶段”说清。进度只按计划阶段/验收项完成数报告，附证据和 blocker；ETA 无数据时标未知；授权在原边界不变时继续有效，不重复请求开始确认。 |
| OBS-20 | 后续 Agent 启动验证前没有盘点现有 formal receipt；即使 Runtime、Contract、source snapshot、候选 Runtime 和必需检查集合均未改变且已有通过证据，也可能再次启动昂贵验证。不同类别证据（尤其本地与 hosted）还可能被混用。 | 普通 WI 指南必须要求每次验证前查询 Runtime freshness 并检查现有 receipt 的 WI/repository/Contract/source/Runtime/plan/target/coverage 绑定。只有完整且 Runtime 接受为 fresh/verified 的证据才可复用；否则保留旧证据并按 Runtime admission 定向重跑。精确 PR-head hosted 结果独立判断，coverage 消费同一 formal receipt，不重复执行其 package checks。 |

## 架构与不变量

1. 保持一个 WI/分支/worktree/repository context，写入本 Contract scope；实现串行。提交拆分用于独立审查，不创建并行子 WI。
2. Runtime 是唯一 admission/current-state authority；task guide 负责引导、Outcome 负责人机交接。任何文档建议不授予操作权。
3. 分离四种身份：source-content snapshot、Contract/scope/authority、Runtime 可执行文件 digest、preflight/verification receipt。改变一项只能使依赖该项的证据失效；不能把一次新快照当成人类授权变化。
4. 查询（inspect/status/doctor/outcome/依赖观察）只读。显式 preflight 是记录决策的写命令；对重复相同输入幂等，并给出具体持久化变化。协调/影响/恢复也只能通过命名明确的写入口。
5. 新增 Agent 发现能力保持和 Runtime 0.2.113 manifest parser 向后兼容：不在 deny-unknown 的 `interfaces` 子对象增加字段；通过现有 capabilities 字符串及 MCP `tools/list` 暴露实际支持。运行时能力声明不能绕过 admission。
6. 普通单 WI 默认串行。并行必须显式请求、显式兼容、隔离 worktree 且由 Runtime 准入；没声明并行不能阻断串行。
7. 不覆盖/删除旧 attempt、失败 receipt、archive、close 或 decision；修正一律追加并绑定 predecessor digest。

## 成功判定

- OBS-01 至 OBS-20 每项均映射到已通过的回归/真实接受证据，或经证据确认无需代码修改且已有受 gate 保护的测试；没有“只改展示字段”的交付。
- agent 可从 `.ai/agent-interface.json` 和真实 MCP `tools/list` 发现协作能力、从普通 WI 指南获得使用/退回串行的方法；真实并发多进程、多 linked worktree 验收通过；真实单 WI 串行也通过。
- Runtime/Cargo 本地 canonical 检查和 GitHub PR/merge SHA 的 canonical quality gate 成功；独立 PR review、精确 cleanup、WI lineage/文档投影按 Runtime 完成；不发布、不打 tag、不升级公开版本。
- 向用户交付独立可见 Outcome，明确实现、verification、review、merge、cleanup、历史 lineage 与 release 状态各自事实。最终状态是“等待用户全面 review，发布未授权/未执行”。
