---
author: AI Cockpit maintainers
title: "WI-283 — current main の reference Contract semantics"
workItemId: WI-283-reference-contract-semantics-current-main
description: "最新の reviewed default branch から bounded Rust Contract-semantics parity batch を再検証し、WI-282 の immutable history を保持します。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-283-reference-contract-semantics-current-main
authority: canonical
---

# WI-283 — current main の reference Contract semantics

WI-283 は immutable WI-282 の明示的な successor です。default branch revision
`622836157e945a46f8cb34ee747084d3193e7f28` から同じ bounded Contract-semantics
実装を再検証し、predecessor の Contract、evidence、archive、recovery bytes を
保持します。predecessor は書き換えず、hosted quality rejection は recovery
history として記録します。
