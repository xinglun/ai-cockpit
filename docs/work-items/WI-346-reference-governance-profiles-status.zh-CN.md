---
author: AI Cockpit 维护者
title: "WI-346——治理配置与 Cockpit 状态阅读"
workItemId: WI-346-reference-governance-profiles-status
description: "逐一比较六个固定的治理配置和状态阅读文档，补充有界的三语 Rust 指引。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-346-reference-governance-profiles-status
terminalArchive: .ai/work-items/archive/WI-346-reference-governance-profiles-status.contract.json
terminalVerification: .ai/evidence/WI-346-reference-governance-profiles-status.verification.json
terminalFinalization: .ai/decisions/WI-346-reference-governance-profiles-status.finalize.json
terminalDecision: .ai/decisions/WI-346-reference-governance-profiles-status.close.json
capabilityClaims:
  - reference_parity
---

# WI-346——治理配置与 Cockpit 状态阅读

[English](WI-346-reference-governance-profiles-status.md) · [日本語](WI-346-reference-governance-profiles-status.ja.md)

## 意图与边界

本 Work Item 逐一比较六个固定参考文档：三种治理配置文档和三种面向人的状态阅读文档。
目标是让 adopter 获得有用指引，但不复制源 Make/Python 编排、源 JSON wire shape 或 provider/global 配置。

目标对象是共享 Rust Runtime 及仓库本地文档。对象工程/adopter 工程继续使用显式 `--repo`、隔离的
`.ai/` 状态、人类拥有的 Contract 决定和可见的 Outcome 交接。

## 逐文件决定

| 固定参考路径 | 分类 | 有界目标决定 |
| --- | --- | --- |
| `docs/reference/governance-profiles.ja.md` | `implemented-different-by-design` | 增加日语路线，说明按比例选择 Light/Standard/Strict、release 升级、强制控制 fail-closed，以及 tier、assurance、cost 分离。 |
| `docs/reference/governance-profiles.md` | `implemented-different-by-design` | 增加英文规范路线，把源 profile 指引映射到 `gate --repo`、类型化 Contract/verification 证据和 Rust/CI 边界。 |
| `docs/reference/governance-profiles.zh-CN.md` | `implemented-different-by-design` | 增加中文路线，表达相同事实，不声明源专有命令。 |
| `docs/reference/how-to-read-cockpit-status.ja.md` | `implemented-different-by-design` | 增加日语阅读路线，说明面向人的 Outcome 颜色、停止条件、证据边界和下一步。 |
| `docs/reference/how-to-read-cockpit-status.md` | `implemented-different-by-design` | 增加英文规范阅读路线，把源 reader 标签映射到 Rust Outcome 章节。 |
| `docs/reference/how-to-read-cockpit-status.zh-CN.md` | `implemented-different-by-design` | 增加中文阅读路线，保留 Contract 原文和人工决定边界。 |

comparison/parity 台账、reference index 链接、inventory 脚本与清单，以及本三语记录，均属于交付证据。

## 验收与验证

- 每个固定路径在 inventory 中恰好出现一次，分类如上，不保留 deferred 或 migrate-gap。
- 六个页面在三种语言的 reference index 中互相链接。
- Profile 页面说明 `VerificationTier`、`EvidenceAssurance` 和成本正交；release 是操作类别；强制控制和无效
  override fail-closed；源 Make/Python 命令不是 Rust 要求。
- Status 页面说明 🟢/🟡/🔴 和 `unknown` 是语义信号，保留 Contract 原文，区分 CLI/MCP 面向人的交接与机器 JSON，
  并明确对象/adopter 的 `--repo` 边界。
- 三语 comparison、三语 parity 和机器清单保持固定参考身份和当前计数一致。
- 文档、inventory、治理完整性、格式、lint 和锁定 workspace 验证通过。不修改 Runtime 代码或全局 Agent/MCP 配置。

固定参考提交：`e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
目标基线提交：`8bf06612a0f0a8adda0aacfdf65e17ece9c2ca0f`。

