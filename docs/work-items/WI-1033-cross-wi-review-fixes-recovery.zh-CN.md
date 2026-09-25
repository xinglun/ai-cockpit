---
author: AI Cockpit 维护者
title: WI-1033——跨 WI 审查修复恢复
description: 为选定的跨 WI 审查修复链完成 Runtime 绑定验收与生命周期收尾。
workItemId: WI-1033-cross-wi-review-fixes-recovery
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-1033-cross-wi-review-fixes-recovery
---

# WI-1033——跨 WI 审查修复恢复

## 意图

沿用同一有限的跨 Work Item 审查修复目标，完成 Runtime 验证、独立 PR 审查、合并和精确清理；停在发布之前。

## 恢复链与当前状态

WI-1033 是 WI-1032 的 Runtime 绑定恢复后继项。Runtime 已将 WI-1032 归档为 replaced，verification claim 为 not_verified，并保留其源码记录与失败准入 evidence。实现提交已经存在，但 WI-1033 仍需自己的 Contract 绑定验证；既有本地测试报告不是 Runtime verification receipt。

较早的 WI-1031 归档项有有效的历史验证，但仍待人工 close。其字节保持不可变；只有 Runtime 收齐终态 evidence 后，才能通过所选 successor lineage 处理。

独立复审发现目录句柄交换竞态之前，Runtime 曾在提交 `6f5c4aaf` 上完成 34/34 节点的完整 canonical verification；该回执现为历史证据，不能绑定当前树。之后 Runtime 曾在 `faa3eb4` 上完成 35/35 节点，但该回执早于必需检查环境 overlay 的 Contract 修订，因此同样不再是当前证据。独立复审证明未绑定的环境 overlay 可以让匹配的必需命令以 `forged-pass` 成功；新增回归先复现了这一点，窄范围拦截随后使 admission 套件 38/38 通过（发生在 Contract 修订之前）。Contract 现有 29 项验收标准、24 个场景，并要求 CI 实际执行 Windows 专属的目录重命名拒绝回归。Hosted CI run `36155139121` 的 governance-integrity 和 Windows-runtime job 失败；诊断附件指出 WI-1031 缺少 close 证据、三语 parity 过期、日文 parity 缺少 WI-1032/WI-1033 行，以及 Windows library tests 缺少 `tempfile` 依赖。当前证据新鲜度以 Runtime 为准。在本页记录的检查点，修订后完整 canonical verification、成功的 hosted CI、独立复审、合并和精确清理仍待完成；不宣称发布。

## 范围

- 重新核验受信登记/组合身份（包括执行仓库与 CoordinationStore 的 Git common directory 一致）、必需检查完整性及环境绑定（拒绝 Contract 未声明的调用方 overlay）、崩溃与活动锁安全、证据边界、按 provider 区分的依赖、CLI 实际复用、MCP 身份 parity 及只读查询/写入边界。
- 证明真实多进程 linked-worktree 行为，并保留普通单 WI 串行执行。
- 保留提供者删除产出后的跨依赖链失效；拒绝不安全或绕过确认初态的协调请求，并在查询 Contract facts 前绑定加载到的 registration 身份。
- 使用候选 CLI 检查 Sentinel，且不写入其源码、Contract/evidence、生命周期 Runtime 或协调存储。
- 在现有 Windows CI job 中运行 repository 库级 containment 回归，并由 gate-manifest 回归固定该步骤。
- 维护英文、简体中文和日文投影与 reference parity。

## 不在范围内

Task 8（跨提交的 Runtime snapshot binding）按要求作为后续独立串行 WI。Runtime 升级、发布准备、tag 变更、公共发布、Sentinel 写入及改写历史记录均不在范围内。

## 验收与验证

参见[规格](WI-1033-cross-wi-review-fixes-recovery/spec.md)和[实施计划](WI-1033-cross-wi-review-fixes-recovery/implementation-plan.md)。二十四个必需场景均声明了明确的预期结果和验证计划。文档 canonical gate 为 `bash tests/docs/documentation_acceptance.sh`。

## 交付边界

Work Item 仍在进行中。当前验证新鲜度只以 Runtime 为准；此前回执作为历史证据保留。完成前仍需最终 Runtime 验证、独立审查、hosted CI、合并、精确清理，以及 Runtime 实际允许的历史 lineage close。发布不在本 WI 范围内。
