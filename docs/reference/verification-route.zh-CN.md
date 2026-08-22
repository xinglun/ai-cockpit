# Verification 路线

AI Cockpit 将验证事实经过显式阶段、策略计划、依赖/受影响图、执行、证据回执和 CI 边界。

## 阶段

阶段固定为 `task`、`pre_ci`、`pr`、`merge`、`release`。`pre_ci` 是本地反馈，
`pr`、`merge`、`release` 仍属于独立 provider 或受保护门。未知阶段直接 fail-closed。
阶段不是 assurance 等级。

## 两个正交维度

`VerificationTier`（`T0`–`T3`）表示验证强度；`EvidenceAssurance`
（`SelfDeclared`、`RepositoryVerified`、`ProviderVerified`、`EnterpriseVerified`）
表示证据来源。T3 不等于 provider 或 enterprise assurance，必须由实际 provider 或外部证据提供。

Planner 可以提出 tier，但要求必须可追溯到 Organization Policy、Project Policy、Release Policy 或 Protected Gate。

## 路线与回执

路线保留 `DependencyConfidence`（`complete`、`partial`、`unknown`）以及受影响/受保护节点事实。
成本与复用仅是建议，不能降低要求。`VerificationPlanReceipt` 记录阶段、起始/最终 tier、独立 assurance、
理由、升级和执行事实；tier 降级直接 fail-closed。

Work Item route 还会绑定 `workItemId`、`repositoryId`、repository snapshot digest、`baseRevision`、Policy 引用、所需 tier/assurance、受影响路径和 dependency confidence。生命周期校验会重新解析声明的 Policy requirement，并拒绝缺失、过期或被篡改的绑定。`pr`、`merge`、`release` route 在执行边界必须有有效 base revision；`task` 仍不依赖 base revision。

如果 effective Policy 声明 `T3` 或 `ProviderVerified`，本地 Runtime 不能声称满足该要求：验证会在写入完成证据前停止。Hosted/provider 证据必须来自实际 provider。没有 typed verification requirement 的仓库继续使用无 Policy 兼容路径和历史 receipt 兼容行为。

物理执行可以共享，但每个 Work Item 必须拥有自己的绑定证据回执；不得把另一个 Work Item 的回执当作授权证据。

## CI 边界

`pre_ci` 不是 hosted CI 证据。CI shadow 阶段同时运行 Runtime 验证和现有 Cargo 检查；CI 结果不能覆盖红色治理决定。
删除重复 CI 检查必须进入后续明确的收敛阶段。

成本观察只是建议性遥测，未知或 partial 证据不能因此变成绿色治理。
