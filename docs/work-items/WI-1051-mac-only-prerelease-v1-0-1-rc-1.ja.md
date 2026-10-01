---
author: AI Cockpit maintainers
workItemId: WI-1051-mac-only-prerelease-v1-0-1-rc-1
title: v1.0.1-rc.1 macOS arm64 prerelease
description: v1.0.0 を stable に保ち、WI-1050 Contract amendment admission 修正を試す macOS arm64 prerelease を一つ公開する。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1051-mac-only-prerelease-v1-0-1-rc-1
---

[English](WI-1051-mac-only-prerelease-v1-0-1-rc-1.md) · [简体中文](WI-1051-mac-only-prerelease-v1-0-1-rc-1.zh-CN.md)

# WI-1051 — v1.0.1-rc.1 macOS arm64 prerelease

本 Work Item は、WI-1050 の Contract amendment admission 修正を macOS arm64
で試すため、任意参加の `v1.0.1-rc.1` prerelease を一つ準備・公開する。
stable release ではなく、`v1.0.0` が最新 stable release と installation
baseline のままである。

## 境界

- 候補ソースには WI-1050 修正 commit
  `09e7eead64bebacda98d71e1907c01e6008a8f88`（tree
  `3c2ac29a38ea1eb8d8e1c856424834761ab16630`）を含める。
- 公開するのは macOS arm64 executable と一致する SHA-256 sidecar のみ。
- non-macOS asset、stable pointer の変更、無関係な tag/release、Task9 の変更は含めない。
- Runtime formal verification、hosted CI、non-macOS validation は独立した証拠境界であり、local build で実証したと扱わない。

## macOS local evidence

次の build を独立して完了した。

```text
cargo build --locked --release --package cockpit-cli --target aarch64-apple-darwin
Finished `release` profile [optimized] target(s) in 1m 31s
```

isolated prefix にコピーした executable は `ai-cockpit 1.0.1-rc.1` を表示し、
`--help` は exit 0、file type は arm64 Mach-O である。コピーの bytes は build
artifact と一致する。生成した sidecar は `shasum -a 256 -c` で検証済み。

```text
SHA-256: 7e15777b24480dc88c880698876a0cff03aeb5dfe019a6a68db3ecfbe9409b7f
Size: 10086320 bytes
```

これは独立した local build / launch の観測であり、Runtime verification
receipt ではない。現在の Runtime Summary に formal verification evidence はなく、
Contract の 8 シナリオは Runtime 上で未検証のままである。

## 公開と acceptance の境界

prerelease notes には、未検証の Runtime review/verification、hosted CI、
non-macOS targets、regression checks を明記する。公開後に正確な provider
release を確認し、executable と checksum を download して bytes を検証し、
`v1.0.0` が最新 stable のままであることを再確認する。残りの証拠と lifecycle
decision が解決するまでは Work Item を進行中として扱う。

withdrawal が必要な場合は provider identity を確認したうえで、正確な
`v1.0.1-rc.1` prerelease、その asset と tag のみを削除する。`v1.0.0` や無関係な
release/tag は変更しない。修正版には新しい prerelease version を使う。
