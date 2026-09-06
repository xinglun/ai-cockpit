---
author: AI Cockpit maintainers
title: "WI-621——参考安装与生命周期安全对等"
description: "逐一比较下一批 18 个维护中的参考测试路径，不复制源实现。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-621-reference-installer-lifecycle-batch
status: implemented
authority: canonical
lastVerifiedBy: WI-621-reference-installer-lifecycle-batch
terminalArchive: .ai/work-items/archive/WI-621-reference-installer-lifecycle-batch.contract.json
terminalVerification: .ai/evidence/WI-621-reference-installer-lifecycle-batch.verification.json
terminalFinalization: .ai/decisions/WI-621-reference-installer-lifecycle-batch.finalize.json
terminalDecision: .ai/decisions/WI-621-reference-installer-lifecycle-batch.close.json
---

# WI-621——参考安装与生命周期安全对等

## 意图

逐个阅读下一批 18 个当前参考测试文件，为每个文件记录有证据的 Rust 对应能力或精确的源专属边界。目标是 shared Runtime 与对象工程的语义对等，不是复制 Python、Shell、向导、fixture 或源 JSON 实现。

## 边界与决定

固定本地参考源提交为
`fde3380f81fea5fd2e288f7a8849f737dc074060`；Rust 比较基线为
`8adac3379d8cb3e7a3dc59c70d6fb0b26176b990`，使用已发布的 `ai-cockpit`
`0.2.83` binary（`sha256:9f44d14278a614636ca47ee660656ce3b5eb5a0b969059b46d3132947310d130`）。选定路径在现有追加式台账中是当前但 deferred 的记录。17 项责任由 Rust Runtime、仓库原生测试、release/adopter harness 或文档以不同设计承载；交互式源向导保持 `reference-only`。未发现 `migrate-gap`，退休参考历史保持不可变。

| 固定参考路径 | 分类 | Rust 对应或有界决定 |
| --- | --- | --- |
| `tests/test_install_entrypoint.py` | implemented-different-by-design | 显式 Release 安装、`attach --repo`、inspect/doctor 与非交互 fail-closed 测试。 |
| `tests/test_install_facts.py` | implemented-different-by-design | 类型化 Release manifest/archive/SBOM 身份与规范化事实测试。 |
| `tests/test_install_script.py` | implemented-different-by-design | 发布 archive、SHA-256、source-archive policy 与 distribution 检查。 |
| `tests/test_install_sh.py` | implemented-different-by-design | 安装文档、release CLI 测试与 workflow policy；不复制源 quick-install 脚本。 |
| `tests/test_install_status.py` | implemented-different-by-design | Release manifest 身份，以及 `doctor` 与 installed-lifecycle 投影。 |
| `tests/test_install_wizard.py` | reference-only | 源交互式 provider/技术栈向导；Rust 明确采用产物安装和 repository binding。 |
| `tests/test_installed_lifecycle_e2e.py` | implemented-different-by-design | Public/N-1 adopter acceptance 与类型化生命周期证据。 |
| `tests/test_installer_boundaries.sh` | implemented-different-by-design | Agent/repository ownership 与显式 context 隔离测试。 |
| `tests/test_installer_conflict_matrix.py` | implemented-different-by-design | Attach/Agent ownership、安全路径、symlink、traversal 与 trust boundary 检查。 |
| `tests/test_installer_detection.py` | implemented-different-by-design | 显式 compatibility、migration proposal、repository identity 与 active Work Item 投影。 |
| `tests/test_installer_domains.py` | implemented-different-by-design | 只读 inspect/doctor/plan 与显式 attach/migrate 写入边界。 |
| `tests/test_installer_evidence.py` | implemented-different-by-design | 类型化 release handoff、manifest 与 delegated evidence/assurance 记录。 |
| `tests/test_installer_repository.py` | implemented-different-by-design | Git/Observer snapshot 与 request-scoped identity 的仓库事实。 |
| `tests/test_installer_transaction.py` | implemented-different-by-design | 不可变 archive 校验、原子 repository 写入/锁、migration receipt 与 Agent 隔离。 |
| `tests/test_issue_log.py` | implemented-different-by-design | 不可变 Work Item evidence、结构化 decision、unknown 与 recovery lineage 替代源 issue-log 数据库。 |
| `tests/test_lifecycle_facts.py` | implemented-different-by-design | 确定性的 request-scoped status/observe/doctor 投影。 |
| `tests/test_lifecycle_safety_gate.py` | implemented-different-by-design | 类型化 governance controls、preflight review 与操作时 fail-closed policy。 |
| `tests/test_negative_scenarios.py` | implemented-different-by-design | Rust adversarial 与 input-trust 套件保留不安全拒绝和安全替代路径边界。 |

源路径不会复制进 Runtime 或对象工程。源交互提示、provider API、Python 模块、Make target、fixture toolchain 和 source wire format 都在目标边界之外。

## 对象工程继承

attach 的对象工程继承一份 shared external Runtime、显式 `--repo` context、隔离的 Protocol/Contract/evidence/knowledge、不可变 Release identity、fail-closed 生命周期和 human Outcome 边界；不会继承源向导、技术栈 preset、issue-log 存储或源安装文件。

## 验证

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit 8adac3379d8cb3e7a3dc59c70d6fb0b26176b990
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/conformance/reference_inventory_docs_test.py
python3 tests/docs/reference_comparison_metadata_test.py
cargo test --locked --workspace
```

Contract acceptance 保留原始语言。presentation chrome 可以本地化，但治理事实和 human decision 不翻译、不臆造。

另见：[English](WI-621-reference-installer-lifecycle-batch.md) · [日本語](WI-621-reference-installer-lifecycle-batch.ja.md)。
