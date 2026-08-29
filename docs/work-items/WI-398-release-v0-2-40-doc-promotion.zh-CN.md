---
author: AI Cockpit maintainers
title: "WI-398——v0.2.40 文档晋升"
description: "根据不可变 Runtime 证据，将已关闭的 v0.2.40 发布准备文档晋升为终态。"
workItemId: WI-398-release-v0-2-40-doc-promotion
audience: [maintainer, reviewer]
status: recovered
authority: human-authorized
lastVerifiedBy: WI-398-release-v0-2-40-doc-promotion
capabilityClaims: [documentation_governance, release_distribution]
---

# WI-398——v0.2.40 文档晋升

[English](WI-398-release-v0-2-40-doc-promotion.md) · [日本語](WI-398-release-v0-2-40-doc-promotion.ja.md)

## 意图

在创建 v0.2.40 发布标签前，根据不可变的归档、验证、收尾和关闭记录，
将已关闭的 WI-397 文档晋升为可审计的终态。晋升不会改写这些记录。

## 边界

本 Work Item 仅更新三语 WI-397 文档和 parity ledger 的状态及终态链接，
并保留审查交付所需的 WI-397 close/finalization receipt。Runtime 行为、
发布实现和公开 adopter 验收不在本边界内。

## 验证

晋升脚本、文档验收、状态一致性、治理完整性和 diff 检查必须在合并前通过。
公开 binary 和 adopter 验收由后续 release-adopter Work Item 负责。
