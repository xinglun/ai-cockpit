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
