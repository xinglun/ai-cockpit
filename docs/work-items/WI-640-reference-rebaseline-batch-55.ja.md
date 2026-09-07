---
author: AI Cockpit maintainers
title: WI-640 — reference rebaseline batch 55
description: pinned reference の 60 path を一件ずつ再確認し、source implementation をコピーしない。
audience: [maintainer, reviewer, adopter]
workItemId: WI-640-reference-rebaseline-batch-55
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-640-reference-rebaseline-batch-55
---

# WI-640 — reference rebaseline batch 55

この Work Item は pinned commit `a9224aed77b5c317b53c4551a9eec306d91ee330` の 60 path を一件ずつ再確認し、
Rust counterpart または reference-only boundary を記録します。Python/Shell/Make/provider/fixture/JSON wire はコピー
しません。54 path は implemented-different-by-design、6 path は reference-only で、migrate-gap/deferred は残りません。
以前の ledger に source-change marker が無い path は `sourceChangedSincePrevious=false` と明示し、再確認を省略
したとは扱いません。完全な path 順、counterpart、reason は[英語 Work Item](WI-640-reference-rebaseline-batch-55.md)と
[machine ledger](../../tests/conformance/reference_file_inventory.json)にあります。attached adopter は shared Runtime、
明示的 `--repo`、isolated Contract/evidence/knowledge、dynamic verification、fail-closed lifecycle、visible human
Outcome を継承します。

See also: [English](WI-640-reference-rebaseline-batch-55.md) · [中文](WI-640-reference-rebaseline-batch-55.zh-CN.md)。
