---
author: AI Cockpit maintainers
title: "WI-619 — reference adoption、enterprise、lifecycle、trust test parity"
description: "Maintained reference の次の test batch を file 単位で比較し、Rust semantic counterpart を記録します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-619-reference-script-semantic-batch
status: in_progress
authority: canonical
lastVerifiedBy: WI-619-reference-script-semantic-batch
---

# WI-619 — reference adoption、enterprise、lifecycle、trust test parity

## Intent

Maintained reference の次の 21 test path を一件ずつ比較し、evidence-backed な Rust counterpart または source/provider boundary を記録します。この batch は adoption、locked environment、enterprise evidence、dependency projection、lifecycle readiness、governance guard、hosted verification、input trust を対象にします。

## Boundary

Pinned local reference は commit `fde3380f81fea5fd2e288f7a8849f737dc074060` です。これは semantic parity であり、Python、Make、provider config、fixture matrix、source JSON wire を copy しません。Portable governance は shared Rust Runtime と repository-native test が担い、provider と adopter は自分の toolchain と external assurance を保持します。

## Decisions

21 path は [machine inventory](../../tests/conformance/reference_file_inventory.json) に `implemented-different-by-design` として記録しました。file-level table は [reference comparison ledger](../reference/reference-file-comparison.ja.md#wi-619--reference-adoption-enterprise-lifecycle-and-trust-test-parity) にあります。`migrate-gap` はありません。attached object/adopter は shared Runtime、explicit repository context、isolated Contract/evidence/knowledge、dynamic verification、fail-closed lifecycle、人間向け Outcome boundary を継承しますが、source fixture stack や provider command は継承しません。

## Verification

Repository context を明示して実行します。

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit 2536e4db399a7c20479e09693c8eab968c635ac9
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/conformance/reference_inventory_docs_test.py
cargo test --locked --workspace
```

Contract acceptance は原言語を保持し、fixed presentation chrome のみ localize します。unsupported benefit や compliance claim は追加しません。

参照：[English](WI-619-reference-script-semantic-batch.md) · [中文](WI-619-reference-script-semantic-batch.zh-CN.md)。
