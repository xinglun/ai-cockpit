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

Runtime 会读取可选且严格校验的 `.ai/policy.json`（`schemaVersion: 1`，包含
`organization`/`project` 槽位）。Work Item 可以在 contract 中增加
`layer: "work_item"` 的 `governancePolicy`。有效规则由显式的 contract
`operation` 选择（缺省时确定性地使用 `modify_source` 或
`production_destructive`）；自然语言不会改变规则。`preflight` 会暴露缺失的
权限或 policy evidence；verification 不会执行已经缺少权限的操作；`finish`、
`archive` 和 `close` 只有在有效决定为 green 时才会继续。受策略保护的 close
必须使用结构化 decision，并在 `policyRefs` 绑定 policy ID。多方审批和外部
provider 审批在导入外部审批回执前保持 fail-closed。

## 委托证据与审计边界

外部 provider 仍负责生成自己的证明。Delegated evidence model 绑定 provider、subject、origin、
assurance、收集时间、digest、validity 和 raw evidence reference。AI Cockpit 可以要求、验证、展示和
归档该引用，但不会伪造 provider signature、branch protection、SBOM、provenance 或企业审批。

使用 `ai-cockpit evidence import --repo <repo> --work-item <id> --metadata
<metadata.json> --raw <provider-output>`，将 provider metadata 绑定到原始 bytes
的 digest。raw reference 必须位于 `.ai/evidence/external/`；相同 bytes 的重复导入
是幂等的，冲突 receipt、路径逃逸、symlink、未知字段以及 repository/Work Item 不一致
都会 fail closed。`ai-cockpit evidence list` 和 repository-bound MCP 的
`delegated_evidence_list` 只展示重新验证过的 receipt。过期、撤销或 unknown receipt
仍可审计，但不能满足 `delegated:<provider>` evidence 要求。

Audit event 携带稳定 event ID、repository/Work Item identity、Runtime identity、时间、digest 和
evidence references。不能宣称本地 Git 或 `.ai/` 是独立不可变的企业审计日志。需要更高保证的组织应将
事件导出到 SIEM、WORM、S3 Object Lock、企业审计系统或外部 ledger。

## 敏感证据与保留

Evidence 分类为 `public`、`internal`、`confidential`、`restricted` 或 `secret_prohibited`。
持久化方式为 `full_capture`、`redacted_capture`、`digest_only` 或 `no_persistence`；
`secret_prohibited` 不得使用完整或脱敏捕获。Retention metadata 记录过期时间和确定性的 disposal
action。保留策略可以要求 purge plan；AI Cockpit 不会静默删除历史 evidence，也不声称本地 archive
满足企业法定保留要求。

实际入口是 `evidence policy` 和 `evidence purge-plan`。前者把严格策略绑定到 Work Item；后者返回
带 digest 的稳定 `retain`/`purge_planned` 清单，交给外部责任方审查。任何命令都不会静默删除 evidence。
`digest_only` 只保留 receipt digest 和治理摘要，不保留命令输出；如果策略为 `no_persistence` 而操作会
依赖不可保留的 receipt 来宣称完成，Runtime 会 fail closed。

这些控制支持企业合规工作，但不等同于 ISO 27001、SOC 2 或其他组织级认证。
