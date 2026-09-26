# WI-1035 Task 8：实施与验收计划

## 执行约束

- 一个 WI、一个分支、一个 worktree；实现串行，不拆并行子 WI。代码提交用于审查分层，不表示并行实现。
- 所有改动先按本计划失败优先增加回归，再做最小实现；每个提交后显式查询 Runtime，确认新的源树快照与动作准入。不得因 HEAD 改变就推断 source identity 改变，也不得复用旧 Runtime 二进制证据。
- 首轮提交前先由 Runtime 激活并核准 WI-1035 Contract。Contract 的 Acceptance ID 使用 `A01`…`A19`，`OBS-01`…`OBS-19` 只作追踪标签；运行 Runtime validate 确认 ID 格式后才能 checkpoint。实际触及文件若超出 Contract，停止并通过 Runtime 增补范围、重新 preflight；不靠预先猜测的宽泛 scope 绕过准入。
- `inspect/status/doctor/outcome` 只读；`preflight` 是显式记录入口。失败、超时、过期、旧 Runtime 或 hosted 证据均追加保留。
- 每个阶段记录：完成/总验收项、关联 OBS、提交 SHA、Runtime digest、检查命令及结果、未决项。百分比仅按验收项数量计算；无可靠时长样本不报 ETA。当前有效授权与边界不变时继续，不重复问“是否开始”。
- 不发布、不打 tag、不升级版本。最终停在用户全面 review、决定是否 release 的边界。

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

### Stage 2 — 一致准入、前置覆盖、恢复路径与 scope 早发现（审查提交 2）

覆盖 OBS-04/05/06/07/08/11。预计检查 `cockpit-repository` admission/preflight/lifecycle、CLI help/output 与对应测试。

红测与通过条件：

- status、preflight 与被拒绝命令共享同一版本化 admission/blocker/next-action 结果；同一事实不出现互相矛盾的允许/拒绝理由。
- checkpoint WI 的 preflight 过期或缺失時，status 给出可执行 `run_preflight`；未刷新拒绝 verify、刷新且快照仍匹配后才允许，不增加第二次 start confirmation。
- 任一 required scenario/check/evidence 映射缺失，在 spawn 前拒绝，精确报告缺项且启动进程数为 0；不得由调用方布尔值代替覆盖证明。
- amend 的自动 revalidation 与显式 `revalidate-amendment` 语义一致，help/结果给出唯一后续动作；同一 Contract 不要求无效重复 revalidation。
- formal receipt 被拒绝时保留原 execution，并给出有界、可读的 failed predicate/unknown/snapshot/evidence 诊断；成功和拒绝均通过 verify→Outcome→status 闭环断言。
- 规划期可识别同一已批准 intent 所需的支持文件；新增 scope 与新增 intent/authority 清楚区分。缺路径在启动昂贵验证前报告，Runtime amendment/re-preflight 后才可继续。

依次跑相应 repository integration tests 与 CLI tests。记录“拒绝时零 spawn”作为验收证据。

### Stage 3 — 协作 admission、串行回退与 Agent 能力发现（审查提交 3）

覆盖 OBS-09/16。预计检查 `.ai/agent-interface.json`、`cockpit-protocol`、`cockpit-agent`、`cockpit-mcp`、普通 WI guide、serial/parallel admission tests 与现有 linked-worktree process acceptance。

实现边界与验收：

1. 仅用当前 manifest 已支持的 capabilities 字符串扩充发现面；不向 deny-unknown interface object 加新字段，不改全局配置。旧 `0.2.113` parser 和当前 candidate 都必须能读该 manifest。
2. MCP `tools/list` 必须返回对应协作工具和完整参数 schema；能力文档必须告诉 Agent 如何发现 Runtime 准入、隔离 linked worktree、登记/续租 lease、报告/去重影响、暂停/确认/安全暂停/恢复，并说明 stale generation 如何拒绝。
3. 单 WI、没有 parallel 声明时，serial readiness 明确为允许/可继续；并行请求缺少兼容声明、隔离 checkout 或 lease 时 fail closed。不能以 `compatible:false` 将可选 parallel 缺失投影成 serial 禁止。
4. 能力暴露完成后立即做独立的“发现与使用验收”，不能等到最后只检查文档存在：解析 manifest、调用 agent doctor、MCP tools/list schema 断言、按 guide 走单 WI 串行正例/并行未声明负例，并运行真实多进程、多 linked worktree 协调验收。保存实际返回的 tool names/schema、binary digest 和进程计数。

验收失败即本阶段未完成，不进入下一阶段。无权限或工具缺失须列为明确 blocker，不以模拟工具列表代替。

### Stage 4 — bounded execution、耐中断记录与诊断保真（审查提交 4）

覆盖 OBS-10/15/17。预计检查 `cockpit-verification` bounded executor、composition/lifecycle attempt store、lint aggregation、test-weakening analyzer 及相关测试。

红测与通过条件：

- 子进程 spawn 前 durable attempt envelope 绑定 WI/Contract/source/Runtime digest；各节点完成即落盘；SIGINT/强制中断后 attempt 明确 interrupted、保留 signal 和清理结果，不可投影成 passed 或 reuse。
- 同一 lint 根因的人类摘要去重、可读；每个 crate/node 仍保留 raw bytes、exit status 和独立结果，不折叠不同失败。
- 诊断字符串里的 “remove”等词不构成 test weakening；真实 skip/绕过（如 xit）仍由强制 gate 拒绝，错误报告包含匹配文本与来源。

以真正 subprocess/SIGINT 验收验证持久化时间点，不能只单测分类器或调用 mock。运行对应 Rust suites、CI script tests 与必要的跨平台编译测试。

### Stage 5 — 状态投影、三语关闭、lineage 与平台 gate（审查提交 5）

覆盖 OBS-12/13/18。预计检查 lifecycle/documentation projection、英语/简中/日语模板与验收脚本、gate manifest、quality route/workflow 及 Windows-specific tests。

红测与通过条件：

- `close_pending` → terminal close 按 lifecycle 定义单向转换；Runtime 不要求把归档文档倒退写成 `in_progress`。英语/简中/日语 projection 一致；`readyOnBase` 与未关闭 archived WI 列表一致，或由 Runtime 给出明确可执行 lineage 修复。
- hosted gate 仅接受精确 staged candidate 上的有效 recovery/retirement/archive/close receipts 和三语 parity；工作区未提交 receipt 不能冒充 hosted evidence；不得生成成功 close/verification receipt。
- 每个受影响的平台测试依赖在对应 target 可用；Unix-only import 受 cfg 约束、整数类型正确、路径按 `Path` 语义比对。manifest 有精确测试行，Windows hosted job 实际执行并作为 gate evidence。

先运行本地 manifest/docs regression，再运行相关 Windows target checks；若仓库没有可用本地 Windows runner，以 GitHub Windows job 为必要阻塞验收，不以交叉编译替代执行测试。

### Stage 6 — 全量追踪、canonical 验证、独立审查与合并清理（审查提交 6 / 收敛）

先做完整性矩阵核对 OBS-01..19，每项都必须链接到：Contract criterion → test/acceptance → CI gate → Runtime evidence。已有修复若不需改码，必须引用本轮实际受保护的回归，而非测试名称或旧报告。

本地 canonical 检查按 Contract/manifest 执行，至少包括：

1. `cargo fmt --all -- --check`
2. `cargo test --locked --workspace`
3. `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
4. `python3 tests/ci/repository_gate_manifest_test.py`
5. release candidate `cargo build --locked --release -p cockpit-cli --bin ai-cockpit`
6. `python3 tests/acceptance/cross_wi_coordination_processes.py --binary target/release/ai-cockpit`（扩展后的真实并发和串行验收）
7. `bash tests/docs/documentation_acceptance.sh`
8. `bash tests/ci/governance_integrity_gate_test.sh`
9. Manifest 中所有新增/受影响 gate，包含 Windows 实际测试 job。

若 Contract canonical commands 有变化，以 Runtime 核准的命令为准，不静默替换。随后对最终 committed tree 构建唯一候选 Runtime；用该 executable digest 运行最终 preflight/verification/status/Outcome，并确认 hosted PR quality gate 对应相同 merge candidate。候选源码再改动即回到相应阶段、重跑受影响 gate，并重新绑定 Runtime evidence。

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
