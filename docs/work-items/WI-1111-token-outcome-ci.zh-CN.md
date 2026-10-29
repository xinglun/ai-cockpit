---
author: AI Cockpit maintainers
title: "WI-1111-token-outcome-ci — Usage intake and Outcome parity"
description: "已实现并归档；原生人工关闭及后续交付仍待完成。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1111-token-outcome-ci
status: implemented
authority: native-contract
lastVerifiedBy: WI-1111-token-outcome-ci
---

# WI-1111-token-outcome-ci — 用量接入、Outcome 一致性及有限 CI 提速

实现和本地验证已记录。原生 Work Item 于 2026-10-26 归档，目前为黄色状态，等待单独的真人关闭。本页不表示已关闭、已公开交付、已运行托管 CI、已测得性能收益或已取得本任务的真实 token 用量。

## 已交付并在本地验证

- 原生用量接入保留来源身份、不可变回执、可空计数和调用方来源。当前会话用量不可取得或格式不支持时继续标为未知；不把累计 Codex JSONL 当作单次调用用量。
- JSON、Markdown、CLI 和 MCP 的 Outcome 视图在中、英、日文中一致保留已知／未知用量。finish/archive 字节保持冻结，close 使用独立截止时间。
- 报告投影绑定类型化策略、生产者和渲染产物；有固定证据支持的历史格式按有限规则读取，不改写历史字节。
- 本地测试覆盖先行前检、有限拆分 Windows 命令、稳定聚合门禁、包覆盖和零警告检查。
- 原生验证回执记录 54/54 项通过、54 项新执行、0 项复用、单 worker、每项 900 秒上限、无超时、耗时 24 分 15 秒。归档 Summary 将 13/13 个场景标为已验证。固定 Runtime 为 `sha256:134567563cc90a9a20744f075952eae6812b21c7f606ae3722d49ae249b0bd5b`；归档记录绑定了对应源码、Contract 和快照。

## 证据边界与关闭前核对

- 归档 Outcome 有 16 条验收结果，但归档 Summary 只报告 14 条 `acceptanceEvidence`。只读归档 validator 返回 `acceptanceEvidence: verified`，但原始条目仍是标记未验证的计划声明，并指向已移出的 active Contract 路径；`/acceptanceCriteria/13` 和 `/acceptanceCriteria/14` 没有条目。场景文本还引用 attempt `8c1471fc…` 和 receipt digest `6dd8ccc4…`，而当前 verification 文件标识 attempt `9ecb62e3…` 和 receipt digest `c32b0b13…`。在把 54 项回执或场景状态当成 16 项验收的一一证明、或进行真人关闭前，须核对这些原生引用。
- 成功的 repository 测试节点 stdout 被 Runtime 捕获上限截断。零警告门禁通过；较早的 ENOSPC 失败尝试及其中的构建警告行仍保留。
- 三项源码材料语法分类仍为 Unknown；代理自声明审阅未将其改成 Clean。当前会话用量没有回执，所有用量计数仍为 null。面向用户的收益未知。
- 文档同步后，Runtime 状态除了待 close 外还报告 `material_review_receipt_stale` 和 `repository_material_inspection_unavailable`。没有记录新的材料审阅或正式验证；现有审阅回执仍绑定先前快照。只读归档 validator 仍判定冻结归档为 verified，但这不会清除当前状态的 Unknown。
- 最终 HEAD 的 Linux／Windows 托管 CI、真实 draft PR 运行和可比较性能测量仍未验证。归档 Contract 没有 `resourceContext`；当前模拟 PR 路由实际选择 `ordinary_repository_route`，因此不声称已存在绑定此 WI 的 PR 或托管运行。精确绑定 branch+PR 的归档 Contract 通用路由回归测试通过。
- 当前为 `archived`，不是 closed。`close_decision_pending` 和 `user_visible_benefit_not_declared` 仍存在；没有记录 close 决定。

## 原生证据

- 源码 HEAD：`e9e6526008f73459064026852d79bbaee6400c23`。
- 归档 Contract：`.ai/work-items/archive/WI-1111-token-outcome-ci.contract.json`（`sha256:a82110bf724bca97055a265b3792f95942f1b4c98f394dad52c100d350baa2f4`）。
- 归档清单和冻结产物：`.ai/work-items/archive/WI-1111-token-outcome-ci.archive.json`、`summary.json`、`outcome.json`、`task-report.json`、`task-report.md`。
- 验证回执：`.ai/evidence/WI-1111-token-outcome-ci.verification.json`。
- 材料审阅回执：`.ai/evidence/material-inspection-review/WI-1111-token-outcome-ci/99092156923ce9d1bd906919ab9cf163a5acfa7effea682805c7dc152dd5f1e9.json`。

公开源码／原生材料／辅助材料及 draft PR 需要单独、范围明确的授权。托管 CI 必须针对精确交付 HEAD 运行，并保留现有门禁。此页不授权合并、发布、生产启用、替换共享 Runtime 或真人 close。
