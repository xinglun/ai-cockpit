---
author: AI Cockpit maintainers
title: "WI-278 — pre-close tri-language Work Item document gate"
workItemId: WI-278-preclose-docs-gate
description: "Fail closed on missing parity/documentation projections while preserving a lightweight ordinary-code path."
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-278-preclose-docs-gate
authority: canonical
---

# WI-278 — pre-close tri-language Work Item document gate

This Work Item closes the process gap exposed after WI-277: hosted governance
could pass before close, while the post-close documentation promotion still
found missing Work Item documents. The static gate now selects the policy from
declared parity ownership, checks regular non-symlink English/Japanese/Chinese
projections, and repairs current-cycle omissions without rewriting `.ai`
history.

Ordinary code Work Items remain lightweight and are not required to create
documentation merely because this gate exists. The same rule is inherited by
adopter repositories through their repository-bound CI gate.
