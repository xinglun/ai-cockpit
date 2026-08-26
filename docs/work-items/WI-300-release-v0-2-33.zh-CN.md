---
author: AI Cockpit maintainers
title: "WI-300——v0.2.33 发布与安装验收"
workItemId: WI-300-release-v0-2-33
description: "发布已修正的 Runtime，验证不可变制品，并用公开 binary 完成仓库与 adopter 验收。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-300-release-v0-2-33
authority: canonical
---

# WI-300——v0.2.33 发布准备

## 意图

在 WI-299 修正 adopter finalization base binding 后，从已审阅的默认分支准备
v0.2.33。公开制品的校验、安装和 adopter 验收由发布后的必需 successor
WI-301 完成。

## 范围

- 将 workspace 包版本和发布示例统一到 v0.2.33。
- 明确保留失败的 staged v0.2.32 历史，不改写它。
- 发布前运行源码、文档、策略和完整 workspace 验证。
- 仅通过 hosted release workflow 发布，并绑定 manifest、校验和、SBOM、
  provenance 与 artifact smoke 证据。
- 配置 hosted workflow 与 handoff，明确发布后安装和 adopter 验收的 successor。

## 不在范围内

不改写 v0.2.32 历史、不新增 Runtime 治理行为、不发布外部 Homebrew tap、不
执行发布后安装或 adopter 验收、不扩展 adopter 技术栈矩阵，也不修改全局
Agent/MCP 配置。

## 验收标准

1. 所有 workspace 包和 Cargo.lock 都解析为 0.2.33，三语文档使用同一当前基线。
2. 失败的 staged v0.2.32 发布明确保持历史状态，不宣称存在公开 Release。
3. 打标签前通过版本一致性、文档、治理完整性、发布策略和完整 workspace 测试。
4. Hosted workflow 发布绑定标签提交的 manifest、SHA256SUMS、各 target SBOM、
   provenance 和 artifact smoke 证据。
5. Reviewed release workflow 只在发布前门通过后发布，并将公开制品检查交给
   WI-301。
6. WI-300 不宣称公开制品安装或 adopter 验收；这些结论必须由 WI-301 的公开
   Release 证据支持。

## 验证

- `cargo test --locked --workspace`
- `bash tests/docs/documentation_acceptance.sh --repo <repo>`
- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/release/release_policy_test.sh`
- adopter 与 N-1 acceptance 静态测试
- Hosted release quality、Windows runtime 与 behavioral-oracle 检查
- WI-301 的发布后公开 manifest、checksum、安装、repository 与 adopter 验收 receipt
  （不属于本 WI 证据）

## 历史边界

v0.2.32 tag 记录了 finalization base-revision 缺陷导致的 staged 发布失败。其
失败事实保持不变；本 Work Item 准备新的 v0.2.33 发布，不通过重写历史来修复它。
WI-301 负责所有发布后公开制品与 adopter 结论。
