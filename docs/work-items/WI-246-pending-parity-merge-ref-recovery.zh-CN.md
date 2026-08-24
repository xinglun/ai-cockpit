---
author: AI Cockpit maintainers
title: "WI-246——Pending parity merge-ref 恢复"
workItemId: WI-246-pending-parity-merge-ref-recovery
description: "恢复 WI-244 交付，并将 parity 绑定到 hosted merge ref 带入的权威决定。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-249-parity-finalization-registration
authority: canonical
---

# WI-246——Pending parity merge-ref 恢复

WI-244 已交付 typed pending parity registry，并在 PR #196 达到不可变 verified archive。
push tree 通过，但 hosted PR merge ref 还包含默认分支新加入的权威 WI-243 close receipt。
三条 WI-243 行仍只列出合并前 finalize receipt，因此治理门禁正确阻止了组合树。

## 恢复边界

- Runtime receipt `.ai/decisions/WI-244-pending-parity-registry.recovery.json`
  绑定准确的 predecessor Contract、Summary、Outcome 与 Events digest。
- WI-244 archive、verification、finalization、PR #196 与 hosted-run bytes 均不可变；
  WI-246 只投影这些记录，不改写 predecessor。
- Contract base 是 `origin/main` 的
  `3fd982560ee28563bfab69d414f60575f3b2894a`；recovery bootstrap commit
  `3a5693a` 是治理历史，不可替代 base。
- Draft PR #197 的准确 branch/worktree context 已在 checkpoint 与实现之前通过
  `finalize-plan` 绑定。

## 验收

三条 WI-243 行保留合并前 finalize 路径并加入 close 路径。WI-244 以“已恢复”展示并引用
recovery receipt。PR #197 以 `98d6575` 合并；WI-246 append-only chain 已记录合并观察、
准确 branch/worktree cleanup 与结构化 close receipt，因此 WI-246 现在为“已实现”。确定性回归构造
base-plus-feature merge tree：缺少 base close decision 时产生三条
`missing_parity_decision`，三语行同时包含两个路径后通过。pending registry 的严格 schema、
identity、Git ancestry、symlink 与 lifecycle 校验保持不变。WI-247 因自己的 parity 行投影过晚
而作为不可变 recovered predecessor 保留；WI-249 登记该恢复，并防止相同 archive 顺序缺陷。

## 验证

先执行 governance、pending-registry、manifest、route、documentation 与 parity 聚焦测试，
再运行 strict typed repository gate。Rustfmt、Clippy 与完整 workspace suite 仍为必需项。
Runtime v0.2.31 记录最终 verification、可见人类 Outcome、archive 与 append-only finalization。
