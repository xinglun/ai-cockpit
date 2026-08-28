---
author: AI Cockpit maintainers
title: "Provider reconciliation の境界"
description: "過去の provider inventory は文脈であり、現在の provider truth ではない。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Provider reconciliation の境界

[English](provider-reconciliation-boundary.md) · [简体中文](provider-reconciliation-boundary.zh-CN.md)

`open-pr-issue-reconciliation-662.*` と `pre-release-documentation-alignment.json` は、
source repository の過去の revision における provider/reviewer assessment です。過去に
観測された状態を記録しますが、現在の repository、GitHub PR、release、enterprise approval
を証明しません。

AI Cockpit は provider の責任を明確に分けます。

- Runtime は delegated evidence を require、bind、display、archive できます。
- GitHub/Hosted CI、reviewer、branch protection、release publication、enterprise retention は外部です。
- stale または欠落した reconciliation は unknown であり、merge/release/close を許可しません。
- 新しい provider observation は現在の repository と Work Item identity で再取得し、digest、timestamp、source を持ちます。

したがって file-by-file ledger では source JSON/Markdown を `reference-only` とします。
`.ai/` にコピーせず current status に混ぜず、repository-local Contract や Runtime evidence を上書きしません。
target の境界は [Release distribution](../release/distribution.ja.md) と [Reference parity](reference-parity.ja.md) を参照してください。
