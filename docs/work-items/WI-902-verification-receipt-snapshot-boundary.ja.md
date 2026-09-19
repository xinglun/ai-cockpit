---
author: AI Cockpit maintainers
workItemId: WI-902-verification-receipt-snapshot-boundary
title: verification receipt の snapshot 境界
description: Runtime の governance write をまたいで成功した typed verification を current のまま保ち、実際の source change では失効させる。
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:xinglun
lastVerifiedBy: WI-902-verification-receipt-snapshot-boundary
---

# WI-902 — verification receipt の snapshot 境界

この WI は Issue #902 に対応する。成功した current-Runtime typed verification
は、Runtime の governance record だけが変わった場合、直後の preflight、gate、
finish 境界でも利用できなければならない。実際の source file の変更では同じ
evidence を失効させる。typed-first verification、red preflight の zero-process
semantics、receipt identity、tamper safety は維持する。

## Acceptance と evidence

- repository-bound fixture が governance-only write で有効な verification receipt
  が stale にならないことを示す。
- source mutation は receipt を stale にし、進行を block する。
- CLI と MCP は同じ identity と snapshot 判定を使う。
- malformed、foreign-runtime、tampered、symlink receipt の既存 fail-closed
  behavior を維持する。
