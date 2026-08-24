---
author: AI Cockpit maintainers
title: "WI-251——Outcome 交接 base 绑定恢复"
workItemId: WI-251-outcome-handoff-base-binding-recovery
description: "重新交付直接生命周期 Outcome，并让资源收尾拒绝归档 Contract 与 PR base 不一致。"
audience:
  - adopter
  - maintainer
status: current
lastVerifiedBy: WI-251-outcome-handoff-base-binding-recovery
authority: canonical
---

# WI-251——Outcome 交接 base 绑定恢复

WI-250 已生成不可变的 verified archive 与 canonical finalization receipt，但 hosted
governance 发现 rebase 后归档 Contract base 与 provider PR base 不同；已安装 Runtime
却把该 sequence-0 receipt 报告为 verified。WI-251 保留 predecessor 全部 bytes，绑定
recovery decision，并从正确的当前 base 重新交付 Outcome handoff。

## 行为

- 直接生命周期 handoff 保持向后兼容：`finish`、`archive`、`close` 保留 stdout JSON，
  默认在 stderr 渲染已校验的人类 Outcome，`--json` 则抑制该 handoff。
- 被阻止的 `finish` 渲染已持久化的红色或黄色 Outcome，并保留原有 nonzero 结果。
- 资源收尾记录在写入 canonical 或 transition decision 之前，会拒绝
  `pullRequest.baseRevision` 与归档 Contract `baseRevision` 不一致的 receipt。
- `finalize-verify` 会重复同一 cross-binding 校验，包括 canonical sequence 0；已有
  mismatch 绝不能报告为 verified。

## 不可变边界

archive 会冻结 Contract base。rebase 必须在 Work Item active 时完成，随后刷新 Contract
绑定并重新评审。archive 之后改变 base 必须 fail closed 并走 recovery；archive 与
finalization receipt 都不得改写。WI-250 的 archive、evidence、Outcome、events 与
finalization bytes 保持为历史事实，其 recovery decision 指向 WI-251。

## 验证

repository 回归覆盖记录拒绝且不生成 decision、测试 fixture 受控篡改后的 sequence-0
verify 拒绝、matching-base 成功，以及现有 transition controls。CLI 测试覆盖三种语言、
stdout 兼容、`--json`、结构化决定和 blocked handoff。documentation、parity、governance、
formatting、Clippy 与 locked workspace suite 仍为必需项。
