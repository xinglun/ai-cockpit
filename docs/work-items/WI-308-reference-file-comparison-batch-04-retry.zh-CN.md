---
author: AI Cockpit maintainers
title: "WI-308——参考证据治理、信任层与回滚腐化批次 04 重试"
workItemId: WI-308-reference-file-comparison-batch-04-retry
description: "逐个比对四个固定参考文件，记录 Rust-native 且面向 adopter 的 parity 边界。"
audience:
  - maintainer
  - reviewer
status: in progress
lastVerifiedBy: WI-308-reference-file-comparison-batch-04-retry
authority: canonical
---

# WI-308——参考证据治理、信任层与回滚腐化批次 04 重试

## 意图与目标

本 Work Item 比对固定参考提交
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` 中的四个文件：演示 GIF、
`docs/case-study-ai-rollback-corruption.md`、`docs/concepts/evidence-governance.md`
和 `docs/concepts/trust-layer.md`。目标是为 adopter 建立有证据的逐文件 parity，
但不复制参考源 Python/Make/installer 实现或二进制资产。

## 文件结论

| 参考文件 | 分类 | 目标证据与边界 |
| --- | --- | --- |
| `docs/assets/ai-cockpit-demo.gif` | reference-only | GIF89a、800x435、587,945 字节，SHA-256 为 `88838de7221dc859efde7e8e87913d0a23a21466195647ded60612adbad1f795`；仅作视觉参考，不复制二进制。 |
| `docs/case-study-ai-rollback-corruption.md` | implemented-different-by-design | 三语 adversarial-validation 文档与 typed Contract/scope 校验覆盖越权路径、无关变更和受控恢复。案例是假设性的，不宣称自动回滚或批准合并。 |
| `docs/concepts/evidence-governance.md` | implemented-different-by-design | 企业治理、Outcome/evidence 文档和 typed Protocol/Repository 记录投影 Evidence → Governance Decision → Human Control；Provider evidence 仍由外部负责。 |
| `docs/concepts/trust-layer.md` | implemented-different-by-design | 产品边界、设计哲学、企业治理和 capability truth 文档定义校准信任、fail-closed 未知、人类控制及非目标。 |

这是语义责任 parity，不是 source wire 或字节兼容。Contract 值与 evidence 保持作者事实；
文字不是证明，本地 evidence 也不会被静默提升为 provider 或 enterprise assurance。

## Successor 与恢复边界

这部分实现最初记录在 WI-306 及其已审阅的 PR #268 中，但该归档交付从未合并。
WI-307 改变默认分支的 parity 投影后，旧 PR 若继续更新就必须改写已归档 Contract/base，
或解决归档后的分支冲突。因而 WI-306 作为不可变的历史 provider evidence 保留；本
successor 从当前远端 `main` 开始，在新的 Contract 下重新治理同一有界文件对比。旧 PR
不会被当作当前成功或失败，也不会复活。

## 范围

- 更新参考 inventory 生成器、生成的 ledger 和回归断言。
- 以中文、英文、日文记录逐文件对比。
- 在三语 adversarial-validation 文档中加入回滚腐化边界。
- 保持 reader route 与 parity ledger 同步。
- 使用显式 repository context 验证安装的共享 Runtime；adopter 工程继承同一语义，但
  `.ai/` 状态必须独立。

## 不在范围内

Rust 生产代码、新命令或治理语义、release/adopter/CI 变更、全局 Agent/MCP 配置、参考源
Python/Make/installer、参考 GIF 或其他二进制复制，以及不可变历史 evidence/archive bytes。

## 验收与验证

1. 四个固定文件均已读取并逐个分类；GIF 的摘要、类型、尺寸和大小已记录。
2. 三语对抗性验证文档说明范围违规、无关变更和已完成工作回滚风险，使用 Rust-native
   evidence 边界且不作安全能力过度声明。
3. Evidence Governance 与 Trust Layer 明确链接企业治理、Outcome/evidence、产品边界、
   设计哲学和 capability truth 文档。
4. inventory、comparison、parity 与本 Work Item 同步；WI-308 不再 deferred，也不新增
   `migrate-gap`。
5. 使用安装的 `ai-cockpit`，所有命令显式 `--repo`，完成 preflight、checkpoint、verify、
   finish、archive、审阅后的 PR、merge、finalization verification、close 和精确清理；
   最终人类 Outcome 以中文可见输出。

必需检查：`cargo test --locked --workspace`，以及 Runtime/CI 声明的参考 inventory、文档、
治理完整性和发布质量检查。
