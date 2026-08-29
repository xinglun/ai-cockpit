---
author: AI Cockpit maintainers
title: Verification evidence reuse Runtime
description: 保護された検証を弱めずに証拠再利用を計画・実行する Rust Runtime の境界。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/verification-evidence-reuse-runtime.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Verification evidence reuse Runtime

[English](verification-evidence-reuse-runtime.md) · [简体中文](verification-evidence-reuse-runtime.zh-CN.md) · [日本語](verification-evidence-reuse-runtime.ja.md)

AI Cockpit は planning と execution を分離します。request-scoped な plan は node を
`execute` または `reuse` と表示できますが、command を実行できるのは宣言された route
だけです。再利用された結果は evidence であり、required gate を飛ばす権限ではありません。

## 再利用の条件

repository、Work Item、base/head snapshot、正規化した変更集合、command、scope、stage、
runner/toolchain、policy、output identity がすべて一致する passed・未期限切れ receipt
だけを再利用できます。content、diff、environment は既存 verification node の binding
dimension であり、別 checker API は作りません。欠落、malformed、stale、foreign、矛盾した
receipt は `unknown` となり、required node は再実行します。

scope、security/trust、governance、coverage、identity、source-bound、supply-chain の
protected gate は policy または stage が要求する限り必ず実行します。
`stage_not_applicable` は実行 evidence ではありません。

## 観測可能な事実

結果には planned、executed、reused、stale-rerun、unknown-rerun、protected-node、時間、
worker、receipt identity を記録します。呼び出し数の削減は実際の adapter call-count 観測
で示し、時間や cache label から推論しません。物理実行を共有しても各 Work Item は固有の
identity-bound receipt を受け取ります。

これは Rust-native な semantic boundary です。reference source の Python module、Make
target、JSON wire shape は Runtime や adopter にコピーしません。
