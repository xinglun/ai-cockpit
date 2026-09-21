---
author: AI Cockpit maintainers
title: "WI-958 — v0.2.103 Runtime リリース"
description: "governed verification、正確な cleanup、隔離された artifact acceptance の後にのみ v0.2.103 を公開する。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-958-v0-2-103-release
lastVerifiedBy: WI-958-v0-2-103-release
---

[English](WI-958-v0-2-103-release.md) · [简体中文](WI-958-v0-2-103-release.zh-CN.md)

# WI-958 — v0.2.103 Runtime リリース

この Work Item は検証済み main から次の Runtime リリースを準備する。公開は最後に
行う。immutable release identity、公開 artifact、隔離された install/upgrade acceptance、
正確な resource cleanup の後でのみ、Sentinel #902 の所有者に downstream replay の
確認を求める事実ベースの通知を送る。Sentinel は変更せず、第三者 host の chat display
確認も主張しない。
