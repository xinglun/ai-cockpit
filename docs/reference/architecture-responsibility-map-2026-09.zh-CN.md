---
author: AI Cockpit maintainers
title: 架构职责与依赖关系图(2026-09)
description: 观察、治理、生命周期、证据、执行、展示投影各类事实当前的归属，以及 P1-P3 架构 Work Item 的事实依据。
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-652-architecture-responsibility-map
---

# 架构职责与依赖关系图(2026-09)

本文档是 2026-09 架构优化专项的 P0 交付物。它记录的是当前的职责边界——而非目标
设计——以便后续每一个 Work Item（P1-B 观察上下文、P2-A 生命周期/存储职责提取、
P2-B 状态类型、P2-C 多文件一致性、P3 物理执行边界）共享同一份有引用依据的事实
基础，而不必各自重新推导。本 Work Item 不改动任何代码。

North Star：Calibrated Human-Agent Trust。记录当前职责在哪里混杂，并不是在
断言现有行为是错的；这是在不改变治理真相的前提下降低修改耦合的前提条件。

## 单体规模

`crates/cockpit-repository/src/lib.rs` 有 18220 行——是整个工作区中体量遥遥
领先的文件。同一个 crate 还有 `governance_controls.rs`（1753 行）、
`outcome_render.rs`（639 行，已由 WI-653 纯化，本文不再重新讨论）、
`project_governance.rs`（402 行）。

## lib.rs 中的功能簇及其当前 I/O 与判定的混合情况

| 功能簇 | 大致行范围 | 是否在同一函数内混合了 I/O 与判定 |
|---|---|---|
| 仓库身份（`repository_id`、`new_repository_id`） | 728-763 | 否——纯粹的派生/读取 |
| 验证复用评估（`assess_verification_reuse*`、可执行文件身份哈希） | 763-1658 | 是——例如 `assess_verification_reuse_measured`(777) 既收集身份事实又返回复用结论 |
| Receipt 存储/capability-scoped nofollow 文件 I/O | 1660-2832 | 否——一个自成一体的存储原语层，已经与治理逻辑分离 |
| Attach / migration | 2832-3337 | 是——文件 I/O 与协议版本判定按函数混合 |
| Status / readiness（`status_with_runtime` 3343、`repository_readiness*` 3426-3542、`historical_finalization_inventory` 3832） | 3337-4209 | 是——每个函数都读取文件/git 并就地计算 readiness 结论（这一簇正是 WI-648/649 瓶颈所在） |
| 生命周期 start/checkpoint/preflight（`preflight_work_item_internal` 5107 等） | 4209-5293 | 是——preflight 读取 Contract+Summary+快照、调用治理判定辅助函数，并在同一函数内写入 preflight 决定 |
| Finish（`finish_work_item_internal` 5320） | 5293-5670 | 是——同样的读取+判定+写入混合 |
| Recovery / revalidation | 5670-6846, 16162-16451 | 是——前驱/后继绑定校验与 recovery 决定记录混在一起 |
| Evidence 留存/审计导出 | 7470-8102 | 部分——策略驱动的读+写，相对自成一体 |
| Delegated（外部）evidence 导入/列举 | 8102-8328 | 否——绑定文件，自成一体 |
| 快照摘要计算/策略解析（`source_tree_digest`、`snapshot_digest` 8548、`policy_document`、`effective_policy_for_contract`、`resolve_verification_route`、`evaluate_contract_quality_gate`） | 8328-9016 | 在这一层否——是对已捕获快照的只读操作，是真正的治理判定核心 |
| 治理判定入口（`governance_decision_for_contract*` 等） | 9349-10389 | 分支较多但集中——每个都有 2-4 个近乎重复的 `_internal`/`_with_archive`/`_with_runtime` 变体 |
| Archive（`archive_work_item_internal` 10554） | 10389-11022 | 是——与 finish 相同的读取+判定+写入混合 |
| Resource finalization（plan/record/verify/resolve-head） | 11022-13152 | 是——文件 I/O + 校验 + 链式解析混合（WI-648/649 所在区域；`resolve_resource_finalization_head_with_candidates` 现已将候选收集与链式解析分开） |
| Close（`close_work_item_with_structured_decision_internal` 13242，约 300 行） | 13152-13615 | 是——读取 Contract/Summary/archive、校验策略、校验 resource finalization、写入决定文件，全部内联在一起 |
| Knowledge / capability truth / performance diagnosis / work-item intelligence | 13906-16784 | 否——大多是只读投影 |
| Parallel slot leasing | 17001-17322 | 否——基于文件租约的互斥，自成一体 |
| Compatibility/scope relations | 17402-17720 | 否——纯函数，无 I/O，隔离良好 |
| 共享底层辅助函数（`read_json`、`atomic_json`、`atomic_write`、`observe`/`observe_cached` 17848/17957、`collect_files`） | 17770-末尾 | 否——但 `observe`/`observe_cached` 是通用的快照观察入口，其他生命周期函数常常各自独立重新推导，而不是调用它们 |

## 具体的重复读取与职责混合示例

- **`repository_id` 推导的重复**：在 lib.rs、`governance_controls.rs`、
  `project_governance.rs` 中几乎每个调用点都会从 `.ai/cockpit.toml` 重新
  计算/读取（例如 `project_governance.rs:306,352,398` 各自独立调用
  `crate::repository_id`，而不是通过 request-scoped 的上下文接收一个已解析好
  的值）。
- **`snapshot_digest` 计算的重复**：`lib.rs:8548` 定义了它；
  `project_governance.rs:305,349,395` 在加载声明时各自独立重新计算它，即使
  调用方已经持有同一请求中新鲜的快照。
- **读取+判定+持久化混在一个函数里**：`preflight_work_item_internal`(5107)、
  `finish_work_item_internal`(5320)、`archive_work_item_internal`(10554)、
  `close_work_item_with_structured_decision_internal`(13242) 都会读取
  Contract/Summary/快照文件，调用多个治理判定辅助函数，并写入最终的决定/
  outcome/archive 产物——函数体内并没有把"收集事实"、"判定"、"持久化"三个
  阶段区分开。
- **一个会写入的"校验器"**：`governance_controls.rs:1190
  record_work_item_governance_controls` 会执行写入操作，尽管该模块自己的
  头部注释描述它不生成场景或最终维度的 evidence——模块所宣称的只读校验器边界
  并未在其内部所有函数中被完全遵守。
- **依赖方向**：未发现循环依赖。`governance_controls.rs` 与
  `project_governance.rs` 回调 lib.rs 仅是为了共享原语（`ObserverError`、
  `repository_id`、`snapshot_digest`、`reject_duplicate_json_keys`）；
  lib.rs 则前向调用这两个模块。这是单向分层（lib.rs → 两个模块），而非循环——
  但这意味着，只要这两个模块依赖 lib.rs 的原语，它们目前就无法被抽取为独立的
  crate，除非 lib.rs 继续作为共享基础存在。

## 已经分离良好、可直接复用的部分

- **`RepositorySnapshot`**（`cockpit-git/src/lib.rs:212`）与
  `GitRepository::snapshot()`(268) 是一个干净的、单一的事实捕获点，已经作为
  参数传递给下层函数（`repository_readiness_from_snapshot`、
  `project_governance_projection`、`source_tree_digest`、`snapshot_digest`）。
  这是 P1-B 应该在其上构建的正确抽象；只是今天它还没有从每个入口到每个叶子
  函数都被一致地传递下去。
- **`RuntimeContext`**（`cockpit-protocol/src/lib.rs:120`）是一个干净、最小化
  的身份结构体，在每一个 `_with_runtime` 变体中都以 `&RuntimeContext` 的形式
  一致传递。
- **纯校验器**：`governance_controls.rs` 的 `required_verification_checks`、
  `validate_checkpoint_evidence_bindings`，以及 lib.rs 的
  `scope_pattern_relation`/`work_item_compatibility`(17450-17720) 已经是
  接收类型化事实作为输入、不带 I/O 或副作用地产出判定——P2-B 的状态类型工作
  应当遵循这一模板。
- **capability-scoped nofollow 文件层**(2313-2832) 是一个可复用的、自成一体的
  存储原语，已经与治理逻辑隔离——P2-A 的 evidence/存储层应当复用它，而不是
  重新构建。
- **`OutcomeRenderInput`/`outcome_render_input(_with_runtime)`**（WI-653，
  `outcome_render.rs`）是本专项希望推广的"一次观察/组装/渲染"分离模式的第一个
  具体实例。

## 物理执行与治理判定(P3 的事实依据)

`PhysicalSingleFlightCoordinator` 位于 `crates/cockpit-verification/
src/lib.rs:1473`——与所有治理判定代码
（`governance_decision_for_contract*`、`require_green_governance*`、
`evaluate_contract_quality_gate`）所在的 `cockpit-repository` 完全是不同的
crate。物理执行与治理判定之间的 crate 级别分离已经存在，P3 不需要重新发明它。
目前还没有清晰分离的是：`assess_verification_reuse*`（"这个 receipt 是否可
复用"的身份匹配判定）位于 `cockpit-repository::lib.rs:763`，而不是与它所服务的
coordinator/执行器同处 `cockpit-verification`——复用判定与物理执行/合并机制
分散在两个 crate 中，没有明确的单一边界所有者。本 Work Item 未对此做进一步
调查；这是 P3 在决定是否扩大执行共享之前应该解决的具体开放问题。

## 候选重构：问题、目标边界、兼容性风险、验证方式

### P1-B — 明确的观察上下文

**问题**：底层判定函数经常自行调用 `GitRepository::discover`/`.snapshot()`
或重新读取 Contract/Summary，而不是接收一个已经捕获好的快照，导致服务于同一
请求的两个函数可能在略微不同的时刻观察仓库。**目标边界**：入口按实际的观察
阶段（编辑前、执行后、持久化前）各捕获一次快照并向下传递；底层函数接收快照/
上下文参数，而不是重新观察。**兼容性风险**：任何目前容忍略微过时的内部重读的
函数，都需要检查调用方是否依赖那个隐式的重新检查（例如检测操作过程中的并发
修改）；如果把快照过度共享到跨越真实编辑边界的场景，将是正确性上的退化，而
不仅仅是重构。**验证**：用调用次数断言证明没有任何观察阶段被跳过，并针对
"捕获与使用之间发生并发修改"编写明确的测试。

### P2-A — 提取生命周期/存储/执行/投影职责

**问题**：`preflight_work_item_internal`、`finish_work_item_internal`、
`archive_work_item_internal`、`close_work_item_with_structured_decision_internal`
都把观察、判定、持久化嵌入在同一个函数体内。**目标边界**：按本专项建议的划分
（Observation / Governance / Lifecycle / Evidence / Execution / Projection），
一次迁移一个完整的用例——从最小的开始（例如 checkpoint）以先验证边界是否成立，
再动 finish/archive/close。**兼容性风险**：公共 API 签名、`.ai/` 文件布局、
历史记录的可读性都不能改变；任何被提取的"Port"都必须有真实的替换/故障注入
需求作为理由，而不是为每个函数都创建一个。**验证**：现有集成测试必须无变化地
通过——它们本就断言文件内容与布局，这正是这类内部重排所需要的回归网。

### P2-B — 收紧状态类型与合法转换

**问题**：生命周期状态、证据的新鲜度/适用性、治理判定、人工授权、历史/被替代
状态目前是通过许多函数中随意的字符串和布尔值来表达的，而不是一小组带有强制
合法转换的枚举。**目标边界**：复用现有枚举（`OutcomeState`、`DecisionState`
已经存在并在使用）；只在确实存在"缺失/无效/过期/不适用"被塌缩成同一个字符串
的地方才新增狭义的类型。**兼容性风险**：外部协议/JSON 表示不能在没有明确
版本/迁移路径的情况下改变；历史证据绝不能为了适配新的内部类型而被改写。
**验证**：对合法组合与非法转换做表格化测试；把历史归档记录（包括未知/旧版
schema）重放通过任何新类型，确认它们仍能作为只读历史被正确解析。

### P2-C — 多文件一致性、并发与恢复

**仅陈述问题（尚未按本专项的指示去调查是否存在缺陷，不预先假设）**：
finish/archive/close 及其恢复路径每次操作会写入多个文件；本 Work Item 未
深入审计其写入顺序、幂等性标识或中断恢复。**目标边界**：为每一种多文件操作
确定代表"已提交"的那一条记录，并确认恢复过程只会从那条权威记录重建投影。
**兼容性风险**：这里的任何修复本质上都影响范围巨大（涉及提交语义）；不能与
无关的重构捆绑在一起。**验证**：在提出任何改动之前，必须先做故障注入（在每个
写入步骤后中断、模拟写入失败、重复执行同一操作、两个进程同时推进同一 Work
Item）——遵循本专项自身"先调查再假设缺陷"的指示。

### P3 — 物理执行与治理绑定的分离

**问题**：如上所述，复用资格判定与物理执行/合并机制分散在不同的 crate 中，
没有单一的、有文档记录的边界所有者，尽管与治理判定的 crate 级别分离已经存在。
**目标边界**：确认（尚未完成）一次共享/复用的物理执行结果本身不能直接授予
某个 Work Item 的通过状态——缓存命中、执行成功、治理许可必须始终是三个分别
校验的事实。**兼容性风险**：对 `PhysicalSingleFlightCoordinator` 接线方式的
任何改动都会直接影响并发验证的正确性与仓库隔离；根据性能优化专项自身的发现，
该 coordinator 目前在生产环境中没有调用者，因此接入它是一项新的架构承诺，而
不是恢复某个被移除的行为。**验证**：在考虑扩大执行共享之前，需要针对并发
验证场景（相同身份、不同身份、失败传播、取消）做资源峰值与执行次数的断言
验证。
