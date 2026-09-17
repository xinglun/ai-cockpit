---
author: AI Cockpit maintainers
title: "パフォーマンスベースライン"
description: "再現可能な local performance evidence と release 上の制限。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - performance_baseline
---

# パフォーマンスベースライン（local evidence）

この baseline は次の条件で取得しました。

```text
command: cargo test -p cockpit-cli --test performance -- --nocapture
source base: 9177b119d3232bbc48dacca71c0beff31089e82b
host: aarch64-apple-darwin（Darwin arm64）
toolchain: rustc/cargo 1.94.1
profile: dev、incremental test fixture
date: 2026-08-21
```

測定時の source tree は未 commit の local candidate でした。数値は machine-specific な
baseline であり release evidence ではありません。公開前に immutable release candidate
から再測定してください。

| Surface | Fixture | 結果 |
| --- | --- | --- |
| `status` warm startup | 12 samples | 中央値 23 ms |
| repository observation（incremental cache hit） | 200 files、405 files read | 63 ms |
| knowledge 無関係 query | 10,000 records | historical records accessed 0 |

今回の status 目標（<50 ms）と incremental observation 目標（<100 ms）は達成しました。
初回の uncached scan は別に測定し、受入れ目標は incremental cache-hit path に適用します。
raw command output は release candidate の acceptance record と一緒に保持してください。

## 現在の paired capture（WI-876）

現在の候補は同じ `aarch64-apple-darwin` machine、`rustc/cargo 1.98.1`、Runtime `0.2.93` で、外部 baseline binary と個別に build した candidate binary を paired 測定しました。小規模 clean、many-file clean（ORG-X）、大量 historical Work Item（ai-investigation-orchestrator）、single-file change、multi-file change、large-file change、evidence path change の七つの isolated fixture を使い、各 operation は 100 個の有効な warm sample を持ちます。raw sample、identity、counter、unavailable reason は `.ai/evidence/WI-876-performance-current-proof/paired-current-seven-scenarios.json` に保持します。

5 ms の noise budget で比較した結果、改善は**まだ証明できません**。23 operation は noise 内、2 operation は改善、10 operation は暫定 noise budget を超えました。これは測定 evidence であり release performance pass ではありません。diagnostics on/off overhead は `diagnostics-overhead-org-x.json` に分離しています。開発 cycle cost は別報告で、現在取得できたのは Contract→checkpoint の 63,000 ms（1 sample）のみです。agent operation/preflight reject 数、verification→finish、merge 後 cleanup は明示的に unavailable です。
