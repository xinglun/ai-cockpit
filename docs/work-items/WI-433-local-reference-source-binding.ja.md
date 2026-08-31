---
author: AI Cockpit maintainers
title: "WI-433 — local reference source binding"
workItemId: WI-433-local-reference-source-binding
description: "公開 repository への network fallback や source copy を行わず、maintainer の local checkout と固定 commit に比較を束縛する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-433-local-reference-source-binding
---

# WI-433 — local reference source binding

現在の semantic comparison は `AI_COCKPIT_REFERENCE_ROOT` の local checkout
だけを使用し、正確な commit を `tests/conformance/reference-source.lock`
に記録します。checkout が欠落、dirty、または commit drift の場合は
fail-closed となり、hosted CI は offline corpus のみを実行します。

Historical inventory は immutable のまま保持し、黙って rebaseline しません。
これは semantic parity と governance boundary の文書化であり、reference
Runtime、Python module、Make rule、toolchain の copy ではありません。

[English](WI-433-local-reference-source-binding.md) · [简体中文](WI-433-local-reference-source-binding.zh-CN.md)
