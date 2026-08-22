---
author: AI Cockpit maintainers
title: "WI-149——结构化发布 adopter 决定"
description: "将发布后 adopter 验收绑定到完整且与仓库绑定的 Human Decision receipt。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-149-release-decision-acceptance
workItemId: WI-149-release-decision-acceptance
---

# WI-149——结构化发布 adopter 决定

发布后的 adopter 验收与 N-1 升级验收必须为每个受治理 Work Item 使用完整的结构化
Human Decision close。harness 向不可变 Release binary 提供 actor、authority source、
reason、evidence reference、policy reference、决定时间和 resume condition。

close 之后，harness 要求 `.ai/decisions/<work-item>.close.json` 是常规且非符号链接文件，
校验其 Work Item、closed 状态、confirmed 决定和结构化字段，然后将它复制到验收 artifact。
binding record 追加 adopter `repositoryId`、Work Item ID、决定摘要和校验结果。缺失或不匹配
的决定证据会 fail closed，绝不会修改已发布的 Release truth。

静态 wrapper 会持续检查结构化 close、复制和校验边界。三语发布分发指南描述同一验收合同。
Runtime Core 和 CLI 语义不属于本 Work Item。

证据：`.ai/evidence/WI-149-release-decision-acceptance.verification.json`。
关闭决定：`.ai/decisions/WI-149-release-decision-acceptance.close.json`。
