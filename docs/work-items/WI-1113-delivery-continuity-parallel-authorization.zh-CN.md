---
author: AI Cockpit maintainers
title: "WI-1113-delivery-continuity-parallel-authorization — 有界交付连续性与并行工作项授权"
description: "承接 Windows 归档修复，并处理回执迁移、明确的一次性范围授权、并行工作项协调、有界重试和中断恢复。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1113-delivery-continuity-parallel-authorization
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1113-delivery-continuity-parallel-authorization
---

# WI-1113 — 交付连续性与并行授权

此工作项是 WI-1112 的原生 successor；固定 Runtime 已将 WI-1112 保留为 `replaced/not_verified`，并保全原始证据。固定 Runtime 于 2026-10-26T21:14:40Z 记录了 WI-1113 staged activation。当前有效 Contract 为 `sha256:98fa4e72be9f32e399e1b7e5c05df89e8092e4461b3a5df0d96b8e36b5e9736d`，`baseRevision` 为 `b4adab2e55119d50bb72ec29f3fdfd2693d14504`；前任 recovery HEAD `5a942f93acaf0a514a291e39674465feddfa69e7` 是独立身份。当前 Contract 的精确 native owner 确认仍待处理。尚无实现或正式验证记录；25 个必需场景均未验证。

## 范围与边界

- 保留 WI-1112 的 Windows 本地 Git 路径和 `.ai` 产物精确字节保护、严格摘要检查及单字节篡改拒绝。
- 增加由已验证恢复 lineage 派生的只读交付连续性投影，区分前任本地归档、当前 successor、外部检查和 Runtime 准入的下一动作。归档或创建 PR 本身不代表交付完成。
- 增加 append-only 材料 profile 迁移：依据历史 Contract、profile、request、manifest 和合法的同一工作项修约链验证旧回执；保留旧回执及指针历史；打开新 request。迁移本身不授予新批准，也不把 Unknown 变为 Clean。未支持、跨身份、损坏或已变化的材料仍保持阻塞。
- 增加修约前 lint，提前发现无法合法迁移的回执。
- 为明确列出的同仓库工作项增加精确范围授权清单、独立子授权和状态、逐项原子启动额度、仅对未变化检查的有界重试、中断恢复和准确的部分结果。禁止工作项间授权继承、隐式成员、共享计数器、任意命令重试及删除失败历史。
- 保持 CLI/MCP 行为一致，并提供英、日、简体中文说明。不改 CI workflow、削弱断言或全局 Git 设置；不合入 main、不发布、不启用生产、不替换固定 Runtime，也不在缺少独立精确决定时 close。

## 验证计划

Contract 声明 25 个必需场景和 19 条验证命令：14 个 workspace 包各自串行运行 all-targets 测试，并分别执行严格 Clippy、route/resolver、format 和 diff 检查。覆盖 Windows 路径和字节完整性、有效/无效 successor lineage、归档与交付状态区分、精确材料引用及差异、append-only 迁移负例、修约前 lint、明确 batch 成员及独立授权、原子额度、有界重试与执行结果不确定、中间结果、CLI/MCP parity，以及 scope/target/expiry。Runtime 记录真实证据前，所有场景均为 `unverified`。hosted CI run 总计最多 15 次，保留已使用的 7 次，自动触发、入口失败、rerun 和合入后的 main push run 都计入；每个本地检查最多 900 秒、单 worker，同一未变化检查最多追加尝试 2 次。


## PR 比较与基线

WI-1113 PR 从 #1075 当前 head 继续并指向 WI-1111 branch，因此比较结果包含从 WI-1112 继承的变更和 WI-1113 新增变更。无需先合并 #1075；保留 #1074/#1075 原样打开。新 PR 必须审阅完整累积 diff 并追溯每项来源，不能称为纯 WI-1113 差异。分别记录前任 recovery identity、固定 Runtime 在正常 staged activation 时写入的 Contract `baseRevision`、provider 比较 base。

## 当前状态

typed plan 已记录完整累积 PR 比较和按包验证计划。WI-1113 当前为 `implementation_active`，但精确 native owner 确认仍待处理；尚未绑定 PR/resource context。本文反映当前原生状态，不表示已经实现、验证、通过 hosted CI 或证明用户可见收益。
