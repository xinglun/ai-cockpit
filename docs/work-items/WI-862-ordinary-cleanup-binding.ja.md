---
author: AI Cockpit maintainers
title: "WI-862 — ordinary cleanup binding continuity"
description: "valid な historical verification evidence を close が受理しても ordinary cleanup binding を保持する。"
audience: [maintainer, reviewer]
status: in_progress
authority: explicit-user-authorization
workItemId: WI-862-ordinary-cleanup-binding
lastVerifiedBy: WI-862-ordinary-cleanup-binding
---

[English](WI-862-ordinary-cleanup-binding.md) · [简体中文](WI-862-ordinary-cleanup-binding.zh-CN.md)

# WI-862 — ordinary cleanup binding continuity

この Work Item は work-result の assurance と cleanup identity を分離して保持
します。valid な historical verification receipt を close が受理しても、Runtime
が取得する branch と linked-worktree の binding を抑制しません。
