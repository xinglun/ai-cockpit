---
author: AI Cockpit maintainers
title: "Mixed-monorepo fixture 适配"
description: "不复制业务代码或工具链，对固定 mixed Python/Node fixture 建立逐文件 Rust 原生边界。"
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-420-reference-mixed-monorepo
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# Mixed-monorepo fixture 适配

本页逐一比较固定参考源 `examples/fixtures/mixed-monorepo/` 下的五个文件。
该 fixture 是可执行的业务示例，不是 Rust Runtime 代码或可移植的企业证据。
目标只记录可迁移的治理含义，不复制 Python 或 Node 工具链。

[English](mixed-monorepo-fixture-adaptation.md) · [简体中文](mixed-monorepo-fixture-adaptation.zh-CN.md) · [日本語](mixed-monorepo-fixture-adaptation.ja.md)

## 逐文件映射

| 固定源文件 | 源文件事实 | Rust 原生对应与边界 |
| --- | --- | --- |
| `fixture.json` | 声明混合 Python/Node 示例、通用安装元数据、三个平台以及 safe/test 路径。 | Project Observer/Profile 可以记录对象工程实际观察到的事实；Runtime 不从 fixture 元数据推断工具链能力或安全范围。 |
| `package.json` | 没有依赖和脚本的私有 Node 包元数据。 | 只是业务示例输入。Node 安装、依赖、脚本和执行由对象工程/provider 负责。 |
| `pyproject.toml` | 最小 Python 项目元数据。 | 不是可移植 Contract 或 Runtime 依赖。Python 安装、依赖和测试命令需要对象工程明确提供证据。 |
| `services/api/app.py` | 返回 `ok` 的健康函数。 | 是业务代码，不是治理逻辑。Runtime 可以绑定对象工程声明的 argv 结果，但不会携带或推断 Python 行为。 |
| `services/api/tests/test_app.py` | 用 pytest 检查健康函数的断言。 | 仅是 fixture 证据。对象工程必须声明并运行自己的验证命令；源测试不会被提升为目标证据。 |

## 安装与对象工程边界

fixture 不定义 AI Cockpit 的安装方式。应在对象工程外安装一份共享 Runtime，
再显式 attach：

```bash
repo=/path/to/mixed-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

对象工程负责 Python/Node 解释器、依赖锁、测试命令和 hosted provider 证据。
后续 Runtime 命令都带同一个显式 `--repo`；Contract、snapshot、evidence、
knowledge 和 Agent adapter 仍保存在各自仓库中。

## 对象工程继承的能力

attach 后的混合仓库继承共享 Runtime 的 Contract 校验、fail-closed unknown、
身份绑定证据、生命周期、仓库隔离和面向人的 Outcome 规则。但不会继承 fixture
的包元数据、源码、测试运行器、安装行为，也不会自动宣称任一工具链可用。
这属于语义/文档 parity，不是 mixed-stack 工具链支持、源命令兼容或第二技术栈
adopter 验收。

[参考索引](README.zh-CN.md) · [参考文件比对](reference-file-comparison.zh-CN.md)
