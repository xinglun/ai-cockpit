---
author: AI Cockpit maintainers
workItemId: WI-128-release-acceptance-cleanup
title: 发布 adopter 验收清理与隔离事实
description: 让发布后验收清理 fail-closed，并保持隔离 receipt 可审计。
audience:
  - maintainer
  - release-engineer
status: implemented
authority: canonical
lastVerifiedBy: release-acceptance
---

# WI-128——发布验收清理

N-1 发布后 harness 现在在 finish trap 中使用一个明确的退出状态变量。
升级和清理都成功时必定返回零；未设置的状态不能把有效验收变成 shell 错误。

两个 adopter harness 继续保留经过验证的临时根目录清理、不可变的
`releasePublished` truth、清理 receipt 和可审计的 typed 隔离 manifest。隔离策略
仍明确规定：HOME/XDG 配置根禁止 Runtime 写入，TMPDIR/CARGO_HOME 是隔离且允许
写入的根，所有写入都会被记录。
