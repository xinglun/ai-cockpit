# WI-1090 分段材料决定替换设计 v4（仅提案）

记录者：agent:Codex。状态：等待 Raydot 技术审阅；未执行修约、checkpoint、实现、构建或验证。

## 判断与边界

该方向可以保持 WI-1090 的目标：只为一项全新的当前材料决定提供有限准入，保留旧收据无效、Unknown 不消除、严格原生授权、不可变历史及可恢复的逻辑提交。它需要修改现有 Contract：v3 将整个旧收据到当前 Contract 的链限定为 authoritySource-only，而真实消费者的历史中已有合法且已审阅的 scope/验收变化。仅修改实现会违背现有验收和 outOfScope。

本提案把证明分成 H、P、C，三者不可替代。H 证明历史完整；P 证明最后一次相邻变更只有来源换绑；C 证明当前材料、授权与新决定。H/P 同时成立也不能使旧收据成为当前有效收据。

拟保留原两份 Rust 文件与六份治理文档的八路径 scope、high 风险、base b8ec269、七项 verification、现有 protocol/API/CLI/MCP schema，以及固定治理 Runtime b5cbd857。不修改 status_projection.rs。生产代码不得硬编码本 WI、Ray/Raydot、17c/a5 或此处路径与 digest。

## H：旧收据 R 到换绑前 A 的历史证明

1. 从当前 Summary 的旧 pointer 定位不可变 sidecar。按既有 bounded/nofollow/重复键拒绝规则验证其原始 digest、canonical receipt digest、repository/WI/protocol、原始 request/profile、pointer/history 身份及 source ancestry。损坏旧收据不能经替换通道洗白。
2. 调用原生 amendment history loader 验证完整 journal，包括 prepared/committed 对应、请求与 trace、activation continuity/recovery、序号及当前 tail。任何 pending、断链、分叉、外来记录或无法解释的直接编辑均拒绝。再从已验证的 prepared 原始 Contract 中定位 R 和 A；不能凭 Summary 标记声明一个锚点。
3. R→A 的有限历史段允许且仅允许已由原生政策和所需真实审阅覆盖的 `/scope`、`/acceptanceCriteria`、`/scenarioCoverage`、`/sources` 变化。四项是本次为真实消费者提出的明确边界，不是任意历史变更豁免。profile、risk、base、authority、intent、goal、outOfScope 等其他语义变更不在本通道内；包括中间修改再还原也拒绝。原生 activation 证据按其既有严格格式验证，不被当作业务字段扩权。
4. R→A 的整个 review profile 必须逐操作保持相同。R=A 是允许的退化情况，保留原纯来源换绑用例。
5. 每个必要审阅按它自己的 Contract 与原生记录上下文核验；不能要求旧审阅使用当前 B 的 snapshot，也不能把当前 Summary 的 decisionEvidence 当作全部历史的证明。缺失所需历史审阅时给出有限诊断并拒绝，不补造旧决定。
6. `PreparedAmendment.previousSummaryDigest` 只有 digest，没有完整旧 Summary。它可作已有原生关联事实，不能凭 digest 重建不存在的完整旧 Summary 或提高历史认证强度。历史审阅的保证不高于现有原生 self_declared 模型。

现有旧 receipt 只保存 requestDigest/materialManifestDigest 等摘要，并不内嵌完整历史 plan 或 blob/hunk 列表。H 校验原始 receipt 自身完整性及其当时的原生身份，不宣称能从摘要重建历史材料真相。需要额外历史材料时只能消费实际保留且可核对的原件；缺失就报告未知，不能用当前 request 拼成旧 request。

H 的输出是历史身份及链证明，不是当前审阅通过。旧材料风险接受、旧 reviewer 决定、旧 source snapshot 和旧验证结果均不向 C 继承。

## P：相邻 A→B 的来源换绑证明

1. B 是当前 active Contract；A 是最后一条已提交原生 amendment 的真实 previousContract。必须直接相邻、唯一且位于已验证 tail。不得从任意较早 digest、Summary 自报标记、未提交 prepared 或推算的新 digest 选择 A。
2. 重放该 prepared 的实际 request，核对每步 trace、原始 A/B Contract 与 committed receipt。逐操作计算语义叶级差异，唯一允许的真实变化是 `/governanceProfile/materialInspectionReview/authoritySource`。
3. 当前原生协议的 `set` 只接受标量叶节点；有效输入优先使用 authoritySource 叶节点 `set`，或对已存在的 `/governanceProfile` 使用整对象 `replace`。证明必须支持后者，不能只按字面路径拒绝。整对象 replace 中夹带 reviewer/assurance/acceptResidualRisk/其他 profile 或顶层字段变化必须拒绝。多步修改再还原、夹带其他路径的 no-op 不构成合格的单一来源迁移；要求一次有实际旧新值差异的来源变化。
4. 除该叶节点外，A/B 的 profile、源范围、policy/risk 和其他 Contract 语义必须相同。source-only request 不能顺便增加 source 文档、scope 或验收字段。
5. 当前 request 的干净、已提交材料与该相邻原生记录捕获的来源上下文相符。repositorySnapshotDigest 与材料 sourceSnapshotDigest 属于不同派生上下文，须按各自原生语义核对，不得直接要求两个不同领域的 digest 字符串相等。此条件针对 A→B 的当前材料，不要求它等于 R 的历史来源。换绑后代码或材料成员再变化，须重新证明当前授权，不自动扩展。

P 不撤销 strict reader 的 profile mismatch。B 下旧收据仍报告 invalid、仍不消除 Unknown。另有有限的 new-decision eligibility，不能 catch 任意 read error 后放行。

## C：当前授权、材料与实际新决定

C0 供 projection 和锁内 record 共同使用：当前 opt-in/profile、有效的当前原生 Contract 审阅与 admission、干净且已提交的可读来源、完整当前 plan/request、确切 sourceSnapshot/manifest/profile/policy/scanner 与所有 reviewable blob/hunk 身份、无 Finding、只有 permitted reviewable syntax Unknown。所有其他阻断继续有效。projection 只可呈现“可以提交新决定”，不能预先声称尚未提供的 reviewer input 已审阅。

C1 仅在 record 时验证：新决定的实际 current requestDigest、reviewer、authoritySource、self_declared assurance、evidenceRefs、rationale 和 residualRisk。H/P/C0 在现有 lifecycle lock 内重新读取并计算；C1 必须是这次实际提交的决定，不能由旧 receipt 构造。拒绝错误 request/member、旧来源冒用、错误 reviewer、Finding、其他 Unknown、dirty/unavailable source 或过期原生授权。

这里必须保留认证边界：当前 `evidenceRefs` 只要求非空路径、合法 digest 和唯一性，不证明文件内容存在、更不证明作者是真人。此修复不引入隐含的人类授权 JSON schema、签名认证、真人解析器或 Ray 专用字段。新决定必须如实引用不可变的实际批准与审阅材料；操作者/审阅者须核对该批准确实覆盖当前精确成员和风险，并记录实际来源。若要求机器验证新 grant 的语义/身份，那是额外协议需求，应先修订设计与授权，不能在本范围内宣称已实现。

消费者当前真实 Ray 批准仅绑定 a5/5b 与三项既定 blob/hunk。该记录可保留作事实，不能改写成尚未生成的 B/request、组合候选或新扫描结果的批准。换绑后先读回完整真实新绑定；如果需要新的 Contract 决定或当前风险接受，由授权的 Raydot 协调实际决定并保留原话与身份。Contract confirm_review 和材料残余风险接受是不同事项。本线程重复的裸 confirm_review 不预先覆盖 v4 或未来消费者 B。

## 实现接合（不新增公共 schema）

只在 `crates/cockpit-repository/src/material_review.rs` 内新增私有 typed eligibility。正常 reader 的 Missing/Current/Stale/Invalid 语义保持；当前 profile mismatch 仍 invalid。私有有限 proof 使用已验证历史及 bounded 原始 prepared 数据，而不是把旧 reader 传入虚构 A 上下文以跳过当前 tail 校验。沿用原生 replay/Contract 类型与现有校验能力，不复制一个弱化的 journal validator。

现有 projection 消费 `material_review_decision_available`，因此 material_review.rs 返回有限 eligibility 即可；保持 status_projection.rs 的生命周期、blocking、preflight-binding 等现有谓词。record 在锁内重算同一 typed helper，再验 C1。不能递归调用完整 status 产生循环。

实现前如发现现有内部 API 无法在这两份 Rust 文件内验证历史审阅上下文或原生 prepared 关联，先向 Raydot 报告具体缺口；不得借此弱化证明、增加未批准文件或第三个 WI。

## 逻辑原子替换及恢复

H/P/C0/C1 全部通过之前不发布 prepare、receipt 或 Summary 写入。既有 lifecycle lock 的获取不是新材料证据。所有失败均保留原 pointer 与历史。

按顺序发布：不可变 sealed prepare → 不可变新 receipt sidecar → 一次原子 Summary pointer/history commit。prepare 绑定 H/P 的原始记录与 digest、当前 request/审阅/输入、runtime、旧 pointer、当前 B 的完整 Summary 原始 digest、新 receipt bytes 和目标 Summary。这里的 CAS 使用本次当前 Summary，不是虚构的历史 A Summary。

原子可见性以最后一次 Summary commit 为准；没有跨多文件物理原子性承诺。提交前残余 prepare/sidecar 不产生 discharge。精确 pending resume 只能使用原 sealed bytes，重新校验当前所有身份与前置状态后最多提交一次。冲突 pending、已完成 replay、错输入或无法解释的当前 Summary 均拒绝，不重复 history、不删除残余。写入结果不确定时先原生只读区分 committed/pending/conflict，再决定精确续接；不能盲重试。

旧 sidecar 原始 bytes 保留，旧 pointer 追加到 history 一次。测试 failpoint 仅限私有 cfg(test)，不新增公开绕过参数。

## 当前真实链、验收与依赖顺序

消费者 HEAD ba1c6c90f4ac039521567c1365b4351fe7ce92ae；R=17c4100f053157817aa64257c2177e4c549e3f451ee03ecb7bf765957d864b07；A=a5deac9a750543678deff0e444da6e2924976423fb21f419250692c768fc4a69。真实 amendment 3 的四类变更不能从夹具删去。B 尚未生成；旧 source-only 提案 SHA-256 44881b1a1b3feafb3c36f0743b9debdce18d34bb3bdb7765cb8db4100b9af637 从未执行。它使用整对象 set，按当前协议会被 set_requires_leaf 拒绝；这是源码读回推断，不是已执行的原生失败。保留原件。后续必须由 Raydot 审阅修正后的标量来源叶节点 set（或语义等价的整对象 replace）精确参数，不能直接执行旧文件。

必须先有真实形状 regression：旧四项 canonical verification、旧材料收据、实际 blocked finish/Outcome → 原生多字段 A 与必要历史审阅 → 原生相邻 source-only B（使用协议有效的叶节点 set 或整对象 replace）与当前实际授权 → 新材料决定 → exact current plan → 四项 fresh execution/reuse0/max1 → 新鲜绿色 preflight → fixture 实际 finish/archive。不能以 Missing、单一 true、全部复用或省略旧 canonical receipt 的夹具代替。

真实消费者的 frozen 只读重放和 Native 产生的新合成链分别报告。合成 actor 明确使用 fixture 标识，不冒充 Ray、Raydot 或 Sentinel。真实同内容双 producer 组合验收仍需独立候选同时含 ba1 修复及完成的 WI1090；base b8+1090 单独通过不能证明消费者 blocked-Outcome 已解决。此前未发布的 fixture07 payload 及变体继续留在本地，不作为本提案公开材料。

顺序：Raydot 审阅本包 → 必要时一次原生 WI1090 修约 → 读回新实际 binding，完成真正需要的 human review → native admission/checkpoint 后实现和七项正式验证 → 审阅独立组合候选及准入 → 才考虑消费者 source-only 修约、新当前材料决定与后续 A5。没有一步自动部署新治理 Runtime、操作 Sentinel、结束真实 WI、合并、关闭或清理。

所有新方案文件目前仅在外部 scratch；原 worktree、Contract、Summary、旧决定、批准、CI 和 c960/420 历史均保留。详细验收矩阵见 FIXTURES.proposal-only.zh-CN.md；原始输入索引为只读事实，不是新授权或通过结果。
