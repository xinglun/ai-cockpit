---
author: AI Cockpit maintainers
title: "WI-281 — recovery integrity gate"
workItemId: WI-281-recovery-integrity-gate
description: "CI が append-only recovery head を解決し、current-cycle Work Item projection の完全性を要求します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-281-recovery-integrity-gate
authority: canonical
---

# WI-281 — recovery integrity gate

この Work Item は、immutable predecessor に canonical retry と digest-suffixed
successor/supersession receipt が併存する場合の hosted governance gap を閉じます。
gate は有効な recovery head を deterministic に選択し、invalid candidate は
fail-closed のままにし、current release cycle が宣言する三言語の Work Item と
parity projection を要求します。
