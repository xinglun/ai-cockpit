---
author: AI Cockpit maintainers
title: "WI-455——v0.2.52 annotated tag 发布恢复"
workItemId: WI-455-release-v0-2-52-annotated-tag
description: "仅通过已审查的 annotated tag 和不可变公开制品发布下一 patch。"
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-455-release-v0-2-52-annotated-tag
terminalArchive: .ai/work-items/archive/WI-455-release-v0-2-52-annotated-tag.contract.json
terminalVerification: .ai/evidence/WI-455-release-v0-2-52-annotated-tag.verification.json
terminalFinalization: .ai/decisions/WI-455-release-v0-2-52-annotated-tag.finalize.json
terminalDecision: .ai/decisions/WI-455-release-v0-2-52-annotated-tag.close.json
---

# WI-455——v0.2.52 annotated tag 发布恢复

本 Work Item 处理不可变的 v0.2.51 lightweight-tag 发布失败后的下一 patch 发布。
保留失败历史，增加 annotated tag 的可重复检查，并让 provider Release 只由已审查的
workflow 创建。本 Work Item 不操作对象工程。

[English](WI-455-release-v0-2-52-annotated-tag.md) · [日本語](WI-455-release-v0-2-52-annotated-tag.ja.md)

## 来源

- `docs/release/distribution.*.md`
- `docs/architecture/release-distribution.*.md`
- `.github/workflows/release.yml`
- `tests/release/annotated_tag_identity.sh`
- 失败的 v0.2.51 workflow run `33417057474`

## 验收

- workspace metadata 与三语发布文档标识 v0.2.52，且不改写 v0.2.51 历史。
- lightweight tag 被拒绝；annotated tag 被 peel 并绑定到已审查提交。
- 明确要求推送 annotated tag，禁止预先创建 provider Release。
- strict 发布门禁、公开制品 checksum/SBOM/provenance 与 staged/public adopter acceptance 在无源码 fallback 下通过。
- 发布二进制在安装前完成 checksum 校验，安装后的当前仓库 Runtime 检查保持健康。

## 验证

- `tests/release/annotated_tag_identity.sh`
- `tests/release/version_consistency_test.sh`
- `tests/release/workflow_policy.sh .github/workflows/release.yml`
- strict `quality_route.py` + `run_repository_gates.py`
- `cargo test --locked --workspace`
