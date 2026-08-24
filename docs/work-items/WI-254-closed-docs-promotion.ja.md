---
author: AI Cockpit maintainers
title: "WI-254 — deterministic closed documentation promotion"
workItemId: WI-254-closed-docs-promotion
description: "exact immutable close evidence から controlled Work Item documentation fields を promote し、check を required quality gate にします。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-254-closed-docs-promotion
authority: canonical
---

# WI-254 — deterministic closed documentation promotion

WI-254 は WI-253 の Runtime-recorded successor です。recovery receipt は
WI-253 の canonical Contract、Summary、Outcome、events、archive、verification、
sequence-2 finalization、close evidence を bind します。これらの lifecycle
records は immutable のままです。

## Acceptance boundary

- `tests/docs/promote_closed_work_item.py` は documentation change を計画する前に、
  repository/Work Item identity、archive Contract の raw digest、passing
  verification、linear finalization chain、sequence-2 deleted receipt、merge
  identity、structured approved close を厳密に検証します。
- write boundary は exact 3 Work Item documents の `status`、`lastVerifiedBy`、
  4 個の `terminal*` frontmatter fields と、各 reference-parity document の
  1 個の exact row だけです。Contract-language body prose とすべての `.ai`
  lifecycle truth は rewrite しません。
- `--check-all` は governed closed Work Item の mandatory documentation/quality
  gate です。invalid identity/filesystem input は document write 前に fail closed
  となり、stale noncanonical projection も check を通過しません。
- この change では同じ helper で WI-253 を promote し、WI-254 close 後は
  synchronized default branch の detached closure context から WI-254 を
  promote します。

## Lifecycle handoff

完全な delivery sequence は `close → promote closed docs → terminal CI` です。
helper は explicit repository workflow command であり、Runtime Core が Markdown
を自動編集するとは主張しません。したがって pre-close PR run の green は、
terminal projection と default-branch terminal run の代わりになりません。

## References

- [WI-253 predecessor](WI-253-docs-terminalization.ja.md)
- [Agent workflow](../reference/agent-workflow.ja.md)
- [Reference parity](../reference/reference-parity.ja.md)
