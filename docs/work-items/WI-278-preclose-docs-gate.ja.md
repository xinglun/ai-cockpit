---
author: AI Cockpit maintainers
title: "WI-278 — pre-close tri-language Work Item document gate"
workItemId: WI-278-preclose-docs-gate
description: "parity/documentation projection の欠落を fail-closed で検出し、通常の code path は軽量に保ちます。"
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

WI-277 後に判明した process gap を閉じます。hosted governance は close 前に成功できても、
post-close documentation promotion が Work Item 文書の欠落を検出していました。static gate は
Contract の parity ownership に基づいて動的に選択し、英語・日本語・中国語の regular な
non-symlink projection を検証します。既存の `.ai` history は書き換えません。

通常の code Work Item はこの gate だけを理由に文書作成を強制されません。この規則は
repository-bound CI gate を通じて adopter repository にも継承されます。
