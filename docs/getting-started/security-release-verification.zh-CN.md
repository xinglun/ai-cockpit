---
author: AI Cockpit maintainers
title: "安全与 Release 验证"
description: "AI Cockpit Release 证据能证明什么，以及外部责任从哪里开始。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# 安全与 Release 验证

可执行命令与当前不可变基线见[发布与分发](../release/distribution.zh-CN.md)。
不同证据不能互相替代：

| 证据 | 支持的事实 | 不能证明 |
| --- | --- | --- |
| 稳定 provider Release | 指定 asset 已公开可用 | digest 或 source 正确 |
| Git tag | 存在不可变 source reference | 存在稳定 provider Release |
| `SHA256SUMS` 与 manifest | 所选制品匹配发布 bytes 与 metadata | 谁批准了 Release |
| Provider attestation | provider statement 绑定 artifact subject | 企业合规或安全执行 |
| SBOM | 组件已列出 | 没有漏洞或具备 build provenance |
| Adopter acceptance receipt | 固定的公开 binary 通过有边界的 harness | 所有 target、stack 或组织 policy 均通过 |

证据缺失、过期、foreign 或矛盾时都不能算通过。Runtime 会记录 repository 与 executable
identity；publication、身份、branch protection、private mirror、事件策略与企业保证仍由外部
provider 和人员负责。

普通采用先验证公开制品，再 attach 目标 repository。维护者运行发布后检查时只能使用已发布
binary；失败验收必须保留为失败历史，不能用 workspace build 替换。

[严格安装安全](installation-security.zh-CN.md) | [English](security-release-verification.md) | [日本語](security-release-verification.ja.md)
