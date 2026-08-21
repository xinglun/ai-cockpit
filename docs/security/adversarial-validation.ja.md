---
author: AI Cockpit maintainers
title: "敵対的検証"
description: "Fail-closed security boundary と adversarial validation surface。"
audience:
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - security_validation
---

# 敵対的検証

セキュリティ境界は fail-closed と evidence-driven です。conformance corpus は文字列ではなく、
decision state、blockers、unknowns、safe actions、required checks、authority、outcome state の
意味を比較します。

runtime 境界テストでは、repository text を data として扱うこと、Work Item ID の path traversal
防止、MCP evidence path の repository 内制限、allowlist と対象 cwd の検証、fresh な passed receipt
なしに finish が完了を自己宣言できないことも確認します。

## Verification と reuse の trust boundary

Reusable receipt が node を満たす前に、runtime は repository snapshot と source range、attached
profile/configuration の raw bytes、toolchain と resolved executable identity、完全な execution
environment、command、scope、policy、stage、runner、output identity を candidate に bind します。
Protected node、explicit command、Work Item-bound verification は常に fresh です。

Receipt store は symlink の parent/leaf、malformed 内容、hard-linked commit marker、uncertain index
commit、unknown schema field、oversized file、tampered receipt ID、failed/expired receipt、binding
不一致を拒否します。失敗は unknown または rerun となり、reuse を許可しません。Verification は
command time、capture output、worker count にも上限を持ち、timeout、descendant、capture failure は pass ではありません。

失敗または未知の provider result は常に non-green です。human authority は decision requirement
を解決できますが、verification receipt を捏造できません。
