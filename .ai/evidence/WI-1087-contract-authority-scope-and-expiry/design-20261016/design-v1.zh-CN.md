# WI-1087：有限期限的目标授权 — 设计 v1

日期：2026-10-16（Asia/Tokyo）。状态：**planned / 待 Raydot 技术定稿；无新能力实施或现有授权变更**。原生 Work Item 为 mode=planning、state=not_ready、checkpointCount=0，产品实施 authority=missing。当前修复 WI-1086 优先。

## 1. 已确定的目标与边界

**有期限的目标授权，范围内连续执行，实质越界才请求决定。** 防止 agent 把旧任务 A 的批准直接沿用于新目标 B；同时避免正常拆分、换 WI、阶段切换、并行、范围内修复、重试和恢复逐个要求真人再批。

期限必填、有限、明确，由有权授权者确认。expiresAt 不得为 null、无限、仅 until_done，或用默认极大哨兵模拟无限。900 秒是单操作上限，与授权窗口独立。缺期限只可准备计划和申请，不能启动依赖该授权的执行。合理窗口按预计工作、验证和缓冲提出，不为所有任务强加短窗口或周期性续批。例如预计6小时实施+2小时验证+2小时缓冲，可建议10小时窗口，由授权者确认；这只是示例，不是本项目的实际授权。

生效条件：notBefore <= now < expiresAt。目标完成、有效撤销或到期任一成立即终止对应执行权；已完成的合法历史事实保留。必要资源清理需要自身明确授权，不从目标完成或到期自动推出删除权。

适用权限 = 一个可核查来源授权链 ∩ 当前 WI 的更窄 scope/限制 ∩ 当前 Runtime 动作准入 ∩ 既有安全与资源协调规则。多个互不覆盖的 grant 不可拼接成更大的单次操作权限。范围、期限、actor 或限制无法确认时，明确 missing/unknown，不能推定授权。

## 2. 最小模型：一个目标来源，每个 WI 明确引用

不新建重型 IAM、角色注册表或预算体系。复用现有 Contract、AuthorityEvidence、HumanDecision/receipt、typed amendment 和 coordination。补足的是 **start 的 authority 声明与可核查来源/具体意图覆盖之间的绑定**，不是重做已有 repository/WI/Contract receipt 校验。

| 所属 | 内容与作用 |
| --- | --- |
| 目标授权来源（唯一权威记录） | grantId；稳定 subject/goalRef；明确目标、交付范围和排除项；真实授权者、可验证依据；允许的 actor/action/resources/constraints；notBefore/expiresAt；版本、撤销/取代前驱和目标终止事实。使用现有权威来源与不可变决定记录承载，精确存储形状在协议阶段定稿。 |
| 目标授权确认 | 绑定上述完整语义、来源版本及授权者实际身份的现有批准凭据。AuthorityEvidence.actor / authoritySource / assurance 字符串只是元数据，不是认证；不能由 agent 摘要替代真人或 provider 依据。 |
| WI Contract | 显式记录来源 grantId/subjectRef/当前版本解析规则与本 WI 的更窄交付范围、scope/排除项、actor/action/resource 限制和可选更早截止。由 Runtime 验证明确覆盖依据，不从“前一个 WI 已批准”推导。不要复制一份可独立修改的上层批准成为第二 SSOT。 |
| 派生/执行证据 | 记录源授权实际版本 digest、当前 WI/Contract、真实执行 actor/委托链、Runtime、当前 snapshot、动作/资源、准入检查与真实开始/结束时间。它证明这次引用和执行，不创造新的目标授权。 |

稳定目标标识优先复用已批准的目标/计划来源身份；不为每个任务另造 taskInstanceId。若来源目前只有自然语言而无可绑定身份，须在正式批准目标记录时确定稳定 subjectRef 和交付边界，不能由 agent 随后自报“同目标”。源记录的存储和当前版本解析复用现有 journal/decision 索引；不引入平行的可变批准数据库。

repo 绑定使用规范 repositoryId；WI 使用真实 workItemId 和自身 Contract digest；资源使用规范 provider/resource ID 或受约束的 repository-relative path/ref。路径拒绝逃逸、.. 与编码歧义，实际 I/O 前按现有安全句柄/身份规则重验 symlink/路径替换。action 使用已知操作语义，不以任意 shell 字符串冒充授权动作。资源通配只有来源明确允许且可规范验证时适用。

## 3. 连续工作与派生覆盖

授权来源应明确允许的任务集合/交付项及连续、拆分、并行、正常修复、重试、恢复规则。派生 WI 必须引用该目标，声明自己覆盖的已批准交付/修复关系，并证明资源、操作、限制与期限均不超父范围。仅有相同 goalRef 标签还不够；需要可追溯的目标分解/交付覆盖事实，不能只凭 agent 称为“必要修复”。结构无法证明意图覆盖时，给出具体差异请求范围决定；不假称能由 NLP 保证任意自然语言意图等价。

同一来源明确覆盖的 B/C 可连续或并行推进，无新增逐 WI 人审点；实际 actor 来自可信 host/provider 执行上下文，不能从请求字段自封。子 agent 只能收窄父 actor 委托、范围、限制和有效期；不具备再委托规则时不再派生。代理技术审阅必须记录为代理技术审阅，不能冒充真人。

新目标、操作/资源扩大、触及明确排除项或必须真人决定的动作，走正式变更/新授权。只换 WI 编号、正常阶段切换不会自动增加审批。目标完成事实来自适用的目标验收/终止凭据，不能以 agent 宣称完成替代；某个阶段 WI 的关闭也不自动宣告整个目标完成。

并发仍走现有冲突、依赖、边界和 lease/admission。来源有效不等于文件无冲突。若既有约束含总额/并发额，必须在现有共享原子记账点消费，不能每子 WI 复制额度；没有真实记账/协调能力时报告 unsupported 或采用已准入串行，不伪造 lease。本阶段不为此新建通用预算系统。

## 4. 时间与 Clock

统一可注入 Clock，返回绝对 UTC 观察和进程内 monotonic 观察；比较使用整数时间精度，不以向上取整获得宽限。规范化 notBefore/expiresAt 为 UTC，并保留来源显示时区/原偏移供人核对；不同偏移但同一瞬间的边界一致。

每次执行建立实际剩余授权预算，截止取：原单操作上限、已有政策上限、父/子授权剩余时间中的最小值。进程内 monotonic 剩余预算只递减；绝对时钟向前跳应立即缩短/终止。已知时间下界配合 monotonic 防止进程内回拨延长窗口；明显回拨/不一致返回 clock_untrusted，停止新业务动作，不自动忽略或加宽宽限。正常精度/校时容差只用于诊断可信度，不能延长 expiresAt。

重启后重新读取当前来源版本、期限和保留的时间观察，不能重用旧进程的剩余秒数。保留记录存在时可发现部分回拨；**纯本地系统时钟与记录整体一起回滚无法可靠检测**。本方案不默认新增远程授时、硬件信任或系统权限修改，清楚呈现 local-clock assurance 能力边界。

## 5. 动作边界重查

计划检查之后，在锁后、实际副作用前、重试/恢复、不可逆提交前重新解析当前来源版本、主体终止/撤销状态、actor、目标覆盖、资源、期限及现有 Runtime admission。拿锁前的许可和缓存 receipt 不得延长授权。

同一个受管动作绑定当前 grantVersionDigest + WI Contract digest + snapshot + actor/operation/resource。锁等待跨过 expiresAt 必须在副作用前拒绝。当前授权变化、并发 typed amend、撤销或旧版本重放不能沿用旧许可。继续保留 Runtime 的业务门禁、可信身份来源及现有安全/Provider 限制，不以“有目标授权”绕过门禁。

## 6. 在途到期与停止

启动前须声明操作类别及安全停止策略：可取消操作监视有限截止并停止自身受管进程；事务操作使用预声明的有界回滚/一致性策略；不可中断动作须有可信最大成本/安全点，剩余时间不足时不启动，或申请足够窗口。到期不能再启动新的业务步骤/不可逆提交。

必要一致性收尾只允许事先声明、独立有界的安全操作，不借“收尾”继续业务。最小监督停止能力只管理自己已启动的受管进程及已有输出；读新业务数据、外传、删除外部资源或清理都不由到期自动获得。若确需清理/补偿业务资源，必须有自己的明确有效授权。

无法可靠中断的外部操作可能在到期后返回。保留真实 actor、开始/完成时间、外部 receipt 与实际效果，注明 deadline overrun/未知结果；不得伪造停止、倒填时间、追认越权或把后续业务自动算入收尾。Runtime 只能保证其受管入口/进程，不能保证控制所有外部 agent 或已提交的 provider 事务。

## 7. 到期控制面与统一延期

到期仍可在已有可信管理身份/最小安全能力下读取本 WI/授权的最小状态、保全已捕获证据、停止自己受管工作、准备/提交延期申请。此控制面是固定 metadata 路径和已有输出，**不授予新业务数据读取、外传或删除**。当前普通 status/typed-amend 会读取 source/material；实施必须提供 metadata-only 观察/请求路径，不能直接把完整业务扫描标成控制面，防止“申请需要授权、授权又已到期”的互锁。

预计超期时提前统一申请，包含剩余任务、原因、原截止、新截止、scope/限制差异。目标范围未变只审期限差异；变范围必须拆出独立变更决定。申请本身不是续期，等待时旧期限内可依原范围继续，到期即停；无自动续、默认同意或自我追认。

有相应权限的授权者通过正式 typed amend/版本化决定批准新来源版本，严格绑定 expected source digest/current version；并发申请/批准、重放、旧 digest 拒绝。关联的适用活跃 WI 在下一次准入统一解析新版本，保留各自更窄限制和更早截止，不逐 WI 再批准。旧执行 receipt 仍是历史，不授权新动作。已关闭、撤销、取代的 WI/授权不因统一延期复活。

延期控制面只需当前目标授权/请求的 metadata 绑定；若到期已禁止 source 读取，记录 last-observed snapshot 为历史或显式 not-observed，不假造当前 snapshot。批准新期限后，业务恢复才重新获取受授权的当前 snapshot 和完整准入。授权者本人的可批准权限与过期业务 grant 分开验证，不能用过期 agent 权限批准自己。

## 8. 兼容与迁移

历史合法批准和成果不因当前到期而失效；未来执行权与历史真实性分开报告。旧 Contract/receipt 仍能只读解析，绝不补造当年的授权者、期限或批准。新强制模式缺来源或有限期限时给 missing/unknown 并禁止受其管理的业务执行，不把旧 authority=authorized 当无限授权。

上线开关、当前活跃旧任务如何取得新有限来源、授权者认证、版本/索引存储、独立管理能力与安全收尾边界均另行审阅。此次 planned WI 的 authority=missing，不是实际配置任何目标 grant，也不改动 WI-1086 的现存授权或执行条件。

## 9. 复用机制与代码接入

以 main 681df30e2fe8c869b6a41498822818c017dc4fb6 的实际源码为设计基线；集成前先依赖 WI-1086 修复与最终 Runtime 审阅。

| 现有机制 | 复用与需补部分 |
| --- | --- |
| protocol::AuthorityEvidence / validate_authority_evidence | 复用来源、actor、operations、policy/evidence refs；现有验证仅校验元数据形状，不冒充外部认证。新增目标/期限与来源引用类型需技术审阅。 |
| repository action_admission / status_projection | 复用新鲜动作集合和当前解释；增加授权来源/有效期的独立交集及纯 metadata 控制面，不重造全局门禁。 |
| lifecycle start/scaffold / typed contract_amendment | 复用新ID保护、前后继、append-only journal、expected digest 和正常准入；在 start 将 authority 声明绑定可核查目标来源，在延期写当前源版本。 |
| governance_controls / preflight receipt | 复用 repository/WI/Contract 与本人/代理区分、同 Contract 合理批准复用；不以换 WI 强制人审，也不接受旧目标批准。依赖 WI-1086 的真实绑定修复。 |
| existing coordination / leases | 复用冲突和依赖判定；无已实现能力不宣称可并行。 |
| verification execution / existing Instant timeout | 接统一 Clock/剩余有效窗口及受管停止；900秒操作上限独立，保留真实效果和失败记录。 |
| CLI / MCP | 同一核心判定、同一失败码/解释与确定性 Clock fixture；host 实际身份能力如实呈现。 |

具体阶段和精确建议文件列于 implementation-stages-v1.json。当前 Contract 只允许自身设计/治理路径，以下实施 scope 只是提案，**不是已授予的产品编辑权限**。

## 10. 实施顺序与待决项

先完成此 planned 设计及技术审阅；WI-1086 集成后再按正常准入分阶段实施。Clock/类型新模块仅在接口定稿、真实 ownership 无交集且明确允许时可单独推进；Contract amendment/status/admission/receipt/lifecycle 与 CLI/MCP 集成阶段串行。当前声明 parallelizable=false，依赖并 conflictsWith WI-1086；不申请/伪造资源 lease。

技术定稿尚需决定：①权威来源的精确 wire/存储与既有 source 索引复用；②host/provider 可提供何种真实身份与 assurance，目标语义覆盖不能验证时如何呈现；③统一 Clock 的回拨诊断阈值及可证明的本地时间能力；④metadata-only 申请和独立有界安全收尾的最小权限；⑤新强制模式激活、旧活跃任务迁移/回退与版本能力要求。期限必填、无自续、无逐 WI 重复审批、旧目标不得冒用为新目标均已确定，不作为可选项重开。

验收矩阵见 acceptance-v1.json：安全与效率并列，全部状态 not_run，使用可控时钟，不长 sleep。产品实现及新权限启用另经正常准入和 Raydot 技术审阅。
