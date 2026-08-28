---
author: Ray
title: "WI-351 — Runtime retry recovery receipt binding"
workItemId: WI-351-runtime-recovery-binding
description: "Runtime 自身の状態 projection 後も retry recovery receipt を有効に保ち、不正 evidence の fail-closed を維持する。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: translation
canonical: docs/work-items/WI-351-runtime-recovery-binding.md
lastVerifiedBy: WI-351-runtime-recovery-binding
terminalArchive: .ai/work-items/archive/WI-351-runtime-recovery-binding.contract.json
terminalVerification: .ai/evidence/WI-351-runtime-recovery-binding.verification.json
capabilityClaims:
  - recovery_receipt_binding
---

# WI-351 — Runtime retry recovery receipt binding

[English](WI-351-runtime-recovery-binding.md) · [简体中文](WI-351-runtime-recovery-binding.zh-CN.md)

## Intent と boundary

この Work Item は共有 Rust Runtime の retry recovery lifecycle を修正します。retry
後に Runtime が現在の Summary、Outcome、Events projection を更新しても、同じ retry
receipt を foreign または stale と誤認しないようにします。一方、foreign、stale、
malformed、誤った名前の evidence は従来どおり fail-closed にします。

実装範囲は recovery binding logic と regression test に限定します。Sentinel business
code、Provider discovery、trading decision、gate、execution、position sizing、global
configuration、historical archive は対象外です。

## Verification

- Runtime 自身の retry 後 projection 更新を模擬する `retry → verify → preflight → finish`
  regression test を追加しました。
- 既存の recovery negative path は不正な evidence を引き続き拒否します。
- local の `cargo fmt --all -- --check`、locked workspace tests、clippy は通過し、hosted
  verification は [PR #318](https://github.com/xinglun/ai-cockpit/pull/318) で実行します。

この Work Item は recovered historical predecessor です。不変の archive と recovery
decision を lifecycle evidence の source of truth として保持し、predecessor bytes を
書き換えず delivery は WI-353 で継続します。
