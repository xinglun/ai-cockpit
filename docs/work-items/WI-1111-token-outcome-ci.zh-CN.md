---
author: AI Cockpit maintainers
title: "WI-1111-token-outcome-ci — Usage intake and Outcome parity"
description: "Source quality and separately authorized later delivery."
audience:
  - maintainer
  - reviewer
workItemId: WI-1111-token-outcome-ci
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1111-token-outcome-ci
---

# WI-1111-token-outcome-ci — 用量接入、Outcome 一致性及有限 CI 提速

实现计划以原生 Contract 为准。真实用户确认及 checkpoint 1 已记录，12 项必需场景均未验证；本文不宣称测试或交付成功。

经原生用量准入导入明确提供的 completed Responses 对象，保留响应身份、不可变回执、可空计数和调用方来源。当前宿主会话用量不可取得，本项不会补造历史用量；累计 Codex JSONL 不按单次调用相加。

JSON、Markdown、CLI/MCP 简短及完整 Outcome 的三语输出一致展示已知或未知用量。保留 finish／archive 冻结原件及独立 close 截止时间。

在昂贵的受管验证前执行既有文档预算、必需指引、策略／工作流／测试消费者检查。Windows 完整命令集合分入有界隔离子任务，保留稳定汇总检查、零警告、全部包覆盖及 source／tested 绑定。依据可比实际耗时判断收益。

finish 前需要源码质量证据和独立复审。公开源码／原生／辅助材料、PR／CI／main 核验及可恢复资源最终化属于后续义务，依各自实际授权执行。真正 close 另行确认。当前未授权公开推送、合并、版本发布、生产启用或共享 Runtime 替换。

当前 Contract：`.ai/work-items/active/WI-1111-token-outcome-ci.contract.json`。
以下计划路径表示义务，不表示文件已存在：

- `.ai/work-items/archive/WI-1111-token-outcome-ci.contract.json`
- `.ai/evidence/WI-1111-token-outcome-ci.verification.json`
- `.ai/decisions/WI-1111-token-outcome-ci.finalize.json`
- `.ai/decisions/WI-1111-token-outcome-ci.close.json`

保留历史失败、输出截断、未知 token 用量和收益未测量事实。
