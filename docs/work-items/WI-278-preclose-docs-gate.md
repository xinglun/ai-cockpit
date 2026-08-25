---
author: AI Cockpit maintainers
title: "WI-278 — pre-close tri-language Work Item document gate"
workItemId: WI-278-preclose-docs-gate
description: "Fail closed on missing parity/documentation projections while preserving a lightweight ordinary-code path."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-278-preclose-docs-gate
terminalArchive: .ai/work-items/archive/WI-278-preclose-docs-gate.contract.json
terminalVerification: .ai/evidence/WI-278-preclose-docs-gate.verification.json
terminalFinalization: .ai/decisions/WI-278-preclose-docs-gate.finalize.4e0abb3fdff7fe5eb4446e3253b6b457bc3906f38641699ed8f11ecdde4e3d07.json
terminalDecision: .ai/decisions/WI-278-preclose-docs-gate.close.json
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
