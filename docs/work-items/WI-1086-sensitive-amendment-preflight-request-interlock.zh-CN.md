---
author: AI Cockpit maintainers
title: "WI-1086 — 敏感修订 preflight 审阅请求互锁"
description: "生成当前绑定请求，同时保留收据完整性和既有阻断。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1086-sensitive-amendment-preflight-request-interlock
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1086-sensitive-amendment-preflight-request-interlock
terminalArchive: .ai/work-items/archive/WI-1086-sensitive-amendment-preflight-request-interlock.contract.json
terminalVerification: .ai/evidence/WI-1086-sensitive-amendment-preflight-request-interlock.verification.json
terminalDecision: .ai/decisions/WI-1086-sensitive-amendment-preflight-request-interlock.close.json
---

[English](WI-1086-sensitive-amendment-preflight-request-interlock.md) · [日本語](WI-1086-sensitive-amendment-preflight-request-interlock.ja.md)

# WI-1086 — 敏感修订 preflight 审阅请求互锁

本工作项进行中。Runtime 已于 2026-10-16 通过正常入口创建 before-edit checkpoint；测试、实现、正式验证及验收仍待完成。lastVerifiedBy 标识当前文档投影，不表示通过或关闭。

## Benefit

让开发者在敏感修订后取得当前绑定的审阅请求，解除请求与确认相互等待；保留原始阻断证据和严格的身份、Contract、decision、snapshot 检查。

## Design and acceptance

提交 f49a001ece01209ed9e4cf743f879b7a5931143c 的 v2 方案已获 Raydot 技术审阅通过。范围仅含完整原生 journal 与当前失效绑定、请求生成准入，以及 state/outcome/blockers 保持。CLI/MCP 同时覆盖 documentation 首错和 governance_red 复现。额外收据缺陷须先交具体 RED 和最小范围供 Raydot 审阅。

## Execution boundary

严格串行：单 worker，隔离 target/wi1086-sensitive-amendment-preflight-request-interlock，每次命令上限 900 秒。原 WI-1085 保留，它是修复未来消费者，不要求其先完成。共享源码、账本、构建和生命周期动作不并行。不伪造 lease/真人确认，不安装 Runtime，不操作 Sentinel，不合并、关闭或清理。
