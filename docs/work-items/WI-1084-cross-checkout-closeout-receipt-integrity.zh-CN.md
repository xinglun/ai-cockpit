---
author: AI Cockpit maintainers
title: "WI-1084 — 跨 checkout 恢复收据完整性"
description: "在保留历史 close 字节的前提下识别必需恢复收据丢失的实施计划。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1084-cross-checkout-closeout-receipt-integrity
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1084-cross-checkout-closeout-receipt-integrity
contractDigest: sha256:507bedef85c7d3cb7ff2b8fd3a885d5441384ea23b8d6e4b010fa9be8c150107
---

[English](WI-1084-cross-checkout-closeout-receipt-integrity.md) · [日本語](WI-1084-cross-checkout-closeout-receipt-integrity.ja.md)

# WI-1084 — 跨 checkout 恢复收据完整性

Ray 已确认绑定的 Contract/preflight/snapshot，Native 已记录 confirm_review。仅删收据、恢复后 Retained close→Deleted transition 两份真实 RED 均保留。有界修正通过18项共享单元及14项 repository/CLI/MCP 集成回归，严格 workspace clippy 与格式检查通过。当前正式证据及剩余验收以 Native 为准；lastVerifiedBy 标识本进行中 WI 投影，不表示已验证关闭。

## 预期收益与已观察问题

独立 checkout 的 RED 已复现错误的 closed 投影，GREEN 与后续 P2 组合矩阵已保留。原 Sentinel 验收历史保持原样。正常下一条 Deleted transition 经完整既有规则校验后继续保留不可变历史 close 的有效性。

## 最小设计

[最终计划](../../.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/final-plan-20261015/implementation-plan.md)明确拟实现格式：同 WI 的不可变 origin（schema1）绑定 repository/WI、canonical Contract、source snapshot、provider/Runtime 事实、原 close 及完整原始 payload 清单，并声明必需 recovery receipt 的固定路径和 version2。version2 receipt 单向引用 origin 的实际摘要，不形成内容哈希环。任一必需记录缺失、错绑或版本不符时，不得回退普通 close 或降级 v1。

复用现有生命周期锁和不可变安装器，顺序安装历史 payload、origin、receipt、原始 close marker；发布 marker 前复查全部必需字节。中断后保留已发布路径，冲突重试拒绝覆盖。恢复 checkout 再作为 source 时先通过共享校验。origin 不含变化中的时间戳；receipt 的创建元数据如实记录，不新增 TTL。

普通当前/旧历史 close 保持原语义。旧 v1 证据不改写、不静默迁移；在独立来源绑定建立前已经丢失的旧唯一收据无法倒推出其历史，也不宣称能检测所有来源记录一起被移除。

## RED 与验收计划

1. 在独立 Git common-dir 中正常恢复并确认有效 closed 基线，仅删除 receipt；允许查询退出0，但断言 `archived`、`blocking=true` 和完整性诊断。先在生产修复前看到断言失败。
2. 证明重叠工作不会错误准入；覆盖 origin/receipt 缺失、畸形、非普通文件、错误版本/路径/摘要/身份、close/payload 错绑及归档完整性。
3. 用现有私有 hook 覆盖 origin 后、receipt 后、close 前后中断；保留 source 和已发布路径，证明支持的恢复/幂等及恢复 checkout 再作 source。
4. 保留普通/v1 兼容和历史 `not_ready`、`historical_evidence_not_revalidated`；补窄范围 CLI/MCP 一致性。
5. 真实 policyReview 和 fresh 准入之后执行聚焦 RED/GREEN 恢复回归、`cargo test --locked --workspace` 及仓库必需门禁。保留900秒上限、隔离构建目录、单构建任务和原始证据；正式证据以当前 Native 回执为准，不声称终态验收通过。

## 执行边界

WI-1084 使用单 worker 串行；正式并行兼容性为 false，因为 WI-1070～1074 缺少边界/声明，其记录不改。构建输出隔离于 `target/wi1084-closeout-receipt-integrity/`，checkpointCount 保持1。PR#1040 保持 Draft，集成串行。授权不含 Sentinel 业务修改、WI-1078 资源变动、Task9、TTL、机制重构、发布、新 WI 真人 close、合并或删除。

依当前 Native 准入继续正式验证及 Raydot 审阅，参见[已保留的 RED/GREEN 证据](../../.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity/p2-transition-20261015/red-summary.md)。Contract 不变无需重复人工审阅。PR#1040 保持 Draft；本次批准不包含合并、关闭或资源删除。
