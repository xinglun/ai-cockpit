---
author: Ray
title: "WI-353 — Runtime recovery delivery binding"
workItemId: WI-353-runtime-recovery-delivery-binding
description: "Bind the recovered WI-351 delivery to its actual reviewed PR while preserving immutable history."
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-353-runtime-recovery-delivery-binding
predecessor: WI-351-runtime-recovery-binding
terminalArchive: .ai/work-items/archive/WI-353-runtime-recovery-delivery-binding.archive.json
terminalVerification: .ai/evidence/WI-353-runtime-recovery-delivery-binding.verification.json
capabilityClaims:
  - recovery_delivery_binding
---

# WI-353 — Runtime recovery delivery binding

[简体中文](WI-353-runtime-recovery-delivery-binding.zh-CN.md) · [日本語](WI-353-runtime-recovery-delivery-binding.ja.md)

## Intent and boundary

This successor Work Item preserves the immutable WI-351 archive and binds the
recovered Runtime delivery to the actual reviewed GitHub PR #318. It records
the exact `main`/`origin` base, branch, worktree, and Runtime-owned evidence
before finalization.

The scope is limited to recovery binding, its fail-closed regression coverage,
and the governance records required to deliver that change. Sentinel business
code, Provider discovery, trading decisions, gates, execution, position
sizing, global configuration, and any rewrite of WI-351 history are outside
the boundary.

## Verification and delivery boundary

- The locked workspace test suite, formatting check, and clippy run are
  required before the successor can be archived.
- The PR resource context is bound to [PR #318](https://github.com/xinglun/ai-cockpit/pull/318)
  with base `main`/`origin` and the dedicated recovery worktree.
- Provider finalization, exact branch/worktree cleanup, and structured close
  remain pending until the reviewed PR is merged. Pre-merge state must not be
  reported as completed.

The predecessor archive and evidence remain immutable; this successor carries
the delivery and finalization boundary without rewriting predecessor bytes.
