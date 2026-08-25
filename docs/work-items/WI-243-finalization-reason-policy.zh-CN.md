---
author: AI Cockpit maintainers
title: "WI-243——合并前 finalization reason 政策"
workItemId: WI-243-finalization-reason-policy
description: "要求合并前 finalization 使用 Runtime 可验证的非空 reason。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-243-finalization-reason-policy
terminalArchive: .ai/work-items/archive/WI-243-finalization-reason-policy.contract.json
terminalVerification: .ai/evidence/WI-243-finalization-reason-policy.verification.json
terminalFinalization: .ai/decisions/WI-243-finalization-reason-policy.finalize.json
terminalDecision: .ai/decisions/WI-243-finalization-reason-policy.close.json
authority: canonical
---

# WI-243——合并前 finalization reason 政策

WI-243 将合并前 finalization 的 reason 定义为明确的审计字段。门禁消费
Runtime 校验过的非空文本，不要求未文档化的 magic token。repository、Contract、
evidence、Runtime、PR、base、head、resource context 和 blocked/unmerged
绑定继续失败关闭。

权威记录包括 archive Contract、verification evidence、finalization chain、
close decision 和三语 parity 行。
