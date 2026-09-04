---
author: AI Cockpit maintainers
title: "WI-572：安装器与质量参考源比对批次 45"
description: "逐个比较 20 个维护中参考路径并记录有界的 Rust 语义决定。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-572-reference-installer-quality-batch-45
lastVerifiedBy: WI-572-reference-installer-quality-batch-45
terminalArchive: .ai/work-items/archive/WI-572-reference-installer-quality-batch-45.contract.json
terminalVerification: .ai/evidence/WI-572-reference-installer-quality-batch-45.verification.json
terminalFinalization: .ai/decisions/WI-572-reference-installer-quality-batch-45.finalize.ce778cafe4377bd38aad5238a5fd182cee9611e7017c91e83f40f0a1116cda6f.json
terminalDecision: .ai/decisions/WI-572-reference-installer-quality-batch-45.close.json
---

[English](WI-572-reference-installer-quality-batch-45.md) · [日本語](WI-572-reference-installer-quality-batch-45.ja.md)

# WI-572：安装器与质量参考源比对批次 45

## 目标

在固定本地参考 checkout 提交 `fde3380f81fea5fd2e288f7a8849f737dc074060`
上逐个重读下一组 20 个维护中路径，记录 Rust 对应或有界的
source/provider-only 决定。这是语义比较，不是复制实现或 JSON wire 迁移。

## 比较结果

完整逐路径台账位于 `tests/conformance/reference_file_inventory.json` 及三语
比对页面。其中 19 项为 `implemented-different-by-design`：安装器的
`git_state`、`inspection`、`legacy`、`ownership`、`planning`、`presentation`、
`rollback`、`transaction`、`upgrade`，质量的 `measurements`、`session_lock`、
`test_manifest`，以及 release/archive、quality gate/session、summary、发布
投影同步、unsupported claim、quick-install 校验脚本。

`scripts/real_adopter_reference_validation.py` 为 `reference-only`：其七项目
矩阵是参考模板专属，不是可移植的 Rust Runtime 合约。

## 边界与对象工程继承

目标能力由 shared Rust Runtime、显式 `--repo`、typed Agent/release/
verification 服务、动态质量路由、隔离 Contract/evidence/knowledge 和
human Outcome handoff 承载。不复制源 Python、Make/provider 编排、source
wire 或模板专属技术栈矩阵。attach 的对象/adopter 工程继承同一 Runtime
能力与边界，而不是源实现。

本批同时修正生命周期恢复：人类授权的 Contract amendment 使旧验证回执失效时，
预检将其识别为 stale（而非篡改），允许新验证替换；格式错误或外部身份的证据仍
保持 contradictory 并 fail-closed。新验证替换完成后只消费活动 retry 标记投影，
append-only recovery receipt 仍作为历史证据保留。

## 验证

- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report /tmp/ai-cockpit-governance-integrity.json`
- `git diff --check`
