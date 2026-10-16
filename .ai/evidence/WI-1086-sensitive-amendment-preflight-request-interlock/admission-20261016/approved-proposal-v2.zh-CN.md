# preflight 审阅请求互锁：Raydot 审阅补正 v2

2026-10-16 UTC。补充 `826dc486304ff65feb6edac4523dcef45df79db5` 中的原方案；原文和全部证据保留。Raydot 结论是 A 有条件通过、B 须修正，尚不放行产品实施。本轮只有只读定位和受限证据追加，没有产品修改、测试、独立 WI 创建或 Runtime 准入绕过。

## 1. 独立修复 WI 的最小 scope，提交 Raydot 决定

目标是修复通用 Runtime 的敏感修订审阅请求生成与原阻塞投影；这与 WI-1085 的 legacy v1 source registration 目标不同。新 WI 编号待协调确定，不借用 WI-1085 的文件 scope 作为实施授权。

建议预先声明以下路径：

- `crates/cockpit-repository/src/contract_amendment.rs`：内部当前修订绑定检查，复用现有原生完整 journal reader。
- `crates/cockpit-repository/src/status_projection.rs`：独立加入请求生成动作。
- `crates/cockpit-repository/src/lib.rs`：B 的状态保持及局部单元断言。
- `crates/cockpit-cli/tests/contract_amendment_processes.rs`：沿现有 CLI/MCP fixture 增加成对进程回归，不再把通用回归塞入 legacy source registration 测试。
- 新 WI 自身三语工作项文档、三个既有 reference-parity 账本中该 WI 的行，以及该 WI 自身 `.ai` 生命周期/证据路径；采用确定编号后的精确路径，不使用覆盖其他工作项的通配范围。

不改协议/schema、公共参数、全局 action admission、收据校验器、材料分析器或现有政策。若严格收据负例暴露额外缺陷，先将实际 RED 与最小补充范围交 Raydot，不删负例、不暗改未声明文件。

### 正常准入路径和当前真实结果

1. Raydot 先审阅上述 scope；之后从正常 fetch 核实的 main 基线，在同一 Git common-dir 创建专用分支和 linked worktree。保留并明确报告 WI-1085 原 worktree、未完成状态及全部证据。这是不同目标的独立修复，不是原工作项的通过、恢复 successor 或关闭。
2. 使用当前受支持 Runtime 的正常 `start` / `start --prepare`，从首次声明即包含上述完整 scope、用户价值、验收及验证命令，避免先开窄合同再用敏感 amend 自救。不复制另一工作项 receipt 或使用无 Runtime 的入口。
3. 保存原生 start/preflight/唯一 before_edit checkpoint、当前 Contract/snapshot/Runtime 绑定，重新查询 status 的真实动作准入。只有实际允许实施后，才写 RED fixture 和产品文件。若 start、preflight 或 checkpoint 被阻断，保留原始输出并交 Raydot；不手改记录、不反向补批准、不将 standalone Contract preflight 冒充实施准入。
4. WI-1085 是待消费该修复的原工作项，不是修复 WI 的已完成输入。其证据是只读复现来源。修复先完成自身技术复核和所需验证，候选 Runtime 的采用顺序另由 Raydot 判断；之后才按 WI-1085 原 Contract、原政策批准和实际 Runtime 动作恢复。
5. 所有重叠路径（至少 lib.rs、status_projection.rs、三 parity 账本）、源码提交、构建与生命周期动作串行。修复期间 WI-1085 不写产品源码/账本，不并行 build，不 cherry-pick 工作项记录。协调 help 的存在不证明候选写协议可用；本轮 coordination inspect 只有空集合，不把它当注册、lease 或跨 WI 准入。

**实际结果：独立 WI 尚未创建，尚未取得其 start/preflight/checkpoint 或实施准入；不是已准入。** 本轮保留的真实查询均针对旧 WI-1085：

- Runtime 1.0.1，SHA-256 `5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38`。
- HEAD `826dc486304ff65feb6edac4523dcef45df79db5`；当前发现的本地 origin/main `681df30e2fe8c869b6a41498822818c017dc4fb6`。本轮未 fetch，不把该值称为远端最新 main。
- repository readiness 为 blocked，blockers 为 `active_work_items_present`、`base_revision_not_synchronized`；dirtyPaths=[]、unclosedArchivedWorkItems=[]。普通 start 的代码把无关活跃对象列为 advisory，仍检查精确资源冲突及基线；这里不是新 WI 的执行结果。
- WI-1085 为 checkpointed/yellow、humanDecisionRequired=true；safeActions 只有 controls/usage/refresh_status，admission `sha256:7b475950fa9932c88fd242c9cc9c0aec11d5961af705d62a39dae111df129245`。不能在该上下文先自修门禁。

未创建的原因是本轮仍处于 Raydot 要求的“先确认独立 scope”阶段；当前 CLI help 没有只读的候选 start-admission 接口。不能把查询、静态推导或空 coordination store 说成独立 WI 已获准。下一步由 Raydot 技术审阅决定正常建项/准入阶段，不向 Ray 重复请求同一政策批准。

## 2. A：完整原生绑定条件

复用 `read_work_item_contract_amendments` 的完整 prepared/committed journal 验证（序号、链、身份、typed request 导出、digest、敏感字段重新计算及未决事务拒绝），并在私有 helper 中追加核对：

1. 当前 canonical repository、active Contract、Summary 和 WI 身份一致；仅 implementation_active、checkpointed、finish_ready 三个非终态可进入该特例。
2. journal 不空；逐项 Contract 变迁连续（相邻 receipt 的 previousContractDigest 等于前项 newContractDigest），链尾 newContractDigest 精确等于当前 Contract digest。旧 reader 验证 journal hash 链不替代这两项当前绑定。
3. 至少存在经 typed request 重新推导的敏感修订，且现有严格校验判断当前 review 仍 Missing/Invalid。不能仅凭历史 sensitive 布尔值、任意 Summary object 或 marker 授予动作。
4. 当前原生 `verificationInvalidatedByContractAmendment.contractHash` 必须等于当前 Contract digest 和 journal 链尾 newContractDigest。对应的最新原生 amendment revalidation checkpoint 应绑定同一 repository/WI/Contract、previousContractHash、amendment snapshot 和 invalidatedRequiredChecks；与原生 receipt 不一致就拒绝特例。这里的 hash 是实际已有 contractHash，不臆造另一种“invalidation digest”字段。
5. 历史 amendment snapshot 保持历史绑定，不要求它等于后续已变化的当前源码 snapshot。每次请求（包括重复取得）仍走真实 Runtime 的 WI lock、当前 action admission、retry 约束、观察及 snapshot 重验，产生真实当前绑定。

只加入 `run_preflight` 请求生成动作，保留原 blockers、人审待办及所有执行/finish/archive 门禁。坏链、链尾不符、外来身份、错误失效 hash、未决事务或终态不得取得这个例外。错误传播，不降级成“无 journal”。本修复不自动写确认，也不复用不匹配的新旧字段。

## 3. B：三个投影维度一起保留

在 `apply_preflight_review_evidence` 同时修正：

- sensitive_amendment 且没有有效 receipt 的前置分支；目前它先无条件覆盖 outcome_state，Missing receipt 不经过 Invalid 也会受影响。
- Missing 路径：不得因前置分支把原 blocked 改为 needs_human_decision。
- Invalid 路径：不得无条件把 Red 改 Yellow 或把 blocked 改 needs_human_decision。

对原始 `state=Red`、`outcome_state=blocked` 或含 blockers 的决定，保留各原字段和值，待办只追加到 review/request/required-check 等已有说明字段；不清除任何 blocker、Unknown、Finding 或不一致事实。仅当决定原来可等待人审且无阻断时，才沿用 Yellow/needs_human_decision 投影。

## 4. 验收与兼容性（计划，未执行）

| 用例 | 必须断言 |
| --- | --- |
| WI-1085 型 documentation 首错；Sentinel 型 governance_red/旧验证失效 | 生成真实当前请求；原材料/验证门禁仍保留；命令 marker 不出现；不写确认 |
| 三个允许非终态分别测试 | 只有完整原生链及当前绑定成立时加入 run_preflight；覆盖 not_run 与重复取得 |
| B：敏感修订 × Missing/Invalid × 原 Red/blocked/非空 blockers | 同时断言 state、outcome_state、blockers 三个维度；待办存在不等于原阻断解除 |
| 坏 journal、prepared/committed 篡改/缺口/未决、断 Contract 变迁、旧链尾、foreign repo/WI | 拒绝该特例，canonical bytes 与旧收据保持；覆盖单 marker 伪装及错误 invalidation hash |
| archived/closed/recovered 与其他非法态 | 拒绝新增请求生成特例 |
| 旧 Contract/decision/snapshot、混搭新 Contract 和旧 Summary 字段、锁等待期间 snapshot 改变 | 严格入口拒绝不匹配输入；重复请求也读取当前 snapshot，不复活旧确认 |
| 当前真实有效批准、普通 flow、retry、standalone/scaffold | 不新增政策人审、不改协议或旧断言；材料审阅、验证和 finish 各自需要真实证据 |

通过技术 scope 审阅及真实实施准入之后，再依次 RED → 最小 A/B → CLI/MCP 回归与完整声明验证 → Raydot 技术复核；隔离 target、单 build job/worker，每次上限 900 秒。不得先补 parity 获取旧 WI-1085 准入。本轮没有运行测试、验证或采用候选 Runtime。

## 保留与健康

WI-1085 当前 Contract `sha256:eae15cbbdbc36606635db999976611d3c036260296d2b5b24b9afa183ae03bd5`、journal `sha256:74c3f7649500d6416d4c2df88ba68ad925759d95674a01525ac94dc1fbb4c822` 及原批准事实不变。Sentinel 精确来源仍为 issue 1041 comment 6204404531；本轮未接触其本地工作区。

观测 PID1=tail，21,316 个继承僵尸，均 PPID1；workspace 约 4.13 GB、tmp 约 2.29 GB 可用。本轮 Runtime 查询使用仅回收本命令后代的既有 wrapper，没有新增僵尸或全局进程/权限更改。此文是修正方案和准入缺口报告，不是实施完成、独立准入通过或 WI-1085 通过回执。
