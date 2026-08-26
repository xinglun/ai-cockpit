---
author: AI Cockpit maintainers
title: "WI-294——生命周期恢复状态机"
workItemId: WI-294-lifecycle-recovery-state-machine
description: "让人工授权的生命周期恢复明确、身份绑定且可重复，同时不改写 predecessor bytes。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-294-lifecycle-recovery-state-machine
terminalArchive: .ai/work-items/archive/WI-294-lifecycle-recovery-state-machine.contract.json
terminalVerification: .ai/evidence/WI-294-lifecycle-recovery-state-machine.verification.json
terminalFinalization: .ai/decisions/WI-294-lifecycle-recovery-state-machine.finalize.json
terminalDecision: .ai/decisions/WI-294-lifecycle-recovery-state-machine.close.json
authority: canonical
---

# WI-294——生命周期恢复状态机

## 意图

让生命周期门失败后的人工授权重试变得明确、安全且可重复。

## 范围

- 只恢复到合法的 checkpointed 重试状态。
- 保留 blocked Outcome、前置摘要和追加式 recovery 历史。
- 不复用过期的 report 或 completion 事件，重新执行验证与 finish。
- 让 Rust Runtime、仓库门禁和三语文档保持一致。

## 不在范围内

发布打包、adopter 验收、CI 替换和 Runtime 模块拆分属于独立边界。

## 验收

- 失败的 finish 只能通过身份绑定的 recovery receipt 重试。
- retry 不得伪造绿色 preflight，也不得改写不可变 predecessor bytes。
- 过期 recovery candidate 不得遮蔽更新的有效投影。
- superseded archive 必须保持内部 digest 绑定。
- Rust、治理、文档和 hosted checks 全部通过后才能关闭。

## 验证

见 `.ai/evidence/WI-294-lifecycle-recovery-state-machine.verification.json` 以及 reviewed PR/closure receipt。

## 未知项

Work Item owner 尚未声明面向用户的收益；该项保持显式 unknown。
