---
author: AI Cockpit maintainers
title: "WI-161——历史 Runtime 证据的关闭兼容"
description: "保持归档证据不可变，同时允许 Runtime 升级后关闭 Work Item。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-161-historical-runtime-close
workItemId: WI-161-historical-runtime-close
---

# WI-161——历史 Runtime 证据的关闭兼容

## 意图

Runtime 升级不能让已经归档的 Work Item 永远无法 close。Active Work Item
仍严格绑定执行 verification 的 Runtime；归档 evidence 是不可变的历史事实，
升级后应按历史 evidence 投影。

## 边界

关闭已归档 Work Item 时，Runtime 先不应用当前 Runtime identity，验证归档的
verification evidence。如果这些 bytes 其他方面有效，不同 Runtime 只是明确的
历史兼容情况，不是当前验证失败。资源收尾仍是 request-scoped，并且必须绑定
执行 `close` 的 Runtime。

这不会改写 evidence、把历史 evidence 变绿，也不会削弱 active 的
`finish`/`archive` gate。

WI-159 引入的 Runtime command 与 receipt 边界保持不变；本 Work Item 只定义历史兼容路径。

## 验收

1. Active lifecycle 拒绝 foreign Runtime verification evidence。
2. 归档的 foreign-Runtime evidence 显示为历史事实，并继续绑定 digest、identity
   和 archive manifest。
3. Runtime 升级后的 close 只有在满足当前资源收尾要求时才成功。
4. 英文、简体中文和日文 workflow/parity 文档描述相同的 Runtime 与历史 evidence 边界。

## 验证

Evidence：`.ai/evidence/WI-161-historical-runtime-close.verification.json`。
Archive：`.ai/work-items/archive/WI-161-historical-runtime-close.archive.json`。
