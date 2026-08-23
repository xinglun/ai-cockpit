---
author: AI Cockpit maintainers
title: "WI-157 — v0.2.17 Release と adopter acceptance"
description: "Immutable Runtime を公開し、新しい adopter repository を治理できることを確認する。"
audience:
  - adopter
  - contributor
  - maintainer
status: in_progress
authority: canonical
lastVerifiedBy: WI-157-release-v0-2-17-adopter-acceptance
workItemId: WI-157-release-v0-2-17-adopter-acceptance
---

# WI-157 — v0.2.17 Release と adopter acceptance

この Work Item は source、package、documentation の identity が一致した後に
Runtime を公開します。公開後の acceptance は immutable な public archive だけを
使用し、workspace build や fallback binary は使用しません。Runtime digest、adopter
repository identity、isolation manifest、evidence reuse、scaffold の `not_ready`、
完全な Work Item lifecycle receipt を記録します。

Acceptance receipt は post-release evidence です。失敗しても公開済み Release の
truth を書き換えません。
