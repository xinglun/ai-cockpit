---
author: AI Cockpit maintainers
title: "WI-579 — reference template parity batch 46"
description: "残り 16 の reference template path を一つずつ再読し、source implementation をコピーせず Rust の意味的判断を記録する。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-579-reference-template-parity-batch-46
lastVerifiedBy: WI-579-reference-template-parity-batch-46
---

[English](WI-579-reference-template-parity-batch-46.md) · [简体中文](WI-579-reference-template-parity-batch-46.zh-CN.md)

# WI-579 — Reference template parity batch 46

## 目的

固定した local reference checkout の commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` にある残りの `templates/**` path を
すべて一つずつ読み、Rust/repository-native な意味的 counterpart または限定した
`reference-only` 判断を記録する。これは semantic parity であり、source
implementation、Make target、stack command、JSON wire の移行ではない。

## ファイル単位の判断

| 固定 reference path | 分類 | Rust counterpart / 限定判断 |
| --- | --- | --- |
| `templates/agents/AI_COCKPIT_RULES.md` | implemented-different-by-design | `AGENTS.md`、`.ai/README.md`、`.ai/glossary.md`、`crates/cockpit-agent/src/lib.rs`、三言語 agent workflow が repository binding、Contract-first review、pause、evidence、Outcome、正確な cleanup を担う。template Markdown/Make surface はコピーしない。 |
| `templates/glossary.md` | implemented-different-by-design | `.ai/glossary.md`、`docs/reference/commands.md`、`docs/reference/agent-workflow.md` が governance vocabulary を担う。project domain の placeholder は adopter の責任であり、Runtime は推測しない。 |
| `templates/make/Makefile.ai` | implemented-different-by-design | Rust CLI/Repository/Verification service と reviewed gate manifest が lifecycle、quality、evidence の責任を担う。source Make/Python target 名と shell default は adopter/provider integration の選択である。 |
| `templates/stacks/android.mk` | reference-only | Gradle/Android command は source template の convenience default。adopter が toolchain と verification argv を宣言し、shared Runtime は推測・コピーしない。 |
| `templates/stacks/csharp.mk` | reference-only | .NET command は adopter-owned delegated check であり、Runtime は C# preset を配布しない。 |
| `templates/stacks/flutter.mk` | reference-only | Flutter/Dart toolchain default は source/adopter configuration であり、Runtime governance ではない。 |
| `templates/stacks/generic.mk` | reference-only | generic fail-closed placeholder は source onboarding aid。Runtime は不足する check を表示し、command を作らない。 |
| `templates/stacks/go.mk` | reference-only | Go format/test/lint は adopter-owned delegated verification で、portable Runtime Contract ではない。 |
| `templates/stacks/java.mk` | reference-only | Java/JAVA_HOME と Gradle/Maven 選択は stack/provider fact。Runtime は JDK を選択・インストールしない。 |
| `templates/stacks/kotlin.mk` | reference-only | Kotlin/Gradle default は source template convenience であり Core 外に置く。 |
| `templates/stacks/php.mk` | reference-only | PHP format/test/static analysis は adopter が明示する。 |
| `templates/stacks/python.mk` | reference-only | Python/Ruff/Pytest は source template tooling。Rust Runtime は Python environment を install/copy しない。 |
| `templates/stacks/ruby.mk` | reference-only | Ruby/Bundler/Rake は adopter-owned delegated verification。 |
| `templates/stacks/rust.mk` | reference-only | Cargo command は adopter の選択肢だが stack preset としてコピーしない。Runtime は repository declaration と profile-authorized route のみ使用する。 |
| `templates/stacks/swift.mk` | reference-only | Swift/SPM/Xcode assumption は adopter/platform-specific。Runtime は Xcode/CocoaPods coverage を主張しない。 |
| `templates/stacks/typescript.mk` | reference-only | npm format/test/lint default は adopter-owned で、shared Runtime は推測しない。 |

## 境界と adopter の継承

三つの `implemented-different-by-design` 判断は shared external Runtime と
repository-local documentation で portable governance responsibility を保持する。
13 個の stack file は command、toolchain version、platform assumption を安全に
一般化できないため `reference-only` とする。各 attached object/adopter project は
shared Runtime、明示的な `--repo` context、isolated Contract/evidence/knowledge、
dynamic verification boundary、人間向け Outcome handoff を継承するが、source
Python、Make、stack preset、provider policy value、source wire format は継承しない。

## 検証

- `python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --apply-wi579-batch`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `git diff --check`
