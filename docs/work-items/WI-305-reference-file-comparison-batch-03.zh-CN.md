---
author: AI Cockpit maintainers
title: "WI-305——参考架构安装与验证批次 03"
workItemId: WI-305-reference-file-comparison-batch-03
description: "逐个比对固定参考源的四个架构文件，记录 Rust/adopter 边界，不复制源 installer 或 Wizard。"
audience:
  - maintainer
  - reviewer
status: in progress
lastVerifiedBy: WI-305-reference-file-comparison-batch-03
authority: canonical
---

# WI-305——参考架构安装与验证批次 03

## 意图与目标

逐个比较接下来的四个 deferred 参考架构文档，确认 Rust Runtime 与对象工程是否继承了安装侦测、
交互式向导边界、轻量验证/软门以及 Wizard 输入/本地化责任。每个文件都记录对应物或明确的
reference-only/external 边界；不复制源 Python、Make、Installer 或 Wizard 实现。

## 范围与边界

范围内：

- `docs/architecture/installation-detection-boundary.md`
- `docs/architecture/interactive-installation-wizard.md`
- `docs/architecture/lightweight-verification-and-soft-gates.md`
- `docs/architecture/wizard-io-and-localization.md`
- `tests/conformance/reference_file_inventory.py`
- `tests/conformance/reference_file_inventory.json`
- `tests/conformance/reference_file_inventory_test.sh`
- 三语参考比较页、安装路线更新和本 Work Item 的三语投影。

范围外：

- 复制 `scripts/**`、源 Python、Make target、`install_ai_cockpit.py`、locale 或交互式 Wizard；
- 新增交互式 Installer Wizard 或 Runtime 命令；
- 修改 Rust Runtime 语义、发布版本、Homebrew 或 adopter 验收；
- 全局 Agent/MCP 配置、第二技术栈 adopter 和不可变历史证据。

## 固定来源与观察到的边界

参考源为 `spirex-ds-dev/ai-cockpit-template` 的
`e5acb677da6621004d96f0ef353c58fe8d3acfbf`。台账 Rust 比对基线仍为
`a533d49dfa848d95742833f8cd1b5f7e1bb897d5`；本 Work Item 自身从最新远端 `main` 开始。

使用的安装版 Runtime 为 `ai-cockpit 0.2.33`，binary SHA256 为
`sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`。所有 Runtime 命令
都显式带 `--repo`。

源 detector 与 Wizard 是 repository-local 的 Python presentation/transaction adapter。目标 Rust
改为安装一份不可变共享 Runtime，并通过显式的 `inspect`、`attach`、profile proposal/confirm、
`doctor` 完成 onboarding。因此源 Wizard 在目标中是 reference-only；不会用 parity 声明隐藏缺失的
Runtime 功能。

文件级阅读还覆盖了各页面引用的源证据：`scripts/ai_installer_detection.py`、
`scripts/ai_install_wizard.py`、`scripts/ai_install_plan.py`、`scripts/ai_installer_evidence.py`、
`scripts/ai_wizard_io.py`、`scripts/ai_wizard_localization.py`、`scripts/install_ai_cockpit.py`、
calibration-wizard adapter，以及对应的 installer、Wizard IO/localization、quality 和 calibration
测试模块。这些源路径仍只作为 corpus；目标证据是下表列出的 Rust code/tests 和面向读者的路线。

## 文件级比对结论

| 参考文件 | 结果 | 目标证据 / 边界 |
| --- | --- | --- |
| `installation-detection-boundary.md` | implemented-different-by-design | `inspect`、`status`、`doctor`、`attach`、`profile propose`、校准文档及测试分担只读事实与显式写入边界。Release 安装属于独立不可变制品边界。 |
| `interactive-installation-wizard.md` | reference-only | 十阶段 dry-run/确认 UI 是源 Installer 的外层，不由 Rust Runtime 提供。目标的显式命令路线和 provider-owned 对话 UI 防止提示直接变成批准。 |
| `lightweight-verification-and-soft-gates.md` | implemented-different-by-design | typed stage、policy 驱动 tier、hard/soft/informational 决定、skipped/unknown 原因、动态 light/standard/strict 路由、request-scoped context 与 advisory cost/reuse 由 Rust verification、CI、cost 文档/测试覆盖。 |
| `wizard-io-and-localization.md` | implemented-different-by-design | CLI/MCP presentation 支持 en/zh-CN/ja 的 Runtime 生成文本，并保留 Contract 值原文。Wizard 专用 TTY 控件不属于 Runtime；对话控制由 adapter 负责。 |

## 验收标准

1. 读取四个固定文件，记录具体责任、边界以及源模块/测试依据。
2. 每个文件都有证据支持的对应物或明确的 reference-only/external 边界；不得把不存在的交互向导称为等价实现。
3. 三语安装文档明确共享外部 Runtime、显式 `--repo`、attach/校准路线及有意不提供 Wizard 的边界。
4. Rust 文档保留源软门的安全边界：按阶段的 fail-closed 决定、显式 skipped/unknown、动态 light/standard/strict 与 advisory cost telemetry；明确源 `hard`/`soft`/`informational` 标签不会复制成目标 wire enum。
5. 台账恰好四条记录移入 WI-305，reason 与 counterpart 非空，且没有 WI-305 的 `migrate-gap` 或 deferred 记录。
6. 使用安装版 Runtime 生命周期运行 inventory 回归、文档检查、治理门禁和 `cargo test --locked --workspace`。
7. 完成 reviewed PR 合并、合并后 finalize、精确 branch/worktree 清理和可见 human Outcome；对象工程边界仍为共享 Runtime + 隔离 repository 状态。

## 明确不宣称

本 Work Item 不宣称 source JSON/wire 兼容、通用语言翻译、Rust 交互式 installer、provider identity、
hosted CI 证明或生产就绪。Localize 只改变 presentation chrome；Contract intent、acceptance criteria、
命令、路径和 machine evidence 保持编写时的值。
