# WI-1090 v4 验收夹具矩阵（均尚未执行）

| 组 | 必须使用的输入/动作 | 预期与必要断言 |
| --- | --- | --- |
| F1 真实历史只读重放 | 真实旧 receipt fa5e765f、R=17c4100f、全部三组原生 prepared/committed、必要 activation 证据、R 与 A 的真实审阅、真实 amendment3 的 scope/acceptance/scenario/sources 变化 | 保留全部原始 bytes/digest；H 可建立但绝不代表当前 discharge；B 尚不存在时不得宣称 P 或 replacement 成功 |
| F2 原生分段正向链 | 原生生成旧四节点 canonical receipt/材料决定、实际 blocked finish/Outcome、合法多字段 R→A 且 profile 不变、所需真实 fixture review、以合法叶节点 set 和整对象 governanceProfile replace 分别生成相邻 source-only B、当前 fixture 批准与实际新 reviewer input | projection 与锁内 record 同一有限 eligibility；旧 reader 在 B 仍 invalid/no discharge；新决定与本次完整 request 绑定，成功提交后旧 sidecar unchanged、旧 pointer history 恰一次 |
| F3 完整生命周期 | 在 F2 的同一根/同一明确来源链上使用同时含 ba1+WI1090 的实际候选；读取当前四节点计划并实际执行 | plan 进程0；执行4/reuse0/max1；逐节点实际 exit0/no timeout；新鲜绿色 preflight；fixture 实际 finish0/archive0；旧 canonical/attempt/Outcome/receipt bytes 全保留。不得以 Missing、单节点、all-reuse 或只设置 digest 替代 |
| F4 历史反例 | 缺/损坏/外来/重复键/符号链接 receipt、pointer、prepared/committed/activation；断链/分叉/pending；缺必要历史 review；未审阅 scope；R→A 禁止字段或 profile 修改再还原 | fail closed；不写新 prepare/receipt/Summary，不删除旧证据；不能通过重新计算攻击者 digest 变成正向样例 |
| F5 相邻反例 | Summary 自报 a5、跳过 transition、较早非相邻锚点、未 committed 的 B、root replace 夹带 reviewer/assurance/risk/scope/sources、混合/还原/no-op 请求、换绑后来源变化 | P 拒绝；使用逐操作语义差异；正常 `/governanceProfile replace` 仅来源变化必须是正向对照；旧整对象 set 必须由协议先拒绝，不作为产品正向样例 |
| F6 当前结构反例 | 错 request/Contract/profile/policy/scanner、blob/hunk 改变、Finding、其他 Unknown、dirty/unavailable source、缺/过期 B 原生审阅、错 reviewer/source/assurance、无效 evidenceRefs | C0/C1 拒绝；投影中的输入未提供阶段不假装验证 C1；保留全部原生 blockers 与 invalid 诊断 |
| F7 决定来源工作流 | 当前批准/实际审阅材料须覆盖精确当前 request/member/risk；对旧5b回复、Agent JSON 或裸 confirm 的误用做明确人工审阅负例 | 如实报告未满足并停在决定步骤；不宣称格式合法 evidenceRefs 认证真人。该项是工作流审阅证据，不伪称仅靠现有 schema 的 Rust 测试可认证人类 |
| F8 竞态与原子失败 | projection 后及锁内校验时改变 HEAD/material/review/source/pointer；prepare 前后、sidecar 前后、Summary commit 前后注入真实私有 failpoint | 未提交残余不能 discharge；精确 pending resume 成功一次；错 pending、错输入、完成后 replay、CAS 冲突无重复历史；对不确定写先只读分类 |
| F9 兼容 | Missing/Current/same-profile Stale 原路径；R=A 的 source-only 正向；同 profile 续期；既有 repo status/CLI/MCP 套件 | 不扩大通用 Stale/Invalid 豁免；不改 schema/状态机 allow-list；原有 public 错误与 strict receipt 语义保留 |

F1 输入索引：real-history-input-index.readonly.actual.json。它列出的真实原始收据 SHA-256 为 81938787c43abb057ca587cc861c8725c3cab36eba4082cfefe3c7a964639a2a；amendment3 prepared 为 9ae86b0f83f32a8390c6e1fdd552fbbc7e92e884a0a4e60c5ead0f0f791384b5；committed 为 8fbb1788c82091141223dcb75bc5b027190e7abdff73de88b228c9ccc0bb0cef。R 审阅 raw SHA-256 ba2a5b80cc930f5c933e23b69be913b466d3fcf13f6ddb253860102ee25cb125；A 审阅 d4050c09726d434b76ca2e22356df4e1c9c945e32ab4275358da13dcacebabc7。

七项现有正式 verification 命令不改。F1/F2/F4/F5/F6/F8/F9 由两份既定 Rust 文件内测试及现有 CLI/MCP 回归承载；F7 保留真实审阅工作流证据；F3 在另外已批准的外部治理夹具中以真实候选执行，不新增第三份 Rust 文件。单 worker、隔离 target、每个外层命令不超过900秒。遇到真实超时、错误或未覆盖项目须保存并报告，不能把矩阵勾选当测试结果。

所有 synthetic review 必须明确 actor 为 fixture，不伪造 Ray 的原话/身份；真实冻结证据只能只读使用，破坏性负例在隔离副本内执行。本文件没有运行夹具的效果，也没有启动任何验证。
