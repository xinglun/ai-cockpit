---
author: AI Cockpit maintainers
title: "企业治理边界"
description: "面向企业采用者的权限、策略、证据、数据、保留和审计边界。"
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - enterprise_governance_contracts
---

# 企业治理边界

AI Cockpit 不规定企业必须有多少审批人，而是要求权限明确、证据充分、范围受限、未知可见、
决定可审计。

## 权限与人类决定

权限证据区分 `self_declared`、`repository_verified`、`provider_verified` 和
`enterprise_verified`。每条记录都说明 actor、权限来源、允许的 operation、policy references
和 evidence references。人类决定记录 decision、actor、reason、evidence/policy references、时间
以及可选的恢复条件。

审批方式由策略定义，不硬编码为双人审批。组织可以选择低风险不需要审批、单个授权人、多方审批或
外部 provider 审批。单一负责人也可以形成有效治理，只要范围、新鲜证据、可见未知、必需检查和决定
回执都明确记录。

## 策略优先级

策略层级为 organization → project → Work Item。下层可以增加要求或沿用上层规则，但不能降低上层
的审批强度，也不能删除上层要求的 evidence。发现下层试图弱化任一绑定时，overlay validation
必须 fail closed。

## 委托证据与审计边界

外部 provider 仍负责生成自己的证明。Delegated evidence model 绑定 provider、subject、origin、
assurance、收集时间、digest、validity 和 raw evidence reference。AI Cockpit 可以要求、验证、展示和
归档该引用，但不会伪造 provider signature、branch protection、SBOM、provenance 或企业审批。

Audit event 携带稳定 event ID、repository/Work Item identity、Runtime identity、时间、digest 和
evidence references。不能宣称本地 Git 或 `.ai/` 是独立不可变的企业审计日志。需要更高保证的组织应将
事件导出到 SIEM、WORM、S3 Object Lock、企业审计系统或外部 ledger。

## 敏感证据与保留

Evidence 分类为 `public`、`internal`、`confidential`、`restricted` 或 `secret_prohibited`。
持久化方式为 `full_capture`、`redacted_capture`、`digest_only` 或 `no_persistence`；
`secret_prohibited` 不得使用完整或脱敏捕获。Retention metadata 记录过期时间和确定性的 disposal
action。保留策略可以要求 purge plan；AI Cockpit 不会静默删除历史 evidence，也不声称本地 archive
满足企业法定保留要求。

这些控制支持企业合规工作，但不等同于 ISO 27001、SOC 2 或其他组织级认证。
