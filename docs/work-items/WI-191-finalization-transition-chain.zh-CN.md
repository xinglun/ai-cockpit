---
author: AI Cockpit maintainers
title: WI-191 — Append-only finalization transition 链
workItemId: WI-191-finalization-transition-chain
description: "以追加方式记录合并与清理 transition，保留不可变的 pre-merge evidence。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-191-finalization-transition-chain
---

# WI-191 — Append-only finalization transition 链

WI-190 暴露出合法的 pre-merge blocked canonical receipt 无法在合并与清理后继续推进。WI-191 保留该 receipt，并增加 typed、digest-addressed、具有精确状态连续性的 transition 与唯一 head resolver。合并观察和资源清理是两个独立 transition；`finalize-verify` 与 `close` 绑定最新 head。foreign、stale、forked、malformed、symlinked 或 sequence-invalid 链都会 fail closed；legacy canonical receipt 保持兼容。
