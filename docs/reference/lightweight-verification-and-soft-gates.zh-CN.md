---
author: AI Cockpit maintainers
title: "轻量验证与软门"
description: "按比例选择验证强度，同时不削弱强制治理控制。"
audience: [adopter, maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# 轻量验证与软门

[English](lightweight-verification-and-soft-gates.md) · [日本語](lightweight-verification-and-soft-gates.ja.md)

AI Cockpit 根据仓库事实、Work Item Contract、阶段和适用策略选择验证
路线。`light`、`standard`、`strict` 表示验证强度，不表示证据 assurance，
也不会授予权限。

## 规则

- 路线可以增加检查；缓存只有在内容、diff、环境、Runtime、策略、仓库、
  Work Item 和阶段的所有绑定都一致时才能复用。
- 依赖规划必须确定性。循环、格式错误或未知依赖不能被静默视为完成，
  保持 `partial` 或 `unknown`，并升级受影响的检查。
- 升级是单调的：`light → standard → strict` 只能增加必要工作；成本、复用
  或 provider 提示不能降低要求。
- soft、跳过或 advisory 观察必须在证据中可见；缺失、过期、矛盾或受保护
  的证据不能因此变绿。

规范门使用显式仓库上下文：

```sh
ai-cockpit gate --repo /path/to/repository --contract .ai/work-items/active/WI.contract.json
```

门收据只说明路线，不是执行令牌。Hosted CI、发布、provider 和企业
assurance 仍是独立的委托边界。参见[治理配置](governance-profiles.zh-CN.md)
和[验证语义](verification-semantics.zh-CN.md)。

## 对象工程继承

每个 adopter 使用同一份共享 Runtime 和显式 `--repo` 绑定。路线和证据
保存在各自仓库，没有全局当前项目或 Work Item。较轻路线只是按比例选择
验证强度，不是省略 Contract、人工复核、范围或证据完整性的许可。
