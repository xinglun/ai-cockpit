---
author: AI Cockpit maintainers
title: "WI-260 — 面向恢复的治理门"
workItemId: WI-260-recovery-gate
description: "让不可变 predecessor 的恢复状态在治理清单和文档晋级中收敛。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-260-recovery-gate
authority: canonical
---

# WI-260 — 面向恢复的治理门

## 意图

当不可变 predecessor 带有有效 recovery、但历史 close 非规范时，统一投影
为 `recovered`；普通文档晋级仍只允许 approved close。

## 范围

本 Work Item 更新 governance-integrity 清单、已关闭 Work Item 文档晋级辅助器
及其回归测试。不增加 Runtime lifecycle 行为，也不改写 WI-258 的历史 close bytes。

## 验收

- 有效 recovery 加无效历史 close 时投影为 `recovered`，且不产生
  `invalid_terminal_decision`。
- 有效 approved close 仍优先于旧 recovery。
- 文档晋级只跳过有效 recovered predecessor；无效 recovery 必须 fail closed。
- retry recovery 可以省略 `successorWorkItemId`；successor/supersede 决定仍必须显式绑定 successor。
- 不含歧义的缩写 Git revision 会解析为唯一 commit 用于 finalization 绑定；歧义或无效 revision 继续 fail closed。
- 门和晋级两侧都有回归测试。
- 三语 Work Item 与 parity 行绑定修复证据。

## 证据边界

Recovery 是历史终态投影，不是绿色完成声明。后继 Work Item 负责未来的实现晋级；
前驱原始 bytes 保持不可变并可审计。
