# WI1090 changedPaths 原生根因修复集中方案

状态：待 agent:Raydot 技术审阅；没有执行修约、源码修改、构建或新一轮验证。依据真人 Ray 2026-10-20T01:37:48Z 的决定“发现的流程问题要从根因修正，要不然以后还会卡住”。该指令明确修复目标，不是假定批准尚未生成的 Contract，也不是材料残余风险接受。旧“前项普通进行中”草案保留为未实施历史，不作为最终方向。

补充产品原则：Ray 2026-10-20T01:45:29Z 指出“这也是流程摩擦和阻碍的问题，本来ai cockpit是帮助人和AI协作的平台，如果因为治理摩擦太高就失去了核心价值。”正常合法恢复必须连续完成，不能让实现缺陷变成日常手动刷新或额外确认。该原则不取消明确真人门槛，不扩大授权。

## 目标与不变约束

解除“验证 gate 需要 changedPaths，而 changedPaths 要等全部验证成功才登记”的循环。原生登记必须覆盖已提交改动；成功记录不得用空工作区列表清掉集合。既有 all3 ownership 断言、原 AC3、前项与本项条件登记、两项 required 文档检查、失败记录、归档完整性和真实终态授权全部保留。

不新增 Work Item、CLI/MCP 动作、controls 类型、调用者 supplied-changedPaths 字段或通用能力框架。只扩展现有 Work Item verification 启动/完成边界的内部实现，并在已有 agent-workflow 三语文档固化体验验收条款。当前已批准的8项副本迁移、2个 locator、9份文档范围继续保留，保护前项原生27项、两个 manifest、前项两个 Rust 文件、其余19条路径及外部 fixture。

## 原因与当前证据

固定 Runtime b5cbd857 的 `lifecycle.rs:9069` 用验证输入的工作区 changed_paths 覆盖 Summary；CLI `main.rs:2662` 的失败分支先保存 attempt 再退出，根本不会到该 writer。`checkpoint` 是 count0 的 before_edit，不是编辑后刷新入口；controls 六字段不接受 changedPaths。当前 source HEAD2748e2d3 的工作区列表为空，但 Git 已证明三份 parity 已提交。真实 attempt b569 执行2/max1/reuse0，文档验收 exit0，parity exit1/两项错误；da4a 零进程拒绝也保留。

现有 `run_repository_verification_inner` 和现有成功记录函数都没有包住所需完整操作的生命周期临界区。不能假设已经有一把覆盖检查执行的锁；也不能只凭低层 request.runtime_digest 给通用服务新增有权写 Summary 的行为。

## 两个基线与责任归属

仓库/PR 比较基线 B 保持原生 `Contract.baseRevision=b8ec2690976ab4c554aa9f5b2aa7fcca715156ca`。本项起点 S 为 `cba3ea1661749ebbf1929bc523c5b86ff10eb7e7`，来自已提交原生 activation transition 的 previousContract.baseRevision；其 newContract.baseRevision 为 B。起点证据 prepared digest 为 `sha256:c7548ebca2858defb83b2aa472f07562cb41fd57f87cda2a0a13a5e4e76a98ab`，通过现有 prepared/committed、digest、owner、repository 和 amendment-chain 验证器读取，再验证 S→activation observedHead→当前 HEAD 的祖先关系。

- B→HEAD 是仓库 observed delta，继续供现有 material/CI 比较使用；不整体放成本项责任。
- S→HEAD 加实际 index/working/untracked 差异，使用现有 `GitRepository::snapshot_against(S)` 的 no-renames 语义，得到本项真实增量。不是把 Contract.scope 抄成 changedPaths。scope 仍单独约束授权；缺少实际改动的 scoped 路径不进入集合。
- 已确认 B→S 的 `material_review.rs` 和其同名 integration test 属于前项；S→当前 HEAD 的 Rust 差异为空。后续本项仅允许修改新增的7个精确文件，不改这两个继承文件。
- 路径责任只表示 S 之后该文件确有本项增量，不等于创作该文件全部历史内容。若两域共享同一路径，也保留 B/S 两个比较对象与 before/after 字节绑定，不把继承 hunk 重记为本项实现。
- 不改 baseRevision；baseCommit 不能用来偷放 S。当前 baselineDirtyPaths 为空。缺少原生起点、partial/foreign/tampered activation、断裂或多义链、祖先不成立、未解释既有 dirty 输入时拒绝责任推断；不采用 Agent 索引或旧未提交日志作为原生起点。普通/legacy 无相应绑定的路径保持兼容，不以当前任务发明它们的历史。

## 现有 verification 的闭环

1. 在路由/命令计划前只读捕获内部 start binding：当前 Contract 原始/模型 digest、repository/owner、Runtime、journal head、HEAD、source snapshot、配置/策略及原生责任起点。路由和选定 requests 必须与该捕获对应。`plan-only` 继续只读，不登记事实、不启动进程。
2. `plan-only` 返回分支之后、执行第一个 check 之前进入共享 prepare。取得同一 WI lifecycle lock 并 require_subject；锁内重新读取并比对计划时绑定，重新检查当前 run_verification admission、preconditions、policy、required requests。等待锁时 HEAD 或 Contract 已变，应拒绝旧计划，零进程，不悄悄改绑新状态。
3. 在该短临界区用 S 的真实比较派生路径事实，并做前后 source/HEAD/Contract/起点一致性检查。原子写入当前 Summary.changedPaths 及其原生 observation binding。只登记材料事实；不写 verification passed、scenario verified、finish/close 或任何旧真人决定。
4. 释放锁再运行外部 checks。生命周期锁只串行化原生写入，不冻结 Git/编辑器文件；不得持锁等待 checks 或递归等待自身的记录函数。
5. 原始执行结果和 attempt 按现有机制保存。首次失败只保留真实失败，不提升完成；启动前材料事实不因 check 失败而回滚成旧集合。重新验证重新观察并登记当前事实，同时保留所有旧失败字节。当前状态直接允许 run_verification，不预设要补 retry；未来仍按实际 admission 判断。
6. 完成记录再取得同一锁，重新检查 Contract/journal/Runtime/HEAD/source/责任增量绑定以及现有政策和生命周期条件。解锁到 spawn 或 checks 运行期间发生未恢复的绑定变化时，保留实际执行结果并拒绝提升；不能称它验证了新状态。现有 execution_completed→formal_receipt_rejected/superseding attempt 机制可保留“进程成功、绑定拒绝”的区别。
7. 仅所有 required checks 实际成功且绑定仍匹配，调用原有完成事务。该事务使用已绑定、重新确认的责任路径事实，不能执行工作区-only空集覆盖。重用/零进程路径也必须先做同样的原生事实与绑定校验。legacy raw recorder 不得到伪造起点或新权限，也不得覆盖已存在而不能验证的当前 Runtime observation。

observation binding 只需 repositoryId、workItemId、Contract/journal/Runtime、B/S与起点证据 digest、observed HEAD、source snapshot 和实际路径/字节差异 digest 等本次内部资料；不是新的人类决策或成功收据。可存为 Summary 的原生观测元数据，归档原样保留。事实写入与完成事务分离，完成事务 rollback 捕获的基线应包含启动前的真实观测。所有旧 receipts/attempts/history 不重写。边界观察不声称物理多文件原子性。

HEAD 的严格相等只约束本次计划、锁等待和实际执行/完成绑定。完成后正常追加并提交 `.ai` 证据，按现有 source snapshot 语义不使源码验证失效；不把历史 observation 的 HEAD 当作永久授权条件。未改变 Contract/源码/必要政策依赖时，不能因此要求新确认或新一轮验证。相同事实刷新应幂等，不因时间戳或重复调用制造新的决定。

## 精确实现范围：4个实现文件 +3个回归文件 +3份现有流程文档

1. `crates/cockpit-repository/src/lifecycle.rs`：共享只读 start binding、锁内 prepare、原生责任 delta、完成时复核/短锁，以及成功 writer 不清空。沿用既有 public re-export，不新增命令。
2. `crates/cockpit-repository/src/contract_amendment.rs`：仅暴露复用现有完整验证器的内部责任起点读取；不放宽 transition/recovery 校验，不新增可写 baseline 字段。
3. `crates/cockpit-cli/src/main.rs`：现有 verify 的只读捕获、plan-only 后 prepare、传递绑定及完成记录接线；失败分支和 required checks 保留。
4. `crates/cockpit-mcp/src/lib.rs`：现有 verify 对同一内部逻辑接线；工具 schema、交互及调用者字段不扩展。
5. `crates/cockpit-repository/tests/lifecycle_order.rs`：核心 native 轨迹及竞态回归。
6. `crates/cockpit-cli/tests/verify.rs`：CLI plan-only 和真实首次执行/失败/重验接线。
7. `crates/cockpit-mcp/tests/rpc.rs`：MCP plan-only 和执行前事实可见性。
8. `docs/reference/agent-workflow.md`。
9. `docs/reference/agent-workflow.zh-CN.md`。
10. `docs/reference/agent-workflow.ja.md`。

后三份复用现有 Project-specific adaptation→Resource finalization 边界之前的位置，增加等价的6项流程体验验收：合法连续流、必要决定的一次完整绑定、失败可恢复、正向自动回归、实际介入计数与审阅证据、摩擦作为缺陷处理。精确文档草稿见 `workflow-experience.proposed.diff`，不新增模板/管理器或第二套问题清单。最终 scope 从27扩为37。

不改 `governance_integrity_gate.py`、原有断言、cockpit-core/protocol schema、治理全局配置、CI workflow 或继承 Rust2。若这10个明确文件不足，返回具体缺口给 Raydot；不自行扩展。

## 最小回归矩阵

所有新增测试统一前缀 `source_change_observation_`，至少覆盖以下核心情况，并在 CLI/MCP 各覆盖只读 planning 与实际启动路径。fixture 明确为 synthetic，不借用真人授权。不能只手写 Summary 三路径来让 ownership 成立。

1. 三份文档已提交、工作区 clean：第一个 check 读取到原生三路径，且此时无 canonical success receipt。
2. B→S 继承 Rust、S→H 改文档：继承 Rust 不进入 successor changedPaths；B 比较事实不被删除或改标来源。
3. scope 有三路径但只实际改两份，或第三份从未改变：既有 ownership 判定仍 false；不把 scope 当变更证明。
4. 首轮 check 确实失败：attempt 和输出保留，事实仍在，无完成声明；重验正确刷新事实并保留首轮 raw SHA。
5. 等待 lifecycle lock 时 HEAD 改变：零进程拒绝、Summary 不被旧计划更新；使用真实 Git 提交构造竞争。
6. 等待锁时 Contract/journal 改变：零进程拒绝；通过并发原生修约或明确的 tamper negative fixture 检验，不把 fixture 历史当真实用户证据。
7. 成功记录、随后提交 evidence、干净工作区：已提交路径不被清空，既有 gate 仍可验证 ownership。
8. 起点 proof pending/foreign/tampered、owner/repository 不符、祖先关系不成立：拒绝且无完成/事实误写。
9. 运行期间源码/Contract 发生变化：实际执行输出保留，但不提升为当前源码的验证成功。
10. 正向恢复：已有真实失败→现有 verify 自动刷新→实际检查成功→正常 `.ai` evidence commit→controls/finish/archive 连续完成。除既有协议确实因绑定变化要求的决定外，新增手工刷新次数0、常规确认次数0。分别统计必要真人决定、Raydot 技术裁决、实现缺陷造成的错误阻断；相同有效绑定不得重复索要确认。CLI/MCP planning、事实幂等与该正向流均需回归。

最后必须在本真实工作项实际运行原两项文档检查，并按实际原生 admission 走通自身 finish/archive；synthetic 回归通过不能替代这一验收。前项与本项 close、合并、删除、发版、全局 Runtime 采用及 Sentinel 均继续禁止。

## 六个声明检查与构建

保留原两项 bash 文档检查，新增 fmt 和三个精确 test target/filter，required=true；不继承旧7项 Rust 验证，不跑全 workspace。检查清单见同目录严格修约 JSON。测试列表必须先证明 prefix 匹配到已声明用例，零匹配不是验证通过。

使用单 worker、test-threads1、隔离 target；每个命令900秒内。需要时先编译这三个测试 target 的二进制，编译资料不充当测试通过。固定源提交后构建一次隔离 CLI candidate，记录真实源 SHA/命令/环境/输出/二进制 SHA，不覆盖 b5cbd 固定 Runtime。candidate 的执行副本与 Cargo 输出路径分开，防止正式验证覆盖正在使用的 Runtime。正式图的依赖先就绪并控制总墙钟预算，仍保留 Runtime 每节点 timeout。

candidate 只用于本 WI 自身原生验证/finish/archive；实际 hash、environment drift、preflight/material/request 与新 Contract 绑定由原生读回判断。不得安装到共享位置、采用到 Sentinel/消费者或继承旧材料风险接受。文档帮助子检查可使用其已有 `AI_COCKPIT_INTERFACE_DESCRIPTION_BIN` 指向同一已绑定 candidate，并如实记录该进程环境。

## 一次性执行顺序与真实门槛

Raydot 技术审阅这份精确方案和修约请求后，用现有固定 Runtime 原生 amend，读回新 Contract、journal、source/environment、材料与 preflight。只有 Runtime 实际提出真人决定时，才返回完整新绑定；旧 f263/af4d 确认不能冒充新确认。允许后完成7个 Rust 文件和3份流程文档的实现/回归，并更新原有 WI 投影，正常 commit/push；构建并绑定隔离 candidate；刷新实际 admission；执行六项检查、真实审计与 controls，依次 own finish/archive，保存 Outcome，提交必要证据，再按原授权处理独立 Draft PR 和真实 CI。普通技术问题交 Raydot，原生门槛不被技术决定覆盖。除真实新门槛外，不逐步骤暂停。

最终 Outcome 必须报告实际人工介入：本次根因方案准备阶段新增审批问题0；两条 Ray 主动产品指令是需求来源，不计为系统索要确认。修约后的必要真人决定次数目前未知，待原生输出后逐次按完整绑定和原因记录；测试预期不能冒充真实次数。将实现错误、过期/重复请求和合法依赖变化分别列明。

## 当前原生修约绑定

严格请求仅需要 CLI 显式 repo/WI 和 `expectedContractDigest=f263774f...`，具体完整值见 JSON；amend 在锁内比较并自行捕获实际 source/environment。当前最新 native journal sequence3/head24c25f79，snapshot7b1e1bae，material64bfbce5，Runtime1.0.1/b5cbd857，当前 humanDecisionRequired=false。最后一次 native amendment 的 environment7d1291fc 是历史值，不能称作尚未发生的新修约环境绑定。新 Contract digest、candidate digest 和未来 human decision binding 均尚未生成，不在提案中虚构。
