---
author: AI Cockpit maintainers
workItemId: WI-1121-v1-0-2-release
title: v1.0.2 release
description: 四つの対象の v1.0.2 release と公開 upgrade acceptance を準備し検証します。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1121-v1-0-2-release
---

[English](WI-1121-v1-0-2-release.md) · [简体中文](WI-1121-v1-0-2-release.zh-CN.md)

# WI-1121 — v1.0.2 release

この Work Item は v1.0.2 の source と配布文書を準備し、正確な source を検証した後、immutable な annotated tag と download した artifact に公開と acceptance を結び付けます。各検査が通るまで公開や acceptance の完了は主張しません。

## 境界

- 予定する対象は `aarch64-apple-darwin`、`aarch64-unknown-linux-gnu`、`x86_64-unknown-linux-gnu`、`x86_64-pc-windows-msvc` です。
- 以前の stable v1.0.1 Release は N-1 upgrade source です。過去の failure と Unknown evidence は保持します。
- build の成功だけでは公開 install、upgrade acceptance、user-visible benefit を証明しません。
