# WI-1086：真实收据负例 RED 与最小扩展请求

## 真实结果

原生正常 start/boundary/declare/preflight/checkpoint 已完成。唯一 before-edit Contract 为 `sha256:36b1a87c47d85caedd16259c4b685ce0097ddbf127cbceb4e53b520f0516c734`。建项提交 `8aeb8e17ac71290a980dff0a1aaa736320ed6f54` 已推送读回；提交后的 Runtime status admission=allowed，humanDecisionRequired=false，run_verification 明确列为 safeAction，admission digest `sha256:df05f13cdac7b67a6577a6598946f0abbfafc09c8151a2fb4d72fc6d2be0b161`。因此 RED 不是在旧 WI-1085 blocked 上下文自修。

在该许可后，仅向已批准 `crates/cockpit-cli/tests/contract_amendment_processes.rs` 添加两个隔离临时仓库负例；生产代码一字未改。真实 CLI preflight/checkpoint、native typed amendment 成功后，保留 native Summary 的旧 preflight 字段，构造新 Contract digest 与旧 preflight digest/snapshot 混合的负例输入。**没有手改 fixture Summary 或 journal**。CLI 和 MCP controls 都错误接受并写入 decisionEvidence。

Cargo 命令：`cargo test --locked -p cockpit-cli --test contract_amendment_processes mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected -- --test-threads=1 --nocapture`。结果 **0 passed / 2 failed / 5 filtered out，exit101**；耗时68.424秒，未超时。两个失败均是“必须拒绝却成功”，不是编译、schema、参数或 fixture 错误。原始 stdout/stderr/meta 与派生精确字段均在本目录。

两个用例：

- `mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected_by_cli`
- `mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected_by_mcp`

`red-mixed-binding-facts.json` 证明两例均 `preflightState=not_run`，新 receipt 的 contractDigest 等于 native amendment 的当前 Contract，且不等于 Summary.preflightContractDigest。MCP 返回 isError=false，CLI exit0。所有 human fixture 字段只存在于隔离自动化临时仓库，不能当作 Ray 或真实 WI 的批准。

## 定位与最小范围

`governance_controls.rs::validate_preflight_decision_evidence` 校验 evidence.contractDigest 与实际 Contract，却分别将 preflightDecisionDigest 和 repositorySnapshotDigest 与 Summary 保留字段比较；没有验证 Summary 的 preflightContractDigest 是当前 Contract，也没有拒绝 native amendment 后的 not_run。故旧字段可与新 Contract 混搭，原生 writer 会发布收据并写 Summary。

请 Raydot 审阅只额外加入一个产品文件：`crates/cockpit-repository/src/governance_controls.rs`（scope、implementationPaths、serializedProjectionPaths）。现有 CLI 测试文件已经在 scope 内，无需新测试路径、协议、schema、公共参数或政策变化。

最小修复：新收据写入前要求真实已记录的 preflight 状态，而非 not_run；其 preflightContractDigest 必须匹配当前 canonical Contract，再沿用原有 repository/WI、decision、snapshot、时间/actor/reason 和 append-only 检查。不得一刀切只允许 green/yellow，因为 A/B 需要保留原 Red/blocked 的真实请求。保留当前同 Contract 的既有政策批准复用语义；源码快照改变所需的 fresh preflight/verification 不应被改成重复人审。此 RED 不证明额外的 snapshot-drift 或全字段攻击都已覆盖，后续既定负例照常保留。

验收：这两个真实 RED 在 CLI/MCP 均变为拒绝，并验证 Summary、Contract、journal、旧收据字节不变且不发布新收据；原生新 preflight 后的合法当前绑定仍可记录，原 receipt/普通 flow/retry/锁等待回归保持。之后继续 A/B 全矩阵和声明验证，不能删除负例、放宽断言或以 focused GREEN 冒充正式回执。

## 当前停止点

按 Raydot 指示：真实负例暴露额外校验器缺口时，先交具体 RED 和最小扩展范围。**A/B 与 validator 产品实现均未开始**；待审 typed amend request 已保存但未应用，新 Contract/preflight digest 尚不存在。本技术请求交 Raydot，不重复向 Ray 请求同一政策批准。Runtime、原 WI-1085 与旧资源均未改；无 PR、合并、关闭、清理或实际 Sentinel 操作。

编译工具链 rustc1.98.1/cargo1.98.1；隔离 target、单 build job/测试线程、900秒上限。候选 CLI 路径及实际 SHA-256 见 meta；它恰与保留 Runtime 字节相同，不据此宣称已采用候选。环境21,316个继承僵尸保持稳定，workspace约3.22GB、tmp约2.29GB可用。
