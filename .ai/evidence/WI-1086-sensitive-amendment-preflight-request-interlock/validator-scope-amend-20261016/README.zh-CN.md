# WI-1086：正常 typed amend 已应用，真实准入待办

日期：2026-10-16（Asia/Tokyo）。原生日志保留实际 UTC 时间及原始字节。

## 已完成

按本轮 Raydot 技术批准，原样应用已提交请求 `bc012d4cd1c9413f5835f05658969b8f77bdbea45c576bed237773ca764b3a71`。原生 work-item amend 退出0、sequence1，仅追加 governance_controls.rs 到 scope、implementationPaths 和 serializedProjectionPaths。Runtime 自动填入 requiredRuntimeCapabilities，并按类型序列化补齐7个默认空数组（逐项见绑定JSON）；没有额外政策/验收修改，checkpointCount保持1。旧请求、原 Contract 历史和两个真实 RED 均保留。

随后 status 明确允许 run_preflight，遂通过同一受支持 Runtime 执行正常 preflight。退出0并生成真实当前 Contract/decision/snapshot 绑定；没有使用旧字段、混绑负例或无 Runtime 入口。

## 精确绑定

- Contract：`sha256:5747b1fee5d38a8546a75fecaab50493344fd078adc79dee32d15ab2a44be119`
- preflightDecision：`sha256:e99fb57e73cfd14fffb5de5447ab433fb9d4faa6d33180fdc9ffd8712f56bedd`
- repositorySnapshot：`sha256:411c0eea3b82e9017791d2d2ba926fa99db04c2792b3ed82bb78a3f68c6499b2`
- amendment journal：`sha256:8db8892a01479bc6143b721c0321527d998b568405675a80fdcd0a0a9dbd5bee`
- Runtime1.0.1：`sha256:5307ba5324d397e0fb971aa8b926ba8176336a0a6e76dfae7bd3a6e5cd11fa38`

## 当前停止原因

原生 journal 将 /scope 与 /concurrencyBoundary 判为 sensitive，policyReviewRequired=true。新 preflight 为 yellow、needs_human_confirmation、outcome=needs_human_decision，返回真实 contract-preflight-review 请求；Unknown 含 contract_amendment_policy_review_required。当前 decisionEvidence 缺失，新 Contract 没有可复用的有效批准。

最终 status 为 needs_human_decision，仅允许 record_governance_controls / record_usage / refresh_status，没有 run_verification。AGENTS.md 要求遇到缺少授权或真实人审条件时停止，本轮也明确要求实际准入允许时继续。因此不开始 A/B 或 validator 产品修改，不继续跑测试或正式验证，不把 scoped 技术批准写成 confirm_review，不以混绑漏洞解锁。

交 Raydot 的剩余事项是这个真实身份绑定准入，不是重新审阅已通过的技术范围。完整 native humanDecisionRequest、admission digest 和原始输出均在本目录。这里未提交真人确认，未向 Ray 重复请求同一政策批准；协调方需判断受支持的合法继续路径。

## 准入允许后的实施约束

沿既有真实 RED 修复 validator：preflightContractDigest 必须等于当前 canonical Contract；preflightState 仅接受 green/yellow/red，missing、未知、not_run 一律拒绝。拒绝时不修改 Summary、Contract、journal、旧 receipt，不发布新 receipt。原 Red/blocked 的合法当前请求仍须允许记录 review；同 Contract 有效批准复用语义保留。

随后按 v2 实施 A 的完整原生 journal/current invalidation 绑定，以及 B 对 sensitive 前置分支、Missing/Invalid 路径的 state/outcome_state/blockers 保持。完成 CLI/MCP、兼容回归及所需验证，单 worker/build job、隔离 target、900秒上限。当前两个 RED 保持失败历史，尚未修复。

原 WI-1085 和旧 control/review 资源保持。无实际 Sentinel、候选 Runtime 采用、PR 合并、真人关闭或清理。
