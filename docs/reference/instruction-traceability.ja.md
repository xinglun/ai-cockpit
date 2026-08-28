---
author: AI Cockpit maintainers
title: Instruction traceability
description: 比較 instruction から Work Item と verification までを evidence に bind する関係。
audience:
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/instruction-traceability.md
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - comparison_traceability
---

# Instruction traceability

[English](instruction-traceability.md) · [简体中文](instruction-traceability.zh-CN.md) · [日本語](instruction-traceability.ja.md)

File-by-file comparison は machine-readable inventory [`tests/conformance/reference_file_inventory.json`](../../tests/conformance/reference_file_inventory.json) が管理します。各 pinned source path には一つの classification、bounded counterpart decision、reason があります。Comparison/parity page は人向けの説明であり、inventory は omission を検出する gate です。

## Forward / reverse check

各 batch の forward chain は次の通りです。

```text
pinned source path
  → Work Item Contract
  → target counterpart または明示的 boundary
  → acceptance と verification evidence
  → reviewed PR、merge、close receipt
```

Reverse check はすべての Work Item に source set、evidence、delivered counterpart があるか、または no-change/reference-only の理由が記録されているかを確認します。Archive は delivery history の truth であり、未追跡の note で置き換えません。Hosted performance（使用する場合）は `pass`、`not_run`、`fail` と reason を明示します。

Inventory script は structural gate です。coverage と identity を検証しますが、自然言語の claim の真偽を証明しません。新しい semantic responsibility には独立した bounded Contract と evidence が必要で、無関係な Work Item に隠しません。

## Non-copy と adopter boundary

Rust project は reference remediation JSON、Make command、Python checker を Runtime authority として取り込みません。Adopter は同じ inventory と明示的 repository lifecycle を利用できますが、source path、Work Item、evidence、provider receipt は独立です。
