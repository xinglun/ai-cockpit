---
author: AI Cockpit maintainers
title: "WI-558——WI-557 终态文档投影"
description: "依据终态证据晋升 WI-557 文档，并以有界自投影注册本 Work Item。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-558-doc-promotion-wi557
lastVerifiedBy: WI-558-doc-promotion-wi557
---

[English](WI-558-doc-promotion-wi557.md) · [日本語](WI-558-doc-promotion-wi557.ja.md)

# WI-558——WI-557 终态文档投影

## 目标

依据不可变 archive、verification、finalization 和 close receipt，将三语
WI-557 Work Item 页面及对应 reference-parity 行从条件投影晋升为确定的
终态。本 Work Item 同时登记自己的三语页面为有界自投影，避免关闭后无限
产生文档 successor。

## 范围

- WI-557 的英文、简体中文和日语 Work Item 页面。
- 本 Work Item 的三语页面。
- 英文、简体中文和日语 reference-parity 行。

## 边界

只有官方 promotion helper 可以写入终态投影。Runtime、协议、参考源、对象
工程和无关文档不变。自投影只适用于这个精确的有界文档范围，且不绕过证据
校验。

## 验收

- WI-557 页面和 parity 行在三种语言中带有终态证据绑定及 `已实现` 状态。
- repository-wide closed Work Item promotion check、文档验收和声明的验证命令通过。
- 本 Work Item 的页面在 close 前保持条件注册；关闭后门只将这个精确的自投影视为终态。
- 不修改不可变 receipt、Runtime 行为或无关投影。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo . --work-item WI-557-reference-file-comparison-batch-41 --check`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `git diff --check`
