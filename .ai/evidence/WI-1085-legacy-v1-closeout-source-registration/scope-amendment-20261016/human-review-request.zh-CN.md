# 待真人决定：新 Contract 的原生 preflight

此文件是待答请求，不是决定或审批收据。Raydot 已批准两份测试的技术方案；这里仅因 Runtime 要求新身份绑定的真人 review 而暂停。

- repositoryId：`sha256:ee02a04ca242d830086432bd4d3f81602505371269852721ee83e117e35da22b`
- Work Item：`WI-1085-legacy-v1-closeout-source-registration`
- Contract：`sha256:7fea3f7d77255e61459ff1c51ac46ce9e9b73ff4a627939ad3b844d96ed093a1`
- repositorySnapshot：`sha256:6bcd443d4cdb6326a937021ef096717ef1beb5a6b7375456be93bb500ed818a7`
- preflightDecisionDigest：`sha256:7b2ccbca2b332f533676c4f3d3e04ac3d83ec9df916977db582212b8a90b3c72`
- native decisionId：`contract-preflight-review`

变更仅为将 `crates/cockpit-mcp/tests/material_review.rs`、`crates/cockpit-mcp/tests/usage.rs` 加入 scope 与对应 implementationPaths；其他 Contract 值不变。测试尚未修改。旧 Contract eae15cbb… 的真人收据不可复用，Raydot 技术批准也不冒充此真人决定。

Runtime 实际选项为 `confirm_review`、`complete_contract`、`stop_work`；推荐 `confirm_review`。须收到真实、明确绑定当前事实的决定，才能通过原生 controls 记录并继续；本请求不自行记录确认。
