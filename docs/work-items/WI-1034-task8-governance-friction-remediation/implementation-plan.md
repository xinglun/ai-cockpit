# WI-1035 Task 8：实施与验收计划

## 执行约束

- 一个 WI、一个分支、一个 worktree；实现串行，不拆并行子 WI。代码提交用于审查分层，不表示并行实现。
- 所有改动先按本计划失败优先增加回归，再做最小实现；每个提交后显式查询 Runtime，确认新的源树快照与动作准入。不得因 HEAD 改变就推断 source identity 改变，也不得复用旧 Runtime 二进制证据。
- 首轮提交前先由 Runtime 激活并核准 WI-1035 Contract。Contract 的 Acceptance ID 使用 `A01`…`A19`，`OBS-01`…`OBS-19` 只作追踪标签；运行 Runtime validate 确认 ID 格式后才能 checkpoint。实际触及文件若超出 Contract，停止并通过 Runtime 增补范围、重新 preflight；不靠预先猜测的宽泛 scope 绕过准入。
- `inspect/status/doctor/outcome` 只读；`preflight` 是显式记录入口。失败、超时、过期、旧 Runtime 或 hosted 证据均追加保留。
- 每个阶段记录：完成/总验收项、关联 OBS、提交 SHA、Runtime digest、检查命令及结果、未决项。百分比仅按验收项数量计算；无可靠时长样本不报 ETA。当前有效授权与边界不变时继续，不重复问“是否开始”。
- 不发布、不打 tag、不升级版本。最终停在用户全面 review、决定是否 release 的边界。

## 治理摩擦完整盘点

本轮先将此前独立审查、执行过程与用户明确边界合并核对到活动 Contract 的 19 个 Acceptance ID；下表是问题域到验收项的覆盖索引，不新增 Contract 意图或权限。逐条预期、失败条件和测试入口见下方 Stage 与 OBS 矩阵。

| 治理摩擦域 | 必须覆盖的 Contract ID | 明确验收边界 |
| --- | --- | --- |
| commit/snapshot 变化使 Runtime 证据陈旧；同版本不同 Runtime 二进制容易混用 | A01, A02, A14 | 分离 source、Contract、Runtime digest 与授权；只重做必要的新鲜证据，不重复索要仍有效的授权；每个最终流程固定候选二进制 digest |
| 查询、preflight、验证和拒绝结果之间职责/状态不一致 | A03, A04, A05, A07, A08 | 查询只读；preflight 显式且幂等；各入口共享版本化准入/下一动作；失败保留原执行与诊断 |
| required check 缺失或实现支持文件太晚才暴露 | A06, A11 | spawn 前核对完整覆盖；同 intent 的必要 scope 可治理增补，新意图/权限不得借 amendment 偷渡 |
| Agent 看不到能力或把可选并行误当串行禁用；只展示未实际验收 | A09, A16 | manifest、MCP schema、指南之后立即跑真实多进程/多 worktree验收；无 parallel 声明仍可串行，未满足并行条件则拒绝 |
| 长执行中断、诊断难读/丢失，或把文本命中误判为测试绕过 | A10, A15, A17 | spawn 前与逐节点持久化；真实中断不显示通过/可复用；摘要去重但保留原始节点证据；诊断文案与真实绕过分别验收 |
| merge/close lineage、三语投影、平台 gate 与 hosted 证据不能互相证明 | A12, A13, A18 | staged candidate 与 receipt/parity 精确绑定；Windows job 真正执行；close 状态、三语文档与 readyOnBase 一致 |
| 单 WI 串行治理、进度汇报和 release 边界造成重复门禁或流程漂移 | A19 | 一个 WI 串行实现与分层提交；按有证据的验收项报告；PR 独立审查、合并清理后停止在发布前，不发布/打 tag |

完整性约束：上表覆盖 A01…A19 共 19 项；不得将已有修复、历史绿色 CI 或“有命令/有字段”计作本轮验收。仓库没有 Makefile 或受支持 Make 入口，因此 canonical 路径固定为 repository-bound Runtime CLI、Cargo 和 gate manifest/CI；不能为通过通用模板而新增 Make，也不能以门禁例外代替缺少的真实证据。

## 源码与验收入口盘点

首轮实现范围以 Runtime Contract 为准。当前已定位的主路径如下；新增路径必须先经 Contract scope 检查：

| 领域 | 当前实现/协议入口 | 回归与验收入口 |
| --- | --- | --- |
| snapshot、preflight、生命周期、Outcome | `crates/cockpit-repository/src/lib.rs`、`crates/cockpit-repository/src/lifecycle.rs`、`crates/cockpit-cli/src/main.rs` | `crates/cockpit-repository/tests/recovery_decision.rs`、`contract_preflight.rs`、`status_projection.rs`、`lifecycle_entry.rs` |
| admission、coordination、并行兼容 | `crates/cockpit-repository/src/collaboration.rs`、`crates/cockpit-repository/src/coordination_store.rs`、`crates/cockpit-repository/src/lib.rs` | `crates/cockpit-repository/tests/collaboration_admission.rs`、`parallel_boundary.rs`、`intelligence.rs` |
| bounded verification、attempt、composition | `crates/cockpit-verification/src/lib.rs`、`crates/cockpit-repository/src/composition.rs`、CLI verification route | `crates/cockpit-repository/tests/verification_attempts.rs`、`composition.rs`、`verification_route.rs`；多进程验收脚本 |
| Agent interface 与 MCP | `crates/cockpit-protocol/src/lib.rs`、`crates/cockpit-agent/src/lib.rs`、`crates/cockpit-mcp/src/lib.rs`、`.ai/agent-interface.json` | interface/context 与 MCP `tools/list` tests；`agents/skills/ordinary-work-item.md` |
| 文档投影、平台 gate、CI lineage | 三语 Work Item guide/projection、`.github/workflows/ci.yml`、`tests/ci/repository_gate_manifest.json` | `tests/ci/repository_gate_manifest_test.py`、`tests/ci/quality_route_test.py`、documentation/gate scripts、Windows CI |

实现前逐一打开以上模块中对应函数和 test fixture，避免仅依文件名推断行为。特别确认：公共 interface 结构拒绝未知字段；不得改全局 Agent/MCP 配置；没有 root Makefile 或受支持 Make 入口，故使用 Runtime + Cargo + canonical CI manifest。

## 阶段、顺序与提交边界

### Stage 0 — Contract 激活与基线锁定（不改生产代码）

1. 完成 Runtime recovery scaffold 的 WI-1035 Contract：明确意图、19 条 Acceptance ID 与 OBS 映射、受控路径、验收场景、必需证据、CI 命令、停止边界和 predecessor lineage；确认 WI-1034 已被精确保留为 replaced/not_verified。
2. Runtime fresh inspect/status/doctor/agent doctor；按当前能力执行 start/preflight/checkpoint。保留旧失败与历史 recovery/retirement receipt。
3. 对照主线 source 与 CI 事实复核本计划文件路径；任何新增支持文件由 Runtime 核准后加入 Contract。

退出条件：Runtime 允许修改；19/19 OBS 有映射；所需平台和 canonical gate 均有归属。否则不写实现。

### Stage 1 — 身份、快照、授权与持久化语义（审查提交 1）

覆盖 OBS-01/02/03/14。预计先检查并按 Contract 触及 `cockpit-repository` snapshot/preflight/lifecycle、CLI preflight 输出及相关 integration tests。

红测与通过条件：

- dirty source 改为同 tree commit/空 commit：source-content identity 稳定；真实 source 内容改变：依赖它的 receipt 失效。
- 有边界的用户授权在仅 snapshot 变化后仍可复用；Contract scope/authority/base/stop boundary 改变时失效并要求新决定；随后必须 fresh preflight/verification。
- inspect/status/doctor/outcome 查询前后受控文件字节和 Git diff 完全不变；重复相同 preflight 幂等并返回 changed paths；不得暗跑验证。
- 已安装版与候选版即使版本号相同，只要 binary digest 不同就明确显示身份差异并拒绝混用旧候选证据；以一个最终 candidate digest 验证完整流程。

提交前运行针对性 repository tests 与 fmt。提交后重新查询 Runtime；明确记录 HEAD、source snapshot digest、Contract digest、Runtime executable digest 四者，验证一次提交没有把内容身份误当作变更。

截至 2026-09-27 的只读 Runtime 核对：HEAD `7ace37a6a59f7057b0de1d4a4a5e579d3d987244`，source snapshot `sha256:36c62c1f2efe617b9619ecc4337e1d3e8b23c3911302f6dc6d12e7a008ad40f7`，Contract `sha256:be2f649e83c3bb672eb4e6ce94d2d38348bf32cf61a447f78f5fd9bab3860f25`，当前候选 Runtime `0.2.113 / sha256:acf5e1175bde3414240738ab9314798fda74c723d7597aa4cce767ea2bac198b`（doctor=ok、agent doctor=VERIFIED）。现有 `snapshot_digest_ignores_governance_only_commits_but_tracks_source_commits`、`empty_commit_preserves_source_identity_and_current_verification_evidence`、`snapshot_digest_is_stable_when_the_same_source_change_is_committed` 覆盖治理提交/空提交/源码变化边界；旧 verification 仍绑定 Runtime `e4fe327e…` 与 snapshot `e4f57c00…`，因此此处只记录身份现状，不将旧 receipt 计作当前通过，最终结论等待唯一的当前候选 canonical verification。

### Stage 2 — 一致准入、前置覆盖、恢复路径与 scope 早发现（审查提交 2）

覆盖 OBS-04/05/06/07/08/11。预计检查 `cockpit-repository` admission/preflight/lifecycle、CLI help/output 与对应测试。

红测与通过条件：

- status、preflight 与被拒绝命令共享同一版本化 admission/blocker/next-action 结果；同一事实不出现互相矛盾的允许/拒绝理由。
- 必需 scenario/control 证据未齐时，status 不得列出 `finish`；`work-item controls`/MCP `work_item_controls` 仅在 fresh Runtime admission 允许时写入；拒绝需回传同一 admission digest 和恢复动作。可选且尚未声明的 intent alignment 维持 unknown，不制造额外人类门禁。
- checkpoint WI 的 preflight 过期或缺失時，status 给出可执行 `run_preflight`；未刷新拒绝 verify、刷新且快照仍匹配后才允许，不增加第二次 start confirmation。
- 任一 required scenario/check/evidence 映射缺失，在 spawn 前拒绝，精确报告缺项且启动进程数为 0；不得由调用方布尔值代替覆盖证明。
- amend 的自动 revalidation 与显式 `revalidate-amendment` 语义一致，help/结果给出唯一后续动作；同一 Contract 不要求无效重复 revalidation。
- formal receipt 被拒绝时保留原 execution，并给出有界、可读的 failed predicate/unknown/snapshot/evidence 诊断；成功和拒绝均通过 verify→Outcome→status 闭环断言。
- 规划期可识别同一已批准 intent 所需的支持文件；新增 scope 与新增 intent/authority 清楚区分。缺路径在启动昂贵验证前报告，Runtime amendment/re-preflight 后才可继续。

依次跑相应 repository integration tests 与 CLI tests。记录“拒绝时零 spawn”作为验收证据。

2026-09-27 收敛复核发现 A04 一处实现遗漏：Outcome 报必需 scenario evidence 不足时，旧 Status 仍把 `finish` 列为 safe action；finish 的 controls gate 也只拒绝 error finding，未覆盖 unknown required-scenario 状态。已改为共享 finish-control gap 判定；status 在 verified-but-incomplete 时撤销 finish、推荐 `record_governance_controls`，CLI/MCP 写入口重新计算 Runtime admission，finish 拒绝回传 status admission digest；可选且未声明的 intent alignment 仍为 unknown、但不新建阻塞门禁。新增 repository regression 覆盖 preflight/status 投影一致、写 controls 后恢复 finish、拒绝不改变 scope；CLI lifecycle 与 MCP controls 分别覆盖准入成功/归档后拒绝。首轮 `cargo test --locked --workspace` 唯一失败为 `source_mutation_after_typed_verification_stales_the_receipt_and_blocks_finish` 仍断言旧的底层错误文本；按当前 A04 入口改为核对 Status 不含 `finish`、拒绝含 `evidence_stale` 与同一 admission digest。该测试定向重跑通过，随后第二轮全 workspace 通过（退出码 0）。之后全量 Clippy 首次发现 `scope_amendment` 中 Unix/Windows 两分支的参数相同，已去掉冗余条件；对应 CLI 回归和全 workspace/all-targets/all-features Clippy 重跑通过（退出码 0）。`cockpit-repository` 的 `status_projection`、`lifecycle_entry`、`governance_controls`、`scenario_matrix_next_action`、`collaboration_admission`、`recovery_decision`，`cockpit-cli` 的 `lifecycle`、`preflight`，以及 `cockpit-mcp` 的 `rpc`/`collaboration_rpc` 均通过；本轮源码快照的 canonical Runtime verification 仍待最终候选阶段，旧 receipt 不计当前通过。

### Stage 3 — 协作 admission、串行回退与 Agent 能力发现（审查提交 3）

覆盖 OBS-09/16。预计检查 `.ai/agent-interface.json`、`cockpit-protocol`、`cockpit-agent`、`cockpit-mcp`、普通 WI guide、serial/parallel admission tests 与现有 linked-worktree process acceptance。

实现边界与验收：

1. 仅用当前 manifest 已支持的 capabilities 字符串扩充发现面；不向 deny-unknown interface object 加新字段，不改全局配置。旧 `0.2.113` parser 和当前 candidate 都必须能读该 manifest。CI 的兼容运行时从不可变基线 v0.2.113（commit 73f8bd2b86338f8025ef12ba9fff15bf45ef5782）构建，不能把该输入做成可选；不跟随合并后的默认分支，以免“旧 Runtime”实际包含新能力。
2. MCP `tools/list` 必须返回对应协作工具和完整参数 schema；能力文档必须告诉 Agent 如何发现 Runtime 准入、隔离 linked worktree、登记/续租 lease、报告/去重影响、暂停/确认/安全暂停/恢复，并说明 stale generation 如何拒绝。
3. 单 WI、没有 parallel 声明时，serial readiness 明确为允许/可继续；并行请求缺少兼容声明、隔离 checkout 或 lease 时 fail closed。不能以 `compatible:false` 将可选 parallel 缺失投影成 serial 禁止。
4. 能力暴露完成后立即做独立的“发现与使用验收”，不能等到最后只检查文档存在：解析 manifest、调用 agent doctor、MCP tools/list schema 断言、按 guide 走单 WI 串行正例/并行未声明负例，并运行真实多进程、多 linked worktree 协调验收。保存实际返回的 tool names/schema、binary digest 和进程计数。
5. Agent 默认读取集保持在既有硬预算内：详细流程放在参考手册，普通 WI guide 保留最短可执行入口；运行 `tests/docs/governance_cost_baseline_test.py`，不得通过提高 `MAXIMUM_BYTES` 消除超限。

验收失败即本阶段未完成，不进入下一阶段。无权限或工具缺失须列为明确 blocker，不以模拟工具列表代替。

Stage 3 当前候选实测：`cross_wi_coordination_processes.py` 返回 `state=passed`；Runtime `0.2.113` binary digest `sha256:acf5e1175bde3414240738ab9314798fda74c723d7597aa4cce767ea2bac198b`。两 linked worktree 并发登记成功，重复 impact event 只保留一个；串行正例获准并 spawn 1 个进程，未声明并行 lease 被拒；composition 首次 1、相同输入再次 0、观察到环境变化后 1。MCP `tools/list` 返回的工具名为 `blockers`, `capability_show`, `delegated_evidence_list`, `evidence_get`, `knowledge_query`, `preflight`, `repository_observe`, `safe_actions`, `status`, `verify`, `work_item_composition`, `work_item_controls`, `work_item_coordination`, `work_item_get`, `work_item_list`, `work_item_outcome`, `work_item_parallel`, `work_item_recover`, `work_item_recover_selected_lineage`, `work_item_start`, `work_item_status`, `work_item_validate`；完整 coordination input schema 随 acceptance JSON 输出，schema digest 为 `sha256:bb20cd6825ca19f4a8be31812c4bbde3694f5de320162352a5f4039401884f5e`。gate runner 已补为将此成功输出写入 `target/repository-gate-diagnostics/conformance_cross_wi_coordination_processes.log`，并把路径/digest放入 gate report/receipt；空输出或超 32 KiB 被截断均 fail closed，CI 上传该日志。runner 有完整输出、摘要一致性和截断拒绝回归。固定旧 Runtime 同为版本 `0.2.113`、digest `sha256:0c13b850e0b844ae33d718244f04f2dd4e0a4e56f4faa10e4cffef9f28c632ab`：可读 manifest，但不暴露 coordination/composition tools；不同 digest 未被误判为候选能力。此为本地进程验收与 gate-runner 回归；最终 manifest gate 与 hosted Windows 仍待最终候选验证。

2026-09-27 用当前最终候选重新执行上述发现与使用验收：`target/release/ai-cockpit` 为 `0.2.113 / sha256:ce862280d5a21f6f89a6280fc99221549a437756523e46fc31da2ce32e0e3380`；`--legacy-binary target/legacy-runtime/release/ai-cockpit` 为同版本、不同 digest `sha256:0c13b850e0b844ae33d718244f04f2dd4e0a4e56f4faa10e4cffef9f28c632ab`。结果 `state=passed`：2 linked worktrees、2 个并发登记、重复事件去重为 1；单 WI 串行执行 1 个进程且获准，未声明并行 lease 被拒；composition 相同输入从 1 个进程复用至 0，运行环境变化后重新执行 1 个。MCP coordination schema 完整；旧 Runtime 可读 manifest 但不广告 coordination/composition tools。此候选运行的完整 stdout 由命令返回；需由 canonical gate runner 再执行并持久化诊断/receipt，才能作为正式 gate artifact。

### Stage 4 — bounded execution、耐中断记录与诊断保真（审查提交 4）

覆盖 OBS-10/15/17。预计检查 `cockpit-verification` bounded executor、composition/lifecycle attempt store、lint aggregation、test-weakening analyzer 及相关测试。

2026-09-27 scope revalidation：Contract quality report 暴露 Stage 4 实际调用的
`crates/cockpit-verification/tests/composition.rs` 与 `execution.rs` 未被初始
scope 覆盖。通过 Runtime `work-item amend` 仅追加
`crates/cockpit-verification/tests/**`；Acceptance、intent、authority 和停止边界未变。
Runtime 自动记录 amendment revalidation，并指定 `run_preflight` 为下一动作；该动作需由
包含本轮 analyzer 修正的候选 Runtime 执行。

同日复核发现 test-weakening 扫描器把文档 `spec.md` 与夹具清理/decision 断言的词面组合误判为绕过。已将测试路径识别限制到测试目录及常见测试源码命名，并将绕过判断收敛到明确可执行标记；新增配对负/正例：文档 spec 和夹具清理不触发，动态构造的 `cargo test || true` 仍触发。`cargo test --locked -p cockpit-repository --test governance_signals` 当前通过 8/8；完整 Workspace 验证也在当前候选上通过，包含原有 xit/skip 正例。候选 preflight 不再报告 `test_weakening`。后续计划文档更新会改变仓库快照，因此这些结果需在文档更新后重新绑定最终 Runtime verification。

红测与通过条件：

- 子进程 spawn 前 durable attempt envelope 绑定 WI/Contract/source/Runtime digest；各节点完成即落盘；SIGINT/强制中断后 attempt 明确 `command_interrupted`/`interrupted_owner_terminated`，持久化 signal 和清理结果，不可投影成 passed 或 reuse。新 signal 字段需提升 Composition attempt schema；旧版记录保留可读，但不得因此继续复用。
- 同一 lint 根因的人类摘要去重、可读；每个 crate/node 仍保留 raw bytes、exit status 和独立结果，不折叠不同失败。
- 诊断字符串里的 “remove”等词不构成 test weakening；真实 skip/绕过（如 xit）仍由强制 gate 拒绝，错误报告包含匹配文本与来源。

2026-09-27 A15 收敛更正：先前将 CI gate failure-code 去重回归当作 Clippy crate-node 根因去重，覆盖不足。现以 Rust `VerificationExecutionRecord` 为输入生成仅展示用 `diagnosticSummary`，CLI/MCP 返回摘要但持久化 typed receipt 不变；定向库/CLI/MCP 测试已通过，canonical workspace 与 hosted PR/Windows 证据尚待完成。

以真正 subprocess/SIGINT 验收验证持久化时间点，不能只单测分类器或调用 mock。运行对应 Rust suites、CI script tests 与必要的跨平台编译测试。

### Stage 5 — 状态投影、三语关闭、lineage 与平台 gate（审查提交 5）

覆盖 OBS-12/13/18。预计检查 lifecycle/documentation projection、英语/简中/日语模板与验收脚本、gate manifest、quality route/workflow 及 Windows-specific tests。

红测与通过条件：

- `close_pending` → terminal close 按 lifecycle 定义单向转换；Runtime 不要求把归档文档倒退写成 `in_progress`。英语/简中/日语 projection 一致；`readyOnBase` 与未关闭 archived WI 列表一致，或由 Runtime 给出明确可执行 lineage 修复。
- hosted gate 仅接受精确 staged candidate 上的有效 recovery/retirement/archive/close receipts 和三语 parity；工作区未提交 receipt 不能冒充 hosted evidence；不得生成成功 close/verification receipt。
- 每个受影响的平台测试依赖在对应 target 可用；Unix-only import 受 cfg 约束、整数类型正确、路径按 `Path` 语义比对。manifest 有精确测试行，Windows hosted job 实际执行并作为 gate evidence。

当前全仓 Runtime readiness 另有既存 lineage：`readyOnBase=false`，`unclosedArchivedWorkItems` 与 `historicalDebt` 均明确列出 `WI-1031-cross-wi-acceptance-corrections`（`legacy_missing_or_invalid_finalization`）。这与投影中的未关闭列表一致，不能伪称已修复；保留其 archive/evidence 不变，仅在 Runtime 查询证明 Task 8 的当前 PR/merge/close 被它实际阻断时再停下报告，不在本 WI 中擅自创建或恢复另一条历史 Work Item。

先运行本地 manifest/docs regression，再运行相关 Windows target checks；若仓库没有可用本地 Windows runner，以 GitHub Windows job 为必要阻塞验收，不以交叉编译替代执行测试。

### Stage 6 — 全量追踪、本地/Hosted 分段验收、独立审查与合并清理（审查提交 6 / 收敛）

先做完整性矩阵核对 OBS-01..19，每项都必须链接到：Contract criterion → test/acceptance → CI gate → Runtime evidence。已有修复若不需改码，必须引用本轮实际受保护的回归，而非测试名称或旧报告。

#### Stage 6a — 本地候选与 PR 前检查

先冻结源码、测试、文档和 gate 变更并形成最终提交；之后才从这个 committed tree 构建唯一候选 Runtime。所有本地 canonical 检查、Runtime preflight/verification 和独立审查必须绑定该候选及其精确 executable digest。CLI `verify` 在 stderr 逐个报告实际验证节点的启动与完成状态、结果和已完成百分比，stdout 保持机器可读 JSON；它是运行中的事实进度，不替代 Task/WI 百分比。Task/WI 百分比只按已验收的 Contract criteria 计算，不给 ETA。保留单 WI 串行执行能力和治理准入；进度机制不改变既有 Runtime/Contract worker 配置，并发时按收到的真实完成事件更新，不估算。

本地 canonical 检查按 Contract/manifest 执行，至少包括：

1. `cargo fmt --all -- --check`
2. `cargo test --locked --workspace`
3. `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
4. `python3 tests/ci/repository_gate_manifest_test.py`
5. release candidate `cargo build --locked --release -p cockpit-cli --bin ai-cockpit`
6. `python3 tests/acceptance/cross_wi_coordination_processes.py --binary target/release/ai-cockpit --legacy-binary target/legacy-runtime/release/ai-cockpit`（扩展后的真实并发、串行和旧 Runtime 兼容验收）
7. `bash tests/docs/documentation_acceptance.sh`
8. `bash tests/ci/governance_integrity_gate_test.sh`
9. Manifest 中所有新增/受影响 gate，包含 Windows 实际测试 job。

若 Contract canonical commands 有变化，以 Runtime 核准的命令为准，不静默替换。本地检查、A09 真实进程验收和候选 Runtime evidence 全部新鲜且独立审查通过后，Runtime 当前准入允许时创建 PR。

#### Stage 6b — PR Hosted 证据与合并准入

A12/A13 依赖精确 staged candidate 的 hosted lineage/receipt/三语 parity 和 Windows 实际执行，因此只能在 PR CI 针对确切 PR head/candidate 完成后投影为已验收；本地检查不能代替。`.github/workflows/ci.yml` 仅由 `main` push 和 PR 触发，未提供 `workflow_dispatch`；仓库也没有受支持的 hosted-snapshot/预检命令。等效流程为：先完成 6a 本地 canonical 验收和 Runtime 候选绑定，再创建 PR，由真实 PR CI 在精确 head 上执行 canonical hosted/Windows gates，随后读取其实际日志与 artifacts 并记录 Runtime evidence。A12/A13 在 Hosted 证据到达前保持 pending；它们不阻止在其余条件满足时创建 PR，但必须阻止声称完整验收、合并或 close，除非 Runtime 对确切 PR evidence 的新鲜状态明确准入。不得以本地伪造 receipt、旧 CI 记录或人工标记替代。

任何候选源码、测试、文档或 gate 改动都会使相应 snapshot/evidence 失效：回到本地受影响检查；最终树变化后重建候选、重新绑定 Runtime evidence，并让 PR CI 对新 head 运行。只在 Runtime 当前准入时独立审查、合并和精确 cleanup；本 WI 的终点仍是用户 release review 前，不创建 tag/Release。

2026-09-27 历史候选验证（均已被本轮进度实现和计划变更的源码快照取代，不得作为当前通过证据）：早先候选 Runtime digest `sha256:ce862280d5a21f6f89a6280fc99221549a437756523e46fc31da2ce32e0e3380` 绑定 source snapshot `sha256:f62c65b2870d81052e4805dfc16df49e8f3fd79dd6e1ac082c14ab11d5d39d1e`，13/13 workspace crate 通过，耗时 `342933 ms`；随后候选 Runtime digest `sha256:77ea8eef14d136876e21df6f8966fcc1412004114ca0a6b99d50a1d36bd8a0d7` 绑定 source snapshot `sha256:b8ce6ceafe7b1a94bf9278c5fcb0bcb5fec2447e34766e2967eb8906112a70539`，13/13 crate 通过，耗时 `368140 ms`。第二次运行期间没有逐节点 CLI 进度，促成本段 A19 改善。此后 CLI/测试/本计划再次变化，最终候选及其 Runtime evidence 必须重新生成。之前通过的 `cargo fmt --all -- --check`、全量 Clippy、manifest 和 A09 acceptance 仅是各自快照的历史证据；Hosted strict gate runner、PR Windows job 和最终 19 项 Runtime projection 仍待验收。

依据 Runtime 当前 safeActions 推进 PR/独立审查、合并、精确 worktree/branch cleanup 和 successor lineage 收尾；每一步都查询并保存回执。最终交付独立 Outcome，分开描述实现、CI、review、merge、cleanup、lineage 和发布状态；只停在用户 release review 前，不创建 tag/Release。

## 19 项追踪矩阵

| OBS | 阶段 | 必须留下的关键证明 |
| --- | --- | --- |
| 01 | 1 | 同 tree/空 commit 稳定，真实 source 改变失效的端到端测试 |
| 02 | 1 | snapshot 变化不重问授权；权限边界改变才重问；新鲜证据仍强制 |
| 03 | 1 | query 字节不变；preflight 幂等和 changed-paths |
| 04 | 2 | status/preflight/command admission 一致断言 |
| 05 | 2 | stale/absent preflight 的拒绝→run_preflight→新鲜 verify 正例 |
| 06 | 2 | 缺 check 映射时 spawn count = 0 |
| 07 | 2 | amend/revalidate help 和状态转换一致 |
| 08 | 2 | formal receipt reject 保留执行、细节有界、Outcome/status 同步 |
| 09 | 3 | manifest + MCP schema + 普通 guide + 真多进程/多 worktree 发现验收 |
| 10 | 4 | spawn 前 envelope、逐节点 durable、SIGINT/interrupted/cleanup 真实进程证明 |
| 11 | 2 | 未声明支持路径早发现、受控 amendment、没有 late `scope_exceeded` |
| 12 | 5 | staged candidate lineage/parity 校验；缺项 fail closed |
| 13 | 5 | 对应依赖/平台修正及 Windows hosted tests 实际运行 |
| 14 | 1 | installed/candidate digest 分离且证据绑定单一可执行文件 |
| 15 | 4 | 去重人读 lint 摘要 + 完整 per-node/raw evidence |
| 16 | 3 | serial allowed 与 parallel denied/allowed 的正负例 |
| 17 | 4 | diagnostic-text 负例与真实 test-bypass 正例都进入 gate |
| 18 | 5 | 三语 close_pending→terminal、readyOnBase/未关闭清单一致 |
| 19 | 全阶段 | 单 WI 串行阶段记录、按验收项报进度、无空洞 ETA/重复确认 |

## 进度与报告模板

以 7 个阶段分别报告，不把阶段平均值冒充全局完成率。每次更新格式：`Stage n/N：已通过 x/y 条验收（累计 z/19 个 OBS 有当前证据）；当前提交/Runtime digest；正在做；明确 blocker；ETA（只有可测依据时提供，否则未知）`。OBS 只有在本轮测试或当前 gate 证据可复核时才算通过；代码已写、旧 CI 成功、文档已加均不计为验收通过。

最终 Outcome 必须含状态、issue/blocker 数、19 项追踪状态、证据链接/摘要、未知风险、用户决策、验证结果、影响（未经证明的收益标 inference）、下一动作，并声明没有执行 release。
