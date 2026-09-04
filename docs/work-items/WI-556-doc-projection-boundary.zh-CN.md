---
author: AI Cockpit maintainers
title: "WI-556——有界文档投影"
description: "为已关闭发布工作记录有限且精确的文档投影边界。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-556-doc-projection-boundary
lastVerifiedBy: WI-556-doc-projection-boundary
---

[English](WI-556-doc-projection-boundary.md) · [日本語](WI-556-doc-projection-boundary.ja.md)

# WI-556——有界文档投影

## 目标

为仅文档的终态投影记录有限、精确的范围，避免关闭工作项晋级检查产生无限后继链。

## 边界

仅 Contract 指定的三个 WI 页面和三个 reference-parity 文件在范围内。Runtime、源代码、CI、证据和对象工程均不在范围内。

## 验收

- 六个精确文档路径在归档前登记，并与终态证据保持一致。
- 已关闭 Work Item 晋级检查识别为有界自投影。
