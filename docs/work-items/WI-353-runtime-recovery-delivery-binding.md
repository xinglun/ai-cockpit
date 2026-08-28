---
author: Ray
title: "WI-353 — Runtime recovery delivery binding"
workItemId: WI-353-runtime-recovery-delivery-binding
description: "Bind the recovered WI-351 delivery to its actual reviewed PR while preserving immutable history."
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-355-runtime-archive-recovery-binding
predecessor: WI-351-runtime-recovery-binding
successor: WI-355-runtime-archive-recovery-binding
terminalArchive: .ai/work-items/archive/WI-353-runtime-recovery-delivery-binding.archive.json
terminalVerification: .ai/evidence/WI-353-runtime-recovery-delivery-binding.verification.json
capabilityClaims:
  - recovery_delivery_binding
---

# WI-353 — Runtime recovery delivery binding

[简体中文](WI-353-runtime-recovery-delivery-binding.zh-CN.md) · [日本語](WI-353-runtime-recovery-delivery-binding.ja.md)

## Intent and boundary

This successor Work Item preserved the immutable WI-351 archive and bound the
recovered Runtime delivery to the actual reviewed GitHub PR #318. Its archive
and evidence remain historical bytes. The distinct archived-retry defect is
continued through WI-355 under the explicit recovery receipt
`.ai/decisions/WI-353-runtime-recovery-delivery-binding.recovery.json`.

The scope is limited to recovery binding, its fail-closed regression coverage,
and the governance records required to deliver that change. Sentinel business
code, Provider discovery, trading decisions, gates, execution, position
sizing, global configuration, and any rewrite of WI-351 history are outside
the boundary.

## Verification and delivery boundary

- The locked workspace test suite, formatting check, and clippy run are
  recorded for the reviewed delivery before the predecessor was recovered.
- The PR resource context is bound to [PR #318](https://github.com/xinglun/ai-cockpit/pull/318)
  with base `main`/`origin` and the dedicated recovery worktree.
- WI-355 owns the fresh archive-retry correction, its verification, provider
  finalization, exact branch/worktree cleanup, and structured close. This
  document does not rewrite the predecessor archive or claim successor work
  as part of the predecessor's historical result.

The predecessor archive and evidence remain immutable; this successor carries
the delivery and finalization boundary without rewriting predecessor bytes.
