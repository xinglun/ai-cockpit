---
author: AI Cockpit maintainers
title: "パフォーマンス診断"
description: "一つの repository-bound Work Item のガバナンスコストを証拠だけで診断する。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# パフォーマンス診断

[English](performance-diagnosis.md) · [简体中文](performance-diagnosis.zh-CN.md)

パフォーマンス診断は測定されたガバナンスコストを説明するもので、ガバナンス判断を
変更しません。Runtime の request-scoped `diagnose` と verification cost observation は、
一つの repository と任意の Work Item について、snapshot 作業、read/hash files、検証回数、
実行/再利用ノード、worker/process 数、elapsed time、限定的な bottleneck hint を記録できます。

次を区別して記録します。

- execution と reuse は物理的な観測です。各 Work Item は固有の identity-bound evidence receipt を受け取ります。
- local process time は provider/human wait、token usage、release time、adopter speedup の証明ではありません。
- malformed、cross-Work-Item、mismatch、不完全な observation は unknown/partial のままで、必要な検証ルートを下げません。
- 比較は repository、Runtime、profile、policy、command、stage、input identity が一致する場合だけ有効です。

source の JSONL parser と report wire shape は Runtime protocol の要件ではありません。AI Cockpit は
P95、provider wait、enterprise performance claim を発明しません。[Governance cost metrics](governance-cost-metrics.ja.md) と
[Governance profiles](governance-profiles.ja.md)も参照してください。

明示的な `--repo` を使うすべての adopter で、性能事実はローカル telemetry です。global project state や
チェックを省略する権限ではありません。
