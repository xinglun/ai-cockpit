---
author: AI Cockpit maintainers
title: "Multilingual semantic parity"
description: "Language projections preserve governance facts without translating authoritative Contract text."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Multilingual semantic parity

[简体中文](multilingual-semantic-parity.zh-CN.md) · [日本語](multilingual-semantic-parity.ja.md)

English, Simplified Chinese, and Japanese are presentation projections of the
same repository-bound Runtime facts. The fixed headings, status labels,
stop/next-action guidance, risk signals, limitations, and human-decision
fields must carry the same meaning in every supported language.

The CLI tests cover the stable projection markers and summaries in all three
languages. A localized projection must not:

- turn yellow or red evidence into green;
- invent an approval, benefit, capability, or provider/enterprise claim;
- omit a blocker, unknown, required check, safety warning, or recovery action;
- translate or rewrite acceptance criteria, intent, scope, or other
  human-owned Contract values.

Contract values remain in their authoring language and are labelled as such.
Only Runtime-owned presentation text is localized. An Agent adapter may offer
an additional non-authoritative translation, but the original value and its
digest/reference remain the governance source.

This is semantic parity, not source wire or Python-comparator compatibility.
All projections retain explicit `--repo` context and repository-local
evidence; language selection cannot change a governance decision.
