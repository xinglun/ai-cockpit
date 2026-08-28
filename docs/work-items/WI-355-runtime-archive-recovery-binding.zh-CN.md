---
author: Ray
title: "WI-355——Runtime archive recovery 绑定"
workItemId: WI-355-runtime-archive-recovery-binding
description: "将合法 stale retry receipt 作为 archived Work Item 的历史证据消费，同时保持 fail-closed 校验。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/work-items/WI-355-runtime-archive-recovery-binding.md
lastVerifiedBy: WI-355-runtime-archive-recovery-binding
terminalArchive: .ai/work-items/archive/WI-355-runtime-archive-recovery-binding.contract.json
terminalVerification: .ai/evidence/WI-355-runtime-archive-recovery-binding.verification.json
terminalFinalization: .ai/decisions/WI-355-runtime-archive-recovery-binding.finalize.json
terminalDecision: .ai/decisions/WI-355-runtime-archive-recovery-binding.close.json
predecessor: WI-353-runtime-recovery-delivery-binding
capabilityClaims:
  - archived_retry_recovery_binding
---

# WI-355——Runtime archive recovery 绑定

[English](WI-355-runtime-archive-recovery-binding.md) · [日本語](WI-355-runtime-archive-recovery-binding.ja.md)

## 意图与边界

本 successor Work Item 修复合法 stale retry recovery receipt 的 archived read path。
当 retry 已完成且新的 archived projection 已存在时，旧 retry receipt 应作为历史证据被消费，
不应再以当前 recovery 的身份阻断 Outcome 或 close 评估。

malformed、foreign、错误命名、ambiguous 以及仍处于 pending 的 retry evidence 继续
fail-closed。WI-353 的 archive bytes 保持不可变，不在本实现编辑范围内。

## 验证与交付边界

- 增加 archived stale-retry 回归测试，并保持既有 recovery negative tests。
- 执行 formatting、locked workspace tests、clippy、governance integrity 与 documentation
  acceptance。
- reviewed PR merge、provider finalization 验证和结构化 close 均已完成。
