---
author: AI Cockpit maintainers
title: "WI-255——Recovery decision 读取侧校验"
workItemId: WI-255-recovery-read-side
description: "在 Outcome 或 archive 消费 current recovery decision 前重新校验，同时保留不可变历史记录。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-255-recovery-read-side
terminalArchive: .ai/work-items/archive/WI-255-recovery-read-side.contract.json
terminalVerification: .ai/evidence/WI-255-recovery-read-side.verification.json
terminalFinalization: .ai/decisions/WI-255-recovery-read-side.finalize.70b8faaab38e83dcd7d4fe55892abfe4c553ec1efb369bf81c2e259a9fe8566b.json
terminalDecision: .ai/decisions/WI-255-recovery-read-side.close.json
authority: canonical
---

# WI-255——Recovery decision 读取侧校验

WI-255 从同步后的 default branch 重建 current recovery 读取侧。它只选择性迁移未合并
PR #192 与 #202 中已审阅的代码、测试和用户可见边界；不会复制或改写 WI-242、
WI-248 lifecycle bytes，也不会把它们表述为当前 evidence。

## 验收边界

- 每个 current recovery candidate 必须是有大小上限的 regular non-symlink JSON
  文件。重复 key、malformed/oversized 输入，以及与 canonical JSON digest 不匹配的
  digest 后缀文件名都会 fail closed。
- Outcome 或 superseded archive 消费 candidate 前，会重新校验 repository、Work Item、
  当前 Runtime、predecessor Contract/Summary/Outcome/Events、时间戳、decision shape
  与 successor Contract 绑定。
- 任一无效 current candidate 都不能通过回退到较旧 valid candidate 而被跳过；同时间的
  valid candidates 按确定性的路径顺序选择。
- 失败统一落入 `recovery_decision_invalid`，current Outcome 为 red，且不得移动 active
  artifacts。
- 历史不可变 archive 保留其已记录 Runtime identity 与投影；current-read 规则不会追溯
  重新分类它们。

## 验证场景

Contract 要求五个场景：valid current recovery、forged current recovery、invalid current
candidate files、deterministic candidate selection 与 historical archive compatibility。
focused repository tests 使用真实 filesystem artifacts 覆盖全部场景，随后执行 repository、
documentation、governance、clippy 与 full-workspace checks。

## 生命周期投影

本行在 verified close 前保持条件状态。未来 evidence 路径为
`.ai/work-items/archive/WI-255-recovery-read-side.contract.json`、
`.ai/evidence/WI-255-recovery-read-side.verification.json`、
`.ai/decisions/WI-255-recovery-read-side.finalize.json` 与
`.ai/decisions/WI-255-recovery-read-side.close.json`。

## 参考

- [Agent workflow](../reference/agent-workflow.zh-CN.md)
- [Commands](../reference/commands.zh-CN.md)
- [Reference parity](../reference/reference-parity.zh-CN.md)
