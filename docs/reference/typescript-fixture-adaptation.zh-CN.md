---
author: AI Cockpit maintainers
title: “TypeScript Web fixture 适配边界”
description: “逐文件记录固定 TypeScript Web fixture 的 Rust-native 映射，不复制应用、npm 工具链或生命周期脚本。”
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-432-reference-typescript-fixture-boundary
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# TypeScript Web fixture 适配边界

本页逐一比较固定参考 fixture `examples/fixtures/typescript-web/` 中的 11 个文件。
它保留对 TypeScript/Web 对象工程有用的语义，但不会把 fixture 应用、npm 依赖、Node
命令或源生命周期实现复制进 Rust Runtime。

[English](typescript-fixture-adaptation.md) · [简体中文](typescript-fixture-adaptation.zh-CN.md) · [日本語](typescript-fixture-adaptation.ja.md)

## 逐文件映射

| 固定源文件 | 源文件事实 | Rust-native 对应与边界 |
| --- | --- | --- |
| `.gitignore` | 忽略 `node_modules/`、`dist/` 和生成的 `.fixture-state.json`。 | 构建产物清理由对象工程负责；发布 harness 使用自己的隔离目录，不生成或复制该 ignore 文件。 |
| `evidence.json` | 描述本地 npm 生命周期证据，并明确 Provider evidence 不可用。 | Runtime verification 绑定仓库、snapshot、Runtime、命令与结果身份；源本地证据不会被提升为 Provider、托管 CI、sandbox、不可变审计或企业证据。 |
| `fixture.json` | 声明 TypeScript Web stack、Node/npm/TypeScript 工具链、平台、安全路径和测试路径。 | Project Observer/Profile 可以记录已确认的对象工程事实；Runtime 不从 fixture 元数据推断 capability、平台就绪度或 Contract scope。 |
| `package-lock.json` | 锁定 TypeScript 5.8.3 npm 依赖及 registry integrity。 | 依赖清单与 registry 属于对象工程；共享 Runtime 不安装 Node 包、不携带此 lockfile，也不把它当作 Runtime supply-chain evidence。 |
| `package.json` | 定义 build、test、lint、format-check 和 lifecycle npm scripts。 | 对象工程在 Contract 中提供显式 verification argv；Runtime 记录结果，并把治理 lifecycle（`preflight` 到 `close`）与 npm 编排分开。 |
| `scripts/format-check.mjs` | 检查 `src/index.ts` 末尾换行并拒绝 tab。 | 这是 fixture 专用格式规则；对象工程可以声明自己的 formatter 命令，本地结果只绑定为本地证据。 |
| `scripts/lifecycle.mjs` | 执行 install/configure/normal，阻塞模糊和关键域请求，演练 upgrade/rollback 并执行 release checks。 | 已安装 Runtime 提供仓库绑定 lifecycle、人工 review 暂停、证据绑定、恢复和可见 Outcome；不执行或复制源 Node 脚本作为 Runtime authority。 |
| `scripts/lint.mjs` | 检查样例源码包含 `evaluateRequest` 且不含 `any`。 | 这是应用专用 lint，不是可移植治理控制；lint 命令与 acceptance evidence 由对象工程负责。 |
| `src/index.ts` | 样例请求 evaluator 返回 `allow` 或 `block`，并给出原因与恢复条件。 | 应用行为由对象工程负责；Runtime 的决定与停止状态是类型化治理记录，不导入或推断样例策略。 |
| `test/index.test.mjs` | Node tests 断言正常请求允许、危险请求阻塞。 | 对象工程提供并执行自己的测试命令；源 fixture 断言永远不会被提升为 Runtime、Provider 或企业证据。 |
| `tsconfig.json` | 使用 strict TypeScript、NodeNext module 和 declaration 输出。 | TypeScript compiler 配置由对象工程负责；Runtime 接受显式命令结果，但不承诺 Node/TypeScript 工具链或复制 compiler 配置。 |

## 安装与对象工程边界

fixture 的 stack 元数据不是 AI Cockpit 安装方案。先在对象工程外安装一份共享 Runtime，
再显式 attach：

```bash
repo=/path/to/typescript-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

Node.js、npm、TypeScript、依赖锁、构建产物和 hosted/provider evidence 由对象工程负责。
之后每次 Runtime 命令都带同一个显式 `--repo`；Contract scope、profile、snapshot、
evidence、knowledge 和 Agent adapter 记录均保持仓库本地隔离。

## 对象工程继承什么

已 attach 的 TypeScript/Web 工程继承共享 Runtime 的 Contract 校验、未知项 fail-closed、
身份绑定证据、生命周期和面向人的 Outcome 规则。它不会继承 fixture 的 Node 依赖、npm
脚本、应用代码、测试，也不能据此声称命令已经运行。除非相应外部权威提供证据，本地 npm
结果不等同于 Provider、托管 CI、Release 或企业证据。

这是语义/文档对齐，不是 TypeScript toolchain 支持、源命令兼容或 JSON-wire 兼容。第二技术栈
adopter 验收仍需单独授权的发布后 Work Item。

[参考索引](README.zh-CN.md) · [参考源逐文件比较](reference-file-comparison.zh-CN.md)
