---
author: AI Cockpit maintainers
title: "WI-198 — Governance gate default-branch discovery"
description: "identity 検査を弱めず、detached pull-request checkout の pre-merge governance 検証を決定的にする。"
audience:
  - maintainer
  - reviewer
workItemId: WI-198-governance-gate-default-branch-discovery
status: in_progress
authority: canonical
lastVerifiedBy: WI-198-governance-gate-default-branch-discovery
---

# WI-198 — Governance gate default-branch discovery

WI-198 は immutable な WI-197 の明示的 successor です。hosted quality により、detached
pull request merge checkout では `origin/HEAD` と event の base branch metadata が
ともにない場合があることが分かりました。gate は Contract の不変な
`resourceContext.baseBranch` だけを狭い fallback として使用し、repository、PR、branch、
worktree、evidence、runtime、digest の全 binding を引き続き要求します。

回帰テストは metadata がない有効な checkout と、外部に示された base branch が不一致の
ケースを検証します。WI-197 は immutable のまま recovery receipt でリンクされます。

[English](WI-198-governance-gate-default-branch-discovery.md) ·
[简体中文](WI-198-governance-gate-default-branch-discovery.zh-CN.md)
