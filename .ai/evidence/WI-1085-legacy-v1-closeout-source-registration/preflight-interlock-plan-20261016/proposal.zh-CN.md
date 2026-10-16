# preflight 审阅请求生成互锁：最小方案，待 Raydot 技术审阅

日期：2026-10-16 JST。性质：现有流程内的局部修复设计；本轮未改产品代码、Contract、parity、收据或人审记录，未运行测试或验证。实施方法仍沿用原单实施者、阶段技术审阅、单 worker、隔离 target、900 秒上限。此文不构成实施准入。

## 目标与授权

让已批准敏感修订的 active Work Item 能由 Runtime 生成真实、当前绑定的审阅请求，解除请求生成与等待确认之间的循环。既有政策、有限材料 profile、actor/authority 与收据 schema 不变；不为相同政策重复请求 Ray 批准，不新增人审点，不把政策批准、评论中的确认选择或模板写成已经存在的 confirm_review 收据。实际机器绑定仍只能由支持的 Runtime 生成、由现有严格入口校验。

当前工作分支检查点为 `6339a7712863cbe0b3bd8f07a55ef27f4b90520a`；原实现源码仍是已审阅 `9ecd4495` 的字节。WI-1085 当前 Contract 为 `sha256:eae15cbbdbc36606635db999976611d3c036260296d2b5b24b9afa183ae03bd5`。Ray 的既有提案批准绑定原始请求 SHA-256 `d946d7256496a33e52e35b330bc104055f2148006a05952afd63cc0a3a380cad`，已由原生 typed amend sequence 4 应用；本方案不改该批准内容。

## 两个复现及证据边界

| 事实 | WI-1085：本机原生观察 | Sentinel：GitHub 评论报告，未操作本地现场 |
| --- | --- | --- |
| 新 Contract | `eae15cbb…03bd5` | `e27b9841…0c1e2` |
| Runtime | 1.0.1，SHA-256 `5307ba53…1fa38` | 源码 `253dd007…d5e8`，1.0.1，SHA-256 `f6aed2b7…906132` |
| 已应用政策 | sequence 4，三 parity 路径及窄 profile | sequence 2，窄材料 profile；产品提交 `f96211c8…5201b` |
| 材料 | enabled；两份 permitted syntax Unknown；无 Finding | enabled；无 Finding；评论明确实际材料 review 尚未完成 |
| 阻塞 | checkpointed/yellow，旧 review/preflight 绑定，`preflightState=not_run`；缺三 parity 行先遮蔽验证前置错误 | checkpointed，旧 verification/decision 失效；`governance_red` 与旧失败事实使请求路由更早消失 |
| 请求生成 | `run_preflight` 退出 1，stdout 空，新 digest 未生成 | 相同拒绝；评论报告 Contract/Summary 失败前后哈希不变 |

Sentinel 来源：<https://github.com/xinglun/ai-cockpit/issues/1041#issuecomment-6204404531>。通过 GitHub connector 按确切 comment id 读取；正文保存为 connector-normalized bytes，不冒充远端原始 HTTP 字节。connector 的 created_at/updated_at 均为空，不推测发布时间。评论报告 Ray 已选择新 Contract 的 confirm_review；本机不据此制造或认证真人收据。

## 根因定位

1. `status_projection.rs` 的 `amendment_review_required` 已知道当前 Contract 尚缺匹配 review，但 `run_preflight` 主要从验证路由的首个错误推导。`blocking` 分支可能先返回 resolve/stop；即使走到验证检查，`lib.rs::check_verification_preconditions` 先返回 documentation finding，后续“当前 preflight 失效”条件被遮蔽。
2. 现有 `preflight_can_recover_verification_precondition` 的敏感修订特例绑定 pending retry；它不能普遍表示这两个请求生成场景。仅删除 retry 条件也不能修复前面被 red/doc 分支遮蔽的情况。
3. `lifecycle.rs` 在同 WI lock 内先调用 `require_current_action_admission(run_preflight)`，然后才更新真实 preflight 绑定。`action_admission.rs` 已支持 aggregate blocked/needs-human 时允许明确 safe action；不需要移除该检查或调用无 Runtime 的入口。
4. `lib.rs::apply_preflight_review_evidence` 的 Invalid 分支直接设置 Yellow/needs_human_decision。开放请求生成后，应防止这一步覆盖原始 Red/blocked；blocker、Unknown、证据一致性判断均必须保留。

上述是共同的路由缺口；不据此声称原 #1041 的 finish/preflight 现象、材料扫描或旧 verification 的有效性都已解决。

## 推荐方案与写入范围

### A. 独立的请求生成准入

仅修改 `crates/cockpit-repository/src/status_projection.rs`：在当前身份、Contract、journal 和状态投影已校验后，独立计算私有的 amendment-review-request-generation 可用性，不依赖验证门禁的第一条错误，也不要求 aggregate governance 非红。

同时满足：active 且非 archived/closed/recovered；生命周期属于现有 preflight 接受的 implementation_active/checkpointed/finish_ready；现有 `amendment_review_required` 为真；原生 amendment audit 确认当前 Contract，且 `verificationInvalidatedByContractAmendment.contractHash` 精确匹配它。不能只信任一个可自报的 Summary boolean 或任意 object。初始 `not_run` 可生成；已生成但尚待确认的 yellow/red 请求可重读/重新观察，以免落盘后回报中断再次锁死。非法状态、坏 journal、错身份不进入此特例。

这个分支只加入并优先解释 `run_preflight`；保留 blocking、humanDecisionRequired、原 blocker/Unknown、原旧证据及所有执行/完成门禁。它不能加入 run_verification、finish、archive、材料接受或清理动作。不能因同一流程的旧 receipt 失效而删除 receipt。

真实执行仍经过现有生命周期锁、active Contract canonical identity、当前 Runtime action admission、retry 绑定及 observation.validate_current。错误沿原路径返回，不写假的 pending request。CLI/MCP 的 public surface 不增加参数、flag 或 bypass。

### B. 保留原 Red/blocked

仅修改同一已有函数所在的 `crates/cockpit-repository/src/lib.rs`：Invalid review 分支仅在原决定非 Red 且无 blocker 时设置 Yellow/needs_human_decision；已有 Red/blocked 保留，仍可附加“review 待绑定”的说明。不得清除 evidence_contradictory、Finding、identity/Runtime/snapshot 错误或原 Unknown；不修改 receipt 验证器、材料分析器或 verification precondition 的执行顺序来取得放行。

### 测试位置与不变项

- helper 和 Red 保持的单元/仓库 fixture 放在上述已授权 Rust 文件的测试模块。
- 新 CLI/MCP 成对回归放在当前已授权的 `crates/cockpit-cli/tests/legacy_closeout_source.rs` 独立 `preflight_review_request` 模块：该测试文件已有真实 CLI binary、MCP 和完整仓库 fixture 依赖，可覆盖当前 WI 的修订阻塞，无需新路径或依赖。
- 原 `contract_amendment_processes.rs`、MCP `rpc.rs` 的既有回归保持原断言并纳入复验；不为了塞入新文件而扩大当前 Contract scope。
- 不改 action_admission 的全局规则、lifecycle 的锁/重读、schema、材料 profile、Contract、人审点或 before_finish；不增 WI/工作树隐藏旧冲突。

不推荐全局允许 needs-human 下任意 preflight，也不推荐重排/跳过验证门禁：前者范围过大，后者不能处理 red 分支并会改变错误优先级。

## 验收矩阵（计划，尚未执行）

| 隔离 fixture / 边界 | 必须证明 |
| --- | --- |
| WI-1085 型：无 retry、not_run、三 parity 缺失先返回 | status 明确给出仅用于请求生成的 run_preflight；CLI 与 MCP 成功返回当前 Contract/真实 decision/snapshot 绑定，parity 仍原样缺失且阻断后续验证 |
| Sentinel 型：旧 passed verification 与旧 decision 经敏感 amend 失效、带 governance_red | 无需删旧 receipt/改 Summary；可生成请求；旧失败事实和 raw blocker 保留，不得报告 verified/完成 |
| 原始 Red、Finding、foreign/missing/tampered receipt | 若请求可生成，只是带原 blocker 的请求；Red 不被 review 投影改黄/绿；验证/finish/archive 仍拒绝，项目命令 marker 不出现 |
| 坏 journal、错 repository/WI、invalidation hash 错、非法/terminal 状态 | 不授予新特例；拒绝前后 canonical bytes 与现有收据不变 |
| 旧完整 receipt、旧 decision digest、旧 snapshot，以及新 Contract 与旧字段混搭 | 现有严格入口均须拒绝不匹配当前真实 preflight 的输入；不能只测试旧 Contract digest 一项 |
| 同一当前请求重读、发布后报告中断 | 原绑定可重新取得；不自动写 decisionEvidence 或接受材料，不产生重复 checkpoint |
| 等锁期间状态/Contract/snapshot/retry 变化 | 沿现有锁内重读与绑定校验拒绝或重新产生真实当前请求；不得复用旧准入；原 lock waiter 断言保持 |
| 有效、已记录的既有当前批准 | 继续复用其真实授权事实；技术修复不创造新的政策问题或人审；后续材料 review 与 verification 仍各自取得实际证据 |
| 原普通 flow、pending retry、scaffold/standalone、post-verification finish | 原路径保持；本修复不以重跑历史检查或虚构成功解除其他 #1041 失败 |

静态审阅注意：当前 decision validator 分别比对当前 Contract 和 Summary 中的 preflight/snapshot 字段。因此“新 Contract + 尚未刷新的旧 Summary 字段”必须在隔离 RED 里明确覆盖，不能假设校验已足够。若出现误收，先给 Raydot 返回具体 RED 与最小补充范围；不偷偷改未授权的 governance_controls.rs，不删此负例来压缩方案。本轮不向真实 WI 提交这类输入。

## 执行与交付顺序（待 Raydot 技术审阅）

1. 先写上述两个代表 fixture 与关键拒绝用例，保留真实 RED；不对 Sentinel 本地或 WI-1085 实际记录做试验。
2. 实施 A/B；按原公共 CLI/MCP 路径复验，单 Cargo job、独立 target、每次最多 900 秒。先核对当前 Runtime 准入；本方案不是自行执行或绕过准入的批准。
3. 保留失败证据和原字节；独立技术复核；正常阶段提交推送。声明完整 workspace 回执前仍须有当前正式验证，不以 focused GREEN 替代。
4. 构建候选 CLI 后记录源码提交、工具链、命令、路径与 SHA-256，不覆盖共享安装版。仅在 Raydot 给定受支持的运行顺序且当前准入允许后，用候选对真实 WI 生成绑定并读回。
5. 继续沿用已保存的实际政策批准/确认事实；补齐真实机器绑定，不重复问 Ray 同一政策，也不把不存在的收据说成已记录。若输入事实变化，交 Raydot 判断差异，不自动扩大批准。

本轮交付仅是这个方案、精确来源和自审结果。当前新 preflight digest 仍未生成；parity、材料决定、verification、finish/archive、PR/合并/关闭/清理均未执行。
