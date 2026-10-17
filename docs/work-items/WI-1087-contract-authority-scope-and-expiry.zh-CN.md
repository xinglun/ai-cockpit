---
author: AI Cockpit maintainers
title: "WI-1087 — 有限期限目标授权的范围与到期"
description: "P1 数据模型与 Clock 基础实现；有限授权强制执行仍为规划。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1087-contract-authority-scope-and-expiry
status: in_progress
authority: p1-software-development-only
lastVerifiedBy: WI-1087-contract-authority-scope-and-expiry
---

[English](WI-1087-contract-authority-scope-and-expiry.md) · [日本語](WI-1087-contract-authority-scope-and-expiry.ja.md)

# WI-1087 — 有限期限目标授权的范围与到期

当前为同一 WI 的 P1 软件实施阶段，已记录原生 preflight 和 before-edit checkpoint。获准范围仅包含独立模型、可测试 Clock 与纯归一化接口；未配置真实 grant 或系统权限。全功能产品验收保持 not_run。

## Goal

可核查的有限期限目标授权，明确覆盖同目标的连续/并行WI、正常修复、重试和恢复，不逐WI重复真人审批；旧目标授权不能用于新目标。notBefore/expiresAt必填且有限，完成/撤销/到期终止后续执行，历史真实成果保留。

## Versioned design

下列版本化设计为规范来源，三语摘要只引用，不复制另一套授权SSOT。设计涵盖来源认证/目标覆盖、只收窄派生、锁后与副作用前重查、有界安全停止、到期纯metadata延期申请、一次合法延期统一作用适用活跃WI、禁止自续/复活、时钟/身份边界及独立迁移。

- [Design v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/design-v1.zh-CN.md)
- [Acceptance v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/acceptance-v1.json)
- [Staged scope v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/implementation-stages-v1.json)

## Dependency and ownership

依赖WI-1086。amendment/status/admission/receipt/lifecycle及账本共享路径串行，不启动第二个构建或并行共享改动。未来Clock/类型模块只有技术审阅、真实ownership隔离和正常准入后才考虑并行。

## v2 边界澄清

v1 原件保留。v2 明确撤销/提交的排序与外部 fencing 能力边界、来源版本替换与主体终止的区别、继承期限与固定更早截止，以及在途撤销的真实停止证据。仍只做规划，不启用产品能力。

- [Design v2 addendum](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v2-20261016/addendum-v2.zh-CN.md)
- [Acceptance v2 — planned / not_run](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v2-20261016/acceptance-v2.json)

## v3

范围收窄导致失去覆盖、真实 actor 失效均触发与撤销相同的在途停止；R04 参数化，全部未执行。

- [Design v3 clarification](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v3-20261016/addendum-v3.zh-CN.md)
- [R04 parameterized acceptance — not_run](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v3-20261016/acceptance-delta-v3.json)

## P1 进展

局部 TDD 与模块回归覆盖 Clock 边界、严格模型结构及纯 UTC 归一化。首轮模块测试通过：core 12、protocol 12、repository 164（含新增归一化测试 12 项）。完整 workspace 验证与 CI 另按实际执行结果记录；全功能验收矩阵继续保持 not_run。

实际执行者为 `agent:Codex`，技术计划审阅者为 `agent:Raydot`。`lastVerifiedBy` 引用本 WI，不代表真人身份；[真人决定来源](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/p1-activation-20261017/human-decision-source-20261017.json) 保留用户原文。[模块测试证据](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/p1/module-results.json)。P2–P4 与真实启用仍在范围外。
