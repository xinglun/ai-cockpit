---
author: Ray
title: "WI-355 — Runtime archive recovery binding"
workItemId: WI-355-runtime-archive-recovery-binding
description: "正当な stale retry receipt を archived Work Item の historical evidence として扱い、fail-closed validation を維持する。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: translation
canonical: docs/work-items/WI-355-runtime-archive-recovery-binding.md
lastVerifiedBy: WI-355-runtime-archive-recovery-binding
predecessor: WI-353-runtime-recovery-delivery-binding
capabilityClaims:
  - archived_retry_recovery_binding
---

# WI-355 — Runtime archive recovery binding

[English](WI-355-runtime-archive-recovery-binding.md) · [简体中文](WI-355-runtime-archive-recovery-binding.zh-CN.md)

## Intent と boundary

この successor Work Item は、正当な stale retry recovery receipt の archived read path
を修正します。retry が完了して新しい archived projection が存在する場合、古い retry
receipt は historical evidence として消費され、current recovery として Outcome や close
評価を停止させません。

malformed、foreign、誤った名前、ambiguous、また pending 中の retry evidence は引き続き
fail-closed です。WI-353 の archive bytes は immutable で、実装の編集範囲外です。

## Verification と delivery boundary

- archived stale-retry regression を追加し、既存の recovery negative test を維持します。
- formatting、locked workspace tests、clippy、governance integrity、documentation acceptance
  を実行します。
- reviewed PR の merge、provider finalization の検証、structured close が完了するまで
  この Work Item は in progress とします。
