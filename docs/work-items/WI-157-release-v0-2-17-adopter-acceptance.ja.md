---
author: AI Cockpit maintainers
title: "WI-157 — v0.2.17 Release と adopter acceptance"
description: "Immutable Runtime を公開し、新しい adopter repository を治理できることを確認する。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
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

Release: https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.17

Workflow: https://github.com/xinglun/ai-cockpit/actions/runs/32606940727

ローカルの public artifact evidence: `.ai/evidence/external/v0.2.17/adopter/` と
`.ai/evidence/external/v0.2.17/upgrade/`。インストール済みの public binary は
Runtime `0.2.17`、digest は
`sha256:4157cc04a23a24e6ac618e7079c123210920fba2e7fc5335c9f6a734c74721e3` です。
公開前の v0.2.16 evidence bytes は
`.ai/evidence/external/v0.2.16/WI-157-release-v0-2-17-adopter-acceptance/` に保持し、
現在の verification evidence として再利用していません。
