---
author: AI Cockpit maintainers
title: "WI-516——发布、采用、校准与证据比对批次 34"
description: "逐个比较下一批参考源表面，不复制 Python、打包或 provider 字节。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-516-reference-file-comparison-batch-34
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-516-reference-file-comparison-batch-34
terminalArchive: .ai/work-items/archive/WI-516-reference-file-comparison-batch-34.contract.json
terminalVerification: .ai/evidence/WI-516-reference-file-comparison-batch-34.verification.json
terminalFinalization: .ai/decisions/WI-516-reference-file-comparison-batch-34.finalize.json
terminalDecision: .ai/decisions/WI-516-reference-file-comparison-batch-34.close.json
---

[English](WI-516-reference-file-comparison-batch-34.md) · [日本語](WI-516-reference-file-comparison-batch-34.ja.md)

## 目标

逐个阅读固定的本地参考源路径，为每个路径记录有证据的 Rust 对应或明确的非声明。本批次覆盖发布投影、Python 开发元数据、adopter evidence、归档行为、baseline/成本观测、校准、能力事实和 canonical evidence。不复制源 Python、Shell、Make、打包、provider 状态、交互向导或 JSON wire 格式。

## 逐文件决定

| 固定源路径与 digest | 分类 | Rust 对应/非声明 |
| --- | --- | --- |
| `next-release.json`——`sha256:b5189750265e8b09350c153b47a9ffbff629042fe035a7dfe143b5e15c8949c2` | implemented-different-by-design | `crates/cockpit-release`、release workflow、版本检查和发布文档绑定不可变 artifact、checksum、SBOM/provenance 与 adopter 验收；源 candidate 字段不是 Runtime wire。 |
| `pyproject.toml`——`sha256:4d5ad0892ea3ee4bafc744c59a64dda3111d24ca6238873cf1107d537693c9c2` | implemented-different-by-design | Cargo 元数据、lockfile 和动态 CI gate manifest 替代 Python Ruff/mypy/coverage/pytest 配置；Python 工具设置仍是 source/provider 事实。 |
| `release-state.json`——`sha256:c747a4a6cb48190e55765eb76675f271af389c8db92b03efc720395844132f4c` | implemented-different-by-design | Rust release manifest/evidence 保留不可变 tag、artifact、供应链与发布后状态；源投影 bookkeeping 不复制。 |
| `release.json`——`sha256:1e8ce44257efb4b8267bc30e6866a2ac085afad49bc621011599c1f2900615f8` | implemented-different-by-design | 目标 release manifest 与 `SHA256SUMS` 绑定本仓库 artifact；源 URL、schema 和历史发布声明不可移植。 |
| `requirements-dev.in`——`sha256:296d516b6548e2fa541e6eec23223a160bda0ea887d2ffccec8f50cfe550449c` | implemented-different-by-design | `Cargo.toml`、`Cargo.lock` 与 CI 声明 Rust 工具链；adopter 自己负责其语言工具链。 |
| `requirements-dev.lock`——`sha256:b07fca668d49671422fb8213908d475b3698dd375ca3cfb03346d5ad51483537` | implemented-different-by-design | Cargo lock 与 Rust archive/supply-chain tests 提供目标可复现边界；Python package hash 不是 Runtime evidence。 |
| `scripts/ai_adoption_evidence.py`——`sha256:87c883e556132cb759c792c4c106d112e2a0917222063f8a797658666d52e161` | implemented-different-by-design | 公开 Release adopter 验收绑定下载 artifact、repository identity、隔离 manifest 与生命周期证据；源 Work Item id 和 JSON wire 不复制。 |
| `scripts/ai_adoption_reality_report.py`——当前 pinned checkout 已退休 | reference-only（历史） | 仅按 inventory 的 retired ledger 核对，不当作当前源文件；不声明 Rust 继承源历史 Python report/evidence。 |
| `scripts/ai_archive_work_item.py`——`sha256:ceef1b14e6760a38b6873eeb971f6b20165fa831016e83393bdc52d8d7ec9324` | implemented-different-by-design | Rust archive/manifest/recovery/close 服务与 archive-integrity 测试保留不可变历史和精确清理，不复制 Python 路径重写 helper。 |
| `scripts/ai_baseline_evidence.py`——`sha256:ba47fbec6d2a9dbb66d43230dac5b25dbedbd9861726401a413828a69a4974a0` | implemented-different-by-design | Rust performance baseline、snapshot-bound verification 与成本观测保留身份和可复现性；源 Python coverage 字段仍由项目负责。 |
| `scripts/ai_calibrate.py`——`sha256:99a126a836b518c49d76349c286fc491fe1556652c36b1d22c676daf4b4af965` | implemented-different-by-design | typed project governance、`profile propose/confirm` 与校准文档保留 owner review、unknown 和 snapshot 绑定；不复制源十阶段 Python session。 |
| `scripts/ai_calibration_corrective.py`——`sha256:6839e84e5309d32ad06b3e851a89eab5ddf1134bea2bf84f5c6692a65bf71635` | implemented-different-by-design | Rust profile/amendment 校验和 project-governance tests 提供 repository-bound corrective 边界；不导入源 session 路径。 |
| `scripts/ai_calibration_inventory.py`——`sha256:d0fff777e86e1746b393952c1f5ce96fb8cbe5b2570ca778d8b9fc56e6a50d164` | implemented-different-by-design | typed capability truth、profile facts、evidence assurance 和 external exclusions 替代源 inventory aggregation；源 status key 不是通用协议。 |
| `scripts/ai_calibration_profiles.py`——`sha256:8c6be65cca8ee0340a113dcfb4120b395b8421d26dfcd4275d6fcdb21e21f8e7` | implemented-different-by-design | Rust 比例化 project policy 与显式 profile confirm 保留 lite/standard/strict 意图，不复制源 YAML/选择字节。 |
| `scripts/ai_calibration_wizard.py`——`sha256:63aa3f26f0cdd98c00ad88ffb1ec16e890f29dd18cbe16a360017ec00178d005` | implemented-different-by-design | CLI 与 reader-first calibration guide 提供可审查 propose/confirm 展示；不提供第二套 provider interactive wizard。 |
| `scripts/ai_canonical_evidence.py`——`sha256:421c6ab34cc80ce1ac6f4b19cd4304a0491a9c38322c0aef8131ea13465dae28` | implemented-different-by-design | typed evidence、audit-export、digest、receipt 与 archive schema 保留确定性身份/状态；源 canonical JSON/Markdown wire 不复制。 |
| `scripts/ai_capability_freshness.py`——`sha256:e6471b84dcab07396a4a24f3454b41ff55632e762ad6b3cfd41d41c26103a397` | implemented-different-by-design | capability projection 绑定当前 repository snapshot 与 Runtime identity；toolchain/provider freshness 必须由显式 repository evidence 提供。 |
| `scripts/ai_capability_truth.py`——`sha256:5cda977775e5b4fa6531886f963f1c8a4a976344ed974e34bcf39b58b1a3500e` | implemented-different-by-design | typed `CapabilityTruth`/`AdopterCapabilityTruth` 通过 CLI/tests 暴露 confidence、evidence refs、unknowns 与 exclusions；源 matrix row/Python validator 不复制。 |

## 对象/采用方工程继承边界

对象工程继承共享 Runtime 的 repository-bound attach、profile、Contract、evidence、knowledge、capability、release acceptance 与 human Outcome 边界；不继承 Python 依赖、源 release projection、源 calibration session、provider credentials 或源 JSON wire。每个 adopter 必须提供自己的项目事实和显式 verification evidence。

## 验收标准

- 本批每个 current path 都有源 digest、分类、对应或明确非声明及 inventory evidence；退休 adoption-report 路径明确记录为历史/非当前。
- inventory 将 17 个 current decision 归属于 `WI-516-reference-file-comparison-batch-34`，不保留 deferred 或 `migrate-gap`。
- 不修改源字节、Python 打包行为、provider 状态或对象工程。
- 英文、简体中文、日文 comparison/parity 文档表达相同的 semantic/non-wire 与 adopter inheritance 边界。
- Contract 声明的 conformance、文档、parity 和 workspace checks 全部通过。

## 验证

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

参考 checkout 固定为 `fde3380f81fea5fd2e288f7a8849f737dc074060`，不通过网络读取，也不把源实现加入本仓库。
