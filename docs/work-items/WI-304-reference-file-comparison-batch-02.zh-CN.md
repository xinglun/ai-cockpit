---
author: AI Cockpit maintainers
title: "WI-304——参考 workflow 比对批次 02"
workItemId: WI-304-reference-file-comparison-batch-02
description: "逐文件比对固定参考源的下两个 workflow，记录 Rust-native 与外部/adopter 边界，不复制源工具链。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-304-reference-file-comparison-batch-02
authority: canonical
---

# WI-304——参考 workflow 比对批次 02

## 意图与目标

逐个比较下两个 deferred 参考文件：`.github/workflows/compatibility.yml` 与
`.github/workflows/smoke.yml`。记录每个 trigger、矩阵、依赖、artifact、
release/measurement 条件和 installer 责任，并映射到 Rust-native 对应物或明确的
外部/adopter 边界。本 Work Item 不复制 Python、Make、installer 或 workflow 字节。

## 范围与边界

范围内：参考源 inventory generator 与回归台账、中文/英文/日文比较页，以及本 Work
Item 的三语投影。可以执行现有 inventory、文档和 workspace 检查，但不改变 Runtime
语义。

范围外：复制参考源 Python module、Make target、`install.sh` 或多技术栈 fixture；实现完整
多语言/mobile compatibility matrix 或第二技术栈 adopter；修改 `crates/**`、Runtime
语义、全局 Agent/MCP 配置、发布版本/发布过程或不可变历史证据。

## 固定来源与比对事实

- 参考源：`spirex-ds-dev/ai-cockpit-template`，提交
  `e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
- Rust 比对台账基线：目标提交
  `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`。
- 使用的安装版 Runtime：`ai-cockpit 0.2.33`，binary SHA256 为
  `sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`。
- 参考 workflow：compatibility 的八类责任（ShellCheck、Python 平台、lockfile 可复现性、
  real/extended/mobile 矩阵、latest probe 和两个汇总 gate），以及 smoke 的 project shard、
  installation/release/measurement 路径、artifact 和最终 CI receipt。
- 目标边界：`ci.yml`、`release.yml`、规范 gate manifest 与不可变公开/N-1 adopter harness
  提供 Rust 产品/发布证据；adopter 工具链和源特定 installer/多技术栈测试保持外部或由
  adopter 负责。

## 验收标准

1. 完整比对两个固定 workflow，包含 trigger、permission、concurrency、所有 job/matrix、
   `needs`、input、artifact 路径、阻断条件、release/measurement 分支和 installer 命令。
2. 每项源责任都有 Rust 对应物或明确 external/adopter/deferred 边界，不做静默 parity 声明。
3. 台账只将这两个记录从 WI-302 deferred 集合移到 WI-304，每条都有非空 reason 和 counterpart，
   且没有未分类记录。
4. 三语比较页与 Work Item 投影说明相同的 semantic/non-wire 边界；没有目标等价物的
   Python/Make/installer 责任保留为外部边界。
5. 现有动态 `light`/`standard`/`strict` 路由、显式 `--repo`、共享 Runtime 和隔离 adopter
   evidence 不变。
6. inventory、文档、治理和 workspace 检查通过，并使用安装版 Runtime 完成生命周期和审阅 PR。

## 已知边界

源 ShellCheck job 检查的是源专属 `install.sh`；目标没有 installer，目前对 Shell 脚本做语法
校验。是否增加目标 ShellCheck policy 是独立的 CI hygiene 决策，不因此复制源 installer 或宣称
源矩阵在 Runtime 内运行。
