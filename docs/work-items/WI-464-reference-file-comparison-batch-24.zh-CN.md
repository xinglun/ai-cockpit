---
author: AI Cockpit maintainers
title: "WI-464——工作流与构建重新基线"
workItemId: WI-464-reference-file-comparison-batch-24
description: "逐个比对四个发生源变更的路径，并记录 Rust 原生 CI 与发布边界。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-464-reference-file-comparison-batch-24
---

# WI-464——工作流与构建重新基线

本 Work Item 重新阅读此前工作流比对后发生变化的四个路径。参考源是本地
固定提交 `fde3380f81fea5fd2e288f7a8849f737dc074060`；它是规格语料，不是要复制的实现。

| 固定源路径 | 分类 | Rust 原生决定 |
| --- | --- | --- |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | 源 ShellCheck 安装和 Python/多技术栈矩阵仍属于源/provider 边界。Rust 保留 action pin 策略、动态质量路由、Rust workspace/平台检查和公开 adopter 验收。 |
| `.github/workflows/release.yml` | implemented-different-by-design | 源 `release-digests.json` 归档投影及删除旧 `release.json` 双资产检查，对应 Rust release manifest/`SHA256SUMS`、SBOM/provenance、平台 smoke 和 adopter 证据；不复制源投影 bytes。 |
| `.github/workflows/smoke.yml` | implemented-different-by-design | 源文件移除了 `REPORT_LANGUAGE` Make 参数。Rust 没有源 `smoke.yml`；CI、release、gate manifest 和不可变 adopter harness 通过显式仓库上下文承担有界检查。 |
| `Makefile` | implemented-different-by-design | 源 Python/Make 分片、knowledge 和语言辅助逻辑仅属于源工程。Rust 使用 Cargo、CLI、规范 gate manifest 和显式 `--repo`，不需要第二套 Make 治理层。 |

本次重新基线没有发现 Rust 实现遗漏。目标工程的 action pin 继续由自身审核过的
action-runtime policy 管理；不会把源矩阵的 pin 静默替换到 Rust CI/release 路径。

机器清单将四个路径都记录到本 Work Item，并保留
`sourceChangedSincePrevious` 溯源，同时移除 deferred 分类。本次是语义/文档对等，
不是源文件、Python/Make、provider 或 JSON wire 兼容。对象/采用方工程继承 Rust
共享 Runtime 和仓库本地证据边界，而不是这些源工作流文件。

## 验证

- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- 本 Work Item 声明的文档和 repository gate 检查
