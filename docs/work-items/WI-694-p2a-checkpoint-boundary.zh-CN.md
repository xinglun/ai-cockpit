---
author: AI Cockpit maintainers
title: “WI-694 — P2-A checkpoint 职责边界”
description: “在一个完整生命周期用例中分离 checkpoint 的观察、治理校验和持久化职责。”
audience: [maintainer, reviewer]
workItemId: WI-694-p2a-checkpoint-boundary
status: implemented
authority: human:repository-owner
lastVerifiedBy: WI-694-p2a-checkpoint-boundary
terminalArchive: .ai/work-items/archive/WI-694-p2a-checkpoint-boundary.contract.json
terminalVerification: .ai/evidence/WI-694-p2a-checkpoint-boundary.verification.json
terminalFinalization: .ai/decisions/WI-694-p2a-checkpoint-boundary.finalize.json
terminalDecision: .ai/decisions/WI-694-p2a-checkpoint-boundary.close.json
---

[English](WI-694-p2a-checkpoint-boundary.md) · [日本語](WI-694-p2a-checkpoint-boundary.ja.md)

# WI-694 — P2-A checkpoint 职责边界

## 边界

本 successor 从最新远程 `main` 开始，因为旧 WI-655 分支已过期。只修改
repository lifecycle 模块中的 `checkpoint_work_item` 及其定向验证；公共函数、
Summary/checkpoint evidence 序列化、错误优先级、授权语义和文件布局保持不变。

## 职责拆分

- `checkpoint_observe` 读取 active Contract、捕获一次 Git snapshot，并派生 Contract
  和 repository snapshot digest。
- `checkpoint_governance_checks` 校验当前 preflight 绑定，复用已捕获 snapshot 交给
  既有治理权威，并且不执行持久化。
- `checkpoint_work_item` 保留生命周期顺序，只有前置校验通过后才写入 checkpoint
  evidence 和 Summary。

这是一个窄化的完整用例提取，不新增 Port 或 trait，不复制治理规则，不跨编辑边界扩大
snapshot 有效期，也不改变 finish/archive/close 语义。

## 验证

生命周期入口、顺序和并发定向测试覆盖原有 fail-closed 行为。关闭前还必须完成 workspace
格式、Clippy、测试、Runtime evidence、托管检查和终态生命周期记录。

## 剩余风险

治理决定仍由既有权威负责，并可能读取 repository 声明；本 WI 只阻止 checkpoint 用例主体
重新捕获自身的 Contract/snapshot/digest。更广泛的观察上下文传递属于独立 P1-B 边界。
