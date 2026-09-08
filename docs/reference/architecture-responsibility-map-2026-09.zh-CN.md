---
author: AI Cockpit maintainers
title: 架构职责与依赖关系图（2026-09）
description: 以当前源码证据记录观察、治理、生命周期、证据、执行和展示投影的职责边界，并为 P1-B 至 P3 提供后续调查边界。
audience:
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human:repository-owner
workItemId: WI-691-p0-responsibility-map
lastVerifiedBy: WI-691-p0-responsibility-map
---

# 架构职责与依赖关系图（2026-09）

这是 2026-09 架构优化专项的 P0 事实图。它记录当前职责和调用边界，而不是目标
实现。North Star 保持 **Calibrated Human-Agent Trust**。本 Work Item 只修改文档，
不修改 Rust 行为、wire format、治理规则或 `.ai/` 记录协议。

## 入口与当前调用链

CLI 在 `crates/cockpit-cli/src/main.rs:619-831` 解析命令并选择仓库/Runtime 适配器，
Work Item 子命令和人类 Outcome 展示在 `1208-1380` 再次分发。MCP 在
`crates/cockpit-mcp/src/lib.rs:7-21` 声明工具，在 `574-674` 校验并分发请求，并在
`1005-1055` 复用同一 Outcome 输入和渲染器。适配器不是治理事实的权威来源。

```text
CLI / MCP
  -> cockpit-repository 操作
     -> cockpit-git RepositorySnapshot 与仓库本地记录
     -> cockpit-protocol 类型化事实与治理校验器
     -> 需要命令时由 cockpit-verification 规划/执行
     -> 生命周期/证据 receipt 与 status/Outcome 投影
  -> CLI / MCP JSON 或人类可读渲染
```

`RuntimeContext` 和 `RepositoryContext` 位于
`crates/cockpit-protocol/src/lib.rs:120-131`。repository crate 在
`crates/cockpit-repository/src/lib.rs:48-92` 已导出 execution_context、evidence_store、
lifecycle、outcome_render、project_governance、status_projection 等模块。这是同一
crate 内的模块边界，不代表所有职责已经纯化。

## 职责表

| 职责 | 权威事实 | 允许的 I/O | 校验/决策/展示归属 |
|---|---|---|---|
| CLI/MCP 入口 | 解析后的参数和 Runtime 身份；CLI `619-831`，MCP `574-674` | 仅适配器输入输出，不拥有独立仓库事实 | repository 操作决策；适配器序列化或选择语言 |
| 仓库观察 | `RepositorySnapshot`：`cockpit-git/lib.rs:212-231,268-300`；identity/digest：`repository/lib.rs:536-552,2443-2663` | 观察边界内的 Git 子进程和仓库文件读取 | `observe`/`observe_cached`：`repository/lib.rs:12084-12231`；消费者校验新鲜度 |
| Runtime/仓库身份 | `RuntimeContext`、`RepositoryContext`、`.ai/cockpit.toml`、`.ai/project.json` | status projection 和 attach 的仓库读取 | protocol 类型和身份校验；不推断人工授权 |
| Contract、policy、项目声明 | protocol 中的 `Contract`、`GovernancePolicyDocument`、`ProjectGovernanceProjection`：`2603-2645,609-627,322-335` | `project_governance.rs:53-127,241-317` 读取声明；policy 解析在 `repository/lib.rs:2742-2990` | 严格解析、identity/snapshot 绑定和 unknown 由 project_governance 返回 |
| 治理校验与决策 | Contract/Summary 证据、policy、snapshot、Runtime 身份：`repository/lib.rs:3359-3555,4395-4450` | 决策 helper 读取仓库记录；`governance_controls.rs:1038-1184` 校验投影 | `required_verification_checks` 等纯校验器在 `governance_controls.rs:33-72`；preflight/治理入口记录 receipt |
| 生命周期协调 | Contract、Summary、checkpoint/verification/finalization/close 记录 | `lifecycle.rs:324-467,469-560,883-905,1087-1165`；archive/close 在 `repository/lib.rs:4504-4752,7353-7817` | 顺序和 gate 属于 lifecycle/repository 操作；存储层不能授予授权 |
| 证据存储与历史 | reusable receipt、repository/profile/node 绑定、delegated evidence 与 validity | nofollow 读写：`evidence_store.rs:36-39,225-280`；protocol 证据类型：`927-960` | receipt 校验属于 evidence/protocol；证据不是治理决策 |
| 物理执行与调度 | verification graph/plan、`PhysicalExecution`、`ExecutionResult`、Work Item receipt | `cockpit-verification/lib.rs:1206-1441,1468-1525,1595-1833` 负责进程、worker、资源预算和 single-flight | 执行只报告成功/失败；repository 另行校验证据适用性和授权 |
| Status/Outcome 投影 | `OutcomeState`、`TaskOutcomeReport`、`WorkItemStatusSnapshot`、历史/新鲜度字段：`protocol/lib.rs:3236-3505` | status 读取 config/profile、一次 Git snapshot 和记录：`status_projection.rs:3-90` | status_projection 组装机器状态；outcome_v2 组装 Outcome；投影不能授予权限 |
| 人类 Outcome 渲染 | 已校验的 `OutcomeRenderInput` 和语言 | `outcome_render.rs:70-76` 的 `render_human_outcome` 不接收仓库目录，仅格式化输入 | 渲染是展示边界；输入组装仍在同模块，属于后续收紧事项 |
| 持久化与恢复 | atomic JSON、生命周期锁、archive manifest、finalization/close 记录 | `repository/lib.rs:12033-12081`；finalization `5246-7140`；readiness/recovery `status_projection.rs:464-585` | 明确权威记录和恢复校验；投影只是可重建视图 |

## 当前重复与职责混合

1. `project_governance_projection`、`project_governance_unknowns`、
   `project_success_criteria` 分别计算 snapshot digest 和 repository ID
   (`project_governance.rs:288-317,320-383,385-401`)。这是 P1-B 的 request-scoped
   observation-context 候选，不是原子 snapshot 的证明。
2. `create_work_item_scaffold` 同时发现 Git、取得 snapshot、读取 profile、派生
   scaffold facts 并写入 Contract/Summary (`lifecycle.rs:324-467`)；
   `preflight_work_item_internal` 也在读取 Contract/snapshot 的同时评估并持久化
   决策 (`lifecycle.rs:883-905` 及其后续)。它们是带 I/O 的生命周期协调器，不是纯治理函数。
3. `governance_controls` 主要负责校验，但 `record_work_item_governance_controls`
   会按设计写入 Summary (`governance_controls.rs:1186-1250`)。必须把这个写入边界
   与只读校验区分开。
4. `render_human_outcome` 是纯函数，但 `outcome_render_input_from_outcome` 将 root
   传给 `build_outcome_render_input`，后者读取 archive 和 close decision
   (`outcome_render.rs:14-76`)；同模块还读取 lifecycle Summary 和 human decision
   (`666-705,816-875`)。因此 P1-A 已完成渲染纯化，但还没有完成投影组装的文件系统隔离。
5. repository 子模块通过 `super::*` 使用 root 的 `ObserverError`、`repository_id`、
   `snapshot_digest`。当前是 crate 内单向依赖，没有理由因为本图新增 crate 或循环 Cargo
   依赖；后续应先收窄共享 helper 依赖。

status 路径已经为完整 status projection 复用一次 Git snapshot，并避免 readiness 再取
第二次 snapshot (`status_projection.rs:50-90`)。这是一项可复用机制，但不应把 snapshot
有效期扩展到执行或持久化边界。

## 已有可复用机制

- `RepositorySnapshot`/`GitRepository::snapshot` 是观察边界：`cockpit-git/lib.rs:212-231,268-300`。
- `RuntimeContext` 是最小的执行 Runtime 绑定：`cockpit-protocol/src/lib.rs:120-124`。
- `required_verification_checks` 与 `validate_checkpoint_evidence_bindings` 接收类型化
  输入而不执行命令：`governance_controls.rs:33-72`。
- capability-scoped nofollow receipt store 已与治理逻辑分离：`evidence_store.rs:36-39,225-280`。
- 生命周期锁、单文件 atomic replacement 和 pending-index 检测已经存在：
  `repository/lib.rs:12041-12081`、`evidence_store.rs:71-100`。
- 共享 CLI/MCP 的 `OutcomeRenderInput`/renderer 是“一次组装、多个展示”的正确方向，
  但当前组装还应外移：`outcome_render.rs:14-76`。
- 物理执行拥有独立 identity/result digest，并单独绑定 Work Item receipt：
  `cockpit-verification/lib.rs:1261-1441`。

## 后续 Work Item 的有界调查

### P1-B：明确观察上下文

问题是多个入口自行获取 snapshot 或重复计算 identity/digest，可能混用不同观察时刻。
目标是按编辑前、执行后、持久化前等真实阶段建立 context，并向下传递；不能跨越真实
修改边界共享，也不能建立全局当前仓库缓存。风险是隐藏并发修改、改变 unknown 结果或
digest 语义。验证应包括请求内读取调用计数，以及文件、配置和仓库身份在观察/执行期间
变化时的停止、重试或 unknown 测试。

### P2-A：生命周期、证据、执行、投影归属

问题是模块虽已存在，但 scaffold/preflight/archive/close 仍在完整用例中混合读取、治理
检查和写入。目标是每次迁移一个完整用例：Observation 取事实，Governance 决策，Lifecycle
编排，Evidence 存储，Execution 执行，Projection 组装；Port 只在真实替换/故障注入边界
引入。风险是破坏公共 API、`.ai/` 布局、错误和历史读取。验证使用现有集成测试并增加
纯治理无 Git/文件/进程 I/O 的依赖测试。

### P2-B：状态类型与合法转换

当前 protocol 已有 `OutcomeState`、verification stage、evidence validity、finalization
state 和 `HumanDecision` (`protocol/lib.rs:368-415,559-568,927-960,962-1082,3236-3505`)，
但生命周期、证据适用性、治理、人工保证级别和历史状态仍通过字符串/可选字段连接。目标
是复用现有 enum，只为确实合并了 missing/invalid/expired/revoked/not-applicable 的位置
增加狭义类型。风险是外部 JSON 兼容和旧证据读取；禁止改写历史。验证使用合法组合/非法
转换表测试并只读重放未知/旧 schema 的历史记录。

### P2-C：多文件一致性、并发与恢复

当前已有 lifecycle lock、atomic 单文件替换、pending-index 检测、archive manifest、
finalization receipt 和 close decision 校验；本 P0 不据此推断多文件事务已经成立。目标是
为 finish/archive/close/recovery 明确提交记录、写入顺序、操作身份、冲突边界和恢复入口，
且只从权威记录重建投影。风险是提交语义的高影响变化；单文件 rename 不能证明多文件事务。
验证必须做受控故障注入：每个写入点中断、写入/空间失败、重复操作、双进程竞争、投影缺失
或损坏，并确认未完成操作不会展示为完成。

### P3：物理执行与治理绑定

`PhysicalSingleFlightCoordinator`、物理执行 identity/result、Work Item receipt 在
`cockpit-verification` (`1206-1441,1468-1525`)，治理决策和 policy gate 在
`cockpit-repository` (`repository/lib.rs:2934-3131,3359-3555`)；但 reuse eligibility
在 `execution_context.rs:14-230`，边界所有者仍是分开的。目标是把 cache hit、执行成功、
Work Item 证据绑定和治理许可保持为三个独立事实。风险是扩大 single-flight 会改变并发和
仓库隔离，不能作为纯重构顺手接入。验证应覆盖相同/不同 key、失败传播、取消、资源峰值、
执行次数、receipt 绑定以及独立治理决策。

## P0 结论与未知项

当前已经存在有价值的边界：类型化 protocol、Git snapshot、受限证据存储、生命周期锁、
有界执行以及无文件系统的最终渲染器。仍需调查的是端到端 observation context 的所有权、
每个多文件生命周期操作的提交记录，以及 verification reuse 与 physical execution 的
生产边界。这些是 P1-B、P2-A/P2-C、P3 的调查输入，本 P0 不实现它们。

