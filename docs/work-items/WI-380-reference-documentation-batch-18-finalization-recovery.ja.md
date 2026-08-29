---
author: AI Cockpit maintainers
title: "WI-380 — WI-379 provider finalization 復旧"
description: "reviewed successor delivery を bind し、WI-379 の history を書き換えずに documentation batch を close します。"
workItemId: WI-380-reference-documentation-batch-18-finalization-recovery
canonical: docs/work-items/WI-380-reference-documentation-batch-18-finalization-recovery.md
audience: [maintainer, reviewer]
status: implemented
authority: translation
lastVerifiedBy: WI-380-reference-documentation-batch-18-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-380-reference-documentation-batch-18-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-380-reference-documentation-batch-18-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.close.json
capabilityClaims: [governance_integrity, resource_finalization]
---

# WI-380 — WI-379 provider finalization 復旧

[English](WI-380-reference-documentation-batch-18-finalization-recovery.md) · [简体中文](WI-380-reference-documentation-batch-18-finalization-recovery.zh-CN.md)

## Intent と boundary

WI-379 は reviewed PR #343 で reference documentation を delivery しましたが、provider
PR identity が確定する前に archive されました。この明示的 successor は WI-379 の immutable
archive、evidence、Outcome、recovery decision を保持し、復旧そのものの provider-bound lifecycle
を記録します。

## Scope

- WI-379 predecessor の digest と recovery lineage を可視化します。
- 三言語 parity 文書で WI-379 を recovered とし、この successor を登録します。
- verification 前にこの Work Item の reviewed PR context を bind します。
- close 前に branch/worktree の正確な cleanup を証明します。

## Out of scope

Runtime code、Release artifact、global Agent/MCP configuration、WI-379 の immutable な
archive/evidence/Outcome/PR bytes は対象外です。

## Acceptance

- recovery decision が predecessor の Contract、Summary、Outcome、Events、repository、Runtime
  identity を bind すること。
- WI-379 bytes を変更せず historical/recovered と明示すること。
- successor PR context を verification evidence より前に bind すること。
- Hosted checks、installed Runtime verification、finalization、close、visible human Outcome が通ること。

## Verification と terminal records

明示的な `--repo` を使う installed Runtime、documentation/governance checks、
`cargo test --locked --workspace` を実行します。reviewed merge 後に次を記録します。

- Archive：`.ai/work-items/archive/WI-380-reference-documentation-batch-18-finalization-recovery.contract.json`
- Verification：`.ai/evidence/WI-380-reference-documentation-batch-18-finalization-recovery.verification.json`
- Finalization：`.ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.finalize.json`
- Close：`.ai/decisions/WI-380-reference-documentation-batch-18-finalization-recovery.close.json`
