# authoritySource 单字段材料决定替换实施计划

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task when available. 原生 Contract 审阅及 checkpoint 之前不得改实现；按本线程委托由 Codex 串行执行，无子代理。

**Goal:** 经严格的 authoritySource 单字段原生修约证明，允许针对当前精确材料请求生成新决定，同时保留旧收据 invalid 语义与不可变历史。

**Architecture:** `material_review.rs` 提供内部、只读、强类型的有限替换证明，供投影和持锁 record 共用。投影只提供现有布尔动作资格；record 另外严格校验新输入，使用不可变 prepare 记录及原子 Summary 切换实现逻辑失败原子性。现有 `status_projection.rs` 不修改。

**Tech Stack:** Rust、现有 capability nofollow/bounded I/O、原生 amendment 日志、现有 immutable sidecar 与 atomic Summary writer。

**Spec:** `strict-authority-source-replacement.proposal-only.v3.json`，SHA-256 `07acccb8d7b8849c97378f2f8ef1f35657aab9b52a28b08524831c89186460fd`。

## Global Constraints

- 实现与测试仅 `crates/cockpit-repository/src/material_review.rs`、`crates/cockpit-repository/tests/material_review.rs`。
- 另有 WI-1090 三语工作项文档及三语 reference-parity 六份治理文档，Contract 合计八路径。此 scratch 计划不是生成 Contract/审批/测试证据。
- 独立 WI-1090、独立 linked worktree，从 main b8ec269 开始；c960、组合消费者 721129、消费者 Contract 420 与其原生修约链保持不动。
- 从旧收据 Contract 到当前 Contract 的唯一完整原生修约段，所有 changedValues 必须仅 `/governanceProfile/materialInspectionReview/authoritySource`；拒绝其间任何其他字段变更和改后改回。
- 旧收据仍 invalid、无 Unknown discharge；新决定必须匹配当前请求与当前有效授权。self_declared 不升级身份保证。
- 单 worker、单命令最多 900 秒、隔离 target、离线、`CARGO_INCREMENTAL=0`。本次不自动执行验证。
- 不操作 Sentinel，不接 live grant，不合并、finish/archive/关闭事项，不手改生成记录，不把现有裸 confirm_review 归给尚未生成的新绑定。

## Review Focus

1. 合法 source-only 修约夹杂 scope 或改后改回：Task 1 参数化拒绝测试覆盖整段，不只比较终态 profile。
2. 旧 sidecar/指针/历史损坏或 symlink：Task 1/2 在任何 prepare/receipt 写入前拒绝，原字节不变。
3. 投影资格与持锁 record 输入间发生源/授权/修约变更：Task 1/2 重算共同证明和实际输入，拒绝过期资格。
4. prepare/sidecar/Summary 发布各阶段中断与调用结果不明：Task 2 故障注入覆盖 pending、committed、conflict，并查实际 canonical 状态。
5. 治理记录变更、重复调用或错误 pending 输入：Task 2 只允许精确未提交事务续作；已提交 replay 拒绝，不追加重复历史。

---

### Task 1: 共同有限资格与动作投影

**Files:** Modify `crates/cockpit-repository/src/material_review.rs`; Test `crates/cockpit-repository/tests/material_review.rs`。

**Interfaces:**
- 保留 `read_material_review_receipt(...) -> Result<MaterialReviewReceiptState, ObserverError>` 当前语义。
- 新内部 `AuthoritySourceReplacementProof` 绑定旧 pointer/receipt、当前 Contract/request、完整修约尾与当前原生审阅依据。
- `finite_authority_source_replacement_proof(root: &Path, contract: &Contract, contract_digest: &Digest, request: &MaterialReviewRequest, summary_path: &Path) -> Result<Option<AuthoritySourceReplacementProof>, ObserverError>`。无资格返回 None；完整性/I/O 错误拒绝；不得匹配错误文本放行。

- [ ] 新增 `authority_source_only_native_chain_exposes_fresh_decision_without_discharge`：真实旧收据、仅 source 原生修约、当前原生 review；断言 invalid Unknown 仍在、receipt digest 为 None、discharged 为空，而当前安全动作包含精确 record。
- [ ] 运行对应 repository material_review 测试，确认现有实现因缺少该有限动作资格失败。
- [ ] 提取旧 pointer/sidecar 严格读取为内部可复用校验；只在当前 profile mismatch 情形单独构造历史证明。以原生完整 prepared/committed 日志反向恢复历史 raw Contract/profile，验证历史 digest 与收据，逐条要求唯一字段，完整 source ancestry 与当前授权检查。任何其他错误仍 fail closed。
- [ ] 在 `material_review_gate_projection_with_contract_digest` 错误分支调用有限证明；只有 Some(proof) 可令 `review_decision_available=true`。原 invalid Unknown、无收据、无 discharge、Finding 与其他 Runtime blockers 全保留。`status_projection.rs:1870` 现有 guards 消费该布尔量，不改第三个 Rust 文件。
- [ ] 参数化 `authority_source_replacement_rejects_any_other_amendment`：scope、acceptance、reviewer、assurance、risk、其他字段、混合、多次修改后改回、断链/分叉/pending/foreign/direct edit 全拒绝，无新治理写入。
- [ ] 用同一集成测试文件调用真实 status 和 record 边界，验证两层使用相同有限资格；保持既有 reviewerActor 变化拒绝测试不变。

### Task 2: 持锁 record 与可恢复逻辑原子替换

**Files:** 同两文件；私有故障注入 unit tests 留在 `src/material_review.rs`。

**Interfaces:**
- record 持既有 lifecycle lock 调同一 `finite_authority_source_replacement_proof`；普通 Missing/Current/Stale 路径行为保持。
- 新私有 `PreparedAuthoritySourceReplacement`，严格字段：schema/repo/WI/current Contract/request/profile、修约尾/当前审阅依据、旧 Summary raw digest/旧 pointer、新 input digest、原始 receipt 字节与 digest、预期新 Summary 字节与 digest、实际 runtime/time。所有字段在读回时核验，不能接受自报 proof。
- 新私有 `commit_prepared_authority_source_replacement(...) -> Result<MaterialInspectionReviewDecisionReceipt, ObserverError>` 以 Summary 为唯一 commit point；仅精确 pending 可继续。

- [ ] 新增 `authority_source_replacement_validates_new_current_input_before_publication`：错 request/reviewer/source/evidence、缺/过期/foreign review、Finding、dirty/unavailable、policy/scanner/profile mismatch、损坏旧 sidecar/pointer/history，全部在 prepare/新收据发布前拒绝，旧原字节不变。
- [ ] 运行对应测试确认失败，然后在 record 锁内重算 proof、复核全部当前请求及权限，并使用既有 typed decision validator 校验实际新输入。不能只放宽 action allow-list。
- [ ] 新增私有 `replacement_transaction_failure_matrix`：注入 prepare 发布前/后、新 sidecar 发布前/后、Summary rename 前/后失败，比较 canonical Summary、所有旧 sidecar、历史与修约链字节。
- [ ] 验证失败时不创建事务；通过全部校验后 seal 不可变 prepared，按 prepared→新 sidecar→原子 Summary 顺序发布。提交前确认旧 Summary 仍精确一致。Summary 一次切换同时追加旧 pointer 历史和安装新 pointer。
- [ ] 新增 `replacement_pending_resume_is_exact_and_committed_replay_rejects`：精确未提交 pending 使用 sealed 原始 receipt/time/runtime；先重新验证当前 proof/admission/input，再续作，不重新造收据。异 input/runtime/request、损坏/冲突 prepared 或变更 Summary 拒绝；已提交 replay 拒绝、history 只追加一次。
- [ ] 对 commit 返回不明，读 canonical：Summary/sidecar 精确为新态则已提交；精确旧态则 pending；其他态 conflict。不得把实际提交说成干净失败，也不删除 canonical 残留伪装回滚。
- [ ] 文档/诊断明确是“可恢复事务与逻辑原子性”，不声称多文件物理原子或断电后零残留。测试注入仅 cfg(test)，生产无 bypass flag/env/schema。

### Task 3: 回归、治理文档与实际验证

**Files:** 同两 Rust 文件；必要 WI-1090 三语文档及 reference-parity 三语记录。

**Interfaces:** 使用现有 Native verification、material plan/record、原生 controls，不修改 transport/schema、消费者 Contract 或固定候选。

- [ ] 补齐同 profile stale renewal、governance-only descendant、history malformed、missing/corrupt chain 既有回归；新增 sidecar symlink 与并发源/授权变更测试，检查零未授权发布。
- [ ] 更新八路径范围内的三语治理文档，明确 source-only 门槛、两层调用关系、失败/未知状态、当前真实测试结果及 release/Runtime 采用仍独立。
- [ ] 在当前原生 Contract review/checkpoint 允许后，按声明串行运行七节点：fmt、repository/CLI/MCP all-targets clippy、repository lib material_review、repository integration material_review、status_projection、CLI contract_amendment_processes、MCP material_review。仅依据实际终态记录结果；不复用旧环境或 c960 收据。
- [ ] 实际 material plan 若产生新 Unknown/风险绑定，交 Raydot 普通技术审阅；只在原生明确要求新的真人决定时展示当前完整绑定。旧 2ed9306a 审批不自动适用新材料/analysisImplementationDigest。
- [ ] 验证后正常 commit/push、Draft PR 和一次最终 CI 仍受当前 Contract/用户授权与实际 native admission 约束；不因这个计划宣称已经允许发布或采用新 Runtime。

### Task 4: 独立组合后的消费者恢复（此实现不直接执行）

- [ ] 保留 c960 和 721129 原记录，在另行准入的隔离组合中整合新实现；若包含 WI-1087 guard，执行其真实回归。
- [ ] 编译独立候选并记录实际 Runtime SHA，按原生 environment/drift/admission 采用，保留 b5 pinned Runtime。
- [ ] 消费者 Contract 420 不扩大范围；仅在当前精确 source/runtime/request/profile/policy 读回并获得实际所需决定后，原生 record 新材料决定、真实验证、报告 Outcome。不 finish/archive/merge/关闭。

## 自审

- 整段 source-only、旧 invalid、投影/record 同一证明、历史完整性、Finding/dirty/auth/corruption/replay 拒绝均落实到 Task 1/2。
- 原子性可证明边界为 Summary 单点提交与精确 pending 续作；明确保留中断残留，不使用跨文件物理原子的错误承诺。
- 原生 scope、review 与 Runtime 采用留在真实门禁，不将 Raydot 普通技术批准冒充 human:Ray。
- 未改代码、未执行测试；全部步骤尚未实施。若两文件不能承载内部资格/事务证明，先回报真实范围变化，不扩文件绕门禁。
