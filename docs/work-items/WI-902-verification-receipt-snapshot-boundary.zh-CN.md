---
author: AI Cockpit maintainers
workItemId: WI-902-verification-receipt-snapshot-boundary
title: 验证 receipt 的快照边界
description: 让成功的 typed verification 跨越 Runtime 治理写入仍保持当前，同时对真实源码变更失效。
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:xinglun
lastVerifiedBy: WI-902-verification-receipt-snapshot-boundary
---

# WI-902 — 验证 receipt 的快照边界

本 WI 对应 Issue #902。成功的 current-Runtime typed verification 在随后紧邻的
preflight、gate 和 finish 边界中，如只有 Runtime 治理记录发生变化，必须仍可使用；
真实源码文件变更仍必须使同一证据失效。修复保留 typed-first verification、红色
preflight 的零进程语义、receipt 身份和篡改防护。

## 验收与证据

- 仓库绑定 fixture 证明仅治理写入不会使有效 verification receipt 过期。
- 源码变更会使 receipt 过期并阻止继续。
- CLI 与 MCP 使用相同的身份和快照判定。
- malformed、foreign-runtime、tampered 和 symlink receipt 的既有 fail-closed
  行为保持不变。
