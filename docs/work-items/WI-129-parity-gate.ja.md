---
author: AI Cockpit maintainers
workItemId: WI-129-parity-gate
title: Reference parity completeness を強制する
description: 固定リストだけに依存せず、最新の implemented Work Item を documentation gate が導出する。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-129 — Reference parity completeness

3 言語の parity baseline に WI-128 を追加した。documentation acceptance gate は
`status: implemented` と記された canonical English Work Item 文書から最大の数値 ID
も導出し、その ID が各 parity language にあることを要求する。これにより新しい
実装の文書漏れは、固定リストの更新忘れに依存せず fail closed になる。

gate は read-only のままで、governance facts を推論したり Runtime、Contract、
Summary、evidence state を変更したりしない。
