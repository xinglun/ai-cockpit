---
author: AI Cockpit maintainers
title: "WI-285 — reference Contract semantics final recovery"
workItemId: WI-285-reference-contract-semantics-final
description: "事前に文書復旧を完了したうえで、限定された Rust Contract semantics parity batch を完了する。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-285-reference-contract-semantics-final
authority: canonical
---

# WI-285 — reference Contract semantics final recovery

WI-285 は不変の WI-284 に対する明示的な successor です。predecessor の
Contract、evidence、archive、recovery bytes はすべて保持します。WI-284
archive 後に WI-281 の履歴文書 promotion と predecessor status が不足して
いることを hosted quality が検出したため、検証前に補完して同じ bounded
batch を完了します。

受入条件は Rust Contract scenario 実装とテスト、三言語 parity/document
binding、current default branch の workspace 全体検証、reviewed hosted PR、
不変の recovery linkage です。無関係な CI、release、planner、global adapter
変更は対象外です。
