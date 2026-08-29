---
author: AI Cockpit maintainers
title: "WI-387——参考文档第 20 批"
workItemId: WI-387-reference-documentation-batch-20
description: "逐一比较四个固定安全与供应链文档，记录有界的 Rust-native parity，不复制源 authority。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-387-reference-documentation-batch-20
terminalArchive: .ai/work-items/archive/WI-387-reference-documentation-batch-20.contract.json
terminalVerification: .ai/evidence/WI-387-reference-documentation-batch-20.verification.json
terminalFinalization: .ai/decisions/WI-387-reference-documentation-batch-20.finalize.edfed06a65d511b9c23bddb70acd78685adbb2caefa38024af11721e276e4839.json
terminalDecision: .ai/decisions/WI-387-reference-documentation-batch-20.close.json
---

# WI-387——参考文档第 20 批

## 意图与边界

在固定参考提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 上逐一比较四个 deferred 安全与供应链文档，
并在 inventory 与三语 parity 台账中为每个文件记录一个有界决定。

目标是语义/文档 parity，不是源命令、JSON-wire 或 Provider 状态兼容。Rust-native Runtime 可以在仓库操作
与声明的治理事实冲突时拒绝或暂停，但不是通用提示词注入检测器。供应链 provenance、签名、SBOM、漏洞结果和
信任根仍属于外部委托证据。不复制源 Python、Make、Provider 配置或历史证据作为当前 authority。

## 文件决定

| 固定路径 | 决定 | 目标维护边界 |
| --- | --- | --- |
| `docs/security/injection-boundary.ja.md` | `implemented-different-by-design` | 日语 `adversarial-validation`、`input-trust-dataflow` 和 `operation-time-policy-reevaluation` 保留有界注入处理、fail-closed 重评估和外部控制限制。 |
| `docs/security/injection-boundary.md` | `implemented-different-by-design` | Rust-native 安全/信任流文档保留仓库治理边界；不可信文本仍是数据，不声明通用检测器。 |
| `docs/security/injection-boundary.zh-CN.md` | `implemented-different-by-design` | 中文 Rust-native 安全/信任流文档保留确定性 fail-closed 处理和明确非声明。 |
| `docs/security/supply-chain.md` | `implemented-different-by-design` | threat-model、ci-release-evidence、distribution、security-release-verification 文档保留委托证据责任与精确制品绑定；Runtime 不生成外部 assurance。 |

## 验收

- 四个固定源文件均已阅读；每个文件在 inventory 中只有一个分类、明确 Rust-native 对应和有界理由；`migrate-gap` 保持为 0。
- 英文、中文、日文 comparison/parity 台账描述相同的四项决定，计数更新为 `4262/298/1/4/47/507/0`。
- 注入与供应链边界区分本地治理证据和外部 provider/安全控制，不复制源命令或历史结论。
- 每个 attach 的对象/adopter 仓库都通过共享 Runtime 继承相同 Rust-native 文档边界，而 repository facts、Work Item、evidence 和 snapshot 继续由显式 `--repo` 隔离。
- 文档、inventory、治理及已安装 Runtime 生命周期检查通过；不修改无关 Runtime 代码或历史 evidence。

## 验证

声明的检查包括 reference inventory 文档/脚本测试、文档状态一致性、治理完整性 gate，以及使用显式仓库上下文的已安装 Runtime `inspect`、`status`、`doctor`、`preflight`、`checkpoint`、`verify`、`finish`、`archive`、`close` 生命周期。

[English](WI-387-reference-documentation-batch-20.md) · [日本語](WI-387-reference-documentation-batch-20.ja.md)
