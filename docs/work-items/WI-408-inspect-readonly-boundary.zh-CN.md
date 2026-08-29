---
author: AI Cockpit 维护者
title: WI-408——Work Item inspect 只读边界
description: 保持 work-item inspect 只读，同时保留显式 approach 物化能力。
workItemId: WI-408-inspect-readonly-boundary
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-408-inspect-readonly-boundary
---

# WI-408——Work Item inspect 只读边界

## 意图

使 `work-item inspect` 成为真实的只读投影。它必须在不静默物化仓库文件的
情况下推导兼容性、implementation approach 和并行 slot；显式的
`work-item approach` 仍然是有意写入的边界。

## 范围

- 为 inspect 增加 request-scoped、非持久化的 implementation-approach 路径。
- 保持显式 `work-item approach` 的持久化和归档消费语义不变。
- 增加 CLI 与 repository 回归测试，证明重复 inspect（含新 attach 的 adopter）不改变权威或派生字节。
- 在英文、简体中文和日文文档中说明边界，并加入防止实现或文档矛盾的静态 CI 门。

## 不在范围内

Knowledge 物化、生命周期状态转换、Agent provider/全局配置、发布/adopter
脚本，以及显式 `work-item approach` 的写入语义均不变。

## 验收

1. `work-item inspect --repo <path> --id <id>` 返回投影，但不创建或刷新
   `.ai/work-items/active/<id>.approach.json`。
2. 显式 `work-item approach` 仍创建 repository-local artifact。
3. 重复 CLI 与 repository 投影保持仓库字节不变。
4. 三语文档与静态 CI 门描述相同边界。
5. 新 attach 的 adopter 在显式 `--repo` 下具有相同隔离行为。

## 证据

Verification、repository-bound regression 与 documentation-integrity evidence
由 Runtime 生命周期记录，并在 reviewed merge 后补充链接。
