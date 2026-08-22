# WI-147 — Verification 路线收敛

## 目标

将 Contract 与 Policy 要求连接到类型化 VerificationStage、计划、依赖/受影响事实、执行回执和 CI 边界，且复用或成本优化不能削弱治理要求。

## 设计基线

先统一验证语义。`VerificationTier` 与 `EvidenceAssurance` 正交；Planner 只能提出可追溯到策略的要求。
依赖置信度可以是 partial；物理执行共享不等于共享授权回执。此阶段继续保留 CI shadow 对照。

## 验收边界

未知阶段和 tier 降级 fail-closed。回执记录路线事实以及 Runtime/Repository identity。成本观察仅为建议，空计划并行度为零，
身份格式错误为 unknown。本阶段不删除既有 Cargo 检查，也不伪造 provider/enterprise assurance。

参见 [Verification 路线](../reference/verification-route.zh-CN.md) 以及英文、日文版本。
