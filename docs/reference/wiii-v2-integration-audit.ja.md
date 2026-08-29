---
author: AI Cockpit maintainers
title: Work Item Intelligence の統合境界
description: source wire compatibility を主張しない Rust-native な Work Item Intelligence projection。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/wiii-v2-integration-audit.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Work Item Intelligence の統合境界

[English](wiii-v2-integration-audit.md) · [简体中文](wiii-v2-integration-audit.zh-CN.md) · [日本語](wiii-v2-integration-audit.ja.md)

Rust Runtime は request-scoped、read-only の Work Item Intelligence projection を公開します。
schema version は明示し、source-bound inconsistency は `inconsistent` として返して黙って
rebuild しません。Work Item の scheduling、provider 呼び出し、人の approval の生成は行いません。

`status` と intelligence command は repository-local record/evidence を読みます。V2 projection
の rebuild は明示的 command に限り、source identity を検証します。壊れた record は
unknown/inconsistent のままです。query、pagination、cursor は `--repo` の repository context
に束縛されます。

これは reference Python CLI より狭い projection であり、JSON/API の直接互換ではありません。
過去の assessment score、generated audit bytes、provider result は reference-only です。
shared Runtime は複数 repository を扱えますが、Work Item、evidence、knowledge、snapshot は
常に分離されます。

provider identity、distributed scheduling、network isolation、人の approval、enterprise
compliance はこの audit の範囲外で、別の policy/provider evidence が必要です。
