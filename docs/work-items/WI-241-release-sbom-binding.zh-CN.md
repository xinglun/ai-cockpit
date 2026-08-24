---
author: AI Cockpit maintainers
title: "WI-241——Release SBOM 制品绑定"
workItemId: WI-241-release-sbom-binding
description: "把后续每个 target SBOM 绑定到准确的打包 bytes，并封闭公开 Release 资产集合。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
lastVerifiedBy: WI-241-release-sbom-binding
authority: canonical
---

# WI-241——Release SBOM 制品绑定

WI-241 修复 v0.2.31 企业合规审计中发现的 Release 构建边界。它只影响后续
candidate；公开 v0.2.31 tag、Release assets、checksums、attestations 与 acceptance
receipts 都保持不可变的历史事实。

## 已交付边界

- `cockpit-release bind-sbom` 计算实际 staged archive 及从该 archive 读取的 executable
  member 的 SHA-256。它插入标准 SPDX 2.3 release Package 与 File，用 `DESCRIBES` 和
  `CONTAINS` 连接，并在写入前校验完整文档。
- validator 要求准确 target、规范 version、按 target 命名的 archive/SBOM、一个保留
  Package、一个保留 File、各一个绑定关系，以及匹配且非零的 SHA-256。
- 保留 Anchore dependency scan，但关闭其自动 artifact 与 Release upload。只有五个
  按 target 命名的 SBOM 能进入 candidate、attestation 与 publication allowlist。
- Formula 生成后才生成 checksums。它按稳定顺序准确一次覆盖五个 archive、五个 SBOM、
  canonical manifest 与 Formula。checksum 文件自身是第十三个公开资产，不能校验自身。
- candidate validation 会在下游 staged adopter acceptance 之前拒绝 missing/orphan
  publishable asset、重复 checksum 名、未排序或格式错误的行、缺失/额外 entry 以及 digest mismatch。

## 证据边界

SPDX 文件名或 dependency scan 不等于 adopter acceptance。SBOM 只证明其准确 archive/binary
绑定。Hosted attestation 与现有 staged/public adopter acceptance jobs 仍是独立的下游 gates。

回归覆盖位于 `crates/cockpit-release/tests/sbom.rs`、
`crates/cockpit-release/tests/manifest.rs` 与 `tests/release/workflow_policy.sh`。
Runtime verification 记录在
`.ai/evidence/WI-241-release-sbom-binding.verification.json`。

## 参考

- [发布与分发](../release/distribution.zh-CN.md)
- [参考源对等矩阵](../reference/reference-parity.zh-CN.md)
