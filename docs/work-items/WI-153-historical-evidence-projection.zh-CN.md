---
author: AI Cockpit maintainers
title: "WI-153——历史证据投影"
description: "将旧 Runtime 生成的有效归档证据投影为历史状态，同时保持活动证据 fail-closed。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-153-historical-evidence-projection
workItemId: WI-153-historical-evidence-projection
---

# WI-153——历史证据投影

WI-153 保留归档 evidence 的不可变字节，同时区分“由较旧 Runtime 生成、但本身有效”的历史证据与当前验证失败。由旧 Runtime 生成的有效 v2 归档 evidence 现在显示为带有 `historical_evidence_not_revalidated` 的历史黄色状态；损坏、篡改或身份无效的 evidence 仍然 fail-closed。活动 Work Item 继续将 foreign Runtime identity 判定为红色。

同时修正三语 parity 索引，补齐 WI-147 至 WI-152。没有改写既有 Work Item archive 或 evidence 字节。

证据：`.ai/evidence/WI-153-historical-evidence-projection.verification.json`。
决定：`.ai/decisions/WI-153-historical-evidence-projection.close.json`。
