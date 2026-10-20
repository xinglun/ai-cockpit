# 提案对比：当前真实投影与独立路径事实功能

状态：proposal only。当前推荐仅修正前项三语当前状态并原生修约两处计划字段；Runtime、gate 和当前 Contract 均未修改。早先“直接 snapshot_against(baseRevision) 写成本项责任”的提案不再采用。

## 三条路径

| 方案 | 当前收尾必要性 | 入口及交互 | 验证与事实边界 |
| --- | --- | --- | --- |
| 前项纯进行中；本项条件登记不变 | 推荐；与当前 awaiting_successor_close 实际状态一致 | 仅三份现有 ledger 的前项第二列；两处原生计划修约 | 保持全部 gate 及终态断言，以实际 Git/字节审计证明三份 ledger 修改，不请求旧条件晋级例外 |
| 现有 run_verification 启动前刷新原生事实 | 若将来确需该产品功能，这是比独立入口更合适的候选 | 同一个 verify 用户动作；plan-only 返回后，由 CLI/MCP 共用的库级准备逻辑完成；不增 controls 类型或外部动作 | 短锁更新材料事实，释放锁运行 checks，失败仍只保留 attempt，成功另行验证并记录完成证据 |
| 独立刷新入口/附加 controls 刷新语义 | 当前收尾不需要 | 多一次用户/Agent 操作；或者给既有 controls 增加隐含材料刷新语义 | 仍需要 verify 启动前与结束后的新鲜度验证；并不能减少 TOCTOU 或成功覆盖问题 |

先前选择 controls 是因为它已经共享 CLI/MCP、已有当前准入和生命周期锁。但这些条件可以复用于现有 verify 启动边界，所以不是必须使用 controls 的理由。当前收尾不应承担新增路径事实产品功能。

## 两个基线的事实和责任

- `Contract.baseRevision=b8ec2690976ab4c554aa9f5b2aa7fcca715156ca`：保留原生激活后的仓库/PR 比较基线。不可改成 cba3。
- successor scaffold 起点为 `cba3ea1661749ebbf1929bc523c5b86ff10eb7e7`。原生 activation-transition prepared 记录的 previousContract.baseRevision 保留该值，newContract.baseRevision 为 b8ec，prepared digest 为 `sha256:c7548ebca2858defb83b2aa472f07562cb41fd57f87cda2a0a13a5e4e76a98ab`。正式产品设计若使用它，必须经现有 committed/chain/owner/repository 校验器读取，而不是直接信任 JSON 或 Agent 索引。
- 实际 `b8ec→cba3` 的 Rust 差异是前项的 `crates/cockpit-repository/src/material_review.rs` 与 `crates/cockpit-repository/tests/material_review.rs`；`cba3→当前HEAD` 没有 Rust 差异。
- 仓库比较差异不能直接当作 successor 自身实施。当前文档方案的实际责任审计使用 `cba3→HEAD` 加当前工作区差异，再核对精确批准范围。Git 已证明27个范围路径发生改变，包含全部三份 ledger；这证明本项对这些文件承担修改责任，不证明本项作者创作了其中全部历史内容。
- `baseCommit` 按现有 schema 必须与 baseRevision 一致；不能把 cba3 偷放在那里。当前 baselineDirtyPaths 为空。缺少可校验原生责任起点、祖先关系不成立或存在未解释的既有 dirty 输入时，未来产品功能应保留 Unknown/拒绝责任归属，不能回退为复制 scope 或使用 Agent 自报起点。

## 如以后单独选择 verify 启动前方案

以下只是接口级 proposed diff，未实现、未编译或测试，不是本轮建议的 Contract scope 扩展。

```diff
 CLI/MCP 现有 verify：
   保留路由解析、required checks、worker、timeout、只读 planning 与所有既有准入
   if plan_only: return 现有计划
+  在现有 Work Item run_verification 动作内调用共享准备逻辑
+    取得同一 WI lifecycle lock，并 require_subject
+    在锁内重查 action admission 与当前 preconditions
+    校验 Contract/repository/Runtime/原生责任起点/HEAD/源码快照
+    原子更新实际观察到的 scoped successor changedPaths（无 pass/close 声明）
+    返回绑定该观察的内部 token；释放生命周期锁
   执行原有 checks
+  每次 spawn 前/执行结束后验证同一绑定仍成立
   保留失败或执行完成 attempt
   仅全部 checks 真正成功且绑定仍新鲜时，走现有完成收据事务
-  成功记录时以工作区-only snapshot.changed_paths 覆盖 Summary.changedPaths
+  使用同一已绑定、重新验证的 successor 实际增量，保留已提交路径
```

现有 `run_repository_verification_inner` 本身没有 lifecycle lock，也没有完整 RuntimeContext；不能只凭 request.runtime_digest 给低层服务增加有权写 Summary 的行为。应在现有 Work Item 验证启动层引入共享内部准备函数，保留通用/无 Work Item 执行和只读 preconditions 的语义。建议研究位置为 `lifecycle.rs`、`contract_amendment.rs` 的已验证原生起点访问器，以及 CLI/MCP 两个现有启动调用点；是否还需通用服务的每次 spawn 前 hook，取决于绑定传递实现，不能预先宣称四文件足够。

锁只覆盖原生状态转换与事实写入，不能持锁等待外部 checks。它串行化遵守该锁的 Runtime 写入，不冻结 Git 或编辑器文件。应复用 ObservationContext 的身份/配置/策略/源码一致性检查，并另外验证当前 HEAD、责任起点和 scoped 路径字节/差异绑定；不能声称物理多文件原子性。发生漂移时保存真实执行结果和拒绝提升的 attempt，不能将真实执行通过改写成通过当前源码，也不能删除已有 b569/da4a 记录。启动前事实和验证完成证据属于不同事务，后者失败不应抹掉前者的真实历史观察。

独立刷新方案有相同的事实来源、锁和后验绑定要求，而且刷新到执行之间又多一个陈旧窗口，仍须 verify 再验证。当前没有业务需求支持多一套交互。

## 当前优先的最小文档修约

`predecessor-open-state.proposed.diff` 仅改变三份 ledger 的前项第二列。第三列和本项完整条件行逐字节不变。`contract-open-state.amendment.proposed.json` 只替换 `/acceptanceCriteria/2` 和 `/scenarioCoverage/1/expected`；scope27、required checks2、原生审阅/材料政策、字节保护和终态授权均不变。

本项 finish/archive 后，前项仍是 awaiting_successor_close。将来真正的后继确认关闭后，gate 会将前项投影为 recovered，此时需在另行获准的派生文档更新中使用 Recovered/已恢复/復旧済み 并绑定实际 recovery 与后继终态证据。这不等于前项已有 close，也不授予任一 close。原来的 In progress→Implemented 条件行届时同样不能自动满足 recovered 的严格状态列检查；采用纯进行中没有新增必须伪造旧登记的义务。

这是收回当前过度承诺的历史晋级声明，不是取消 required gate。现有 positive pending fixture 覆盖该业务表达；提前 Recovered/Implemented 的 negative fixtures、原生 successor 绑定、归档/证据完整性及真正 close 规则继续有效。当前 AC3 已绑定那个更强机制，必须先经原生修约及实际新绑定审阅/准入，不能把它直接标成通过。未运行候选全 gate、未重写上下文以取得通过；实际两项检查仍须在获准修约与源码提交之后执行。
