---
author: AI Cockpit maintainers
title: "严格安装安全"
description: "共享 AI Cockpit Runtime 安装的供应链边界。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# 严格安装安全

使用[发布与分发](../release/distribution.zh-CN.md)记录的不可变公开 Release 路线。
archive 文件名、target、SHA-256 条目、release manifest、tag 与可选 provider attestation
必须指向同一制品。只有 tag 或 upload 不能作为安装证据；checksum 不匹配时必须停止。

边界如下：

- private mirror 必须由其 owner 独立保护 metadata、artifact 与 digest；Runtime 不为镜像运营者背书；
- local source build 是 contributor 证据，不能替代 adopter acceptance 使用的不可变公开 Release；
- SBOM 是组件清单，provenance 是另外的 source/build 声明；
- repository-local 证据与 Agent prompt 都不能证明企业身份、隔离、合规或 provider 控制。

不得静默改用移动分支或旧制品。任何例外都要由负责 Release 或安全的人员记录并解决。
继续阅读[安全与 Release 验证](security-release-verification.zh-CN.md)。

[安装](installation.zh-CN.md) | [English](installation-security.md) | [日本語](installation-security.ja.md)
