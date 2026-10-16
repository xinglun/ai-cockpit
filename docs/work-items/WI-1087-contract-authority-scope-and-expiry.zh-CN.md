---
author: AI Cockpit maintainers
title: "WI-1087 — 有限期限目标授权的范围与到期"
description: "范围内连续执行、实质越界才决定的planned设计。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1087-contract-authority-scope-and-expiry
status: planned
authority: design-and-registration-only
---

[English](WI-1087-contract-authority-scope-and-expiry.md) · [日本語](WI-1087-contract-authority-scope-and-expiry.ja.md)

# WI-1087 — 有限期限目标授权的范围与到期

本项仅planned设计，Runtime通过new登记为planning/not_ready；没有start、checkpoint、产品实现、实际权限启用、合并或清理。产品实施authority保持missing。

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
