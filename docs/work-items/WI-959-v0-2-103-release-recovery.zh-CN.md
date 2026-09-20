---
author: AI Cockpit maintainers
title: "WI-959 — v0.2.103 发布验证恢复"
description: "替代将发布后证据造成验证循环阻塞的发布 Contract。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-959-v0-2-103-release-recovery
lastVerifiedBy: WI-959-v0-2-103-release-recovery
---

[English](WI-959-v0-2-103-release-recovery.md) · [日本語](WI-959-v0-2-103-release-recovery.ja.md)

# WI-959 — v0.2.103 发布验证恢复

WI-958 正确保留了 v0.2.103 候选，但其不可变 Contract 把 GitHub 与发布后事实作为本地验证前置，形成循环阻塞。WI-959 是其显式后继：先完成当前验证，再按顺序完成评审、不可变发布、公开产物验收和精确清理。它不宣称已关闭的 #936 候选成功。
