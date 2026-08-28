---
author: AI Cockpit maintainers
title: "WI-360——发布 adopter 的 close 清理"
workItemId: WI-360-release-adopter-close-cleanup
description: "修复 staged/N-1 adopter acceptance 的资源收尾与临时运行目录清理。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-360-release-adopter-close-cleanup
authority: canonical
---

# WI-360：发布 adopter 的 close 清理

## 目的

让 staged 和 N-1 发布 adopter 验收 harness 在 `close` 前完成 Runtime
生命周期收尾，不再把 feature branch 或 worktree 留在 retained 状态。

## 范围

- `tests/release/adopter_acceptance.sh`
- `tests/release/adopter_upgrade_acceptance.sh`
- 两个脚本的静态回归 wrapper
- 三语发布分发文档

该 harness 仍属于发布后验收逻辑，不放宽 Runtime 的资源收尾规则，也不修改
不可变的 `v0.2.36` staged 失败事实。

## 设计

每个 fixture 使用一个幸存的 control checkout 和一个专用的 lifecycle
checkout。归档后，harness 提交生成的归档记录，将其 fast-forward 到 control
checkout，删除精确的 lifecycle checkout 与 branch，并记录
`disposition: deleted` 的资源收尾 receipt。随后从幸存的 control worktree
执行 `finalize`、`finalize-verify` 和 `close`。

EXIT trap 仍会在删除经过校验的临时 `run_root` 前写入 acceptance receipt 和
checksums。成功、失败和中断路径都会记录 cleanup 状态；清理失败时保留
receipt 并返回非零。

## 验收

- staged adopter lifecycle 以 `disposition: deleted` 成功到达 `close`；
- N-1 的 old/new 两条 lifecycle 都执行同样的收尾；
- 没有 receipt 声称未执行的 retained 资源状态；
- 静态测试拒绝 retained close receipt，并要求删除 branch/worktree；
- 三语文档说明 control-worktree 过渡，并说明不可变的 `v0.2.36` staged 失败历史；
- source checkout 与禁止写入的 HOME/XDG root 保持不变；
- 成功和失败路径都会删除精确的临时 run root。

## 验证证据

先由静态 wrapper 验证 release harness，再由公开发布 artifact 的 staged/N-1
验收 job 验证。receipt 记录 Runtime identity、repository identity、lifecycle
输出、隔离 manifest、cleanup 状态和 checksums。发布后验收失败不会重写 Release
是否发布的事实。
