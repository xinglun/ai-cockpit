---
author: AI Cockpit maintainers
title: "WI-512——治理参考文档批次 33"
description: "逐个比较治理与验证边界参考页的有界批次。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-512-reference-docs-batch-33
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-512-reference-docs-batch-33
terminalArchive: .ai/work-items/archive/WI-512-reference-docs-batch-33.contract.json
terminalVerification: .ai/evidence/WI-512-reference-docs-batch-33.verification.json
terminalFinalization: .ai/decisions/WI-512-reference-docs-batch-33.finalize.json
terminalDecision: .ai/decisions/WI-512-reference-docs-batch-33.close.json
---

[English](WI-512-reference-docs-batch-33.md) · [日本語](WI-512-reference-docs-batch-33.ja.md)

## 目标

在固定的本地参考提交上逐个阅读参考页，为每个路径记录有界的 Rust-native counterpart。保留语义责任，但明确不复制源 Python、Shell、Make、provider、installer、wizard 或 JSON wire。

## 逐文件决定

| 固定源路径与摘要 | 分类 | Rust counterpart / 非声明 |
| --- | --- | --- |
| `docs/reference/schemas.md` — `4ed6c44bfcfea93300c39fa467170902932e4371d218f09269bed9da26fbf625` | implemented-different-by-design | `docs/reference/schemas.*`、typed Protocol 与 schema tests；源 registry 不是 wire 要求。 |
| `docs/reference/test-architecture.md` — `3c475a84e6b7634c6d98c44af029d6b01aff6a36da5649195d1daa1d52d2a82f` | implemented-different-by-design | 三语测试架构、动态 quality route 与治理 gate；tier 不等于 assurance。 |
| `docs/reference/test-weakening-guard.md` — `17824614224f43bde778ab3985d1abc42d6c53ad0b5a5a26d3fc371e25a3ba7c` | implemented-different-by-design | Rust governance signals/回归与三语 guard 文档；不复制源实现。 |
| `docs/reference/test-weakening-guard.zh-CN.md` — `9b5b06cc25f0e05443a3b5b9181b3a04c076a9e67c5d8f17c02f2f45412f548e` | implemented-different-by-design | 同一 typed guard 边界的中文 presentation；locale 不授予 authority。 |
| `docs/reference/test-weakening-guard.ja.md` — `0ba5c1fd600990111a1942dcd48e4e4bda5903f6119fc51bbd67f0ffd7702b76` | implemented-different-by-design | 同一 typed guard 边界的日文 presentation；locale 不授予 authority。 |
| `docs/reference/verification-fixture-boundary.md` — `712ecf6a4aed8793464b40ac41cb8b9d19a47663da5c28b61403e14552990f1e` | implemented-different-by-design | 三语 fixture 边界及 isolation/adopter manifest；不复制源 helper bytes。 |
| `docs/reference/troubleshooting.md` — `57f2415177d9135c506ef9c325dd7dc8bb989ee4801907da173bac5df640dee3` | implemented-different-by-design（WI-504 重新核对） | 显式 `--repo` Runtime recovery；provider wizard 和工具链仍由外部负责。 |
| `docs/reference/troubleshooting.ja.md` — `0addca04e66d0118311cf7a169b8dd060d42b500c265f85496b014300749bbf9` | implemented-different-by-design | 日文 recovery 与 adapter 边界；不复制源 session 实现。 |
| `docs/reference/upgrade.md` — `3ebbb05b52a281c1974dc446e6707fc8cbd5f3fddd2897f6c8bf868133ac92f4` | implemented-different-by-design | shared Runtime upgrade 与显式 repository migration、不可变 evidence。 |
| `docs/reference/upgrade.ja.md` — `48367289304c82e14a7ace646092b6f115b1c18b4ee01ab96122f41255ec01e9` | implemented-different-by-design | 日文 upgrade/migration presentation；不复制源 installer 或 locale JSON。 |
| `docs/reference/work-item-lifecycle-closure.md` — `91fab8d045cd45eeb616d768e174ba840b5c5f7ba1a0a4a819065515822ad324` | implemented-different-by-design（WI-504 重新核对） | Rust finalize/close、recovery 和 `ready_on_base`；源 Make/Python recovery 不是 Rust 命令。 |
| `docs/reference/work-item-lifecycle-closure.ja.md` — `e47bdb794178855b2f4dad4b40fc6d5ee4f150d1c2c58a3f69eb897835593ea3` | implemented-different-by-design | 日文 closure 与历史 recovery 边界；provider 专用路线仍是外部责任。 |

目标与对象工程继承 shared external Runtime、隔离 repository context、Contract/evidence/knowledge 记录、Agent adapter 边界和 human Outcome 交接；不会继承源专用命令、provider policy、generated history 或源 wire shape。WI-504 的两个路径重新核对但不重复登记，其余十个路径有当前 WI-512 记录。

## 验收与验证

- 每行均在源提交 `fde3380f81fea5fd2e288f7a8849f737dc074060` 阅读。
- 每行都有分类、counterpart 和明确非声明；本批不产生 `migrate-gap` 或 deferred 记录。
- 台账仍为 5,119 个路径，669 个 retired 路径的追加记录保留；不修改参考源或对象工程。
- 三语 comparison、parity、documentation、inventory 和 workspace 检查按 Contract 执行，并写入终态 evidence。

这是语义/文档 parity，不是源命令、Python 模块或 JSON-wire 兼容。
