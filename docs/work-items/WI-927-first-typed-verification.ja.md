---
author: AI Cockpit maintainers
workItemId: WI-927-first-typed-verification
title: 初回 typed required verification の実行境界
description: 終端 lifecycle gate を緩めずに、初回の宣言済み required verification が正式な receipt を作れるようにする。
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: authorized
lastVerifiedBy: WI-927-first-typed-verification
---

# WI-927 — 初回 typed verification の実行境界

この Work Item は [Issue #893](https://github.com/xinglun/ai-cockpit/issues/893)
の修正を継続します。前身の Contract は governance field の形式不備を
Runtime のルールに従って保存したうえで置き換えました。Summary に typed
required verification の entry がまだ無いため、entry を生成する初回の宣言済み
検証そのものを起動できない Runtime の precondition 循環を修正します。

Contract、repository snapshot、checkpoint、非 red の preflight が現在有効で、
保留中の governance fact が typed verification entry の不足だけである場合、
Runtime は宣言済み check を一度実行できます。正式な identity-bound receipt が
Summary に書かれてから `finish`、`archive`、`close` が続行可能になります。
failed、重複、stale、foreign、malformed な evidence は process spawn 前に
引き続き fail-closed です。

ユーザーに見える benefit は、focused と hosted の evidence が揃うまで未宣言です。
