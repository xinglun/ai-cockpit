---
author: AI Cockpit maintainers
title: "企业部署边界"
description: "共享 Runtime 与 repository-local Protocol 提供什么，以及企业必须从外部提供什么。"
audience:
  - adopter
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - enterprise_deployment_boundary
---

# 企业部署边界

每台机器或 toolchain 安装一份共享 `ai-cockpit` Runtime。每个 repository 显式 attach；repository identity、
Contract、evidence、knowledge 和 adapter 隔离在各自的 `.ai/` 中。

AI Cockpit 提供 typed contract、有界 verification、fail-closed reuse、repository-local record 和可导出的
audit event。它不提供全局 identity provider、OS sandbox、branch protection、生产变更控制、secret manager、
企业 SIEM、WORM 保留、签名服务、SBOM 生成器、provenance authority 或组织级审批目录。

采用者必须用 delegated evidence 将外部控制绑定到 Work Item，并定义数据分类、保留、销毁和导出规则。本地
green decision 不是外部控制已满足的证明，除非对应 external evidence 已存在、有效且完成绑定。
