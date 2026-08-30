---
author: AI Cockpit maintainers
title: “Python fixture 适配边界”
description: “逐文件记录固定 Python fixture 的 Rust-native 映射，不复制其应用、打包或测试实现。”
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-414-reference-python-fixture-boundary
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# Python fixture 适配边界

本页逐一比较固定参考 fixture `examples/fixtures/python/` 中的四个文件。
它记录对 Python 对象工程有用的语义，但不会把 fixture、打包元数据或测试
运行器复制进 Rust Runtime。

[English](python-fixture-adaptation.md) · [简体中文](python-fixture-adaptation.zh-CN.md) · [日本語](python-fixture-adaptation.ja.md)

## 逐文件映射

| 固定源文件 | 源文件事实 | Rust-native 对应与边界 |
| --- | --- | --- |
| `fixture.json` | 声明 Python service、`python3` toolchain、Linux/macOS 平台以及安全/测试路径。 | Project Observer/Profile 可以把它们记录为仓库本地事实或候选事实。共享 Runtime 不会从本文件推断 Python 能力、平台就绪度或安全范围；精确 Contract 仍需负责人确认。 |
| `pyproject.toml` | 声明包元数据（`requires-python >=3.11`）以及 pytest 的 `tests` 路径。 | Python 打包和 pytest 属于对象工程/Provider 责任。负责人提供显式命令（例如 `python -m pytest`）；Runtime 记录 argv 与结果，但不安装 Python 或复制此清单。 |
| `src/service.py` | 最小应用函数返回健康值 `ok`。 | 这是 fixture 应用代码，不是治理逻辑。Rust verification 可以执行对象工程声明的命令并绑定证据，但目标不会携带或从本源码推断 Python 语义。 |
| `tests/test_service.py` | pytest 测试断言健康函数结果。 | 这是样例断言，不是可移植 Runtime 测试 Contract 或企业证据。对象工程必须自行声明并执行测试命令；源 fixture 测试永远不会被提升为目标证据。 |

## 安装与对象工程边界

参考 fixture 的 stack 元数据不是 AI Cockpit 安装方案。先在对象工程外安装
一份共享 Runtime，再显式 attach：

```bash
repo=/path/to/python-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

Python 解释器、虚拟环境、依赖锁、pytest 配置和 CI/Provider 证据由对象工程
负责。之后每次 Runtime 命令都带同一个显式 `--repo`；Contract scope、profile、
snapshot、evidence、knowledge 与 Agent adapter 均保持仓库本地隔离。

## 对象工程继承什么

已 attach 的 Python 工程继承共享 Runtime 的 Contract 校验、未知项 fail-closed、
身份绑定证据、生命周期和面向人的 Outcome 规则。它不会继承参考 fixture 的
`pyproject.toml`、Python 源码或 pytest 安装，也不能据此声称测试已运行。除非
相应外部权威提供证据，本地测试结果不等同于 Provider、托管 CI、Release 或企业证据。

这是语义/文档对齐，不是 Python toolchain 支持、源命令兼容或 JSON-wire 兼容。
真实 Python adopter 验收仍需单独授权的发布后测试。

[参考索引](README.zh-CN.md) · [参考源逐文件比较](reference-file-comparison.zh-CN.md)
