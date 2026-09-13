---
author: AI Cockpit maintainers
title: "WI-821——无资源文档 promotion"
description: "保持 post-close 文档 promotion 严格，同时不为本地 Work Item 虚构 provider finalization。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorized
workItemId: WI-821-no-resource-document-promotion
lastVerifiedBy: WI-821-no-resource-document-promotion
---

[English](WI-821-no-resource-document-promotion.md) · [日本語](WI-821-no-resource-document-promotion.ja.md)

# WI-821——无资源文档 promotion

## 意图与边界

本 Work Item 让文档 promotion helper 区分显式无资源 Contract 与绑定 provider 的 Contract。
本地 Work Item 可以在没有虚构 finalization receipt 的情况下 close；绑定 provider 的 Work Item
仍必须具备完整、绑定身份的 finalization chain。

WI-819 close 记录及其投影修复仅用于恢复发布后的治理边界，不改写 WI-819 的 archived evidence。

## 验证

定向测试覆盖缺失与 null `resourceContext`、资源绑定缺失 finalization、确定性的无资源终态引用，
以及不写入的 check 路径。
